use crate::domain::models::screening::model::{ScoreComponentDetail, ScreeningMetrics};
use crate::domain::models::screening::scoring::{calculate_sales_growth, score_with_details};
use crate::domain::models::statement::model::RowStatement;
use crate::presenter::presenters::fetch_scoring_data::output::{FetchScoringData, FetchStatus};
use crate::presenter::presenters::score_stock::output;
use crate::shared::float::validate_value;
use crate::use_case::interfaces::score_stock::{input, use_case};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct ScoreStock;

impl ScoreStock {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl use_case::ScoreStock for ScoreStock {
    async fn handle(&self, input: input::ScoreStock) -> Result<output::ScoreStock> {
        let code = input.data.code.clone();
        let scored_at = input.scored_at;

        if input.data.status != FetchStatus::Ok {
            return Ok(skipped_result(code, scored_at));
        }

        let metrics = build_metrics(&input.data);
        let computation = score_with_details(&metrics, input.policy);

        Ok(output::ScoreStock {
            code,
            scored_at,
            status: output::ScoreStatus::Ok,
            error_type: None,
            score: Some(output::Score {
                total: computation.total_score,
                components: to_presenter_components(computation.components),
            }),
        })
    }
}

fn build_metrics(data: &FetchScoringData) -> ScreeningMetrics {
    let price = data.price.as_ref().and_then(|price| price.adj_close);
    let latest_statement = data.latest_statement.as_ref();
    let per = match (price, latest_statement.and_then(|statement| statement.eps)) {
        (Some(price), Some(eps)) => validate_value(price / eps),
        _ => None,
    };
    let pbr = match (price, latest_statement.and_then(|statement| statement.bps)) {
        (Some(price), Some(bps)) => validate_value(price / bps),
        _ => None,
    };
    let dividend_yield = match (
        price,
        latest_statement.and_then(|statement| statement.annual_dividend_forecast),
    ) {
        (Some(price), Some(dividend)) => validate_value(dividend / price).map(|v| v * 100.0),
        _ => None,
    };
    let roe = match (
        latest_statement.and_then(|statement| statement.profit),
        latest_statement.and_then(|statement| statement.equity),
    ) {
        (Some(profit), Some(equity)) => validate_value(profit / equity).map(|v| v * 100.0),
        _ => None,
    };
    let sales_growth = calculate_sales_growth(&build_row_statements(data));

    ScreeningMetrics {
        per,
        pbr,
        dividend_yield,
        roe,
        sales_growth,
    }
}

fn build_row_statements(data: &FetchScoringData) -> Vec<RowStatement> {
    data.full_year_sales
        .iter()
        .map(|sales| RowStatement {
            code: data.code.clone(),
            disclosed_date: sales.disclosed_date,
            current_fiscal_year_end_date: sales.current_fiscal_year_end_date,
            net_sales: sales.net_sales,
            ..Default::default()
        })
        .collect()
}

fn to_presenter_components(
    components: BTreeMap<String, ScoreComponentDetail>,
) -> BTreeMap<String, output::ComponentDetail> {
    components
        .into_iter()
        .map(|(name, detail)| {
            let presenter_detail = output::ComponentDetail {
                raw: detail.raw,
                normalized: detail.normalized,
                points: detail.points,
            };
            (name, presenter_detail)
        })
        .collect()
}

fn skipped_result(code: String, scored_at: DateTime<Utc>) -> output::ScoreStock {
    output::ScoreStock {
        code,
        scored_at,
        status: output::ScoreStatus::Skipped,
        error_type: Some(output::ScoreErrorType::InvalidInput),
        score: None,
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, NaiveDate, Utc};

    use crate::domain::models::screening::scoring::ValueScorePolicy;
    use crate::presenter::presenters::fetch_scoring_data::output::{
        FetchCompany, FetchFullYearSales, FetchLatestStatement, FetchPrice, FetchScoringData,
        FetchStatus,
    };
    use crate::presenter::presenters::score_stock::output::{
        ComponentDetail, ScoreErrorType, ScoreStatus,
    };
    use crate::use_case::interfaces::score_stock::input;
    use crate::use_case::interfaces::score_stock::use_case::ScoreStock as _;

    use super::ScoreStock;

    // テスト用の固定時刻定数
    fn fixed_now() -> DateTime<Utc> {
        "2026-08-26T18:00:00Z".parse().unwrap()
    }

    fn build_ok_data() -> FetchScoringData {
        FetchScoringData {
            code: "1301".to_string(),
            fetched_at: fixed_now(),
            status: FetchStatus::Ok,
            error_type: None,
            company: Some(FetchCompany {
                name: Some("Test Company".to_string()),
            }),
            price: Some(FetchPrice {
                date: Some(NaiveDate::from_ymd_opt(2025, 3, 31).unwrap()),
                adj_close: Some(100.0),
            }),
            latest_statement: Some(FetchLatestStatement {
                disclosed_date: Some(NaiveDate::from_ymd_opt(2025, 5, 9).unwrap()),
                eps: Some(20.0),
                bps: Some(200.0),
                annual_dividend_forecast: Some(5.0),
                profit: Some(20.0),
                equity: Some(100.0),
            }),
            full_year_sales: vec![
                FetchFullYearSales {
                    current_fiscal_year_end_date: Some(
                        NaiveDate::from_ymd_opt(2025, 3, 31).unwrap(),
                    ),
                    disclosed_date: Some(NaiveDate::from_ymd_opt(2025, 5, 9).unwrap()),
                    net_sales: Some(115.0),
                },
                FetchFullYearSales {
                    current_fiscal_year_end_date: Some(
                        NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
                    ),
                    disclosed_date: Some(NaiveDate::from_ymd_opt(2024, 5, 10).unwrap()),
                    net_sales: Some(100.0),
                },
            ],
        }
    }

    fn build_input(data: FetchScoringData, policy: ValueScorePolicy) -> input::ScoreStock {
        input::ScoreStock::new(data, policy, fixed_now())
    }

    #[tokio::test]
    async fn handle_returns_skipped_when_fetch_status_is_not_ok() {
        let mut data = build_ok_data();
        data.status = FetchStatus::EmptyData;

        let actual = ScoreStock::new()
            .handle(build_input(data, ValueScorePolicy::standard()))
            .await
            .unwrap();

        assert_eq!(actual.code, "1301");
        assert_eq!(actual.status, ScoreStatus::Skipped);
        assert_eq!(actual.error_type, Some(ScoreErrorType::InvalidInput));
        assert!(actual.score.is_none());
    }

    #[tokio::test]
    async fn handle_returns_ok_and_maps_components() {
        let actual = ScoreStock::new()
            .handle(build_input(build_ok_data(), ValueScorePolicy::standard()))
            .await
            .unwrap();

        assert_eq!(actual.code, "1301");
        assert_eq!(actual.status, ScoreStatus::Ok);
        assert_eq!(actual.error_type, None);

        let score = actual.score.expect("score should exist");
        assert_eq!(score.total, 100.0);
        assert_eq!(score.components.len(), 5);
        assert_eq!(
            score.components.get("per"),
            Some(&ComponentDetail {
                raw: 5.0,
                normalized: 1.0,
                points: 20.0,
            })
        );
        assert_eq!(
            score.components.get("pbr"),
            Some(&ComponentDetail {
                raw: 0.5,
                normalized: 1.0,
                points: 20.0,
            })
        );
        assert_eq!(
            score.components.get("dividend_yield"),
            Some(&ComponentDetail {
                raw: 5.0,
                normalized: 1.0,
                points: 20.0,
            })
        );
        assert_eq!(
            score.components.get("roe"),
            Some(&ComponentDetail {
                raw: 20.0,
                normalized: 1.0,
                points: 20.0,
            })
        );
        assert_eq!(
            score.components.get("sales_growth"),
            Some(&ComponentDetail {
                raw: 15.0,
                normalized: 1.0,
                points: 20.0,
            })
        );
    }
}
