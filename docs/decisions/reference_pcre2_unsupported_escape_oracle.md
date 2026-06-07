---
name: reference-pcre2-unsupported-escape-oracle
description: PCRE2 10.47 compile-time oracle for backslash-letter escapes (tool-verified via pcre2test). PCRE2 rejects EVERY unrecognized `\<letter>` escape, context-insensitively (atom == class == class-range): `\u \U \F \l \L` → error 137, `\i \I \J …` → error 103; it accepts only its recognized set (`\d \w \s \h \v \R \X \K \G \A \Z \z \b \B \N \Q \E \C \p \P \o \x \c \e \f \n \r \t \a` + octal/backrefs). `alt_bsux` is the mode that gives `\u`/`\u{…}` meaning. Implication: `regex_compile_validation.rs::find_invalid_escape_i` catches only 6 of the unrecognized letters (`i F l L u U`); full PGEN escape fidelity is a WHITELIST problem (strict = recognized-only, relaxed = the broad catch-all). Tool-backed 2026-06-07 (PGEN-REGEX-PCRE2-0006).
metadata:
  node_type: memory
  type: reference
  created: 2026-06-07
  owning_tree: REGEX-PCRE2-FIDELITY
---

# PCRE2 unsupported-escape oracle (compile-time accept/reject for `\<letter>`)

**Source:** `pcre2test` (PCRE2 version **10.47**, 2025-10-21, 8-bit build) — the in-repo authoritative
oracle (`regex_pcre2_compile_oracle_gate`). Tool-verified 2026-06-07 during `REGEX-PCRE2-FIDELITY.3.1`
design (`PGEN-REGEX-PCRE2-0006`). Reproduce any row with:

```bash
printf '/%s/%s\n\n' '\u' '' | pcre2test -q        # => Failed: error 137 ...
printf '/%s/%s\n\n' '\u' 'alt_bsux' | pcre2test -q # => (accepted)
```

## The matrix (default 8-bit; identical under `utf`)

| Escape | atom (`\X`) | class (`[\X]`) | class-range (`[\X-z]`) | PCRE2 verdict |
| --- | --- | --- | --- | --- |
| `\u` `\U` `\F` `\l` `\L` | REJECT | REJECT | REJECT | **error 137** "PCRE2 does not support \F, \L, \l, \N{name}, \U, or \u" |
| `\u{41}` | REJECT | — | — | **error 137** (`\u` is NOT special by default) |
| `\i` `\I` `\J` (and any other unrecognized letter) | REJECT | REJECT | REJECT | **error 103** "unrecognized character follows \\" |
| `\d \w \s \h \H \v \V \R \X \K \G \A \Z \z \b \B \N` | ACCEPT | ACCEPT (where class-legal) | — | recognized |
| `\e \f \n \r \t \a \C \Q…\E \p{…} \P{…} \o{…} \x{…} \cX` | ACCEPT | ACCEPT | — | recognized |
| `\u` / `\u{41}` under `alt_bsux` | ACCEPT | — | — | the **relaxed** analogue |

Key facts:

1. **Context-insensitive.** PCRE2 rejects an unrecognized `\<letter>` identically in atom, class, and
   class-range position (offsets only differ). So a PCRE2-faithful grammar must reject it in **all three**
   PGEN catch-alls — `simple_escape` (atom), `class_simple_escape` (class), and
   `class_range_literal_escape_letter` (class-range).
2. **Two error codes, one PGEN verdict.** PCRE2 distinguishes a documented-unsupported set
   (`\F \L \l \U \u` + `\N{name}` → 137) from "everything else unrecognized" (→ 103). PGEN reproduces
   only **accept/reject**, not error codes, so both map to a single hard REJECT.
3. **`find_invalid_escape_i` is a strict SUBSET.** The out-of-band validator
   (`rust/src/regex_compile_validation.rs::find_invalid_escape_i`) rejects exactly six letters
   (`i F l L u U`) context-insensitively. PCRE2 rejects a much larger set (every unrecognized letter:
   `\I \J \m \M \y …`). PGEN's `simple_escape`/`class_simple_escape` are "accept ANY letter except a few
   guarded forms" catch-alls, so **PGEN-default accepts a large set PCRE2 rejects** — a pre-existing
   divergence the validator only partially masks.
4. **Full escape fidelity is a WHITELIST.** Strict/PCRE2 mode should accept only PCRE2's recognized
   escape letters; the broad catch-all becomes the `relaxed` superset. The six-letter migration
   (`.3.1`) is a conformance-neutral stepping stone (it only re-homes `find_invalid_escape_i` from the
   validator into the grammar); the full recognized-escape whitelist is the eventual end-state
   (tracked as `REGEX-PCRE2-FIDELITY.3.11`, which would subsume `.3.1`'s six exclusions).
5. **`alt_bsux` ≈ relaxed.** PCRE2's `PCRE2_ALT_BSUX`/`PCRE2_EXTRA_ALT_BSUX` is the upstream knob that
   gives `\u`/`\u{…}` meaning — the conceptual analogue of PGEN's `relaxed` profile re-admitting them.

Composes with [[project_regex_pcre2_faithful_by_default_relaxed_optout]] and
[[project_ebnf_is_single_source_of_truth]]. See [[feedback_report_expected_verify_against_oracle]]
(run the executable oracle, do not trust a stated "expected").
