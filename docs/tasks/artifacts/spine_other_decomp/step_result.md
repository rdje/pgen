# `PGEN-RGX-0078-0211` result — spine_other decomposition (read-only research slice)

Leaf `RGX-0078.5.j.4`, session #183, 2026-07-21. The `-0210` NEXT pointer
executed: selection on the still-honest `-0208` prices, then the selected
research slice run to an adjudication. **No product change.**

## Custody (the `-0208` capture reused — no new capture)

| gate | result |
|---|---|
| probe custody | `fxstore_a4067793` SHA-256 asserted (`a4067793…`) |
| raw captures | the three `-0208` raw SHAs asserted vs `raw_custody.json` |
| legitimacy | floor, probe, and ALL 11 artifacts UNCHANGED since the capture (`-0209`/`-0210` were SHA-verified byte-restores) |
| re-sum | spine_other per-symbol total **203.7475 ns = the `-0208` bucket price exactly** (REFUSE-gated); VIEW 3c re-sums to the same total (REFUSE-gated) |
| convention | weighted-ns = band log-shares × floor 1037.804231341058 ns; noise 23.657 ns carried |

## Selection adjudication (the three banked candidates)

1. **SELECTED — spine_other decomposition** (203.75 ns ≈ 8.6× noise): the only
   population large enough to host a ≥noise mechanism against the −8.5%
   margined-bar gap.
2. **BANKED — construct-LESS fork**: whole population ≈30–55 ns gross ≤ half
   the gap; `-0210` law prices its sub-1 µs branch-tax risk; MUST-SURFACE
   before code per #175 if ever selected.
3. **BANKED — `-0170` sub-noise batching**: ~23 ns combined ≈ 1.0× noise gross
   BEFORE replacement costs — weakest.

## The decomposition (instrument: `decompose_spine_other.py` → `decomposition.txt`)

**VIEW 1 (per-symbol, complete, 170 symbols):** a long-tail population — top:
cascade_regex 25.08 (sub-1 µs-heavy = fixed entry ceremony), parse_pattern
19.04 (upper-band = input-scaling), parse_regex::{closure} 15.82 (sub-1 µs),
scan_letter 11.10, parse_capture_open::{closure} 11.05, cascade_match_alternative
10.35, cascade_match_class_atom 10.22, then <8 ns each. **No single symbol
reaches noise** — the mass is a MECHANISM, not a hotspot.

**VIEW 2 (annotated clusters):** the recurring mechanism across every top
symbol is **72-byte (0x48) result-payload traffic**:

- `parse_regex::{closure}`: 4.12 ns pure q-register stack-to-stack shuffles of
  result payloads; 3.67 ns copying the callee's out-slot after `cascade_regex`
  returns; 1.61 ns copying the final result to the caller's out pointer.
- `cascade_regex`: the 7-word checkpoint spill ceremony before each
  `cascade_match_piece` call (offsets 0xF0/0x108/0x120/0x150/0x178/0x198/0x238
  — the parser register is x19 by offset-proof); a 24-byte-element Vec build
  loop feeding `cascade_build_piece`; a 127-sample epilogue spike on an
  864-byte frame.
- `parse_pattern`: 72-byte ParseNode stores into the typed arena (×72 element
  addressing + the arena borrow-flag check at parser offset 0x350) + large
  q-register sp-to-sp copies of node payloads.
- `scan_letter`: 4.54 ns on PROLOGUE + position/input-view loads and 3.82 ns
  on the success-path ParseNode materialization through the out-pointer
  (header q-store + text-slice ptr/len + rule-name `&'static str` "letter"
  (len 6, static addr, in-disassembly) + start/end span) + a 262-sample
  epilogue spike.

**VIEW 3b (instruction-class × family):** memory-class = **169.5 of 203.7 ns
(83%)**; control ≈ 8.3 ns; calls ≈ 0.1 ns. Matching/dispatch compute is NOT
the residual — memory traffic is.

**VIEW 3c (mechanical transport tally, re-sums exactly):**

