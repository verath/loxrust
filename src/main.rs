use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process;

use loxrust::Scanner;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: loxrust [script]");
        process::exit(1);
    } else if args.len() == 2 {
        run_file(&args[1]).unwrap();
    } else {
        run_prompt().unwrap();
    }
}

fn run_file(path: &str) -> std::io::Result<()> {
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    run(&buf);
    Ok(())
}

fn run_prompt() -> std::io::Result<()> {
    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut buf = String::new();
        stdin.read_line(&mut buf)?;
        run(buf.trim_end())
    }
}

fn run(source: &str) {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    for token in tokens {
        println!("{:?}", token);
    }
}
