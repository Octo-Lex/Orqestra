# Public Beta Issue Triage

This document describes how incoming public beta issues are managed.

---

## Labels

| Label | Purpose |
|-------|---------|
| `beta` | Public beta feedback |
| `install` | Installation or launch failures |
| `windows` | Windows-specific issues |
| `ai-mode` | AI agent or AI service issues |
| `dashboard` | Dashboard (orqestra.pages.dev) issues |
| `bug` | General bugs |
| `docs` | Documentation issues |
| `security-sensitive` | Reports involving credentials, tokens, or security concerns |
| `needs-repro` | Needs steps to reproduce |
| `needs-logs` | Needs terminal output or log files |
| `platform-linux` | Linux-specific issues |
| `platform-macos` | macOS-specific issues |
| `platform-windows` | Windows-specific issues |
| `release-blocker` | Blocks a release |

## Severity Levels

| Level | Meaning | Example |
|-------|---------|---------|
| Critical | App does not launch or data loss | Crash on startup, corrupted roadmap |
| High | Core feature broken | Cannot open repository, dashboard down |
| Medium | Feature degraded | AI service unreachable, slow rendering |
| Low | Cosmetic or minor | UI glitch, documentation typo |

## Install Issues

1. Apply `beta`, `install`, `platform-windows` labels
2. Verify the reporter included: Windows version, installer SHA256, SmartScreen behavior, install result, launch result
3. If SHA256 does not match, ask them to re-download
4. If SmartScreen blocked, refer to [Troubleshooting](troubleshooting.md)
5. If launch fails, ask for terminal output from running the exe directly
6. See [Installer Diagnostics](installer-diagnostics.md) for full diagnostic steps

## Security-Sensitive Reports

1. Apply `security-sensitive` label
2. **Immediately request the reporter to redact or delete any API keys, tokens, or secrets from the issue**
3. Do not request or accept `.env` files, certificate material, or credential files
4. If credentials were exposed, advise the reporter to rotate them immediately
5. Issues involving credential vault behavior should reference the diagnostics redaction system, not raw credential values

## AI Mode Issues

1. Apply `beta`, `ai-mode` labels
2. Verify the reporter specified: mode (no-key/real-AI), ZAI_API_KEY status, health check result, endpoint affected
3. If `ZAI_API_KEY` not set, explain no-key beta mode (expected degradation)
4. If health check fails, ask them to restart the AI service
5. If real-AI mode returns unexpected results, ask for the endpoint and response

## Dashboard Issues

1. Apply `beta`, `dashboard` labels
2. Verify the reporter included: timestamp, expected release, actual release, browser
3. Check if the dashboard is currently live: `curl -s https://orqestra.pages.dev/`
4. If stale, check if the latest CI dashboard deploy succeeded
5. Hard-refresh may resolve cached versions

## Platform Issues

1. Apply `platform-linux` or `platform-macos` as appropriate
2. Orqestra is currently a **Windows-only beta** — Linux and macOS are not supported
3. Acknowledge the report but explain the current platform status
4. Link to [Platform Confidence](platform-confidence.md)

## Response Policy

- Acknowledge new issues within 48 hours (business days)
- Request missing information promptly
- Apply appropriate labels and severity
- Close duplicates with reference to the original

## Closing Policy

- Close when fixed and verified
- Close as "not supported" if the issue is on an unsupported platform, with explanation
- Close as "needs more info" if the reporter has not responded in 7 days
- Never close security-sensitive reports without confirming the reporter has rotated any exposed credentials

## Information We Will Ask For

Depending on the issue type:

- Orqestra version (from Setup panel)
- Windows version
- Installer filename and SHA256
- SmartScreen behavior
- Install result (success/failure)
- Launch result (success/failure/crash)
- Terminal output
- Screenshots of error dialogs
- AI mode (no-key/real-AI)
- AI service health check result
- Dashboard timestamp and browser

## Information Users Must Not Share

- ZAI API keys
- GitHub Personal Access Tokens
- `.env` files
- Certificate material
- Password files
- Any string containing `ghp_`, `sk-`, `Bearer`, `secret:`, `token:`
