# GRAMMAR-WELLFORMED.H.12.8.4.4 — land the LRM-faithful `file_path_spec` path lexeme (released-SV LRM-fidelity fix)

Replace the `file_path_spec` **literal-keyword extraction artifact** (`/file_path_spec\b/`) with an
LRM-faithful **file-system path lexeme**, so the `include`/`library` library-cohort productions accept
real §33 file paths (`../rtl/cpu.v`, `/path/to/*.sv`, `./src/*.sv`) instead of only the literal word
`file_path_spec`. This is the `.8.4` follow-up that `-0141` retained after `-0140` proved the change is
**cert-neutral** (NOT the witnessing blocker — that was the hardwired verification entry, fixed in
`-0141`): it is a genuine **LRM-fidelity / well-formedness correctness** fix, and it makes the existing
library-cohort cert witnesses HONEST (today they witness on a literal-keyword interpretation; after, on
a real path lexeme — removing a false-confidence witness, directly serving "Verified, not trusted").

> Slice `PGEN-GRAMMAR-WELLFORMED-0143` (**CODE** — released-SV grammar edit; single token regex,
> rule name/arity preserved ⇒ AST-shape-neutral / schema unchanged). Status: `done`.
> Reads with [[feedback_systematically_use_debug_toolbox]], [[feedback_why_and_where_before_solution]],
> [[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_no_rule_deletion_without_lrm_proof]],
> [[feedback_corpus_expected_from_spec_not_fix]] (expecteds derived from IEEE §33, independent of the fix),
> [[project_ebnf_is_single_source_of_truth]], [[feedback_regex_book_live]], [[feedback_always_signoff_decisions]].
> Parent: `GRAMMAR-WELLFORMED.H.12.8.4` design (`-0139`); enabler `.8.4.3` (`-0141`, entry-aware cert
> verification) which reframed this from witnessing-blocker to LRM-fidelity cleanup.

## Root cause (WHY + WHERE — tool-proven, reproduced this session)

`grammars/systemverilog.ebnf:5736` — `kw_file_path_spec_c26c9dc9 := trivia /file_path_spec\b/` — matches
the **literal word** `file_path_spec`. Referenced by exactly two productions, both library-cohort:
`include_statement` (`:2313`) and `library_declaration` (`:2591`), reachable only from `library_text`
(`:2601`), NOT from the `systemverilog_file` embedding entry. IEEE 1800-2017 §33.3.1 / Annex A.1.1
(independently read: `docs/systemverilog/2017/txt/section-33-configuring-the-contents-of-a-design.txt:117-160`)
defines `file_path_spec` as a **file-system path token** — *"file-system-specific notation to specify an
absolute or relative path to a particular file or set of files"* with wildcards `?` (single char),
`*` (multi char), `...` (hierarchical), `..`/`.` and `/` separators; examples `*.v`, `./*.vg`,
`library rtlLib *.v;` — with **no formal lexical production** (described textually). So the literal
`/file_path_spec\b/` is an LRM-extraction artifact (same class as `kw_n_29`/`kw_n_48`).

**Decisive BEFORE-state** (release `parseability_probe --entry-rule library_text`, this session):

| fixture | input | before |
|---|---|---|
| `lib_literal`  | `include file_path_spec;`                            | **PASS** (artifact accepts only the literal word) |
| `lib_relpath`  | `include ../rtl/cpu.v;`                              | **REJECT** ← valid §33 path rejected (the bug) |
| `lib_libdecl`  | `library mylib /path/to/*.sv;`                      | **REJECT** ← valid §33 path rejected (the bug) |
| `lib_incdir`   | `library rtl ./src/*.sv, ./pkg/*.sv -incdir ./inc;` | **REJECT** ← valid §33 path rejected (the bug) |

## The fix (minimal, LRM-faithful, AST-shape-neutral)

`systemverilog.ebnf:5736` → `kw_file_path_spec_c26c9dc9 := trivia /[A-Za-z0-9_.\/?*~$+]+/`

- Covers the §33 path alphabet (letters, digits, `_`, `.`, `/` (escaped `\/`), wildcards `?`/`*`) plus
  safe common file-system path chars (`~` home, `$` env, `+`); still matches the literal `file_path_spec`
  (so existing witnesses keep parsing) and every LRM example (`*.v`, `./*.vg`, `../rtl/cpu.v`, `/path/to/*.sv`).
- Stops at the cohort's structural delimiters — whitespace, `comma` (`,`), `semi` (`;`) — none in the class.
- **Excludes `-` (tool-grounded refinement of the `-0139` design).** The `kw_incdir` cert witness uses a
  GLUED forced sample (`…file_path_spec-incdir…`); a class containing `-` would greedily swallow `-incdir`
  and regress that witness. The LRM path examples need no `-`, so excluding it is both LRM-faithful and
  boundary-safe (the path stops at `-`, `minus`+`kw_incdir` then match). Documented limitation: a
  hyphenated filename (`uvm-1.2`) is not accepted by this lexeme (no LRM example uses one; a future
  negative-lookahead extension could admit `-` without the `-incdir` collision — out of scope here).
- Excludes `\` (no `escaped_identifier` collision) and `"` (no quoted form per the LRM).
- Rule **name + arity unchanged** (single token, positional `$1` references in `include_statement` /
  `library_declaration` unchanged) ⇒ typed carrier / AST-dump schema unchanged.

Fix-hierarchy: declarative grammar lexeme correction (the EBNF is the single source of truth); no engine
change, no rule deletion (the rule is referenced and LRM-mandated — it must accept a path, not a literal).

## Verification plan (regen + rebuild, then measure)

