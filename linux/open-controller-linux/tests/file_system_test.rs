use open_controller_linux::tools::file_system::{FileSystemArgs, FileSystemMode, run_file_system};
use tempfile::TempDir;

fn make_args(mode: FileSystemMode, path: String) -> FileSystemArgs {
    FileSystemArgs {
        mode,
        path,
        destination: None,
        content: None,
        pattern: None,
        recursive: false,
        append: false,
        overwrite: false,
        offset: None,
        limit: None,
        encoding: "utf-8".to_string(),
        show_hidden: false,
    }
}

#[test]
fn read_write_and_delete_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("hello.txt");

    let mut write_args = make_args(FileSystemMode::Write, file.to_string_lossy().to_string());
    write_args.content = Some("hello world".to_string());
    let out = run_file_system(&write_args, true, dir.path()).unwrap();
    assert!(out.contains("wrote 11 bytes"));

    let read_args = make_args(FileSystemMode::Read, file.to_string_lossy().to_string());
    let content = run_file_system(&read_args, false, dir.path()).unwrap();
    assert_eq!(content, "hello world");

    let delete_args = make_args(FileSystemMode::Delete, file.to_string_lossy().to_string());
    let out = run_file_system(&delete_args, true, dir.path()).unwrap();
    assert!(out.contains("deleted"));
    assert!(!file.exists());
}

#[test]
fn copy_requires_confirm() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("src.txt");
    let dst = dir.path().join("dst.txt");
    std::fs::write(&src, "data").unwrap();

    let mut args = make_args(FileSystemMode::Copy, src.to_string_lossy().to_string());
    args.destination = Some(dst.to_string_lossy().to_string());
    let err = run_file_system(&args, false, dir.path()).unwrap_err();
    assert!(err.to_string().contains("confirm"));

    let out = run_file_system(&args, true, dir.path()).unwrap();
    assert!(out.contains("copied"));
    assert_eq!(std::fs::read_to_string(&dst).unwrap(), "data");
}

#[test]
fn list_and_info_directory() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("a.txt"), "a").unwrap();
    std::fs::create_dir(dir.path().join("sub")).unwrap();

    let list_args = make_args(
        FileSystemMode::List,
        dir.path().to_string_lossy().to_string(),
    );
    let out = run_file_system(&list_args, false, dir.path()).unwrap();
    assert!(out.contains("a.txt"));

    let info_args = make_args(
        FileSystemMode::Info,
        dir.path().join("a.txt").to_string_lossy().to_string(),
    );
    let out = run_file_system(&info_args, false, dir.path()).unwrap();
    assert!(out.contains("bytes"));
}

#[test]
fn path_traversal_is_blocked() {
    let dir = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    let secret = outside.path().join("secret.txt");
    std::fs::write(&secret, "secret").unwrap();

    let args = make_args(
        FileSystemMode::Read,
        format!(
            "{}/../{}",
            dir.path().display(),
            secret.file_name().unwrap().to_string_lossy()
        ),
    );
    let err = run_file_system(&args, false, dir.path()).unwrap_err();
    assert!(err.to_string().contains("base directory"));
}

#[test]
fn absolute_path_outside_base_is_blocked() {
    let dir = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    let secret = outside.path().join("secret.txt");
    std::fs::write(&secret, "secret").unwrap();

    let args = make_args(FileSystemMode::Read, secret.to_string_lossy().to_string());
    let err = run_file_system(&args, false, dir.path()).unwrap_err();
    assert!(err.to_string().contains("base directory"));
}

#[test]
fn write_creates_parent_directories_inside_base() {
    let dir = TempDir::new().unwrap();
    let nested = dir.path().join("a/b/c.txt");

    let mut args = make_args(FileSystemMode::Write, nested.to_string_lossy().to_string());
    args.content = Some("nested".to_string());
    let out = run_file_system(&args, true, dir.path()).unwrap();
    assert!(out.contains("wrote 6 bytes"));
    assert_eq!(std::fs::read_to_string(&nested).unwrap(), "nested");
}
