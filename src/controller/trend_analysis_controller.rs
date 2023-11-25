use crate::infrastructure::data_format::DataFormat;
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
        stock_data_format: DataFormat,
        company_data_format: DataFormat,
    ) -> Result<TrendAnalysisResponse> {
        let input = TrendAnalysisInput::new(
            code,
            start_date,
            end_date,
            stock_data_format,
            company_data_format,
        );
        let output = self.interactor.handle::<N>(input).await?;
        let response = self.presenter.handle(output)?;
        Ok(response)
    }
}
