use crate::domain::entity::cross::VecCross;
use crate::domain::entity::sma::VecSMA;
use crate::domain::entity::stock::VecStock;
use crate::domain::entity::trend_analysis::VecTrendAnalysis;
use anyhow::Result;

pub mod chart;

pub struct TrendAnalysisOutput<const N: usize> {
    vec_stock: VecStock,
    vec_sma_5: VecSMA<5>,
    vec_sma_25: VecSMA<25>,
    vec_cross: VecCross,
    vec_trend: VecTrendAnalysis<N>,
}

impl<const N: usize> TrendAnalysisOutput<N> {
    pub fn new(
        vec_stock: VecStock,
        vec_sma_5: VecSMA<5>,
        vec_sma_25: VecSMA<25>,
        vec_cross: VecCross,
        vec_trend: VecTrendAnalysis<N>,
    ) -> Self {
        Self {
            vec_stock,
            vec_sma_5,
            vec_sma_25,
            vec_cross,
            vec_trend,
        }
    }
}

pub enum TrendAnalysisResponse {
    Chart { body: String },
}

pub trait TrendAnalysisPresenter {
    fn handle<const N: usize>(
        &self,
        output: TrendAnalysisOutput<N>,
    ) -> Result<TrendAnalysisResponse>;
}
