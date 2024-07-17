use std::backtrace::Backtrace;

use anyhow::{Context, Result};
use charts_rs::{
    Align, BarChart, Box, CandlestickChart, ChildChart, Color, LegendCategory, MultiChart, Series,
    SeriesCategory, TableChart,
};
use itertools::Itertools;

use crate::domain::entity::cross::CrossDirectionType;
use crate::presenter::{
    trend_analysis_presenter::{
        DisplayCrossPattern, TrendAnalysisOutput, TrendAnalysisPresenter, TrendAnalysisResponse,
    },
    view_model::{
        vec_candle_stick::VecCandleStickExt,
        vec_close_cross_trend_analysis::VecCloseCrossTrendAnalysisExt, vec_cross::VecCrossExt,
        vec_sma::VecSMAExt, vec_stock::VecStockExt,
        vec_volume_cross_trend_analysis::VecVolumeCrossTrendAnalysisExt,
    },
};
use crate::utils::float;

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Chart {
    theme: String,
    width: f32,
    height: f32,
    date_format: String,
}

impl Chart {
    pub fn new(
        theme: impl Into<String>,
        width: f32,
        height: f32,
        date_format: impl Into<String>,
    ) -> Self {
        Self {
            theme: theme.into(),
            width,
            height,
            date_format: date_format.into(),
        }
    }
}

