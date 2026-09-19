use anyhow::Result;
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
            });
        };

        let query = get_statements::Query {
            code: Some(input.code.as_str()),
        };
        let mut row_statements = self
            .statement_repository
            .get_row_statements(&query)
            .await
            .unwrap_or_else(|_| vec![]);

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
            });
        }

        row_statements.sort_by(|a, b| {
            let a_date = a.disclosed_date.unwrap_or_default();
            let b_date = b.disclosed_date.unwrap_or_default();
            a_date.cmp(&b_date)
        });
        let latest_statement = row_statements.last().unwrap().clone();

        let full_year_sales = row_statements
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
            latest_statement: Some(output::FetchLatestStatement::from(latest_statement)),
            full_year_sales,
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