| mechanism class | ns | × noise |
|---|---|---|
| x_frame_spill (sp/fp: frame ceremony + on-stack result staging) | **73.33** | 3.10× |
| x_scalar_state_or_elem (state loads + arena/Vec elements + OUT-POINTER result writes) | 86.49 | 3.66× |
| q_payload_copy (sp) | 10.35 | 0.44× |
| q_payload_copy (heap/arena) | 11.00 | 0.47× |
| other_alu | 14.16 | 0.60× |
| control | 8.32 | 0.35× |
| call | 0.10 | — |

sp-relative traffic alone (73.33 + 10.35 = **83.68 ns ≈ 3.54× noise**) is
frame/staging ceremony — the direct product of the by-value 72-byte result
ABI; the out-pointer result writes inside the 86.49 scalar bucket add more.

## ⭐ THE NAMED MECHANISM — the by-value 72-byte committed-result carrier

`ParseNode<'input> = { rule_name: &'static str (16 B), content:
ParseContent<'input> (≈40–48 B; largest variant Quantified(Vec, &'static str)),
span: Range<usize> (16 B) } ≈ 72 B`, returned by value as
`ParseResult<ParseNode>` through an out-pointer at EVERY rule boundary. Every
boundary therefore pays: build the 72-byte struct in the callee frame → sret
write → caller copies to a local slot → caller copies into the next frame's
argument slot / a Vec element / an arena slot. The frames sized to hold
multiple such temporaries (0x360 in cascade_regex) then pay the
prologue/epilogue and spill ceremony on top. This mechanism also feeds the
adjacent out-of-target populations (arena_alloc 72.17, build_value 73.74,
other_text OUTLINED q-copy helpers ≈25+) — the spine_other pricing here is a
LOWER bound on the carrier's true whole-corpus cost.

## ADJUDICATION

- **The mechanism is above noise with margin** (sp-staging alone 83.7 ns ≈
  3.5× noise; the addressable carrier family conservatively ≥100 ns inside
  spine_other + adjacent populations). It is the largest honest lever left on
  the road to the −8.5% margined bar.
- **The designed fix = the RESULT-CARRIER SLIMMING unit** (banked as the next
  session's design fork): shrink `ParseNode` toward ≤48 B so every staging
  slot, q-chain, Vec element, memo entry, and arena node shrinks
  proportionally. Concrete pre-scoped ingredients (to be design-prereg'd):
  (a) `rule_name: &'static str` → `u16` rule-id + static name table
  (wire-format-identical serde via id→name lookup; the `-0104/-0105`
  REPRESENTATION precedent for public-variant surgery with byte-identical
  JSON); (b) `Quantified(Vec, &'static str)` kind → `u8`/enum with
  Display-compatible serialize; (c) optionally `span: Range<usize>` →
  `u32` pair. Alternative attack shapes priced and REFUSED for now:
  return-by-arena-ref ABI (unbounded arena-garbage hazard on
  backtracking-heavy/pathological inputs — needs its own careful fork);
  cold-path outlining/frame tuning (compiler-behavior class, unpriceable —
  the refuted target-cpu/PGO family).
- **HOLD for THIS session (no fix implemented):** the unit is a public
  core-type representation change (pub struct/enum fields + serde surface +
  the embedded-ebnf cold-bootstrap tax + the ADDITIVE-transient-migration
  rule for emitted types) — design width that exceeds what a same-session
  unit can carry at design-prereg quality. Per the `-0197` fresh-session
  rationale (design quality is the point of the one-fix/fresh-session
  contract) the fix is BANKED as the NEXT session's ONE unit: design prereg
  BEFORE code, ratchet-adjudicated vs base `fxstore_a4067793`, all-11 regen
  + full battery, wire-format byte-identity as an explicit oracle.
- Note the risk profile of the banked unit is the GOOD kind (`-0210`
  contrast): it strictly REMOVES memory traffic per boundary — no new
  branches, no ceremony, no capacity-carrying state.

## Reproduction

```sh
python3 docs/tasks/artifacts/spine_other_decomp/decompose_spine_other.py
# REFUSE-gated: probe SHA, the three banked raw SHAs, both re-sum identities.
```
