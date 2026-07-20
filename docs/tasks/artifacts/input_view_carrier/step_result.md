# PGEN-RGX-0078-0192 result

Verdict: **HOLD; a safe full input view is feasible, but its 12.316 ns figure
is only a gross current-load ceiling.**

The source-pinned parser input is an immutable `&'input str`, initialized once.
Complete preserved-region disassembly finds 93 length-carrier and 77
data-carrier loads; 50+14 receive samples. Every length value feeds required
bounds control and every data value feeds required byte access. Dynamic counts
are **226/205/297 + 4/4/11 = 230/209/308**, with zero accepted/held overlap.

A compiled safe-Rust probe rejects the tempting one-word design: forwarding a
scalar length does not eliminate the real slice-length reload used by safe
indexing. Forwarding the complete `&'input [u8]` does work—the sub-root copies
the view once and match callees use its data/length without reloading parser
input. No unsafe indexing, input cap, UTF-8 change, or public API change is
needed.

Current input-field loads price **12.316155375 ns gross**. The future entry
`ldp`, two-register forwarding, register pressure, and possible spills are
unmeasured, so strict net is zero. Even the gross ceiling leaves only
**5.278746322 ns** accumulated margin at 30% capture before those debits.
Implementation is not licensed.

No parser, runtime, emitter, generated artifact, corpus, probe, capture, floor,
MAX, public contract, mdBook, or product behavior changed.
