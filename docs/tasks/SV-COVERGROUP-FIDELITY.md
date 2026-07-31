# SV-COVERGROUP-FIDELITY: restore IEEE-1800 LRM bracket/brace fidelity in covergroup trans/bins syntax

## Metadata

- Tree ID: `SV-COVERGROUP-FIDELITY`
- Status: `done` (both leaves landed: `.1` `PGEN-SV-COVERGROUP-FIDELITY-0001` / `.2` `PGEN-SV-COVERGROUP-FIDELITY-0002`, 2026-06-24; SV cert `UNKNOWN 32 → 30`)
- Roadmap lane: parser sign-off — SV grammar/LRM parse-fidelity (a parser rejecting valid SV is a
  defect, [[feedback_uvm_is_valid_sv]] / [[project_ebnf_is_single_source_of_truth]]); also unblocks the
  SV cert tail (`STORE-AWARE-GEN.4b.14` 9C-iii — `repeat_range` / `with_covergroup_expression`).
- Family / slice-id prefix: `PGEN-SV-COVERGROUP-FIDELITY-<NNNN>`
- Created: `2026-06-24`
- Owner: repo-local workflow
- Origin: tool-proven in `STORE-AWARE-GEN.4b.14` (`PGEN-STORE-AWARE-GEN-0022`) — the `systemverilog.ebnf`
  covergroup trans/bins syntax DROPPED the LRM's disambiguating brackets/braces, which (a) REJECTS valid
  SV and (b) makes a greedy `expression` shadow the `repeat_range` / `with_covergroup_expression` wrapper
  rules (cert UNKNOWN tail). Disciplines: [[feedback_no_codebase_change_without_tool_backed_facts]]
  (real-failure proof gathered), [[feedback_correctness_before_speed]], [[feedback_uvm_is_valid_sv]],
  [[project_ebnf_is_single_source_of_truth]], [[feedback_regex_book_live]].

## The frame (why this tree exists)

`grammars/systemverilog.ebnf` diverges from IEEE 1800-2017/2023 Annex A.2.11 in the covergroup
`coverpoint`/`bins` syntax: the LRM's **trans-repeat brackets** `[* repeat_range]` / `[-> repeat_range]` /
`[= repeat_range]` and the **bins set braces** `= { covergroup_range_list }` were flattened to bare
`( star repeat_range )?` and bare `covergroup_range_list*`. Because `star := "*"`, `implies := "->"`,
`assign := "="` are the BARE expression operators and `with(...)` is itself an `expression` form
(`array_manipulation_call`), an adjacent greedy `expression` swallows the very tokens the dedicated
wrappers exist to capture — so:

1. **Real parse-fidelity defect** (tool-proven in `.4b.14`): valid SV `bins b = (1[*2]);` / `(1[->2])` /
   `(1[=2])` / `(1=>2[*3])` are REJECTED (`did not consume full input … furthest_position=43`), while the
   non-LRM bare `(1*2)`/`(1->2)`/`(1=2)` are accepted. Every commercial simulator parses the bracketed forms.
2. **Cert UNKNOWN tail** (`STORE-AWARE-GEN.4b.14` 9C-iii): `repeat_range` (`:4383`) and
   `with_covergroup_expression` (`:5466`) are `parsed=true witnessed_target=false` — the wrapper never
   commits because the greedy `expression` shadows it. Restoring the LRM brackets/braces removes the
   ambiguity, so the wrappers become reachable and witness as a consequence.

## Goal

Make `grammars/systemverilog.ebnf` LRM-faithful in covergroup trans/bins syntax so valid SV parses and the
wrapper rules are no longer shadowed — closing the real parse defect and the 9C-iii cert tail. Generated
SV parser regenerated; all SV gates green; SV cert `UNKNOWN` strictly decreases (witnesses
`repeat_range`/`with_covergroup_expression`) with ZERO newly-UNKNOWN; book/contract/shape-contract lockstep.

## Non-Goals

- Touching any non-covergroup SV construct, or any other grammar (the change is `systemverilog.ebnf`-local
  ⇒ the other 6 generated parsers are inert by construction).
