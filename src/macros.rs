macro_rules! debug {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            println!($($arg)*)
        }
    };
}

#[allow(unused_macros)]
macro_rules! unit_test {
    ($name:ident, $source:expr, $expected:expr) => {
        #[test]
        fn $name() {
            use crate::sourcefile::{SourceFile, MetaData};
            let source = $source;

            let mut parser = Parser::new(source).unwrap();
            let ast = parser.parse();
            match ast {
                Ok(ast) => {
                    debug!("{:#?}", &ast);
                    let chunk = Compiler::compile(SourceFile {
                        ast, 
                        metadata: MetaData::default(),
                    });
                    match chunk {
                        Ok(chunk) => {
                            let mut vm = Vm::new();
                            let result: FluxResult<Value> = vm.run(chunk).map_err(|e| e.into());
                            assert_eq!(result, $expected);
                        },
                        Err(err) => {
                            let err: FluxResult<Value> = Err(err.into());
                            assert_eq!(err, $expected)
                        },
                    }
                },
                Err(err) => {
                    let err: FluxResult<Value> = Err(err.into());
                    assert_eq!(err, $expected)
                },
            }
            
            // debug!("{:#?}", &chunk);
        }
    };
}
