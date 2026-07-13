use rmcp::schemars;
use serde::Deserialize;

use crate::linux::input::InputBackend;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TypeArgs {
    pub text: String,
}

pub fn run_type(args: &TypeArgs, confirm_destructive: bool) -> anyhow::Result<String> {
    if !confirm_destructive {
        anyhow::bail!("type requires --confirm-destructive");
    }
    let mut backend = InputBackend::try_new()?;
    backend.type_text(&args.text)?;
    Ok(format!("typed {} characters", args.text.len()))
}
