use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern;
use crate::domain::models::crossover_strategy::latest_chance::model::LatestChance;
use crate::domain::models::crossover_strategy::rate_of_chance::model::RateOfChance;
use strum::Display;

/// クロスオーバーのどのパターン（ゴールデンクロス、デッドクロス、または両方）を
/// 考慮に入れるかを選択するためのフィルター。
///
/// このフィルターは、表示するパターンを絞り込むだけでなく、関連データ（発生率、最新発生日など）の
/// 取得方法にも影響します。現状はゴールデンクロスとデッドクロスに特化した実装です。
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, Display)]
pub enum CrossoverPatternFilter {
    GoldenOnly,
    DeadOnly,
    #[default]
    Both,
}

impl CrossoverPatternFilter {
    /// 指定されたCrossoverパターンが、現在のフィルター条件に一致するかどうかを判定します。
    ///
    /// # 引数
    /// * `pattern` - 判定対象の `model::Pattern`。
    ///
    /// # 戻り値
    /// フィルター条件に一致する場合は `true`、そうでない場合は `false`。
    pub fn matches_filter(&self, crossover_pattern: &CrossoverPattern) -> bool {
        match self {
            CrossoverPatternFilter::Both => true,
            CrossoverPatternFilter::GoldenOnly => {
                crossover_pattern.eq(&CrossoverPattern::GoldenCross)
            }
            CrossoverPatternFilter::DeadOnly => crossover_pattern.eq(&CrossoverPattern::DeadCross),
        }
    }

    /// フィルターの種類に応じて、対応する発生率 (`RateOfChance`) を取得します。
    ///
    /// # 引数
    /// * `rate_of_chance` - 発生率データを含む `RateOfChance` 構造体への参照。
    ///
    /// # 戻り値
    /// フィルターに対応する発生率 (`f64`)。
    /// `All` の場合は全体(`whole`)、`GoldenOnly` の場合は `golden`、`DeadOnly` の場合は `dead`。
    pub fn get_rate_of_chance(self, rate_of_chance: &RateOfChance) -> f64 {
        match self {
            CrossoverPatternFilter::Both => rate_of_chance.whole,
            CrossoverPatternFilter::GoldenOnly => rate_of_chance.golden,
            CrossoverPatternFilter::DeadOnly => rate_of_chance.dead,
        }
    }

    /// フィルターの種類に応じて、対応する最新発生日情報 (`LatestChance`) を加工して取得します。
    /// `GoldenOnly` や `DeadOnly` の場合、不要な側の情報は `None` になります。
    ///
    /// # 引数
    /// * `latest_chance` - 最新発生日データを含む `LatestChance` 構造体への参照。
    ///
    /// # 戻り値
    /// フィルターに基づいて加工された `LatestChance`。
    pub fn get_latest_chance(self, latest_chance: &LatestChance) -> LatestChance {
        match self {
            CrossoverPatternFilter::Both => *latest_chance,
            CrossoverPatternFilter::GoldenOnly => LatestChance {
                golden_cross: latest_chance.golden_cross,
                dead_cross: None,
            },
            CrossoverPatternFilter::DeadOnly => LatestChance {
                golden_cross: None,
                dead_cross: latest_chance.dead_cross,
            },
        }
    }

