# GRAMMAR-WELLFORMED.H.12.8.4.3 — entry-aware certificate witness verification (the REAL entry-relative enabler, parser-AGNOSTIC)

DESIGN+IMPLEMENT the `-0140`-proven real fix: make certificate witness VERIFICATION honor the
configured entry rule, so the 11 entry-relative SV rules (rooted under the LRM `library_text` /
parseable-fragment start symbols) become witness-able at all. Today every witness is verified by
parsing the generated sample back through `parse_full_systemverilog_file` ONLY
(`parser_registry.rs:571`), so a sample for `include_statement` / `library_text` / … can never be
confirmed regardless of the cert's `--entry-rule` / `--cert-union-config` entry — they are
un-witnessable by construction (`-0140`, tool-proven: `--entry-rule include_statement` ⇒
`sample_parse_failures=8/8`).

> Slice `PGEN-GRAMMAR-WELLFORMED-0141` (**CODE** — codegen + registry + cert plumbing + probe tool).
> Status: `active`.
> Reads with [[feedback_systematically_use_debug_toolbox]], [[feedback_no_codebase_change_without_tool_backed_facts]],
> [[feedback_pinpoint_real_blocker_not_menu]], [[feedback_ast_pipeline_parser_agnostic]],
> [[feedback_never_edit_generated_artifacts]] (the fix is in the codegen, regen the artifact),
> [[project_cert_coverage_tournament_loser_leak]] (commit only improvements),
> [[project_sv_full_certification_via_multi_entry]], [[project_ebnf_is_single_source_of_truth]].
> Parent: the SV `UNKNOWN`→0 lane `GRAMMAR-WELLFORMED.H.12.8`. **Implements the `-0140` REAL fix
> design** (supersedes the `.8.4.1` `file_path_spec`-is-the-blocker prediction).

## Root cause (recap — `-0140`, tool-proven)

- `rust/src/parser_registry.rs:559-575` `parse_and_cover_systemverilog(sample, profile)` takes **no
  entry parameter** and always calls `parser.parse_full_systemverilog_file()` (`:571`).
- The generated SV parser (`generated/systemverilog_parser.rs`) exposes exactly **one** general
  `pub fn parse_full_*` entry: `parse_full_systemverilog_file` (`:4040`, an alias to `parse_full()`
  → `parse()` → `parse_systemverilog_file()`). The `parse_full_*path*` methods are ordinary rules
  whose names start with `full_` (`full_path_description`, `full_path_arrow`, …), NOT entry hooks.
- The alternate start-symbol rule methods DO exist and are already `pub` (`parse_library_text`
  `:397899`, `parse_systemverilog_parseable_file` `:6291`, `parse_sv_multi_entry_root` `:5209`,
  `parse_include_statement` `:353338`) — they are simply never reachable as a *full-input entry*
  because there is no `parse_full_<entry>` wrapper and no entry-aware verification dispatch.

So **every** cert witness verification parses from `systemverilog_file`. The 11 entry-relative rules
are `no_path` from `systemverilog_file` *by LRM design* (the LRM `library_text` start symbol, IEEE
1800-2017 §33 / Annex A.1.1, is a SEPARATE entry), hence un-witnessable until verification honors the
entry. The `-0138` union changes the GENERATION reach graph (the rules get *targeted*) but
verification still parses from `systemverilog_file`, so none can be confirmed.

## The fix (parser-AGNOSTIC infrastructure — NOT a grammar edit)

