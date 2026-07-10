use open_controller_linux::state::AppState;
use open_controller_linux::tools::shell::{has_elevated_privileges, run_shell};
use regex::Regex;

#[tokio::test]
async fn echo_works_when_allowed() {
    let allowlist = vec![Regex::new(r"^echo ").unwrap()];
    let state = AppState::new(false, allowlist);
    let out = run_shell("echo hello", 5, &state).await.unwrap();
    assert_eq!(out, "hello");
}

#[tokio::test]
async fn blocked_without_allowlist() {
    let state = AppState::default();
    let err = run_shell("echo hello", 5, &state).await.unwrap_err();
    assert!(err.to_string().contains("allowlist"));
}

#[tokio::test]
async fn elevated_privilege_blocked() {
    let allowlist = vec![Regex::new(r".*").unwrap()];
    let state = AppState::new(false, allowlist);
    let err = run_shell("sudo ls", 5, &state).await.unwrap_err();
    assert!(err.to_string().contains("elevated privileges"));
}

#[test]
fn detects_elevated_patterns() {
    assert!(has_elevated_privileges("sudo apt update"));
    assert!(has_elevated_privileges("doas ls"));
    assert!(!has_elevated_privileges("ls -la"));
}
