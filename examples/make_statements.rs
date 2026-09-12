use bytes::Bytes;
use csv::WriterBuilder;
use quantitative_data_analysis_rs::domain::models::statement::model;
use quantitative_data_analysis_rs::domain::repositories::company::repository::Company;
use quantitative_data_analysis_rs::domain::repositories::statement::queries::get_statement;
use quantitative_data_analysis_rs::domain::repositories::statement::repository::Statement;
use quantitative_data_analysis_rs::infrastructure::dsv::Dsv;
use quantitative_data_analysis_rs::infrastructure::jquants_api::JQuantsAPI;
use quantitative_data_analysis_rs::infrastructure::repositories::company::structures::internal::tsv as company_tsv;
use quantitative_data_analysis_rs::infrastructure::repositories::statement::structures::internal::tsv as statement_tsv;
use quantitative_data_analysis_rs::shared::jquants_api::setup::Setup;
use std::fs::File;
use std::time::Instant;
use tokio::fs::remove_file;
use tokio::time::{Duration, sleep};

const COMPANIES_TSV: &[u8] = include_bytes!("../assets/companies.tsv");
const PARTIAL_STATEMENTS_PATH: &str = "./examples/statements.tsv.partial";
const STATEMENTS_PATH: &str = "./examples/statements.tsv";
const PROGRESS_STEP: usize = 50;
const REQUEST_INTERVAL_MILLIS: u64 = 1_100;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    clear_partial_output().await?;

    let token = Setup::run()?;
    let jquants_api = JQuantsAPI::new(token)?;

    let company_tsv = Dsv::<company_tsv::Structure>::new(false, Bytes::from_static(COMPANIES_TSV));
    let companies = company_tsv.get_companies().await?;
    let companies = companies.domestic_prime_standard_growth_companies();

    let partial_file = File::create(PARTIAL_STATEMENTS_PATH)?;
    let mut writer = WriterBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_writer(partial_file);
    let started_at = Instant::now();
    let total = companies.len();
    let mut success_count = 0usize;
    let mut failure_count = 0usize;

    for (index, company) in companies.into_iter().enumerate() {
        let code = company.code;
        let query = get_statement::Query {
            code: code.as_str(),
        };

        match jquants_api.get_row_full_year_statements(&query).await {
            Ok(mut rows) => {
                rows.sort_by(|left, right| right.disclosed_date.cmp(&left.disclosed_date));
                for row in rows {
                    let statement = model::Statement::try_from(row)?;
                    writer.serialize(statement_tsv::Structure::from(statement))?;
                }
                success_count += 1;
            }
            Err(error) => {
                eprintln!("failed to fetch statements for code {}: {}", code, error);
                failure_count += 1;
            }
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

        writer.flush()?;
        sleep(Duration::from_millis(REQUEST_INTERVAL_MILLIS)).await;
    }

    writer.flush()?;
    drop(writer);
    std::fs::rename(PARTIAL_STATEMENTS_PATH, STATEMENTS_PATH)?;
    Ok(())
}

async fn clear_partial_output() -> anyhow::Result<()> {
    match remove_file(PARTIAL_STATEMENTS_PATH).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}
