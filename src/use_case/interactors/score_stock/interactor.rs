use crate::domain::models::scoring::model::{
    ScoreDirection, ScoringMetrics, ValueScorePolicy, normalized_score, score_with_details,
};
use crate::domain::models::statement::model::{AccountingScope, FinancialDisclosure};
use crate::presenter::presenters::fetch_scoring_data::output::{FetchScoringData, FetchStatus};
use crate::presenter::presenters::score_stock::output;
use crate::shared::float::validate_value;
use crate::use_case::interfaces::score_stock::{input, use_case};
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use std::collections::{BTreeMap, BTreeSet};

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
        let target_date = input.data.target_date;
        let cutoff_at = analysis_cutoff(target_date);

        if input.data.status == FetchStatus::Skipped {
            return Ok(skipped_result(code, scored_at, target_date, cutoff_at));
        }
        if input.data.status != FetchStatus::Ok {
            let mut metrics = empty_metrics();
            attach_global_reasons(&mut metrics, &[output::ScoreReasonCode::IncompleteHistory]);
            return Ok(build_result(
                &input.data,
                scored_at,
                cutoff_at,
                metrics,
                Vec::new(),
                input.policy,
                Some(output::ScoreErrorType::IncompleteHistory),
            ));
        }

        evaluate(&input.data, input.policy, scored_at)
    }
}

fn analysis_cutoff(target_date: NaiveDate) -> NaiveDateTime {
    let winter_schedule = NaiveDate::from_ymd_opt(2024, 11, 5).expect("valid cutoff date");
    let cutoff_time = if target_date >= winter_schedule {
        NaiveTime::from_hms_opt(15, 30, 0).expect("valid cutoff time")
    } else {
        NaiveTime::from_hms_opt(15, 0, 0).expect("valid cutoff time")
    };
    target_date.and_time(cutoff_time)
}

fn derive_single_quarter_actuals(
    disclosures: &[&FinancialDisclosure],
) -> Result<Vec<output::SingleQuarterActual>> {
    type GroupKey = (NaiveDate, NaiveDate, AccountingScope, String);
    let mut groups =
        BTreeMap::<GroupKey, BTreeMap<output::FiscalQuarter, Vec<&FinancialDisclosure>>>::new();
    let mut actuals = Vec::new();

    for disclosure in disclosures {
        if !disclosure.document_type.contains("FinancialStatements") {
            continue;
        }
        let Some((quarter, period_type)) = fiscal_quarter(disclosure) else {
            continue;
        };
        let prefix = format!("{period_type}FinancialStatements_");
        let Some(document_basis) = disclosure.document_type.strip_prefix(&prefix) else {
            actuals.push(undetermined_quarter(
                disclosure,
                quarter,
                None,
                output::ScoreReasonCode::IncompatibleDisclosureBasis,
            ));
            continue;
        };
        let (Some(fiscal_year_start), Some(fiscal_year_end), Some(period_start), Some(period_end)) = (
            disclosure.current_fiscal_year_start,
            disclosure.current_fiscal_year_end,
            disclosure.current_period_start,
            disclosure.current_period_end,
        ) else {
            actuals.push(undetermined_quarter(
                disclosure,
                quarter,
                Some(document_basis.to_owned()),
                output::ScoreReasonCode::UnknownFiscalPeriod,
            ));
            continue;
        };
        if period_start != fiscal_year_start
            || period_end < fiscal_year_start
            || period_end > fiscal_year_end
            || (quarter == output::FiscalQuarter::Q4 && period_end != fiscal_year_end)
            || (quarter != output::FiscalQuarter::Q4 && period_end == fiscal_year_end)
        {
            actuals.push(undetermined_quarter(
                disclosure,
                quarter,
                Some(document_basis.to_owned()),
                output::ScoreReasonCode::IncompatibleFiscalPeriods,
            ));
            continue;
        }
        groups
            .entry((
                fiscal_year_start,
                fiscal_year_end,
                disclosure.accounting_scope,
                document_basis.to_owned(),
            ))
            .or_default()
            .entry(quarter)
            .or_default()
            .push(disclosure);
    }

    for ((fiscal_year_start, fiscal_year_end, scope, document_basis), reports) in groups {
        for (&quarter, candidates) in &reports {
            let current = latest_disclosures(candidates);
            let prior_quarter = match quarter {
                output::FiscalQuarter::Q1 => None,
                output::FiscalQuarter::Q2 => Some(output::FiscalQuarter::Q1),
                output::FiscalQuarter::Q3 => Some(output::FiscalQuarter::Q2),
                output::FiscalQuarter::Q4 => Some(output::FiscalQuarter::Q3),
            };
            let prior_candidates = prior_quarter.and_then(|period| reports.get(&period));
            let prior = prior_candidates
                .map(|candidates| latest_disclosures(candidates))
                .unwrap_or_default();
            let current_disclosure = current.first().copied().filter(|_| current.len() == 1);
            let prior_disclosure = prior.first().copied().filter(|_| prior.len() == 1);
            let mut actual_period_start = if quarter == output::FiscalQuarter::Q1 {
                Some(fiscal_year_start)
            } else {
                None
            };
            let current_period_end = current_disclosure.and_then(|item| item.current_period_end);
            let mut period_reasons = Vec::new();

            if current.len() > 1 || prior.len() > 1 {
                period_reasons.push(output::ScoreReasonCode::AmbiguousDisclosure);
            } else if quarter != output::FiscalQuarter::Q1 {
                match (current_disclosure, prior_disclosure) {
                    (Some(current), Some(prior)) => {
                        let time_ordered = matches!(
                            (disclosure_timestamp(current), disclosure_timestamp(prior)),
                            (Some(current_time), Some(prior_time)) if current_time > prior_time
                        );
                        match (prior.current_period_end, current.current_period_end) {
                            (Some(prior_end), Some(current_end))
                                if prior_end < current_end && time_ordered =>
                            {
                                actual_period_start =
                                    prior_end.checked_add_signed(Duration::days(1));
                                if actual_period_start.is_none() {
                                    period_reasons
                                        .push(output::ScoreReasonCode::IncompatibleFiscalPeriods);
                                }
                            }
                            _ => period_reasons
                                .push(output::ScoreReasonCode::IncompatibleFiscalPeriods),
                        }
                    }
                    _ if prior_candidates.is_none() => {
                        period_reasons.push(output::ScoreReasonCode::NoPriorQuarter)
                    }
                    _ => period_reasons.push(output::ScoreReasonCode::AmbiguousDisclosure),
                }
            }

            if scope == AccountingScope::Unknown {
                period_reasons.push(output::ScoreReasonCode::UnknownAccountingScope);
            }
            let current_sources = current
                .iter()
                .map(|item| disclosure_reference(item, item.accounting_scope))
                .collect::<Vec<_>>();
            let prior_sources = prior
                .iter()
                .map(|item| disclosure_reference(item, item.accounting_scope))
                .collect::<Vec<_>>();
            let mut values = BTreeMap::new();

            for (name, field) in quarterly_metric_fields(scope) {
                let mut reasons = period_reasons.clone();
                let current_value = match (current_disclosure, field) {
                    (Some(disclosure), Some(field)) => disclosure.numeric_field(field)?,
                    (_, None) => None,
                    _ => None,
                };
                let prior_value = match (prior_disclosure, field) {
                    (Some(disclosure), Some(field)) if quarter != output::FiscalQuarter::Q1 => {
                        disclosure.numeric_field(field)?
                    }
                    _ => None,
                };
                if field.is_none() {
                    if !reasons.contains(&output::ScoreReasonCode::UnknownAccountingScope) {
                        reasons.push(output::ScoreReasonCode::UnknownAccountingScope);
                    }
                } else if (current_disclosure.is_some() && current_value.is_none())
                    || (prior_disclosure.is_some() && prior_value.is_none())
                {
                    reasons.push(output::ScoreReasonCode::MissingValue);
                }
                let result = if reasons.is_empty() {
                    match (current_value, prior_value) {
                        (Some(current), None) if quarter == output::FiscalQuarter::Q1 => {
                            Some(current)
                        }
                        (Some(current), Some(prior)) => validate_value(current - prior),
                        _ => None,
                    }
                } else {
                    None
                };
                if reasons.is_empty() && result.is_none() {
                    reasons.push(output::ScoreReasonCode::InvalidValue);
                }
                let mut source_disclosures = current_sources.clone();
                for source in &prior_sources {
                    push_source(&mut source_disclosures, source.clone());
                }
                values.insert(
                    name.to_owned(),
                    output::SingleQuarterValue {
                        current_cumulative_input: current_value,
                        prior_cumulative_input: prior_value,
                        result,
                        source_disclosures,
                        reasons,
                    },
                );
            }

            actuals.push(output::SingleQuarterActual {
                fiscal_year_start: Some(fiscal_year_start),
                fiscal_year_end: Some(fiscal_year_end),
                quarter,
                period_start: actual_period_start,
                period_end: current_period_end,
                accounting_scope: scope,
                document_basis: Some(document_basis.clone()),
                values,
            });
        }
    }

    actuals.sort_by_key(|item| (item.fiscal_year_start, item.quarter));
    Ok(actuals)
}

