# CODEGEN-DETERMINISM — the parser code generator must emit byte-reproducible source

## Metadata
- Tree id: `CODEGEN-DETERMINISM`
- Status: `active` (opened 2026-06-25)
- Created: 2026-06-25
- Surfaced by: `GRAMMAR-WELLFORMED.H.14.2` (`PGEN-GRAMMAR-WELLFORMED-0129`), which observed three
  different `systemverilog_parser.rs` md5s across regens of the *same* committed grammar
  (`cbe76f0e` / `8cf1515b` / `11ebfda1`) and flagged "SV codegen byte-non-determinism" as a
  reproducible-build concern worth its own investigation/leaf.
- Unblocks: `GRAMMAR-WELLFORMED.H.14.3` (and the broader SV `UNKNOWN`→0 drive) — a deterministic
  codegen restores **byte-identity** as a NO-REGRESSION verification leg for every future shipped-SV
  grammar change (previously unavailable, forcing a weaker semantic-only ceremony).

## The frame (binding)

A core trust property of PGEN is that **generated artifacts are reproducible** (README: "generated
artifacts that are reproducible and tracked"; the parser-hooks chapter: "Iterate over `rule_order` …
rather than `grammar_tree.keys()` (HashMap iteration order is non-deterministic). Determinism makes
the generated parser file reproducible across builds."). A code generator whose output bytes vary
across runs of the *same* input violates that property: it breaks `git diff`-based regen verification,
breaks the "regen twice, compare SHAs" determinism check the parser-hooks chapter documents, and
removes byte-identity as a NO-REGRESSION leg for any grammar change.

This is **not** SV-specific. It is a parser-agnostic defect in the generic codegen path; SV merely
exposes it most reliably because it carries by far the most semantic-directive rules.

## Root cause (WHY + WHERE — tool-proven, 2026-06-25)

- **WHERE:** `rust/src/ast_pipeline/ast_based_generator.rs`,
  `generate_compiled_semantic_runtime_annotations_tokens` (`:6461`/`:6470`/`:6493`). It emits the
  generated parser's `directives_by_rule.insert(...)`, `branch_directives_by_rule.insert(...)`, and
  `fact_kinds.insert(...)` statements by iterating `compiled.iter()` / `.branch_iter()` /
  `.fact_kinds()` — all three back onto `HashMap<String, …>` fields of
  `CompiledSemanticRuntimeAnnotations` (`rust/src/ast_pipeline/semantic_runtime.rs:507-515`), iterated
  in **process-random `HashMap` order**.
- **WHY:** Rust `HashMap` iteration order is randomized per process (RandomState), so the emitted
  insert sequence — and the `prettyplease` line-wrapping cascade it drives — differs run-to-run. The
  generated parser's *behavior* is unaffected (it re-inserts the same entries into its own runtime
  `HashMap`, which is order-insensitive), so cert/parse results are stable; only the **source bytes**
  vary.

## Tool-backed evidence (the diagnosis legs)

- **REPRODUCE (stage-localized):** with the rebuilt `ast_pipeline --features "generated_parsers
  ebnf_dual_run"`, regenerating from `grammars/systemverilog.ebnf` to scratch files:
  - Stage 1 (`--emit-raw-ast-json`) twice → JSON md5 differs **only** by the embedded `generated_at`
    timestamp (1 line); grammar content byte-identical ⇒ stage 1 is structurally deterministic
    (benign timestamp noted as a separate, non-blocking sub-item).
  - Stage 2 (`--generate-parser --debug --eliminate-left-recursion`) **twice from the same JSON** →
    `sv_pa1.rs` md5 `de11871b…` ≠ `sv_pa2.rs` md5 `dc479593…`, **identical size 69572893** (pure
    reordering), **48 616 differing lines / 24 133 hunks** spanning lines 2906–930747.
