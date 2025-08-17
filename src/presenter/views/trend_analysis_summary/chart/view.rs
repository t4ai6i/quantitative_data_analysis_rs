use anyhow::bail;
use deref_derive::Deref;
use rayon::prelude::*;

use crate::domain::models::company::model::Company;
use crate::domain::models::macos::model::Pattern;
use crate::presenter::presenters::trend_analysis::response;

pub struct ChartRow(pub Vec<String>);

impl TryFrom<&response::TrendAnalysis> for ChartRow {
    type Error = anyhow::Error;

    fn try_from(value: &response::TrendAnalysis) -> Result<Self, Self::Error> {
        match value {
            response::TrendAnalysis::Chart {
                company,
                crossover_pattern_filter,
                rate_of_chance,
                latest_chance,
                ..
            } => {
                let Company { code, symbol, .. } = company;
                let latest_chance = crossover_pattern_filter.get_latest_chance(latest_chance);
                let macos = Pattern::from(latest_chance).to_string();
                let latest_chance = latest_chance.to_string();
                let rate_of_chance_percent =
                    crossover_pattern_filter.format_rate_of_chance_percent(rate_of_chance);
                Ok(ChartRow(vec![
                    code.to_string(),
                    symbol.to_string(),
                    macos,
                    latest_chance,
                    rate_of_chance_percent,
                ]))
            }
            _ => bail!(
                "Invalid response::TrendAnalysis variant for ChartRow. {:?}",
                value
            ),
        }
    }
}

#[derive(Deref)]
pub struct Chart(pub Vec<ChartRow>);

impl Chart {
    pub fn to_tabular_format(self) -> Vec<Vec<String>> {
        self.0
            .into_iter()
            .map(|row| row.0)
            .collect::<Vec<Vec<String>>>()
    }
}

impl From<response::TrendAnalyses> for Chart {
    fn from(value: response::TrendAnalyses) -> Self {
        let vec_chart_row = value
            .par_iter()
            .filter_map(|trend_analysis| TryFrom::try_from(trend_analysis).ok())
            .collect();
        Self(vec_chart_row)
    }
}
