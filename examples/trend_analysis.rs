use anyhow::Result;
use chrono::NaiveDate;
use quantitative_data_analysis_rs::controller::trend_analysis_controller::TrendAnalysisController;
use quantitative_data_analysis_rs::controller::trend_summary_controller::TrendSummaryController;
use quantitative_data_analysis_rs::infrastructure::data_format::DataFormat;
use quantitative_data_analysis_rs::infrastructure::jquants_api::JQuantsAPI;
use quantitative_data_analysis_rs::presenter::trend_analysis_presenter;
use quantitative_data_analysis_rs::presenter::trend_analysis_presenter::{
    DisplayCrossPattern, TrendAnalysisResponse,
};
use quantitative_data_analysis_rs::presenter::trend_summary_presenter;
use quantitative_data_analysis_rs::presenter::trend_summary_presenter::TrendSummaryResponse;
use quantitative_data_analysis_rs::presenter::view_model::analysis::Analysis;
use quantitative_data_analysis_rs::use_case::interactor::trend_analysis_interactor::TrendAnalysisInteractor;
use quantitative_data_analysis_rs::use_case::interactor::trend_summary_interactor::TrendSummaryInteractor;
use quantitative_data_analysis_rs::utils::jquants_api::setup::Setup;
use tokio::fs::write;

const AFTER_5DAYS: usize = 5;
const FOR_7DAYS: usize = 7;
const MARUBOZU_MIN_RATE: usize = 90;

#[tokio::main]
async fn main() -> Result<()> {
    let display_cross_pattern = DisplayCrossPattern::All;
    let code = "8473";
    let market = "T";
    let token = Setup::run().await?;
    let data_format = DataFormat::JQuantsAPI;
    let repository = JQuantsAPI::new(&token.id_token.value, data_format)?;
    let interactor = TrendAnalysisInteractor::new(&repository, &repository);
    let presenter = trend_analysis_presenter::chart::Chart::new("chalk", 1280.0, 720.0);
    let controller = TrendAnalysisController::new(&interactor, &presenter);
    let start_date = NaiveDate::from_ymd_opt(2022, 9, 9).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2023, 9, 8).unwrap();
    let trend_analysis_response = controller
        .analyze::<AFTER_5DAYS, FOR_7DAYS, MARUBOZU_MIN_RATE>(
            code,
            market,
            start_date,
            end_date,
            display_cross_pattern,
        )
        .await?;
    if let TrendAnalysisResponse::Chart { body, .. } = trend_analysis_response.clone() {
        write("./examples/8473.T.from_jquants_api.svg", &body).await?;
        assert_eq!(include_str!("../assets/8473.T.from_jquants_api.svg"), &body);
    }

    let interactor = TrendSummaryInteractor::new();
    let vec_trend_analysis_response = vec![trend_analysis_response.clone()];
    let presenter = trend_summary_presenter::chart::Chart::new("chalk", 1280.0, 720.0);
    let controller = TrendSummaryController::new(&interactor, &presenter);
    if let TrendSummaryResponse::Chart { body } = controller
        .analyze(vec_trend_analysis_response, display_cross_pattern)
        .await?
    {
        write("./examples/trend_summary.svg", &body).await?;
        assert_eq!(include_str!("../assets/trend_summary.svg"), &body);
    };

    let vec_trend_analysis_response = vec![trend_analysis_response.clone()];
    let presenter = trend_summary_presenter::json::JSON::new();
    let controller = TrendSummaryController::new(&interactor, &presenter);
    if let TrendSummaryResponse::JSON { data } = controller
        .analyze(vec_trend_analysis_response, display_cross_pattern)
        .await?
    {
        let (cross, buy_sell_signal): (Vec<_>, Vec<_>) = data
            .into_iter()
            .map(|e| {
                let Analysis {
                    cross_analysis,
                    buy_sell_signal_analysis,
                } = e;
                (cross_analysis, buy_sell_signal_analysis)
            })
            .unzip();
        let json_str = serde_json::to_string(&cross)?;
        assert_eq!(
            include_str!("../assets/cross_analysis_summary.json"),
            &json_str
        );
        let json_str = serde_json::to_string(&buy_sell_signal)?;
        assert_eq!(
            include_str!("../assets/buy_sell_analysis_summary.json"),
            &json_str
        );
    };

    let code = "9223";
    let market = "T";
    let data_format = DataFormat::JQuantsAPI;
    let repository = JQuantsAPI::new(&token.id_token.value, data_format)?;
    let interactor = TrendAnalysisInteractor::new(&repository, &repository);
    let presenter = trend_analysis_presenter::chart::Chart::new("chalk", 1280.0, 720.0);
    let controller = TrendAnalysisController::new(&interactor, &presenter);
    let start_date = NaiveDate::from_ymd_opt(2023, 12, 25).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2024, 2, 16).unwrap();
    let trend_analysis_response = controller
        .analyze::<AFTER_5DAYS, FOR_7DAYS, MARUBOZU_MIN_RATE>(
            code,
            market,
            start_date,
            end_date,
            display_cross_pattern,
        )
        .await?;
    if let TrendAnalysisResponse::Chart { body, .. } = trend_analysis_response.clone() {
        write("./examples/9223.T.from_jquants_api.svg", &body).await?;
    }

    Ok(())
}
