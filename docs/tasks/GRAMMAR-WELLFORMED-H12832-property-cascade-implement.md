# GRAMMAR-WELLFORMED.H.12.5.8.3.2 — SVA PROPERTY-layer precedence-cascade IMPLEMENT

Implementation leaf for the PROPERTY layer of the `H.12.5.8` SVA infix binary-operator parse bug.
The SEQUENCE layer (`.8.3.1` / `.8.3.1.1`, `PGEN-GRAMMAR-WELLFORMED-0132`, release `1.0.148`) is landed;
this leaf is its dual on the PROPERTY layer and REUSES the same already-landed engine fix
(`prefer_sole_reference_passthrough_sites` in `reach_hops_pass`).

> Slice `PGEN-GRAMMAR-WELLFORMED-0133` (GRAMMAR FIX — flagship SystemVerilog, released-SV ceremony).
> Status: `done` — LANDED. SV cert `UNKNOWN 26 → 22` (the 4 `until`-family keywords witnessed),
> deterministic seeds 0/7/42, zero newly-UNKNOWN; release `1.0.148 → 1.0.149`, schema stays `6`, ledger `SV-0011`.
> Reads with [[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]],
> [[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_always_signoff_decisions]],
> [[feedback_no_workarounds_fix_hierarchy]], [[project_ebnf_is_single_source_of_truth]],
> [[feedback_correctness_before_speed]], [[feedback_ebnf_meta_grammar_lockstep]],
> [[feedback_quantified_group_extraction]], [[feedback_regex_book_live]].
> Parent: `GRAMMAR-WELLFORMED.H.12.5.8` (split by `.8.1`); blueprint `.8.2` (`-0119`); sequence layer `.8.3.1.1` (`-0132`).

## THE ROOT CAUSE (WHY + WHERE) — tool-backed, decisive

- **WHY (PARSE).** `property_expr_sv_2017` (`grammars/systemverilog.ebnf:4054`) and `property_expr_sv_2023`
  (`:4126`) are flat, directly-left-recursive ordered choices (`property_expr OP property_expr` for
  `or`/`and`/`iff`/`until`/`s_until`/`until_with`/`s_until_with`/`implies_binary`). PGEN's LR eliminator
  handles only the indirect bare-reference wrapper chain, not direct inline `A := A op A`, so the runtime
  cycle-breaker blocks every left-recursive infix branch and the property-only infix operators reject at
  the operator. Same defect class as the now-fixed `sequence_expr`.
- **WHERE (WITNESS).** Of the LR-blocked property infix operators, only the `until`-family keywords are
  **property-only** (`until`/`s_until`/`until_with`/`s_until_with` have no other use site), so their
  keyword rules are the cert residual; `kw_or`/`kw_and`/`kw_iff` are witnessed via other constructs.
- **TOOL EVIDENCE (this session, reproduced on the released `1.0.148` binary):**
  - Baseline cert (seed 0, count 40, `--grammar-profile sv_2017 --entry-rule systemverilog_file`):
    `CERTIFICATE-COVERAGE: … total=1299 proof=1 witness=1272 UNKNOWN=26 spf=0`, deterministic at seeds
    0/7/42 (all = 26). `PGEN_CERT_COVERAGE_DUMP_ALL=1` shows the 4 `until`-family keyword rules in the
    residual: `kw_s_until_073539cf`, `kw_s_until_with_3a7bcf5b`, `kw_until_b310f4d4`, `kw_until_with_f31dfb3c`.
  - Pre-change parse matrix (`parseability_probe --parse systemverilog --profile sv_2017`, property body in a
    `property … endproperty` shell): REJECT = `a iff b`, `a until b`, `a s_until b`, `a until_with b`,
    `a s_until_with b`, `a until b until c`, `a until b or c`, `(a until b)`, `not a until b`,
    `(always a) or (always b)`, `a until b iff c`, `a until accept_on(c) b`, `a #=# b until c`.
    PARSE (must stay) = `a`, `not a`, `not not a`, `always a`, `nexttime a`, `a or b`, `a and b`,
    `a #-# b`, `a #=# b`, `a |= b`, `-> a`, `a -> b`, `strong(a)`, `accept_on(a) b`.
  - Current nested-carrier shape (AST dump of `not not a`): each nesting level goes through the
    `property_expr` dispatch wrapper — `{kind:"not", body:{kind:"sv_2017", body:{kind:"not",
    body:{kind:"sv_2017", body:{kind:"sequence", body:a}}}}}`.

