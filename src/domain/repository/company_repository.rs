use crate::domain::entity::company::Company;
use crate::infrastructure::company_repository::data_format::DataFormat;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait CompanyRepository {
    async fn get_company(
        &self,
        code: impl Into<String> + Send,
        data_format: DataFormat,
    ) -> Result<Company>;
}
