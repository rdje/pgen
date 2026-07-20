# `PGEN-RGX-0078-0202` — drop-free internal cascade control error: DESIGN (pre-registered BEFORE code)

Leaf `RGX-0078.5.j.4` ordered member 5 of the `-0197` held program
(`docs/tasks/artifacts/held_carrier_batch/execution_contract.md`), session #177,
2026-07-20. Design first, then exactly ONE fix, adjudicated by the binding
`-0197` strict same-session corpus-geomean ratchet. This document is written
and banked before any code is touched.

## 1. The priced target (what the 4.422 ns is)

`-0177` priced the executed **`drop_in_place::<Result<(), ParseError>>`**
flat-self row at **43/11,875 · 36/11,805 · 43/10,673 samples** across the three
custody-pinned fused-path profiles = conservative **0.3500% / 4.422 ns**
(tail priced at zero). The mechanism connection is the `-0173` layout probe:
`ParseError` is **80 B / `needs_drop=true`** because the single cold
`ContextualError` variant carries `String + Vec<&'static str> + String`;
boxing keeps `needs_drop=true`; only a carrier with **no owning payload** is
drop-free. The `-0197` dependency map fixes the boundary: public `ParseError`
keeps its rich contextual data — the fix is a **drop-free INTERNAL cascade
control error converted only at the region boundary**, never an ABI-breaking
public index.

## 2. Current-vintage re-enumeration (custody `3814aea1`; `site_census.sh` → `site_census.txt`)

All counts are whole-file `perl -0777` matches (the standing generated-artifact
rule — the first draft of this census undercounted `try_parse_bare` to 0 and
the boundary call-outs to 83 via line-oriented grep; the banked numbers below
are the corrected whole-file counts).

- The fused region = `generated/regex_parser.rs` line 413832 → EOF: **490
  `cascade_*` fns + 30 `scan_*` fns**.
- Error constructions INSIDE the region: **`Backtrack` ×1054, `InvalidSyntax`
  ×56, `RecursionDepthExceeded` ×28, `ContextualError` ×0, `UnexpectedEof` ×0,
  `UnexpectedToken` ×0**. The two legacy variants are constructed in **zero of
  the 11 shipped artifacts** (their only repo construction site is the legacy
  `ast_code_generator.rs`, not the AST-based generator).
- In-region producers/edges of the error channel:
  - `parser.match_lit_ascii(` ×**1076** — the hottest producer; constructs
    `ParseError::Backtrack` per literal refutation; carries a `logger_enabled`
    fallback that is **statically dead on the bare path** (the emitted routing
    invariant: `bare_parse ⇒ !logger_enabled ⇒ !trace_enabled`).
  - `parser.match_string(` ×**42**; `parser.match_regex(` ×**0** in the regex
    artifact (the emission path exists for other grammars).
  - `parser.try_parse_bare(` ×**193** — **discards** its closure error
    (`Err(_)`): the direct site of the measured drop glue.
  - `parser.scan_*(` ×**231** (boundary scanners, shared with 273
    protocol-side call sites in the prelude).
  - `parser.parse_*(` ×**93** across 21 distinct rules — protocol BOUNDARY
    call-outs to sub-root/ineligible rules. **Rich inbound errors are REAL**:
    the prelude has 13 `create_contextual_error` callers on semantic
    `@requires`/scope/import-library paths reachable from those callees.
- The region's **only outbound error edge** = the **19 sub-root orchestrators**
  `cascade_<rule>() -> ParseResult<ParseNode<'input>>` (twin-dispatched at 108
  prelude sites; `.cascade_match_*` is called from the prelude **0** times).

## 3. The design

