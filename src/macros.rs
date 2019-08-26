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
            let ast = parser.parse().unwrap();
            let chunk = Compiler::compile(SourceFile {
                ast, 
                metadata: MetaData::default(),
            }).unwrap();
            let mut vm = Vm::new();

            assert_eq!(vm.run(chunk), $expected);
        }
    };
}
