use crate::domain::entity::candle_stick_pattern_date_set::CandleStickPatternDateSet;
use crate::domain::entity::company::Company;
use crate::domain::entity::cross::CrossDirectionType;
use crate::presenter::trend_analysis_presenter::TrendAnalysisResponse;
use crate::presenter::view_model::analysis::Analysis;
use crate::presenter::view_model::buy_sell_signal_analysis::BuySellSignalAnalysis;
use crate::presenter::view_model::cross_analysis::CrossAnalysis;
use chrono::NaiveDate;
use itertools::Itertools;

pub struct VecTrendAnalysisResponse(pub Vec<TrendAnalysisResponse>);

pub trait VecTrendAnalysisResponseExt {
    fn table_chart_rows(self) -> Vec<Vec<String>>;
    fn vec_json(self) -> Vec<Analysis>;
}

impl VecTrendAnalysisResponseExt for VecTrendAnalysisResponse {
    fn table_chart_rows(self) -> Vec<Vec<String>> {
        self.0
            .into_iter()
            .filter_map(|response| match response {
                TrendAnalysisResponse::Chart {
                    company,
                    display_cross_pattern,
                    chance_rate,
                    latest_chance,
                    ..
                } => {
                    let Company { code, symbol, .. } = company;
                    let latest_chance = display_cross_pattern.get_latest_chance(&latest_chance);
                    let cross_direction_type = CrossDirectionType::from(latest_chance);
                    let cross_direction = cross_direction_type.to_string();
                    let latest_chance = latest_chance.to_string();
                    let chance_rate = chance_rate.to_string(&display_cross_pattern);
                    Some(vec![
                        code,
                        symbol,
                        cross_direction,
                        latest_chance,
                        chance_rate,
                    ])
                }
                _ => None,
            })
            .collect_vec()
    }

    fn vec_json(self) -> Vec<Analysis> {
        self.0
            .into_iter()
            .filter_map(|response| match response {
                TrendAnalysisResponse::Json {
                    company,
                    display_cross_pattern,
                    chance_rate,
                    latest_chance,
                    latest_golden_cross,
                    latest_dead_cross,
                    latest_ecp2_buy,
                    latest_ecp2_sell,
                    latest_msesp_buy,
                    latest_msesp_sell,
                    vec_ecp1,
                } => {
                    let Company { code, symbol, .. } = company;
                    let latest_chance = display_cross_pattern.get_latest_chance(&latest_chance);
                    let cross_direction_type = CrossDirectionType::from(latest_chance);
                    let latest_chance = NaiveDate::from(latest_chance);
                    let chance_rate = display_cross_pattern.get_chance_rate(&chance_rate);
                    let cross_analysis = CrossAnalysis {
                        cross_direction: cross_direction_type,
                        latest_chance,
                        chance_rate,
                    };
                    let ecp1_analysis = BuySellSignalAnalysis::from(vec_ecp1.0.as_slice());
                    let golden_buy_ecp2_msesp =
                        match (latest_golden_cross, latest_ecp2_buy, latest_msesp_buy) {
                            (Some(cross_date), Some(ecp2_date), Some(msesp_date)) => {
                                Some(CandleStickPatternDateSet {
                                    cross_date,
                                    ecp2_date,
                                    msesp_date,
                                })
                            }
                            _ => None,
                        };
                    let dead_sell_ecp2_msesp =
                        match (latest_dead_cross, latest_ecp2_sell, latest_msesp_sell) {
                            (Some(cross_date), Some(ecp2_date), Some(msesp_date)) => {
                                Some(CandleStickPatternDateSet {
                                    cross_date,
                                    ecp2_date,
                                    msesp_date,
                                })
                            }
                            _ => None,
                        };
                    Some(Analysis {
                        code,
                        symbol,
                        cross_analysis,
                        ecp1_analysis,
                        buy_candle_stick_pattern: golden_buy_ecp2_msesp,
                        sell_candle_stick_pattern: dead_sell_ecp2_msesp,
                    })
                }
                _ => None,
            })
            .collect_vec()
    }
}
