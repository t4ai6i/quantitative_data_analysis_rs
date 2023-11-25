use crate::domain::entity::company::Company;
use crate::infrastructure::data_format::DataFormat;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait CompanyRepository {
    async fn get_company(
        &self,
        code: impl Into<String> + Send + Copy,
        data_format: DataFormat,
    ) -> Result<Company>;
}
