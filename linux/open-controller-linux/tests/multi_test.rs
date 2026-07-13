use open_controller_linux::linux::detect::{Compositor, detect};
use open_controller_linux::tools::multi_edit::{MultiEditArgs, MultiEditLoc, run_multi_edit};
use open_controller_linux::tools::multi_select::{MultiSelectArgs, run_multi_select};

fn skip_if_headless() -> bool {
    detect() == Compositor::Headless
}

#[test]
fn multi_select_requires_confirm() {
    let args = MultiSelectArgs {
        locs: Some(vec![[10, 10]]),
    };
    let err = run_multi_select(&args, false).unwrap_err();
    assert!(err.to_string().contains("confirm"));
}

#[test]
fn multi_select_headless_returns_error() {
    if !skip_if_headless() {
        return;
    }
    let args = MultiSelectArgs {
        locs: Some(vec![[10, 10]]),
    };
    let err = run_multi_select(&args, true).unwrap_err();
    assert!(err.to_string().contains("no display"));
}

#[test]
fn multi_edit_requires_confirm() {
    let args = MultiEditArgs {
        locs: Some(vec![MultiEditLoc {
            x: 10,
            y: 10,
            text: "hi".to_string(),
        }]),
    };
    let err = run_multi_edit(&args, false).unwrap_err();
    assert!(err.to_string().contains("confirm"));
}

#[test]
fn multi_edit_headless_returns_error() {
    if !skip_if_headless() {
        return;
    }
    let args = MultiEditArgs {
        locs: Some(vec![MultiEditLoc {
            x: 10,
            y: 10,
            text: "hi".to_string(),
        }]),
    };
    let err = run_multi_edit(&args, true).unwrap_err();
    assert!(err.to_string().contains("no display"));
}
