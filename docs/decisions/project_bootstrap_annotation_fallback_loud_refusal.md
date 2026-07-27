# Bootstrap annotation fallback: loud refusal + explicit opt-in (never silent)

- **Type:** project
- **Date:** 2026-07-15 (session #121)
- **Owner:** RGX-0078 leaf `.5.i.1.t2` (the `.5.i.1.t1` queued enforcement)

## Context

The `.5.i.1.t1` regen-drift incident (2026-07-13): a feature-gated regen
(`ebnf_dual_run`-only `ast_pipeline`, non-bootstrap mode) **silently** fell back to the
hand-rolled bootstrap annotation parsers, which re-interpreted the annotation `null`
literal as the identifier/STRING `"null"` in the emitted parser tables (regex-only blast
radius: the 5 `max: null` open-ended quantifier bounds). The only signal was a
⚠️ warning routed through the module-local `eprintln!` shadow (`mod.rs:492`), which
forwards to **debug-gated** tracing — i.e. invisible at default verbosity (a severity-
doctrine violation in itself). Two enforcement prongs were queued in the incident leaf.

## Decision

1. **Faithful subset extension** — `UnifiedReturnAST::parse_bootstrap` recognizes the bare
   keyword `null` → `NullLiteral` → `serde_json::Value::Null`, exactly like the canonical
   generated path (`null_literal := 'null' -> {type: "null"}`). Agreement, not refusal,
   because the canonical generated path itself delegates node-text fallbacks to
   `parse_bootstrap` (`unified_return_ast.rs` Terminal/postfix/text fallbacks) — a refusal
   there could break canonical parsing.
2. **Loud refusal at the silent fallback** — in non-bootstrap mode without
   `--features generated_parsers`, BOTH annotation lanes (return + semantic) **hard-error**
   (`REFUSED: … needs the generated annotation backend …`) instead of silently degrading.
   The explicit opt-in **`PGEN_ALLOW_BOOTSTRAP_ANNOTATION_FALLBACK=1`** licenses the
   legitimate cold-bootstrap / chicken-and-egg recovery flow and prints a once-per-process,
   verbosity-independent (`std::eprintln!`, bypassing the debug-gated shadow) banner:
   artifacts are **NON-CANONICAL** until re-derived via `make -C rust focus_<grammar>` and
   verified by `make -C rust parse_harness_equivalence_gate`.

Canonical flows are structurally unaffected: `make focus_*` builds the pipeline with
`generated_parsers`; the annotation parsers regenerate under `--bootstrap-mode`
(`bootstrap_mode == true` is exempt — bootstrap is the *selected* backend there); the
`ebnf_dual_run`-only frontend binary performs only standalone raw-AST export (no
annotation parsing). Precedent for the hard refusal: `--report-certificate-coverage`
already bails without the feature.

## Consequences

- The `.5.i.1.t1` incident flow now fails fast (rc≠0, first annotation) instead of
  emitting drifted artifacts; the recovery flow requires a conscious opt-in and is
  stamped non-canonical.
- Measured (session #121): with the `null` fix, the opted-in fallback regen of regex is
  byte-identical to the canonical artifact modulo the embedded output-path strings — the
  incident's entire 24-site degradation class is closed at the root; the banner remains
  correct for the general class (e.g. the known `\"`-escape bootstrap gap,
  [feedback_annotation_no_dquote_escape](feedback_annotation_no_dquote_escape.md)).
- Canonical regen proven unaffected: 11/11 artifacts byte-identical at re-regen.
- **Latent finding (not fixed here):** the `eprintln!` shadows in `mod.rs` and
  `ast_based_generator.rs` route EVERY ⚠️ in those modules to debug-gated tracing —
  a standing severity-doctrine gap
  ([feedback_severity_never_gated_by_verbosity](feedback_severity_never_gated_by_verbosity.md));
  surfaced to the director for a dedicated re-classification leaf. **OWNER (cross-referenced
  2026-07-27): the `DIAG-SEVERITY` task tree** (`docs/tasks/DIAG-SEVERITY.md`), which owns the general
  severity-classification campaign for exactly these always-on-vs-trace-gated channels. Still live:
  171 `eprintln!` sites in `mod.rs`, 76 in `ast_based_generator.rs`.
