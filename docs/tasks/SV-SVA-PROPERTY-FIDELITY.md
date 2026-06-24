# SV-SVA-PROPERTY-FIDELITY: restore IEEE-1800 LRM bracket fidelity in the SVA bounded-property operators

## Metadata

- Tree ID: `SV-SVA-PROPERTY-FIDELITY`
- Status: `done` (`.1` landed: `PGEN-SV-SVA-PROPERTY-FIDELITY-0001`, 2026-06-24; SV release `1.0.146`, ledger `SV-0008`; SV cert `UNKNOWN 30 → 29`)
- Roadmap lane: parser sign-off — SV grammar/LRM parse-fidelity (a parser rejecting valid SV is a
  defect, [[feedback_uvm_is_valid_sv]] / [[project_ebnf_is_single_source_of_truth]]); also closes the
  SV cert tail (`STORE-AWARE-GEN.4b.15` 9A-ii — the dead `kw_constant_d810ca96` token).
- Family / slice-id prefix: `PGEN-SV-SVA-PROPERTY-FIDELITY-<NNNN>`
- Created: `2026-06-24`
- Last updated: `2026-06-24`
- Owner: repo-local workflow
- Origin: tool-proven in `STORE-AWARE-GEN.4b.15` (`PGEN-STORE-AWARE-GEN-0023`) — the
  `systemverilog.ebnf` `property_expr` bounded-property family DROPPED the LRM's disambiguating
  `[ ]` brackets (and sv_2017's `nexttime` additionally mis-tokenized `constant_expression` into the
  literal keyword `constant`), which REJECTS valid SV. Disciplines:
  [[feedback_no_codebase_change_without_tool_backed_facts]] (real-failure proof gathered),
  [[feedback_correctness_before_speed]], [[feedback_uvm_is_valid_sv]],
  [[project_ebnf_is_single_source_of_truth]], [[feedback_regex_book_live]].

## The frame (why this tree exists)

`grammars/systemverilog.ebnf` diverges from IEEE 1800-2017/2023 Annex A.2.10 in the SVA `property_expr`
bounded-property operators. The LRM bounds the six unary property operators with literal `[ ]` brackets:

```
nexttime    [ constant_expression ]                 property_expr
s_nexttime  [ constant_expression ]                 property_expr
always      [ cycle_delay_const_range_expression ]  property_expr
s_always    [ constant_range ]                      property_expr
eventually  [ constant_range ]                      property_expr
s_eventually[ cycle_delay_const_range_expression ]  property_expr
```

The synthesized grammar flattened each `[ … ]` to a bracket-less `( … )?` optional group — and in the
`sv_2017` profile the `nexttime` index was further mis-tokenized: `constant_expression` became the literal
keyword `constant` (`kw_constant_d810ca96 := trivia /constant\b/`, `:5598`) followed by a bare
`expression` (`:4083`). Consequences (both tool-proven in `.4b.15`):

1. **Real parse-fidelity defect:** all six LRM-bracketed forms are REJECTED — `nexttime [3] a`,
   `s_nexttime [3] a`, `always [2:4] a`, `s_always [2:4] a`, `eventually [2:4] a`, `s_eventually [2:4] a`
   (6/6 `did not consume full input`), while the bare `nexttime a` / `always a` parse. Conversely the
   non-LRM `nexttime constant 3 a` (sv_2017) is wrongly ACCEPTED.
2. **Cert UNKNOWN tail (`STORE-AWARE-GEN.4b.15` 9A-ii):** `kw_constant_d810ca96` is the lone keyword token
   reachable ONLY through the `:4083` mis-synthesis, so it sits in the SV cert `UNKNOWN` residual
   (`parsed=false` — the witness generator emits a malformed real number after `constant`). Restoring the
   LRM brackets removes the mis-synthesis, so the dead token leaves the grammar.

The inner range nonterminals are themselves bracket-free (`constant_range := constant_expression colon
constant_expression`, `:1372`; `cycle_delay_const_range_expression := …`, `:1564`), so the `[ ]` belong at
the operator level (no double-bracketing).

