use open_controller_linux::linux::detect::{detect, Compositor};
use open_controller_linux::tools::click::{run_click, ClickArgs};
use open_controller_linux::tools::move_::{run_move, MoveArgs};
use open_controller_linux::tools::scroll::{run_scroll, ScrollArgs};
use open_controller_linux::tools::shortcut::{run_shortcut, ShortcutArgs};
use open_controller_linux::tools::type_::{run_type, TypeArgs};

fn skip_if_display() -> bool {
    detect() == Compositor::Headless
}

#[test]
fn click_requires_confirm() {
    let args = ClickArgs {
        button: 0,
        x: 10,
        y: 10,
        confirm: false,
    };
    let err = run_click(&args).unwrap_err();
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
        confirm: true,
    };
    let err = run_click(&args).unwrap_err();
    assert!(err.to_string().contains("no display"));
}

#[test]
fn move_requires_confirm() {
    let args = MoveArgs {
        x: 10,
        y: 10,
        relative: false,
        confirm: false,
    };
    let err = run_move(&args).unwrap_err();
    assert!(err.to_string().contains("confirm"));
}

#[test]
fn scroll_requires_confirm() {
    let args = ScrollArgs {
        direction: 1,
        x: 10,
        y: 10,
        amount: None,
        confirm: false,
    };
    let err = run_scroll(&args).unwrap_err();
    assert!(err.to_string().contains("confirm"));
}

#[test]
fn type_requires_confirm() {
    let args = TypeArgs {
        text: "hello".to_string(),
        confirm: false,
    };
    let err = run_type(&args).unwrap_err();
    assert!(err.to_string().contains("confirm"));
}

#[test]
fn shortcut_requires_confirm() {
    let args = ShortcutArgs {
        keys: "ctrl+c".to_string(),
        confirm: false,
    };
    let err = run_shortcut(&args).unwrap_err();
    assert!(err.to_string().contains("confirm"));
}
