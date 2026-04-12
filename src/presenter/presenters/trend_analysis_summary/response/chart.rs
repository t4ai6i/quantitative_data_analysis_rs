use anyhow::{Context, Result};
use charts_rs::TableChart;
use std::backtrace::Backtrace;

use crate::presenter::presenters::trend_analysis_summary::output;
use crate::presenter::presenters::trend_analysis_summary::presenter;
use crate::presenter::presenters::trend_analysis_summary::response;
use crate::presenter::views::shared::crossover_pattern_filter::CrossoverPatternFilter;
use crate::presenter::views::trend_analysis_summary::chart::view;

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

impl presenter::TrendAnalysisSummary for Chart {
    fn handle(
        &self,
        output: output::TrendAnalysisSummary,
        crossover_pattern_filter: CrossoverPatternFilter,
    ) -> Result<response::TrendAnalysisSummary> {
        let chance_rate = format!("chance rate({})", crossover_pattern_filter);
        let mut rows = vec![vec![
            "code".to_string(),
            "symbol".to_string(),
            "sma cos".to_string(),
            "latest".to_string(),
            chance_rate,
        ]];
        let chart = view::Chart::from(output.trend_analyses);
        rows.append(&mut chart.to_tabular_format());
        let mut table_chart = TableChart::new_with_theme(rows, self.theme.as_str());
        table_chart.width = self.width;
        Ok(response::TrendAnalysisSummary::Chart {
            body: table_chart
                .svg()
                .with_context(|| format!("{}", Backtrace::force_capture()))?,
        })
    }
}
