use crate::domain::models::candle_stick::model::CandleSticks;
use crate::domain::models::ecp1::model::ECP1s;
use crate::domain::models::ecp2::model::ECP2s;
use crate::domain::models::indicator::model::Indicator;
use crate::domain::models::indicator_analysis::model::{IndicatorAnalysis, IndicatorAnalysisSet};
use crate::domain::models::macos::model::MACOSes;
use crate::domain::models::macos_analysis::close::model::MACOSAnalysisCloses;
use crate::domain::models::macos_analysis::volume::model::MACOSAnalysisVolumes;
use crate::domain::models::macps::model::MACPS;
use crate::domain::models::msesp::model::MSESPes;
use crate::domain::models::sma::model::{SMAListPair, SMAListTrio, SMAs};
use crate::domain::models::stocks_macoses_pair::model::StocksMACOSESPair;
use crate::domain::models::trend_reversal_analysis::model::TrendReversalAnalysis;
use crate::domain::models::trend_reversal_analysis::model::TrendReversalAnalysisSet;
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
            .get_stocks(
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

        let smas_5 = SMAs::<5>::from(stocks.as_slice());
        let smas_25 = SMAs::<25>::from(stocks.as_slice());

        let sma_list_pair = SMAListPair {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
        };
        let macoses = MACOSes::from(sma_list_pair);

        let smas_50 = SMAs::<50>::from(stocks.as_slice());
        let sma_list_trio = SMAListTrio {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
            smas_p: smas_50.as_slice(),
        };
        let macps = MACPS::from((stocks.as_slice(), sma_list_trio));

        let stocks_from_end_days = VecT(stocks.as_slice()).get_from_end(FROM_END_DAYS);
        let ecp1s = ECP1s::from(stocks_from_end_days.as_slice());

        let stocks_macoses_pair = StocksMACOSESPair {
            stocks: stocks_from_end_days.as_slice(),
            macoses: macoses.as_slice(),
        };
        let macos_analysis_closes = MACOSAnalysisCloses::<AFTER_DAYS>::from(&stocks_macoses_pair);
        let rate_of_chance = macos_analysis_closes.rate_of_chance();
        let latest_chance = macos_analysis_closes.latest_chance();

        let macos_analysis_volumes = MACOSAnalysisVolumes::from(&stocks_macoses_pair);

        let candle_sticks =
            CandleSticks::<MARUBOZU_MIN_RATE>::try_from(stocks_from_end_days.as_slice())?;
        let candle_sticks_from_end_days =
            VecT(candle_sticks.as_slice()).get_from_end(FROM_END_DAYS);
        let ecp2s = ECP2s::from(candle_sticks_from_end_days.as_slice());
        let msespes = MSESPes::from(candle_sticks_from_end_days.as_slice());
        // 相場転換を分析
        let trend_reversal_analysis_set = TrendReversalAnalysisSet {
            ecp2s: &ecp2s,
            msesps: &msespes,
            macps: &macps,
            macoses: &macoses,
        };
        let trend_reversal_analysis = TrendReversalAnalysis::from(trend_reversal_analysis_set);

        let indicator = Indicator::from((stocks_from_end_days.as_slice(), &statement));
        let indicator_analysis_set = IndicatorAnalysisSet { indicator };
        let indicator_analysis = IndicatorAnalysis::from(indicator_analysis_set);

        let output = TrendAnalysisOutput {
            company,
            stocks,
            smas_5,
            smas_25,
            smas_50,
            macoses,
            macos_analysis_closes,
            rate_of_chance,
            latest_chance,
            macos_analysis_volumes,
            ecp1s,
            candle_sticks,
            trend_reversal_analysis,
            indicator_analysis,
            display_macos_pattern: input.display_macos_pattern,
        };
        Ok(output)
    }
}
