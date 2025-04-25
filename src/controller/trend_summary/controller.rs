use anyhow::Result;

use crate::presenter::macos_pattern_filter::MACOSPatternFilter;
use crate::presenter::presenters::trend_analysis::response::TrendAnalyses;
use crate::presenter::presenters::trend_summary::presenter;
use crate::presenter::presenters::{trend_analysis, trend_summary};
use crate::use_case::interfaces::trend_summary::input;
use crate::use_case::interfaces::trend_summary::use_case;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TrendSummary<'a, I, P> {
    interactor: &'a I,
    presenter: &'a P,
}

impl<'a, I, P> TrendSummary<'a, I, P>
where
    I: use_case::TrendSummary,
    P: presenter::TrendSummary,
{
    pub fn new(interactor: &'a I, presenter: &'a P) -> Self {
        Self {
            interactor,
            presenter,
        }
    }

    pub async fn analyze(
        &self,
        vec_trend_analysis: Vec<trend_analysis::response::TrendAnalysis>,
        macos_pattern_filter: MACOSPatternFilter,
    ) -> Result<trend_summary::response::TrendSummary> {
        let input = input::TrendSummary::new(TrendAnalyses(vec_trend_analysis));
        let output = self.interactor.handle(input).await?;
        let response = self.presenter.handle(output, macos_pattern_filter)?;
        Ok(response)
    }
}
