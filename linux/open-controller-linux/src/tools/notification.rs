use notify_rust::Notification;
use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NotificationArgs {
    pub title: String,
    pub message: String,
    #[serde(default = "default_app_id")]
    pub app_id: String,
}

fn default_app_id() -> String {
    "open-controller-linux".to_string()
}

pub fn run_notification(
    args: &NotificationArgs,
    confirm_destructive: bool,
) -> anyhow::Result<String> {
    if !confirm_destructive {
        anyhow::bail!("notification spoofing requires --confirm-destructive");
    }
    Notification::new()
        .summary(&args.title)
        .body(&args.message)
        .appname(&args.app_id)
        .show()?;
    Ok("notification sent".to_string())
}
