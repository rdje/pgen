---
id: one-report-two-vintages-a-tool-with-two-inputs-can-be-half-stale
title: A tool with two inputs can be half-stale — and the half that IS fresh is what convinces you the report is current
answers:
  - "I regenerated a parser and my certificate coverage says the new rules are all UNKNOWN"
  - "why does a cert report show my new rule count but not witness any of the new rules"
  - "plannable-probe says parsed=true witnessed_target=false, what does that mean"
  - "how do I know whether ast_pipeline is stale relative to the parser I just generated"
  - "which pgen tools are affected by a stale binary and which are immune"
  - "my before/after control used the old grammar with the new parser — is that valid"
  - "how do I build a control for a tool that reads two different inputs"
  - "a measurement got worse right after my fix, how do I tell a real regression from an artifact"
tags: [instruments, controls, evidence, certificate-coverage, generated-parsers, staleness, claim-verification]
date: 2026-08-23
status: current
evidence: |
  GRAMMAR-WELLFORMED.H.16.6b (`PGEN-GRAMMAR-WELLFORMED-0173`), 2026-08-23. A grammar change added four
  rules to `grammars/semantic_annotation.ebnf` and regenerated the parser. Re-running
  `--report-certificate-coverage` returned `total=119 witness=84 UNKNOWN=35 spf=0/1/0` — the new rule
  count, with all four new rules unwitnessed, reading as "the edit made things worse".
  `rust/target/debug/ast_pipeline` was 00:13; `generated/semantic_annotation_parser.rs` was 00:23.
  `--report-certificate-coverage` takes `total` and the rule inventory from the `.ebnf` argument, but
  verifies witnesses through the generated parser the BINARY was linked against — so the report was
  half new and half ten minutes old. `PGEN_CERT_COVERAGE_DEBUG_PROBES=1` showed
  `[plannable-probe] rule='map_key' parsed=true witnessed_target=false`. Confirmed structurally before
  rebuilding (`parse_map_entry` contains `parser.parse_map_key()?`; all four `fn parse_map_key*` are
  emitted), then by re-measurement after `cargo build --features "generated_parsers ebnf_dual_run"
  --bin ast_pipeline` (1 m 20 s): `total=119 witness=90 UNKNOWN=29 spf=0/0/0`. The real delta was
  `UNKNOWN 31 → 29`, an improvement.
  A "control" run in the same breath was also mixed-vintage: feeding the pre-change `.ebnf` to the
  fresh binary verifies it through the LANDED parser, and duly reported `spf=0` at seed 7 for a
  grammar whose own parser rejects that sample.
reverify: "ls -la rust/target/debug/ast_pipeline generated/<grammar>_parser.rs   # the binary MUST be newer. If it is not: cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline  (~1m20s). For the release probe: cargo build --release --features generated_parsers --bin parseability_probe (~20 min — sequence it early)."
---

**Staleness is usually framed as "is my number old?". The harder case is a tool that reads *two*
inputs and is current on one of them** — because the fresh half produces exactly the signal that
persuades you the whole report is current.

`--report-certificate-coverage` has two inputs:

| what | comes from | fresh after a grammar edit? |
|---|---|---|
| `total`, the rule inventory, reach paths | the `.ebnf` you pass on the command line | **yes, immediately** |
| every witness verdict | the generated parser the BINARY was linked against | **only after you rebuild** |

Add four rules and the report says `total=119`. That is your new number. It is proof the tool read
your grammar — and it is not proof of anything about the witnesses sitting next to it.

## The per-rule signature reads as healthy, which is the trap

```text
[plannable-probe] rule='map_key' parsed=true witnessed_target=false sample="@ type : { \"8\" => \":^F@\" }"
```

`parsed=true`. The probe sample really does parse — the *old* parser handles string-keyed maps
perfectly well. It simply contains no `map_key` rule to record, so the target is never witnessed.

Contrast TOOLBOX §1.3's session-#218 form of the same root cause: `UNKNOWN=9, witness=0, every probe
parsed=false`. That reads as a broken grammar and sends you looking immediately. ⭐⭐ **The same
staleness has a benign-looking and an alarming-looking face, and the benign-looking one costs more.**

## Which tools are exposed, and which are immune

Narrower than it feels, and knowing the boundary saves the rest of your session's measurements:

- **Exposed** (they verify through the generated parser): `--report-certificate-coverage`, and the
  whole `parseability_probe` surface — `--parse`, `--parse-dump-ast`, `--trace-rules`, the rule-count
  dumps.
- **Immune** (they read the `.ebnf` and dispatch over the gen-AST, never linking a generated parser):
  `--interpret-parse` (TOOLBOX §1.5b), `--lint-grammar`, `--dump-gen-ast`, `--report-indirect-lr-plan`.

In the founding slice that boundary is why an accept-set ledger and two AST-identity sweeps stood
untouched while exactly one number had to be re-derived.

## When a tool has two inputs, a control must pin both

The obvious before-arm — hand the tool the *old* `.ebnf` and compare — is not a control. It pairs the
old grammar with the new parser, and it will happily report that the old grammar passes samples its
own parser rejects. Either pin both halves (grammar + parser + binary, all one vintage, which usually
means "the reading you took at the previous commit") or do not call it a control.

## Two habits that catch it

1. **`ls -la` the binary against the artifact before quoting any number that came from it.** One
   command, and it is the whole defence. Note the asymmetry in cost: the debug `ast_pipeline` is
   ~1 m 20 s, the release `parseability_probe` is **~20 minutes** — so start the rebuild before you
   start writing up, not after.
2. **Predict the number before you measure it.** The founding slice wrote down "four rules added,
   probably three stay UNKNOWN, so expect `31 → 34`" *and* the fallback it would trigger. The stale
   reading said 35 — close enough to the prediction to be waved through, which is precisely how an
   artifact becomes a finding. The real answer was 29. ⭐ A prediction that is wrong in the *good*
   direction is worth keeping: here it stopped a needless flattening of the grammar.

See also [[a-control-that-cannot-fail-is-not-a-control]],
[[an-instrument-can-be-green-in-both-arms-of-the-defect-it-describes]] and
[[a-measurement-that-cannot-name-its-instrument-cannot-be-checked-for-staleness]].
