# Setup Checks Reference

## Environment Checks

Orqestra checks these items when you open the **Setup** panel:

### Local Tools

| Tool | Required For | Required? |
|------|-------------|-----------|
| Git | Git sync, semantic commits | Yes for sync, no for local PM |
| Node.js | Dashboard build | Developer only |
| npm | Desktop build | Developer only |
| Rust | Core crate development | Developer only |
| Python | AI service | AI features only |

### AI Service

| Check | Meaning |
|-------|---------|
| Service reachable | `http://localhost:8000/health` returns OK |
| API key configured | `ZAI_API_KEY` environment variable is set |
| Mode: real | Both service and key are available |
| Mode: degraded_mock | Service up but no key — fallback responses |
| Mode: unavailable | Service not running |

### Credentials

| Check | Meaning |
|-------|---------|
| GitHub token: stored | PAT saved in OS keychain |
| GitHub token: missing | No PAT saved — push/pull unavailable |
| Provider: keyring | Using OS keychain (recommended) |
| Provider: none | Keyring unavailable |

### Dashboard

| Check | Meaning |
|-------|---------|
| Local JSON: present | Roadmap JSON generated locally |
| Live URL: ok | Dashboard accessible at orqestra.pages.dev |
| Cloudflare secrets: unknown | Cannot verify remote secrets |

## Fixing Issues

### AI Service Unreachable
```bash
cd services/ai
uv run uvicorn orqestra_ai.main:app --port 8000
```

### API Key Missing
Set the `ZAI_API_KEY` environment variable before launching Orqestra.

### GitHub Token Missing
Open Settings in the app and enter your PAT.

### Keyring Unavailable
On Windows: ensure Windows Credential Manager is accessible.
On Linux: install `libsecret` libraries.
On macOS: Keychain should be available by default.

### Cloudflare Secrets Unknown
Add to GitHub repository settings → Secrets and variables → Actions:
- `CLOUDFLARE_API_TOKEN`
- `CLOUDFLARE_ACCOUNT_ID`