fn fiscal_quarter(
    disclosure: &FinancialDisclosure,
) -> Option<(output::FiscalQuarter, &'static str)> {
    match disclosure.current_period_type.as_deref()? {
        "1Q" => Some((output::FiscalQuarter::Q1, "1Q")),
        "2Q" => Some((output::FiscalQuarter::Q2, "2Q")),
        "3Q" => Some((output::FiscalQuarter::Q3, "3Q")),
        "FY" => Some((output::FiscalQuarter::Q4, "FY")),
        _ => None,
    }
}

fn latest_disclosures<'a>(disclosures: &[&'a FinancialDisclosure]) -> Vec<&'a FinancialDisclosure> {
    let Some(latest) = disclosures
        .iter()
        .filter_map(|disclosure| disclosure_timestamp(disclosure))
        .max()
    else {
        return Vec::new();
    };
    disclosures
        .iter()
        .copied()
        .filter(|disclosure| disclosure_timestamp(disclosure) == Some(latest))
        .collect()
}

fn quarterly_metric_fields(scope: AccountingScope) -> [(&'static str, Option<&'static str>); 4] {
    let operating_profit = match scope {
        AccountingScope::Consolidated => Some("OP"),
        AccountingScope::NonConsolidated => Some("NCOP"),
        AccountingScope::Unknown => None,
    };
    [
        ("sales", Some("Sales")),
        ("operating_profit", operating_profit),
        ("ordinary_profit", Some("OdP")),
        ("net_income", Some("NP")),
    ]
}

fn undetermined_quarter(
    disclosure: &FinancialDisclosure,
    quarter: output::FiscalQuarter,
    document_basis: Option<String>,
    reason: output::ScoreReasonCode,
) -> output::SingleQuarterActual {
    let source = disclosure_reference(disclosure, disclosure.accounting_scope);
    let values = quarterly_metric_fields(disclosure.accounting_scope)
        .into_iter()
        .map(|(name, _)| {
            (
                name.to_owned(),
                output::SingleQuarterValue {
                    current_cumulative_input: None,
                    prior_cumulative_input: None,
                    result: None,
                    source_disclosures: vec![source.clone()],
                    reasons: vec![reason.clone()],
                },
            )
        })
        .collect();
    output::SingleQuarterActual {
        fiscal_year_start: disclosure.current_fiscal_year_start,
        fiscal_year_end: disclosure.current_fiscal_year_end,
        quarter,
        period_start: None,
        period_end: disclosure.current_period_end,
        accounting_scope: disclosure.accounting_scope,
        document_basis,
        values,
    }
}