    /// 指定された RateOfChance を、フィルターに基づいて選択し、パーセンテージ形式の文字列にフォーマットします。
    /// 例: `75.0` -> `"75%"`
    ///
    /// # 引数
    /// * `rate_of_chance` - 発生率データを含む `RateOfChance` 構造体への参照。
    ///
    /// # 戻り値
    /// フォーマットされたパーセンテージ文字列。
    pub fn format_rate_of_chance_percent(&self, rate_of_chance: &RateOfChance) -> String {
        let specific_rate = self.get_rate_of_chance(rate_of_chance);
        format!("{:.0}%", specific_rate)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern;
    use crate::domain::models::crossover_strategy::latest_chance::model::LatestChance;
    use crate::domain::models::crossover_strategy::rate_of_chance::model::RateOfChance;
    use crate::presenter::views::shared::crossover_pattern_filter::CrossoverPatternFilter;
    use chrono::NaiveDate;

    fn ymd(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn matches_filter_test() {
        assert!(CrossoverPatternFilter::Both.matches_filter(&CrossoverPattern::GoldenCross));
        assert!(CrossoverPatternFilter::Both.matches_filter(&CrossoverPattern::DeadCross));
        assert!(CrossoverPatternFilter::Both.matches_filter(&CrossoverPattern::Neither)); // NeutralもBothではtrue

        assert!(CrossoverPatternFilter::GoldenOnly.matches_filter(&CrossoverPattern::GoldenCross));
        assert!(!CrossoverPatternFilter::GoldenOnly.matches_filter(&CrossoverPattern::DeadCross));
        assert!(!CrossoverPatternFilter::GoldenOnly.matches_filter(&CrossoverPattern::Neither));

        assert!(!CrossoverPatternFilter::DeadOnly.matches_filter(&CrossoverPattern::GoldenCross));
        assert!(CrossoverPatternFilter::DeadOnly.matches_filter(&CrossoverPattern::DeadCross));
        assert!(!CrossoverPatternFilter::DeadOnly.matches_filter(&CrossoverPattern::Neither));
    }

    #[test]
    fn get_rate_of_chance_test() {
        let rates = RateOfChance {
            whole: 50.0,
            golden: 75.0,
            dead: 25.0,
        };
        assert_eq!(
            CrossoverPatternFilter::Both.get_rate_of_chance(&rates),
            50.0
        );
        assert_eq!(
            CrossoverPatternFilter::GoldenOnly.get_rate_of_chance(&rates),
            75.0
        );
        assert_eq!(
            CrossoverPatternFilter::DeadOnly.get_rate_of_chance(&rates),
            25.0
        );
    }

    #[test]
    fn get_latest_chance_test() {
        let latest = LatestChance {
            golden_cross: Some(ymd(2023, 1, 10)),
            dead_cross: Some(ymd(2023, 1, 5)),
        };

        let both_chance = CrossoverPatternFilter::Both.get_latest_chance(&latest);
        assert_eq!(both_chance.golden_cross, Some(ymd(2023, 1, 10)));
        assert_eq!(both_chance.dead_cross, Some(ymd(2023, 1, 5)));

        let golden_chance = CrossoverPatternFilter::GoldenOnly.get_latest_chance(&latest);
        assert_eq!(golden_chance.golden_cross, Some(ymd(2023, 1, 10)));
        assert_eq!(golden_chance.dead_cross, None);

        let dead_chance = CrossoverPatternFilter::DeadOnly.get_latest_chance(&latest);
        assert_eq!(dead_chance.golden_cross, None);
        assert_eq!(dead_chance.dead_cross, Some(ymd(2023, 1, 5)));

        let latest_none = LatestChance {
            golden_cross: None,
            dead_cross: None,
        };
        let both_none = CrossoverPatternFilter::Both.get_latest_chance(&latest_none);
        assert_eq!(both_none.golden_cross, None);
        assert_eq!(both_none.dead_cross, None);
        let golden_none = CrossoverPatternFilter::GoldenOnly.get_latest_chance(&latest_none);
        assert_eq!(golden_none.golden_cross, None);
        assert_eq!(golden_none.dead_cross, None); // 元がNoneならNone
        let dead_none = CrossoverPatternFilter::DeadOnly.get_latest_chance(&latest_none);
        assert_eq!(dead_none.golden_cross, None); // 元がNoneならNone
        assert_eq!(dead_none.dead_cross, None);
    }

    #[test]
    fn format_rate_of_chance_percent_test() {
        let rates = RateOfChance {
            whole: 50.0,
            golden: 75.5,
            dead: 24.9,
        };
        assert_eq!(
            CrossoverPatternFilter::Both.format_rate_of_chance_percent(&rates),
            "50%"
        );
        assert_eq!(
            CrossoverPatternFilter::GoldenOnly.format_rate_of_chance_percent(&rates),
            "76%"
        ); // 四捨五入される
        assert_eq!(
            CrossoverPatternFilter::DeadOnly.format_rate_of_chance_percent(&rates),
            "25%"
        ); // 四捨五入される

        let rates_zero = RateOfChance {
            whole: 0.0,
            golden: 0.0,
            dead: 0.0,
        };
        assert_eq!(
            CrossoverPatternFilter::Both.format_rate_of_chance_percent(&rates_zero),
            "0%"
        );

        let rates_hundred = RateOfChance {
            whole: 100.0,
            golden: 100.0,
            dead: 100.0,
        };
        assert_eq!(
            CrossoverPatternFilter::Both.format_rate_of_chance_percent(&rates_hundred),
            "100%"
        );
    }
}
