# Platform Confidence

This document explains what each platform status means, what evidence exists, and why each platform has its current classification.

---

## Current Platform Matrix (v1.0.12)

| Platform | Status | Compile | Bundle | Artifact | Signed | Smoke | Blocking |
|----------|--------|---------|--------|----------|--------|-------|----------|
| Windows x64 | tested | pass | NSIS installer | Yes | No | Yes (15/15) | yes |
| Linux x64 | tested | pass | AppImage | Yes | N/A | Yes (9/9) | no |
| macOS | build-feasibility-verified | pass | not configured | No | No | No | no |

---

## What "Tested" Means

A platform is marked **tested** when:

1. A bundled installer or artifact exists for that platform
2. A SHA256 checksum is generated and published
3. A smoke test passes on that platform
4. The release manifest, README, and release notes all agree
5. Smoke evidence is recorded in `demo/`

Both Windows x64 and Linux x64 meet all five criteria.

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
| Smoke evidence | `demo/v1.0.12-windows-smoke.md` |
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
| Status | tested |
| Compile status | pass (CI Run #26878403707) |
| Bundle status | pass (AppImage produced) |
| Artifact | Orqestra_1.0.12_x64.AppImage |
| SHA256 | `839ee8a629b33c82ea39a35dbd6d69e92e2363ae988c2fda0343cc5860900b05` |
| Signed | Not applicable |
| Smoke evidence | `demo/v1.0.12-linux-native-smoke.md` |
| Release blocking | no |

### Linux Smoke Verification

Linux was promoted to `tested` in v1.0.12 after native Ubuntu 24.04 GNOME smoke verification on a QEMU VM (Proxmox 8.4.10).

**Runtime environment:**
- Ubuntu 24.04.4 LTS
- GNOME 46 (Wayland + Xwayland rootless)
- WebKit2GTK 2.52.3-0ubuntu0.24.04.1
- GTK 3.24.41-4ubuntu1.3
- FUSE 3.14.0-5build1
- QEMU VM, virtio-gpu (software rendering)

**Smoke result:** 9/9 steps pass with one documented caveat:
- Dashboard link open: deferred because the VM had no browser installed
- Dashboard availability/version: independently verified 200 OK

**Screenshot note:** Screenshot capture was blocked by the Wayland rootless compositor (XGetImage not supported). Process and window evidence (`xwininfo`) substitutes for visual screenshot evidence.

### Linux Maturity Progression

| Release | Status | What changed |
|---------|--------|--------------|
| v1.0.8 | compiled-binary-only | CI compile evidence |
| v1.0.9 | bundle-produced-unverified | AppImage + SHA256 |
| v1.0.10 | runtime-blocked | GTK init fails without display server |
| v1.0.11 | runtime-evidence-wslg | App runs under WSLg (not promoted) |
| v1.0.12 | **tested** | Native Ubuntu 24.04 GNOME smoke pass |

### Contributor Smoke Kit

v1.0.12 also published a contributor smoke kit for community testing on additional distros:

- **Guide:** `docs/linux-native-smoke-guide.md`
- **Evidence template:** `docs/linux-smoke-evidence-template.md`
- **Issue form:** `.github/ISSUE_TEMPLATE/linux_smoke_report.yml`

Contributor reports on other distros (Fedora, Debian, Mint, etc.) are welcome and will expand the recorded Linux support matrix.

---

## macOS

| Property | Value |
|----------|-------|
| Status | build-feasibility-verified |
| Compile status | pass (CI Run #26878403707) |
| Bundle status | not configured |
| CI target | universal-apple-darwin |
| CI runner | macos-latest |
| Artifact | None |
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
| Linux x64 | no | Tested but not primary |
| macOS | no | Not promoted |

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
- Linux screenshot blocked by Wayland rootless compositor; process+window evidence recorded instead
- Linux smoke tested on Ubuntu 24.04 only; other distros welcome via contributor reports
- macOS binary compiles but no DMG/app bundle is produced
- No platform has code-signing or notarization configured
- Platform support is beta-grade and evidence-limited
