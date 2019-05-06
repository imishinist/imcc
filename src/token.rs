use std::cmp;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    NUM,
    Symbol(char),
    Ident,
    Return,
    EOF,
}

#[derive(Debug)]
pub struct Token {
    pub ty: TokenType,
    pub val: i32,
    pub name: String,
    pub input: String,
}

impl Token {
    fn with_val(val: i32) -> Self {
        Token {
            ty: TokenType::NUM,
            val: val,
            name: "".to_string(),
            input: val.to_string(),
        }
    }

    fn with_symbol(sym: char) -> Self {
        Token {
            ty: TokenType::Symbol(sym),
            val: 0,
            name: "".to_string(),
            input: sym.to_string(),
        }
    }

    fn with_return() -> Self {
        Token {
            ty: TokenType::Return,
            val: 0,
            name: "".to_string(),
            input: "".to_string(),
        }
    }

    fn with_ident(name: String, message: String) -> Self {
        Token {
            ty: TokenType::Ident,
            val: 0,
            name,
            input: message,
        }
    }

    fn eof(message: String) -> Self {
        Token {
            ty: TokenType::EOF,
            val: 0,
            name: "".to_string(),
            input: message,
        }
    }
}

#[derive(Debug)]
pub struct Tokenizer<'a> {
    cursor: Cursor<&'a String>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a String) -> Self {
        Tokenizer {
            cursor: Cursor::new(input),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::with_capacity(100);

        loop {
            let c = match self.take_char() {
                Some(c) => c,
                None => break,
            };

            if c.is_whitespace() {
                self.inc_cursor(1);
                continue;
            }

            if c == '+'
                || c == '-'
                || c == '('
                || c == ')'
                || c == '*'
                || c == '/'
                || c == ';'
                || c == '='
            {
                let token = Token::with_symbol(c);
                tokens.push(token);
                self.inc_cursor(1);
                continue;
            }

            if self.strncmp("return", 6) {
                self.inc_cursor(6);
                if !self.take_char().unwrap().is_alphanumeric() {
                    tokens.push(Token::with_return());
                    continue;
                } else {
                    self.dec_cursor(6);
                }
            }

            if c.is_alphabetic() {
                self.inc_cursor(1);
                tokens.push(Token::with_ident(c.to_string(), c.to_string()));
                continue;
            }

            if c.is_ascii_digit() {
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

    fn take_char(&mut self) -> Option<char> {
        let mut p = [0];
        // 内部でpositionがincrementされる
        let res = self.cursor.read_exact(&mut p);
        if let Err(_e) = res {
            return None;
        }
        self.dec_cursor(1);
        Some(p[0] as char)
    }

    fn strncmp(&mut self, str: &str, n: usize) -> bool {
        let nn = cmp::min(
            n,
            self.cursor.get_ref().len() - self.cursor.position() as usize,
        );
        if n != nn {
            return false;
        }

        let mut buf = vec![0; n];
        let mut buf = buf.as_mut();
        let res = self.cursor.read_exact(&mut buf);
        self.dec_cursor(n as u64);

        if let Err(_e) = res {
            return false;
        }

        let s = String::from_utf8_lossy(&buf);
        if s != str.to_string() {
            return false;
        }

        true
    }

    fn inc_cursor(&mut self, n: u64) {
        let pos = self.cursor.position();
        self.cursor.set_position(pos + n);
    }

    fn dec_cursor(&mut self, n: u64) {
        let pos = self.cursor.position();
        self.cursor.set_position(pos - n);
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