fn evaluate(
    data: &FetchScoringData,
    policy: ValueScorePolicy,
    scored_at: DateTime<Utc>,
) -> Result<output::ScoreStock> {
    let cutoff_at = analysis_cutoff(data.target_date);
    let mut metrics = empty_metrics();
    let mut global_reasons = Vec::new();

    if data.financial_disclosures.is_empty()
        || data.financial_disclosures.iter().any(|disclosure| {
            !disclosure.matches_code(&data.code)
                || disclosure.disclosed_date.is_none()
                || disclosure.disclosed_time.is_none()
        })
    {
        global_reasons.push(output::ScoreReasonCode::IncompleteHistory);
    }
    if data.financial_disclosures.iter().any(|disclosure| {
        disclosure.disclosed_date.is_none() || disclosure.disclosed_time.is_none()
    }) {
        global_reasons.push(output::ScoreReasonCode::UnknownDisclosureTime);
    }

    if !global_reasons.is_empty() {
        attach_global_reasons(&mut metrics, &global_reasons);
        return Ok(build_result(
            data,
            scored_at,
            cutoff_at,
            metrics,
            Vec::new(),
            policy,
            Some(output::ScoreErrorType::IncompleteHistory),
        ));
    }

    let eligible = data
        .financial_disclosures
        .iter()
        .filter(|disclosure| disclosure_timestamp(disclosure).is_some_and(|time| time <= cutoff_at))
        .collect::<Vec<_>>();

    if eligible.is_empty() {
        attach_global_reasons(
            &mut metrics,
            &[output::ScoreReasonCode::NoMatchingDisclosure],
        );
        return Ok(build_result(
            data,
            scored_at,
            cutoff_at,
            metrics,
            Vec::new(),
            policy,
            None,
        ));
    }

    let quarterly_actuals = derive_single_quarter_actuals(&eligible)?;

    let scope = match latest_accounting_scope(&eligible) {
        Ok(scope) => scope,
        Err(reason) => {
            attach_global_reasons(&mut metrics, &[reason]);
            return Ok(build_result(
                data,
                scored_at,
                cutoff_at,
                metrics,
                quarterly_actuals,
                policy,
                None,
            ));
        }
    };
    let target_period = match target_fiscal_period(&eligible, data.target_date) {
        Ok(period) => Some(period),
        Err(reason) => {
            add_reason(&mut metrics, "per", reason.clone());
            add_reason(&mut metrics, "dividend_yield", reason);
            None
        }
    };

    let (price, price_reasons) = price_input(data);
    let mut per = metric_audit(price, data.price.as_ref().and_then(|price| price.date));
    per.reasons.extend(price_reasons.clone());
    if let Some(period) = target_period {
        per.period_start = Some(period.0);
        per.period_end = Some(period.1);
        match select_forecast(&eligible, scope, period, "EPS")? {
            Ok((disclosure, field)) => {
                let eps = disclosure.numeric_field(&field)?;
                per.inputs.insert("forecast_eps".to_string(), eps);
                per.source_disclosures
                    .push(disclosure_reference(disclosure, scope));
                per.result = calculate_ratio(price, eps, &mut per.reasons);
            }
            Err(reason) => per.reasons.push(reason),
        }
    }
    metrics.insert("per".to_string(), per);

    let annual = annual_groups(&eligible, scope, data.target_date)?;
    let latest_annual = annual.first();

    let mut pbr = metric_audit(price, data.price.as_ref().and_then(|price| price.date));
    pbr.reasons.extend(price_reasons.clone());
    let mut roe = metric_audit(None, None);
    if let Some(group) = latest_annual {
        pbr.period_start = Some(group.period.0);
        pbr.period_end = Some(group.period.1);
        roe.period_start = Some(group.period.0);
        roe.period_end = Some(group.period.1);
        match group_value(group, scope, "BPS")? {
            Ok((disclosure, bps)) => {
                pbr.inputs.insert("actual_bps".to_string(), bps);
                pbr.source_disclosures
                    .push(disclosure_reference(disclosure, scope));
                pbr.result = calculate_ratio(price, bps, &mut pbr.reasons);
            }
            Err(reason) => pbr.reasons.push(reason),
        }
        match (
            group_value(group, scope, "NP")?,
            group_value(group, scope, "Eq")?,
        ) {
            (Ok((profit_disclosure, profit)), Ok((equity_disclosure, equity))) => {
                roe.inputs.insert("net_income".to_string(), profit);
                roe.inputs.insert("net_assets".to_string(), equity);
                push_source(
                    &mut roe.source_disclosures,
                    disclosure_reference(profit_disclosure, scope),
                );
                push_source(
                    &mut roe.source_disclosures,
                    disclosure_reference(equity_disclosure, scope),
                );
                if equity == Some(0.0) {
                    roe.reasons.push(output::ScoreReasonCode::ZeroDenominator);
                } else {
                    roe.result = calculate_percent(profit, equity);
                    if roe.result.is_none() {
                        roe.reasons.push(output::ScoreReasonCode::InvalidValue);
                    }
                }
            }
            (Err(reason), _) | (_, Err(reason)) => roe.reasons.push(reason),
        }
    } else {
        pbr.reasons
            .push(output::ScoreReasonCode::NoMatchingDisclosure);
        roe.reasons
            .push(output::ScoreReasonCode::NoMatchingDisclosure);
    }
    metrics.insert("pbr".to_string(), pbr);
    metrics.insert("roe".to_string(), roe);

    let mut dividend = metric_audit(price, data.price.as_ref().and_then(|price| price.date));
    dividend.reasons.extend(price_reasons);
    if let Some(period) = target_period {
        dividend.period_start = Some(period.0);
        dividend.period_end = Some(period.1);
        match select_forecast(&eligible, scope, period, "DivAnn")? {
            Ok((disclosure, field)) => {
                let amount = disclosure.numeric_field(&field)?;
                dividend
                    .inputs
                    .insert("forecast_dividend_per_share".to_string(), amount);
                dividend
                    .source_disclosures
                    .push(disclosure_reference(disclosure, scope));
                dividend.result = match (amount, price) {
                    (Some(amount), Some(price)) => validate_value(amount / price * 100.0),
                    (None, _) => {
                        dividend.reasons.push(output::ScoreReasonCode::MissingValue);
                        None
                    }
                    (_, None) => None,
                };
                if amount.is_some() && price.is_some() && dividend.result.is_none() {
                    dividend.reasons.push(output::ScoreReasonCode::InvalidValue);
                }
            }
            Err(reason) => dividend.reasons.push(reason),
        }
    }
    metrics.insert("dividend_yield".to_string(), dividend);

    let mut sales_growth = metric_audit(None, None);
    match sales_growth_from_annual(&annual, scope)? {
        Ok((current, previous, growth)) => {
            sales_growth.period_start = Some(current.period.0);
            sales_growth.period_end = Some(current.period.1);
            sales_growth
                .inputs
                .insert("current_fiscal_year_sales".to_string(), Some(current.value));
            sales_growth.inputs.insert(
                "previous_fiscal_year_sales".to_string(),
                Some(previous.value),
            );
            sales_growth
                .source_disclosures
                .push(disclosure_reference(current.disclosure, scope));
            sales_growth
                .source_disclosures
                .push(disclosure_reference(previous.disclosure, scope));
            sales_growth.result = Some(growth);
        }
        Err((reason, sources)) => {
            sales_growth.reasons.push(reason);
            sales_growth.source_disclosures = sources;
        }
    }
    metrics.insert("sales_growth".to_string(), sales_growth);

    Ok(build_result(
        data,
        scored_at,
        cutoff_at,
        metrics,
        quarterly_actuals,
        policy,
        None,
    ))
}

#[derive(Debug)]
struct AnnualGroup<'a> {
    period: (NaiveDate, NaiveDate),
    disclosures: Vec<&'a FinancialDisclosure>,
}

fn empty_metrics() -> BTreeMap<String, output::MetricAudit> {
    ["per", "pbr", "dividend_yield", "roe", "sales_growth"]
        .into_iter()
        .map(|name| (name.to_string(), metric_audit(None, None)))
        .collect()
}

