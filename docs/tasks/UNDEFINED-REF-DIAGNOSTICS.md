# UNDEFINED-REF-DIAGNOSTICS — surface referenced-but-undefined rules at lint/codegen time (F6)

- Tree ID: `UNDEFINED-REF-DIAGNOSTICS`
- Status: `active` (created 2026-07-06, session #47, spawned by `SEM-FINDINGS` — director directive
  2026-07-06; this tree owns finding **F6**, the residual diagnosability half)

## 1. The finding (tool-established, PARSE-HARNESS.6.2 session #47)

A rule that REFERENCES an undefined rule compiles **silently** into a parser that can never accept
through that path:

- Codegen synthesizes native methods for a small builtin set and a bare `Err(ParseError::Backtrack)`
  **stub** for every OTHER unknown reference (`generate_unresolved_reference_method`,
  `ast_based_generator.rs:833-946`) — no `furthest_position` bump, no memo, no rule context (the
  no-preamble fact the `.6.2` interpreter now mirrors; the PARITY half of F6 is DONE in
  `PGEN-PARSE-HARNESS-0015`).
- MEASURED: the session-#47 `sem_count_gate` grammar accidentally omitted its `word` rule —
  `ast_pipeline --generate-parser` succeeded with NO warning; every parse rejected at
  `furthest_position=0` with nothing pointing at the cause; the full trace was needed to find the
  bare stub. An undefined reference makes every referencing rule DEAD — squarely the
  GRAMMAR-WELLFORMED "unreachable = DEFECT" contract, yet nothing gates it.

## 2. The elegant fix (design; confirm the gap in `.1`, implement in `.2`)

**A first-class diagnostic** (linter tier — the right tier by construction: this is a static
grammar-wellformedness property, no engine behavior change):

- `--lint-grammar` reports every `rule X references undefined rule 'Y' (codegen will emit a
  never-matching stub)` — severity ERROR-by-default is the honest reading of the wellformedness
  contract (a never-matching stub is a stronger defect than unreachable), but `.1` checks the
  linter's existing severity taxonomy + how the SV/regex families would grade (regex DEPENDS on
  intentional unresolved builtins) before locking severity.
- The **builtin allowlist** is excluded and must be derived from codegen's OWN dispatch (the
  `.5.2` single-source-of-truth pattern — expose/reuse the match set of
  `generate_unresolved_reference_method`: `true`/`false`/`semantic_annotation`/`builtin_any_char`/
  `builtin_ascii_char`), so the linter can never drift from what codegen actually synthesizes.
- Adjudicate in `.1` whether `--generate-parser` should ALSO warn (belt-and-braces at the moment of
  stub emission) — cheap, same allowlist.
- Severity discipline per [[feedback_severity_never_gated_by_verbosity]]: the diagnostic emits
  unconditionally, verbosity governs INFO only.

## 3. Ownership note

