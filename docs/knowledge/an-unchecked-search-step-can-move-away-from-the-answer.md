---
id: an-unchecked-search-step-can-move-away-from-the-answer
title: A diagnostic that iterates toward an explanation must measure each step — an unchecked step can move AWAY from the answer, and it then reports "unexplained" for inputs that are fully explained
answers:
  - "my adjudicator declares the missing names and re-parses until it converges — what can go wrong"
  - "why does my instrument report a candidate defect for a file that parses once I fix it by hand"
  - "the parser's trace says this name is missing — is it safe to just supply it"
  - "how do I stop PEG speculation noise from poisoning an automated diagnosis"
  - "I filtered keywords out of my candidate list using the grammar's own reserved-word regex — is that enough"
  - "how do I keep an iterative repair loop from making the parse worse"
  - "my residue bucket keeps changing size as I improve the instrument — which number do I publish"
tags: [instrument-soundness, corpus-adjudication, measurement, toolbox, systemverilog, peg]
date: 2026-08-11
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaf .13c.2 (the 47 -> 22 -> 20 residue history and the three corrections); stimuli/sv/adjudicate_dark_worklist.py (`reach()` + the admission test in `drive()`, and `declarable()`); stimuli/sv/subs/opentitan/hw/dv/sv/cip_lib/cip_lc_tx_cov_if.sv (the canonical row — its own port `rst_ni` was harvested as a missing name); docs/tasks/artifacts/sv_corpus_grad/dark_chained_only/adjudication.md
reverify: "F=stimuli/sv/subs/opentitan/hw/dv/sv/cip_lib/cip_lc_tx_cov_if.sv; { echo 'package uvm_pkg; endpackage'; echo 'package dv_base_reg_pkg; endpackage'; echo 'package lc_ctrl_pkg; parameter int On=1; parameter int Off=0; endpackage'; echo 'class mubi_cov; endclass'; cat $F; } > tmp/ok.sv && { echo 'class rst_ni; endclass'; cat tmp/ok.sv; } > tmp/worse.sv && ./rust/target/release/parseability_probe --parse systemverilog tmp/ok.sv --profile sv_2017 2>&1 | tail -1 && ./rust/target/release/parseability_probe --parse systemverilog tmp/worse.sv --profile sv_2017 2>&1 | tail -1"
---

A whole class of diagnostic works by **iterating toward an explanation**: parse, ask the parser what
it was missing, supply that, parse again, repeat until it either succeeds or stops improving. It is a
good design — every step is grounded in the parser's own testimony rather than in a guess — and it
is where the SV corpus adjudicator (`stimuli/sv/adjudicate_dark_worklist.py`) gets its verdicts.

The trap is that **the loop assumes its steps are improvements, and nothing in the loop checks that.**

## What it looks like when a step goes backwards

A PEG parser *speculates*: it tries a declaration reading on ordinary tokens constantly, so a trace
at `PGEN_TRACE_VERBOSITY=high` reports a missing fact for names that are not types at all. On
`cip_lc_tx_cov_if.sv` the harvest returned `rst_ni` — the interface's **own port**:

```systemverilog
interface cip_lc_tx_cov_if(input [3:0] val, input rst_ni);
```

Supplying it (`class rst_ni; endclass`) shadowed the port. The parse got **shorter**, the loop had no
way to notice, and the row was published as `RESIDUAL` — the bucket that means *candidate parser
defect* — for a file that parses perfectly once its two genuinely-missing packages are declared.

That is the failure mode worth naming: an unchecked repair step does not merely waste a round. It
**manufactures a finding**, because "the cheap explanations did not work" is exactly how an
adjudicator says *the parser is probably wrong here*.

## The fix is one line of discipline: admit by measurement

Give the loop an objective function and make every candidate earn its place against it:

```python
def reach(raw, wrapper, names, forms) -> int:
    ok, surface, furthest, _ = probe_text(compose(raw, wrapper, names, forms), trace=False)
    if ok:
        return 1 << 30
    return max(furthest or 0, surface or 0) - len(prelude_for(names, forms))

base = reach(raw, wrapper, names, forms)
admitted = {n for n in fresh if reach(raw, wrapper, names | {n}, forms) >= base}
```

Subtracting the prelude's own length matters: the objective must measure progress **through the
input**, not through the scaffolding the instrument added.

## The same shape, one layer down: the keyword filter

The instrument asked the grammar for its own reserved words rather than typing a list — the right
instinct, and still not sufficient. `reserved_non_keyword_identifier_sv` holds **173** spellings and
does not include `covergroup`, `endgroup`, `virtual`, `bind` or `new`, every one of which really does
appear in a trace as demanded-and-missing. A prelude containing `class covergroup; endclass` is a
**syntax error**, and it poisons every later probe of that row into a false `RESIDUAL`.

The cure was not a longer list. It was to ask the parser the question directly — each candidate's own
declaration block is parsed before it may enter a prelude:

```python
def declarable(name: str) -> bool:
    ok, _, _, _ = probe_text(prelude_for({name}, FORM_LADDER[-1]), trace=False)
    return bool(ok)
```

## How to tell you are in this trap

The tell is a **residue bucket that keeps moving as you improve the instrument** — here 47 → 22 → 20,
after 1 902 → … → 57 one level up ([[a-refutation-names-what-is-not-the-cause]]). Each intermediate
value was defensible and wrong, and each was caught the same way: by taking **one member of the
bucket** and hand-checking it before believing the bucket. When the last correction of three is
"my search could descend", the earlier numbers were never censuses.

Two corollaries worth carrying:

- **Publish the transformation, not just the verdict.** Every row in the adjudication artifact names
  the wrapper, the declaration form and the minimal name set that produced its verdict, so the claim
  is re-runnable and a wrong one is visible rather than merely wrong.
- **Minimize after you converge.** A loop that accumulates candidates ends up with more than the row
  needs; dropping each one and keeping only those whose removal costs the verdict turns "here is what
  the search collected" into "here is what the row requires".

Related: [[a-furthest-position-names-a-region-not-a-token]] (why the harvested region is full of
innocent names in the first place), [[a-check-whose-inputs-all-pass-has-not-been-tested]] and
[[a-gate-must-be-able-to-fail-and-able-to-run]] (the reproducer ratchet this adjudication is pinned
by), [[a-rising-pass-rate-is-not-evidence-of-correctness]] (why the *invalid-SV* half of an
adjudication has to be pinned too).
