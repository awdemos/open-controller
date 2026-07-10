use rmcp::schemars;
use serde::Deserialize;

use crate::linux::input::InputBackend;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TypeArgs {
    pub text: String,
    #[serde(default = "default_false")]
    pub confirm: bool,
}

fn default_false() -> bool {
    false
}

pub fn run_type(args: &TypeArgs) -> anyhow::Result<String> {
    if !args.confirm {
        anyhow::bail!("type requires confirm=true");
    }
    let mut backend = InputBackend::try_new()?;
    backend.type_text(&args.text)?;
    Ok(format!("typed {} characters", args.text.len()))
}
