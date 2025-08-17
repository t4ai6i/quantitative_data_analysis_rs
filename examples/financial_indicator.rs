use anyhow::Result;
use chrono::NaiveDate;
use quantitative_data_analysis_rs::shared::jquants_api::setup::Setup;
use quantitative_data_analysis_rs::{controller, infrastructure, presenter, use_case};

#[tokio::main]
async fn main() -> Result<()> {
    // JQUANTS APIのためのトークン準備
    let token = Setup::run().await?;

    // JQUANTS APIを用いたレポジトリの準備
    let jquants_api = infrastructure::jquants_api::JQuantsAPI::new(token.id_token.value)?;

    let interactor =
        use_case::interactors::financial_indicator::interactor::FinancialIndicator::new(
            &jquants_api,
            &jquants_api,
        );
    let presenter = presenter::presenters::financial_indicator::response::json::Json;
    let controller = controller::financial_indicator::controller::FinancialIndicator::new(
        &interactor,
        &presenter,
    );
    let response = controller
        .analyze("84730", "T", NaiveDate::from_ymd_opt(2023, 9, 8).unwrap())
        .await?;
    let presenter::presenters::financial_indicator::response::FinancialIndicator::JSON {
        code,
        market,
        financial_indicator: indicator_analysis,
    } = response;

    todo!()
}
