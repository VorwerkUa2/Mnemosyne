# Mnemosyne — Integration Notes

## Local model API

LM Studio is the primary model backend. Mnemosyne uses its local Anthropic-compatible API:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/v1/models` | GET | Discover loaded models |
| `/v1/messages` | POST | Chat and context summarization |

Default base URL: `http://127.0.0.1:1234`. Ollama is an optional local alternative at `http://127.0.0.1:11434` when its version supports both routes.

## MCP Config

When enabled, Decod and Designatio MCP servers are added to the Claude Code CLI session:

```
--mcp-config {"mcpServers":{"decod":{"command":"python3","args":["<Decod>/core/mcp_server.py"]},"designatio":{"command":"python3","args":["-m","designatio.mcp.server"],"cwd":"<Designatio>"}}}
```

On Windows the configured Python command is `python`; on Linux it is `python3`. Mnemosyne resolves the module directories from platform-specific Desktop/home folders and also permits explicit paths in Settings. The personal Figma token, when configured, is passed only to Designatio's process environment.

Designatio's local JSON mockup import does not require a Figma token.

## Claude Code CLI paths

**Windows:**
- `claude` (PATH)
- `%APPDATA%\npm\claude.cmd`
- `%USERPROFILE%\.claude\local\claude.exe`

**Linux:**
- `claude` (PATH)
- `~/.claude/local/claude`
- `/usr/local/bin/claude`

## Linux Desktop Check

Env vars to check:
- `DISPLAY` (X11)
- `WAYLAND_DISPLAY` (Wayland)
- `XDG_SESSION_TYPE`
