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

fn build_financial_forecast_audit(
    data: &FetchScoringData,
    cutoff_at: NaiveDateTime,
) -> output::FinancialForecastAudit {
    let mut history_incomplete = data.status != FetchStatus::Ok
        || data.financial_disclosures.is_empty()
        || data
            .financial_disclosures
            .iter()
            .any(|disclosure| !disclosure.matches_code(&data.code));
    let mut reasons = Vec::new();
    if history_incomplete {
        push_forecast_reason(&mut reasons, output::ForecastReasonCode::IncompleteHistory);
    }
    if data.financial_disclosures.iter().any(|disclosure| {
        disclosure.disclosed_date.is_none() || disclosure.disclosed_time.is_none()
    }) {
        history_incomplete = true;
        push_forecast_reason(
            &mut reasons,
            output::ForecastReasonCode::UnknownDisclosureTime,
        );
        push_forecast_reason(&mut reasons, output::ForecastReasonCode::IncompleteHistory);
    }

    let mut candidates = Vec::new();
    for disclosure in &data.financial_disclosures {
        if !disclosure.matches_code(&data.code)
            || !disclosure.document_type.contains("FinancialStatements")
                && !disclosure.document_type.contains("EarnForecastRevision")
        {
            continue;
        }
        let timestamp = disclosure_timestamp(disclosure);
        if timestamp.is_some_and(|time| time > cutoff_at) {
            continue;
        }

        let mut scope_by_prefix = BTreeMap::<String, AccountingScope>::new();
        for scope in [
            AccountingScope::Consolidated,
            AccountingScope::NonConsolidated,
        ] {
            if let Some(prefix) = forecast_prefix(disclosure, scope) {
                scope_by_prefix.insert(prefix.to_string(), scope);
            }
        }

        for prefix in ["F", "FNC"] {
            let scope = scope_by_prefix
                .get(prefix)
                .copied()
                .unwrap_or(AccountingScope::Unknown);
            for metric in [
                output::ForecastMetric::Sales,
                output::ForecastMetric::OperatingProfit,
                output::ForecastMetric::OrdinaryProfit,
                output::ForecastMetric::NetIncome,
            ] {
                let suffix = forecast_metric_suffix(metric);
                for year_reference in [
                    output::ForecastYearReference::Current,
                    output::ForecastYearReference::Next,
                ] {
                    let period = forecast_period(disclosure, year_reference);
                    for horizon in [
                        output::ForecastHorizon::FullYear,
                        output::ForecastHorizon::SecondQuarter,
                    ] {
                        if horizon == output::ForecastHorizon::SecondQuarter
                            && interim_actual_period(disclosure).is_some_and(|actual_period| {
                                scope == disclosure.accounting_scope
                                    && period == Some(actual_period)
                            })
                        {
                            continue;
                        }
                        let field = forecast_field_name(prefix, suffix, year_reference, horizon);
                        let source_value = disclosure.source_fields.get(&field);
                        if source_value.is_none() && scope == AccountingScope::Unknown {
                            continue;
                        }
                        let mut candidate_reasons = Vec::new();
                        if scope == AccountingScope::Unknown {
                            push_forecast_reason(
                                &mut candidate_reasons,
                                output::ForecastReasonCode::UnknownAccountingScope,
                            );
                        }
                        match period {
                            None => push_forecast_reason(
                                &mut candidate_reasons,
                                output::ForecastReasonCode::UnknownFiscalPeriod,
                            ),
                            Some((start, end)) if start > end => push_forecast_reason(
                                &mut candidate_reasons,
                                output::ForecastReasonCode::IncompatibleFiscalPeriod,
                            ),
                            _ => {}
                        }
                        if timestamp.is_none() {
                            push_forecast_reason(
                                &mut candidate_reasons,
                                output::ForecastReasonCode::UnknownDisclosureTime,
                            );
                            push_forecast_reason(
                                &mut candidate_reasons,
                                output::ForecastReasonCode::IncompleteHistory,
                            );
                        }
                        match source_value {
                            None => push_forecast_reason(
                                &mut candidate_reasons,
                                output::ForecastReasonCode::MissingSourceField,
                            ),
                            Some(None) => push_forecast_reason(
                                &mut candidate_reasons,
                                output::ForecastReasonCode::NullSourceField,
                            ),
                            Some(Some(value)) if value.is_empty() => push_forecast_reason(
                                &mut candidate_reasons,
                                output::ForecastReasonCode::MissingValue,
                            ),
                            _ => {}
                        }

                        let raw_value = source_value.cloned().flatten();
                        let value = match disclosure.numeric_field(&field) {
                            Ok(value) => value,
                            Err(_) => {
                                history_incomplete = true;
                                push_forecast_reason(
                                    &mut reasons,
                                    output::ForecastReasonCode::IncompleteHistory,
                                );
                                push_forecast_reason(
                                    &mut candidate_reasons,
                                    output::ForecastReasonCode::InvalidValue,
                                );
                                push_forecast_reason(
                                    &mut candidate_reasons,
                                    output::ForecastReasonCode::IncompleteHistory,
                                );
                                None
                            }
                        };
                        candidates.push(ForecastCandidate {
                            fiscal_year_start: period.map(|value| value.0),
                            fiscal_year_end: period.map(|value| value.1),
                            year_reference: Some(year_reference),
                            horizon,
                            accounting_scope: scope,
                            source_field_family: Some(if prefix == "FNC" {
                                output::ForecastFieldFamily::NonConsolidatedPrefix
                            } else {
                                output::ForecastFieldFamily::Unprefixed
                            }),
                            metric,
                            source_field: field,
                            disclosure,
                            timestamp,
                            value,
                            raw_value,
                            reasons: candidate_reasons,
                        });
                    }
                }
            }
        }
    }

    let known_values = candidates
        .iter()
        .filter(|candidate| candidate.value.is_some())
        .map(|candidate| {
            (
                candidate.fiscal_year_start,
                candidate.fiscal_year_end,
                candidate.year_reference,
                candidate.horizon,
                candidate.accounting_scope,
                candidate.metric,
                candidate.source_field.clone(),
                candidate.timestamp,
            )
        })
        .collect::<Vec<_>>();
    candidates.retain(|candidate| {
        if !candidate
            .reasons
            .contains(&output::ForecastReasonCode::MissingSourceField)
        {
            return true;
        }
        known_values.iter().any(
            |(start, end, year_reference, horizon, scope, metric, field, timestamp)| {
                *start == candidate.fiscal_year_start
                    && *end == candidate.fiscal_year_end
                    && *year_reference == candidate.year_reference
                    && *horizon == candidate.horizon
                    && *scope == candidate.accounting_scope
                    && *metric == candidate.metric
                    && field == &candidate.source_field
                    && match (*timestamp, candidate.timestamp) {
                        (Some(previous), Some(current)) => previous < current,
                        _ => true,
                    }
            },
        )
    });

    let interim_actual_periods = data
        .financial_disclosures
        .iter()
        .filter(|disclosure| {
            disclosure.matches_code(&data.code)
                && interim_actual_period(disclosure).is_some()
                && disclosure_timestamp(disclosure).is_some_and(|time| time <= cutoff_at)
        })
        .filter_map(|disclosure| {
            let (start, end) = interim_actual_period(disclosure)?;
            Some((start, end, disclosure.accounting_scope))
        })
        .collect::<BTreeSet<_>>();

    let mut grouped = BTreeMap::<
        (
            NaiveDate,
            NaiveDate,
            AccountingScope,
            output::ForecastHorizon,
            output::ForecastMetric,
        ),
        Vec<ForecastCandidate<'_>>,
    >::new();
    let mut forecasts = Vec::new();
    for candidate in candidates {
        match (
            candidate.fiscal_year_start,
            candidate.fiscal_year_end,
            candidate.accounting_scope,
        ) {
            (Some(start), Some(end), scope)
                if start <= end && scope != AccountingScope::Unknown =>
            {
                grouped
                    .entry((start, end, scope, candidate.horizon, candidate.metric))
                    .or_default()
                    .push(candidate);
            }
            _ => forecasts.push(undetermined_forecast(candidate, history_incomplete)),
        }
    }

    for ((start, end, scope, horizon, metric), mut observations) in grouped {
        observations.sort_by_key(|candidate| candidate.timestamp);
        let unknown_timestamp = observations
            .iter()
            .any(|candidate| candidate.timestamp.is_none());
        let latest_time = observations
            .iter()
            .filter_map(|candidate| candidate.timestamp)
            .max();
        let latest = latest_time.map(|time| {
            observations
                .iter()
                .enumerate()
                .filter_map(|(index, candidate)| {
                    (candidate.timestamp == Some(time)).then_some(index)
                })
                .collect::<Vec<_>>()
        });
        let ambiguous = latest.as_ref().is_some_and(|indices| indices.len() > 1);
        let selected_index = latest
            .as_ref()
            .and_then(|indices| (indices.len() == 1).then_some(indices[0]));
        let selected = selected_index.map(|index| &observations[index]);
        let period_ended = end < data.target_date;
        let interim_reported = horizon == output::ForecastHorizon::SecondQuarter
            && interim_actual_periods.contains(&(start, end, scope));
        let mut forecast_reasons = Vec::new();
        if history_incomplete || unknown_timestamp {
            push_forecast_reason(
                &mut forecast_reasons,
                output::ForecastReasonCode::IncompleteHistory,
            );
        }
        if ambiguous {
            push_forecast_reason(
                &mut forecast_reasons,
                output::ForecastReasonCode::AmbiguousDisclosure,
            );
        }
        if period_ended {
            push_forecast_reason(
                &mut forecast_reasons,
                output::ForecastReasonCode::FiscalPeriodEnded,
            );
        }
        if interim_reported {
            push_forecast_reason(
                &mut forecast_reasons,
                output::ForecastReasonCode::InterimActualReported,
            );
        }
        if let Some(candidate) = selected {
            for reason in &candidate.reasons {
                push_forecast_reason(&mut forecast_reasons, reason.clone());
            }
        }

        let status = if history_incomplete || unknown_timestamp {
            output::ForecastStatus::IncompleteHistory
        } else if ambiguous {
            output::ForecastStatus::Ambiguous
        } else if period_ended || interim_reported {
            output::ForecastStatus::Historical
        } else if selected.is_some_and(|candidate| candidate.value.is_some()) {
            output::ForecastStatus::Selected
        } else {
            output::ForecastStatus::Undetermined
        };
        if status == output::ForecastStatus::Undetermined && selected.is_none() {
            push_forecast_reason(
                &mut forecast_reasons,
                output::ForecastReasonCode::NoMatchingDisclosure,
            );
        }

        let history = observations
            .iter()
            .enumerate()
            .map(|(index, candidate)| {
                let observation_status = if history_incomplete || unknown_timestamp {
                    output::ForecastStatus::IncompleteHistory
                } else if ambiguous
                    && latest
                        .as_ref()
                        .is_some_and(|indices| indices.contains(&index))
                {
                    output::ForecastStatus::Ambiguous
                } else if Some(index) == selected_index {
                    status
                } else {
                    output::ForecastStatus::Historical
                };
                let extra_reason = if observation_status == output::ForecastStatus::Ambiguous {
                    Some(output::ForecastReasonCode::AmbiguousDisclosure)
                } else if observation_status == output::ForecastStatus::IncompleteHistory {
                    Some(output::ForecastReasonCode::IncompleteHistory)
                } else if Some(index) == selected_index && period_ended {
                    Some(output::ForecastReasonCode::FiscalPeriodEnded)
                } else if Some(index) == selected_index && interim_reported {
                    Some(output::ForecastReasonCode::InterimActualReported)
                } else {
                    None
                };
                forecast_observation(candidate, observation_status, extra_reason)
            })
            .collect();
        let chosen = if status == output::ForecastStatus::Selected
            || status == output::ForecastStatus::Historical
            || status == output::ForecastStatus::Undetermined
        {
            selected
        } else {
            None
        };
        forecasts.push(output::FinancialForecast {
            fiscal_year_start: Some(start),
            fiscal_year_end: Some(end),
            year_reference: chosen.and_then(|candidate| candidate.year_reference),
            horizon,
            accounting_scope: scope,
            source_field_family: chosen.and_then(|candidate| candidate.source_field_family),
            metric,
            status,
            value: chosen.and_then(|candidate| candidate.value),
            raw_value: chosen.and_then(|candidate| candidate.raw_value.clone()),
            source_field: chosen.map(|candidate| candidate.source_field.clone()),
            source_disclosure: chosen.map(|candidate| {
                disclosure_reference(candidate.disclosure, candidate.accounting_scope)
            }),
            reasons: forecast_reasons,
            history,
        });
    }

    forecasts.sort_by_key(|forecast| {
        (
            forecast.fiscal_year_start,
            forecast.accounting_scope,
            forecast.horizon,
            forecast.metric,
        )
    });
    for forecast in &forecasts {
        for reason in &forecast.reasons {
            push_forecast_reason(&mut reasons, reason.clone());
        }
    }
    if forecasts.is_empty() {
        push_forecast_reason(
            &mut reasons,
            output::ForecastReasonCode::NoMatchingDisclosure,
        );
    }
    let history_status = if history_incomplete {
        output::ForecastHistoryStatus::Incomplete
    } else {
        output::ForecastHistoryStatus::Available
    };
    output::FinancialForecastAudit {
        target_date: data.target_date,
        cutoff_at,
        history_status,
        reasons,
        forecasts,
    }
}

