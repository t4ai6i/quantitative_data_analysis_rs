use crate::domain::entity::macos::MACOSType;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub enum ChanceLoss {
    #[default]
    None,
    GoldenChance,
    DeadChance,
    GoldenLoss,
    DeadLoss,
}

pub struct MACOSTypeRateOfChangePair {
    pub r#type: MACOSType,
    pub rate_of_change: f64,
}

impl From<MACOSTypeRateOfChangePair> for ChanceLoss {
    fn from(value: MACOSTypeRateOfChangePair) -> Self {
        // チャンスロス分析
        // Golden+増減率↑= GoldenChance, Golden+増減率↓= GoldenLoss
        // Dead+増減率↓= DeadChance, Dead+増減率↑= DeadLoss
        match value {
            MACOSTypeRateOfChangePair {
                r#type: MACOSType::Golden,
                rate_of_change: change,
            } if change > 0.0 => ChanceLoss::GoldenChance,
            MACOSTypeRateOfChangePair {
                r#type: MACOSType::Dead,
                rate_of_change: change,
            } if change < 0.0 => ChanceLoss::DeadChance,
            MACOSTypeRateOfChangePair {
                r#type: MACOSType::Golden,
                rate_of_change: change,
            } if change <= 0.0 => ChanceLoss::GoldenLoss,
            MACOSTypeRateOfChangePair {
                r#type: MACOSType::Dead,
                rate_of_change: change,
            } if change >= 0.0 => ChanceLoss::DeadLoss,
            _ => ChanceLoss::None,
        }
    }
}
