use open_controller_linux::tools::notification::{NotificationArgs, run_notification};

#[test]
fn notification_does_not_panic() {
    let args = NotificationArgs {
        title: "open-controller test".to_string(),
        message: "notification test".to_string(),
        app_id: "open-controller-linux".to_string(),
    };
    // On a system with a notification server this succeeds; in headless CI it may fail.
    let _ = run_notification(&args);
}
