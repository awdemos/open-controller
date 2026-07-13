use open_controller_linux::tools::notification::{NotificationArgs, run_notification};

fn make_args() -> NotificationArgs {
    NotificationArgs {
        title: "open-controller test".to_string(),
        message: "notification test".to_string(),
        app_id: "open-controller-linux".to_string(),
    }
}

#[test]
fn notification_requires_confirm() {
    let args = make_args();
    let err = run_notification(&args, false).unwrap_err();
    assert!(err.to_string().contains("confirm"));
}

#[test]
fn notification_does_not_panic_when_confirmed() {
    let args = make_args();
    // On a system with a notification server this succeeds; in headless CI it may fail.
    let _ = run_notification(&args, true);
}
