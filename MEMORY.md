# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> This is the bounded layer-A resume pointer per `MEMORY_ARCHITECTURE.md`.
> OVERWRITE the "Current state" block each update — never append history here.
> (History is in git (layer D) + the task-tree logs (layer B); durable
> facts/decisions are in `docs/decisions/` (layer C).)

## How to resume
- Read `MEMORY_ARCHITECTURE.md` (the memory/continuity system) and `README.md` (the project).
- Work is tracked in task-trees under `docs/tasks/`; the index is `docs/TASK_TREE.md`; follow `COMMIT.md`.
- Durable facts / standing disciplines / decisions live in `docs/decisions/` (see its `INDEX.md`).
- Live status: `LIVE_ACHIEVEMENT_STATUS.md`; changelog: `CHANGES.md`.

## Current state (OVERWRITE this block each update — do not append)
- latest_commit: (this commit) `PGEN-STIMULI-SIGNOFF-0003` (leaf STIMULI-SIGNOFF.2.2) — k-path coverage NUMERATOR: k_path_recording (default-OFF/monotone) records last-k call_stack window at each generate_rule entry; covered_k_paths()/k_path_coverage(). So the k-path MEASURE is COMPLETE (universe .2.1 + numerator .2.2 + coverage). lib 586/586, clippy 0, unit-tested. .2.3 pending = wire into SV gate + restate signoff bar in k-path terms + re-express 273 residual as k-path debt. Prior: `-0002` (.2.1 universe), `PARSE-PILLARS-0002`, `STIMULI-SIGNOFF-0001`. PUSHED through 275fc3f6; unpushed ~3.
- active_work_unit: PARSER SIGN-OFF PILLARS — all 3 trees `active`, each `.1` DONE. B.1 measured (stateful packrat super-linear ~N^1.66, CC 2020 confirmed). A.1 harness designed. C.1 oracle inventory done. STIMULI-SIGNOFF (pillar D) `.1` audit done (6 gaps→leaves). PARSE-SOTA `.11` (A5 _meta) parked.
- next_action: the remaining PARSE-* leaves are GATED — (A) `PARSE-COMPLETENESS.2` needs slang/Verible + sv-tests PROVISIONED (real-world setup, director go); (B) `PARSE-TERMINATION.3` (conditional memoization, CC 2020 fix — evidence-justified) = parser-agnostic ENGINE change needing sign-off; (C) `SV-EXH-PROOF.7.4.6` (derivation-directed construction → 273→literal-0; cite FDLOOP) = generation change + sign-off; (D) `PARSE-FIDELITY.2` blocked on PARSE-SOTA.11 (_meta); (E) `STIMULI-SIGNOFF.2` k-path metric (code leaf). Pick per director. KNOWLEDGE-MAP `.2` = card on demand.
- in_flight_uncommitted: none (this commit includes MEMORY.md sync).
- blockers: none. (Pre-existing: full-workspace `cargo test` RED from stale GlobalOptions ctors at parseability_probe.rs:738/754 — unrelated; use `cargo test --lib`.)

## Other open threads (not the active unit)
- **SV-EXH-PROOF.7.4** literal-0: `.7.4.1` criterion + `.7.4.2` Purdom min-length table + `.7.4.3a` depth-budget ROOT CAUSE + `.7.4.3` witness pass (KEPT) + `.7.4.4.1` (-0142) reason-split classifier + bounded WHY+WHERE sample → PROVED the "other" tail = SLOW GENERATION not context-gating (other=0 on 30- & 150-target real-SV; failures = target_timeout). Frontier `.7.4.4` RE-SCOPED: Purdom min-length-guided greedy witness expansion so slow witnesses converge fast (NOT reach-from-context — that targeted the disproven context-gating cause).
- **PARSE-SOTA** (active): research `.1`-`.7` done; A1 (`.8`, non-terminating detection — LR-reject dropped, PGEN handles LR) + A2 (`.9`, shadowing lint) ANALYSES done in grammar_wellformedness.rs; wiring `.8.1`/`.9.1` next; A4/A5 pending.
- **DIAG-SEVERITY** (CLOSED `-0001..0006`): severity never gated by verbosity; canonical GenerationErrorReason; enforcement gate (pre-commit+CI); book. Fixed the masking that hid the SV depth cause. Also reclaimed ~24 GB + relativized live-doc paths (PGEN-DOCPATH-0001).
- **SV-EXH-PROOF** literal-0: ROOT CAUSE pinned (`.7.4.3a`, -0140) = DEPTH-BUDGET exhaustion (deep-factored rules can't reach+complete within max_depth=24 from the top entry; ansi_port_declaration needs ≥10 depth; measured). Fix `.7.4.3` = per-target minimal witnesses with FRESH/adequate depth (root at/short-path to target + Purdom shortest subtree from `.7.4.2`'s min-length table + generalize the :4869 depth-slack), measured vs 888. NOT a global max_depth raise (blows up property_expr; reshapes the diverse pass).
- **PARSE-SOTA** (active, research `.1`–`.6` DONE): parser-gen path validated as a recognized published architecture; §1 adoption backlog awaits director review (Tier A: well-formedness check, ⭐shadowing lint, labeled failures, round-trip tests, ship `_meta`).
- **KNOWLEDGE-MAP** (active): `knowledge-map/` = self-contained, copyable, project-agnostic bundle (gen/check scripts + standard + FAQ + template + hook/CI + installer) deriving `KNOWLEDGE_MAP.md` from `answers:`-front-matter fact files in `docs/knowledge/`. Composes with MEMORY_ARCHITECTURE.md; no doc conversion. Decision [[project_knowledge_map_retrieval_layer]]. To answer "where is X documented", READ `KNOWLEDGE_MAP.md` FIRST (don't re-derive from code).
- Push: push IS release; default wait ~30 unpushed OR explicit "push". Restore tag `checkpoint/sv-exh-proof-3.2-clean` @ 41bef35e.
