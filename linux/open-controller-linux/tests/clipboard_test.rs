use arboard::Clipboard;
use open_controller_linux::tools::clipboard::{run_clipboard, ClipboardArgs, ClipboardMode};
use std::sync::Mutex;

static CLIPBOARD_LOCK: Mutex<()> = Mutex::new(());

fn skip_if_no_clipboard() -> Option<()> {
    Clipboard::new().ok()?;
    Some(())
}

fn make_args(mode: ClipboardMode, content: Option<&str>) -> ClipboardArgs {
    ClipboardArgs {
        mode,
        content: content.map(|s| s.to_string()),
        confirm: false,
    }
}

#[test]
fn write_and_read_clipboard() {
    let _g = CLIPBOARD_LOCK.lock().unwrap();
    if skip_if_no_clipboard().is_none() {
        return;
    }

    let write = make_args(ClipboardMode::Write, Some("open-controller-test"));
    run_clipboard(&write).unwrap();

    let read = make_args(ClipboardMode::Read, None);
    let value = run_clipboard(&read).unwrap();
    assert_eq!(value, "open-controller-test");
}

#[test]
fn clear_clipboard() {
    let _g = CLIPBOARD_LOCK.lock().unwrap();
    if skip_if_no_clipboard().is_none() {
        return;
    }

    let write = make_args(ClipboardMode::Write, Some("before"));
    run_clipboard(&write).unwrap();

    let clear = make_args(ClipboardMode::Clear, None);
    run_clipboard(&clear).unwrap();

    let read = make_args(ClipboardMode::Read, None);
    let value = run_clipboard(&read).unwrap();
    assert_eq!(value, "");
}

#[test]
fn read_without_provider_does_not_panic() {
    let _g = CLIPBOARD_LOCK.lock().unwrap();
    let read = make_args(ClipboardMode::Read, None);
    let result = run_clipboard(&read);
    if Clipboard::new().is_ok() {
        assert!(result.is_ok());
    } else {
        assert!(result.is_err());
    }
}
