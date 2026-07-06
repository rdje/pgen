# UNDEFINED-REF-DIAGNOSTICS — surface referenced-but-undefined rules at lint/codegen time (F6)

- Tree ID: `UNDEFINED-REF-DIAGNOSTICS`
- Status: `complete` (created 2026-07-06, session #47, spawned by `SEM-FINDINGS` — director directive
  2026-07-06; this tree owns finding **F6**, the residual diagnosability half. `.1` evidence + `.2`
  fix both landed session #50 — **F6 CLOSED**: `--lint-grammar` hard-gates undefined references
  (13/13 shipped grammars clean at 0) and codegen warns unconditionally at stub emission; the
  interpreter-parity half landed earlier in `PGEN-PARSE-HARNESS-0015`)

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

- `.2` — **FIX: the linter diagnostic (+ codegen warning) — `done` (2026-07-06, session #50,
  `PGEN-UNDEFINED-REF-DIAGNOSTICS-0002`).** All four `.1`-adjudicated pieces landed:
  - **`detect_undefined_references`** (`grammar_wellformedness.rs`) — the structural dual of
    `detect_unreachable_rules`; reuses the linter's own `collect_node_rule_refs` walker;
    deterministic (rule_order iteration, sorted refs); NEW `UndefinedReference` issue variant with
    a self-explaining `[error]` message (never-matching stub + typo hint).
  - **`NATIVE_UNRESOLVED_REFERENCE_BUILTINS`** — a pub associated const on `AstBasedGenerator`
    adjacent to the dispatch, consumed by the linter; the
    `native_unresolved_builtins_const_matches_dispatch` ORACLE test locks const ↔ dispatch in both
    directions (every const name emits non-stub tokens; a non-const probe emits exactly the bare
    stub; count pinned at 5).
  - **`run_grammar_lint` wiring** — summary field `undefined_references=N (error)`, `[error]`
    prints, HARD gate + problems entry. ⚠️ **`.2`-discovered design requirement (tools-first):**
    the detector runs on the **UNFILTERED bundle** — the first sweep ran on the filtered view and
    fired 7 FALSE positives on regex.ebnf (`unicode_escape` ×3, `…_relaxed` rules): the pcre2
    generation-default profile filter deliberately STRIPS `@profiles:["relaxed"]` rule DEFINITIONS
    while codegen always compiles the FULL grammar (profile selection = runtime guard, per the
    long-standing main.rs comment). The lint call site now keeps the unfiltered bundle (the
    cert-coverage pattern) and the detector sees exactly codegen's view → regex 0.
  - **The codegen warning** — unconditional `pgen_warn!` in `generate_unresolved_reference_methods`
    for every NON-native unresolved reference at the moment of stub emission.

## Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — `.1` (commit `587e3e6c`): the missing-`word` reproducer lints
  ALL-ZEROS exit 0 while `--generate-parser` silently emits the bare stub; the F6 origin incident
  rejected everything at `furthest_position=0` with nothing naming the cause.
- [x] **ROOT CAUSE (WHY + WHERE)** — `.1`: NO existing detector covers the class
  (`unreachable_rules=0` on the reproducer — an undefined ref is not a rule, reachability cannot
  see it); codegen's `_ =>` fallback (`ast_based_generator.rs` dispatch) silently emits
  `Err(Backtrack)` stubs; verified via `--lint-grammar` output + the emitted parser text.
- [x] **FIX** — linter tier per the `.1` adjudication (a static wellformedness property; no engine
  behavior change): the detector + const + oracle test + lint wiring + the codegen warning.
- [x] **ADDRESSED (verified)** — before→after on the reproducer: `--lint-grammar` silence →
  `undefined_references=1 (error)` + the exact `rule 'item' references UNDEFINED rule 'word'`
  message + **exit 1**; `--generate-parser` now prints the unconditional
  `[PGEN][WARN] … reference to UNDEFINED rule 'word'` at stub emission (verified live; silent on
  regex codegen — 0 warning lines).
- [x] **NO REGRESSION** — the fires/silent/builtin unit triple + wellformedness suite **43/43**;
  codegen units **68/68** (incl. the new oracle lock); the all-shipped-grammars lint sweep: **13/13
  grammars rc=0 with `undefined_references=0`** (ebnf/json/regex/return_annotation/rtl_const_expr/
  rtl_frontend/semantic_annotation/systemverilog/systemverilog_preprocessor/vhdl/scratch/
  builtin_return_annotation/builtin_semantic_annotation — regex byte-identically clean after the
  unfiltered-view fix); `verilog_2005_conformance_gate` GREEN (the one gate that consumes
  `--lint-grammar` — the lint lock + corpus matrix + profiled cert baseline deterministic across
  seeds 0/7/42); clippy strict-source GREEN (generated stage = pre-existing `eq_op` debt only);
  `mdbook_docs_gate` GREEN.
- [x] **LOCKSTEP** — book `grammar-wellformedness.md` (*Where PGEN stands* gains the
  undefined-reference gate paragraph with both design points); `TOOLBOX.md` §5.1 rewritten (full
  error-class list + the `furthest_position=0` WHEN-signature); `GRAMMAR-WELLFORMED` tree
  cross-link (Decisions); tree + TASK_TREE.md + live docs this commit.

## 6. Current Frontier

| # | Leaf | Status | Notes |
| --- | --- | --- | --- |
| 1 | `.1` (gap check + severity/ownership + shipped-grammar sweep) | `done` (2026-07-06 #50, `PGEN-UNDEFINED-REF-DIAGNOSTICS-0001`) | Gap live-confirmed; `[error]` hard-gate adjudicated; sweep = 0 live defects; allowlist + SSoT shape decided. |
| 2 | `.2` (linter diagnostic + tests + lockstep) | `done` (2026-07-06 #50, `PGEN-UNDEFINED-REF-DIAGNOSTICS-0002`) | F6 CLOSED: the gate lands hard at 0 across all 13 grammars; unfiltered-view requirement discovered + fixed in-leaf. **TREE COMPLETE.** |

## 7. Relationships

- Spawned by [`SEM-FINDINGS`](SEM-FINDINGS.md) (F6). Evidence base: `PARSE-HARNESS.6.2` — the
  missing-`word` trace + the emitted bare-stub read; the interpreter-parity half already landed in
  `PGEN-PARSE-HARNESS-0015`.
- Charter owner: `GRAMMAR-WELLFORMED` (cross-link on landing); single-source-of-truth precedent:
  `.5.2`'s `comment_arm_suppression_for_grammar`.
