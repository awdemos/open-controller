use rmcp::schemars;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WaitArgs {
    pub duration: u64,
}

pub async fn run_wait(duration: u64) -> String {
    tokio::time::sleep(Duration::from_secs(duration)).await;
    format!("waited {} seconds", duration)
}
