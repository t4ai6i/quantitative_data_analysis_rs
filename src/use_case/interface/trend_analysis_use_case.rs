use crate::infrastructure::vec_stock_repository::data_format::DataFormatType;
use crate::presenter::trend_analysis_presenter::TrendAnalysisOutput;
use anyhow::Result;
use chrono::NaiveDate;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct TrendAnalysisInput {
    pub code: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub data_format_type: DataFormatType,
}

impl TrendAnalysisInput {
    pub fn new(
        code: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        data_format_type: DataFormatType,
    ) -> Self {
        Self {
            code: code.into(),
            start_date,
            end_date,
            data_format_type,
        }
    }
}

pub trait TrendAnalysisUseCase {
    fn handle<const N: usize>(&self, input: TrendAnalysisInput) -> Result<TrendAnalysisOutput<N>>;
}
