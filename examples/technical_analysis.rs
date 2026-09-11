use anyhow::{Context, Result};
use chrono::{Days, Utc};
use quantitative_data_analysis_rs::controller;
use quantitative_data_analysis_rs::domain::repositories::market_calendar::queries::get_previous_business_day::Query as PreviousBusinessDayQuery;
use quantitative_data_analysis_rs::domain::repositories::market_calendar::repository::MarketCalendar;
use quantitative_data_analysis_rs::infrastructure::jquants_api::JQuantsAPI;
use quantitative_data_analysis_rs::presenter::presenters::technical_analysis::presenter::Json;
use quantitative_data_analysis_rs::shared::jquants_api::setup::Setup;
use quantitative_data_analysis_rs::use_case::interactors::technical_analysis::interactor;
use std::fs::File;
use std::io::Write;
use tokio::fs::remove_file;

const PARTIAL_OUTPUT_PATH: &str = "./examples/technical_analysis.json.partial";
const OUTPUT_PATH: &str = "./examples/technical_analysis.json";
const TARGET_CODE: &str = "84730";
const MARUBOZU_BODY_MIN_RATIO: usize = 90;
const MARUBOZU_WICK_MAX_RATIO: usize = 2;
const DOJI_MAX_BODY_RATIO: usize = 5;
const SMA_SHORT_PERIOD: usize = 5;
const SMA_MEDIUM_PERIOD: usize = 25;
const SMA_LONG_PERIOD: usize = 50;
const MACD_FAST_PERIOD: usize = 12;
const MACD_SLOW_PERIOD: usize = 26;
const MACD_SIGNAL_PERIOD: usize = 9;
const RSI_PERIOD: usize = 14;
const BOLLINGER_PERIOD: usize = 20;
const ATR_PERIOD: usize = 14;
const VOLUME_SPIKE_PERIOD: usize = 20;

#[tokio::main]
async fn main() -> Result<()> {
    clear_partial_output().await?;

    let token = Setup::run()?;
    let jquants_api = JQuantsAPI::new(token)?;
    let interactor = interactor::TechnicalAnalysis::<
        _,
        MARUBOZU_BODY_MIN_RATIO,
        MARUBOZU_WICK_MAX_RATIO,
        DOJI_MAX_BODY_RATIO,
        SMA_SHORT_PERIOD,
        SMA_MEDIUM_PERIOD,
        SMA_LONG_PERIOD,
        MACD_FAST_PERIOD,
        MACD_SLOW_PERIOD,
        MACD_SIGNAL_PERIOD,
        RSI_PERIOD,
        BOLLINGER_PERIOD,
        ATR_PERIOD,
        VOLUME_SPIKE_PERIOD,
    >::new(&jquants_api);
    let presenter = Json;
    let controller =
        controller::technical_analysis::controller::TechnicalAnalysis::new(&interactor, &presenter);

    let now = Utc::now();
    let end_date = jquants_api
        .get_previous_business_day(&PreviousBusinessDayQuery {
            target_date: now.date_naive(),
        })
        .await?;
    let start_date = end_date
        .checked_sub_days(Days::new(365))
        .unwrap_or(end_date);
    let analysis_at = now;

    let result = controller
        .analyze(TARGET_CODE, start_date, end_date, analysis_at)
        .await?;

    let mut partial_file = File::create(PARTIAL_OUTPUT_PATH)
        .with_context(|| format!("failed to create partial output: {}", PARTIAL_OUTPUT_PATH))?;
    serde_json::to_writer_pretty(&mut partial_file, &result)?;
    partial_file.write_all(b"\n")?;
    partial_file.flush()?;
    drop(partial_file);

    std::fs::rename(PARTIAL_OUTPUT_PATH, OUTPUT_PATH).with_context(|| {
        format!(
            "failed to finalize output: {} -> {}",
            PARTIAL_OUTPUT_PATH, OUTPUT_PATH
        )
    })?;

    Ok(())
}

async fn clear_partial_output() -> Result<()> {
    match remove_file(PARTIAL_OUTPUT_PATH).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}
