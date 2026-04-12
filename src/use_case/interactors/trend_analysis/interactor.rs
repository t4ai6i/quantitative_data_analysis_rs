use crate::domain::models::body_engulfing::model::BodyEngulfings;
use crate::domain::models::candle_stick::model::CandleSticks;
use crate::domain::models::crossover_strategy::sma_cos::analysis_result;
use crate::domain::models::crossover_strategy::sma_cos::model::SmaCoses;
use crate::domain::models::crossover_strategy::sma_cps::model::SmaCps;
use crate::domain::models::high_low_direction_signal::model::HighLowDirectionSignals;
use crate::domain::models::ms_es::model::MsEses;
use crate::domain::models::sma::model::{SMAListPair, SMAListTrio, SMAs};
use crate::domain::models::stocks_sma_coses_pair::model::StocksSmaCosesPair;
use crate::domain::models::trend_reversal_analysis::model::TrendReversalAnalysis;
use crate::domain::models::trend_reversal_analysis::model::TrendReversalAnalysisSet;
use crate::domain::repositories;
use crate::domain::repositories::company::queries::get_company;
use crate::domain::repositories::stock::queries::get_stocks;
use crate::presenter::presenters::trend_analysis::output;
use crate::shared::iterator::{FromEnd, SliceWrapper};
use crate::use_case::interfaces::trend_analysis::input;
use crate::use_case::interfaces::trend_analysis::use_case;
use anyhow::Result;
use async_trait::async_trait;

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
    SR: repositories::stock::repository::Stock + Send + Sync,
    CR: repositories::company::repository::Company + Send + Sync,
{
    async fn handle<
        const AFTER_DAYS: usize,
        const FROM_END_DAYS: isize,
        const MARUBOZU_BODY_MIN_RATIO: usize,
        const MARUBOZU_WICK_MAX_RATIO: usize,
        const DOJI_MAX_BODY_RATIO: usize,
    >(
        &self,
        input: input::TrendAnalysis,
    ) -> Result<
        output::TrendAnalysis<
            AFTER_DAYS,
            MARUBOZU_BODY_MIN_RATIO,
            MARUBOZU_WICK_MAX_RATIO,
            DOJI_MAX_BODY_RATIO,
        >,
    > {
        let query = get_company::Query {
            code: input.code.as_str(),
            market: Some(input.market.as_str()),
        };
        let company = self.company_repository.get_company(&query).await?;

        let query = get_stocks::Query {
            code: Some(input.code.as_str()),
            market: Some(input.market.as_str()),
            start_date: Some(input.start_date),
            end_date: Some(input.end_date),
        };
        let stocks = self.stock_repository.get_stocks(&query).await?;
        let smas_5 = SMAs::<5>::from(stocks.as_slice());
        let smas_25 = SMAs::<25>::from(stocks.as_slice());

        let sma_list_pair = SMAListPair {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
        };
        let sma_coses = SmaCoses::from(sma_list_pair);

        let smas_50 = SMAs::<50>::from(stocks.as_slice());
        let sma_list_trio = SMAListTrio {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
            smas_p: smas_50.as_slice(),
        };
        let sma_cps = SmaCps::from((stocks.as_slice(), sma_list_trio));

        let stocks_from_end_days =
            SliceWrapper::from(stocks.as_slice()).get_from_end(FROM_END_DAYS);
        let high_low_direction_signals =
            HighLowDirectionSignals::from(stocks_from_end_days.as_slice());

        let stocks_sma_coses_pair = StocksSmaCosesPair {
            stocks: stocks.as_slice(),
            sma_coses: sma_coses.as_slice(),
        };
        let sma_cos_analysis_result_closes = analysis_result::close::model::AnalysisResults::<
            AFTER_DAYS,
        >::from(&stocks_sma_coses_pair);
        let rate_of_chance = sma_cos_analysis_result_closes.rate_of_chance();
        let latest_chance = sma_cos_analysis_result_closes.latest_chance();

        let sma_cos_analysis_result_volumes =
            analysis_result::volume::model::AnalysisResults::from(&stocks_sma_coses_pair);

        let candle_sticks = CandleSticks::<
            MARUBOZU_BODY_MIN_RATIO,
            MARUBOZU_WICK_MAX_RATIO,
            DOJI_MAX_BODY_RATIO,
        >::try_from(stocks.as_slice())?;
        let candle_sticks_from_end_days =
            SliceWrapper::from(candle_sticks.as_slice()).get_from_end(FROM_END_DAYS);
        let body_engulfings = BodyEngulfings::from(candle_sticks_from_end_days.as_slice());
        let ms_eses = MsEses::from(candle_sticks_from_end_days.as_slice());
        // 相場転換を分析
        let trend_reversal_analysis_set = TrendReversalAnalysisSet {
            body_engulfings: &body_engulfings,
            ms_eses: &ms_eses,
            sma_cps: &sma_cps,
            sma_coses: &sma_coses,
        };
        let trend_reversal_analysis = TrendReversalAnalysis::from(trend_reversal_analysis_set);

        let output = output::TrendAnalysis {
            company,
            stocks,
            smas_5,
            smas_25,
            smas_50,
            sma_coses,
            sma_cos_analysis_result_closes,
            rate_of_chance,
            latest_chance,
            sma_cos_analysis_result_volumes,
            high_low_direction_signals,
            candle_sticks,
            trend_reversal_analysis,
            crossover_pattern_filter: input.crossover_pattern_filter,
        };
        Ok(output)
    }
}
