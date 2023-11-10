use anyhow::Result;
use chrono::NaiveDate;
use quantitative_data_analysis_rs::controller::trend_analysis_controller::TrendAnalysisController;
use quantitative_data_analysis_rs::infrastructure::vec_stock_repository;
use quantitative_data_analysis_rs::infrastructure::vec_stock_repository::data_format::DataFormatType;
use quantitative_data_analysis_rs::presenter::trend_analysis_presenter::chart::Chart;
use quantitative_data_analysis_rs::presenter::trend_analysis_presenter::TrendAnalysisResponse;
use quantitative_data_analysis_rs::use_case::interactor::trend_analysis_interactor::TrendAnalysisInteractor;
use std::fs::write;
use std::path::PathBuf;
use vec_stock_repository::file_system::FileSystem;

fn main() -> Result<()> {
    // TODO: Create SMA from zero-based data, like download via yahoo_finance2_api with term.
    // TODO: Create SMA from a day of end, like get daily stock data.
    // TODO: Consider DB schema
    // TODO: Register data

    let repository = FileSystem::new(PathBuf::from("./assets/"));
    // let repository = vec_stock_repository::InMemory::new();
    // let repository = vec_stock_repository::YahooFinanceAPI::new();
    let presenter = Chart::new("chalk", 1280.0, 720.0);
    // let presenter = trend_analysis_presenter::SummaryText::new();
    // let presenter = trend_analysis_presenter::DetailText::new();
    let interactor = TrendAnalysisInteractor::new(&repository);
    let controller = TrendAnalysisController::new(&interactor, &presenter);

    let code = "8473.T";
    let start_date = NaiveDate::default();
    let end_date = NaiveDate::default();
    let data_format_type = DataFormatType::CSVFormat { has_headers: true };
    let TrendAnalysisResponse::Chart { body } =
        controller.analyze(code, start_date, end_date, data_format_type)?;
    assert_eq!(include_str!("../assets/8473.T.svg"), &body);
    write("./examples/trend_analysis.svg", body)?;
    Ok(())
}
