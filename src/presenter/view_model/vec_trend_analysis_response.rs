use crate::domain::entity::candle_stick_pattern_date_set::CandleStickPatternDateSet;
use crate::domain::entity::company::Company;
use crate::domain::entity::macos::MACOSType;
use crate::presenter::trend_analysis_presenter::TrendAnalysisResponse;
use crate::presenter::view_model::analysis::Analysis;
use crate::presenter::view_model::buy_sell_signal_analysis::BuySellSignalAnalysis;
use crate::presenter::view_model::macos_analysis::MACOSAnalysis;
use crate::presenter::view_model::macps_analysis::MACPSAnalysis;
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
                    display_macos_pattern,
                    chance_rate,
                    latest_chance,
                    ..
                } => {
                    let Company { code, symbol, .. } = company;
                    let latest_chance = display_macos_pattern.get_latest_chance(&latest_chance);
                    let macos = MACOSType::from(latest_chance).to_string();
                    let latest_chance = latest_chance.to_string();
                    let chance_rate = chance_rate.to_string(&display_macos_pattern);
                    Some(vec![code, symbol, macos, latest_chance, chance_rate])
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
                    display_macos_pattern,
                    chance_rate,
                    latest_chance,
                    latest_golden_macos,
                    latest_dead_macos,
                    latest_ecp2_buy,
                    latest_ecp2_sell,
                    latest_msesp_buy,
                    latest_msesp_sell,
                    vec_ecp1,
                    macps,
                } => {
                    let Company { code, symbol, .. } = company;
                    let latest_chance = display_macos_pattern.get_latest_chance(&latest_chance);
                    let r#type = MACOSType::from(latest_chance);
                    let latest_chance = NaiveDate::from(latest_chance);
                    let chance_rate = display_macos_pattern.get_chance_rate(&chance_rate);
                    let macos_analysis = MACOSAnalysis {
                        r#type,
                        latest_chance,
                        chance_rate,
                    };
                    let ecp1_analysis = BuySellSignalAnalysis::from(vec_ecp1.0.as_slice());
                    let macps_analysis = MACPSAnalysis {
                        r#type: macps.r#type,
                        date: macps.date,
                    };
                    let golden_buy_ecp2_msesp =
                        match (latest_golden_macos, latest_ecp2_buy, latest_msesp_buy) {
                            (Some(macos_date), Some(ecp2_date), Some(msesp_date)) => {
                                Some(CandleStickPatternDateSet {
                                    macos_date,
                                    ecp2_date,
                                    msesp_date,
                                })
                            }
                            _ => None,
                        };
                    let dead_sell_ecp2_msesp =
                        match (latest_dead_macos, latest_ecp2_sell, latest_msesp_sell) {
                            (Some(macos_date), Some(ecp2_date), Some(msesp_date)) => {
                                Some(CandleStickPatternDateSet {
                                    macos_date,
                                    ecp2_date,
                                    msesp_date,
                                })
                            }
                            _ => None,
                        };
                    Some(Analysis {
                        code,
                        symbol,
                        macos_analysis,
                        ecp1_analysis,
                        macps_analysis,
                        buy_candle_stick_pattern: golden_buy_ecp2_msesp,
                        sell_candle_stick_pattern: dead_sell_ecp2_msesp,
                    })
                }
                _ => None,
            })
            .collect_vec()
    }
}
