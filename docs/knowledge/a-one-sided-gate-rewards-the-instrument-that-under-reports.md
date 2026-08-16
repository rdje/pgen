---
id: a-one-sided-gate-rewards-the-instrument-that-under-reports
title: A ratchet only breaches one way — so a broken instrument that reads LOW is not merely unnoticed, it is reported as an improvement and invited into the baseline
answers:
  - "my gate only fails on a rise — what happens when the measurement itself is wrong"
  - "how do I decide whether an unguarded input is a note in a document or a hard refusal"
  - "is it enough that my identity table pins every input to the measurement"
  - "why should a gate hash the executable and not just the sources"
  - "how bad is it if someone measures with a stale or experimental build of the probe"
  - "my instrument reported a big improvement — what should I check before promoting it"
  - "how do I let people measure an experimental arm without letting the result become a baseline"
  - "where should a build-time fingerprint of a generated artifact be computed"
  - "should I emit a fingerprint into the generated file or compute it in build.rs"
tags: [gates, ratchet, instrument-honesty, failure-direction, provenance, build-flow, baselines, fingerprint]
date: 2026-08-17
status: current
evidence: docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .24 (opened by .20 slice 4, closed by slice 2 PGEN-ENGINE-UNIVERSAL-SERVICES-0066); docs/tasks/artifacts/engine_universal_services/probe_fingerprint_gate/wrong_probe_cost.sh + wrong_probe_cost.txt (11,240,430 vs 762,345 rule entries on the same four files = 14.7x, the wrong probe reading LOWER); docs/tasks/artifacts/engine_universal_services/probe_fingerprint_gate/probe.sh + probe.txt (14/14 arms, 3 GREEN / 11 RED); rust/build.rs (emit_parser_fingerprint); rust/src/bin/parseability_probe.rs (--parser-fingerprint); scripts/check_parse_cost_ratchet.sh tier-1 arm 5 + the tier-2 pre-flight.
reverify: "python3 stimuli/sv/corpus_parse_cost.py --verify-probe-fingerprint && PGEN_PARSE_COST_PROBE=rust/target/lr_ab_arms/probe_arm3 python3 stimuli/sv/corpus_parse_cost.py --verify-probe-fingerprint; test $? -eq 1 && echo ONE-SIDED-GAP-NOW-REFUSES"
---

**A one-sided gate has a blind flank, and it is the flank a broken instrument falls on.**

`PARSE-COST-RATCHET` refuses when a counter RISES. A fall is not a breach; it prints
*"an improvement — promote it deliberately so the ratchet tightens"*. That asymmetry is correct for
the thing being guarded — costs are rejected, not traded — and it creates a second, unguarded
question nobody was asking: **what if the number fell because the instrument was wrong?**

Measured, on the same four corpus files, with two binaries that differ only in which generated
parser they were compiled from:

| probe | rule entries |
|---|---:|
| shipped (embeds the pinned parser) | **11,240,430** |
| an experimental arm with left-recursion guard emission suppressed | **762,345** |

**14.7× lower.** Through a gate that breaches only on a rise, that is not a near miss — it is a
green run with a congratulatory note attached. The honest description of the gap is therefore not
*"a wrong binary could be measured unnoticed"* but **"a wrong binary is measured, reported as
progress, and invited into the baseline, where it lowers the ratchet permanently to a number no
real parser can produce."**

⇒ **the size of a gap is not what decides between a footnote and a refusal; the DIRECTION is.**
The same hole in a two-sided check would announce itself on the first run. Before filing an
unguarded input as *"latent, blast radius small"*, work out which way an error travels through the
verdict. If it travels toward PASS, it is not latent — it is silently active.

## The specific shape: an identity table full of SOURCES

The baseline pinned four inputs — the grammar, the generated parser, the instrument, a digest over
the sampled corpus files. Every one is a *source*. The numbers are produced by an **untracked build
artifact** nothing hashed and nothing tied to the parser it was compiled from. So the gate could
say, truthfully, *"every input is byte-identical"* while `nm … | grep -c _lr_guard` read **0**
against a pinned parser declaring **6**.

⭐ **A measurement's provenance is not complete until the EXECUTABLE is in it.** Sources answer
*"what was this computed from?"*; only the binary answers *"what actually computed it?"* — and on
any project where experimental builds are a routine act, those two diverge on purpose.

## Where to compute the fingerprint — and the option that looks obvious and is not

The natural fix is to emit the fingerprint *into* the generated artifact. Don't. That moves every
generated file and re-baselines everything keyed on them — byte-identity controls in six gates and
a whole reproducibility doctrine — to answer a question about the build.

⭐ **Compute it at BUILD time instead.** `rust/build.rs` already resolved each generated parser and
already declared `cargo:rerun-if-changed` for it, so it re-runs *exactly* when one moves. Hashing
there and publishing `PGEN_<FAMILY>_PARSER_SHA256` gives the emit-time property at **zero generated
bytes**. The hook was already present and unused — which is the general lesson: before adding a
mechanism, check whether the dependency edge you need is already declared somewhere.

Three details that turned out to matter:

1. **Absent ≠ mismatched.** A build with no parser on disk leaves the variable UNSET
   (`option_env!`), and the reader reports *"built with no such parser"*. A placeholder digest would
   have compared unequal and read as *"wrong parser"* — pointing the next reader at the parser
   instead of at the build.
2. **Two independent implementations, cross-checked.** The digest is computed by Rust `sha2` and
   compared against Python `hashlib` (OpenSSL) — 9/9 identical across every resolved parser, and
   re-checked on every run. A fingerprint compared against a differently-computed digest fails
   *closed* forever, which looks exactly like a real mismatch.
3. ⛔ **Cargo builds build scripts at `opt-level = 0` in every profile.** Hashing 236 MB there cost
   **5.5 s** per re-run; `[profile.*.build-override] opt-level = 2` made it **0.39 s** — 14.1×. A
   cheap check that is not cheap gets removed later by someone with a deadline.

## Let the legitimate act happen, and make it leave a mark

Measuring an experimental arm on purpose is a real workflow — this campaign's own A/B slices exist
because of it. An unconditional refusal breaks it and gets worked around, so the override exists
(`PGEN_PARSE_COST_ALLOW_PROBE_MISMATCH=1`) and it **stamps the mismatch into the output artifact**
(`probe_parser_matches_generated: false`). The result stays usable and can never later pass for a
baseline.

⛔ **And the gate strips that variable from the environment it hands the instrument.** An escape
hatch reachable from the path whose output becomes the tracked reference is not an escape hatch; it
is a hole with a friendly name. Related: [[a-gate-must-be-able-to-fail-and-able-to-run]],
[[a-measurement-that-cannot-name-its-instrument-cannot-be-checked-for-staleness]] (the same question
one layer up — that one asks *which instrument*, this one asks *which binary of it*), and
[[a-check-whose-inputs-all-pass-has-not-been-tested]], whose newest sub-shape was found while
proving this very fix.
