use std::env;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::str::FromStr;
use std::rc::Rc;
use std::error;
use std::fmt;
use core::borrow::Borrow;

#[derive(Debug, PartialEq, Clone)]
enum TokenType {
    NUM,
    Symbol(char),
    Ident,
    EOF,
}

#[derive(Debug)]
struct Token {
    ty: TokenType,
    val: i32,
    input: String,
}

impl Token {
    fn with_val(val: i32) -> Self {
        Token {
            ty: TokenType::NUM,
            val: val,
            input: val.to_string(),
        }
    }

    fn with_symbol(sym: char) -> Self {
        Token {
            ty: TokenType::Symbol(sym),
            val: 0,
            input: sym.to_string(),
        }
    }

    fn with_ident(message: String) -> Self {
        Token {
            ty: TokenType::Ident,
            val: 0,
            input: message,
        }
    }

    fn eof(message: String) -> Self {
        Token {
            ty: TokenType::EOF,
            val: 0,
            input: message,
        }
    }
}

#[derive(Debug)]
struct Tokenizer<'a> {
    cursor: Cursor<&'a String>,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a String) -> Self {
        Tokenizer {
            cursor: Cursor::new(input),
        }
    }

    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::with_capacity(100);

        loop {
            let mut p = [0];
            // 内部でpositionがincrementされる
            let res = self.cursor.read_exact(&mut p);
            if let Err(_e) = res {
                break;
            }
            let c = p[0] as char;

            if c.is_whitespace() {
                continue;
            }

            if c == '+' || c == '-' || c == '(' || c == ')'
             || c == '*' || c == '/' || c == ';' || c == '=' {
                let token = Token::with_symbol(c);
                tokens.push(token);
                continue;
            }

            if c.is_alphabetic() {
                tokens.push(Token::with_ident(c.to_string()));
                continue;
            }

            if c.is_ascii_digit() {
                // positionがincrementされているので、1つ戻す
                self.dec_cursor();
                let num = strtol(&mut self.cursor).unwrap();
                let token = Token::with_val(num);
                tokens.push(token);
                continue;
            }

            eprintln!("トークナイズできません: {}", c.to_string());
            std::process::exit(1);
        }

        tokens.push(Token::eof("EOF".to_string()));
        tokens
    }

    fn inc_cursor(&mut self) {
        let pos = self.cursor.position();
        self.cursor.set_position(pos + 1);
    }

    fn dec_cursor(&mut self) {
        let pos = self.cursor.position();
        self.cursor.set_position(pos - 1);
    }
}

fn strtol(p: &mut Cursor<&String>) -> Option<i32> {
    let s = p.get_ref().as_str();
    let pos = p.position() as usize;
    let s = &s[pos..];

    if let Some(endp) = s.find(|c: char| !c.is_ascii_digit()) {
        p.set_position((pos + endp) as u64);
        let num_str = &s[0..endp];

        return FromStr::from_str(num_str).ok();
    }
    p.seek(SeekFrom::End(0)).ok();
    FromStr::from_str(s).ok()
}

#[derive(Debug)]
enum ParserError {
    UnexpectedToken(String)
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserError::UnexpectedToken(err) => write!(f, "Unexpected token: {}", err),
        }
    }
}

impl error::Error for ParserError {
    fn description(&self) -> &str {
        match self {
            ParserError::UnexpectedToken(err) => err.as_str(),
        }
    }
}