## THE FIX — schema-PRESERVING hybrid precedence cascade (fix-hierarchy tier: grammar, level-1 declarative)

Director decision (`.8.2`/`-0119`): direction A — restructure to an IEEE 1800-2017 §16 (Table 16-3)
precedence cascade. The dominant `.8.3` decision (`.8.2` sub-question 1) is the AST-shape/schema outcome:
**"prefer shape-preserving if achievable."** It IS achievable here, via the hybrid:

- **Binary-infix cascade** (the LR-blocked, property-only/property-level infix bug), loosest→tightest per
  Table 16-3: `until-family (10, right) > iff (9, right) > or (8, left) > and (7, left) > prop_primary`.
- **`prop_primary` keeps every non-LR branch BYTE-IDENTICAL** — `sequence`/`strong`/`weak`/`paren`/`not`/
  `implies_unary`/`sequence_or_assign`/`if`/`case`/`imp_minus`/`imp_assign`/`implies_binary`(left dead, as
  today)/`nexttime*`/`s_nexttime*`/`always*`/`s_always`/`eventually*`/`accept_on`/`reject_on`/
  `sync_accept_on`/`sync_reject_on`/`instance`/`clocking` — with their bodies STILL referencing the
  `property_expr` dispatch. This PRESERVES (a) the exact current nested `{kind:"sv_2017"}`-wrapped carriers
  of every currently-parsing form, and (b) the correct "bind-whole-rhs" precedence of the LOOSE operators
  (impl `#-#`/`#=#`/`|=` at 11, `always`/`eventually` at 12, `accept_on`-family at 13) — because a dispatch
  body re-enters the full cascade, so a loose prefix op binds its whole rhs (= loosest), which is correct.
- **Schema stays 6** — the SV-0008/SV-0010 strictly-more-permissive category: every previously-VALID
  property parse keeps its carrier via `-> $1` passthroughs and the byte-identical primary; only
  previously-REJECTED forms (`until`-family, property-level `iff`/`or`/`and` over property operands) gain
  new `{kind, …}` shapes. Left-assoc `or`/`and` adopt the sequence-cascade list shape
  `{kind, lhs, rest:[…]}`; right-assoc `iff`/`until`-family keep binary `{kind, lhs, rhs}` (natural right
  fold). Neither realized before this fix, so no previously-realized carrier changes.

### DOCUMENTED LIMITATION (transparency, book + this leaf)

The *tight* prefix operators `not`/`nexttime`/`s_nexttime` (level 6) remain in `prop_primary` referencing
the dispatch body, so they bind their whole rhs (e.g. `not a until b` parses as `not (a until b)` rather
than the strict-Table-16-3 `(not a) until b`). This is **unchanged from the pre-fix behavior** (these
forms parenthesize in practice; `not a until b` simply rejected before), preserves schema 6, and is the
smallest honest fix. A strict full cascade for prefix precedence would require pulling `not`/`nexttime`
out of the primary with tighter-level bodies → strips the dispatch wrappers → schema bump; deferred as a
future correctness refinement if a downstream consumer needs strict prefix precedence.

## VERIFICATION PLAN (mandatory before commit)

1. Apply the cascade to both profiles in `grammars/systemverilog.ebnf`; regen the SV parser; rebuild
   `ast_pipeline` (debug, `generated_parsers ebnf_dual_run`) + `parseability_probe` (release) with DEFAULT
   parser paths.
2. Parse matrix all-green: the REJECT forms above now PARSE with correct precedence (spot-check
   `a until b iff c` ⇒ `a until (b iff c)`; `a until b or c` ⇒ `a until (b or c)`); the PARSE forms keep
   their exact carriers (re-dump `not not a` ⇒ unchanged).
3. Cert: `UNKNOWN 26 → 22` (the 4 `until`-family keywords newly WITNESS), deterministic seeds 0/7/42,
   `spf=0`, zero newly-UNKNOWN.
4. NO REGRESSION: the 6 fully-certified grammars BYTE-IDENTICAL (engine unchanged this leaf; grammar change
   is SV-only); SV external corpus 14/14 (`sv_external_corpus_triage_gate`); `ast_shape_contract` GREEN;
   `--lint-grammar` clean (no new `non_terminating`/`unreachable_rules`/`ordered_choice_shadowing`/
   `profile_orphans`); `clippy_on_rust_change` source-clean.
