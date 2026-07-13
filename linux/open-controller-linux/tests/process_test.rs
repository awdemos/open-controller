use open_controller_linux::tools::process::{ProcessArgs, ProcessMode, run_process};
use sysinfo::get_current_pid;

fn make_args(mode: ProcessMode) -> ProcessArgs {
    ProcessArgs {
        mode,
        process_id: None,
        name: None,
        user: None,
        signal: None,
        force: false,
    }
}

#[test]
fn list_includes_current_process() {
    let mut args = make_args(ProcessMode::List);
    let current = get_current_pid().expect("current pid").as_u32();
    args.process_id = Some(current);
    let out = run_process(&args, false).unwrap();
    assert!(out.contains(&format!("pid={}", current)));
}

#[test]
fn list_filters_by_name() {
    let current_pid = get_current_pid().expect("current pid");
    let current_name = sysinfo::System::new_all()
        .process(current_pid)
        .map(|p| p.name().to_string_lossy().to_string())
        .unwrap_or_else(|| "process_test".to_string());

    let mut args = make_args(ProcessMode::List);
    args.name = Some(current_name.clone());
    let out = run_process(&args, false).unwrap();
    assert!(
        !out.is_empty(),
        "expected non-empty output filtering by name '{}'",
        current_name
    );
    assert!(out.contains(&format!("pid={}", current_pid.as_u32())));
}

#[test]
fn kill_requires_confirm() {
    let mut args = make_args(ProcessMode::Kill);
    args.process_id = Some(get_current_pid().expect("current pid").as_u32());
    let err = run_process(&args, false).unwrap_err();
    assert!(err.to_string().contains("confirm"));
}

#[test]
fn kill_self_rejected() {
    let mut args = make_args(ProcessMode::Kill);
    args.process_id = Some(get_current_pid().expect("current pid").as_u32());
    let err = run_process(&args, true).unwrap_err();
    assert!(err.to_string().contains("cannot kill protected"));
}

#[test]
fn kill_pid_one_rejected() {
    let mut args = make_args(ProcessMode::Kill);
    args.process_id = Some(1);
    let err = run_process(&args, true).unwrap_err();
    assert!(err.to_string().contains("cannot kill protected"));
}
