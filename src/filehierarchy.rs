use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::bail;

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct FileDirPosition {
    pub source: PathBuf,
    pub position: usize,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct FileHierarchy {
    pub list: BTreeSet<FileDirPosition>,
    pub raw: String,
}

impl FileHierarchy {
    pub fn new() -> Self {
        Self {
            list: BTreeSet::new(),
            raw: String::new(),
        }
    }

    pub fn new_from_raw(raw: String) -> Result<Self, anyhow::Error> {
        let mut new_self = Self::new();
        new_self.raw = raw.clone();
        for (i, path) in raw.split('\n').enumerate() {
            if path.is_empty() {
                continue;
            }

            let outcome = new_self.insert(PathBuf::from(path), i);
            if !outcome {
                bail!("Duplicate path {:?}", path)
            }
        }

        Ok(new_self)
    }

    pub fn insert(&mut self, value: PathBuf, position: usize) -> bool {
        let found = self.list.iter().find(|file| file.source == value);
        if found.is_some() {
            return false;
        }

        self.list.insert(FileDirPosition {
            source: value,
            position,
        })
    }

    pub fn enumerate(&mut self) -> Self {
        let mut new_self = Self::new();
        for (i, path) in self.list.iter().enumerate() {
            new_self.insert(path.source.to_owned(), i);

            let output_path = path.source.display().to_string() + "\n";
            new_self.raw += &output_path;
        }

        new_self
    }

    pub fn get_by_index(&self, index: usize) -> Option<&FileDirPosition> {
        self.list.iter().find(|file| file.position == index)
    }

    pub fn get_by_file(&self, path: &Path) -> Option<&FileDirPosition> {
        self.list
            .iter()
            .find(|file| file.source.to_str() == path.to_str())
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_hierarchy() {
        todo!();
    }
}