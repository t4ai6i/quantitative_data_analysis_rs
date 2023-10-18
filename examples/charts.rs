use anyhow::Result;
use charts_rs::{CandlestickChart, LegendCategory, Series, SeriesCategory};
use quantitative_data_analysis_rs::domain::entity::cross::{
    CrossDirectionType, SMAListPair, VecCross, VecCrossExt,
};
use quantitative_data_analysis_rs::domain::entity::sma::{VecSMA, VecSMAExt};
use quantitative_data_analysis_rs::domain::entity::stock::{VecStock, VecStockExt};
use std::fs::write;

const CSV_8473: &[u8] = include_bytes!("../assets/8473.T.csv");

fn main() -> Result<()> {
    // TODO: GoldenCross発生当日の調整後終値と比べ、3営業日後の値が上昇したか判定していく。
    let vec_stock = VecStock::<true>::from(CSV_8473);
    let vec_sma_5 = VecSMA::<5>::from(vec_stock.0.as_slice());
    let vec_sma_25 = VecSMA::<25>::from(vec_stock.0.as_slice());
    let sma_list_pair = SMAListPair {
        smas_n: vec_sma_5.0.as_slice(),
        smas_o: vec_sma_25.0.as_slice(),
    };
    let vec_cross = VecCross::from(sma_list_pair);

    let vec_sma_5_ave = vec_sma_5.collect_vec_ave();
    let vec_sma_25_ave = vec_sma_25.collect_vec_ave();
    let candlesticks = vec_stock.collect_vec_candlestick();
    let dead_crosses = vec_cross.collect_vec_cross(CrossDirectionType::Dead);
    let golden_crosses = vec_cross.collect_vec_cross(CrossDirectionType::Golden);
    let min = candlesticks.iter().copied().fold(f32::INFINITY, f32::min);
    let max = candlesticks
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    let min = min - 10.0;
    let max = max + 10.0;

    let x_axis_data = vec_stock.collect_vec_day();

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
    let svg = candlestick_chart.svg()?;
    write("./examples/charts.svg", svg)?;
    Ok(())
}
