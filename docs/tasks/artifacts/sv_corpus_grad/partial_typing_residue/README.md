# The raw-envelope census (`SV-CORPUS-GRAD.3.45`, `RAW-ENVELOPE-CENSUS:`)

**For the eight rules the partial-typing census names, what does the corpus actually WRITE at each alternative — the
typed sibling, or the raw positional `Sequence` envelope — and which accepting files must the release move?** Answered by
the parser's own reading of every lane file (TOOLBOX §1.2's `--parse-dump-ast`, the same dump `ast_field_census.py`
uses — the two share `dump_tree`), classified node by node against the eight rules' typed and raw SIGNATURES, in the
parent's context where the shape alone is ambiguous. `partial_typing_census.py` (§1.5g) reads the GRAMMAR and names the
alternatives without a template; this instrument reads the CORPUS and says how often each of them fires.

```bash
# the census (one lane; the list is EVERY lane row — no token pre-selection, so the census is of the corpus)
python3 docs/tasks/artifacts/sv_corpus_grad/partial_typing_residue/raw_envelope_census.py \
    --probe rust/target/release/parseability_probe --profile sv_2017 --files rust/target/sNNN/lane_sv.txt \
    --out rust/target/sNNN/raw_census_sv_2017.tsv --prediction rust/target/sNNN/movers_sv_2017.txt --jobs 6
#   … --profile sv_2023 …            (the same 16 336 rows)
#   … --profile verilog_2005 --files rust/target/sNNN/lane_v2005.txt …   (the 2 606-row 1364-2005 lane)
python3 .../raw_envelope_census.py --self-test                                              # 9 synthetic arms
python3 .../raw_envelope_census.py --self-test --probe rust/target/release/parseability_probe   # + the 19 reproducers (either reading: pre-1.0.219 = 18 RAW + 3 alias_shadowed; 1.0.219+ = 0 RAW, 3 typed alias rows)
```

