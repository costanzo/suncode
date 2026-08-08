# Requirements

Each delivery has one dated directory named `YYYY-MM-DD-short-topic`.

## Standard files

- `README.md` — status and document index
- `requirement.md` — goals, scope, rules, and acceptance criteria
- `architecture.md` — technical design and boundaries
- `changes.md` — affected files and components
- `plan.md` — ordered implementation plan
- `progress.md` — current state and work log
- `todo.md` — checkable task list
- `test-plan.md` — verification scope and results

Copy `_template/` for new work. Keep only requirement records that still help explain current work or a decision trail. Once a delivery is stable, move durable facts into `features/`, `specs/`, or `DECISIONS.md` and trim the requirement record down to the essentials.
