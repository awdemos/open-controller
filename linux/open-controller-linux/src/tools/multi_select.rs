use rmcp::schemars;
use serde::Deserialize;

use crate::linux::input::InputBackend;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MultiSelectArgs {
    #[serde(default)]
    pub locs: Option<Vec<[i32; 2]>>,
    #[serde(default = "default_false")]
    pub confirm: bool,
}

fn default_false() -> bool {
    false
}

pub fn run_multi_select(args: &MultiSelectArgs) -> anyhow::Result<String> {
    if !args.confirm {
        anyhow::bail!("multi_select requires confirm=true");
    }
    let locs = args
        .locs
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("locs is required"))?;
    if locs.is_empty() {
        anyhow::bail!("locs must not be empty");
    }

    let mut backend = InputBackend::try_new()?;
    for &[x, y] in locs {
        backend.click(0, x, y)?;
    }
    Ok(format!("selected {} locations", locs.len()))
}