5. LOCKSTEP: release `1.0.148 → 1.0.149`, schema stays `6`, ledger `SV-0011`; SV integration contract; SV
   book `changelog-index.md` + `json-carrier.md` (property cascade rules + or/and list shape + limitation);
   `PGEN_RELEASED_PARSER_BUG_LEDGER.md`; `CHANGES.md`/`DEVELOPMENT_NOTES.md`/`MEMORY.md`/
   `LIVE_ACHIEVEMENT_STATUS.md`. EBNF meta-grammar lockstep: the cascade adds no new EBNF *construct*
   (only new rules using existing forms) → `ebnf.ebnf` needs no change (verify).

## Acceptance Checklist (enforced) — H.12.5.8.3.2
- [x] **REPRODUCE / ISSUE** — baseline `PGEN_CERT_COVERAGE_DUMP_ALL=1 … --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0` ⇒ `CERTIFICATE-COVERAGE: … total=1299 witness=1272 UNKNOWN=26 spf=0` with `kw_until_b310f4d4`/`kw_s_until_073539cf`/`kw_until_with_f31dfb3c`/`kw_s_until_with_3a7bcf5b` in the residual; parse matrix (`parseability_probe --parse … --profile sv_2017`, property body in a `property … endproperty` shell): `a until b`/`a s_until b`/`a until_with b`/`a s_until_with b`/`a iff b`/`(always a) or (always b)` all REJECT pre-fix.
- [x] **ROOT CAUSE (WHY + WHERE)** — directly-left-recursive `property_expr_sv_2017` (`grammars/systemverilog.ebnf:4054`) / `property_expr_sv_2023` (`:4126`) (`property_expr OP property_expr`); PGEN's indirect-only LR eliminator can't eliminate direct inline LR → runtime cycle-breaker blocks the infix branches; the 4 `until`-family operators are property-only so their keyword rules are the cert residual (DUMP_ALL + parse matrix + AST-dump of the dispatch-wrapped nesting, all pasted above).
- [x] **FIX** — schema-preserving hybrid precedence cascade (grammar, level-1 declarative): both profiles → §16 Table 16-3 binary-infix cascade (`prop_until > prop_iff > prop_or > prop_and > prop_primary`, 5 new `prop_*` rules/profile, all right-assoc inline `head op self` so no tail rules); `prop_primary` keeps every non-LR branch byte-identical with bodies still referencing the `property_expr` dispatch. Reuses the landed `prefer_sole_reference_passthrough_sites` engine fix (no engine change).
- [x] **ADDRESSED (verified)** — SV cert `UNKNOWN 26 → 22` (`CERTIFICATE-COVERAGE: … total=1304 proof=1 witness=1281 UNKNOWN=22 spf=0`), **deterministic seeds 0/7/42** (all = 22); set-diff vs baseline = newly WITNESSED exactly the 4 `until`-family keywords, newly-UNKNOWN = ∅. Parse matrix REJECT→PASS with correct precedence (`a until b iff c` ⇒ `until(a, iff(b,c))`; `a and b or c` ⇒ `or(and(a,b), c)`); previously-parsing forms byte-identical (`not not a` AST dump unchanged: `not/sv_2017/not/sv_2017/sequence`).
- [x] **NO REGRESSION** — cert seeds **0/7/42** `spf=0`; the **6 fully-certified grammars** all `fully_certified=true` (json 9/9, regex 198/198, vhdl 216/216, svpp 74/74, rtl_frontend 169, rtl_const_expr 48/48 — grammar-only change is SV-isolated, engine untouched); **SV external corpus 14/14** (`sv_external_corpus_triage_gate`, all `primary_parse_failure_* <none>`); `ast_shape_contract_gate` GREEN (18 passed/0 failed); `--lint-grammar` clean (`1425` rules, +10; `non_terminating=0`, `unreachable_rules=0`, `ordered_choice_shadowing=0`, `profile_orphans=0`, pre-existing `always_matches=8` A2 backlog unchanged); `clippy_on_rust_change` source-clean (generated `eq_op` debt pre-existing, non-strict — unchanged from `-0132`).
- [x] **LOCKSTEP** — release `1.0.148 → 1.0.149`, schema stays `6`, ledger `SV-0011`; SV integration contract (version + schema note); SV book `changelog-index.md` (new 1.0.149 section) + `json-carrier.md` (new `property_expr` cascade row); `PGEN_RELEASED_PARSER_BUG_LEDGER.md` (`SV-0011`); `CHANGES.md` / `DEVELOPMENT_NOTES.md` / `LIVE_ACHIEVEMENT_STATUS.md` / `MEMORY.md`; parent tree `GRAMMAR-WELLFORMED.md` frontier. `ebnf.ebnf` unchanged — the cascade adds no new EBNF *construct* (only new rules using existing forms).
