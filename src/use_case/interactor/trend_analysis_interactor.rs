use crate::domain::entity::candle_stick::VecCandleStick;
use crate::domain::entity::close_macos_trend_analysis::VecCloseMACOSTrendAnalysis;
use crate::domain::entity::ecp1::VecECP1;
use crate::domain::entity::ecp2::VecECP2;
use crate::domain::entity::indicator::Indicator;
use crate::domain::entity::indicator_analysis::{IndicatorAnalysis, IndicatorAnalysisSet};
use crate::domain::entity::macos::VecMACOS;
use crate::domain::entity::macps::MACPS;
use crate::domain::entity::msesp::VecMSESP;
use crate::domain::entity::sma::{SMAListPair, SMAListTrio, VecSMA};
use crate::domain::entity::stocks_macoses_pair::StocksMACOSESPair;
use crate::domain::entity::trend_reversal_analysis::TrendReversalAnalysis;
use crate::domain::entity::trend_reversal_analysis::TrendReversalAnalysisSet;
use crate::domain::entity::volume_macos_trend_analysis::VecVolumeMACOSTrendAnalysis;
use crate::domain::repository::company_repository::CompanyRepository;
use crate::domain::repository::statement_repository::StatementRepository;
use crate::domain::repository::stock_repository::StockRepository;
use crate::presenter::trend_analysis_presenter::TrendAnalysisOutput;
use crate::use_case::interface::trend_analysis_use_case::{
    TrendAnalysisInput, TrendAnalysisUseCase,
};
use crate::utils::iterator::{FromEnd, VecT};
use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TrendAnalysisInteractor<'a, SCR, CR, SMR> {
    stock_repository: &'a SCR,
    company_repository: &'a CR,
    statement_repository: &'a SMR,
}

impl<'a, SCR, CR, SMR> TrendAnalysisInteractor<'a, SCR, CR, SMR> {
    pub fn new(
        stock_repository: &'a SCR,
        company_repository: &'a CR,
        statement_repository: &'a SMR,
    ) -> Self {
        Self {
            stock_repository,
            company_repository,
            statement_repository,
        }
    }
}

#[async_trait]
impl<'a, SCR, CR, SMR> TrendAnalysisUseCase for TrendAnalysisInteractor<'a, SCR, CR, SMR>
where
    SCR: StockRepository + Sync,
    CR: CompanyRepository + Sync,
    SMR: StatementRepository + Sync,
{
    async fn handle<
        const AFTER_DAYS: usize,
        const FROM_END_DAYS: isize,
        const MARUBOZU_MIN_RATE: usize,
    >(
        &self,
        input: TrendAnalysisInput,
    ) -> Result<TrendAnalysisOutput<AFTER_DAYS, MARUBOZU_MIN_RATE>> {
        let company = self
            .company_repository
            .get_company(input.code.as_str(), input.market.as_str())
            .await?;

        let stocks = self
            .stock_repository
            .get_vec_stock(
                input.code.as_str(),
                input.market.as_str(),
                input.start_date,
                input.end_date,
            )
            .await?;

        let statement = self
            .statement_repository
            .get_statement(input.code.as_str())
            .await?;

        let vec_sma_5 = VecSMA::<5>::from(stocks.as_slice());
        let vec_sma_25 = VecSMA::<25>::from(stocks.as_slice());

        let sma_list_pair = SMAListPair {
            smas_n: vec_sma_5.0.as_slice(),
            smas_o: vec_sma_25.0.as_slice(),
        };
        let vec_macos = VecMACOS::from(sma_list_pair);

        let vec_sma_50 = VecSMA::<50>::from(stocks.as_slice());
        let sma_list_trio = SMAListTrio {
            smas_n: vec_sma_5.0.as_slice(),
            smas_o: vec_sma_25.0.as_slice(),
            smas_p: vec_sma_50.0.as_slice(),
        };
        let macps = MACPS::from((stocks.as_slice(), sma_list_trio));

        let stocks_from_end_days = VecT(stocks.as_slice()).get_from_end(FROM_END_DAYS);
        let vec_ecp1 = VecECP1::from(stocks_from_end_days.as_slice());

        let stocks_macoses_pair = StocksMACOSESPair {
            stocks: stocks_from_end_days.as_slice(),
            macoses: vec_macos.0.as_slice(),
        };
        let vec_close_macos_trend_analysis =
            VecCloseMACOSTrendAnalysis::<AFTER_DAYS>::from(&stocks_macoses_pair);
        let vec_volume_macos_trend_analysis =
            VecVolumeMACOSTrendAnalysis::from(&stocks_macoses_pair);

        let vec_candle_stick =
            VecCandleStick::<MARUBOZU_MIN_RATE>::from(stocks_from_end_days.as_slice());
        let candle_sticks = VecT(vec_candle_stick.0.as_slice()).get_from_end(FROM_END_DAYS);
        let vec_ecp2 = VecECP2::from(candle_sticks.as_slice());
        let vec_msesp = VecMSESP::from(candle_sticks.as_slice());
        // 相場転換を分析
        let trend_reversal_analysis_set = TrendReversalAnalysisSet {
            ecp2s: vec_ecp2.0.as_slice(),
            msesps: vec_msesp.0.as_slice(),
            macps: &macps,
            macoses: &vec_macos,
        };

        let trend_reversal_analysis = TrendReversalAnalysis::from(&trend_reversal_analysis_set);

        let indicator = Indicator::from((stocks_from_end_days.as_slice(), &statement));
        let indicator_analysis_set = IndicatorAnalysisSet { indicator };
        let indicator_analysis = IndicatorAnalysis::from(indicator_analysis_set);

        let output = TrendAnalysisOutput {
            company,
            vec_stock: stocks,
            vec_sma_5,
            vec_sma_25,
            vec_sma_50,
            vec_macos,
            vec_close_macos_trend_analysis,
            vec_volume_macos_trend_analysis,
            vec_ecp1,
            vec_candle_stick,
            trend_reversal_analysis,
            indicator_analysis,
            display_macos_pattern: input.display_macos_pattern,
        };
        Ok(output)
    }
}
