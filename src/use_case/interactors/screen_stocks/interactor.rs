use crate::presenter::presenters::score_stock::output::ScoreStatus;
use crate::presenter::presenters::screen_stocks::output;
use crate::presenter::presenters::screen_stocks::output::{RankingEntry, SummaryDetail};
use crate::use_case::interfaces::screen_stocks::{input, use_case};
use async_trait::async_trait;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct ScreenStocks;

impl ScreenStocks {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl use_case::ScreenStocks for ScreenStocks {
    async fn handle(&self, input: input::ScreenStocks) -> anyhow::Result<output::ScreenStocks> {
        let generated_at = input.generated_at;
        let mut scores = input.scores;

        scores.sort_by(|a, b| {
            let a_score = a.score.as_ref().map(|s| s.total);
            let b_score = b.score.as_ref().map(|s| s.total);
            match (a_score, b_score) {
                (Some(a), Some(b)) => b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        let total = scores.len();
        let ok = scores
            .iter()
            .filter(|s| s.status == ScoreStatus::Ok)
            .count();
        let failed = scores
            .iter()
            .filter(|s| s.status == ScoreStatus::Failed)
            .count();
        let skipped = scores
            .iter()
            .filter(|s| s.status == ScoreStatus::Skipped)
            .count();
        let summary = SummaryDetail {
            total,
            ok,
            failed,
            skipped,
        };

        let ranking: Vec<RankingEntry> = scores
            .into_iter()
            .map(|score_stock| {
                let components = score_stock
                    .score
                    .as_ref()
                    .map(|s| {
                        s.components
                            .iter()
                            .map(|(k, v)| {
                                (
                                    k.clone(),
                                    output::ComponentDetail {
                                        raw: v.raw,
                                        normalized: v.normalized,
                                        points: v.points,
                                    },
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                RankingEntry {
                    code: score_stock.code,
                    total_score: score_stock.score.as_ref().map_or(0.0, |s| s.total),
                    components,
                }
            })
            .collect();
        Ok(output::ScreenStocks {
            generated_at,
            preset_name: input.preset_name,
            summary,
            ranking,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::presenter::presenters::score_stock::output::{
        ComponentDetail, Score, ScoreErrorType, ScoreStatus, ScoreStock,
    };
    use crate::use_case::interactors::screen_stocks::interactor;
    use crate::use_case::interfaces::screen_stocks::input;
    use crate::use_case::interfaces::screen_stocks::use_case::ScreenStocks;
    use chrono::{DateTime, Utc};
    use std::collections::BTreeMap;

    // テスト用の固定時刻定数
    fn fixed_now() -> DateTime<Utc> {
        "2026-08-26T18:00:00Z".parse().unwrap()
    }

    #[tokio::test]
    async fn test_empty_scores() {
        let interactor = interactor::ScreenStocks::new();
        let input = input::ScreenStocks::new(vec![], "fixed_value", fixed_now());

        let result = interactor.handle(input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.generated_at, fixed_now());
        assert_eq!(output.summary.total, 0);
        assert_eq!(output.summary.ok, 0);
        assert_eq!(output.summary.failed, 0);
        assert_eq!(output.summary.skipped, 0);
        assert_eq!(output.ranking.len(), 0);
        assert_eq!(output.preset_name, "fixed_value");
    }

    #[tokio::test]
    async fn test_mixed_status_summary() {
        let interactor = interactor::ScreenStocks::new();
        let mut components = BTreeMap::new();
        components.insert(
            "per".to_string(),
            ComponentDetail {
                raw: 10.0,
                normalized: 50.0,
                points: 10.0,
            },
        );

        let scores = vec![
            ScoreStock {
                code: "1234".to_string(),
                scored_at: fixed_now(),
                status: ScoreStatus::Ok,
                error_type: None,
                score: Some(Score {
                    total: 75.0,
                    components: components.clone(),
                }),
            },
            ScoreStock {
                code: "5678".to_string(),
                scored_at: fixed_now(),
                status: ScoreStatus::Failed,
                error_type: Some(ScoreErrorType::InvalidInput),
                score: None,
            },
            ScoreStock {
                code: "9012".to_string(),
                scored_at: fixed_now(),
                status: ScoreStatus::Skipped,
                error_type: None,
                score: None,
            },
            ScoreStock {
                code: "3456".to_string(),
                scored_at: fixed_now(),
                status: ScoreStatus::Ok,
                error_type: None,
                score: Some(Score {
                    total: 85.0,
                    components: components.clone(),
                }),
            },
        ];

        let input = input::ScreenStocks::new(scores, "fixed_value", fixed_now());
        let result = interactor.handle(input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.generated_at, fixed_now());
        assert_eq!(output.summary.total, 4);
        assert_eq!(output.summary.ok, 2);
        assert_eq!(output.summary.failed, 1);
        assert_eq!(output.summary.skipped, 1);
    }

    #[tokio::test]
    async fn test_ranking_descending_by_total_score() {
        let interactor = interactor::ScreenStocks::new();
        let mut components = BTreeMap::new();
        components.insert(
            "per".to_string(),
            ComponentDetail {
                raw: 10.0,
                normalized: 50.0,
                points: 10.0,
            },
        );

        // intentionally unordered
        let scores = vec![
            ScoreStock {
                code: "1111".to_string(),
                scored_at: fixed_now(),
                status: ScoreStatus::Ok,
                error_type: None,
                score: Some(Score {
                    total: 60.0,
                    components: components.clone(),
                }),
            },
            ScoreStock {
                code: "2222".to_string(),
                scored_at: fixed_now(),
                status: ScoreStatus::Ok,
                error_type: None,
                score: Some(Score {
                    total: 90.0,
                    components: components.clone(),
                }),
            },
            ScoreStock {
                code: "3333".to_string(),
                scored_at: fixed_now(),
                status: ScoreStatus::Ok,
                error_type: None,
                score: Some(Score {
                    total: 75.0,
                    components: components.clone(),
                }),
            },
        ];

        let input = input::ScreenStocks::new(scores, "fixed_value", fixed_now());
        let result = interactor.handle(input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.generated_at, fixed_now());
        assert_eq!(output.ranking.len(), 3);
        assert_eq!(output.ranking[0].code, "2222");
        assert_eq!(output.ranking[0].total_score, 90.0);
        assert_eq!(output.ranking[1].code, "3333");
        assert_eq!(output.ranking[1].total_score, 75.0);
        assert_eq!(output.ranking[2].code, "1111");
        assert_eq!(output.ranking[2].total_score, 60.0);
    }
}
