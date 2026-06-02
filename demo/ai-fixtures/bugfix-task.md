# Demo Task: Fix Off-by-One in Gantt View

## Task
The GanttView component renders task bars one day later than their actual start date. Investigate and propose a fix.

## Scope
- `apps/desktop/src/pm/gantt-canvas.ts` — canvas rendering logic
- `apps/desktop/src/pm/GanttView.tsx` — date-to-pixel mapping

## Expected Behavior
A task with `start: "2026-06-01"` should render starting at the June 1 column, not June 2.

## Constraints
- Propose-only mode — do not auto-commit
- Minimal diff preferred
