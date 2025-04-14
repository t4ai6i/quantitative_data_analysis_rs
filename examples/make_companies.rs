use csv::WriterBuilder;
use quantitative_data_analysis_rs::domain::repositories::company::repository::Company;
use quantitative_data_analysis_rs::infrastructure::data_format::DataFormat;
use quantitative_data_analysis_rs::infrastructure::jquants_api::JQuantsAPI;
use quantitative_data_analysis_rs::infrastructure::repositories::company::data_format::tsv::Tsv;
use quantitative_data_analysis_rs::utils::jquants_api::setup::Setup;
use tokio::fs::write;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = Setup::run().await?;
    let data_format = DataFormat::JQuantsAPI;
    let repository = JQuantsAPI::new(token.id_token.value, data_format)?;
    let companies = repository.get_companies().await?;
    let mut writer = WriterBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_writer(vec![]);
    companies.iter().for_each(|company| {
        let tsv = Tsv {
            code: company.code.clone(),
            name: company.name.clone(),
        };
        writer.serialize(tsv).unwrap();
    });
    writer.flush()?;
    write("./examples/companies.tsv", writer.get_ref()).await?;
    Ok(())
}
