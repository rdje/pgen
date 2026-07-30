---
id: ebnf-self-hosting-what-it-means
title: "What EBNF 'self-hosting' actually means in PGEN — a SPEC-DRIFT ALARM, not a bootstrap property; the 12/12 number is a RECOGNITION claim and nothing consumes the meta-parser's AST"
answers:
  - "what does EBNF self-hosting mean in PGEN"
  - "what is the 12/12 self-hosting number and what is it worth"
  - "does PGEN parse grammars with a PGEN-generated parser"
  - "why does it matter that generated/ebnf.rs can parse the tracked grammars"
  - "what breaks if the ebnf meta-grammar lags behind the frontend"
  - "who consumes the generated ebnf parser's AST"
  - "is ebnf.ebnf allowed to model an annotation payload opaquely"
  - "why is ebnf_frontend_dual_run_gate red and does it affect users"
tags: [ebnf, self-hosting, meta-grammar, doctrine, gate, spec-drift, architecture]
date: 2026-07-30
status: current
evidence: "docs/decisions/feedback_ebnf_meta_grammar_lockstep.md (director doctrine 2026-06-10, verbatim: 'production compilation flows through the hand-written frontend, so nothing user-facing breaks when the meta-grammar lags — only the dual-run differential can see it, and only if it is run'); the gap it was written for: SIX EBNF features shipped in frontend+codegen+docs and were never ported to grammars/ebnf.ebnf. CONSUMER CENSUS (LANG-CAPABILITY-AUDIT.10.5, session #228) — every consumer of the generated EbnfParser takes a VERDICT, never a payload: parser_registry.rs:627 parse_with_ebnf -> .is_ok(); :633 parse_with_ebnf_detail -> Result<(),String>; ebnf_frontend.rs:70 -> soft warn-only cross-check (hard only under PGEN_EBNF_FRONTEND_REQUIRE_GENERATED_VERIFY); bin/ebnf_dual_run_diff.rs:157 -> diagnostic report; parser_registry.rs:648 parse_with_ebnf_ast_json is the ONLY AST consumer and its only caller is parse_harness_equivalence.rs, the interpreter-vs-generated self-consistency oracle. `ebnf` also has NO docs/contracts/ integration contract and NO rust/test_data/ast_shape_contract/ manifest — the only tracked generated parser with neither."
reverify: "grep -rn 'EbnfParser\\|parse_full_grammar_file' rust/src --include=*.rs | grep -v '^rust/src/../../generated'; ls rust/test_data/ast_shape_contract/ | grep -c ebnf; grep -c 'ebnf' docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md"
---

**The property:** generate an EBNF parser from `grammars/ebnf.ebnf` and have *that* parser
consume every other `.ebnf` file. That is exactly what `ebnf_dual_run_diff` /
`ebnf_frontend_dual_run_gate` measure.

**Today it is a TEST, not the production path.** Compilation flows through the
**hand-written** `rust/src/ebnf_frontend.rs` (`scan_top_level_rules`) — see
[[ebnf-frontend-architecture]].

## ⭐ The stated endgame IS replacement — but it is doctrine, not a tracked plan

The hand-written frontend is officially **scaffolding**: `README.md:28` — *"Handwritten
parsers exist only as bootstrap scaffolding and never count as closure"*; the lockstep
decision record — *"the self-hosting doctrine (handwritten frontend = bootstrap
scaffolding only) rots invisibly"*. So "over time the generated parser replaces the
hand-written frontend" is the project's own intent. Three honest caveats:

1. **No task-tree owns it.** The doctrine states the intent; nothing schedules the work.
2. **Recognition ≠ replacement.** 12/12 says the generated parser *accepts* the files. To
   *replace* the frontend it must also emit the **raw_ast token envelope**
   `transform_from_raw_ast` consumes, and absorb the frontend's non-parsing jobs: include
   resolution (`scan_rules_with_includes`) + the cross-file duplicate-rule check,
   multi-line annotation grouping, and annotation-backend dispatch. None of that is in
   `ebnf.ebnf` today.
3. ⛔ **Full retirement is structurally impossible while `generated/` is untracked.** A
   cold clone has no `generated/ebnf.rs`, so *something* hand-written must parse
   `ebnf.ebnf` to seed it — which is why the include is cfg-gated on
   `has_generated_ebnf_parser` and why `regex_parser_bootstrap` exists. The reachable
   endgame is **demote to a seed-only bootstrap**, not delete.

⭐ **What the frontend does with an annotation is already opaque**, which tells you the
shape a replacement needs: `parse_semantic_annotation_text` (ebnf_frontend.rs:1410) strips
`@`, finds the **top-level colon** (brace/bracket/paren-depth *and* quote aware), and emits
`["semantic_annotation", [name, payload]]` with **payload an opaque `String`** — handed on
to the annotation backend ([[bootstrap-builtin-annotation-parsers]]). So a meta-grammar
that *delimits* the annotation and leaves the payload opaque models the authoritative code
faithfully; it is not a shortcut that self-hosting would later have to undo.

## ⛔ What "dual run" actually compares — three pairs, and NOT the one the name suggests

