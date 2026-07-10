# open-controller-linux Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust MCP server binary named `open-controller-linux` that exposes Linux equivalents of the Windows `open-controller` MCP tools over stdio (and optional SSE), then wire it into the existing TypeScript plugin for Linux hosts.

**Architecture:** A single Rust binary under `linux/open-controller-linux/` uses `rmcp` (`#[tool_router]` + `#[tool_handler]`) to expose tools over stdio. Tools live in `src/tools/`. Platform-specific desktop work is delegated to `src/linux/`, which detects X11 vs Wayland and chooses `x11rb`/`enigo`/`xcap` for X11 and `ashpd`/portals for native Wayland. The existing TypeScript plugin is extended to register the Rust binary on Linux while keeping the Windows Python path unchanged.

**Tech Stack:**

| Concern | Crate | Justification |
|---|---|---|
| MCP server / stdio / SSE | `rmcp` | Official Rust MCP SDK; `#[tool]` auto-generates JSON schemas |
| Async runtime | `tokio` | Required by `rmcp`; process/network/timer support |
| Serialization / CLI | `serde`, `serde_json`, `clap` | Standard derives and CLI parsing |
| Shell execution | `tokio::process` | Async process spawning |
| File system | `std` + `walkdir` | Safe recursive traversal |
| Clipboard | `arboard` | Cross-platform; enable `wayland-data-control` |
| Notifications | `notify-rust` | D-Bus notifications on X11 and Wayland |
| Process info / kill | `sysinfo` | Process enumeration, UIDs, signals |
| Screenshots (X11) | `xcap`/`x11rb` image reading | Linux X11 capture |
| Input simulation (X11) | `enigo` | Mouse/keyboard injection |
| Window management (X11) | `x11rb` | Window enumeration, raising, resizing |
| Wayland portals | `ashpd` + `zbus` | Screen capture, remote desktop input |
| Web scraping | `reqwest` + `scraper` | HTTP client + HTML parsing |
| Error handling | `anyhow` + `thiserror` | Ergonomic errors |
| Tracing | `tracing` + `tracing-subscriber` | Logging to stderr |

---

## 1. Linux Tool Mapping

| Windows Tool | Linux Tool | Status | Notes |
|---|---|---|---|
| `PowerShell` | `shell` | Reimplemented | `/bin/sh -c`; privilege block + regex allowlist |
| `FileSystem` | `file_system` | Reimplemented | 8 modes; destructive ops require confirmation |
| `Process` | `process` | Reimplemented | list/kill restricted to current UID |
| `Clipboard` | `clipboard` | Reimplemented | `arboard` get/set/clear |
| `Notification` | `notification` | Reimplemented | `notify-rust` |
| `Scrape` | `scrape` | Reimplemented | `reqwest` + `scraper` |
| `Screenshot` | `screenshot` | Reimplemented | X11 via `xcap`/`x11rb`; Wayland via portal |
| `App` | `app` | Partial | `launch` everywhere; `switch`/`resize` require X11 |
| `Shortcut` | `shortcut` | Partial | X11 via `enigo`; Wayland via portal/XWayland |
| `Click` | `click` | Partial | Same as above |
| `Type` | `type` | Partial | Same as above |
| `Scroll` | `scroll` | Partial | Same as above |
| `Move` | `move` | Partial | Same as above |
| `Wait` | `wait` | Reimplemented | Async sleep |
| `WaitFor` | `wait_for` | Partial | Wait for process/window/clipboard text |
| `Snapshot` | `snapshot` | Partial | X11 window tree + annotated screenshot |
| `MultiSelect` | `multi_select` | Partial | Coordinates on X11 |
| `MultiEdit` | `multi_edit` | Partial | Coordinates on X11 |
| `Registry` | — | Dropped | Windows-only; not registered |

---

## 2. Project Layout

