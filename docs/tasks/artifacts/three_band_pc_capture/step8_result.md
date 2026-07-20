# Three-band raw-PC capture — exact known-mechanism lower bounds, residual unpriced

`PGEN-RGX-0078-0182`, leaf `RGX-0078.5.j.4`, session #170,
2026-07-20. Measurement-only over the immutable preserved C1 probe. No parser,
emitter, generated artifact, corpus expectation, release floor, or settled MAX
changed.

## Capture result

All pre-registered capture gates pass. The inputs are byte-derived from the
canonical PCRE2 corpus and the custody-pinned timing population; their counts
are exactly 939 / 751 / 488 for sub-1 us / 1–2.5 us / 2.5–20 us. The three
workloads ran sequentially under `caffeinate` and the project memory guard.

| band | stored PCs | target PCs | target share | piece | atom | drop | verdict identity |
|---|---:|---:|---:|---:|---:|---:|---|
| sub-1 us | 22,521 | 2,986 | 13.2587% | 1,564 | 1,422 | 0 | PASS, 939 rows |
| 1–2.5 us | 24,279 | 2,987 | 12.3028% | 1,516 | 1,471 | 0 | PASS, 751 rows |
| 2.5–20 us | 28,308 | 3,868 | 13.6640% | 1,954 | 1,914 | 0 | PASS, 488 rows |

Every band clears >=10,000 stored and >=1,000 target PCs. Every target PC
ASLR-normalizes to one instruction in the full-SHA-pinned probe disassembly,
so target disassembly coverage is **100%**. Accepted/rejected results and all
expectation-mismatch identities equal the banked band outputs.

## Teardown race found and closed before admission

The first complete run was rejected after the high band emitted all evidence
but exited 155 (`SIGPROF`). The qualified v2 teardown restored the prior signal
action before unmasking; a signal already pending at the block/disable boundary
therefore took the default action. The independently owned v3 sampler leaves
the signal blocked, disables the timer, drains any pending `SIGPROF` with
Darwin `sigpending` + `sigwait`, and only then restores action and mask. Its
signal handler is otherwise unchanged: one lock-free atomic reservation and one
fixed-buffer PC store.

A hard-disabled test hook deterministically raised a signal at that boundary.
Qualification recorded `forced_pending_sigprof=1`,
`pending_sigprof_drained=1`, zero drops, and exit zero before the corpus run was
spent. Normal captures assert the hook is off. `attempt1_teardown_failure.md`
banks the rejected run's hashes and root cause without retaining redundant raw
files.

## Exact lower-bound mechanism attribution

`classify_mechanisms.py` pins every accepted raw-capture hash and the probe
custody, then recognizes only machine ranges proven to implement an already
named mechanism:

- all seven G1-C success sites: 72-byte `ParseNode` construction, inline/cold
  arena allocation, and `deriv_boundary` append;
- the two successful thin-memo stores for `RULE_ATOM=27` and `RULE_PIECE=8`:
  exact event/boundary segment construction and successful map insertion.

The classifier mechanically proves all seven target-symbol arena-field loads
are owned, checks the node-width/stride, allocation, boundary-vector,
SmallVec-copy, rule-ID, and map-insert fingerprints, and rejects any overlap.
Failure memo inserts are excluded because the held BATCH-1 member is the pair
of segment copies on successful stores.

| band | G1-C | thin-memo success | known total | residual |
|---|---:|---:|---:|---:|
| sub-1 us | 58 (1.9424%) | 211 (7.0663%) | 269 (9.0087%) | 2,717 (90.9913%) |
| 1–2.5 us | 62 (2.0757%) | 194 (6.4948%) | 256 (8.5705%) | 2,731 (91.4295%) |
| 2.5–20 us | 43 (1.1117%) | 221 (5.7135%) | 264 (6.8252%) | 3,604 (93.1748%) |

For each band, G1-C + thin memo + residual exactly re-sums the target total and
G1-C/thin-memo instruction overlap is zero. These are **in-target lower-bound
instruction samples**, not complete mechanism time: time in callees is sampled
outside the target symbols. Therefore the old caller-attributed G1-C and memo
prices are neither replaced nor added to these percentages.

## Adjudication — no new nanoseconds

The residual is 90.99%, 91.43%, and 93.17% of target samples and remains a mix
of required parsing, carrier movement, spills, control flow, and other work.
Generic memory dominance is not a removable mechanism. In particular, the
residual memory instructions re-sum into stack-frame-addressed / direct parser
field / indirect counts of 902/894/560, 848/859/585, and 987/1,081/852 across
the three bands. Those address bases describe where operands live, not why the
operation exists.

**No nanoseconds are derived. No implementation is admitted.** The existing
BATCH-1 + G3 bundle remains **84.385 ns / HOLD**: the new lower bounds do not
establish a larger non-overlapping removable population, so the conservative
30% capture case still lacks the required noise margin.

The attempted DWARF/source-line route was stopped after two externally
terminated guarded builds and is documented in `dwarf_mapping_attempt.md`; no
cross-vintage source attribution is claimed.

## Reproduction

The banked evidence is independently checkable from the repository root:

```sh
python3 docs/tasks/artifacts/three_band_pc_capture/analyze_capture.py
python3 docs/tasks/artifacts/three_band_pc_capture/classify_mechanisms.py
bash -n docs/tasks/artifacts/three_band_pc_capture/run_capture.sh
clang -fsyntax-only -std=c11 -Wall -Wextra -Werror \
  docs/tasks/artifacts/three_band_pc_capture/pc_sampler.c
```

The first two commands reproduce `capture_analysis.txt` and
`mechanism_attribution.txt`. The runner is the count-gated capture recipe; raw
captures are stochastic, so the exact mechanism script deliberately pins the
banked raw hashes rather than pretending a fresh run is byte-identical.

## Next

`PGEN-RGX-0078-0183` owns a read-only residual stack-frame carrier attribution.
It must partition those stack-addressed samples into exact machine-structure
carriers—such as unavoidable ABI/prologue work, `ParseResult`/`ParseError`
movement, or semantic checkpoint/delta movement—before naming a lever. It may
reprice the held batch only if a concrete non-overlapping carrier is proven;
otherwise it records a diagnostic refusal. No implementation belongs in that
leaf.
