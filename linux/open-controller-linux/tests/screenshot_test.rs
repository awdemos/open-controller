use open_controller_linux::linux::detect::{Compositor, detect};
use open_controller_linux::tools::screenshot::{ScreenshotArgs, run_screenshot};

fn make_args() -> ScreenshotArgs {
    ScreenshotArgs {
        display: None,
        region: None,
    }
}

#[test]
fn screenshot_requires_confirm() {
    let args = make_args();
    let err = run_screenshot(&args, false).unwrap_err();
    assert!(err.to_string().contains("confirm"));
}

#[test]
fn screenshot_headless_returns_error() {
    if detect() != Compositor::Headless {
        // When running under a real display, just ensure confirm is required.
        let args = make_args();
        let result = run_screenshot(&args, false);
        assert!(result.is_err());
        return;
    }
    let args = make_args();
    let err = run_screenshot(&args, true).unwrap_err();
    assert!(err.to_string().contains("no display"));
}
