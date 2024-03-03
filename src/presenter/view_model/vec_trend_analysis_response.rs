use crate::domain::entity::cross::CrossDirectionType;
use crate::presenter::trend_analysis_presenter::TrendAnalysisResponse;
use crate::presenter::view_model::analysis::Analysis;
use crate::presenter::view_model::buy_sell_signal_analysis::BuySellSignalAnalysis;
use crate::presenter::view_model::cross_analysis::CrossAnalysis;
use chrono::NaiveDate;
use itertools::Itertools;

pub struct VecTrendAnalysisResponse(pub Vec<TrendAnalysisResponse>);

pub trait VecTrendAnalysisResponseExt {
    fn table_chart_rows(&self) -> Vec<Vec<String>>;
    fn vec_json(&self) -> Vec<Analysis>;
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

    fn vec_json(&self) -> Vec<Analysis> {
        self.0
            .iter()
            .map(|response| {
                let TrendAnalysisResponse::Chart {
                    company,
                    display_cross_pattern,
                    chance_rate,
                    latest_chance,
                    vec_buy_sell_signal,
                    ..
                } = response;
                let code = company.code.clone();
                let symbol = company.symbol.clone();
                let latest_chance = display_cross_pattern.get_latest_chance(latest_chance);
                let cross_direction_type = CrossDirectionType::from(latest_chance);
                let latest_chance = NaiveDate::from(latest_chance);
                let chance_rate = display_cross_pattern.get_chance_rate(chance_rate);
                let cross_analysis = CrossAnalysis {
                    code: code.clone(),
                    symbol: symbol.clone(),
                    cross_direction: cross_direction_type,
                    latest_chance,
                    chance_rate,
                };
                let mut buy_sell_signal_analysis =
                    BuySellSignalAnalysis::from(vec_buy_sell_signal.0.as_slice());
                buy_sell_signal_analysis.code = code.clone();
                buy_sell_signal_analysis.symbol = symbol.clone();
                Analysis {
                    cross_analysis,
                    buy_sell_signal_analysis,
                }
            })
            .collect_vec()
    }
}
