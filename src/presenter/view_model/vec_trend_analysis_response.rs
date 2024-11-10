use crate::domain::entity::company::Company;
use crate::domain::entity::macos::MACOSType;
use crate::presenter::trend_analysis_presenter::TrendAnalysisResponse;
use crate::presenter::view_model::analysis::Analysis;
use crate::presenter::view_model::buy_sell_signal_analysis::BuySellSignalAnalysis;
use crate::presenter::view_model::macos_analysis::MACOSAnalysis;
use crate::presenter::view_model::trend_reversal_analysis::TrendReversalAnalysis;
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
                    vec_ecp1,
                    trend_reversal_analysis,
                } => {
                    let Company { code, symbol, .. } = company;
                    let macos_analysis =
                        MACOSAnalysis::from((display_macos_pattern, chance_rate, latest_chance));
                    let ecp1_analysis = BuySellSignalAnalysis::from(vec_ecp1.0.as_slice());
                    let trend_reversal_analysis =
                        TrendReversalAnalysis::from(trend_reversal_analysis);
                    Some(Analysis {
                        code,
                        symbol,
                        macos_analysis,
                        ecp1_analysis,
                        trend_reversal_analysis,
                    })
                }
                _ => None,
            })
            .collect_vec()
    }
}
