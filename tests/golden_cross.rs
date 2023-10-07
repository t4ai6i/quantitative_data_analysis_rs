use anyhow::Result;
use quantitative_data_analysis_rs::domain::entity::cross::{PairSMA, VecCross};
use quantitative_data_analysis_rs::domain::entity::sma::VecSMA;
use quantitative_data_analysis_rs::domain::entity::stock::VecStock;
use simple_moving_average::{SumTreeSMA, SMA};

const CSV_8473: &[u8] = include_bytes!("../assets/8473.T.csv");

#[test]
fn golden_cross_sandbox() -> Result<()> {
    let mut ma = SumTreeSMA::<_, f32, 5>::new(); // Sample window size = 2
    ma.add_sample(100.0);
    ma.add_sample(200.0);
    ma.add_sample(300.0);
    ma.add_sample(400.0);
    ma.add_sample(500.0);
    let expected = 300.0;
    assert_eq!(ma.get_average(), expected);
    Ok(())
}

#[test]
fn golden_cross_test() -> Result<()> {
    const FIVE_DAY: usize = 5;
    const TWENTY_FIVE_DAY: usize = 25;
    let VecStock(stocks) = VecStock::<true>::from(CSV_8473);
    let VecSMA(five_days) = VecSMA::<FIVE_DAY>::from(stocks.as_slice());
    let VecSMA(twenty_five_days) = VecSMA::<TWENTY_FIVE_DAY>::from(stocks.as_slice());
    let pair_sma = PairSMA {
        pair1: five_days.as_slice(),
        pair2: twenty_five_days.as_slice(),
    };
    let VecCross(crosses) = VecCross::from(pair_sma);
    assert_eq!(crosses.len(), 242);
    Ok(())
}
