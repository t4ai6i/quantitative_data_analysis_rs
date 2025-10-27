use anyhow::Result;
use async_trait::async_trait;
use rayon::prelude::*;

use crate::domain::models::company::model;
use crate::domain::repositories::company::queries;

#[async_trait]
pub trait Company {
    async fn get_company<'a>(
        &self,
        query: &queries::get_company::Query<'a>,
    ) -> Result<model::Company> {
        let row_company = self.get_row_company(query).await?;
        TryFrom::try_from(row_company)
    }

    async fn get_companies(&self) -> Result<model::Companies> {
        let vec_row_company = self.get_vec_row_company().await?;
        let vec_company = vec_row_company
            .into_par_iter()
            .filter_map(|row_company| TryFrom::try_from(row_company).ok())
            .collect::<Vec<model::Company>>();
        Ok(model::Companies(vec_company))
    }

    async fn get_row_company<'a>(
        &self,
        query: &queries::get_company::Query<'a>,
    ) -> Result<model::RowCompany>;

    async fn get_vec_row_company(&self) -> Result<Vec<model::RowCompany>>;
}
