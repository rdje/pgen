# KNOWLEDGE-MAP — derived, question-keyed retrieval layer so an AI never re-does archaeology

> Task tree. **Metadata** — Status: `active` (bundle landed `.1`); Created: 2026-06-03
> (`PGEN-KNOWLEDGE-MAP-0001`); Roadmap lane: cross-cutting durable-memory / continuity.
>
> Director directive (2026-06-03): *"create all the necessary parts (.sh, .md, …) in
> addition to … task-trees + memory-architecture. A self-contained deliverable, git-tracked
> in PGEN, which other projects can use by copying the bundle. As project-agnostic as
> possible. Thorough in what shall be done and what not to do, with working scripts — so an
> AI/LLM never has to do archaeology again to rediscover a fact already logged."*
>
> Trigger: a self-caught "tools-first archaeology" framing — re-deriving the SV residual's
> cause from code because the durable facts (depth-budget cause, error taxonomy, two-surface
> build, coverage model) were in task-leaves/commits but **not** in the book or any
> retrieval index (verified: 0 book files, 0 live docs mentioned them).
>
> Discipline: [[feedback_regex_book_live]] (books are the user's only window),
> [[feedback_commit_workflow_strict]], composes with `MEMORY_ARCHITECTURE.md` (does not
> replace it).

---

## The principle (binding)

Archaeology — a future session re-deriving a fact from code/runtime that was **already
logged once** — is a **retrieval** failure, not a recording failure. The fix is a
**machine-derived, question-keyed index** over small, self-describing fact files, so
retrieval is one lookup, not an excavation. The index is **derived, never hand-curated**
(hand-curation is the `MEMORY_ARCHITECTURE.md` §6/§12 anti-pattern and costs commit time);
the machine pays the (millisecond) generation cost in the pre-commit hook.

Honest ceiling: this kills archaeology for **structural/causal facts**; it does **not**
remove first-time **diagnostics** of changing runtime state (you cannot pre-write a
measurement) — there, the durable *conclusion* becomes a fact card.

## What this is NOT

Not a replacement for the book, live docs, task-trees, or decision records, and **not a
conversion project**. The KM is an additive retrieval layer that points *into* those. See
`knowledge-map/FAQ.md` ("Do we have to convert all our docs?" → no).

---

## Leaves

### `.1` — the bundle (DONE, `PGEN-KNOWLEDGE-MAP-0001`)
Self-contained, project-agnostic `knowledge-map/` bundle + dogfood in pgen.

- **Acceptance:** generator deterministic; checker fails on missing fields / dup ids /
  stale map; both pass clean; enforcement wired (pre-commit + CI); ≥1 real fact seeded.
- **Delivered:**
  - `knowledge-map/KNOWLEDGE_MAP_ARCHITECTURE.md` (the standard: model, fact format, what
    to index / what NEVER to convert, the six LLM-retrieval properties, enforcement, read
    path, adoption checklist, anti-patterns).
  - `knowledge-map/FAQ.md` (plain-language explainer: no-conversion, the ceiling, costs,
    sizing — preserves the design-conversation answers).
  - `knowledge-map/README.md`, `knowledge-map/install.sh` (idempotent adoption helper).
  - `knowledge-map/scripts/gen_knowledge_map.sh` (derives the deterministic map; portable
    POSIX sh + awk), `check_knowledge_map.sh` (validate facts + derive-and-diff sync gate),
    `knowledge_map.conf` (`:=` config: env > repo override > bundle default).
  - `knowledge-map/templates/FACT_TEMPLATE.md`, `hooks/pre-commit.snippet`,
    `ci/knowledge-map-gate.yml`.
  - **Dogfood:** `docs/knowledge/` seeded with the 4 facts re-derived today
    ([[sv-residual-depth-budget-cause]], [[stimuli-generation-error-reasons]],
    [[ast-two-surface-construction]], [[stimuli-residual-coverage-model]]); generated
    `KNOWLEDGE_MAP.md` (4 facts / 20 question keys); pgen `.githooks/pre-commit` +
    `.github/workflows/memory-architecture-gate.yml` run the KM gate.
- **Verification:** `gen` deterministic (regenerate-and-diff identical); `check` PASS clean,
  FAIL on injected missing-field fact, FAIL on stale map, PASS after regen; `install.sh`
  idempotent.

### `.2` — incremental seeding (OPEN, on demand — NOT a sweep)
Add a fact card whenever a durable fact is established, or whenever archaeology is caught.
Optionally fold high-traffic `docs/decisions/` records in by adding `answers:` front-matter
in place. **No big-bang migration** (see FAQ).

Seeded (demand-driven, this lane stays OPEN):
- `2026-07-01` (`PGEN-KNOWLEDGE-MAP-0004`): [[ebnf-supported-vs-aspirational-constructs]] — captures the
  EBNF SUPPORTED / NOT-IMPLEMENTED / PARTIAL surface re-derived this session while authoring the `ebnf`
  grammar-author book (`EBNF-BOOK.1+.2`): which constructs the codegen consumes vs which the self-hosting
  meta-grammar only self-describes, plus the element-level `[ … ]`=optional-vs-character-class footgun. A
  genuine "archaeology caught" trigger — the supported surface had no retrieval card, only the codegen +
  shipped grammars. Map regenerated → 32 facts / 217 question keys; `check_knowledge_map.sh` PASS.

---

## Frontier
`.2` — grow the map one card at a time, demand-driven. Each new durable fact (or each caught
re-derivation) earns a card before the turn ends.
