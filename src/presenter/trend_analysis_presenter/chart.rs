use crate::domain::entity::cross::CrossDirectionType;
use crate::presenter::trend_analysis_presenter::DisplayCrossPattern;
use crate::presenter::{
    trend_analysis_presenter::{
        TrendAnalysisOutput, TrendAnalysisPresenter, TrendAnalysisResponse,
    },
    view_model::{
        vec_cross::VecCrossExt, vec_sma::VecSMAExt, vec_stock::VecStockExt,
        vec_trend_analysis::VecTrendAnalysisExt,
    },
};
use crate::utils::float;
use anyhow::{Context, Result};
use charts_rs::{
    Align, Box, CandlestickChart, ChildChart, Color, LegendCategory, MultiChart, Series,
    SeriesCategory, TableChart,
};
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

impl TrendAnalysisPresenter for Chart {
    fn handle<const N: usize>(
        &self,
        output: TrendAnalysisOutput<N>,
    ) -> Result<TrendAnalysisResponse> {
        let company = output.company;
        let vec_sma_5_ave = output.vec_sma_5.collect_vec_ave();
        let vec_sma_25_ave = output.vec_sma_25.collect_vec_ave();
        let candlesticks = output.vec_stock.collect_vec_candlestick();
        let min = float::min(&candlesticks) - 10.0;
        let max = float::max(&candlesticks) + 10.0;
        let x_axis_data = output.vec_stock.collect_vec_day();
        let series_list = match output.display_cross_pattern {
            DisplayCrossPattern::All => {
                let dead_crosses = output
                    .vec_cross
                    .collect_vec_sma_25_ave(CrossDirectionType::Dead);
                let golden_crosses = output
                    .vec_cross
                    .collect_vec_sma_25_ave(CrossDirectionType::Golden);
                vec![
                    Series::from(("SMA5", vec_sma_5_ave)),
                    Series::from(("SMA25", vec_sma_25_ave)),
                    Series::from(("Dead", dead_crosses)),
                    Series::from(("Golden", golden_crosses)),
                    Series::from(("Daily", candlesticks)),
                ]
            }
            DisplayCrossPattern::GoldenOnly => {
                let golden_crosses = output
                    .vec_cross
                    .collect_vec_sma_25_ave(CrossDirectionType::Golden);
                vec![
                    Series::from(("SMA5", vec_sma_5_ave)),
                    Series::from(("SMA25", vec_sma_25_ave)),
                    Series::from(("Golden", golden_crosses)),
                    Series::from(("Daily", candlesticks)),
                ]
            }
            DisplayCrossPattern::DeadOnly => {
                let dead_crosses = output
                    .vec_cross
                    .collect_vec_sma_25_ave(CrossDirectionType::Dead);
                vec![
                    Series::from(("SMA5", vec_sma_5_ave)),
                    Series::from(("SMA25", vec_sma_25_ave)),
                    Series::from(("Dead", dead_crosses)),
                    Series::from(("Daily", candlesticks)),
                ]
            }
        };

        let mut charts = MultiChart::new();
        charts.margin = 10.0.into();
        let mut candlestick_chart =
            CandlestickChart::new_with_theme(series_list, x_axis_data, self.theme.as_str());
        candlestick_chart.title_text = format!("{}({})", company.name, company.symbol);
        candlestick_chart.width = self.width;
        candlestick_chart.height = self.height;
        candlestick_chart.legend_margin = Some(Box::from(30.0));
        candlestick_chart.legend_align = Align::Center;
        candlestick_chart.legend_category = LegendCategory::Normal;
        candlestick_chart.series_list[0].category = Some(SeriesCategory::Line);
        candlestick_chart.series_list[0].start_index = 5;
        candlestick_chart.series_list[1].category = Some(SeriesCategory::Line);
        candlestick_chart.series_list[1].start_index = 25;
        match output.display_cross_pattern {
            DisplayCrossPattern::All => {
                candlestick_chart.series_list[2].category = Some(SeriesCategory::Line);
                candlestick_chart.series_list[2].start_index = 6;
                candlestick_chart.series_list[3].category = Some(SeriesCategory::Line);
                candlestick_chart.series_list[3].start_index = 6;
            }
            DisplayCrossPattern::GoldenOnly => {
                candlestick_chart.series_list[2].category = Some(SeriesCategory::Line);
                candlestick_chart.series_list[2].start_index = 6;
            }
            DisplayCrossPattern::DeadOnly => {
                candlestick_chart.series_list[2].category = Some(SeriesCategory::Line);
                candlestick_chart.series_list[2].start_index = 6;
            }
        }
        candlestick_chart.y_axis_configs[0].axis_min = Some(min);
        candlestick_chart.y_axis_configs[0].axis_max = Some(max);
        candlestick_chart.y_axis_configs[0].axis_formatter = Some("{t}".to_string());
        candlestick_chart.candlestick_up_color = Color::from((0, 218, 60));
        candlestick_chart.candlestick_up_color = Color::from((0, 218, 60));
        candlestick_chart.candlestick_up_border_color = Color::from((0, 143, 40));
        candlestick_chart.candlestick_down_color = Color::from((236, 0, 0));
        candlestick_chart.candlestick_down_border_color = Color::from((138, 0, 0));
        charts.add(ChildChart::Candlestick(candlestick_chart, None));

        let mut rows = vec![vec![
            "date".to_string(),
            "chance loss".to_string(),
            "direction".to_string(),
            "per inc/dec".to_string(),
            "close at cross".to_string(),
            format!("close after {} days", N),
        ]];
        let mut body = output
            .vec_trend
            .table_chart_rows(&output.display_cross_pattern);
        rows.append(&mut body);
        let mut summary = output
            .vec_trend
            .table_chart_summary(&output.display_cross_pattern);
        rows.append(&mut summary);

        let mut table_chart = TableChart::new_with_theme(rows, self.theme.as_str());
        table_chart.width = self.width;
        charts.add(ChildChart::Table(table_chart, None));

        Ok(TrendAnalysisResponse::Chart {
            company,
            body: charts
                .svg()
                .with_context(|| format!("{}", Backtrace::force_capture()))?,
            display_cross_pattern: output.display_cross_pattern,
            chance_rate: output.vec_trend.chance_rate,
            latest_chance: output.vec_trend.latest_chance,
        })
    }
}
