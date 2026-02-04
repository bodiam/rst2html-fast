use std::io::{self, Read};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let input = if args.len() > 1 {
        std::fs::read_to_string(&args[1]).expect("Failed to read file")
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).expect("Failed to read stdin");
        buf
    };

    let html = rst2html::convert(&input);
    print!("{}", html);
}
