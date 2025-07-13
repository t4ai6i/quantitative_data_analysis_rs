use crate::domain::models::macos_analysis::close::model::RateOfChance;
use crate::presenter::view_models::shared::crossover_pattern_filter::CrossoverPatternFilter;

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
    use anyhow::Result;
    use pretty_assertions::assert_eq;

    use crate::domain::models::macos::model::MACOSes;
    use crate::domain::models::macos_analysis::close::model::MACOSAnalysisCloses;
    use crate::domain::models::sma::model::{SMAListPair, SMAs};
    use crate::domain::models::stocks_macoses_pair::model::StocksMACOSESPair;
    use crate::domain::repositories::stock::queries;
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::stock::structures::internal::csv;
    use crate::presenter::view_models::shared::crossover_pattern_filter::CrossoverPatternFilter;

    const AFTER_DAYS: usize = 5;
    const CSV: &[u8] = include_bytes!("../../../../assets/8473.T.csv");

    #[tokio::test]
    async fn table_chart_summary_test() -> Result<()> {
        let dsv = Dsv::<csv::Structure>::new(true, CSV.to_vec());
        let query = queries::get_stocks::Query {
            ..Default::default()
        };
        let stocks = dsv.get_stocks(query).await?;
        let smas_5 = SMAs::<5>::from(stocks.as_slice());
        let smas_25 = SMAs::<25>::from(stocks.as_slice());
        let sma_list_pair = SMAListPair {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
        };
        let macoses = MACOSes::from(sma_list_pair);
        let stocks_macoses_pair = StocksMACOSESPair {
            stocks: stocks.as_slice(),
            macoses: macoses.as_slice(),
        };
        let macos_analysis_closes = MACOSAnalysisCloses::<AFTER_DAYS>::from(&stocks_macoses_pair);
        let rate_of_chance = macos_analysis_closes.rate_of_chance();
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
