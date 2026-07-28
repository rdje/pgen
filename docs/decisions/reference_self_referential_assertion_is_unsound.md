---
name: reference-self-referential-assertion-is-unsound
description: REFERENCE (2026-07-28, session #220, CI-PARITY-GATE-ROT step c) — an assertion about a file cannot live inside that file as a literal: the positive form matches its own text and passes VACUOUSLY, the negative form trips on its own text and fails FALSELY. Measured: 8 of 13 probe arms failed on an audit tripping over its own source. The fix is to extract the declared value and compare it, with the extraction pattern anchored so it cannot self-match. DELIBERATELY NOT MECHANIZED — priced at exactly ONE site repo-wide.
metadata:
  node_type: memory
  type: reference
---

**The case.** `CI-PARITY-GATE-ROT` flipped `PGEN_CI_WORKFLOW_LOCAL_PREPARE` to default `true` and
needed to guard that default, because this repository had already watched the identical erosion once:
`PGEN_CLIPPY_GENERATED_STRICT` defaulted to `0`, was set by no gate, no aggregate and no workflow,
and left a 291 → 0 correctness win unguarded from the day it landed.

The obvious guard, inside `rust/scripts/ci_workflow_local_gate.sh`:

```bash
assert_file_not_contains "rust/scripts/ci_workflow_local_gate.sh" \
  'PREPARE_RAW="${PGEN_CI_WORKFLOW_LOCAL_PREPARE:-false}"'
```

This puts the forbidden literal **into the file it forbids it from**. Measured consequence:
**8 of 13 probe arms failed**, every one on `found 'PREPARE_RAW=…:-false…'` — the audit tripping over
its own source, before reaching the invariant any arm was testing.

The positive form is no better and is worse in the way that matters:

```bash
assert_file_contains "rust/scripts/ci_workflow_local_gate.sh" \
  'PREPARE_RAW="${PGEN_CI_WORKFLOW_LOCAL_PREPARE:-true}"'
```

It matches **its own text**, so it passes whatever the declaration says. That is a vacuous green —
the failure mode this project treats as worse than a red.

## The rule

> **An assertion about a file cannot live inside that file as a literal.** Either it matches itself
> (vacuous) or it trips on itself (false positive).

The `PGEN_CLIPPY_GENERATED_STRICT` precedent works only because the asserted file
(`clippy_on_rust_change.sh`) is a **different** file from the asserting one
(`ci_workflow_local_gate.sh`). That is not a stylistic difference; it is what makes the assertion
mean anything.

## The fix: compare the VALUE, not the text

```bash
prepare_default="$(grep -oE '^PREPARE_RAW="\$\{PGEN_CI_WORKFLOW_LOCAL_PREPARE:-[a-z]+' \
  "$ROOT_DIR/rust/scripts/ci_workflow_local_gate.sh" | sed 's/.*:-//' | head -n 1)"
[[ "$prepare_default" == "true" ]] || fail "…"
```

Two properties make it sound:

- the pattern is **anchored at column 0** on the declaration's own form, and the extraction line is
  indented, so neither the check nor its comment can match itself;
- after `:-` the pattern requires `[a-z]+`, and the check's own text has `[` there — so even an
  unanchored match would not fire.

## Deliberately NOT mechanized — and that is the priced decision

`GENERATED-LINT-CORRECTNESS.4`'s rule is to **price a candidate against the whole corpus before
adopting it**; its own chartered hypothesis, generalized from one clean sample, would have admitted
2 of 304 boxes. Priced here across every `rust/scripts/*.sh` and `scripts/*.sh`:

```
$ for f in rust/scripts/*.sh scripts/*.sh; do … self-targeting assert_* … done
rust/scripts/ci_workflow_local_gate.sh: 1 self-targeting assertion(s)
```

**Exactly one site repo-wide** — and it is the one already written in the sound form. A doctrine for
a single occurrence is over-mechanization, so this stays a reference note plus the in-place comment
at the site. Re-price before adopting it as a check: if the count reaches a handful, the mechanical
form is a grep for `assert_file_*` whose target path is the calling script's own basename.

Related: [[feedback_instrument_needs_ground_truth]] — the same session's larger instance of an
instrument measuring itself.
