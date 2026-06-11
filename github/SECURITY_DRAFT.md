# Security Policy Draft

Status: superseded by [SECURITY.md](../SECURITY.md).

This draft is retained only as historical planning context. Use the root security policy for current public reporting guidance.

## Historical Notes

Security-sensitive areas include:

- Plugin permission bypass
- MCP permission bypass
- Original file modification
- Export path vulnerabilities
- Updater/signing issues
- Model/plugin downloads
- Catalog corruption/data loss

## Security Principles

- No original file mutation
- No telemetry by default
- No cloud upload by default
- Deferred MLX, MCP, plugin, auto-update, and distribution surfaces require explicit security review before implementation.
