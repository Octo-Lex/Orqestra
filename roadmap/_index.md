---
pm-index: true
version: 1
sprints:
  - id: "Sprint 13"
    title: "Core Engine"
    tasks:
      - "TASK-2026-038"
      - "TASK-2026-040"
    start_date: "2026-05-15"
    end_date: "2026-05-31"
    status: "done"
  - id: "Sprint 14"
    title: "AI Intelligence & Semantic Git"
    tasks:
      - "TASK-2026-042"
      - "TASK-2026-045"
      - "TASK-2026-050"
    start_date: "2026-06-01"
    end_date: "2026-06-14"
    status: "done"
  - id: "Sprint 15"
    title: "Cloud Sync & Self-Hosting"
    tasks:
      - "TASK-2026-055"
      - "TASK-2026-056"
    start_date: "2026-06-15"
    end_date: "2026-06-28"
    status: "active"
  - id: "Sprint 16"
    title: "Operational Hardening"
    tasks:
      - "TASK-2026-060"
      - "TASK-2026-061"
      - "TASK-2026-062"
      - "TASK-2026-066"
      - "TASK-2026-067"
      - "TASK-2026-068"
      - "TASK-2026-069"
      - "TASK-2026-070"
    start_date: "2026-06-01"
    end_date: "2026-06-14"
    status: "active"
  - id: "Sprint 17"
    title: "Productization & Trust Hardening"
    tasks:
      - "TASK-2026-063"
      - "TASK-2026-064"
      - "TASK-2026-065"
      - "TASK-2026-071"
      - "TASK-2026-072"
      - "TASK-2026-073"
      - "TASK-2026-074"
      - "TASK-2026-075"
    start_date: "2026-06-01"
    end_date: "2026-06-14"
    status: "done"
  - id: "Sprint 18"
    title: "User-Ready Beta"
    tasks:
      - "TASK-2026-076"
      - "TASK-2026-077"
      - "TASK-2026-078"
      - "TASK-2026-079"
      - "TASK-2026-080"
      - "TASK-2026-081"
    start_date: "2026-06-02"
    end_date: "2026-06-15"
    status: "done"
epics:
  - id: "epic-core"
    title: "Core Engine"
    tasks:
      - "TASK-2026-038"
      - "TASK-2026-040"
      - "TASK-2026-042"
      - "TASK-2026-060"
    status: "done"
    theme: "v1.0 Foundation"
  - id: "epic-ai"
    title: "AI Intelligence"
    tasks:
      - "TASK-2026-045"
      - "TASK-2026-050"
      - "TASK-2026-065"
    status: "done"
    theme: "v1.0 Foundation"
  - id: "epic-cloud"
    title: "Cloud & Sync"
    tasks:
      - "TASK-2026-055"
      - "TASK-2026-056"
    status: "in-progress"
    theme: "v1.0 Foundation"
  - id: "epic-dx"
    title: "DX & Polish"
    tasks:
      - "TASK-2026-061"
      - "TASK-2026-062"
    status: "upcoming"
    theme: "v1.1 Hardening"
  - id: "epic-security"
    title: "Security & CI"
    tasks:
      - "TASK-2026-063"
      - "TASK-2026-064"
    status: "upcoming"
    theme: "v1.1 Hardening"
team:
  - id: "alice"
    role: "tech-lead"
  - id: "bob"
    role: "backend"
  - id: "charlie"
    role: "infra"
---

# Orqestra Project Tracker

This directory is the **single source of truth** for all Orqestra planning.
Every task, sprint, and epic is defined here as a markdown file with YAML frontmatter.

## Status Legend

| Status | Meaning |
|--------|---------|
| `backlog` | Known but not yet scheduled |
| `ready` | Scheduled, waiting to start |
| `in-progress` | Actively being worked on |
| `in-review` | Implementation done, under review |
| `done` | Completed and merged |
| `cancelled` | Descoped or superseded |

## How to Add a Task

1. Create `roadmap/TASK-2026-NNN.md` using the schema in §3.2 of the spec
2. Add the task ID to the relevant sprint and epic in this file
3. Commit with a semantic message referencing the task ID

## How Agents Work

When a GitHub Issue is created, the `orqestra-agents.yml` workflow:
1. Parses the issue title/body for task references
2. Routes to the appropriate agent workspace
3. Agent reads skill definitions, executes, commits with semantic metadata
4. ConfidenceGate determines auto-commit vs. propose vs. flag
