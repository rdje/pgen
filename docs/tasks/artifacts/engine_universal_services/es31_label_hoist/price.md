# ENGINE-UNIVERSAL-SERVICES.31 (b) — the measured price of hoisting class L

Produced by `probe.sh` beside this file (two arms, generated into a mimic
`<work>/root/{generated,rust}` tree so both `-o` strings are byte-identical to what `rust/Makefile`
passes). Recorded 2026-08-17, session #243, at `da32012d` + the slice-2 emitter change.

⛔ **Never re-derive these from `sites × Δlen` and call it a measurement.** The arithmetic happens
to reproduce them here — see "where the bytes went" below — and that is a *result of this run*, not
a licence. `.31` acceptance (b) forbade the shortcut, and the only reason we know the shortcut would
have worked is that the shortcut was not taken.

## ARM 1 — un-hoisted (a `"<path>"` literal at every `Logger::log_*` site, the emission through
## `PGEN-ENGINE-UNIVERSAL-SERVICES-0069`) vs ARM 2 — hoisted (one `const PGEN_SOURCE_LABEL`)

| family | ARM 1 bytes | ARM 2 bytes | delta | ARM 1 sites | ARM 2 sites | bytes/site |
|---|---:|---:|---:|---:|---:|---:|
| json | 688 577 | 686 144 | −2 433 | 208 | 1 | −11.70 |
| regex | 38 320 117 | 38 172 358 | −147 759 | 11 371 | 1 | −12.99 |
| return_annotation | 2 122 576 | 2 106 652 | −15 924 | 640 | 1 | −24.88 |
| rtl_const_expr | 2 119 412 | 2 107 231 | −12 181 | 557 | 1 | −21.87 |
| rtl_frontend | 10 218 076 | 10 171 367 | −46 709 | 2 339 | 1 | −19.97 |
| scratch | 262 372 | 261 403 | −969 | 69 | 1 | −14.04 |
| semantic_annotation | 12 181 378 | 12 083 068 | −98 310 | 3 644 | 1 | −26.98 |
| systemverilog | 143 801 858 | 143 072 432 | **−729 426** | 34 738 | 1 | −21.00 |
| systemverilog_preprocessor | 2 887 789 | 2 859 960 | −27 829 | 821 | 1 | −33.90 |
| vhdl | 11 761 507 | 11 727 370 | −34 137 | 2 850 | 1 | −11.98 |
| ebnf | 11 684 875 | 11 668 706 | −16 169 | 3 245 | 1 | −4.98 |
| **TOTAL** | **236 048 537** | **234 916 691** | **−1 131 846** | **60 482** | **11** | |

**−1 131 846 bytes = −0.48 % of the generated tree**, and the embedded `-o` site count falls
**60 482 → 11**, i.e. one per artifact.

⭐ **The bytes are the smaller half of the result.** The load-bearing consequence is that *the
artifact's size is no longer a function of its own output path at 60 482 sites but at one*: a
3-character difference in the `-o` spelling moved the SystemVerilog parser by **109 038** bytes and
now moves it by **3**. That coupling is the trap `TOOLBOX.md` 5.6 exists for; it has inverted three
published readings in this repository, one of which founded a task leaf on two wrong hypotheses.
⛔ **Reduced is not removed** — one site is still one byte per character, and the effect is now much
harder to notice, so every normalisation and site-count assertion in the tree is kept at full
strength rather than relaxed.

## ARM 1 reproduces the shipped tree exactly (so the A/B measures what ships)

All eleven ARM 1 artifacts were byte-identical (sha256) to the `generated/` tree as it stood before
this slice; all eleven ARM 2 artifacts are byte-identical to it afterwards. The mimicry is proven,
not assumed.

## The change is EXACTLY the substitution (acceptance (c), source level)

For all eleven artifacts, ARM 2 **minus** its `const PGEN_SOURCE_LABEL: &str = "<path>";`
declaration, with every `PGEN_SOURCE_LABEL` token replaced by the literal that declaration holds, is
**byte-identical to ARM 1**. Nothing else moved, and the `file` argument of every emitted
`Logger::log_*` call therefore still receives the same string. `reconstruct.py` refuses (exit 2)
rather than compare when an artifact carries zero or two declarations, and its RED control — the
same reconstruction with a deliberately wrong literal — fails as designed.

## The runtime leg of (c)

Traces captured before and after the regeneration, with the release probe rebuilt against each:

| grammar | command | lines | lines carrying the `file` label | sha256 before → after |
|---|---|---:|---:|---|
| json | `--parse json … --trace` | 1 669 | 1 668 | `55208a9fac66df98…` → identical |
| systemverilog | `--parse systemverilog … --trace-rules module_declaration` | 12 920 | 12 760 | `49122e6bcd1282ba…` → identical |

**14 589 trace lines, 14 428 of them carrying the label, byte-identical.** No diagnostic loses
information.

## Where the bytes went — measured, then explained, in that order

Per artifact the delta equals `−sites × (len(literal) − len("PGEN_SOURCE_LABEL")) + len(declaration
line)` **exactly**, in all eleven rows, with no remainder:

```text
json  −2 433 = −208 × (29 − 17) + 63       regex   −147 759 = −11 371 × (30 − 17) + 64
sv  −729 426 = −34 738 × (38 − 17) + 72    ebnf     −16 169 =  −3 245 × (22 − 17) + 56
```

⭐ ⇒ **`prettyplease` did not re-wrap.** `.31` acceptance (b) forbade the arithmetic on the grounds
that it would, and measured, it does not: every log site already occupies its own line, so
shortening one argument moves no line break. The prohibition was still right — that sentence is a
*measurement*, and it took two arms to be able to write it.

## Runtime cost: zero, proven by the shipped instrument

`make -C rust SHELL=/bin/bash sv_parse_cost_ratchet` re-measured the pinned 192-file sample and
`entries.tsv` came back **byte-identical** to the baseline: `entries 416 841 264`,
`committed 7 124 616`, `memo_hits 186 981 263`, `lr_entries 12 440 690`, `lr_committed 514` — all
five unmoved. The published LR-family share re-derived over all 16 336 corpus files as **2.741 %**,
reproducing the carried constant exactly. Only the identity sha256 rows moved.
