use anyhow::Result;
use bytes::Bytes;
use chrono::NaiveDate;
use itertools::{multiunzip, Itertools};
use quantitative_data_analysis_rs::infrastructure::repositories::company::structures::internal::tsv;
use quantitative_data_analysis_rs::presenter::views::high_low_direction_signal_analysis::view::HighLowDirectionSignalAnalysis;
use quantitative_data_analysis_rs::presenter::views::shared::crossover_pattern_filter::CrossoverPatternFilter;
use quantitative_data_analysis_rs::presenter::views::sma_cos_analysis::view::SmaCosAnalysis;
use quantitative_data_analysis_rs::presenter::views::trend_analysis_summary::json::view::JsonRow;
use quantitative_data_analysis_rs::presenter::views::trend_reversal_analysis::view::TrendReversalAnalysis;
use quantitative_data_analysis_rs::shared::jquants_api::setup::Setup;
use quantitative_data_analysis_rs::{controller, infrastructure, presenter, use_case};
use tokio::fs::write;

const AFTER_DAYS_5: usize = 5;
const FROM_END_DAYS_7: isize = 7;
const MARUBOZU_BODY_MIN_RATIO: usize = 90;
const MARUBOZU_WICK_MAX_RATIO: usize = 2;
const DOJI_MAX_BODY_RATIO: usize = 5;
const FAST_PERIOD_12: usize = 12;
const SLOW_PERIOD_26: usize = 26;
const SIGNAL_PERIOD_9: usize = 9;
const DATE_FORMAT: &str = "%Y/%m/%d";

const COMPANIES_TSV: &[u8] = include_bytes!("../assets/companies.tsv");

