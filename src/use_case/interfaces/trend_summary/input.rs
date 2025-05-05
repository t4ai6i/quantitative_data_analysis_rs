use crate::presenter::presenters::trend_analysis::response::TrendAnalyses;

pub struct TrendSummary {
    pub trend_analyses: TrendAnalyses,
}

impl TrendSummary {
    pub fn new(trend_analyses: TrendAnalyses) -> Self {
        Self { trend_analyses }
    }
}
