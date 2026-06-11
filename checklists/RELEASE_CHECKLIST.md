# Release Checklist

## Public Release Gate

- [ ] CI passing
- [ ] No S0 bugs
- [ ] Original file protection tests pass
- [ ] Export tests pass
- [ ] Migration tests pass
- [ ] Color QA pass
- [ ] Known issues documented
- [ ] Checksums generated
- [ ] Release notes drafted
- [ ] Signing/notarization verified, if public beta/stable

## Stable Release Gate

- [ ] Signed/notarized/stapled build
- [ ] Homebrew Cask and auto-update prepared or deferred with explanation ([ADR 0007](../docs/wiki/decisions/adr-0007-homebrew-and-auto-update-deferral.md))
- [ ] Security policy exists
- [ ] Contribution guide exists
- [ ] Upgrade path tested