#[tokio::main]
async fn main() -> Result<()> {
    // JQUANTS APIのためのトークン準備
    let token = Setup::run()?;

    // DSVを用いたレポジトリの準備
    let dsv = infrastructure::dsv::Dsv::<tsv::Structure>::new(false, Bytes::from(COMPANIES_TSV));
    // JQUANTS APIを用いたレポジトリの準備
    let jquants_api = infrastructure::jquants_api::JQuantsAPI::new(token)?;
    // TrendAnalysisのドメインロジックを実行するInteractorの準備
    let interactor =
        use_case::interactors::trend_analysis::interactor::TrendAnalysis::new(&jquants_api, &dsv);
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
    let trend_analysis = controller
        .analyze::<
            AFTER_DAYS_5,
            FROM_END_DAYS_7,
            MARUBOZU_BODY_MIN_RATIO,
            MARUBOZU_WICK_MAX_RATIO,
            DOJI_MAX_BODY_RATIO,
            FAST_PERIOD_12,
            SLOW_PERIOD_26,
            SIGNAL_PERIOD_9,
        >(
            "84730",
            "T",
            NaiveDate::from_ymd_opt(2022, 9, 9).unwrap(),
            NaiveDate::from_ymd_opt(2023, 9, 8).unwrap(),
            CrossoverPatternFilter::Both,
        )
        .await?;
    if let presenter::presenters::trend_analysis::response::TrendAnalysis::Chart {
        ref body, ..
    } = trend_analysis
    {
        write("./examples/8473.T.from_jquants_api.svg", body).await?;
        assert_eq!(include_str!("../assets/8473.T.from_jquants_api.svg"), body);
    }

    // TrendAnalysisのドメインロジックを実行するInteractorの準備
    let interactor =
        use_case::interactors::trend_analysis::interactor::TrendAnalysis::new(&jquants_api, &dsv);
    // PresenterはJSON型でJSON形式のデータを出力する
    let presenter = presenter::presenters::trend_analysis::response::json::JSON;
    // 指定された証券コードのトレンド解析を行う
    let controller =
        controller::trend_analysis::controller::TrendAnalysis::new(&interactor, &presenter);
    let trend_analysis = controller
        .analyze::<
            AFTER_DAYS_5,
            FROM_END_DAYS_7,
            MARUBOZU_BODY_MIN_RATIO,
            MARUBOZU_WICK_MAX_RATIO,
            DOJI_MAX_BODY_RATIO,
            FAST_PERIOD_12,
            SLOW_PERIOD_26,
            SIGNAL_PERIOD_9,
        >(
            "84730",
            "T",
            NaiveDate::from_ymd_opt(2022, 9, 9).unwrap(),
            NaiveDate::from_ymd_opt(2023, 9, 8).unwrap(),
            CrossoverPatternFilter::Both,
        )
        .await?;

    // TrendAnalysisSummaryのドメインロジックを実行するInteractorの準備
    let interactor =
        use_case::interactors::trend_analysis_summary::interactor::TrendAnalysisSummary;
    // PresenterはJSON型でJSON形式のデータを出力する
    let presenter = presenter::presenters::trend_analysis_summary::response::json::JSON;
    let vec_trend_analysis = vec![trend_analysis];
    let trend_analyses =
        presenter::presenters::trend_analysis::response::TrendAnalyses(vec_trend_analysis);
    // TrendAnalysisの集合からサマリーを出力する
    let controller = controller::trend_analysis_summary::controller::TrendAnalysisSummary::new(
        &interactor,
        &presenter,
    );
    if let presenter::presenters::trend_analysis_summary::response::TrendAnalysisSummary::JSON {
        json_rows,
    } = controller
        .analyze(trend_analyses, CrossoverPatternFilter::Both)
        .await?
    {
        let tuples = json_rows
            .iter()
            .map(|row| {
                let JsonRow {
                    sma_cos_analysis,
                    trend_reversal_analysis,
                    high_low_direction_signal_analysis,
                    ..
                } = row;
                (
                    sma_cos_analysis,
                    trend_reversal_analysis,
                    high_low_direction_signal_analysis,
                )
            })
            .collect_vec();
        let (sma_cos_analysis, trend_reversal_analysis, high_low_direction_signal_analysis): (
            Vec<&SmaCosAnalysis>,
            Vec<&TrendReversalAnalysis>,
            Vec<&HighLowDirectionSignalAnalysis>,
        ) = multiunzip(tuples);
        let json_str = serde_json::to_string_pretty(&sma_cos_analysis)?;
        assert_eq!(
            include_str!("../assets/8473.T.sma_cos_analysis.json"),
            &json_str
        );
        let json_str = serde_json::to_string_pretty(&trend_reversal_analysis)?;
        assert_eq!(
            include_str!("../assets/8473.T.trend_reversal_analysis.json"),
            &json_str
        );
        let json_str = serde_json::to_string_pretty(&high_low_direction_signal_analysis)?;
        assert_eq!(
            include_str!("../assets/8473.T.high_low_direction_signal_analysis.json"),
            &json_str
        );
    };

    // エンガルフィンパターン以外（モーニングスター・イブニングスターパターン）の結果が正しく行われたか確認するため、株価データが少ない証券コードを用いる
    let interactor =
        use_case::interactors::trend_analysis::interactor::TrendAnalysis::new(&jquants_api, &dsv);

    let presenter = presenter::presenters::trend_analysis::response::json::JSON;
    let controller =
        controller::trend_analysis::controller::TrendAnalysis::new(&interactor, &presenter);
    let trend_analysis = controller
        .analyze::<
            AFTER_DAYS_5,
            FROM_END_DAYS_7,
            MARUBOZU_BODY_MIN_RATIO,
            MARUBOZU_WICK_MAX_RATIO,
            DOJI_MAX_BODY_RATIO,
            FAST_PERIOD_12,
            SLOW_PERIOD_26,
            SIGNAL_PERIOD_9,
        >(
            "92230",
            "T",
            NaiveDate::from_ymd_opt(2023, 12, 25).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 16).unwrap(),
            CrossoverPatternFilter::Both,
        )
        .await?;

    let interactor =
        use_case::interactors::trend_analysis_summary::interactor::TrendAnalysisSummary;
    let presenter = presenter::presenters::trend_analysis_summary::response::json::JSON;
    let controller = controller::trend_analysis_summary::controller::TrendAnalysisSummary::new(
        &interactor,
        &presenter,
    );

    let vec_trend_analysis = vec![trend_analysis];
    let trend_analyses =
        presenter::presenters::trend_analysis::response::TrendAnalyses(vec_trend_analysis);
    if let presenter::presenters::trend_analysis_summary::response::TrendAnalysisSummary::JSON {
        json_rows,
    } = controller
        .analyze(trend_analyses, CrossoverPatternFilter::Both)
        .await?
    {
        let tuples = json_rows
            .iter()
            .map(|row| {
                let JsonRow {
                    sma_cos_analysis,
                    trend_reversal_analysis,
                    high_low_direction_signal_analysis,
                    ..
                } = row;
                (
                    sma_cos_analysis,
                    trend_reversal_analysis,
                    high_low_direction_signal_analysis,
                )
            })
            .collect_vec();
        let (sma_cos_analysis, trend_reversal_analysis, high_low_direction_signal_analysis): (
            Vec<&SmaCosAnalysis>,
            Vec<&TrendReversalAnalysis>,
            Vec<&HighLowDirectionSignalAnalysis>,
        ) = multiunzip(tuples);
        let json_str = serde_json::to_string_pretty(&sma_cos_analysis)?;
        assert_eq!(
            include_str!("../assets/9223.T.sma_cos_analysis.json"),
            &json_str
        );
        let json_str = serde_json::to_string_pretty(&trend_reversal_analysis)?;
        assert_eq!(
            include_str!("../assets/9223.T.trend_reversal_analysis.json"),
            &json_str
        );
        let json_str = serde_json::to_string_pretty(&high_low_direction_signal_analysis)?;
        assert_eq!(
            include_str!("../assets/9223.T.high_low_direction_signal_analysis.json"),
            &json_str
        );
    };

    Ok(())
}
