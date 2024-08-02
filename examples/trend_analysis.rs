use anyhow::Result;
use chrono::NaiveDate;
use itertools::{Itertools, multiunzip};
use tokio::fs::write;

use quantitative_data_analysis_rs::controller::trend_analysis_controller::TrendAnalysisController;
use quantitative_data_analysis_rs::controller::trend_summary_controller::TrendSummaryController;
use quantitative_data_analysis_rs::infrastructure::data_format::DataFormat;
use quantitative_data_analysis_rs::infrastructure::jquants_api::JQuantsAPI;
use quantitative_data_analysis_rs::presenter::display_cross_pattern::DisplayCrossPattern;
use quantitative_data_analysis_rs::presenter::trend_analysis_presenter;
use quantitative_data_analysis_rs::presenter::trend_analysis_presenter::TrendAnalysisResponse;
use quantitative_data_analysis_rs::presenter::trend_summary_presenter;
use quantitative_data_analysis_rs::presenter::trend_summary_presenter::TrendSummaryResponse;
use quantitative_data_analysis_rs::presenter::view_model::analysis::Analysis;
use quantitative_data_analysis_rs::use_case::interactor::trend_analysis_interactor::TrendAnalysisInteractor;
use quantitative_data_analysis_rs::use_case::interactor::trend_summary_interactor::TrendSummaryInteractor;
use quantitative_data_analysis_rs::utils::jquants_api::setup::Setup;

const AFTER_5DAYS: usize = 5;
const FOR_7DAYS: usize = 7;
const MARUBOZU_MIN_RATE: usize = 90;
const DATE_FORMAT: &str = "%Y/%m/%d";

