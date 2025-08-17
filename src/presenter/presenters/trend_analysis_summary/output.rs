use crate::presenter::presenters::trend_analysis::response::TrendAnalyses;
use crate::presenter::presenters::trend_analysis::response::TrendAnalysis;

pub struct TrendAnalysisSummary {
    pub trend_analyses: TrendAnalyses,
    pub vec_trend_analysis: Vec<TrendAnalysis>,
}

impl TrendAnalysisSummary {
    pub fn new(trend_analyses: TrendAnalyses, vec_trend_analysis: Vec<TrendAnalysis>) -> Self {
        Self {
            trend_analyses,
            vec_trend_analysis,
        }
    }
}
