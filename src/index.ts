import type { Plugin } from "@opencode-ai/plugin"
import { tool } from "@opencode-ai/plugin"
import { execSync } from "child_process"
import { join } from "path"
import { homedir, platform } from "os"

const isWindows = platform() === "win32"
const isLinux = platform() === "linux"

const WINDOWS_MCP_VERSION = "0.8.2"
const PYTHON_CMD = findPython()
const MCP_WRAPPER = join(homedir(), ".windows-mcp", "mcp_runner.py")
const LINUX_BIN = join(
  __dirname,
  "..",
  "linux",
  "open-controller-linux",
  "target",
  "release",
  "open-controller-linux",
)

function findPython(): string {
  const candidates = [
    "C:\\Program Files\\Python311\\python.exe",
    "C:\\Program Files\\Python312\\python.exe",
    "C:\\Program Files\\Python313\\python.exe",
    "C:\\Users\\oguzhan\\AppData\\Local\\Programs\\Python\\Python313\\python.exe",
    "python.exe",
    "python",
  ]
  for (const candidate of candidates) {
    try {
      execSync(`"${candidate}" --version`, { encoding: "utf8", timeout: 5000 })
      return candidate
    } catch {
      // try next
    }
  }
  // Fallback to the most common system path for the warning message.
  return "C:\\Program Files\\Python311\\python.exe"
}

const PS_ALLOWED_CMDLETS = new Set([
  "get-childitem",
  "get-content",
  "get-process",
  "get-service",
  "get-item",
  "get-itemproperty",
  "get-date",
  "get-location",
  "select-string",
  "where-object",
  "sort-object",
  "measure-object",
  "foreach-object",
  "format-list",
  "format-table",
  "write-output",
  "out-string",
  "convertfrom-json",
  "convertto-json",
])

const PS_BLOCKED_PATTERNS = [
  /invoke-expression/i,
  /invoke-webrequest/i,
  /invoke-restmethod/i,
  /start-process/i,
  /new-object/i,
  /iex\b/i,
  /net\.webclient/i,
  /downloadstring/i,
  /downloadfile/i,
  /-encodedcommand/i,
  /-enc\b/i,
]

function validatePowerShell(command: string): string | undefined {
  for (const pattern of PS_BLOCKED_PATTERNS) {
    if (pattern.test(command)) {
      return `blocked PowerShell pattern: ${pattern.source}`
    }
  }

  const segments = command.split(/[;|]/)
  for (const segment of segments) {
    const trimmed = segment.trim()
    if (!trimmed) continue
    const firstToken = trimmed.split(/\s+/)[0].toLowerCase()
    // Allow quoted literals such as variable assignments like "$x = ...".
    if (firstToken.startsWith("$") || firstToken.startsWith("@")) continue
    if (!PS_ALLOWED_CMDLETS.has(firstToken)) {
      return `PowerShell cmdlet not in allowlist: ${firstToken}`
    }
  }

  return undefined
}

const ps = (cmd: string, skipValidation = false): string => {
  if (!skipValidation) {
    const validationError = validatePowerShell(cmd)
    if (validationError) {
      return `Error: ${validationError}`
    }
  }
  try {
    return execSync(cmd, {
      encoding: "utf8",
      timeout: 30000,
      shell: "powershell",
    }).trim()
  } catch (e: any) {
    return `Error: ${e.message}`
  }
}

function checkMcpInstalled(): boolean {
  if (!isWindows) return false
  try {
    const output = execSync(
      `"${PYTHON_CMD}" -c "import windows_mcp; print(windows_mcp.__version__)"`,
      { encoding: "utf8", timeout: 10000 },
    ).trim()
    return output === WINDOWS_MCP_VERSION
  } catch {
    return false
  }
}

function installMcpCommand(): string {
  return `pip install windows-mcp==${WINDOWS_MCP_VERSION}`
}

const PcControllerPlugin: Plugin = async (_ctx) => {
  const installed = isWindows ? checkMcpInstalled() : false

  if (installed) {
    try {
      execSync(
        `"${PYTHON_CMD}" -c "import comtypes.client; import windows_mcp; print('ok')"`,
        { encoding: "utf8", timeout: 15000 },
      )
    } catch {
      /* pre-warm failed, non-fatal */
    }
  }

  if (!installed && isWindows) {
    console.warn(
      `[open-controller] windows_mcp ${WINDOWS_MCP_VERSION} not found. Install: ${installMcpCommand()}`,
    )
  }

  const localTools = isWindows
    ? {
        "pc-exec": tool({
          description:
            "Execute a restricted PowerShell command on this Windows PC. Only read-only/cmdlet-based commands are allowed.",
          args: {
            command: tool.schema.string().describe("PowerShell command to execute"),
          },
          execute: async (args) => {
            const out = ps(args.command)
            return out.length > 4000
              ? out.slice(0, 4000) + "\n... (truncated)"
              : out
          },
        }),

        "pc-screenshot": tool({
          description: "Capture a screenshot of the current desktop as base64 PNG.",
          args: {},
          execute: async () => {
            const b64 = ps(`
              Add-Type -AssemblyName System.Drawing
              $bmp = [Drawing.Bitmap]::new([Drawing.SystemInformation]::VirtualScreen.Width, [Drawing.SystemInformation]::VirtualScreen.Height)
              $g = [Drawing.Graphics]::FromImage($bmp)
              $g.CopyFromScreen([Drawing.SystemInformation]::VirtualScreen.X, [Drawing.SystemInformation]::VirtualScreen.Y, 0, 0, $bmp.Size)
              $ms = New-Object IO.MemoryStream
              $bmp.Save($ms, [Drawing.Imaging.ImageFormat]::Png)
              [Convert]::ToBase64String($ms.ToArray())
              $g.Dispose(); $bmp.Dispose(); $ms.Dispose()
            `, true)
            return {
              output: b64,
              attachments: [
                {
                  type: "file",
                  mime: "image/png",
                  url: `data:image/png;base64,${b64}`,
                },
              ],
            }
          },
        }),
      }
    : undefined

  return {
    config: async (cfg: any) => {
      cfg.mcp ??= {}

      if (isWindows && installed) {
        cfg.mcp["windows-mcp"] = {
          type: "local",
          command: [PYTHON_CMD, MCP_WRAPPER, "serve", "--transport", "stdio"],
          enabled: true,
        }
      }

      if (isLinux) {
        // Default to an empty shell allowlist. The user must explicitly configure
        // allowed command patterns in opencode.jsonc under the "open-controller"
        // key, e.g.:
        //   "open-controller": {
        //     "linuxShellAllowlist": ["^echo ", "^ls "]
        //   }
        const userConfig = cfg["open-controller"] ?? {}
        const allowlist: string[] = userConfig.linuxShellAllowlist ?? []
        const command = [LINUX_BIN, "serve", "--transport", "stdio"]
        if (allowlist.length > 0) {
          command.push("--shell-allowlist", allowlist.join(","))
        }
        cfg.mcp["open-controller-linux"] = {
          type: "local",
          command,
          enabled: true,
        }
      }
    },

    tool: localTools,
  }
}

export default { id: "open-controller", server: PcControllerPlugin }
