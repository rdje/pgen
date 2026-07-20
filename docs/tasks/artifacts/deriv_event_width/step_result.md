# PGEN-RGX-0078-0191 result

Verdict: **HOLD; a lossless 8-byte common event carrier is feasible, but its
strict measured contribution is only 0.530 ns.**

The source-pinned layout probe measures the current `DerivEvent` at 16 bytes
and a packed word at 8 bytes. A two-word in-band escape preserves the full
`usize` payload for arbitrary positions, iteration counts, and branch indices;
there is no hidden input/count cap. Marks, copies, and replay cursors operate on
word indices, while a rare wide placeholder inserts its second word.

The full-SHA-pinned classifier identifies 22 match-side second-word stores, 134
build-side second loads, 16 event-growth callers, 23 outlined event-copy callers,
15 direct event memmoves, and the constructor's exact event allocation. Direct
instruction counts are **7/7/17 stores** and **0/3/0 loads**. Event growth has
zero child samples. Exact copy children are **47/44/41** and event-allocation
children are **22/23/24** across the three bands.

Only the removable second loads/stores are strict: **0.530179151 ns**. Giving
half of exact copy-child time as a linear byte ceiling raises the view to
2.942322856 ns. Giving the still-required half-sized allocation its entire
child time raises an intentionally impossible absolute ceiling to
5.439322139 ns. Encoding/escape/patch work is not yet debited. Added to the
pre-event batch, the strict, copy-linear, and absolute views leave only
1.584/2.308/3.057 ns margin above the 28.8 ns noise floor at 30% capture.

No parser, runtime, emitter, generated artifact, corpus, probe, capture, floor,
MAX, public contract, mdBook, or product behavior changed.