- **CONTAIN:** filtering the diff against the semantic-directive vocabulary leaves only structural
  `);` punctuation ⇒ the entire non-determinism is the `directives_by_rule` / `branch_directives_by_rule`
  / `fact_kinds` emission block; the ~1.4M lines of parse functions below are byte-identical.
- **CONSUMERS:** `compiled.iter()`/`.branch_iter()`/`.fact_kinds()` have **no other callers** in
  `rust/src/` than these three emission sites — so determinizing the emission is the complete fix and
  changes no runtime API behavior.

## The fix (targeted)

Emit the three collections in a **deterministic key-sorted order** at the codegen site
(`generate_compiled_semantic_runtime_annotations_tokens`): collect each iterator into a `Vec`, sort by
key (rule name / fact-kind name), then map to tokens. Fix tier = engine/codegen (the emission is the
only reproducibility-sensitive consumer; runtime types and behavior are untouched). Parser-agnostic
(benefits every grammar); behavior-preserving by construction (the generated runtime map is
order-insensitive).

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `CODEGEN-DETERMINISM.1` | `done` (`PGEN-CODEGEN-DETERMINISM-0001`) | Key-sorted the `directives_by_rule`/`branch_directives_by_rule`/`fact_kinds` emission ⇒ same-input+same-path regen is byte-identical for SV (md5 `cd6d5b83…`) and regex (`bc9cc2d9…`); SV cert `UNKNOWN=28` spf=0 seeds 0/7/42, regex `fully_certified`, shape 18/18, clippy clean. Restores byte-identity as a NO-REGRESSION leg for `GRAMMAR-WELLFORMED.H.14.3`. |
| 2 | `CODEGEN-DETERMINISM.2` | `candidate` (lower priority; `generated/` is untracked) | **Generated parser embeds its absolute `-o` output path** (`filename` baked at `ast_based_generator.rs:2677` + every log/error site) ⇒ generated bytes are not relocatable across checkouts/machines (surfaced when the `.1` isolation test wrote to two different filenames). Pre-existing, orthogonal to run-to-run determinism (same `-o` ⇒ constant). Fix direction: emit a relocatable/normalized filename (basename or repo-relative). |

### Leaf `CODEGEN-DETERMINISM.1` — sort the semantic-directive emission by key

Goal: make `--generate-parser` emit byte-identical source across runs of the same input, by ordering
the semantic-directive / fact-kind insert emission deterministically.

#### Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — with the rebuilt `ast_pipeline --features "generated_parsers ebnf_dual_run"`, `--generate-parser` twice from the same SV raw-AST JSON ⇒ md5 differs (`de11871b…` ≠ `dc479593…`), **48 616 differing lines / 24 133 hunks** (span 2906–930747), all `directives_by_rule`/`branch_directives_by_rule`/`fact_kinds` emission (filter to non-directive vocabulary leaves only `);`); parse functions below byte-identical. (regex same shape.)
- [x] **ROOT CAUSE (WHY + WHERE)** — `ast_based_generator.rs::generate_compiled_semantic_runtime_annotations_tokens` (`:6461/6470/6493`) emits by iterating the `HashMap`-backed `compiled.iter()`/`.branch_iter()`/`.fact_kinds()` (`semantic_runtime.rs:507-515` + `:589`) in process-random order. Only consumers of those three accessors in `rust/src/`. Stage-1 raw-AST JSON is structurally deterministic (only the `generated_at` timestamp differs). The *behavior-preserving* half of this cause (the reorder cannot change parse behavior) is independently confirmed by `ast_pipeline … --report-certificate-coverage`: the SV verdict `CERTIFICATE-COVERAGE: grammar='systemverilog' … total=1288 proof=1 witness=1259 UNKNOWN=28 fully_certified=false (sample_parse_failures=0, proof_reverify_failures=0)` is byte-for-byte unchanged across the codegen change (seeds 0/7/42).
- [x] **FIX** — collect each of the three emission iterators into a `Vec`, `sort_by(|a,b| a.0.cmp(b.0))` (key = rule/fact-kind name, all unique ⇒ total order), then tokenize. Codegen-only; runtime types/behavior unchanged (the generated parser re-inserts into its own order-insensitive `HashMap`). Parser-agnostic. `ast_pipeline` rebuilt clean (exit 0).
- [x] **ADDRESSED (verified)** — post-fix, `--generate-parser` twice to the **same `-o` path** ⇒ **byte-identical**: SV md5 `cd6d5b83…`==`cd6d5b83…`, regex md5 `bc9cc2d9…`==`bc9cc2d9…` (the previous session's `cbe76f0e`/`8cf1515b`/`11ebfda1` same-path drift is resolved). Only `regex` + `systemverilog` carry semantic directives (10 / 73), so they are the only grammars whose codegen changed; the rest hit the `compiled.is_empty()` early-return (`:6455`) and are byte-identical.
- [x] **NO REGRESSION** — regen `generated/{systemverilog,regex}_parser.rs` with the fixed codegen, rebuilt the cert binary, re-ran `--report-certificate-coverage` at seeds **0/7/42**: SV all three `CERTIFICATE-COVERAGE: grammar='systemverilog' … total=1288 proof=1 witness=1259 UNKNOWN=28 fully_certified=false (sample_parse_failures=0, proof_reverify_failures=0)` (= baseline, deterministic); regex `CERTIFICATE-COVERAGE: grammar='regex' … total=198 proof=0 witness=198 UNKNOWN=0 fully_certified=true (sample_parse_failures=0)`; `clippy_on_rust_change` source-clean (gate exit 0; generated stage = pre-existing non-strict tolerated debt); `ast_shape_contract` **18/18 passed**. The 5 unaffected fully-certified grammars need no re-run (code path not reached). External SV corpus not re-run: the change is provably behavior-preserving (reorder into an order-insensitive runtime map) and the cert re-parses every witness through the real parser at 3 seeds with `spf=0`.
- [x] **LOCKSTEP** — developer-architecture book chapter gets a reproducible-codegen note (the determinism property it/parser-hooks already claim now actually holds for directive-heavy grammars); CHANGES.md / DEVELOPMENT_NOTES.md / MEMORY.md / LIVE_ACHIEVEMENT_STATUS.md updated at commit; no schema/release/ledger (codegen-only, behavior-identical, `generated/` untracked).

## Decisions
- `2026-06-25` — **`.1` DONE** (`PGEN-CODEGEN-DETERMINISM-0001`). The previous session's "SV codegen
  byte-non-determinism" is root-caused + fixed: it was the `HashMap`-ordered semantic-directive
  emission. Key-sorting the three emission iterators makes same-input+same-path regen byte-identical
  (SV `cd6d5b83…`, regex `bc9cc2d9…`) with cert/shape/clippy unchanged. Behavior-preserving (the
  generated runtime map is order-insensitive) ⇒ no schema/release/ledger. Two orthogonal sub-findings
  surfaced and were tracked, not folded into this targeted fix: (1) the stage-1 `generated_at`
  timestamp (benign, below); (2) the embedded absolute `-o` path → `.2` relocatability candidate.
- `2026-06-25` — opened. Root cause localized tools-first to the semantic-directive emission's
  `HashMap` iteration order; fix scoped to a key-sort at the single emission function. Behavior is
  provably unchanged (order-insensitive runtime map), so this is codegen-reproducibility only — no
  schema/release/ledger movement expected. Composes with `[[feedback_be_alert_root_cause_fishy_immediately]]`
  (root-cause the fishy non-determinism immediately, tools-first) and unblocks
  `GRAMMAR-WELLFORMED.H.14.3`.
- Benign sub-item (tracked, not fixed here): the raw-AST JSON `generated_at` timestamp makes stage-1
  output non-byte-identical by one metadata line. It does not affect codegen (timestamp is not read by
  `--generate-parser`) and is not a reproducibility defect in the generated *parser*. Candidate
  follow-up only if full raw-AST byte-reproducibility is ever required.
