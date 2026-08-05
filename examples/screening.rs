use anyhow::Result;
use chrono::NaiveDate;
use quantitative_data_analysis_rs::infrastructure::repositories::company::structures::internal::csv;
use quantitative_data_analysis_rs::shared::jquants_api::setup::Setup;
use quantitative_data_analysis_rs::{controller, infrastructure, presenter, use_case};

const PRESET_NAME: &str = "standard";

// J-Quants eq-master の市場区分コード
const PRIME_MARKET_CODE: &str = "0111";
const STANDARD_MARKET_CODE: &str = "0112";
const GROWTH_MARKET_CODE: &str = "0113";

const DEFAULT_MARKETS: &[&str] = &[PRIME_MARKET_CODE, STANDARD_MARKET_CODE, GROWTH_MARKET_CODE];

const CSV: &[u8] = include_bytes!("../assets/companies.csv");

#[tokio::main]
async fn main() -> Result<()> {
    // JQUANTS APIのためのトークン準備
    let token = Setup::run()?;

    // JQUANTS APIを用いたレポジトリの準備
    let jquants_api = infrastructure::jquants_api::JQuantsAPI::new(token)?;
    // DSVを用いたレポジトリの準備
    let dsv = infrastructure::dsv::Dsv::<csv::Structure>::new(false, bytes::Bytes::from(CSV));

    // Screeningのユースケース実行気を準備
    let interactor = use_case::interactors::screening::interactor::Screening::new(
        &jquants_api,
        &jquants_api,
        &dsv,
    );

    // PresenterはJSON型で結果を出力
    let presenter = presenter::presenters::screening::response::json::Json;

    let controller = controller::screening::controller::Screening::new(&interactor, &presenter);
    let response = controller
        .analyze(
            DEFAULT_MARKETS
                .iter()
                .map(|market| (*market).to_string())
                .collect(),
            20,
            Some(30),
            PRESET_NAME,
            NaiveDate::from_ymd_opt(2025, 8, 27).unwrap(),
        )
        .await?;

    let presenter::presenters::screening::response::Screening::JSON { screening_results } =
        response;
    println!("{:#?}", screening_results);

    Ok(())
}
