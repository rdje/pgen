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
- latest_commit: (this commit) `PGEN-GRAMMAR-WELLFORMED-0059` (leaf `GRAMMAR-WELLFORMED.H.9`, svpp grammar) — **the H.7.2-discovered svpp PARSER BUG FIXED**: LRM-22.5.1-faithful `macro_default_paren_group` (balanced parens; commas legal only inside) replaces the bare lparen/rparen default atoms; `` `define M(a=x) y`` now parses structured formals. **svpp cert-coverage sweep 32/32 seeds fully_certified + spf=0 — svpp seed-robustly UNKNOWN=0 (H.5.4 closed completely).** svpp release 1.0.6→1.0.7, schema 4→5, inventory 67→68, ledger SVPP-0004, contract+book+HTML lockstep. PRIOR: `-0058` (H.7.2 reach pass: regex 98→19, vhdl 69→31, rtl_fe 133→75, SV 1134→738), `-0155`, `-0057`, `-0154`.
- 🧭 PNT batch (2026-06-09/10, director: "PNT yourself — do not involve me unless you can't decide"): (1) ✅ SV-EXH-PROOF.8 E0063 (`-0154`); (2) ✅ H.8 Defect A (`-0057`); (3) ✅ SV-EXH-PROOF.3.3.5 auto-gate matcher (`-0155`); (4) ✅ H.7.2 constructive-reach (`-0058`, Q1–Q3 agent-resolved, tree Decisions); (5) ✅ H.9 svpp parser bug (`-0059`). NEXT candidates: per-grammar UNKNOWN drives (regex 19 [+12 no-path], vhdl 31, rtl_frontend 75 [20 gen-failures to inspect], SV 738 [+68 no-path = multi-entry/number-literal family, A2.1 territory]) — each likely needs no-path adjudication (linter) + routed-elsewhere root-causing (more H.9-class parser-bug candidates!); also auto-gate coverage-sample warnings (regex 7, rtl ternary/unary).
- 🧠 PROOF-DISCIPLINE (director challenge 2026-06-09, standing): when claiming a rule is "dead"/unreachable, PROVE it — `--lint-grammar unreachable_rules=0` is multi-entry-LENIENT (does NOT prove single-entry reachability); use the static ref-graph closure + the `--gap-report-json` oracle (`reachable:false unreachable_from_entry`). Reachability is PER-GRAMMAR (svpp `trivia` dead; a same-named `trivia` is ALIVE in `rtl_frontend.ebnf`). Don't call same-lineage corroboration "independent". KM `prove-rule-dead-or-reachable`.
- 🧭 DIRECTOR DIRECTIVE 2026-06-09 (3×, standing): **PGEN is NOT bug-free / NOT feature-complete — it is evolving toward both.** When a general, parser-agnostic enhancement to semantic annotations / the pipeline is needed, **discuss → own → design carefully → implement** (goal: parse as many languages as humanly possible — C, JS, HTML, Perl6, …). The stimuli generator is a BUG-FINDING ORACLE.
- 🚨 DIRECTOR PRINCIPLE 2026-06-09 (emphatic, standing): **fix parser bugs ASAP = HIGHEST priority** — "no point on cert-cover if a clearly identifiable parser bug shows up." (Defect A: FIXED by `GRAMMAR-WELLFORMED.H.8` `-0057`; the post-SV-PARSE-STRICT.2 delta WAS measured — spf 3→6 from the now-correct rejections, →0 after H.8.)
- 🧭 DIRECTOR-CONFIRMED BOUNDARY (standing): SV parser = **context-aware, SOUND gating** — rejects *provably*-undeclared type/nettype ids (current-TU + `--lib-in` facts), accepts when it can't prove (unresolved `import pkg::*`); cross-unit binding + type/width/param/generate = elaboration.
- locked_program (director 2026-06-08): ALL EXISTING PARSERS → `Done` (cert-coverage clean + UNKNOWN=0). **spf=0 on ALL wired grammars (H.8); seed-robust UNKNOWN=0: rtl_const_expr, json, svpp (H.9, 32/32 seeds).** Remaining UNKNOWN: vhdl 31, regex 19 (+12 no-path), rtl_frontend 75, SV 738 (+68 no-path/multi-entry).
- active_work_unit: PNT loop continues — next = per-grammar UNKNOWN→0 drives (start with the routed-elsewhere/no-path adjudication on regex or vhdl — smallest residuals; each routed-elsewhere cluster is a potential H.9-class parser bug). in_flight_uncommitted: none. blockers: none.
- push: ~90 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
