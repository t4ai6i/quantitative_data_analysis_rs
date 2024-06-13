use crate::presenter::trend_analysis_presenter::DisplayCrossPattern;
use crate::presenter::trend_summary_presenter::{
    TrendSummaryOutput, TrendSummaryPresenter, TrendSummaryResponse,
};
use crate::presenter::view_model::vec_trend_analysis_response::VecTrendAnalysisResponseExt;
use anyhow::{Context, Result};
use charts_rs::TableChart;
use std::backtrace::Backtrace;

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Chart {
    theme: String,
    width: f32,
    height: f32,
}

impl Chart {
    pub fn new(theme: impl Into<String>, width: f32, height: f32) -> Self {
        Self {
            theme: theme.into(),
            width,
            height,
        }
    }
}

impl TrendSummaryPresenter for Chart {
    fn handle(
        &self,
        output: TrendSummaryOutput,
        display_cross_pattern: DisplayCrossPattern,
    ) -> Result<TrendSummaryResponse> {
        let chance_rate = format!("chance rate({})", display_cross_pattern);
        let mut rows = vec![vec![
            "code".to_string(),
            "symbol".to_string(),
            "cross direction".to_string(),
            "latest".to_string(),
            chance_rate,
        ]];
        let mut body = output.vec_trend_analysis_response.table_chart_rows();
        rows.append(&mut body);
        let mut table_chart = TableChart::new_with_theme(rows, self.theme.as_str());
        table_chart.width = self.width;
        Ok(TrendSummaryResponse::Chart {
            body: table_chart
                .svg()
                .with_context(|| format!("{}", Backtrace::force_capture()))?,
        })
    }
}
