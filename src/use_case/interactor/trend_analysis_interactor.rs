use crate::domain::entity::cross::VecCross;
use crate::domain::entity::sma::{SMAListPair, VecSMA};
use crate::domain::entity::trend_analysis::{StockCrossPair, VecTrendAnalysis};
use crate::domain::repository::stock_repository::StockRepository;
use crate::presenter::trend_analysis_presenter::TrendAnalysisOutput;
use crate::use_case::interface::trend_analysis_use_case::{
    TrendAnalysisInput, TrendAnalysisUseCase,
};
use anyhow::Result;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TrendAnalysisInteractor<'a, R> {
    repository: &'a R,
}

impl<'a, R> TrendAnalysisInteractor<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }
}

impl<'a, R> TrendAnalysisUseCase for TrendAnalysisInteractor<'a, R>
where
    R: StockRepository,
{
    fn handle<const N: usize>(&self, input: TrendAnalysisInput) -> Result<TrendAnalysisOutput<N>> {
        let vec_stock = self.repository.get_vec_stock(
            input.code,
            input.start_date,
            input.end_date,
            input.data_format_type,
        )?;
        let vec_sma_5 = VecSMA::<5>::from(vec_stock.0.as_slice());
        let vec_sma_25 = VecSMA::<25>::from(vec_stock.0.as_slice());
        let sma_list_pair = SMAListPair {
            smas_n: vec_sma_5.0.as_slice(),
            smas_o: vec_sma_25.0.as_slice(),
        };
        let vec_cross = VecCross::from(sma_list_pair);

        let stock_cross_pair = StockCrossPair {
            stocks: vec_stock.0.as_slice(),
            crosses: vec_cross.0.as_slice(),
        };
        let vec_trend_analysis = VecTrendAnalysis::<N>::from(stock_cross_pair);
        let output = TrendAnalysisOutput::new(
            vec_stock,
            vec_sma_5,
            vec_sma_25,
            vec_cross,
            vec_trend_analysis,
        );
        Ok(output)
    }
}
