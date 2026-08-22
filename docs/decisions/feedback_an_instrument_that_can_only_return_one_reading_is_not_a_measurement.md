---
name: feedback_an_instrument_that_can_only_return_one_reading_is_not_a_measurement
description: "DISCIPLINE (2026-08-22, SV-CORPUS-GRAD.13c.2x.9 / PGEN-SV-CORPUS-GRAD-0274) — before a probe's reading is cited, prove the probe can return the OTHER reading. `SV-CORPUS-GRAD.13c.2x.9` published `SEVEN CARRIERS, ALL PARSE, NONE COMMITS` and built two successive root-cause labels on it; the scoring column was `grep -c <rule-name> <ast.json>`, PGEN's AST is ANNOTATION-shaped and carries only what a `->` return annotation writes, and the rule scored has NO return annotation — so the column was pinned at 0 for every possible input and could never have reported a commit. Re-measured with `--dump-rule-outcome-counts-json` (TOOLBOX 3.5, committed counts by rule NAME), the plainest two-line carrier reports `committed=1`, and NINE of thirteen carriers commit the rule. ⛔ Every surrounding discipline held: the carriers were tracked, re-runnable, deterministic, pinned beside the script and honest about not proving unreachability. None of them asks whether the needle can move. The cheap guard is one RED control — an input the instrument MUST score differently — and it costs one file."
id: feedback_an_instrument_that_can_only_return_one_reading_is_not_a_measurement
title: "An instrument that can only return one reading is not a measurement — ship a RED control with it"
date: 2026-08-22
evidence: docs/tasks/artifacts/sv_corpus_grad/kupi_shadowing/probe.sh + kupi_arms.py + probe.txt (four arms, thirteen carriers, r1/r2 are the RED controls); docs/tasks/SV-CORPUS-GRAD.md leaf .13c.2x.9 WHAT LANDED (PGEN-SV-CORPUS-GRAD-0274); the retracted readings are PGEN-SV-CORPUS-GRAD-0272 (label "ordered-choice shadowing", refuted by -0273) and -0273 (label "an earlier alternative claims the bytes / unreachable for a bare name", refuted by -0274)
reverify: "bash docs/tasks/artifacts/sv_corpus_grad/kupi_shadowing/probe.sh | awk '/^(c9|r1)/ {print $1, $NF}' | sort | tr '\\n' ' '   # must print `c9 COMMITS:in-tree r1 predicate-false` — the same instrument scoring the SAME syntax two different ways is the whole point; if both rows ever agree, the needle is stuck again"
answers:
  - "how do I know my probe is measuring anything"
  - "why did a tracked deterministic re-runnable probe still produce a wrong root cause"
  - "what is a RED control and why does every instrument need one"
  - "can I grep an AST dump for a rule name to see whether the rule fired"
  - "which counter says whether a PGEN rule committed"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-22
---

## The rule

Before a probe's reading is cited for anything, run the probe on an input it **must** score
differently, and pin that row beside the others.

A probe that has only ever produced one value has not been shown to be sensitive to the thing it
claims to measure. "It is tracked / deterministic / re-runnable / honest about its limits" are all
true of a needle glued to zero.

## What it cost here

`SV-CORPUS-GRAD.13c.2x.9` scored thirteen SystemVerilog carriers for whether
`known_unscoped_property_identifier` — SystemVerilog's last certificate-union `UNKNOWN` — commits,
using:

```bash
grep -c 'known_unscoped_property_identifier' "$OUT/$b.json"
```

PGEN's AST is **annotation-shaped**: a node carries the `kind:`/field names a `->` return annotation
writes, never rule names. That rule has no return annotation. The column was therefore `0` for every
input in the language, and the table it produced —

```text
carrier      parses  commits_known_unscoped_property_identifier
c1           1       0
…            1       0
```

— read as seven independent negatives when it was one stuck instrument reported seven times.
Two successive root-cause labels were built on it (*ordered-choice shadowing*, then *an earlier
alternative claims the bytes, so the rule is unreachable for a bare name*), published to the task
tree, the changelog, the commit messages and the live book. Both are wrong.

One command with an instrument that **can** move settles it:

```bash
parseability_probe --parse systemverilog c1.sv --profile sv_2017 \
  --dump-rule-outcome-counts-json out.json
# rule_entry_counts     [known_unscoped_property_identifier] = 2
# rule_committed_counts [known_unscoped_property_identifier] = 1
```

A two-line carrier commits the rule. Nine of the thirteen do.

## Why the usual disciplines did not catch it

The probe was tracked in the repository, re-runnable by anyone, deterministic across runs, its output
pinned beside it, and its header stated plainly that seven negatives are not a proof of
unreachability. Every one of those is a real discipline and every one was satisfied. **None of them
asks whether the instrument can produce the other answer**, which is the only question that would
have failed.

## The guard, and it is cheap

Ship a **RED control**: an input the instrument must score differently, in the same table, run by the
same command. Here that is two extra files —

| control | same syntax as the positive | must score |
|---|---|---|
| `r1` — the property name is **undeclared** | `zz(not x)` vs `pr(not x)` | not committed (the `has_fact` gate rejects) |
| `r2` — the name is **dotted** | `blk.pr(not x)` | not committed (a sibling alternative takes it) |

If the positive row and the RED row ever agree, the needle is stuck and every other row is void.

## Related

- [[feedback_a_control_that_passes_under_both_hypotheses_is_not_evidence]] — the adjacent failure: an
  instrument that *does* move, on a case where both hypotheses predict the same movement. That one is
  about the case; this one is about the instrument.
- [[feedback_verify_a_claim_three_ways_before_publishing_it]] — leg 2 (falsify against an oracle you
  did not build, and prove the control can go RED) is exactly the leg that was missing.
- `TOOLBOX.md` Protocol A cause map — now names `--dump-rule-outcome-counts-json` and its C3-B
  committed semantics as the instrument for "did rule R commit", and forbids scoring a rule by
  grepping an AST dump for its name.