## Goal

Make `grammars/systemverilog.ebnf` LRM-faithful in the SVA bounded-property operators (both `sv_2017` and
`sv_2023` profiles) so valid SV parses and the dead `kw_constant_d810ca96` token is removed. Generated SV
parser regenerated; all SV gates green; SV cert `UNKNOWN` strictly decreases (the `kw_constant_d810ca96`
residual removed) with ZERO newly-UNKNOWN; book/contract/ledger/shape-contract lockstep.

## Non-Goals

- Touching any non-`property_expr` SV construct, or any other grammar (the change is
  `systemverilog.ebnf`-local ⇒ the other 6 generated parsers are inert by construction).
- The SVA infix binary-operator parse bug (`until`/`s_until`/`intersect`/`within` — owned by
  `GRAMMAR-WELLFORMED.H.12.5.8`); the 6 `no_path` / SVA-`kw_*` residual is out of scope.
- The `cycle_delay_const_range_expression` `[constant_expression : $]` `$`-bound inner form (a separate
  inner-rule fidelity item, not the operator-bracket drop fixed here).

## Acceptance Criteria

- The six LRM bracketed bounded-property forms (`nexttime [N]` / `s_nexttime [N]` / `always [R]` /
  `s_always [R]` / `eventually [R]` / `s_eventually [R]`) PARSE; the bare forms (`nexttime a` …) still
  parse (no rejection regression); the non-LRM `nexttime constant 3 a` is now correctly REJECTED.
- SV cert `UNKNOWN` strictly decreases (`kw_constant_d810ca96` removed), ZERO newly-UNKNOWN (`comm -13`
  empty), deterministic at seeds 0/7/42, `spf=0`.
- The 6 fully-certified grammars BYTE-IDENTICAL (inert — SV-only grammar change); `ast_shape_contract_gate`
  GREEN; `cargo test --lib` green; clippy source-clean; SV external corpus 14/14.
- Lockstep: this tree, the SV parser book, the SV integration contract + released-parser ledger, the
  shape-contract manifest, CHANGES / DEVELOPMENT_NOTES / MEMORY / LIVE_ACHIEVEMENT_STATUS (SV cert number).
  Schema bump iff the typed carrier shape changes for previously-parseable input (decided empirically).

## Task Tree

- ID: `SV-SVA-PROPERTY-FIDELITY`
  Status: `active`
  Goal: LRM-faithful SVA bounded-property operator brackets in `systemverilog.ebnf`
  Children: `.1` (restore the six bracketed forms, both profiles; remove the dead `kw_constant_d810ca96`)

- ID: `SV-SVA-PROPERTY-FIDELITY.1`
  Status: `done` (`PGEN-SV-SVA-PROPERTY-FIDELITY-0001`, 2026-06-24 — SV release `1.0.146`, schema `5` (no bump), ledger `SV-0008`; cert `UNKNOWN 30 → 29`, dead `kw_constant_d810ca96` removed)
  Goal: restore the LRM `[ ]` on the six `property_expr` bounded-property forms in both profiles
    (`nexttime`/`s_nexttime [ constant_expression ]`, `always`/`s_eventually [ cycle_delay_const_range_expression ]`,
    `s_always`/`eventually [ constant_range ]`); remove the dead `kw_constant_d810ca96` token.
  Acceptance: the six bracketed forms PARSE; bare forms still parse; `nexttime constant 3 a` now REJECTED;
    SV cert `UNKNOWN` strictly decreases (`kw_constant_d810ca96` removed), ZERO newly-UNKNOWN, seeds 0/7/42,
    spf=0; 6 grammars inert; `ast_shape_contract_gate` green; tests + clippy green; SV corpus 14/14;
    book/contract/ledger/shape-contract lockstep.
  Verification: 6/6 LRM forms REJECT→PASS (both profiles), bare PASS, `nexttime constant 3 a` REJECT; SV cert
    `UNKNOWN 30 → 29` (total `1289 → 1288`, witness `1258`, spf=0) deterministic seeds 0/7/42; strict-subset
    (`comm -13` empty); `ast_shape_contract_gate` 18/0 (no schema bump); `cargo test --lib` 768/0; SV external
    corpus 14/14 (`parse_fail_total=0`); regex inert `198/198`; clippy source-clean.
  Commit: `PGEN-SV-SVA-PROPERTY-FIDELITY-0001`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `SV-SVA-PROPERTY-FIDELITY.1` | `done` | landed (`PGEN-SV-SVA-PROPERTY-FIDELITY-0001`): SV `1.0.146`, cert `UNKNOWN 30 → 29`, the six LRM bounded-property bracket forms parse + the dead `kw_constant_d810ca96` removed. |

