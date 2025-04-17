use crate::domain::models::macos_analysis::close::model::RateOfChance;
use crate::presenter::display_macos_pattern::DisplayMACOSPattern;

impl RateOfChance {
    pub fn to_string(&self, display_macos_pattern: &DisplayMACOSPattern) -> String {
        let rate_of_chance = display_macos_pattern.get_rate_of_chance(self);
        format!("{:.0}%", rate_of_chance)
    }

    pub fn table_chart_summary(&self, pattern: &DisplayMACOSPattern) -> Vec<Vec<String>> {
        let rate_of_chance = self.to_string(pattern);
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

    use crate::domain::models::macos::model::MACOSes;
    use crate::domain::models::macos_analysis::close::model::MACOSAnalysisCloses;
    use crate::domain::models::sma::model::{SMAListPair, SMAs};
    use crate::domain::models::stocks_macoses_pair::model::StocksMACOSESPair;
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::repositories::stock::data_format::csv::Csv;
    use crate::presenter::display_macos_pattern::DisplayMACOSPattern;

    const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");
    const AFTER_DAYS: usize = 5;

    #[test]
    fn table_chart_summary_test() -> Result<()> {
        let (successes, _): (Vec<_>, Vec<_>) = Csv::from_slice::<true>(CSV_8473)
            .into_iter()
            .partition(Result::is_ok);
        let successes: Vec<_> = successes.into_iter().map(|e| e.unwrap()).collect();
        let vec_stock = Csv::from_deserialize(successes);
        let vec_stock: Vec<_> = vec_stock.into_iter().map(|e| e.unwrap()).collect();
        let smas_5 = SMAs::<5>::from(vec_stock.as_slice());
        let smas_25 = SMAs::<25>::from(vec_stock.as_slice());
        let sma_list_pair = SMAListPair {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
        };
        let macoses = MACOSes::from(sma_list_pair);
        let stocks_macoses_pair = StocksMACOSESPair {
            stocks: vec_stock.as_slice(),
            macoses: macoses.as_slice(),
        };
        let macos_analysis_closes = MACOSAnalysisCloses::<AFTER_DAYS>::from(&stocks_macoses_pair);
        let rate_of_chance = macos_analysis_closes.rate_of_chance();
        let summary = rate_of_chance.table_chart_summary(&DisplayMACOSPattern::All);
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
        let summary = rate_of_chance.table_chart_summary(&DisplayMACOSPattern::GoldenOnly);
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
        let summary = rate_of_chance.table_chart_summary(&DisplayMACOSPattern::DeadOnly);
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
