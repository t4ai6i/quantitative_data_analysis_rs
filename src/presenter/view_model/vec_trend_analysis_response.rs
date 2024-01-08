use chrono::NaiveDate;
use crate::domain::entity::cross::CrossDirectionType;
use crate::presenter::trend_analysis_presenter::TrendAnalysisResponse;
use itertools::Itertools;
use crate::presenter::trend_summary_presenter::JSON;

pub struct VecTrendAnalysisResponse(pub Vec<TrendAnalysisResponse>);

pub trait VecTrendAnalysisResponseExt {
    fn table_chart_rows(&self) -> Vec<Vec<String>>;
    fn vec_json(&self) -> Vec<JSON>;
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
                let latest_chance = display_cross_pattern.get_latest_chance(latest_chance);
                let cross_direction_type = CrossDirectionType::from(latest_chance);
                let cross_direction = cross_direction_type.to_string();
                let latest_chance = latest_chance.to_string();
                let chance_rate = chance_rate.to_string(display_cross_pattern);
                vec![code, symbol, cross_direction, latest_chance, chance_rate]
            })
            .collect_vec()
    }

    fn vec_json(&self) -> Vec<JSON> {
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
                let latest_chance = display_cross_pattern.get_latest_chance(latest_chance);
                let cross_direction_type = CrossDirectionType::from(latest_chance);
                let latest_chance = NaiveDate::from(latest_chance);
                let chance_rate = display_cross_pattern.get_chance_rate(chance_rate);
                JSON {
                    code,
                    symbol,
                    cross_direction: cross_direction_type,
                    latest_chance,
                    chance_rate,
                }
            })
            .collect_vec()
    }
}
