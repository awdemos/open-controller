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
}

pub fn run_clipboard(args: &ClipboardArgs, confirm_destructive: bool) -> anyhow::Result<String> {
    if !confirm_destructive {
        anyhow::bail!("clipboard operations require --confirm-destructive");
    }
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
