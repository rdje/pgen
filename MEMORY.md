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
- latest_commit: (this commit) `PGEN-BRANCH-BROADCAST-FIX-0005` (leaf `.5` + closes `GRAMMAR-WELLFORMED.H.10.2.1`) — **✅ BRANCH-BROADCAST-FIX TREE COMPLETE (.1–.5)**: the H.10.2.1 regex atomicity edits re-applied exactly as recorded (whole-body group `-> $text` on `short_prop_letter`/`ascii_restrict_modifier`); **regex cert-coverage `UNKNOWN 7→5`** (198/193/spf=0, seeds 0/7/42); **`restrict:"D"` = the live-fire proof of both engine fixes**; A/B byte-identical; manifest 167→186; oracle gate PASS ⇒ NO bump; dual-feature 768/0; book wellformedness chapter synced (98→19→7→5 arc). PRIOR: `-0004` (ship/ledger RETANN-0001+SEMANN-0001), `-0003` ($text span), `-0002` (broadcast remap), `-0001` (root-cause).
- 🧭 PNT batch (2026-06-09/10, director: "PNT yourself — do not involve me unless you can't decide"): (1-10) ✅ … `-0002`, `-0003`, `-0004`; (11) ✅ `.5` (`-0005`, this — tree COMPLETE). NEXT = **the per-grammar UNKNOWN→0 drives (the locked program)**: regex H.10.2 survivors = 5 (terminal-selection trio `letter_no_upper_e`/`unicode_char`/`quoted_class_literal_escaped_char` + store-gated pair `numeric_backreference`/`backreference_digits` [needs the fact-emitting prelude, B2/C1/C2 lane]), vhdl 31, rtl_frontend 75, SV 738 (+68 no-path adjudications).
- 🧠 PROOF-DISCIPLINE (director challenge 2026-06-09, standing): when claiming a rule is "dead"/unreachable, PROVE it — `--lint-grammar unreachable_rules=0` is multi-entry-LENIENT (does NOT prove single-entry reachability); use the static ref-graph closure + the `--gap-report-json` oracle (`reachable:false unreachable_from_entry`). Reachability is PER-GRAMMAR (svpp `trivia` dead; a same-named `trivia` is ALIVE in `rtl_frontend.ebnf`). Don't call same-lineage corroboration "independent". KM `prove-rule-dead-or-reachable`.
- 🧭 DIRECTOR DIRECTIVE 2026-06-09 (3×, standing): **PGEN is NOT bug-free / NOT feature-complete — it is evolving toward both.** When a general, parser-agnostic enhancement to semantic annotations / the pipeline is needed, **discuss → own → design carefully → implement** (goal: parse as many languages as humanly possible — C, JS, HTML, Perl6, …). The stimuli generator is a BUG-FINDING ORACLE.
- 🚨 DIRECTOR PRINCIPLE 2026-06-09 (emphatic, standing): **fix parser bugs ASAP = HIGHEST priority** — "no point on cert-cover if a clearly identifiable parser bug shows up." (Defect A: FIXED by `GRAMMAR-WELLFORMED.H.8` `-0057`; the post-SV-PARSE-STRICT.2 delta WAS measured — spf 3→6 from the now-correct rejections, →0 after H.8.)
- 🧭 DIRECTOR-CONFIRMED BOUNDARY (standing): SV parser = **context-aware, SOUND gating** — rejects *provably*-undeclared type/nettype ids (current-TU + `--lib-in` facts), accepts when it can't prove (unresolved `import pkg::*`); cross-unit binding + type/width/param/generate = elaboration.
- locked_program (director 2026-06-08): ALL EXISTING PARSERS → `Done` (cert-coverage clean + UNKNOWN=0). **spf=0 on ALL wired grammars (H.8); seed-robust UNKNOWN=0: rtl_const_expr, json, svpp (H.9, 32/32 seeds).** Remaining UNKNOWN: regex 7 (H.10.1 cleared the 12 no-path), vhdl 31, rtl_frontend 75, SV 738 (+68 no-path/multi-entry).
- active_work_unit: `GRAMMAR-WELLFORMED.H.10.2` next (regex survivors drive: the terminal-selection trio is the natural next leaf — probes parse-but-route-elsewhere, the minimal terminal choice misses the target alternative; see the H.10.2 pool record). in_flight_uncommitted: none. blockers: none.
- push: ~96 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
