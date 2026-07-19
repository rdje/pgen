# K4b C1 STEP-A — FIRSTₖ census, banked BEFORE regen (`PGEN-RGX-0078-0160`)

Session #162, 2026-07-19. The `-0158` design's census obligation (§6): the
emission and this census are driven by the SAME shared license
(`first_set::branch_prefix_trie_guard`) — zero drift by construction. Run on a
guard-built dual-feature `ast_pipeline` (debug, `ebnf_dual_run,generated_parsers`,
artifacts at the canonical `95fdb3c7` M1 vintage) over ALL 12 registry grammars
(the ALL-11 battery set + `systemverilog`). Raw outputs: `census_<grammar>.{txt,json}`
in this directory (`firstk_branches` per top-level site in the JSON).

## 1. Aggregates

| grammar | top-level branches | guarded | deeper-than-level-1 | with emulation | truncated | d1-fallback |
|---|---|---|---|---|---|---|
| regex | 852 | **852** | **202** (d2=65 d3=44 d4=93) | 29 | 241 | 1 |
| systemverilog | 1810 | 0 (R2) | 0 | 0 | 0 | 0 |
| every other grammar (10) | 15–282 | 0 (R2) | 0 | 0 | 0 | 0 |

⭐ **Only regex has whitespace-sensitive terminals**, so — exactly like the
level-1/D1 guards today — C1 emission fires for regex alone. PREDICTION BANKED
for the regen step: **all 11 non-regex artifacts regenerate byte-identical**;
the C1 artifact delta is confined to `regex_parser.rs`. (The refusal histogram
for every non-regex grammar is uniformly `terminals skip leading layout (R2
raw-byte peek unsound)` — the same gate as `emit_first_set_guard`.)

## 2. Target-family verdicts vs the `-0158` design constants (the GO check)

| family (design §4 constant) | census verdict | match |
|---|---|---|
| `\`-escapes refuted at j=1 ⇒ w=1 | `backreference#b1(d2,w1,trunc)` `#b2(d2,w1)`; `atom#b4(d2,w1,trunc)`; deeper members `#b4/#b5(d4,w3)` (`\g<` → `signed_digits` at offset 3), `#b7(d3,w2)` | ✅ exact |
| `anchor` w=0 at every refutation depth | `anchor#b3..b9(d2)`, emulation set EMPTY (1-byte `^`/`$` branches level-1-degenerate accepting — the per-path `len1` generalization) | ✅ exact |
| `(?`-family: w=0 for j≤2, per-node at j=3 | `lookaround` b1/b2 (`(?=` `(?!`) d3; b3/b4 (`(?<=` `(?<!`) **d4** — the j=3 lookbehind/named-group fork discriminates; `group` b2–b4 d3/d4; w=0 on all discriminating literal prefixes | ✅ exact |
| the K1-prize sites (6526/2880 delta cycles) | `atom(24)`: 18 branches deepened (d2–d4); `lookaround(7)`: ALL 7 deepened (d3/d4) — post-winner `(?`-siblings refute within the walk | ✅ |
| the 725 doomed-attempt spine population (~6k/parse: nonzero_digit 1,081 via backreference*, anchor 541, subroutine* ~540) | the corresponding calling branches all deepened: `backreference` all 7 (d2–d4, w1..w3), `anchor` all multi-byte forms (d2), `atom#b12..b23` (the `(?`-family incl. `subroutine_call`) d3/d4 | ✅ coverage reaches the census populations |

No census surprise fired (the §6 STOP condition — e.g. the `\`-family failing
to reach depth 2 — did not occur). **STEP-A verdict: GO for emission.**

## 2b. The battery catch + license fix (re-censused above)

The FIRST full-battery run (999-test lib suite) caught a REAL C1 over-prune the
unit tests and census had not exercised: `version_conditional`
(`(?(VERSION>=10.0)cat|dog)`) failed to parse — the `digits "." digits` branch
guard demanded `.` immediately after ONE digit. Trace-pinned mechanism
(`parseability_probe --trace-rules`): budget truncation on the ACCEPTING
`digit+` boundary nodes kept `unresolved=false`, so the later `.`-graft cleared
`accepting` and left refutation-live nodes with INCOMPLETE children. Fix:
`degrade_unresolved` sets `unresolved` UNCONDITIONALLY (truncation is sticky
under composition); regression pin
`prefix_trie_truncated_repetition_stays_unrefutable`. Cost: exactly 2 branches
(the over-pruning ones) degrade to level-1 (204→202 deep); every target family
unchanged. Re-run lib battery: **999 passed / 0 failed** (ALL-11 differential
equivalence byte-identical).

## 3. Caps behavior (sound path-local truncation)

241 of 852 guarded regex branches carry a truncation somewhere (depth-4 horizon
on long literal openers, the >24-byte fanout on name/identifier heads — e.g.
`alpha_lookaround_name#b1(d4,trunc)`, the named-group capture-name head).
Truncation is path-local `unresolved` = the shallower guard; the known
consequence (recorded in `-0158`): on lookbehind-shaped inputs the NAMED-GROUP
sibling stays attemptable at j=3 (its name head exceeds the fanout cap) while
the lookbehind siblings on named-group inputs DO refute at j=3 — the asymmetry
is the cap design, not a defect.

## 4. Instrument

- Carrier + license: `rust/src/ast_pipeline/first_set.rs` (`PrefixTrieNode`,
  `branch_prefix_trie_guard`; caps depth≤4 / fanout≤24 / ≤16 nodes per
  composite accumulator; D0.1 cache-coherence discipline inherited verbatim;
  10 new unit tests incl. the min-0 greedy-parity pin that CAUGHT a real
  fabricated-repetition emulation bug pre-emission — `X?` must not claim rep-2+
  entries).
- Census lane: `fusibility_census.rs` `FirstkBranchCensus` + the
  `FIRSTK-CENSUS` report section; `ChoiceSiteCensus.firstk_branches` in the JSON.
- Emission (same slice): `first_set_prune_guard_for_branch` in
  `ast_based_generator.rs` (+ the cascade/scan mirrors) consumes the SAME
  guard; degenerate depth-1 tries emit today's exact level-1 expression and the
  uniform w=0 rectangle emits today's exact D1 conjunct, so untargeted branches
  stay byte-identical in the artifact.
