# GRAMMAR-WELLFORMED.H.12.8.4.2 — the `file_path_spec` fix is REFUTED as the blocker; the TRUE root cause is the hardwired cert witness-verification entry

The IMPLEMENT attempt of the `.8.4.1` design (replace the `file_path_spec` lexeme) was carried
out, **measured, and REFUTED**: the fix is correct-by-LRM but **cert-NEUTRAL** (SV union
`UNKNOWN=14` before AND after). A tools-first dig found the real blocker — one shared,
architectural cause for ALL 11 entry-relative rules — and the `file_path_spec` edit was
reverted (cert-neutral flagship-grammar changes do not land; *commit only improvements*).

> Slice `PGEN-GRAMMAR-WELLFORMED-0140` (**PURE-DOCS** — implement attempted, MEASURED, REVERTED;
> NO net code/grammar/generated/release/schema/ledger change committed). Status: `done`.
> Reads with [[feedback_systematically_use_debug_toolbox]], [[feedback_no_codebase_change_without_tool_backed_facts]],
> [[feedback_always_signoff_decisions]], [[feedback_be_alert_root_cause_fishy_immediately]],
> [[feedback_pinpoint_real_blocker_not_menu]], [[feedback_research_grounded_sota_no_trial_and_revert]],
> [[project_cert_coverage_tournament_loser_leak]] (commit only improvements),
> [[project_sv_full_certification_via_multi_entry]].
> Parent: the SV `UNKNOWN`→0 lane `GRAMMAR-WELLFORMED.H.12.8`. **Supersedes the `.8.4.1`
> prediction** (`-0139`) that the `file_path_spec` fix would move the union `14 → ≤ 6`.

## What was done (and measured)

1. Edited `grammars/systemverilog.ebnf:5736` `kw_file_path_spec_c26c9dc9 := trivia /file_path_spec\b/`
   → `trivia /[A-Za-z0-9_.\/?*~$+-]+/` (the `.8.4.1` LRM-faithful path lexeme).
2. Regenerated `generated/systemverilog_parser.rs` + rebuilt both binaries. **Verified the edit took
   effect**: the generated parser no longer contains `/file_path_spec\b/` and the cert probe samples
   changed from the literal (`"include file_path_spec;"`) to real paths (`"include q;"`,
   `"library \foo G0;"`, `"include 3M-3;"`).
3. **Measured the cert (seed 0):** canonical `UNKNOWN=22`, union `UNKNOWN=14` — **byte-identical to
   the pre-fix baseline**. The 6 mechanism-A rules stayed `parsed=false`. **The fix moved nothing.**

## THE TRUE ROOT CAUSE (WHY + WHERE — tool-proven)

The cert witnesses a rule by generating a sample (entry-aware) and **parsing it back through the real
generated parser**, then checking the target rule is in the exercised set. The verification step for
SystemVerilog is hardwired:

- `rust/src/parser_registry.rs:559-574` — `parse_and_cover_systemverilog(sample, grammar_profile)` takes
  **no entry parameter** and always calls `parser.parse_full_systemverilog_file()` (`:571`).
- `generated/systemverilog_parser.rs` exposes exactly **one general `pub fn parse_full_*` entry** —
  `parse_full_systemverilog_file` (plus the specify-path `parse_full_*path*` helpers). The library
  cohort rules are PRIVATE inner fns (`fn parse_include_statement`, `fn parse_library_text`,
  `fn parse_sv_multi_entry_root`, …), reachable only when called from `systemverilog_file`.

Therefore **every** cert witness verification parses from `systemverilog_file`, **regardless of the
cert's `--entry-rule` or `--cert-union-config` entry.** The 11 entry-relative rules are `no_path` from
`systemverilog_file` *by LRM design* (the LRM `library_text` start symbol, IEEE 1800-2017 §33 / Annex
A.1.1, is a SEPARATE entry), so **no input parsed from `systemverilog_file` can ever exercise them** —
they are **un-witnessable by construction**, independent of the grammar lexeme.

**Proof it is the verification entry, not the grammar:** running the cert with `--entry-rule
include_statement` (the target as its own entry) still reports `sample_parse_failures=8/8` — the
generated parser physically cannot parse `include …;` because it only has `parse_full_systemverilog_file`,
and `include` is not a top-level `systemverilog_file` construct. `library_description` is already
COMPLETE (`systemverilog.ebnf:2594-2597` has all 4 LRM alternatives `library_declaration |
include_statement | config_declaration | semi`), so the grammar chain `library_text → include_statement`
is wired — the gap is purely the verification entry.

**Why `.8.1`'s union added 0 from `sv_multi_entry_root:sv_2017`:** the same cause. The union config
changes the GENERATION reach graph (so the rules are *targeted*), but the verification still parses from
`systemverilog_file`, so none of the targeted entry-relative rules can be confirmed. The `-0136`/`-0139`
notes attributed this to "trivial-alt routing + the `file_path_spec` literal"; those are real surface
symptoms, but the *binding* cause is the hardwired single verification entry.

