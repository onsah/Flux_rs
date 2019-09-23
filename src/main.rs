#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
extern crate dirs;

mod compiler;
pub mod error;
#[macro_use]
mod macros;
mod parser;
mod scanner;
mod vm;
mod sourcefile;

use compiler::{Chunk, Compiler};
use parser::Parser;
use sourcefile::{SourceFile, MetaData};
use std::fs::File;
use std::path::PathBuf;
use std::io::{Read, Write};
use vm::{Vm, Value};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        match repl() {
            Ok(()) => (),
            Err(err) => println!("{:?}", err),
        }
    } else {
        let path = &args[1];
        let mut file = File::open(path).unwrap();
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();
        let value = eval(buffer.as_str(), path.as_str());
        match value {
            Ok(value) => println!("Exited program. Evaluated: {}", value), 
            Err(err) => println!("{}", err),
        }
    }
}

fn eval(source: &str, path: &str) -> Result<Value, error::FluxError> {
    let mut parser = Parser::new(source)?;
    let ast = parser.parse()?;
    debug!("{:#?}", &ast);
    let dir = {
        let mut dir = PathBuf::from(path);
        dir.pop();
        dir
    };
    let metadata = MetaData { dir };
    debug!("Metadata: {:?}", &metadata);
    let chunk = Compiler::compile(SourceFile { ast, metadata })?;
    debug!("{:#?}", &chunk);
    print_instructions(&chunk);
    let mut vm = Vm::new();
    vm.run(chunk).map_err(|e| e.into())
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
        let ast = parser.parse()?;
        debug!("{:?}", &ast);
        match Compiler::compile(SourceFile {
            ast, 
            metadata: MetaData::default()
        }) {
            Ok(chunk) => {
                debug!("{:?}", &chunk);
                match vm.run(chunk) {
                    Ok(value) => println!("{}", value),
                    Err(error) => println!("{:?}", error),
                }
            },
            Err(err) => {
                println!("{:?}", err);
            }
        };
        line.clear();
    }
}

fn print_instructions(chunk: &Chunk) {
    for (i, instr) in chunk.instructions().iter().enumerate() {
        debug!("{}: {:?}", i, instr);
    }
}
