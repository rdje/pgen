---
id: guard-the-head-not-the-statement-and-a-template-typed-by-field-set-alone-is-a-discriminator-by-absence
title: Guard the HEAD, not the statement — a negative lookahead that names the terminator of the form it refuses admits the same head with any other continuation; and a template typed by field set alone is a discriminator by ABSENCE, which stops discriminating the moment its siblings gain a `kind`
answers:
  - "why did nettype wT x with f parse after release 1.0.219 refused the alias head with a negative lookahead"
  - "how should a negative lookahead that breaks an overlap between two alternatives be spelled — the head or the whole form"
  - "why does PGEN reject nettype <declared nettype> <name> with <fn>; since 1.0.220"
  - "is a nettype identifier a data_type in IEEE 1800-2017 6.6.7 / A.2.1.3"
  - "what changed in block_data_type alternative 1 at schema 32 — kind integer_vector"
  - "why was block_data_type's integer-vector form the only integer-vector shape without a kind"
  - "how were the 259 files that release 1.0.220 moves pre-registered — the .3.45 raw-envelope census alternative-1 rows"
  - "what does a consumer do when a block_data body has no kind after schema 32"
tags: [systemverilog, grammar, lookahead, branch-policy, ast-shape, annotation-campaign, census, sv-corpus-grad]
date: 2026-09-07
status: current
evidence: "SV-CORPUS-GRAD.3.46 (PGEN-SV-CORPUS-GRAD-0376, release 1.0.220, ledger SV-0093, schema 31 -> 32). Interpreter A/B HEAD vs candidate (rust/target/s346/ab_346.tsv): nt_with_alias.sv accepted=true@142 -> false@114 on sv_2017 and sv_2023, the scoped twin 178 -> 157, every nettype control ast=SAME; bdt1.sv / bdt1_v2005.v / control_block_data_type_vector.sv ast=DIFF with verdicts unchanged, the integer-atom control ast=SAME. HEAD census on the 1.0.219 probe: block_data_type:1:typed 712 rows in 259 files on sv_2017 and sv_2023, 130 in 73 on verilog_2005, file sets identical to the .3.45 census; `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9756 identical=9497 ast_moved=259 verdict_moved=0 both_nonaccepting=6580`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9780 identical=9521 ast_moved=259 verdict_moved=0 both_nonaccepting=6556`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2237 ast_moved=73 verdict_moved=0 both_nonaccepting=296`. IEEE 1800-2017 A.2.1.3 docs/systemverilog/2017/md/section-41-data-read-api.md:828-833; 6.6.7 docs/systemverilog/2017/md/section-6-data-types.md:617-619 and :692."
reverify: "./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/invalid_nettype_alias_with_resolution_function.sv --profile sv_2017 >/dev/null 2>&1; test $? -ne 0 && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_nettype_data_type_scoped_with.sv --profile sv_2017 2>&1 | grep -q 'parse_full passed' && ./rust/target/release/parseability_probe --parse-dump-ast systemverilog stimuli/sv/adjudication_repros/control_block_data_type_vector.sv rust/target/km_bdt.json --profile sv_2017 >/dev/null 2>&1 && grep -q '\"integer_vector\"' rust/target/km_bdt.json"
---

**The first fact.** Release `1.0.219` broke the overlap between the two nettype declaration forms with a negative
lookahead on the data-type alternative: `!( [ scope ] checked_nettype_identifier declaration_identifier semi )`. It
refused exactly the LRM's example `nettype wTsum nettypeid2;` — and exactly it. The same head followed by `with f;` —
`nettype wT x with f;`, a form IEEE 1800-2017 A.2.1.3 gives NO production, because the alias form has no `with` and a
nettype identifier is not a `data_type` — was not an alias head to the guard and parsed as the data-type form, the
declared nettype read as a `known_unscoped_data_type` (a nettype declaration plants a `type_name` fact). The `.3.45`
census's alias detector measured it and routed it. The fix drops the `semi`: the guard refuses every text whose head
reads `[ scope ] <declared nettype> <identifier>` whatever follows, the text falls to the alias alternative, which meets
`with` where it needs `;`, and nothing legal starts that way on the data-type alternative (one namespace: a data_type
is never a declared nettype). **The rule:** when a lookahead exists to route a HEAD to the alternative that owns it,
spell the head — the terminator belongs to the alternative that will parse it, and naming it in the guard admits every
other continuation.

**The second fact.** SV-Slice-96 typed `block_data_type` alternative 1 — the block-scoped integer vector — as
`{base, signing, dims}`, `data_type`'s field set without `data_type`'s `kind`. Every register that counts annotations
counted it as typed. When `1.0.219` gave alternatives 2 / 4 / 5 / 8 `data_type`'s own kinds, alternative 1 became the one
`block_data` body a consumer recognised by the ABSENCE of a discriminator — a discriminator by absence, which is none.
`1.0.220` gives it `kind: "integer_vector"`: a REPLACEMENT, schema 32, pre-registered by the `.3.45` census's own
alternative-1 rows (259 files per SV lane, 73 on the 1364-2005 lane — reproduced exactly by the HEAD re-census) and measured
two-sided by the sweep. **The rule:** `kind` on every alternative (`.3.44` said it of rules; this is the last vector
shape), and when a slice measures its successor's population on the way out, that measurement IS the successor's
pre-registration — re-run it, compare the sets, and start.