| column / bucket | meaning |
|---|---|
| `rule`, `alt` | the rule and the alternative (grammar order at release `1.0.218`) that produced the node; `other` = one of `block_data_type`'s thirteen single-reference alternatives |
| `typed` | the node carries the alternative's return template (the shipped shape of the typed sibling, or the post-release shape of a newly typed alternative) |
| `RAW` | the positional `Sequence` envelope — what the untyped alternatives emitted through release `1.0.218` |
| `passthrough` | a child node or keyword token passed through unwrapped (`block_data_type`'s single-reference alternatives; counted, not moved) |
| `unclassified` | a rendering none of the signatures admits — a defect in the classifier, never a bucket to publish over; the note carries the first 80 bytes |
| note `alias_shadowed:<name>` | a nettype declaration parsed down the data-type alternative whose data type is a dimension-less type identifier naming a nettype declared in the same file — IEEE 1800-2017 §6.6.7's ALIAS form, shadowed by the longest-match tie (see below) |

`PREDICTION` names every accepted file carrying at least one `RAW` row (`raw_files`) and every one carrying an
`alias_shadowed` note (`alias_shadowed_files`); their union (`movers`, written to `--prediction`) is the set of accepted
files whose typed AST the release MUST move, and every other accepted file must be byte-identical — the corpus AST
identity sweep (§1.5e) is read against that list two-sidedly (predicted-but-identical = 0, moved-but-not-predicted = 0).

## How a node is read (the signatures, and why some need the parent)

| rule | context | typed sibling(s) | raw envelope(s) |
|---|---|---|---|
| `assignment_pattern` | the `pattern` field of `assignment_pattern_expression`'s `{type, pattern}` | `{exprs}` (positional, alt 1; `exprs: []` = alt 5); post-release `{kind: structure\|array, keys, values}`, `{kind: replication, count, exprs}` | 7 elements `[tick, lbrace, key, colon, expr, [(comma key colon expr)*], rbrace]` — alt 2 when the key node's kind is `member` / `pattern`, alt 3 when `expression` / `pattern_key`; 8 elements with an inner `lbrace … rbrace rbrace` = alt 4 |
| `block_data_type` | the `body` of `{kind: "block_data", body}` | `{base, signing, dims}` (alt 1 through release `1.0.219`) or `{kind: "integer_vector", base, signing, dims}` (alt 1 since `1.0.220`, `SV-CORPUS-GRAD.3.46` — both read as alt 1 typed, so the same classifier reads the `1.0.219` and the `1.0.220` census); post-release `data_type`'s own `{kind: integer_atom \| struct_union \| enum \| virtual_interface, …}` | `[<integer atom kind>, signing?]` (alt 2), 7 elements with `lbrace` at 2 and `rbrace` at 5 (alt 4 struct/union when element 0 is the edition-wrapped `struct_union`, alt 5 when it is the `enum` keyword), `[virtual, interface?, name, params?, modport?]` (alt 8) |
| `case_statement` | context-free (the key sets and the raw signature occur nowhere else) | `{unique_priority, keyword, expr, items}` (alt 1); post-release with `kind: matches \| inside` | 9 elements with `lparen` at 2, `rparen` at 4, the `matches` / `inside` keyword at 5 and `endcase` at 8 |
| `clocking_declaration` | context-free | `{default_keyword, name, event, items, end_label}` (alt 1); post-release `{kind: global, name, event, end_label}` | 7 elements `[global, clocking, name?, event, semi, endclocking, end_label?]` |
| `clocking_event_sv_2017` | the `{kind: "sv_2017", body}` wrapper reached under an `event` or `clocking` key, under `{kind: "clocking", body}` (`coverage_event`, `event_control_sv_2023`), at index 3 of the raw `cover sequence (` envelope or at index 3 of the raw `global clocking` envelope — because `event_control_sv_2017` emits the SAME `{kind: expression, body}` shape under the same wrapper (five sv_2017-wrapped rules emit a `{kind: expression, body}`; the alt-1 shape `{body: {body: <string>}}` is unique, checked over the raw_ast) | `{body: <identifier>}` (alt 1); post-release `{kind: expression, body}` | `[at_sign, lparen, event_expression, rparen]` |
| `list_of_path_delay_expressions` | the `delay.body` of a path declaration `{kind, path, delay}` with `delay` = `{kind: bare \| paren, body}` (`kind: paren` alone is shared by eleven other rules) | `{body}` (alt 1); post-release `{kind: two \| three \| six \| twelve, <LRM position names>}` | an odd-length list of `{body}` nodes separated by `comma` tokens: 3 / 5 / 11 / 23 elements = alts 2 / 3 / 4 / 5 (alt 6 spells the same six expressions as 4 and never wins the tie; alt 7 needs the literal text `tx_path_delay_expression`) |
| `net_type_declaration_sv_2017` (`nettype_declaration_sv_2023` under `sv_2023`) | context-free (the key sets are unique) | `{data_type, name, with_clause}` / `{kind: data_type, …}` (the data-type form), `{kind: nettype, scope, base, name}` (the alias form) | `[nettype, scope?, base, name, semi]` — never observed: see below |
| `property_case_item` | an item of `property_expr`'s `{kind: "case", expr, items}` | `{expressions, body}` (alt 1); post-release `{kind: default, body}` | `[default, colon?, property_expr, semi]` |

Uniqueness of the context-free key sets, measured on the grammar: `grep -c 'unique_priority:'` = 2 (one template, one
comment), `default_keyword:` = 1, the nettype key sets 1 + 1; the `{kind: "sv_2017", body: {body: <identifier>}}`
shape is produced by exactly one of the 75 rules an `sv_2017` wrapper wraps (`clocking_event_sv_2017`).

## ⭐ The nettype alias form is SHADOWED, not raw

IEEE 1800-2017 §6.6.7 gives a nettype declaration two forms; the second, `nettype wTsum nettypeid2;`, creates another
name for an existing nettype. On the LRM's own example BOTH alternatives of `net_type_declaration_sv_2017` succeed — the
data-type form reads `wTsum` as a `known_unscoped_data_type`, because every declared nettype is a `type_name` fact —
and the longest-match tie goes to declaration order, so the alias alternative never won and the typed AST called `wTsum`
a data type. Protocol D on the release-1.0.218 probe: `🏁 Rule 'net_type_declaration_sv_2017' selected branch 1/2` and,
for the fully typed `sv_2023` twin, `🏁 Rule 'nettype_declaration_sv_2023' selected branch 1/2`. The census therefore
reads ZERO raw nettype rows on either lane and instead predicts the alias rows by the `alias_shadowed` note; release
`1.0.219` types the `sv_2017` alias alternative, makes the data-type alternative REFUSE an alias head by negative
lookahead (`[ scope ] <declared nettype> <identifier> ;`, order as shipped) and routes the alias base from a bare
`declaration_identifier` to `checked_nettype_identifier | wildcard_escape_nettype_identifier` (a declared nettype, or an
undeclared name under a wildcard import — the `net_declaration` precedent). ⛔ Two spellings were refuted on the way, each
by an instrument: reordering the ungated alternative read `nettype T wT;` (T a struct typedef, ispras `06.07.02_01.sv`) as an
alias — the two-sided sweep's one moved-but-not-predicted file — and reordering the gated one broke the certificate's own
nettype prelude (the generator renders an unplanned declaration with the first of two equally cheap alternatives, so the
prelude became `nettype \foo \foo;` and the union certificate read `UNKNOWN=2` — `checked_nettype_identifier`,
`nettype_scope_prefix` — where 1.0.218 had 0). The census's alias detector
requires the base to be a nettype declared in the same file, which is exactly the gated alternative's language minus the
wildcard escape; an undeclared base under no import (`nettype undeclared_t x;`) is the one form the release stops
accepting (0 accepting corpus files, pinned `must_reject`).

## Measured (release `1.0.218` probe, HEAD of the slice) — `census.md`

`census.md` carries the three lanes' HEAD readings with every bucket, the movers per lane, and the final-probe re-read.
