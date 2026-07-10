use arboard::Clipboard;
use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardMode {
    Read,
    Write,
    Clear,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ClipboardArgs {
    pub mode: ClipboardMode,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default = "default_false")]
    pub confirm: bool,
}

fn default_false() -> bool {
    false
}

pub fn run_clipboard(args: &ClipboardArgs) -> anyhow::Result<String> {
    let mut cb = Clipboard::new()?;
    match args.mode {
        ClipboardMode::Read => Ok(cb.get_text()?),
        ClipboardMode::Write => {
            let text = args.content.as_deref().unwrap_or("");
            cb.set_text(text.to_string())?;
            Ok(format!("clipboard set to {} bytes", text.len()))
        }
        ClipboardMode::Clear => {
            cb.set_text(String::new())?;
            Ok("clipboard cleared".to_string())
        }
    }
}
