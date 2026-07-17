# The Compile-Contract Validator

> **Consumer takeaway up front.** Regex acceptance is
> `accepted = (what the grammar accepts) ∩ (what the compile-contract validator accepts)`.
> Both layers reject with the **same error code** (`E_PARSE_FAILURE`); only the message
> text differs. So **match on the code, never on the message** — and treat *which* layer
> rejected a given pattern as an internal detail that shifts release-to-release as checks
> migrate from the validator into the grammar.

## What it is

The `regex` parser is the **one** PGEN family that layers an out-of-band acceptance
check on top of its generated parser. After the generated parser accepts an input, the
registry runs a second pass — `validate_regex_compile_contract`
(`rust/src/regex_compile_validation.rs`) — that rejects a small set of patterns which
**PCRE2 rejects at compile time but the grammar, at the current release, still
structurally accepts**. If the validator finds a violation, the overall parse fails.

Concretely, `rust/src/parser_registry.rs` runs the two layers **in series**:

```
parse_full_regex(input)      →  the generated parser (grammars/regex.ebnf → regex_parser.rs)
        │  accepted?
        ▼
validate_regex_compile_contract(input)   →  the out-of-band PCRE2 compile contract
```

The validator is only consulted **after** the grammar accepts, so a pattern the grammar
already rejects never reaches it.

## Why it exists (and why it is shrinking)

PGEN's north-star doctrine is that **the EBNF grammar is the single source of truth for
the accepted language** (see the platform book's *Quality & Closure Model → THE EBNF IS
THE SINGLE SOURCE OF TRUTH*). An acceptance constraint that lives *outside* the EBNF is
invisible to the stimuli generator, which derives its samples from the grammar alone — a
defect class, because the generator can emit structurally-valid samples the parser then
rejects.

The compile-contract validator is therefore **transitional**: every check in it is a
PCRE2 rule that has not *yet* been expressed in `regex.ebnf`. The `REGEX-PCRE2-FIDELITY`
task tree is migrating them into the grammar one family at a time; each migration is a
**behavior-neutral** move — the accept/reject *set* is unchanged (proven byte-identical
against the `pcre2test` oracle, `regex_pcre2_compile_oracle_gate`), only the reject's
source (and message) moves from the validator into the grammar. When the last family is
grammar-owned, the capstone (`REGEX-PCRE2-FIDELITY.4`) deletes the module entirely and
the EBNF becomes the sole source of truth.

A mechanical gate (`scripts/check_ebnf_source_of_truth.sh`, in the pre-commit hook and
CI) forbids any *new* out-of-band validator from being wired into the registry, so this
one instance is the only one and it can only ever shrink.

## The current conformance snapshot

The oracle gate (`regex_pcre2_compile_oracle_gate`, `pcre2test` 10.47) executes a frozen
2189-cell corpus and asserts **exact** case counts but only **bounds** on the
match/divergence split — so the gate stays green while the real tuple *improves* as
fidelity fixes land. The real tuple (cells / oracle matches / tracked divergences /
skips) has evolved `2189/1858/285/46` → `2189/1867/274/48` → `2189/1871/270/48` →
**`2189/1879/262/48`** (current as of release `1.1.104`; the last step is the
named-reference acceptance fix, ledger `REGEX-0098`). The per-release notes further down
this chapter cite the tuple **at their own era** — that is deliberate historical
accuracy, not drift; only the value in this paragraph is maintained as current
(`REGEX-PCRE2-FIDELITY.DOCSYNC.1`, 2026-07-17).

## The remaining check families

As of regex release `1.1.104`, the validator dispatches these families (each row is a
`find_*` check in `regex_compile_validation.rs`, and each maps to the
`REGEX-PCRE2-FIDELITY` leaf that will migrate it into the grammar):

| Family (what it rejects) | Example load-bearing inputs | Migration leaf |
|---|---|---|
| Character-class **RANGE** validity — nonliteral endpoints + descending ranges (**nonliteral shorthand/property endpoints — `.4.5.b`; descending literal ranges with a DECODABLE non-whitespace endpoint — `.4.5.c`; POSIX-class endpoints — `.4.5.d`; collating/equivalence bracket-token endpoints — `.4.12` — now ALL grammar-owned**; only quoted/`\u{}`/bare-whitespace-endpoint descending stays validator-owned) | `[\Q..\E-a]`/`[a- ]` (deferred blocked-descending endpoints) | blocked descending |

