use anyhow::Result;
use chrono::NaiveDate;
use quantitative_data_analysis_rs::shared::jquants_api::setup::Setup;
use quantitative_data_analysis_rs::{controller, infrastructure, presenter, use_case};

#[tokio::main]
async fn main() -> Result<()> {
    // JQUANTS APIのためのトークン準備
    let token = Setup::run()?;

    // JQUANTS APIを用いたレポジトリの準備
    let jquants_api = infrastructure::jquants_api::JQuantsAPI::new(token)?;

    // Screeningのユースケース実行気を準備
    let interactor = use_case::interactors::screening::interactor::Screening::new(
        &jquants_api,
        &jquants_api,
        &jquants_api,
    );

    // PresenterはJSON型で結果を出力
    let presenter = presenter::presenters::screening::response::json::Json;

    let controller = controller::screening::controller::Screening::new(&interactor, &presenter);
    let response = controller
        .analyze(
            "0111",
            20,
            Some(30),
            "standard",
            NaiveDate::from_ymd_opt(2025, 8, 27).unwrap(),
        )
        .await?;

    let presenter::presenters::screening::response::Screening::JSON { screening_results } =
        response;
    println!("{:#?}", screening_results);

    Ok(())
}
