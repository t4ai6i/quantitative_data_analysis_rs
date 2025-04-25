use anyhow::Result;
use chrono::NaiveDate;
use itertools::{multiunzip, Itertools};
use tokio::fs::write;

use quantitative_data_analysis_rs::presenter::macos_pattern_filter::MACOSPatternFilter;
use quantitative_data_analysis_rs::presenter::view_models::analysis::view_model::Analysis;
use quantitative_data_analysis_rs::shared::jquants_api::setup::Setup;
use quantitative_data_analysis_rs::{controller, infrastructure, presenter, use_case};

const AFTER_DAYS_5: usize = 5;
const FROM_END_DAYS_7: isize = 7;
const MARUBOZU_MIN_RATE: usize = 90;
const DATE_FORMAT: &str = "%Y/%m/%d";

#[tokio::main]
async fn main() -> Result<()> {
    // JQUANTS APIのためのトークン準備
    let token = Setup::run().await?;

    // JQUANTS APIを用いたレポジトリの準備
    let data_format = infrastructure::data_format::DataFormat::JQuantsAPI;
    let repository =
        infrastructure::jquants_api::JQuantsAPI::new(token.id_token.value, data_format)?;

    // Stock/Company/Statementのレポジトリは、JQuantsAPIを用いる
    let interactor = use_case::interactors::trend_analysis::interactor::TrendAnalysis::new(
        &repository,
        &repository,
        &repository,
    );
    // PresenterはChart型でSVG形式の画像データを出力する
    let presenter = presenter::presenters::trend_analysis::response::chart::Chart::new(
        "chalk",
        2560.0,
        720.0,
        DATE_FORMAT,
    );
    // 指定された証券コードのトレンド解析を行う
    let controller =
        controller::trend_analysis::controller::TrendAnalysis::new(&interactor, &presenter);
    let trend_analysis_response = controller
        .analyze::<AFTER_DAYS_5, FROM_END_DAYS_7, MARUBOZU_MIN_RATE>(
            "8473",
            "T",
            NaiveDate::from_ymd_opt(2022, 9, 9).unwrap(),
            NaiveDate::from_ymd_opt(2023, 9, 8).unwrap(),
            MACOSPatternFilter::All,
        )
        .await?;
    if let presenter::presenters::trend_analysis::response::TrendAnalysis::Chart {
        ref body, ..
    } = trend_analysis_response
    {
        write("./examples/8473.T.from_jquants_api.svg", body).await?;
        assert_eq!(include_str!("../assets/8473.T.from_jquants_api.svg"), body);
    }

    let interactor = use_case::interactors::trend_analysis::interactor::TrendAnalysis::new(
        &repository,
        &repository,
        &repository,
    );
    // PresenterはJSON型でJSON形式のデータを出力する
    let presenter = presenter::presenters::trend_analysis::response::json::JSON;
    let controller =
        controller::trend_analysis::controller::TrendAnalysis::new(&interactor, &presenter);
    let trend_analysis = controller
        .analyze::<AFTER_DAYS_5, FROM_END_DAYS_7, MARUBOZU_MIN_RATE>(
            "8473",
            "T",
            NaiveDate::from_ymd_opt(2022, 9, 9).unwrap(),
            NaiveDate::from_ymd_opt(2023, 9, 8).unwrap(),
            MACOSPatternFilter::All,
        )
        .await?;

    let interactor = use_case::interactors::trend_summary::interactor::TrendSummary;
    let vec_trend_analysis = vec![trend_analysis];
    // PresenterはJSON型でJSON形式のデータを出力する
    let presenter = presenter::presenters::trend_summary::response::json::JSON;
    let controller =
        controller::trend_summary::controller::TrendSummary::new(&interactor, &presenter);
    if let presenter::presenters::trend_summary::response::TrendSummary::JSON { data } = controller
        .analyze(vec_trend_analysis, MACOSPatternFilter::All)
        .await?
    {
        let vec = data
            .into_iter()
            .map(|e| {
                let Analysis {
                    macos_analysis,
                    trend_reversal_analysis,
                    ecp1_analysis,
                    indicator_analysis,
                    ..
                } = e;
                (
                    macos_analysis,
                    trend_reversal_analysis,
                    ecp1_analysis,
                    indicator_analysis,
                )
            })
            .collect_vec();
        let (macos_analysis, trend_reversal_analysis, ecp1_analysis, indicator_analysis): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = multiunzip(vec);
        let json_str = serde_json::to_string_pretty(&macos_analysis)?;
        assert_eq!(
            include_str!("../assets/8473.T.macos_analysis.json"),
            &json_str
        );
        let json_str = serde_json::to_string_pretty(&trend_reversal_analysis)?;
        assert_eq!(
            include_str!("../assets/8473.T.trend_reversal_analysis.json"),
            &json_str
        );
        let json_str = serde_json::to_string_pretty(&ecp1_analysis)?;
        assert_eq!(
            include_str!("../assets/8473.T.ecp1_analysis.json"),
            &json_str
        );
        let json_str = serde_json::to_string_pretty(&indicator_analysis)?;
        assert_eq!(
            include_str!("../assets/8473.T.indicator_analysis.json"),
            &json_str
        );
    };

    // エンガルフィンパターン以外（モーニングスター・イブニングスターパターン）の結果が正しく行われたか確認するため、株価データが少ない証券コードを用いる
    let interactor = use_case::interactors::trend_analysis::interactor::TrendAnalysis::new(
        &repository,
        &repository,
        &repository,
    );

    let presenter = presenter::presenters::trend_analysis::response::json::JSON;
    let controller =
        controller::trend_analysis::controller::TrendAnalysis::new(&interactor, &presenter);
    let trend_analysis = controller
        .analyze::<AFTER_DAYS_5, FROM_END_DAYS_7, MARUBOZU_MIN_RATE>(
            "9223",
            "T",
            NaiveDate::from_ymd_opt(2023, 12, 25).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 16).unwrap(),
            MACOSPatternFilter::All,
        )
        .await?;

    let interactor = use_case::interactors::trend_summary::interactor::TrendSummary;
    let vec_trend_analysis = vec![trend_analysis];
    let presenter = presenter::presenters::trend_summary::response::json::JSON;
    let controller =
        controller::trend_summary::controller::TrendSummary::new(&interactor, &presenter);

    if let presenter::presenters::trend_summary::response::TrendSummary::JSON { data } = controller
        .analyze(vec_trend_analysis, MACOSPatternFilter::All)
        .await?
    {
        let vec = data
            .into_iter()
            .map(|e| {
                let Analysis {
                    macos_analysis,
                    trend_reversal_analysis,
                    ecp1_analysis,
                    indicator_analysis,
                    ..
                } = e;
                (
                    macos_analysis,
                    trend_reversal_analysis,
                    ecp1_analysis,
                    indicator_analysis,
                )
            })
            .collect_vec();
        let (macos_analysis, trend_reversal_analysis, ecp1_analysis, indicator_analysis): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = multiunzip(vec);
        let json_str = serde_json::to_string_pretty(&macos_analysis)?;
        assert_eq!(
            include_str!("../assets/9223.T.macos_analysis.json"),
            &json_str
        );
        let json_str = serde_json::to_string_pretty(&trend_reversal_analysis)?;
        assert_eq!(
            include_str!("../assets/9223.T.trend_reversal_analysis.json"),
            &json_str
        );
        let json_str = serde_json::to_string_pretty(&ecp1_analysis)?;
        assert_eq!(
            include_str!("../assets/9223.T.ecp1_analysis.json"),
            &json_str
        );
        let json_str = serde_json::to_string_pretty(&indicator_analysis)?;
        assert_eq!(
            include_str!("../assets/9223.T.indicator_analysis.json"),
            &json_str
        );
    };

    Ok(())
}
