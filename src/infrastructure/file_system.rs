use crate::infrastructure::data_format::DataFormat;

pub struct FileSystem {
    pub data_format: DataFormat,
}

impl FileSystem {
    pub fn new(data_format: DataFormat) -> Self {
        Self { data_format }
    }
}
