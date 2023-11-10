use anyhow::Result;
use indoc::indoc;
use quantitative_data_analysis_rs::domain::entity::cross::VecCross;
use quantitative_data_analysis_rs::domain::entity::sma::{SMAListPair, VecSMA};
use quantitative_data_analysis_rs::domain::entity::stock::VecStock;
use quantitative_data_analysis_rs::domain::entity::trend_analysis::{
    StockCrossPair, VecTrendAnalysis,
};
use quantitative_data_analysis_rs::infrastructure::vec_stock_repository::data_format::csv::VecCSVFormat;

const CSV_8473: &[u8] = include_bytes!("../assets/8473.T.csv");
const MD_8473: &str = "./8473.T.md";

fn main() -> Result<()> {
    let vec_csv_format = VecCSVFormat::<true>::from(CSV_8473);
    let vec_stock = VecStock::from(vec_csv_format);
    let vec_sma_5 = VecSMA::<5>::from(vec_stock.0.as_slice());
    let vec_sma_25 = VecSMA::<25>::from(vec_stock.0.as_slice());
    let sma_list_pair = SMAListPair {
        smas_n: vec_sma_5.0.as_slice(),
        smas_o: vec_sma_25.0.as_slice(),
    };
    let vec_cross = VecCross::from(sma_list_pair);
    let stock_cross_pair = StockCrossPair {
        stocks: vec_stock.0.as_slice(),
        crosses: vec_cross.0.as_slice(),
    };
    // 3日後トレンドを取得
    let VecTrendAnalysis(trends) = VecTrendAnalysis::<3>::from(stock_cross_pair);
    dbg!(&trends);
    let expected = indoc! {r#"
        | date | close on cross | close after 3 days | direction | per inc/dec |
        | :---: | :---: | :---: | :---: | :---: |
        | 2022/10/26 | 2668.0 | 2688.0 | Golden | +0.00749 |
        | 2022/11/16 | 2608.0 | 2619.0 | Dead | +0.00421 |
    "#};
    assert_eq!("", expected);
    Ok(())
}
