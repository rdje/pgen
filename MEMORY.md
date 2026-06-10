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
- latest_commit: (this commit) `PGEN-BRANCH-BROADCAST-FIX-0002` (leaf `BRANCH-BROADCAST-FIX.2`, engine) — **🔧 defect A FIXED: whole-body-group trailing-annotation broadcast restored** (`extract_rule_annotations` tracks `branch_to_body` depth<=1 + structural whole-body discriminator, shared with the `ast_shape_contract` cross-extractor; patterns (A)–(D) locked green by 6 new unit tests). Runtime-proven `'x'` ≡ `"x"` typed on shipped `string_literal`; manifest inventory 19→20. **Blast radius MEASURED (stash-baseline regen, 9 targets): BOTH annotation grammars only** — return_annotation +1 row, **semantic_annotation +43 broadcast rows (108→151)**; regex/SV set-equal order-churn; others byte-identical; regex cert-coverage IDENTICAL (198/191/7/spf=0 seeds 0/7/42). Suites 682/0 + 767/0 dual-feature. PRIOR: `-0001` (defect-pair root-cause), `-0060` (H.10.1), `-0059` (H.9).
- 🧭 PNT batch (2026-06-09/10, director: "PNT yourself — do not involve me unless you can't decide"): (1-7) ✅ … `-0060`, BRANCH-BROADCAST-FIX `-0001`; (8) ✅ `.2` (`-0002`, this). NEXT = **`BRANCH-BROADCAST-FIX.3` ($text/MatchedText tournament span — use the branch's `candidate_end`, not post-rollback `parser.position`; audit other transform forms read captured content = expected unaffected)** → `.4` (ship BOTH annotation parsers: schema assessment + ledger + contract/book + release versioning) → `.5` (re-apply H.10.2.1). THEN the per-grammar UNKNOWN drives: regex H.10.2 survivors, vhdl 31, rtl_frontend 75, SV 738.
- 🧠 PROOF-DISCIPLINE (director challenge 2026-06-09, standing): when claiming a rule is "dead"/unreachable, PROVE it — `--lint-grammar unreachable_rules=0` is multi-entry-LENIENT (does NOT prove single-entry reachability); use the static ref-graph closure + the `--gap-report-json` oracle (`reachable:false unreachable_from_entry`). Reachability is PER-GRAMMAR (svpp `trivia` dead; a same-named `trivia` is ALIVE in `rtl_frontend.ebnf`). Don't call same-lineage corroboration "independent". KM `prove-rule-dead-or-reachable`.
- 🧭 DIRECTOR DIRECTIVE 2026-06-09 (3×, standing): **PGEN is NOT bug-free / NOT feature-complete — it is evolving toward both.** When a general, parser-agnostic enhancement to semantic annotations / the pipeline is needed, **discuss → own → design carefully → implement** (goal: parse as many languages as humanly possible — C, JS, HTML, Perl6, …). The stimuli generator is a BUG-FINDING ORACLE.
- 🚨 DIRECTOR PRINCIPLE 2026-06-09 (emphatic, standing): **fix parser bugs ASAP = HIGHEST priority** — "no point on cert-cover if a clearly identifiable parser bug shows up." (Defect A: FIXED by `GRAMMAR-WELLFORMED.H.8` `-0057`; the post-SV-PARSE-STRICT.2 delta WAS measured — spf 3→6 from the now-correct rejections, →0 after H.8.)
- 🧭 DIRECTOR-CONFIRMED BOUNDARY (standing): SV parser = **context-aware, SOUND gating** — rejects *provably*-undeclared type/nettype ids (current-TU + `--lib-in` facts), accepts when it can't prove (unresolved `import pkg::*`); cross-unit binding + type/width/param/generate = elaboration.
- locked_program (director 2026-06-08): ALL EXISTING PARSERS → `Done` (cert-coverage clean + UNKNOWN=0). **spf=0 on ALL wired grammars (H.8); seed-robust UNKNOWN=0: rtl_const_expr, json, svpp (H.9, 32/32 seeds).** Remaining UNKNOWN: regex 7 (H.10.1 cleared the 12 no-path), vhdl 31, rtl_frontend 75, SV 738 (+68 no-path/multi-entry).
- active_work_unit: `BRANCH-BROADCAST-FIX.3` next (engine $text span fix; design direction in the tree — branch arm must use `candidate_end`). in_flight_uncommitted: none. blockers: none.
- push: ~93 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