```
linux/open-controller-linux/
├── Cargo.toml
├── Cargo.lock               # commit for a binary crate
├── .gitignore              # ignore target/
├── src/
│   ├── main.rs              # CLI + stdio/SSE bootstrap
│   ├── lib.rs               # public modules for tests
│   ├── state.rs             # AppState (config, allowlists)
│   ├── mcp/
│   │   ├── mod.rs
│   │   └── server.rs        # ToolRouter + server info
│   ├── tools/
│   │   ├── mod.rs
│   │   ├── shell.rs
│   │   ├── file_system.rs
│   │   ├── process.rs
│   │   ├── clipboard.rs
│   │   ├── screenshot.rs
│   │   ├── app.rs
│   │   ├── shortcut.rs
│   │   ├── click.rs
│   │   ├── type_.rs
│   │   ├── scroll.rs
│   │   ├── move_.rs
│   │   ├── wait.rs
│   │   ├── wait_for.rs
│   │   ├── snapshot.rs
│   │   ├── multi_select.rs
│   │   ├── multi_edit.rs
│   │   ├── notification.rs
│   │   └── scrape.rs
│   └── linux/
│       ├── mod.rs           # Compositor enum + re-exports
│       ├── detect.rs        # X11/Wayland/headless detection
│       ├── display.rs       # display enumeration
│       ├── input.rs         # input abstraction
│       ├── screen.rs        # screenshot abstraction
│       ├── window.rs        # window enumeration/raise/resize
│       ├── x11.rs           # X11 helpers
│       └── wayland.rs       # portal helpers
└── tests/
    ├── mcp_smoke.rs
    ├── shell_test.rs
    ├── file_system_test.rs
    ├── process_test.rs
    ├── clipboard_test.rs
    ├── screenshot_test.rs
    ├── input_test.rs
    ├── app_test.rs
    ├── wait_for_test.rs
    └── scrape_test.rs
```

Modified existing files:

- `src/index.ts` — register Rust binary on Linux, rename tools to `pc-exec` / `pc-screenshot`.
- `package.json` — add `linux` to `os`, add build script if needed.

---

## 3. Security Model

1. **Shell execution**
   - Block `sudo`, `su -`, `doas`, `pkexec`, `visudo`, `chmod +s`, `setuid`.
   - Require a non-empty regex allowlist configured at startup; reject everything else.
   - Default timeout; kill on expiry.

2. **File system**
   - Destructive modes (write, copy, move, delete) require either `--confirm-destructive` startup flag or per-call `confirm=true`.
   - Paths are canonicalized; symlink traversal follows the OS.

3. **Process kill**
   - Only kill processes whose UID matches the current UID.
   - Reject PID 0, PID 1, and the server's own PID.
   - Require confirmation like file-system destructive ops.

4. **UI input / screenshots**
   - Tools accept an explicit `confirm=true` argument for every input action.
   - On Wayland, portal dialogs prompt the user per session; headless environments degrade gracefully with a clear error.
   - No background capture; screenshots only taken when a tool is called.

5. **Notifications / scrape**
   - Network requests limited to `reqwest`; no local network discovery.

---

## 4. MCP Tool Schemas

All parameter structs derive `serde::Deserialize` and `schemars::JsonSchema`. Tools are registered via `#[tool(name = "...", description = "...")]` on methods of `ControllerServer` in `src/mcp/server.rs`.

Common helpers:

```rust
fn default_false() -> bool { false }
fn default_timeout() -> u32 { 30 }
fn default_encoding() -> String { "utf-8".to_string() }
```

### 4.1 `shell`

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ShellArgs {
    pub command: String,
    #[serde(default = "default_timeout")]
    pub timeout: u32,
}
```

```json
{
  "type": "object",
  "properties": {
    "command": { "type": "string" },
    "timeout": { "type": "integer", "default": 30 }
  },
  "required": ["command"]
}
```

### 4.2 `file_system`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum FileSystemMode { Read, Write, Copy, Move, Delete, List, Search, Info }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FileSystemArgs {
    pub mode: FileSystemMode,
    pub path: String,
    #[serde(default)] pub destination: Option<String>,
    #[serde(default)] pub content: Option<String>,
    #[serde(default)] pub pattern: Option<String>,
    #[serde(default = "default_false")] pub recursive: bool,
    #[serde(default = "default_false")] pub append: bool,
    #[serde(default = "default_false")] pub overwrite: bool,
    #[serde(default)] pub offset: Option<usize>,
    #[serde(default)] pub limit: Option<usize>,
    #[serde(default = "default_encoding")] pub encoding: String,
    #[serde(default = "default_false")] pub show_hidden: bool,
    #[serde(default = "default_false")] pub confirm: bool,
}
```

### 4.3 `process`

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ProcessMode { List, Kill }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProcessArgs {
    pub mode: ProcessMode,
    #[serde(default)] pub process_id: Option<u32>,
    #[serde(default)] pub name: Option<String>,
    #[serde(default)] pub signal: Option<String>,
    #[serde(default = "default_false")] pub force: bool,
    #[serde(default = "default_false")] pub confirm: bool,
}
```

### 4.4 `clipboard`

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardMode { Read, Write, Clear }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ClipboardArgs {
    pub mode: ClipboardMode,
    #[serde(default)] pub content: Option<String>,
    #[serde(default = "default_false")] pub confirm: bool,
}
```

### 4.5 `screenshot`

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScreenshotArgs {
    #[serde(default)] pub display: Option<String>,
    #[serde(default)] pub region: Option<String>,
    #[serde(default = "default_false")] pub confirm: bool,
}
```

### 4.6 `shortcut`

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ShortcutArgs {
    pub keys: String,
    #[serde(default = "default_false")] pub confirm: bool,
}
```

