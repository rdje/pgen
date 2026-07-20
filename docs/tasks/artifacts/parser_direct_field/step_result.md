# Residual direct-parser field attribution (`PGEN-RGX-0078-0184`)

This read-only slice classifies every residual target-memory sample whose
operand addresses the parser directly through `x20`. It uses the accepted
three-band raw captures and the preserved C1 floor probe. No parser, emitter,
generated artifact, corpus result, or floor measurement changed.

## Custody boundary caught and enforced

The current exported `RegexParser` probe produced a 1,312-byte, align-8 type
with DWARF member offsets. Those offsets are not the preserved probe's offsets:
for example, the current input is at `0x300` while preserved-probe byte-load
dataflow uses `0x2f0/0x2f8`; current position is `0x4a8`, while the preserved
monotone position pair is `0x498/0x4e0`; current `logger_enabled` is `0x514`,
while the preserved logger gates load `0x504`.

This is a custody boundary, not a field-discovery shortcut. The floor probe is
the C1 artifact (`cb322b75`, commit `9df9d466…`, binary SHA `1d3fa0ee…`); the
current generated parser is the later G1-A artifact (`e4924024`, commit
`bb69f418…`). Rust's default struct layout is not a stable cross-artifact ABI.
The prior `-0182` rule therefore applies: **no current DWARF address is
transferred**. `current_parser_layout_dwarf.txt` and
`parser_layout_provenance.txt` bank this as negative evidence.

## Machine-qualified partition

`classify_parser_fields.py` first reproduces the accepted G1-C and successful
thin-memo exclusions, pins the probe and all three raw files, then qualifies
each candidate offset by binary behavior:

- input length bounds checks lead to byte loads through the paired data
  pointer;
- two 16-byte-element vectors and a maximum-depth comparison form the
  recursion guard;
- 35 of the 39 already-qualified speculation sites reload every physical
  seven-word checkpoint source immediately before materialization (the other
  four retain some values in registers);
- two Vec grow/push/copy families form the event and boundary derivation tapes;
- position/furthest-position is a monotone load/compare/conditional-store pair;
- all 51 sampled cached-logger loads immediately feed a branch gate (31 full
  trace gates, 19 direct bit gates, one direct compare gate);
- coverage is a length-only speculation snapshot/conditional-truncate field;
- five table-occupancy loads enter the same thin-memo hash-mixing sequence;
- semantic observer fields show an increment or snapshot/compare counter shape.

The exact per-band result is:

| qualified role | sub-1 µs | 1–2.5 µs | 2.5–20 µs |
|---|---:|---:|---:|
| input view | 230 | 209 | 308 |
| recursion guard | 163 | 182 | 200 |
| semantic checkpoint state | 137 | 125 | 143 |
| derivation tapes | 134 | 93 | 118 |
| position progress | 66 | 73 | 69 |
| trace gate | 104 | 95 | 120 |
| coverage rollback | 30 | 29 | 38 |
| thin-memo lookup | 21 | 40 | 68 |
| semantic observer counters | 9 | 13 | 17 |
| **re-sum** | **894** | **859** | **1,081** |

There is no overlap and no unmatched remainder. `parser_direct_census.txt`
banks every contributing PC and context; `parser_field_attribution.txt` is the
compact executable proof and re-sum.

## Adjudication

Three role groups are diagnostic-only on the ordinary bare parse:
`trace_gate + coverage_rollback + semantic_observer_counters` contribute
**143 / 137 / 175** direct memory samples, a geomean-profile-weighted
**0.601206% of all captured PCs**. This is a strict lower bound: the companion
compare/branch/control instructions are outside the direct-memory population.

No nanoseconds are derived here. A sampled instruction group is not yet a
complete removable mechanism, and the held BATCH-1/G3 bundle must not absorb a
paper estimate. The next leaf expands these exact diagnostic gates across all
instruction classes, checks overlap, and only then decides whether a
bare-fused specialization has conservative noise-clearing margin.

The residual thin-memo lookup is separately proved real but not removable in
this leaf. Its five hash-entry sites deserve their own later whole-mechanism
expansion; the 125-sample `0x490` population must not be mistaken for
`bare_parse` (the current-vintage DWARF mismatch directly refutes that guess).

**New nanoseconds: 0.000.** The held bundle remains **84.385 ns / HOLD**.
