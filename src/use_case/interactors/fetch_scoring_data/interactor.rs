use anyhow::{Result, ensure};
use async_trait::async_trait;

use crate::domain::models::company::model as company_model;
use crate::domain::models::statement::model as statement_model;
use crate::domain::models::stock::model as stock_model;
use crate::domain::repositories;
use crate::domain::repositories::company::queries::get_company;
use crate::domain::repositories::statement::queries::get_statements;
use crate::domain::repositories::stock::queries::get_stock;
use crate::presenter::presenters::fetch_scoring_data::output;
use crate::use_case::interfaces::fetch_scoring_data::{input, use_case};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct FetchScoringData<'a, CR, SR, SMR> {
    company_repository: &'a CR,
    stock_repository: &'a SR,
    statement_repository: &'a SMR,
}

impl<'a, CR, SR, SMR> FetchScoringData<'a, CR, SR, SMR> {
    pub fn new(
        company_repository: &'a CR,
        stock_repository: &'a SR,
        statement_repository: &'a SMR,
    ) -> Self {
        Self {
            company_repository,
            stock_repository,
            statement_repository,
        }
    }
}

#[async_trait]
impl<CR, SR, SMR> use_case::FetchScoringData for FetchScoringData<'_, CR, SR, SMR>
where
    CR: repositories::company::repository::Company + Send + Sync,
    SR: repositories::stock::repository::Stock + Send + Sync,
    SMR: repositories::statement::repository::Statement + Send + Sync,
{
    async fn handle(&self, input: input::FetchScoringData) -> Result<output::FetchScoringData> {
        let query = get_company::Query {
            code: input.code.as_str(),
            market: None,
        };
        let row_company = self.company_repository.get_row_company(&query).await;
        let Ok(row_company) = row_company else {
            return Ok(output::FetchScoringData {
                code: input.code,
                fetched_at: input.fetched_at,
                status: output::FetchStatus::EmptyData,
                error_type: Some(output::FetchErrorType::EmptyCompany),
                company: None,
                price: None,
                latest_statement: None,
                full_year_sales: vec![],
                financial_disclosures: vec![],
            });
        };

        let query = get_stock::Query {
            code: Some(input.code.as_str()),
            market: None,
            target_date: input.target_date,
        };
        let row_stock = self.stock_repository.get_row_stock(&query).await;
        let Ok(row_stock) = row_stock else {
            return Ok(output::FetchScoringData {
                code: input.code,
                fetched_at: input.fetched_at,
                status: output::FetchStatus::EmptyData,
                error_type: Some(output::FetchErrorType::EmptyStock),
                company: Some(output::FetchCompany::from(row_company)),
                price: None,
                latest_statement: None,
                full_year_sales: vec![],
                financial_disclosures: vec![],
            });
        };

        let query = get_statements::Query {
            code: Some(input.code.as_str()),
        };
        let mut row_statements = self.statement_repository.get_row_statements(&query).await?;

        if row_statements.is_empty() {
            return Ok(output::FetchScoringData {
                code: input.code,
                fetched_at: input.fetched_at,
                status: output::FetchStatus::EmptyData,
                error_type: Some(output::FetchErrorType::EmptyStatement),
                company: Some(output::FetchCompany::from(row_company)),
                price: Some(output::FetchPrice::from(row_stock)),
                latest_statement: None,
                full_year_sales: vec![],
                financial_disclosures: vec![],
            });
        }
        ensure!(
            row_statements.iter().all(|row| row.disclosure.is_some()),
            "financial disclosure history is missing for code {}; regenerate the statement data",
            input.code
        );

        row_statements.sort_by_key(|row| {
            (
                row.disclosed_date,
                row.disclosure
                    .as_ref()
                    .map(|disclosure| disclosure.disclosed_time),
            )
        });
        let financial_disclosures = row_statements
            .iter()
            .filter_map(|row| row.disclosure.clone())
            .collect();
        let full_year_statements = row_statements
            .into_iter()
            .filter(statement_model::RowStatement::is_usable_full_year)
            .collect::<Vec<_>>();
        let latest_statement = full_year_statements.last().cloned();
        let full_year_sales = full_year_statements
            .into_iter()
            .map(output::FetchFullYearSales::from)
            .collect::<Vec<_>>();

        Ok(output::FetchScoringData {
            code: input.code,
            fetched_at: input.fetched_at,
            status: output::FetchStatus::Ok,
            error_type: None,
            company: Some(output::FetchCompany::from(row_company)),
            price: Some(output::FetchPrice::from(row_stock)),
            latest_statement: latest_statement.map(output::FetchLatestStatement::from),
            full_year_sales,
            financial_disclosures,
        })
    }
}

impl From<company_model::RowCompany> for output::FetchCompany {
    fn from(value: company_model::RowCompany) -> Self {
        Self { name: value.name }
    }
}

impl From<stock_model::RowStock> for output::FetchPrice {
    fn from(value: stock_model::RowStock) -> Self {
        Self {
            date: value.date,
            close: value.close,
            adj_close: value.adj_close,
        }
    }
}

