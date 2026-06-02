# Diagnostics Guide

## Exporting Diagnostics

1. Open the **Diagnostics** panel from the toolbar
2. Click **Export Diagnostics**
3. A bundle is created in `.Orqestra/orqestra-diagnostics-<timestamp>/`

## What's Included

| File | Content |
|------|---------|
| `app.json` | App version, platform, git SHA |
| `readiness.json` | Full environment readiness report |
| `project-validation.json` | Project structure validation result |
| `recent-errors.json` | Recent command errors |
| `system.txt` | OS, architecture, timestamps |
| `ai-health.json` | AI service health status |
| `dashboard-status.json` | Dashboard deployment status |
| `README.txt` | Bundle overview and redaction summary |

## What's NOT Included

- Raw GitHub PATs
- Raw API keys
- OS keychain records
- Full repository source
- Full environment variable dump
- `.Orqestra/agents/` local workspace state

## Secret Redaction

All known secret patterns are automatically redacted:

| Pattern | Redaction |
|---------|-----------|
| `ghp_...` | `[REDACTED:TOKEN]` |
| `sk-...` | `[REDACTED:TOKEN]` |
| `Bearer ...` | `[REDACTED:BEARER]` |
| `ZAI_API_KEY=...` | `ZAI_API_KEY=[REDACTED:ENV_VAR]` |
| `password: ...` | `[REDACTED]` |

## Common Error Recovery

| Error | Recovery |
|-------|----------|
| `ROADMAP_NOT_FOUND` | Open a folder with a `roadmap/` directory |
| `AI_SERVICE_UNREACHABLE` | Start the local AI service |
| `AI_KEY_MISSING` | Set `ZAI_API_KEY` environment variable |
| `GITHUB_TOKEN_MISSING` | Save a PAT in Settings |
| `KEYRING_UNAVAILABLE` | Check OS credential manager access |
| `DASHBOARD_JSON_MISSING` | Generate dashboard JSON from roadmap |
| `DUPLICATE_TASK_ID` | Rename tasks with duplicate IDs |

## Sharing Diagnostics

Before sharing a diagnostic bundle:
1. Open the bundle directory
2. Review `README.txt` for the redaction summary
3. Verify `contains_raw_secrets: false`
4. Do NOT manually add secrets to the bundle