### 4.7 `click`

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ClickArgs {
    pub button: i32,
    pub x: i32,
    pub y: i32,
    #[serde(default = "default_false")] pub confirm: bool,
}
```

### 4.8 `type`

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TypeArgs {
    pub text: String,
    #[serde(default = "default_false")] pub confirm: bool,
}
```

### 4.9 `scroll`

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScrollArgs {
    pub direction: i32,
    pub x: i32,
    pub y: i32,
    #[serde(default)] pub amount: Option<i32>,
    #[serde(default = "default_false")] pub confirm: bool,
}
```

### 4.10 `move`

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MoveArgs {
    pub x: i32,
    pub y: i32,
    #[serde(default = "default_false")] pub relative: bool,
    #[serde(default = "default_false")] pub confirm: bool,
}
```

### 4.11 `app`

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AppArgs {
    pub name: String,
    #[serde(default = "default_false")] pub launch: bool,
    #[serde(default = "default_false")] pub focus: bool,
    #[serde(default = "default_false")] pub close: bool,
    #[serde(default = "default_false")] pub confirm: bool,
}
```

### 4.12 `wait`

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WaitArgs {
    pub duration: u64,
}
```

### 4.13 `wait_for`

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum WaitForTarget { Process, Window, Clipboard }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WaitForArgs {
    pub target: WaitForTarget,
    pub value: String,
    #[serde(default = "default_timeout")] pub timeout: u32,
}
```

### 4.14 `snapshot`

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SnapshotArgs {
    #[serde(default = "default_false")] pub annotate: bool,
    #[serde(default = "default_false")] pub confirm: bool,
}
```

### 4.15 `multi_select`

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MultiSelectArgs {
    #[serde(default)] pub locs: Option<Vec<[i32; 2]>>,
    #[serde(default = "default_false")] pub confirm: bool,
}
```

### 4.16 `multi_edit`

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MultiEditLoc {
    pub x: i32,
    pub y: i32,
    pub text: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MultiEditArgs {
    #[serde(default)] pub locs: Option<Vec<MultiEditLoc>>,
    #[serde(default = "default_false")] pub confirm: bool,
}
```

### 4.17 `notification`

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NotificationArgs {
    pub title: String,
    pub message: String,
    #[serde(default = "default_app_id")] pub app_id: String,
}

fn default_app_id() -> String { "open-controller-linux".to_string() }
```

### 4.18 `scrape`

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScrapeArgs {
    pub url: String,
    #[serde(default)] pub query: Option<String>,
}
```

---

## 5. Phased Implementation Tasks

### Phase 0 — Scaffold (already done)

- [x] **Task 0.1:** Create `linux/open-controller-linux/Cargo.toml` with dependencies and `src/main.rs` placeholder.
- [x] **Task 0.2:** Add `src/lib.rs`, `src/state.rs`, `src/linux/detect.rs` with `Compositor` detection and tests.
- [x] **Task 0.3:** Implement `src/mcp/server.rs` with `#[tool_router]`, an `echo` tool, and stdio bootstrap.
- [x] **Task 0.4:** Add `tests/mcp_smoke.rs` that builds the binary and runs initialize → initialized → tools/list over NDJSON.

Verification:

```bash
cd linux/open-controller-linux
cargo test
```

Expected: all tests pass, including `mcp_smoke::stdio_initialize_and_list_tools`.

### Phase 1 — Core Platform Tools (in progress)

#### Task 1.1: `shell`

**Files:**
- Create: `linux/open-controller-linux/src/tools/shell.rs`
- Modify: `linux/open-controller-linux/src/tools/mod.rs`
- Modify: `linux/open-controller-linux/src/mcp/server.rs`
- Create: `linux/open-controller-linux/tests/shell_test.rs`

- [x] **Step 1: Write the failing tests**

```rust
use open_controller_linux::state::AppState;
use open_controller_linux::tools::shell::{has_elevated_privileges, run_shell};
use regex::Regex;

#[tokio::test]
async fn echo_works_when_allowed() {
    let state = AppState::new(false, vec![Regex::new(r"^echo ").unwrap()]);
    let out = run_shell("echo hello", 5, &state).await.unwrap();
    assert_eq!(out, "hello");
}

#[tokio::test]
async fn blocked_without_allowlist() {
    let state = AppState::default();
    let err = run_shell("echo hello", 5, &state).await.unwrap_err();
    assert!(err.to_string().contains("allowlist"));
}

#[tokio::test]
async fn elevated_privilege_blocked() {
    let state = AppState::new(false, vec![Regex::new(r".*").unwrap()]);
    let err = run_shell("sudo ls", 5, &state).await.unwrap_err();
    assert!(err.to_string().contains("elevated privileges"));
}
```

