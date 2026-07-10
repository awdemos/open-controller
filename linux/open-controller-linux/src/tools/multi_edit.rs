use rmcp::schemars;
use serde::Deserialize;

use crate::linux::input::InputBackend;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MultiEditLoc {
    pub x: i32,
    pub y: i32,
    pub text: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MultiEditArgs {
    #[serde(default)]
    pub locs: Option<Vec<MultiEditLoc>>,
    #[serde(default = "default_false")]
    pub confirm: bool,
}

fn default_false() -> bool {
    false
}

pub fn run_multi_edit(args: &MultiEditArgs) -> anyhow::Result<String> {
    if !args.confirm {
        anyhow::bail!("multi_edit requires confirm=true");
    }
    let locs = args.locs.as_deref().ok_or_else(|| anyhow::anyhow!("locs is required"))?;
    if locs.is_empty() {
        anyhow::bail!("locs must not be empty");
    }

    let mut backend = InputBackend::try_new()?;
    let mut total = 0usize;
    for loc in locs {
        backend.click(0, loc.x, loc.y)?;
        backend.type_text(&loc.text)?;
        total += loc.text.len();
    }
    Ok(format!("edited {} locations with {} characters", locs.len(), total))
}