This is linter-completeness work; `GRAMMAR-WELLFORMED` is the charter owner. DECIDED at spawn time:
implement HERE (a small, self-contained tree spawned by the director's SEM-FINDINGS directive) and
cross-link from `GRAMMAR-WELLFORMED` when it lands — re-open the call in `.1` only if the linter
internals demand deeper GRAMMAR-WELLFORMED integration (e.g. the reachability pass already computes
the reference graph this needs).

## 4. Verification battery (leaf `.2`)

Linter suite (a fires-on-undefined-ref + silent-on-defined + silent-on-builtin test triple) + every
shipped grammar stays lint-clean (`--lint-grammar` over `grammars/*.ebnf` — any NEW hit is a real
latent defect to surface, not to suppress) + `sota_exit_gate`/`ci_workflow_local_gate` untouched +
clippy + `mdbook_docs_gate`. Book lockstep: the linter chapter (`grammar-wellformedness.md`) gains
the diagnostic; TOOLBOX `--lint-grammar` entry updated.

## 5. Leaves

- `.1` — **EVIDENCE: the gap check + severity/ownership adjudication — `done` (2026-07-06,
  session #50, `PGEN-UNDEFINED-REF-DIAGNOSTICS-0001`).** All four parts tool-backed:

  **(a) Gap CONFIRMED live.** Reproducer (`program := item+` / `item := "(" word ")"` — `word`
  undefined): `--lint-grammar` reports ALL ZEROS and exits 0 —
  `grammar lint: 'undefref' (2 rules) — left_recursive=0 … non_terminating=0 (error),
  ordered_choice_shadowing=0 (error), … unreachable_rules=0 (error), unbound_fact_kinds=0 (error),
  nullable_repetition=0 (warning), profile_orphans=0 (error…)`. `--generate-parser` on the same
  grammar succeeds silently and emits the bare `Err(Backtrack)` stub for `word` (verified in the
  emitted parser text). Notably `unreachable_rules=0` — an undefined ref cannot trip reachability
  (it is not a rule), so NO existing detector covers this class.

  **(b) Severity taxonomy + placement (from `run_grammar_lint`, `main.rs:3176+`):** classes are
  `[error]` (HARD-gate — the lint exits nonzero: non_terminating, profile_orphans,
  ordered_choice_shadowing, unreachable_rules, unbound_fact_kinds), `[warn]` (non-gating:
  nullable_repetition), `[note]` (never gates: always_succeeds), `[info]` (left-recursion).
  ADJUDICATED: **`[error]`, hard-gating** — a referenced-but-undefined rule makes every
  referencing path NEVER-match (strictly stronger than `unreachable_rules`, which is error), and
  the (c) sweep proves all shipped grammars are clean at 0, so the hard gate binds immediately
  (the F1/A1b precedent: authored grammars clean → lock at 0). The regex-family concern is nil by
  construction (the allowlist is excluded). Placement: a NEW detector
  `detect_undefined_references(g, order)` in `grammar_wellformedness.rs` — the structural DUAL of
  `detect_unreachable_rules` (defined-but-unreferenced vs referenced-but-undefined).

  **(c) Shipped-grammar sweep: ZERO live defects.** Oracle-style sweep of what codegen ACTUALLY
  emitted — grep all 11 on-disk generated parsers for the bare-stub method shape
  (`pub fn parse_X … { Err(ParseError::Backtrack { position: self.position }) }`), pattern
  POSITIVE-CONTROLLED on the reproducer's emitted `word` stub: **bare_stubs=[] in all 11**
  (json/regex/return_annotation/rtl_const_expr/rtl_frontend/scratch/semantic_annotation/
  systemverilog/systemverilog_preprocessor/vhdl/ebnf). Natives legitimately in use:
  regex → `builtin_any_char` + `builtin_ascii_char`; semantic_annotation + ebnf →
  `semantic_annotation`.

  **(d) Allowlist + SSoT exposure shape.** Codegen's dispatch
  (`generate_unresolved_reference_method`, `ast_based_generator.rs:884–997`) natively synthesizes
  EXACTLY `true`, `false`, `semantic_annotation`, `builtin_any_char`, `builtin_ascii_char`; the
  `_ =>` fallback (`:990–996`) is the bare stub. DECIDED for `.2`: (i) a
  `pub(crate) const NATIVE_UNRESOLVED_REFERENCE_BUILTINS: &'static [&'static str]` adjacent to
  the dispatch; (ii) an ORACLE-style unit test locking const ↔ dispatch (every const name's
  emitted tokens ≠ the bare-stub tokens; a non-const probe name's tokens == the bare-stub tokens)
  so the two can never drift; (iii) the linter detector consumes the const. ALSO ADJUDICATED:
  yes to the belt-and-braces codegen warning at stub-emission time, unconditional per
  [[feedback_severity_never_gated_by_verbosity]]. OWNERSHIP re-check: stays HERE — the new
  detector is a small standalone pass (references − defined − allowlist); no deeper
  GRAMMAR-WELLFORMED integration needed (cross-link on landing).

- `.2` — **FIX: the linter diagnostic (+ codegen warning) — `not-started` (frontier).** Blocked on
  `.1` → now UNBLOCKED. The §2 diagnostic at the `.1`-adjudicated severity + the §4 battery + the
  enforced acceptance checklist + book/TOOLBOX lockstep.

## 6. Current Frontier

| # | Leaf | Status | Notes |
| --- | --- | --- | --- |
| 1 | `.1` (gap check + severity/ownership + shipped-grammar sweep) | `done` (2026-07-06 #50, `PGEN-UNDEFINED-REF-DIAGNOSTICS-0001`) | Gap live-confirmed; `[error]` hard-gate adjudicated; sweep = 0 live defects; allowlist + SSoT shape decided. |
| 2 | `.2` (linter diagnostic + tests + lockstep) | `not-started` (**frontier**) | Linter tier; `detect_undefined_references` + const + oracle test + codegen warning. |

## 7. Relationships

- Spawned by [`SEM-FINDINGS`](SEM-FINDINGS.md) (F6). Evidence base: `PARSE-HARNESS.6.2` — the
  missing-`word` trace + the emitted bare-stub read; the interpreter-parity half already landed in
  `PGEN-PARSE-HARNESS-0015`.
- Charter owner: `GRAMMAR-WELLFORMED` (cross-link on landing); single-source-of-truth precedent:
  `.5.2`'s `comment_arm_suppression_for_grammar`.
