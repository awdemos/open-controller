use open_controller_linux::linux::detect::{Compositor, detect};
use open_controller_linux::tools::snapshot::{SnapshotArgs, run_snapshot};

#[test]
fn snapshot_requires_confirm() {
    let args = SnapshotArgs {
        annotate: false,
        confirm: false,
    };
    let err = run_snapshot(&args).unwrap_err();
    assert!(err.to_string().contains("confirm"));
}

#[test]
fn snapshot_headless_returns_error() {
    if detect() != Compositor::Headless {
        return;
    }
    let args = SnapshotArgs {
        annotate: false,
        confirm: true,
    };
    let err = run_snapshot(&args).unwrap_err();
    assert!(err.to_string().contains("no display"));
}
