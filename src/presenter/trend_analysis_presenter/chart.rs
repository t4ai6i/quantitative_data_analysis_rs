use crate::domain::entity::cross::CrossDirectionType;
use crate::presenter::trend_analysis_presenter::{
    TrendAnalysisOutput, TrendAnalysisPresenter, TrendAnalysisResponse,
};
use crate::utils::float;
use crate::view_model::vec_cross::VecCrossExt;
use crate::view_model::vec_sma::VecSMAExt;
use crate::view_model::vec_stock::VecStockExt;
use anyhow::Result;
use charts_rs::{CandlestickChart, LegendCategory, Series, SeriesCategory};

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
        let vec_sma_5_ave = output.vec_sma_5.collect_vec_ave();
        let vec_sma_25_ave = output.vec_sma_25.collect_vec_ave();
        let candlesticks = output.vec_stock.collect_vec_candlestick();
        let x_axis_data = output.vec_stock.collect_vec_day();
        let dead_crosses = output
            .vec_cross
            .collect_vec_sma_25_ave(CrossDirectionType::Dead);
        let golden_crosses = output
            .vec_cross
            .collect_vec_sma_25_ave(CrossDirectionType::Golden);
        let min = float::min(&candlesticks) - 10.0;
        let max = float::max(&candlesticks) + 10.0;
        let mut candlestick_chart = CandlestickChart::new_with_theme(
            vec![
                Series::from(("SMA5", vec_sma_5_ave)),
                Series::from(("SMA25", vec_sma_25_ave)),
                Series::from(("Dead", dead_crosses)),
                Series::from(("Golden", golden_crosses)),
                Series::from(("Daily", candlesticks)),
            ],
            x_axis_data,
            "chalk",
        );
        candlestick_chart.width = 1280.0;
        candlestick_chart.height = 720.0;
        candlestick_chart.legend_category = LegendCategory::RoundRect;
        candlestick_chart.series_list[0].category = Some(SeriesCategory::Line);
        candlestick_chart.series_list[0].start_index = 5;
        candlestick_chart.series_list[1].category = Some(SeriesCategory::Line);
        candlestick_chart.series_list[1].start_index = 25;
        candlestick_chart.series_list[2].category = Some(SeriesCategory::Line);
        candlestick_chart.series_list[2].start_index = 6;
        candlestick_chart.series_list[3].category = Some(SeriesCategory::Line);
        candlestick_chart.series_list[3].start_index = 6;
        candlestick_chart.y_axis_configs[0].axis_min = Some(min);
        candlestick_chart.y_axis_configs[0].axis_max = Some(max);
        candlestick_chart.y_axis_configs[0].axis_formatter = Some("{t}".to_string());
        let body = candlestick_chart.svg()?;
        Ok(TrendAnalysisResponse::Chart { body })
        // // 3日後トレンドを取得
        // let vec_trend_analysis = VecTrendAnalysis::<3>::from(stock_cross_pair);
        // let mut table_chart = TableChart::new(vec![]);
    }
}