**LIB (`rust/src/ast_pipeline/mod.rs`, additive):**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CascadeControlError {
    InvalidSyntax { message: &'static str, position: usize },
    Backtrack { position: usize },
    RecursionDepthExceeded { position: usize, depth: usize },
    /// A rich/legacy boundary error parked in the parser's slot.
    Parked,
}
pub type CascadeResult<T> = Result<T, CascadeControlError>;
```

Mirrors exactly the three variants the fused region constructs; **32 B, `Copy`,
`needs_drop = false`** (pinned by a new lib test). `Result<(), CascadeControlError>`
is `Copy` ⇒ the measured `drop_in_place` monomorphization cannot be emitted for
the fused graph. `UnexpectedEof`/`UnexpectedToken` deliberately get **no mirror
variants** (zero construction sites in all 11 artifacts); with `ContextualError`
they take the total `Parked` route, so conversion is total for every public
variant while the hot inbound refutation (`Backtrack`) never touches the slot.

**EMITTER (cascade-plan-gated, parser-agnostic — all 11 regen):**

1. `cascade_match_*` signatures, the `__pgen_thin_result` binding, and both
   speculation-scope closures → `CascadeResult<()>`; the ~10 in-region
   construction sites (`cascade.rs` 600–616/645/879–899/1166/1407/1525 …,
   `scan.rs` untouched) → `CascadeControlError::*`.
2. `try_parse_bare<F, T>` → `try_parse_bare<F, T, E>` with
   `F: FnOnce(&mut Self) -> Result<T, E>` — the error is already discarded;
   with `E = CascadeControlError` the discard is drop-free.
3. **Bare twins at the hottest producer**: `match_lit_ascii_bare` /
   `match_string_bare` — the fast-path body verbatim minus the
   dead-by-invariant logger/trace branches, constructing
   `CascadeControlError::Backtrack` at the source (the `-0201` bare-twin
   precedent). Fused terminal emission calls the twins.
4. Fused call-site conversions in `mtb_match_atom_logic`:
   - `scan_*` / `match_regex` sites → explicit
     `Err(e) => return Err(parser.cascade_error_from_parse(e))`;
   - `parse_*` boundary call-outs → the same **total** conversion:
     `InvalidSyntax`/`Backtrack`/`RecursionDepthExceeded` map 1:1 (no slot
     write); anything else (`ContextualError`, legacy) is **parked** in a new
     parser field `cascade_parked_error: Option<ParseError>` and carried as
     `Parked`.
5. Outbound, at the **single** sub-root orchestrator emission site: map the
   match-fn error through `rehydrate_cascade_error` (mirrors 1:1; `Parked` →
   `take()` on the slot, `.expect` per the MTB-A drift stance — a live `Parked`
   without a parked value is codegen drift, not an input error). The 108 twin
   dispatch sites, all 273 protocol scan sites, and every protocol method are
   **untouched**.
6. Parser struct + init: the park slot field, cascade-gated next to
   `bare_parse`.

**Park-slot soundness.** A `Parked` marker is created only together with a slot
write; error propagation is synchronous and single-threaded, so at most one
marker is live; a marker discarded by a bare speculation rollback leaves a
stale slot value that the next park overwrites and nothing else reads; the
outbound `take()` therefore always observes its own park. Round-trip
`ParseError → CascadeControlError → ParseError` is the identity for every
variant — public error payloads are **byte-identical by bijection**.

**Scope-exact (what this unit does NOT own):** no protocol-method change, no
scan-fn signature change, no telemetry/memo/carrier (`-0203`) change, no
grammar change, no public-API change (the lib type is additive; `ParseError`
untouched). The observability twin runs the protocol graph verbatim.
`furthest_position` semantics are untouched (a separate max-update mechanism).

## 4. Expected effect (honest, per the `-0197` accounting)

Exact-current target **4.422 ns ≈ −0.38%** before replacement cost, plus
deliberately **unpriced** carrier-width upside (the fused error channel narrows
80 → 32 B `Copy` across 1,076 `match_lit_ascii` sites, 1,054 in-region
`Backtrack` constructions, and all fused propagation) minus replacement cost
(the conversion matches at 93 inbound + 231 scan-site error arms + 19 outbound
orchestrator sites). **No net is banked before measurement**; the strict
same-session ratchet adjudicates, exactly as `-0198`…`-0201`.

## 5. Acceptance (binding, pre-registered)

The `-0197` ratchet verbatim: canonical 2,189-cell PCRE2 corpus A/B vs base
probe `preserved_probes/regex_perf_probe_barediag_fba8d1df` (same-session
re-read), land only if the unrounded candidate corpus geomean is **strictly
below** the same-session base, verdict flips are **0/2,189**, and candidate MAX
≤ **483,583 ns**; battery green at the changed vintage (dual-feature lib suite
incl. the ALL-11 interpreter↔generated byte-identical oracle + the new layout
pin test, cert ×3 seeds 0/7/42, `ast_shape_contract_gate`, `duality_hunt_gate`,
`regex_pcre2_compile_oracle_gate`, clippy source-strict, all-11 regen train
with the ebnf fixed point + expected-delta review `overall=0` outside the
`-0202` surfaces). Otherwise: unconditional product reversion; only design +
rejection evidence commits. Heavy builds under
`scripts/run_with_memory_guard.sh --budget-mb 16384`; ONE heavy job at a time;
`caffeinate` custody for the A/B; alternated base/candidate rounds.
