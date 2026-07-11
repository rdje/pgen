# MCP-CONTROL: AI-Native / MCP-Controllable PGEN (parked brainstorm)

## Metadata

- Tree ID: `MCP-CONTROL`
- Status: `parked` (brainstorm captured 2026-06-14; NOT activated — sequenced
  AFTER the locked-program closure drive unless the director re-prioritizes)
- Roadmap lane: `Platform expansion — make PGEN agent-controllable (north-star
  trust goal + signoff/breathtaking vision)`
- Created: `2026-06-14`
- Last updated: `2026-07-11` (session #90 refinement — LIVE in-flight introspection + stall detection)
- Owner: repo-local workflow
- Slice ID for this capture: `PGEN-MCP-0001` (pure-docs; no code)

## Provenance

Director brainstorm 2026-06-14 (explicitly framed as "pure brainstorming, not a
pivot"). Captured as a parked task tree so the idea survives session loss
(layer B) and is ready to scope if/when prioritized. The director's standing
instruction was: capture as much information as we can, then resume the active
work (`RTL-FE-CLOSURE.5.3`). This tree must NOT be picked by PNT while `parked`.

## Goal

Expose PGEN as an **AI-native platform**: an LLM/agent can drive the whole PGEN
loop — generate a parser from an EBNF, generate stimuli, run certificate
coverage, lint a grammar, parse an input and inspect the AST/trace, root-cause
why a rule won't witness, run a gate — through a standard interface (MCP =
Model Context Protocol), without bespoke per-tool integration.

Design law (from the brainstorm): **machine-controllable first, MCP-exposed
second.** MCP is an integration *adapter*, never baked into the engine.

## Why this fits PGEN (rationale)

- The brainstorm's real insight: the differentiator is not "MCP support" — it is
  **deep semantic introspection exposed through a clean API**; MCP is just the
  standard bridge. **PGEN already IS a deep-introspection engine.** The exact
  loop a maintainer/agent runs today (reproduce cert-coverage → pull per-rule
  debug probes → get the reach path → classify generator-vs-grammar via the
  attribution rule → fix → decisive cross-grammar A/B → gate) is PGEN's existing
  machinery. MCP would let *any* agent drive that loop.
- Aligns with the README north-star ("make PGEN the de-facto go-to platform for
  parsers because projects can trust it") and `project_stimuli_generator_signoff_vision`.
- The lift is **adapter, not engine**: PGEN already has the three layers the
  brainstorm prescribes — a Rust lib core, the embedding API
  (`rust/docs/EMBEDDING_API_CONTRACT.md`), and CLIs (`ast_pipeline`,
  `parseability_probe`, …). MCP is a thin wrapper over the existing API.

## Architecture (target)

```
+----------------+
| AI / MCP client | (Claude / IDE agent / verification copilot)
+--------+-------+
         | MCP (resources / tools / prompts)
+--------v-------+
| pgen_mcp        | thin adapter — NO engine logic
+--------+-------+
         | stable API (embedding API; optional long-lived session)
+--------v-------+
| PGEN core lib   | AST pipeline, generators, parser registry, linter,
| (rust/src/)     | cert-coverage, semantic store, trace
+----------------+
```

- The CLI and the MCP adapter are BOTH thin frontends over the same stable API.
- A **stateful session** (load grammar + generated parser once; run many
  fine-grained reach probes / parses against it) is the one genuinely-new piece
  worth adding — it accelerates the iterative debug loop, which is otherwise
  one-shot-per-CLI-invocation. Batch ops (regen, gates) stay one-shot.

## MCP surface inventory (mapped to real PGEN capabilities)

### Resources (read-only introspection)
- `pgen://grammar/<name>` — the EBNF source + rule inventory + linter verdict.
- `pgen://grammar/<name>/cert-coverage` — per-rule proof/witness/UNKNOWN (the
  trustworthiness number; `--report-certificate-coverage`).
- `pgen://grammar/<name>/lint` — well-formedness/well-definedness (`--lint-grammar`).
- `pgen://grammar/<name>/gap-report` — residual targets + per-target
  `reach_classification` (`reachable_by_plan` / `no_reach_path` / …).
- `pgen://parse/<run>/ast` and `pgen://parse/<run>/trace` — structured
  parse/generation AST + trace (`none/low/medium/high/debug`).
- `pgen://parse/<run>/live-status` — ⭐ **LIVE in-flight introspection** (session #90
  refinement): while a parse RUNS, its current rule + input position + recursion depth
  (piggyback the recursion-guard `parse_stack`), `furthest_position` as a progress signal,
  memo hit/miss, current input case, and a **stall metric** (`furthest_position` Δ/sec — the
  signature of spinning-not-progressing). See the refinement section below.
- the per-parser books + `docs/contracts/`.

### Tools (actions; writes are gated — see Security)
- read-only: `report_certificate_coverage`, `lint_grammar`, `parse_input`
  (dump AST), `reach_probe(rule)`, `generate_stimuli`, `query_semantic_store`.
- write/heavy (explicit approval): `generate_parser`, `run_gate(<name>)`,
  `edit_grammar` (MUST stay task-tree-owned per the Code-Change Doctrine),
  `file_pgen_bug(...)`.

### Prompts (workflows — PGEN's recurring tasks, packaged)
- `drive_cert_coverage_to_zero(grammar)` — close UNKNOWN→0.
- `root_cause_unwitnessed_rule(grammar, rule)`.
- `diagnose_parse_rejection(grammar, input)`.
- `add_return_annotation_and_prove_roundtrip(grammar, rule)`.

## KEY DESIGN ANSWER — two LLMs, and the fault is in PGEN's Rust (not the EBNF)

Scenario (director, 2026-06-14): two LLMs drive PGEN via MCP on different EBNFs.
As long as a fault lives in an EBNF, the owning agent fixes its grammar. But
when deep debug clearly shows the fault is in **PGEN's Rust (AST pipeline /
generator / engine)**, how is it handled? Answer (file a structured bug report —
and PGEN already has the machinery):

1. **It is the attribution rule, one level up.** PGEN already classifies a
   generator-rejected sample as *generator-reach deficiency* vs *grammar
   ill-formedness*, with the **linter as adjudicator** ("no verdict without a
   certificate"). The MCP case adds a THIRD bucket: a **PGEN platform defect**.
   The agent must PROVE the attribution before filing — e.g. a minimal grammar +
   input where the generated parser rejects text that is valid-per-EBNF AND the
   linter reports the grammar well-formed. That proof IS the bug report payload.
   The `file_pgen_bug` tool REQUIRES the attribution evidence (the
   certifying-algorithm discipline; never the agent's hunch).
2. **The protocol already exists.** `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
   + `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` are exactly the
   structured downstream-consumer bug pipeline (required bundle: minimal repro,
   grammar, input, expected vs actual, trace). An MCP agent is just another
   consumer; `file_pgen_bug(repro, grammar, input, expected, actual,
   trace_excerpt, attribution_evidence)` → a ledger row.
3. **The write boundary makes this safe by construction.** A per-parser agent
   gets *gated* write access to ITS OWN grammar, but the **engine is read-only to
   it**. Critical with two concurrent agents: an engine change affects EVERY
   parser (the reason every `.5.x` generator tweak does a decisive cross-grammar
   SV A/B). Concurrent un-A/B'd engine edits = cross-grammar blast radius. So
   engine fixes are **serialized behind the existing task-tree + gate pipeline**:
   bug report → a maintainer (human, or a single privileged "platform" agent)
   owns a task-tree leaf → cross-grammar A/B → gates → release/ledger. Never an
   inline ad-hoc engine patch by a per-parser agent.
4. **Auto-fix allowed, auto-merge never.** A privileged platform agent MAY
   propose the engine fix, but only through the full leaf + A/B + gates; the
   **gates remain the source of truth** (PGEN's "machine-checkable gates behind
   every claim" doctrine, unchanged).

## Security posture (from the brainstorm; aligns with existing doctrine)
- read-only by default; explicit approval for any write (grammar edits, parser
  regen, generated artifacts — all "code changes" under the Code-Change Doctrine).
- project-root restriction; no arbitrary shell tool; audit log of every tool
  call; deterministic seeds + reproducible command lines (PGEN is already
  determinism-obsessed).
- **Do NOT stream huge dumps through MCP** — the generated parser is ~5.6 MB, a
  debug trace is GB-scale, LIVE status is ~866 KB, a cert-coverage debug capture
  is hundreds of lines. Expose **structured queries** ("which rules are
  UNKNOWN?", "show the probe sample for rule R"), not raw dumps. MCP is the
  control/context layer, not a bulk transport.

## Phased plan (proposed)
- Phase 1: read-only MCP server over existing introspection (grammar, lint,
  cert-coverage, gap-report, AST/trace queries).
- Phase 2: stateful session (load-once, many fine-grained probes/parses).
- Phase 3: controlled tools (generate_parser, generate_stimuli, run_gate) with
  approval + audit.
- Phase 4: `file_pgen_bug` into the existing protocol/ledger, attribution-gated.
- Phase 5: packaged workflow prompts (the four above).
- Phase 6: multi-tool orchestration (git, issue tracker, regression).

## Refinement (session #90, 2026-07-11) — LIVE in-flight introspection + stall detection

Director conversation, born from concrete pain: the `regex_pcre2_compile_oracle_gate` ground for
**~2.5 hours** on a catastrophic-backtracking corpus cell (a DEBUG probe) with **zero live visibility**
into WHICH case / how deep / progressing-vs-spinning; the only recovery was `kill`. Director's ask
(re-surfaced — this IS part of the MCP vision above): *"a way to interact with jobs via an API to know
what they are doing … deep semantic introspection exposed via a clean API used through MCP, in debug mode,
zero runtime cost in release."* Post-mortem `kill` is not observability.

This refines — does not replace — the MCP vision: the resources above are STATIC/post-hoc (a completed
run's AST/trace/cert). The missing dimension is **LIVE, in-flight** self-report ("what is the parser doing
RIGHT NOW, while it spins"). Two tiers:

- **Tier 1 — sweep/harness progress + stall detection (cheap, highest value/effort).** The corpus/gate
  runner emits a per-case heartbeat (`case N/M, elapsed, furthest_position, Δ-since-last`); a stall detector
  (`furthest_position` flat while call-count climbs) classifies a case as **catastrophic-backtracking
  suspected → skip + record** under a per-case budget. This alone turns today's 2.5h-blind into a one-minute
  "case 878 is pathological, skipped." Audit + extend the corpus harness's existing timeout. Consumable by a
  `Monitor`/`tail`; the natural MCP resource is `pgen://sweep/<run>/progress`.
- **Tier 2 — the generated-parser LIVE introspection channel** (`pgen://parse/<run>/live-status` above):
  cfg-gated to **debug** (`#[cfg(debug_assertions)]` / a `pgen_introspect` feature) so the RELEASE parser has
  ZERO added instructions (prove via a codegen-diff + release-artifact size/disasm check). Two access modes:
  (1) a **periodic status-file heartbeat** the parser rewrites **at rule boundaries** (robust — it keeps
  reporting even while the parser spins in catastrophic backtracking, and avoids async-signal-safety hazards);
  (2) an **on-demand `SIGINFO`/`SIGUSR1` snapshot** ("ask the stuck process what it's doing"). MCP exposes
  these as the `live-status` resource / a `live_status(run)` tool.

**Zero-release-cost** matches the project's deferred posture [[feedback_debug_only_trace_deferred]] (debug-only
trace/counter gating, deferred until parsers released — now TIMELY: regex is released). **Double duty:** the
same live introspection (memo stats, furthest_position progress, per-rule depth/timing) IS perf-attribution
infra — it directly serves the RGX-0078 SPEED campaign (WHERE-does-the-time-go, live). Build once, use for
both live debugging AND profiling. **Parser-AGNOSTIC:** a codegen/engine capability (teach the generator to
emit the cfg-gated introspection hooks) ⇒ every generated parser (SV/VHDL/JSON/regex) inherits it — the same
generator-maturity thesis as `project_rgx_0078_regex_slowness_followup`.

**Seeds already in-tree** (this is an evolution, not a greenfield): `--dump-rule-call-counts` (a live 250ms
dashboard — but terminal-attached, not a queryable API), always-on `furthest_position`, `PGEN_REPORT_MEMO_STATS`.
The refinement makes them queryable + semantic-position-aware + sweep-level + stall-classifying.

**Sequencing:** still `parked` (this refinement does NOT activate the tree). When prioritized, Tier 1 is a
cheap near-term win worth pulling forward (it prevents recurrence of the 2.5h-blind failure); Tier 2 slots
into Phase 1 (read-only introspection) as the live-status resource. Not part of RGX-0078, though it doubles
as its profiling substrate.

## Non-Goals
- No simulation-kernel analogues (waveforms, X/Z, run_until) — PGEN does not
  execute designs. The translation: trace / coverage-DB / gap-DB / semantic-store
  queries play the role "waveform query" plays for a simulator.
- Not MCP-first: the engine must stay deterministic, fast, scriptable, API-first;
  MCP is an adapter only.

## Acceptance Criteria (when/if activated)
- A read-only MCP server (Phase 1) that exposes the resource inventory, proven by
  a gate; the tracing contract (`none/low/medium/high/debug`),
  parser-agnosticism, and EBNF-as-single-source-of-truth all respected.
- Each code leaf task-tree-owned + committed through `COMMIT.md`.

## Task Tree

- ID: `MCP-CONTROL`
  Status: `parked`
  Goal: `AI-native / MCP-controllable PGEN — adapter over the existing API.`
  Children: `MCP-CONTROL.1` (scoping, when activated)

- ID: `MCP-CONTROL.1`
  Status: `pending` (blocked — parked pending director prioritization)
  Goal: `SCOPING (pure docs): when activated, pin Phase 1 (read-only MCP server) scope against the embedding API + the introspection surfaces; ordered leaf plan. Tools-first; no code until a code leaf owns it.`
  Acceptance: `Phase-1 scope + ordered leaf plan recorded.`
  Verification: `pending (parked)`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `MCP-CONTROL.1` | `blocked` (parked) | Not PNT-eligible while the tree is `parked`. Activate only on director prioritization, after the locked-program closure drive. |

## Decisions
- `2026-06-14`: Created as a PARKED brainstorm capture (director-authorized).
  Recommendation: park now, pursue after the closure drive (all parsers →
  UNKNOWN=0). The engine-bug-attribution answer is the load-bearing design point
  (see "KEY DESIGN ANSWER").

## Open Questions
- Stateful session lifetime/ownership model (per-agent vs shared)?
- Concurrency policy for two agents on different grammars sharing one engine
  build (read-only engine resolves most of it; confirm at scoping).

## Blockers
- Parked pending director prioritization; the active locked-program closure drive
  (`RTL-FE-CLOSURE`, SV `UNKNOWN`→0, ebnf 6-gap) has priority.

## Changelog
- `2026-06-14`: Created parked brainstorm capture (`PGEN-MCP-0001`).
- `2026-07-11` (session #90): Refinement — added the **LIVE in-flight introspection + stall detection**
  dimension (Tier 1 sweep-progress + Tier 2 debug-only zero-release-cost parser live-status via
  heartbeat + `SIGINFO` snapshot), motivated by the 2.5h-blind `regex_pcre2_compile_oracle_gate` grind.
  Reconciled a mistakenly-created duplicate `PARSER-INTROSPECTION` tree back into here (the director had
  already filed this vision as MCP-CONTROL). Still `parked`; Tier 1 flagged as a cheap near-term win.
