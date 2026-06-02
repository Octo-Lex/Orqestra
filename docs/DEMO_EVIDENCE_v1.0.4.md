# v1.0.4 Demo Evidence

- **Tag:** v1.0.4
- **Commit:** ef0c0e8e4d97459d5378e90470222d2e576d6b0d (v1.0.4 tag points here)
- **Merge commit:** 022b703 (master HEAD after merge)
- **Pre-release base:** b1a37f3 (artifact build base — not the tagged commit)
- **Artifact:** Orqestra_1.0.4_x64-setup.exe (NSIS installer)
- **Artifact SHA256:** `a55ba906db85f2f3650501211de32abcf691d0df2d622997bb367d7158542a80`
- **Dashboard URL:** https://orqestra.pages.dev
- **Dashboard status:** 200 OK
- **Demo mode:** real-AI maintainer
- **Result:** pass

---

## Mode A — No-Key Beta Demo

| Step | Status | Notes |
|------|--------|-------|
| Desktop artifact built | pass | Fresh build from v1.0.4 source |
| Release manifest generated | pass | SHA256 checksums for all artifacts |
| Dashboard live | pass | orqestra.pages.dev returns 200 OK |
| Diagnostics redaction | pass | All 11 secret patterns handled |

## Mode B — Real-AI Maintainer Demo

| Step | Status | Notes |
|------|--------|-------|
| AI service health | pass | `/health` returns `{"status":"ok"}` |
| AI service reads ZAI_API_KEY from .env | pass | python-dotenv loads key automatically |
| Extract intent (real AI) | pass | Confidence 0.95, real reasoning trace |
| Docs agent (real AI) | pass | Confidence 0.7, not mock 0.5 |
| Bugfix agent (real AI) | pass | Review-only enforced, propose-only mode |
| No autonomous commit | pass | Both agents return proposals only |

## Test Suite

| Suite | Result |
|-------|--------|
| Rust tests (141) | all pass |
| Desktop TS build | 331 KB JS, clean |
| Dashboard TS build | 205 KB JS, clean |
| HTTP endpoint tests | 7/7 pass |
| CDP UI tests | 10/10 pass |

## Platform Artifacts

| Platform | Status | SHA256 |
|----------|--------|--------|
| Windows x64 NSIS | tested | `a55ba906...` |
| Windows x64 binary | tested | `2fdd2780...` |
| macOS | not-built | — |
| Linux x64 | built-but-unverified (CI) | — |

## Notes

- All artifacts are unsigned beta builds
- AI service requires `ZAI_API_KEY` in `services/ai/.env` for real-AI mode
- Dashboard CI deployment uses explicit `accountId` to avoid Cloudflare error 9106
- Demo fixtures available at `demo/ai-fixtures/`
