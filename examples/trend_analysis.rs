use anyhow::Result;
use chrono::NaiveDate;
use quantitative_data_analysis_rs::controller::trend_analysis_controller::TrendAnalysisController;
use quantitative_data_analysis_rs::controller::trend_summary_controller::TrendSummaryController;
use quantitative_data_analysis_rs::infrastructure::data_format::DataFormat;
use quantitative_data_analysis_rs::infrastructure::file_system::FileSystem;
use quantitative_data_analysis_rs::infrastructure::yahoo_finance_api::YahooFinanceAPI;
use quantitative_data_analysis_rs::presenter::trend_analysis_presenter;
use quantitative_data_analysis_rs::presenter::trend_analysis_presenter::{
    DisplayCrossPattern, TrendAnalysisResponse,
};
use quantitative_data_analysis_rs::presenter::trend_summary_presenter;
use quantitative_data_analysis_rs::presenter::trend_summary_presenter::{
    AnalysisJSON, TrendSummaryResponse,
};
use quantitative_data_analysis_rs::use_case::interactor::trend_analysis_interactor::TrendAnalysisInteractor;
use quantitative_data_analysis_rs::use_case::interactor::trend_summary_interactor::TrendSummaryInteractor;
use std::path::PathBuf;
use tokio::fs::write;
use yahoo_finance_api::YahooConnector;

const AFTER_5DAYS: usize = 5;
const FOR_7DAYS: usize = 7;

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: Create SMA from zero-based data, like download via yahoo_finance2_api with term.
    // TODO: Create SMA from a day of end, like get daily stock data.
    // TODO: Consider DB schema
    // TODO: Register data

    let display_cross_pattern = DisplayCrossPattern::All;
    let code = "8473";
    let market = "T";
    let file_path = PathBuf::from("./assets/8473.T.csv");
    let stock_repository = FileSystem::new(DataFormat::CSV {
        has_headers: true,
        file_path,
    });
    let file_path = PathBuf::from("./assets/companies.json");
    let company_repository = FileSystem::new(DataFormat::JSON { file_path });
    let interactor = TrendAnalysisInteractor::new(&stock_repository, &company_repository);
    let presenter = trend_analysis_presenter::chart::Chart::new("chalk", 1280.0, 720.0);
    let controller = TrendAnalysisController::new(&interactor, &presenter);
    let start_date = NaiveDate::default();
    let end_date = NaiveDate::default();
    let TrendAnalysisResponse::Chart { body, .. } = controller
        .analyze::<AFTER_5DAYS, FOR_7DAYS>(
            code,
            market,
            start_date,
            end_date,
            display_cross_pattern,
        )
        .await?;
    write("./examples/8473.T.from_csv.svg", &body).await?;
    assert_eq!(include_str!("../assets/8473.T.from_csv.svg"), &body);

    let provider = YahooConnector::new();
    let stock_repository = YahooFinanceAPI::new(&provider, DataFormat::YahooFinanceAPI);
    let company_repository = YahooFinanceAPI::new(&provider, DataFormat::YahooFinanceAPI);
    let interactor = TrendAnalysisInteractor::new(&stock_repository, &company_repository);
    let controller = TrendAnalysisController::new(&interactor, &presenter);
    let start_date = NaiveDate::from_ymd_opt(2022, 9, 9).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2023, 9, 8).unwrap();
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
    write("./examples/8473.T.from_yfapi.svg", &body).await?;
    assert_eq!(include_str!("../assets/8473.T.from_yfapi.svg"), &body);

    let vec_trend_analysis_response = vec![trend_analysis_response.clone()];
    let interactor = TrendSummaryInteractor::new();
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
                let AnalysisJSON {
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

    Ok(())
}
