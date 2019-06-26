#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;

mod compiler;
pub mod error;
mod parser;
mod scanner;
mod vm;

use compiler::{compile, Chunk};
use parser::Parser;
use std::fs::File;
use std::io::{Read, Write};
use vm::Vm;

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
        //println!("AST:\n {:#?}", ast);
        let chunk = compile(ast).unwrap();
        println!("Chunk: {:#?}\n", chunk);
        print_instructions(&chunk);
        let mut vm = Vm::new();
        vm.run(chunk).unwrap();
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
        println!("AST:\n {:#?}", ast);
        /* let chunk = compile(ast).unwrap();
        println!("Chunk: {:?}\n", chunk);
        let mut vm = Vm::new();
        println!("{:?}", vm.run(chunk).unwrap());
        */
        line.clear();
    }
}

fn print_instructions(chunk: &Chunk) {
    for (i, instr) in chunk.instructions().iter().enumerate() {
        println!("{}: {:?}", i, instr)
    }
}