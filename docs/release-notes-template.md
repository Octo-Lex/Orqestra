# Orqestra v1.0.7 — Signed Windows Beta / Platform Confidence

## Summary

v1.0.7 does not yet include a signed Windows installer. It adds explicit signing-blocker evidence, signature verification documentation, installer diagnostics, and platform confidence criteria so reviewers can understand and verify the current Windows beta distribution state.

## Download

Download `Orqestra_1.0.7_x64-setup.exe` from the assets below.

## Verify SHA256

```powershell
Get-FileHash .\Orqestra_1.0.7_x64-setup.exe -Algorithm SHA256
```

Compare against `checksums.txt` or `release-manifest.json`.

## Verify Signature

```powershell
Get-AuthenticodeSignature .\Orqestra_1.0.7_x64-setup.exe
```

Expected: `Status: NotSigned` — the installer is unsigned. See `demo/v1.0.7-signature-verification.md` for full verification evidence.

## Windows SmartScreen

The installer is unsigned. Windows SmartScreen warnings are expected. Even when signing is implemented, SmartScreen may still warn until reputation is established.

## Platform Status

| Platform | Status |
|----------|--------|
| Windows x64 | tested (unsigned) |
| macOS | not-built |
| Linux x64 | built-but-unverified |

See [Platform Confidence](https://github.com/Elephant-Rock-Lab/Orqestra/blob/master/docs/platform-confidence.md) for what each status means.

## Signing Status

**Unsigned.** Blocker: certificate not available. Next action: purchase or configure Windows code-signing. See [Signing Plan](https://github.com/Elephant-Rock-Lab/Orqestra/blob/master/docs/release-signing-plan.md).

## Installer Diagnostics

See [Installer Diagnostics](https://github.com/Elephant-Rock-Lab/Orqestra/blob/master/docs/installer-diagnostics.md) for SHA256, signature, WebView, log, and AI service diagnostic steps.

## Troubleshooting

See [Troubleshooting Guide](https://github.com/Elephant-Rock-Lab/Orqestra/blob/master/docs/troubleshooting.md) — now includes unsigned and signed-but-low-reputation SmartScreen guidance.

## Report an Issue

- [Install issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=install_issue.yml)
- [AI mode issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=ai_mode_issue.yml)
- [Dashboard issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=dashboard_issue.yml)
- [Bug report](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=bug_report.yml)

**Do not paste API keys or secrets in issues.**

## Known Limitations

- Unsigned installer (SmartScreen warnings expected)
- macOS artifacts not available
- Linux not verified for public beta
- Architect agent is mock-mode
- ML-Master is stub
- Edge relay / CRDT sync not available
- Code signing blocked (certificate not available)

## Release Provenance

See `release-manifest.json` for full provenance including signing blocker, diagnostics, platform evidence, and dashboard freshness.

## Checksums

See `checksums.txt` in the release assets.

## Demo Evidence

- [Demo evidence](https://github.com/Elephant-Rock-Lab/Orqestra/blob/master/demo/v1.0.7-demo-evidence.md)
- [Windows smoke test](https://github.com/Elephant-Rock-Lab/Orqestra/blob/master/demo/v1.0.7-windows-smoke.md)
- [Signature verification](https://github.com/Elephant-Rock-Lab/Orqestra/blob/master/demo/v1.0.7-signature-verification.md)
