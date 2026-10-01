---
id: a-generator-change-is-accepted-on-every-familys-samples
title: A stimuli-generator change is accepted on every family's samples — and its control must be a generator that predates the mechanism, not one that shares it
answers:
  - "which families must I check after changing the stimuli generator"
  - "why did a parser family's computed status drop after a stimuli generator change"
  - "how do I prove a generator change did not move another grammar's samples"
  - "why does the generator bake a newline after a token in the preprocessor grammar"
  - "why did my probe's control arm not see the regression"
  - "what does parseability_parser_rejections_total greater than zero mean for a family"
tags: [stimuli_generator, acceptance, control, whitespace_sensitive, certificate-coverage, family-status, regression]
date: 2026-10-01
status: current
evidence: "ENGINE-UNIVERSAL-SERVICES.119 (PGEN-ENGINE-UNIVERSAL-SERVICES-0196): .98 (C) was accepted on the SystemVerilog certificate alone and baked a newline after the preprocessor grammar's three comment-aware text tokens, where a newline ends the directive; the preprocessor certificate read sample_parse_failures 0/0/0 on the pre-.98 generator and 1/0/1 after, five generated samples were rejected by the family's own parser and its computed status fell a tier — found two days later by a release ceremony's family umbrella; .98's correction (-0190) had printed the same 1/0/1 as its CONTROL arm and called it the grammar's own residue, because both of its binaries carried (C)"
reverify: "PGEN_BEFORE_BIN=<an ast_pipeline built from 9f00c30d3 or earlier, back to b69eceb39> bash docs/tasks/artifacts/engine_universal_services/eus119_layout_sensitive_separator/probe.sh | tail -2   # expect: identical=48 moved=3 (only the preprocessor grammar) and EUS119: OK"
---

The stimuli generator is one engine feeding every family, so a change to how it separates tokens
is a change to every family's samples — whichever family motivated it. `ENGINE-UNIVERSAL-SERVICES.98`
(C) taught the generator to bake a separator after a greedy repetition over an alternation, was
accepted on the SystemVerilog certificate it was written for, and broke the SystemVerilog
*preprocessor*: that grammar is `@whitespace_sensitive`, its newline ends a directive, and three of
its text tokens have exactly that tail.

- **Accept a generator change per family, not per motivating case.** The cheap instrument is stream
  identity: `--generate-stimuli` on every shipped grammar, the old binary against the new, at the
  contract's seeds. A grammar the change was not meant to reach must render byte-identical samples;
  each one that moves gets its own certificate (`--report-certificate-coverage`, read
  `sample_parse_failures`) and its family's closure gate. The tracked probe
  `docs/tasks/artifacts/engine_universal_services/eus119_layout_sensitive_separator/probe.sh` does
  both in about ninety seconds.
- **A control that shares the mechanism cannot see it.** `.98`'s correction measured BEFORE and AFTER
  binaries that both carried (C), read the untouched preprocessor grammar at `1 / 0 / 1` on both and
  wrote it down as "the grammar's own residue". Equal readings on two arms prove the arms agree, not
  that the reading is healthy; the reference for "healthy" is a generator from before the mechanism
  existed (it read `0 / 0 / 0`, and was on disk). See
  `docs/decisions/feedback_a_control_that_passes_under_both_hypotheses_is_not_evidence.md`.
- **A separator is free only where the grammar skips layout.** In a layout-skipping grammar a baked
  space or newline is invisible to the parser. In a `@whitespace_sensitive` grammar it is a token the
  rules never derived. A separator rule added to the generator states which of the two it assumes —
  the comment-introducer guard and the compound-tail rule both read the grammar's own directive.
- **Where it shows up when nobody looks.** A family's status gate reports
  `parseability_parser_rejections_total` above zero and a computed status below its tracker row;
  the family's standalone closure gate exits 0 and only PRINTS the unmet criterion. Read the
  summary, not the exit code.

Related: [[a-filtered-acceptance-run-is-not-the-module]],
[[a-positive-control-built-on-a-live-defect-rots-when-the-defect-is-fixed]].
