# Live Site Smoke Checklist — orqestra.pages.dev

Run after every dashboard deployment.

## Pre-deployment

- [ ] CI passes: secret-scan, dashboard build+test+bundle scan, DCO
- [ ] Rust tests pass (pre-existing flake `missing_ai_service_returns_error` acceptable)
- [ ] Dashboard tests pass

## Post-deployment — Evidence Tab

- [ ] Evidence tab visible without token
- [ ] All six panels render (Release History, Test Count, Security, Autonomy, Runtime, Freshness)
- [ ] Static Export badge visible on DataFreshnessPanel
- [ ] Runtime evidence shows "Not external beta data" badge
- [ ] Runtime evidence type is "structural-runtime-decision-matrix"
- [ ] Autonomy max cap shows 10
- [ ] Autonomy auto_commit shows "Always False"
- [ ] Evidence provenance lines visible on panels

## Post-deployment — Security

- [ ] Dashboard has no write/admin copy anywhere
- [ ] Former token literal absent from public output (full dist/ scan)
- [ ] Token gate says "Connect Private View" only
- [ ] Only `ork_read_*` tokens accepted
- [ ] Auth badge shows "Private View" only

## Post-deployment — Layout

- [ ] Kanban, Gantt, Table, Evidence views all switch correctly
- [ ] Mobile viewport renders without horizontal breakage
- [ ] Evidence panels stack cleanly under narrow width
- [ ] View switcher has accessible tab roles

## Post-deployment — Data

- [ ] `orqestra-roadmap.json` loads and contains tasks
- [ ] Source commit matches expected HEAD
- [ ] Generation timestamp is recent
