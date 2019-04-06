use std::env;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::str::FromStr;

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

            if c == '+' || c == '-' {
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

fn error(token: &Token) {
    eprintln!("予期しないトークンです: {}", token.input);
    std::process::exit(1);
}

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        return;
    }

    //    let num = strtol(&mut p).unwrap();

    let mut tokenizer = Tokenizer::new(args.get(1).unwrap());
    let tokens = tokenizer.tokenize();

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

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

    println!("  ret");
}
