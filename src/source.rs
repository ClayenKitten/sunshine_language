//! Source code and file hierarchy management.

use std::{
    collections::{hash_map::Entry, HashMap},
    fs,
    io::{self, Read},
    ops::IndexMut,
    path::{Path, PathBuf},
};
use thiserror::Error;

use crate::{path::ItemPath, util::MonotonicVec};

/// The structure that holds the whole source code of the compiled program.
#[derive(Debug)]
pub struct SourceMap {
    root: PathBuf,
    mapping: HashMap<PathBuf, SourceId>,
    files: MonotonicVec<SourceFile>,
}

impl SourceMap {
    /// Creates new [SourceMap] with path to the main file.
    ///
    /// # Errors
    ///
    /// Error is only returned if `root` is not found or couldn't be opened.
    pub fn new(main: PathBuf) -> Result<Self, SourceError> {
        let mut map = Self {
            mapping: HashMap::new(),
            files: MonotonicVec::new(),
            root: {
                let mut root = main.clone();
                root.pop();
                root
            },
        };
        map.insert_path(main)?;
        Ok(map)
    }

    #[cfg(test)]
    pub fn new_test() -> Result<Self, SourceError> {
        use std::str::FromStr;

        Ok(Self {
            mapping: HashMap::new(),
            files: MonotonicVec::new(),
            root: PathBuf::from_str("/dev/null").unwrap(),
        })
    }

    /// Inserts new source file to the map and returns its id.
    pub fn insert(&mut self, path: ItemPath) -> Result<SourceId, SourceError> {
        let mut source_path = self.root.clone();
        source_path.extend(path.into_path_buf().iter());
        self.insert_path(source_path)
    }

    /// Inserts new source file to the map and returns its id.
    pub fn insert_path(&mut self, path: PathBuf) -> Result<SourceId, SourceError> {
        let id = self.generate_id();
        Ok(match self.mapping.entry(path.clone()) {
            Entry::Vacant(entry) => {
                let file = SourceFile::new(path)?;
                entry.insert(id);
                self.files.push(file);
                id
            }
            Entry::Occupied(entry) => *entry.get(),
        })
    }

    /// Gets file by id.
    pub fn get(&mut self, id: SourceId) -> &mut SourceFile {
        self.files.index_mut(id.0 as usize)
    }

    /// Gets path of the file.
    ///
    /// That function may be slow as it traverses internal HashMap to find the value.
    pub fn get_path(&self, id: SourceId) -> &Path {
        self.mapping
            .iter()
            .find_map(|(path, checked_id)| (*checked_id == id).then_some(path.as_path()))
            .expect("each SourceId should have corresponding entry in mapping")
    }

    /// Create new [SourceId].
    fn generate_id(&self) -> SourceId {
        SourceId(self.files.len() as u32)
    }
}

/// A sequential id of the file.
///
/// It is guaranteed that every SourceId maps to a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceId(u32);

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
            Ok(meta) if !meta.is_file() => Err(SourceError::NotAFile(path.to_owned())),
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
            Err(err) => Err(SourceError::IoErrorWithSource(path.to_owned(), err)),
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
