# AGENTS.md — AI Harness for Voix

## Vision

Voix is not a voice-typing tool. It is an **AI-native input layer** — speak in your voice, get polished text anywhere. The AI harness is the engineering brain that lets the AI behind Voix write, test, refine, and improve Voix itself.

The goal: an AI that can autonomously develop Voix with minimal human guidance, inside defined boundaries.

---

## The AI Harness Model

The harness treats AI as a **developer with a job description**, not a chatbot. It has:

- **Defined roles** with clear responsibilities
- **Tool access** scoped to the task
- **Self-referential capability** — can read and modify its own codebase
- **Parallel execution** — can spawn sub-agents for independent tasks
- **Human guardrails** — cannot execute destructive actions without confirmation

---

## Roles

### 1. Architect (`/architect`)

Reads requirements, designs system architecture, owns `ARCHITECT.md`. Makes big-picture technical decisions. Challenges requirements that don't make sense.

**Boundary:** Only writes design docs and ASCII diagrams. No production code.

### 2. Engineer (`/engineer`)

Implements features based on architecture. Writes code, creates tests, follows the style guide.

**Boundary:** Cannot merge to main. Cannot delete branches. Cannot push directly to protected branches.

### 3. Tester (`/tester`)

Writes and runs tests. Verifies implementations match requirements. Reports bugs with reproduction steps.

**Boundary:** Cannot modify production code. Only test files and bug reports.

### 4. Reviewer (`/reviewer`)

Reviews code changes. Flags issues, suggests improvements, approves or requests changes.

**Boundary:** Read-only on production. Can comment anywhere.

### 5. Cleaner (`/cleaner`)

Refactors dead code, updates docs, removes tech debt, updates dependencies.

**Boundary:** Cannot change functionality. Only refactors. Must not break tests.

### 6. Self-Improver (`/self-improve`)

The meta-role. Can modify the harness itself — AGENTS.md, SKILL.md, CI pipeline, tooling. The AI that improves the AI that builds Voix.

**Boundary:** Cannot modify its own self-improvement instructions without human approval. Cannot expand its own permissions.

---

## Execution Flow

```
Human triggers /improve → Self-Improver reviews → Architect redesigns if needed 
→ Engineer implements → Tester verifies → Reviewer approves → Human approves merge
```

Each step produces artifacts that feed into the next.

---

## Safety Rules

1. **No force-push to main** — ever
2. **No deletion of branches** without explicit human approval
3. **No external network calls** from AI-generated code without audit
4. **AI cannot modify its own permission boundaries** without human sign-off
5. **Secrets stay in `.env`, never in code**
6. **Every merge to main requires CI green**

---

## Skill Activation

When the user triggers a slash command, activate the corresponding role:

| Command | Role |
|---------|------|
| `/architect` | Architect |
| `/build` | Engineer |
| `/test` | Tester |
| `/review` | Reviewer |
| `/clean` | Cleaner |
| `/improve` | Self-Improver (full pipeline) |

---

## Harness Files

- `AGENTS.md` — This file. Role definitions and safety rules.
- `SKILL.md` — Technical skill definitions for each role.
- `SKILLS/` — Actual prompt templates and scripts for each role.
- `docs/` — Architecture, requirements, design documents.

---

## 🤖 Ralph Loop — Autonomous Development Cycle

When triggered (cron or human), the AI enters the **Ralph Loop**:

```
┌─────────────────────────────────────────────────────────┐
│  RALPH LOOP (Recurring Autonomous Lifelong Handoff)     │
└─────────────────────────────────────────────────────────┘

   ┌──────────┐    ┌──────────┐    ┌──────────┐
   │ SURVEY   │───▶│ IMPLEMENT│───▶│  TEST    │
   └──────────┘    └──────────┘    └──────────┘
        │                               │
        │                         ┌─────┴─────┐
        │                         │           │
        ▼                         ▼           ▼
   ┌──────────┐              ┌─────────┐  ┌─────────┐
   │  REPORT  │◀─────────────│ PASS?  │──│  FIX    │
   └──────────┘              └─────────┘  └─────────┘
                                  │
                            ┌─────┴─────┐
                            │           │
                            ▼           ▼
                       ┌────────┐  ┌────────┐
                       │ COMMIT │  │ CLEAN  │
                       └────────┘  └────────┘
                            │
                            ▼
                       ┌────────┐
                       │  DONE  │ (until next trigger)
                       └────────┘
```

