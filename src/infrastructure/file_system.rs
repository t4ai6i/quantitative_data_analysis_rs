use anyhow::{Context, Result};
use std::backtrace::Backtrace;
use std::path::PathBuf;
use tokio::fs::read;

pub struct FileSystem {
    pub file_path: PathBuf,
}

impl FileSystem {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }

    pub(crate) async fn read_file(&self) -> Result<Vec<u8>> {
        read(&self.file_path).await.with_context(|| {
            format!(
                "File not found: {:?})\n{}",
                &self.file_path,
                Backtrace::force_capture()
            )
        })
    }
}
