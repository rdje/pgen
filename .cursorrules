# Agent bootstrap (read this first, whatever AI / harness you are — Claude Code, Codex, Gemini, Cursor, …)

1. Read `README.md` (project objective, layout, standard commands).
2. Read `MEMORY_ARCHITECTURE.md` (how durable, harness-agnostic memory + continuity
   work here — MANDATORY; defines the 4 layers + enforcement).
3. Read `TOOLBOX.md` (the diagnostic & debug toolbox). ⛔ TOOLBOX-FIRST: for ANY issue —
   an `UNKNOWN`, a rejected parse, a hang, a reach gap, a "why isn't this witnessed" — run
   the debug tools FIRST; never eyeball a grammar or guess a root cause. Mechanically enforced.
4. Read `DOCTRINE_ENFORCEMENT.md` (how EVERY mechanizable doctrine is enforced — the portable
   enforcer + the task-acceptance checklist a code change MUST pass).
5. Resume from `MEMORY.md` (the bounded layer-A resume pointer) → the active
   task-tree's frontier.
6. Track ALL work in task-trees under `docs/tasks/` (index `docs/TASK_TREE.md`);
   record durable facts/decisions in `docs/decisions/` (index there); commit per
   `COMMIT.md` with the work-unit id in the subject. A code change MUST pass the
   `TOOLBOX.md` acceptance checklist (root cause + addressed + no regression) in its task leaf.
7. Activate the local git hooks once per clone: `git config core.hooksPath .githooks`. The
   pre-commit hook runs `scripts/check_doctrines.sh` (the GENERAL doctrine enforcer —
   memory-architecture + the toolbox/task-acceptance gate + more); CI runs the same. These are
   git-level and harness-agnostic — they fire for Codex, Claude Code, Gemini, … identically.

Nothing important may live only in this conversation — route it to a layer and commit.