fn metric_audit(price: Option<f64>, price_date: Option<NaiveDate>) -> output::MetricAudit {
    let mut inputs = BTreeMap::new();
    inputs.insert("adjusted_close".to_string(), price);
    output::MetricAudit {
        inputs,
        price_date,
        period_start: None,
        period_end: None,
        source_disclosures: Vec::new(),
        result: None,
        score_component: None,
        reasons: Vec::new(),
    }
}

fn analysis_reason(metric: &str, code: output::ScoreReasonCode) -> output::ScoreReason {
    output::ScoreReason {
        metric: metric.to_string(),
        code,
    }
}

fn add_reason(
    metrics: &mut BTreeMap<String, output::MetricAudit>,
    metric: &str,
    reason: output::ScoreReasonCode,
) {
    if let Some(audit) = metrics.get_mut(metric) {
        audit.reasons.push(reason);
    }
}

fn attach_global_reasons(
    metrics: &mut BTreeMap<String, output::MetricAudit>,
    reasons: &[output::ScoreReasonCode],
) {
    for audit in metrics.values_mut() {
        audit.reasons.extend(reasons.iter().cloned());
    }
}

fn build_result(
    data: &FetchScoringData,
    scored_at: DateTime<Utc>,
    cutoff_at: NaiveDateTime,
    mut metrics: BTreeMap<String, output::MetricAudit>,
    quarterly_actuals: Vec<output::SingleQuarterActual>,
    policy: ValueScorePolicy,
    error_type: Option<output::ScoreErrorType>,
) -> output::ScoreStock {
    let complete = metrics
        .values()
        .all(|metric| metric.reasons.is_empty() && metric.result.is_some());
    let definitions = [
        ("per", policy.per, ScoreDirection::LowerIsBetter),
        ("pbr", policy.pbr, ScoreDirection::LowerIsBetter),
        (
            "dividend_yield",
            policy.dividend_yield,
            ScoreDirection::HigherIsBetter,
        ),
        ("roe", policy.roe, ScoreDirection::HigherIsBetter),
        (
            "sales_growth",
            policy.sales_growth,
            ScoreDirection::HigherIsBetter,
        ),
    ];
    let mut components = BTreeMap::new();
    for (name, range, direction) in definitions {
        if let Some(audit) = metrics.get_mut(name)
            && let Some(value) = audit.result
        {
            let normalized = normalized_score(value, range, direction);
            let component = output::ComponentDetail {
                raw: value,
                normalized,
                points: normalized * range.weight,
            };
            audit.score_component = Some(component.clone());
            components.insert(name.to_string(), component);
        }
    }
    let reasons = metrics
        .iter()
        .flat_map(|(metric, audit)| {
            audit
                .reasons
                .iter()
                .cloned()
                .map(|code| analysis_reason(metric, code))
        })
        .collect::<Vec<_>>();
    let score = if complete {
        let scoring_metrics = ScoringMetrics {
            per: metrics.get("per").and_then(|metric| metric.result),
            pbr: metrics.get("pbr").and_then(|metric| metric.result),
            dividend_yield: metrics
                .get("dividend_yield")
                .and_then(|metric| metric.result),
            roe: metrics.get("roe").and_then(|metric| metric.result),
            sales_growth: metrics.get("sales_growth").and_then(|metric| metric.result),
        };
        let computation = score_with_details(&scoring_metrics, policy);
        Some(output::Score {
            total: computation.total_score,
            components,
        })
    } else {
        None
    };
    output::ScoreStock {
        code: data.code.clone(),
        scored_at,
        target_date: data.target_date,
        cutoff_at,
        status: if complete {
            output::ScoreStatus::Ok
        } else {
            output::ScoreStatus::Undetermined
        },
        error_type,
        reasons,
        metrics,
        quarterly_actuals,
        score,
    }
}

fn price_input(data: &FetchScoringData) -> (Option<f64>, Vec<output::ScoreReasonCode>) {
    let Some(price) = data.price.as_ref() else {
        return (None, vec![output::ScoreReasonCode::MissingPrice]);
    };
    if price.date != Some(data.target_date) {
        return (None, vec![output::ScoreReasonCode::PriceDateMismatch]);
    }
    let Some(value) = price.adj_close else {
        return (None, vec![output::ScoreReasonCode::MissingPrice]);
    };
    if !value.is_finite() {
        return (None, vec![output::ScoreReasonCode::InvalidValue]);
    }
    if value <= 0.0 {
        return (None, vec![output::ScoreReasonCode::NonPositivePrice]);
    }
    (Some(value), Vec::new())
}

fn calculate_ratio(
    numerator: Option<f64>,
    denominator: Option<f64>,
    reasons: &mut Vec<output::ScoreReasonCode>,
) -> Option<f64> {
    match (numerator, denominator) {
        (None, _) | (_, None) => {
            reasons.push(output::ScoreReasonCode::MissingValue);
            None
        }
        (Some(_), Some(0.0)) => {
            reasons.push(output::ScoreReasonCode::ZeroDenominator);
            None
        }
        (Some(numerator), Some(denominator)) => {
            let result = validate_value(numerator / denominator);
            if result.is_none() {
                reasons.push(output::ScoreReasonCode::InvalidValue);
            }
            result
        }
    }
}

fn calculate_percent(numerator: Option<f64>, denominator: Option<f64>) -> Option<f64> {
    match (numerator, denominator) {
        (Some(numerator), Some(denominator)) if denominator != 0.0 => {
            validate_value(numerator / denominator * 100.0)
        }
        _ => None,
    }
}

fn disclosure_timestamp(disclosure: &FinancialDisclosure) -> Option<NaiveDateTime> {
    Some(
        disclosure
            .disclosed_date?
            .and_time(disclosure.disclosed_time?),
    )
}

fn latest_accounting_scope(
    disclosures: &[&FinancialDisclosure],
) -> std::result::Result<AccountingScope, output::ScoreReasonCode> {
    let statements = disclosures
        .iter()
        .filter(|disclosure| disclosure.document_type.contains("FinancialStatements"))
        .collect::<Vec<_>>();
    let latest_time = statements
        .iter()
        .filter_map(|disclosure| disclosure_timestamp(disclosure))
        .max()
        .ok_or(output::ScoreReasonCode::NoMatchingDisclosure)?;
    let latest = statements
        .into_iter()
        .filter(|disclosure| disclosure_timestamp(disclosure) == Some(latest_time))
        .fold(
            Vec::new(),
            |mut unique: Vec<&FinancialDisclosure>, disclosure| {
                if !unique.contains(disclosure) {
                    unique.push(*disclosure);
                }
                unique
            },
        );
    if latest.len() != 1 {
        return Err(output::ScoreReasonCode::AmbiguousDisclosure);
    }
    match latest[0].accounting_scope {
        AccountingScope::Unknown => Err(output::ScoreReasonCode::UnknownAccountingScope),
        scope => Ok(scope),
    }
}

