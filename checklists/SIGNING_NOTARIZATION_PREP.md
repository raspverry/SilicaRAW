# Signing and Notarization Prep

Use this checklist for Phase 7.1 before any signed or notarized release candidate is built.

This checklist records prerequisites only. Do not commit certificates, `.p12` files, private keys, app-specific passwords, or notarization credentials.

## Required Local State

- [ ] Apple Developer Program membership is active.
- [ ] A `Developer ID Application` certificate exists for the maintainer team.
- [ ] The certificate is installed in the maintainer keychain for local signing, or exported as a password-protected `.p12` for CI.
- [ ] `security find-identity -v -p codesigning` lists a `Developer ID Application:` identity.
- [ ] `APPLE_SIGNING_IDENTITY` matches the intended Developer ID identity.

## Required GitHub Secrets

The Apple ID app-specific password path uses these repository secrets:

- [ ] `APPLE_CERTIFICATE`
- [ ] `APPLE_CERTIFICATE_PASSWORD`
- [ ] `KEYCHAIN_PASSWORD`
- [ ] `APPLE_ID`
- [ ] `APPLE_PASSWORD`
- [ ] `APPLE_TEAM_ID`

The signing identity can be stored as either a repository secret or variable:

- [ ] `APPLE_SIGNING_IDENTITY`

## Local Preflight

Run:

```bash
python3 scripts/harness/check-signing-prereqs.py \
  --output .tmp/signing-prereqs/signing-prereqs.json
```

Use `--fail-on-missing` only when the release environment is expected to be complete.

Current Phase 7.1 audit result on 2026-06-11:

- Local code signing identity found: `Apple Development: Hansol Choi (U6J9CG9B7K)`
- `Developer ID Application` identity found: no
- GitHub Actions secrets found: no required signing/notarization secrets reported by `gh secret list`
- Phase 7.1 status: blocked until Developer ID certificate and required GitHub secrets are prepared

## References

- Tauri macOS code signing: https://v2.tauri.app/distribute/sign/macos/
- Apple Developer ID certificates: https://developer.apple.com/help/account/certificates/create-developer-id-certificates/
- Apple notarization: https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution
- GitHub Actions secrets: https://docs.github.com/actions/security-guides/using-secrets-in-github-actions
