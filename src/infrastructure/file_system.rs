use crate::infrastructure::data_format::DataFormat;
use anyhow::{bail, Context, Result};
use std::backtrace::Backtrace;
use tokio::fs::read;

pub struct FileSystem {
    pub data_format: DataFormat,
}

impl FileSystem {
    pub fn new(data_format: DataFormat) -> Self {
        Self { data_format }
    }

    pub(crate) async fn read_file(&self) -> Result<Vec<u8>> {
        let file_path = match &self.data_format {
            DataFormat::JSON { file_path } => file_path,
            DataFormat::CSV { file_path, .. } => file_path,
            DataFormat::TSV { file_path, .. } => file_path,
            _ => bail!(
                "Unsupported data format: {:?}\n{}",
                self.data_format,
                Backtrace::force_capture()
            ),
        };
        read(file_path).await.with_context(|| {
            format!(
                "File not found: {:?})\n{}",
                file_path,
                Backtrace::force_capture()
            )
        })
    }
}
