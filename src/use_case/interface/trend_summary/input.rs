use crate::presenter::view_model::vec_trend_analysis_response::VecTrendAnalysisResponse;

pub struct TrendSummary {
    pub vec_trend_analysis_response: VecTrendAnalysisResponse,
}

impl TrendSummary {
    pub fn new(vec_trend_analysis_response: VecTrendAnalysisResponse) -> Self {
        Self {
            vec_trend_analysis_response,
        }
    }
}
