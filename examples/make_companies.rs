use csv::WriterBuilder;
use tokio::fs::write;

use quantitative_data_analysis_rs::domain::repositories::company::repository::Company;
use quantitative_data_analysis_rs::infrastructure::jquants_api::JQuantsAPI;
use quantitative_data_analysis_rs::infrastructure::repositories::company::structures::internal::tsv;
use quantitative_data_analysis_rs::shared::jquants_api::setup::Setup;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = Setup::run().await?;
    let jquants_api = JQuantsAPI::new(token.id_token.value)?;
    let companies = jquants_api.get_companies().await?;
    let mut writer = WriterBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_writer(vec![]);
    companies.iter().for_each(|company| {
        let tsv = tsv::Structure {
            code: company.code.clone(),
            name: company.name.clone(),
            market: company.market.clone(),
        };
        writer.serialize(tsv).unwrap();
    });
    writer.flush()?;
    write("./examples/companies.tsv", writer.get_ref()).await?;
    Ok(())
}
