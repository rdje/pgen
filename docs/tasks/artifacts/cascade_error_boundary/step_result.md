# `PGEN-RGX-0078-0202` — result: **LANDED** (the fifth fix through the `-0197` strict-geomean ratchet — the first clearly-outside-noise win of the held program)

## What landed

The pre-registered design (`design_prereg.md`) + exactly one fix:

- **LIB** (`mod.rs`): additive `CascadeControlError` — the drop-free `Copy`
  internal error carrier for the fused cascade graph (3 mirror variants +
  `Parked`; 32 B, `needs_drop = false`, layout-pinned by the new
  `cascade_control_error_is_copy_dropfree_and_narrower_than_parse_error`
  test) + `CascadeResult<T>`. Public `ParseError` untouched.
- **EMITTER** (`cascade.rs` / `ast_based_generator.rs`): the fused match
  channel (`cascade_match_*`, speculation scopes, thin-memo binding,
  `try_parse_bare`'s closure bound) switched to `CascadeResult`; all
  in-region error constructions to `CascadeControlError`; NEW bare terminal
  twins `match_lit_ascii_bare`/`match_string_bare` construct the Copy error
  at the source (1,076 + 42 fused sites; diagnostic branches statically dead
  by the routing invariant; the defensive UTF-8 rich error PARKS); TOTAL
  inbound conversion `cascade_error_from_parse` at the 93 protocol
  boundary call-outs, 231 scan sites, and the `match_regex` emission path
  (Copy variants map 1:1, rich/legacy errors park in the new
  `cascade_parked_error` slot); outbound rehydration
  `rehydrate_cascade_error` at the single sub-root-orchestrator emission
  site (bijective; `Parked` → `take().expect`, the MTB-A drift stance).
  Protocol graph, scan signatures, and all 108 twin-dispatch + 273 protocol
  scan sites untouched. One scan-emitter text-pin re-anchored to the new
  conversion form.

## Adjudication (binding `-0197` ratchet — `adjudication.txt`)

- Corpus (2,189 cells): unrounded geomean **1165.6890898132085 →
  1100.9114487671768 ns = −5.5570%** — strict same-session decrease ⇒
  **LAND**. ⭐ Unlike `-0198`…`-0201`, this is **well OUTSIDE the ≈2.3%
  noise span (2.4×)** — the first individually-measurable win of the held
  program.
- Verdict flips **0/2,189**; candidate MAX **442,750 ≤ 483,583 ns** settled
  bound (same worst cell `line_725`, itself −4.2%).
- Floorval custody: base bench geomean 1799.5 vs banked 1805.3 = **−0.32%** OK.
- Bench steering: geomean 1813.3 → 1708.8 ns (**−5.76%**); `digit_sequence`
  −14.62%, `email_basic` −10.72%, `literal_simple` −7.58%, `alternation`
  −6.11%, `url_simple` −3.88%, `capture_groups` −2.69%, `character_class`
  −1.65%, `anchor_complex` +2.24% (single-pattern jitter).
- Honest magnitude note: the exact-current target was **4.422 ns ≈ −0.38%**
  (the measured drop-glue row alone); the delivered −5.56% is dominated by
  the deliberately **unpriced carrier-width upside** the `-0177` re-price
  named — the fused error channel narrowing 80 → 32 B `Copy` across every
  construction, propagation, and discard in the region (`-0172`'s
  memory-traffic-dominant spine finding is consistent with width being
  worth far more than drop glue).

## Floor re-baseline

Floor of record moves to the candidate's same-session reading: corpus
geomean **1,100.9 ns** (unrounded 1100.9114487671768), bench ≈**1,708.8 ns**,
settled MAX bound unchanged (483,583 ns). Cross-session drift this time was
small: the base re-read 1165.69 vs the `-0201` banked 1163.91 = +0.15%.
The margined <1 µs call-off bar (unrounded geomean ≤ **950.0 ns**, director
#176) now needs **−13.7%** from the floor-of-record reading.

## Custody

- Base = `preserved_probes/regex_perf_probe_barediag_fba8d1df`
  (`fba8d1df…`, cmp-verified against the pre-change release probe, SHA
  banked pre-regen). Candidate probe preserved
  `preserved_probes/regex_perf_probe_cascerr_e88beee8` (`e88beee8…`,
  cmp-verified).
- Regex artifact: pre `3814aea1…` → post `c3d91959…`
  (`artifacts_pre/post_regen.sha256`); all-11 regen train green incl. the
  ebnf fixed point; expected-delta review **`overall=0` over all 11**
  (`artifact_delta_review.txt`) — every hunk is the `-0202` surfaces only;
  artifact growth ≤ **+0.50%** (bar 5%).
- Battery (all green, `battery_summary.txt`): dual-feature lib **1001/0
  (29 ignored)** incl. the ALL-11 interpreter↔generated oracle + the new
  layout-pin test; cert ×3 seeds byte-exact `268/9/259/0 fully_certified`
  (spf=0); `ast_shape_contract_gate`, `duality_hunt_gate`,
  `regex_pcre2_compile_oracle_gate`, clippy source-strict.
- Every heavy step memory-guarded (16384 MB, floor 10%), serialized,
  `caffeinate -i`; alternated base/candidate rounds.

## Next

Per the one-fix-per-fresh-session directive this session stops after the
clean commit. NEXT (brand-new session) = `PGEN-RGX-0078-0203` (carrier
core — G1-C + memo segment-copy + packed events/unified tape — ONLY after
its joint replacement/overlap contract is composed), per the `-0197`
ordered leaves.
