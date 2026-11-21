use crate::domain::models::statement::model::AccountingScope;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreStock {
    pub code: String,
    pub scored_at: DateTime<Utc>,
    pub target_date: NaiveDate,
    pub cutoff_at: NaiveDateTime,
    pub status: ScoreStatus,
    pub error_type: Option<ScoreErrorType>,
    pub reasons: Vec<ScoreReason>,
    pub metrics: BTreeMap<String, MetricAudit>,
    pub quarterly_actuals: Vec<SingleQuarterActual>,
    pub financial_forecasts: FinancialForecastAudit,
    pub score: Option<Score>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreStatus {
    Ok,
    Undetermined,
    Failed,
    Skipped,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreErrorType {
    InvalidInput,
    CalculationError,
    IncompleteHistory,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreReasonCode {
    IncompleteHistory,
    UnknownDisclosureTime,
    NoTargetFiscalYear,
    AmbiguousTargetFiscalYear,
    UnknownAccountingScope,
    AccountingScopeMismatch,
    NoMatchingDisclosure,
    AmbiguousDisclosure,
    UnknownFiscalPeriod,
    MissingValue,
    MissingPrice,
    PriceDateMismatch,
    InvalidValue,
    NonPositivePrice,
    ZeroDenominator,
    NoPriorFiscalYear,
    NoPriorQuarter,
    IncompatibleFiscalPeriods,
    IncompatibleDisclosureBasis,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreReason {
    pub metric: String,
    pub code: ScoreReasonCode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisclosureReference {
    pub code: String,
    pub disclosure_number: String,
    pub disclosed_date: Option<NaiveDate>,
    pub disclosed_time: Option<NaiveTime>,
    pub document_type: String,
    pub document_scope: crate::domain::models::statement::model::AccountingScope,
    pub value_scope: crate::domain::models::statement::model::AccountingScope,
    pub current_period_type: Option<String>,
    pub current_period_start: Option<NaiveDate>,
    pub current_period_end: Option<NaiveDate>,
    pub current_fiscal_year_start: Option<NaiveDate>,
    pub current_fiscal_year_end: Option<NaiveDate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricAudit {
    pub inputs: BTreeMap<String, Option<f64>>,
    pub price_date: Option<NaiveDate>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub source_disclosures: Vec<DisclosureReference>,
    pub result: Option<f64>,
    pub score_component: Option<ComponentDetail>,
    pub reasons: Vec<ScoreReasonCode>,
}

/// A fiscal quarter's actuals derived from compatible cumulative disclosures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SingleQuarterActual {
    pub fiscal_year_start: Option<NaiveDate>,
    pub fiscal_year_end: Option<NaiveDate>,
    pub quarter: FiscalQuarter,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub accounting_scope: crate::domain::models::statement::model::AccountingScope,
    pub document_basis: Option<String>,
    pub values: BTreeMap<String, SingleQuarterValue>,
}

/// Identifies a fiscal quarter, with Q4 represented by the full-year disclosure.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FiscalQuarter {
    Q1,
    Q2,
    Q3,
    Q4,
}

/// Audits a single-quarter value and its cumulative source values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SingleQuarterValue {
    pub current_cumulative_input: Option<f64>,
    pub prior_cumulative_input: Option<f64>,
    pub result: Option<f64>,
    pub source_disclosures: Vec<DisclosureReference>,
    pub reasons: Vec<ScoreReasonCode>,
}

/// Forecast records selected using the same as-of cutoff as the stock score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FinancialForecastAudit {
    pub target_date: NaiveDate,
    pub cutoff_at: NaiveDateTime,
    pub history_status: ForecastHistoryStatus,
    pub reasons: Vec<ForecastReasonCode>,
    pub forecasts: Vec<FinancialForecast>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastHistoryStatus {
    Available,
    Incomplete,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FinancialForecast {
    pub fiscal_year_start: Option<NaiveDate>,
    pub fiscal_year_end: Option<NaiveDate>,
    pub year_reference: Option<ForecastYearReference>,
    pub horizon: ForecastHorizon,
    pub accounting_scope: AccountingScope,
    pub source_field_family: Option<ForecastFieldFamily>,
    pub metric: ForecastMetric,
    pub status: ForecastStatus,
    pub value: Option<f64>,
    pub raw_value: Option<String>,
    pub source_field: Option<String>,
    pub source_disclosure: Option<DisclosureReference>,
    pub reasons: Vec<ForecastReasonCode>,
    pub history: Vec<ForecastObservation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForecastObservation {
    pub fiscal_year_start: Option<NaiveDate>,
    pub fiscal_year_end: Option<NaiveDate>,
    pub year_reference: Option<ForecastYearReference>,
    pub value: Option<f64>,
    pub raw_value: Option<String>,
    pub source_field: String,
    pub source_disclosure: DisclosureReference,
    pub status: ForecastStatus,
    pub reasons: Vec<ForecastReasonCode>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastYearReference {
    Current,
    Next,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastHorizon {
    FullYear,
    SecondQuarter,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastMetric {
    Sales,
    OperatingProfit,
    OrdinaryProfit,
    NetIncome,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastFieldFamily {
    Unprefixed,
    NonConsolidatedPrefix,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastStatus {
    Selected,
    Historical,
    Undetermined,
    Ambiguous,
    IncompleteHistory,
    Skipped,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastReasonCode {
    IncompleteHistory,
    UnknownDisclosureTime,
    UnknownFiscalPeriod,
    IncompatibleFiscalPeriod,
    UnknownAccountingScope,
    MissingValue,
    InvalidValue,
    AmbiguousDisclosure,
    FiscalPeriodEnded,
    InterimActualReported,
    NoMatchingDisclosure,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Score {
    pub total: f64,
    pub components: BTreeMap<String, ComponentDetail>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentDetail {
    pub raw: f64,
    pub normalized: f64,
    pub points: f64,
}
