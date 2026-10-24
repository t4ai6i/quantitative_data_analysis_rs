use crate::domain::models::crossover_strategy::rate_of_chance::model::RateOfChance;
use crate::presenter::views::shared::crossover_pattern_filter::CrossoverPatternFilter;

impl RateOfChance {
    pub fn table_chart_summary(&self, pattern: &CrossoverPatternFilter) -> Vec<Vec<String>> {
        let rate_of_chance = pattern.format_rate_of_chance_percent(self);
        vec![vec![
            "".to_string(),
            rate_of_chance,
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        ]]
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::models::crossover_strategy::sma_cos::analysis_result::close::model::AnalysisResults;
    use crate::domain::models::crossover_strategy::sma_cos::model::SmaCoses;
    use crate::domain::models::sma::model::{SMAListPair, SMAs};
    use crate::domain::models::stocks_sma_coses_pair::model::StocksSmaCosesPair;
    use crate::domain::repositories::stock::queries;
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::stock::structures::internal::csv;
    use crate::presenter::views::shared::crossover_pattern_filter::CrossoverPatternFilter;
    use anyhow::Result;
    use bytes::Bytes;
    use pretty_assertions::assert_eq;

    const AFTER_DAYS: usize = 5;
    const CSV: &[u8] = include_bytes!("../../../../assets/8473.T.csv");

    #[tokio::test]
    async fn table_chart_summary_test() -> Result<()> {
        let dsv = Dsv::<csv::Structure>::new(true, Bytes::from(CSV));
        let query = queries::get_stocks::Query {
            ..Default::default()
        };
        let stocks = dsv.get_stocks(&query).await?;
        let smas_5 = SMAs::<5>::from(stocks.as_slice());
        let smas_25 = SMAs::<25>::from(stocks.as_slice());
        let sma_list_pair = SMAListPair {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
        };
        let sma_coses = SmaCoses::from(sma_list_pair);
        let stocks_sma_coses_pair = StocksSmaCosesPair {
            stocks: stocks.as_slice(),
            sma_coses: sma_coses.as_slice(),
        };
        let analysis_results = AnalysisResults::<AFTER_DAYS>::from(&stocks_sma_coses_pair);
        let rate_of_chance = analysis_results.rate_of_chance();
        let summary = rate_of_chance.table_chart_summary(&CrossoverPatternFilter::Both);
        assert_eq!(
            summary,
            vec![vec![
                "".to_string(),
                "27%".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ]]
        );
        let summary = rate_of_chance.table_chart_summary(&CrossoverPatternFilter::GoldenOnly);
        assert_eq!(
            summary,
            vec![vec![
                "".to_string(),
                "33%".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ]]
        );
        let summary = rate_of_chance.table_chart_summary(&CrossoverPatternFilter::DeadOnly);
        assert_eq!(
            summary,
            vec![vec![
                "".to_string(),
                "20%".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ]]
        );
        Ok(())
    }
}