## THE REAL FIX (design for `.8.4.3`+ — parser-AGNOSTIC infrastructure, NOT a grammar edit)

Make cert witness verification honor the configured entry:
1. **Codegen** (`ast_based_generator.rs`): emit `pub fn parse_full_<entry>` for the grammar's alternate
   start symbols (at minimum the rules used as cert `--entry-rule` / `--cert-union-config` entries —
   `library_text`, `systemverilog_parseable_file`, `sv_multi_entry_root`), not only `rule_order[0]`.
   General + parser-agnostic (any grammar with multiple start symbols benefits).
2. **Registry** (`parser_registry.rs`): give `parse_and_cover` an entry parameter and dispatch to the
   matching generated `parse_full_<entry>` (fall back to the default entry when absent), so the cert's
   `--entry-rule` drives BOTH generation and verification.
3. **Then** the `library_text` cohort can be witnessed via union configs `library_text:sv_2017` +
   `systemverilog_parseable_file:sv_2017` (un-shadowed real start symbols), at which point the
   `.8.4.1` `file_path_spec` lexeme fix becomes load-bearing (and gate-exercised) and the
   `library_description` chain witnesses. `sv_multi_entry_root`'s own shadowing (linter: alt#0
   `systemverilog_file` always matches empty, shadowing alts #1/#2) is handled by choosing the
   un-shadowed `library_text`/`systemverilog_parseable_file` entries for the union rather than the
   umbrella.

This is the genuine `.8.4` enabler; `file_path_spec` + `library_description` are necessary follow-ons
that only become *verifiable* once (1)+(2) land. Sequencing: `.8.4.3` design+implement the entry-aware
verification infra (decisive A/B: does the library cohort witness via the new entries?); `.8.4.4`
land `file_path_spec` + measure the union drop; `.8.4.5` the parseable/scaffolding residue.

## A tooling gap this exposed (candidate tool-build)

`parseability_probe --parse <grammar> <file>` has **no `--entry-rule`** — it always parses from the
registered default entry (`systemverilog_file`), so an entry-relative rule cannot be reproduced/traced
in isolation (Step 3 of the `UNKNOWN` protocol is unavailable for these rules). Adding `--entry-rule`
to `parseability_probe` (same dispatch as fix item 2) would make entry-relative rules debuggable. Noted
for the toolbox backlog; folds naturally into `.8.4.3`'s codegen+registry work.

## Acceptance Checklist (PURE-DOCS — implement attempted, measured, reverted)
- [x] **REPRODUCE / ISSUE** — `.8.4.1` predicted union `14 → ≤ 6` from the `file_path_spec` fix; measured union `14` (unchanged) after applying + regenerating + rebuilding (seed 0).
- [x] **ROOT CAUSE (WHY + WHERE)** — `parse_and_cover_systemverilog` (`parser_registry.rs:571`) hardwires verification to `parse_full_systemverilog_file`; the generated parser exposes no `pub fn parse_full_*` for the LRM alternate start symbols ⇒ entry-relative rules un-witnessable regardless of `--entry-rule`. Proven by `--entry-rule include_statement` ⇒ `sample_parse_failures=8/8`; `library_description` confirmed complete (`:2594-2597`).
- [x] **FIX** — N/A net (the `file_path_spec` edit was REVERTED as cert-neutral, per *commit only improvements*). The REAL fix (entry-aware cert verification: codegen `pub fn parse_full_<entry>` + registry dispatch) is designed here for `.8.4.3`.
- [x] **ADDRESSED (verified)** — the lane's binding blocker is now correctly identified (hardwired verification entry, not the grammar); the `.8.4.1` prediction is superseded; the real fix is scoped parser-agnostic.
- [x] **NO REGRESSION** — net no code/grammar/generated/release/schema/ledger change committed (the edit was reverted; `grammars/systemverilog.ebnf` byte-identical to HEAD). Baseline cert canonical `22` / union `14` (seeds 0/7/42 `spf=0`) intact; the 6 fully-certified grammars unaffected. (Local `generated/`/binaries carry the reverted-away lexeme — cert-neutral — and are regenerated by `.8.4.3`.)
- [x] **LOCKSTEP** — this detail file; `.8.4.1` file gets a SUPERSEDED-BY pointer; tree `GRAMMAR-WELLFORMED.md` (`.8.4.2` row recharacterized + Decisions); `MEMORY.md` / `CHANGES.md` / `DEVELOPMENT_NOTES.md`. No book/contract/ledger (no behaviour change). `LIVE_ACHIEVEMENT_STATUS.md` unchanged (SV `Mostly Done`).
