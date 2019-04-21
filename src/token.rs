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
    pub input: String,
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

    fn with_return() -> Self {
        Token {
            ty: TokenType::Return,
            val: 0,
            input: "".to_string(),
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
                continue;
            }

            if c == 'r' {
                let pp: [u8; 5] = [0,0,0,0,0];
                let res = self.cursor.read_exact(&mut p);
                if let Err(_e) = res {
                    break;
                }
                let s = String::from_utf8_lossy(&pp);
                if s != "eturn".to_string() {
                    break;
                }
                tokens.push(Token::with_return());
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
