# Agent bootstrap (read this first, whatever AI / harness you are)

1. Read `README.md` (project objective, layout, standard commands).
2. Read `MEMORY_ARCHITECTURE.md` (how durable, harness-agnostic memory + continuity
   work here — MANDATORY; defines the 4 layers + enforcement).
3. Resume from `MEMORY.md` (the bounded layer-A resume pointer) → the active
   task-tree's frontier.
4. Track ALL work in task-trees under `docs/tasks/` (index `docs/TASK_TREE.md`);
   record durable facts/decisions in `docs/decisions/` (index there); commit per
   `COMMIT.md` with the work-unit id in the subject.
5. Before committing, run `scripts/check_memory_architecture.sh` — CI runs it too.
   Activate the local git hooks once per clone: `git config core.hooksPath .githooks`.

Nothing important may live only in this conversation — route it to a layer and commit.
