use std::env;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::str::FromStr;
use std::rc::Rc;
use std::error;
use std::fmt;
use core::borrow::Borrow;

#[derive(Debug, PartialEq)]
enum TokenType {
    NUM,
    Symbol(char),
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

            if c == '(' || c == ')' {

            }

            if c == '+' || c == '-' || c == '(' || c == ')'
             || c == '*' || c == '/' {
                let token = Token::with_symbol(c);
                tokens.push(token);
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

            eprintln!("トークナイズできません: {}", "");
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
            let node = self.add();
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

#[derive(Debug)]
struct Node {
    ty: TokenType,
    lhs: Option<Rc<Node>>,
    rhs: Option<Rc<Node>>,
    val: i32,
}

impl Node {
    fn new_node(ty: TokenType, lhs: Rc<Node>, rhs: Rc<Node>) -> Node {
        Node {
            ty: ty,
            lhs: Some(lhs),
            rhs: Some(rhs),
            val: 0,
        }
    }

    fn new_node_num(val: i32) -> Node {
        Node {
            ty: TokenType::NUM,
            lhs: None,
            rhs: None,
            val: val,
        }
    }
}

fn gen(node: &Node) {
    if node.ty == TokenType::NUM {
        println!("  push {}", node.val);
        return
    }

    match &node.lhs {
        None => return,
        Some(lhs) => gen(lhs.borrow()),
    }

    match &node.rhs {
        None => return,
        Some(rhs) => gen(rhs.borrow()),
    }

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
    let ret = parser.add();

    if let Err(e) = ret {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    let node = ret.unwrap();

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    gen(&node);
    /*
    if tokens[0].ty != TokenType::NUM {
        eprintln!("error");
        std::process::exit(1);
    }

    println!("  mov rax, {}", tokens[0].val);

    let mut i = 1;
    loop {
        let token = &tokens[i];

        match token.ty {
            TokenType::Symbol('+') => {
                i += 1;
                if tokens[i].ty != TokenType::NUM {
                    error(&tokens[i]);
                }
                println!("  add rax, {}", tokens[i].val);
                i += 1;
                continue;
            }
            TokenType::Symbol('-') => {
                i += 1;
                if tokens[i].ty != TokenType::NUM {
                    error(&tokens[i]);
                }
                println!("  sub rax, {}", tokens[i].val);
                i += 1;
                continue;
            }
            TokenType::EOF => break,
            _ => (),
        }
        error(&tokens[i]);
    }
    */
    println!("  pop rax");
    println!("  ret");
}
