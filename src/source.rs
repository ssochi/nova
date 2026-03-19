use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile {
    pub path: PathBuf,
    pub contents: String,
}

impl SourceFile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let path = path.as_ref().to_path_buf();
        let contents = fs::read_to_string(&path)?;
        Ok(Self { path, contents })
    }
}
