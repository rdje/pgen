---
id: a-cap-that-preserves-the-count-still-destroys-the-diagnosis
title: A truncated report with an honest total is still UN-DIAGNOSABLE — publishing the uncapped count makes truncation visible, not the finding recoverable
answers:
  - "my report caps its findings list but publishes the true total — is that honest enough"
  - "a ratchet went RED and I cannot tell which new finding caused it"
  - "diffing two runs of my instrument returns no differences but the totals differ — why"
  - "should a diagnostic list have a cap at all, and what must come with it"
  - "how do I name the finding that pushed a count past its ceiling"
  - "when is the right move to fix the instrument instead of bisecting history"
tags: [instruments, diagnostics, truncation, ratchets, debugging, escape-hatch]
date: 2026-08-18
status: current
evidence: SV-CORPUS-GRAD.13c.2i. `ebnf_frontend_dual_run_gate` was RED on `systemverilog` at `151 > ceiling 150`. The report's `divergences` list is capped at 40 with `divergence_total` alongside, and the source comment reads *"the uncapped count travels alongside it, so truncation is always visible"*. A set-diff of the divergence lists from the ceiling's founding commit and from today came back EMPTY ON BOTH SIDES — the same first 40 rows, and the extra divergence past the cap. Nine grammar revisions were probed before the cap was recognised as the blocker; adding `PGEN_ENVELOPE_DUMP_ALL=1` (one edit) named the row on the first run - `use_clause_param_override_sv_only`, `arm1=semantic_annotation_inline` vs `arm2=semantic_annotation`. The repository had already paid this exact price once: `--lint-grammar`'s per-class cap of 40 hid 20 of SystemVerilog's 30 left-recursion findings until `PGEN_LINT_DUMP_ALL` existed.
reverify: "bash docs/tasks/artifacts/sv_corpus_grad/es13c2i_envelope_cap/probe.sh 2>&1 | tail -3   # A1 lists 40, A2 lists all, A3 x4 prove the counts do not move"
---

**A cap on a findings list is a rendering decision that becomes a diagnostic one the first time
something goes RED.** Publishing the true total answers *"is this list complete?"* — it does not
answer *"what is missing?"*, and past the cap those are the same question.

## The failure, in the shape you will meet it

A ratchet fails: a count rose past its ceiling. The obvious move is to diff the instrument's output
against the era when the ceiling was set:

```text
150-era: listed=40 total=150   |   today: listed=40 total=151
ONLY TODAY:      (empty)
ONLY IN THE 150-ERA: (empty)
```

Both lists are the same first 40 rows. The report is internally honest — nothing is hidden, the
total is right, truncation is announced — and it cannot tell you the one thing you need. What
follows is usually a bisect through history, which narrows *when* and never *what*.

## The fix is one edit, and it goes in the instrument

```rust
pub const MAX_REPORTED_FINDINGS: usize = 40;

/// The cap in force for this run — unbounded when `PGEN_<X>_DUMP_ALL` is set.
pub fn reported_findings_cap() -> usize {
    match std::env::var("PGEN_X_DUMP_ALL") {
        Ok(v) if !v.is_empty() && v != "0" => usize::MAX,
        _ => MAX_REPORTED_FINDINGS,
    }
}
```

Three properties make it a debug flag rather than a second code path:

1. **Read once per run**, not per finding — otherwise the instrument's cost depends on its own
   finding count.
2. **The measurement must not move.** Assert it: every count the ratchet keys on is byte-identical
   with the cap lifted and in force. Only the LIST grows.
3. **`0` and empty mean OFF**, so the flag cannot be enabled by an inherited environment.

⭐ **Name it after the escape your project already has.** A reader who knows `PGEN_LINT_DUMP_ALL`
guesses `PGEN_ENVELOPE_DUMP_ALL` without reading anything; a novel spelling has to be discovered.

## The rule

⛔ **If your instrument caps a list, ship the escape in the same commit as the cap.** The cap is for
the ordinary run; the escape is for the run that matters, and by construction that run happens on a
day when nobody wants to be editing the instrument.

⚠️ And when a bisect is answering *"somewhere in these ten commits"* while you still cannot name the
thing: **stop bisecting and fix the instrument.** That is what the toolbox-first directive means in
the case where the tool exists but is blindfolded — see
[[an-instrument-firing-is-not-the-defect-reproducing]] for the neighbouring failure, where the tool
speaks and says the wrong thing.
