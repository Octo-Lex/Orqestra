# v1.0.4 Demo Evidence

- **Tag:** v1.0.4
- **Commit:** b1a37f3d445cd27d11d84cc4af378a9d9cbae526
- **Artifact:** Orqestra_0.1.0_x64-setup.exe (NSIS installer)
- **Artifact SHA256:** `780f04c7f2154d2d442eab8dcdb45926a33d6b01344b55488117c9257be57e15`
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
| Windows x64 NSIS | tested | `780f04c7...` |
| Windows x64 binary | tested | `efb2bb76...` |
| macOS | not-built | — |
| Linux x64 | built-but-unverified (CI) | — |

## Notes

- All artifacts are unsigned beta builds
- AI service requires `ZAI_API_KEY` in `services/ai/.env` for real-AI mode
- Dashboard CI deployment uses explicit `accountId` to avoid Cloudflare error 9106
- Demo fixtures available at `demo/ai-fixtures/`
