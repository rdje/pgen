# `PGEN-RGX-0078-0212` design pre-registration — the RESULT-CARRIER SLIMMING unit

Leaf `RGX-0078.5.j.4`, session #184, 2026-07-21. The `-0211`-banked ONE fix for
this fresh session, design-prereg'd BEFORE code per the `-0197` contract.
Base = preserved probe `fxstore_a4067793`; floor of record = corpus geomean
**1037.804231341058 ns** (noise 23.657 ns).

## The unit

Shrink the by-value committed-result carrier `ParseNode` **72 B → 48 B**
(−33.3%), so every boundary payment the `-0211` decomposition priced — callee
build, sret write, caller local copy, argument-slot/Vec-element/arena-slot
copy, and the big-frame staging these temporaries force — shrinks
proportionally. The unit **strictly removes memory traffic**: no new branches,
no ceremony, no capacity-carrying state (the GOOD risk profile recorded by
`-0211`).

## The three ingredients (each adjudicated on tool-backed facts)

### (a) `rule_name: &'static str` (16 B) → `&'static &'static str` (8 B thin ref)

- **ADJUDICATION vs the `-0211` sketch's `u16` id + static table:** BOTH land
  `ParseNode` at exactly 48 B — with `ParseContent` at 32 B (align 8) and the
  span at 8 B, a `u16` id pads 32+8+2 → 48, identical to the thin ref's
  32+8+8 = 48 (layout replica banked below). The thin ref is therefore chosen:
  same size, strictly less machinery — NO per-grammar id table, NO id→name
  resolution in shared serde/consumer code (a `u16`-carrying node cannot
  serialize its name without a table in scope — `parse_node_to_json` at
  `rust/src/parser_registry.rs:1559` serializes `ParseNode` directly in shared
  code), NO new global/thread-local state.
- **Behavior-identity for free:** serde's blanket `impl Serialize for &T`
  delegates, so the derived serializer emits the SAME string bytes; `&&str`
  `PartialEq`/`Debug` delegate to `str` — derived impls stay observably
  identical.
- **Emitted spelling:** `rule_name: &"pattern"` — rvalue static promotion
  gives `&'static &'static str` from any literal (proven in the banked
  replica). Codegen-time `format!` name templates
  (`unified_return_ast.rs:1721/1737`, `return_annotation_handler.rs:355`)
  expand to literals in the artifact — same spelling applies.
- **Interpreter:** a second-level `intern_ref(&str) -> &'static &'static str`
  (dedup + one-time leak) atop the existing `intern()`
  (`parse_harness_interpreter.rs:485`).

### (b) `Quantified(Vec<&ParseNode>, &'static str)` → `Quantified(Vec<&ParseNode>, &'static &'static str)`

- This is what shrinks `ParseContent` 40 → 32 B (the `Quantified` variant is
  the size-dominant one: 24 + 16 → 24 + 8).
- **A `u8`/enum kind is REFUTED by tool-backed fact:** the kind set is OPEN —
  the artifacts carry grammar-static bounded forms (`Quantified(results,
  "0,127")` in `generated/regex_parser.rs`, alongside `"*"`/`"+"`/`"?"`), so a
  closed enum cannot reproduce the wire strings. The thin ref covers every
  literal with byte-identical serialization (same serde delegation as (a)).

### (c) `span: Range<usize>` (16 B) → `Span { start: u32, end: u32 }` (8 B, `Copy`)

- **Wire identity:** serde serializes `Range` as a struct `{"start": s,
  "end": e}`; a `Span` with fields declared `start, end` derives to the SAME
  shape, and `u32` vs `usize` JSON integers are byte-identical. (The wire
  readers — `parser_registry.rs:2102`, `embedding_api.rs:1934/1995` — read
  `"span"` as an object with numeric fields; unchanged.)