1. **Codegen** (`rust/src/ast_pipeline/ast_based_generator.rs`, `generate_parse_method`): emit two new
   public methods alongside the existing `parse()` / `parse_full()` / `parse_full_<entry>`:
   - `pub fn parse_from(&mut self, entry: &str) -> ParseResult<ParseNode>` — runs the SAME reset
     ceremony as `parse()` (extracted into a private `prepare_parse_state()` helper, so `parse()`
     stays behavior-identical), then dispatches `entry` → `self.parse_<rule>()` via a `match` with one
     arm per rule in `rule_order` (filtered to rules present in `grammar_tree`, i.e. exactly the rules
     that get a `parse_<rule>` method emitted at `:674`/`:834`), defaulting to the canonical
     `self.parse_<rule_order[0]>()` for an unknown/None entry.
   - `pub fn parse_full_from(&mut self, entry: &str) -> ParseResult<ParseNode>` — wraps `parse_from`
     with the trailing-layout consume + full-input-consumption check (byte-identical to `parse_full`).
   - General + parser-agnostic: any grammar with multiple start symbols gains entry-aware full parse.
     Default-entry path is behavior-identical to today ⇒ inert for the 6 fully-certified grammars.
2. **Registry** (`rust/src/parser_registry.rs`): add an `entry: Option<&str>` parameter to
   `ParseAndCoverFn` (`:101`), the generic `parse_and_cover` (`:517`), and every `parse_and_cover_*`
   closure. Each closure dispatches `entry` via `parse_full_from(entry)` (falling back to the default
   entry when `entry` is `None` or unrecognized) so the cert's entry drives BOTH generation and
   verification. Non-SV grammars route through the same `parse_full_from`; with `entry=None` they are
   byte-identical to today.
3. **Cert plumbing** (`rust/src/main.rs`): thread the active entry into the 6 `parse_and_cover(...)`
   call sites (`:2476..2769`). The canonical pass uses the generation `entry_rule`; the
   `--cert-union-config` passes use their own config entry (so a `library_text:sv_2017` union config
   verifies from `parse_full_library_text`).
4. **Tool gap (split to `.8.4.3.1`)** — adding `--entry-rule <RULE>` to `parseability_probe --parse`
   / `--parse-dump-ast*` (so an entry-relative rule can be reproduced/traced in isolation, Step 3 of
   the UNKNOWN protocol) is the SAME `parse_full_from` dispatch but a parallel plumbing change to the
   detail-parse path (`ParseDetailFn` + the 7 `parse_with_*_detail` closures + a release rebuild). To
   keep `.8.4.3` a single-purpose, reviewable commit (the cert A/B — the actual proof — does NOT use
   the probe), the probe flag is carved out into sub-leaf `.8.4.3.1` (a tool-build slice landed next).

## Decisive A/B (the acceptance proof)

After the fix + regen, run the SV cert with the library-cohort union configs:
```
ast_pipeline grammars/systemverilog.ebnf --report-certificate-coverage \
  --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0 \
  --cert-union-config systemverilog_file:sv_2023 \
  --cert-union-config library_text:sv_2017 \
  --cert-union-config systemverilog_parseable_file:sv_2017
```
**Expectation:** at least one entry-relative rule that was un-witnessable before (e.g.
`systemverilog_parseable_file` / `parseable_source_item` / `sv_multi_entry_root`, which do NOT depend
on the `file_path_spec` lexeme) now WITNESSES — i.e. the union `UNKNOWN` drops below `14`. Some of the
11 (`include_statement`, `kw_include`, `kw_incdir`, `kw_library`, `kw_file_path_spec`,
`library_declaration`, and the `library_text`/`library_description` rules that require a path token)
remain blocked by the `file_path_spec` LRM-extraction artifact — those close in `.8.4.4` once this
infra makes the `file_path_spec` lexeme fix load-bearing. The PROOF of `.8.4.3` is: entry-aware
verification turns ≥1 previously-un-witnessable entry-relative rule from UNKNOWN → witnessed
(infra correct), with NO regression to the canonical `22` or the 6 fully-certified grammars.

## Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — baseline (this session, freshly-regenerated SV at baseline grammar): canonical `CERTIFICATE-COVERAGE: … UNKNOWN=22`, union (`systemverilog_file:sv_2023` + `sv_multi_entry_root:sv_2017`) `CERTIFICATE-COVERAGE-UNION: … UNKNOWN=14`, `spf=0`, seed 0. (`-0140`: `--entry-rule include_statement` ⇒ `sample_parse_failures=8/8` — un-witnessable.)
- [x] **ROOT CAUSE (WHY + WHERE)** — `parse_and_cover_systemverilog` (`parser_registry.rs:571`) hardwired verification to `parser.parse_full_systemverilog_file()`; the codegen emitted exactly one `pub fn parse_full_<entry>` (for `rule_order[0]`), so alternate LRM start symbols (`library_text` `:397899`, `systemverilog_parseable_file` `:6291`, …) had only private rule methods ⇒ entry-relative rules un-witnessable regardless of `--entry-rule`. (`-0140`.)
- [x] **FIX** — codegen `ast_based_generator.rs::generate_parse_method` emits `parse_from(entry)`/`parse_full_from(entry)` (a per-rule `match`, default arm = canonical; reset extracted to `prepare_parse_state()`) — engine-tier infra in the GENERATOR, NOT a hand-edit of the artifact; `parser_registry.rs` `ParseAndCoverFn` + 7 closures + generic `parse_and_cover` take `entry: Option<&str>`; `main.rs` threads the per-config entry into the 6 cert sites. Fix-hierarchy: a parser-agnostic generator capability, no grammar edit. (`parseability_probe --entry-rule` split to `.8.4.3.1`.)
- [x] **ADDRESSED (verified)** — SV union adding `--cert-union-config library_text:sv_2017` + `--cert-union-config systemverilog_parseable_file:sv_2017` ⇒ `CERTIFICATE-COVERAGE-UNION: … witness=1300 UNKNOWN=3` — **`14 → 3`**, all 11 entry-relative rules witness. **Deterministic at seeds 0/7/42** (canonical `22`, union `3`). `PGEN_CERT_COVERAGE_DUMP_ALL=1` confirms the union residual is EXACTLY the 3 `.8.3` reach-gaps (`context_member_method_call`, `known_unscoped_class_scoped_call_interface_class_identifier`, `known_unscoped_class_scoped_call_type_parameter_identifier`) — sound, not false-witness inflation.
- [x] **NO REGRESSION** — SV canonical byte-identical `CERTIFICATE-COVERAGE: … UNKNOWN=22 (spf=0)` at seeds 0/7/42; the **6 fully-certified grammars byte-identical `fully_certified=true UNKNOWN=0`** (json 9/9, regex 198/198, vhdl 216/216, svpp 74/74, rtl_frontend 169(proof1)/168, rtl_const_expr 48/48 @ `--max-depth 32`); `cargo test --lib --features "generated_parsers ebnf_dual_run"` = `771 passed; 0 failed` (incl. the `-0138` union soundness lock); `clippy_on_rust_change` source stage clean (stage 1 passed; generated-stage warnings are pre-existing, non-strict); SV external corpus triage `parse_fail_total=0` (14/14).
- [ ] **LOCKSTEP** — `grammar-wellformedness.md` (the endgame-accounting paragraph: entry-aware verification capability + the new union figure), `parser-hooks.md` (`parse_from`/`parse_full_from` entry-aware methods); SV integration contract + ledger only if the union figure becomes a consumer-facing claim; `MEMORY.md` / `CHANGES.md` / `DEVELOPMENT_NOTES.md` / `LIVE_ACHIEVEMENT_STATUS.md`. NO grammar/release/schema change (infra only; SV stays `Mostly Done`). (`cli-and-workflows.md` / `parseability-probe-debug.md` `--entry-rule` docs land with `.8.4.3.1`.)

## Notes
- This is the `.8.4` enabler. `.8.4.4` (land `file_path_spec` + measure the remaining union drop) and
  `.8.4.5` (parseable scaffolding residue) follow.
- The 6 fully-certified grammars never pass an alternate entry ⇒ `entry=None` ⇒ default path ⇒
  byte-identical. Inertness is provable by construction.
