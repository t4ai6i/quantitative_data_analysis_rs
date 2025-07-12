use crate::domain::models::company::model;
use crate::domain::repositories::company::repository;
use crate::infrastructure::dsv::Dsv;
use crate::infrastructure::from_slice::FromSlice;
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::backtrace::Backtrace;
use std::fmt::{Debug, Display};

#[async_trait::async_trait]
impl<T> repository::Company for Dsv<T>
where
    T: FromSlice<Item = model::Company> + Send + Sync,
    <T as FromSlice>::Deserialize: Send + Sync,
    <T::Item as TryFrom<T::Deserialize>>::Error: Debug + Display + Send + Sync,
    model::Company: TryFrom<<T as FromSlice>::Deserialize>,
{
    async fn get_company(&self, code: &str, _: &str) -> Result<model::Company> {
        let companies = self.get_companies().await?;
        companies
            .into_par_iter()
            .find_first(|company| company.code.eq(code))
            .with_context(|| {
                format!(
                    "Not found company: {}\n{}",
                    code,
                    Backtrace::force_capture()
                )
            })
    }

    async fn get_companies(&self) -> Result<Vec<model::Company>> {
        let mut cache = self.cache.lock().await;
        if let Some(companies) = cache.as_ref() {
            return Ok(companies.clone());
        }
        let buffer = self.buffer.as_slice();
        let processed = T::process_tabular_data(buffer, self.has_headers)?;
        *cache = Some(processed.clone());
        Ok(processed)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::models::company::model;
    use crate::domain::repositories::company::repository::Company;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::company::structures::internal::tsv;

    const TSV: &[u8] = include_bytes!("../../../../assets/companies.tsv");

    #[tokio::test]
    async fn get_company_test() -> anyhow::Result<()> {
        let code = "13080";
        let default_str = "";
        let dsv = Dsv::<tsv::Structure>::new(false, TSV.to_vec());
        let actual = dsv.get_company(code, default_str).await?;
        let expected = model::Company {
            code: "13080".to_string(),
            name: "Listed Index Fund TOPIX".to_string(),
            market: "0109".to_string(),
            symbol: "13080.0109".to_string(),
        };
        assert_eq!(actual, expected);
        Ok(())
    }
}
