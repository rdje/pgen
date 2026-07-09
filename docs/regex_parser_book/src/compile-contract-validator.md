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

## The remaining check families

As of regex release `1.1.95`, the validator dispatches these families (each row is a
`find_*` check in `regex_compile_validation.rs`, and each maps to the
`REGEX-PCRE2-FIDELITY` leaf that will migrate it into the grammar):

| Family (what it rejects) | Example load-bearing inputs | Migration leaf |
|---|---|---|
| Character-class **RANGE** validity — nonliteral endpoints + descending ranges | `[\d-x]`, `[a-\p{Lu}]`, `[z-a]`, `[\x{100}-z]` | `.4.5` |

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
| Scan-substring capture **inventory** — `(*scs:(N))`/`(*scs:(<name>))` must reference an available capture | `(*scs:(1)a)`@0-groups, `(*scs:(0)…)` | `.4.7` |
| **Start-option POSITION** — a recognized `(*UTF)`-class start option may appear only in the start-option prefix | `a(*CR)b`, `(*FAIL)(*LIMIT_HEAP=5)a` | `.4.8` |
| **Unbounded quantified lookbehind** — a variable-length lookbehind body must be bounded | `(?<=a+)b`, `(?<=a{2,})b` | `.4.9` |
| `\K` inside a **lookaround** body | `(?=a\Kb)`, `(?<=\K.)`, `(*pla:a\Kb)` | `.4.10` |

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
`validate_counted_quantifier_body` DELETED). A related divergence with no validator check at all — the
named-reference *unknown-name* inventory (`\k<zzz>`… @undefined) — is tracked as `.4.11`
(ledger `REGEX-0098`).

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