- [x] **Step 2: Run tests to verify they fail**

```bash
cargo test shell_test
```

Expected: compile errors because `tools::shell` does not exist.

- [x] **Step 3: Implement `shell.rs`**

```rust
use regex::Regex;
use rmcp::schemars;
use serde::Deserialize;
use std::time::Duration;

use crate::state::AppState;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ShellArgs {
    pub command: String,
    #[serde(default = "default_timeout")]
    pub timeout: u32,
}

fn default_timeout() -> u32 { 30 }

const FORBIDDEN_PATTERNS: &[&str] = &[
    "sudo", "su -", "doas", "pkexec", "visudo", "chmod +s", "setuid",
];

pub fn check_shell_allowed(command: &str, allowlist: &[Regex]) -> bool {
    !allowlist.is_empty() && allowlist.iter().any(|r| r.is_match(command))
}

pub fn has_elevated_privileges(command: &str) -> bool {
    let lower = command.to_lowercase();
    FORBIDDEN_PATTERNS.iter().any(|p| lower.contains(p))
}

pub async fn run_shell(
    command: &str,
    timeout_secs: u32,
    state: &AppState,
) -> anyhow::Result<String> {
    if has_elevated_privileges(command) {
        anyhow::bail!("elevated privileges are not allowed");
    }
    if !check_shell_allowed(command, &state.shell_allowlist) {
        anyhow::bail!("command is not allowed by the configured shell allowlist");
    }
    let output = tokio::time::timeout(
        Duration::from_secs(timeout_secs as u64),
        tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output(),
    )
    .await
    .map_err(|_| anyhow::anyhow!("command timed out after {} seconds", timeout_secs))??;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut result = String::new();
    if !stdout.is_empty() { result.push_str(&stdout); }
    if !stderr.is_empty() {
        if !result.is_empty() { result.push('\n'); }
        result.push_str("stderr: ");
        result.push_str(&stderr);
    }
    if !output.status.success() {
        anyhow::bail!("command exited with status {}: {}", output.status, result.trim());
    }
    Ok(result.trim().to_string())
}
```

- [x] **Step 4: Wire into server and tools module**

In `src/tools/mod.rs`:
```rust
pub mod file_system;
pub mod process;
pub mod shell;
```

In `src/mcp/server.rs` add an async `shell` tool method that calls `run_shell`.

- [x] **Step 5: Run tests**

```bash
cargo test shell_test
```

Expected: 4 tests pass.

- [x] **Step 6: Commit**

```bash
git add linux/open-controller-linux/src linux/open-controller-linux/tests
git commit -m "feat(linux): add shell tool with privilege block and allowlist"
```

#### Task 1.2: `file_system`

**Files:**
- Create: `linux/open-controller-linux/src/tools/file_system.rs`
- Modify: `src/tools/mod.rs`, `src/mcp/server.rs`
- Create: `tests/file_system_test.rs`

Implementation supports `Read, Write, Copy, Move, Delete, List, Search, Info`. Destructive modes require confirmation. Use `walkdir` for recursive list/search.

Tests: read/write/delete roundtrip, copy confirmation gate, list/info directory.

Commit: `feat(linux): add file_system tool with confirmation guards`

#### Task 1.3: `process`

**Files:**
- Create: `linux/open-controller-linux/src/tools/process.rs`
- Modify: `src/tools/mod.rs`, `src/mcp/server.rs`
- Create: `tests/process_test.rs`

Implementation:

```rust
use sysinfo::{get_current_pid, Pid, ProcessRefreshKind, ProcessesToUpdate, Signal, System};

pub fn current_uid(sys: &System) -> Option<u32> {
    let pid = get_current_pid().ok()?;
    sys.process(pid).and_then(|p| p.user_id()).map(|u| **u as u32)
}

pub fn run_process(args: &ProcessArgs, confirm_destructive: bool) -> anyhow::Result<String> {
    let mut sys = System::new_all();
    match args.mode {
        ProcessMode::List => list_processes(&sys, args.name.as_deref()),
        ProcessMode::Kill => kill_process(&mut sys, args, confirm_destructive),
    }
}
```

Kill logic:

```rust
fn kill_process(sys: &mut System, args: &ProcessArgs, confirm_destructive: bool) -> anyhow::Result<String> {
    if !confirm_destructive && !args.confirm {
        anyhow::bail!("process kill requires --confirm-destructive or confirm=true");
    }
    let pid = args.process_id.ok_or_else(|| anyhow::anyhow!("process_id is required for kill"))?;
    let own = get_current_pid()?;
    let own_u32: u32 = own.into();
    if pid == 0 || pid == 1 || pid == own_u32 {
        anyhow::bail!("cannot kill protected process {}: PID 0, 1, or self", pid);
    }
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
        true,
        ProcessRefreshKind::nothing(),
    );
    let proc = sys.process(Pid::from_u32(pid)).ok_or_else(|| anyhow::anyhow!("process {} not found", pid))?;
    let current_uid = current_uid(sys).ok_or_else(|| anyhow::anyhow!("unable to determine current user"))?;
    if let Some(uid) = proc.user_id() {
        if **uid as u32 != current_uid {
            anyhow::bail!("process {} is not owned by current user", pid);
        }
    }
    let signal = if args.force {
        Signal::Kill
    } else {
        args.signal.as_deref().and_then(parse_signal).unwrap_or(Signal::Terminate)
    };
    match proc.kill_with(signal) {
        Some(true) => Ok(format!("sent {:?} to process {}", signal, pid)),
        Some(false) => anyhow::bail!("failed to send {:?} to process {}", signal, pid),
        None => anyhow::bail!("signal {:?} is not supported on this platform", signal),
    }
}
```

Tests: list includes current process, kill requires confirm, self-kill rejected, PID 1 rejected.

Commit: `feat(linux): add process tool with UID-restricted kill`

#### Task 1.4: `clipboard`

**Files:**
- Create: `src/tools/clipboard.rs`
- Modify: `src/tools/mod.rs`, `src/mcp/server.rs`
- Create: `tests/clipboard_test.rs`

Implementation uses `arboard::Clipboard`. Because the clipboard is a global singleton, tests must run serially. Add a test helper or use a mutex.

```rust
use arboard::Clipboard;

pub fn run_clipboard(args: &ClipboardArgs) -> anyhow::Result<String> {
    let mut cb = Clipboard::new()?;
    match args.mode {
        ClipboardMode::Read => Ok(cb.get_text()?),
        ClipboardMode::Write => {
            let text = args.content.as_deref().unwrap_or("");
            cb.set_text(text.to_string())?;
            Ok(format!("clipboard set to {} bytes", text.len()))
        }
        ClipboardMode::Clear => {
            cb.set_text(String::new())?;
            Ok("clipboard cleared".to_string())
        }
    }
}
```

Tests (serial):

```rust
use open_controller_linux::tools::clipboard::{run_clipboard, ClipboardArgs, ClipboardMode};
use std::sync::Mutex;

static CLIPBOARD_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn write_and_read_clipboard() {
    let _g = CLIPBOARD_LOCK.lock().unwrap();
    let write = ClipboardArgs { mode: ClipboardMode::Write, content: Some("test-123".to_string()), confirm: false };
    run_clipboard(&write).unwrap();
    let read = ClipboardArgs { mode: ClipboardMode::Read, content: None, confirm: false };
    assert_eq!(run_clipboard(&read).unwrap(), "test-123");
}
```

In headless CI the test may fail if no clipboard provider exists; skip if `run_clipboard` returns a connection error. Use `#[test]` with early return on `Err` containing `clipboard` provider.

Commit: `feat(linux): add clipboard tool with serial tests`

#### Task 1.5: `screenshot`

**Files:**
- Create: `src/linux/screen.rs`, `src/linux/x11.rs`, `src/linux/wayland.rs`
- Create: `src/tools/screenshot.rs`
- Modify: `src/tools/mod.rs`, `src/mcp/server.rs`
- Create: `tests/screenshot_test.rs`

X11 implementation:

- Use `x11rb` to connect to the display.
- Query the default screen root window.
- Get geometry / image via `get_image` to capture the root window or a region.
- Convert raw X image to a PNG using `image` crate and base64-encode.

Wayland implementation:

- Use `ashpd::desktop::screenshot::Screenshot` to request a portal screenshot.
- This returns a file path; read it, encode to PNG base64, return.

Headless: return a clear error.

Because capturing is environment-dependent, the test should verify the tool is registered and returns either an image or a graceful error. In `tests/screenshot_test.rs`, call the server via MCP or call `run_screenshot` directly and assert it does not panic; on headless CI expect an error containing "no display".

Commit: `feat(linux): add screenshot tool with X11 and portal backends`

### Phase 2 — UI Automation

#### Task 2.1: Platform input abstraction

**Files:**
- Create: `src/linux/input.rs`
- Create: `src/linux/window.rs`

Implement a small backend trait:

```rust
pub enum Backend {
    X11,
    Wayland,
    Headless,
}

pub trait InputBackend {
    fn move_mouse(&mut self, x: i32, y: i32, relative: bool) -> anyhow::Result<()>;
    fn click(&mut self, button: i32, x: i32, y: i32) -> anyhow::Result<()>;
    fn scroll(&mut self, direction: i32, x: i32, y: i32, amount: i32) -> anyhow::Result<()>;
    fn type_text(&mut self, text: &str) -> anyhow::Result<()>;
    fn shortcut(&mut self, keys: &str) -> anyhow::Result<()>;
}
```

X11 implementation uses `enigo` (ensure `x11rb` connection for window focus if needed). Wayland uses `ashpd::desktop::remote_desktop` for input when available; otherwise returns a clear error instructing the user to run under XWayland. Headless returns an error.

Tests for input abstraction can be unit tests that verify key parsing / button mapping without a display.

Commit: `feat(linux): add input backend abstraction`

#### Task 2.2: `wait`

**Files:**
- Create: `src/tools/wait.rs`
- Modify: `src/tools/mod.rs`, `src/mcp/server.rs`

Implementation is a simple async sleep.

```rust
pub async fn run_wait(duration_secs: u64) -> String {
    tokio::time::sleep(Duration::from_secs(duration_secs)).await;
    format!("waited {} seconds", duration_secs)
}
```

Test with a short duration and measure elapsed time.

Commit: `feat(linux): add wait tool`

#### Task 2.3: Input tools (`click`, `type`, `scroll`, `move`, `shortcut`)

**Files:**
- Create: `src/tools/click.rs`, `src/tools/type_.rs`, `src/tools/scroll.rs`, `src/tools/move_.rs`, `src/tools/shortcut.rs`
- Modify: `src/tools/mod.rs`, `src/mcp/server.rs`
- Create: `tests/input_test.rs`

Each tool constructs the appropriate backend via `linux::input::InputBackend::new()` and calls the corresponding method. All require `confirm=true`.

Tests: without a display, expect a graceful error. With a display, verify no panic. Do not assert actual pointer position unless running in a controlled environment.

Commit each tool separately or one commit for all five: `feat(linux): add click/type/scroll/move/shortcut tools`.

#### Task 2.4: `app` and window listing

**Files:**
- Create: `src/linux/window.rs` (if not done in 2.1)
- Create: `src/tools/app.rs`
- Modify: `src/tools/mod.rs`, `src/mcp/server.rs`
- Create: `tests/app_test.rs`

`app` implementation:

- `launch`: use `tokio::process::Command` to run the command/executable. If `name` is a `.desktop` file, parse with `freedesktop_entry_parser` or just try `gtk-launch`/`xdg-open`. Start with direct command execution.
- `focus`: on X11, use `x11rb` to find a window by class/title and raise it via `configure_window` + `set_input_focus` + `send_event` for `_NET_ACTIVE_WINDOW`.
- `close`: send `WM_DELETE_WINDOW` or `kill_client` on X11; on Wayland, limited to process kill if we can map the window to a PID.

Tests: `launch` can run `echo` and verify the child exits; focus/close require a display and can be skipped in headless.

Commit: `feat(linux): add app tool with launch/focus/close`

#### Task 2.5: `wait_for`

**Files:**
- Create: `src/tools/wait_for.rs`
- Modify: `src/tools/mod.rs`, `src/mcp/server.rs`
- Create: `tests/wait_for_test.rs`

Targets:

- `Process`: poll `sysinfo` for a PID or process name to appear/disappear.
- `Window`: poll X11 window list for a title/class match.
- `Clipboard`: poll clipboard text for a substring.

Implementation:

```rust
pub async fn run_wait_for(args: &WaitForArgs) -> anyhow::Result<String> {
    let deadline = Instant::now() + Duration::from_secs(args.timeout as u64);
    while Instant::now() < deadline {
        if target_met(args).await? {
            return Ok(format!("target '{}' met", args.value));
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    anyhow::bail!("timeout waiting for '{}'", args.value)
}
```

Tests: wait for a known process name (the test runner itself) and verify it returns quickly; wait for a non-existent target with timeout 1s and verify timeout error.

Commit: `feat(linux): add wait_for tool for process/window/clipboard`

#### Task 2.6: `snapshot`

**Files:**
- Create: `src/tools/snapshot.rs`
- Modify: `src/tools/mod.rs`, `src/mcp/server.rs`
- Create: `tests/snapshot_test.rs`

Snapshot returns a JSON description of the desktop state plus a base64 screenshot. On X11, include the window tree (id, class, title, geometry). On Wayland, include only a screenshot and a note that window tree is unavailable. If `annotate=true`, draw reference lines/numbers on the screenshot to help coordinate-based tools.

Use `x11rb` to query the tree. The image annotation can be done with `image` crate drawing primitives.