fn target_fiscal_period(
    disclosures: &[&FinancialDisclosure],
    target_date: NaiveDate,
) -> std::result::Result<(NaiveDate, NaiveDate), output::ScoreReasonCode> {
    let mut periods = BTreeSet::new();
    for disclosure in disclosures {
        let candidates = [
            disclosure
                .current_fiscal_year_start
                .zip(disclosure.current_fiscal_year_end),
            disclosure
                .next_fiscal_year_start
                .zip(disclosure.next_fiscal_year_end),
        ];
        for period in candidates.into_iter().flatten() {
            if period.0 <= period.1 && period.1 >= target_date {
                periods.insert(period);
            }
        }
    }
    let active = periods
        .iter()
        .filter(|(start, end)| *start <= target_date && *end >= target_date)
        .copied()
        .collect::<Vec<_>>();
    if active.len() == 1 {
        return Ok(active[0]);
    }
    if active.len() > 1 {
        return Err(output::ScoreReasonCode::AmbiguousTargetFiscalYear);
    }
    let next_start = periods
        .iter()
        .filter(|(start, _)| *start > target_date)
        .map(|(start, _)| *start)
        .min()
        .ok_or(output::ScoreReasonCode::NoTargetFiscalYear)?;
    let next = periods
        .iter()
        .filter(|(start, _)| *start == next_start)
        .copied()
        .collect::<Vec<_>>();
    if next.len() != 1 {
        return Err(output::ScoreReasonCode::AmbiguousTargetFiscalYear);
    }
    Ok(next[0])
}

fn forecast_prefix(
    disclosure: &FinancialDisclosure,
    scope: AccountingScope,
) -> Option<&'static str> {
    let doc_type = disclosure.document_type.as_str();
    let consolidated = doc_type.contains("_Consolidated_");
    let non_consolidated = doc_type.contains("_NonConsolidated_");
    let forecast_revision = doc_type.contains("EarnForecastRevision");
    match (consolidated, non_consolidated, forecast_revision, scope) {
        (true, false, _, AccountingScope::Consolidated) => Some("F"),
        (true, false, _, AccountingScope::NonConsolidated) => Some("FNC"),
        (false, true, _, AccountingScope::NonConsolidated) => Some("F"),
        (false, false, true, AccountingScope::Consolidated) => Some("F"),
        (false, false, true, AccountingScope::NonConsolidated) => Some("FNC"),
        _ => None,
    }
}

fn forecast_fields(
    disclosure: &FinancialDisclosure,
    scope: AccountingScope,
    metric_suffix: &str,
) -> Vec<(String, (NaiveDate, NaiveDate))> {
    let Some(prefix) = forecast_prefix(disclosure, scope) else {
        return Vec::new();
    };
    let mut fields = Vec::new();
    if let Some(period) = disclosure
        .current_fiscal_year_start
        .zip(disclosure.current_fiscal_year_end)
    {
        fields.push((format!("{prefix}{metric_suffix}"), period));
    }
    if let Some(period) = disclosure
        .next_fiscal_year_start
        .zip(disclosure.next_fiscal_year_end)
    {
        fields.push((format!("Nx{prefix}{metric_suffix}"), period));
    }
    fields
}

fn select_forecast<'a>(
    disclosures: &[&'a FinancialDisclosure],
    scope: AccountingScope,
    period: (NaiveDate, NaiveDate),
    metric_suffix: &str,
) -> Result<std::result::Result<(&'a FinancialDisclosure, String), output::ScoreReasonCode>> {
    let mut candidates = disclosures
        .iter()
        .filter_map(|disclosure| {
            let timestamp = disclosure_timestamp(disclosure)?;
            let field = forecast_fields(disclosure, scope, metric_suffix)
                .into_iter()
                .find(|(_, candidate_period)| *candidate_period == period)?;
            Some((timestamp, *disclosure, field.0))
        })
        .collect::<Vec<_>>();
    let Some(latest_time) = candidates.iter().map(|(time, _, _)| *time).max() else {
        return Ok(Err(output::ScoreReasonCode::NoMatchingDisclosure));
    };
    candidates.retain(|(time, _, _)| *time == latest_time);
    if candidates.len() != 1 {
        return Ok(Err(output::ScoreReasonCode::AmbiguousDisclosure));
    }
    let (_, disclosure, field) = candidates.remove(0);
    Ok(Ok((disclosure, field)))
}

fn annual_groups<'a>(
    disclosures: &[&'a FinancialDisclosure],
    scope: AccountingScope,
    target_date: NaiveDate,
) -> Result<Vec<AnnualGroup<'a>>> {
    let mut periods = BTreeSet::new();
    for disclosure in disclosures {
        if disclosure.current_period_type.as_deref() != Some("FY") {
            continue;
        }
        let (Some(start), Some(end), Some(period_start), Some(period_end)) = (
            disclosure.current_fiscal_year_start,
            disclosure.current_fiscal_year_end,
            disclosure.current_period_start,
            disclosure.current_period_end,
        ) else {
            continue;
        };
        if disclosure.accounting_scope == AccountingScope::Unknown {
            continue;
        }
        let scope_matches = disclosure.accounting_scope == scope;
        if scope_matches
            && start <= end
            && period_start == start
            && period_end == end
            && end <= target_date
        {
            periods.insert((start, end));
        }
    }
    let mut groups = Vec::new();
    for period in periods {
        let candidates = disclosures
            .iter()
            .copied()
            .filter(|disclosure| {
                disclosure.current_period_type.as_deref() == Some("FY")
                    && disclosure.current_fiscal_year_start == Some(period.0)
                    && disclosure.current_fiscal_year_end == Some(period.1)
                    && disclosure.current_period_start == Some(period.0)
                    && disclosure.current_period_end == Some(period.1)
                    && disclosure.accounting_scope != AccountingScope::Unknown
                    && disclosure.accounting_scope == scope
            })
            .collect::<Vec<_>>();
        let latest_time = candidates
            .iter()
            .filter_map(|disclosure| disclosure_timestamp(disclosure))
            .max()
            .context("annual disclosure timestamp is missing")?;
        let latest = candidates
            .into_iter()
            .filter(|disclosure| disclosure_timestamp(disclosure) == Some(latest_time))
            .collect::<Vec<_>>();
        groups.push(AnnualGroup {
            period,
            disclosures: latest,
        });
    }
    groups.sort_by(|left, right| right.period.1.cmp(&left.period.1));
    Ok(groups)
}

