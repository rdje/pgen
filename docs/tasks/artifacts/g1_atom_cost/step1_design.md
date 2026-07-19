# G1 STEP-1 — per-atom cascade cost, pinned at INSTRUCTION level (`PGEN-RGX-0078-0164`)

Session #164, 2026-07-19. Toolbox-first, **docs-only** — zero code, grammar, or
artifact changes; the release floor is byte-untouched. Steering per the `-0161`
director directive: the campaign CALL-OFF trigger is **corpus GEOMEAN < 1 µs**
(now 1,263.4 ns ⇒ ≈ −21% needed); corpus MAX is SETTLED.

This is the STEP-1 the `-0163` self-audit mandated: the "cannot-fail arm" license
plus a **fused-path-valid** census lane. It delivers both — and in the process
**refutes the primary lever the `-0163` re-aim had named**, on evidence stronger
than a census: the disassembly of the actual release floor binary.

## Method + custody (proven in-run)

- Release floor probe `rust/target/release/regex_perf_probe` **byte-identical to
  `preserved_probes/regex_perf_probe_c1_1d3fa0ee`** (sha256 `1d3fa0ee…` both).
  Artifact `generated/regex_parser.rs` = canonical **`cb322b75`** (`custody.txt`).
- Instrument: `otool -tV` on the fat-LTO+mimalloc release probe — the SAME binary
  the banked floor numbers were measured on. This satisfies the `-0163` **STANDING
  INSTRUMENT RULE** directly: it reports what the fused path *actually executes*,
  with no observer effect at all (reading a binary cannot flip the twin).
  Excerpts banked: `disasm_scan_letter.txt`, `disasm_scan_letter_fastpath_tail.txt`.
- Static emission-site census over ALL 11 generated parsers, banked BEFORE any
  regen (`static_emission_census.txt`) — the C1 census pattern (static counts over
  emitted code), not a runtime counter census.

## 1. ⛔ REFUTED — G1(i) "refutation-free speculation elision" (the `-0163` primary lever)

The `-0163` re-aim named this as G1's primary target: where a byte-class `match`
arm already proves the single-byte scan cannot fail, the speculation block
(`position` / `deriv_events.len()` / `deriv_boundary.len()` saves + the unreachable
rollback arm) is dead weight to be elided by a license predicate.

**The compiler already does it.** `scan_letter` in the release binary
(`disasm_scan_letter.txt`) is not 52 speculation-wrapped arms — LLVM collapsed the
whole construct:

```
ldr  x19, [x1, #0x498]     ; position
ldr  x22, [x21, #0x2f8]    ; input.len()
cmp  x19, x22
b.hs <Err>                 ; pos >= len
ldrb w8, [x23, x19]        ; load byte
sub  w8, w8, #0x41         ; - 'A'
cmp  w8, #0x39
b.hi <Err>                 ; outside A..z
ldrh w11, [x9, x8, lsl #1] ; jump table
br   x10
```

One range check and a jump table. **Zero speculation bookkeeping, zero rollback
arms, and the per-arm `match_lit_ascii` byte comparison is gone entirely** — LLVM
proved each arm's literal compare redundant against the dispatch byte and folded
it. There is nothing for an emitter-side license to remove here.

⇒ **G1(i) is REFUTED for the Tier-1 (intraprocedural, single-byte-literal)
population**, which is where the `-0163` exemplar (`cascade_match_literal_char`'s
letter chain) lives. Recorded as measured-refuted, not parked.

Honest residual: the *interprocedural* case (Tier 2 — an arm whose body calls a
separate non-inlined cascade/scan function, e.g. `cascade_match_literal_char`'s
letter arm calling `scan_letter()`) **does** keep its speculation wrapper, because
the callee's `Err` is opaque to the caller. But the surviving cost there is a
handful of register saves/restores around a call that already costs far more —
priced below the noise floor, and **not** worth an emitter change on its own.

## 2. ⭐ What the fused path ACTUALLY pays per atom (three costs, instruction-pinned)

Reading the fast-path tail LLVM emits for a matched letter
(`disasm_scan_letter_fastpath_tail.txt`) names the real population:

### G1-A — the UTF-8 re-slice of already-matched ASCII (SAFE, ~3 lines, all 11 parsers)

```
add  x24, x19, #0x1        ; position += 1
str  x24, [x25]
bl   core::str::…SliceIndex::get      ; &self.input[start..end]
…
ldrsb w8, [x0, x19]        ; is_char_boundary(start)
cmn   w8, #0x41
b.gt  …
ldrsb w8, [x0, x3]         ; is_char_boundary(end)
cmn   w8, #0x41
bl    core::str::slice_error_fail     ; panic scaffolding
```

