# OpenCode PC Controller Plugin

Control your Windows or Linux PC directly from OpenCode.
- **Windows:** Uses the [windows-mcp](https://github.com/CursorTouch/Windows-MCP) MCP server.
- **Linux:** Uses a native Rust MCP server (`open-controller-linux`) included in this package.

## Features

- **Desktop UI Automation** — click, type, scroll, drag, switch apps, resize windows
- **File System** — read, write, copy, move, delete files and directories within a configured base directory
- **PowerShell / Shell** — execute PowerShell commands on Windows or shell commands on Linux with allowlist controls
- **Screenshots** — capture and analyze desktop screenshots
- **Registry** — read, write, delete registry keys (Windows only; not available on Linux)
- **Process** — list and kill running processes
- **Clipboard** — get and set clipboard content (requires confirmation)
- **Notifications** — send desktop notifications (requires confirmation)
- **Web Scraping** — fetch and extract web page content from public URLs
- **Snapshot** — inspect UI elements on screen with coordinates
- **Multi-actions** — batch select and edit multiple UI elements

## Security model

This plugin is intentionally powerful. To reduce the risk of prompt-injection or remote
exploitation, several gates are enforced:

- **Linux shell commands require an explicit allowlist.** No command is allowed by default.
- **Linux file_system operations are restricted to a configurable base directory** and cannot
  traverse outside it.
- **The `scrape` tool refuses private, localhost, and cloud-metadata URLs** unless an explicit
  allowlist overrides the default.
- **Destructive UI, file, process, clipboard, and notification operations require the server
  to be started with `--confirm-destructive`.** The previous caller-controlled `confirm=true`
  JSON argument is no longer honored.
- **Windows `pc-exec` restricts PowerShell to a read-only cmdlet allowlist.** Arbitrary
  `Invoke-Expression`, `Start-Process`, encoded commands, and download/cradle patterns are blocked.
- **The SSE transport binds to `127.0.0.1` by default** and can be protected with a bearer token
  via `--sse-token`.

## Prerequisites

- Node.js 18+
- OpenCode
- Python 3.11+ (Windows only)

## Installation

### Linux

On Linux the plugin uses the bundled Rust MCP server. Build it before running OpenCode:

```bash
# Install system build dependencies (X11 / Wayland headers)
./scripts/install-deps.sh

# Build the Rust MCP server
cargo build --release --manifest-path linux/open-controller-linux/Cargo.toml

# Install npm dependencies and build the plugin
npm install
npm run build
```

The Rust server is then registered automatically when the plugin loads on a Linux host.

By default the Linux shell tool denies every command. Add an explicit allowlist to your
OpenCode config (`opencode.jsonc`):

```jsonc
{
  "plugin": ["open-controller"],
  "open-controller": {
    "linuxShellAllowlist": ["^echo ", "^ls ", "^cat "]
  }
}
```

> **Warning:** Do not use `.*` as an allowlist pattern — it permits arbitrary shell execution.

### Windows

### 1. Install the npm plugin

In your OpenCode config directory (typically `%USERPROFILE%\.config\opencode`):

```cmd
cd %USERPROFILE%\.config\opencode
npm install open-controller
```

Or with PowerShell:

```powershell
cd ~\.config\opencode
npm install open-controller
```

### 2. Install the MCP server (Python package)

```bash
pip install windows-mcp==0.8.2
```

The plugin verifies that the installed `windows-mcp` version matches `0.8.2`.

### 3. Add to OpenCode config

Add `"open-controller"` to the `"plugin"` array in `%USERPROFILE%\.config\opencode\opencode.jsonc` if not already present:

```jsonc
{
  "plugin": [
    "open-controller",
    // ... other plugins
  ]
}
```

### 4. Restart OpenCode

Try these commands:
- "list files on desktop"
- "take a screenshot"
- "open chrome"
- "run notepad"

## MCP Tools

The plugin registers MCP tools via `windows-mcp` on Windows or `open-controller-linux` on Linux:

| Tool | Windows | Linux | Notes |
|---|---|---|---|
| App | ✅ | ⚠️ | Linux `switch`/`resize` require X11 |
| Shortcut | ✅ | ⚠️ | Linux X11 via `enigo`; Wayland limited |
| Snapshot | ✅ | ⚠️ | Linux X11 window tree + screenshot |
| Screenshot | ✅ | ✅ | Linux X11 via `x11rb`; Wayland via portal |
| Click / Type / Scroll / Move | ✅ | ⚠️ | Linux X11 via `enigo`; Wayland limited |
| Wait / WaitFor | ✅ | ⚠️ | Linux WaitFor limited to process/window/clipboard |
| FileSystem | ✅ | ✅ | Restricted to configured base directory on Linux |
| PowerShell / Shell | ✅ | ✅ | Linux uses `/bin/sh` with explicit allowlist |
| Clipboard | ✅ | ✅ | Requires `--confirm-destructive` on Linux |
| Process | ✅ | ✅ | Linux kill restricted to current UID |
| Registry | ✅ | ❌ | Windows only |
| Notification | ✅ | ✅ | Requires `--confirm-destructive` on Linux |
| Scrape | ✅ | ✅ | Blocks private/localhost/metadata URLs by default |
| MultiSelect / MultiEdit | ✅ | ⚠️ | Linux coordinate-based on X11 |

### App Control
- **App** — launch, switch, or resize application windows
- **Shortcut** — execute keyboard shortcuts (`ctrl+c`, `win+r`, etc.)

### Desktop & UI
- **Snapshot** — capture desktop state with UI tree, interactive elements, and coordinates
- **Screenshot** — fast screenshot with optional annotation overlay
- **Click** — single, double, right-click at coordinates or on UI element labels
- **Type** — type text at coordinates or into UI elements
- **Scroll** — scroll vertically/horizontally at a location
- **Move** — move cursor (hover) or drag-and-drop
- **Wait** — pause execution for a duration
- **WaitFor** — wait for UI condition (text exists, element enabled, window active)

### File System
- **FileSystem** — read, write, copy, move, delete, list, search, get info (8 modes)

### Shell
- **PowerShell** — execute PowerShell commands with allowlist controls on Windows

### Clipboard
- **Clipboard** — get or set clipboard text

### Process
- **Process** — list processes (sorted by CPU/memory/name) or kill by PID/name

### Registry
- **Registry** — get, set, delete, list Windows Registry keys/values

### Notifications
- **Notification** — send Windows toast notifications

### Web
- **Scrape** — fetch and extract web page content with optional DOM mode

### Multi
- **MultiSelect** — batch select multiple items (files, checkboxes)
- **MultiEdit** — enter text into multiple input fields

## Built-in Plugin Tools

Two extra tools are registered directly on the plugin:

| Tool | Description |
|------|-------------|
| `pc-exec` | Execute a restricted PowerShell command with output |
| `pc-screenshot` | Capture desktop as base64 PNG image |

## Linux server CLI options

```
open-controller-linux serve
  --transport <stdio|sse>     # default: stdio
  --port <PORT>               # default: 8080 (SSE only)
  --confirm-destructive       # required for destructive UI/file/process/clipboard/notification tools
  --shell-allowlist <PATTERNS> # comma-separated regex patterns allowed for the shell tool
  --base-dir <PATH>            # directory the file_system tool is restricted to (default: cwd)
  --scrape-allowlist <PATTERNS> # comma-separated regex patterns allowed for the scrape tool
  --sse-bind <IP>              # default: 127.0.0.1
  --sse-token <TOKEN>          # bearer token required for SSE requests
```

## Architecture

### Windows

```
OpenCode
  └── open-controller plugin
       ├── pc-exec tool (restricted PowerShell)
       ├── pc-screenshot tool (base64 PNG)
       └── MCP Server: windows-mcp (Python)
            ├── App, Snapshot, Screenshot
            ├── Click, Type, Scroll, Move, Wait
            ├── FileSystem, PowerShell, Registry
            ├── Clipboard, Process, Notification
            ├── Scrape, MultiSelect, MultiEdit
            └── Shortcut, WaitFor
```

### Linux

```
OpenCode
  └── open-controller plugin
       └── MCP Server: open-controller-linux (Rust)
            ├── shell (bash with explicit allowlist)
            ├── file_system (base-dir restricted), process, clipboard
            ├── screenshot, snapshot
            ├── click, type, scroll, move, shortcut
            ├── app, wait, wait_for
            ├── notification, scrape
            ├── multi_select, multi_edit
            └── echo
```

The plugin:
1. Loads on OpenCode startup
2. Detects the host OS
3. Registers the correct MCP server config in OpenCode
4. On Windows, adds `pc-exec` and `pc-screenshot` as native plugin tools

## Development

The plugin source is at the [GitHub repo](https://github.com/amazing-things/open-controller). To modify locally:

```cmd
:: Clone the repo
git clone https://github.com/amazing-things/open-controller.git
cd open-controller

:: Install dependencies
npm install

:: Build
npm run build

:: Use the local version in OpenCode
:: Add "./path/to/open-controller" to opencode.jsonc plugin list
```

Restart OpenCode to apply changes.

## License

MIT — see [LICENSE](LICENSE)

Original windows-mcp by [Jeomon George](https://github.com/CursorTouch)
