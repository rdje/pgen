---
id: generated-raw-lane-text-replays-through-a-stage-in-its-consumer-resolves-mode
title: Generated raw-lane text replays through a stage in its consumer-resolves mode — a stage policy that is right for a corpus file is a category error for text the generator derived from the residue the stage leaves behind
answers:
  - "why did the SV closed-loop replay_preprocess stage go red after the preprocessor started refusing unresolved backticks"
  - "should the stimuli generator stop emitting opaque directive lines so the preprocessor accepts them"
  - "what does undefined_macro_policy=keep cover — does it skip the macro-string token refusal too"
  - "where is the undefined-macro policy for generated text declared in the SV stimuli quality gate"
  - "why do the corpus lanes keep undefined_macro_policy=error while the closed loop runs keep"
  - "a gate replays generated text through a stage that gained a strictness and the gate goes red — which side moves"
tags: [systemverilog, preprocessor, closed-loop, stimuli-generator, raw-lane, stage-policy, contract, svpp-expansion]
date: 2026-10-08
status: current
evidence: "SVPP-EXPANSION.6g (`PGEN-SVPP-EXPANSION-0047`). `.6f` taught the SV preprocessor to refuse a backtick that survives expansion (an undefined name, a bare function-like usage, a stray `` `\" `` / `` `\\`\" `` / ```` `` ```` token) under its default `undefined_macro_policy=error`. The SV stimuli quality gate and the SV formal umbrella then went RED at `profile_2017_closed_loop_replay_preprocess`: the replayed text is GENERATED from the grammar's raw lane, whose `compiler_directive` residue derives an opaque backtick line on purpose, and the sample read `` `\\ `` + newline. Measured on the isolated sample (`rust/target/sv47/opaque_probe.sv`): the default policy `preprocess_failed … a macro-string or paste token … outside macro text`, rc 1; `--stage-opt undefined_macro_policy=keep` rc 0 with the text passed through unchanged and the raw parse `parse_full passed` — the parked note that `keep` does not cover the token class was wrong, the policy skips the whole scan. The gate's three generated-text stages (closed-loop initial, closed-loop replay, per-sample: 2 of 16 samples refused under the default) now read `preprocess.generated_text_undefined_macro_policy` from the contract; the hand-written realistic cases and every corpus lane keep the default."
reverify: "cd rust && ./target/debug/ast_pipeline ../rust/target/sv47/opaque_probe.sv --preprocess systemverilog --output /dev/null; echo rc=$? (expect 1); ./target/debug/ast_pipeline ../rust/target/sv47/opaque_probe.sv --preprocess systemverilog --stage-opt undefined_macro_policy=keep --output /dev/null; echo rc=$? (expect 0); jq -r .preprocess.generated_text_undefined_macro_policy test_data/grammar_quality/systemverilog_core_v0_contract.json (expect keep); grep -c 'undefined_macro_policy=\\$generated_text_undefined_macro_policy' scripts/sv_stimuli_quality_gate.sh (expect 3)"
---

**A generator cannot define what the residue rule leaves undefined.** The SystemVerilog grammar's raw
lane carries `compiler_directive := !lrm_directive_name opaque_directive_line → {kind: "opaque"}` so an
unexpanded file parses around its macro uses; the stimuli generator derives from that grammar and so
emits opaque backtick lines by right. The preprocessing stage, since `SVPP-EXPANSION.6f`, refuses a
backtick that survives expansion — the correct post-condition for a corpus file, where an unresolved
usage means a missing definition. Replaying the generator's text through the stage under that
post-condition asks the stage to resolve what no definition can resolve. Two correct designs composed
into a wrong claim, the same shape `.6f` itself repaired from the other side.

- **Which side moves.** Not the generator: excluding the residue rule from generation would stop
  witnessing a rule the raw lane ships and move the certificate (`canonical UNKNOWN=14` is a pinned
  baseline). Not the stage's default: the corpus lanes need `error`, and the pipeline's `pass` was a
  hole until they had it. The GATE moves: its generated-text stages declare the stage's
  consumer-resolves mode — `undefined_macro_policy=keep`, "kept as written for a consumer that resolves
  the rest itself", and the raw-lane parser IS that consumer — through one contract key,
  `preprocess.generated_text_undefined_macro_policy`, beside the other closed-loop stage policies the
  contract already declares for generated text (`conditional_symbol_policy=assume_false_warn` tolerates
  a generated `` `ifdef UNDEFINED `` the same way). The hand-written realistic cases and every corpus
  lane keep the default, and the key's name says what it is for.
- **Measure the knob before designing around a note.** The parked leaf had written that `keep` "does
  not cover this token class — it is an engine refusal, not the policy's". One command on the isolated
  sample showed the policy returns before the scan begins. A routed leaf's note is a claim like any
  other; the design that would have followed it (a `@generate`-side exclusion, a reconciled stage)
  was priced against the certificate for nothing.
- **The closed loop's preprocessed output is not parsed as a whole.** Its `--output` is written and
  only the diagnostics are read (the error / warning debt ratchet), so the whole-batch parse rejection
  of the kept output at a top-level `bins` fragment is not a stage and not a regression; the
  per-sample stage's output IS parsed, and under `keep` it reads exactly what it read before `.6f`.
