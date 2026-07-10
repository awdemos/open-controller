use base64::Engine;
use rmcp::schemars;
use serde::Deserialize;

use crate::linux::screen;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScreenshotArgs {
    #[serde(default)]
    pub display: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default = "default_false")]
    pub confirm: bool,
}

fn default_false() -> bool {
    false
}

pub fn run_screenshot(args: &ScreenshotArgs) -> anyhow::Result<String> {
    if !args.confirm {
        anyhow::bail!("screenshot requires confirm=true");
    }
    let img = screen::capture()?;
    let b64 = base64::prelude::BASE64_STANDARD.encode(&img.png_bytes);
    Ok(format!(
        "data:image/png;base64,{} {}x{}",
        b64, img.width, img.height
    ))
}