#[tokio::main]
async fn main() -> Result<()> {
    // JQUANTS APIのためのトークン準備
    let token = Setup::run().await?;

    // JQUANTS APIを用いたレポジトリの準備
    let data_format = DataFormat::JQuantsAPI;
    let repository = JQuantsAPI::new(&token.id_token.value, data_format)?;

    // StockRepositoryとCompanyRepositoryは、JQuantsAPIを用いる
    let interactor = TrendAnalysisInteractor::new(&repository, &repository);
    // PresenterはChart型でSVG形式の画像データを出力する
    let presenter =
        trend_analysis_presenter::chart::Chart::new("chalk", 2560.0, 720.0, DATE_FORMAT);
    // 指定された証券コードのトレンド解析を行う
    let controller = TrendAnalysisController::new(&interactor, &presenter);
    let trend_analysis_response = controller
        .analyze::<AFTER_5DAYS, FOR_7DAYS, MARUBOZU_MIN_RATE>(
            "8473",
            "T",
            NaiveDate::from_ymd_opt(2022, 9, 9).unwrap(),
            NaiveDate::from_ymd_opt(2023, 9, 8).unwrap(),
            DisplayCrossPattern::All,
        )
        .await?;
    if let TrendAnalysisResponse::Chart { ref body, .. } = trend_analysis_response {
        write("./examples/8473.T.from_jquants_api.svg", body).await?;
        assert_eq!(include_str!("../assets/8473.T.from_jquants_api.svg"), body);
    }

    // このexampleではひとつの証券コードだが、運用ではJQuantsAPIで取得できる全証券コード毎のトレンド解析結果のサマリーを出力する
    let interactor = TrendSummaryInteractor;
    let vec_trend_analysis_response = vec![trend_analysis_response];
    // PresenterはChart型でSVG形式の画像データを出力する
    let presenter = trend_summary_presenter::chart::Chart::new("chalk", 1280.0, 720.0);
    let controller = TrendSummaryController::new(&interactor, &presenter);
    if let TrendSummaryResponse::Chart { body } = controller
        .analyze(vec_trend_analysis_response, DisplayCrossPattern::All)
        .await?
    {
        write("./examples/trend_summary.svg", &body).await?;
        assert_eq!(include_str!("../assets/trend_summary.svg"), &body);
    };

    // 運用では、NocoDBで取り扱えるJSON形式でトレンド解析とサマリーを出力する
    let interactor = TrendAnalysisInteractor::new(&repository, &repository);
    // PresenterはJSON型でJSON形式のデータを出力する
    let presenter = trend_analysis_presenter::json::JSON;
    let controller = TrendAnalysisController::new(&interactor, &presenter);
    let trend_analysis_response = controller
        .analyze::<AFTER_5DAYS, FOR_7DAYS, MARUBOZU_MIN_RATE>(
            "8473",
            "T",
            NaiveDate::from_ymd_opt(2022, 9, 9).unwrap(),
            NaiveDate::from_ymd_opt(2023, 9, 8).unwrap(),
            DisplayCrossPattern::All,
        )
        .await?;

    let interactor = TrendSummaryInteractor;
    let vec_trend_analysis_response = vec![trend_analysis_response];
    // PresenterはJSON型でJSON形式のデータを出力する
    let presenter = trend_summary_presenter::json::JSON;
    let controller = TrendSummaryController::new(&interactor, &presenter);
    if let TrendSummaryResponse::JSON { data } = controller
        .analyze(vec_trend_analysis_response, DisplayCrossPattern::All)
        .await?
    {
        let vec = data
            .into_iter()
            .map(|e| {
                let Analysis {
                    cross_analysis,
                    ecp1_analysis,
                    ecp2_buy_golden,
                    ecp2_sell_dead,
                    ..
                } = e;
                (
                    cross_analysis,
                    ecp1_analysis,
                    ecp2_buy_golden,
                    ecp2_sell_dead,
                )
            })
            .collect_vec();
        let (cross_analysis, ecp1_analysis, ecp2_buy_golden, ecp2_sell_dead): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = multiunzip(vec);
        let json_str = serde_json::to_string_pretty(&cross_analysis)?;
        assert_eq!(
            include_str!("../assets/8473.T.cross_analysis.json"),
            &json_str
        );
        let json_str = serde_json::to_string_pretty(&ecp1_analysis)?;
        assert_eq!(
            include_str!("../assets/8473.T.ecp1_analysis.json"),
            &json_str
        );
        let json_str = serde_json::to_string_pretty(&ecp2_buy_golden)?;
        assert_eq!(
            include_str!("../assets/8473.T.ecp2_buy_golden.json"),
            &json_str
        );
        let json_str = serde_json::to_string_pretty(&ecp2_sell_dead)?;
        assert_eq!(
            include_str!("../assets/8473.T.ecp2_sell_dead.json"),
            &json_str
        );
    };

    // エンガルフィンパターン以外（モーニングスター・イブニングスターパターン）の結果が正しく行われたか確認するため、株価データが少ない証券コードを用いる
    let interactor = TrendAnalysisInteractor::new(&repository, &repository);

    let presenter =
        trend_analysis_presenter::chart::Chart::new("chalk", 2560.0, 720.0, DATE_FORMAT);
    let controller = TrendAnalysisController::new(&interactor, &presenter);
    let trend_analysis_response = controller
        .analyze::<AFTER_5DAYS, FOR_7DAYS, MARUBOZU_MIN_RATE>(
            "9223",
            "T",
            NaiveDate::from_ymd_opt(2023, 12, 25).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 16).unwrap(),
            DisplayCrossPattern::All,
        )
        .await?;
    if let TrendAnalysisResponse::Chart { body, .. } = trend_analysis_response {
        write("./examples/9223.T.from_jquants_api.svg", &body).await?;
    }

    let presenter = trend_analysis_presenter::json::JSON;
    let controller = TrendAnalysisController::new(&interactor, &presenter);
    let trend_analysis_response = controller
        .analyze::<AFTER_5DAYS, FOR_7DAYS, MARUBOZU_MIN_RATE>(
            "9223",
            "T",
            NaiveDate::from_ymd_opt(2023, 12, 25).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 16).unwrap(),
            DisplayCrossPattern::All,
        )
        .await?;

    let interactor = TrendSummaryInteractor;
    let vec_trend_analysis_response = vec![trend_analysis_response];
    let presenter = trend_summary_presenter::json::JSON;
    let controller = TrendSummaryController::new(&interactor, &presenter);

    if let TrendSummaryResponse::JSON { data } = controller
        .analyze(vec_trend_analysis_response, DisplayCrossPattern::All)
        .await?
    {
        let vec = data
            .into_iter()
            .map(|e| {
                let Analysis {
                    cross_analysis,
                    ecp1_analysis,
                    ..
                } = e;
                (cross_analysis, ecp1_analysis)
            })
            .collect_vec();
        let (cross_analysis, ecp1_analysis): (Vec<_>, Vec<_>) = vec.iter().cloned().unzip();
        let json_str = serde_json::to_string_pretty(&cross_analysis)?;
        assert_eq!(
            include_str!("../assets/9223.T.cross_analysis.json"),
            &json_str
        );
        let json_str = serde_json::to_string_pretty(&ecp1_analysis)?;
        assert_eq!(
            include_str!("../assets/9223.T.ecp1_analysis.json"),
            &json_str
        );
    };

    Ok(())
}
