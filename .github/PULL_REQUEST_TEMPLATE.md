# Pull Request

## Summary

## Linked Issue

## Scope

## Out of Scope

## Tests

- [ ] `scripts/harness/check.sh`
- [ ] Unit tests
- [ ] Integration tests
- [ ] Manual testing
- [ ] Not applicable, explain:

## Local Alpha Safety

- [ ] Does not modify original photo files.
- [ ] Does not add network upload, telemetry, analytics, or cloud sync.
- [ ] Does not add MLX/MCP/plugin runtime unless explicitly scoped.
- [ ] Does not bypass Rust Core ownership of storage/render/edit operations.
- [ ] Does not add unapproved dependencies.
- [ ] Updates `docs/DEPENDENCIES.md` for every new dependency.
- [ ] Updates docs/wiki decisions or roadmap when behavior or scope changes.

## Public Trust

- [ ] Does not claim production readiness.
- [ ] Does not claim broad RAW support without fixture-backed evidence.
- [ ] Does not claim color correctness without tagged fixtures and tolerance checks.
- [ ] Does not treat unsigned developer-preview DMGs as user-ready releases.
- [ ] Keeps deferred MLX, MCP, plugin, Homebrew, auto-update, cloud sync, telemetry, and Mac App Store surfaces clearly marked as deferred unless explicitly scoped.

## Release Blocker Check

- [ ] No app launch failure.
- [ ] No original-file mutation.
- [ ] No edit-loss risk.
- [ ] No catalog corruption risk.
- [ ] No silently wrong color export path.
- [ ] No unauthorized mutation path.
