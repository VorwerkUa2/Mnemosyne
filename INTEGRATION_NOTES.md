# Mnemosyne × Decod — Integration Notes

## MCP Config (для claude_command у main.rs)

Коли `decod_mcp_enabled = true`, треба додати до команди:
```
--mcp-config {"mcpServers":{"decod":{"command":"python","args":["C:/Users/knyaz/Desktop/Decod/core/mcp_server.py"]}}}
```

## FreeLLMAPI Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/v1/models` | GET | List available models |
| `/v1/messages` | POST | Send message (streaming) |

Тест-запит для перевірки з'єднання:
```json
{
  "model": "claude-sonnet-4-5",
  "max_tokens": 1,
  "messages": [{"role":"user","content":"ping"}]
}
```

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
