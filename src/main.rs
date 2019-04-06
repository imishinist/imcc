use std::env;

fn main() {

    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        return;
    }

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");
    println!("  mov rax, {}", args.get(1).unwrap());
    println!("  ret");
}
