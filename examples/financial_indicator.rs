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
    let jquants_api = infrastructure::jquants_api::JQuantsAPI::new(token)?;

    // FinancialIndicatorのドメインロジックを実行するInteractorの準備
    let interactor =
        use_case::interactors::financial_indicator::interactor::FinancialIndicator::new(
            &jquants_api,
            &jquants_api,
        );
    // PresenterはJSON型でJSON形式のデータを出力する
    let presenter = presenter::presenters::financial_indicator::response::json::Json;
    // 指定された証券コードの財務指標分析を行う
    let controller = controller::financial_indicator::controller::FinancialIndicator::new(
        &interactor,
        &presenter,
    );
    let financial_indicator = controller
        .analyze("84730", "T", NaiveDate::from_ymd_opt(2025, 8, 27).unwrap())
        .await?;

    // FinancialIndicatorSummaryのドメインロジックを実行するInteractorの準備
    let interactor =
        use_case::interactors::financial_indicator_summary::interactor::FinancialIndicatorSummary;
    // PresenterはJSON型でJSON形式のデータを出力する
    let presenter = presenter::presenters::financial_indicator_summary::response::json::JSON;
    let vec_financial_indicator = vec![financial_indicator];
    let financial_indicators =
        presenter::presenters::financial_indicator::response::FinancialIndicators(
            vec_financial_indicator,
        );
    // FinancialIndicatorの集合からサマリーを出力する
    let controller =
        controller::financial_indicator_summary::controller::FinancialIndicatorSummary::new(
            &interactor,
            &presenter,
        );
    let presenter::presenters::financial_indicator_summary::response::FinancialIndicatorSummary::JSON {
        json_rows
    } = controller.analyze(
        financial_indicators,
    ).await?;

    let vec_financial_indicator = json_rows
        .iter()
        .map(|json_row| {
            let presenter::views::financial_indicator_summary::json::view::JsonRow {
                financial_indicator,
                ..
            } = json_row;
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