fn forecast_observation(
    candidate: &ForecastCandidate<'_>,
    status: output::ForecastStatus,
    extra_reason: Option<output::ForecastReasonCode>,
) -> output::ForecastObservation {
    let mut reasons = candidate.reasons.clone();
    if let Some(reason) = extra_reason {
        push_forecast_reason(&mut reasons, reason);
    }
    output::ForecastObservation {
        fiscal_year_start: candidate.fiscal_year_start,
        fiscal_year_end: candidate.fiscal_year_end,
        year_reference: candidate.year_reference,
        value: candidate.value,
        raw_value: candidate.raw_value.clone(),
        source_field: candidate.source_field.clone(),
        source_disclosure: disclosure_reference(candidate.disclosure, candidate.accounting_scope),
        status,
        reasons,
    }
}

fn undetermined_forecast(
    candidate: ForecastCandidate<'_>,
    history_incomplete: bool,
) -> output::FinancialForecast {
    let mut reasons = candidate.reasons.clone();
    let status = if history_incomplete || candidate.timestamp.is_none() {
        push_forecast_reason(&mut reasons, output::ForecastReasonCode::IncompleteHistory);
        output::ForecastStatus::IncompleteHistory
    } else {
        output::ForecastStatus::Undetermined
    };
    output::FinancialForecast {
        fiscal_year_start: candidate.fiscal_year_start,
        fiscal_year_end: candidate.fiscal_year_end,
        year_reference: candidate.year_reference,
        horizon: candidate.horizon,
        accounting_scope: candidate.accounting_scope,
        source_field_family: candidate.source_field_family,
        metric: candidate.metric,
        status,
        value: if status == output::ForecastStatus::IncompleteHistory {
            None
        } else {
            candidate.value
        },
        raw_value: candidate.raw_value.clone(),
        source_field: Some(candidate.source_field.clone()),
        source_disclosure: Some(disclosure_reference(
            candidate.disclosure,
            candidate.accounting_scope,
        )),
        reasons: reasons.clone(),
        history: vec![forecast_observation(&candidate, status, None)],
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
    let financial_forecasts = build_financial_forecast_audit(data, cutoff_at);
    if financial_forecasts.history_status == output::ForecastHistoryStatus::Incomplete {
        for metric in metrics.values_mut() {
            if !metric
                .reasons
                .contains(&output::ScoreReasonCode::IncompleteHistory)
            {
                metric
                    .reasons
                    .push(output::ScoreReasonCode::IncompleteHistory);
            }
        }
    }
    let complete = financial_forecasts.history_status != output::ForecastHistoryStatus::Incomplete
        && metrics
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
        financial_forecasts,
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

#[derive(Debug, Clone)]
struct ForecastCandidate<'a> {
    fiscal_year_start: Option<NaiveDate>,
    fiscal_year_end: Option<NaiveDate>,
    year_reference: Option<output::ForecastYearReference>,
    horizon: output::ForecastHorizon,
    accounting_scope: AccountingScope,
    source_field_family: Option<output::ForecastFieldFamily>,
    metric: output::ForecastMetric,
    source_field: String,
    disclosure: &'a FinancialDisclosure,
    timestamp: Option<NaiveDateTime>,
    value: Option<f64>,
    raw_value: Option<String>,
    reasons: Vec<output::ForecastReasonCode>,
}

fn forecast_field_name(
    prefix: &str,
    metric_suffix: &str,
    year_reference: output::ForecastYearReference,
    horizon: output::ForecastHorizon,
) -> String {
    let next_year = year_reference == output::ForecastYearReference::Next;
    let suffix = if next_year && prefix == "F" && metric_suffix == "NP" {
        "Np"
    } else {
        metric_suffix
    };
    format!(
        "{}{}{}{}",
        if next_year { "Nx" } else { "" },
        prefix,
        suffix,
        if horizon == output::ForecastHorizon::SecondQuarter {
            "2Q"
        } else {
            ""
        }
    )
}

fn forecast_period(
    disclosure: &FinancialDisclosure,
    year_reference: output::ForecastYearReference,
) -> Option<(NaiveDate, NaiveDate)> {
    match year_reference {
        output::ForecastYearReference::Current => disclosure
            .current_fiscal_year_start
            .zip(disclosure.current_fiscal_year_end),
        output::ForecastYearReference::Next => disclosure
            .next_fiscal_year_start
            .zip(disclosure.next_fiscal_year_end),
    }
}

fn interim_actual_period(disclosure: &FinancialDisclosure) -> Option<(NaiveDate, NaiveDate)> {
    if !disclosure.document_type.contains("FinancialStatements")
        || disclosure.current_period_type.as_deref() != Some("2Q")
        || disclosure.accounting_scope == AccountingScope::Unknown
    {
        return None;
    }
    let start = disclosure.current_fiscal_year_start?;
    let end = disclosure.current_fiscal_year_end?;
    (disclosure.current_period_start == Some(start)
        && disclosure
            .current_period_end
            .is_some_and(|period_end| period_end < end))
    .then_some((start, end))
}

fn forecast_metric_suffix(metric: output::ForecastMetric) -> &'static str {
    match metric {
        output::ForecastMetric::Sales => "Sales",
        output::ForecastMetric::OperatingProfit => "OP",
        output::ForecastMetric::OrdinaryProfit => "OdP",
        output::ForecastMetric::NetIncome => "NP",
    }
}

