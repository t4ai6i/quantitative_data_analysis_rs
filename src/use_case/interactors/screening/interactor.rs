use crate::domain::models::screening::model::{ScreeningResult, ScreeningResults};
use crate::domain::models::screening::scoring::{ValueScorePolicy, score_value};
use crate::domain::repositories;
use crate::use_case::interactors::screening::query_screener::QueryScreener;
use crate::use_case::interfaces::screening::input;
use crate::use_case::interfaces::screening::use_case;
use anyhow::{Result, bail};
use async_trait::async_trait;
use std::cmp::Ordering;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Screening<'a, SR, SMR, CR> {
    stock_repository: &'a SR,
    statement_repository: &'a SMR,
    company_repository: &'a CR,
}

impl<'a, SR, SMR, CR> Screening<'a, SR, SMR, CR> {
    pub fn new(
        stock_repository: &'a SR,
        statement_repository: &'a SMR,
        company_repository: &'a CR,
    ) -> Self {
        Self {
            stock_repository,
            statement_repository,
            company_repository,
        }
    }

    pub fn stock_repository(&self) -> &'a SR {
        self.stock_repository
    }

    pub fn statement_repository(&self) -> &'a SMR {
        self.statement_repository
    }

    pub fn company_repository(&self) -> &'a CR {
        self.company_repository
    }
}

#[async_trait]
impl<SR, SMR, CR> use_case::ScreeningEngine for Screening<'_, SR, SMR, CR>
where
    SR: repositories::stock::repository::Stock + Send + Sync,
    CR: repositories::company::repository::Company + Send + Sync,
    SMR: repositories::statement::repository::Statement + Send + Sync,
{
    async fn handle(&self, input: input::Screening) -> Result<ScreeningResults> {
        validate_input(&input)?;

        let policy = ValueScorePolicy::try_from(input.preset_name.trim())?;

        let query = QueryScreener::new(
            self.company_repository,
            self.stock_repository,
            self.statement_repository,
        );
        let price_map = query.fetch_base_date_prices(input.target_date).await?;
        let candidates = query.fetch_universe_candidates().await?;
        let min_total_score = input
            .min_total_score
            .map(|score| score as f64)
            .unwrap_or(0.0);

        let mut results = Vec::new();

        for candidate in candidates.into_iter().filter_map(Result::ok) {
            if !input
                .markets
                .iter()
                .any(|market| market == &candidate.market)
            {
                continue;
            }

            let Some(adj_close) = price_map.get(candidate.code.as_str()).copied().flatten() else {
                continue;
            };

            let metrics = match query
                .fetch_financial_metrics(candidate.code.as_str(), adj_close)
                .await
            {
                Ok(metrics) => metrics,
                Err(error) if is_statement_unavailable_error(&error) => continue,
                Err(error) => return Err(error),
            };

            let (score_breakdown, total_score, reasons) = score_value(&metrics, policy);
            if total_score < min_total_score {
                continue;
            }

            results.push(ScreeningResult {
                candidate,
                metrics,
                score_breakdown,
                total_score,
                rank: 0,
                reasons,
            })
        }

        results.sort_by(|left, right| {
            right
                .total_score
                .partial_cmp(&left.total_score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.candidate.code.cmp(&right.candidate.code))
        });

        if results.len() > input.limit {
            results.truncate(input.limit);
        }

        for (index, result) in results.iter_mut().enumerate() {
            result.rank = index + 1;
        }

        Ok(ScreeningResults(results))
    }
}

fn is_statement_unavailable_error(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    message.contains("response[data] in response not found")
        || message.contains("response[data] in response is empty array")
        || message.contains("FY statement not found in response[data]")
}

fn validate_input(input: &input::Screening) -> Result<()> {
    if input.markets.is_empty() {
        bail!("markets is empty");
    }
    if input.markets.iter().any(|market| market.trim().is_empty()) {
        bail!("markets contains empty value");
    }
    if input.limit == 0 {
        bail!("limit must be greater than 0");
    }
    if input.preset_name.trim().is_empty() {
        bail!("preset name is empty");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::is_statement_unavailable_error;

    #[test]
    fn unavailable_statement_errors_are_classified() {
        assert!(is_statement_unavailable_error(&anyhow!(
            "response[data] in response not found. code: 1305"
        )));
        assert!(is_statement_unavailable_error(&anyhow!(
            "response[data] in response is empty array. code: 1308"
        )));
        assert!(is_statement_unavailable_error(&anyhow!(
            "FY statement not found in response[data]. code: 9999"
        )));
    }

    #[test]
    fn non_unavailable_errors_are_not_classified() {
        assert!(!is_statement_unavailable_error(&anyhow!(
            "network timeout while calling fins summary"
        )));
    }
}
