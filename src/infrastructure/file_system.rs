use std::path::PathBuf;

pub struct FileSystem {
    pub(crate) root: PathBuf,
}

impl FileSystem {
    pub fn new(path_buf: PathBuf) -> Self {
        Self { root: path_buf }
    }
}
