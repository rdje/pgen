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

- `.1` — **EVIDENCE: the gap check + severity/ownership adjudication — `not-started` (frontier).**
  (a) Run `--lint-grammar` on the missing-`word` reproducer — confirm silence (the gap). (b) Read the
  linter's severity taxonomy + existing report classes (where does this diagnostic belong). (c) Sweep
  ALL shipped grammars for undefined refs TODAY (any hit = a live latent defect, surfaced
  immediately). (d) Enumerate the builtin allowlist from codegen's dispatch + decide the
  single-source-of-truth exposure shape. NO code.
- `.2` — **FIX: the linter diagnostic (+ optional codegen warning) — `not-started`.** Blocked on
  `.1`. The §2 diagnostic + the §4 battery + the enforced acceptance checklist + book/TOOLBOX
  lockstep.

## 6. Current Frontier

| # | Leaf | Status | Notes |
| --- | --- | --- | --- |
| 1 | `.1` (gap check + severity/ownership + shipped-grammar sweep) | `not-started` (**frontier**) | Tools-first; NO code. |
| 2 | `.2` (linter diagnostic + tests + lockstep) | `not-started` | Blocked on `.1`. Linter tier. |

## 7. Relationships

- Spawned by [`SEM-FINDINGS`](SEM-FINDINGS.md) (F6). Evidence base: `PARSE-HARNESS.6.2` — the
  missing-`word` trace + the emitted bare-stub read; the interpreter-parity half already landed in
  `PGEN-PARSE-HARNESS-0015`.
- Charter owner: `GRAMMAR-WELLFORMED` (cross-link on landing); single-source-of-truth precedent:
  `.5.2`'s `comment_arm_suppression_for_grammar`.
