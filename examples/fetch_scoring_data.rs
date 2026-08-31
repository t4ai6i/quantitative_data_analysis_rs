use anyhow::Result;
use bytes::Bytes;
use chrono::Utc;
use chrono_tz::Asia::Tokyo;
use quantitative_data_analysis_rs::domain::repositories::company::repository::Company;
use quantitative_data_analysis_rs::infrastructure::dsv::Dsv;
use quantitative_data_analysis_rs::infrastructure::jquants_api::JQuantsAPI;
use quantitative_data_analysis_rs::infrastructure::repositories::company;
use quantitative_data_analysis_rs::infrastructure::repositories::statement;
use quantitative_data_analysis_rs::presenter::presenters::fetch_scoring_data::output::FetchStatus;
use quantitative_data_analysis_rs::shared::jquants_api::setup::Setup;
use quantitative_data_analysis_rs::use_case::interactors::fetch_scoring_data::interactor;
use quantitative_data_analysis_rs::{controller, presenter};
use std::fs::File;
use std::io::Write;
use std::time::Instant;
use tokio::fs::remove_file;
use tokio::time::{Duration, sleep};
use quantitative_data_analysis_rs::domain::repositories::market_calendar::queries::get_previous_business_day::Query;
use quantitative_data_analysis_rs::domain::repositories::market_calendar::repository::MarketCalendar;

const COMPANIES_TSV: &[u8] = include_bytes!("companies.tsv");
const STATEMENTS_TSV: &[u8] = include_bytes!("statements.tsv");
const PARTIAL_OUTPUT_PATH: &str = "./examples/fetch_scoring_data.json.partial";
const OUTPUT_PATH: &str = "./examples/fetch_scoring_data.json";
const PROGRESS_STEP: usize = 50;
const REQUEST_INTERVAL_MILLIS: u64 = 1_100;

#[tokio::main]
async fn main() -> Result<()> {
    clear_partial_output().await?;

    let token = Setup::run()?;
    let jquants_api = JQuantsAPI::new(token)?;
    let company_tsv = Dsv::<company::structures::internal::tsv::Structure>::new(
        false,
        Bytes::from_static(COMPANIES_TSV),
    );
    let statement_tsv = Dsv::<statement::structures::internal::tsv::Structure>::new(
        true,
        Bytes::from_static(STATEMENTS_TSV),
    );
    let interactor = interactor::FetchScoringData::new(&company_tsv, &jquants_api, &statement_tsv);
    let presenter = presenter::presenters::fetch_scoring_data::presenter::Json;
    let controller =
        controller::fetch_scoring_data::controller::FetchScoringData::new(&interactor, &presenter);
    let now = Utc::now();
    let target_date = now.with_timezone(&Tokyo).date_naive();
    let query = Query { target_date };
    let target_date = jquants_api.get_previous_business_day(&query).await?;
    let fetched_at = now;

    let companies = company_tsv.get_companies().await?;
    let companies = companies.domestic_prime_standard_growth_companies();
    let total = companies.len();
    let started_at = Instant::now();
    let mut success_count = 0usize;
    let mut failure_count = 0usize;

    let mut partial_file = File::create(PARTIAL_OUTPUT_PATH)?;
    partial_file.write_all(b"[\n")?;

    for (index, company) in companies.into_iter().enumerate() {
        let result = controller
            .fetch(company.code, target_date, fetched_at)
            .await?;

        if index > 0 {
            partial_file.write_all(b",\n")?;
        }
        serde_json::to_writer(&mut partial_file, &result)?;
        if result.status == FetchStatus::Ok {
            success_count += 1;
        } else {
            failure_count += 1;
            eprintln!(
                "failed: code={} status={:?} error_type={:?}",
                result.code, result.status, result.error_type
            );
        }

        if (index + 1) % PROGRESS_STEP == 0 || index + 1 == total {
            let elapsed = started_at.elapsed();
            let average_per_item = elapsed / (index as u32 + 1);
            let remaining = total.saturating_sub(index + 1) as u32;
            let eta = average_per_item * remaining;
            eprintln!(
                "progress: {}/{} success={} failed={} elapsed={:?} eta={:?}",
                index + 1,
                total,
                success_count,
                failure_count,
                elapsed,
                eta,
            );
        }

        partial_file.flush()?;
        sleep(Duration::from_millis(REQUEST_INTERVAL_MILLIS)).await;
    }

    partial_file.write_all(b"\n]\n")?;
    partial_file.flush()?;
    drop(partial_file);
    std::fs::rename(PARTIAL_OUTPUT_PATH, OUTPUT_PATH)?;
    Ok(())
}

async fn clear_partial_output() -> Result<()> {
    match remove_file(PARTIAL_OUTPUT_PATH).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}