`match_lit_ascii` ends with `return Ok(&self.input[start..end]);`
(`generated/regex_parser.rs:413266`). Slicing a `&str` by a byte range goes through
`str`'s `Index<Range<usize>>`, which performs **two UTF-8 char-boundary checks plus
bounds checks and can panic** — even though the branch was entered only after
`self.input.as_bytes()[start..end] == *expected_bytes` proved the bytes equal an
**ASCII** literal.

⭐ **Why the optimizer cannot remove it:** the check is not dead code — it is a
*potential panic*, an observable side effect LLVM must preserve. That is precisely
why this survives fat-LTO while the speculation wrapper (§1) does not.

**The fix is safe, total, and needs no `unsafe`:** return the `expected: &'static
str` literal itself. It is byte-identical to `input[start..end]` **by construction**
(the equality was just tested), and `&'static str` coerces to `&'input str`. This
deletes the slice-index call, both boundary checks, the bounds check and the panic
scaffolding, per atom.

Provenance audit (the one real risk — recorded as cleared): **zero `as_ptr()`
occurrences in any generated parser**, and the only `as_ptr()` uses in `rust/src`
are on gen-AST alternative slices (`parse_harness_interpreter.rs:650/819`) and
census vectors (`regex_construction_census_probe.rs:143/154`) — never on a matched
`&str`. Nothing depends on the returned slice's address, only on its bytes. Spans
are tracked separately through `parser.position`.

Additional strengthener: the **majority of emitted call sites discard the return
value entirely** (`parser.match_lit_ascii(")", b")")?;` — 88 sites for `)` alone in
regex), so today that slice work is pure waste that only the panic-preservation
rule keeps alive.

### G1-B — `ParseError` is fat and non-trivially droppable (parser-agnostic)

The same tail contains:

```
bl __ZN4core3ptr…drop_in_place$LT$core..result..Result$LT$$LP$$RP$$C$pgen..ast_pipeline..ParseError$GT$$GT$
```

**A real drop-glue call on the per-atom success path.** Root cause
(`rust/src/ast_pipeline/mod.rs:655-681`): `ParseError::ContextualError` carries
`message: String` + `rule_stack: Vec<&'static str>` + `input_context: String`. So
`ParseError` is ~80 bytes AND non-trivially droppable — while the hot path emits
essentially only `Backtrack { position }` (one `usize`).

Every `ParseResult` in the fused path therefore pays a wide carrier and drop glue
sized for a variant it never constructs. **Lever:** box the cold payload
(`ContextualError(Box<ContextualErrorData>)`) ⇒ `ParseError` becomes small and
trivially droppable ⇒ the drop-glue call disappears and every `?` / speculation arm
/ `ParseResult` return narrows. Parser-agnostic; benefits all 11 parsers.

### G1-C — per-atom 72-byte `ParseNode` materialization + arena copy + tape push

The `cascade_match_literal_char` letter arm's post-`scan_letter` code
(`disasm_scan_letter.txt`, caller context) is an arena bump-allocate with a
**72-byte** node copy:

```
ldr x8, [x20, #0x300]      ; arena
ldr x9, [x8]  ; cbnz       ; RefCell borrow flag  (set to -1, cleared after)
ldr x9, [x8, #0x18]        ; len
ldr x10, [x8, #0x8]        ; cap   -> capacity check
add x11, x9, x9, lsl #3
lsl x11, x11, #3           ; index * 72
ldp/stp q0,q1 …            ; 72-byte copy, stack -> arena
```

Stride 72 confirms the layout: `ParseNode { rule_name: &'static str (16),
content: ParseContent (40), span: Range<usize> (16) }` = **72 bytes**
(`mod.rs:1122`). Per matched atom the fused path pays: build the node on the stack,
a `RefCell` borrow set/clear, a capacity check, a **72-byte memcpy** into the arena,
and an 8-byte pointer push onto the `deriv_boundary` tape
(`generated/regex_parser.rs:94` — `Vec<&'input ParseNode<'input>>`, consumed later
by the build pass at `:413790` via `deriv_b_cursor`).

This is exactly the population the `-0162` release sampling independently measured
— allocator+memmove+arena **27–31% self across all three bands**,
`typed_arena::alloc_extend` 5.3–8.6%, `_platform_memmove` 5.3–7.4% — so G1-C is
**confirmed as the largest per-atom cost**, now with a mechanism rather than a
share. It is also the highest-risk (a carrier/representation change touching the
build pass), which is why it is sequenced last.

## 3. The static emission-site census (all 11 grammars, banked BEFORE regen)

`static_emission_census.txt` — counts of emitted sites over the shipped artifacts.