impl From<statement_model::RowStatement> for output::FetchLatestStatement {
    fn from(value: statement_model::RowStatement) -> Self {
        Self {
            disclosed_date: value.disclosed_date,
            eps: value.eps,
            bps: value.bps,
            annual_dividend_forecast: value.annual_dividend_forecast,
            profit: value.profit,
            equity: value.equity,
        }
    }
}

impl From<statement_model::RowStatement> for output::FetchFullYearSales {
    fn from(value: statement_model::RowStatement) -> Self {
        Self {
            current_fiscal_year_end_date: value.current_fiscal_year_end_date,
            disclosed_date: value.disclosed_date,
            net_sales: value.net_sales,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::statement::structures::jquants_api::Response;
    use chrono::{NaiveDate, Utc};
    use serde_json::json;

    struct Company;
    struct Stock;
    struct Statements {
        rows: Vec<statement_model::RowStatement>,
        fail: bool,
    }

    #[async_trait]
    impl repositories::company::repository::Company for Company {
        async fn get_row_company<'a>(
            &self,
            _: &get_company::Query<'a>,
        ) -> Result<company_model::RowCompany> {
            Ok(company_model::RowCompany::default())
        }

        async fn get_vec_row_company(&self) -> Result<Vec<company_model::RowCompany>> {
            Ok(vec![])
        }
    }

    #[async_trait]
    impl repositories::stock::repository::Stock for Stock {
        async fn get_row_stock<'a>(
            &self,
            _: &get_stock::Query<'a>,
        ) -> Result<stock_model::RowStock> {
            Ok(stock_model::RowStock::default())
        }

        async fn get_vec_row_stock<'a>(
            &self,
            _: &repositories::stock::queries::get_stocks::Query<'a>,
        ) -> Result<Vec<stock_model::RowStock>> {
            Ok(vec![])
        }

        async fn get_vec_row_stock_by_date(
            &self,
            _: &repositories::stock::queries::get_stocks_by_date::Query,
        ) -> Result<Vec<stock_model::RowStock>> {
            Ok(vec![])
        }
    }

    #[async_trait]
    impl repositories::statement::repository::Statement for Statements {
        async fn get_row_statements<'a>(
            &self,
            _: &get_statements::Query<'a>,
        ) -> Result<Vec<statement_model::RowStatement>> {
            if self.fail {
                anyhow::bail!("page fetch failed");
            }
            Ok(self.rows.clone())
        }
    }

    fn row(
        date: &str,
        time: &str,
        number: &str,
        period: &str,
        sales: &str,
    ) -> statement_model::RowStatement {
        let value = json!({
            "Code": "12340", "DiscDate": date, "DiscTime": time, "DiscNo": number,
            "DocType": "FinancialStatements_Consolidated_JP", "CurPerType": period,
            "CurFYEn": "2026-03-31", "Sales": sales, "OP": "90", "FOP": ""
        });
        statement_model::RowStatement::try_from(Response {
            code: "1234".into(),
            value: &value,
        })
        .unwrap()
    }

    fn input() -> input::FetchScoringData {
        input::FetchScoringData::new(
            "1234".into(),
            NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
            Utc::now(),
        )
    }

    #[tokio::test]
    async fn history_is_sorted_but_annual_projections_exclude_quarters() {
        use crate::use_case::interfaces::fetch_scoring_data::use_case::FetchScoringData as _;
        let statements = Statements {
            rows: vec![
                row("2026-08-01", "15:00:00", "1", "1Q", "25"),
                row("2026-05-01", "15:00:00", "2", "FY", "100"),
                row("2026-08-01", "09:00:00", "9", "1Q", "20"),
            ],
            fail: false,
        };
        let result = FetchScoringData::new(&Company, &Stock, &statements)
            .handle(input())
            .await
            .unwrap();
        assert_eq!(
            result
                .financial_disclosures
                .iter()
                .map(|d| d.disclosure_number.as_str())
                .collect::<Vec<_>>(),
            ["2", "9", "1"]
        );
        assert_eq!(result.full_year_sales.len(), 1);
        assert_eq!(result.full_year_sales[0].net_sales, Some(100.0));
        assert_eq!(
            result.latest_statement.unwrap().disclosed_date,
            NaiveDate::from_ymd_opt(2026, 5, 1)
        );
        let json = serde_json::to_value(result.financial_disclosures).unwrap();
        assert_eq!(
            json[2]["forecast_operating_profit"],
            serde_json::Value::Null
        );
    }

    #[tokio::test]
    async fn repository_failures_and_missing_history_are_errors() {
        use crate::use_case::interfaces::fetch_scoring_data::use_case::FetchScoringData as _;
        let failing = Statements {
            rows: vec![],
            fail: true,
        };
        assert!(
            FetchScoringData::new(&Company, &Stock, &failing)
                .handle(input())
                .await
                .is_err()
        );
        let missing = Statements {
            rows: vec![statement_model::RowStatement::default()],
            fail: false,
        };
        assert!(
            FetchScoringData::new(&Company, &Stock, &missing)
                .handle(input())
                .await
                .is_err()
        );
    }
}
