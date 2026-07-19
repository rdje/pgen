# K4b STEP-1 — the C1 design: per-first-byte bounded prefix-trie dispatch (FIRSTₖ, k ≤ 4) (`PGEN-RGX-0078-0158`)

Session #161, 2026-07-19. Docs-only pre-emission design, recorded BEFORE any code
change (the house discipline). Root: `step0_analysis.md` (the `-0157` census).
Lineage: P2 FIRST-byte prune guards (`.5.c.2`) → D1 SECOND-byte refinement
(`-0118`) → **C1 = the bounded per-path generalization both were approximating**.
The K1c order-of-operations lesson applies verbatim: kill the POPULATION (doomed
attempts) before shrinking the constant (fact-op costs).

## 1. What exists today (recon, at HEAD)

- Level-1 guard (`ast_based_generator.rs::first_set_prune_guard_for_branch:5009`):
  `matches!(input[parse_start], B…)` — sound, furthest-neutral because top-level
  `Or` ⇒ `parse_start` was already written by the enclosing rule's entry preamble
  (**the ONLY `furthest_position` writers are rule-entry preambles** — emitter
  `:3903`, plus the Q-GUARD emulation `:5151`; terminals never write it).
- Level-2 refinement (D1, `first_set.rs::branch_prefix2_guard_bytes:1177`): ONE
  GLOBAL `SecondByteSummary` per branch. Its license matrix REFUSES exactly our
  populations, each for a different reason (`-0157` evidence):
  - `len1_possible` is GLOBAL ⇒ `anchor` (1-byte `^`/`$` forms) is byte-2
    unguardable even under first-byte `\` where a second byte is REQUIRED;
  - `second_bytes` is GLOBAL over all admitting first bytes ⇒ multi-first
    families union into uselessness;
  - `offset1_rule_entry` refuses any branch whose refuted attempt would have
    ENTERED a rule at offset ≥ 1 (furthest-position parity) ⇒ the whole
    `\ <rule>` escape family (`backreference = "\" nonzero_digit …`) is
    unguardable at byte 2 despite having singleton FIRST and exact byte-2 facts;
  - depth stops at 2 ⇒ the `(?`-family (`(?=` `(?!` `(?<=` `(?<!` `(?:`
    `(?<name>`) is undiscriminable, which is what feeds the K1 deferred-cleanup
    delta cycles on the scope-open cells (6526/2880).

## 2. The carrier: `PrefixTrieSummary` (generalizes `SecondByteSummary`)

Per branch, a bounded trie over input bytes at offsets 0..k−1 (k ≤ 4):