| grammar | `match_lit_ascii` | `arena.alloc` | `deriv_boundary` | `cascade_match_*` | spec blocks |
|---|---|---|---|---|---|
| ebnf | 378 | 380 | 1,249 | 130 | 302 |
| json_parser | 0 | 23 | 134 | 9 | 30 |
| regex_parser | 2,774 | 1,531 | 4,957 | 235 | 2,414 |
| return_annotation_parser | 98 | 106 | 340 | 32 | 14 |
| rtl_const_expr_parser | 80 | 148 | 269 | 48 | 70 |
| rtl_frontend_parser | 243 | 1,228 | 1,547 | 169 | 538 |
| scratch_parser | 10 | 2 | 21 | 2 | 4 |
| semantic_annotation_parser | 511 | 290 | 2,361 | 114 | 662 |
| systemverilog_parser | 513 | 13,544 | 13,238 | 1,046 | 744 |
| systemverilog_preprocessor_parser | 43 | 329 | 482 | 62 | 116 |
| vhdl_parser | 0 | 1,429 | 1,755 | 216 | 596 |
| **total** | **4,650** | **19,010** | **26,353** | **2,063** | **5,490** |

⚠️ **Honest reading of this census (no silent over-claim).** These are **static
emission-site counts**, i.e. *how broadly each lever applies across the parser
family* — they are **not** runtime frequencies and must never be multiplied by a
per-site cost to price a win. The runtime weighting comes from the independent
`-0162` release sampling (§2 above) and the entry model
(`min_ns ≈ 306 + 21.3 × entries`). What the census establishes:

- G1-A and G1-B are **parser-agnostic and broad** (every grammar carries
  `ParseResult`; 9 of 11 carry `match_lit_ascii` sites).
- ⚠️ `json_parser` and `vhdl_parser` emit **zero** `match_lit_ascii` sites — they
  take a different terminal-matching route, so G1-A does **not** reach them. Named
  explicitly so no cross-family claim is made from a regex-weighted number.
- G1-C's site population is the largest everywhere and dominant in SV (13,544
  `arena.alloc` / 13,238 boundary sites).
- **No census surprise fired** (the `-0163` stop-condition): the distribution is
  ordinary and consistent with the sampled populations.

## 4. ▶️ Program, sequenced cheapest-and-safest first

| lever | risk | scope | expected |
|---|---|---|---|
| **G1-A** literal-return in `match_lit_ascii` | LOW (safe Rust, byte-identical by construction, provenance cleared) | runtime helper; regen all 11 | small but broad per-atom cut |
| **G1-B** slim/box `ParseError` | LOW–MED (type change; emitted, so mechanical regen) | parser-agnostic | removes per-`ParseResult` drop glue + narrows carrier |
| **G1-C** per-atom node/tape representation | HIGH (build-pass carrier change) | structural | the measured 27–31% cluster |

**One lever per slice**, in that order, each with its own A/B and full battery.

### Acceptance bands — G1-A (the next emission slice)

- **Primary:** corpus geomean **−1 … −4%** off the 1,263.4 ns floor.
- **Falsification:** better than **−0.5%** ⇒ the lever is below bar ⇒ **REVERT**
  under `.5.d.2`. *Pre-adjudicated:* G1-A is complexity-**REMOVING** (it deletes a
  slice + panic path), so if it lands neutral-but-proven it follows the **M1
  precedent** (`-0154`): landed as a proven simplification with the **perf claim
  refused and no floor number banked** — never banked as a win it did not earn.
- **Bench:** ±2% guard; **corpus MAX** must not regress (settled at 483,583 ns).
- **Correctness (immovable):** verdict-identity on all 2,189 corpus cells + 39
  ladder cells; ALL-11 differential-equivalence **byte-identical**; cert ×3 seeds
  `268/9/259/0` byte-exact; typed-diff 8/8; PCRE2 oracle tuple exact;
  shape/duality/clippy PASS.
- **Artifact:** size delta measured; >5% growth ⇒ stop-and-reassess.
- **A/B:** base = the floor probe `1d3fa0ee`, floor-validated best-of-3 first;
  alternated 5×2000 + full corpus + ladder under `caffeinate -ims`, ONE runner,
  guard `--budget-mb 16384`.

## 5. Lessons banked

1. ⭐ **Disassemble the actual release binary before designing an emitter-side
   optimization.** Two consecutive re-aims (`-0163` counter-twin, `-0164` here)
   were both caused by reasoning about *emitted source* as if it were *executed
   code*. Fat-LTO had already deleted the `-0163` target. The emitted-source form
   is not the machine the fused path runs.
2. ⭐ **A check that can PANIC is not dead code.** The UTF-8 boundary check
   survives the optimizer precisely because it is a preserved potential panic —
   which is also why removing it at the source is a genuine win, not a
   compiler-redundant one. Good heuristic for finding surviving hot-path waste:
   look for what the optimizer is *forbidden* to remove.
3. ⭐ **A cold error variant taxes every hot return.** `ParseError` is sized and
   drop-glued for `ContextualError`, a variant the hot path never constructs.
   Check `Result` error types for width and drop-triviality in any hot path.
