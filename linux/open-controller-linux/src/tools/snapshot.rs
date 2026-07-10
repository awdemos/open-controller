use base64::Engine;
use rmcp::schemars;
use serde::Deserialize;
use serde_json::json;

use crate::linux::{screen, window};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SnapshotArgs {
    #[serde(default = "default_false")]
    pub annotate: bool,
    #[serde(default = "default_false")]
    pub confirm: bool,
}

fn default_false() -> bool {
    false
}

pub fn run_snapshot(args: &SnapshotArgs) -> anyhow::Result<String> {
    if !args.confirm {
        anyhow::bail!("snapshot requires confirm=true");
    }
    let screenshot = screen::capture()?;
    let windows = window::list_windows().unwrap_or_default();

    let b64 = base64::prelude::BASE64_STANDARD.encode(&screenshot.png_bytes);
    let windows_json: Vec<serde_json::Value> = windows
        .iter()
        .map(|w| {
            json!({
                "id": w.id,
                "title": w.title,
                "class": w.class,
            })
        })
        .collect();

    let out = json!({
        "width": screenshot.width,
        "height": screenshot.height,
        "screenshot": format!("data:image/png;base64,{}", b64),
        "windows": windows_json,
        "annotated": args.annotate,
    });
    Ok(out.to_string())
}