- The SVA infix binary-operator parse bug (owned by `GRAMMAR-WELLFORMED.H.12.5.8`).
- Generator witness-machinery changes (`.4b.14` proved no current generator pass closes 9C-iii; the fix is
  the grammar correction, after which the EXISTING generator witnesses the wrappers).

## Acceptance Criteria

- Valid SV trans-repeat (`(1[*2])`/`(1[->2])`/`(1[=2])`/`(1=>2[*3])`) and (Part 2) braced bins-set
  (`bins b = {1,2} with (3)`) PARSE; the non-LRM bare forms still parse (no rejection regression).
- SV cert `UNKNOWN` strictly decreases (`repeat_range` and/or `with_covergroup_expression` witnessed),
  ZERO newly-UNKNOWN (`comm -13` empty), deterministic at seeds 0/7/42, `spf=0`.
- The 6 fully-certified grammars BYTE-IDENTICAL (inert — SV-only grammar change); `ast_shape_contract_gate`
  GREEN (no SV sample exercises trans/bins — verified `.4b.14`); `cargo test --lib` green; clippy source-clean.
- Lockstep: this tree, the SV parser book (covergroup trans/bins coverage), the SV integration contract +
  released-parser ledger if the accepted-language surface is contract-relevant, CHANGES / DEVELOPMENT_NOTES /
  MEMORY / LIVE_ACHIEVEMENT_STATUS (SV cert number).

## Task Tree

- ID: `SV-COVERGROUP-FIDELITY`
  Status: `active`
  Goal: LRM-faithful covergroup trans/bins syntax in `systemverilog.ebnf`
  Children: `.1` (trans-repeat brackets), `.2` (bins-set braces)

- ID: `SV-COVERGROUP-FIDELITY.1`
  Status: `done` (`PGEN-SV-COVERGROUP-FIDELITY-0001`, 2026-06-24 — SV release `1.0.144`, ledger `SV-0006`; cert `UNKNOWN 32 → 31`, `repeat_range` witnessed)
  Goal: restore the LRM trans-repeat brackets in `trans_range_list` (`trans_item [* repeat_range]` etc.)
  Acceptance: `(1[*2])`/`(1[->2])`/`(1[=2])`/`(1=>2[*3])` PARSE; `(1*2)` still parses (kind `simple`);
    SV cert witnesses `repeat_range` (`UNKNOWN 32 → 31`), ZERO newly-UNKNOWN, seeds 0/7/42, spf=0;
    6 grammars inert; `ast_shape_contract_gate` green; tests + clippy green; book/contract lockstep.
  Verification: `pending`
  Commit: `pending`

- ID: `SV-COVERGROUP-FIDELITY.2`
  Status: `done` (`PGEN-SV-COVERGROUP-FIDELITY-0002`, 2026-06-24 — SV release `1.0.145`, schema `4 → 5`, ledger `SV-0007`; cert `UNKNOWN 31 → 30`, `with_covergroup_expression` witnessed)
  Goal: restore the LRM bins-set braces in `bins_or_options :643` (`= { covergroup_range_list } ( with … )?`)
    so `with_covergroup_expression` is no longer shadowed by the greedy range-list.
  Acceptance: braced bins-set + with-clause PARSE and witness `with_covergroup_expression`; the
    accepted-language change (bare comma-list `bins b = 1,2,3` ⇒ requires `{1,2,3}` per LRM) checked against
    the LRM + the SV corpus and accepted with a schema bump + contract/ledger notice (the single bare value
    `bins b = v` still parses via `set`; `id with(e)` via `cover_point_with`); no gated-corpus rejection
    regression; full SV gate lockstep.
  Verification: braced `bins b = {3} with(e)` witnesses `with_covergroup_expression` (cert `UNKNOWN 31 → 30`,
    witness `1257 → 1258`, seeds 0/7/42, spf=0, zero newly-UNKNOWN `comm -13` empty, total=1289); external corpus
    14/14; `ast_shape_contract_gate` 18/0; `cargo test --lib` 768/0; clippy source-clean; 6 grammars inert.
  Commit: `PGEN-SV-COVERGROUP-FIDELITY-0002`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `SV-COVERGROUP-FIDELITY.1` | `done` | landed (`PGEN-SV-COVERGROUP-FIDELITY-0001`): SV `1.0.144`, cert `UNKNOWN 32 → 31`, `repeat_range` witnessed + `(1[*2])` real defect fixed. |
