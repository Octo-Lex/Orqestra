# Platform Confidence

This document explains what each platform status means and why only Windows x64 is promoted as a tested public beta platform.

---

## Current Public Beta Platform

**Windows x64** is the only tested platform for Orqestra public beta.

---

## What "Tested" Means

A platform is marked **tested** when:

1. An installer or artifact exists for that platform
2. A SHA256 checksum is generated and published
3. A smoke test passes on that platform
4. The release manifest, README, and release notes all agree
5. Smoke evidence is recorded in `demo/`

Windows x64 meets all five criteria.

---

## Windows x64

| Property | Value |
|----------|-------|
| Status | tested |
| Artifact | NSIS installer |
| Signed | No |
| SmartScreen | Warnings expected (unsigned) |
| Smoke evidence | `demo/v1.0.7-windows-smoke.md` |
| Checksums | `checksums.txt` in release assets |

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
| Artifact | AppImage/DEB (CI builds) |
| Signed | No |
| Smoke evidence | None |
| Promotion criteria | Not met |

Linux artifacts are built in CI but not locally verified. Linux is **not recommended** for public beta users. No smoke test has been performed on a Linux system.

To promote Linux to "tested," the following is needed:
1. Smoke test on a Linux distribution
2. Artifact checksum recorded
3. Smoke evidence file created
4. Manifest and README updated

---

## macOS arm64 / x64

| Property | Value |
|----------|-------|
| Status | not-built |
| Artifact | None |
| Signed | No |
| Notarized | No |
| Smoke evidence | None |

macOS artifacts are not built for this release. macOS requires:
1. Apple Developer Program enrollment
2. Developer ID Application certificate
3. Bundler target configuration in Tauri
4. CI runner with macOS support

No timeline is committed for macOS support.

---

## Promotion Criteria

A platform cannot be marked "tested" unless all five criteria are met:

1. **Artifact exists** — installer, binary, or package for that platform
2. **Checksum published** — SHA256 in manifest and checksums.txt
3. **Smoke test passes** — documented in demo/ with pass result
4. **Documents agree** — manifest, README, release notes all say the same thing
5. **Signing state is explicit** — signed or unsigned, with evidence

---

## Current v1.0.7 Platform Matrix

| Platform | Status | Artifact | Signed | Smoke |
|----------|--------|----------|--------|-------|
| Windows x64 | tested | Orqestra_1.0.7_x64-setup.exe | No | Yes |
| macOS arm64 | not-built | None | No | No |
| macOS x64 | not-built | None | No | No |
| Linux x64 | built-but-unverified | CI only | No | No |
