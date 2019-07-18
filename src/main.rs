#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;

mod compiler;
pub mod error;
#[macro_use]
mod macros;
mod parser;
mod scanner;
mod vm;

use compiler::{Chunk, Compiler};
use parser::Parser;
use std::fs::File;
use std::io::{Read, Write};
use vm::Vm;

fn main() -> Result<(), error::FluxError> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        repl()
    } else {
        let path = &args[1];
        let mut file = File::open(path).unwrap();
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();
        let mut parser = Parser::new(&buffer)?;
        let ast = parser.parse().unwrap();
        let chunk = Compiler::compile(ast)?;
        debug!("{:?}", &chunk);
        print_instructions(&chunk);
        let mut vm = Vm::new();
        vm.run(chunk)?;
        Ok(())
    }
}

fn repl() -> Result<(), error::FluxError> {
    let stdin = std::io::stdin();
    let mut line = String::new();
    let mut vm = Vm::new();
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        stdin.read_line(&mut line).unwrap();
        let mut parser = Parser::new(&line)?;
        let ast = parser.parse().unwrap();
        debug!("{:?}", &ast);
        let chunk = Compiler::compile(ast)?;
        debug!("{:?}", &chunk);
        match vm.run(chunk) {
            Ok(_) => (),
            Err(error) => println!("Error: {:?}", error),
        }
        line.clear();
    }
}

fn print_instructions(chunk: &Chunk) {
    for (i, instr) in chunk.instructions().iter().enumerate() {
        debug!("{}: {:?}", i, instr);
    }
}
