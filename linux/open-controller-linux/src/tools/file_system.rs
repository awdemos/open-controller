use rmcp::schemars;
use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum FileSystemMode {
    Read,
    Write,
    Copy,
    Move,
    Delete,
    List,
    Search,
    Info,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FileSystemArgs {
    pub mode: FileSystemMode,
    pub path: String,
    #[serde(default)]
    pub destination: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default = "default_false")]
    pub recursive: bool,
    #[serde(default = "default_false")]
    pub append: bool,
    #[serde(default = "default_false")]
    pub overwrite: bool,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default = "default_encoding")]
    pub encoding: String,
    #[serde(default = "default_false")]
    pub show_hidden: bool,
    #[serde(default = "default_false")]
    pub confirm: bool,
}

fn default_false() -> bool {
    false
}

fn default_encoding() -> String {
    "utf-8".to_string()
}

const DESTRUCTIVE_MODES: &[FileSystemMode] = &[
    FileSystemMode::Delete,
    FileSystemMode::Move,
    FileSystemMode::Copy,
    FileSystemMode::Write,
];

pub fn requires_confirmation(mode: &FileSystemMode) -> bool {
    DESTRUCTIVE_MODES.contains(mode)
}

fn resolve_path(input: &str) -> anyhow::Result<PathBuf> {
    let path = Path::new(input);
    Ok(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()))
}

pub fn run_file_system(args: &FileSystemArgs, confirm_destructive: bool) -> anyhow::Result<String> {
    if requires_confirmation(&args.mode) && !confirm_destructive && !args.confirm {
        anyhow::bail!(
            "destructive file_system operation requires --confirm-destructive or per-call confirm=true"
        );
    }

    let path = resolve_path(&args.path)?;

    match args.mode {
        FileSystemMode::Read => read_file(&path, args.offset, args.limit, &args.encoding),
        FileSystemMode::Write => write_file(&path, args.content.as_deref().unwrap_or(""), args.append),
        FileSystemMode::Copy => {
            let dest = resolve_path(args.destination.as_deref().ok_or_else(|| anyhow::anyhow!("destination required for copy"))?)?;
            if dest.exists() && !args.overwrite {
                anyhow::bail!("destination already exists; set overwrite=true to replace");
            }
            if path.is_dir() {
                anyhow::bail!("copying directories is not supported");
            }
            fs::copy(&path, &dest)?;
            Ok(format!("copied {} to {}", path.display(), dest.display()))
        }
        FileSystemMode::Move => {
            let dest = resolve_path(args.destination.as_deref().ok_or_else(|| anyhow::anyhow!("destination required for move"))?)?;
            if dest.exists() && !args.overwrite {
                anyhow::bail!("destination already exists; set overwrite=true to replace");
            }
            fs::rename(&path, &dest)?;
            Ok(format!("moved {} to {}", path.display(), dest.display()))
        }
        FileSystemMode::Delete => {
            if path.is_dir() {
                if args.recursive {
                    fs::remove_dir_all(&path)?;
                } else {
                    fs::remove_dir(&path)?;
                }
            } else {
                fs::remove_file(&path)?;
            }
            Ok(format!("deleted {}", path.display()))
        }
        FileSystemMode::List => list_directory(&path, args.show_hidden, args.recursive, args.limit),
        FileSystemMode::Search => search_files(&path, args.pattern.as_deref().unwrap_or(""), args.recursive, args.show_hidden, args.limit),
        FileSystemMode::Info => file_info(&path),
    }
}

fn read_file(path: &Path, offset: Option<usize>, limit: Option<usize>, encoding: &str) -> anyhow::Result<String> {
    if !path.is_file() {
        anyhow::bail!("path is not a file: {}", path.display());
    }
    if encoding.to_lowercase() != "utf-8" && encoding.to_lowercase() != "utf8" {
        anyhow::bail!("unsupported encoding: {}", encoding);
    }
    let bytes = fs::read(path)?;
    let start = offset.unwrap_or(0).min(bytes.len());
    let end = limit.map(|l| start + l).unwrap_or(bytes.len()).min(bytes.len());
    let slice = &bytes[start..end];
    Ok(String::from_utf8_lossy(slice).to_string())
}

fn write_file(path: &Path, content: &str, append: bool) -> anyhow::Result<String> {
    let mut file = if append {
        fs::OpenOptions::new().create(true).append(true).open(path)?
    } else {
        fs::File::create(path)?
    };
    file.write_all(content.as_bytes())?;
    Ok(format!("wrote {} bytes to {}", content.len(), path.display()))
}

fn list_directory(path: &Path, show_hidden: bool, recursive: bool, limit: Option<usize>) -> anyhow::Result<String> {
    let mut entries: Vec<String> = Vec::new();
    let walker = if recursive {
        WalkDir::new(path)
    } else {
        WalkDir::new(path).max_depth(1)
    };
    for (i, entry) in walker.into_iter().enumerate() {
        if let Some(l) = limit
            && i >= l
        {
            break;
        }
        let entry = entry?;
        let name = entry.file_name().to_string_lossy();
        if !show_hidden && name.starts_with('.') {
            continue;
        }
        let meta = entry.metadata()?;
        let kind = if meta.is_dir() { "dir" } else { "file" };
        entries.push(format!("{} {} {}", kind, entry.path().display(), meta.len()));
    }
    Ok(entries.join("\n"))
}

fn search_files(path: &Path, pattern: &str, recursive: bool, show_hidden: bool, limit: Option<usize>) -> anyhow::Result<String> {
    let mut results: Vec<String> = Vec::new();
    let walker = if recursive {
        WalkDir::new(path)
    } else {
        WalkDir::new(path).max_depth(1)
    };
    for (i, entry) in walker.into_iter().enumerate() {
        if let Some(l) = limit
            && i >= l
        {
            break;
        }
        let entry = entry?;
        let name = entry.file_name().to_string_lossy();
        if !show_hidden && name.starts_with('.') {
            continue;
        }
        if name.contains(pattern) {
            results.push(entry.path().display().to_string());
        }
    }
    Ok(results.join("\n"))
}

fn file_info(path: &Path) -> anyhow::Result<String> {
    let meta = fs::metadata(path)?;
    let kind = if meta.is_dir() { "dir" } else { "file" };
    Ok(format!(
        "{} {} {} bytes",
        kind,
        path.display(),
        meta.len()
    ))
}
