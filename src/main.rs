use std::env;
use std::rc::Rc;

mod gen;
mod parse;
mod token;

use gen::gen;
use parse::Parser;
use token::Tokenizer;

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
