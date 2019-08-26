use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct MetaData {
    pub dir: PathBuf,
}

impl MetaData {
    pub fn current_dir(&self) -> &str {
        self.dir.to_str().unwrap()
    }
}

impl Default for MetaData {
    fn default() -> Self {
        MetaData {
            dir: PathBuf::default(),
        }
    }
}