# Safe Diff Context Pilot

## What Changed in v1.5.0

v1.5.0 introduces an **opt-in, strictly bounded** safe diff context pilot for review-only agents. When explicitly enabled, agents receive structured diff hunks from eligible normal-risk text files. The pilot is **disabled by default**.

## Default-Off Behavior

Safe diff context is disabled unless explicitly enabled. Agents continue to receive Agent Context v2 metadata (paths, statuses, risk summaries, commit groups, proposal summaries) without any diff content.

## How to Enable the Pilot

Set the environment variable:

```
ORQESTRA_SAFE_DIFF_CONTEXT=true
```

The `enabled_source` field in the payload records how it was enabled (`"env:ORQESTRA_SAFE_DIFF_CONTEXT"` or `"default-off"`).

The legacy `SEMANTIC_PREP_DIFF_BODY_ENABLED` env var is **not** used for Agent Context v2 safe diff context.

## Eligibility Rules

A file is eligible only when all conditions are met:

- Safe diff context is enabled
- `file_kind == "text"`
- `risk == "normal"`
- Status is `modified`, `staged`, `added`, or `renamed`
- File is within configured size/line caps

## Status Policy

| Status | Behavior |
|--------|----------|
| modified | Eligible if safe |
| staged | Eligible if safe |
| added | Eligible if safe |
| renamed | Eligible for new path; `original_path` preserved |
| deleted | Excluded (`unsupported-status`) |
| untracked | Excluded (`unsupported-status`) |

## Exclusion Reasons

Every excluded file has a recorded reason:

| Reason | Meaning |
|--------|---------|
| `disabled` | Pilot is not enabled |
| `secret-risk` | Secret-risk file |
| `binary` | Binary file |
| `large` | Large file |
| `symlink` | Symlink or unknown file kind |
| `workflow-risk` | Workflow-risk file (excluded by default) |
| `file-limit` | Max files cap reached |
| `non-text` | Non-text file kind |
| `unsupported-status` | Deleted or untracked |
| `read-error` | Git diff extraction failed |
| `absolute-path` | Absolute path rejected |

## Policy Caps

| Cap | Value |
|-----|-------|
| Max files | 5 |
| Max file size | 256 KiB |
| Max lines per hunk | 80 |
| Max lines per file | 120 |
| Max total lines | 250 |

Truncation is recorded with `truncated: true`.

## Agent Payload Shape

When disabled:

```json
{
  "safe_diff_context": {
    "enabled": false,
    "enabled_source": "default-off",
    "included": false,
    "provider": null,
    "policy": { ... },
    "files": [],
    "summary": { "included_files": 0, "excluded_files": 0, "total_lines": 0, "truncated": false }
  }
}
```

When enabled with eligible files:

```json
{
  "safe_diff_context": {
    "enabled": true,
    "enabled_source": "env:ORQESTRA_SAFE_DIFF_CONTEXT",
    "included": true,
    "provider": "git-cli-fallback",
    "policy": { ... },
    "files": [
      {
        "path": "src/lib.rs",
        "included": true,
        "exclusion_reason": null,
        "hunks": [{ "lines": ["-old", "+new"] }],
        "line_count": 12
      }
    ],
    "summary": { "included_files": 1, "excluded_files": 0, "total_lines": 12, "truncated": false }
  }
}
```

No field is named `diff`, `raw_diff`, or `patch`. The allowed field names are `safe_diff_context`, `hunks`, `lines`, `exclusion_reason`, and `truncated`.

## Security and Secret Safety

- Disabled by default — no surprise context expansion
- Secret-risk files are always excluded
- Binary, large, symlink, and absolute-path files are always excluded
- Workflow-risk files are excluded by default
- Diff extraction uses `git diff` — never reads file contents directly
- Provider is labeled `git-cli-fallback`
- Agents remain review-only

## What This Does Not Enable

- No auto-apply
- No auto-commit
- No staging
- No repository writes
- No push/pull
- No new agent roles
- No autonomous behavior

## Known Limitations

- Safe diff context is a pilot
- Provider is CLI-backed in v1.5.0
- No native diff-body provider is claimed
- No subjective AI quality improvement is claimed without separate evaluation

## Backlog

- Safe diff context stabilization (v1.5.1)
- Workflow-risk override option
- Native diff-body provider evaluation
- Tree-sitter/AST context exploration