| — | `SV-COVERGROUP-FIDELITY.2` | `done` | landed (`PGEN-SV-COVERGROUP-FIDELITY-0002`): SV `1.0.145`, schema `4 → 5`, cert `UNKNOWN 31 → 30`, `with_covergroup_expression` witnessed via the LRM braced `= { … } with (…)` form; bare comma-list now LRM-correctly requires braces. |

**Frontier EMPTY — tree `done`.** Both leaves landed (`.1` trans-repeat brackets, `.2` bins-set braces); SV cert `UNKNOWN 32 → 30` across the tree. The covergroup trans/bins LRM delimiter-drop class (the bins siblings of `SV-0002`/`SV-0004`) is closed. Remaining SV cert residual (`UNKNOWN=30`) is tracked elsewhere: `STORE-AWARE-GEN.4b.15` (9A `wildcard_escape_nettype_identifier`/`kw_constant_d810ca96`), the DEFERRED `context_member_method_call` + 2 family-B `class_scoped_call`, the 6 SVA `kw_*` parse-bugs (`GRAMMAR-WELLFORMED.H.12.5.8`), and the 19 `no_path` LRM-legitimate rules.

## Decisions

- `2026-06-24`: Split into `.1` (trans brackets, low-risk) + `.2` (bins braces, heavier). `.1` first — the
  bare `( star repeat_range )?` / `( implies … )?` / `( assign … )?` branches are effectively DEAD (a bare
  `*`/`->`/`=` is always consumed by `trans_item`'s `expression`), so bracketing them LOSES no real
  acceptance and GAINS the LRM `[*n]` forms + un-shadows `repeat_range`.

## Open Questions

- `.2` only: does requiring LRM braces on the bins range-list reject any currently-accepted real design?
  **RESOLVED (2026-06-24, tools-first):** No gated-corpus regression — the realistic corpus (0 bins files), the
  SV external corpus (14/14, `parse_fail_total=0`), the shape-contract samples (18/0; none exercise bins), and
  `cargo test --lib` (768/0) carry NO bins usage. The only narrowing is the non-LRM bare comma-list
  `bins b = 1,2,3` (no braces), which IEEE-1800 §A.2.11 does not permit anyway (must be `{1,2,3}`); the single
  bare value `bins b = v` still parses via the LRM `set` branch (647) and `bins b = id with(e)` via
  `cover_point_with` (645). Accepted with a schema `4 → 5` bump + contract/ledger consumer notice (the typed
  `range_list` carrier `ranges`/`with_expr` shape changes).

## Blockers

- None.

## Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — at SV release `1.0.143`, valid SV trans-repeat bins `covergroup\foo ;coverpoint 1{bins\foo_0 =(1[*2]);}endgroup` (and `(1[->2])`/`(1[=2])`/`(1=>2[*3])`/`([1:2][*3])`) → `parseability_probe --parse … --profile sv_2017` = `did not consume full input … furthest_position=43` (REJECTED); cert `with_covergroup_expression`+`repeat_range` ∈ residual (`UNKNOWN=32`). Tool-proven in `STORE-AWARE-GEN.4b.14`.
- [x] **ROOT CAUSE (WHY + WHERE)** — `grammars/systemverilog.ebnf` `trans_range_list` (`:5063`) DROPPED the IEEE-1800 A.2.11 brackets: it had bare `trans_item ( star repeat_range )?` / `( implies … )?` / `( assign … )?` where `star := "*"` (`:6177`), `implies := "->"` (`:5504`), `assign := "="` (`:5484`) are the BARE expression operators, so `trans_item`'s greedy `expression` parsed `a*b` as one multiplication and the optional `repeat_range` never fired (AST kind `"star"`, optional EMPTY); the LRM-bracketed `[*n]` forms were unparseable. The identical delimiter-drop class as ledger `SV-0004` (sequence-repetition) / `SV-0002` (streaming).
- [x] **FIX** — Level-1 declarative GRAMMAR fix (LRM-faithful), the same `lbrack X rbrack -> {range:$N}` idiom as `SV-0004`: `trans_range_list := trans_item -> {kind:"simple",body:$1} | trans_item lbrack star repeat_range rbrack -> {kind:"star",body:$1,range:$4} | … implies … | … assign …`. No engine/codegen change; grammar-only + SV regen.
- [x] **ADDRESSED (verified)** — on the regen-lockstep build (SV parser regenerated `make focus_systemverilog` + binaries rebuilt): `(1[*2])`/`(1[->2])`/`(1[=2])`/`(1=>2[*3])`/`([1:2][*3])` REJECT→PASS; the bare `(1*2)`/`(1)`/`(1=>2)`/`(1,2)` still PASS (`(1*2)` now a `simple` multiply item). SV cert `UNKNOWN 32 → 31`, witness `1256 → 1257` (+1: `repeat_range` witnessed via the natural `[*n]` form), `total=1289`, `spf=0`, `proof_reverify_failures=0`; DETERMINISTIC at seeds 0/7/42 (`UNKNOWN=31`, residual byte-identical at all three).
- [x] **NO REGRESSION** — strict SUBSET: `comm -13 before after` EMPTY (ZERO newly-UNKNOWN; the only rule that left is the witnessed `repeat_range`). `ast_shape_contract_gate` GREEN (18 passed / 0 failed; no SV sample exercises trans/bins). `cargo test --lib --features "generated_parsers ebnf_dual_run"` **768 passed / 0 failed**. `make clippy_on_rust_change` ✅ `clippy_source_all_targets` ok (source-clean); the 188 generated-`eq_op` errors are pre-existing non-strict debt (identical to every prior SV release). 6 fully-certified grammars inert by construction (SV-only `.ebnf` change; their generated parsers Jun-17 byte-identical — spot-check regex `198/198 fully_certified=true`). **SV external corpus 14/14** (`sv_external_corpus_triage_gate` — `parse_fail_total=0`; the dead-branch analysis makes regression impossible for previously-accepted input: the old bare star/implies/assign branches were unreachable since `trans_item`'s `expression` always consumed `a*b`).
- [x] **LOCKSTEP** — this leaf + the tree Status/frontier; SV release `1.0.143 → 1.0.144` (Contract Identity + 1.0.144 Highlights + CONSUMER NOTICE in `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`); released-parser bug ledger `SV-0006`; SV parser book changelog-index `1.0.144`; shape-contract manifest `calibration_history`; `STORE-AWARE-GEN.4b.14` 9C-iii residual update (repeat_range CLOSED, with_covergroup_expression → `.2`); CHANGES / DEVELOPMENT_NOTES / MEMORY / LIVE_ACHIEVEMENT_STATUS (SV cert `32 → 31`). **NO schema bump** (schema stays `4` — the brackets are folded away by the return annotation, `{kind, range}` shape preserved, mirroring `SV-0004`).

## Acceptance Checklist (enforced) — leaf `.2`
- [x] **REPRODUCE / ISSUE** — at SV release `1.0.144` the covergroup `with_covergroup_expression` rule sat in the SV cert `UNKNOWN` residual (`UNKNOWN=31`, seeds 0/7/42). `PGEN_CERT_COVERAGE_DEBUG_PROBES=1 ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --report-certificate-coverage --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0` → `[plannable-probe] rule='with_covergroup_expression' parsed=true witnessed_target=false sample="covergroup\foo ;coverpoint 5_3.3{bins\foo_0 =with(7.23E37938);}endgroup"` (reachable-but-never-witnessed).
- [x] **ROOT CAUSE (WHY + WHERE)** — `grammars/systemverilog.ebnf` `bins_or_options:643` was bracket-less (`assign covergroup_range_list*`), missing the IEEE-1800 §A.2.11 braces `= { covergroup_range_list }`. `--parse-dump-ast-pretty` on the witness sample shows `bins … = with(7.23E37938)` parses as `{kind:"range_list"}` with the `with(…)` consumed as a `plain_tf` subroutine call INSIDE `ranges` (AST `kind:"call"/"plain_tf", name:"with"`) and the dedicated `( kw_with … with_covergroup_expression … )?` optional EMPTY (`with_expr:[]`) — because `covergroup_range_list → covergroup_value_range → covergroup_expression → expression` parses `with(x)` as a call, the greedy range-list swallows the whole `with(…)`. The bins sibling of the `SV-0006`/`SV-0004`/`SV-0002` delimiter-drop class.
- [x] **FIX** — Level-1 declarative GRAMMAR fix (LRM A.2.11-faithful): `bins_or_options:643` → `… assign lbrace covergroup_range_list rbrace ( kw_with lparen with_covergroup_expression rparen )? ( kw_iff … )? -> {kind:"range_list", … ranges:$7, with_expr:$9, iff:$10}`. The `}` terminates the range-list so a following `with(…)` is unambiguously the dedicated clause; bare forms route to the LRM `set` (647) / `cover_point_with` (645) homes. No engine/codegen change; grammar-only + SV regen.
- [x] **ADDRESSED (verified)** — regen-lockstep build (SV parser regenerated `make focus_systemverilog` + both binaries rebuilt). Cert `UNKNOWN 31 → 30`: `with_covergroup_expression` now `parsed=true witnessed_target=true` (forced sample `covergroup\foo ;coverpoint 5.73070{bins\foo_0 ={3937.4237}with(60.09_);}endgroup` — the LRM braced form); witness `1257 → 1258`; `total=1289`; `spf=0`; `proof_reverify_failures=0`; DETERMINISTIC at seeds 0/7/42 (`UNKNOWN=30`, residual rule-list byte-identical). Braced `bins b = {3} with(e)` populates `with_expr`; `bins b = {[0:63],65}` parses the embedded range.
- [x] **NO REGRESSION** — strict SUBSET: the AFTER 30-rule `UNKNOWN` list = the documented before-31 minus `with_covergroup_expression` (19 `no_path` + 6 SVA `kw_*` + `context_member_method_call` + 2 family-B `class_scoped_call` + 9A `wildcard_escape_nettype_identifier`/`kw_constant_d810ca96` = 30) → ZERO newly-UNKNOWN (`comm -13` empty), seeds 0/7/42 byte-identical. `ast_shape_contract_gate` GREEN (18/0; no SV sample exercises bins). `cargo test --lib --features "generated_parsers ebnf_dual_run"` **768 passed / 0 failed**. `make clippy_on_rust_change` source-clean (generated non-strict debt pre-existing). 6 fully-certified grammars inert (only `systemverilog_parser.rs` regenerated; regex `198/198 fully_certified spf=0`, byte-identical). **SV external corpus 14/14** (`parse_fail_total=0`, `preprocess_fail_total=0`). No-rejection-regression spot: bare `bins b = v` (set 647), `bins b = id with(e)` (cover_point_with 645) parse; the non-LRM bare comma-list `bins b = 1,2,3` now LRM-correctly requires `{1,2,3}`.
- [x] **LOCKSTEP** — this leaf + tree Status/frontier (tree `done`); SV release `1.0.144 → 1.0.145`, **schema `4 → 5`** (Contract Identity + 1.0.145 schema-version note + CONSUMER NOTICE in `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`); released-parser bug ledger `SV-0007`; SV parser book changelog-index `1.0.145` + schema-versioning row `5 (was 4)`; shape-contract manifest `calibration_history`; `STORE-AWARE-GEN` 9C-iii residual update (`with_covergroup_expression` CLOSED); CHANGES / DEVELOPMENT_NOTES / MEMORY / LIVE_ACHIEVEMENT_STATUS (SV cert `31 → 30`). **Schema bump `4 → 5`** — the typed `range_list` carrier `ranges` nesting collapses (`[[…]]` → `[…]`) + `with_expr` becomes populatable (the `SV-0002` carrier-shape class).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-24` | `.1` | regen-lockstep build (SV regen + binaries rebuilt); LRM forms `(1[*2])`/`(1[->2])`/`(1[=2])`/`(1=>2[*3])`/`([1:2][*3])` REJECT→PASS, bare `(1*2)` still PASS; SV cert `UNKNOWN 32 → 31` (witness `1256 → 1257`, +`repeat_range`), seeds 0/7/42 byte-identical residual, `spf=0`; strict-subset `comm -13` EMPTY; `ast_shape_contract_gate` GREEN 18/0; `cargo test --lib` 768/0; clippy source-clean (188 generated `eq_op` pre-existing); regex inert `198/198 fully_certified`; SV external corpus 14/14 | **IMPLEMENT DONE** — grammar-only LRM-bracket-fidelity fix; release `1.0.144` / schema `4` (no bump); ledger `SV-0006`; closes the 9C-iii `repeat_range` tail + a real parse defect; `with_covergroup_expression` (bins-set braces) → `.2` |
| `2026-06-24` | `.2` | regen-lockstep build (SV regen + binaries rebuilt); `--parse-dump-ast-pretty` on the cert witness sample proved `with(…)` parsed as a `plain_tf` call inside `ranges` with `with_expr:[]`; braced fix `bins_or_options:643 := … assign lbrace covergroup_range_list rbrace …`; AFTER: `with_covergroup_expression` `witnessed_target=true` (braced `={…}with(…)`); SV cert `UNKNOWN 31 → 30` (witness `1257 → 1258`), seeds 0/7/42 byte-identical residual, `spf=0`; strict-subset `comm -13` EMPTY; `ast_shape_contract_gate` GREEN 18/0; `cargo test --lib` 768/0; clippy source-clean; 6 grammars inert (regex `198/198`); SV external corpus 14/14 (`parse_fail_total=0`) | **IMPLEMENT DONE** — grammar-only LRM-brace-fidelity fix; release `1.0.145` / **schema `4 → 5`** (range_list `ranges` nesting + `with_expr` carrier change); ledger `SV-0007`; closes the 9C-iii `with_covergroup_expression` tail; **tree `SV-COVERGROUP-FIDELITY` complete** (`UNKNOWN 32 → 30` across `.1`+`.2`) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.1` | `PGEN-SV-COVERGROUP-FIDELITY-0001` | **DONE (IMPLEMENT, GRAMMAR + SV regen).** Restored the LRM A.2.11 trans-repeat brackets in `trans_range_list`. `(1[*2])`/`(1[->2])`/`(1[=2])`/`(1=>2[*3])` REJECT→PASS; SV cert `UNKNOWN 32 → 31` (`repeat_range` witnessed), seeds 0/7/42, spf=0, strict-subset; 6 grammars inert; shape-contract GREEN; 768/0 tests; clippy source-clean; SV corpus 14/14. SV release `1.0.144` / schema `4` (no bump); ledger `SV-0006`. |
| `.2` | `PGEN-SV-COVERGROUP-FIDELITY-0002` | **DONE (IMPLEMENT, GRAMMAR + SV regen).** Restored the LRM A.2.11 bins-set braces in `bins_or_options:643` (`assign lbrace covergroup_range_list rbrace`). `with_covergroup_expression` witnessed via the braced `= {…} with(…)` form; SV cert `UNKNOWN 31 → 30` (witness `1257 → 1258`), seeds 0/7/42, spf=0, strict-subset (`comm -13` empty); 6 grammars inert; shape-contract GREEN 18/0; 768/0 tests; clippy source-clean; SV corpus 14/14. SV release `1.0.145` / **schema `4 → 5`**; ledger `SV-0007`. |

## Changelog

- `2026-06-24`: Created task tree from the `STORE-AWARE-GEN.4b.14` hand-off (real-failure proof gathered).
- `2026-06-24`: `.1` DONE — `trans_range_list` LRM brackets restored; SV `1.0.144` / ledger `SV-0006`; cert `UNKNOWN 32 → 31`.
- `2026-06-24`: `.2` DONE — `bins_or_options:643` LRM bins-set braces restored (`= { covergroup_range_list }`); `with_covergroup_expression` witnessed; SV `1.0.145` / **schema `4 → 5`** / ledger `SV-0007`; cert `UNKNOWN 31 → 30`. **Tree `done`** (both leaves landed; SV cert `32 → 30`).
