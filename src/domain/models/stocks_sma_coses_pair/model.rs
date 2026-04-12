use crate::domain::models::crossover_strategy::sma_cos::model::SmaCos;
use crate::domain::models::stock::model::Stock;

pub struct StocksSmaCosesPair<'a> {
    pub stocks: &'a [Stock],
    pub sma_coses: &'a [SmaCos],
}
