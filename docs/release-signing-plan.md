# Release Signing Plan

## Current State

Orqestra v1.0.5 desktop artifacts are **unsigned beta builds**. No code signing, no notarization, no authenticode. This is acceptable for a public beta but must be resolved before any production or enterprise distribution.

## Windows Code Signing

### What Is Needed

- **Certificate type:** EV Code Signing Certificate or Standard Code Signing Certificate
- **Provider options:** DigiCert, Sectigo, SSL.com
- **Purpose:** Authenticode signature on `.exe` and NSIS installer
- **Benefit:** Eliminates SmartScreen warnings, establishes publisher identity

### Implementation

1. Purchase code signing certificate from a trusted CA
2. Install certificate on Windows CI runner or local build machine
3. Add signing step to `desktop-release.yml`:
   ```yaml
   - name: Sign Windows artifact
     run: |
       signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /a Orqestra_*.exe
     env:
       SIGNING_CERT: ${{ secrets.WINDOWS_SIGNING_CERT }}
       SIGNING_PASSWORD: ${{ secrets.WINDOWS_SIGNING_PASSWORD }}
   ```
4. Verify signature: `signtool verify /pa Orqestra_*.exe`

### CI Secret Requirements

- `WINDOWS_SIGNING_CERT` — base64-encoded PFX certificate
- `WINDOWS_SIGNING_PASSWORD` — certificate password

### Estimated Cost

- Standard code signing: $200-400/year
- EV code signing: $400-800/year (includes hardware token)

### Target Release

v1.1.0 or v1.0.6 if expedited.

## macOS Developer ID and Notarization

### What Is Needed

- **Apple Developer Account** ($99/year)
- **Developer ID Application Certificate** (issued by Apple)
- **Notarization** via `notarytool` (Apple's automated notarization service)

### Implementation

1. Enroll in Apple Developer Program
2. Generate Developer ID Application certificate via Xcode or `certificates`
3. Add signing and notarization steps to CI:
   ```yaml
   - name: Sign macOS artifact
     run: |
       codesign --deep --force --verify --verbose --sign "Developer ID Application: ..." Orqestra.app
   - name: Notarize macOS artifact
     run: |
       xcrun notarytool submit Orqestra.dmg --apple-id "$APPLE_ID" --team-id "$TEAM_ID" --password "$APP_PASSWORD" --wait
       xcrun stapler staple Orqestra.dmg
   ```
4. Verify: `spctl --assess --type open --context context:primary-signature Orqestra.app`

### CI Secret Requirements

- `APPLE_ID` — Apple ID email
- `APPLE_TEAM_ID` — Developer team ID
- `APPLE_APP_PASSWORD` — app-specific password for notarytool

### Estimated Cost

- Apple Developer Program: $99/year
- Hardware: macOS runner available in GitHub Actions (free tier)

### Target Release

v1.1.0 or later (requires macOS bundler target configuration first).

## Linux Packaging and Checksums

### What Is Needed

- No signing infrastructure required for AppImage
- GPG signature for `.deb` packages (optional but recommended)
- SHA256 checksums (already implemented)

### Implementation

1. Continue generating SHA256 checksums in CI
2. Optionally add GPG signing for `.deb`:
   ```yaml
   - name: Sign deb package
     run: dpkg-sig -s builder -k "$GPG_KEY_ID" Orqestra_*.deb
   ```
3. Publish checksums alongside artifacts

### Estimated Cost

Free (GPG keys are self-generated)

### Target Release

v1.1.0 for verified Linux artifact.

## Required Accounts

| Account | Purpose | Cost |
|---------|---------|------|
| Apple Developer Program | macOS signing + notarization | $99/year |
| DigiCert or Sectigo | Windows code signing certificate | $200-800/year |
| GitHub Actions | CI/CD runners (macOS, Windows, Linux) | Free tier sufficient |

## Implementation Plan

| Phase | Task | Release Target |
|-------|------|---------------|
| 1 | Windows Standard Code Signing | v1.0.6 or v1.1.0 |
| 2 | macOS bundler targets + build | v1.1.0 |
| 3 | macOS Developer ID signing | v1.1.0 |
| 4 | macOS notarization | v1.1.0 |
| 5 | Linux GPG signing (optional) | v1.1.0 |

## v1.0.6 Signing Readiness Status

| Item | Status |
|------|--------|
| Windows certificate selected | no |
| Certificate purchased | no |
| CI secret names defined | yes — `WINDOWS_SIGNING_CERT`, `WINDOWS_SIGNING_PASSWORD` |
| Signing command documented | yes — `signtool sign /fd SHA256` |
| Verification command documented | yes — `signtool verify /pa` |
| macOS Apple Developer requirement documented | yes |
| Notarization workflow drafted | yes — `codesign` + `notarytool` |
| Blocker | Certificate procurement pending |

## Status

- [x] SHA256 checksums generated for all artifacts
- [x] Unsigned status documented in README, release notes, manifest
- [x] This plan linked from README
- [ ] Windows code signing certificate obtained
- [ ] macOS Developer ID certificate obtained
- [ ] Signing integrated into CI workflow
- [ ] Notarization integrated into CI workflow
