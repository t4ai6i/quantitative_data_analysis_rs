use csv::{ReaderBuilder, WriterBuilder};
use quantitative_data_analysis_rs::domain::models::statement::model;
use quantitative_data_analysis_rs::domain::repositories::statement::queries::get_statement;
use quantitative_data_analysis_rs::domain::repositories::statement::repository::Statement;
use quantitative_data_analysis_rs::infrastructure::jquants_api::JQuantsAPI;
use quantitative_data_analysis_rs::infrastructure::repositories::statement::structures::internal::tsv as statement_tsv;
use quantitative_data_analysis_rs::shared::jquants_api::setup::Setup;
use serde::Deserialize;
use tokio::fs::{read, write};

const COMPANIES_PATH: &str = "./examples/sample_companies.tsv";
const STATEMENTS_PATH: &str = "./assets/statements.tsv";

#[derive(Debug, Deserialize)]
struct SampleCompany {
    code: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = Setup::run()?;
    let jquants_api = JQuantsAPI::new(token)?;

    let companies = read_sample_companies().await?;

    let mut writer = WriterBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_writer(vec![]);

    for company in companies {
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
            }
            Err(error) => {
                eprintln!("failed to fetch statements for code {}: {}", code, error);
            }
        }
    }

    writer.flush()?;
    write(STATEMENTS_PATH, writer.get_ref()).await?;
    Ok(())
}

async fn read_sample_companies() -> anyhow::Result<Vec<SampleCompany>> {
    let buffer = read(COMPANIES_PATH).await?;
    let mut reader = ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_reader(buffer.as_slice());

    let mut companies = Vec::new();
    for row in reader.deserialize() {
        companies.push(row?);
    }

    Ok(companies)
}