Tests: headless returns an error; with display, verify JSON includes `screenshot` and `windows` keys.

Commit: `feat(linux): add snapshot tool with window tree and annotation`

#### Task 2.7: `multi_select` and `multi_edit`

**Files:**
- Create: `src/tools/multi_select.rs`, `src/tools/multi_edit.rs`
- Modify: `src/tools/mod.rs`, `src/mcp/server.rs`
- Create: `tests/multi_test.rs`

`multi_select` clicks each coordinate with `Ctrl` held (or not) to multi-select.

`multi_edit` clicks each coordinate and types the provided text. Optionally click between edits.

Implementation uses the input backend. On Wayland, degrade gracefully.

Tests: without display, expect graceful error; with display, verify no panic.

Commit: `feat(linux): add multi_select and multi_edit tools`

### Phase 3 — Notifications & Web

#### Task 3.1: `notification`

**Files:**
- Create: `src/tools/notification.rs`
- Modify: `src/tools/mod.rs`, `src/mcp/server.rs`
- Create: `tests/notification_test.rs`

Implementation:

```rust
use notify_rust::Notification;

pub fn run_notification(args: &NotificationArgs) -> anyhow::Result<String> {
    Notification::new()
        .summary(&args.title)
        .body(&args.message)
        .app_name(&args.app_id)
        .show()?;
    Ok("notification sent".to_string())
}
```

Tests: in a session with a notification server, verify no error. In headless CI, skip if D-Bus is unavailable.

Commit: `feat(linux): add notification tool`

#### Task 3.2: `scrape`

**Files:**
- Create: `src/tools/scrape.rs`
- Modify: `src/tools/mod.rs`, `src/mcp/server.rs`
- Create: `tests/scrape_test.rs`

Implementation:

```rust
use reqwest;
use scraper::{Html, Selector};

pub async fn run_scrape(args: &ScrapeArgs) -> anyhow::Result<String> {
    let body = reqwest::get(&args.url).await?.text().await?;
    if let Some(query) = args.query.as_deref() {
        let doc = Html::parse_document(&body);
        let selector = Selector::parse(query).map_err(|e| anyhow::anyhow!("invalid CSS selector: {:?}", e))?;
        let texts: Vec<String> = doc.select(&selector).map(|e| e.text().collect::<String>().trim().to_string()).collect();
        Ok(texts.join("\n"))
    } else {
        Ok(body)
    }
}
```

Tests: start a tiny local HTTP server in the test, serve a small HTML page, and verify scraping returns expected text. Use `tokio::net::TcpListener` + `hyper` or a simple `tokio::spawn` server. Alternatively, test against `httpbin.org/html` only if network is available; prefer a local server to avoid flakes.

Commit: `feat(linux): add scrape tool with local server test`

### Phase 4 — Transport, Wiring, Docs, CI

#### Task 4.1: SSE transport

**Files:**
- Modify: `src/main.rs`
- Modify: `Cargo.toml` if a feature flag is needed.
- Create: `tests/sse_smoke.rs`

Implement SSE using `rmcp` server-side HTTP transport. The CLI already has `--transport sse --port 8080`; currently it bails. Replace the bail with an actual `rmcp` HTTP SSE serve call. Because the exact API depends on `rmcp` version, consult the `rmcp` examples for `ServerHandler::serve_http` or use `axum` with `rmcp`'s SSE upgrade handler.

A pragmatic first version: use `tokio::net::TcpListener` + `axum` (add `axum` dep) to expose an `/sse` endpoint that creates an `rmcp` service over the SSE transport. If `rmcp` 2.2 does not expose a ready SSE transport, document SSE as experimental and keep the bail, but add an integration test marked `#[ignore]`.

Given time constraints, the plan allows leaving SSE behind a compile-time feature or an ignored test if `rmcp` support is incomplete. The primary transport is stdio, which is what the TypeScript plugin uses.

Commit: `feat(linux): add SSE transport endpoint`

#### Task 4.2: TypeScript plugin wiring

**Files:**
- Modify: `src/index.ts`
- Modify: `package.json`

In `src/index.ts`, detect the host OS:

```typescript
const isWindows = process.platform === 'win32';
const isLinux = process.platform === 'linux';
```

For Linux, register the Rust binary:

```typescript
const linuxBin = path.resolve(__dirname, '../linux/open-controller-linux/target/release/open-controller-linux');
server.server('open-controller-linux', {
  command: linuxBin,
  args: ['serve', '--transport', 'stdio', '--shell-allowlist', '.*'],
});
```

Also rename tool registrations if needed so the plugin exposes `pc-exec` and `pc-screenshot` consistently on both platforms.

Update `package.json`:

```json
"os": ["win32", "linux"],
"scripts": {
  "build:linux": "cargo build --release --manifest-path linux/open-controller-linux/Cargo.toml"
}
```

Build the TypeScript plugin:

```bash
npm run build
```

Expected: `tsc` compiles without errors.

Commit: `feat(plugin): register open-controller-linux binary on Linux`

#### Task 4.3: Install-deps script and README

**Files:**
- Create: `scripts/install-deps.sh`
- Modify: `README.md`

`scripts/install-deps.sh` prints the distro-specific package install command for the X11 / Wayland / Rust build dependencies:

```bash
#!/bin/bash
set -e
if command -v apt-get &>/dev/null; then
    sudo apt-get update
    sudo apt-get install -y libx11-dev libxrandr-dev libxext-dev libwayland-dev pkg-config
elif command -v dnf &>/dev/null; then
    sudo dnf install -y libX11-devel libXrandr-devel libXext-devel wayland-devel pkgconfig
elif command -v pacman &>/dev/null; then
    sudo pacman -S --needed libx11 libxrandr libxext wayland pkgconf
else
    echo "Unsupported package manager. Install X11/Wayland development headers manually."
    exit 1
fi
```

Make it executable: `chmod +x scripts/install-deps.sh`.

Update `README.md` with a "Linux" section explaining:
- Build the Rust binary: `cargo build --release --manifest-path linux/open-controller-linux/Cargo.toml`
- Run install deps: `./scripts/install-deps.sh`
- Plugin usage on Linux.

Commit: `docs: add Linux build deps script and README section`

#### Task 4.4: CI workflow

**Files:**
- Create: `.github/workflows/linux-rust.yml`

Workflow:

```yaml
name: Linux Rust
on: [push, pull_request]
jobs:
  linux-rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - run: sudo apt-get update && sudo apt-get install -y libx11-dev libxrandr-dev libxext-dev libwayland-dev pkg-config
      - run: cargo fmt --manifest-path linux/open-controller-linux/Cargo.toml -- --check
      - run: cargo clippy --manifest-path linux/open-controller-linux/Cargo.toml -- -D warnings
      - run: cargo test --manifest-path linux/open-controller-linux/Cargo.toml
      - run: cargo build --release --manifest-path linux/open-controller-linux/Cargo.toml
      - run: npm ci
      - run: npm run build
```

Commit: `ci: add Linux Rust build and test workflow`

#### Task 4.5: Final smoke test asserting 18 tools

**Files:**
- Modify: `tests/mcp_smoke.rs`

Expand `stdio_initialize_and_list_tools` to assert that the returned tools array contains exactly the 18 expected tool names: `shell`, `file_system`, `process`, `clipboard`, `screenshot`, `shortcut`, `click`, `type`, `scroll`, `move`, `app`, `wait`, `wait_for`, `snapshot`, `multi_select`, `multi_edit`, `notification`, `scrape`.

Also add a second test that calls `shell` with an allowed pattern and verifies the echo response, and calls `file_system` read on a known file.

Verification:

```bash
cd linux/open-controller-linux
cargo test
```

Expected: all tests pass.

Commit: `test: expand mcp smoke test to assert all 18 tools`

---

## 6. Verification Commands

Run these after every task and before finishing:

```bash
cd linux/open-controller-linux
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

At the repo root:

```bash
npm ci
npm run build
```

All commands must exit 0 before claiming the task is complete.

---

## 7. Spec Coverage Review

| Requirement | Task |
|---|---|
| Rust-based Linux MCP server | Phase 0, Task 0.1 |
| Mirrors Windows tool feature set | Phases 1-3 |
| Uses rmcp + tokio | Tech stack, Task 0.3 |
| X11 primary, Wayland best-effort | `src/linux/`, Tasks 1.5, 2.1 |
| Drops Registry | Tool mapping, server registration |
| Wires into existing plugin | Task 4.2 |
| Wayland/X11 detection | Task 0.2 |
| Permission guards | Security model, per-tool tasks |
| Exact tool names and JSON schemas | Section 4 |
| TDD-oriented atomic tasks | Every task includes failing test first |
| Verification commands | Section 6 |
| Security risks & mitigations | Section 3 |
| Atomic commit strategy | Commit step in every task |

---

## 8. Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-07-10-rust-linux-controller.md`.**

**Execution options:**

1. **Subagent-Driven (recommended):** I dispatch a fresh subagent per remaining task, review between tasks, and keep atomic commits.
2. **Inline Execution:** Continue in this session using `superpowers:executing-plans`, batching tasks with checkpoints.

**Completed so far:** crate scaffold, detect, MCP server scaffold, shell, file_system, process.

**Next pending task:** clipboard tool.
