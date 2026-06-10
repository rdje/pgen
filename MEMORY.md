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
- latest_commit: (this commit) `PGEN-BRANCH-BROADCAST-FIX-0004` (leaf `BRANCH-BROADCAST-FIX.4`, ship/ledger) — **📦 corrected shapes SHIPPED for BOTH annotation parsers**: ledger `RETANN-0001`+`SEMANN-0001` (regression window 2026-05-14→2026-06-10); dated contract entries (versioning RESOLVED: annotation families are date-versioned, no numeric schema to bump — the SVPP-0004 equivalent is contract entry + ledger + manifest locks); manifest locks = discriminating `'x'` runtime sample (return_annotation) + FULL 151-row inventory with both gate comparisons (semantic_annotation — the only lock class that discriminates its branch shapes). 14 shape gates green; `annotation_contract_gate` exit 0; dual-feature 768/0. PRIOR: `-0003` (.3 $text span), `-0002` (.2 broadcast remap), `-0001` (root-cause).
- 🧭 PNT batch (2026-06-09/10, director: "PNT yourself — do not involve me unless you can't decide"): (1-9) ✅ … `.2` (`-0002`), `.3` (`-0003`); (10) ✅ `.4` (`-0004`, this). NEXT = **`BRANCH-BROADCAST-FIX.5` (re-apply GRAMMAR-WELLFORMED.H.10.2.1: the regex atomicity edits — parens-group broadcast `-> $text` on `short_prop_letter`/`ascii_restrict_modifier`; verify end-to-end: AST A/B byte-identical, fused rendering, `restrict:"D"` not `""` [the .3 live-fire proof], regex cert-coverage UNKNOWN 7→5)** — unblocks + closes H.10.2.1. THEN the per-grammar UNKNOWN drives: regex H.10.2 remaining survivors, vhdl 31, rtl_frontend 75, SV 738.
- 🧠 PROOF-DISCIPLINE (director challenge 2026-06-09, standing): when claiming a rule is "dead"/unreachable, PROVE it — `--lint-grammar unreachable_rules=0` is multi-entry-LENIENT (does NOT prove single-entry reachability); use the static ref-graph closure + the `--gap-report-json` oracle (`reachable:false unreachable_from_entry`). Reachability is PER-GRAMMAR (svpp `trivia` dead; a same-named `trivia` is ALIVE in `rtl_frontend.ebnf`). Don't call same-lineage corroboration "independent". KM `prove-rule-dead-or-reachable`.
- 🧭 DIRECTOR DIRECTIVE 2026-06-09 (3×, standing): **PGEN is NOT bug-free / NOT feature-complete — it is evolving toward both.** When a general, parser-agnostic enhancement to semantic annotations / the pipeline is needed, **discuss → own → design carefully → implement** (goal: parse as many languages as humanly possible — C, JS, HTML, Perl6, …). The stimuli generator is a BUG-FINDING ORACLE.
- 🚨 DIRECTOR PRINCIPLE 2026-06-09 (emphatic, standing): **fix parser bugs ASAP = HIGHEST priority** — "no point on cert-cover if a clearly identifiable parser bug shows up." (Defect A: FIXED by `GRAMMAR-WELLFORMED.H.8` `-0057`; the post-SV-PARSE-STRICT.2 delta WAS measured — spf 3→6 from the now-correct rejections, →0 after H.8.)
- 🧭 DIRECTOR-CONFIRMED BOUNDARY (standing): SV parser = **context-aware, SOUND gating** — rejects *provably*-undeclared type/nettype ids (current-TU + `--lib-in` facts), accepts when it can't prove (unresolved `import pkg::*`); cross-unit binding + type/width/param/generate = elaboration.
- locked_program (director 2026-06-08): ALL EXISTING PARSERS → `Done` (cert-coverage clean + UNKNOWN=0). **spf=0 on ALL wired grammars (H.8); seed-robust UNKNOWN=0: rtl_const_expr, json, svpp (H.9, 32/32 seeds).** Remaining UNKNOWN: regex 7 (H.10.1 cleared the 12 no-path), vhdl 31, rtl_frontend 75, SV 738 (+68 no-path/multi-entry).
- active_work_unit: `BRANCH-BROADCAST-FIX.5` next (re-apply the H.10.2.1 regex atomicity consumer; the attempt record + expected results are in `docs/tasks/GRAMMAR-WELLFORMED.md` H.10.2.1). in_flight_uncommitted: none. blockers: none.
- push: ~95 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
