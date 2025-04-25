use rayon::prelude::*;

use crate::domain::models::company::model::Company;
use crate::domain::models::macos::model::Pattern;
use crate::presenter::presenters::trend_analysis::response;
use crate::presenter::presenters::trend_analysis::response::TrendAnalysis;
use crate::presenter::view_models::analysis::view_model::Analysis;
use crate::presenter::view_models::buy_sell_signal_analysis::view_model::BuySellSignalAnalysis;
use crate::presenter::view_models::macos_analysis::view_model::MACOSAnalysis;
use crate::presenter::view_models::trend_reversal_analysis::view_model::TrendReversalAnalysis;

pub trait TrendAnalyses {
    fn table_chart_rows(self) -> Vec<Vec<String>>;
    fn vec_json(self) -> Vec<Analysis>;
}

impl TrendAnalyses for response::TrendAnalyses {
    fn table_chart_rows(self) -> Vec<Vec<String>> {
        self.0
            .into_par_iter()
            .filter_map(|response| match response {
                TrendAnalysis::Chart {
                    company,
                    macos_pattern_filter,
                    rate_of_chance,
                    latest_chance,
                    ..
                } => {
                    let Company { code, symbol, .. } = company;
                    let latest_chance = macos_pattern_filter.get_latest_chance(&latest_chance);
                    let macos = Pattern::from(latest_chance).to_string();
                    let latest_chance = latest_chance.to_string();
                    let rate_of_chance_percent =
                        macos_pattern_filter.format_rate_of_chance_percent(&rate_of_chance);
                    Some(vec![
                        code,
                        symbol,
                        macos,
                        latest_chance,
                        rate_of_chance_percent,
                    ])
                }
                _ => None,
            })
            .collect()
    }

    fn vec_json(self) -> Vec<Analysis> {
        self.0
            .into_par_iter()
            .filter_map(|response| match response {
                TrendAnalysis::Json {
                    company,
                    macos_pattern_filter,
                    rate_of_chance,
                    latest_chance,
                    ecp1s,
                    trend_reversal_analysis,
                    indicator_analysis,
                } => {
                    let Company { code, symbol, .. } = company;
                    let macos_analysis =
                        MACOSAnalysis::from((macos_pattern_filter, rate_of_chance, latest_chance));
                    let ecp1_analysis = BuySellSignalAnalysis::from(ecp1s.as_slice());
                    let trend_reversal_analysis =
                        TrendReversalAnalysis::from(trend_reversal_analysis);
                    Some(Analysis {
                        code,
                        symbol,
                        macos_analysis,
                        ecp1_analysis,
                        trend_reversal_analysis,
                        indicator_analysis,
                    })
                }
                _ => None,
            })
            .collect()
    }
}
