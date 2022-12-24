use std::{fs, collections::HashMap, path::{PathBuf, Path}, io::{Read, self}};
use thiserror::Error;

/// The structure that holds the whole source code of the compiled program.
#[derive(Debug)]
pub struct SourceMap {
    root: PathBuf,
    files: HashMap<PathBuf, SourceFile>,
}

impl SourceMap {
    /// Create new [SourceMap] with path to root folder.
    /// 
    /// Compiler will seek for `bin.sun` in provided folder.
    /// 
    /// # Errors
    /// 
    /// Error is only returned if `root/bin.sun` is not found or couldn't be opened.
    pub fn new(root: PathBuf) -> Result<Self, SourceError> {
        if !root.is_dir() {
            return Err(SourceError::NotADirectory(root));
        }
        Ok(Self {
            root,
            files: HashMap::new(),
        })
    }

    pub fn add_source(&mut self, path: PathBuf) -> Result<(), SourceError> {
        if !path.is_relative() {
            return Err(SourceError::NotRelative(path));
        }
        let mut source_path = self.root.clone();
        source_path.extend(path.iter());
        self.files.insert(path, SourceFile::new(source_path)?);
        Ok(())
    }
}

/// A single file of the source code.
/// 
/// File's content is buffered.
#[derive(Debug)]
enum SourceFile {
    Loaded(String),
    Opened(fs::File),
}

impl SourceFile {
    /// Open new file without reading it.
    pub fn new(path: impl AsRef<Path>) -> io::Result<SourceFile> {
        fs::File::open(path)
            .map(SourceFile::Opened)
    }

    /// Read file to string slice.
    pub fn read(&mut self) -> io::Result<&str> {
        match self {
            SourceFile::Opened(file) => {
                let mut buf = String::new();
                file.read_to_string(&mut buf)?;
                *self = SourceFile::Loaded(buf);
                self.read()
            }
            SourceFile::Loaded(string) => {
                Ok(string.as_str())
            },
        }
    }

    pub fn try_into_string(self) -> io::Result<String> {
        Ok(match self {
            SourceFile::Loaded(src) => src,
            SourceFile::Opened(mut file) => {
                let mut buf = String::new();
                file.read_to_string(&mut buf)?;
                buf
            },
        })
    }
}

impl TryFrom<SourceFile> for String {
    type Error = io::Error;

    fn try_from(value: SourceFile) -> Result<Self, Self::Error> {
        value.try_into_string()
    }
}

/// Error
#[derive(Debug, Error)]
pub enum SourceError {
    #[error("provided path `{0}` is expected to be a directory")]
    NotADirectory(PathBuf),
    #[error("provided path `{0}` is expected to be relative")]
    NotRelative(PathBuf),
    #[error("provided path `{0}` is not found")]
    NotFound(PathBuf),
    #[error("{0}")]
    IoError(#[from] io::Error)
}
