use crate::compiler::{Chunk, Compiler};
use crate::error::FluxResult;
use crate::parser::Parser;
use crate::sourcefile::{MetaData, SourceFile};
use crate::vm::{Value, Vm};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn run_file(path: &str) -> FluxResult<Value> {
    let mut file = File::open(path).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();

    eval(buffer.as_str(), path)
}

pub fn eval(source: &str, path: &str) -> FluxResult<Value> {
    let mut parser = Parser::new(source)?;
    let ast = parser.parse()?;
    dbg!(&ast);
    let dir = {
        let mut dir = PathBuf::from(path);
        dir.pop();
        dir
    };
    let metadata = MetaData { dir };
    dbg!(&metadata);
    let compiled = Compiler::compile(SourceFile { ast, metadata })?;
    dbg!(&compiled.chunk);
    print_instructions(&compiled.chunk);
    let mut vm = Vm::new();
    vm.run(compiled).map_err(|e| e.into())
}

fn print_instructions(chunk: &Chunk) {
    for (i, instr) in chunk.instructions().iter().enumerate() {
        debug!("{}: {:?}", i, instr);
    }
}
