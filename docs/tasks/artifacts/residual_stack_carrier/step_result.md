# Residual stack-carrier attribution — exact lower bounds, no new lever

`PGEN-RGX-0078-0183`, leaf `RGX-0078.5.j.4`, session #171,
2026-07-20. Read-only analysis of the immutable `-0182` raw-PC captures and
preserved C1 probe. No parser, emitter, generated artifact, corpus expectation,
release floor, or settled MAX changed.

## Result

The exact classifier accounts for all **902 / 848 / 987** previously residual
target-memory samples whose operands address `sp` or `x29`. It pins the probe
and all three raw-capture SHAs, re-runs the accepted G1-C/thin-memo exclusion,
qualifies every new machine structure, rejects overlap, and leaves all
unproved stack work explicit rather than assigning it a nearby source label.

| carrier | sub-1 us | 1–2.5 us | 2.5–20 us |
|---|---:|---:|---:|
| ABI frame save/restore | 81 (8.98%) | 85 (10.02%) | 114 (11.55%) |
| semantic checkpoint materialization | 68 (7.54%) | 66 (7.78%) | 74 (7.50%) |
| semantic delta gate/movement/apply | 36 (3.99%) | 48 (5.66%) | 50 (5.07%) |
| ParseResult 32-byte prefix | 13 (1.44%) | 17 (2.00%) | 19 (1.93%) |
| ParseResult 48-byte width tail | 5 (0.55%) | 8 (0.94%) | 3 (0.30%) |
| explicit unmatched | 699 (77.49%) | 624 (73.58%) | 727 (73.66%) |

Every column re-sums exactly to the accepted `-0182` stack population. The
recognized structures are lower bounds, not a claim that all semantic or
result-carrier work has been found.

## Exact machine ownership

- Four six-instruction prologue/epilogue regions are exclusively callee-saved
  register frame traffic in the two target functions.
- Five regions start at an `Option<SemanticRuntimeDelta>` discriminant gate
  and terminate at the pinned `SemanticRuntimeState::apply_delta` call. The
  final region includes the exact 168-byte winner-delta movement into its apply
  buffer. The type probe fixes `SemanticRuntimeDelta` and its `Option` at 168 B.
- The final `cascade_match_piece` sret block copies an 80-byte
  `ParseResult<()>`. Its two prefix loads cover bytes 0–31; its three tail loads
  cover bytes 32–79 and are the exact portion removed by the already designed
  80→32-byte G1-B indexed-error carrier.
- The generated source orders `checkpoint()` immediately before the first
  speculative trace guard. The probe contains 39 inlined “Starting
  speculative parse” literals. Before every corresponding guard, the
  classifier finds exactly one trailing, contiguous seven-word stack-write
  span. The banked type probe fixes `SemanticRuntimeCheckpoint` at 56 B,
  `Copy`, and drop-free. Those blocks are therefore checkpoint
  materializations, not a trace-only spill population.

**Later correction (`PGEN-RGX-0078-0193`, 2026-07-20):** this detector is a
complete census of its trace anchor, not of every checkpoint. A full physical
field-load scan found one additional trace-less C3-B tournament checkpoint at
`0x100073f40`. Its 0/1/1 sampled required store moves the checkpoint row to
68/67/75 and unmatched to 699/623/726; total re-sums and all prices are
unchanged. See `../checkpoint_compaction/`.

## Fishy hypothesis resolved

The initial disassembly made the seven stores before disabled trace guards
look like unconditional preservation for cold formatting/logger calls. That
would have suggested an out-of-line cold helper. The complete trace-anchored
39-site census falsifies it: seven words exactly match the semantic checkpoint
that must survive the subsequent speculative parser call for rollback. Some
trace code may still influence register allocation, but this capture proves no exclusive
trace carrier and therefore admits no trace optimization or performance claim.

## Adjudication

Only the ParseResult width tail is mechanically removable, and it is an exact
subset of the already held G1-B mechanism—not a new, additive population. Its
weighted share of all raw samples is just **0.022823%**. ABI work is baseline;
checkpoint/delta work is required by current transactional semantics; and the
remaining 73.58–77.49% is deliberately unmatched.

**No new nanoseconds are derived.** BATCH-1 + G3 remains **84.385 ns / HOLD**;
the conservative 30% case still lacks its required noise margin. The next
read-only residual pass should classify direct parser-field memory operands
(894 / 859 / 1,081 samples), because their fixed offsets can be tied to exact
state fields without guessing from source lines.

## Reproduction

From the repository root:

```sh
rustc --edition=2021 \
  docs/tasks/artifacts/residual_stack_carrier/carrier_layout_probe.rs \
  -L dependency=rust/target/debug/deps \
  --extern pgen=rust/target/debug/deps/libpgen-2487bd25a1b10d71.rlib \
  -o /tmp/pgen-0183-carrier-layout
/tmp/pgen-0183-carrier-layout
python3 docs/tasks/artifacts/residual_stack_carrier/classify_stack_carriers.py
python3 docs/tasks/artifacts/residual_stack_carrier/inspect_stack_residual.py
```

The first output equals `carrier_layout_probe.txt`; the classifier output
equals `stack_carrier_attribution.txt`. `carrier_layout_provenance.txt` pins the
source, rlib, compiler, and exact compile command. The census script emits the
full residual stack-PC/slot/context inventory used during attribution.