1. Edit `:5736`; `make focus_systemverilog` (regen `generated/systemverilog_parser.rs`); rebuild
   `ast_pipeline` (debug, `generated_parsers ebnf_dual_run`) + `parseability_probe` (release, `generated_parsers`).
2. AFTER-state: the 3 REJECT fixtures ⇒ PASS; `lib_literal` still PASS (REJECT→PASS oracle, expecteds from §33).
3. NO REGRESSION: SV cert canonical `UNKNOWN=22` + the library-union `UNKNOWN=3` byte-identical at seeds
   0/7/42 (`spf=0`) — critically `kw_incdir` still witnesses (the `-` exclusion preserves it); the 6
   fully-certified grammars byte-identical `fully_certified=true`; SV external corpus 14/14;
   `ast_shape_contract` GREEN; `--lint-grammar` clean; `clippy_on_rust_change` source-clean.
4. Released-SV ceremony: release `1.0.149 → 1.0.150`, schema stays `6` (strictly-more-permissive accept
   change, AST-shape-neutral — the `SV-0008`/`SV-0010`/`SV-0011` category); ledger `SV-0012`; per-parser SV
   book + integration contract + top-level book `grammar-wellformedness.md` lockstep.

## Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — release probe `--entry-rule library_text` BEFORE-state table above: `include ../rtl/cpu.v;` / `library mylib /path/to/*.sv;` / `library rtl ./src/*.sv, ./pkg/*.sv -incdir ./inc;` all REJECT; only the literal `include file_path_spec;` PASSes. The released SV grammar rejects valid IEEE 1800-2017 §33 file paths.
- [x] **ROOT CAUSE (WHY + WHERE)** — `systemverilog.ebnf:5736` `kw_file_path_spec_c26c9dc9 := trivia /file_path_spec\b/` is a literal-keyword LRM-extraction artifact; §33.3.1/Annex A.1.1 (read independently, `section-33-…txt:117-160`) defines `file_path_spec` as a file-system path token (wildcards `?`/`*`/`...`, `/` separators), not a keyword. Referenced only by `include_statement` (`:2313`) / `library_declaration` (`:2591`) — library cohort, unreachable from `systemverilog_file`.
- [x] **FIX** — `:5736` → `kw_file_path_spec_c26c9dc9 := trivia /[A-Za-z0-9_.\/?*~$+]+/` (declarative grammar lexeme; name/arity preserved ⇒ schema-neutral; `-` deliberately excluded for `-incdir` witness boundary safety).
- [x] **ADDRESSED (verified)** — AFTER regen (`make focus_systemverilog`) + rebuild of both binaries, the freshly-built release `parseability_probe --parse … --profile 2017 --entry-rule library_text`: `include ../rtl/cpu.v;` ⇒ **PASS (rc0)** (was REJECT), `library mylib /path/to/*.sv;` ⇒ **PASS** (was REJECT), `library rtl ./src/*.sv, ./pkg/*.sv -incdir ./inc;` ⇒ **PASS** (was REJECT — confirms the `-` exclusion kept the `-incdir` boundary), and `include file_path_spec;` ⇒ **PASS** (literal still accepted — no witness regression). REJECT→PASS oracle green; expecteds derived from IEEE 1800-2017 §33 independent of the fix.
- [x] **NO REGRESSION** — SV canonical cert `total=1304 proof=1 witness=1281 UNKNOWN=22 (spf=0)` **byte-identical at seeds 0/7/42**. SV complete 4-config union (the book-canonical command, incl. `sv_multi_entry_root:sv_2017`) `witness=1300 UNKNOWN=3` **deterministic seeds 0/7/42**, residual = EXACTLY the 3 `.8.3` reach-gaps (`context_member_method_call` + the two `…scoped_call…` cousins) — byte-identical to the `-0141` baseline; the entire library cohort incl. `kw_incdir` still witnesses. (NOTE: a 3-config set that omits `sv_multi_entry_root:sv_2017` reports union `UNKNOWN=4` because that parent umbrella is `no_path` from all three child entries and can only witness from its own entry config — structural + pre-existing, not introduced by this edit; the book's documented command uses the complete 4-config set.) regex 198/198 + vhdl 216/216 `fully_certified=true` (json/svpp/rtl_frontend/rtl_const_expr inert by construction — only `generated/systemverilog_parser.rs` regenerated); `ast_shape_contract` **18/18 GREEN**; SV external corpus **14/14** (`parse_fail_total=0`, `preprocess_fail_total=0`); `--lint-grammar` clean (`1425` rules, `non_terminating=0`, `unreachable_rules=0`, `ordered_choice_shadowing=0`, `profile_orphans=0`, pre-existing `always_matches=8` A2 backlog unchanged); `clippy_on_rust_change` source-clean (generated-stage non-strict, baseline); `cargo test --lib --features "generated_parsers ebnf_dual_run"` **771 passed / 0 failed / 21 ignored**.
- [x] **LOCKSTEP** — released-SV ceremony: parser/contract release `1.0.149 → 1.0.150`, AST-dump schema stays `6` (strictly-more-permissive accept change, AST-shape-neutral — the `SV-0008`/`SV-0010`/`SV-0011` category); ledger `SV-0012`; top-level book `grammar-wellformedness.md` (the `file_path_spec` literal-artifact note reframed to "fixed"); SV per-parser book `changelog-index.md` (1.0.150 row); `CHANGES.md` / `DEVELOPMENT_NOTES.md` / `LIVE_ACHIEVEMENT_STATUS.md` / `MEMORY.md`. NO `ast_shape_contract` manifest change (schema-neutral).
