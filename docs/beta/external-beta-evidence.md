# External Beta Evidence

**Version:** v2.12.0
**Purpose:** What beta evidence is, how it is captured, and what is never included.

---

## What Beta Evidence Is

Beta evidence is a structured, redacted, consented bundle that captures whether an external beta user completed the self-serve beta path successfully.

It is **not** telemetry. It is **not** uploaded automatically. It is created only when the user explicitly chooses to export it.

---

## What Gets Included

| Category | Included | Example |
|----------|----------|---------|
| Session outcome | ✓ | Steps completed, warnings, blocked features |
| Repo metadata (hashed) | ✓ | `sha256:abc123...` for project path |
| Git status | ✓ | Branch, dirty/clean, remote configured |
| Platform info | ✓ | OS, architecture |
| Failure taxonomy | ✓ | Structured codes: `AI_SERVICE_UNAVAILABLE`, `CONSENT_DECLINED` |
| User feedback (if provided) | ✓ | Ratings, optional free text (redacted) |
| Share permissions | ✓ | Aggregate vs. quote use (separate) |

---

## What Never Gets Included

| Category | Rule |
|----------|------|
| Tokens / PATs / API keys | Pattern-based redaction removes all known formats |
| Raw repo paths | SHA-256 hashed |
| Raw user home paths | Never included |
| Full file contents | Excluded entirely |
| Bearer tokens | Pattern-based redaction |
| Remote URLs with credentials | Hashed or stripped |
| Secret/password/key values | Pattern-based redaction |

---

## How to Export

1. Open Orqestra desktop app
2. Complete the self-serve beta path (see [demo-scenario.md](demo-scenario.md))
3. Navigate to **Diagnostics** or **Readiness** panel
4. Click **"Export Beta Evidence"**
5. Review the consent dialog — it shows exactly what will be included and excluded
6. Click **Export** to create a local bundle
7. The bundle is saved to `.Orqestra/beta-evidence-<timestamp>/`
8. **No automatic upload occurs**

---

## How to Inspect Before Sharing

1. Open the `.Orqestra/beta-evidence-<timestamp>/` directory
2. Read each JSON file — they are human-readable
3. Verify no secrets appear (redaction runs automatically, but verify)
4. Do **not** share if you see anything sensitive

---

## How to Submit

Beta evidence is submitted manually:

1. Export the bundle locally
2. Review the files
3. Attach the bundle to a GitHub issue or send via the beta feedback channel
4. Do **not** paste contents directly into public issues

---

## Bundle Contents

```
beta-evidence-<timestamp>/
├── beta-evidence-manifest.json      # Bundle metadata, consent record
├── beta-session-outcome.json        # Steps completed, outcome, warnings
├── beta-feedback.json               # User ratings and optional text
└── beta-failure-taxonomy.json       # Any failures encountered
```

---

## Failure Codes

| Code | Severity | Category |
|------|----------|----------|
| INSTALL_BLOCKED | blocking | install |
| SMARTSCREEN_WARNING | warning | install |
| APP_LAUNCH_FAILED | critical | app_launch |
| REPO_OPEN_FAILED | blocking | repo |
| ROADMAP_NOT_FOUND | warning | roadmap |
| GIT_UNAVAILABLE | warning | git |
| KEYCHAIN_UNAVAILABLE | warning | credential |
| AI_SERVICE_UNAVAILABLE | warning | ai_service |
| AGENT_FLOW_FAILED | warning | agent |
| DIFF_REVIEW_FAILED | warning | diff_review |
| DASHBOARD_EXPORT_FAILED | warning | dashboard |
| EVIDENCE_SCHEMA_INVALID | warning | evidence |
| DIAGNOSTICS_EXPORT_FAILED | warning | diagnostics |
| USER_ABANDONED | info | user_action |
| UNKNOWN_FAILURE | warning | unknown |
| CONSENT_DECLINED | info | consent |

---

## Privacy

- Free text feedback is redacted for secret patterns before writing
- Quote use requires separate permission from aggregate use
- No session replay
- No screen capture
- No network capture
- No file content capture

---

## Public Dashboard Display

The public evidence dashboard shows:

```
External Beta Evidence
Status: none
Collection mode: local export only
Consent required: yes
Automatic upload: no
Public data: aggregate only
```

This will show "none" until real, curated external beta evidence exists. Internal test data and fixtures do not change this status.
