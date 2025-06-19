use anyhow::Result;
use quantitative_data_analysis_rs::shared::jquants_api::setup::Setup;

#[tokio::main]
async fn main() -> Result<()> {
    // JQUANTS APIのためのトークン準備
    let token = Setup::run().await?;

    todo!()
}
