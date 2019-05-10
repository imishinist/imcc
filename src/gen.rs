use crate::parse::Node;
use crate::token::TokenType;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Generator {
    vars: HashMap<String, usize>,
}

impl Generator {
    pub fn new() -> Self {
        Generator {
            vars: HashMap::new(),
        }
    }

    fn max_offset(&self) -> usize {
        match self.vars.values().max() {
            Some(c) => *c,
            None => 0,
        }
    }

    fn get_offset(&mut self, name: String) -> usize {
        match self.vars.get(&name) {
            Some(offset) => *offset,
            None => {
                let offset = self.max_offset() + 8;
                self.vars.insert(name, offset);
                offset
            }
        }
    }

    fn gen_lval(&mut self, node: &Option<Rc<Node>>) {
        if let Some(node) = node {
            let node: &Node = node.borrow();

            if node.ty != TokenType::Ident {
                eprintln!("left value must be variable");
                std::process::exit(1);
            }

            let offset = self.get_offset(node.name.clone());
            println!("  mov rax, rbp");
            println!("  sub rax, {}", offset);
            println!("  push rax");
        }
    }

    pub fn gen(&mut self, node: &Option<Rc<Node>>) {
        if let Some(node) = node {
            let node: &Node = node.borrow();

            if node.ty == TokenType::Return {
                self.gen(&node.lhs);
                println!("  pop rax");
                println!("  mov rsp, rbp");
                println!("  pop rbp");
                println!("  ret");
                return;
            }

            if node.ty == TokenType::NUM {
                println!("  push {}", node.val);
                return;
            }

            if node.ty == TokenType::Ident {
                let node = (*node).clone();
                self.gen_lval(&Some(Rc::new(node)));
                println!("  pop rax");
                println!("  mov rax, [rax]");
                println!("  push rax");
                return;
            }

            if node.ty == TokenType::Symbol('=') {
                self.gen_lval(&node.lhs);
                self.gen(&node.rhs);

                println!("  pop rdi");
                println!("  pop rax");
                println!("  mov [rax], rdi");
                println!("  push rdi");
                return;
            }

            self.gen(&node.lhs);
            self.gen(&node.rhs);

            println!("  pop rdi");
            println!("  pop rax");

            match node.ty {
                TokenType::Symbol('+') => {
                    println!("  add rax, rdi");
                }
                TokenType::Symbol('-') => {
                    println!("  sub rax, rdi");
                }
                TokenType::Symbol('*') => {
                    println!("  mul rdi");
                }
                TokenType::Symbol('/') => {
                    println!("  mov rdx, 0");
                    println!("  div rdi");
                }
                _ => (),
            }
            println!("  push rax");
        }
    }
}
