use anyhow::Result;
use async_trait::async_trait;

use crate::domain::models::candle_stick::model::CandleSticks;
use crate::domain::models::ecp1::model::ECP1s;
use crate::domain::models::ecp2::model::ECP2s;
use crate::domain::models::macos::model::MACOSes;
use crate::domain::models::macos_analysis::close::model::MACOSAnalysisCloses;
use crate::domain::models::macos_analysis::volume::model::MACOSAnalysisVolumes;
use crate::domain::models::macps::model::MACPS;
use crate::domain::models::msesp::model::MSESPes;
use crate::domain::models::sma::model::{SMAListPair, SMAListTrio, SMAs};
use crate::domain::models::stocks_macoses_pair::model::StocksMACOSESPair;
use crate::domain::models::trend_reversal_analysis::model::TrendReversalAnalysis;
use crate::domain::models::trend_reversal_analysis::model::TrendReversalAnalysisSet;
use crate::domain::repositories;
use crate::domain::repositories::company::queries::get_company;
use crate::domain::repositories::stock::queries::get_stocks;
use crate::presenter::presenters::trend_analysis::output;
use crate::shared::iterator::{FromEnd, SliceWrapper};
use crate::use_case::interfaces::trend_analysis::input;
use crate::use_case::interfaces::trend_analysis::use_case;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TrendAnalysis<'a, SR, CR> {
    stock_repository: &'a SR,
    company_repository: &'a CR,
}

impl<'a, SR, CR> TrendAnalysis<'a, SR, CR> {
    pub fn new(stock_repository: &'a SR, company_repository: &'a CR) -> Self {
        Self {
            stock_repository,
            company_repository,
        }
    }
}

#[async_trait]
impl<SR, CR> use_case::TrendAnalysis for TrendAnalysis<'_, SR, CR>
where
    SR: repositories::stock::repository::Stock + Sync,
    CR: repositories::company::repository::Company + Sync,
{
    async fn handle<
        const AFTER_DAYS: usize,
        const FROM_END_DAYS: isize,
        const MARUBOZU_MIN_RATE: usize,
    >(
        &self,
        input: input::TrendAnalysis,
    ) -> Result<output::TrendAnalysis<AFTER_DAYS, MARUBOZU_MIN_RATE>> {
        let query = get_company::Query {
            code: input.code.as_str(),
            market: Some(input.market.as_str()),
        };
        let company = self.company_repository.get_company(query).await?;

        let query = get_stocks::Query {
            code: Some(input.code.as_str()),
            market: Some(input.market.as_str()),
            start_date: Some(input.start_date),
            end_date: Some(input.end_date),
        };
        let stocks = self.stock_repository.get_stocks(query).await?;

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

        let stocks_from_end_days =
            SliceWrapper::from(stocks.as_slice()).get_from_end(FROM_END_DAYS);
        let ecp1s = ECP1s::from(stocks_from_end_days.as_slice());

        let stocks_macoses_pair = StocksMACOSESPair {
            stocks: stocks.as_slice(),
            macoses: macoses.as_slice(),
        };
        let macos_analysis_closes = MACOSAnalysisCloses::<AFTER_DAYS>::from(&stocks_macoses_pair);
        let rate_of_chance = macos_analysis_closes.rate_of_chance();
        let latest_chance = macos_analysis_closes.latest_chance();

        let macos_analysis_volumes = MACOSAnalysisVolumes::from(&stocks_macoses_pair);

        let candle_sticks = CandleSticks::<MARUBOZU_MIN_RATE>::try_from(stocks.as_slice())?;
        let candle_sticks_from_end_days =
            SliceWrapper::from(candle_sticks.as_slice()).get_from_end(FROM_END_DAYS);
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

        let output = output::TrendAnalysis {
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
            crossover_pattern_filter: input.crossover_pattern_filter,
        };
        Ok(output)
    }
}
