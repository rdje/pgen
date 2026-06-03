---
name: project-knowledge-map-retrieval-layer
description: Adopted the Knowledge Map (KM) — a derived, question-keyed retrieval index over durable facts so an AI/LLM never re-does archaeology. Self-contained copyable bundle; composes with (does not replace) MEMORY_ARCHITECTURE.md. Director-commissioned 2026-06-03.
metadata:
  node_type: memory
  type: project
---

**Director decision (2026-06-03):** build a self-contained, project-agnostic, git-tracked
deliverable — the **Knowledge Map (KM)** bundle — so that an AI/LLM **never has to do
archaeology** (re-derive a fact from code/runtime) to rediscover something already logged.
Owned by task tree `KNOWLEDGE-MAP` (`PGEN-KNOWLEDGE-MAP-0001`). To be copyable into the
director's other 9 projects.

**Trigger (the honest one):** a self-caught "tools-first *archaeology*" framing — I had
re-derived the SV stimuli residual's cause from source because the durable facts (depth
budget root cause, error-reason taxonomy, two-surface AST build, coverage model) lived only
in task-leaves + commit messages, never in the book or any retrieval index. Verified the
gap with tools: **0** book files and **0** live docs mentioned them. Archaeology is a
*retrieval* failure, not a recording one — see [[feedback_regex_book_live]] (the book is the
director's only window).

**Decision / design (Context → Decision → Consequences):**
- *Context.* `MEMORY_ARCHITECTURE.md` already nails durability + the write path, but its
  layers are organized by **lifecycle**; retrieval is organized by **topic/question**, so a
  fact written to the convenient layer (a task leaf) is not findable at read time. A
  hand-curated index would rot and cost commit time — the §6/§12 anti-pattern.
- *Decision.* A **fact** is one `.md` whose front-matter has a non-empty `answers:` list
  (plus `id`/`title`/`date` and `evidence`/`reverify`). A portable script **derives** a
  deterministic, question-keyed `KNOWLEDGE_MAP.md`; a checker validates facts + asserts
  sync via regenerate-and-diff. The pre-commit hook regenerates + `git add`s the map (zero
  agent cost; drift structurally impossible); CI is the backstop. The map is a derived
  artifact, never hand-edited; facts can also be grepped directly (map = cache, not
  load-bearing).
- *Consequences.* It is **additive** — it replaces nothing and requires **no conversion**
  of the book, live docs, task-trees, or decision records (those are the *destinations* a
  fact card points into). Cards are added **lazily/on-demand**, one per established or
  re-derived fact — never a migration sweep. Honest ceiling: it kills archaeology for
  **structural/causal facts**, not first-time **diagnostics** of changing runtime state
  (whose durable *conclusion* then becomes a card).

**Delivered (`.1`):** `knowledge-map/` bundle — `KNOWLEDGE_MAP_ARCHITECTURE.md`, `FAQ.md`,
`README.md`, `install.sh`, `scripts/{gen,check}_knowledge_map.sh` + `knowledge_map.conf`,
`templates/FACT_TEMPLATE.md`, `hooks/pre-commit.snippet`, `ci/knowledge-map-gate.yml`.
Dogfood: `docs/knowledge/` seeded with the 4 re-derived facts; `KNOWLEDGE_MAP.md` generated;
pgen `.githooks/pre-commit` + the memory-architecture CI workflow run the KM gate. Verified:
generator deterministic; checker passes clean and fails on missing-field / stale-map.

**Composes with:** [[project-memory-architecture-adoption]] (the durable-memory standard the
KM extends). Discipline: [[feedback_commit_workflow_strict]], [[feedback_regex_book_live]].
