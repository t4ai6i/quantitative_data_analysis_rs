use std::backtrace::Backtrace;

use anyhow::{Context, Result};
use charts_rs::TableChart;

use crate::presenter::presenters::trend_summary::output;
use crate::presenter::presenters::trend_summary::presenter;
use crate::presenter::presenters::trend_summary::response;
use crate::presenter::view_models::shared::crossover_pattern_filter::CrossoverPatternFilter;
use crate::presenter::view_models::trend_analysis::view_model::TrendAnalyses;

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

impl presenter::TrendSummary for Chart {
    fn handle(
        &self,
        output: output::TrendSummary,
        crossover_pattern_filter: CrossoverPatternFilter,
    ) -> Result<response::TrendSummary> {
        let chance_rate = format!("chance rate({})", crossover_pattern_filter);
        let mut rows = vec![vec![
            "code".to_string(),
            "symbol".to_string(),
            "macos type".to_string(),
            "latest".to_string(),
            chance_rate,
        ]];
        let mut body = output.trend_analyses.table_chart_rows();
        rows.append(&mut body);
        let mut table_chart = TableChart::new_with_theme(rows, self.theme.as_str());
        table_chart.width = self.width;
        Ok(response::TrendSummary::Chart {
            body: table_chart
                .svg()
                .with_context(|| format!("{}", Backtrace::force_capture()))?,
        })
    }
}