> **`1.1.95` (REGEX-0105, `.4.5.a`) — a range-validity *correctness fix*, not a migration.** A range whose
> right endpoint began with `[` (or `||`) inside a NORMAL class — `[a-[b]]`, `[x-[:alpha:]]`, `[~-||]`,
> `[\d-[z]]` — was **accepts-invalid**: the validator's `scan_char_class` gated range detection on a guard
> (`dash_starts_alt_extended_class_operator`) meant for the ALTERNATE extended-class syntax `(?[...])`, but
> `scan_char_class` only ever runs on normal classes, so the guard was mis-scoped and skipped the check. The
> guard was removed and the range right-endpoint reader now classifies a `[:..:]`/`[...]`/`[=..=]` bracket
> token NON-LITERAL. The class-range family **stays validator-owned** (the grammar migration is the deferred
> `.4.5.b`/`.4.5.c`); this release just makes the existing check correct. A **separate** newly-found divergence
> — STANDALONE collating `[[.a.]]` / equivalence `[[=a=]]` members (PCRE2 err 113, PGEN accepts) — is tracked
> as `.4.12` and NOT addressed here.
>
> **`1.1.96` (REGEX-0106, `.4.5.a.1`) — a SECOND range-validity *correctness fix*.** The vertical-whitespace
> shorthands `\v` / `\V` were treated as LITERAL range endpoints, so `[\v-x]`, `[\V-x]`, `[a-\v]` were
> **accepts-invalid** (PCRE2 err 150). Root cause, two halves: the validator's `is_nonliteral_class_escape`
> omitted `v` / `V` from the nonliteral set (so they decoded to literal `118` / `86`), and the grammar's
> `class_range_literal_escape_letter_strict` listed `v` / `V` (so they formed a range endpoint, and the
> generator could emit `\v`-ranges). The fix makes `\v` / `\V` byte-identical to their already-correct
> horizontal analogs `\h` / `\H`: dropped from the grammar range-letter set (members via `class_simple_escape`
> untouched; duality-safe) and added to `is_nonliteral_class_escape` (err-150 reject). Members `[\v]` / `[\V]`
> and hex/octal endpoints of the same code point (`[\x0b-\x0c]`) stay valid. The class-range family **stays
> validator-owned** pending `.4.5.b`/`.4.5.c`. Oracle byte-identical (`2189/1867/274/48` — corpus-invisible,
> proven by the direct `pcre2test` sweep + the unit pin).
>
> **`1.1.97` (REGEX-0107, `.4.5.b`) — a behavior-NEUTRAL validator→grammar migration (NOT a correctness
> fix).** The nonliteral-shorthand/property half of the range family is now GRAMMAR-owned: a class range
> whose LEFT or RIGHT endpoint is a shorthand (`\d \D \h \H \s \S \v \V \w \W`) or property (`\p…` / `\P…`)
> escape — `[\d-x]`, `[a-\d]`, `[\p{Lu}-x]`, `[a-\p{Lu}]` — is rejected by a new zero-width negative
> lookahead `!invalid_class_range` on the three class-item positions. The GRAMMAR previously ACCEPTED these
> (splitting `[\d-x]` into members `\d`, `-`, `x`, because `class_atom` excludes the shorthand/property
> escapes) and only THIS validator rejected them; the reject is now encoded in the EBNF. Because the
> validator still ALSO rejects them (this row's check is NOT yet deleted — the deletion waits until the whole
> family is grammar-owned, `.4.5.c` + the `-[`/`\v\V` cases), the released verdict is unchanged; the
> migration is observable only at the grammar layer (the certified interpreter). The literal-dash carve-outs
> (`[a-]`, `[-a]`, `[\d-]`, `[\d\-x]`, `[\da-z]`, `[-\d]`) and valid ranges/members stay ACCEPT. Cert 239→245
> (+3 witnessed `*_core` rules, +3 lookahead-only PROOF rules), oracle byte-identical, duality unchanged.
>
> **`1.1.98` (REGEX-0108, `.4.5.c`) — a behavior-NEUTRAL validator→grammar migration (the descending-literal
> sibling of REGEX-0107).** A class range of two LITERAL endpoints whose left code point exceeds the right —
> `[z-a]`, `[9-0]`, `[\x39-\x30]`, `[\x{100}-a]` (PCRE2 err 108) — is now rejected at the GRAMMAR layer by a
> THIRD `!invalid_class_range` alternative `descending_class_range`, gated by the new `value_compare_codepoint`
> `@predicate` (RULE-SPAN-VALUE-CONSTRAINT.3): it decodes each RAW endpoint spelling (bare non-whitespace
> literal + `\xHH`/`\x{}`/`\NNN`/`\o{}`/`\cX`/named escapes) to its Unicode code point and matches when left >
> right. The braced `[\x{100}-a]` is the code-point discriminator — a textual compare (`\`=92 < `a`=97) would
> wrongly ACCEPT; only decoding (256 > 97) rejects. Endpoints that cannot be reliably decoded stay
> validator-owned: `\Q..\E`/`\u{}` (decode `None`) and a BARE WHITESPACE endpoint (whose raw `$text` reaches
> the predicate EMPTY — a latent pipeline finding recorded in the decisions log), so ascending whitespace
> ranges (`[ -!]`, `[ --]`, `[\t-a]`) keep ACCEPTing. Because the validator still ALSO rejects descending
> ranges (this row's check is NOT yet deleted — the deletion waits for `.4.5.d`), the released verdict is
> unchanged; observable only at the grammar layer. Cert 245→249 (+4 lookahead-only PROOF rules), oracle
> byte-identical `2189/1867/274/48` (an in-slice bare-whitespace over-block regression `48→50` was
> root-caused + fixed before release), duality unchanged.
>
> **`1.1.99` (REGEX-0109, `.4.5.d`) — MOSTLY a behavior-NEUTRAL validator→grammar migration, PLUS a
> PCRE2-convergent correctness fix for one subset (the POSIX-endpoint sibling of REGEX-0107/0108).** A class
> range whose LEFT or RIGHT endpoint is a valid POSIX class (`[:name:]`) — `[[:alpha:]-z]` (POSIX left),
> `[!-[:alpha:]]` (POSIX right, ascending) — is PCRE2 err 150 and is now rejected at the GRAMMAR layer by two
> new `!invalid_class_range` alternatives (`posix_class - class_atom` / `class_atom - posix_class`) that
> reference the EXISTING positively-reachable `posix_class` rule. The GRAMMAR previously ACCEPTED these (reading
> the bracket's `[` as a `class_safe_special` literal `0x5B`). Because `posix_class` is already positively
> entered, NO new rule is added and cert `total` is unchanged (249) — unlike `.4.5.b`/`.4.5.c`. The
> `class_atom`-after-`-` requirement keeps the trailing-dash carve-out valid (`[[:alpha:]-]` and the zero-width
> `[[:alpha:]-\Q\E]` ACCEPT — PCRE2-verified). **This row's check has an accepts-invalid HOLE the migration
> exposed:** `dash_is_trailing_literal` (used by both `scan_char_class` range branches) skips whitespace/`\Q\E`
> zero-width after the dash and returns "trailing literal dash" even when the range's LEFT endpoint is a
> NonLiteral POSIX class — so the validator ACCEPTED `[[:digit:]-   ]` (dash + only whitespace before `]`) where
> PCRE2 rejects err 150. The grammar migration flips that subset **ACCEPT→REJECT** at `--parse`
> (PCRE2-convergent; one contract success sample moved to failure, counts 92/26). For every other posix-endpoint
> case the validator already rejected, so those stay behavior-neutral. When this range-check is eventually
> deleted (once the blocked-descending endpoints land — collating/equivalence is now grammar-owned by `.4.12`),
> the `dash_is_trailing_literal` whitespace-skip hole must be resolved validator-side too, or fully subsumed by
> the grammar (the grammar already handles it correctly). Only VALID-POSIX-name endpoints migrate here; the
> collating (`[.a.]`) / equivalence (`[=a=]`) endpoints are handled by the `.4.12` `class_bracket_token`
> recognizer (below). Cert 249 UNCHANGED, ast_shape 225 UNCHANGED (no new `->` shape), oracle byte-identical
> `2189/1867/274/48` (flip cells corpus-invisible), duality unchanged.
>
> **`1.1.100` (REGEX-0110, `.4.12`) — a genuine accepts-invalid CORRECTION (not a migration): the
> collating/equivalence bracket-tokens.** A POSIX collating-element `[.….]` or equivalence-class `[=…=]`
> bracket-token in a class is PCRE2 err 113 (standalone / member / range-LEFT) or err 150 (range-RIGHT). This
> row's validator NEVER recognised a standalone/member/range collating or equivalence token — and neither did
> the grammar — so the released parser ACCEPTED all of them (`class_literal` reads the token's `[` as a literal
> `0x5B`): a single-source-of-truth DOUBLE hole. `.4.12` adds a lookahead-only `class_bracket_token` recognizer
> (`"[" class_bracket_token_tail`) and blocks the shape at the member position (`!class_bracket_token` on
> `class_member_literal`/`_nocaret`), the class-OPEN position (`!class_bracket_token_tail` on `char_class`'s
> no-caret alternative — the opening `[` itself forms the opener, `[.a.]` err 113), and the range-RIGHT position
> (a `class_atom - class_bracket_token` `invalid_class_range` alternative for an ascending `[!-[.a.]]`). The
> content scan is escape-aware and stops at an unescaped `]` (`[.].]` ACCEPT, `[.\].]` REJECT). Behavior-CHANGING
> at `--parse` (accepts-invalid → PCRE2-convergent reject); corpus-invisible + contract-sample-invisible (counts
> UNCHANGED 92/26). Cert 249→251 (+2 lookahead-only PROOF rules), ast_shape 18/18 (3 inventory entries
> re-baselined `$2`→`$3`; accepted-AST byte-identical), oracle byte-identical `2189/1867/274/48`, duality
> unchanged. This does NOT delete the range-check — the blocked-descending endpoints still keep it wired.
| Scan-substring **NUMERIC** capture inventory — `(*scs:(N))`/`(*scs:(±N))` must reference an available capture; a RELATIVE reference of value zero (`(*scs:(+0))`/`(*scs:(-0))`) is rejected (`.4.7.b`, ledger `REGEX-0113` — PCRE2 err 126; the NAMED half migrated out in `.4.7.a`) | `(*scs:(1)a)`@0-groups, `(*scs:(0)…)`, `()(*scs:(+0)a)` | `.4.7.c` (grammar migration BLOCKED — witnessing wall) |
| **Unbounded quantified lookbehind** — a variable-length lookbehind body must be bounded | `(?<=a+)b`, `(?<=a{2,})b` | `.4.9` |

Already migrated **out** of the validator (now grammar-owned, listed here for the audit
trail): the six PCRE2-unsupported escape letters `\i \F \l \L \u \U` (`.3.1`), verb/
start-option NAME + argument shapes (`.3.2`/`.3.14`), quantified anchors (`.3.13`),
the class-open visibility model (`.3.15`), the counted-quantifier VALUE bound + brace
tokenization (`.3.18`), the numeric-callout range (`.3.16`), bare `\p`/`\P` property
escapes (`.4.1`), POSIX character-class **NAME** validity (`.4.6`), the
**escape-in-class** rejects `\A \B \C \G \K \N`(unbraced)`\R \X \Z \z` (`.4.4`), and
`\k`/group **NAME** validity — the `\k` shape + charset + length ≤ 128 code-points
(`.4.2`; `find_invalid_named_escape_or_group_name` + its 5 exclusive helpers +
`PCRE2_MAX_NAME_SIZE` DELETED), and the counted-quantifier **min > max** order (err 104)
— the first consumer of the general RULE-SPAN `value_compare` `@predicate` primitive, on
the new `counted_quantifier_range` rule (`.4.3`; `find_invalid_counted_quantifier` +
`validate_counted_quantifier_body` DELETED), the collating-element / equivalence-class
bracket-tokens `[.a.]`/`[=a=]` (`.4.12`; a genuine accepts-invalid correction via the
lookahead-only `class_bracket_token` recognizer), and **`\K` inside a lookaround** body
(`.4.10`; `find_invalid_keep_out_escape_in_lookaround` + its 4 exclusive helpers DELETED —
each lookaround opens a `lookaround` scope and the extracted `keep_out` rule carries
`@predicate not_in_scope_kind(lookaround)`, the first consumer of the scope-ancestry
predicate primitive `SCOPE-CONTEXT-PREDICATE.1`; behavior-neutral, the deleted validator
rejected exactly the same set), and the **start-option POSITION** rule — a recognized
`(*UTF)`-class start option is valid only as a contiguous run at the very start of the whole
pattern, never after any construct / in a later alternative / nested (`a(*CR)b`, `(a)(*CRLF)`,
`((*CRLF)a)`, `(*CRLF)a|(*LF)b` all err 160) — migrated PURELY STRUCTURALLY (`.4.8`, grammar
design A″: `find_invalid_verb_construct` + `is_start_option_position` DELETED). Start options
now derive ONLY off the distinguished `entry_concatenation` (the top-level-first-alternative
concatenation off the `entry_alternation` chain); every later alternative and every nested
pattern uses the shared `alternative`/`concatenation`, which have no start-option branch — so a
mis-positioned start option has no derivation and fails to parse, with NO semantic
fact/predicate (zero hot-path cost) and NO generator-side steering (the stimuli generator cannot
emit one either — the duality is closed by construction). Behavior-neutral; byte-identical AST
on every accepted pattern. A related divergence with no validator check at all — the
named-reference *unknown-name* inventory (`\k<zzz>`… @undefined) — is tracked as `.4.11`
(ledger `REGEX-0098`), and the **NAMED scan-substring** reference inventory — a
`(*scs:(<name>))`/`(*scs:('name'))` must reference a capture name defined somewhere in the
pattern (forward references legal) — is now grammar-owned too (`.4.7.a`; the **second consumer**
of the `phase: final` whole-input deferred-obligation primitive after `.4.11`: a
`has_fact(regex_defined_capture_name, $name) phase: final` gate on `scs_capture_name`, reshaped
`-> { name: $1 }` so `$name` resolves; behavior-neutral, the validator's NAMED branch +
`CaptureInventory.names` DELETED). The NUMERIC scan-substring inventory stays validator-owned:
`.4.7.b` (ledger `REGEX-0113`) fixed the one real accepts-invalid in it — a RELATIVE reference of
value zero (`(*scs:(+0))`/`(*scs:(-0))`, PCRE2 err 126) was ACCEPTED (it resolved `+0`→prior_count,
`-0`→prior_count+1) and now REJECTS via a single value guard. The full grammar migration of the
numeric refs is DEFERRED to `.4.7.c` (a WITNESSING WALL: sound scan-substring generation is
absolute-only, so a sign-split regresses `fully_certified` or `spf` — see
`docs/decisions/project_scs_numeric_migration_witnessing_wall.md`; forward `+N` also needs a new
forward/suffix-count primitive).

## Which layer rejected my pattern?

Because the two layers run in series, the **rejection message names its own source**,
and a one-line probe classifies any reject without guessing. This is the
**message-source probe**, documented in full in the platform book's *Diagnostic & Debug
Toolbox → Is a reject grammar-owned or validator-owned?*:

- exit `0` → **ACCEPT** (both layers passed);
- `Parser did not consume full input …` → the **grammar** rejected (the validator was
  never reached);
- any other message (e.g. `… character class …`) → the **validator** rejected — the
  grammar accepted, so the reject is **load-bearing** (deleting the validator would make
  the input accept).

Downstream consumers should not depend on this distinction — it is exactly what the
migration keeps shifting — but it is the authoritative way to scope and verify each
behavior-neutral validator→grammar slice.

## Cross-references

- Platform book — *Quality & Closure Model → THE EBNF IS THE SINGLE SOURCE OF TRUTH* —
  the doctrine that makes the validator a shrinking, transitional layer.
- Platform book — *Diagnostic & Debug Toolbox → the message-source probe* — the tool
  that measures load-bearing vs shadowed rejects.
- [Changelog Index](changelog-index.md) — the per-release record of which check migrated
  when, and the version each migration shipped in.
