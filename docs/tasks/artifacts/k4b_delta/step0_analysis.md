# K4b STEP-0 — the delta slow-leg census + mechanism pin (`PGEN-RGX-0078-0157`)

Session #161, 2026-07-19. Toolbox-first, measurement + docs only — the release floor
is byte-untouched (artifact custody `95fdb3c7`/`bac7ae6b` unchanged; nothing rebuilt
but the debug `parseability_probe`, guard-built exit 0 peak 6,744 MB, mtime-coherent
against the 14:53 artifact write — the 12:54 binary was an ambiguous M3-window
vintage and was re-derived rather than trusted).

Method: single-cell corpus-mode loops on the on-disk floor probe
`rust/target/release/regex_perf_probe` (**`e34f3229`**, fat-LTO+mimalloc, the M1
artifacts — the `-0156` floorval −0.46% binary), `/usr/bin/sample` @1 ms × 10 s per
cell, tree-walk attribution (`../k3_constants/step2_treewalk.py`); store-counter +
rule-outcome dumps (`--dump-rule-outcome-counts-json`) and scoped HIGH traces on the
fresh debug probe. Raw evidence in this directory: `sample_{725,2880,6526}_k4b.txt`,
`treewalk_*.txt`, `outcome_*.json`, `trace_{6526,2880}_delta_events.txt`,
`custody_*.jsonl`.

Floor custody in-run (vs the banked `m3_corpus_base.jsonl` numbers for this exact
binary): `line_725` 732,959 ns (+1.9%), `line_2880` 93,541 (−0.7%), `line_6526`
112,334 (−1.6%), `nest_80` 158,250 (−2.2%) — floor intact.

## 0. Scope correction recorded first (honest)

The `-0156` NEXT pointer framed K4b as "the `line_2880` cluster ~22% + the
memo/content-level clone population". Two of its premises moved under measurement:

- **The memo/content-level clone population on the MAX cell prices at ~1.9% cum**
  (`clone` 160 of 8,342 samples; 1,889 real Vec clones/parse ≈ 7 ns each) — the
  M3 road-fact repeating (small immediately-freed allocs ≈ free under mimalloc).
  It is NOT a standalone lever population.
- **`line_6526` (114 µs) outranks `line_2880` (94 µs) inside the class and is
  MORE delta-heavy** — the K4b exemplar cell is 6526, not 2880.
- The rest of the 73–91 µs cluster is NOT this mechanism: `line_965` emits 0 facts
  (1,573 rollbacks all unchanged), `line_1224`/`line_640` roll back 0 — their cost
  is spine/memo (K5-class). The delta class = {6526, 2880, 6538}.

## 1. The populations (fresh profiles, this floor)

**`line_6526`** (261 B, 8,196 in-graph samples): `extract_delta_since` 803 (9.8%) +
`apply_delta` 753 (9.2%) + `rollback_to_labeled_slow` 679 (8.3%) = **Δ-cluster
27.3%**; `apply_semantic_runtime_effect_directive` 1,050 (12.8%);
`with_semantic_runtime_rule_transaction` 1,955 (23.9%, 1,930 under
`parse_capture_open`); `FactIndex` 1,032 (661 under apply_delta, 353 under the
effect directive); `hash` 942; memmove 1,078. Counters: **384 facts emitted, 256
rolled back in exactly 2 slow rollbacks** (~128 each), scopes 1/1.

**`line_2880`** (450 B, 8,171 samples): extract 682 (8.3%, memoized 257 +
parse_pattern 161) + apply_delta 589 (7.2%) + slow-rollback 428 (5.2%) =
**Δ-cluster 20.8%**; FactIndex 478; hash 559. Counters: 97 emitted / 83 rolled
back, 43 slow of 2,296 rollbacks, 117 memo hits, one long-lived scope.

**`line_725`** (the corpus MAX, 8,342 samples): extract 34 / apply 0 /
slow-rollback 0 — **the Δ-cluster is ~0 on the MAX cell** (271 facts emitted, ZERO
rolled back, all 20,490 rollbacks unchanged — the `-0148` counter pin still exact).
Its remaining populations: memo-probe/insert `HashMap` 666 (8.0%), build pass
(`to_shaped_value` 555 + `alloc_extend` 554, V1-closed legitimate work), memmove
486 (5.8%), allocator ~655 (7.9%), effect-directive insert 124.

## 2. ⭐ THE MECHANISM (trace-pinned, WHY+WHERE)

`trace_6526_delta_events.txt` — the whole 27% cluster on 6526 is TWO events:

```
rollback_to(fact_len=0, caller=lookaround (C3-B branch 4/7 cleanup)) — discarding 128 fact(s)
apply_delta — facts now at 128     (chain … atom > lookaround)
rollback_to(fact_len=0, caller=atom (C3-B branch 16/24 cleanup)) — discarding 128 fact(s)
apply_delta — facts now at 128     (chain … piece > atom)
```

The pattern's 128 `regex_capture_group` facts sit inside a lookaround body. The
winner (`lookaround` branch 4, then `atom` branch 16) carries all 128 facts live
(K1 winner-in-place); because `|` is a longest-match TOURNAMENT, the later
byte-viable siblings still attempt bodies after the winner — the first such attempt
triggers the K1 deferred cleanup: **extract 128 record clones → slow-rollback 128
`FactIndex::remove` → (siblings fail) → winner `apply_delta` re-inserts 128** —
×2 levels ≈ 768 heavy fact-ops/parse. Each op pays the `-0135` constant
(`to_ascii_lowercase` String alloc + `FactNameKey` String clone + 2 SipHashes —
`semantic_runtime.rs:2154/:2166`; record clone = 2+ String clones + attributes Vec).

