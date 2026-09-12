use deref_derive::{Deref, DerefMut};
use serde::Deserialize;

const DOMESTIC_STOCK_PRODUCT_CATEGORY: &str = "011";
const TARGET_MARKETS: [&str; 3] = ["0111", "0112", "0113"];

#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct RowCompany {
    pub code: Option<String>,
    pub name: Option<String>,
    pub market: Option<String>,
    pub product_category: Option<String>,
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct Company {
    pub code: String,
    pub name: String,
    pub market: String,
    pub product_category: String,
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct Companies(pub Vec<Company>);

impl Companies {
    pub fn domestic_prime_standard_growth_companies(&self) -> Vec<Company> {
        self.iter()
            .filter(|company| is_target_company(company))
            .cloned()
            .collect()
    }
}

impl TryFrom<RowCompany> for Company {
    type Error = anyhow::Error;

    fn try_from(value: RowCompany) -> Result<Self, Self::Error> {
        let RowCompany {
            code,
            name,
            market,
            product_category,
            symbol,
        } = value;
        Ok(Self {
            code: code.ok_or_else(|| anyhow::anyhow!("code is None"))?,
            name: name.ok_or_else(|| anyhow::anyhow!("name is None"))?,
            market: market.ok_or_else(|| anyhow::anyhow!("market is None"))?,
            product_category: product_category
                .ok_or_else(|| anyhow::anyhow!("product_category is None"))?,
            symbol: symbol.ok_or_else(|| anyhow::anyhow!("symbol is None"))?,
        })
    }
}

fn is_target_company(company: &Company) -> bool {
    company.product_category.as_str() == DOMESTIC_STOCK_PRODUCT_CATEGORY
        && TARGET_MARKETS.contains(&company.market.as_str())
}

#[cfg(test)]
mod tests {
    use super::{Companies, Company};

    #[test]
    fn target_companies_returns_domestic_prime_standard_growth_only() {
        let companies = Companies(vec![
            Company {
                code: "1111".to_string(),
                name: "Prime Domestic".to_string(),
                market: "0111".to_string(),
                product_category: "011".to_string(),
                symbol: "1111.0111".to_string(),
            },
            Company {
                code: "2222".to_string(),
                name: "ETF".to_string(),
                market: "0109".to_string(),
                product_category: "014".to_string(),
                symbol: "2222.0109".to_string(),
            },
            Company {
                code: "3333".to_string(),
                name: "Standard Domestic".to_string(),
                market: "0112".to_string(),
                product_category: "011".to_string(),
                symbol: "3333.0112".to_string(),
            },
            Company {
                code: "4444".to_string(),
                name: "Growth Domestic".to_string(),
                market: "0113".to_string(),
                product_category: "011".to_string(),
                symbol: "4444.0113".to_string(),
            },
        ]);

        let actual = companies.domestic_prime_standard_growth_companies();

        assert_eq!(actual.len(), 3);
        assert_eq!(actual[0].code, "1111");
        assert_eq!(actual[1].code, "3333");
        assert_eq!(actual[2].code, "4444");
    }
}
