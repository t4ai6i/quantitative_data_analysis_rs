use crate::domain::models::screening::model::{ScoreBreakdown, ScreeningMetrics};
use anyhow::bail;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct ScoreRange {
    pub floor: f64,
    pub ceiling: f64,
    pub weight: f64,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ScoreDirection {
    HigherIsBetter,
    LowerIsBetter,
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct ValueScorePolicy {
    pub per: ScoreRange,
    pub pbr: ScoreRange,
    pub dividend_yield: ScoreRange,
    pub roe: ScoreRange,
    pub sales_growth: ScoreRange,
}

impl TryFrom<&str> for ValueScorePolicy {
    type Error = anyhow::Error;

    fn try_from(preset_name: &str) -> Result<Self, Self::Error> {
        match preset_name {
            "standard" => Ok(ValueScorePolicy::standard()),
            "value" => Ok(ValueScorePolicy::value()),
            "dividend" => Ok(ValueScorePolicy::dividend()),
            _ => bail!(
                "unsupported preset name. preset_name: {}. available presets: standard, value, dividend",
                preset_name
            ),
        }
    }
}

impl ValueScorePolicy {
    pub fn standard() -> Self {
        Self {
            per: ScoreRange {
                floor: 5.0,
                ceiling: 20.0,
                weight: 20.0,
            },
            pbr: ScoreRange {
                floor: 0.5,
                ceiling: 2.0,
                weight: 20.0,
            },
            dividend_yield: ScoreRange {
                floor: 1.0,
                ceiling: 5.0,
                weight: 20.0,
            },
            roe: ScoreRange {
                floor: 5.0,
                ceiling: 20.0,
                weight: 20.0,
            },
            sales_growth: ScoreRange {
                floor: 0.0,
                ceiling: 15.0,
                weight: 20.0,
            },
        }
    }

    pub fn value() -> Self {
        Self {
            per: ScoreRange {
                floor: 3.0,
                ceiling: 15.0,
                weight: 30.0,
            },
            pbr: ScoreRange {
                floor: 0.3,
                ceiling: 1.5,
                weight: 30.0,
            },
            dividend_yield: ScoreRange {
                floor: 1.0,
                ceiling: 4.0,
                weight: 15.0,
            },
            roe: ScoreRange {
                floor: 5.0,
                ceiling: 20.0,
                weight: 15.0,
            },
            sales_growth: ScoreRange {
                floor: 0.0,
                ceiling: 12.0,
                weight: 10.0,
            },
        }
    }

    pub fn dividend() -> Self {
        Self {
            per: ScoreRange {
                floor: 5.0,
                ceiling: 20.0,
                weight: 15.0,
            },
            pbr: ScoreRange {
                floor: 0.5,
                ceiling: 2.0,
                weight: 15.0,
            },
            dividend_yield: ScoreRange {
                floor: 2.0,
                ceiling: 6.0,
                weight: 40.0,
            },
            roe: ScoreRange {
                floor: 5.0,
                ceiling: 20.0,
                weight: 15.0,
            },
            sales_growth: ScoreRange {
                floor: 0.0,
                ceiling: 12.0,
                weight: 15.0,
            },
        }
    }
}

/// 指標値を 0.0..1.0 の範囲に正規化する。
pub fn normalized_score(value: f64, range: ScoreRange, direction: ScoreDirection) -> f64 {
    if !value.is_finite() || !range.floor.is_finite() || !range.ceiling.is_finite() {
        return 0.0;
    }
    if range.ceiling <= range.floor {
        return 0.0;
    }

    match direction {
        ScoreDirection::HigherIsBetter => {
            if value <= range.floor {
                0.0
            } else if value >= range.ceiling {
                1.0
            } else {
                (value - range.floor) / (range.ceiling - range.floor)
            }
        }
        ScoreDirection::LowerIsBetter => {
            if value <= range.floor {
                1.0
            } else if value >= range.ceiling {
                0.0
            } else {
                (range.ceiling - value) / (range.ceiling - range.floor)
            }
        }
    }
}

/// 欠損値は 0 点化し、理由に `missing:<metric>` を残す。
pub fn score_value(
    metrics: &ScreeningMetrics,
    policy: ValueScorePolicy,
) -> (ScoreBreakdown, f64, Vec<String>) {
    let mut reasons = Vec::new();

    let per = score_or_missing(
        metrics.per,
        policy.per,
        ScoreDirection::LowerIsBetter,
        "per",
        &mut reasons,
    );
    let pbr = score_or_missing(
        metrics.pbr,
        policy.pbr,
        ScoreDirection::LowerIsBetter,
        "pbr",
        &mut reasons,
    );
    let dividend_yield = score_or_missing(
        metrics.dividend_yield,
        policy.dividend_yield,
        ScoreDirection::HigherIsBetter,
        "dividend",
        &mut reasons,
    );
    let roe = score_or_missing(
        metrics.roe,
        policy.roe,
        ScoreDirection::HigherIsBetter,
        "roe",
        &mut reasons,
    );
    let sales_growth = score_or_missing(
        metrics.sales_growth,
        policy.sales_growth,
        ScoreDirection::HigherIsBetter,
        "sales_growth",
        &mut reasons,
    );

    let breakdown = ScoreBreakdown {
        per: Some(per),
        pbr: Some(pbr),
        dividend_yield: Some(dividend_yield),
        roe: Some(roe),
        sales_growth: Some(sales_growth),
    };

    let total_score = per + pbr + dividend_yield + roe + sales_growth;
    (breakdown, total_score, reasons)
}

fn score_or_missing(
    value: Option<f64>,
    range: ScoreRange,
    direction: ScoreDirection,
    metric_name: &str,
    reasons: &mut Vec<String>,
) -> f64 {
    match value {
        Some(v) if v.is_finite() => normalized_score(v, range, direction) * range.weight,
        _ => {
            reasons.push(format!("missing:{metric_name}"));
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::domain::models::screening::model::ScreeningMetrics;
    use crate::domain::models::screening::scoring::{
        ScoreDirection, ScoreRange, ValueScorePolicy, normalized_score, score_value,
    };

    #[test]
    fn normalized_score_returns_boundaries_for_higher_is_better() {
        let range = ScoreRange {
            floor: 10.0,
            ceiling: 20.0,
            weight: 20.0,
        };

        assert_eq!(
            normalized_score(5.0, range, ScoreDirection::HigherIsBetter),
            0.0
        );
        assert_eq!(
            normalized_score(25.0, range, ScoreDirection::HigherIsBetter),
            1.0
        );
        assert_eq!(
            normalized_score(15.0, range, ScoreDirection::HigherIsBetter),
            0.5
        );
    }

    #[test]
    fn normalized_score_returns_boundaries_for_lower_is_better() {
        let range = ScoreRange {
            floor: 10.0,
            ceiling: 20.0,
            weight: 20.0,
        };

        assert_eq!(
            normalized_score(5.0, range, ScoreDirection::LowerIsBetter),
            1.0
        );
        assert_eq!(
            normalized_score(25.0, range, ScoreDirection::LowerIsBetter),
            0.0
        );
        assert_eq!(
            normalized_score(15.0, range, ScoreDirection::LowerIsBetter),
            0.5
        );
    }

    #[test]
    fn score_value_fits_in_100_points_with_standard_policy() {
        let metrics = ScreeningMetrics {
            per: Some(5.0),
            pbr: Some(0.5),
            dividend_yield: Some(5.0),
            roe: Some(20.0),
            sales_growth: Some(15.0),
        };

        let (breakdown, total, reasons) = score_value(&metrics, ValueScorePolicy::standard());
        assert_eq!(breakdown.per, Some(20.0));
        assert_eq!(breakdown.pbr, Some(20.0));
        assert_eq!(breakdown.dividend_yield, Some(20.0));
        assert_eq!(breakdown.roe, Some(20.0));
        assert_eq!(breakdown.sales_growth, Some(20.0));
        assert_eq!(total, 100.0);
        assert!(reasons.is_empty());
    }

    #[test]
    fn score_value_sets_zero_and_reason_for_missing_metric() {
        let metrics = ScreeningMetrics {
            per: Some(10.0),
            pbr: Some(1.0),
            dividend_yield: None,
            roe: Some(10.0),
            sales_growth: Some(5.0),
        };

        let (breakdown, _total, reasons) = score_value(&metrics, ValueScorePolicy::standard());
        assert_eq!(breakdown.dividend_yield, Some(0.0));
        assert_eq!(reasons, vec!["missing:dividend".to_string()]);
    }

    #[test]
    fn value_policy_weights_sum_to_100() {
        let p = ValueScorePolicy::value();
        let sum = p.per.weight
            + p.pbr.weight
            + p.dividend_yield.weight
            + p.roe.weight
            + p.sales_growth.weight;
        assert_eq!(sum, 100.0);
    }

    #[test]
    fn dividend_policy_weights_sum_to_100() {
        let p = ValueScorePolicy::dividend();
        let sum = p.per.weight
            + p.pbr.weight
            + p.dividend_yield.weight
            + p.roe.weight
            + p.sales_growth.weight;
        assert_eq!(sum, 100.0);
    }

    #[test]
    fn dividend_preset_scores_higher_for_high_dividend_yield() {
        let high_dividend = ScreeningMetrics {
            per: Some(15.0),
            pbr: Some(1.0),
            dividend_yield: Some(6.0),
            roe: Some(10.0),
            sales_growth: Some(5.0),
        };

        let (_, standard_score, _) = score_value(&high_dividend, ValueScorePolicy::standard());
        let (_, dividend_score, _) = score_value(&high_dividend, ValueScorePolicy::dividend());

        // dividend プリセットは配当利回りの重みが大きいので高配当銘柄は高スコアになる
        assert!(
            dividend_score > standard_score,
            "dividend={dividend_score} should be > standard={standard_score}"
        );
    }

    #[test]
    fn value_preset_scores_higher_for_low_per_pbr() {
        let low_per_pbr = ScreeningMetrics {
            per: Some(3.0),
            pbr: Some(0.3),
            dividend_yield: Some(2.0),
            roe: Some(10.0),
            sales_growth: Some(5.0),
        };

        let (_, standard_score, _) = score_value(&low_per_pbr, ValueScorePolicy::standard());
        let (_, value_score, _) = score_value(&low_per_pbr, ValueScorePolicy::value());

        // value プリセットは PER/PBR の重みが大きいのでバリュー銘柄は高スコアになる
        assert!(
            value_score > standard_score,
            "value={value_score} should be > standard={standard_score}"
        );
    }

    #[test]
    fn standard_policy_is_backward_compatible() {
        let perfect = ScreeningMetrics {
            per: Some(5.0),
            pbr: Some(0.5),
            dividend_yield: Some(5.0),
            roe: Some(20.0),
            sales_growth: Some(15.0),
        };
        let (_, total, _) = score_value(&perfect, ValueScorePolicy::standard());
        assert_eq!(total, 100.0);
    }

    #[test]
    fn try_from_standard_returns_standard_policy() {
        let policy = ValueScorePolicy::try_from("standard").unwrap();
        assert_eq!(policy, ValueScorePolicy::standard());
    }

    #[test]
    fn try_from_value_returns_value_policy() {
        let policy = ValueScorePolicy::try_from("value").unwrap();
        assert_eq!(policy, ValueScorePolicy::value());
    }

    #[test]
    fn try_from_dividend_returns_dividend_policy() {
        let policy = ValueScorePolicy::try_from("dividend").unwrap();
        assert_eq!(policy, ValueScorePolicy::dividend());
    }

    #[test]
    fn try_from_unknown_returns_error() {
        let err = ValueScorePolicy::try_from("unknown").unwrap_err();
        let message = err.to_string();

        assert!(message.contains("unsupported preset name. preset_name: unknown"));
        assert!(message.contains("available presets: standard, value, dividend"));
    }
}
