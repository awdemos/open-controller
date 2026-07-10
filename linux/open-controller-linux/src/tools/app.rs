use rmcp::schemars;
use serde::Deserialize;

use crate::linux::window;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AppArgs {
    pub name: String,
    #[serde(default = "default_false")]
    pub launch: bool,
    #[serde(default = "default_false")]
    pub focus: bool,
    #[serde(default = "default_false")]
    pub close: bool,
    #[serde(default = "default_false")]
    pub confirm: bool,
}

fn default_false() -> bool {
    false
}

pub async fn run_app(args: &AppArgs) -> anyhow::Result<String> {
    if !args.confirm {
        anyhow::bail!("app operation requires confirm=true");
    }
    if args.launch {
        let mut cmd = tokio::process::Command::new("sh");
        cmd.arg("-c").arg(&args.name);
        let child = cmd.spawn()?;
        let pid = child.id().unwrap_or(0);
        return Ok(format!("launched '{}' as process {}", args.name, pid));
    }
    if args.focus {
        window::raise_window(&args.name)?;
        return Ok(format!("focused window matching '{}'", args.name));
    }
    if args.close {
        window::close_window(&args.name)?;
        return Ok(format!("closed window matching '{}'", args.name));
    }
    anyhow::bail!("no app action requested; set launch, focus, or close to true")
}
