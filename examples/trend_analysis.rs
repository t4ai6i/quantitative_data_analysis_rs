use anyhow::Result;
use chrono::NaiveDate;
use quantitative_data_analysis_rs::controller::trend_analysis_controller::TrendAnalysisController;
use quantitative_data_analysis_rs::infrastructure::data_format::DataFormat;
use quantitative_data_analysis_rs::infrastructure::file_system::FileSystem;
use quantitative_data_analysis_rs::infrastructure::yahoo_finance_api::YahooFinanceAPI;
use quantitative_data_analysis_rs::presenter::trend_analysis_presenter::chart::Chart;
use quantitative_data_analysis_rs::presenter::trend_analysis_presenter::{
    DisplayCrossPattern, TrendAnalysisResponse,
};
use quantitative_data_analysis_rs::use_case::interactor::trend_analysis_interactor::TrendAnalysisInteractor;
use std::path::PathBuf;
use tokio::fs::write;
use yahoo_finance_api::YahooConnector;

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: Create SMA from zero-based data, like download via yahoo_finance2_api with term.
    // TODO: Create SMA from a day of end, like get daily stock data.
    // TODO: Consider DB schema
    // TODO: Register data

    let code = "8473.T";
    let file_path = PathBuf::from(format!("./assets/{}.csv", code));
    let stock_repository = FileSystem::new(DataFormat::CSV {
        has_headers: true,
        file_path,
    });
    let file_path = PathBuf::from("./assets/companies.json");
    let company_repository = FileSystem::new(DataFormat::JSON { file_path });
    let interactor = TrendAnalysisInteractor::new(&stock_repository, &company_repository);
    let presenter = Chart::new("chalk", 1280.0, 720.0);
    // let presenter = trend_analysis_presenter::SummaryText::new();
    // let presenter = trend_analysis_presenter::DetailText::new();
    let controller = TrendAnalysisController::new(&interactor, &presenter);
    let start_date = NaiveDate::default();
    let end_date = NaiveDate::default();
    let TrendAnalysisResponse::Chart {
        body,
        chance_rate,
        latest_chance,
        ..
    } = controller
        .analyze::<5>(code, start_date, end_date, DisplayCrossPattern::Both)
        .await?;
    dbg!(latest_chance, chance_rate);
    write("./examples/8473.T.from_csv.svg", &body).await?;
    assert_eq!(include_str!("../assets/8473.T.from_csv.svg"), &body);

    let provider = YahooConnector::new();
    let stock_repository = YahooFinanceAPI::new(&provider, DataFormat::YahooFinanceAPI);
    let company_repository = YahooFinanceAPI::new(&provider, DataFormat::YahooFinanceAPI);
    let interactor = TrendAnalysisInteractor::new(&stock_repository, &company_repository);
    let controller = TrendAnalysisController::new(&interactor, &presenter);
    let start_date = NaiveDate::from_ymd_opt(2022, 9, 9).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2023, 9, 8).unwrap();
    let TrendAnalysisResponse::Chart { body, .. } = controller
        .analyze::<5>(code, start_date, end_date, DisplayCrossPattern::Both)
        .await?;
    write("./examples/8473.T.from_yfapi.svg", &body).await?;
    assert_eq!(include_str!("../assets/8473.T.from_yfapi.svg"), &body);

    Ok(())
}
