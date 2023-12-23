use crate::domain::entity::cross::CrossDirectionType;
use crate::presenter::trend_analysis_presenter::TrendAnalysisResponse;
use itertools::Itertools;

pub struct VecTrendAnalysisResponse(pub Vec<TrendAnalysisResponse>);

pub trait VecTrendAnalysisResponseExt {
    fn table_chart_rows(&self) -> Vec<Vec<String>>;
}

impl VecTrendAnalysisResponseExt for VecTrendAnalysisResponse {
    fn table_chart_rows(&self) -> Vec<Vec<String>> {
        self.0
            .iter()
            .map(|response| {
                let TrendAnalysisResponse::Chart {
                    company,
                    display_cross_pattern,
                    chance_rate,
                    latest_chance,
                    ..
                } = response;
                let code = company.code.clone();
                let symbol = company.symbol.clone();
                let name = company.name.clone();
                let latest_chance = display_cross_pattern.get_latest_chance(latest_chance);
                let cross_direction_type = CrossDirectionType::from(latest_chance);
                let cross_direction = cross_direction_type.to_string();
                let chance_rate = chance_rate.to_string(display_cross_pattern);
                let latest_chance = latest_chance.to_string();
                vec![
                    code,
                    symbol,
                    name,
                    cross_direction,
                    chance_rate,
                    latest_chance,
                ]
            })
            .collect_vec()
    }
}
