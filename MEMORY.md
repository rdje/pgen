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
- latest_commit: (this commit) `PGEN-BRANCH-BROADCAST-FIX-0003` (leaf `BRANCH-BROADCAST-FIX.3`, engine) — **🔧 defect B FIXED: branch-level `$text` returns the exact matched span in tournament rules** (the branch arm evaluates the transform BEFORE the position rollback; audit: MatchedText is the ONLY `parser.position`-reading transform form ⇒ one-statement reorder is complete; codegen-ordering unit test locks both arms). 9 targets regenerated; suites 683/0 + 768/0; regex cert-coverage IDENTICAL (198/191/7/spf=0 seeds 0/7/42); corpora unaffected (no shipped branch-level tournament `$text` yet — `.5` is the live-fire proof). PRIOR: `-0002` (.2 broadcast remap fix: BOTH annotation grammars corrected — return_annotation +1 row runtime-proven `'x'`≡`"x"`, semantic_annotation +43 rows 108→151), `-0001` (root-cause), `-0060` (H.10.1).
- 🧭 PNT batch (2026-06-09/10, director: "PNT yourself — do not involve me unless you can't decide"): (1-8) ✅ … `-0001`, `.2` (`-0002`); (9) ✅ `.3` (`-0003`, this). NEXT = **`BRANCH-BROADCAST-FIX.4` (ship BOTH annotation parsers: AST-dump schema assessment [mis-shape-correction category, SVPP-0004 precedent] + ledger rows + contract + per-parser book lockstep + release versioning — return_annotation string_literal branch 1 AND semantic_annotation's 43 broadcast rows)** → `.5` (re-apply H.10.2.1: regex atomicity, expect UNKNOWN 7→5 + restrict:"D" proof). THEN the per-grammar UNKNOWN drives: regex H.10.2 survivors, vhdl 31, rtl_frontend 75, SV 738.
- 🧠 PROOF-DISCIPLINE (director challenge 2026-06-09, standing): when claiming a rule is "dead"/unreachable, PROVE it — `--lint-grammar unreachable_rules=0` is multi-entry-LENIENT (does NOT prove single-entry reachability); use the static ref-graph closure + the `--gap-report-json` oracle (`reachable:false unreachable_from_entry`). Reachability is PER-GRAMMAR (svpp `trivia` dead; a same-named `trivia` is ALIVE in `rtl_frontend.ebnf`). Don't call same-lineage corroboration "independent". KM `prove-rule-dead-or-reachable`.
- 🧭 DIRECTOR DIRECTIVE 2026-06-09 (3×, standing): **PGEN is NOT bug-free / NOT feature-complete — it is evolving toward both.** When a general, parser-agnostic enhancement to semantic annotations / the pipeline is needed, **discuss → own → design carefully → implement** (goal: parse as many languages as humanly possible — C, JS, HTML, Perl6, …). The stimuli generator is a BUG-FINDING ORACLE.
- 🚨 DIRECTOR PRINCIPLE 2026-06-09 (emphatic, standing): **fix parser bugs ASAP = HIGHEST priority** — "no point on cert-cover if a clearly identifiable parser bug shows up." (Defect A: FIXED by `GRAMMAR-WELLFORMED.H.8` `-0057`; the post-SV-PARSE-STRICT.2 delta WAS measured — spf 3→6 from the now-correct rejections, →0 after H.8.)
- 🧭 DIRECTOR-CONFIRMED BOUNDARY (standing): SV parser = **context-aware, SOUND gating** — rejects *provably*-undeclared type/nettype ids (current-TU + `--lib-in` facts), accepts when it can't prove (unresolved `import pkg::*`); cross-unit binding + type/width/param/generate = elaboration.
- locked_program (director 2026-06-08): ALL EXISTING PARSERS → `Done` (cert-coverage clean + UNKNOWN=0). **spf=0 on ALL wired grammars (H.8); seed-robust UNKNOWN=0: rtl_const_expr, json, svpp (H.9, 32/32 seeds).** Remaining UNKNOWN: regex 7 (H.10.1 cleared the 12 no-path), vhdl 31, rtl_frontend 75, SV 738 (+68 no-path/multi-entry).
- active_work_unit: `BRANCH-BROADCAST-FIX.4` next (ship surface for BOTH annotation parsers; open questions in the tree — schema bump decisions per release policy). in_flight_uncommitted: none. blockers: none.
- push: ~94 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
