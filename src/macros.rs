#[allow(unused_macros)]
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
            use crate::util::eval;

            assert_eq!(eval($source, ""), $expected);
        }
    };
}
