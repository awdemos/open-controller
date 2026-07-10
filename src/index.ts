import type { Plugin } from "@opencode-ai/plugin"
import { tool } from "@opencode-ai/plugin"
import { execSync } from "child_process"
import { join } from "path"
import { homedir, platform } from "os"

const isWindows = platform() === "win32"
const isLinux = platform() === "linux"

const PYTHON_CMD = "C:\\Program Files\\Python311\\python.exe"
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

const ps = (cmd: string): string => {
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

const checkMcpInstalled = (): boolean => {
  if (!isWindows) return false
  try {
    execSync(
      `"${PYTHON_CMD}" -c "from windows_mcp.__main__ import main"`,
      { encoding: "utf8" },
    )
    return true
  } catch {
    return false
  }
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
      "[open-controller] windows_mcp package not found. Install: pip install windows-mcp",
    )
  }

  const localTools = isWindows
    ? {
        "pc-exec": tool({
          description:
            "Execute a PowerShell command on this Windows PC. Use for file, process, registry, clipboard, UI automation tasks.",
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
            `)
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
        cfg.mcp["open-controller-linux"] = {
          type: "local",
          command: [LINUX_BIN, "serve", "--transport", "stdio", "--shell-allowlist", ".*"],
          enabled: true,
        }
      }
    },

    tool: localTools,
  }
}

export default { id: "open-controller", server: PcControllerPlugin }
