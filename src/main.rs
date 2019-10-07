#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
extern crate dirs;

#[macro_use]
mod macros;
mod compiler;
pub mod error;
mod parser;
mod scanner;
mod sourcefile;
mod util;
mod vm;

use compiler::Compiler;
use parser::Parser;
use sourcefile::{MetaData, SourceFile};
use std::io::Write;
use util::run_file;
use vm::Vm;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        match repl() {
            Ok(()) => (),
            Err(err) => println!("{:?}", err),
        }
    } else {
        let path = &args[1];
        /* let mut file = File::open(path).unwrap();
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap(); */

        let value = run_file(path);
        match value {
            Ok(value) => println!("Exited program. Evaluated: {}", value),
            Err(err) => println!("Error: {}", err),
        }
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
        let ast = parser.parse()?;
        dbg!(&ast);
        match Compiler::compile(SourceFile {
            ast,
            metadata: MetaData::default(),
        }) {
            Ok(compiled) => {
                dbg!(&compiled.chunk);
                match vm.run(compiled) {
                    Ok(value) => println!("{}", value),
                    Err(error) => println!("{:?}", error),
                }
            }
            Err(err) => {
                println!("{:?}", err);
            }
        };
        line.clear();
    }
}
