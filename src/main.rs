use std::env;
use std::rc::Rc;

mod gen;
mod parse;
mod token;

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        return;
    }

    let mut tokenizer = token::Tokenizer::new(args.get(1).unwrap());
    let tokens = tokenizer.tokenize();

    let mut parser = parse::Parser::new(tokens);
    let ret = parser.program();

    let mut generator = gen::Generator::new();

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
        generator.gen(&node);
        println!("  pop rax");
    }

    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");
}
