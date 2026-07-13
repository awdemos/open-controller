use open_controller_linux::linux::detect::{Compositor, detect};
use open_controller_linux::tools::click::{ClickArgs, run_click};
use open_controller_linux::tools::move_::{MoveArgs, run_move};
use open_controller_linux::tools::scroll::{ScrollArgs, run_scroll};
use open_controller_linux::tools::shortcut::{ShortcutArgs, run_shortcut};
use open_controller_linux::tools::type_::{TypeArgs, run_type};

fn skip_if_display() -> bool {
    detect() == Compositor::Headless
}

#[test]
fn click_requires_confirm() {
    let args = ClickArgs {
        button: 0,
        x: 10,
        y: 10,
    };
    let err = run_click(&args, false).unwrap_err();
    assert!(err.to_string().contains("confirm"));
}

#[test]
fn input_headless_returns_error() {
    if !skip_if_display() {
        return;
    }
    let args = ClickArgs {
        button: 0,
        x: 10,
        y: 10,
    };
    let err = run_click(&args, true).unwrap_err();
    assert!(err.to_string().contains("no display"));
}

#[test]
fn move_requires_confirm() {
    let args = MoveArgs {
        x: 10,
        y: 10,
        relative: false,
    };
    let err = run_move(&args, false).unwrap_err();
    assert!(err.to_string().contains("confirm"));
}

#[test]
fn scroll_requires_confirm() {
    let args = ScrollArgs {
        direction: 1,
        x: 10,
        y: 10,
        amount: None,
    };
    let err = run_scroll(&args, false).unwrap_err();
    assert!(err.to_string().contains("confirm"));
}

#[test]
fn type_requires_confirm() {
    let args = TypeArgs {
        text: "hello".to_string(),
    };
    let err = run_type(&args, false).unwrap_err();
    assert!(err.to_string().contains("confirm"));
}

#[test]
fn shortcut_requires_confirm() {
    let args = ShortcutArgs {
        keys: "ctrl+c".to_string(),
    };
    let err = run_shortcut(&args, false).unwrap_err();
    assert!(err.to_string().contains("confirm"));
}
