# GRAMMAR-WELLFORMED.H.12.7 — post-SVA-cascade SV `UNKNOWN=22` full residual re-adjudication + book lockstep

Tools-first re-adjudication of the SystemVerilog certificate-coverage residual at its
post-SVA-cascade count (`UNKNOWN=22`), the dual of the `H.12.6` `no_path` LRM re-audit
applied to the now-much-smaller residual, plus closing the **6 slices of accumulated
top-level book drive-arc drift** (`grammar-wellformedness.md` narrated the SV `UNKNOWN`
arc only down to `32`; the `32 → 22` reductions landed in the per-parser SV book +
contract + ledger but never reached the top-level mastery chapter).

> Slice `PGEN-GRAMMAR-WELLFORMED-0134` (**PURE-DOCS** — tracker + book/continuity lockstep;
> NO code/grammar/generated/release/schema/ledger change). Status: `done`.
> Reads with [[feedback_systematically_use_debug_toolbox]], [[feedback_why_and_where_before_solution]],
> [[feedback_tools_first_no_guessing]], [[feedback_no_rule_deletion_without_lrm_proof]],
> [[feedback_regex_book_live]], [[project_ebnf_is_single_source_of_truth]],
> [[feedback_always_signoff_decisions]].
> Parent: the SV `UNKNOWN`→0 lane `GRAMMAR-WELLFORMED.H.12`; precedent `H.12.6`
> (`-0108`, the 20→19 `no_path` LRM re-audit) and `H.12.5.1`/`.5.2` (`-0080`/`-0081`,
> the A1 entry-relative / A2 profile-relative adjudication).

## Why now (the frontier)

The SVA-infix left-recursion class is CLOSED (`H.12.5.8.3.1.1`/`.8.3.2`, `-0132`/`-0133`):
SV cert `UNKNOWN` fell `28 → 26 → 22`. The residual `22` is now small enough to adjudicate
in full at one sitting, and the entry/profile-relative `no_path` adjudication was last run
tools-backed at the `UNKNOWN=86`/`20`-`no_path` era (`H.12.6`). This slice reproduces the
adjudication at the current count, proves each residual rule's category with a fresh cert
run, and confirms the residual is now its **irreducible non-defect + documented-limit core**.

## THE EVIDENCE (tools-first, reproduced this session on the released `1.0.149` binary)

Canonical baseline (`ast_pipeline --features "generated_parsers ebnf_dual_run"`, DEFAULT parser paths):

```
PGEN_CERT_COVERAGE_DUMP_ALL=1 ./target/debug/ast_pipeline ../grammars/systemverilog.ebnf \
  --report-certificate-coverage --grammar-profile sv_2017 --entry-rule systemverilog_file \
  --count 40 --seed 0
# CERTIFICATE-COVERAGE: grammar='systemverilog' entry='systemverilog_file' samples=40
#   total=1304 proof=1 witness=1281 UNKNOWN=22 spf=0
# WARNING plannable-rule reach pass: 19 UNKNOWN rules have NO reach path from the entry …
```

Deterministic: seeds 0/7/42 all report `total=1304 proof=1 witness=1281 UNKNOWN=22 spf=0`.

**Two narrowing adjudication runs (the H.12.6 / book mechanism, reproduced at UNKNOWN=22):**

- `--entry-rule sv_multi_entry_root` (the umbrella over the LRM start symbols): `no_path`
  collapses **19 → 8**. The 11 that gain a reach path are EXACTLY the alternate-entry +
  `library_text` cohort — `sv_multi_entry_root`, `systemverilog_parseable_file`,
  `parseable_source_item`, `include_statement`, `library_declaration`, `library_description`,
  `library_text`, `kw_file_path_spec`, `kw_incdir`, `kw_include`, `kw_library` — rooted under
  the LRM's separate `library_text` start symbol (IEEE 1800-2017 §33 / Annex A.1.1). (That run
  reports `spf=1`: the multi-entry umbrella is an analysis root, not a clean parse entry — the
  canonical `systemverilog_file` entry stays `spf=0`.)
- `--grammar-profile sv_2023`: the 6 profile-relative rules — `interface_class_declaration`,
  `interface_class_item`, `interface_class_method`, `declared_interface_class_identifier`,
  `class_constructor_super_args`, `union_modifier` — LEAVE the residual entirely (they witness
  under SV-2023; that run reports `total=1324 witness=1306 UNKNOWN=17 spf=0`, and the residual
  there carries different sv_2023-only `no_path` rules `dot_star`/`kw_function_declaraton`/`kw_n_43`).

## THE ADJUDICATION (22 = 19 non-defects + 3 deferred honest-limits)

