use crate::domain::models::statement::model;
use crate::domain::repositories::statement::{queries, repository};
use crate::infrastructure::dsv::Dsv;
use crate::infrastructure::from_slice::FromSlice;
use anyhow::Result;
use async_trait::async_trait;
use std::fmt::{Debug, Display};

#[async_trait]
impl<T> repository::Statement for Dsv<T>
where
    T: FromSlice<Item = model::RowStatement> + Send + Sync,
    <T as FromSlice>::Deserialize: Send + Sync,
    <T::Item as TryFrom<T::Deserialize>>::Error: Debug + Display + Send + Sync,
    model::RowStatement: TryFrom<<T as FromSlice>::Deserialize>,
{
    async fn get_row_statements<'a>(
        &self,
        query: &queries::get_statements::Query<'a>,
    ) -> Result<Vec<model::RowStatement>> {
        let processed = T::process_tabular_data(self.buffer.as_ref(), self.has_headers)?;

        let vec_row_statement: Vec<model::RowStatement> = match query.code {
            Some(code) => processed
                .into_iter()
                .filter(|statement| statement.code.eq(code))
                .collect(),
            // Noneの場合は、全ての行を返す
            None => processed,
        };

        Ok(vec_row_statement)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::models::statement::model::OperatingProfitPeriod;
    use crate::domain::repositories::statement::queries;
    use crate::domain::repositories::statement::repository::Statement;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::statement::structures::internal::tsv;
    use bytes::Bytes;
    use chrono::NaiveDate;
    use serde_json::json;

    const SAMPLE_CODE: &str = "72030";

    #[tokio::test]
    async fn get_row_statements_filters_code_and_preserves_disclosures() -> anyhow::Result<()> {
        let mut writer = csv::WriterBuilder::new()
            .delimiter(b'\t')
            .from_writer(Vec::new());
        for (code, date, kind, start, end, fiscal_end, profit) in [
            (
                SAMPLE_CODE,
                "2026-05-08",
                "FY",
                "2025-04-01",
                "2026-03-31",
                "2026-03-31",
                Some(100.0),
            ),
            (
                "13010",
                "2026-05-11",
                "FY",
                "2025-04-01",
                "2026-03-31",
                "2026-03-31",
                Some(50.0),
            ),
            (
                SAMPLE_CODE,
                "2026-08-07",
                "1Q",
                "2026-04-01",
                "2026-06-30",
                "2027-03-31",
                None,
            ),
        ] {
            writer.serialize(tsv::Structure {
                code: code.into(),
                disclosed_date: NaiveDate::parse_from_str(date, "%Y-%m-%d")?,
                current_fiscal_year_end_date: Some(NaiveDate::parse_from_str(
                    fiscal_end, "%Y-%m-%d",
                )?),
                opp: profit,
                disclosure_json: Some(
                    json!({
                        "code": code,
                        "disclosed_date": date,
                        "disclosed_time": "15:00:00",
                        "disclosure_number": format!("{code}-{date}"),
                        "document_type": format!("{kind}FinancialStatements_Consolidated_JP"),
                        "current_period_type": kind,
                        "current_period_start": start,
                        "current_period_end": end,
                        "current_fiscal_year_start": start,
                        "current_fiscal_year_end": fiscal_end,
                        "operating_profit": profit,
                        "operating_profit_period": "unspecified"
                    })
                    .to_string(),
                ),
                ..Default::default()
            })?;
        }
        let repository = Dsv::<tsv::Structure>::new(true, Bytes::from(writer.into_inner()?));
        let query = queries::get_statements::Query {
            code: Some(SAMPLE_CODE),
        };
        let actual = repository.get_row_statements(&query).await?;
        assert_eq!(actual.len(), 2);
        assert!(actual.iter().all(|row| row.code == SAMPLE_CODE));
        assert_eq!(
            actual[0].disclosed_date,
            NaiveDate::from_ymd_opt(2026, 5, 8)
        );
        assert_eq!(actual[0].opp, Some(100.0));
        let annual = actual[0].disclosure.as_ref().unwrap();
        assert_eq!(annual.current_period_type.as_deref(), Some("FY"));
        assert_eq!(annual.operating_profit, Some(100.0));
        assert_eq!(
            actual[1].disclosed_date,
            NaiveDate::from_ymd_opt(2026, 8, 7)
        );
        let quarter = actual[1].disclosure.as_ref().unwrap();
        assert_eq!(quarter.current_period_type.as_deref(), Some("1Q"));
        assert_eq!(
            quarter.current_period_end,
            NaiveDate::from_ymd_opt(2026, 6, 30)
        );
        assert_eq!(quarter.operating_profit, None);
        assert_eq!(
            quarter.operating_profit_period,
            OperatingProfitPeriod::Unspecified
        );
        assert_eq!(
            repository
                .get_row_statements(&queries::get_statements::Query { code: None })
                .await?
                .len(),
            3
        );
        assert!(
            repository
                .get_row_statements(&queries::get_statements::Query {
                    code: Some("99990"),
                })
                .await?
                .is_empty()
        );
        Ok(())
    }
}
