use anyhow::Result;
use simple_moving_average::{SMA, SumTreeSMA};

#[test]
fn simple_moving_average_sandbox() -> Result<()> {
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
