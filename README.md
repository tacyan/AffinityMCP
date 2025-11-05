# AffinityMCP

The Universal MCP Server (Rust) exposes tools for Canva workflows—and **Affinity Photo/Designer/Publisher automation**—designed for **natural language** usage in MCP-compatible clients.

## Features

- 🎨 **Natural Language Control**: Control Affinity applications using natural language commands
- 🖼️ **File Operations**: Open files, create new documents, export in various formats
- ⚡ **16-Parallel Processing**: Process up to 16 files simultaneously for maximum efficiency
- 🔧 **Advanced Tools**: Apply filters, get document info, and more
- 🚀 **Easy Setup**: Simple setup for beginners with comprehensive documentation
- 🌐 **Cross-Platform Ready**: Currently macOS (AppleScript), Windows support planned

## Quick Start

For beginners, see [SETUP_GUIDE.md](SETUP_GUIDE.md) for detailed step-by-step instructions.

**Quick installation:**

```bash
# Build
cargo build --release
cp target/release/affinity-mcp dist/

# Use with Cursor
# Add to Cursor MCP settings:
{
  "mcpServers": {
    "affinity-mcp": {
      "command": "npx",
      "args": ["affinity-mcp"],
      "autoStart": true
    }
  }
}
```

## Installation

### Prerequisites

- Node.js 18+ (for the thin npx wrapper)

- Rust 1.76+ / Cargo (for local builds)

- Set `AFFINITY_MCP_API_KEY` if using Canva Connect/MCP endpoints.

### Get an API key

- If your tools require Canva APIs, obtain credentials from the Canva developer console or MCP server guidance. Otherwise, you can skip this step.

### Build locally

```bash
cd /path/to/affinity-mcp
cargo build --release
# copy target/release/affinity-mcp -> ./dist/
cp target/release/affinity-mcp dist/
```

## Setup: Claude Code (CLI)

Use this one-liner (replace with your real values):

```bash
claude mcp add AffinityMCP -s user -e AFFINITY_MCP_API_KEY="sk-your-real-key" -- npx affinity-mcp
```

To remove:

```bash
claude mcp remove AffinityMCP
```

## Setup: Cursor (Do not commit .cursor/mcp.json here)

Create the config in your client:

```json
{
  "mcpServers": {
    "affinity-mcp": {
      "command": "npx",
      "args": ["affinity-mcp"],
      "env": { "AFFINITY_MCP_API_KEY": "sk-your-real-key" },
      "autoStart": true
    }
  }
}
```

## Other Clients and Agents

<details><summary>VS Code</summary>

```bash
code --add-mcp '{"name":"affinity-mcp","command":"npx","args":["affinity-mcp"],"env":{"AFFINITY_MCP_API_KEY":"sk-your-real-key"}}'
```

</details>

<details><summary>Claude Desktop</summary>

Follow the MCP install guide and reuse the standard config above.

</details>

<details><summary>LM Studio</summary>

- Command: npx
- Args: ["affinity-mcp"]
- Env: AFFINITY_MCP_API_KEY=sk-your-real-key

</details>

<details><summary>Goose</summary>

- Type: STDIO
- Command: npx
- Args: ["affinity-mcp"]
- Enabled: true

</details>

<details><summary>opencode</summary>

~/.config/opencode/opencode.json:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "affinity-mcp": {
      "type": "local",
      "command": ["npx", "affinity-mcp"],
      "enabled": true
    }
  }
}
```

</details>

<details><summary>Qodo Gen</summary>

Add a new MCP and paste the standard JSON config.

</details>

<details><summary>Windsurf</summary>

See docs and reuse the standard config above.

</details>

## Setup: Codex (TOML)

Example (Serena):

```toml
[mcp_servers.serena]
command = "uvx"
args = ["--from", "git+https://github.com/oraios/serena", "serena", "start-mcp-server", "--context", "codex"]
```

This server (minimal):

```toml
[mcp_servers.affinity-mcp]
command = "npx"
args = ["affinity-mcp"]

