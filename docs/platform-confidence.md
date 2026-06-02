# Platform Confidence

This document explains what each platform status means, what CI evidence exists, and why only Windows x64 is promoted as a tested public beta platform.

---

## Current Public Beta Platform

**Windows x64** is the only tested platform for Orqestra public beta.

Linux x64 has proven CI compilation. macOS has proven CI compilation. Neither has a bundled artifact or smoke evidence.

---

## What "Tested" Means

A platform is marked **tested** when:

1. A bundled installer or artifact exists for that platform
2. A SHA256 checksum is generated and published
3. A smoke test passes on that platform
4. The release manifest, README, and release notes all agree
5. Smoke evidence is recorded in `demo/`

Windows x64 meets all five criteria.

---

## What "Built but Unverified" Means

- The platform compiles successfully in CI
- A raw binary is produced
- **No standard installer artifact** (AppImage, DEB, etc.) is produced
- No smoke test has been performed
- The platform is **not recommended** for public beta users

Compile success alone does not qualify a platform as tested.

---

## What "Build Feasibility Verified" Means

- The platform compiles successfully in CI
- The build target is known (e.g., `universal-apple-darwin`)
- **No bundled artifact** is produced (DMG, app bundle, etc.)
- No smoke test, signing, or notarization has been performed
- The platform is **not a public beta platform**

This status confirms that the codebase can compile for the platform, which is useful for future planning.

---

## Windows x64

| Property | Value |
|----------|-------|
| Status | tested |
| Artifact | NSIS installer |
| Signed | No |
| SmartScreen | Warnings expected (unsigned) |
| Smoke evidence | `demo/v1.0.8-windows-smoke.md` |
| Checksums | `checksums.txt` in release assets |
| Release blocking | yes |

### Windows Signing Status

The Windows installer is currently **unsigned**. No code-signing certificate or managed signing service has been configured.

**Blocker:** Certificate procurement pending.

**Next action:** Purchase or configure Windows code-signing, then integrate into CI.

See [Signing Plan](release-signing-plan.md) for full details.

### SmartScreen Expectations

- **Unsigned installer:** Windows SmartScreen will warn. This is expected.
- **Signed installer (future):** SmartScreen may still warn until reputation is established.

See [Troubleshooting](troubleshooting.md) for detailed guidance.

---

## Linux x64

| Property | Value |
|----------|-------|
| Status | built-but-unverified |
| Compile status | pass (CI Run #26847116112) |
| Bundle status | not-configured |
| Artifact | None (raw binary only) |
| Signed | No |
| Smoke evidence | None |
| Release blocking | no |
| Evidence | `demo/v1.0.8-linux-build-evidence.md` |

Linux compiles successfully in CI (141 Rust tests pass, Tauri build completes), but **no AppImage, DEB, or RPM bundle is produced** because the Tauri bundler targets are not configured for Linux.

The CI smoke check confirms: "Binary found (bundling may not be configured for this platform)."

Linux is **not recommended** for public beta users.

### To Promote Linux to "Tested"

1. Configure Tauri bundler targets in `tauri.conf.json` (e.g., `"deb"`, `"appimage"`)
2. Produce a standard installer artifact in CI
3. Compute and publish SHA256 checksum
4. Run smoke test on a Linux desktop environment
5. Record smoke evidence in `demo/`
6. Update manifest, README, and release notes

---

## macOS

| Property | Value |
|----------|-------|
| Status | build-feasibility-verified |
| Compile status | pass (CI Run #26847116112) |
| Bundle status | not-configured |
| CI target | universal-apple-darwin |
| CI runner | macos-latest |
| Artifact | None (raw binary only) |
| Signed | No |
| Notarized | No |
| Smoke evidence | None |
| Release blocking | no |
| Evidence | `demo/v1.0.8-macos-feasibility.md` |

macOS compiles successfully in CI for the `universal-apple-darwin` target (141 Rust tests pass, Tauri build completes), but **no DMG or app bundle is produced** because the Tauri bundler targets are not configured for macOS.

macOS is **not a public beta platform**.

### To Promote macOS

1. Configure Tauri bundler for macOS in `tauri.conf.json` (e.g., `"dmg"`, `"app"`)
2. Produce a standard `.dmg` or `.app.tar.gz` artifact
3. Code sign with Apple Developer ID certificate
4. Notarize via Apple notary service
5. Compute and publish SHA256 checksum
6. Run smoke test on a macOS desktop environment
7. Record smoke evidence in `demo/`
8. Update manifest, README, and release notes

---

## What Is Release-Blocking

A platform is **release-blocking** when its failure prevents the release from shipping.

| Platform | Release Blocking | Reason |
|----------|-----------------|--------|
| Windows x64 | **yes** | Primary tested public beta platform |
| Linux x64 | no | Not promoted, no bundled artifact |
| macOS | no | Not promoted, no bundled artifact |

---

## Promotion Criteria

A platform cannot be marked "tested" unless all five criteria are met:

1. **Bundled artifact exists** -- installer, DMG, AppImage, or package for that platform
2. **Checksum published** -- SHA256 in manifest and checksums.txt
3. **Smoke test passes** -- documented in demo/ with pass result
4. **Documents agree** -- manifest, README, release notes all say the same thing
5. **Signing state is explicit** -- signed or unsigned, with evidence

Additionally:
- compile_status must be pass
- bundle_status must be configured (not "not-configured")
- smoke_tested must be true

---

## Known Platform Limitations

- Windows installer is unsigned -- SmartScreen warnings are expected
- Linux binary compiles but no AppImage/DEB bundle is produced
- macOS binary compiles but no DMG/app bundle is produced
- No platform has code-signing or notarization configured
- Platform support is beta-grade and evidence-limited

---

## Current v1.0.8 Platform Matrix

| Platform | Status | Compile | Bundle | Artifact | Signed | Smoke | Blocking |
|----------|--------|---------|--------|----------|--------|-------|----------|
| Windows x64 | tested | pass | NSIS installer | Yes | No | Yes | yes |
| Linux x64 | built-but-unverified | pass | not configured | No | N/A | No | no |
| macOS | build-feasibility-verified | pass | not configured | No | No | No | no |
