# PRE-REGISTRATION — `ENGINE-UNIVERSAL-SERVICES.67`(c)(i), thin-memo OCCUPANCY

Written after the engine change and the probe change compiled and **before any occupancy number was
read**. Producer: `rust/src/ast_pipeline/mod.rs` (`ThinMemoScratch` census) +
`rust/src/bin/parse_alloc_census_probe.rs` (`PARSE-THIN-OCCUPANCY:`); ladder: the same 11 inputs as
`-0121`/`-0124`/`-0126`/`-0128`.

## ⛔ THE QUANTITY, NAMED BEFORE THE NUMBER

`-0128`'s carry-forward was about LIFETIME — I predicted a thing was gone without reading its
`Drop`. The frame to check here is the DENOMINATOR, because this table has two plausible ones and
they differ by an arbitrary factor.

| symbol | quantity | why it is or is not the denominator |
|---|---|---|
| `need_slots` | `THIN_RULE_COUNT × (input_len + 1)` — what THIS parse asked for | **the denominator.** It is what the parse pays for and what `-0128` measured at 872 B per input byte |
| `capacity_slots` | `slots.len()` — the RECYCLED table's size | **NOT the denominator.** The table never shrinks, so a small file after a large one sees a capacity unrelated to its own cost; dividing by it would understate occupancy by however much bigger the previous file was |
| `distinct_slots_written` | slots whose generation did not already match when `store` wrote them | the numerator — **exact iff `clear_calls == 0`**, an upper bound otherwise |
| `store_calls` | every `store`, including restores of a slot already written | NOT the numerator; a re-store costs no memory |

⇒ occupancy is `distinct_slots_written / need_slots`, and both halves are allocation-side counts of
the same thing, so the ratio is not a cross-quantity ratio.

## Predictions

| # | prediction |
|---|---|
| 1 | occupancy is **below 10 %** on every ladder input ≥ 30 KB |
| 2 | the **median** occupancy over those inputs is in **[0.5 %, 5 %]** |
| 3 | occupancy **varies more by SHAPE than by SIZE** — the two `syn_netlist_instances` rungs (2 500 vs 10 000, same generator, 4× the items) agree within a factor of 1.5, while the spread across different shapes exceeds 3× |
| 4 | `clear_calls == 0` on at least 8 of the 11 inputs, so `distinct_slots_written` is an exact count and not a bound on most of the ladder |
| 5 | `store_calls / distinct_slots_written` is **≥ 1.5** — the table is re-stored, not written once |
| 6 | occupancy is **lowest on `pinmux_reg_top.sv`**, the largest input and the one where the table costs the most absolute bytes (1 029 MB) |
| 7 | **NO REGRESSION with the census disarmed**: worst absolute wall-clock delta **< 1.5 %** and worst absolute peak-RSS delta **< 0.5 %**, new binary against the preserved `646475a2…` |
| 8 | all **eleven generated artifacts re-derive BYTE-IDENTICAL** — this slice edits `ast_pipeline/mod.rs`, which is in the emission-source digest, but changes no emission |

## ⭐ WHAT THE ANSWER DECIDES, WRITTEN DOWN BEFORE IT IS KNOWN

So that the ruling cannot be fitted to the number afterwards:

- **occupancy ≥ ~30 %** ⇒ a sparse structure is not smaller. A dense `u64` slot costs 8 bytes; a
  hash entry costs roughly 24–32 with its control byte and load factor, so the break-even is around
  **8/28 ≈ 29 %**. Above it the dense table is the RIGHT design and `.67`(c) should say so and stop.
- **occupancy well below 30 %** ⇒ the dense table is buying O(1) with no hashing at a memory price,
  and the fix is a TRADE. ⛔ **The north star rejects costs rather than trading them, and peak speed
  is the non-negotiable half** — so a sparse replacement that slows the parse is refused however
  much memory it saves. The candidate that is NOT a trade is narrowing the slot: the word is
  `(generation: u32, index + 1: u32)`, and if either half fits in fewer bits the table shrinks by
  the same factor with **identical O(1) access and no hashing**. That is what `.67`(c)(ii) should
  price first.
- Either way, **occupancy alone cannot rule**: the residency of the table is still an allocation
  figure (`-0128` named that gap), and a fix that shrinks allocation without shrinking RESIDENT
  bytes moves nothing. That measurement is `.67`(c)(iii).