fn actual_field(
    disclosure: &FinancialDisclosure,
    scope: AccountingScope,
    metric: &str,
) -> Option<String> {
    let field = match metric {
        "Sales" => "Sales",
        "OP" => "OP",
        "OdP" => "OdP",
        "NP" => "NP",
        "EPS" => "EPS",
        "BPS" => "BPS",
        "Eq" => "Eq",
        _ => return None,
    };
    if disclosure.accounting_scope == scope {
        Some(field.to_string())
    } else {
        None
    }
}

fn group_value<'a>(
    group: &'a AnnualGroup<'a>,
    scope: AccountingScope,
    metric: &str,
) -> Result<std::result::Result<(&'a FinancialDisclosure, Option<f64>), output::ScoreReasonCode>> {
    if group.disclosures.len() != 1 {
        return Ok(Err(output::ScoreReasonCode::AmbiguousDisclosure));
    }
    let disclosure = group.disclosures[0];
    let Some(field) = actual_field(disclosure, scope, metric) else {
        return Ok(Err(output::ScoreReasonCode::AccountingScopeMismatch));
    };
    let value = disclosure.numeric_field(&field)?;
    if value.is_none() {
        return Ok(Err(output::ScoreReasonCode::MissingValue));
    }
    Ok(Ok((disclosure, value)))
}

#[derive(Clone, Copy)]
struct AnnualValue<'a> {
    period: (NaiveDate, NaiveDate),
    disclosure: &'a FinancialDisclosure,
    value: f64,
}

type SalesGrowthResult<'a> = Result<
    std::result::Result<
        (AnnualValue<'a>, AnnualValue<'a>, f64),
        (output::ScoreReasonCode, Vec<output::DisclosureReference>),
    >,
>;

fn sales_growth_from_annual<'a>(
    groups: &'a [AnnualGroup<'a>],
    scope: AccountingScope,
) -> SalesGrowthResult<'a> {
    let mut values = Vec::new();
    for group in groups.iter().take(2) {
        match group_value(group, scope, "Sales")? {
            Ok((disclosure, Some(value))) => values.push(AnnualValue {
                period: group.period,
                disclosure,
                value,
            }),
            Ok((_, None)) => return Ok(Err((output::ScoreReasonCode::MissingValue, Vec::new()))),
            Err(reason) => return Ok(Err((reason, Vec::new()))),
        }
    }
    if values.len() < 2 {
        return Ok(Err((
            output::ScoreReasonCode::NoPriorFiscalYear,
            Vec::new(),
        )));
    }
    let current = values[0];
    let previous = values[1];
    if current.period.0 != previous.period.1 + Duration::days(1)
        || (current.period.1 - current.period.0)
            .num_days()
            .abs_diff((previous.period.1 - previous.period.0).num_days())
            > 1
    {
        let sources = vec![
            disclosure_reference(current.disclosure, scope),
            disclosure_reference(previous.disclosure, scope),
        ];
        return Ok(Err((
            output::ScoreReasonCode::IncompatibleFiscalPeriods,
            sources,
        )));
    }
    if previous.value <= 0.0 {
        return Ok(Err((output::ScoreReasonCode::ZeroDenominator, Vec::new())));
    }
    let growth = validate_value((current.value - previous.value) / previous.value * 100.0)
        .ok_or_else(|| anyhow::anyhow!("sales growth is not finite"))?;
    Ok(Ok((current, previous, growth)))
}

fn disclosure_reference(
    disclosure: &FinancialDisclosure,
    value_scope: AccountingScope,
) -> output::DisclosureReference {
    output::DisclosureReference {
        code: disclosure.code.clone(),
        disclosure_number: disclosure.disclosure_number.clone(),
        disclosed_date: disclosure.disclosed_date,
        disclosed_time: disclosure.disclosed_time,
        document_type: disclosure.document_type.clone(),
        document_scope: disclosure.accounting_scope,
        value_scope,
        current_period_type: disclosure.current_period_type.clone(),
        current_period_start: disclosure.current_period_start,
        current_period_end: disclosure.current_period_end,
        current_fiscal_year_start: disclosure.current_fiscal_year_start,
        current_fiscal_year_end: disclosure.current_fiscal_year_end,
    }
}

fn push_source(
    sources: &mut Vec<output::DisclosureReference>,
    source: output::DisclosureReference,
) {
    if !sources.contains(&source) {
        sources.push(source);
    }
}

