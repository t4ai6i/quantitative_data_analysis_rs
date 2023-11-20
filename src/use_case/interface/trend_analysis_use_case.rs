use crate::infrastructure::company_repository::data_format::DataFormat as CPDF;
use crate::infrastructure::stock_repository::data_format::DataFormat as SRDF;
use crate::presenter::trend_analysis_presenter::TrendAnalysisOutput;
use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct TrendAnalysisInput {
    pub code: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub sr_data_format: SRDF,
    pub cp_data_format: CPDF,
}

impl TrendAnalysisInput {
    pub fn new(
        code: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        sr_data_format: SRDF,
        cp_data_format: CPDF,
    ) -> Self {
        Self {
            code: code.into(),
            start_date,
            end_date,
            sr_data_format,
            cp_data_format,
        }
    }
}

#[async_trait]
pub trait TrendAnalysisUseCase {
    async fn handle<const N: usize>(
        &self,
        input: TrendAnalysisInput,
    ) -> Result<TrendAnalysisOutput<N>>;
}
