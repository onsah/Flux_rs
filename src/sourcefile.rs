mod metadata;

pub use metadata::MetaData;
use crate::parser::Ast;

#[derive(Clone, Debug, PartialEq)]
pub struct SourceFile {
    pub ast: Ast,
    pub metadata: MetaData,
}