#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;

mod compiler;
pub mod error;
mod parser;
mod scanner;
mod vm;

use compiler::compile;
use parser::Parser;
use vm::Vm;
use std::fs::File;
use std::io::{Read, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        repl()
    } else {
        let path = &args[1];
        let mut file = File::open(path).unwrap();
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();
        let mut parser = Parser::new(&buffer).unwrap();
        let ast = parser.parse().unwrap();
        println!("AST:\n {:?}", ast);
        let chunk = compile(ast).unwrap();
        println!("Chunk: {:?}\n", chunk);
    }
}

fn repl() {
    let stdin = std::io::stdin();
    let mut line = String::new();
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        stdin.read_line(&mut line).unwrap();
        let mut parser = Parser::new(&line).unwrap();
        let ast = parser.parse().unwrap();
        println!("AST:\n {:?}", ast);
        let chunk = compile(ast).unwrap();
        println!("Chunk: {:?}\n", chunk);
        let mut vm = Vm::new();
        println!("{:?}", vm.run(chunk).unwrap());
        line.clear();
    }
}
