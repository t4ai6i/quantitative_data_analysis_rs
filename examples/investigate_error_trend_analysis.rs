use anyhow::Result;
use chrono::{Days, Local};
use quantitative_data_analysis_rs::controller::trend_analysis_controller::TrendAnalysisController;
use quantitative_data_analysis_rs::controller::trend_summary_controller::TrendSummaryController;
use quantitative_data_analysis_rs::infrastructure::data_format::DataFormat;
use quantitative_data_analysis_rs::infrastructure::yahoo_finance_api::YahooFinanceAPI;
use quantitative_data_analysis_rs::presenter::trend_analysis_presenter;
use quantitative_data_analysis_rs::presenter::trend_analysis_presenter::{
    DisplayCrossPattern, TrendAnalysisResponse,
};
use quantitative_data_analysis_rs::presenter::trend_summary_presenter;
use quantitative_data_analysis_rs::presenter::trend_summary_presenter::TrendSummaryResponse;
use quantitative_data_analysis_rs::use_case::interactor::trend_analysis_interactor::TrendAnalysisInteractor;
use quantitative_data_analysis_rs::use_case::interactor::trend_summary_interactor::TrendSummaryInteractor;
use tokio::fs::write;
use yahoo_finance_api::YahooConnector;

const AFTER_5DAYS: usize = 5;
const FOR_7DAYS: usize = 7;

#[tokio::main]
async fn main() -> Result<()> {
    let display_cross_pattern = DisplayCrossPattern::All;
    let code = "9223";
    let market = "T";

    let provider = YahooConnector::new();
    let stock_repository = YahooFinanceAPI::new(&provider, DataFormat::YahooFinanceAPI);
    let company_repository = YahooFinanceAPI::new(&provider, DataFormat::YahooFinanceAPI);
    let interactor = TrendAnalysisInteractor::new(&stock_repository, &company_repository);
    let presenter = trend_analysis_presenter::chart::Chart::new("chalk", 1280.0, 720.0);
    let controller = TrendAnalysisController::new(&interactor, &presenter);
    let end_date = Local::now().date_naive();
    let days_ago = Days::new(365);
    let start_date = end_date.checked_sub_days(days_ago).unwrap();
    let trend_analysis_response = controller
        .analyze::<AFTER_5DAYS, FOR_7DAYS>(
            code,
            market,
            start_date,
            end_date,
            display_cross_pattern,
        )
        .await?;
    let body = trend_analysis_response.clone();
    let TrendAnalysisResponse::Chart { body, .. } = body;
    write("./examples/9223.T.from_yfapi.svg", &body).await?;

    let vec_trend_analysis_response = vec![trend_analysis_response.clone()];
    let interactor = TrendSummaryInteractor::new();
    let presenter = trend_summary_presenter::chart::Chart::new("chalk", 1280.0, 720.0);
    let controller = TrendSummaryController::new(&interactor, &presenter);
    if let TrendSummaryResponse::Chart { body } = controller
        .analyze(vec_trend_analysis_response, display_cross_pattern)
        .await?
    {
        write("./examples/9223_trend_summary.svg", &body).await?;
    };

    let vec_trend_analysis_response = vec![trend_analysis_response.clone()];
    let presenter = trend_summary_presenter::json::JSON::new();
    let controller = TrendSummaryController::new(&interactor, &presenter);
    if let TrendSummaryResponse::JSON { data } = controller
        .analyze(vec_trend_analysis_response, display_cross_pattern)
        .await?
    {
        let json_str = serde_json::to_string(&data)?;
        dbg!(&json_str);
    };

    Ok(())
}