# Optional environment variables:
# AFFINITY_MCP_API_KEY = "sk-your-real-key"
# MCP_NAME = "affinity-mcp"
```

## Configuration (Env)

- `AFFINITY_MCP_API_KEY`: Canva API key or token (if applicable)
- `MCP_NAME`: Server name override (default: affinity-mcp)

If your tools are purely local, no API keys are required.

## Available Tools

### Affinity Tools (Natural Language Support)

All Affinity tools support natural language commands. Examples:
- "Open the file /path/to/image.jpg in Affinity Photo"
- "Create a new document with width 1920 and height 1080"
- "Export the current document as PDF to /path/to/output.pdf"

#### affinity.open_file

Open a file in Affinity application (auto-detects app from file extension).

- inputs: { "path": string, "app"?: "Photo"|"Designer"|"Publisher" }
- outputs: { "opened": boolean, "app": string, "path": string }

#### affinity.create_new

Create a new Affinity document.

- inputs: { "app": "Photo"|"Designer"|"Publisher", "width"?: number, "height"?: number }
- outputs: { "created": boolean, "app": string }

#### affinity.export

Export the currently open document.

- inputs: { "path": string, "format": "pdf"|"png"|"jpg"|"tiff"|"svg", "quality"?: number }
- outputs: { "exported": boolean, "path": string }

#### affinity.apply_filter

Apply a filter to the current document.

- inputs: { "filter_name": string, "intensity"?: number }
- outputs: { "applied": boolean, "filter_name": string }

#### affinity.get_active_document

Get information about the currently active document.

- inputs: {}
- outputs: { "is_open": boolean, "name"?: string, "path"?: string }

#### affinity.close_document

Close the currently open document.

- inputs: {}
- outputs: { "closed": boolean }

#### affinity.batch_open_files ⚡ **16-Parallel**

Open multiple files simultaneously (up to 16 files in parallel).

- inputs: { "paths": string[], "app"?: "Photo"|"Designer"|"Publisher" }
- outputs: { "success_count": number, "failure_count": number, "results": OpenFileResult[] }

**Natural language example**: "Open multiple files: /path/to/image1.jpg, /path/to/image2.jpg, /path/to/image3.jpg"

#### affinity.batch_export ⚡ **16-Parallel**

Export multiple documents simultaneously (up to 16 exports in parallel).

- inputs: { "exports": ExportParams[] }
- outputs: { "success_count": number, "failure_count": number, "results": ExportResult[] }

**Natural language example**: "Export all open documents as PDF files"

### Canva Tools

#### canva.create_design

- inputs: { "title": string, "template_id"?: string, "width"?: number, "height"?: number }
- outputs: { "design_id": string, "url"?: string }

## Example Usage

### Natural Language Commands

In Cursor or other MCP-compatible clients, you can use natural language:

**Example 1: Open a file**
```
User: "Open /Users/john/Desktop/image.jpg in Affinity Photo"
AI: [Calls affinity.open_file with path="/Users/john/Desktop/image.jpg", app="Photo"]
```

**Example 2: Create and export**
```
User: "Create a new Affinity Photo document with size 1920x1080, then export it as PDF to my Desktop"
AI: [Calls affinity.create_new, then affinity.export]
```

**Example 3: Get document info**
```
User: "What document is currently open?"
AI: [Calls affinity.get_active_document]
```

**Example 4: Batch operations (16-parallel)**
```
User: "Open these 5 images simultaneously: /path/to/img1.jpg, /path/to/img2.jpg, /path/to/img3.jpg, /path/to/img4.jpg, /path/to/img5.jpg"
AI: [Calls affinity.batch_open_files with paths array]
```

**Example 5: Batch export (16-parallel)**
```
User: "Export all open documents as PDF files to the Desktop"
AI: [Calls affinity.batch_export with multiple export configurations]
```

### Direct Tool Invocation (MCP tool call)

```json
{
  "type": "tool",
  "name": "affinity.open_file",
  "arguments": {
    "path": "/path/to/image.jpg",
    "app": "Photo"
  }
}
```

## Documentation

- **[SETUP_GUIDE.md](SETUP_GUIDE.md)**: Beginner-friendly setup guide with step-by-step instructions
- **[SPEC.md](SPEC.md)**: Technical specification (Japanese)
- **[README.md](README.md)**: This file (English documentation)

For complex schemas, see `src/tools/*.rs`.

## Name Consistency & Troubleshooting

Always use CANONICAL_ID (`affinity-mcp`) for identifiers and keys.

Use CANONICAL_DISPLAY (`AffinityMCP`) only for UI labels.

Do not mix legacy keys after registration.

### Consistency Matrix

- npm package name → `affinity-mcp`
- Binary name → `affinity-mcp`
- MCP server name → `affinity-mcp`
- Env default MCP_NAME → `affinity-mcp`
- Client registry key → `affinity-mcp`
- UI label → `AffinityMCP`

### Conflict Cleanup

Remove stale keys and re-add with `affinity-mcp` only.

Cursor is UI-configured only; this repo intentionally omits `.cursor/mcp.json`.

