# The regex out-of-band validator is NOT deletable yet — 8 check families are still LOAD-BEARING

- Category: `project`
- Created: `2026-07-09` (session #72)
- Slice: `PGEN-REGEX-PCRE2-0022` (`REGEX-PCRE2-FIDELITY.4` SCOPING, PURE-DOCS)
- Related: [[project_regex_pcre2_faithful_by_default]] · [[project_ebnf_is_single_source_of_truth]] ·
  [[feedback_be_alert_root_cause_fishy_immediately]] · [[feedback_report_expected_verify_against_oracle]]

## Context

The `REGEX-PCRE2-FIDELITY` tree's capstone `.4` is "delete `validate_regex_compile_contract`
(the out-of-band PCRE2 acceptance validator) — the EBNF becomes the single source of truth."
The `MEMORY.md` resume pointer (through session #71) framed the next action as *"delete the
residual validator — **only the start-option POSITION rule remains validator-owned**"*, implying
the validator was all-but-dead.

On resuming (session #72), a tools-first check of the ACTUAL code found `validate_regex_compile_contract`
still dispatches **eight** live checks (`parser_registry.rs:393-394` runs the grammar parse THEN the
validator). That "only start-option position remains" claim needed verification, not trust.

## Decision / finding (tool-backed, definitive)

**Method.** The parse path runs `parse_full_regex()` (grammar) FIRST, then the validator ONLY if the
grammar accepted. So the rejection **message names the source**: `"Parser did not consume full input"`
= the GRAMMAR rejected (validator shadowed / dead for that input); any validator "…compile contract"
string = the grammar ACCEPTED and only the validator rejected (validator **load-bearing** — deleting
it would make that input ACCEPT = an accepts-invalid PCRE2 divergence). I ran the validator's own
pinned reject corpus (its `#[test]` `expect_err` inputs, 58 cases) through the current release
`parseability_probe --parse regex --profile pcre2` and classified each by message source.

**Result: ALL 8 checks are LOAD-BEARING. 54 / 58 reject inputs are uniquely owned by the validator**
(the grammar accepts them; only the validator rejects). Only **4** inputs are grammar-shadowed — the
empty-`\Q\E`-region / orphan-`\E` char-class cases already closed by `.3.15`/`.3.23`. Deterministic
(stable re-runs). So deleting the validator today would silently re-admit **54 PCRE2-invalid patterns**.

| # | Validator check | Owns (still validator-only) | Load-bearing inputs | Encodability |
|---|---|---|---|---|
| 1 | `find_invalid_property_escape` | bare `\p`/`\P` must be a 1-letter category; `\p` at EOF | 3 | **structural** (easy) |
| 2 | `find_invalid_named_escape_or_group_name` | `\k`/group NAME charset validity + length ≤ 128 | 6 | charset **structural**; len ≤128 = `len_bounds` predicate |
| 3 | `find_invalid_counted_quantifier` | `{N,M}` min>max ORDER (err 104) | 2 | **value-comparison** (leading-zero-hostile; `.3.18`-deferred) |
| 4 | `find_invalid_char_class_construct` | `\B`/`\K`/`\N`-in-class; nonliteral range ends; DESCENDING ranges; unknown POSIX name; mixed `[[:<:]]` alias | 27 | mixed: escapes/POSIX = **structural**; ranges = **value-comparison over decoded codepoints** (hard) |
| 5 | `find_invalid_scan_substring_capture_list` | `(*scs:(N)/(<name>))` must reference an AVAILABLE capture | 5 | **whole-pattern capture inventory** two-pass (store-aware, like `.3.22`) |
| 6 | `find_invalid_verb_construct` | start-option `(*UTF)`… POSITION (must be a start-option prefix) | 3 | **whole-pattern** two-pass (the known `.4` target + duality pin) |
| 7 | `find_unbounded_quantified_lookbehind` | variable-length lookbehind must be bounded | 5 | **lookbehind-length analysis** (hard; needs a new primitive) |
| 8 | `find_invalid_keep_out_escape_in_lookaround` | `\K` inside a lookaround | 3 | **contextual** (the original `.1` #10 "hard one"; needs a primitive) |

## Why the "only start-option remains" belief was wrong — two different axes

- **Generator↔parser DUALITY** (hunter-visible, `duality_hunt_gate`): clean except start-option-position
  (the generator, being store-aware/faithful, never GENERATES these invalid forms). This is the true
  statement that got over-generalized.
- **Validator LOAD-BEARING-ness** (this probe): whether the GRAMMAR ALONE accepts a *hand-written*
  invalid pattern. 8 families still do. A downstream user (Nexsim/RGX) can hand-write `[z-a]`,
  `(?<=a+)b`, `(?=a\Kb)`, `x{5,4}`, `\pA`, `(*scs:(1))`, `a(*CR)b`, `\k<>` — all accepted by the
  grammar, all rejected by PCRE2, today caught ONLY by the validator.

"Duality-clean" ≠ "validator-deletable." The capstone is about deletion, so it needs the grammar to
reject all 54, not just the duality-clean subset.

## Consequences

- **`.4` is NOT a single "delete" slice.** It is re-scoped into child leaves `.4.1`..`.4.11` (one per
  family; see the `REGEX-PCRE2-FIDELITY.4` SCOPING LOG), each a released encode-in-EBNF slice, THEN a
  final `.4` deletion once all land. Several (`.4.5` descending ranges, `.4.7` scs inventory, `.4.9`
  lookbehind length, `.4.10` `\K`-in-lookaround, plus the `.3.22` named-reference inventory) plausibly
  need a NEW parser-agnostic annotation primitive (value-constraint over decoded ranges / whole-pattern
  two-pass / contextual gate), NOT a plain structural gate — the same class flagged as hard back in the
  `.1` scoping table (rows 9/10) and the `.3.22`/`.3.18` deferrals.
- **Recommended order** (cheapest, purely-structural, lowest-risk first, so the validator shrinks
  monotonically with released proof): `.4.1` (bare property) → `.4.6` (POSIX names + alias) →
  `.4.4` (`\B`/`\K`/`\N`-in-class) → `.4.2` (name charset + length) → then the hard/primitive-needing
  families as their own design+build slices.
- Anything claiming "the validator is gone / nearly gone" must be re-derived by this message-source
  probe, never asserted.