`ebnf_dual_run` / `ebnf_frontend_dual_run_gate` sound like *"run the hand-written frontend
and the `ebnf.ebnf`-derived parser and diff their output."* Measured, that comparison does
not exist. Three different pairs run, and each compares something weaker:

| pair | what is compared | where |
|---|---|---|
| hand-written frontend ↔ **generated `ebnf.rs`** | **verdict only** (`Ok`/`Err`), and **soft** — warns unless `PGEN_EBNF_FRONTEND_REQUIRE_GENERATED_VERIFY=1` | **live, on every grammar load** — `ebnf_frontend.rs:65-79` |
| ~~**Perl** `tools/ebnf_to_json.pl` ↔ hand-written frontend~~ | ~~**rule-NAME sets** — `sorted(set(names))` over `["rule", name]` heads; **not** bodies, tokens or ASTs~~ ⛔ **RETIRED** by `LANG-CAPABILITY-AUDIT.10.6`, files deleted by `.10.7`: measured, this arm was blind to 25 of `regex.ebnf`'s 276 rules and the gate passed that as `perl_under_reports` | *(gone)* |
| interpreter ↔ generated `ebnf.rs` | **byte-identical AST** ✅ | `parse_harness_equivalence` (`ebnf` is CERTIFIED) |

⇒ **The `ebnf.ebnf`-derived parser's OUTPUT is never compared with the hand-written
frontend's output, anywhere** — and retiring the Perl arm did not change that, it only removed
the *weakest* of the three pairs. Building that raw-AST differential is `.10.6` part 2.** Its AST *is* byte-compared — but against the **interpreter
running the same grammar**, which proves the two *engines* agree, not that `ebnf.ebnf`
describes what the frontend accepts. And the only output-level diff in the gate is against
the **Perl** frontend, on rule names alone — so two frontends could tokenize every rule
body differently and it would still report `parity`.

⚠️ **Correction (session #228):** an earlier revision of this card called the Perl arm *"a
frozen legacy reference"* on the strength of its mtime. **Running it refuted that** — it
still parses, and agrees exactly with the Rust frontend on `ebnf` (131/131) and `json`
(9/9); it is blind to **25 of 276** rules on `regex.ebnf` (the modern `code_*` family), and
the gate *passes* that as `perl_under_reports`. More importantly the opposite of "retired"
is true: `ci_workflow_local_gate.sh:810-823` **`assert_file_contains`** the Perl invocations
in three scripts, so removing them FAILS that gate — and `sota_exit_gate` runs two of those
scripts as **required** stages. Tracked as `LANG-CAPABILITY-AUDIT.10.6`.

**Consequence for the replacement endgame:** the instrument that would prove `ebnf.ebnf`
ready to replace the hand-written frontend — a raw-AST differential between the two — has
never been built. `12/12` says the spec still *reads* every grammar we ship; it says
nothing about whether it reads them the *same way*.

## What the number is worth today

`grammars/ebnf.ebnf` is the **formal declarative specification of PGEN's own input
language**. Generating a parser from it and running that parser over the grammars PGEN
ships is how that specification gets **tested**. "Self-hosting 12/12" therefore means:

> every EBNF construct the 12 tracked grammars actually use is modelled in the spec.

It is a **drift alarm**, and the drift it catches is silent by construction. The doctrine
exists because six EBNF features (per-branch return annotations, `**` flatten-spread,
`::N*` extraction-spread, `[> …]` lexical annotations, dotted and indexed `$refs`) were
each added to the frontend, codegen and docs — and never ported to the meta-grammar.
Nothing user-facing broke, so nobody noticed. Only the differential can see it.

## ⭐ It is a RECOGNITION claim, not a structural one

Every consumer of the generated `EbnfParser` takes a **verdict** — `.is_ok()`, a
`Result<(),String>`, a warn-only cross-check, a diagnostic report. The single AST consumer
exists only to feed the interpreter-vs-generated byte-identity oracle. **No product path
reads the meta-parser's AST**, and `ebnf` is the only tracked generated parser with
neither an integration contract nor an `ast_shape_contract` manifest.

⇒ the meta-grammar must **recognize** the language; it makes no claim about producing a
particular shape. Consequences worth knowing:

- **Modelling a sub-language opaquely is legitimate**, not a cheat — e.g. an annotation
  payload delimited but not structured. The payload's real spec lives with the annotation
  parsers ([[bootstrap-builtin-annotation-parsers]]); duplicating it in the meta-grammar is
  what lets the two copies diverge.
- **A red `ebnf_frontend_dual_run_gate` is a spec defect, not a user-facing outage.** It
  says the declarative spec no longer describes what the implementation accepts.
- **A green one is not a correctness proof of the frontend** — the two are only compared on
  accept/reject over the tracked corpus.

## Honest bound

Because nothing consumes the AST, a construct can be *recognized* by the meta-grammar with
a structure that means nothing. The number is only as strong as its definition:
**"the spec still reads every grammar we ship."** That is genuinely worth holding — it is
just not a bootstrap guarantee, and should never be reported as one.