#[derive(Debug)]
struct Parser {
    pos: usize,
    tokens: Vec<Token>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            pos: 0,
            tokens: tokens,
        }
    }

    fn program(&mut self) -> Result<Vec<Node>, ParserError> {
        let mut code = Vec::new();
        loop {
            if self.tokens[self.pos].ty == TokenType::EOF { break }
            code.push(self.stmt()?);
        }
        Ok(code)
    }

    fn stmt(&mut self) -> Result<Node, ParserError> {
        let node = self.assign()?;
        if !self.consume(TokenType::Symbol(';')) {
            let message = self.tokens[self.pos].input.clone();
            return Err(ParserError::UnexpectedToken(message));
        }
        Ok(node)
    }

    fn assign(&mut self) -> Result<Node, ParserError> {
        let mut node = self.add()?;
        if self.consume(TokenType::Symbol('=')) {
            node = Node::new_node(TokenType::Symbol('='), Rc::new(node), Rc::new(self.assign()?));
        }
        Ok(node)
    }

    fn add(&mut self) -> Result<Node, ParserError> {
        let mut node = self.mul()?;

        loop {
            if self.consume(TokenType::Symbol('+')) {
                node = Node::new_node(TokenType::Symbol('+'), Rc::new(node), Rc::new(self.mul()?));
            } else if self.consume(TokenType::Symbol('-')) {
                node = Node::new_node(TokenType::Symbol('-'), Rc::new(node), Rc::new(self.mul()?));
            } else {
                return Ok(node);
            }
        }
    }

    fn mul(&mut self) -> Result<Node, ParserError> {
        let mut node = self.term()?;

        loop {
            if self.consume(TokenType::Symbol('*')) {
                node = Node::new_node(TokenType::Symbol('*'), Rc::new(node), Rc::new(self.term()?));
            } else if self.consume(TokenType::Symbol('/')) {
                node = Node::new_node(TokenType::Symbol('/'), Rc::new(node), Rc::new(self.term()?));
            } else {
                return Ok(node);
            }
        }
    }

    fn term(&mut self) -> Result<Node, ParserError> {
        if self.consume(TokenType::Symbol('(')) {
            let node = self.assign();
            if !self.consume(TokenType::Symbol(')')) {
                let message = self.tokens[self.pos].input.clone();
                return Err(ParserError::UnexpectedToken(message));
            }
            return node;
        }
        if self.tokens[self.pos].ty == TokenType::NUM {
            let pos = self.pos;
            self.pos += 1;
            return Ok(Node::new_node_num(self.tokens[pos].val));
        }
        if self.tokens[self.pos].ty == TokenType::Ident {
            let pos = self.pos;
            self.pos += 1;
            let name = self.tokens[pos].input.as_bytes()[0];
            return Ok(Node::new_node_ident(name as char));
        }

        let message = self.tokens[self.pos].input.clone();
        Err(ParserError::UnexpectedToken(message))
    }

    fn consume(&mut self, ty: TokenType) -> bool {
        if self.tokens[self.pos].ty != ty {
            return false
        }
        self.pos+=1;
        true
    }
}

#[derive(Debug, Clone)]
struct Node {
    ty: TokenType,
    lhs: Option<Rc<Node>>,
    rhs: Option<Rc<Node>>,
    val: i32,
    name: char,
}

impl Node {
    fn new_node(ty: TokenType, lhs: Rc<Node>, rhs: Rc<Node>) -> Node {
        Node {
            ty: ty,
            lhs: Some(lhs),
            rhs: Some(rhs),
            val: 0,
            name: 0 as char,
        }
    }

    fn new_node_ident(name: char) -> Node {
        Node {
            ty: TokenType::Ident,
            lhs: None,
            rhs: None,
            val: 0,
            name: name,
        }
    }

    fn new_node_num(val: i32) -> Node {
        Node {
            ty: TokenType::NUM,
            lhs: None,
            rhs: None,
            val: val,
            name: 0 as char,
        }
    }
}

fn gen_lval(node: &Option<Rc<Node>>) {
    if let Some(node) = node {
        let node: &Node = node.borrow();

        if node.ty != TokenType::Ident {
            eprintln!("left value must be variable");
            std::process::exit(1);
        }

        let offset = ('z' as usize - node.name as usize + 1) * 8;
        println!("  mov rax, rbp");
        println!("  sub rax, {}", offset);
        println!("  push rax");
    }
}

fn gen(node: &Option<Rc<Node>>) {
    if let Some(node) = node {
        let node: &Node = node.borrow();

        if node.ty == TokenType::NUM {
            println!("  push {}", node.val);
            return
        }

        if node.ty == TokenType::Ident {
            let node = (*node).clone();
            gen_lval(&Some(Rc::new(node)));
            println!("  pop rax");
            println!("  mov rax, [rax]");
            println!("  push rax");
            return
        }

        if node.ty == TokenType::Symbol('=') {
            gen_lval(&node.lhs);
            gen(&node.rhs);

            println!("  pop rdi");
            println!("  pop rax");
            println!("  mov [rax], rdi");
            println!("  push rdi");
            return
        }

        gen(&node.lhs);
        gen(&node.rhs);

        println!("  pop rdi");
        println!("  pop rax");

        match node.ty {
            TokenType::Symbol('+') => { println!("  add rax, rdi"); },
            TokenType::Symbol('-') => { println!("  sub rax, rdi"); },
            TokenType::Symbol('*') => { println!("  mul rdi"); },
            TokenType::Symbol('/') => {
                println!("  mov rdx, 0");
                println!("  div rdi");
            },
            _ => ()
        }
        println!("  push rax");
    }
}

fn error(token: &Token) {
    eprintln!("予期しないトークンです: {}", token.input);
    std::process::exit(1);
}

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        return;
    }

    let mut tokenizer = Tokenizer::new(args.get(1).unwrap());
    let tokens = tokenizer.tokenize();


    let mut parser = Parser::new(tokens);
    let ret = parser.program();

    if let Err(e) = ret {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    let code = ret.unwrap();

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    println!("  push rbp");
    println!("  mov rbp, rsp");
    println!("  sub rsp, 208");

    for node in code {
        let node = Some(Rc::new(node));
        gen(&node);
        println!("  pop rax");
    }

    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");
}
