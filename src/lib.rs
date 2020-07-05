//! Performs path normalization for *nix and Windows systems.
//!
//! Windows and Unix-like systems handle glob patterns completely differently.
//! This library is meant to paper over the differences between the two in order
//! to simplify the construction of cross-platform applications.

use glob::Paths;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub enum PathIter {
    Directory(walkdir::IntoIter),
    File(Option<PathBuf>),
    Glob(Paths),
}

impl Iterator for PathIter {
    type Item = io::Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            PathIter::Directory(dir) => {
                let next = dir.next()?;
                Some(next.map(|x| x.into_path()).map_err(|e| e.into()))
            }
            PathIter::File(path) => path.take().map(Ok),
            PathIter::Glob(paths) => {
                let next = paths.next()?;
                Some(
                    next.map(|x| x.as_path().into())
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e)),
                )
            }
        }
    }
}

pub fn extract_paths(path: &str) -> io::Result<PathIter> {
    {
        let path: &Path = path.as_ref();

        // Not a glob
        if path.exists() {
            if path.is_file() {
                return Ok(PathIter::File(Some(path.into())));
            } else {
                let iter = WalkDir::new(path).into_iter();
                return Ok(PathIter::Directory(iter));
            }
        }
    }

    glob::glob(path)
        .map(PathIter::Glob)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}
