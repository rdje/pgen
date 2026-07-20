# Position-progress carrier audit

## Contract

`furthest_position` is not redundant with the current `position`. It is the
monotone maximum of positions observed at rule-entry/emulated-entry points.
Speculation restores `position` but deliberately never restores
`furthest_position`, so a failed deep branch can leave:

```text
position = outer checkpoint
furthest_position = deepest attempted rule entry
```

The public generated-parser accessor is consumed by the parse harness and
parser registry on accepted and rejected outcomes. Differential gates compare
the exact value with the interpreter. Computing it from final `position`, or
updating only on final failure, changes this observable contract.

The source-pinned writer classes are:

- protocol rule-entry preambles in `ast_based_generator.rs`;
- inlined-frame and exact FIRST/Q refutation emulation in the same emitter;
- fused match-rule entries in `cascade.rs`;
- scan/recognizer entries in `scan.rs`.

Every writer performs an unsigned monotone max. The carrier is initialized to
zero and has no decrement/reset during a parse.

## Complete preserved machine mechanism

The earlier direct-parser row intentionally included only `[x20, ...]`
instructions that received samples. The complete target-region scan finds:

- 40 `furthest_position` loads: one `cascade_match_piece` entry load through
  incoming parser register `x1`, plus 39 inner `[x20,#0x4e0]` loads;
- 43 compare / unsigned-`b.ls` / conditional-store update arms (three CFG
  joins share a loaded maximum across multiple arms);
- 43 stores to `[x20,#0x4e0]`;
- 227 direct `[x20,#0x498]` position accesses, all required for cursor
  semantics rather than removable furthest-carrier work.

The direct row re-sums exactly as position accesses 53/55/50 + inner furthest
loads 13/18/19 + stores 0/0/0 = **66/73/69**. The complete carrier adds the
entry load's **8/21/25**, which the x20-only classifier correctly could not
see. All furthest loads therefore sample **21/39/44** and price
**1.674630678 ns**; update stores sample zero and required compare/branch
control prices **0.051290423 ns**. The whole direct position row prices
**3.548562496 ns**, but its cursor accesses and max comparison remain required.

## Replacement feasibility

Three representations were separated:

1. **Current parser field.** The callee loads the maximum, compares, and
   conditionally stores.
2. **Forwarded `&mut usize`.** This is lossless and compatible with recursive
   calls, but pinned safe-Rust codegen emits the same load, compare, branch,
   and store. It changes the base register, not the mechanism. Strict saving:
   zero.
3. **By-value `usize`.** The local max compiles to register-only `cmp+csel`,
   but correctness requires every protocol method, fused matcher, scanner,
   recognizer, recursion edge, and refutation-emulation path to receive and
   return the updated value. That widens the transitive call/return protocol
   (or an already-wide `ParseResult`) and still needs an entry load and final
   store. Those replacement costs do not exist in the preserved binary and
   cannot be priced as zero.

Safe Rust also cannot simultaneously pass `&mut Parser` and a separate
`&mut` reference to one of its fields. A disjoint-state design would first
need to split the parser/cursor representation, expanding scope well beyond a
local forwarding change.

The most generous defensible ceiling credits all 39 inner loads—**13/18/19 =
0.828243916 ns**—while charging none of the transitive ABI, result-width,
entry-load, final-store, or register-pressure cost. Even that raises the
accumulated 30% margin only from **1.841692968 ns** to **2.090166143 ns**.
Strict net remains zero; implementation is **HOLD**.

## Custody

`classify_position_progress.py` pins all three emitter sources, its safe-Rust
probe, rustc, the preserved probe, all three raw captures, target symbols,
machine counts, prior accepted/held mechanisms through `-0193`, and arithmetic.
It refuses if any input or structural count changes.
