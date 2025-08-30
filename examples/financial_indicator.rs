use anyhow::Result;
use chrono::NaiveDate;
use pretty_assertions::assert_eq;

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
    let financial_indicator = controller
        .analyze("84730", "T", NaiveDate::from_ymd_opt(2025, 8, 27).unwrap())
        .await?;

    let interactor =
        use_case::interactors::financial_indicator_summary::interactor::FinancialIndicatorSummary;
    let vec_financial_indicator = vec![financial_indicator];
    let financial_indicators =
        presenter::presenters::financial_indicator::response::FinancialIndicators(
            vec_financial_indicator,
        );
    let presenter = presenter::presenters::financial_indicator_summary::response::json::JSON;
    let controller =
        controller::financial_indicator_summary::controller::FinancialIndicatorSummary::new(
            &interactor,
            &presenter,
        );
    let presenter::presenters::financial_indicator_summary::response::FinancialIndicatorSummary::JSON {
        rows
    } = controller.analyze(
        financial_indicators,
    ).await?;

    let vec_financial_indicator = rows
        .iter()
        .map(|row| {
            let presenter::views::financial_indicator_summary::json::view::JsonRow {
                financial_indicator,
                ..
            } = row;
            financial_indicator
        })
        .collect::<Vec<_>>();

    let json_str = serde_json::to_string_pretty(&vec_financial_indicator)?;
    assert_eq!(
        include_str!("../assets/8473.T.financial_indicator_summary.json"),
        &json_str
    );

    Ok(())
}
