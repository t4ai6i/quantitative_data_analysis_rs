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
        let processed = self.items()?;

        let vec_row_statement: Vec<model::RowStatement> = match query.code {
            Some(code) => processed
                .iter()
                .filter(|statement| statement.code.eq(code))
                .cloned()
                .collect(),
            // Noneの場合は、全ての行を返す
            None => processed.to_vec(),
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
    use crate::infrastructure::from_slice::{DataFormat, FromSlice};
    use crate::infrastructure::repositories::statement::structures::internal::tsv;
    use bytes::Bytes;
    use chrono::NaiveDate;
    use serde_json::json;
    use std::sync::Barrier;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Instant;

    const SAMPLE_CODE: &str = "72030";
    static PARSES: [AtomicUsize; 3] = [const { AtomicUsize::new(0) }; 3];

    struct CountingTsv<const CASE: usize>;

    impl<const CASE: usize> FromSlice for CountingTsv<CASE> {
        type Deserialize = tsv::Structure;
        type Item = crate::domain::models::statement::model::RowStatement;

        fn data_format() -> DataFormat {
            PARSES[CASE].fetch_add(1, Ordering::SeqCst);
            tsv::Structure::data_format()
        }
    }

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
                disclosed_date: Some(NaiveDate::parse_from_str(date, "%Y-%m-%d")?),
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
                        "next_fiscal_year_start": null,
                        "next_fiscal_year_end": null,
                        "accounting_scope": "consolidated",
                        "operating_profit": profit,
                        "operating_profit_period": "unspecified",
                        "forecast_operating_profit": null,
                        "next_forecast_operating_profit": null,
                        "forecast_operating_profit_2q": null,
                        "next_forecast_operating_profit_2q": null,
                        "non_consolidated_operating_profit": null,
                        "non_consolidated_forecast_operating_profit": null,
                        "non_consolidated_next_forecast_operating_profit": null,
                        "non_consolidated_forecast_operating_profit_2q": null,
                        "non_consolidated_next_forecast_operating_profit_2q": null,
                        "source_fields": {}
                    })
                    .to_string(),
                ),
                ..Default::default()
            })?;
        }
        let buffer = Bytes::from(writer.into_inner()?);
        let expected = tsv::Structure::process_tabular_data(&buffer, true)?;
        let repository = Dsv::<CountingTsv<0>>::new(true, buffer.clone());
        assert_eq!(PARSES[0].load(Ordering::SeqCst), 0);
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
        for code in [Some("13010"), Some(SAMPLE_CODE), None] {
            let actual = repository
                .get_row_statements(&queries::get_statements::Query { code })
                .await?;
            let expected: Vec<_> = expected
                .iter()
                .filter(|row| code.is_none_or(|code| row.code == code))
                .cloned()
                .collect();
            assert_eq!(actual, expected);
        }
        assert_eq!(PARSES[0].load(Ordering::SeqCst), 1);
        let mut returned = repository
            .get_row_statements(&queries::get_statements::Query { code: None })
            .await?;
        returned[0].code.clear();
        assert_eq!(
            repository
                .get_row_statements(&queries::get_statements::Query { code: None })
                .await?,
            expected
        );
        let another = Dsv::<CountingTsv<0>>::new(true, buffer);
        assert_eq!(
            another
                .get_row_statements(&queries::get_statements::Query { code: None })
                .await?,
            expected
        );
        assert_eq!(PARSES[0].load(Ordering::SeqCst), 2);
        Ok(())
    }

    #[tokio::test]
    async fn invalid_tsv_is_retried_without_caching_errors() -> anyhow::Result<()> {
        let mut repository = Dsv::<CountingTsv<1>>::new(
            true,
            Bytes::from_static(b"code\tdisclosed_date\n72030\tinvalid\n"),
        );
        let query = queries::get_statements::Query { code: None };
        for expected_calls in 1..=2 {
            assert!(repository.get_row_statements(&query).await.is_err());
            assert_eq!(PARSES[1].load(Ordering::SeqCst), expected_calls);
        }
        repository.buffer = Bytes::from_static(b"code\tdisclosed_date\n72030\t2026-05-08\n");
        assert_eq!(repository.get_row_statements(&query).await?.len(), 1);
        assert_eq!(PARSES[1].load(Ordering::SeqCst), 3);
        Ok(())
    }

    #[test]
    fn concurrent_statement_queries_parse_once() {
        let repository = Dsv::<CountingTsv<2>>::new(
            true,
            Bytes::from_static(b"code\tdisclosed_date\n72030\t2026-05-08\n"),
        );
        let barrier = Barrier::new(8);
        std::thread::scope(|scope| {
            for _ in 0..8 {
                let repository = &repository;
                let barrier = &barrier;
                scope.spawn(move || {
                    barrier.wait();
                    let rows = tokio_test::block_on(repository.get_row_statements(
                        &queries::get_statements::Query {
                            code: Some(SAMPLE_CODE),
                        },
                    ))
                    .unwrap();
                    assert_eq!(rows.len(), 1);
                    assert_eq!(rows[0].code, SAMPLE_CODE);
                });
            }
        });
        assert_eq!(PARSES[2].load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    #[ignore = "manual performance comparison: 10,000 rows and 500 queries"]
    async fn benchmark_statement_cache() -> anyhow::Result<()> {
        let mut writer = csv::WriterBuilder::new()
            .delimiter(b'\t')
            .from_writer(Vec::new());
        let codes: Vec<_> = (10000..10500).map(|code| code.to_string()).collect();
        for index in 0..10_000 {
            let code = &codes[index % codes.len()];
            writer.serialize(tsv::Structure {
                code: code.clone(),
                disclosed_date: Some(NaiveDate::from_ymd_opt(2026, 5, 8).unwrap()),
                opp: Some(index as f64),
                disclosure_json: Some(
                    json!({
                        "code": code,
                        "disclosed_date": "2026-05-08",
                        "disclosed_time": "15:00:00",
                        "disclosure_number": format!("{index:016}"),
                        "document_type": "FYFinancialStatements_Consolidated_JP",
                        "current_period_type": "FY",
                        "current_period_start": "2025-04-01",
                        "current_period_end": "2026-03-31",
                        "current_fiscal_year_start": "2025-04-01",
                        "operating_profit": index as f64,
                        "operating_profit_period": "unspecified",
                        "source_note": "x".repeat(512)
                    })
                    .to_string(),
                ),
                ..Default::default()
            })?;
        }
        let repository = Dsv::<tsv::Structure>::new(true, Bytes::from(writer.into_inner()?));
        let started = Instant::now();
        let mut expected = Vec::new();
        for code in &codes {
            // The previous implementation reparsed and then filtered owned rows.
            let rows = tsv::Structure::process_tabular_data(
                repository.buffer.as_ref(),
                repository.has_headers,
            )?;
            expected.push(
                rows.into_iter()
                    .filter(|row| row.code.eq(code))
                    .collect::<Vec<_>>(),
            );
        }
        let uncached = started.elapsed();
        let started = Instant::now();
        let mut actual = Vec::new();
        for code in &codes {
            actual.push(
                repository
                    .get_row_statements(&queries::get_statements::Query { code: Some(code) })
                    .await?,
            );
        }
        let cached = started.elapsed();
        assert_eq!(actual, expected);
        assert_eq!(actual.iter().map(Vec::len).sum::<usize>(), 10_000);
        println!(
            "rows=10000 queries=500 bytes={} uncached={uncached:?} cached={cached:?} speedup={:.2}x",
            repository.buffer.len(),
            uncached.as_secs_f64() / cached.as_secs_f64()
        );
        Ok(())
    }
}
