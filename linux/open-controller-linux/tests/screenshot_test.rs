use open_controller_linux::linux::detect::{detect, Compositor};
use open_controller_linux::tools::screenshot::{run_screenshot, ScreenshotArgs};

fn make_args(confirm: bool) -> ScreenshotArgs {
    ScreenshotArgs {
        display: None,
        region: None,
        confirm,
    }
}

#[test]
fn screenshot_requires_confirm() {
    let args = make_args(false);
    let err = run_screenshot(&args).unwrap_err();
    assert!(err.to_string().contains("confirm"));
}

#[test]
fn screenshot_headless_returns_error() {
    if detect() != Compositor::Headless {
        // When running under a real display, just ensure confirm is required.
        let args = make_args(false);
        let result = run_screenshot(&args);
        assert!(result.is_err());
        return;
    }
    let args = make_args(true);
    let err = run_screenshot(&args).unwrap_err();
    assert!(err.to_string().contains("no display"));
}
