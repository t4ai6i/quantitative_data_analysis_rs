use crate::infrastructure::company_repository::data_format::DataFormat as CPDF;
use crate::infrastructure::stock_repository::data_format::DataFormat as SRDF;
use crate::presenter::trend_analysis_presenter::{TrendAnalysisPresenter, TrendAnalysisResponse};
use crate::use_case::interface::trend_analysis_use_case::{
    TrendAnalysisInput, TrendAnalysisUseCase,
};
use anyhow::Result;
use chrono::NaiveDate;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TrendAnalysisController<'a, I, P> {
    interactor: &'a I,
    presenter: &'a P,
}

impl<'a, I, P> TrendAnalysisController<'a, I, P>
where
    I: TrendAnalysisUseCase,
    P: TrendAnalysisPresenter,
{
    pub fn new(interactor: &'a I, presenter: &'a P) -> Self {
        Self {
            interactor,
            presenter,
        }
    }

    pub async fn analyze<const N: usize>(
        &self,
        code: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        sr_data_format: SRDF,
        cp_data_format: CPDF,
    ) -> Result<TrendAnalysisResponse> {
        let input =
            TrendAnalysisInput::new(code, start_date, end_date, sr_data_format, cp_data_format);
        let output = self.interactor.handle::<N>(input).await?;
        let response = self.presenter.handle(output)?;
        Ok(response)
    }
}
