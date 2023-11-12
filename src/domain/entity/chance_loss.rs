use crate::domain::entity::cross::CrossDirectionType;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub enum ChanceLoss {
    #[default]
    None,
    GoldenChance,
    DeadChance,
    GoldenLoss,
    DeadLoss,
}

pub struct CrossDirectionChangePair {
    pub cross_direction: CrossDirectionType,
    pub change: f64,
}

impl From<CrossDirectionChangePair> for ChanceLoss {
    fn from(value: CrossDirectionChangePair) -> Self {
        // チャンスロス分析
        // Golden+増減率↑= GoldenChance, Golden+増減率↓= GoldenLoss
        // Dead+増減率↓= DeadChance, Dead+増減率↑= DeadLoss
        match value {
            CrossDirectionChangePair {
                cross_direction: CrossDirectionType::Golden,
                change,
            } if change > 0.0 => ChanceLoss::GoldenChance,
            CrossDirectionChangePair {
                cross_direction: CrossDirectionType::Dead,
                change,
            } if change < 0.0 => ChanceLoss::DeadChance,
            CrossDirectionChangePair {
                cross_direction: CrossDirectionType::Golden,
                change,
            } if change <= 0.0 => ChanceLoss::GoldenLoss,
            CrossDirectionChangePair {
                cross_direction: CrossDirectionType::Dead,
                change,
            } if change >= 0.0 => ChanceLoss::DeadLoss,
            _ => ChanceLoss::None,
        }
    }
}
