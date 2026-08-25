# L4 punctuation adjudication — IEEE 1364-2005 (`verilog_2005`)

⛔ **DERIVED — do not edit by hand.** Regenerate with
`python3 stimuli/sv/v2005_punctuation_adjudication.py --write`.

Candidate population from `stimuli/sv/v2005_punctuation_faithfulness_census.py`'s own
`candidate_set()` — this table names a witness and a matched control per candidate and
never re-types the population. Owning leaf: `SV-CORPUS-GRAD.13e.10` (c).

**19 candidates · 17 CONFIRMED over-acceptances · 2 correctly excluded · 0 neither.**

| rule | literal | leg 1 `verilog_2005` | leg 2 `sv_2017` | leg 3 `committed` | ⭐ leg 3b control | verdict |
|---|---|---|---|---|---|---|
| `arithmetic_shift_left_assign` | `<<<=` | accept | accept | **1** | 0 | ⛔ CONFIRMED over-acceptance |
| `arithmetic_shift_right_assign` | `>>>=` | accept | accept | **1** | 0 | ⛔ CONFIRMED over-acceptance |
| `wildcard_not_equal` | `!=?` | accept | accept | **1** | 0 | ⛔ CONFIRMED over-acceptance |
| `iff_arrow` | `<->` | accept | accept | **1** | 0 | ⛔ CONFIRMED over-acceptance |
| `shift_left_assign` | `<<=` | accept | accept | **1** | 0 | ⛔ CONFIRMED over-acceptance |
| `wildcard_equal` | `==?` | accept | accept | **1** | 0 | ⛔ CONFIRMED over-acceptance |
| `shift_right_assign` | `>>=` | accept | accept | **1** | 0 | ⛔ CONFIRMED over-acceptance |
| `percent_assign` | `%=` | accept | accept | **1** | 0 | ⛔ CONFIRMED over-acceptance |
| `and_assign` | `&=` | accept | accept | **1** | 0 | ⛔ CONFIRMED over-acceptance |
| `star_assign` | `*=` | accept | accept | **1** | 0 | ⛔ CONFIRMED over-acceptance |
| `plus_plus` | `++` | reject | accept | **0** | 0 | correctly excluded |
| `plus_assign` | `+=` | accept | accept | **1** | 0 | ⛔ CONFIRMED over-acceptance |
| `minus_minus` | `--` | reject | accept | **0** | 0 | correctly excluded |
| `minus_assign` | `-=` | accept | accept | **1** | 0 | ⛔ CONFIRMED over-acceptance |
| `dot_star` | `.*` | accept | accept | **3** | 0 | ⛔ CONFIRMED over-acceptance |
| `slash_assign` | `/=` | accept | accept | **1** | 0 | ⛔ CONFIRMED over-acceptance |
| `xor_assign` | `^=` | accept | accept | **1** | 0 | ⛔ CONFIRMED over-acceptance |
| `or_assign` | `|=` | accept | accept | **1** | 0 | ⛔ CONFIRMED over-acceptance |
| `tick` | `'` | accept | accept | **1** | 0 | ⛔ CONFIRMED over-acceptance |

## Why leg 3b exists

`committed >= 1` alone is a **reading**, not a measurement. `SV-CORPUS-GRAD.13e.12`
measured that `wildcard_equal` is **ENTERED twice in BOTH arms** — the parser speculates
that branch whether or not `==?` is present — so the ENTRY counter is provably blind to
the question and only the COMMITTED map discriminates. Every row above therefore carries
a matched control in which the same enclosing construct lacks the operator and the same
terminal must read **0**. A row whose control does not read 0 is REFUSED, never published.

## Witnesses

| rule | witness | control |
|---|---|---|
| `arithmetic_shift_left_assign` | `module m; initial begin a <<<= 1; end endmodule` | `module m; initial begin a = 1; end endmodule` |
| `arithmetic_shift_right_assign` | `module m; initial begin a >>>= 1; end endmodule` | `module m; initial begin a = 1; end endmodule` |
| `wildcard_not_equal` | `module m; initial begin if (a !=? b) ; end endmodule` | `module m; initial begin if (a != b) ; end endmodule` |
| `iff_arrow` | `module m; initial begin if (a <-> b) ; end endmodule` | `module m; initial begin if (a && b) ; end endmodule` |
| `shift_left_assign` | `module m; initial begin a <<= 1; end endmodule` | `module m; initial begin a = 1; end endmodule` |
| `wildcard_equal` | `module m; initial begin if (a ==? b) ; end endmodule` | `module m; initial begin if (a == b) ; end endmodule` |
| `shift_right_assign` | `module m; initial begin a >>= 1; end endmodule` | `module m; initial begin a = 1; end endmodule` |
| `percent_assign` | `module m; initial begin a %= 1; end endmodule` | `module m; initial begin a = 1; end endmodule` |
| `and_assign` | `module m; initial begin a &= 1; end endmodule` | `module m; initial begin a = 1; end endmodule` |
| `star_assign` | `module m; initial begin a *= 1; end endmodule` | `module m; initial begin a = 1; end endmodule` |
| `plus_plus` | `module m; initial begin a++; end endmodule` | `module m; initial begin a = 1; end endmodule` |
| `plus_assign` | `module m; initial begin a += 1; end endmodule` | `module m; initial begin a = 1; end endmodule` |
| `minus_minus` | `module m; initial begin a--; end endmodule` | `module m; initial begin a = 1; end endmodule` |
| `minus_assign` | `module m; initial begin a -= 1; end endmodule` | `module m; initial begin a = 1; end endmodule` |
| `dot_star` | `module s; endmodule ⏎ module m; s u(.*); endmodule` | `module s; endmodule ⏎ module m; s u(.a(1)); endmodule` |
| `slash_assign` | `module m; initial begin a /= 1; end endmodule` | `module m; initial begin a = 1; end endmodule` |
| `xor_assign` | `module m; initial begin a ^= 1; end endmodule` | `module m; initial begin a = 1; end endmodule` |
| `or_assign` | `module m; initial begin a |= 1; end endmodule` | `module m; initial begin a = 1; end endmodule` |
| `tick` | `module m; initial begin a = '{1,2}; end endmodule` | `module m; initial begin a = 1; end endmodule` |