### Phase 1: SURVEY (Discover)

Survey the project state without changing anything:

1. **Read `REQUIREMENT.md`** — understand what's left to build
2. **Read `MEMORY.md`** — recall previous session context
3. **`git log --oneline -20`** — see recent changes
4. **`git status`** — check working tree state
5. **Scan source files** — understand current implementation depth
6. **List missing pieces** — identify P0/P1/P2 gaps
7. **Write findings to `memory/YYYY-MM-DD.md`** — document what you found

### Phase 2: IMPLEMENT (Build)

Work through the highest-priority missing piece:

1. **Create a feature branch**: `git checkout -b feat/<name>`
2. **Implement the feature** following existing code style
3. **Write inline comments** explaining why decisions were made
4. **Do not break existing functionality**
5. **Commit with conventional commit message**

### Phase 3: TEST (Verify)

Test what you built:

1. **Run `cargo check`** (or `npm run build`) — verify compilation
2. **Read test files** — ensure nothing is broken
3. **If tests fail** → fix or skip to CLEAN phase
4. **Document test results** in daily memory

### Phase 4: FIX or COMMIT

**If tests pass:**
```
git add .
git commit -m "feat: <description>"
git push origin feat/<name>
```

**If tests fail (non-critical):**
- Fix the issue if obvious
- Otherwise document in `memory/YYYY-MM-DD.md` and move to CLEAN

### Phase 5: CLEAN (Polish)

Before finishing the loop:

1. **Run `cargo fmt`** — format code
2. **Check for dead code** — remove unused code
3. **Update `REQUIREMENT.md`** — mark completed items
4. **Update `memory/YYYY-MM-DD.md`** — log what was done
5. **Commit cleanup separately**: `git commit -m "chore: clean up"`

### Phase 6: REPORT (Notify)

Write a brief report of what was accomplished:

- What was found (survey)
- What was built (implement)
- What passed/failed (test)
- What remains (next steps)

Post report to configured channel (Feishu).

---

## Ralph Loop Trigger Configuration

### Cron Schedule
- **Daily at 00:00** (midnight) — full Ralph Loop
- **Every 6 hours** — quick status check + implement if idle

### Ralph Loop State File
Store state in `memory/ralph-loop-state.json`:

```json
{
  "lastRun": "2026-03-28T00:00:00+08:00",
  "lastPhase": "REPORT",
  "incompleteTasks": [
    "Global hotkey registration",
    "Accessibility permission flow",
    "Settings panel UI"
  ],
  "currentBranch": "feat/hotkey",
  "completedThisSession": []
}
```

### Boundaries

**Ralph Loop CAN do:**
- Read all files in the project
- Write and modify source code
- Run compilation/tests
- Create commits on feature branches
- Push to remote branches
- Update memory and documentation

**Ralph Loop CANNOT do:**
- Merge to main without human approval
- Delete branches
- Push secrets or credentials
- Execute external network calls from new code without review
- Modify `AGENTS.md` safety rules

---

## CI/CD Pipeline

The harness integrates with GitHub Actions:

```yaml
# .github/workflows/harness.yml
name: Ralph Loop CI
on:
  push:
    branches: [feat/*]
  pull_request:
    branches: [main]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Rust check
        run: cargo check --all-targets
      - name: Format check
        run: cargo fmt --check
      - name: Clippy
        run: cargo clippy -- -D warnings
```

---

## Quick Start (for AI when waking up)

When you wake up for a Ralph Loop session:

1. Read `memory/ralph-loop-state.json` — know where you left off
2. Read `memory/YYYY-MM-DD.md` (today) — recent context
3. If `lastPhase == REPORT` → start fresh from SURVEY
4. If `lastPhase != REPORT` → resume from `lastPhase`
5. Execute your phase
6. Update state file
7. If done → write REPORT → sleep

**Never re-read this AGENTS.md section once you understand it. Just execute.**
