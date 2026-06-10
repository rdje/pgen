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
- latest_commit: (this commit) `PGEN-BRANCH-BROADCAST-FIX-0001` (leaf `BRANCH-BROADCAST-FIX.1`, pure docs) — **🐛 LIVE ENGINE DEFECT PAIR DISCOVERED + pinned (new tree BRANCH-BROADCAST-FIX): (A) the 2026-05-14 inner→outer branch remap REGRESSED #38's parens-group trailing-annotation broadcast for WHOLE-BODY groups** (`RULE = (A|B) -> ann` binds branch 0 only; SHIPPED impact: `return_annotation` `string_literal` — `'x'` raw Sequence vs `"x"` typed, runtime-proven); **(B) branch-level `$text` slices an EMPTY span in tournament rules** (rollback precedes the transform; `ast_return_transform.rs:91` + the arm's `parser.position = parse_start`). Found by H.10.2.1's declarative atomicity attempt (A/B caught `restrict:""`); attempt cleanly REVERTED, baseline re-verified byte-identical (regex UNKNOWN=7 spf=0 @ `-0060`). H.10.2.1 now `blocked` on `.2`+`.3` (its `.5` re-applies). PRIOR: `-0060` (H.10.1 regex dead-rule removal: UNKNOWN 19→7), `-0059` (H.9), `-0058` (H.7.2).
- 🧭 PNT batch (2026-06-09/10, director: "PNT yourself — do not involve me unless you can't decide"): (1-5) ✅ `-0154`/`-0057`/`-0155`/`-0058`/`-0059`; (6) ✅ H.10.1 (`-0060`); (7) ✅ BRANCH-BROADCAST-FIX.1 (`-0001`, this). NEXT = **`BRANCH-BROADCAST-FIX.2` (the remap fix — "fix parser bugs ASAP" applies: a SHIPPED parser mis-shapes single-quoted strings)** → `.3` ($text span) → `.4` (ship+ledger) → `.5` (re-apply H.10.2.1). THEN back to the per-grammar UNKNOWN drives: regex H.10.2 survivors (terminal-selection trio + store-gated pair), vhdl 31, rtl_frontend 75, SV 738.
- 🧠 PROOF-DISCIPLINE (director challenge 2026-06-09, standing): when claiming a rule is "dead"/unreachable, PROVE it — `--lint-grammar unreachable_rules=0` is multi-entry-LENIENT (does NOT prove single-entry reachability); use the static ref-graph closure + the `--gap-report-json` oracle (`reachable:false unreachable_from_entry`). Reachability is PER-GRAMMAR (svpp `trivia` dead; a same-named `trivia` is ALIVE in `rtl_frontend.ebnf`). Don't call same-lineage corroboration "independent". KM `prove-rule-dead-or-reachable`.
- 🧭 DIRECTOR DIRECTIVE 2026-06-09 (3×, standing): **PGEN is NOT bug-free / NOT feature-complete — it is evolving toward both.** When a general, parser-agnostic enhancement to semantic annotations / the pipeline is needed, **discuss → own → design carefully → implement** (goal: parse as many languages as humanly possible — C, JS, HTML, Perl6, …). The stimuli generator is a BUG-FINDING ORACLE.
- 🚨 DIRECTOR PRINCIPLE 2026-06-09 (emphatic, standing): **fix parser bugs ASAP = HIGHEST priority** — "no point on cert-cover if a clearly identifiable parser bug shows up." (Defect A: FIXED by `GRAMMAR-WELLFORMED.H.8` `-0057`; the post-SV-PARSE-STRICT.2 delta WAS measured — spf 3→6 from the now-correct rejections, →0 after H.8.)
- 🧭 DIRECTOR-CONFIRMED BOUNDARY (standing): SV parser = **context-aware, SOUND gating** — rejects *provably*-undeclared type/nettype ids (current-TU + `--lib-in` facts), accepts when it can't prove (unresolved `import pkg::*`); cross-unit binding + type/width/param/generate = elaboration.
- locked_program (director 2026-06-08): ALL EXISTING PARSERS → `Done` (cert-coverage clean + UNKNOWN=0). **spf=0 on ALL wired grammars (H.8); seed-robust UNKNOWN=0: rtl_const_expr, json, svpp (H.9, 32/32 seeds).** Remaining UNKNOWN: regex 7 (H.10.1 cleared the 12 no-path), vhdl 31, rtl_frontend 75, SV 738 (+68 no-path/multi-entry).
- active_work_unit: `BRANCH-BROADCAST-FIX.2` next (engine remap fix; design constraints in the tree — keep the 2026-05-14 remap's patterns (A)–(D) green). in_flight_uncommitted: none. blockers: none.
- push: ~92 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
