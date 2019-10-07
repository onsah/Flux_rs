mod metadata;

use crate::parser::Ast;
pub use metadata::MetaData;

#[derive(Clone, Debug, PartialEq)]
pub struct SourceFile {
    pub ast: Ast,
    pub metadata: MetaData,
}
