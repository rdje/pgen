---
id: an-edition-contrast-in-a-design-unit-is-keyed-on-both-lanes
title: An edition contrast in a design unit is keyed on both lanes — `must_reject` on sv_2017 by the 2017 production, `must_accept` on sv_2023 by the 2023 one, the pipeline cashing the refusal; a MEMBER's contrast cannot be pinned standalone (`SV-CORPUS-GRAD.60`)
answers:
  - "parameter string X[] = { … } rejects under sv_2017 and passes under sv_2023 — which is right"
  - "is -> ev[i]; legal SystemVerilog"
  - "how is a 2017-refusal / 2023-acceptance keyed when the file is a design-corpus member"
  - "why was the event-trigger pin on adc_ctrl_env_cfg.sv withdrawn"
  - "were the cross-item and prototype-lifetime shapes pinned"
tags: [systemverilog, answer-key, edition, lrm, opentitan, pipeline]
date: 2026-10-17
provenance: SV-CORPUS-GRAD.60 (PGEN-SV-CORPUS-GRAD-0454); SV-CORPUS-GRAD.12b.1 (the event-trigger contrast's first ruling); SV-CORPUS-GRAD.14b.2 (key_2023 — the sv_2023 lane reads a contrast as must_accept); SV-CORPUS-GRAD.41 / .43 (the opt-in tolerance list the other two shapes belong to); SVPP-EXPANSION.6d.9 (the measurement that found the seven units)
reverify: "python3 stimuli/sv/run_adjudication_repros.py | tail -1   # failures=0; grep -c 'pinned SV-CORPUS-GRAD.60' stimuli/sv/adjudicate_external_corpus.py   # 7"
---

# An edition contrast in a design unit is keyed on both lanes

**The two shapes.** `parameter string LIST_OF_LEAFS[] = {` — IEEE 1800-2017 A.2.4 gives a parameter
`{ unpacked_dimension }` and an unpacked dimension is `[ constant_range ]` or `[ constant_expression ]`, never `[ ]`;
IEEE 1800-2023 gives it `{ variable_dimension }`, whose `unsized_dimension ::= [ ]`. `->m_adc_push_pull_ev[channel];` —
1800-2017 A.6.5 triggers a `hierarchical_event_identifier` whose final identifier carries no select (A.9.3);
1800-2023 adds `nonrange_select` to the trigger (A.6.5, 15.5). Measured on synthetic carriers under all three profiles:
rc 1 on `sv_2017`, rc 0 on `sv_2023` and `sv_2023_tolerant`; the sized dimension and the bare trigger rc 0 everywhere.

**The keying.** Each PACKAGE file gets one `EXTRA_PINNED` row in the `.12b.1` form — `must_reject` with *an EDITION
CONTRAST* in its basis and both productions cited by line — which `key_2023` reads as `must_accept` on the sv_2023 lane,
so the same row is a refusal the 2017 reading must make and an acceptance the 2023 reading must make. The raw parse of
each stops inside its macro preamble above the construct; an include-dependent `must_reject` is demoted to
`chained_only` by `reject_arm_demotion` (`deferred:chained_only`, the pin kept as the basis — no ceiling moves) and the pipeline, which
reaches the construct, cashes it: `must_reject:match_under_expansion`. One reproducer pair per shape pins both sides in
the bank (the invalid text on `sv_2017`, the same text as a control on `sv_2023`).

**A member's contrast cannot be pinned standalone.** The event trigger lives in a MEMBER (`adc_ctrl_env_cfg.sv:128`)
whose raw parse stops at its class header on a cross-file base. Pinned, the chain read it back as a raw-lane `match`
for the wrong reason (no preprocessor dependency, so nothing demoted it) and as an unexplained divergence on the
sv_2023 lane (`must_accept`, a failure the lane cannot explain). The pin was withdrawn before the commit: the row keeps
`chained_must_accept`, and the reading that can key it is the unit reading `.43` named — a unit that stops at a keyed
construct — not yet built.

**What was not keyed, and why.** The route also named a hierarchical name as a `cross` item and a lifetime in a method
prototype as pin candidates. Both are members of the director's opt-in tolerance profile since 2026-10-02 (`.41`'s
census, `.43`'s release): the strict profiles reject them by that decision and `sv_2023_tolerant` accepts them —
measured again here. A decision already taken is not a question to re-open.

**Measured.** 11656/2943/1737/1232/0 → 11662/2943/1731/1226/0 under the pipeline; the raw tuple 7733/2943/5660/3812/0 → 7733/2943/5660/3812/0 (unmoved); burnable 1,272 → 1,266; the withdrawn pin's first derivation: raw `match` at line 5, sv_2023 `unexplained=1`.

**Related.** [[a-members-barrier-is-its-units-context-not-a-missing-fact-export]] (the measurement that found the units);
[[a-delay-on-a-member-select-needs-parentheses]] (the `.58a` pins the raw lane demotes the same way).
