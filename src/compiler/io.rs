use super::{CompileError, CompileResult};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn absolute_path<S>(dir: &str, file_path: &[S]) -> PathBuf
where
    S: AsRef<str>,
{
    let mut path = PathBuf::from(dir);
    for p in file_path {
        path.push(p.as_ref());
    }
    path.set_extension("flux");
    path
}

pub fn read_file(path: PathBuf) -> CompileResult<String> {
    match File::open(path) {
        Ok(mut file) => {
            let mut string = String::new();
            file.read_to_string(&mut string).expect("failed to write");
            Ok(string)
        }
        Err(err) => Err(CompileError::IoError(err.kind())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_path_works() {
        let dir = "src/";
        let file_path = ["lib", "math", "sqrt"];
        let path = absolute_path(dir, &file_path);

        assert!(path.starts_with("src/lib/math"));
        assert_eq!(path, PathBuf::from("src/lib/math/sqrt.flux"));
    }

    #[test]
    fn read_file_works() {
        // https://stackoverflow.com/questions/30003921/how-can-i-locate-resources-for-testing-with-cargo
        // Gives base directory of the project
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = ["snippets", "new"];
        let path = absolute_path(dir.to_str().expect("Expected unicode path"), &path);
        println!("Path: {:#?}", path);
        assert!(path.exists());
    }
}
