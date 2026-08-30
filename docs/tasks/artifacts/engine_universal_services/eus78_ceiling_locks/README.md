# `ENGINE-UNIVERSAL-SERVICES.78` — measured readings (PGEN-ENGINE-UNIVERSAL-SERVICES-0140)

Debug `cargo test --lib --features generated_parsers`, macOS/aarch64, default 256 MiB reservation
(`PGEN_PARSE_STACK_BYTES` unset). The ceiling locks in `rust/src/embedding_api.rs` pinned depths
and wall-clock budgets calibrated against the compiled-in 4 096-frame ceiling that
`ENGINE-UNIVERSAL-SERVICES.65` removed.

⚠️ Wall clock is the load-sensitive column. Where a reading was taken with other jobs on the host
it is labelled; the verdict columns (accept/reject, refusal cause, byte fields) are load-independent.

## BEFORE — the three locks, re-measured on this host

| lock | measured | verdict |
|---|---|---|
| `parser_embedding_systemverilog_over_ceiling_nesting_fails_in_bounded_time` | 400 parens **ACCEPTS** in 2.60 s | RED — `left: Success, right: Failure` |
| `parser_embedding_over_ceiling_diagnostic_names_the_recursion_ceiling` | same 400-paren input, no diagnostic to inspect | RED — `over-ceiling nesting must carry a diagnostic` |
| `parser_embedding_systemverilog_deep_nesting_yields_clean_diagnostic_not_process_abort` | 2 000 parens × two profiles, **2 904.586 s** | RED — `budget 1800s` |

✅ **ALL THREE RE-DERIVED IN THIS SLICE, not inherited.** The two fast rows read 2.60 s and 2.49 s
against `-0138`'s 2.58 s for the same input (0.8 %). The deep row read **2 904.586 s** against
`-0138`'s **2 920.7 s** — **0.55 %** apart.

⭐ **AND THAT AGREEMENT ANSWERS A WORRY THIS SLICE HAD WRITTEN DOWN IN ADVANCE.** The re-run shared
the host with two `cargo` builds, so it was expected to come in inflated and be usable only for its
VERDICT, not its number. It did not: 0.55 % against an idle-host measurement. ⇒ **this arm's cost is
dominated by single-thread parse work that other cores do not contend for** — worth knowing before
treating any wall-clock reading here as load-fragile. It is NOT a general licence: `-0128` measured
peak RSS moving 27.9 % under exactly this kind of load, so the property is specific to this
quantity.

## The mechanism, from `-0138` / `-0139`

* frames per paren level, bisected: **12.91 … 13.45**
* the ceiling fires between **600** (accept, 5.7 s) and **800** (reject at 9 726 frames, 21.8 s)
  parens in debug at the default reservation
* budget = `reservation − reservation / STACK_HEADROOM_RESERVE_DIVISOR` = 240 MiB of 256 MiB
* SystemVerilog per-frame cost, debug: **25 874 B**; release: **3 071 – 3 083 B**

⇒ predicted crossing = `240 MiB / 25 874 B ÷ 13.2` ≈ **748** nesting levels, which brackets the
bisected 720-accept / 750-reject boundary. The ladder's rungs are powers of two from 32, so it
crosses at **1 024**.

## AFTER — the derived locks

`./target/debug/deps/pgen-<hash> --test-threads=1 --nocapture embedding_api`, idle host:
**61 passed / 0 failed / 19 ignored in 320.56 s** — the WHOLE module, for **9.1×** less than the
single deep lock cost before it.

### The ladders, rung by rung (printed on every run, not only on failure)

```
SystemVerilog/Sv2017      crossed_at=1024 deepest_accepting=512  ladder=207.172s budget=2048s
  32:  0.056s ok     64:  0.116s ok    128:  0.330s ok    256: 1.109s ok    512: 4.093s ok
  1024: 201.469s FAIL  BudgetExhausted   depth=9934 used=251677984 budget=251658240 b/frame=25335

SystemVerilog/Verilog2005 crossed_at=2048 deepest_accepting=1024 ladder=  9.023s budget=2048s
  32:0.022s  64:0.034s  128:0.076s  256:0.220s  512:0.750s  1024:2.787s   (all ok)
  2048:   5.134s FAIL  NoFrameAffordable depth=8668 used=251643616 budget=251658240 b/frame=29031

Vhdl/Vhdl1076_2019        crossed=false   deepest_accepting=4096 ladder= 99.986s cap=256s
  32:0.019s 64:0.036s 128:0.099s 256:0.334s 512:1.243s 1024:4.792s 2048:18.817s 4096:74.648s (all ok)
  STOPPED before depth=8192: spent 99.986s + predicted next 296.135s >= budget 256s
```

### What the run established that the design had only argued for

⭐⭐ **THE TWO SystemVerilog PROFILES CROSS AT DIFFERENT DEPTHS, WITH DIFFERENT REFUSAL CAUSES.**
`Sv2017` crosses at **1 024** levels / **9.70** frames per level and refuses with `BudgetExhausted`;
`Verilog2005` crosses at **2 048** / **4.23** frames per level and refuses with
`NoFrameAffordable`. The deleted locks used ONE pinned depth for both. ⇒ a single literal cannot
serve two profiles of the same family, let alone two families.

⛔ **AND `NoFrameAffordable` REALLY FIRES, so accepting only `BudgetExhausted` would have been a
latent hang.** The ladder's stop condition takes both stack-grounds causes. Had it taken only the
one `.69`(d) calls *"the ONLY refusal that is a fact about the STACK"*, the `Verilog2005` ladder
would have treated its crossing as a non-event and kept doubling — 4 096, 8 192, … — until the
2 048 s hang detector fired. That arm was written from `.69`(d)'s enum, not from a measurement, and
this run is the measurement that justifies it.

⭐ **THE PREDICTIVE STOP DID THE JOB IT WAS ADDED FOR.** VHDL's climb ended at 99.99 s against a
256 s cap, because the growth the ladder had itself observed (74.648 / 18.817 = **3.97×**) put the
next rung at 296 s. A plain `elapsed >= cap` check would have bought that rung first and spent
**396 s** to learn the same thing — the ≈4× overshoot the rule exists to remove.

⚠️ **`used_bytes` vs `budget_bytes` at the two causes**: `BudgetExhausted` overshoots by
**19 744 B** (0.78 frames); `NoFrameAffordable` sits **14 624 B** UNDER budget (0.50 frames), which
is the definition of the cause. Both satisfy `used + bytes_per_frame > budget`, which is why the
anchoring lock checks that one inequality instead of branching on the cause.

## RED controls — every new lock proven able to FAIL

See `red_controls.txt`.

