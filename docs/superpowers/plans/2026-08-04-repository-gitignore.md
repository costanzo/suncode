# Repository Gitignore Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a root `.gitignore` that excludes local and generated machine artifacts without hiding reproducibility inputs or generated protocol source.

**Architecture:** Use one categorized root ignore file for the polyglot monorepo. Keep Cargo and pnpm lockfiles, example environment files, documentation, contracts, and `generated/` source trackable.

**Tech Stack:** Git ignore patterns, Rust/Cargo, Node.js/pnpm, Electron, SQLite

---

### Task 1: Add and verify the repository ignore policy

**Files:**
- Create: `.gitignore`

- [ ] **Step 1: Confirm representative artifacts are not ignored yet**

Run:

```powershell
git check-ignore --no-index .superpowers/session/state target/debug/app.exe node_modules/pkg/index.js local.sqlite generated/rust/types.rs Cargo.lock pnpm-lock.yaml .env.example
```

Expected: no output and exit status 1 because no root ignore policy exists yet.

- [ ] **Step 2: Add the categorized root ignore policy**

Create `.gitignore` with explicit sections for local agent state, operating-system and editor metadata, logs, secret environment files, dependencies, caches, build output, coverage, Rust output, SQLite runtime files, and temporary files. Do not ignore `generated/` or lockfiles. Re-include `.env.example`.

- [ ] **Step 3: Verify ignored artifacts**

Run:

```powershell
git check-ignore --no-index .superpowers/session/state target/debug/app.exe node_modules/pkg/index.js local.sqlite .env
```

Expected: all five paths are printed and the command exits successfully.

- [ ] **Step 4: Verify tracked inputs remain visible**

Run:

```powershell
$paths = 'generated/rust/types.rs', 'Cargo.lock', 'pnpm-lock.yaml', '.env.example'
foreach ($path in $paths) {
  git check-ignore --no-index $path
  if ($LASTEXITCODE -eq 0) { throw "$path is unexpectedly ignored" }
}
```

Expected: no output and exit status 0 from the wrapper because none of the paths is ignored.

- [ ] **Step 5: Validate syntax and repository status**

Run:

```powershell
git diff --check
git status --short
```

Expected: `git diff --check` exits successfully; `.superpowers/` is absent from status; `.gitignore` and this plan are the only new files.