**Frontier EMPTY — tree `done`.** The SVA bounded-property LRM-bracket-drop class is closed (the property sibling of the covergroup `SV-COVERGROUP-FIDELITY` family). Remaining SV cert residual (`UNKNOWN=29`) is tracked elsewhere: `STORE-AWARE-GEN.4b.16` (9A-i `wildcard_escape_nettype_identifier` store-gate COUNT-prelude), the DEFERRED `context_member_method_call` + 2 family-B `class_scoped_call`, the SVA infix parse-bugs (`GRAMMAR-WELLFORMED.H.12.5.8`), and the 19 `no_path` LRM-legitimate rules.

## Decisions

- `2026-06-24`: Fix the WHOLE bounded-property family in ONE slice (one coherent defect class — the LRM
  bracket drop in `property_expr`), mirroring `SV-COVERGROUP-FIDELITY.1`'s "all four trans forms at once"
  granularity, rather than only the cert-flagged `nexttime` member — per the signoff/LRM-fidelity ethos
  ([[feedback_be_alert_root_cause_fishy_immediately]]). The bare `( … )?` optional branches are effectively
  dead (the bare property operator is already covered by the preceding non-indexed branch; without brackets
  the optional inner expr cannot be delimited from the following `property_expr`), so bracketing LOSES no
  real acceptance and GAINS the LRM `[ … ]` forms.

## Open Questions

- Schema bump? **RESOLVED (2026-06-24, evidence-backed): NO bump (schema stays `5`).** The `{kind,
  const_expr/range, body}` field set is preserved and the `[ ]` fold away (not captured). For the five
  forms that already used a nonterminal (`s_nexttime`/`always`/`s_always`/`eventually`/`s_eventually`) the
  captured `const_expr`/`range` content is byte-identical (the same inner-nonterminal AST). The only content
  change is `sv_2017` `nexttime_const.const_expr`, corrected from the bogus-only `(kw_constant expression)`
  group to `constant_expression` (now matching `sv_2023` + LRM) — and that branch was only ever producible
  by the non-LRM `nexttime constant <expr>` input, which no consumer feeds. No previously-VALID input
  changes shape. Empirically confirmed: `ast_shape_contract_gate` GREEN (18/0) — no locked sample's shape
  changed. Mirrors the `SV-COVERGROUP-FIDELITY.1` / `SV-0006` "brackets folded away ⇒ no bump" call.

## Blockers

- None.

## Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — at SV release `1.0.145`, all six IEEE-1800 LRM bounded-property bracket forms are REJECTED: `printf 'module m; assert property (nexttime [3] a); endmodule' \| ./rust/target/release/parseability_probe --parse systemverilog /dev/stdin --profile sv_2017` → `did not consume full input … furthest_position=41`; likewise `s_nexttime [3] a` (43), `always [2:4] a` (41), `s_always [2:4] a` (43), `eventually [2:4] a` (45), `s_eventually [2:4] a` (47) — 6/6 REJECT while the bare `nexttime a`/`always a` parse and the non-LRM `nexttime constant 3 a` wrongly PARSES. The lone keyword token `kw_constant_d810ca96` sat in the SV cert `UNKNOWN` residual (`UNKNOWN=30`, seeds 0/7/42; `PGEN_CERT_COVERAGE_DEBUG_PROBES=1` → `[plannable-probe] rule='kw_constant_d810ca96' parsed=false …` with a malformed real after `constant`). Tool-proven in `STORE-AWARE-GEN.4b.15`.
- [x] **ROOT CAUSE (WHY + WHERE)** — `grammars/systemverilog.ebnf` `property_expr_sv_2017` (`:4083`–`:4099`) + `property_expr_sv_2023` (`:4155`–`:4171`) DROPPED the IEEE-1800 Annex A.2.10 `[ ]` brackets on the six bounded-property operators: each was bracket-less `( <inner> )?` where the LRM is `OP [ <inner> ] property_expr` (Annex A txt 2017/2023 `:884`–`892`). Without brackets the optional inner range/expr cannot be delimited from the following `property_expr`, so the bracketed LRM forms were unparseable (and the bare forms route to the preceding non-indexed branch). The `sv_2017` `nexttime` was additionally mis-tokenized: the LRM nonterminal `constant_expression` became the literal keyword `constant` (`kw_constant_d810ca96 := trivia /constant\b/`, `:5598`) + a bare `expression` (`:4083`), so the lone `kw_constant_d810ca96` token was reachable ONLY via that mis-synthesis → cert `UNKNOWN`. The inner range rules are bracket-free (`constant_range`, `:1372`; `cycle_delay_const_range_expression`, `:1564`), so the `[ ]` belong at the operator level. The same dropped-delimiter class as ledger `SV-0006`/`SV-0007` (covergroup) / `SV-0004` (sequence-repetition) / `SV-0002` (streaming).
- [x] **FIX** — Level-1 declarative GRAMMAR fix (LRM A.2.10-faithful), the `lbrack X rbrack` idiom: the six branches in BOTH profiles → `kw_nexttime lbrack constant_expression rbrack property_expr -> {kind:"nexttime_const", const_expr:$3, body:$5}`, `kw_s_nexttime lbrack constant_expression rbrack …`, `kw_always lbrack cycle_delay_const_range_expression rbrack … -> {kind:"always_range", range:$3, body:$5}`, `kw_s_always lbrack constant_range rbrack …`, `kw_eventually lbrack constant_range rbrack …`, `kw_s_eventually lbrack cycle_delay_const_range_expression rbrack …`. The dead `kw_constant_d810ca96 := trivia /constant\b/` token (`:5598`, sole user removed) deleted. No engine/codegen change; grammar-only + SV regen.
- [x] **ADDRESSED (verified)** — regen-lockstep build (SV parser regenerated `make focus_systemverilog` + both binaries rebuilt). The six LRM forms REJECT→PASS (both `sv_2017` and `sv_2023`); the bare `nexttime a`/`always a` still PASS; the non-LRM `nexttime constant 3 a` now correctly REJECTS (`furthest_position=46`). SV cert `UNKNOWN 30 → 29` (the dead `kw_constant_d810ca96` removed; `total 1289 → 1288`, witness `1258` unchanged, `proof=1`, `spf=0`, `proof_reverify_failures=0`); DETERMINISTIC at seeds 0/7/42 (`UNKNOWN=29`, `total=1288` all three). `nexttime [3] a` AST = `{kind:"nexttime_const", const_expr:<constant_expression>, body:<property_expr>}`.
- [x] **NO REGRESSION** — strict SUBSET: the only rule that left the residual is the removed `kw_constant_d810ca96`; ZERO newly-UNKNOWN (`comm -13` empty, script-verified). `ast_shape_contract_gate` GREEN (18 passed / 0 failed) → **no schema bump** (the `{kind, const_expr/range, body}` field set is preserved; the `[ ]` fold away; the five nonterminal forms capture byte-identical content; only the `sv_2017` `nexttime_const.const_expr` carrier is corrected from the bogus-only `(kw_constant expression)` group to `constant_expression`, now matching `sv_2023` + LRM — no previously-VALID input changes shape). `cargo test --lib --features "generated_parsers ebnf_dual_run"` **768 passed / 0 failed**. **SV external corpus 14/14** (`sv_external_corpus_triage_gate` — `parse_fail_total=0`, `preprocess_fail_total=0`, `cases_blocked_total=0`). 6 fully-certified grammars inert (SV-only `.ebnf` change ⇒ only `systemverilog_parser.rs` regenerated; regex `198/198 fully_certified=true spf=0`). `clippy_on_rust_change` → `clippy_source_all_targets` ok (source-clean); the 188 generated-`eq_op` errors are pre-existing non-strict debt (identical to every prior SV release).
- [x] **LOCKSTEP** — this leaf + the tree Status/frontier (tree `done`); SV release `1.0.145 → 1.0.146` (Contract Identity + 1.0.146 highlight + CONSUMER NOTICE in `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`); released-parser bug ledger `SV-0008`; SV parser book changelog-index `1.0.146` + assertions/property chapter (bounded-property bracket forms); shape-contract manifest `calibration_history` + release version; `STORE-AWARE-GEN.4b.15` 9A-ii residual update (`kw_constant_d810ca96` CLOSED); CHANGES / DEVELOPMENT_NOTES / MEMORY / LIVE_ACHIEVEMENT_STATUS (SV cert `30 → 29`); `docs/TASK_TREE.md` registration. **NO schema bump** (schema stays `5` — brackets folded away by the annotation, `{kind, const_expr/range, body}` shape preserved, mirroring `SV-0006`/`SV-COVERGROUP-FIDELITY.1`).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-24` | `.1` | regen-lockstep build (SV regen `make focus_systemverilog` + both binaries rebuilt); `--lint-grammar` clean (`non_terminating=0`, `ordered_choice_shadowing=0`, `unreachable_rules=0`, `profile_orphans=0`; `always_matches=8` pre-existing A2 backlog unchanged; `1404` rules, −1 from the removed token); the six LRM forms `nexttime/s_nexttime/always/s_always/eventually/s_eventually [..]` REJECT→PASS (both profiles), bare PASS, `nexttime constant 3 a` REJECT; SV cert `UNKNOWN 30 → 29` (total `1289 → 1288`, witness `1258`, spf=0, proof_reverify=0) deterministic seeds 0/7/42; strict-subset (`comm -13` empty — only `kw_constant_d810ca96` left); `ast_shape_contract_gate` GREEN 18/0 (⇒ no schema bump); `cargo test --lib` 768/0; SV external corpus 14/14 (`parse_fail_total=0`); regex inert `198/198 fully_certified`; clippy source-clean (188 generated `eq_op` pre-existing) | **IMPLEMENT DONE** — grammar-only LRM-bracket-fidelity fix; release `1.0.146` / schema `5` (no bump); ledger `SV-0008`; closes a real parse defect (the whole bounded-property family) + the `kw_constant_d810ca96` cert tail; **tree `SV-SVA-PROPERTY-FIDELITY` complete** |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.1` | `PGEN-SV-SVA-PROPERTY-FIDELITY-0001` | **DONE (IMPLEMENT, GRAMMAR + SV regen, RELEASED).** Restored the IEEE-1800 Annex A.2.10 `[ ]` brackets on the six `property_expr` bounded-property operators (both profiles) + removed the dead mis-synthesized `kw_constant_d810ca96` token. Six LRM forms REJECT→PASS, bare PASS, `nexttime constant 3 a` REJECT; SV cert `UNKNOWN 30 → 29`, seeds 0/7/42, spf=0, strict-subset; 6 grammars inert; shape-contract GREEN 18/0; 768/0 tests; SV corpus 14/14; clippy source-clean. SV release `1.0.146` / schema `5` (no bump); ledger `SV-0008`. |

## Changelog

- `2026-06-24`: Created task tree from the `STORE-AWARE-GEN.4b.15` hand-off (real-failure proof gathered).
- `2026-06-24`: `.1` DONE — restored the LRM `[ ]` brackets on the six bounded-property operators (both
  profiles) + removed the dead `kw_constant_d810ca96`; SV `1.0.146` / schema `5` (no bump) / ledger `SV-0008`;
  cert `UNKNOWN 30 → 29`. **Tree `done`.**
