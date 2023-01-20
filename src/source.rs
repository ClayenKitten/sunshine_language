//! Source code and file hierarchy management.

use std::{
    collections::{hash_map::Entry, HashMap},
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};
use thiserror::Error;

use crate::{ast::Identifier, item_table::path::ItemPath};

/// The structure that holds the whole source code of the compiled program.
#[derive(Debug)]
pub struct SourceMap {
    root: PathBuf,
    files: HashMap<ItemPath, SourceFile>,
}

impl SourceMap {
    /// Creates new [SourceMap] with path to the main file.
    ///
    /// # Errors
    ///
    /// Error is only returned if `root` is not found or couldn't be opened.
    pub fn new(mut main: PathBuf, krate: Identifier) -> Result<Self, SourceError> {
        Ok(Self {
            files: {
                let mut files = HashMap::new();
                files.insert(ItemPath::new(krate), SourceFile::new(&main)?);
                files
            },
            root: {
                main.pop();
                main
            },
        })
    }

    /// Inserts new source file to the map and returns a reference to the file.
    pub fn insert(&mut self, path: ItemPath) -> Result<&mut SourceFile, SourceError> {
        let mut source_path = self.root.clone();
        source_path.extend(path.clone().into_path_buf().iter());
        Ok(match self.files.entry(path) {
            Entry::Vacant(entry) => entry.insert(SourceFile::new(source_path)?),
            Entry::Occupied(entry) => entry.into_mut(),
        })
    }
}

/// A single file of the source code.
///
/// File's content is buffered.
#[derive(Debug)]
pub enum SourceFile {
    Loaded(String),
    Opened(fs::File),
}

impl SourceFile {
    /// Open new file without reading it.
    pub fn new(path: impl AsRef<Path>) -> Result<SourceFile, SourceError> {
        let path = path.as_ref();
        match fs::metadata(path) {
            Ok(meta) if !meta.is_file() => {
                Err(SourceError::NotAFile(path.to_owned()))
            }
            Ok(_) => fs::OpenOptions::new()
                .read(true)
                .open(path)
                .map(SourceFile::Opened)
                .map_err(|err| SourceError::IoErrorWithSource(path.to_owned(), err)),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                Err(SourceError::NotFound(path.to_owned()))
            }
            Err(err) if err.kind() == io::ErrorKind::PermissionDenied => {
                Err(SourceError::PermissionDenied(path.to_owned()))
            }
            Err(err) => {
                Err(SourceError::IoErrorWithSource(path.to_owned(), err))
            }
        }
    }

    /// Read file to string slice.
    pub fn read(&mut self) -> Result<&str, SourceError> {
        match self {
            SourceFile::Opened(file) => {
                let mut buf = String::new();
                file.read_to_string(&mut buf)?;
                *self = SourceFile::Loaded(buf);
                self.read()
            }
            SourceFile::Loaded(string) => Ok(string.as_str()),
        }
    }
}

/// Error caused by filesystem interaction.
#[derive(Debug, Error)]
pub enum SourceError {
    #[error("provided path `{0}` is expected to be a file")]
    NotAFile(PathBuf),
    #[error("provided path `{0}` is expected to be relative")]
    NotRelative(PathBuf),
    #[error("permission to access `{0}` was denied")]
    PermissionDenied(PathBuf),
    #[error("provided path `{0}` is not found")]
    NotFound(PathBuf),
    #[error("provided path `{0}` caused `{1}`")]
    IoErrorWithSource(PathBuf, io::Error),
    #[error("{0}")]
    IoError(#[from] io::Error),
}
