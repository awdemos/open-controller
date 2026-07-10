use open_controller_linux::tools::app::{run_app, AppArgs};
use std::path::PathBuf;

fn make_args(name: &str, launch: bool, focus: bool, close: bool, confirm: bool) -> AppArgs {
    AppArgs {
        name: name.to_string(),
        launch,
        focus,
        close,
        confirm,
    }
}

#[tokio::test]
async fn launch_command_spawns_process() {
    let marker = PathBuf::from(format!(
        "/tmp/open-controller-app-test-{}",
        std::process::id()
    ));
    if marker.exists() {
        std::fs::remove_file(&marker).unwrap();
    }
    let cmd = format!("touch {}", marker.display());
    let args = make_args(&cmd, true, false, false, true);
    let out = run_app(&args).await.unwrap();
    assert!(out.contains("launched"));
    // Wait briefly for the touch to complete.
    for _ in 0..20 {
        if marker.exists() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert!(marker.exists(), "launch command did not create marker file");
    std::fs::remove_file(&marker).ok();
}

#[tokio::test]
async fn app_requires_confirm() {
    let args = make_args("echo hello", true, false, false, false);
    let err = run_app(&args).await.unwrap_err();
    assert!(err.to_string().contains("confirm"));
}

#[tokio::test]
async fn focus_requires_display() {
    if open_controller_linux::linux::detect::detect()
        != open_controller_linux::linux::detect::Compositor::Headless
    {
        return;
    }
    let args = make_args("foo", false, true, false, true);
    let err = run_app(&args).await.unwrap_err();
    assert!(err.to_string().contains("display") || err.to_string().contains("window"));
}
