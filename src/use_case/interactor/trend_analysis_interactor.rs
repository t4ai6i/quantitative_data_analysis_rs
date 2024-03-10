use crate::domain::entity::cross::VecCross;
use crate::domain::entity::engulfing_candlestick_pattern::VecEngulfingCandlestickPattern;
use crate::domain::entity::sma::{SMAListPair, VecSMA};
use crate::domain::entity::trend_analysis::{StockCrossPair, VecTrendAnalysis};
use crate::domain::repository::company_repository::CompanyRepository;
use crate::domain::repository::stock_repository::StockRepository;
use crate::presenter::trend_analysis_presenter::TrendAnalysisOutput;
use crate::use_case::interface::trend_analysis_use_case::{
    TrendAnalysisInput, TrendAnalysisUseCase,
};
use crate::utils::iterator::get_vec_containing_number_from_end_of_array;
use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TrendAnalysisInteractor<'a, SR, CR> {
    stock_repository: &'a SR,
    company_repository: &'a CR,
}

impl<'a, SR, CR> TrendAnalysisInteractor<'a, SR, CR> {
    pub fn new(stock_repository: &'a SR, company_repository: &'a CR) -> Self {
        Self {
            stock_repository,
            company_repository,
        }
    }
}

#[async_trait]
impl<'a, SR, CR> TrendAnalysisUseCase for TrendAnalysisInteractor<'a, SR, CR>
where
    SR: StockRepository + Sync,
    CR: CompanyRepository + Sync,
{
    async fn handle<const AFTER_DAYS: usize, const FOR_DAYS: usize>(
        &self,
        input: TrendAnalysisInput,
    ) -> Result<TrendAnalysisOutput<AFTER_DAYS>> {
        let company = self
            .company_repository
            .get_company(input.code.as_str(), input.market.as_str())
            .await?;

        let vec_stock = self
            .stock_repository
            .get_vec_stock(
                input.code.as_str(),
                input.market.as_str(),
                input.start_date,
                input.end_date,
            )
            .await?;

        let vec_sma_5 = VecSMA::<5>::from(vec_stock.0.as_slice());
        let vec_sma_25 = VecSMA::<25>::from(vec_stock.0.as_slice());

        let sma_list_pair = SMAListPair {
            smas_n: vec_sma_5.0.as_slice(),
            smas_o: vec_sma_25.0.as_slice(),
        };
        let vec_cross = VecCross::from(sma_list_pair);

        let stocks = get_vec_containing_number_from_end_of_array(vec_stock.0.as_slice(), FOR_DAYS);
        let vec_buy_sell_signal = VecEngulfingCandlestickPattern::from(stocks.as_slice());

        let stock_cross_pair = StockCrossPair {
            stocks: vec_stock.0.as_slice(),
            crosses: vec_cross.0.as_slice(),
        };
        let vec_trend_analysis = VecTrendAnalysis::<AFTER_DAYS>::from(stock_cross_pair);

        let output = TrendAnalysisOutput::new(
            company,
            vec_stock,
            vec_sma_5,
            vec_sma_25,
            vec_cross,
            vec_trend_analysis,
            vec_buy_sell_signal,
            input.display_cross_pattern,
        );
        Ok(output)
    }
}
