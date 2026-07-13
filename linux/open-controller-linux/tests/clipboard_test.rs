use arboard::Clipboard;
use open_controller_linux::tools::clipboard::{ClipboardArgs, ClipboardMode, run_clipboard};
use std::sync::Mutex;

static CLIPBOARD_LOCK: Mutex<()> = Mutex::new(());

fn acquire_lock() -> std::sync::MutexGuard<'static, ()> {
    CLIPBOARD_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn skip_if_no_clipboard() -> Option<()> {
    Clipboard::new().ok()?;
    Some(())
}

fn make_args(mode: ClipboardMode, content: Option<&str>) -> ClipboardArgs {
    ClipboardArgs {
        mode,
        content: content.map(|s| s.to_string()),
    }
}

#[test]
fn write_and_read_clipboard() {
    let _g = acquire_lock();
    if skip_if_no_clipboard().is_none() {
        return;
    }

    let write = make_args(ClipboardMode::Write, Some("open-controller-test"));
    run_clipboard(&write, true).unwrap();

    let read = make_args(ClipboardMode::Read, None);
    let value = run_clipboard(&read, true).unwrap();
    assert_eq!(value, "open-controller-test");
}

#[test]
fn clear_clipboard() {
    let _g = acquire_lock();
    if skip_if_no_clipboard().is_none() {
        return;
    }

    let write = make_args(ClipboardMode::Write, Some("before"));
    run_clipboard(&write, true).unwrap();

    let clear = make_args(ClipboardMode::Clear, None);
    run_clipboard(&clear, true).unwrap();

    let read = make_args(ClipboardMode::Read, None);
    let value = run_clipboard(&read, true).unwrap();
    assert_eq!(value, "");
}

#[test]
fn read_without_provider_does_not_panic() {
    let _g = acquire_lock();
    let read = make_args(ClipboardMode::Read, None);
    // Provider state varies across environments; the test only verifies no panic.
    let _ = run_clipboard(&read, false);
}

#[test]
fn clipboard_requires_confirm() {
    let args = make_args(ClipboardMode::Read, None);
    let err = run_clipboard(&args, false).unwrap_err();
    assert!(err.to_string().contains("confirm"));
}
