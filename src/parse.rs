use crate::token::*;
use std::error;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken(String),
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
pub struct Parser {
    pos: usize,
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            pos: 0,
            tokens: tokens,
        }
    }

    pub fn program(&mut self) -> Result<Vec<Node>, ParserError> {
        let mut code = Vec::new();
        loop {
            if self.tokens[self.pos].ty == TokenType::EOF {
                break;
            }
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
            node = Node::new_node(
                TokenType::Symbol('='),
                Rc::new(node),
                Rc::new(self.assign()?),
            );
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
            return false;
        }
        self.pos += 1;
        true
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub ty: TokenType,
    pub lhs: Option<Rc<Node>>,
    pub rhs: Option<Rc<Node>>,
    pub val: i32,
    pub name: char,
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
