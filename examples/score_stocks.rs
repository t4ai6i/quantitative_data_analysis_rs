use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use quantitative_data_analysis_rs::domain::models::screening::scoring::ValueScorePolicy;
use quantitative_data_analysis_rs::presenter::presenters::fetch_scoring_data::output::FetchScoringData;
use quantitative_data_analysis_rs::presenter::presenters::score_stock::output::ScoreStatus;
use quantitative_data_analysis_rs::use_case::interactors::score_stock::interactor;
use quantitative_data_analysis_rs::{controller, presenter};
use std::env;
use std::fs::File;
use std::io::Write;
use tokio::fs::remove_file;

const FETCH_SCORING_DATA_JSON: &str = include_str!("fetch_scoring_data.json");
const PARTIAL_OUTPUT_PATH: &str = "./examples/score_stocks.json.partial";
const OUTPUT_PATH: &str = "./examples/score_stocks.json";

#[tokio::main]
async fn main() -> Result<()> {
    clear_partial_output().await?;

    let preset_name = env::args()
        .nth(1)
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| anyhow!("usage: score_stocks <preset_name>"))?;

    let policy = ValueScorePolicy::try_from(preset_name.trim()).with_context(|| {
        format!(
            "unsupported preset name: {}. available presets: standard, value, dividend",
            preset_name
        )
    })?;

    let data: Vec<FetchScoringData> = serde_json::from_str(FETCH_SCORING_DATA_JSON)
        .with_context(|| "failed to parse fetch_scoring_data.json")?;

    let interactor = interactor::ScoreStock::new();
    let presenter = presenter::presenters::score_stock::presenter::Json;
    let controller = controller::score_stock::controller::ScoreStock::new(&interactor, &presenter);

    let mut partial_file = File::create(PARTIAL_OUTPUT_PATH)
        .with_context(|| format!("failed to create partial output: {}", PARTIAL_OUTPUT_PATH))?;
    partial_file.write_all(b"[\n")?;

    for (index, item) in data.into_iter().enumerate() {
        let result = controller.score(item, policy, Utc::now()).await?;

        if index > 0 {
            partial_file.write_all(b",\n")?;
        }
        serde_json::to_writer(&mut partial_file, &result)?;

        if result.status != ScoreStatus::Ok {
            eprintln!(
                "failed: code={} status={:?} error_type={:?}",
                result.code, result.status, result.error_type
            );
        }
    }

    partial_file.write_all(b"\n]\n")?;
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
