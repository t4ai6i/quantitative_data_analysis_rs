use anyhow::Result;
use chrono::NaiveDate;
use quantitative_data_analysis_rs::controller::trend_analysis_controller::TrendAnalysisController;
use quantitative_data_analysis_rs::infrastructure::company_repository::data_format::DataFormat as CPDF;
use quantitative_data_analysis_rs::infrastructure::file_system::FileSystem;
use quantitative_data_analysis_rs::infrastructure::stock_repository::data_format::DataFormat as SRDF;
use quantitative_data_analysis_rs::infrastructure::yahoo_finance_api::YahooFinanceAPI;
use quantitative_data_analysis_rs::presenter::trend_analysis_presenter::chart::Chart;
use quantitative_data_analysis_rs::presenter::trend_analysis_presenter::TrendAnalysisResponse;
use quantitative_data_analysis_rs::use_case::interactor::trend_analysis_interactor::TrendAnalysisInteractor;
use std::fs::write;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: Create SMA from zero-based data, like download via yahoo_finance2_api with term.
    // TODO: Create SMA from a day of end, like get daily stock data.
    // TODO: Consider DB schema
    // TODO: Register data

    let stock_repository = FileSystem::new(PathBuf::from("./assets/"));
    let company_repository = FileSystem::new(PathBuf::from("./assets/"));
    let presenter = Chart::new("chalk", 1280.0, 720.0);
    // let presenter = trend_analysis_presenter::SummaryText::new();
    // let presenter = trend_analysis_presenter::DetailText::new();
    let interactor = TrendAnalysisInteractor::new(&stock_repository, &company_repository);
    let controller = TrendAnalysisController::new(&interactor, &presenter);

    let code = "8473.T";
    let start_date = NaiveDate::default();
    let end_date = NaiveDate::default();
    let sr_data_format = SRDF::CSV { has_headers: true };
    let cp_data_format = CPDF::Json;
    let TrendAnalysisResponse::Chart { body } = controller
        .analyze::<5>(code, start_date, end_date, sr_data_format, cp_data_format)
        .await?;
    write("./examples/8473.T.from_csv.svg", &body)?;
    assert_eq!(include_str!("../assets/8473.T.from_csv.svg"), &body);

    let stock_repository = YahooFinanceAPI::new();
    let interactor = TrendAnalysisInteractor::new(&stock_repository, &company_repository);
    let controller = TrendAnalysisController::new(&interactor, &presenter);
    let start_date = NaiveDate::from_ymd_opt(2022, 9, 9).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2023, 9, 8).unwrap();
    let sr_data_format = SRDF::YahooFinanceAPI;
    let cp_data_format = CPDF::Json;
    let TrendAnalysisResponse::Chart { body } = controller
        .analyze::<5>(code, start_date, end_date, sr_data_format, cp_data_format)
        .await?;
    write("./examples/8473.T.from_yfapi.svg", &body)?;
    assert_eq!(include_str!("../assets/8473.T.from_yfapi.svg"), &body);

    Ok(())
}
