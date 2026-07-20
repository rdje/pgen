# Bare-path diagnostic mechanism expansion (`PGEN-RGX-0078-0185`)

This read-only slice corrects the preceding field-role label, expands the
actually removable diagnostic work across instruction classes, and prices the
result without changing parser code, generated artifacts, corpus verdicts, or
the accepted floor.

## Foundational correction

`PGEN-RGX-0078-0184` correctly found and exactly re-summed the two counter
offsets, but over-labelled both as removable observers. Source and emitted
memo dataflow prove that `predicate_evaluations` (`0x250`) is the mutable-store
taint signal protecting store-blind thin memoization. Its **9 / 13 / 16**
samples are required work. Only `rollbacks_nonempty_chain` (`0x248`) is a
diagnostic-only counter.

The corrected direct-memory removable subset is therefore:

| role | sub-1 µs | 1–2.5 µs | 2.5–20 µs |
|---|---:|---:|---:|
| cached logger loads | 104 | 95 | 120 |
| coverage snapshot/truncate | 30 | 29 | 38 |
| rollback diagnostic counter | 0 | 0 | 1 |
| **corrected direct subset** | **134** | **124** | **159** |

That is **0.552424% weighted all-sample**, superseding the provisional
0.601206% figure. The complete 894 / 859 / 1,081 parser-direct re-sum remains
valid after splitting the former observer row into diagnostic counter
**0 / 0 / 1** and required memo-taint signal **9 / 13 / 16**.

## Whole removable machine mechanism

The custody-pinned classifier enumerates all relevant static sites, not only
sampled PCs:

- 170 cached logger gates = 419 instructions: 78 `cmp+b.ne`, one
  `cmp+b.eq`, 89 `tbz`, and two `tbnz` forms;
- 39 coverage checkpoints = 39 snapshot loads, eight exclusive one-word
  carrier stores, 16 exclusive carrier reloads, and 39 four-instruction
  rollback cores = 219 removable instructions;
- 63 `rollbacks_nonempty_chain` sites = chain-depth compare/branch plus
  load/add/store = 315 instructions.

The **953-PC** union has **437 memory / 453 control / 63 other** instructions,
zero inter-mechanism overlap, and zero overlap with accepted G1-C or successful
thin-memo ranges. Cold trace formatting is excluded. Eight coverage snapshot
values share an `stp` with a required word; those instructions remain and their
2 / 1 / 0 samples are deliberately unpriced, as are register-pressure and
store-width upside.

Raw dynamic counts for the complete removable set are **149 / 143 / 193**.
With the accepted 0.380/0.353/0.260 band weights and the excluded 0.007 tail
conservatively priced at zero, that is **0.636586% weighted all-sample =
8.042632 ns** on the accepted 1,263.4 ns floor. Adding it to the held
BATCH-1/G3 target reprices the bundle
from **84.385 ns** to **92.427632 ns**. The 30/50/70% cases become
**27.728290 / 46.213816 / 64.699342 ns**. The conservative 30% case remains
**1.071710 ns below** the 28.8 ns same-binary noise threshold, so the verdict
is still **HOLD** and implementation is not yet licensed.

The next exact neighbor is the residual thin-memo lookup: five independently
qualified hash-entry sites contributed 21 / 40 / 68 direct samples and may
provide the missing conservative margin when expanded across the full lookup
mechanism.

The same source audit found additional always-on rollback telemetry counters.
They were not in `-0184`'s qualified sampled-field subset and are not silently
priced here; `PGEN-RGX-0078-0187` now owns their reader audit and exact machine
expansion after the thin-memo neighbor.
