use reqwest;
use rmcp::schemars;
use scraper::{Html, Selector};
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScrapeArgs {
    pub url: String,
    #[serde(default)]
    pub query: Option<String>,
}

pub async fn run_scrape(args: &ScrapeArgs) -> anyhow::Result<String> {
    let body = reqwest::get(&args.url).await?.text().await?;
    if let Some(query) = args.query.as_deref() {
        let doc = Html::parse_document(&body);
        let selector = Selector::parse(query)
            .map_err(|e| anyhow::anyhow!("invalid CSS selector: {:?}", e))?;
        let texts: Vec<String> = doc
            .select(&selector)
            .map(|e| e.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Ok(texts.join("\n"))
    } else {
        Ok(body)
    }
}