- **Debug identity:** custom `Debug` printing `start..end` (Range's format), so
  trace/debug output cannot drift.
- **Trait surface:** derive `Clone, Copy, PartialEq, Eq, Hash, Serialize`
  (Range's used surface + `Copy`, which `Range` lacks — a small bonus: span
  copies stop being `clone()` calls). Accessors `fn start(&self) -> usize`,
  `fn end(&self) -> usize`, `fn range(&self) -> Range<usize>`, constructor
  `Span::new(start: usize, end: usize)` (debug-asserted lossless casts).
- **Honest acceptance bound (recorded, contract-noted):** inputs longer than
  `u32::MAX` bytes (4 GiB) are REFUSED at parse entry with an explicit error —
  ONE branch per parse, not per node. Such inputs were never practically
  parseable (a packrat memo over a 4 GiB input is memory-infeasible); the
  refusal makes the bound honest instead of silent.

## Layout proof (banked standalone replica — `layout_probe.rs` output)

```
PgenValue=24
ContentOld=40 NodeOld=72 Result<NodeOld,PE>=72   ← matches the -0211 in-disassembly 0x48 exactly (incl. the Result niche)
ContentNew=32 NodeNew=48 Result<NodeNew,PE>=48   ← −33.3% at every boundary, including the sret payload
promoted=letter deref_eq=true                     ← the &"literal" static-promotion spelling works
```

Compile-time pins land WITH the change (64-bit targets):
`const` asserts `size_of::<ParseNode>() == 48`, `size_of::<ParseContent>() == 32`,
`size_of::<Span>() == 8` — the layout claim becomes un-regressable.

## Priced mechanism + in-band prediction (falsifiable)

`-0211` priced the carrier inside spine_other alone at sp-staging 83.68 ns ≈
3.54× noise (memory-class 83% of 203.7 ns), with the same mechanism feeding
arena_alloc 72.17 / build_value 73.74 / other_text ≈25+ — a whole-corpus
carrier-attributed transport population conservatively ≥100–150 ns. A −33%
per-boundary byte cut, priced in the proven 30–70% delivery band
(the `-0205`/`-0207` precedent), predicts **−10…−35 ns ⇒ −1.0…−3.4% corpus
geomean**. Falsification: any non-decrease ⇒ ratchet REVERT (byte-exact
artifact restore + docs-only record).

## Change inventory

1. **Lib core** (`rust/src/ast_pipeline/mod.rs`): the `Span` type; the three
   field/variant type changes on `ParseNode`/`ParseContent`; `MemoEntry`
   shrinks for free; ~115 `.rule_name` + ~38 `.span` consumer sites across
   `rust/src` adapted mechanically (deref `*node.rule_name`; `span.start()`/
   `.end()`/`.range()`); the compile-time size pins.
2. **Interpreter** (`parse_harness_interpreter.rs`): `intern_ref` + literal
   spellings `&"…"` at its ~46 construction sites.
3. **Live emitters** (`ast_based_generator.rs`, `cascade.rs`,
   `cascade/value.rs`, `scan.rs`, plus the string-template emitters
   `unified_return_ast.rs`, `return_annotation_handler.rs`): construction
   spellings `rule_name: &"…"` / `Quantified(…, &"…")` / `Span::new(a, b)`;
   read-site derefs; the parse-entry u32 guard emitted in
   `parse_full`/`parse`/`parse_from`/`parse_full_from`.
   The legacy self-contained `ast_code_generator.rs` (own local types) is OUT
   of scope.
4. **NOT touched:** `PgenValue`, the semantic store, memo protocol, deriv
   tape, tournament/branch logic, any parse decision.

## Migration/bootstrap (the `-0211`-banked tax, executed by the proven recipe)

A field-type change on the shared carrier is inherently non-additive (struct
literals in every artifact name the old types), so the artifacts and the
embedded ebnf parser regenerate via the PROVEN cold-bootstrap recipe
(`MEMORY.md` ON-DISK block): `generated/` emptied → licensed ebnf seed
(`PGEN_ALLOW_BOOTSTRAP_ANNOTATION_FALLBACK=1`, Step B on the frontend bin) →
annotation pair (bootstrap = their canonical mode) → `generated_parsers`-ONLY
wave → canonical ebnf re-derivation → `focus_*` targets → dual-feature LAST.
Disciplines held: NEVER a bootstrap-binary regen mid-change (`-0090`); NEVER
hand-edit `generated/*`; regen canonical ONLY via `make focus_*`; every heavy
job under `scripts/run_with_memory_guard.sh --budget-mb 16384`, ONE at a time.

## Oracles + adjudication (the `-0197` ratchet, verbatim)

- **Wire-format byte-identity (explicit oracle):** candidate typed-AST JSON
  byte-identical vs base on representative dumps; `ast_shape_contract` GREEN;
  cert ×3 seeds 0/7/42 byte-exact `fully_certified`; PCRE2 compile oracle;
  duality; equivalence gates (all-11 + combinator 27/27 + semantic 36/36);
  clippy source-strict.
- **Perf:** base probe `fxstore_a4067793` SHA-asserted; candidate = fat-LTO
  rebuild (distinct sha256); alternated corpus measurement; ACCEPT iff strict
  unrounded same-session corpus-geomean decrease AND verdict flips 0/2,189 AND
  MAX ≤ the settled 425,000 ns. Else REVERT byte-exact + docs-only record.
- **Post-decision lockstep:** the regex integration contract documents the
  Rust-level `ParseNode { content, rule_name, span }` surface
  (`PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md:1867`) — field-type change
  recorded there + books (public-api / ast-envelope chapters) same-commit;
  the JSON wire (the primary integration surface) is byte-identical by oracle.
  MEMORY/frontier/CHANGES/DEVELOPMENT_NOTES; clean tree; STOP (one fix per
  fresh session).

## Risk register (each with its named check)

| # | risk | check |
|---|---|---|
| R1 | 4 GiB input refusal = acceptance-bound change | explicit guard + error text; contract honest-bound note |
| R2 | Debug-format drift (Span vs Range) | custom `Debug` `start..end`; trace spot-diff |
| R3 | span used as hash key / trait gap | derive Hash+Eq (Range's surface); compiler enforces |
| R4 | pointer-identity name comparison hiding somewhere | grep `ptr::eq` on rule_name (none found pre-change) |
| R5 | `&&str` comparison sites silently changed semantics | impossible silently — `==` against `&str` fails to COMPILE; every site gets an explicit `*` deref |
| R6 | a non-literal `'static` name origin in an emitted helper | thread `&'static &'static str` end-to-end; origins = promoted literals or the interner |
| R7 | downstream Rust-API surface change | contract + book lockstep; JSON wire byte-identity oracle |
| R8 | serde field-order drift on Span | declaration order `start, end` mirrors serde's Range impl; the byte-identity oracle re-proves |

## DATED ADDENDUM (2026-07-21, pre-registered BEFORE the rerun numbers) — corpus-sweep custody breach + the hardened rerun protocol

The first pre-registered A/B pair is **INVALIDATED for adjudication, not
adjudicated** (the analyzer's own custody principle, extended to the corpus
side): the SAME base binary (`a4067793`, SHA-asserted) read corpus geomean
**1184.08 ns = +15.62% vs its own `-0210`-session sweep** (same cells),
roughly uniform per band (+17.2/+16.0/+10.6/+3.7%) — ~6× outside the recorded
cross-session drift family (+0.64%, +1.31%, ≈2.3% span). Root cause
tool-pinned live: a FOREIGN compute job (`repeated_action_result_contract`,
a test binary under `/private/tmp/linkedspec-semantic-rust-audit-target/`,
another harness's workload on this host) held 101% of a core during the
sweeps; bench floorval EARLY in the run read −0.77% (clean), the corpus
sweeps ran LAST. The contaminated pair is preserved as
`corpus_base.jsonl`/`corpus_candidate.jsonl` (its paired verdict — REVERT at
+7.71% — is recorded but NOT adjudicated; its verdict-flip result 0/2,189 and
MAX 395,000 ≤ 425,000 remain valid custody facts).

**Hardened rerun protocol (binding for the rerun):**
1. Four interleaved sweeps: base₁ → cand₁ → base₂ → cand₂ (same guard, same
   corpus, `ps` foreign-load snapshots banked before/after each sweep).
2. Corpus custody gate (must PASS before any adjudication): each base sweep
   within ±3% of the floor-of-record 1037.804231341058 ns AND base₁ vs base₂
   within ±2.5%; a foreign ≥50%-CPU process in a sweep's snapshots marks that
   sweep contaminated → that sweep is re-run. Custody failure ⇒ the rerun is
   again recorded as invalid (no adjudication) and the session stops with the
   unit HELD, tree reverted to clean.
3. Adjudication on the custody-clean rerun: per-cell min across each side's
   two sweeps; the `-0197` ratchet unchanged (strict unrounded decrease vs
   the pooled base, flips 0 across all four sweeps, MAX ≤ 425,000 ns).