- **Node** = `{ children: BTreeMap<u8, Node>, accepting: bool,
  deepest_entry_offset: u8, unresolved: bool, regex_token_derived: bool }`.
  - `children` — the admitted next bytes ALONG THIS PATH (per-path, not global:
    this alone dissolves the first two D1 refusals — under `\` the anchor trie
    requires byte 2 ∈ {A,z,Z,b,B,G}; under `^`/`$` the node is `accepting`).
  - `accepting` — some complete match ends at this depth (the per-path
    generalization of `len1_possible`): refutation is NEVER emitted at or below
    an accepting node's subtree-free bytes… concretely, a walk that reaches an
    accepting node stops pruning (the branch must be attempted).
  - `deepest_entry_offset` — the EXACT deepest rule-entry offset over all
    grammar paths merged into this trie node (0 = only offset-0 entries, which
    are furthest-neutral because the enclosing rule already wrote
    `parse_start`). This replaces the D1 `offset1_rule_entry` REFUSAL with an
    EXACT EMULATION license (§4).
  - `unresolved` — analysis incomplete along this path ⇒ the walk stops pruning
    there (attempt the branch); same semantics as today's summary-level flag,
    but path-local.
- **Computation** (`first_set.rs`): the same recursion skeleton as
  `branch_second_byte_summary` extended to carry a trie instead of one set —
  sequences advance the frontier per consumed byte; Or merges tries (BTree
  union; `deepest_entry_offset` = max — §4 proves max is exact); rule refs
  descend transitively with the existing D0.1 cache-coherence discipline
  (persistent cache admits context-free values only, per-query transient memo,
  cycle taint ⇒ `unresolved`, `MAX_RULE_CHAIN_DEPTH` cap). Regex-token
  terminals extend `regex_hir_prefix`/`regex_hir_second` to a depth-4 HIR
  prefix walk (literal/class heads; anything else ⇒ `unresolved` at that
  depth — sound). Builtins mirror `native_builtin_second_byte_summary` (char
  builtins: depth-1 accepting; any-char: continuation-byte children).
- **Size caps (determinism + artifact-size control):** per-node fanout cap 24
  distinct bytes, whole-trie node cap 16, depth cap 4. Exceeding a cap
  TRUNCATES that subtree to `unresolved` (falls back to the shallower guard —
  never unsound, only less precise). Caps are design constants; the emission
  census (§6) validates they cover the target families.

## 3. Guard emission (`first_set_prune_guard_for_branch` generalized)

Emitted shape per branch (depth-2 example; deeper levels nest identically):

```rust
// walk the trie against input[parse_start..]; refute when the walk falls off
match parser.input.as_bytes().get(parse_start) {
    Some(b'\\') => match parser.input.as_bytes().get(parse_start + 1) {
        Some(b'A' | b'z' | b'Z' | b'b' | b'B' | b'G') => { /* attempt */ }
        _ => { /* refuted at depth 1: emulation if deepest_entry_offset ≥ 1 */ }
    },
    Some(b'^' | b'$') => { /* accepting at depth 1 ⇒ attempt */ }
    _ => { /* refuted at depth 0 — today's level-1 behavior */ }
}
```

- The level-1 single-set guard and the D1 flat two-set guard are the degenerate
  cases (a 1-node / linear-2-node trie) — **the existing guards are subsumed,
  not duplicated**; one emission path replaces both.
- End-of-input: `get()` returning `None` refutes only NON-accepting,
  non-`unresolved` nodes (a branch that requires more bytes than remain cannot
  match) — same soundness as the existing `parse_start + 1 < len` conjunct.
- Refutation arms carry the emulation tokens (§4) and nothing else; attempt
  arms fall through to the unchanged K1 tournament body.
- The P2 degenerate byte-switch (`degenerate_dispatch_byte_sets`) is untouched
  this slice (its disjointness gate could later consume trie facts — recorded,
  out of scope).

## 4. Furthest-position parity: EXACT emulation (the license that unlocks the `\`-family)

Contract: rejected-parse `furthest_position` is a parity surface (the D1 license
text). Writers are rule-entry preambles only (§1). For a refuted walk that
consumed bytes 0..j−1 and died at offset j:

- Every grammar path merged into the walked trie node is GENUINELY attempted by
  the real (unpruned) branch before it fails: tournament `Or` attempts all
  byte-viable alternatives; ordered `Or` attempts later alternatives because
  earlier ones fail (the whole branch is refuted); sequence tails beyond a
  failing element are exactly the paths the trie already excludes. Hence the
  real attempt's deepest furthest write = `parse_start + deepest_entry_offset`
  of the walked node — **the merged max is EXACT, not an over-approximation**.
- Emission: refutation at a node with `deepest_entry_offset = w ≥ 1` emits
  `if parser.furthest_position < parse_start + w { parser.furthest_position = parse_start + w; }`
  (the Q-GUARD `BareRef` emulation generalized from "at the attempt position"
  to "at the deepest entered offset"). `w = 0` ⇒ no tokens (provably neutral).
- Worked constants for the target families:
  - `\`-escapes (`backreference`/`subroutine_call`/`scoped_inline_modifiers`/…,
    shape `"\" <rule> …`): refuted at j=1 ⇒ w=1 (the offset-1 rule entry).
  - `anchor` (bare-ref branch; internal branches are pure literals): w=0 at
    every refutation depth — strictly neutral, no emulation.
  - `(?`-family (`group`/`named_group`/lookarounds: literals through offsets
    2–3, first internal rule entry at offset ≥ 3): refutations at j ≤ 2 ⇒ w=0;
    j=3 refutations (e.g. `(?<=` vs `(?<name>`) ⇒ w per the trie node (0 where
    the discriminating byte precedes any entry — the lookbehind/named-group
    fork does).

## 5. Soundness pillars (each an emission-slice audit obligation)

1. **Pure CANNOT-match pruning**: the trie over-approximates the prefixes of
   every possible match (per-path); a refuted branch would have failed —
   language, AST, winner, and every verdict unchanged by construction.
2. **Furthest parity**: §4 exact emulation; the equivalence/typed-diff oracles
   are the parity proof (byte-identical rejects across all 11 grammars).
3. **Tournament semantics untouched**: pruning only removes attempts whose
   failure the byte walk already proves; longest-match comparison over the
   surviving attempts is identical. The K1 deferred-cleanup consequence is the
   PRIZE: when no post-winner sibling survives the guard, `live` stays true and
   the winner commits in place (zero extract/rollback/re-apply).
4. **Inherited D1 licenses**: rolled-back-effect epoch/memo-taint divergence of
   a pruned-vs-failed attempt is the SAME class accepted since `.5.c.2`/D1
   (batteries byte-identical then and since) — inherited, recorded.
5. **Diagnostic-only counter/trace drift**: fewer entries/rollbacks/memo-fail
   inserts (the point of the lever); counters are not gated surfaces (K1
   precedent). Cert-coverage witnessed sets ride ACCEPTED parses (unchanged);
   AUDIT: confirm the coverage record counts committed rules only, then run
   cert ×3 byte-exactness as always.
6. **Recursion-guard/cycle counters**: pruned attempts skip `enter_id` — the
   guard is per-attempt state with no cross-attempt memory; no soundness
   coupling (audit in emission).

## 6. Census obligation (census↔emission single implementation — the standing lesson)

Emission STEP-A extends `fusibility_census.rs` with a FIRSTₖ lane driven by the
SAME `PrefixTrieSummary` + license predicate the emitter consumes (zero drift by
construction): per Or-branch across ALL 11 grammars — max usable depth, cap
truncations, emulation-constant distribution, and the projected doomed-attempt
coverage on the `-0157` counter populations (the ~6k/parse MAX-cell class, the
6526/2880 `(?`-siblings). Banked before the artifact regen; any census surprise
(e.g. the `\`-family NOT reaching depth 2) stops the slice.

## 7. Acceptance instrument (stated before building)

- **Primary (corpus-MAX)**: `line_725` predicted **−4…−12%** (the ~6k prunable
  doomed attempts of 15,628 failed entries; falsification **<−2%** ⇒ stop +
  re-census the attempt population before further K4b work).
- **The delta class**: `line_6526` **−15…−30%** (the two 128-fact cycles die +
  its own doomed attempts), `line_2880` **−8…−20%** (tournament cleanups die;
  the memo-hit re-apply leg deliberately survives = C3's population).
- **Bench**: −1…−5% expected; the gate is the ±2% NON-REGRESSION guard (a
  tail/structural lever is licensed bench-neutral — the K1/`-0136` bar
  structure, not the −2% build-pass improvement bar).
- **Identity**: corpus verdicts 2,189/2,189 + ladder 39/39; ANY flip =
  stop-and-revert. Every corpus byte band ≤ 0 expected.
- **Battery (artifact-changing ⇒ FULL regen)**: dual-feature rebuild FIRST; lib
  988/988 with ALL-11 differential equivalence BYTE-IDENTICAL (the furthest-
  parity + value-identity oracle); cert ×3 byte-exact (`268/9/259/0` spf 1/0/2);
  typed-diff 8/8 + the silent-restore hash tripwire (15 firings expected-class);
  PCRE2 oracle tuple EXACT `2189/1879/262/48`; shape/duality (re-pin vanished
  signatures same-commit per the gate's standing rule); clippy source-strict;
  canonical `-o` regen spelling from `rust/`; new canonical hashes recorded
  (`95fdb3c7`/`bac7ae6b` retire). Artifact BYTE-SIZE delta measured and
  recorded (guard growth is cap-bounded — a >5% artifact-size growth without a
  matching win is a stop-and-reassess signal).
- **A/B**: base = on-disk `e34f3229` floor-validated first (best-of-3 vs banked
  best-of-5, ±6% — the `-0156` re-specified gate); cand = fat-LTO+mimalloc;
  alternated 5×2000 + full corpus + ladder; guard `--budget-mb 16384`;
  `caffeinate -ims`-wrapped; ONE runner per stage.
- **Land/revert pre-adjudicated**: C1 ADDS mechanism (dispatch machinery) ⇒
  below-bar REVERTS per `.5.d.2`/`-0134` — no complexity-removal defense.

## 8. Deliberately OUT of scope (recorded)

- **C3's trivial empty-check fix** (`memoized_call` clones `semantic_delta`
  before `is_empty()`): same emitter file, but one lever per slice — C3 rides
  its OWN slice post-C1-re-profile (its population is then measured on the
  C1 floor).
- The P2 switch generalization to trie dispatch; FIRSTₖ for k > 4; the
  quantified-site Q-GUARD depth extension — all recorded as later candidates,
  priced by the post-C1 re-profile.
- C2 (fact-op constants) and K1b/K5 sequencing unchanged (`step0_analysis.md` §4).
