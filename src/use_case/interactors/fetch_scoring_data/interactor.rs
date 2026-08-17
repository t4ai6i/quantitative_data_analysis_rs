use anyhow::Result;
use async_trait::async_trait;

use crate::domain::models::company::model;
use crate::domain::models::statement::model as statement_model;
use crate::domain::repositories;
use crate::domain::repositories::company::queries::get_company;
use crate::domain::repositories::statement::queries::get_statement;
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
        let company = self.company_repository.get_company(&query).await;
        let Ok(company) = company else {
            return Ok(output::FetchScoringData {
                code: input.code,
                fetched_at: chrono::Utc::now(),
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
        let stock = self.stock_repository.get_row_stock(&query).await;
        let Ok(stock) = stock else {
            return Ok(output::FetchScoringData {
                code: input.code,
                fetched_at: chrono::Utc::now(),
                status: output::FetchStatus::EmptyData,
                error_type: Some(output::FetchErrorType::EmptyStock),
                company: Some(output::FetchCompany::from(company)),
                price: None,
                latest_statement: None,
                full_year_sales: vec![],
            });
        };

        let query = get_statement::Query {
            code: input.code.as_str(),
        };
        let latest_statement = self.statement_repository.get_statement(&query).await;
        let Ok(latest_statement) = latest_statement else {
            return Ok(output::FetchScoringData {
                code: input.code,
                fetched_at: chrono::Utc::now(),
                status: output::FetchStatus::EmptyData,
                error_type: Some(output::FetchErrorType::EmptyStatement),
                company: Some(output::FetchCompany::from(company)),
                price: Some(output::FetchPrice::from(stock)),
                latest_statement: None,
                full_year_sales: vec![],
            });
        };
        let full_year_statements = self
            .statement_repository
            .get_row_full_year_statements(&query)
            .await
            .unwrap_or_else(|_| vec![]);

        let mut full_year_sales = full_year_statements
            .into_iter()
            .map(output::FetchFullYearSales::from)
            .collect::<Vec<_>>();
        full_year_sales.sort_by(|left, right| {
            right
                .current_fiscal_year_end_date
                .cmp(&left.current_fiscal_year_end_date)
                .then_with(|| right.disclosed_date.cmp(&left.disclosed_date))
        });

        Ok(output::FetchScoringData {
            code: input.code,
            fetched_at: chrono::Utc::now(),
            status: output::FetchStatus::Ok,
            error_type: None,
            company: Some(output::FetchCompany::from(company)),
            price: Some(output::FetchPrice::from(stock)),
            latest_statement: Some(output::FetchLatestStatement::from(latest_statement)),
            full_year_sales,
        })
    }
}

impl From<model::Company> for output::FetchCompany {
    fn from(value: model::Company) -> Self {
        Self {
            name: Some(value.name),
        }
    }
}

impl From<crate::domain::models::stock::model::RowStock> for output::FetchPrice {
    fn from(value: crate::domain::models::stock::model::RowStock) -> Self {
        Self {
            date: value.date,
            adj_close: value.adj_close,
        }
    }
}

impl From<statement_model::Statement> for output::FetchLatestStatement {
    fn from(value: statement_model::Statement) -> Self {
        Self {
            disclosed_date: Some(value.disclosed_date),
            eps: Some(value.eps),
            bps: Some(value.bps),
            annual_dividend_forecast: Some(value.annual_dividend_forecast),
            profit: Some(value.profit),
            equity: Some(value.equity),
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
