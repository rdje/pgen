---
id: the-1800-repetition-served-the-1364-profile-at-every-site-the-list-is-split-once
title: The IEEE 1800 repetition served the IEEE 1364 profile at every site — a packed-dimension list helper split by edition closes all of them at once, the shape question is settled on a synthetic carrier first, and the clause overrides the tool's leniency on the corpus rows it re-keys
answers:
  - "reg [7:0][3:0] r; parses under verilog_2005 — why, and where is the fix"
  - "how do you replace `rule*` with a profile-split helper without moving the typed AST"
  - "a `[$1]` helper inside an optional nests the list — what spelling renders like the repetition"
  - "ten corpus rows keyed must_accept on iverilog's testimony carry a construct the clause refuses — what happens to the bar"
  - "what does the EDITION-PACKED-DIMENSION pass do and how is it different from .3.68's net rule"
  - "an empty #() on a module instance under verilog_2005 — legal or not"
  - "which sites does a 1364-2005 declaration reach in the SV grammar (data_type, block_data_type, implicit_data_type)"
tags: [systemverilog, verilog_2005, grammar, profile-twin, packed-dimension, parameter-value-assignment, answer-key, edition-rule]
date: 2026-10-16
provenance: SV-CORPUS-GRAD.50 (PGEN-SV-CORPUS-GRAD-0442, release 1.0.253); .48 (the previous twin, the finding), .13e.7(b) (the watched pin, 2026-08-24), .3.66 (the net site, 1.0.236), .3.68 (the net edition rule)
reverify: "python3 stimuli/sv/run_adjudication_repros.py   # ADJUDICATION-REPROS: … failures=0 (the five invalid_v2005_*_two_packed_dims.sv pins REJECT, control_v2005_one_packed_dim_sites.sv ACCEPTS everywhere); python3 stimuli/sv/adjudicate_external_corpus.py --self-test   # edition-packed-dimension self-test: PASS"
---

# The 1800 repetition served the 1364 profile at every site; the list is split once

**The finding, and why one pin was not the whole bound.** `accepts_invalid_v2005_reg_two_packed_dims.sv` had sat
watched since `.13e.7(b)` (2026-08-24): `reg [7:0][3:0] r;` parsed under `verilog_2005`, where IEEE 1364-2005 A.2.1.3
`reg_declaration ::= reg [ signed ] [ range ] …` admits one range. Probing the neighbouring sites with the release
probe before designing anything showed the same acceptance at a named-block `reg`, a function's range, an ANSI
`input` port, an `output reg` port and a task port — and a refusal at `wire [1:0][1:0]`, the one site `.3.66` had
already twinned. Every accepting site routes through three rules: `data_type` alternative 1, `block_data_type`
alternative 1 and `implicit_data_type`, each carrying IEEE 1800-2017's `{ packed_dimension }`.

**The fix is one helper, not three twins.** `packed_dimension*` at the three sites became
`packed_dimension_list`, a `-> $1` dispatcher over `packed_dimension_list_sv_only := packed_dimension* -> $1`
(the SV profiles) and `packed_dimension_list_v2005 := ( packed_dimension )? -> $1`. The spelling was settled on
a ten-line synthetic grammar under the interpreter BEFORE the real grammar was touched (TOOLBOX 1.5c): the
SV-side helper renders byte-identically to the inline repetition for 0 / 1 / 2 dimensions; the first v2005
spelling, an optional over a `[$1]` wrapper (the `.3.66` idiom inside a typed template), NESTS the list in a
standalone helper — `dims: [[…]]` — and was refused by the carrier in forty seconds; the plain optional renders
`[…]` / `[]` exactly. The same release made the instance's `#()` require its list (IEEE 1364-2005 A.4.1.1) by
narrowing the sv_2017 twin to its own profile and adding a verilog_2005 twin under the SAME `kind: "sv_2017"`,
which names the shape, as the sv_2023 split already did.

**The corpus rows are re-keyed by the clause, not left in the bar.** Ten verilog_2005-lane rows keyed
`must_accept` on iverilog's testimony (`-g2005` accepts a second packed range and an empty `#()` — tool
extensions) stop on the construct from `1.0.253`. The adjudicator's EDITION-PACKED-DIMENSION pass is `.3.68`'s
net rule extended: the stop must be ON the `[` of a second packed range whose declaration head — found walking
back over balanced ranges and `signed` / `reg` / `automatic` — is `reg` / `input` / `output` / `inout` /
`function` / `task`, or ON the `)` of an empty `#()`, and the sv_2017 profile must accept the bytes; then the row
is `must_reject` with the clause in its basis and the tool's leniency recorded. Nets stay `.3.68`'s; an unpacked
second dimension (`reg [1:0] mem [0:3]`) is never a packed range.

**How to apply.** Before a profile twin: probe every sibling site with the real probe, then settle the shape on
the synthetic carrier, then A/B the pins and controls on the interpreter, then the pre-screens — in that order.
When the key rests on a lenient tool, extend the edition pass rather than widening the grammar or leaving rows
in the bar.
