use crate::parse::Node;
use crate::token::TokenType;
use std::borrow::Borrow;
use std::rc::Rc;

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

pub fn gen(node: &Option<Rc<Node>>) {
    if let Some(node) = node {
        let node: &Node = node.borrow();

        if node.ty == TokenType::NUM {
            println!("  push {}", node.val);
            return;
        }

        if node.ty == TokenType::Ident {
            let node = (*node).clone();
            gen_lval(&Some(Rc::new(node)));
            println!("  pop rax");
            println!("  mov rax, [rax]");
            println!("  push rax");
            return;
        }

        if node.ty == TokenType::Symbol('=') {
            gen_lval(&node.lhs);
            gen(&node.rhs);

            println!("  pop rdi");
            println!("  pop rax");
            println!("  mov [rax], rdi");
            println!("  push rdi");
            return;
        }

        gen(&node.lhs);
        gen(&node.rhs);

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