**WHY the siblings attempt at all: the dispatch horizon is too shallow.** The
`(?`-family branches (`(?=` `(?!` `(?<=` `(?<!` `(?:` `(?<name>` `(?P<` …) are
mutually unprunable at the P2 byte-1 switch AND at the D1 byte-2 refinement — they
differ at bytes 3–4. On `line_2880` the same shape fires 43 slow tournament
cleanups at `named_group`/`group` plus a distinct leg: **memo-HIT delta re-apply**
(dozens of small `apply_delta`s replaying 4–13 facts inside `lookahead_pos`,
`trace_2880_delta_events.txt`).

**The corpus-wide half (`outcome_725.json`): the doomed-attempt population.** Of
the MAX cell's 15,628 failed rule entries/parse, ~6,000 are the `\`-escape family
attempting at every escape position and dying at byte 2–3: `nonzero_digit` 1,081
doomed, `numeric_backreference`/`backreference_digits`(+`_single`) ~540 each,
`subroutine_target` 538, `anchor` 541, `subroutine_call`/`scoped_inline_modifiers`/
`returned_capture_subroutine` 269 each. Root cause in the D1 carrier
(`first_set.rs:724-749`, recorded there as a design compromise): the second-byte
summary is **GLOBAL over all admitting first bytes**, and `len1_possible` is global
too — `anchor`'s 1-byte forms (`^`/`$`) set the byte-2 wildcard, so anchor must be
attempted at every `\x`; multi-first-byte families (`\`-escapes) union their
second-byte sets into uselessness.

## 3. Candidate levers (sized; population×unit-cost bands stated with the M1/M3
lesson — arithmetic fails in both directions, falsification bounds set at design)

- **C1 — per-first-byte prefix-trie dispatch (FIRSTₖ, k ≤ 4) — THE RECOMMENDED
  K4b lever, structural, P2/D1-lineage, parser-agnostic, emitter-level (grammar
  untouched, EBNF stays SoT).** Refine the D1 carrier from one global
  `SecondByteSummary` to per-first-byte prefix facts (and one more level for the
  `(?<`-family), with the SAME soundness contract per level (`lenN_possible`,
  `nullable`, `unresolved`, regex-token layout-trust). Kills BOTH halves:
  (a) 6526's 2×128 fact-op cycles + 2880's tournament cleanups (post-winner
  `(?`-siblings prune at bytes 3–4 ⇒ `live` stays true ⇒ K1 in-place commit,
  ZERO extract/rollback/re-apply); (b) the corpus-wide doomed-attempt spine
  population (`\`-family + anchor at every escape position, ~6k/parse on the MAX
  cell). Pure pruning of CANNOT-match attempts ⇒ language/AST/verdicts unchanged
  by construction; the tournament semantics (longest_match) are untouched —
  pruned branches are exactly those that would fail on bytes the guard already
  read. Predicted (bands, falsification at design): `line_725` −4…−12%
  (falsification < −2%), `line_6526` −15…−30%, `line_2880` −8…−20%, bench
  −1…−5% (escapes/groups are bench-present; ±2% guard still the gate).
- **C2 — fact-op constants** (FxHash both FactIndex levels + pre-normalized/
  interned kind + cheap name key): trims the ~40–70 ns/op constant on whatever
  delta traffic C1 leaves (memo-hit re-applies, K1b extracts, emit-path inserts —
  6526's effect-directive 12.8% keeps its FactIndex 353 + hash 195 share).
  Lib-only, K3b-class battery. Sequenced AFTER C1 re-profile.
- **C3 — memo-hit delta cheapening**: `MemoEntry.semantic_delta` hit-side
  `.clone()` runs BEFORE `is_empty()` (`regex_parser.rs:407043` — clone-on-
  empty-hit waste, trivial emitter fix folds into any artifact-touching slice);
  Rc/borrow-apply for the non-empty case. 2880-leg (117 hits), small on 6526.
- **C4 — move-not-clone tournament extract** (fuse extract+rollback →
  `split_off`): killed by C1 on the tournament leg (nothing left to move);
  memo-side extract must clone (store keeps its facts) — folds into K1b.
- **Runtime-laziness alternatives adjudicated OUT (recorded so they stay dead):**
  (i) "defer isolation until the challenger's first store op" collapses because
  challenger bodies take checkpoints immediately (the trigger would fire at
  once, and exempting checkpoints leaves stale fact_len/epoch snapshots after a
  late rewind); (ii) static whole-branch store-blindness is too coarse (every
  `(`-family branch reaches an emit site statically even though it dies at
  byte 3 dynamically).

## 4. The re-steer

1. **K4b STEP-1 = the C1 design slice** (emitter recon: `generate_or_logic`
   prune-guard emission + `first_set.rs` carrier generalization + the
   admission/eligibility census across all 11 grammars; acceptance instrument
   with the bands above; artifact-changing ⇒ the FULL regen battery).
2. C2/C3 re-priced on the post-C1 re-profile (one lever per slice).
3. K5 (`check_cycle_id` seen-set) + K1b (memo success-insert extraction — 6526's
   `parse_pattern` 472 extract samples are K1b territory, untouched by C1)
   unchanged in the queue behind the MAX levers.