fn push_forecast_reason(
    reasons: &mut Vec<output::ForecastReasonCode>,
    reason: output::ForecastReasonCode,
) {
    if !reasons.contains(&reason) {
        reasons.push(reason);
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
    [
        output::ForecastYearReference::Current,
        output::ForecastYearReference::Next,
    ]
    .into_iter()
    .filter_map(|year_reference| {
        let period = forecast_period(disclosure, year_reference)?;
        let field = forecast_field_name(
            prefix,
            metric_suffix,
            year_reference,
            output::ForecastHorizon::FullYear,
        );
        Some((field, period))
    })
    .collect()
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
        financial_forecasts: output::FinancialForecastAudit {
            target_date,
            cutoff_at,
            history_status: output::ForecastHistoryStatus::Skipped,
            reasons: vec![output::ForecastReasonCode::IncompleteHistory],
            forecasts: Vec::new(),
        },
        score: None,
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
    use std::collections::{BTreeMap, BTreeSet};

    use crate::domain::models::scoring::model::ValueScorePolicy;
    use crate::domain::models::statement::model::{
        AccountingScope, FinancialDisclosure, OperatingProfitPeriod,
    };
    use crate::presenter::presenters::fetch_scoring_data::output::{
        FetchCompany, FetchPrice, FetchScoringData, FetchStatus,
    };
    use crate::presenter::presenters::score_stock::output::{
        ComponentDetail, FiscalQuarter, ForecastHistoryStatus, ForecastHorizon, ForecastMetric,
        ForecastReasonCode, ForecastStatus, ForecastYearReference, ScoreErrorType, ScoreReasonCode,
        ScoreStatus,
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
        assert_eq!(
            actual.financial_forecasts.history_status,
            ForecastHistoryStatus::Available
        );
        let json = serde_json::to_value(&actual).unwrap();
        assert_eq!(json["financial_forecasts"]["history_status"], "available");
        assert!(json["financial_forecasts"]["forecasts"].is_array());

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
        assert_eq!(
            actual.financial_forecasts.history_status,
            ForecastHistoryStatus::Incomplete
        );
        assert!(
            actual
                .financial_forecasts
                .reasons
                .contains(&ForecastReasonCode::UnknownDisclosureTime)
        );
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

    #[test]
    fn financial_forecast_audit_uses_period_and_cutoff_and_retains_revisions() {
        let mut data = build_ok_data();
        let mut before_cutoff = forecast_disclosure("2025-02-10", "15:00:00", "10", "20", "5");
        before_cutoff
            .source_fields
            .insert("FSales".to_string(), Some("100".to_string()));
        before_cutoff
            .source_fields
            .insert("NxFSales".to_string(), Some("200".to_string()));
        before_cutoff.next_fiscal_year_start = Some(date("2025-04-01"));
        before_cutoff.next_fiscal_year_end = Some(date("2026-03-31"));
        let mut at_cutoff = forecast_disclosure("2025-03-31", "15:30:00", "11", "20", "5");
        at_cutoff
            .source_fields
            .insert("FSales".to_string(), Some("110".to_string()));
        at_cutoff
            .source_fields
            .insert("NxFSales".to_string(), Some("210".to_string()));
        at_cutoff.next_fiscal_year_start = Some(date("2025-04-01"));
        at_cutoff.next_fiscal_year_end = Some(date("2026-03-31"));
        let mut after_cutoff = forecast_disclosure("2025-03-31", "15:30:01", "12", "20", "5");
        after_cutoff
            .source_fields
            .insert("FSales".to_string(), Some("999".to_string()));
        after_cutoff
            .source_fields
            .insert("NxFSales".to_string(), Some("999".to_string()));
        after_cutoff.next_fiscal_year_start = Some(date("2025-04-01"));
        after_cutoff.next_fiscal_year_end = Some(date("2026-03-31"));
        data.financial_disclosures
            .extend([before_cutoff, at_cutoff, after_cutoff]);

        let audit =
            super::build_financial_forecast_audit(&data, super::analysis_cutoff(data.target_date));
        let current = audit
            .forecasts
            .iter()
            .find(|forecast| {
                forecast.fiscal_year_start == Some(date("2024-04-01"))
                    && forecast.metric == ForecastMetric::Sales
                    && forecast.horizon == ForecastHorizon::FullYear
            })
            .unwrap();
        let next = audit
            .forecasts
            .iter()
            .find(|forecast| {
                forecast.fiscal_year_start == Some(date("2025-04-01"))
                    && forecast.metric == ForecastMetric::Sales
                    && forecast.horizon == ForecastHorizon::FullYear
            })
            .unwrap();

        assert_eq!(current.status, ForecastStatus::Selected);
        assert_eq!(current.value, Some(110.0));
        assert_eq!(current.history.len(), 2);
        assert_eq!(current.source_field.as_deref(), Some("FSales"));
        assert_eq!(
            current
                .history
                .last()
                .unwrap()
                .source_disclosure
                .disclosure_number,
            "11"
        );
        assert!(current.history.iter().all(|item| item.value != Some(999.0)));
        assert_eq!(next.year_reference, Some(ForecastYearReference::Next));
        assert_eq!(next.value, Some(210.0));
        assert_eq!(audit.cutoff_at.to_string(), "2025-03-31 15:30:00");
        let json = serde_json::to_value(&audit).unwrap();
        assert_eq!(json["history_status"], "available");
        let serialized_current = json["forecasts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|forecast| {
                forecast["fiscal_year_start"] == "2024-04-01"
                    && forecast["horizon"] == "full_year"
                    && forecast["accounting_scope"] == "consolidated"
                    && forecast["metric"] == "sales"
            })
            .unwrap();
        assert_eq!(serialized_current["status"], "selected");
        assert!(serialized_current["history"].is_array());
    }

    #[test]
    fn financial_forecast_audit_does_not_fallback_from_blank_and_marks_ties() {
        let mut data = build_ok_data();
        let mut earlier = forecast_disclosure("2025-02-10", "15:00:00", "1", "20", "5");
        earlier
            .source_fields
            .insert("FSales".to_string(), Some("100".to_string()));
        let mut blank = forecast_disclosure("2025-03-31", "15:30:00", "2", "20", "5");
        blank
            .source_fields
            .insert("FSales".to_string(), Some(String::new()));
        data.financial_disclosures.extend([earlier, blank]);
        let cutoff = super::analysis_cutoff(data.target_date);

        let audit = super::build_financial_forecast_audit(&data, cutoff);
        let forecast = audit
            .forecasts
            .iter()
            .find(|forecast| {
                forecast.metric == ForecastMetric::Sales
                    && forecast.horizon == ForecastHorizon::FullYear
                    && forecast.fiscal_year_start == Some(date("2024-04-01"))
            })
            .unwrap();
        assert_eq!(forecast.status, ForecastStatus::Undetermined);
        assert_eq!(forecast.value, None);
        assert_eq!(forecast.raw_value.as_deref(), Some(""));
        assert!(forecast.reasons.contains(&ForecastReasonCode::MissingValue));
        assert!(
            !forecast
                .reasons
                .contains(&ForecastReasonCode::MissingSourceField)
        );
        assert!(
            !forecast
                .reasons
                .contains(&ForecastReasonCode::NullSourceField)
        );
        assert_eq!(forecast.history.len(), 2);

        let mut tied = forecast_disclosure("2025-03-31", "15:30:00", "3", "20", "5");
        tied.source_fields
            .insert("FSales".to_string(), Some("120".to_string()));
        data.financial_disclosures.push(tied);
        let tied_audit = super::build_financial_forecast_audit(&data, cutoff);
        let tied_forecast = tied_audit
            .forecasts
            .iter()
            .find(|forecast| {
                forecast.metric == ForecastMetric::Sales
                    && forecast.horizon == ForecastHorizon::FullYear
                    && forecast.fiscal_year_start == Some(date("2024-04-01"))
            })
            .unwrap();
        assert_eq!(tied_forecast.status, ForecastStatus::Ambiguous);
        assert_eq!(tied_forecast.value, None);
        assert!(
            tied_forecast
                .reasons
                .contains(&ForecastReasonCode::AmbiguousDisclosure)
        );
    }

    #[test]
    fn financial_forecast_audit_does_not_fallback_when_latest_field_is_absent_or_null() {
        let mut data = build_ok_data();
        let mut earlier = forecast_disclosure("2025-02-10", "15:00:00", "1", "20", "5");
        earlier
            .source_fields
            .insert("FSales".to_string(), Some("100".to_string()));
        let mut absent = forecast_disclosure("2025-03-31", "15:30:00", "2", "20", "5");
        absent.source_fields.remove("FSales");
        data.financial_disclosures.extend([earlier.clone(), absent]);
        let cutoff = super::analysis_cutoff(data.target_date);

        let audit = super::build_financial_forecast_audit(&data, cutoff);
        let forecast = audit
            .forecasts
            .iter()
            .find(|forecast| {
                forecast.metric == ForecastMetric::Sales
                    && forecast.horizon == ForecastHorizon::FullYear
                    && forecast.fiscal_year_start == Some(date("2024-04-01"))
            })
            .unwrap();
        assert_eq!(forecast.status, ForecastStatus::Undetermined);
        assert_eq!(forecast.value, None);
        assert_eq!(forecast.raw_value, None);
        assert_eq!(forecast.history.len(), 2);
        assert!(
            forecast
                .reasons
                .contains(&ForecastReasonCode::MissingSourceField)
        );
        let latest = forecast.history.last().unwrap();
        assert_eq!(latest.source_disclosure.disclosure_number, "2");
        assert!(
            latest
                .reasons
                .contains(&ForecastReasonCode::MissingSourceField)
        );

        let mut null_data = build_ok_data();
        let mut earlier = forecast_disclosure("2025-02-10", "15:00:00", "1", "20", "5");
        earlier
            .source_fields
            .insert("FSales".to_string(), Some("100".to_string()));
        let mut null_value = forecast_disclosure("2025-03-31", "15:30:00", "2", "20", "5");
        null_value.source_fields.insert("FSales".to_string(), None);
        null_data
            .financial_disclosures
            .extend([earlier, null_value]);
        let null_audit = super::build_financial_forecast_audit(&null_data, cutoff);
        let null_forecast = null_audit
            .forecasts
            .iter()
            .find(|forecast| {
                forecast.metric == ForecastMetric::Sales
                    && forecast.horizon == ForecastHorizon::FullYear
                    && forecast.fiscal_year_start == Some(date("2024-04-01"))
            })
            .unwrap();
        assert_eq!(null_forecast.status, ForecastStatus::Undetermined);
        assert!(
            null_forecast
                .reasons
                .contains(&ForecastReasonCode::NullSourceField)
        );
        assert!(
            !null_forecast
                .reasons
                .contains(&ForecastReasonCode::MissingValue)
        );
    }

    #[test]
    fn missing_fields_from_other_period_or_scope_do_not_invalidate_forecast() {
        let mut data = build_ok_data();
        let mut earlier = forecast_disclosure("2025-02-10", "15:00:00", "1", "20", "5");
        earlier
            .source_fields
            .insert("FSales".to_string(), Some("100".to_string()));
        let mut other_period = forecast_disclosure("2025-03-31", "15:30:00", "2", "20", "5");
        other_period.source_fields.remove("FSales");
        other_period.current_fiscal_year_start = Some(date("2025-04-01"));
        other_period.current_fiscal_year_end = Some(date("2026-03-31"));
        let mut other_scope = forecast_disclosure("2025-03-31", "15:30:00", "3", "20", "5");
        other_scope.source_fields.remove("FSales");
        other_scope.document_type = "3QFinancialStatements_NonConsolidated_JP".to_string();
        other_scope.accounting_scope = AccountingScope::NonConsolidated;
        data.financial_disclosures
            .extend([earlier, other_period, other_scope]);

        let audit =
            super::build_financial_forecast_audit(&data, super::analysis_cutoff(data.target_date));
        let forecast = audit
            .forecasts
            .iter()
            .find(|forecast| {
                forecast.metric == ForecastMetric::Sales
                    && forecast.horizon == ForecastHorizon::FullYear
                    && forecast.fiscal_year_start == Some(date("2024-04-01"))
                    && forecast.accounting_scope == AccountingScope::Consolidated
            })
            .unwrap();
        assert_eq!(forecast.status, ForecastStatus::Selected);
        assert_eq!(forecast.value, Some(100.0));
    }

    #[test]
    fn financial_forecast_audit_reports_unknown_basis_period_and_incomplete_history() {
        let mut data = build_ok_data();
        data.financial_disclosures.clear();
        let mut unknown_scope = make_disclosure(
            Some("2025-02-10"),
            Some("15:00:00"),
            "1",
            "3QFinancialStatements_Unknown_IFRS",
            Some("3Q"),
            Some(("2024-04-01", "2025-03-31")),
            Some(("2024-04-01", "2024-12-31")),
            &[("FSales", "100")],
        );
        unknown_scope.accounting_scope = AccountingScope::Unknown;
        data.financial_disclosures.push(unknown_scope);
        let cutoff = super::analysis_cutoff(data.target_date);
        let audit = super::build_financial_forecast_audit(&data, cutoff);
        let forecast = audit
            .forecasts
            .iter()
            .find(|forecast| forecast.metric == ForecastMetric::Sales)
            .unwrap();
        assert_eq!(forecast.accounting_scope, AccountingScope::Unknown);
        assert_eq!(forecast.status, ForecastStatus::Undetermined);
        assert!(
            forecast
                .reasons
                .contains(&ForecastReasonCode::UnknownAccountingScope)
        );

        let mut unknown_period = make_disclosure(
            Some("2025-02-10"),
            Some("15:00:00"),
            "2",
            "3QFinancialStatements_Consolidated_JP",
            Some("3Q"),
            None,
            Some(("2024-04-01", "2024-12-31")),
            &[("FSales", "100")],
        );
        unknown_period.accounting_scope = AccountingScope::Consolidated;
        data.financial_disclosures = vec![unknown_period];
        let period_audit = super::build_financial_forecast_audit(&data, cutoff);
        let forecast = period_audit
            .forecasts
            .iter()
            .find(|forecast| forecast.metric == ForecastMetric::Sales)
            .unwrap();
        assert_eq!(forecast.status, ForecastStatus::Undetermined);
        assert!(
            forecast
                .reasons
                .contains(&ForecastReasonCode::UnknownFiscalPeriod)
        );

        data.status = FetchStatus::IncompleteHistory;
        let incomplete_audit = super::build_financial_forecast_audit(&data, cutoff);
        assert_eq!(
            incomplete_audit.history_status,
            ForecastHistoryStatus::Incomplete
        );
        let forecast = incomplete_audit
            .forecasts
            .iter()
            .find(|forecast| forecast.metric == ForecastMetric::Sales)
            .unwrap();
        assert_eq!(forecast.status, ForecastStatus::IncompleteHistory);
        assert_eq!(forecast.value, None);
        assert!(
            incomplete_audit
                .reasons
                .contains(&ForecastReasonCode::IncompleteHistory)
        );
    }

    #[test]
    fn financial_forecast_audit_uses_document_and_field_family_for_scope() {
        let fiscal_year = Some(("2025-04-01", "2026-03-31"));
        let period = Some(("2025-04-01", "2025-12-31"));
        let scenarios = [
            (
                "3QFinancialStatements_Consolidated_JP",
                AccountingScope::Consolidated,
                vec![("FSales", "10"), ("FNCSales", "11")],
            ),
            (
                "3QFinancialStatements_NonConsolidated_JP",
                AccountingScope::NonConsolidated,
                vec![("FSales", "20")],
            ),
            (
                "EarnForecastRevision_JP",
                AccountingScope::Unknown,
                vec![("FSales", "30"), ("FNCSales", "31")],
            ),
        ];
        for (index, (document_type, document_scope, fields)) in scenarios.into_iter().enumerate() {
            let mut data = build_ok_data();
            data.target_date = date("2025-12-31");
            data.price.as_mut().unwrap().date = Some(data.target_date);
            let values = fields
                .iter()
                .map(|(field, value)| (*field, *value))
                .collect::<Vec<_>>();
            let mut disclosure = make_disclosure(
                Some("2025-12-30"),
                Some("14:00:00"),
                &index.to_string(),
                document_type,
                Some("3Q"),
                fiscal_year,
                period,
                &values,
            );
            disclosure.accounting_scope = document_scope;
            data.financial_disclosures = vec![disclosure];
            let audit = super::build_financial_forecast_audit(
                &data,
                super::analysis_cutoff(data.target_date),
            );
            let found_scopes = audit
                .forecasts
                .iter()
                .filter(|forecast| forecast.metric == ForecastMetric::Sales)
                .map(|forecast| forecast.accounting_scope)
                .collect::<BTreeSet<_>>();
            match index {
                0 => assert_eq!(
                    found_scopes,
                    BTreeSet::from([
                        AccountingScope::Consolidated,
                        AccountingScope::NonConsolidated
                    ])
                ),
                1 => assert_eq!(
                    found_scopes,
                    BTreeSet::from([AccountingScope::NonConsolidated])
                ),
                _ => assert_eq!(
                    found_scopes,
                    BTreeSet::from([
                        AccountingScope::Consolidated,
                        AccountingScope::NonConsolidated
                    ])
                ),
            }
        }
    }

    #[test]
    fn second_quarter_forecast_becomes_historical_after_actual_disclosure() {
        let mut data = build_ok_data();
        data.target_date = date("2024-11-08");
        data.price.as_mut().unwrap().date = Some(data.target_date);
        let mut forecast = make_disclosure(
            Some("2024-07-01"),
            Some("10:00:00"),
            "1",
            "1QFinancialStatements_Consolidated_JP",
            Some("1Q"),
            Some(("2024-04-01", "2025-03-31")),
            Some(("2024-04-01", "2024-06-30")),
            &[("FSales2Q", "50")],
        );
        forecast.accounting_scope = AccountingScope::Consolidated;
        let actual = make_disclosure(
            Some("2024-11-08"),
            Some("14:00:00"),
            "2",
            "2QFinancialStatements_Consolidated_JP",
            Some("2Q"),
            Some(("2024-04-01", "2025-03-31")),
            Some(("2024-04-01", "2024-09-30")),
            &[("Sales", "100")],
        );
        data.financial_disclosures = vec![forecast, actual];

        let audit =
            super::build_financial_forecast_audit(&data, super::analysis_cutoff(data.target_date));
        let forecast = audit
            .forecasts
            .iter()
            .find(|forecast| {
                forecast.metric == ForecastMetric::Sales
                    && forecast.horizon == ForecastHorizon::SecondQuarter
            })
            .unwrap();
        assert_eq!(
            forecast.status,
            ForecastStatus::Historical,
            "reasons: {:?}",
            forecast.reasons
        );
        assert_eq!(forecast.value, Some(50.0));
        assert!(
            forecast
                .reasons
                .contains(&ForecastReasonCode::InterimActualReported)
        );
    }
}
