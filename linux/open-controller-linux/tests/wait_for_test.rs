use open_controller_linux::tools::wait_for::{run_wait_for, WaitForArgs, WaitForTarget};
use std::time::{Duration, Instant};

fn make_args(target: WaitForTarget, value: &str, timeout: u32) -> WaitForArgs {
    WaitForArgs {
        target,
        value: value.to_string(),
        timeout,
    }
}

#[tokio::test]
async fn wait_for_current_process_succeeds() {
    let mut child = tokio::process::Command::new("sleep")
        .arg("10")
        .spawn()
        .expect("sleep command is required for this test");
    let pid = child.id().expect("child pid available");
    let args = make_args(WaitForTarget::Process, &pid.to_string(), 5);
    let result = run_wait_for(&args).await;
    child.kill().await.ok();
    assert!(result.is_ok(), "expected process to be found: {:?}", result);
}

#[tokio::test]
async fn wait_for_missing_process_times_out() {
    let args = make_args(
        WaitForTarget::Process,
        "definitely-not-a-real-process-name-12345",
        1,
    );
    let start = Instant::now();
    let err = run_wait_for(&args).await.unwrap_err();
    assert!(err.to_string().contains("timeout"));
    assert!(start.elapsed() >= Duration::from_millis(800));
}

#[tokio::test]
async fn wait_for_window_headless_times_out() {
    if open_controller_linux::linux::detect::detect()
        != open_controller_linux::linux::detect::Compositor::Headless
    {
        return;
    }
    let args = make_args(WaitForTarget::Window, "no-such-window", 1);
    let err = run_wait_for(&args).await.unwrap_err();
    assert!(err.to_string().contains("timeout"));
}
