use std::env;
use std::io::{Cursor, SeekFrom, Seek};
use std::str::FromStr;


fn strtol(p: &mut Cursor<&String>) -> Option<i32> {
    let s = p.get_ref().as_str();
    let pos = p.position() as usize;

    if let Some(endp) = s.find(|c: char| !c.is_ascii_digit()) {
        p.set_position(endp as u64);
        let num_str = &s[pos..endp];

        return FromStr::from_str(num_str).ok();
    }
    p.seek(SeekFrom::End(0)).ok();
    FromStr::from_str(s).ok()
}

fn main() {

    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        return;
    }

    let mut p: Cursor<&String> = Cursor::new(args.get(1).unwrap());
    let num = strtol(&mut p).unwrap();

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");
    println!("  mov rax, {}", num);
    println!("  ret");
}
