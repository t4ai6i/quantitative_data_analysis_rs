use anyhow::Result;
use itertools::Itertools;
use quantitative_data_analysis_rs::{
    domain::{
        entity::cross::Cross,
        repo::{sma_repo::SMARepo, stock_repo::StockRepo},
    },
    infra::{sma_repo_impl::SMARepoImpl, stock_repo_impl::StockRepoImpl},
};
use simple_moving_average::{SumTreeSMA, SMA};

static CSV_8473: &[u8] = include_bytes!("../assets/8473.T.csv");

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
    let stock_repo = StockRepoImpl::new();
    let stocks = stock_repo.vec_from_csv(CSV_8473, true);
    let sma_repo = SMARepoImpl::new();
    let five_days = sma_repo.collect_vec::<FIVE_DAY>(&stocks);
    let twenty_five_days = sma_repo.collect_vec::<TWENTY_FIVE_DAY>(&stocks);
    // TODO: last_dateをキーにCross構造体を構築する。キーが見つからない場合Noneとする。
    let _list_cross = five_days
        .iter()
        .map(|five_day| {
            let twenty_five_day_found = twenty_five_days
                .iter()
                .find(|twenty_five_day| five_day.date.eq(&twenty_five_day.date));
            Cross {
                date: five_day.date,
                five_day: Some(five_day.value),
                twenty_five_day: twenty_five_day_found.map(|twenty_five_day| twenty_five_day.value),
                ..Default::default()
            }
        })
        .inspect(|cross| {
            dbg!(cross);
        })
        .collect_vec();
    // TODO: five_day_sma_aveとtwenty_five_day_aveの値の大小関係が前日と逆転しているかどうかを判定していく。
    // TODO: GoldenCross発生当日の調整後終値と比べ、3営業日後の値が上昇したか判定していく。
    Ok(())
}
