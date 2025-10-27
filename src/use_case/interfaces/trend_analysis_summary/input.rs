use crate::presenter::presenters::trend_analysis::response::TrendAnalyses;

pub struct TrendAnalysisSummary {
    pub trend_analyses: TrendAnalyses,
}

impl TrendAnalysisSummary {
    pub fn new(trend_analyses: TrendAnalyses) -> Self {
        Self { trend_analyses }
    }
}