impl TrendAnalysisPresenter for Chart {
    fn handle<const N: usize, const M: usize>(
        &self,
        output: TrendAnalysisOutput<N, M>,
    ) -> Result<TrendAnalysisResponse> {
        let mut charts = MultiChart::new();
        charts.margin = 5.0.into();

        let company = output.company;
        let sma_5_averages = output.vec_sma_5.collect_average_close();
        let sma_25_averages = output.vec_sma_25.collect_average_close();
        let ohlcs: Vec<f32> = output
            .vec_stock
            .collect_ohlc()
            .iter()
            .map(|value| *value as _)
            .collect_vec();
        let min = float::min(&ohlcs) - 10.0;
        let max = float::max(&ohlcs) + 10.0;

        let x_axis_data_days = output
            .vec_stock
            .collect_date_string(self.date_format.as_str());

        let series_list = match output.display_cross_pattern {
            DisplayCrossPattern::All => {
                let dead_crosses = output
                    .vec_cross
                    .collect_vec_sma_25_close_average(CrossDirectionType::Dead);
                let golden_crosses = output
                    .vec_cross
                    .collect_vec_sma_25_close_average(CrossDirectionType::Golden);
                vec![
                    Series::from(("SMA5", sma_5_averages)),
                    Series::from(("SMA25", sma_25_averages)),
                    Series::from(("Dead", dead_crosses)),
                    Series::from(("Golden", golden_crosses)),
                    Series::from(("OHLC", ohlcs)),
                ]
            }
            DisplayCrossPattern::GoldenOnly => {
                let golden_crosses = output
                    .vec_cross
                    .collect_vec_sma_25_close_average(CrossDirectionType::Golden);
                vec![
                    Series::from(("SMA5", sma_5_averages)),
                    Series::from(("SMA25", sma_25_averages)),
                    Series::from(("Golden", golden_crosses)),
                    Series::from(("OHLC", ohlcs)),
                ]
            }
            DisplayCrossPattern::DeadOnly => {
                let dead_crosses = output
                    .vec_cross
                    .collect_vec_sma_25_close_average(CrossDirectionType::Dead);
                vec![
                    Series::from(("SMA5", sma_5_averages)),
                    Series::from(("SMA25", sma_25_averages)),
                    Series::from(("Dead", dead_crosses)),
                    Series::from(("OHLC", ohlcs)),
                ]
            }
        };

        let mut candlestick_chart = CandlestickChart::new_with_theme(
            series_list,
            x_axis_data_days.clone(),
            self.theme.as_str(),
        );
        candlestick_chart.title_text = format!("{}({})", &company.name, &company.code);
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
        candlestick_chart.candlestick_up_color = Color::from((236, 0, 0));
        candlestick_chart.candlestick_up_border_color = Color::from((138, 0, 0));
        candlestick_chart.candlestick_down_color = Color::from((0, 60, 218));
        candlestick_chart.candlestick_down_border_color = Color::from((0, 40, 143));
        charts.add(ChildChart::Candlestick(candlestick_chart, None));

        let sma_5_averages = output.vec_sma_5.collect_average_volume();
        let sma_25_averages = output.vec_sma_25.collect_average_volume();
        let volumes: Vec<f32> = output
            .vec_stock
            .collect_volume()
            .iter()
            .map(|value| *value as _)
            .collect_vec();

        let series_list = match output.display_cross_pattern {
            DisplayCrossPattern::All => {
                let dead_crosses = output
                    .vec_cross
                    .collect_vec_sma_25_volume_average(CrossDirectionType::Dead);
                let golden_crosses = output
                    .vec_cross
                    .collect_vec_sma_25_volume_average(CrossDirectionType::Golden);
                vec![
                    Series::from(("SMA5", sma_5_averages)),
                    Series::from(("SMA25", sma_25_averages)),
                    Series::from(("Dead", dead_crosses)),
                    Series::from(("Golden", golden_crosses)),
                    Series::from(("Volume", volumes)),
                ]
            }
            DisplayCrossPattern::GoldenOnly => {
                let golden_crosses = output
                    .vec_cross
                    .collect_vec_sma_25_volume_average(CrossDirectionType::Golden);
                vec![
                    Series::from(("SMA5", sma_5_averages)),
                    Series::from(("SMA25", sma_25_averages)),
                    Series::from(("Golden", golden_crosses)),
                    Series::from(("Volume", volumes)),
                ]
            }
            DisplayCrossPattern::DeadOnly => {
                let dead_crosses = output
                    .vec_cross
                    .collect_vec_sma_25_volume_average(CrossDirectionType::Dead);
                vec![
                    Series::from(("SMA5", sma_5_averages)),
                    Series::from(("SMA25", sma_25_averages)),
                    Series::from(("Dead", dead_crosses)),
                    Series::from(("Volume", volumes)),
                ]
            }
        };

        let mut volume_chart =
            BarChart::new_with_theme(series_list, x_axis_data_days, self.theme.as_str());
        volume_chart.width = self.width;
        volume_chart.height = self.height / 3.0;
        volume_chart.series_list[0].category = Some(SeriesCategory::Line);
        volume_chart.series_list[0].start_index = 5;
        volume_chart.series_list[1].category = Some(SeriesCategory::Line);
        volume_chart.series_list[1].start_index = 25;
        match output.display_cross_pattern {
            DisplayCrossPattern::All => {
                volume_chart.series_list[2].category = Some(SeriesCategory::Line);
                volume_chart.series_list[2].start_index = 6;
                volume_chart.series_list[3].category = Some(SeriesCategory::Line);
                volume_chart.series_list[3].start_index = 6;
            }
            DisplayCrossPattern::GoldenOnly => {
                volume_chart.series_list[2].category = Some(SeriesCategory::Line);
                volume_chart.series_list[2].start_index = 6;
            }
            DisplayCrossPattern::DeadOnly => {
                volume_chart.series_list[2].category = Some(SeriesCategory::Line);
                volume_chart.series_list[2].start_index = 6;
            }
        }
        charts.add(ChildChart::Bar(volume_chart, None));

        let mut rows = output.vec_close_cross_trend_analysis.table_chart_header();
        let mut body = output
            .vec_close_cross_trend_analysis
            .table_chart_rows(&output.display_cross_pattern);
        rows.append(&mut body);
        let mut summary = output
            .vec_close_cross_trend_analysis
            .table_chart_summary(&output.display_cross_pattern);
        rows.append(&mut summary);
        let mut table_chart = TableChart::new_with_theme(rows, self.theme.as_str());
        table_chart.title_text = "CloseCrossTrendAnalysis".to_string();
        table_chart.width = self.width;
        charts.add(ChildChart::Table(table_chart, None));

        let mut rows = output.vec_volume_cross_trend_analysis.table_chart_header();
        let mut body = output
            .vec_volume_cross_trend_analysis
            .table_chart_rows(&output.display_cross_pattern);
        rows.append(&mut body);
        let mut table_chart = TableChart::new_with_theme(rows, self.theme.as_str());
        table_chart.title_text = "VolumeCrossTrendAnalysis".to_string();
        table_chart.width = self.width;
        charts.add(ChildChart::Table(table_chart, None));

        let mut rows = vec![vec![
            "Date".to_string(),
            "Size, Body(%)".to_string(),
            "⤴️⤵️".to_string(),
            "Marubozu".to_string(),
            "UpperWick(%), LowerWick(%)".to_string(),
            "Doji".to_string(),
        ]];
        let mut body = output.vec_candle_stick.table_chart_rows();
        rows.append(&mut body);
        let mut table_chart = TableChart::new_with_theme(rows, self.theme.as_str());
        table_chart.title_text = "Stocks".to_string();
        table_chart.width = self.width;
        charts.add(ChildChart::Table(table_chart, None));

        // ECP1/ECP2の結果をテーブルで表示
        // output.vec_ecp2

        Ok(TrendAnalysisResponse::Chart {
            company,
            body: charts
                .svg()
                .with_context(|| format!("{}", Backtrace::force_capture()))?,
            display_cross_pattern: output.display_cross_pattern,
            chance_rate: output.vec_close_cross_trend_analysis.chance_rate,
            latest_chance: output.vec_close_cross_trend_analysis.latest_chance,
        })
    }
}