fn skipped_result(
    code: String,
    scored_at: DateTime<Utc>,
    target_date: NaiveDate,
    cutoff_at: NaiveDateTime,
) -> output::ScoreStock {
    output::ScoreStock {
        code,
        scored_at,
        target_date,
        cutoff_at,
        status: output::ScoreStatus::Skipped,
        error_type: Some(output::ScoreErrorType::InvalidInput),
        reasons: vec![analysis_reason(
            "history",
            output::ScoreReasonCode::IncompleteHistory,
        )],
        metrics: BTreeMap::new(),
        quarterly_actuals: Vec::new(),
        score: None,
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
    use std::collections::BTreeMap;

    use crate::domain::models::scoring::model::ValueScorePolicy;
    use crate::domain::models::statement::model::{
        AccountingScope, FinancialDisclosure, OperatingProfitPeriod,
    };
    use crate::presenter::presenters::fetch_scoring_data::output::{
        FetchCompany, FetchPrice, FetchScoringData, FetchStatus,
    };
    use crate::presenter::presenters::score_stock::output::{
        ComponentDetail, FiscalQuarter, ScoreErrorType, ScoreReasonCode, ScoreStatus,
    };
    use crate::use_case::interfaces::score_stock::input;
    use crate::use_case::interfaces::score_stock::use_case::ScoreStock as _;

    use super::ScoreStock;

    // テスト用の固定時刻定数
    fn fixed_now() -> DateTime<Utc> {
        "2026-08-26T18:00:00Z".parse().unwrap()
    }

    fn date(value: &str) -> NaiveDate {
        value.parse().unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    fn make_disclosure(
        disclosed_date: Option<&str>,
        disclosed_time: Option<&str>,
        disclosure_number: &str,
        document_type: &str,
        period_type: Option<&str>,
        fiscal_year: Option<(&str, &str)>,
        period: Option<(&str, &str)>,
        values: &[(&str, &str)],
    ) -> FinancialDisclosure {
        FinancialDisclosure {
            code: "13010".to_string(),
            disclosed_date: disclosed_date.map(date),
            disclosed_time: disclosed_time.map(|value| value.parse::<NaiveTime>().unwrap()),
            disclosure_number: disclosure_number.to_string(),
            document_type: document_type.to_string(),
            accounting_scope: AccountingScope::Consolidated,
            current_period_type: period_type.map(str::to_string),
            current_period_start: period.map(|period| date(period.0)),
            current_period_end: period.map(|period| date(period.1)),
            current_fiscal_year_start: fiscal_year.map(|period| date(period.0)),
            current_fiscal_year_end: fiscal_year.map(|period| date(period.1)),
            next_fiscal_year_start: None,
            next_fiscal_year_end: None,
            operating_profit: None,
            operating_profit_period: OperatingProfitPeriod::Unspecified,
            forecast_operating_profit: None,
            next_forecast_operating_profit: None,
            forecast_operating_profit_2q: None,
            next_forecast_operating_profit_2q: None,
            non_consolidated_operating_profit: None,
            non_consolidated_forecast_operating_profit: None,
            non_consolidated_next_forecast_operating_profit: None,
            non_consolidated_forecast_operating_profit_2q: None,
            non_consolidated_next_forecast_operating_profit_2q: None,
            source_fields: values
                .iter()
                .map(|(key, value)| (key.to_string(), Some(value.to_string())))
                .collect::<BTreeMap<_, _>>(),
        }
    }

    fn annual_disclosure(
        fiscal_year: (&str, &str),
        disclosed_date: &str,
        sales: &str,
        bps: Option<&str>,
        net_income: Option<&str>,
        equity: Option<&str>,
    ) -> FinancialDisclosure {
        let mut values = vec![("Sales", sales)];
        if let Some(bps) = bps {
            values.push(("BPS", bps));
        }
        if let Some(net_income) = net_income {
            values.push(("NP", net_income));
        }
        if let Some(equity) = equity {
            values.push(("Eq", equity));
        }
        make_disclosure(
            Some(disclosed_date),
            Some("15:00:00"),
            "1",
            "FYFinancialStatements_Consolidated_JP",
            Some("FY"),
            Some(fiscal_year),
            Some(fiscal_year),
            &values,
        )
    }

    fn forecast_disclosure(
        disclosed_date: &str,
        disclosed_time: &str,
        disclosure_number: &str,
        eps: &str,
        dividend: &str,
    ) -> FinancialDisclosure {
        make_disclosure(
            Some(disclosed_date),
            Some(disclosed_time),
            disclosure_number,
            "3QFinancialStatements_Consolidated_JP",
            Some("3Q"),
            Some(("2024-04-01", "2025-03-31")),
            Some(("2024-04-01", "2024-12-31")),
            &[("FEPS", eps), ("FDivAnn", dividend)],
        )
    }

    fn build_ok_data() -> FetchScoringData {
        FetchScoringData {
            code: "1301".to_string(),
            target_date: date("2025-03-31"),
            fetched_at: fixed_now(),
            status: FetchStatus::Ok,
            error_type: None,
            history_error: None,
            company: Some(FetchCompany {
                name: Some("Test Company".to_string()),
            }),
            price: Some(FetchPrice {
                date: Some(date("2025-03-31")),
                close: Some(100.0),
                adj_close: Some(100.0),
            }),
            latest_statement: None,
            full_year_sales: vec![],
            financial_disclosures: vec![
                annual_disclosure(
                    ("2023-04-01", "2024-03-31"),
                    "2024-05-10",
                    "115",
                    Some("200"),
                    Some("20"),
                    Some("100"),
                ),
                annual_disclosure(
                    ("2022-04-01", "2023-03-31"),
                    "2023-05-10",
                    "100",
                    None,
                    None,
                    None,
                ),
                forecast_disclosure("2025-02-10", "15:00:00", "2", "20", "5"),
            ],
        }
    }

    fn build_input(data: FetchScoringData, policy: ValueScorePolicy) -> input::ScoreStock {
        input::ScoreStock::new(data, policy, fixed_now())
    }

    #[tokio::test]
    async fn handle_returns_undetermined_when_fetch_status_is_not_ok() {
        let mut data = build_ok_data();
        data.status = FetchStatus::IncompleteHistory;

        let actual = ScoreStock::new()
            .handle(build_input(data, ValueScorePolicy::standard()))
            .await
            .unwrap();

        assert_eq!(actual.code, "1301");
        assert_eq!(actual.status, ScoreStatus::Undetermined);
        assert_eq!(actual.error_type, Some(ScoreErrorType::IncompleteHistory));
        assert!(actual.score.is_none());
        assert!(
            actual
                .reasons
                .iter()
                .any(|reason| { reason.code == ScoreReasonCode::IncompleteHistory })
        );
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

    #[tokio::test]
    async fn disclosure_at_cutoff_is_used_but_later_disclosures_are_ignored() {
        let mut data = build_ok_data();
        data.financial_disclosures.push(forecast_disclosure(
            "2025-03-31",
            "15:30:00",
            "3",
            "20",
            "5",
        ));
        data.financial_disclosures.push(forecast_disclosure(
            "2025-03-31",
            "15:30:01",
            "4",
            "10",
            "1",
        ));

        let actual = ScoreStock::new()
            .handle(build_input(data, ValueScorePolicy::standard()))
            .await
            .unwrap();

        assert_eq!(actual.status, ScoreStatus::Ok);
        assert_eq!(actual.cutoff_at.to_string(), "2025-03-31 15:30:00");
        assert_eq!(actual.metrics["per"].result, Some(5.0));
        assert_eq!(actual.metrics["dividend_yield"].result, Some(5.0));
        assert_eq!(
            actual.metrics["per"].source_disclosures[0].disclosure_number,
            "3"
        );
    }

    #[tokio::test]
    async fn same_timestamp_forecast_disclosures_are_ambiguous() {
        let mut data = build_ok_data();
        data.financial_disclosures.push(forecast_disclosure(
            "2025-02-10",
            "15:00:00",
            "3",
            "10",
            "1",
        ));

        let actual = ScoreStock::new()
            .handle(build_input(data, ValueScorePolicy::standard()))
            .await
            .unwrap();

        assert_eq!(actual.status, ScoreStatus::Undetermined);
        assert!(actual.score.is_none());
        assert!(
            actual
                .reasons
                .iter()
                .any(|reason| { reason.code == ScoreReasonCode::AmbiguousDisclosure })
        );
    }

    #[tokio::test]
    async fn unknown_disclosure_timestamp_makes_history_incomplete() {
        let mut data = build_ok_data();
        data.financial_disclosures.push(make_disclosure(
            None,
            None,
            "5",
            "FYFinancialStatements_Consolidated_JP",
            Some("FY"),
            None,
            None,
            &[],
        ));

        let actual = ScoreStock::new()
            .handle(build_input(data, ValueScorePolicy::standard()))
            .await
            .unwrap();

        assert_eq!(actual.status, ScoreStatus::Undetermined);
        assert!(actual.score.is_none());
        assert!(
            actual
                .reasons
                .iter()
                .any(|reason| { reason.code == ScoreReasonCode::UnknownDisclosureTime })
        );
    }

    #[tokio::test]
    async fn missing_forecast_input_does_not_become_a_zero_score() {
        let mut data = build_ok_data();
        data.financial_disclosures[2] = forecast_disclosure("2025-02-10", "15:00:00", "2", "", "5");

        let actual = ScoreStock::new()
            .handle(build_input(data, ValueScorePolicy::standard()))
            .await
            .unwrap();

        assert_eq!(actual.status, ScoreStatus::Undetermined);
        assert!(actual.score.is_none());
        assert!(
            actual.metrics["per"]
                .reasons
                .contains(&ScoreReasonCode::MissingValue)
        );
    }

    #[test]
    fn cutoff_tracks_the_exchange_close_schedule() {
        assert_eq!(
            super::analysis_cutoff(date("2024-11-04")).time(),
            "15:00:00".parse::<NaiveTime>().unwrap()
        );
        assert_eq!(
            super::analysis_cutoff(date("2024-11-05")).time(),
            "15:30:00".parse::<NaiveTime>().unwrap()
        );
    }

    #[test]
    fn cumulative_actuals_are_differenced_only_with_adjacent_compatible_periods() {
        let fiscal_year = ("2025-04-01", "2026-03-31");
        let disclosures = vec![
            make_disclosure(
                Some("2025-08-07"),
                Some("14:00:00"),
                "1",
                "1QFinancialStatements_Consolidated_IFRS",
                Some("1Q"),
                Some(fiscal_year),
                Some(("2025-04-01", "2025-06-30")),
                &[("Sales", "10"), ("OP", "4"), ("OdP", "3"), ("NP", "0")],
            ),
            make_disclosure(
                Some("2025-11-05"),
                Some("14:00:00"),
                "2",
                "2QFinancialStatements_Consolidated_IFRS",
                Some("2Q"),
                Some(fiscal_year),
                Some(("2025-04-01", "2025-09-30")),
                &[("Sales", "25"), ("OP", "8"), ("OdP", "6"), ("NP", "4")],
            ),
            make_disclosure(
                Some("2026-02-06"),
                Some("14:00:00"),
                "3",
                "3QFinancialStatements_Consolidated_IFRS",
                Some("3Q"),
                Some(fiscal_year),
                Some(("2025-04-01", "2025-12-31")),
                &[("Sales", "40"), ("OP", "12"), ("OdP", "9"), ("NP", "7")],
            ),
            make_disclosure(
                Some("2026-05-08"),
                Some("14:00:00"),
                "4",
                "FYFinancialStatements_Consolidated_IFRS",
                Some("FY"),
                Some(fiscal_year),
                Some(fiscal_year),
                &[("Sales", "60"), ("OP", "16"), ("OdP", "12"), ("NP", "10")],
            ),
        ];
        let references = disclosures.iter().collect::<Vec<_>>();

        let actuals = super::derive_single_quarter_actuals(&references).unwrap();

        assert_eq!(actuals.len(), 4);
        assert_eq!(actuals[0].quarter, FiscalQuarter::Q1);
        assert_eq!(actuals[0].values["net_income"].result, Some(0.0));
        assert_eq!(actuals[1].quarter, FiscalQuarter::Q2);
        assert_eq!(actuals[1].period_start, Some(date("2025-07-01")));
        assert_eq!(
            actuals[1].values["sales"].current_cumulative_input,
            Some(25.0)
        );
        assert_eq!(
            actuals[1].values["sales"].prior_cumulative_input,
            Some(10.0)
        );
        assert_eq!(actuals[1].values["sales"].result, Some(15.0));
        assert_eq!(actuals[2].values["operating_profit"].result, Some(4.0));
        assert_eq!(actuals[3].quarter, FiscalQuarter::Q4);
        assert_eq!(actuals[3].values["sales"].result, Some(20.0));
        assert_eq!(actuals[3].values["sales"].source_disclosures.len(), 2);
    }

    #[test]
    fn same_timestamp_quarter_reports_are_not_ordered_by_disclosure_number() {
        let fiscal_year = ("2025-04-01", "2026-03-31");
        let first = make_disclosure(
            Some("2025-08-07"),
            Some("14:00:00"),
            "10",
            "1QFinancialStatements_Consolidated_IFRS",
            Some("1Q"),
            Some(fiscal_year),
            Some(("2025-04-01", "2025-06-30")),
            &[("Sales", "10")],
        );
        let competing = make_disclosure(
            Some("2025-08-07"),
            Some("14:00:00"),
            "11",
            "1QFinancialStatements_Consolidated_IFRS",
            Some("1Q"),
            Some(fiscal_year),
            Some(("2025-04-01", "2025-06-30")),
            &[("Sales", "12")],
        );
        let second = make_disclosure(
            Some("2025-11-05"),
            Some("14:00:00"),
            "12",
            "2QFinancialStatements_Consolidated_IFRS",
            Some("2Q"),
            Some(fiscal_year),
            Some(("2025-04-01", "2025-09-30")),
            &[("Sales", "25")],
        );
        let disclosures = [first, competing, second];
        let references = disclosures.iter().collect::<Vec<_>>();

        let actuals = super::derive_single_quarter_actuals(&references).unwrap();

        assert!(actuals[0].values["sales"].result.is_none());
        assert!(
            actuals[0].values["sales"]
                .reasons
                .contains(&ScoreReasonCode::AmbiguousDisclosure)
        );
        assert!(
            actuals[1].values["sales"]
                .reasons
                .contains(&ScoreReasonCode::AmbiguousDisclosure)
        );
    }

    #[test]
    fn unknown_accounting_scope_does_not_select_an_operating_profit_field() {
        let mut disclosure = make_disclosure(
            Some("2025-08-07"),
            Some("14:00:00"),
            "1",
            "1QFinancialStatements_Unknown_IFRS",
            Some("1Q"),
            Some(("2025-04-01", "2026-03-31")),
            Some(("2025-04-01", "2025-06-30")),
            &[("OP", "4"), ("NCOP", "5")],
        );
        disclosure.accounting_scope = AccountingScope::Unknown;
        let actuals = super::derive_single_quarter_actuals(&[&disclosure]).unwrap();
        let operating_profit = &actuals[0].values["operating_profit"];

        assert_eq!(operating_profit.current_cumulative_input, None);
        assert_eq!(operating_profit.result, None);
        assert!(
            operating_profit
                .reasons
                .contains(&ScoreReasonCode::UnknownAccountingScope)
        );
    }
}
