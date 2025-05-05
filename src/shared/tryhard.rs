use std::time::Duration;
use tryhard::backoff_strategies::ExponentialBackoff;
use tryhard::{NoOnRetry, RetryFutureConfig};

pub fn get_common_retry_future_config() -> RetryFutureConfig<ExponentialBackoff, NoOnRetry> {
    RetryFutureConfig::new(4)
        .exponential_backoff(Duration::from_millis(10))
        .max_delay(Duration::from_secs(2))
}