| Cohort | Count | Rules | Category | Certifiable via | Verdict |
|---|---|---|---|---|---|
| Entry-relative | 11 | `sv_multi_entry_root`, `systemverilog_parseable_file`, `parseable_source_item`, `include_statement`, `library_declaration`, `library_description`, `library_text`, `kw_file_path_spec`, `kw_incdir`, `kw_include`, `kw_library` | rooted under the `library_text` / parseable-fragment start symbols (A.1.1, §33) | `--entry-rule sv_multi_entry_root` (witnessed there) | NON-defect — STAYS |
| Profile-relative | 6 | `interface_class_declaration`, `interface_class_item`, `interface_class_method`, `declared_interface_class_identifier`, `class_constructor_super_args`, `union_modifier` | genuine IEEE 1800-2023 features, correctly inert under `sv_2017` (the 2017 LRM omits them from the item hierarchy — `H.12.5.2`) | `--grammar-profile sv_2023` (witnessed there) | NON-defect — STAYS |
| Blessed decomposition | 2 | `kw_n_29_7719a1c7`, `kw_n_48_64e095fe` | PGEN-synthetic LRM clause-number decomposition leaves; referenced (6 grep hits in `systemverilog.ebnf`) but unreachable from any canonical entry/profile | n/a (blessed synthetic) | NON-defect — STAYS (no LRM-proven-absence ⇒ never deleted, [[feedback_no_rule_deletion_without_lrm_proof]]) |
| Reach-gap (store) | 1 | `context_member_method_call` | store-gated `head.member[idx].method()` — a witness-reach gap: the generator can't yet synthesise the name-coupled declaration-hosting prelude the `has_fact(variable_binding,$head)` gate needs (NOT a parser defect; `a.b[0].c()` with a declared head parses + witnesses) | deferred (STORE-AWARE-GEN declaration-hosting carrier) | genuine residual — DEFERRED honest-limit |
| Reach-gap (call ambiguity) | 2 | `known_unscoped_class_scoped_call_interface_class_identifier`, `known_unscoped_class_scoped_call_type_parameter_identifier` | a `T::method()` call is ambiguous with a package-scoped call at the expression level *above* the rule, so no carrier disambiguates it; closing them needs a grammar tightening of the generic alternative, deliberately deferred until a *real* parse failure justifies it | deferred (grammar tightening) | genuine residual — DEFERRED honest-limit |

Sum: 11 + 6 + 2 + 1 + 2 = **22** ✓. The **19 `no_path`** are entry/profile-relative or blessed
NON-defects; the **3 reach-gaps** are the genuine canonical-entry residual, each a previously
documented + deliberately deferred honest-limit (no speculative poking — the deferral criterion
is an actual parse failure, not an unwitnessed fragment). **0 deletion candidates.**

## OPEN ENDGAME DECISION (surfaced, not decided — director call)

SV "fully_certified under the canonical `systemverilog_file`/`sv_2017` config" can never report
`UNKNOWN=0` for the 17 entry/profile-relative rules — they are unreachable from that entry/profile
*by LRM design*. So the SV `Done`/`fully_certified` endgame needs a **multi-entry/multi-profile
accounting** (witness the 11 via `sv_multi_entry_root`, the 6 via `sv_2023`, bless the 2), leaving
exactly the **3 deferred reach-gaps** as the only canonical-entry residual. This was first flagged
as "an open endgame decision" in `H.12.5.2`; this slice confirms it is now the *whole* gap. The
recommended path (multi-entry/multi-profile cert accounting + the 3 deferred reach-gaps) is recorded
here and in `LIVE_ACHIEVEMENT_STATUS.md`; it is NOT enacted in this PURE-DOCS slice because changing
the flagship parser's canonical certification config is a director-level decision.

## Acceptance Checklist (PURE-DOCS — code-change boxes N/A)
- [x] **REPRODUCE / ISSUE** — `PGEN_CERT_COVERAGE_DUMP_ALL=1 … --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0` ⇒ `total=1304 proof=1 witness=1281 UNKNOWN=22 spf=0`, deterministic seeds 0/7/42; full 22-rule residual + 19-`no_path` WARNING pasted above.
- [x] **ROOT CAUSE (WHY + WHERE)** — narrowing runs prove the categories: `--entry-rule sv_multi_entry_root` collapses `no_path` 19→8 (the 11 entry-relative rules gain reach); `--grammar-profile sv_2023` drops the 6 profile-relative rules from the residual (`UNKNOWN 22→17` set-diff = exactly those 6). The 2 `kw_n_*` are referenced-but-unreachable blessed leaves; the 3 reach-gaps are the documented honest-limits.
- [x] **FIX** — N/A (PURE-DOCS adjudication; no code/grammar change). The adjudication routes 19 as NON-defects and 3 as deferred honest-limits; 0 deletions per [[feedback_no_rule_deletion_without_lrm_proof]].
- [x] **ADDRESSED (verified)** — SV cert `UNKNOWN=22` is now fully categorized (19 non-defect + 3 deferred), reproduced tools-first at the current count; the book drive-arc lockstep gap (`32 → 22`, 6 slices) is closed in `grammar-wellformedness.md`.
- [x] **NO REGRESSION** — PURE-DOCS ⇒ no code/grammar/generated/release/schema/ledger change ⇒ every deterministic gate inherits the `-0133` green state byte-identical (SV cert `UNKNOWN=22` seeds 0/7/42 `spf=0`; 6 fully-certified grammars `fully_certified=true`; SV external corpus 14/14; `ast_shape_contract` 18/18; `--lint-grammar` clean, 1425 rules). `mdbook_docs_gate` re-run GREEN for the book edit.
- [x] **LOCKSTEP** — top-level book `grammar-wellformedness.md` (new `### Closing the SVA operator layer` subsection: the `32 → 22` arc + the consolidated 22-residual adjudication); tree `GRAMMAR-WELLFORMED.md` frontier (`H.12.7` row); `MEMORY.md` / `CHANGES.md` / `DEVELOPMENT_NOTES.md` / `LIVE_ACHIEVEMENT_STATUS.md`. NO per-parser-book/contract/ledger change (no behaviour change this slice; those were locked-stepped by `-0132`/`-0133`).
