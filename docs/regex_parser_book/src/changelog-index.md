# Changelog Index

This chapter is an index — pointers into other docs that carry the full changelog detail. Use it to find what changed in a given release.

## Where the canonical changelogs live

| Source | Granularity | Purpose |
|---|---|---|
| `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` | Per-release shape change | The authoritative contract. Each release's section lists the AST shape changes consumers care about. |
| `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` | Per-bug | When a bug is fixed in a release, the ledger entry records the input/output shape change. |
| `CHANGES.md` (root) | Per-release | Human-readable summary of all changes. |
| Git tags + commit log | Commit-by-commit | The most granular source. |

When investigating "what changed and why," start with the contract document, drop down to the ledger for specific bugs, fall back to git for diffs.

## Releases relevant to this book

This book is **live** and tracks current main HEAD. Versioning summary:

- The most recent **published** parser-release section in the contract is **1.1.33 / Contract 1.1.35** (slice 2 of the typed-shape campaign).
- Slices 3 and 4 (typed `counted_quantifier_body` + `null` literal, then typed `counted_quantifier`) are landed on main but the consolidated contract identity bump for them lands together with the next quantifier-subtree slice that closes the outer `quantifier` rule.
- Until then, the contract document still shows `1.1.33 / 1.1.35` while this book describes the post-slice-3+4 shape that's actually emitted by main HEAD.

Below are the shape-change highlights of recent slices, with pointers to the contract sections (where applicable).

## Known deferred divergences (accepts-invalid, not yet fixed)

These are PCRE2-differential cases the RELEASED parser currently ACCEPTS but PCRE2 10.47 REJECTS. They are oracle-verified and intentionally deferred (each with a documented reason) rather than fixed in the current release — track them against the bug ledger and the owning `REGEX-PCRE2-FIDELITY` leaf.

| Divergence | Examples (all PGEN-ACCEPT / PCRE2-REJECT) | Why deferred | Ledger / leaf |
|---|---|---|---|
| **Named reference to an UNKNOWN group name** — err 115 "reference to non-existent subpattern" | `\k<zzz>` `\k'zzz'` `\k{zzz}` `(?P=zzz)` `\g{zzz}` `(?&zzz)` `(?P>zzz)` `\g<zzz>` `\g'zzz'` (no group `zzz` defined) | Named-reference resolution is inherently **whole-pattern two-pass** — forward references are LEGAL (`\k<aa>(?'aa'x)` is accepted by both), so a single left-to-right check would wrongly reject the legal forward form. Owned by the capstone. | `REGEX-0098` / `.3.22` (fix in `.4`) |

**Not on this list (distinct, pre-existing):** the NUMERIC single-digit `\1`…`\9` / `\g1` N<10 Non-Goal (`numeric_backreference_single` is ungated by design — REGEX-0083/0086). PCRE2 rejects `\g1`@0-groups err 115, but that is the numeric family, not the named-reference family above.

### 1.1.102 / Contract 1.1.104 — REGEX-0112: start-option verb POSITION is now grammar-owned; behavior-neutral validator→grammar migration + validator-check deletion

> **For RGX maintainers**: a BEHAVIOR-NEUTRAL migration (release bump 1.1.101→1.1.102 / contract 1.1.103→1.1.104; **schema stays `1`**), no action required. PCRE2 allows a start-option verb — the newline/UTF/mode class `(*UTF)` `(*UCP)` `(*CRLF)` `(*LF)` `(*ANYCRLF)` `(*NO_AUTO_POSSESS)` `(*NO_START_OPT)` … and the heap/match/depth-limit forms `(*LIMIT_HEAP=…)` `(*LIMIT_MATCH=…)` `(*LIMIT_DEPTH=…)` — **only at the very start of the whole pattern** (a leading run of them, before any other atom); anywhere else they are err 160 "(*VERB) not recognized or malformed" / err 168-class. The released parser already REJECTED a mis-positioned start option via the out-of-band validator (`find_invalid_verb_construct` / `is_start_option_position`), while the GRAMMAR alone ACCEPTED it (`directive_body_nonquantifiable` carried `directive_limit_named | directive_option_named` on the general `piece`, reachable anywhere) — a single-source-of-truth hole. `REGEX-PCRE2-FIDELITY.4.8` migrates the position rule INTO `grammars/regex.ebnf` and DELETES the standalone validator checks.
>
> **Grammar design — the A′→A″ pivot (surfaced to the director).** The rule is POSITIONAL, not contextual: a start option is legal in the leading prefix and illegal everywhere else (mid-pattern, nested inside ANY group, or after the first `|` branch). The originally-endorsed design **A′** made it *fact-based* — emit a `seen_atom` fact and gate the start option on `@predicate lacks_fact(seen_atom)`. A′ broke GENERATION duality: the store-aware stimuli generator has no `lacks_fact` branch-prune (it supports `fact_count_at_least` count-prune and `has_fact` name-draw, but cannot suppress a branch on the ABSENCE of a fact), so it emitted mis-positioned start options the A′ grammar then rejected — a fresh duality break (cert spf 0→65). We **pivoted to a purely STRUCTURAL design A″** (grammar-only, no facts/predicates — fix-hierarchy-preferred): the whole pattern is peeled so the start-option run can only derive in the leading position. `regex = entry_alternation`; the entry chain `entry_alternation = entry_alternative ("|" alternative)*` / `entry_alternative = entry_concatenation?` / `entry_concatenation = start_option_piece+ piece* -> [$1*, $2**] | piece+ -> [$1**]` admits a leading `start_option_piece+` run ONLY on the first concatenation of the first branch; every nested/subsequent context routes through the ordinary `piece`, which no longer carries the start-option verbs. `start_option_piece = start_option_verb !quantifier`; `start_option_verb = "(*" start_option_body ")"`; `start_option_body = directive_limit_named | directive_option_named` (peeled out of `directive_body_nonquantifiable`, now `directive_mark_named | directive_verb_named | directive_mark_shorthand`). The `!quantifier` guard preserves the PCRE2 err-that a start option cannot be quantified. **Verdict UNCHANGED** at the released `--parse` surface: a leading run `(*UTF)(*CRLF)abc` still ACCEPTs; a mid-pattern `a(*UTF)`, nested `(a(*UTF))`, or post-`|` `a|(*UTF)b` start option still REJECTs (`E_PARSE_FAILURE`); ordinary verbs `(*ACCEPT)` `(*FAIL)` `(*MARK:x)` still parse anywhere. Only the owning layer moved validator→grammar. **Conformance**: `.4.8` is corpus-invisible / released-neutral, proven by a grammar-verdict flip-diff over ALL 283 corpus `(*` patterns — EXACTLY ONE grammar verdict changes, `a(*CR)b` (pristine grammar accepted it, A″ rejects it), and the released validator's `is_start_option_position` ALREADY rejected it, so the RELEASED oracle verdict is unchanged on every corpus cell. The `regex_pcre2_compile_oracle_gate` asserts BOUNDS (not an exact tuple): `match ≥ 1845`, `false-accept ≤ 299`, `false-reject ≤ 48`; A″ measures `match 1870 / false-accept 270 / false-reject 48` on the 2188 non-pathological patterns (all within envelope — `fr=48` sits at the ceiling ⇒ zero new false-rejects), plus the non-changed line-1340 pcre2-accept match. ⚠️ NB the `2189/1867/274/48` tuple quoted in prior release entries is STALE (false-accepts drifted 274→270 at an earlier release; the gate's bounds masked it — a separate doc-drift follow-up, not a `.4.8` change). Verified string-by-string via `rust/tests/regex_start_option_position_grammar_migration.rs` (MUST_REJECT flat/nested/post-`|` + MUST_ACCEPT leading-run/empty/verbs-anywhere). regex cert `265/265 UNKNOWN=0 fully_certified=true` seeds 0/7/42 (259→265: +6 structural rules `entry_alternation` / `entry_alternative` / `entry_concatenation` / `start_option_piece` / `start_option_verb` / `start_option_body`), `--lint-grammar` 0 errors (265 rules), `duality_hunt_gate` regex lanes rebaselined (the `"start option must appear at the start-option prefix"` lane VANISHED with its validator; the residual `"Parser did not consume full input at position N"` lane is a PRE-EXISTING generator over-approximation — `(?#)?+` err 109 / unbalanced-paren err 114 — tracked as `.4.8.1`, NOT a regression), `parse_harness_equivalence_gate` regex byte-identical, `ast_shape` aligned (inventory 232→236; accepted-AST byte-identical). **AST**: schema stays `1`. Bug ledger: `REGEX-0112`. **Contract section:** "Release 1.1.102 / Contract 1.1.104 Highlights". Rules detail: `rules-top-level.md` (§ `regex` / the entry-alternation chain) + `rules-piece.md` (§ start-option peel) + `rules-misc.md` (§ `directive_body_nonquantifiable`); component reference: `compile-contract-validator.md`.

### 1.1.101 / Contract 1.1.103 — REGEX-0111: `\K`-in-lookaround is now grammar-owned; behavior-neutral validator→grammar migration + validator-check deletion

> **For RGX maintainers**: a BEHAVIOR-NEUTRAL migration (release bump 1.1.100→1.1.101 / contract 1.1.102→1.1.103; **schema stays `1`**), no action required. PCRE2 forbids `\K` inside any lookaround body (`pcre2test` 10.47 err 199 "\K is not allowed in lookarounds") — every form `(?=` `(?!` `(?<=` `(?<!` `(?*` `(?<*` `(*pla:`…, including when nested inside a group inside the lookaround (`(?=a(b\Kc))`, `((?=x\Ky))`). The released parser already REJECTED all of these via the out-of-band validator `find_invalid_keep_out_escape_in_lookaround`, while the GRAMMAR alone ACCEPTED them (`\K`/`keep_out` was an ungated `anchor` branch) — a single-source-of-truth hole. `REGEX-PCRE2-FIDELITY.4.10` migrates the reject into `grammars/regex.ebnf` and DELETES the standalone validator check. Because `\K`-in-lookaround is CONTEXTUAL (its legality depends on an enclosing lookaround possibly several groups up), it is gated by the parser-agnostic scope-ancestry `@predicate not_in_scope_kind(lookaround)` (`SCOPE-CONTEXT-PREDICATE.1`, the first grammar consumer): each of the 7 lookaround rules opens a `lookaround` scope at its opener (via a small open-marker rule, the `capture_open` idiom) and closes it after its body; the extracted `keep_out` rule carries the predicate. **Verdict UNCHANGED** at the released `--parse` surface: `\K`-in-lookaround still REJECTs (`E_PARSE_FAILURE`), `\K` bare / in atomic-capturing-noncapturing groups / after a closed lookaround (`(?=ab)\K`) still ACCEPTs — only the owning layer moved validator→grammar. **Conformance**: oracle byte-identical `2189/1867/274/48` (the `\K`-in-lookaround cells are corpus-invisible; verified string-by-string via `rust/tests/regex_keep_out_in_lookaround_grammar_migration.rs`, 19 reject + 16 accept cells). regex cert `259/259 UNKNOWN=0 fully_certified=true` seeds 0/7/42 (251→259: +8 WITNESSED rules — 7 lookaround open-markers + `keep_out`), `--lint-grammar` 0 errors (259 rules), `duality_hunt_gate` 9 lanes no new/vanished, `parse_harness_equivalence_gate` regex byte-identical, `ast_shape` 4 aligned (inventory 225→232; accepted-AST byte-identical). **AST**: schema stays `1`. Bug ledger: `REGEX-0111`. **Contract section:** "Release 1.1.101 / Contract 1.1.103 Highlights". Rules detail: `examples-anchors.md` (§ `\K` is rejected inside a lookaround) + `rules-groups.md` (§ `lookaround`); component reference: `compile-contract-validator.md`.

### 1.1.100 / Contract 1.1.102 — REGEX-0110: collating-element / equivalence-class bracket-tokens are now grammar-owned; a genuine accepts-invalid correction

> **For RGX maintainers**: a BEHAVIOR-CHANGING correctness fix (release bump 1.1.99→1.1.100 / contract 1.1.101→1.1.102; **schema stays `1`**), PCRE2-convergent (`REGEX-PCRE2-FIDELITY.4.12`; oracle `pcre2test` 10.47) — closes the collating/equivalence half of the `.4.5.d` deferred class-range residual. A POSIX collating-element `[.….]` or equivalence-class `[=…=]` bracket-token inside a character class is rejected by PCRE2 (err 113 "POSIX collating elements are not supported" as a standalone / member / range-LEFT token; err 150 "invalid range" as a range-RIGHT endpoint). The released parser previously **accepted-invalid** all of these at BOTH layers: `class_literal` (via `class_safe_special`) reads a class `[` as a literal `0x5B`, so a `[.`/`[=` token decomposed into a `[` member plus the surrounding `.`/`a`/`=` literal members, and NEITHER the grammar NOR the out-of-band `validate_regex_compile_contract` recognised the token (a single-source-of-truth DOUBLE hole — unlike the class-range holes `REGEX-0107`/`0108`/`0109`, where the validator did reject). This slice adds a lookahead-only recognizer `class_bracket_token = "[" class_bracket_token_tail` (matching `[.` … `.]` / `[=` … `=]`) and blocks the shape at three sites: (1) a MEMBER guard `!class_bracket_token` on `class_member_literal`/`_nocaret` (mirrors the `.4.6` `[:name:]` idiom); (2) a class-OPEN guard `!class_bracket_token_tail` on `char_class`'s no-caret alternative — the opening `[` itself forms the opener (`[.a.]` err 113); (3) a range-RIGHT `class_atom … "-" … class_bracket_token` alternative on `invalid_class_range` (an ASCENDING `[!-[.a.]]`, not caught by `.4.5.c`'s descending guard). Tokenization is PCRE2-exact (every case oracle-verified): the opener `[` must be IMMEDIATELY followed by `.`/`=` — a `^` negation or an invisible `\E`/`\Q\E` between breaks it; the content scan is escape-aware and stops at an UNESCAPED `]` (the class close). **Verdict FLIPS (ACCEPT→REJECT)**: standalone/member/range-LEFT `[.a.]` `[=a=]` `[..]` `[[.a.]]` `[[=a=]]` `[a[.a.]b]` `[[.a.]-z]` `[\|[.a.]]`; ASCENDING range-RIGHT `[!-[.a.]]` `[!-[=a=]]`; escape-aware `[.\].]` `[.a\.]` `[=a\=]`. **ACCEPT** (unchanged): lone `.`/`=` `[.]` `[=]`; no terminator `[[.]` `[[=]` `[.ab]`; no immediate opener `[a.b]` `[x.a.]`; class closes first `[.].]` `[.a].]`; opener broken `[^.a.]` `[\E.a.]`; escaped `[\.a.]` `[\[.a.]`; controls `[[:alpha:]]` `[a-z]`. The DESCENDING range-RIGHT cases (`[a-[.a.]]` `[z-[.a.]]` `[a-[=a=]]`) were already rejected by `.4.5.c`'s `descending_class_range` (pinned as regression controls). **Conformance**: oracle byte-identical `2189/1867/274/48` (the collating/equivalence cells are not in the corpus ⇒ corpus-invisible; verdicts verified string-by-string via `rust/tests/regex_class_bracket_token_grammar_migration.rs`). regex cert `251/251 UNKNOWN=0 fully_certified=true` seeds 0/7/42 (249→251: +2 lookahead-only PROOF rules `class_bracket_token` / `class_bracket_token_tail`), `--lint-grammar` 0 errors (251 rules), `duality_hunt_gate` 9 lanes no new/vanished, `parse_harness_equivalence_gate` regex byte-identical, `ast_shape` 18/18 (3 inventory entries re-baselined `$2`→`$3` for the inserted lookaheads; accepted-AST shape byte-identical). Contract manifest success/failure sample counts UNCHANGED `92/26` (contract-sample-invisible, unlike `.4.5.d`). **AST**: schema stays `1` (no shape change on accepted inputs). The only new rejections use `E_PARSE_FAILURE` — match on the **code**, not message text. **Scope**: the blocked-descending endpoints (`\Q..\E`/`\u{}` decode-None, bare-whitespace empty-`$text`) stay validator-owned; the `find_invalid_char_class_construct` range-check is deleted only once those land. Bug ledger: `REGEX-0110`. **Contract section:** "Release 1.1.100 / Contract 1.1.102 Highlights". Rules detail: `rules-char-class.md` (§ Collating-element / equivalence-class bracket-tokens); component reference: `compile-contract-validator.md`.

### 1.1.99 / Contract 1.1.101 — REGEX-0109: POSIX-class class-range endpoint reject is now grammar-owned; behavior-neutral validator→grammar migration

> **For RGX maintainers**: **mostly a behavior-NEUTRAL validator→grammar migration, PLUS one PCRE2-convergent correctness tightening** (release bump 1.1.98→1.1.99 / contract 1.1.100→1.1.101; **schema stays `1`**; the POSIX-endpoint sibling of `REGEX-0107`/`0108`). A class range whose LEFT or RIGHT endpoint is a valid POSIX class (`[:name:]`) — `[[:alpha:]-z]` (POSIX left), `[!-[:alpha:]]` (POSIX right) — is PCRE2 err 150 "invalid range in character class". For most of these the released parser already REJECTED them (via the out-of-band `validate_regex_compile_contract`), while the GRAMMAR alone ACCEPTED them (`class_atom` reads the bracket's `[` as a `class_safe_special` literal `0x5B`, so an ascending `!-[` range forms and the trailing `:alpha:]` become separate members). This slice (`REGEX-PCRE2-FIDELITY.4.5.d`) migrates the reject INTO the EBNF: the `!invalid_class_range` guard gains two alternatives referencing the EXISTING positively-reachable `posix_class` rule — `posix_class zw* "-" zw* class_atom` (POSIX left) and `class_atom zw* "-" zw* posix_class` (POSIX right). Because `posix_class` is already positively entered, NO new rule is added and certificate-coverage `total` is unchanged (249). The `class_atom`-after-`-` requirement keeps the trailing-dash carve-out valid (`[[:alpha:]-]` and the zero-width `[[:alpha:]-\Q\E]` ACCEPT — no atom after `-`; PCRE2-verified). **⚠️ ONE downstream-visible change**: the validator had an accepts-invalid HOLE — `dash_is_trailing_literal` skips whitespace/zero-width after the dash and wrongly treated the dash as a literal trailing dash even for a NonLiteral (POSIX) left endpoint — so `[[:digit:]-   ]` (a POSIX class then dash then only whitespace before `]`) was ACCEPTED by the released parser where PCRE2 rejects err 150. This slice flips that subset **ACCEPT→REJECT** at `--parse` (PCRE2-convergent). **REJECT** (grammar-owned now): `[[:alpha:]-z]` `[[:alpha:]-a]` `[[:digit:]-9]` `[[:^alpha:]-z]` `[!-[:alpha:]]` `[[:alpha:]-[:digit:]]` `[a-[:alpha:]]`; **ACCEPT→REJECT flip subset**: `[[:digit:]-   ]` `[[:alpha:]- ]` `[[:digit:]-\t]`. **ACCEPT** (unchanged): `[[:alpha:]]` `[[:alpha:]-]` `[[:alpha:]-\Q\E]` `[[:alpha:]a]` `[a[:alpha:]]` `[a-z[:digit:]]` `[[:alpha:][:digit:]]` `[!-[]` `[a-z]` `[ -!]`. **Conformance**: oracle byte-identical `2189/1867/274/48` (the flip cells are not in the corpus — the CONTRACT MANIFEST caught the flip, moving `[[:digit:]-   ]` success→failure, counts 92/26; verdicts verified string-by-string against `pcre2test` 10.47 via `rust/tests/regex_class_range_posix_grammar_migration.rs`). regex cert `249/249 UNKNOWN=0 fully_certified=true` seeds 0/7/42 (**UNCHANGED** — no new rule), `--lint-grammar` 0 errors (249 rules), `duality_hunt_gate` 9 lanes no new/vanished, `parse_harness_equivalence_gate` regex byte-identical, `ast_shape` inventory 225 **UNCHANGED** (no new `->` shape — `invalid_class_range` is lookahead-only). **Scope**: VALID-POSIX-name endpoints only; the collating/equivalence bracket-tokens (`[!-[.a.]]`, `[!-[=a=]]`, joins `.4.12`) and the quoted/`\u{}`/bare-whitespace-endpoint descending cases stay validator-owned. **AST**: schema stays `1`. Bug ledger: `REGEX-0109`. **Contract section:** "Release 1.1.99 / Contract 1.1.101 Highlights". Rules detail: `rules-char-class.md` (§ POSIX-class range endpoints); component reference: `compile-contract-validator.md`.

### 1.1.98 / Contract 1.1.100 — REGEX-0108: DESCENDING literal class-range reject is now grammar-owned; behavior-neutral validator→grammar migration

> **For RGX maintainers**: a **behavior-NEUTRAL** validator→grammar migration (release bump 1.1.97→1.1.98 / contract 1.1.99→1.1.100; **schema stays `1`**; the descending-literal sibling of `REGEX-0107`) — the released parser's accept/reject verdicts and every accepted AST are byte-identical to `1.1.97`, so **no downstream action is needed**. A class range of two LITERAL endpoints whose left code point exceeds the right — `[z-a]`, `[9-0]`, `[\x39-\x30]`, `[\x{100}-a]` (PCRE2 err 108 "range out of order in character class") — was already REJECTED by the released parser, but the reject was owned only by the out-of-band `validate_regex_compile_contract`; the GRAMMAR alone ACCEPTED them (`class_range` forms for any two literals and its `@validate: ord($1)<=ord($5)` is a non-parse-gating codegen annotation). This slice (`REGEX-PCRE2-FIDELITY.4.5.c`) migrates the reject INTO the EBNF via a **code-point VALUE comparison**: the `!invalid_class_range` guard gains a THIRD shape `descending_class_range = class_range_endpoint zw* "-" zw* class_range_endpoint`, gated by the new `value_compare_codepoint` `@predicate` (RULE-SPAN-VALUE-CONSTRAINT.3, `PGEN-RSVC-0003`) — it decodes each RAW endpoint spelling (bare non-whitespace literal + `\xHH`/`\x{}`/`\NNN`/`\o{}`/`\cX`/named escapes) to its Unicode code point and matches when left > right. The braced `[\x{100}-a]` is the code-point discriminator: a textual compare (`\`=92 < `a`=97) would wrongly ACCEPT; only decoding (256 > 97) rejects. Endpoints that cannot be reliably decoded stay validator-owned: `\Q..\E`/`\u{}` (decode `None`) and a BARE WHITESPACE endpoint (whose raw `$text` reaches the predicate EMPTY — a latent pipeline finding), so ascending whitespace ranges **ACCEPT** unchanged (`[ -!]` `[ --]` `[\t-a]`). Because the validator still also rejects descending ranges (its range-check deletion is deferred to `.4.5.d`), the released `--parse` verdict is UNCHANGED; observable only at the grammar layer (the certified interpreter). **ACCEPT** (unchanged): ascending/equal ranges `[a-z]` `[a-a]` `[0-9]` `[\x30-\x39]` `[a-\x{100}]`; dash carve-outs `[a-]` `[-a]` `[\d-]` `[\d\-x]` `[--/]`; members `[abc]`. **Conformance**: oracle byte-identical `2189/1867/274/48` (released verdicts unchanged ⇒ corpus-invisible; proven at the grammar layer via `rust/tests/regex_class_range_descending_grammar_migration.rs`; an in-slice bare-whitespace over-block regression `48→50` was root-caused via runtime trace + the interpreter before/after bisect, and fixed before release). regex cert `249/249 UNKNOWN=0 fully_certified=true` seeds 0/7/42 (245→249: +4 lookahead-only PROOF rules `descending_class_range` / `class_range_endpoint` / `class_range_decodable_atom` / `class_range_decodable_escape`), `--lint-grammar` 0 errors (249 rules), `duality_hunt_gate` 9 lanes no new/vanished, `parse_harness_equivalence_gate` regex byte-identical, `ast_shape` inventory 224→225 aligned (the new `class_range_endpoint -> $text`). **AST**: schema stays `1`. Bug ledger: `REGEX-0108`. **Contract section:** "Release 1.1.98 / Contract 1.1.100 Highlights". Rules detail: `rules-char-class.md` (§ Nonliteral class-range endpoints); component reference: `compile-contract-validator.md`.

### 1.1.97 / Contract 1.1.99 — REGEX-0107: nonliteral class-range endpoints are now grammar-owned; behavior-neutral validator→grammar migration

> **For RGX maintainers**: a **behavior-NEUTRAL** validator→grammar migration (release bump 1.1.96→1.1.97 / contract 1.1.98→1.1.99; **schema stays `1`**; a sibling of `REGEX-0101`/`0102`/`0104`) — the released parser's accept/reject verdicts and every accepted AST are byte-identical to `1.1.96`, so **no downstream action is needed**. A class range whose LEFT or RIGHT endpoint is a nonliteral shorthand (`\d \D \h \H \s \S \v \V \w \W`) or property (`\p…` / `\P…`) escape is PCRE2 err 150 "invalid range"; the released parser already REJECTED these, but the reject was owned only by the out-of-band `validate_regex_compile_contract` — the GRAMMAR alone ACCEPTED them (`class_atom` excludes the shorthand/property escapes, so `class_range` never forms and `[\d-x]` splits into three members `\d`, `-`, `x`). This slice (`REGEX-PCRE2-FIDELITY.4.5.b`) migrates the reject INTO the EBNF: a zero-width negative lookahead `!invalid_class_range` guards the three class-item positions (`class_item` / `class_item_visible` / `class_item_visible_nocaret`, via `*_core` passthrough rules `-> $2`), recognizing the two shapes nonliteral-LEFT (`[\d-x]`, `[\p{Lu}-x]`) and literal-LEFT/nonliteral-RIGHT (`[a-\d]`, `[a-\p{Lu}]`). Because the validator still also rejects them (its range-check deletion is deferred until the whole class-range family is grammar-owned — `.4.5.c` + the `-[`/`\v\V` cases), the released `--parse` verdict is UNCHANGED; the migration's effect is observable only at the grammar layer (proven by the certified interpreter, which runs the grammar without the validator). **ACCEPT** (unchanged): literal-dash carve-outs `[a-]` `[-a]` `[\d-]` `[\d\-x]` `[\da-z]` `[-\d]` `[--/]`; plain/valid-escape ranges `[a-z]` `[\n-\r]` `[\x30-\x39]`; nonliteral members `[\d]` `[\p{Lu}]` `[\s\w]`. **Conformance**: oracle byte-identical `2189/1867/274/48` (released verdicts unchanged ⇒ corpus-invisible; proven at the grammar layer via `rust/tests/regex_class_range_nonliteral_grammar_migration.rs`). regex cert `245/245 UNKNOWN=0 fully_certified=true` seeds 0/7/42 (239→245: +3 witnessed `*_core` rules, +3 lookahead-only PROOF rules `invalid_class_range` / `class_range_nonliteral_atom` / `class_range_nonliteral_shorthand`), `--lint-grammar` 0 errors (245 rules), `duality_hunt_gate` 9 lanes no new/vanished, `parse_harness_equivalence_gate` regex byte-identical, `ast_shape` inventory 221→224 aligned (the 3 new `-> $2` passthroughs). **AST**: schema stays `1`. Bug ledger: `REGEX-0107`. **Contract section:** "Release 1.1.97 / Contract 1.1.99 Highlights". Rules detail: `rules-char-class.md` (§ Nonliteral class-range endpoints); component reference: `compile-contract-validator.md`.

### 1.1.96 / Contract 1.1.98 — REGEX-0106: `\v` / `\V` class-range accepts-invalid fix (a genuine correctness fix — NOT a neutral migration)

> **For RGX maintainers**: a BEHAVIOR-CHANGING correctness fix (release bump 1.1.95→1.1.96 / contract 1.1.97→1.1.98; **schema stays `1`**), PCRE2-convergent (`REGEX-PCRE2-FIDELITY.4.5.a.1`; oracle `pcre2test` 10.47) — a sibling of `REGEX-0105`, surfaced while mapping the COMPLETE nonliteral class-range endpoint set for the deferred `.4.5.b` migration. The vertical-whitespace shorthands `\v` / `\V` were treated as LITERAL class-range endpoints, so `[\v-x]`, `[\V-x]`, `[a-\v]` were **accepts-invalid** — the released parser (grammar AND the out-of-band validator) BOTH accepted patterns PCRE2 rejects err 150 "invalid range". Root cause, two halves: (validator) `is_nonliteral_class_escape` enumerated `\d \D \h \H \s \S \w \W` (+`\p \P`) but OMITTED `\v` / `\V`, so they decoded to a literal code point (`118` / `86`) in `class_escape_literal_codepoint`; (grammar) `class_range_literal_escape_letter_strict` listed `v` / `V`, so `\v` / `\V` formed a `class_range` endpoint (feeding the mis-read AND letting the store-aware generator emit `\v`-ranges). The already-correct horizontal analogs `\h` / `\H` are handled the right way — members only, never range endpoints. The fix makes `\v` / `\V` byte-identical in behavior to `\h` / `\H`: dropped from the grammar range-letter set (members via `class_simple_escape` untouched ⇒ `[\v]`/`[\V]` still valid; no longer generated as range endpoints ⇒ duality-safe) and added to `is_nonliteral_class_escape` (err-150 reject). A validator-only fix was rejected — the grammar would keep generating `\v`-ranges the corrected validator rejects (a new duality break); the grammar half is required. This is a **GRAMMAR+VALIDATOR-tier fix**: the class-range validity family stays validator-owned pending the deferred `.4.5.b`/`.4.5.c` grammar migration (which now covers a uniformly-correct nonliteral family). **Verdict FLIPS (ACCEPT→REJECT)**: `[\v-x]` `[\V-x]` `[a-\v]` `[\v-\v]` `[\d-\v]` `[\v-\d]`. **ACCEPT** (unchanged): members `[\v]` `[\V]` `[a\vb]`; hex/octal endpoints of the same code point `[\x0b-\x0c]` `[\013-\014]`; siblings `[\h-x]`/`[\d-x]` (reject) and `[a-z]` (accept). **Conformance**: oracle byte-identical `2189/1867/274/48` — the frozen corpus has no `\v` / `\V` class-range cells, so the fix is **corpus-invisible** (proven by the direct `pcre2test` sweep + the new validator unit pin `rejects_vertical_whitespace_shorthand_class_range_endpoints`, the same class as `REGEX-0097`'s `.3.21`). regex cert `239/239 UNKNOWN=0` seeds 0/7/42, `--lint-grammar` 0 errors (239 rules), `duality_hunt_gate` 9 lanes no new/vanished, `ast_shape` inventory 221 — all UNCHANGED. **AST**: schema stays `1` (no shape change on accepted inputs). The only new rejections use `E_PARSE_FAILURE` — match on the **code**, not message text. Bug ledger: `REGEX-0106`. **Contract section:** "Release 1.1.96 / Contract 1.1.98 Highlights". Rules detail: `rules-char-class.md` (§ `class_range_literal_escape_letter`); component reference: `compile-contract-validator.md`.

### 1.1.95 / Contract 1.1.97 — REGEX-0105: `-[` / `-||` class-range accepts-invalid fix (a genuine correctness fix — NOT a neutral migration)

> **For RGX maintainers**: a BEHAVIOR-CHANGING correctness fix (release bump 1.1.94→1.1.95 / contract 1.1.96→1.1.97; **schema stays `1`**), PCRE2-convergent (`REGEX-PCRE2-FIDELITY.4.5.a`; oracle `pcre2test` 10.47). Inside a NORMAL character class, a RANGE whose right endpoint begins with `[` or `||` was **accepts-invalid** — the released parser (grammar AND the out-of-band validator) BOTH accepted patterns PCRE2 rejects. Root cause: `regex_compile_validation.rs::scan_char_class`'s two range branches gated range detection on `!dash_starts_alt_extended_class_operator`, which suppressed the range whenever the char after `-` was `[` or `||` — a guard for PCRE2's ALTERNATE extended-class syntax `(?[...])`, but `scan_char_class` runs ONLY on NORMAL classes, so it was unconditionally mis-scoped. The guard is removed from both branches (and deleted) and the range right-endpoint reader now classifies a `[:..:]`/`[...]`/`[=..=]` bracket token NON-LITERAL. A bare `[` still reads the literal `0x5B` and orders normally. This is a **VALIDATOR-tier fix**: the class-range validity family stays validator-owned pending the deferred `.4.5.b`/`.4.5.c` grammar migration; the grammar, codegen, and generated parser are byte-identical. **Verdict FLIPS (ACCEPT→REJECT)**: err 150 nonliteral `[x-[:alpha:]]` `[a-[:digit:]]` `[a-[.-.]]` `[!-[:alpha:]]` `[\d-[z]]` `[\d-||z]`; err 108 descending `[a-[b]]` `[z-[a]]` `[a-[]` `[~-||]` `[}-||]`. **ACCEPT** (unchanged): `[!-[]` (the case that MASKED the bug — both endpoints ascending literals) `[+-[]` `[Z-[]` `[[-a]` `[--[]` `[a-||b]` `[a-]` `[-a]` `[a-z]`. **Conformance NET IMPROVEMENT**: oracle true-measured `2189/1867/274/48` (was `2189/1858/285/46`) — ~11 default-mode accepts-invalid corpus cells now correctly reject (false-accepts 285→274); the false-reject ratchet moves 46→48 for `[\d-[z]]`/`[\d-||z]`, which the corpus tests under the `alt_extended_class` modifier (`PCRE2_ALT_EXTENDED_CLASS`, a non-default mode PGEN does not model — same documented divergence class as the existing `[A--B]` class-set-operation false-rejects). **AST**: schema stays `1` (no shape change on accepted inputs). The only new rejections use `E_PARSE_FAILURE` — match on the **code**, not message text. `dash_starts_alt_extended_class_operator` deleted; the stale `1.1.27` test that had locked in the bug removed; new pin `regex_class_range_bracket_endpoint_rejects_pcre2_faithfully`. A SEPARATE new divergence — STANDALONE collating `[[.a.]]` / equivalence `[[=a=]]` members (PCRE2 err 113, PGEN accepts) — is tracked as `.4.12`, not fixed here. Bug ledger: `REGEX-0105`. **Contract section:** "Release 1.1.95 / Contract 1.1.97 Highlights". Component reference: `compile-contract-validator.md`.

### 1.1.94 / Contract 1.1.96 — REGEX-0104: counted-quantifier `{n,m}` min>max ORDER is now grammar-owned; behavior-neutral validator→grammar migration

> **For RGX maintainers**: BEHAVIOR-NEUTRAL internal migration (release bump 1.1.93→1.1.94 / contract 1.1.95→1.1.96; **schema stays `1`**), PCRE2-convergent (`REGEX-PCRE2-FIDELITY.4.3`; oracle `pcre2test` 10.47). In PCRE2, a counted quantifier whose minimum exceeds its maximum is err 104 "numbers out of order in {} quantifier". PGEN's GRAMMAR alone previously ACCEPTED min>max (`x{5,4}` parsed with min=5, max=4), masked downstream ONLY by the out-of-band `regex_compile_validation.rs::validate_counted_quantifier_body` — a single-source-of-truth hole. `.3.18` deferred this rule here because it is a cross-number VALUE comparison (leading-zero-hostile) that no context-free structural rule can express. This slice migrates it into the EBNF via the general RULE-SPAN `value_compare` `@predicate` primitive (the FIRST consumer of RULE-SPAN-VALUE-CONSTRAINT.2): the `{n,m}` range form is extracted into a dedicated `counted_quantifier_range` rule gated by `@predicate: { name: value_compare, args: [$min, le, $max], phase: post, view: shaped }` — a cross-capture comparison of the produced `{min,max}` fields, decimal-integer numeric so `{05,4}` = 5>4 rejects while `{05,5}` = 5≤5 accepts. A min>max range loses its (backtrackable) branch and the `literal_open_brace` guard blocks the literal fallback ⇒ whole-pattern REJECT, err-104-faithful (NO guard change). **No verdict flips** — every input's accept/reject is byte-identical to `1.1.93`; oracle gate byte-identical `2189/1858/285/46`. **Now grammar-REJECT** (was validator-reject): `x{5,4}` `a{5,2}` `a{10,2}` `a{\t5\t,\t2\t}` (tab is quantifier whitespace) `a{05,4}` (value 5>4). **ACCEPT** (unchanged verdict): `a{4,5}` `a{5,5}` `a{05,5}` `a{0,65535}` `a{5,}` `a{,5}` `a{5}` `a{\n5,2\n}` (`\n` makes the brace a literal). **Design note:** the gate uses `view: shaped` + NAMED refs (the proven SV `view:shaped` idiom), not raw positional `[$1,le,$5]` — this rule's own `->` shapes its content to Json, and the raw-content capture is not emitted on the sequence-with-return-transform path, so a raw-view positional ref hard-errors (tool-traced). **AST**: every accepted counted quantifier byte-identical, `{min,max}` shape preserved through the extracted rule (**schema stays `1`**). The only observable change is the rejection MESSAGE — validator text → `E_PARSE_FAILURE`; match on the **code**, not message text. `find_invalid_counted_quantifier` + `validate_counted_quantifier_body` (+ 3 unit tests) DELETED; cert 238→239 / ast_shape 220→221 (the new rule). 5 validator check families remain (capstone `.4` deletes the rest). Bug ledger: `REGEX-0104`. **Contract section:** "Release 1.1.94 / Contract 1.1.96 Highlights". Rules detail: `rules-quantifier.md` (§ `counted_quantifier_range`); component reference: `compile-contract-validator.md`.

### 1.1.93 / Contract 1.1.95 — REGEX-0103: `\k`/group NAME validity is now grammar-owned; validator→grammar migration + two fidelity refinements

> **For RGX maintainers**: validator→grammar migration of the `\k`-shape + capture/`\k` NAME charset + NAME length ≤ 128 rule (release bump 1.1.92→1.1.93 / contract 1.1.94→1.1.95; **schema stays `1`**), PCRE2-convergent (`REGEX-PCRE2-FIDELITY.4.2`; oracle `pcre2test` 10.47). In PCRE2, `\k` is ALWAYS a named-backreference introducer needing a non-empty delimited name (`\k<n>`/`\k'n'`/`\k{n}`; bare `\k`/`\kabc` = err 169, empty `\k''`/`\k<>`/`\k{}` = err 162), and every name is ≤ 128 code units (err 148). PGEN's GRAMMAR alone previously ACCEPTED a bare/empty `\k` (`\k` matched the `simple_escape` shorthand catch-all, trailing chars as literals) and an over-length name — masked downstream ONLY by the out-of-band `regex_compile_validation.rs::find_invalid_named_escape_or_group_name`, a single-source-of-truth hole. Three EBNF edits: (1) `name`'s continue run bounded `{0,127}` ⇒ ≤ 128 code-points; (2) `!"k"` on `simple_escape` + `'k'` dropped from `simple_escape_letter_strict` (the `.4.1` `\p`/`\P` precedent); (3) the missing `\k'name'` quote branch added to `backreference`. **Now grammar-REJECT** (was validator-reject): `\k` `\kabc` `\k''` `\k<>` `\k{}`, and any ≥ 129-code-unit name (`(?<A×129>x)`, `(?<n>a)\k<A×129>`). **ACCEPT** (unchanged verdict): `(?<n>a)\k<n>`/`\k'n'`/`\k{n}`, `(?<name>x)`, `(?'name'x)`, `(?P<name>x)`, the 128-code-point boundary. **Two fidelity refinements (NOT byte-neutral):** (a) `\k'name'` was silently mis-parsed as a shorthand-`k` + `'`/`n`/`'` literals (accepted, WRONG shape) — now a proper `{type:"backreference",kind:"named"}`; (b) the deleted validator used 8-bit BYTE `name.len()`, the grammar counts CODE POINTS — a non-ASCII name of 65–128 code points is now ACCEPT (was validator-REJECT under byte counting), the Unicode-only-faithful behavior ([[feedback_rgx_unicode_only_8bit_test_divergence]]); pure-ASCII names byte-identical. **AST**: schema stays `1` (the `backreference` shape already exists). Oracle gate byte-identical `2189/1858/285/46`; cert `238/238/UNKNOWN=0` seeds 0/7/42; differential-CERTIFIED. `find_invalid_named_escape_or_group_name` (+ its 5 exclusive helpers + `PCRE2_MAX_NAME_SIZE` + 3 unit tests) DELETED. Bug ledger: `REGEX-0103`. **Contract section:** "Release 1.1.93 / Contract 1.1.95 Highlights". Rules detail: `rules-escape.md` (§ `simple_escape`), `rules-groups.md` (named backreferences / name length); component reference: `compile-contract-validator.md`.

### 1.1.92 / Contract 1.1.94 — REGEX-0102: escape-in-class rejects are now grammar-owned; behavior-neutral validator→grammar migration

> **For RGX maintainers**: BEHAVIOR-NEUTRAL internal migration (release bump 1.1.91→1.1.92 / contract 1.1.93→1.1.94; **schema stays `1`**), PCRE2-convergent (`REGEX-PCRE2-FIDELITY.4.4`; oracle `pcre2test` 10.47). In PCRE2 the escapes `\A \B \C \G \K \R \X \Z \z` are invalid as a character-class member, and `\N` is valid inside a class only in its braced named-codepoint form `\N{…}` (bare `\N` invalid). PGEN's GRAMMAR alone previously ACCEPTED all of them — `[\B]` as a shorthand escape, `[\C]` via `single_byte_escape`, and `[\A-x]`/`[\z-\x{FFFF}]` via the range path — masked downstream ONLY by the out-of-band `regex_compile_validation.rs::find_invalid_char_class_construct` (`read_class_atom`), a single-source-of-truth hole. This slice migrates the rule into the EBNF via three edits: (1) both `class_simple_escape` variants carry `!"A" !"B" !"C" !"G" !"K" !"R" !"X" !"Z" !"z" !( "N" !"{" )` (the `!( "N" !"{" )` keeps `\N{…}` valid); (2) `single_byte_escape` (`\C`) dropped from `class_escape_unit` (stays in `escape_unit` for the pattern body); (3) `A`/`G`/`z` dropped from `class_range_literal_escape_letter_strict` (range-endpoint reachable). **No verdict flips** — every input's accept/reject is byte-identical to `1.1.91` in BOTH profiles; oracle gate byte-identical `2189/1858/285/46`. **Now grammar-REJECT** (was validator-reject): member `[\A]` `[\B]` `[\C]` `[\G]` `[\K]` `[\N]` `[\R]` `[\X]` `[\Z]` `[\z]`, mid-pattern `a[\NB]c` `[a\Kb]`, range-endpoint `[\B-x]` `[a-\B]` `[\A-x]` `[\G-x]` `[\z-\x{FFFF}]`. **ACCEPT** (unchanged): legal shorthands/c-escapes `[\d]` `[\w]` `[\s]` `[\b]`(backspace) `[\n]` `[\a]` `[\e]`; braced `[\N{U+00E9}]`; property/hex `[\pL]` `[\x41]`; literal-transport letters `[\g]` `[\j]` `[\k]` `[\r]`; ranges `[\I-x]` `[\a-x]` `[\g-x]`; plain `[abc]`; all pattern-body forms `\C` `\B` `\K` `\N` `\A` `\Z` `\z`. **AST**: every accepted class member/range byte-identical (**schema stays `1`**). The only observable change is the rejection MESSAGE — validator text → `E_PARSE_FAILURE`; match on the **code**, not message text. The two `read_class_atom` reject blocks (+ 3 unit tests) deleted; `scan_char_class` range analysis stays. 7 of the (original 8) validator check families remain (capstone `.4` deletes the rest). Bug ledger: `REGEX-0102`. **Contract section:** "Release 1.1.92 / Contract 1.1.94 Highlights". Rules detail: `rules-char-class.md` (§ *Escape validity inside a class*), `rules-escape.md` (§ `single_byte_escape`); component reference: `compile-contract-validator.md`.

### 1.1.91 / Contract 1.1.93 — REGEX-0101: POSIX character-class NAME validity is now grammar-owned; behavior-neutral validator→grammar migration

> **For RGX maintainers**: BEHAVIOR-NEUTRAL internal migration (release bump 1.1.90→1.1.91 / contract 1.1.92→1.1.93; **schema stays `1`**), PCRE2-convergent (`REGEX-PCRE2-FIDELITY.4.6`; oracle `pcre2test` 10.47). In PCRE2, a `[:name:]` token inside a character class is a POSIX-class ATTEMPT — the name (after an optional `^`) MUST be one of the 14 valid names (`alnum alpha ascii blank cntrl digit graph lower print punct space upper word xdigit`) or it is a compile error (err 130). PGEN's GRAMMAR alone previously ACCEPTED an unknown name (`[[:foo:]]` parsed as a class of literals `[:foo:` — `posix_class` failed on the bad name and the `[`/`:`/letters fell back to `class_literal`), masked downstream ONLY by the out-of-band `regex_compile_validation.rs::find_invalid_char_class_construct` — a single-source-of-truth hole. This slice migrates the rule into the EBNF so the grammar owns it: the class-member `[` literal is guarded by an inline negative lookahead for the `[:…:]` posix-token shape (two wrapper rules `class_member_literal` / `class_member_literal_nocaret`, `-> $2`), so a valid name is won by `posix_class` and an invalid name leaves the class unclosable → reject. **No verdict flips** — every input's accept/reject is byte-identical to `1.1.90` (the validator rejected `[[:foo:]]`/`[a[:<:]]`/`[[::]]` before, the grammar rejects them now, in BOTH profiles); oracle gate byte-identical `2189/1858/285/46`. **Now grammar-REJECT** (was validator-reject): `[[:foo:]]` `[[:foo:]` `[a[:<:]]` `[a[:>:]]` `[[::]]` `[[:^foo:]]` `[x[:foo:]y]` `[[:foo:]x]` `[^[:foo:]]` `[[:al pha:]]` `[[:foo:bar:]]` `[[:al:num:]]`. **ACCEPT** (unchanged): valid `[[:alpha:]]` `[[:^alpha:]]` `[[:alnum:][:digit:]]` `[^[:alpha:]]` `[x[:alpha:]y]` `[[:space:]]+`; aliases `[[:<:]]` `[[:>:]]` `[[:<:]]red[[:>:]]`; non-posix `[[:foo]` `[[:]]` `([[:]+)` `[a:foo:]` `[]:foo:]` `[\Q[:foo:]\E]`. **AST**: every accepted class byte-identical incl. the 4 contract-pinned POSIX shapes (**schema stays `1`**). The only observable change is the rejection MESSAGE — validator text → `E_PARSE_FAILURE`; match on the **code**, not message text. `is_valid_posix_class_name` (+ 2 unit tests) deleted; `scan_posix_class` recognition stays for range analysis. **Honest bound (unchanged, pre-existing):** the scan crosses both escaped/unescaped `]` (like the validator), whereas PCRE2 stops at an unescaped `]` (`[x[:foo]bar:]y]` accepts in PCRE2, rejects in PGEN — before AND after); a PCRE2-exact `]`-boundary follow-up. Bug ledger: `REGEX-0101`. **Contract section:** "Release 1.1.91 / Contract 1.1.93 Highlights". Rules detail: `rules-char-class.md` (§ `posix_class`).

### 1.1.90 / Contract 1.1.92 — REGEX-0100: the bare `\p`/`\P` Unicode-property escape is now grammar-owned; behavior-neutral validator→grammar migration

> **For RGX maintainers**: BEHAVIOR-NEUTRAL internal migration (release bump 1.1.89→1.1.90 / contract 1.1.91→1.1.92; **schema stays `1`**), PCRE2-convergent (`REGEX-PCRE2-FIDELITY.4.1`; oracle `pcre2test` 10.47). In PCRE2, `\p`/`\P` are ALWAYS Unicode-property introducers — a bare (un-braced) one MUST be followed by a valid one-letter general category (`\pL`, `\PN`, …) or the `{name}` form; anything else is a compile error. PGEN's GRAMMAR alone previously ACCEPTED a bad bare form (`\pA` parsed as `\p` shorthand + `A` literal), masked downstream ONLY by the out-of-band `regex_compile_validation.rs::find_invalid_property_escape` — a single-source-of-truth hole. This slice migrates the rule into the EBNF so the grammar owns it: `'P'`/`'p'` dropped from `simple_escape_letter_strict` (positive exclusion, generation-faithful), and the `!"p{"`/`!"P{"` guards broadened to whole-letter `!"p"`/`!"P"` on `simple_escape` + both `class_simple_escape` variants. **No verdict flips** — every input's accept/reject is byte-identical to `1.1.89` (the validator rejected `\pA`/`\P_`/`\p`/`[\pA]` before, the grammar rejects them now, in BOTH profiles). **Now grammar-REJECT** (was validator-reject): `\pA` `\P_` (bad category), `\p` `\P` (at EOF), `[\pA]` `[\P_]` (class), `a\pAb` `x\p` (mid-pattern). **ACCEPT** (unchanged): `\pL` `\PN` `\pl` `\Pn` `\pC` `\pZ` (short categories), `[\pL\PN]`, `\p{Lu}` `\P{Han}` `[\p{L}]` (braced). **AST**: every accepted property-escape byte-identical (**schema stays `1`**). The only observable change is the rejection MESSAGE — validator text → `E_PARSE_FAILURE`; match on the **code**, not message text. `find_invalid_property_escape` (+ its 2 unit tests) deleted; 7 of the 8 validator check families remain (capstone `.4` deletes the rest). Bug ledger: `REGEX-0100`. **Contract section:** "Release 1.1.90 / Contract 1.1.92 Highlights". Rules detail: `rules-escape.md` (§ `simple_escape`, § `property_escape`).

### 1.1.89 / Contract 1.1.91 — REGEX-0099: the PCRE2 `\Q` QUOTING model (unterminated quote-to-end + empty-`\Q\E`-quantified); grammar-encoded

> **For RGX maintainers**: BEHAVIOR change in BOTH directions (release bump 1.1.88→1.1.89 / contract 1.1.90→1.1.91; **schema stays `1`**), PCRE2-convergent (`REGEX-PCRE2-FIDELITY.3.23`; oracle `pcre2test` 10.47, an 85-cell matrix; validated pre-rebuild by the regex-CERTIFIED interpreter, 0 divergences). PCRE2 `\Q…\E` quotes everything between `\Q` and `\E` — or, unterminated, from `\Q` to END-OF-PATTERN — as **literal** (the only special sequence inside is `\E`). Before `1.1.89`, `\Q` was a bare shorthand escape (`simple_escape`, char `Q`), so this closes TWO sibling divergence classes with ONE root cause. **Newly ACCEPT** (was REJECTS-VALID — an unterminated `\Q…` metachar tail was mis-parsed as live regex): `\Q)` `\Q(` `\Q[` `\Q(?:` `\Q**` `\Qa**` `\Qa)b` (and the previously-accepted-but-mis-parsed `\Q` `\Qa` `\Qabc` `\Qa*b` `\Q^` `\Q$` `\Q|`, now one literal run). **Newly REJECT** (was ACCEPTS-INVALID — an empty quote is zero-width, unrepeatable): `\Q\E*` `\Q\E+` `\Q\E?` `\Q\E{2}` `\Q\E{2,}` `\Q\E{2,3}` `\Q\E{,2}` `\Q\E\Q\E*` (err 109). Encoded grammar-side (no engine, no validator — the host validator already skipped `\Q…` as quote-to-end): `'Q'` dropped from `simple_escape_letter_strict` (positive exclusion); `quoted_literal` narrowed to nonempty terminated (`char*`→`char+`); NEW `unterminated_quoted_literal` (quote-to-end) is a **NON-atom standalone `piece`** — so the generator can never pick a bare `\Q` as the absorption branch's atom (the generation-faithful fix for an in-slice cert seed-42 duality break; no `!"\\E"` lookahead needed); NEW `empty_quoted_literal` joins the non-quantifiable `zero_width` (so `\Q\E*` rejects via `!quantifier`, `a\Q\E*` = `a*` via absorption). **AST**: terminated `\Q…\E`, bare `\Q\E`, and `\Qa\E*` are byte-identical; intended semantic corrections — unterminated `\Q…` is now one `{type:"atom", kind:"quoted_literal", body}` (was `\Q`-as-escape + live-regex pieces) and `a\Q\E*` = `a*` (empty quote elided). **No new AST vocab — schema stays `1`.** Both directions apply in BOTH profiles. Rejection LAYER: the grammar (`E_PARSE_FAILURE`) — match on the **code**, not message text. Bug ledger: `REGEX-0099`. **Contract section:** "Release 1.1.89 / Contract 1.1.91 Highlights". Rules detail: `rules-piece.md` (§ `piece`, § `zero_width`), `rules-atom.md` (§ `quoted_literal`, § `unterminated_quoted_literal`), `rules-escape.md` (§ `simple_escape`); examples: `examples-quoted-literal.md`.

### 1.1.88 / Contract 1.1.90 — REGEX-0097: a LIMIT `=value` outside [0, 4294967289] now rejects PCRE2-faithfully; grammar-encoded

> **For RGX maintainers**: BEHAVIOR-TIGHTENING (release bump 1.1.87→1.1.88 / contract 1.1.89→1.1.90; **schema stays `1`**), PCRE2-convergent (`REGEX-PCRE2-FIDELITY.3.21`; oracle `pcre2test` 10.47, one-pattern-per-run binary search). PCRE2 bounds a numeric LIMIT start-option value (`(*LIMIT_HEAP=…)` / `(*LIMIT_MATCH=…)` / `(*LIMIT_DEPTH=…)` / `(*LIMIT_RECURSION=…)`) to a maximum of **4294967289** (`0xFFFFFFF9`); a larger value is err 160. The bound is PCRE2's Horner overflow guard `n > UINT32_MAX/10 - 1`, so the max accepted value is `429496728*10 + 9 = 4294967289` — **NOT** u32 max 4294967295 (which itself rejects). Purely LATENT — no prior validator check ever bounded the value (`directive_payload_digits = digit+` bounded only the shape), and it is hunter-invisible (generator + parser agreed). **Newly REJECT**: `(*LIMIT_HEAP=4294967290)` `(*LIMIT_HEAP=4294967295)` `(*LIMIT_HEAP=99999999999999999999)` and the leading-zero form `(*LIMIT_HEAP=00000000004294967290)`, uniform across all 4 LIMIT names. **Still ACCEPT** (unchanged): `(*LIMIT_HEAP=0)` `(*LIMIT_HEAP=500)` `(*LIMIT_HEAP=4294967289)` (the boundary), `(*LIMIT_HEAP=00000000004294967289)` (leading zeros are value-based), all-zeros. Encoded structurally with the `.3.16` `callout_number` / `.3.18` `quant_bound_number` idiom (NOT `@range` — proved atom-scoped/inert by `.3.16`): `directive_payload_digits` is now a wrapper over `directive_limit_value_body` (`"0"+ directive_limit_value_core? | directive_limit_value_core`), whose `directive_limit_value_core` is the nonzero-led 1..4294967289 lexicographic ladder — so an out-of-range run has no fully-consuming parse (the trailing `)` fails) and generation is in-range by construction. **AST**: every in-range value byte-identical — `value` stays the raw digit string (leading zeros preserved: `(*LIMIT_HEAP=00700)` → `value:"00700"`). Rejection LAYER: a NEW grammar rejection — match on the **code** (`E_PARSE_FAILURE`), not message text. The value bound applies in BOTH profiles (recognized-name shapes stay authoritative in relaxed). Bug ledger: `REGEX-0097`. **Contract section:** "Release 1.1.88 / Contract 1.1.90 Highlights". Rules detail: `rules-misc.md` (§ `directive` — LIMIT value bound).

### 1.1.87 / Contract 1.1.89 — REGEX-0096: only `(*ACCEPT)` may take a quantifier; every other `(*...)` directive quantified now rejects PCRE2-faithfully; grammar-encoded

> **For RGX maintainers**: BEHAVIOR-TIGHTENING (release bump 1.1.86→1.1.87 / contract 1.1.88→1.1.89; **schema stays `1`**), PCRE2-convergent (`REGEX-PCRE2-FIDELITY.3.20`; oracle `pcre2test` 10.47, a 17-cell matrix). PCRE2 makes `(*ACCEPT)` the ONLY quantifiable `(*...)` directive; every other form quantified is err 109. The verb/`(*:x)`-shorthand/MARK quantifier rejections were already correct (validator-owned); the NEW divergence is the **start-option-quantified** forms — `(*UTF)+` `(*UCP)+` `(*LIMIT_HEAP=5)+` were latently ACCEPTED (the validator's start-option arm checked only POSITION, never a trailing quantifier). This slice migrates the whole quantified-verb rule out-of-band validator → grammar AND closes the start-option hole (**REGEX-0096**). Encoded structurally via a piece-level split: `directive_verb` (the quantifiable rule, an `atom`) = ACCEPT verb + the relaxed unknown-name catch-all; new `directive_verb_nonquant` (a non-quantifiable `piece` branch `directive_verb_nonquant !quantifier`) = the 6 non-ACCEPT verbs + MARK + `(*:x)` shorthand + LIMIT + bare options. **Newly REJECT**: `(*UTF)+` `(*UCP)+` `(*LIMIT_HEAP=5)+` (start options, the divergence) plus the validator→grammar-moved `(*PRUNE)+` `(*FAIL)*` `(*:x)+` `(*MARK:x)+`. **Still ACCEPT** (unchanged): `(*ACCEPT)+` `(*ACCEPT:x)+` `(*ACCEPT){2,3}` and every non-quantified directive. **AST**: every accepted directive byte-identical (both split rules keep the `{type:"atom",kind:"directive_verb",body}` carrier; `directive_accept_named` keeps `{kind:"named",name,payload}`). **Relaxed**: a quantified KNOWN directive rejects in relaxed too; a quantified UNKNOWN-name verb (`(*foo)+`) stays relaxed-accepted (superset opt-out). Rejection LAYER: match on the **code** (`E_PARSE_FAILURE`), not message text. Both profiles behave identically for recognized names. Bug ledger: `REGEX-0096`. **Contract section:** "Release 1.1.87 / Contract 1.1.89 Highlights". Rules detail: `rules-misc.md` (§ `directive_verb`) + `rules-piece.md` (§ `piece`).

### 1.1.86 / Contract 1.1.88 — REGEX-0095: a quantifier on a stray `\E` (zero-width, TRANSPARENT) now rejects PCRE2-faithfully; grammar-encoded

> **For RGX maintainers**: BEHAVIOR-TIGHTENING (release bump 1.1.85→1.1.86 / contract 1.1.87→1.1.88; **schema stays `1`**), PCRE2-convergent (`REGEX-PCRE2-FIDELITY.3.19`; oracle `pcre2test` 10.47, a 44-cell interaction map). A stray `\E` (an unmatched end-of-quote) is PCRE2 zero-width — and, unlike an anchor (opaque, `REGEX-0088`/`.3.13`), it is **TRANSPARENT**: a quantifier binds THROUGH it to the preceding repeatable atom (`a\E*` = `a*`, ACCEPT), but is err 109 when no repeatable predecessor is reachable through elision. **A family of accepts-invalid spellings now REJECT**: bare `\E*` `\E{2}` `\E\E*`; anchor-blocked `^\E*` `\A\E*` `a^\E*` (an anchor is non-repeatable and blocks the bind); group/alternation-edge `(\E*)` `|\E*` `a|\E*` `(a|\E*)` (**REGEX-0095**; all were latent — no prior validator check). Encoded structurally: stray `\E` is a non-quantifiable `zero_width` piece (dropped from `simple_escape_letter_strict`); `piece` gains a STANDALONE branch and an ABSORPTION branch (`atom zero_width+ quantifier`, placed last). **AST**: the 8 `<atom>\E<quant>` accept cells (`a\E*`, `ab\E*` binds `b`, `a\E\E*`, `\Qa\E\E*`, `()\E*`, `(a)\E*`, `(?:)\E*`, `(a|b)\E*`) now bind the quantifier to the repeatable atom with the stray `\E`(s) elided (`[piece(atom), piece(\E, quant)]` → `[piece(atom, quant)]`); all other accepted patterns byte-identical. Rejection LAYER: match on the **code** (`E_PARSE_FAILURE`). **Deferred** (`REGEX-PCRE2-FIDELITY.3.23`): empty `\Q\E`-quantified (`\Q\E*`, …) is still accepts-invalid and byte-UNCHANGED (entangles with the `\Q`-as-`simple_escape` / unterminated-`\Q...\E` model). Both profiles behave identically. Bug ledger: `REGEX-0095`. **Contract section:** "Release 1.1.86 / Contract 1.1.88 Highlights". Rules detail: `rules-piece.md` (§ `piece`, § `zero_width`); examples: `examples-anchors.md` → "Stray `\E` transparent quantifier binding".

### 1.1.85 / Contract 1.1.87 — REGEX-0092/0093/0094: the PCRE2 counted-quantifier BRACE TOKENIZATION model; the value bound migrated into the grammar

> **For RGX maintainers**: BEHAVIOR-TIGHTENING + one CORRECTION (release bump 1.1.84→1.1.85 / contract 1.1.86→1.1.87; **schema stays `1`**), PCRE2-convergent (`REGEX-PCRE2-FIDELITY.3.18`; oracle `pcre2test` 10.47, a 47-cell matrix with hex-pattern cells freezing the whitespace set). PCRE2's brace model is now grammar-encoded: a syntactically-valid quantifier brace (digits + spaces/tabs anywhere inside — forms `{n}` `{n,}` `{n,m}` `{,m}`) is ALWAYS a quantifier, never a literal. **5 accepts-invalid spellings now REJECT**: `a{4294967296}` (**REGEX-0092** — the bound VALUE limit is 65535, and any >u32 value previously skipped ALL checks via the validator's `parse::<u32>().ok()?` overflow hole; err-105 class), `{2,5}`-at-start / `x|{2,5}` / `a{2}{3}` (**REGEX-0093** — a valid-syntax brace at a non-repeatable position; err-109 class; the new `literal_open_brace` negative-lookahead guard blocks the literal fallback), and `a{\t5\t,\t2\t}` (**REGEX-0094** — tab IS quantifier whitespace, so tab-spaced order violations reject; err-104 class). **1 rejects-valid spelling now ACCEPTS**: `a{\n5,2\n}` — a `\n`/`\f`/`\r`/`\v` anywhere inside the brace makes it a LITERAL in PCRE2 (also REGEX-0094; the quantifier-whitespace set is exactly space + tab). `min`/`max` stay typed ints (the value-structural `quant_bound_number` carries the same `@transform`); the literal `{` AST is byte-identical; the only AST-shape changes are `a{\n2\n}`-family accept/accept cells that now parse as the literal pieces PCRE2 sees. Rejection LAYER for the >65535 family moves from the contract message to a grammar parse error — match on the **code** (`E_PARSE_FAILURE`), not message text; the validator keeps exactly one counted-quantifier rule (min>max order, err-104 class, now space+tab-exact). Generation draws in-range bounds by construction (6th compile-contract migration). Both profiles behave identically on this surface. Bug ledger: `REGEX-0092`/`REGEX-0093`/`REGEX-0094`. **Contract section:** "Release 1.1.85 / Contract 1.1.87 Highlights". Rules detail: `rules-quantifier.md` (§ `counted_quantifier`, § `quant_bound_number`) + `rules-atom.md` (§ `literal_open_brace`); examples: `examples-quantifiers.md` → "The PCRE2 brace tokenization model".

### 2026-07-08 maintenance (release 1.1.84 / Contract 1.1.86 — UNCHANGED) — REGEX-PCRE2-FIDELITY.3.17: scan-substring capture-list generation is now store-aware (generation-side only)

> **For RGX maintainers**: IMPLEMENTATION-ONLY, SURFACE-NEUTRAL / conformance-neutral (no version bump; accept/reject set and every AST byte-identical — 21/21 probe matrix over named groups, scs numeric/named/relative/forward forms, backrefs, and conditionals). The scan-substring capture-list reference rule (`(*scs:('name'))` / `(*scs:(2))`) is validated against the FULL-pattern capture inventory by the post-parse contract, and PCRE2 accepts FORWARD references (`(*scs:('a'))(?<a>x)`) — so this class CANNOT move into the grammar as a parse-time gate. What changed is the GENERATION side: the stimuli generator was inventory-blind (54 of 57 generated scs groups were contract-rejected — invented names, out-of-range indices), and now draws scs names from the capture names it has already generated and scs indices from `1..=`(captures so far), via the new generation-side store directives (`@gen_emit_fact` on the new `capture_name` carrier; `@gen_predicate` on the new `scs_capture_name` / `scs_capture_number` value rules). Grammar-structurally the scs list items are now dedicated wrapper rules (`returned_capture_group = scs_capture_number | scs_capture_name_ref`) that are shape-transparent — the `captures` carrier in `scan_substring_group` is unchanged (see the JSON-carrier chapter's new inventory rows). **You need to do nothing**: parse behavior, error codes, and versions are all unchanged; the parse-side inventory check stays in the compile contract. Duality closure: the `(*scs:('_'))` hunter signature is GONE at seeds 0/7/42; regex certificate coverage holds at `224/224` `UNKNOWN=0`.

### 2026-07-08 maintenance (release 1.1.84 / Contract 1.1.86 — UNCHANGED) — REGEX-PCRE2-FIDELITY.3.16: the numeric-callout [0, 255] bound migrated into the grammar

> **For RGX maintainers**: IMPLEMENTATION-ONLY, SURFACE-NEUTRAL / conformance-neutral (no version bump). The PCRE2 numeric-callout value bound (err 138: `(?C256)` rejects; leading zeros accepted — `(?C0255)` / `(?C000000000255)` compile) previously lived in the out-of-band compile-contract validator (`find_invalid_numeric_callout`), invisible to the stimuli generator — which emitted `(?C4135)`-style out-of-range samples the parser then rejected (a generator⟷parser duality break, 5 rejects per 60 generated callouts at seed 0). The bound is now encoded **structurally** in `grammars/regex.ebnf` (`callout_number` — leading zeros plus an optional nonzero-led core ≤ 255; the `@transform` span parse keeps the exact typed-int carrier: `(?C0255)` → `arg: 255`), the validator check is DELETED (5th compile-contract migration), and generation is in-range BY CONSTRUCTION (duality probe 5/60 → 0/60). Accept/reject set UNCHANGED (`pcre2test` 10.47 oracle byte-identical — 1857/46/292/338; 31-cell matrix incl. the condition-callout site `(?(?C…)…)`, both profiles); every still-accepted callout AST byte-identical (19/19). Rejection LAYER moves from the contract message (`numeric callout argument exceeds PCRE2 compile limit 255`) to a grammar parse error — match on the **code** (`E_PARSE_FAILURE`), not message text. Rules detail: `rules-misc.md` § `callout_number`; contract: "Maintenance Update 2026-07-08".

### 1.1.84 / Contract 1.1.86 — REGEX-0091: character-class member VISIBILITY & the PCRE2 class-open model; the emptiness checks migrated into the grammar

> **For RGX maintainers**: BEHAVIOR-CORRECTING (release bump 1.1.83→1.1.84 / contract 1.1.85→1.1.86; **schema stays `1`**), PCRE2-convergent (`REGEX-PCRE2-FIDELITY.3.15`; oracle `pcre2test` 10.47, an 84-cell matrix). **17 rejects-valid spellings now ACCEPT**: (a) an initial `]` after invisibles (stray `\E` / empty `\Q\E`) is a LITERAL member — `[\E]x]` is the class `]x` (also `[\Q\E]]`, `[^\E]x]`, …); (b) a caret AFTER the negation is an ordinary member — `[^^]`, `[^\E^]`; (c) the negation caret is recognized THROUGH invisibles — `[\E^]x]`, `[\E^^]`. Conversely `[\E^]`/`[\Q\E^]` keep rejecting (the caret negates, the class is then empty) and in-class `\Q` is now ALWAYS the quote-opener (never a shorthand escape): `[\Q]`/`[\Qa]` reject at the grammar, err-106-faithfully. **AST correction (3 spellings)**: `[\E^a]`/`[\Q\E^a]`/`[\E^-z]` were accepted-but-mis-parsed as non-negated member lists; they are now `negated:true` with the opening invisibles dropped. Every other still-accepted pattern's AST is byte-identical (the `{kind:"char_class", negated, initial_close, body}` carrier unchanged). Grammar: `char_class` is a 3-alt split with `class_negated_open` (negation-through-invisibles) and visibility-led bodies (`class_body_nonempty(_nocaret)` — non-emptiness counts only VISIBLE members); the validator's substantive-item/`^`-skip machinery was DELETED (4th compile-contract migration; its class scanner now mirrors the grammar's open model — range/POSIX/escape-letter checks unchanged). Rejection LAYER for the migrated families moves from the contract message ("unterminated character class") to a grammar parse error — match on the **code** (`E_PARSE_FAILURE`), not message text. Both profiles behave identically on this surface. This also closes the standing `[\E]` generator-emits/parser-rejects duality signature at the source. Bug ledger: `REGEX-0091`. **Contract section:** "Release 1.1.84 / Contract 1.1.86 Highlights — REGEX-0091". Rules detail: `rules-char-class.md` (rewritten to the current typed truth); examples: `examples-char-class.md` → "The PCRE2 class-open model".

### 1.1.83 / Contract 1.1.85 — REGEX-0089/0090: verb & start-option ARGUMENT SHAPES reject PCRE2-faithfully; the checks migrated into the grammar

> **For RGX maintainers**: BEHAVIOR-TIGHTENING (release bump 1.1.82→1.1.83 / contract 1.1.84→1.1.85; **schema stays `1`**), PCRE2-convergent (`REGEX-PCRE2-FIDELITY.3.14`; oracle `pcre2test` 10.47). `directive_named` is now a per-name-class split — MARK (named or `(*:...)` shorthand) requires a non-empty `:`-argument (`(*:)` / `(*MARK)` / `(*MARK:)` reject, PCRE2 err-166 class); the other 7 verbs take `:`-suffix only (`(*SKIP=)` / `(*PRUNE=x)` reject, err-160 class; `(*PRUNE:)` stays accepted); the 4 `LIMIT_*` options REQUIRE `=digits`; every other start option is bare-only. Two accepts-invalid divergences fixed: **REGEX-0089** (PGEN accepted `=digits` on non-LIMIT options — `(*UTF=5)`, `(*CR=5)`, `(*TURKISH_CASING=5)` — and bare `(*LIMIT_HEAP)`; PCRE2 rejects all, err 160) and **REGEX-0090** (the host validator's start-option POSITION check skipped `=`-value forms — `a(*LIMIT_HEAP=500)` / `(*FAIL)(*LIMIT_HEAP=5)a` were wrongly accepted; the position rule stays validator-owned but now covers all forms). Every still-accepted pattern's AST is byte-identical (the `{kind:"named", name, payload}` carrier is unchanged). Rejection LAYER for the migrated shapes moves from the post-parse contract message to a grammar parse error — match on the **code** (`E_PARSE_FAILURE`), not message text. The tightening applies in BOTH profiles; `relaxed` still re-admits unrecognized names (incl. extended spellings like `(*LIMIT_HEAPX=5)`) with any suffix. Bug ledger: `REGEX-0089`/`REGEX-0090`. **Contract section:** "Release 1.1.83 / Contract 1.1.85 Highlights — REGEX-0089/0090". Rules detail: `rules-misc.md` § `directive_named`.

### 2026-07-07 maintenance (release 1.1.81 / Contract 1.1.83 — UNCHANGED) — DEFAULT-PROFILE.2: the strict-`pcre2`-by-default resolution is now grammar-declared (`@default_profile: pcre2`)

> **For RGX maintainers**: IMPLEMENTATION-ONLY, SURFACE-NEUTRAL (no version bump). The "unspecified profile = strict `pcre2`, `relaxed` is the opt-out" default was engine-hard-coded (`== "regex"` name literals in the parse registry / generation filter / embedding API); it is now declared in `grammars/regex.ebnf` via the grammar-level `@default_profile: pcre2` directive, and the generated parser carries its own default (`DEFAULT_GRAMMAR_PROFILE` constant; the constructor starts on it; `set_grammar_profile(None)` restores it). **You need to do nothing**: default accept/reject byte-identical (cert `198/198` `fully_certified` seeds 0/7/42; broader-corpus + differential-equivalence gates green), AST shapes/error codes/versions unchanged. See `rules-escape.md` "Profiles — strict default vs `relaxed`" (the new provenance note) and the contract's "Maintenance Update 2026-07-07" section.

### 1.1.82 / Contract 1.1.84 — REGEX-0088: quantified anchors reject PCRE2-faithfully (incl. escape anchors); the check migrated into the grammar

> **For RGX maintainers**: BEHAVIOR-TIGHTENING (release bump 1.1.81→1.1.82 / contract 1.1.83→1.1.84; **schema stays `1`**). PGEN previously ACCEPTED a direct quantifier on the 7 escape anchors — `\A* \b* \B? \G+ \z* \Z* \K*` + counted forms `\A{2}`/`\A{2,}`/`\A{2,3}`/`\A{,2}` — which `pcre2test` 10.47 rejects (err 109); found internally by a differential probe (the old host-validator check covered only `^`/`$`). All now REJECT. Fix is grammar-encoded (`REGEX-PCRE2-FIDELITY.3.13`): `anchor` is its own non-quantifiable `piece` branch (`anchor !quantifier` — the lookahead keeps non-quantifier braces literal: `${`/`\A{a}`/`\A{2`/`\A{}` still ACCEPT, oracle-exact), and `find_invalid_quantified_anchor` was DELETED from the host validator (2nd of the 10 compile-contract checks migrated). `^*`-class diagnostics change from the contract message to a standard grammar parse failure — match on **code** (`E_PARSE_FAILURE`), not text. Every still-accepted pattern's AST is BYTE-IDENTICAL (34/34 probe matrix; grouped `(?:^)*`, class `[$]*`/`[\b]`, POSIX aliases `[[:<:]]*`/`[[:>:]]+` all keep prior shapes + verdicts). Also folded in: `simple_escape`'s letter component is now a positive enumeration (same per-profile accept set; generation-faithful — see `rules-escape.md`). Bug ledger: `REGEX-0088`. **Contract section:** "Release 1.1.82 / Contract 1.1.84 Highlights — REGEX-0088". Examples: this book's anchors chapter (`examples-anchors.md` → "Quantified anchors reject").

### 2026-06-08 maintenance (release 1.1.81 / Contract 1.1.83 — UNCHANGED) — REGEX-SELF-HOSTING: the regex parser is now self-hosting (no Rust `regex`-crate dependency)

> **For RGX maintainers**: IMPLEMENTATION-ONLY, SURFACE-NEUTRAL (no version bump). `grammars/regex.ebnf` was converted to use ONLY native EBNF terminals — no `/.../` regex literals — so the generated `regex_parser.rs` contains zero `match_regex` calls and zero `regex::Regex` references (it no longer uses or links Rust's `regex` crate). Built on four parser-agnostic primitives: `builtin_any_char`/`builtin_ascii_char`, `$text`/`$0` (whole-match string; Perl5 `$0`), and `@transform`-on-span. **You need to do nothing**: accepted language is BYTE-IDENTICAL (verified against the `pcre2test` oracle at every step), AST shape unchanged (shape-contract locked), error codes unchanged, all versions unchanged. Recorded for integrators who care that the embedded regex parser has no regex-engine dependency. Only the regex parser is held to this bar; every other PGEN parser still uses Rust regex. See `welcome.md` "Implementation note: the regex parser is self-hosting" and the contract's "Maintenance Update 2026-06-08" section.

### 2026-06-07 maintenance (release 1.1.81 / Contract 1.1.83 — UNCHANGED) — REGEX-PCRE2-FIDELITY.3.2: verb/start-option NAME acceptance migrated into the grammar; `relaxed` re-admits arbitrary verb names

> **For RGX maintainers**: SURFACE-NEUTRAL (no version bump). `directive_name` split into a strict (default `pcre2`) ordered choice of exactly the recognized PCRE2 verb (`MARK ACCEPT F FAIL COMMIT PRUNE SKIP THEN`) + start-option (26) names and a `@profiles:["relaxed"]` catch-all. Default now rejects unrecognized verb names (`(*FOO)`, `(*MARKX)`, wrong-case `(*accept)`) BY GRAMMAR — conformance-neutral (rejected before via the host validator; `regex_pcre2_compile_oracle_gate` false-reject set byte-identical at 46). The validator's unrecognized-name reject was removed (so `relaxed` accepts arbitrary verb names); its STRUCTURAL checks (MARK needs an arg; start-options at pattern start; `=value` numeric; only `ACCEPT` quantifiable) stay and apply in both profiles. AST shape unchanged (no manifest change). See `rules-misc.md#directive_name`. Validator removal + embedding-API `relaxed` exposure are capstone `.4`/`.5`.

### 2026-06-07 maintenance (release 1.1.81 / Contract 1.1.83 — UNCHANGED) — REGEX-PCRE2-FIDELITY.3.1: `\u \U \F \l \L \i` rejection migrated into the grammar; new `relaxed` profile

> **For RGX maintainers**: SURFACE-NEUTRAL (no version bump). The six PCRE2-unsupported escape letters `\i \F \l \L \u \U` (and braced `\u{…}`) were moved from the out-of-band host validator (`find_invalid_escape_i`) INTO `grammars/regex.ebnf` (strict/relaxed variants of `simple_escape`/`class_simple_escape`/`class_range_literal_escape_letter` + profile-gated `unicode_escape`). Default (`pcre2`) accept/reject is **byte-identical** (`regex_pcre2_compile_oracle_gate` false-reject set unchanged; stash-baseline proven); AST shapes unchanged; diagnostic **code** still `E_PARSE_FAILURE` (message TEXT changed → match on code, not text). A new **`relaxed`** profile re-admits the six (CLI `--grammar-profile relaxed`); the embedding API still exposes only strict `regex_default`. See the escapes chapter "[Profiles — strict default vs `relaxed`](rules-escape.md#profiles--strict-default-vs-relaxed)" and the contract's "Maintenance Update 2026-06-07" section. Broader unrecognized-escape whitelist (`\I`, `\J`, …) tracked as REGEX-PCRE2-FIDELITY.3.11.

### 1.1.81 / Contract 1.1.83 — PGEN-RGX-0088: octal `>0o377` is mode-dependent — FIX2.3's blanket parse-time reject REVERTED (mode-agnostic emission)

> **For RGX maintainers**: `PGEN-RGX-0087-FIX2.3` (rel 1.1.80) made bare `\ddd` octal `>0o377` an unconditional parse-time hard-reject — PCRE2-faithful only for 8-bit non-UTF (err 151). Under **UTF/16/32-bit** `\777`=U+01FF is valid and `pcre2test` 10.47 ACCEPTs `\777`/`\400`/`\6666666666`/`[\666]`/`[\777]` under `,utf`. PGEN parses **mode-agnostically** ⇒ wrong locus. 1.1.81 **reverts exactly FIX2.3's two grammar edits** (`octal_escape_short_payload`→`/([0-7]{1,3})/`; `class_simple_escape`→FIX2.1 unguarded form; non-comment grammar diff vs 1.1.79 = **0**, pure revert): PGEN emits the octal atom for any `\ddd` mode-agnostically; the 8-bit `>0o377` reject is the **mode-aware consumer's** range check (report-prescribed; `feedback_ast_pipeline_parser_agnostic`, not a workaround). FIX2.1/.2 + RGX-0087-backref + RGX-0084 byte-identical (FIX2.3-independent). **Schema stays `1`**. `regex` lib 105/0, cross-parser 8/0, drift gate green @ 1.1.81/1.1.83. RGX adopted 1.1.80 (12,805/5→12,806/4) & rebaselined; this closes testinput10:218 `/\777/,utf` → **12,807/3**. `(?u)` inline modifier is a separate pre-existing gap. Bug ledger: `REGEX-0087` (downstream `PGEN-RGX-0088`; `PGEN-RGX-0087` stays CLOSED). **Contract section:** "Release 1.1.81 / Contract 1.1.83 Highlights — PGEN-RGX-0088". Example: this book's escapes chapter.

### 1.1.80 / Contract 1.1.82 — PGEN-RGX-0087 FIX2.3: octal escape `>\377` overflow now rejects (PCRE2-faithful); `PGEN-RGX-0087` CLOSED

> **For RGX maintainers**: the last `PGEN-RGX-0087` residual. PGEN's bare `\ddd` octal payload `/([0-7]{1,3})/` had no range check, so `\6666666666` (testinput9:287), `\400`, `\666`, `\777`, `\7777` and class `[\666]`/`[\400]` were ACCEPTed where PCRE2 10.47 **hard-errors** (err 151, octal `>\377` in 8-bit non-UTF — in **both** pattern-body & `[...]` context; PCRE2 does not truncate a `>0o377` triple). Fix (grammar-only): `octal_escape_short_payload` split — a 3-octal-digit run valid only if `[0-3]`-led (≤0o377), a 1-2-digit run only if octal-complete (proven `!"0"…!"7"` lookahead idiom, `-> $1` per branch ⇒ `octal_escape`'s `digits:$1` unchanged); FIX2.1's `class_simple_escape` gains `!"0"…!"7"` octal-digit guards (NOT 8/9) so an octal-overflow `\<octal-digit>` is not class-shorthand-rescued while `\8`/`\9` stay (FIX2.1 preserved). **Empirical `--parse-dump-ast-pretty` byte-identical proof** pre vs post for the entire RGX-0084 octal family + `\199`@0g/`\3777`/`\10`@9g/`(a)\12`/`\7`-backref + FIX2.1 `[\8]`/`[\9]`/`[\88]`/`[\377]` + RGX-0087 `\81`@8g — ALL unchanged; only the previously-wrongly-accepted overflow set now REJECTs (`\3777`→`\377`+lit`7`). No new shape ⇒ **schema stays `1`**. regex lib 104/0, cross-parser 8/0, drift gate green @ 1.1.80/1.1.82. **`PGEN-RGX-0087` now fully resolved & closed** — all FIX2 sub-leaves done; RGX PCRE2 ratchet reaches the report's full target **12,807/3**. Braced `\o{}` overflow (err 134, distinct production, never reported) out of scope (RGX-0079 owns `\o{}`). Bug ledger: `REGEX-0086`. **Contract section:** "Release 1.1.80 / Contract 1.1.82 Highlights — PGEN-RGX-0087 FIX2.3". Example: this book's escapes chapter (`examples-escapes.md` → "PGEN-RGX-0087" FIX2.3 note).

### 1.1.79 / Contract 1.1.81 — PGEN-RGX-0087 FIX2: scope the `[89]`-leading hard-reject to non-character-class context (rel-1.1.78 was over-broad)

> **For RGX maintainers**: release 1.1.78's `[89]`-leading multi-digit hard-reject (next entry) was **over-broad** — its `simple_escape` digit-guard also fired **inside `[...]` character classes** (single class-member escapes route via `class_escape = escape`). A character class has no back-references, so `\8`/`\9`/`\<digit>` there are octal/literal and PCRE2 (oracle: `pcre2test` 10.47) **ACCEPTs** them; PGEN 1.1.78 wrongly `E_PARSE_FAILURE`d `[\8]`/`[\9]`/`^[A\8B\9C]+$`/`[\88]` (the class-*range* path's separate already-unguarded `class_range_simple_escape` still accepted `[\8-\9]` ⇒ inconsistency). Net RGX PCRE2 ratchet **−4** (12,805/5 → 12,801/9), not adoptable. **1.1.79 scopes it:** `class_escape` gets its own `class_escape_unit` (mirroring the proven `class_range_escape_unit` precedent) with an UNGUARDED `class_simple_escape` (the pre-RGX-0087 form) — class context restored **byte-identical to pre-1.1.78** (same `{escape,kind:"shorthand",char}` shape, no new vocab, **schema stays `1`**). The non-class `[89]`-leading hard-reject and the `[1-7]`-led octal-degrade are **untouched** (grammar diff confined to the `class_escape` block; `\81`/`\82`/`\91`@8g, `\89`/`\80`@0g, `(x)\81`@1g still REJECT; `\199`/`\10` still ACCEPT — all verified). All 6 class-context cases now ACCEPT (PCRE2 10.47-matched). **Known residual (sub-leaved `RGX-0087-FIX2.3`; `PGEN-RGX-0087` stays open):** `(?i:A{1,}\6666666666)` (testinput9:287) — PGEN's `octal_escape` has no PCRE2 octal `>\377` (8-bit) range check, ACCEPTed where PCRE2 rejects (err 151); distinct mechanism (octal range, not the class/backref scoping) touching the RGX-0084 octal path. Release nonetheless net-positive & adoptable (12,801/9 → 12,806/4, > pre-1.1.78 12,805/5; original 4671/4674 stay closed). Bug ledger: `REGEX-0086` (downstream `PGEN-RGX-0087`). **Contract section:** "Release 1.1.79 / Contract 1.1.81 Highlights — PGEN-RGX-0087 FIX2". Behaviour + example: this book's escapes chapter (`examples-escapes.md` → "PGEN-RGX-0087" + its FIX2 class-context note).

### 1.1.78 / Contract 1.1.80 — PGEN-RGX-0087 fix: `[89]`-leading multi-digit escape hard-rejects (PCRE2-faithful), not a degrade-resplit

> **For RGX maintainers**: family-linked residual of `PGEN-RGX-0084` (the `\NN…` octal-vs-backref disambiguation — that fix stays closed & correct; this is a separate sub-family it did not cover, **not a reopen**). An `[89]`-leading **multi-digit** escape that is not a valid full back-reference (e.g. `((((((((x))))))))\81` — 8 groups) used to be **accepted**: the GATED multi-digit rule's predicate failed and the PEG backtracked, re-splitting `\81` into single-digit back-reference `\8` + literal `1` (and even guarded, the `simple_escape` catch-all would consume `\8` as a shorthand). `\8`/`\9` are **not octal digits** so PCRE2's octal fallback is unavailable and PCRE2 (authoritative oracle: `pcre2test` 10.47, the version family RGX vendors) **rejects** the pattern at compile (error 115, reference to non-existent subpattern). PGEN now likewise **hard-rejects** (`E_PARSE_FAILURE`). `[1-7]`-leading runs that fail the GATED branch still degrade to octal — `\199`@0-groups → `\x01` + "99", `\10`@9-groups → octal — **byte-identical** to the RGX-0084 (`REGEX-0083`) behaviour; single-digit `\1`…`\9` (`N<10` Non-Goal) **unchanged**. Accept-set tightening + one corrected classification (`\199`@0-groups was wrongly a numeric back-reference, now the correct octal escape); **no new shape ⇒ schema stays `1`** (release-only bump — same category as 1.1.76/1.1.77). Fix is grammar-level (two negative-lookahead guards in `grammars/regex.ebnf`, the proven RGX-0079 idiom), parser-agnostic; engine/codegen untouched. **The `#[ignore]`d RGX unit test asserting `\89`@0-groups → literal "89" is wrong vs PCRE2 10.47 (which errors 115); re-enable it expecting REJECT** (its `\199` → `\x01`+"99" second assertion is spec-correct and now matches). Bug ledger: `REGEX-0086` (downstream `PGEN-RGX-0087`). **Contract section:** `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` → "Release 1.1.78 / Contract 1.1.80 Highlights — PGEN-RGX-0087". Behaviour + example: this book's escapes chapter (`examples-escapes.md` → "PGEN-RGX-0087").

### 1.1.77 / Contract 1.1.79 — PGEN-RGX-0085 fix: regex parenthesis-nesting ceiling (no more host SIGABRT on deeply nested patterns)

> **For RGX maintainers**: a deeply nested pattern (`(((…(a)*…)*)*`) used to overflow the thread stack and **hard-abort the host process** (SIGABRT, uncatchable) instead of returning a recoverable error — the worst failure mode for a parsing library and a trivial DoS vector. Fixed by a configurable **parenthesis-nesting ceiling** = **250** (exact PCRE2 `PCRE2_CONFIG_PARENSLIMIT` / Rust `regex` `nest_limit` default), enforced at the embedding-API boundary *before* the recursive-descent parser is ever invoked: a pattern whose `(`-group nesting exceeds 250 now returns a clean `E_PARSE_FAILURE` `ParseDiagnostic` whose `location` points at the `(` that crossed the limit (PCRE2's "parentheses are too deeply nested" model). **No behaviour change for any pattern within the limit** — the AST/dump is byte-identical; only previously-*aborting* input now returns a clean error ⇒ strictly-more-graceful, **schema stays `1`** (release-only bump, no schema change — same category as 1.1.76). The global engine recursion guard is untouched (SV/VHDL unaffected). Bug ledger: `REGEX-0084` (downstream `PGEN-RGX-0085`). **Contract section:** `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` → "Release 1.1.77 / Contract 1.1.79 Highlights — PGEN-RGX-0085". Behaviour + example: this book's groups chapter (`rules-groups.md` → "Parenthesis nesting limit").

### 1.1.76 / Contract 1.1.78 — PGEN-RGX-0084 fix: bare `\NN…` octal-vs-backreference PCRE2 disambiguation at parse time

> **For RGX maintainers**: bare numeric `\N…` was an *unconditional* numeric backreference. Now PCRE2-compliant **at parse time**: single-digit `\1`…`\9` (N<10) is always a numeric back reference (**unchanged**); a two+-digit `\NN…` (N≥10) is a back reference only when ≥ N capturing groups (plain *or* named) were opened *up to that source position*, otherwise it re-splits to an octal/literal escape. AST shape is **byte-identical** (`{type:"backreference",kind:"numeric",index:N}` / `{type:"escape",kind:"octal",digits:"…"}` unchanged); only the *classification* of a previously-misclassified two+-digit `\NN…` changes. No new shape vocabulary ⇒ **schema stays `1`** (atom re-classification — same category as REGEX-0002/0004; consumers pinned to "all `\NNN` are backreferences" should repin to this release). Bug ledger: `REGEX-0083` (downstream `PGEN-RGX-0084`). **Contract section:** `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` → "Release 1.1.76 / Contract 1.1.78 Highlights — PGEN-RGX-0084". Worked-family table: this book's escapes chapter (`examples-escapes.md`).

### 1.1.75 / Contract 1.1.77 — PGEN-RGX-0081 + 0082 fixes: typed shape regressions surfaced by RGX walker migration

> **For RGX maintainers**: this section explains the two AST-shape bugs that landed during the slice-11/12/13 (named-ref family) and code_block typing slice campaigns, and the post-fix shapes you should walk against. Both fixes are additive in spirit — schema stays at `1`, accept set unchanged, only the `kind` discriminator and one positional-ref change.

#### TL;DR for RGX

| Bug | Pre-fix shape | Post-fix shape | RGX impact pre-fix |
|---|---|---|---|
| **0081** `\g`-prefixed | All 5 forms → `kind:"subroutine"` | Bracket form → 4 new kinds | 8+ conformance tests + `tests::g_bracketed_is_subroutine_call_not_backref` regression |
| **0082** `code_block_lang` | `content:[]` (callback name dropped) | `content:[<body chars>]` | 8+ `full_mode_native_*` tests; `register_native` callback resolution broken |

Verify your walker against the 10-pattern matrix below + the post-fix `content` shape on `(?{native:NAME})`.

#### PGEN-RGX-0081 — `\g`-prefixed bracket-form distinction restored

**Background.** Slices 11+12+13 of the regex annotation campaign (parser releases 1.1.41–43) progressively typed the named-ref family, `subroutine_ref` cleanup, and `signed_digits`. The end state collapsed every `\g`-prefixed reference under a single `kind:"subroutine"` shape, irrespective of whether the source used angle (`\g<...>`), apostrophe (`\g'...'`), brace (`\g{...}`), or bare-digit (`\gN`) brackets.

**Why this matters.** PCRE2's `pcre2pattern(3)` § "Subroutine references and recursive patterns" has clear bracket-form-determines-semantic rules:

- `\g<NAME>` / `\g'NAME'` — **subroutine call** (re-execute the group; semantic ≠ back-reference)
- `\g<N>` / `\g'N'` — **subroutine call** (re-execute group N)
- `\g{NAME}` — **back-reference**
- `\g{N}` — **back-reference**
- `\gN` — **back-reference**

Pre-fix, every form produced the same `kind:"subroutine"` shape, so RGX couldn't faithfully lower angle-vs-brace. RGX's heuristic workaround (dispatch by `ref` shape: string → recursion, object → back-ref) got common cases right but mis-handled `\g<N>` (numeric subroutine call routed to back-reference instead of recursion).

**The fix.** Split the single `"\\g" subroutine_ref -> {kind: "subroutine"...}` branch into 7 sub-branches, each pinned to a specific bracket form with its own kind:

```ebnf
backreference = "\\" backreference_digits                              -> {type: "backreference", kind: "numeric",                index: $2}
              | "\\k" name_ref                                          -> {type: "backreference", kind: "named",                  ref:   $2}
              | "\\k" braced_name_ref                                   -> {type: "backreference", kind: "named_braced",           ref:   $2}
              | "\\g" "<" name ">"                                      -> {type: "backreference", kind: "subroutine_named",       ref:   $3}
              | "\\g" "<" signed_digits ">"                             -> {type: "backreference", kind: "subroutine_numeric",     ref:   $3}
              | "\\g" "'" name "'"                                      -> {type: "backreference", kind: "subroutine_named",       ref:   $3}
              | "\\g" "'" signed_digits "'"                             -> {type: "backreference", kind: "subroutine_numeric",     ref:   $3}
              | "\\g" "{" brace_ws? name brace_ws? "}"                  -> {type: "backreference", kind: "named_braced",           ref:   $4}
              | "\\g" "{" brace_ws? signed_digits brace_ws? "}"         -> {type: "backreference", kind: "numeric_backreference",  ref:   $4}
              | "\\g" signed_digits                                     -> {type: "backreference", kind: "numeric_backreference",  ref:   $2}
```

**4 new kinds**:
- `subroutine_named` — `\g<NAME>` / `\g'NAME'` (subroutine call, named)
- `subroutine_numeric` — `\g<N>` / `\g'N'` (subroutine call, numeric)
- `numeric_backreference` — `\g{N}` / `\gN` (numeric back-reference)
- (existing) `named_braced` — `\k{NAME}` AND `\g{NAME}` route here (semantically identical per PCRE2 spec)

**Empirical post-fix matrix** (verified via `parseability_probe --parse-dump-ast-pretty regex .../pattern.txt --profile regex_default`):

| Pattern | Pre-fix kind | Post-fix kind | Post-fix `ref` |
|---|---|---|---|
| `\g<n>`  | `subroutine` | `subroutine_named` | `"n"` |
| `\g'n'`  | `subroutine` | `subroutine_named` | `"n"` |
| `\g<1>`  | `subroutine` | `subroutine_numeric` | `{"sign":[],"value":1}` |
| `\g'1'`  | `subroutine` | `subroutine_numeric` | `{"sign":[],"value":1}` |
| `\g{n}`  | `subroutine` | `named_braced` | `"n"` |
| `\g{1}`  | `subroutine` | `numeric_backreference` | `{"sign":[],"value":1}` |
| `\gN`    | `subroutine` | `numeric_backreference` | `{"sign":[],"value":1}` |
| `\k<n>`  | `named` | `named` (unchanged) | `"n"` |
| `\k{n}`  | `named_braced` | `named_braced` (unchanged) | `"n"` |
| `\1`     | `numeric` | `numeric` (unchanged) | `index:1` |

**Consumer dispatch recipe** (Rust):

```rust
match obj["kind"].as_str().unwrap() {
    "numeric" => {
        // \1, \2, ... — bare numeric backref
        let index = obj["index"].as_u64().unwrap();
        process_numeric_backref(index)
    }
    "named" => {
        // \k<NAME>, \k'NAME'
        let name = obj["ref"].as_str().unwrap();
        process_named_backref(name)
    }
    "named_braced" => {
        // \k{NAME} OR \g{NAME} — both back-references per PCRE2
        let name = obj["ref"].as_str().unwrap();
        process_named_backref(name)
    }
    "subroutine_named" => {
        // \g<NAME>, \g'NAME' — subroutine call
        let name = obj["ref"].as_str().unwrap();
        process_subroutine_call_named(name)
    }
    "subroutine_numeric" => {
        // \g<N>, \g'N' — subroutine call
        let n = obj["ref"]["value"].as_u64().unwrap();
        process_subroutine_call_numeric(n)
    }
    "numeric_backreference" => {
        // \g{N}, \gN, \g+N, \g-N — numeric back-reference
        let n = obj["ref"]["value"].as_u64().unwrap();
        let sign = obj["ref"]["sign"].as_str();  // "+", "-", or empty
        process_numeric_backref_signed(sign, n)
    }
    _ => unreachable!()
}
```

**Regression-lock test:** `regex_parser_pgen_rgx_0081_g_prefixed_backref_preserves_bracket_form` in `rust/src/embedding_api.rs` pins all 10 pattern → kind mappings.

#### PGEN-RGX-0082 — `code_block_lang` content drop fixed (off-by-one positional ref)

**Background.** When the `code_block` typing slice (atom-subtree slice 24, parser release `1.1.54`) typed the two code-block forms, the `code_block_lang` annotation referenced `content: $4`. The 6-element rule:

```ebnf
code_block_lang = "(?{" code_lang ":" ws? code_content "})"
                  -> {type: "atom", kind: "code_block", lang: $2, content: $4}    // ⚠ $4 is `ws?`!
```

has positions $1=`"(?{"`, $2=`code_lang`, $3=`":"`, $4=`ws?`, $5=`code_content`, $6=`"})"`. The annotation's `$4` referenced the optional whitespace slot, not the actual `code_content`. For `(?{native:NAME})` patterns the typed AST emitted `{kind:"code_block", lang:"native", content:[]}` — the callback name was silently dropped. The Perl-style `(?{ NAME })` form (no `lang:` prefix) routed to `code_block_plain` and worked correctly because that branch's `content: $2` correctly referenced `code_content`.

**Why this matters for RGX.** RGX's `Regex::register_native(NAME, callback)` API resolves the callback name from the typed AST's `content` field. With `content:[]`, the lookup found nothing and dispatch silently produced no match — `regex.is_match(...)` returned `false` regardless of whether the callback would have matched. 8+ `full_mode_native_*` regression tests in `rgx-core/src/lib.rs` failed silently after the PGEN bump.

**The fix.** Changed `content: $4` → `content: $5`:

```ebnf
code_block_lang = "(?{" code_lang ":" ws? code_content "})"
                  -> {type: "atom", kind: "code_block", lang: $2, content: $5}    // ✓
```

**Empirical post-fix matrix:**

| Pattern | Pre-fix `content` | Post-fix `content` |
|---|---|---|
| `(?{native:check_env})` | `[]` | `["c","h","e","c","k","_","e","n","v"]` |
| `(?{native:my_callback_42})` | `[]` | per-char array of `my_callback_42` |
| `(?{native:})` | `[]` | `[]` (correct — empty body) |
| `(?{lua: print(1)})` | `[]` | `[" ","p","r","i","n","t","(","1",")"]` |
| `(?{ check_env })` (no `lang:`) | `[" ","c","h","e","c","k","_","e","n","v"," "]` | unchanged |

**Consumer dispatch recipe:**

```rust
fn extract_code_block(atom: &Value) -> Option<(Option<String>, String)> {
    let obj = atom.as_object()?;
    if obj.get("kind")?.as_str()? != "code_block" {
        return None;
    }
    let lang = obj.get("lang")?.as_str().map(String::from);
    let content_arr = obj.get("content")?.as_array()?;
    // content is per-char strings — concat to recover the code body
    let body: String = content_arr.iter()
        .filter_map(|v| v.as_str())
        .collect();
    Some((lang, body))
}
```

**Regression-lock test:** `regex_parser_pgen_rgx_0082_code_block_lang_preserves_content` in `rust/src/embedding_api.rs` pins 4 patterns including the `(?{lua:...})` audit case.

#### Schema, accept set, contract bumps

- **Schema version**: stays at `1` (additive; same accept set).
- **Annotation inventory**: 142 entries (was 138). +4 from 0081's 4 → 10 split.
- **Parser release**: `1.1.74` → `1.1.75`.
- **Contract version**: `1.1.76` → `1.1.77`.
- **Bug ledger**: `REGEX-0081`, `REGEX-0082` → "Released" in 1.1.75.

#### Verification commands RGX can run on its own host

```bash
# Build PGEN's parseability_probe with the regex backend:
cd subs/pgen/rust
cargo build --release --features generated_parsers --features ebnf_dual_run \
    --bin parseability_probe

# Test 0081: probe each pattern + check the typed `kind` discriminator.
for p in '\\g<n>' '\\g{n}' '\\g<1>' '\\g{1}' '\\g1' '\\g'\\''n'\\''' '\\g'\\''1'\\''' '\\k{n}' '\\k<n>' '\\1'; do
    echo -n "$p" > /tmp/p.txt
    ./target/release/parseability_probe --parse-dump-ast-pretty regex /tmp/p.txt /tmp/ast.json --profile regex_default
    jq '.. | objects | select(.type=="backreference") | {kind, ref}' /tmp/ast.json
done

# Test 0082: probe (?{native:check_env}) and confirm content is non-empty.
echo -n '(?{native:check_env})' > /tmp/p.txt
./target/release/parseability_probe --parse-dump-ast-pretty regex /tmp/p.txt /tmp/ast.json --profile regex_default
jq '.. | objects | select(.kind=="code_block") | {lang, content}' /tmp/ast.json
```

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.75 / Contract 1.1.77 Highlights" — full annotation source + parallel matrices.

### 1.1.74 / Contract 1.1.76 — PGEN-RGX-0078 verification + Optim #14 / #15: embedding-API dispatch overhead eliminated

> **For RGX maintainers**: this section explains what changed, why your reported 360x figure no longer holds at HEAD, and what the closure picture looks like now. Read top-to-bottom — the methodology corrections matter for how you should interpret future ratio runs.

#### TL;DR for RGX

- **Your reported geomean PGEN/PCRE2-no-JIT compile ratio was ~360x at PGEN pin `056f6784` (release `1.1.40`, default macOS allocator).** That number was correct for that pin and that methodology, but it does not survive at current HEAD (`1.1.74` / Contract `1.1.76`).
- **Two methodology issues** in the report's `pgen_iteration_flow/` cause the reported ratio to bias HIGH against PGEN (we explain both below).
- **Direct-parser-path geomean ratio at HEAD: ~70x.** That's a 5x compression from the 360x in your report, delivered by Optim #1–#13 + mimalloc that all landed between `1.1.40` and `1.1.73`.
- **Embedding-API-path geomean ratio at HEAD: ~80x** after Optim #14 + #15 (this release). That's a **further 4x compression** of the embedding-API path specifically — driven by eliminating per-call OS-thread-spawn dispatch overhead, not by any parser-side change.
- **Combined: ~360x → ~80x = ~4.5x compression.** Still ~16x over the `<5x` ROADMAP target you cited; that residual gap is structural (PGEN's general-purpose EBNF-driven codegen vs PCRE2's hand-tuned C) and would need either specialised regex codegen or sustained parser-internal hot-path work.

#### Methodology issue #1 — the report's PCRE2 numbers are batch-mean, not p50

The `pcre2_compile_baseline.c` and `pcre2_compile_jit_baseline.c` programs in `pgen-issues/artifacts/PGEN-RGX-0078/pgen_iteration_flow/` measure each pattern like this:

```c
// pcre2_compile_baseline.c:14-23 (RGX-supplied iteration flow)
clock_gettime(CLOCK_MONOTONIC, &t0);
for (int i = 0; i < BATCH; i++) {
    pcre2_code *re = pcre2_compile(...);
    if (re) pcre2_code_free(re);
}
clock_gettime(CLOCK_MONOTONIC, &t1);
long long ns = (t1.tv_sec - t0.tv_sec) * 1000000000LL + (t1.tv_nsec - t0.tv_nsec);
long long avg = ns / BATCH;
printf("%-22s %10lld ns/compile  (%d-compile batch took %lld ns)\n", name, avg, ...);
```

That's `total_elapsed / 10000` — the **arithmetic mean** of 10000 compiles in a batch, not the median (p50). The report's `measurements/ratio_table.md` column header *"PCRE2 no-JIT p50"* is mislabeled — the underlying number from `pcre2_compile_p50.txt` is a mean.

PGEN's side, by contrast, measures true p50 of 5000 individual `Instant::now()` samples. Mixing p50 / mean in a ratio is apples-to-oranges, and the bias goes consistently in one direction:

- **Batch-mean tends to be lower than per-call p50 for compile-like operations** because the batch amortizes allocator hiccups, page faults, branch-predictor warmup, and TLB pressure across 10000 calls. Once the compile path is hot, every subsequent compile in the batch hits warm caches.
- A per-call p50 measured the same way PGEN's side is measured (individual `clock_gettime` brackets per call) would land somewhere higher than the batch-mean number, and the ratio would correspondingly be lower.

We have not re-measured PCRE2 compile per-call p50 on this host (would have required adding a `pcre2` Rust crate dep + libpcre2-8 install — out of scope for this slice). The ~70x / ~80x ratios reported below use RGX's PCRE2 batch-mean numbers as published, and are therefore biased in PGEN's *disfavor* compared to a true p50/p50 comparison. The "real" per-call ratios are likely 10–30% lower still.

**Recommendation for RGX**: when you next re-run the iteration flow, change `pcre2_compile_baseline.c` to time each `pcre2_compile()` call individually and take p50, not batch-mean. Or change the PGEN side to measure batch-mean. Either pick — but **both sides must use the same statistic** for the ratio to be meaningful.

#### Methodology issue #2 — the report's PGEN baseline is 13 perf optims behind HEAD

`measurements/host_metadata.txt` records PGEN pin `056f67842955dd4271cbf6fc6158f9cb9c64002e` (release `1.1.29`, contract `1.1.31`, integration contract document `1.1.40 / 1.1.42`). That's where the 92–377µs PGEN parse times you measured came from.

Between that pin and current HEAD (`1.1.74`), thirteen perf optimisations have landed against PGEN-RGX-0073:

| Optim | What it did | Where to look |
|-------|-------------|---------------|
| #1–#7 | Earlier perf wins | `git log --grep "Optim #[1-7]"` |
| #6 | rustc-hash (FxHash) for the parser memo HashMap | rustc-hash 2.1 dep added |
| #8 | Cached `logger.is_enabled()` inside `try_parse` closures | every rule call sees this |
| #9 | Borrow `Regex` in place; eliminate clone-per-match | regex matcher hot path |
| #10 | Opt-in mimalloc as global allocator for the perf probe | `mimalloc_perf` cargo feature |
| #11 | Fast-path semantic-runtime wrapper for grammars without predicates | regex grammar takes the fast path |
| #12 | Eliminate redundant outer clone in `$N` extraction | annotation-system codegen |
| #13 | Recovery (mimalloc re-enabled correctly in the perf probe) | `regex_perf_probe` feature wiring |

Cumulative effect on the **direct-parser-path** (`RegexParser::new(...).parse_full_regex()`) measured on Apple M4 Pro / mimalloc / current HEAD / 5000 samples / 200 warmup:

| Pattern | RGX-reported PGEN p50 (`1.1.40`, default alloc) | PGEN p50 at HEAD (mimalloc, direct path) | PGEN-side speedup |
|---|---:|---:|---:|
| literal_simple   |  92,042 ns |  13,750 ns | 6.7x |
| digit_sequence   | 216,833 ns |  33,334 ns | 6.5x |
| character_class  | 266,584 ns |  52,083 ns | 5.1x |
| alternation      | 119,709 ns |  27,958 ns | 4.3x |
| capture_groups   | 265,000 ns |  62,334 ns | 4.3x |
| url_simple       | 196,292 ns |  29,041 ns | 6.8x |
| email_basic      | 213,292 ns |  46,458 ns | 4.6x |
| anchor_complex   | 377,209 ns |  98,875 ns | 3.8x |

So even before this release's Optim #14/#15, the direct-parser-path geomean ratio against the PCRE2-no-JIT batch-mean was already down to ~70x — a 5x compression from the report's 360x.

**However**, RGX's iteration flow doesn't measure the direct-parser-path. It measures the **embedding-API path** (`pgen::embedding_api::parse_grammar_profile_named("regex", "regex_default", input)`), per `pgen_pcre2_compile_ratio.rs:79-85`. And that path was carrying a structural overhead that Optim #1–#13 didn't touch. Which brings us to:

#### Where the rest of the 360x came from — the embedding-API per-call thread spawn

The embedding-API path includes a defensive thread spawn that the direct path doesn't:

```rust
// rust/src/embedding_api.rs:run_generated_regex_on_dedicated_stack — pre-Optim-#14
fn run_generated_regex_on_dedicated_stack<T, F>(input: &str, f: F) -> Result<T, ParseDiagnostic> {
    let owned_input = input.to_string();
    let handle = std::thread::Builder::new()
        .name("pgen-generated-regex".to_string())
        .stack_size(GENERATED_REGEX_WORKER_STACK_BYTES)  // 64 MB
        .spawn(move || f(owned_input))
        .map_err(...)?;
    handle.join().map_err(|_| parse_failure_diagnostic("..."))?
}
```

Every call to `parse_grammar_profile_named("regex", "regex_default", ...)` spawned a fresh 64MB-stack OS thread, ran the parser there, and joined. The 64MB safety margin defends against deeply nested PCRE2 conformance patterns + serde_json::Value's recursive `Drop` on the parser's intermediate AST (commit `2311eb1`, "Embedding API tests: align regex integration test stack with production").

OS-thread spawn on macOS arm64 is ~50–100µs depending on system load. Paid on EVERY parse call, regardless of input depth. Even on a 4-byte input like `"test"` where the actual parse takes ~13.7µs, the dispatch was ~76µs — **5.5x the parse work itself**.

Measured per-pattern dispatch overhead at HEAD on the embedding-API path (before Optim #14):

| Pattern | Direct path p50 | Embedding-API p50 (pre-#14) | Per-call dispatch overhead |
|---|---:|---:|---:|
| literal_simple   |  13,750 |  90,292 |  76,542 ns |
| digit_sequence   |  33,334 | 193,542 | 160,208 ns |
| character_class  |  52,083 | 234,792 | 182,709 ns |
| alternation      |  27,958 | 111,834 |  83,876 ns |
| capture_groups   |  62,334 | 230,167 | 167,833 ns |
| url_simple       |  29,041 | 170,125 | 141,084 ns |
| email_basic      |  46,458 | 186,750 | 140,292 ns |
| anchor_complex   |  98,875 | 291,792 | 192,917 ns |

The dispatch overhead **dominated 60–85% of the embedding-API number on shallow patterns**. So the ~317x embedding-API geomean ratio at HEAD (pre-#14) decomposed into:

- **~70x real parser cost** (direct-path geomean) +
- **~250x per-call thread-spawn overhead** (structurally unrelated to parser performance)

This is why RGX's report came in at ~360x even though the underlying parser had already been compressed 5x by Optim #1–#13: those optims fixed the parser, but the embedding-API dispatch was untouched and dominated the path RGX was measuring.

#### What Optim #14 (this release) did

Replaced the per-call `std::thread::Builder::spawn(...).join()` with a single long-lived 64MB-stack worker thread + mpsc channel:

```rust
// rust/src/embedding_api.rs — post-Optim-#14
fn generated_regex_worker() -> &'static GeneratedRegexWorker {
    static WORKER: std::sync::OnceLock<GeneratedRegexWorker> = std::sync::OnceLock::new();
    WORKER.get_or_init(|| {
        let (sender, receiver) = std::sync::mpsc::channel::<GeneratedRegexJob>();
        std::thread::Builder::new()
            .name("pgen-generated-regex".to_string())
            .stack_size(GENERATED_REGEX_WORKER_STACK_BYTES)
            .spawn(move || {
                while let Ok(job) = receiver.recv() {
                    job();
                }
            })
            .expect("...");
        ...
    })
}
```

The worker is lazily initialised on first use via `OnceLock` and lives for the process lifetime. Each parse call sends a closure + sync_channel for the result, and blocks on the result. Panics are caught via `catch_unwind` so a malformed-input panic doesn't poison the worker for subsequent parses.

Per-call cost drops from ~50–100µs (thread spawn + join) to ~6–50µs (channel send + condvar wakeup). The 64MB stack guarantee is preserved — the dedicated worker has it once, amortized.

Behavior change RGX should know about: **parses are now SERIALIZED through the single cached worker** (vs previously parallel via thread-per-call). For RGX's typical use case (one parse at a time per `Regex::compile` call site) this is fine. For multi-threaded callers needing concurrent deep-pattern parses, the worker pool can be expanded as a follow-up — the per-call setup cost is gone regardless of pool size.

`stacker::maybe_grow` was investigated as a true zero-dispatch alternative (run inline on caller's stack with psm-based growth) but its stack-headroom heuristic does not appear to grow segments on cargo test runner threads under Rust 1.95 / macOS arm64 — deeply nested success-sample tests overflow even with `new_stack_size = 64MB`. Tracked as a follow-up; the cached-worker design is the conservative fallback.

#### What Optim #15 (this release) did

Added a per-call depth heuristic that bypasses the worker entirely for shallow inputs:

```rust
const GENERATED_REGEX_INLINE_DEPTH_THRESHOLD: usize = 16;

fn estimate_max_grouping_nesting(input: &str) -> usize {
    let mut depth = 0usize;
    let mut max_depth = 0usize;
    let mut bytes = input.bytes();
    while let Some(b) = bytes.next() {
        match b {
            b'\\' => { let _ = bytes.next(); }  // skip escaped byte
            b'(' | b'[' => { depth += 1; if depth > max_depth { max_depth = depth; } }
            b')' | b']' => { depth = depth.saturating_sub(1); }
            _ => {}
        }
    }
    max_depth
}

fn run_generated_regex_on_dedicated_stack<T, F>(input: &str, f: F) -> Result<T, ParseDiagnostic> {
    if estimate_max_grouping_nesting(input) <= GENERATED_REGEX_INLINE_DEPTH_THRESHOLD {
        return f(input.to_string());  // inline on caller's stack, zero dispatch
    }
    // Otherwise use the cached worker thread (defined above).
    ...
}
```

The PGEN-RGX-0073 8-pattern bench corpus has max grouping depth 4, so all 8 patterns take the inline path. The integration contract's stress samples (`nested_capturing_groups_50`, `deep_nested_backreference_80`) exceed the threshold and continue to use the cached worker thread for the 64MB safety margin. Both deeply nested success-sample tests confirmed passing post-change.

#### Combined post-Optim-#14+#15 numbers (Apple M4 Pro / mimalloc / current HEAD / 5000 samples / 200 warmup)

Embedding-API path (the path RGX measures), p50 in nanoseconds:

| Pattern | Pre-#14 | Post-#14 | Post-#15 (HEAD) | PCRE2 batch-mean (RGX) | Ratio at HEAD |
|---|---:|---:|---:|---:|---:|
| literal_simple   |  90,292 |  20,208 |  14,667 |   346 |  42x |
| digit_sequence   | 193,542 |  44,833 |  37,500 |   682 |  55x |
| character_class  | 234,792 |  65,125 |  61,375 | 1,156 |  53x |
| alternation      | 111,834 |  38,708 |  32,542 |   463 |  70x |
| capture_groups   | 230,167 |  77,042 |  74,833 |   685 | 109x |
| url_simple       | 170,125 |  39,042 |  33,833 |   370 |  91x |
| email_basic      | 186,750 |  58,167 |  50,042 |   389 | 129x |
| anchor_complex   | 291,792 | 116,292 | 116,041 |   766 | 151x |
| **geomean**      | **~218,000** | **~57,000** | **~50,000** | **~605** | **~80x** |

Same patterns measured against PCRE2-with-JIT batch-mean: geomean ratio ~17x at HEAD (was ~85x in the report).

#### What this means for the closure picture

The `<5x of PCRE2 compile` ROADMAP target you cited still holds as the integration-side closure criterion. Where we are vs. that target:

- **Was: ~360x against no-JIT, ~85x against +JIT** (RGX's 2026-05-01 measurement).
- **Now: ~80x against no-JIT, ~17x against +JIT** (this release).
- **Target: <5x against no-JIT** — still ~16x of compression away.

The remaining gap is **structural**, not dispatch overhead:

- **Direct-parser-path geomean is ~70x** at HEAD. The ~10x gap between direct-path (~70x) and embedding-API-path (~80x) is what's left of the dispatch overhead — small enough that further dispatch tuning (~1µs at the per-call level) won't move the geomean meaningfully.
- The 70x is the cost of PGEN's general-purpose EBNF-driven codegen processing PCRE2-like grammars vs. PCRE2's hand-tuned C compile. Closing it requires either:
  1. **Specialised codegen for the regex grammar** (most likely route per the report's analysis): bypass PGEN's general-purpose EBNF-driven codegen for the regex profile specifically, emit a hand-tuned recursive-descent parser matching the published shape. Could be limited to the `regex` profile, leaving SystemVerilog / VHDL grammars on the general path.
  2. **Sustained Optim #16+ track** on parser-internal hot paths. Less likely to deliver order-of-magnitude wins given the structural argument (PGEN parse < PCRE2 compile structurally yet runs slower), but viable if hot paths concentrate in a few rewritable spots.
  3. **Diagnostic out-of-scope decision**: PGEN team determines the general-purpose architecture cannot reach <5x without specialised codegen, and specialised codegen is out-of-scope for the regex grammar — in that case 0078 closes with a forward reference and the `<5x of PCRE2 compile` ROADMAP target gets retired.

#### Verification commands RGX can run on its own host

Reproduce the embedding-API ratio at any current PGEN HEAD without leaving the PGEN tree:

```bash
# In the PGEN tree:
cd rust
cargo build --release --features generated_parsers --features mimalloc_perf \
    --bin regex_perf_probe_embedding_api
./target/release/regex_perf_probe_embedding_api
# Output: per-pattern p50 ns + min/mean/p99/max
```

Reproduce the direct-parser-path ratio (the ~70x number) the same way with `regex_perf_probe`:

```bash
cargo build --release --features generated_parsers --features mimalloc_perf \
    --bin regex_perf_probe
./target/release/regex_perf_probe --samples 5000 --warmup 200
```

Reproduce the validation-pass cost (confirming it's not the bottleneck):

```bash
cargo build --release --features mimalloc_perf --bin regex_validation_microbench
./target/release/regex_validation_microbench
```

Both binaries are committed at `rust/src/bin/`. Their output is stable across reruns within ±10% (allocator + scheduler noise). Compare against the tables above on the same Apple M4 Pro / mimalloc / current HEAD methodology to verify reproducibility.

For the PCRE2 side, RGX's existing C bench programs in `pgen-issues/artifacts/PGEN-RGX-0078/pgen_iteration_flow/` continue to work — but **fix the metric mismatch** before publishing any combined ratio table: either change PGEN-side to batch-mean or change PCRE2-side to per-call p50. Don't mix them.

#### Ledger + bug status

- [`REGEX-0078`](../../contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md) Notes column updated 2026-05-04 with the verification findings + Optim #14/#15 measurement + closure-path implications.
- Status remains **"Acknowledged / Deferred (non-blocking)"** pending the structural ~16x parser-internal compression. The combined Optim #1–#15 + mimalloc compression of 4.5x against the report's 360x is meaningful but not closure.
- No bug-class change. PGEN-RGX-0078 is still classified as "pathological performance per protocol §4" — not a correctness issue.

**Contract section**: [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.74 / Contract 1.1.76 Highlights".

### 1.1.73 / Contract 1.1.75 — PGEN-RGX-0079 fix: invalid braced escapes rejected (no silent misparse)

**What changed:** Negative-lookahead guards added to `simple_escape` to block the `\X{...}` brace-form fallback for invalid braced escapes.

```ebnf
simple_escape = !"o{" !"x{" !"p{" !"P{" any_char
                  -> {type: "escape", kind: "shorthand", char: $5}
```

**Bug class:** "parses but returns the wrong AST/dump" — same as PGEN-RGX-0006 / -0080. Pre-fix, invalid braced escapes (`\o{1239}`, `\o{8}`, `\o{}`, `\o{12abc}`, `\o{12 34}`) silently misparsed as `simple_escape("o")` + `counted_quantifier{1239,1239}` (or literal pieces). PCRE2 rejects with "error 164: non-octal character in \o{}".

**Audit-recommended `\x{12g}` and `\p{!}` cases also fixed** in the same slice (the bug report flagged these as part of the same audit).

**Reproducer matrix verified:**
- REJECT: `\o{1239}`, `\o{8}`, `\o{}`, `\o{12abc}`, `\o{12 34}` (5 patterns from bug report)
- REJECT (audit): `\x{12g}`, `\p{!}`
- ACCEPT (sanity): `\o{777}`, `\o{0}`, `\o{12}`, `\xFF`, `\x{1F}`, `\pL`, `\p{Lu}`, `\P{Nd}`, `\d`, `\w`, `\s`, `\.`

**Bare forms unchanged.** `\o` (no `{`), `\x`, `\p`, `\P` still match as `simple_escape` since the lookaheads require BOTH chars to match `o{`/`x{`/`p{`/`P{`.

**`\u{...}` already rejected** by the host-side compile validator (slice 15 note) — no `!"u{"` lookahead needed.

**Annotation index shift:** `char:$1` → `char:$5` due to 4 added negative-lookahead slots before `any_char`.

**PCRE2 conformance:** testinput2:3979 (`/^A\o{1239}B/`) stops being a false-positive.

**Bug ledger:** [`REGEX-0079`](../../contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md) status moves from "Acknowledged" to "Released".

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.73 / Contract 1.1.75 Highlights".

### 1.1.72 / Contract 1.1.74 — PGEN-RGX-0080 fix: counted_quantifier accepts inner whitespace

**What changed:** `counted_quantifier_body` rule now allows `ws?` at every position between `{` and `}`, matching PCRE2 default-mode behavior.

```ebnf
counted_quantifier_body = digits ws? "," ws? digits ws?  -> {min: $1, max: $5}
                        | digits ws? "," ws?              -> {min: $1, max: null}
                        | digits ws?                       -> {min: $1, max: $1}
                        | "," ws? digits                   -> {min: 0,  max: $3}
```

**Bug class:** "parses but returns the wrong AST/dump" — same as PGEN-RGX-0006 / PGEN-RGX-0079. Pre-fix, whitespace abutting the comma (`a{ 1 , 2 }`, `a{1 ,2}`, `a{1, 2}`) caused the rule to fail and the parser to fall back to per-character literal pieces (10 separate atoms for `a{ 1 , 2 }` instead of 1 quantified atom).

**Reproducer matrix verified — all 5 patterns now produce identical `quantifier:{min:1, max:2}`:**
- `a{1,2}` ✓
- `a{ 1,2 }` ✓
- `a{ 1 , 2 }` ✓ (was 10 literal pieces)
- `a{1 ,2}` ✓ (was 7 literal pieces)
- `a{1, 2}` ✓ (was 7 literal pieces)

**PCRE2 conformance:** testinput1:6679 (`/a{ 1 , 2 }/`) stops being a false-negative.

**Annotation index shift:** branch 0's `max:$3` became `max:$5` due to added `ws?` slots before/after the comma. Other branches unchanged.

**Bug ledger:** [`REGEX-0080`](../../contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md) status moves from "Acknowledged" to "Released".

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.72 / Contract 1.1.74 Highlights".

### 1.1.71 / Contract 1.1.73 — Slice 42: quoted_class_range_atom typed

**What changed:** `quoted_class_range_atom` typed `{type, char}`.

```ebnf
quoted_class_range_atom = "\\Q" quoted_class_literal_char "\\E"
                            -> {type: "class_quoted_range_atom", char: $2}
```

**Before / after (visible inside `class_range.start` / `class_range.end` for the PCRE2 `\Q...\E` quoted form):**

| Source | Before (slice 29) | After |
|---|---|---|
| `[\Qa\E-\Qz\E]` body[0].start | `["\\Q", "a", "\\E"]` (raw 3-element) | `{type:"class_quoted_range_atom", char:"a"}` |
| `[\Qa\E-\Qz\E]` body[0].end | `["\\Q", "z", "\\E"]` | `{type:"class_quoted_range_atom", char:"z"}` |

**`class_range.start` / `class_range.end` now end-to-end typed** across all 3 class_atom branches: `quoted_class_range_atom` (slice 42), `class_range_escape` (slice 29's passthrough surfaces the typed escape_unit), `class_literal` (slice 15's clean string).

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.71 / Contract 1.1.73 Highlights".

### 1.1.70 / Contract 1.1.72 — Slice 40: modifier_item per-branch typed (closes inline_modifiers spec end-to-end)

**What changed:** `modifier_item` split from 3 branches with internal optionals into 5 explicit branches; eliminates the `[]` interleaving from slice 39's set/unset arrays.

```ebnf
modifier_item = "a" ascii_restrict_modifier  -> {char: "a", restrict: $2}
              | "a"                          -> "a"
              | "xx"                         -> "xx"
              | "x"                          -> "x"
              | modifier_char
```

**Before / after:**

| Source | Before (slice 39) | After |
|---|---|---|
| `(?ix)` | `set:["i", "x", []]` | `set:["i", "x"]` |
| `(?ax)` | `set:["a", [], "x", []]` | `set:["a", "x"]` |
| `(?aDx)` | similar interleaved | `set:[{char:"a", restrict:"D"}, "x"]` |
| `(?xx)` | similar | `set:["xx"]` (matched doubled-x branch) |
| `(?ix-m)` | `set:["i", "x", []], unset:["m"]` | `set:["i", "x"], unset:["m"]` |

**Mixed string/object array** for the `aD` form: `(?aDx)` produces `set:[{char:"a", restrict:"D"}, "x"]`. Consumer dispatches `Value::String`/`Value::Object`.

**`inline_modifiers.spec` end-to-end typed.** Combined slices: 24 (`kind:"inline_modifiers"`) + 31 (`spec:{reset, seq}`) + 39 (`seq:{set, unset}`) + 40 (clean items in set/unset).

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.70 / Contract 1.1.72 Highlights".

### 1.1.69 / Contract 1.1.71 — Slice 39: modifier_seq + modifier_group typed (`{set, unset}` shape)

**What changed:** `modifier_seq` split into 3 explicit branches with `{set, unset}` typed shape; `modifier_group` flattened.

```ebnf
modifier_seq   = modifier_group "-" modifier_group  -> {set: $1, unset: $3}
               | modifier_group                      -> {set: $1, unset: []}
               | "-" modifier_group                  -> {set: [], unset: $2}
modifier_group = modifier_item+                      -> [$1**]
```

**Before / after (visible inside `inline_modifiers.spec.seq` / `scoped_inline_modifiers.spec.seq`):**

| Source | Before (slice 31) | After |
|---|---|---|
| `(?i)` | `seq:[[...], []]` (raw 2-element) | `seq:{set:["i"], unset:[]}` |
| `(?ix-m)` | `seq:[[...], ["-", [...]]]` | `seq:{set:["i", "x", []], unset:["m"]}` |
| `(?-i)` | `seq:["-", [...]]` | `seq:{set:[], unset:["i"]}` |
| `(?ix)` | `seq:[[...], []]` | `seq:{set:["i", "x", []], unset:[]}` |

**Set/unset arrays may have interleaved `[]` markers** for optional-sub-element items: `(?ix)` → `set:["i", "x", []]` (the `[]` is the un-matched `"x"?` slot for the `"x" "x"?` modifier_item branch). Per-rule typing of `modifier_item` (would clean these — split `"x" "x"?` into `"xx" | "x"`, type `"a" ascii_restrict_modifier?` to `{char, restrict}`) is a separate concern.

**3-branch split rationale:** annotation language doesn't currently support extracting from a parens-grouped optional pair. Splitting `modifier_seq` into 3 explicit branches gives the cleanest typed shape. Same accept set; PEG tries longest form first.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.69 / Contract 1.1.71 Highlights".

### 1.1.68 / Contract 1.1.70 — Slice 38: returned_capture_subroutine outer typed

**What changed:** `returned_capture_subroutine` typed `{subroutine, captures}`.

```ebnf
returned_capture_subroutine = subroutine_target returned_capture_group_list
                                -> {subroutine: $1, captures: $2}
```

**Before / after (visible inside `subroutine_call.target` for branch 0 / with-captures form):**

| Source | Before (slice 30) | After |
|---|---|---|
| `(?&name(1))` | `target:[{kind:"named", name:"name"}, ["(", {sign:[], value:1}, [], ")"]]` (raw 2-element seq) | `target:{subroutine:{kind:"named", name:"name"}, captures:["(", {sign:[], value:1}, [], ")"]}` |

**Inner field named `subroutine`** (not `target`) to avoid `target.target.kind` collision with the outer `subroutine_call.target`. Consumer reads `obj.target.subroutine.kind` for the typed form and `obj.target.captures` for the list.

**`captures` still raw.** The parens-grouped repetition `("," returned_capture_group)*` produces a comma-interleaved nested shape that the annotation language can't currently flatten via positional/field access. Saved for a follow-up slice (would need either annotation language extension or grammar restructuring to a recursive pair form).

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.68 / Contract 1.1.70 Highlights".

### 1.1.67 / Contract 1.1.69 — Slice 37: version_number `{major, minor}` typed

**What changed:** `version_number` split into 2 explicit branches with `{major, minor}` typed shape.

```ebnf
version_number = digits "." digits  -> {major: $1, minor: $3}
               | digits              -> {major: $1, minor: null}
```

**Before / after (visible inside `conditional.condition.number`):**

| Source | Before (slice 36) | After |
|---|---|---|
| `(?(VERSION>=10.0)foo)` | `number:[10, [".", 0]]` | `number:{major:10, minor:0}` |
| `(?(VERSION>=11)foo)` | `number:[11, []]` | `number:{major:11, minor:null}` |
| `(?(VERSION=10.40)foo)` | `number:[10, [".", 40]]` | `number:{major:10, minor:40}` |

**Why 2 branches instead of one:** the annotation language doesn't currently support extracting from a parens-grouped optional pair via positional or field access. Splitting `digits ("." digits)?` into explicit `digits "." digits | digits` branches gives the cleanest typed shape with `null` (not `[]`) for the absent-minor case. Same accept set — PEG tries the longer form first.

**`version_condition` now end-to-end typed.** Combined with slice 32's outer typing.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.67 / Contract 1.1.69 Highlights".

### 1.1.66 / Contract 1.1.68 — Slice 36: condition_assertion / alpha_condition_assertion / condition_callout_assertion / condition_callout typed (closes condition Or-of-9 fully)

**What changed:** Closes the remaining 4 of the 9 condition Or-alternatives (combined with slice 32's 3, 7 of 9 are now typed; the 2 remaining — `name_ref` and `name` — stay as bare strings since they're reused outside condition).

```ebnf
condition_assertion         = "?=" pattern   -> {kind: "lookahead",  positive: true,  body: $2}
                            | "?!" pattern   -> {kind: "lookahead",  positive: false, body: $2}
                            | "?<=" pattern  -> {kind: "lookbehind", positive: true,  body: $2}
                            | "?<!" pattern  -> {kind: "lookbehind", positive: false, body: $2}
                            | alpha_condition_assertion
alpha_condition_assertion   = "*" atomic_alpha_lookaround_name ":" pattern?
                                -> {kind: "alpha_lookaround", name: $2, body: $4}
condition_callout           = "?C" callout_arg? ")"
                                -> {kind: "callout", arg: $2}
condition_callout_assertion = condition_callout "(" condition_assertion
                                -> {kind: "callout_assertion", callout: $1, assertion: $3}
```

**Before / after (visible inside `conditional.condition`):**

| Source | After |
|---|---|
| `(?(?=foo)yes)` | `condition:{kind:"lookahead", positive:true, body:<pattern>}` |
| `(?(?!bar)yes)` | `condition:{kind:"lookahead", positive:false, body:<pattern>}` |
| `(?(?<=baz)yes)` | `condition:{kind:"lookbehind", positive:true, body:<pattern>}` |
| `(?(?<!qux)yes)` | `condition:{kind:"lookbehind", positive:false, body:<pattern>}` |
| `(?(*pla:foo)yes)` | `condition:{kind:"alpha_lookaround", name:"pla", body:<pattern>}` |

**`kind` value collisions with slice 23's atom-level lookaround typing are intentional.** Both produce `{kind:"lookahead", positive, body}` — the atom-level form has an additional `type:"atom"` field; condition-context doesn't. Consumer dispatches first on `condition.kind`, then on contextual surroundings.

**`condition` Or-of-9 typing status after slice 36:**
- **Typed (7):** define_condition, version_condition, recursion_condition (×2 branches), condition_callout_assertion, condition_assertion (×4 branches + alpha passthrough), alpha_condition_assertion.
- **Untyped (2, intentionally — reused outside condition context):** name_ref, name (both surface as bare strings); plus signed_digits (`{sign, value}` from slice 13) and digits (typed int from slice 1) which are fine as their existing typed shapes.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.66 / Contract 1.1.68 Highlights".

### 1.1.65 / Contract 1.1.67 — Slice 35: directive_payload_suffix typed + directive_payload_simple regex-literal rewrite

**What changed:** `directive_payload_suffix` per-branch typed; `directive_payload_simple` rewritten as regex literal.

```ebnf
directive_payload_suffix = ":" directive_payload_simple?  -> {separator: ":", value: $2}
                         | "=" directive_payload_simple?  -> {separator: "=", value: $2}
directive_payload_simple = /([^)]*)/  # was directive_payload_char* chain
```

**Before / after (visible inside `directive_verb.body.payload`):**

| Source | Before (slice 34) | After |
|---|---|---|
| `(*MARK:foo)` | `payload:[":", ["f","o","o"]]` | `payload:{separator:":", value:"foo"}` |
| `(*COMMIT)` | `payload:[]` (un-matched) | `payload:[]` (unchanged — optional slot) |
| `(*:short)` (mark_shorthand) | `payload:["s","h","o","r","t"]` | `payload:"short"` (clean string) |

**`directive_verb.body` now end-to-end typed** across all 3 sub-rule levels (body / named\|shorthand / payload).

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.65 / Contract 1.1.67 Highlights".

### 1.1.64 / Contract 1.1.66 — Slice 34: directive_body / directive_named / directive_mark_shorthand typed + directive_name regex-literal rewrite

**What changed:** `directive_body`'s 2 sub-rules typed; `directive_name` rewritten as regex literal for clean string output.

```ebnf
directive_named          = directive_name directive_payload_suffix?  -> {kind: "named", name: $1, payload: $2}
directive_mark_shorthand = ":" directive_payload_simple?             -> {kind: "mark_shorthand", payload: $2}
directive_name           = /([A-Za-z][A-Za-z0-9_\-]*)/  # was directive_name_start directive_name_continue*
```

**Before / after (visible inside `directive_verb.body`):**

| Source | Before (slice 24) | After |
|---|---|---|
| `(*MARK:foo)` | `body:[["M", ["A","R","K"]], [":", [...]]]` (raw chain) | `body:{kind:"named", name:"MARK", payload:[":", ["f","o","o"]]}` |
| `(*COMMIT)` | similar raw chain | `body:{kind:"named", name:"COMMIT", payload:[]}` |
| `(*:bar)` | similar raw chain | `body:{kind:"mark_shorthand", payload:["b","a","r"]}` |

**`name` is now a clean string** (slice 34's regex-literal rewrite — same pattern as `name`/`hex_digits`/`octal_digits`/`prop_name`/`comment_text` in earlier slices).

**`payload` carries raw shape.** Per-rule typing of `directive_payload_suffix` / `directive_payload_simple` is a separate concern.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.64 / Contract 1.1.66 Highlights".

### 1.1.63 / Contract 1.1.65 — Slice 33: callout_string typed (8 quote-form variants)

**What changed:** All 8 callout-string quote variants now emit typed `{quote:<text-label>, payload:<string>}` objects.

```ebnf
callout_backtick_string  = '`' callout_backtick_payload '`'    -> {quote: "backtick", payload: $2}
callout_single_string    = "'" callout_single_payload "'"      -> {quote: "single",   payload: $2}
callout_double_string    = '"' callout_double_payload '"'      -> {quote: "double",   payload: $2}
callout_caret_string     = "^" callout_caret_payload "^"       -> {quote: "caret",    payload: $2}
callout_percent_string   = "%" callout_percent_payload "%"     -> {quote: "percent",  payload: $2}
callout_hash_string      = "#" callout_hash_payload "#"        -> {quote: "hash",     payload: $2}
callout_dollar_string    = "$" callout_dollar_payload "$"      -> {quote: "dollar",   payload: $2}
callout_brace_string     = "{" callout_brace_payload "}"       -> {quote: "brace",    payload: $2}
```

**Before / after (visible inside `callout.arg`):**

| Source | Before (slice 24) | After |
|---|---|---|
| `` (?C`hello`) `` | `arg:["` `", "hello", "` `"]` | `arg:{quote:"backtick", payload:"hello"}` |
| `(?C'world')` | `arg:["'", "world", "'"]` | `arg:{quote:"single", payload:"world"}` |
| `(?C"dq")` | `arg:["\"", "dq", "\""]` | `arg:{quote:"double", payload:"dq"}` |
| `(?C{brace})` | `arg:["{", "brace", "}"]` | `arg:{quote:"brace", payload:"brace"}` |

**`quote` is a text label, not the literal character.** PGEN's bootstrap annotation parser doesn't support `"\""` (escaped double-quote) inside string literals — switched all 8 forms to text labels for uniformity. Consumer reads `arg.quote` as an enum-like discriminator: `"backtick"`/`"single"`/`"double"`/`"caret"`/`"percent"`/`"hash"`/`"dollar"`/`"brace"`.

**Numeric callout** (`(?C42)`) continues to surface `arg:42` (typed int from slice 1's digits @transform). Consumer dispatches: object → string callout; number → numeric callout; `[]` → empty `(?C)`.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.63 / Contract 1.1.65 Highlights".

### 1.1.62 / Contract 1.1.64 — Slice 32: define_condition / version_condition / recursion_condition typed

**What changed:** 4 annotations across 3 condition Or-alternatives — `define_condition`, `version_condition`, and `recursion_condition` (2 branches).

```ebnf
define_condition  = "DEFINE"                                          -> {kind: "define"}
version_condition = "VERSION" version_operator version_number         -> {kind: "version", operator: $2, number: $3}
recursion_condition = "R" digits?                                     -> {kind: "recursion", group: $2}
                    | "R&" name                                       -> {kind: "recursion_named", name: $2}
```

**Before / after (visible inside `conditional.condition`):**

| Source | Before (slice 27) | After |
|---|---|---|
| `(?(DEFINE)foo)` | `condition:"DEFINE"` (string — ambiguous w/ name) | `condition:{kind:"define"}` |
| `(?(VERSION>=10.0)foo)` | 3-element raw seq | `condition:{kind:"version", operator:">=", number:[10, [".", 0]]}` |
| `(?(R)bar)` | `condition:["R", []]` | `condition:{kind:"recursion", group:[]}` |
| `(?(R3)baz)` | `condition:["R", 3]` | `condition:{kind:"recursion", group:3}` |
| `(?(R&name)abc)` | `condition:["R&", "name"]` | `condition:{kind:"recursion_named", name:"name"}` |

**6 of 9 condition Or-alternatives now disambiguated.** The remaining 6 (`condition_callout_assertion`, `condition_assertion`, `name_ref`, `name`, `signed_digits`, `digits`) are reused outside the condition context and weren't wrapped to avoid changing their shape across all callers. Consumer dispatches by:
- Object with `kind`: typed condition.
- Object with `sign`/`value`: `signed_digits`.
- Number: `digits`.
- String: `name`/`name_ref`. (Now disambiguated from `(?(DEFINE)...)`.)
- Other: `condition_callout_assertion` / `condition_assertion` (raw, not yet typed).

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.62 / Contract 1.1.64 Highlights".

### 1.1.61 / Contract 1.1.63 — Slice 31: modifier_spec typed

**What changed:** `modifier_spec` per-branch typed `{reset:<bool>, seq:<raw>}`.

```ebnf
modifier_spec = "^" modifier_seq?    -> {reset: true, seq: $2}
              | modifier_seq         -> {reset: false, seq: $1}
```

**Before / after (visible inside `inline_modifiers.spec` / `scoped_inline_modifiers.spec`):**

| Source | Before (slice 24) | After |
|---|---|---|
| `(?i)` | `spec:[[...], []]` | `spec:{reset:false, seq:[["i"], []]}` |
| `(?^i)` | similar w/ `"^"` token | `spec:{reset:true, seq:[["i"], []]}` |
| `(?ix-m)` | similar | `spec:{reset:false, seq:[["i", ["x", []]], ["-", ["m"]]]}` |
| `(?-i)` | similar | `spec:{reset:false, seq:["-", ["i"]]}` |

**`reset:true` distinguishes the `(?^...)` form** (which resets all flags first before applying the seq).

**`seq` carries the raw `modifier_seq` shape.** Per-rule typing of `modifier_seq` / `modifier_group` / `modifier_item` (which would unify the flag-set into a clean `{set:["i", "x"], unset:["m"]}` shape) is a separate concern.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.61 / Contract 1.1.63 Highlights".

### 1.1.60 / Contract 1.1.62 — Slice 30: subroutine_target typed

**What changed:** `subroutine_target` now emits typed `{kind, ...}` objects per branch.

```ebnf
subroutine_target = "&" name           -> {kind: "named", name: $2}
                  | "P>" name          -> {kind: "python_named", name: $2}
                  | "R"                -> {kind: "recursion"}
                  | signed_digits      -> {kind: "numeric", value: $1.value, sign: $1.sign}
```

**Before / after (visible inside `subroutine_call.target`):**

| Source | Before (slice 25) | After |
|---|---|---|
| `(?&name)` | `target:["&", "name"]` | `target:{kind:"named", name:"name"}` |
| `(?P>foo)` | `target:["P>", "foo"]` | `target:{kind:"python_named", name:"foo"}` |
| `(?R)` | `target:"R"` | `target:{kind:"recursion"}` |
| `(?+1)` | `target:{sign:"+", value:1}` | `target:{kind:"numeric", sign:"+", value:1}` |
| `(?-2)` | similar | `target:{kind:"numeric", sign:"-", value:2}` |
| `(?42)` | `target:{sign:[], value:42}` | `target:{kind:"numeric", sign:[], value:42}` |

**`subroutine_call.target` now end-to-end typed.** All 4 syntactic forms surface as `{kind, ...}` objects with consistent dispatch.

**`kind:"python_named"` vs `kind:"named"`** — preserves Python syntax origin, paralleling slice 19's `python_named_backreference` and slice 22's `python_named_group`. Consumers normalizing across name-based forms: `target.kind in {"named", "python_named"}` → name-based subroutine; `target.name` carries the name string in both.

**Numeric form: `signed_digits` field-access inline.** The annotation `{kind:"numeric", value:$1.value, sign:$1.sign}` uses the field-access syntax to inline signed_digits' `{sign, value}` typed shape (slice 13) into subroutine_target's typed shape. Consumer reads `target.value` / `target.sign` directly without an extra nested object.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.60 / Contract 1.1.62 Highlights".

### 1.1.59 / Contract 1.1.61 — Slice 29: class_range / quoted_class_literal / class_range_escape typed

**What changed:** First sub-rule typing slice — the inner shapes that show up under `char_class.body`.

```ebnf
class_range          = class_atom class_zero_width* "-" class_zero_width* class_atom
                        -> {type: "class_range", start: $1, end: $5}
quoted_class_literal = "\\Q" quoted_class_literal_char* "\\E"
                        -> {type: "class_quoted_literal", body: $2}
class_range_escape   = "\\" class_range_escape_unit
                        -> $2
```

**Before / after:**

| Source | Before (slice 28) | After |
|---|---|---|
| `[a-z]` body | `[["a", [], "-", [], "z"]]` | `[{type:"class_range", start:"a", end:"z"}]` |
| `[A-Z0-9_]` body | mix of class_range and class_literal | `[{type:"class_range", start:"A", end:"Z"}, {type:"class_range", start:"0", end:"9"}, "_"]` |
| `[\Qa-z\E]` body | `[["\\Q", ["a", "-", "z"], "\\E"]]` | `[{type:"class_quoted_literal", body:["a", "-", "z"]}]` |
| `[\xA-\xFF]` body | deeply nested | `[{type:"class_range", start:{type:"escape", kind:"hex", digits:"A"}, end:{type:"escape", kind:"hex", digits:"FF"}}]` |

**`class_range_escape -> $2` is a transparent passthrough.** Drops the leading `\` so the typed escape_unit shape (already produced by hex/unicode/octal/control/simple/single_byte/property branches via slices 14-17) surfaces directly inside `class_range.start` / `class_range.end`. Mirrors the outer `escape -> $2` annotation from slice 14.

**Char_class body shapes now typed end-to-end.** Plain literals → bare string (slice 15's class_literal regex literal); ranges → `{type:"class_range"}`; quoted literals → `{type:"class_quoted_literal"}`; escapes → typed escape_unit shapes (slices 14-17); POSIX classes → typed `posix_class` (slice 8).

**Sub-rule typing campaign progress:** First slice. Atom subtree stays 25/25 at outer level; sub-rule typing now covers the most-visible inner shapes inside `char_class.body`.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.59 / Contract 1.1.61 Highlights".

### 1.1.58 / Contract 1.1.60 — Atom subtree slice 28: extended_class typed (recursive structures all closed)

**What changed:** `extended_class` now emits typed `{type:"atom", kind:"extended_class", body:<content>}` objects.

```ebnf
extended_class = "(?[" extended_class_content "])"
                  -> {type: "atom", kind: "extended_class", body: $2}
```

**Before / after:**

| Source | After |
|---|---|
| `(?[abc])` | `{kind:"extended_class", body:["a","b","c"]}` |
| `(?[a-z])` | `{body:["a","-","z"]}` |
| `(?[[abc][def]])` | `{body:[["[", ["a","b","c"], "]"], ["[", ["d","e","f"], "]"]]}` (nested) |
| `(?[])` | `{body:[]}` |

`body` is the raw `extended_class_content` shape (Quantified-* of extended_class_element). Sub-rule typing of content/element/nested (the recursive set-operation structure: `[X]&[Y]`, `[X]+[Y]`, etc.) is a separate concern.

**Atom subtree campaign progress: 25/25 atom alternatives directly typed.** All recursive structures closed. Only the 3 deferred leaf-char alternatives (literal / whitespace_literal / dot) remain as a separate decision.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.58 / Contract 1.1.60 Highlights".

### 1.1.57 / Contract 1.1.59 — Atom subtree slice 27: conditional typed

**What changed:** `conditional` now emits typed `{type:"atom", kind:"conditional", condition, yes_branch, no_branch}` objects.

```ebnf
conditional        = "(?(" condition ")" yes_branch ("|" no_branch)? ")"
                       -> {type: "atom", kind: "conditional", condition: $2, yes_branch: $4, no_branch: $5}
conditional_branch = piece*    -> [$1**]
```

**Before / after:**

| Source | After |
|---|---|
| `(?(1)abc)` | `{kind:"conditional", condition:{sign:[], value:1}, yes_branch:[<3 pieces>], no_branch:[]}` |
| `(?(1)abc|xyz)` | `{condition:{sign:[], value:1}, yes_branch:<3 pieces>, no_branch:["|", <3 pieces>]}` |
| `(?(DEFINE)foo)` | `{condition:"DEFINE", yes_branch:<3 pieces>, no_branch:[]}` |
| `(?(R)bar)` | `{condition:["R", []], yes_branch:<3 pieces>, no_branch:[]}` |
| `(?(<name>)abc)` | `{condition:"name", yes_branch:<3 pieces>, no_branch:[]}` |

**`condition` is the heterogeneous Or-of-9 raw shape.** Typed signed_digits propagation (slice 13) gives `{sign, value}` for numeric refs; `"DEFINE"` string for `(?(DEFINE)...)`; `["R", []]` for recursion conditions; clean `name` string for named-group refs. Sub-rule typing of `condition` is a separate concern.

**`no_branch` preserves the `|` separator.** `[]` when no else-clause; `["|", <pieces>]` when matched. Consumer reads `no_branch[1]` to extract pieces or maps `[]` to null. Distinguishing "no else-clause" from "empty else-clause" is intentional.

**`conditional_branch` flat-piece-array** (`[$1**]`) parallels `concatenation`'s shape — consumer iterates directly.

**Atom subtree campaign progress:** 24/25 atom alternatives directly typed.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.57 / Contract 1.1.59 Highlights".

### 1.1.56 / Contract 1.1.58 — Atom subtree slice 26: char_class outer typed

**What changed:** `char_class` now emits typed `{type:"atom", kind:"char_class", negated:<bool>, initial_close:<bool>, body:<class_body>}` objects.

```ebnf
char_class          = "[" negation? class_initial_close? class_body "]"
                       -> {type: "atom", kind: "char_class", negated: $2, initial_close: $3, body: $4}
class_initial_close = "]"                                            -> true
negation            = "^"                                            -> true
```

**Before / after:**

| Source | Before | After |
|---|---|---|
| `[abc]` | `["[", [], [], <body>, "]"]` | `{type:"atom", kind:"char_class", negated:[], initial_close:[], body:["a","b","c"]}` |
| `[^abc]` | similar w/ `negation` shape | `{kind:"char_class", negated:true, body:["a","b","c"]}` |
| `[a-z]` | similar w/ `class_range` body | `{kind:"char_class", body:[["a", [], "-", [], "z"]]}` |
| `[]abc]` | similar w/ `class_initial_close` | `{kind:"char_class", initial_close:true, body:["a","b","c"]}` |
| `[^]abc]` | both | `{negated:true, initial_close:true, body:["a","b","c"]}` |
| `[[:alpha:]]` | typed posix_class inside | `{kind:"char_class", body:[{type:"posix_class", name:"alpha", negated:[]}]}` |

**`negated` and `initial_close` are real booleans** (`true` matched, `[]` un-matched). Same convention as `posix_negation` from slice 8 (PGEN-RGX-0076) — `BooleanLiteral` rule-level scalar emits `Json(Bool(true))`.

**`body` is raw `class_body` shape.** Inner items already typed by earlier slices propagate transparently — `posix_class` (slice 8), `class_range_escape` (escape subtree slices), `quoted_class_range_atom` (PGEN-RGX-0068 fix). Pure char-class typing is end-to-end at the outer level; `class_body` per-rule typing is a separate concern.

**Atom subtree campaign progress:** 23/25 atom alternatives directly typed.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.56 / Contract 1.1.58 Highlights".

### 1.1.55 / Contract 1.1.57 — Atom subtree slice 25: scan_substring / script_run / subroutine_call typed

**What changed:** Batched slice — 4 annotations across 3 atom alternatives.

```ebnf
scan_substring_group = "(*" scan_substring_name ":" returned_capture_group_list pattern? ")"
                        -> {type: "atom", kind: "scan_substring_group", name: $2, captures: $4, body: $5}
script_run_group     = "(*" script_run_name ":" pattern? ")"
                        -> {type: "atom", kind: "script_run_group", name: $2, body: $4}
subroutine_call      = "(?" returned_capture_subroutine ")"
                        -> {type: "atom", kind: "subroutine_call", target: $2}
                     | "(?" subroutine_target ")"
                        -> {type: "atom", kind: "subroutine_call", target: $2}
```

**Before / after:**

| Source | After |
|---|---|
| `(*sr:abc)` | `{kind:"script_run_group", name:"sr", body:<pattern>}` |
| `(*atomic_script_run:foo)` | `{kind:"script_run_group", name:"atomic_script_run", body:<pattern>}` |
| `(a)(*scs:(1)bcd)` | `{kind:"scan_substring_group", name:"scs", captures:["(", {sign:[], value:1}, [], ")"], body:<pattern>}` |
| `(?&name)` | `{kind:"subroutine_call", target:["&", "name"]}` |
| `(?P>name)` | `{kind:"subroutine_call", target:["P>", "name"]}` |
| `(?R)` | `{kind:"subroutine_call", target:"R"}` |
| `(?+1)` | `{kind:"subroutine_call", target:{sign:"+", value:1}}` |

**`subroutine_call` two-branch collapse:** Both branches produce `kind:"subroutine_call"` with `target` carrying the inner shape. Branch 0 is the "with returned captures" form (`returned_capture_subroutine`); branch 1 is the plain form. Consumers inspect `target` shape to determine which form.

**Sub-rule shapes deferred:** `returned_capture_group_list`, `returned_capture_subroutine`, `subroutine_target` carry raw shapes. Per-rule typing is a separate concern.

**`signed_digits` propagation already works:** Slice 13's `signed_digits -> {sign, value}` typing surfaces directly inside `subroutine_call.target` for numeric subroutine refs like `(?+1)`. This wasn't a slice 25 change but is the visible payoff of the earlier work.

**Pre-existing host-validator note:** PGEN's host-side compile validator rejects scan_substring capture-list references that don't have a corresponding group in the surrounding pattern. The annotation is correct when the validator allows; for inputs the validator rejects, no AST is produced.

**Atom subtree campaign progress:** 22/25 atom alternatives directly typed.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.55 / Contract 1.1.57 Highlights".

### 1.1.54 / Contract 1.1.56 — Atom subtree slice 24: inline-modifier / callout / directive_verb / code_block typed

**What changed:** Batched slice — 6 annotations across 5 atom alternatives.

```ebnf
inline_modifiers        = "(?" modifier_spec? ")"            -> {type: "atom", kind: "inline_modifiers", spec: $2}
scoped_inline_modifiers = "(?" modifier_spec ":" pattern? ")" -> {type: "atom", kind: "scoped_inline_modifiers", spec: $2, body: $4}
callout                 = "(?C" callout_arg? ")"             -> {type: "atom", kind: "callout", arg: $2}
directive_verb          = "(*" directive_body ")"            -> {type: "atom", kind: "directive_verb", body: $2}
code_block_plain        = "(?{" code_content "})"            -> {type: "atom", kind: "code_block", lang: null, content: $2}
code_block_lang         = "(?{" code_lang ":" ws? code_content "})"
                                                              -> {type: "atom", kind: "code_block", lang: $2, content: $4}
```

**Before / after:**

| Source | Before | After |
|---|---|---|
| `(?i)` | `["(?", [<modifier_spec>], ")"]` | `{kind:"inline_modifiers", spec:<modifier_spec>}` |
| `(?i:abc)` | `["(?", <modifier>, ":", <pattern>, ")"]` | `{kind:"scoped_inline_modifiers", spec:<modifier>, body:<pattern>}` |
| `(?C42)` | `["(?C", 42, ")"]` | `{kind:"callout", arg:42}` |
| `(*MARK:foo)` | `["(*", <directive>, ")"]` | `{kind:"directive_verb", body:<directive>}` |
| `(?{print})` | `["(?{", <content>, "})"]` | `{kind:"code_block", lang:null, content:<content>}` |
| `(?{lua: print})` | `["(?{", "lua", ":", <ws?>, <content>, "})"]` | `{kind:"code_block", lang:"lua", content:<content>}` |

**`code_block` two-branch collapse:** Both `code_block_plain` and `code_block_lang` produce `kind:"code_block"`, distinguished by `lang` (null vs string). Consumer always reads `obj.lang` and `obj.content` — no need to dispatch on which branch matched.

**Sub-rule shapes deferred:** `modifier_spec`, `callout_arg`, `directive_body`, `code_content` carry raw shapes. Their per-rule typing is left to follow-up slices. Atom-level dispatch on `kind` is what slice 24 delivers.

**Atom subtree campaign progress:** 19/25 atom alternatives directly typed (code_block counted as 1 atom alternative; its 2 branches collapse).

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.54 / Contract 1.1.56 Highlights".

### 1.1.53 / Contract 1.1.55 — Atom subtree slice 23: lookaround family typed (7 sub-rules)

**What changed:** All 7 lookaround sub-rules now emit typed `{type:"atom", kind:<lookaround_kind>, ..., body:<pattern>}` objects.

```ebnf
lookahead_pos             = "(?=" pattern ")"            -> {type: "atom", kind: "lookahead",   positive: true,  body: $2}
lookahead_neg             = "(?!" pattern ")"            -> {type: "atom", kind: "lookahead",   positive: false, body: $2}
lookbehind_pos            = "(?<=" pattern ")"           -> {type: "atom", kind: "lookbehind",  positive: true,  body: $2}
lookbehind_neg            = "(?<!" pattern ")"           -> {type: "atom", kind: "lookbehind",  positive: false, body: $2}
non_atomic_lookahead_pos  = "(?*" pattern ")"            -> {type: "atom", kind: "non_atomic_lookahead",  positive: true, body: $2}
non_atomic_lookbehind_pos = "(?<*" pattern ")"           -> {type: "atom", kind: "non_atomic_lookbehind", positive: true, body: $2}
alpha_lookaround          = "(*" alpha_lookaround_name ":" pattern? ")"
                                                          -> {type: "atom", kind: "alpha_lookaround", name: $2, body: $4}
```

**Before / after:**

| Source | Before | After |
|---|---|---|
| `(?=foo)` | `["(?=", <pattern>, ")"]` | `{type:"atom", kind:"lookahead", positive:true, body:<pattern>}` |
| `(?!bar)` | `["(?!", <pattern>, ")"]` | `{kind:"lookahead", positive:false, body:<pattern>}` |
| `(?<=baz)` | `["(?<=", <pattern>, ")"]` | `{kind:"lookbehind", positive:true, body:<pattern>}` |
| `(?<!qux)` | `["(?<!", <pattern>, ")"]` | `{kind:"lookbehind", positive:false, body:<pattern>}` |
| `(?*alpha)` | `["(?*", <pattern>, ")"]` | `{kind:"non_atomic_lookahead", positive:true, body:<pattern>}` |
| `(?<*beta)` | `["(?<*", <pattern>, ")"]` | `{kind:"non_atomic_lookbehind", positive:true, body:<pattern>}` |
| `(*pla:gamma)` | `["(*", "pla", ":", <pattern>, ")"]` | `{kind:"alpha_lookaround", name:"pla", body:<pattern>}` |
| `(*nla:delta)` | similar | `{kind:"alpha_lookaround", name:"nla", body:<pattern>}` |

**`kind` + `positive` design:** Lookahead and lookbehind each collapse 2 syntactic forms (`_pos`/`_neg`) to one `kind` with a `positive` boolean — consistent with the property_escape `negated` field convention from slice 17. Non-atomic forms get distinct `kind` values since PCRE2 only supports positive variants for them.

**Alpha-form** carries the alpha_lookaround_name in `name`. PCRE2 admits `pla`/`positive_lookahead`, `nla`/`negative_lookahead`, `plb`/`positive_lookbehind`, `nlb`/`negative_lookbehind`, `napla`/`non_atomic_positive_lookahead`, `naplb`/`non_atomic_positive_lookbehind`. Consumers map by `name` to dispatch on the semantic equivalent.

**Atom subtree campaign progress:** 14/25 atom alternatives directly typed. Lookaround family typed end-to-end.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.53 / Contract 1.1.55 Highlights".

### 1.1.52 / Contract 1.1.54 — Atom subtree slice 22: named groups typed (named/python_named)

**What changed:** `named_group` (both angle and quote syntactic forms) and `python_named_group` now emit typed `{type:"atom", kind:<group_kind>, name:<string>, body:<pattern>}` objects.

```ebnf
named_group        = "(?<" name ">" pattern? ")"  -> {type: "atom", kind: "named_group", name: $2, body: $4}
                   | "(?'" name "'" pattern? ")"  -> {type: "atom", kind: "named_group", name: $2, body: $4}
python_named_group = "(?P<" name ">" pattern? ")" -> {type: "atom", kind: "python_named_group", name: $2, body: $4}
```

**Before / after:**

| Source | Before | After |
|---|---|---|
| `(?<foo>abc)` | `["(?<", "foo", ">", <pattern>, ")"]` | `{type:"atom", kind:"named_group", name:"foo", body:<pattern>}` |
| `(?'bar'xyz)` | `["(?'", "bar", "'", <pattern>, ")"]` | `{kind:"named_group", name:"bar", body:<pattern>}` |
| `(?P<baz>123)` | `["(?P<", "baz", ">", <pattern>, ")"]` | `{kind:"python_named_group", name:"baz", body:<pattern>}` |
| `(?<empty>)` | similar 5-element seq | `{kind:"named_group", name:"empty", body:[[], []]}` |

`name` was already typed to a clean string by slice 11. `body` is the raw pattern shape (pattern outer typing is a separate slice).

**`kind:"python_named_group"` distinct from `kind:"named_group"`.** Paralleling slice 19's `python_named_backreference` decision: PCRE2 treats `(?P<n>...)` and `(?<n>...)` as functionally equivalent, but tooling that displays the source pattern wants to preserve the syntactic origin. Consumers normalizing across name-based group forms: `kind in {"named_group", "python_named_group"}` → name-based group; `name` carries the name in both.

**Group typing now end-to-end.** All 6 group sub-rules typed:
- `capturing_group`, `noncapturing_group`, `named_group`, `python_named_group` (under `group`)
- `branch_reset_group`, `atomic_group` (standalone)

**Atom subtree campaign progress:** 13/25 atom alternatives directly typed.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.52 / Contract 1.1.54 Highlights".

### 1.1.51 / Contract 1.1.53 — Atom subtree slice 21: simple groups typed (capturing/noncapturing/branch_reset/atomic)

**What changed:** Four group forms now emit typed `{type:"atom", kind:<group_kind>, body:<pattern>}` objects.

```ebnf
capturing_group     = "(" pattern? ")"           -> {type: "atom", kind: "capturing_group", body: $2}
noncapturing_group  = "(?:" pattern? ")"         -> {type: "atom", kind: "noncapturing_group", body: $2}
branch_reset_group  = "(?|" pattern? ")"         -> {type: "atom", kind: "branch_reset_group", body: $2}
atomic_group        = "(?>" pattern? ")"         -> {type: "atom", kind: "atomic_group", body: $2}
                    | "(*atomic:" pattern? ")"   -> {type: "atom", kind: "atomic_group", body: $2}
```

**Before / after:**

| Source | Before | After |
|---|---|---|
| `(abc)` | `["(", <pattern>, ")"]` | `{type:"atom", kind:"capturing_group", body:<pattern>}` |
| `(?:abc)` | `["(?:", <pattern>, ")"]` | `{kind:"noncapturing_group", body:<pattern>}` |
| `(?>abc)` | `["(?>", <pattern>, ")"]` | `{kind:"atomic_group", body:<pattern>}` |
| `(*atomic:abc)` | `["(*atomic:", <pattern>, ")"]` | `{kind:"atomic_group", body:<pattern>}` (same kind as `(?>...)`) |
| `(?|a|b)` | `["(?|", <pattern>, ")"]` | `{kind:"branch_reset_group", body:<pattern>}` |
| `()` / `(?:)` | similar 3-element Sequence | `body: [[], []]` (empty alternation shape from `pattern?` matched-empty) |

**`body` is the raw pattern shape**, not itself typed. Pattern outer typing is a separate slice.

**Atomic group's two syntactic forms both produce `kind:"atomic_group"`.** PCRE2 treats `(?>...)` and `(*atomic:...)` as semantically equivalent; the typed shape doesn't preserve the syntactic origin (consistent with how `property_escape`'s 4 forms all produce `kind:"property"`).

**Atom subtree campaign progress:** 11/25 atom alternatives directly typed.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.51 / Contract 1.1.53 Highlights".

### 1.1.50 / Contract 1.1.52 — Atom subtree slice 20: comment_group typed

**What changed:** `comment_group` now emits typed `{type:"atom", kind:"comment", text:<string>}` objects.

```ebnf
comment_group = "(?#" comment_text ")"
                  -> {type: "atom", kind: "comment", text: $2}

# rewritten from `comment_char*` chain:
comment_text  = /([^)]*)/
```

**Before / after:**

| Source | Before | After |
|---|---|---|
| `(?#hello)` | `["(?#", [<comment_char chain>], ")"]` | `{type:"atom", kind:"comment", text:"hello"}` |
| `(?#)` | `["(?#", [], ")"]` (empty Quantified) | `{type:"atom", kind:"comment", text:""}` |
| `(?#multi word comment)` | similar 3-element Sequence | `{text:"multi word comment"}` |
| `(?#with [special] chars)` | similar | `{text:"with [special] chars"}` |

**`text` is always a string.** Empty comments (`(?#)`) emit `text:""` (real empty string), not `[]` from an un-matched optional slot. The `?` after `comment_text` in `comment_group` was dropped because the regex literal accepts the empty match — `comment_text` always succeeds.

**Char-set coverage** of `[^)]*` matches the previous `comment_char*` chain semantics: any char except `)`.

**Atom subtree campaign progress:** 8/25 atom alternatives directly typed.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.50 / Contract 1.1.52 Highlights".

### 1.1.49 / Contract 1.1.51 — Atom subtree slice 19: python_named_backreference typed

**What changed:** `python_named_backreference` now emits typed `{type:"backreference", kind:"python_named", ref:<name>}` objects.

```ebnf
python_named_backreference = "(?P=" name ")"
                              -> {type: "backreference", kind: "python_named", ref: $2}
```

**Before / after:**

| Source | Before | After |
|---|---|---|
| `(?P=foo)` | `["(?P=", "foo", ")"]` | `{type:"backreference", kind:"python_named", ref:"foo"}` |
| `(?P=bar_baz)` | similar 3-element Sequence | `{kind:"python_named", ref:"bar_baz"}` |
| `(?P=x)` | similar | `{kind:"python_named", ref:"x"}` |

`name` was already a clean string after slice 11 (named-ref cleanup), so `$2` extracts directly.

**`kind` distinguishes from `\k<...>` even though semantics are equivalent.** PCRE2 treats `(?P=foo)` and `\k<foo>` as the same match operation, but tooling that wants to preserve the syntax origin can dispatch on `kind`. Consumers normalizing across all name-based forms: `kind in {"named", "named_braced", "python_named"}` → "name-based backref"; `ref` is the name string in all three.

**Atom subtree campaign progress:** 7/25 atom alternatives directly typed. **Backreference family typing is now end-to-end across all 5 syntactic forms** (numeric, named, named_braced, subroutine, python_named).

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.49 / Contract 1.1.51 Highlights".

### 1.1.48 / Contract 1.1.50 — Atom subtree slice 18: quoted_literal typed

**What changed:** `quoted_literal` now emits typed `{type:"atom", kind:"quoted_literal", body:<chars>}` objects.

```ebnf
quoted_literal = "\\Q" quoted_literal_char* "\\E"
                  -> {type: "atom", kind: "quoted_literal", body: $2}
```

**Before / after:**

| Source | Before | After |
|---|---|---|
| `\Qhello\E` | `["\\Q", ["h","e","l","l","o"], "\\E"]` | `{type:"atom", kind:"quoted_literal", body:["h","e","l","l","o"]}` |
| `\Q\E` (empty) | `["\\Q", [], "\\E"]` | `{type:"atom", kind:"quoted_literal", body:[]}` |
| `\Qabc def\E` | similar 3-element seq | `{body:["a","b","c"," ","d","e","f"]}` |

`body` is the array of `quoted_literal_char*` matches — one element per char. `quoted_literal_escaped_char` produces 2-char strings (the `\` and the escaped char). Consumers join to recover the literal string; consumers with semantic needs can distinguish escaped vs raw chars from element shapes.

**Atom subtree campaign progress:** 6/25 atom alternatives directly typed; 7/7 escape_unit branches typed.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.48 / Contract 1.1.50 Highlights".

### 1.1.47 / Contract 1.1.49 — Atom subtree slice 17: escape subtree closes (property)

**What changed:** `property_escape` per-branch annotations now emit typed `{type:"escape", kind:"property", name:<string>, negated:<bool>}` objects. `prop_name` and `short_prop_letter` rewritten from chained forms to regex literals so they emit clean string Terminals.

```ebnf
property_escape = "p{" prop_name "}"      -> {type: "escape", kind: "property", name: $2, negated: false}
                | "P{" prop_name "}"      -> {type: "escape", kind: "property", name: $2, negated: true}
                | "p" short_prop_letter   -> {type: "escape", kind: "property", name: $2, negated: false}
                | "P" short_prop_letter   -> {type: "escape", kind: "property", name: $2, negated: true}

# rewritten from `prop_name_chars+` chain:
prop_name = /([A-Za-z0-9 \t\n\r\f\v_:\-=&^]+)/

# rewritten from Or-of-single-chars chain:
short_prop_letter = /([CLMNPSZclmnpsz])/
```

**Before / after:**

| Source | Before (post-slice-16) | After |
|---|---|---|
| `\p{Lu}` | `["\\", ["p{", [["L"], ["u"]], "}"]]` | `{type:"escape", kind:"property", name:"Lu", negated:false}` |
| `\p{Letter}` | similar 3-level chain | `{type:"escape", kind:"property", name:"Letter", negated:false}` |
| `\P{Nd}` | similar w/ leading `"P{"` | `{type:"escape", kind:"property", name:"Nd", negated:true}` |
| `\pL` | `["\\", ["p", "L"]]` | `{type:"escape", kind:"property", name:"L", negated:false}` |
| `\PN` | `["\\", ["P", "N"]]` | `{type:"escape", kind:"property", name:"N", negated:true}` |

**`negated` is a real boolean.** Consumers get `true` / `false` directly from `obj.negated`, no need to inspect leading-token shape (`"p"` vs `"P"`) to infer negation.

**Atom subtree campaign progress:** 5/25 atom alternatives directly typed; **7/7 escape_unit branches typed** (single_byte, simple, control, hex, unicode, octal, property). The escape subtree is now **closed**.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.47 / Contract 1.1.49 Highlights".

### 1.1.46 / Contract 1.1.48 — Atom subtree slice 16: escape subtree continues (octal)

**What changed:** `octal_escape` per-branch annotations now emit typed `{type:"escape", kind:"octal", digits:<octal-string>}` objects. New `octal_escape_short_payload` regex literal for the 1-3-digit bare form. `octal_digits` rewritten from `octal_digit+` chain to regex literal.

```ebnf
octal_escape = "o{" brace_ws? octal_digits brace_ws? "}"  -> {type: "escape", kind: "octal", digits: $3}
             | octal_escape_short_payload                  -> {type: "escape", kind: "octal", digits: $1}
octal_escape_short_payload = /([0-7]{1,3})/

# rewritten from `octal_digit+` chain:
octal_digits = /([0-7]+)/
```

**Before / after:**

| Source | Before (post-slice-15) | After |
|---|---|---|
| `\o{777}` (atom) | `["\\", ["o{", [], <digits chain>, [], "}"]]` | `{type:"escape", kind:"octal", digits:"777"}` |
| `\o{7777}` (atom) | similar 5-level chain | `{type:"escape", kind:"octal", digits:"7777"}` |
| `[\377]` (in class via `class_range_escape_unit`) | `["\\", ["3", ["7"], ["7"]]]` | `{type:"escape", kind:"octal", digits:"377"}` |

**Important caveat — bare `\NNN` at atom-level:** Outside of character classes, `\377` and similar bare-digit forms are parsed as `{type:"backreference", kind:"numeric", index:377}` under the existing PEG-ordered atom alternation (numeric-backref branch shadows bare-octal). Pre-existing behavior; not changed by this slice. PEG cannot express PCRE2's contextual disambiguation ("if NNN ≤ 9 OR there are NNN capture groups, treat as backref; else octal") directly. Disambiguation is left to consumers via post-parse semantic analysis if/when they need atom-level bare-octal support.

**`digits` is a string, not an int.** Consumers parse with `usize::from_str_radix(digits, 8)`. Same constraint as hex/unicode; extending `@transform` is a separate codegen-feature slice.

**Atom subtree campaign progress:** 5/25 atom alternatives directly typed; **6/7 escape_unit branches typed** (single_byte, simple, control, hex, unicode, octal). Only remaining un-typed escape_unit branch: `property_escape`.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.46 / Contract 1.1.48 Highlights".

### 1.1.45 / Contract 1.1.47 — Atom subtree slice 15: escape subtree continues (hex/unicode)

**What changed:** `hex_escape` and `unicode_escape` now emit typed `{type:"escape", kind:<form>, digits:<hex-string>}` objects:

```ebnf
hex_escape = "x" hex_escape_short_payload                    -> {type: "escape", kind: "hex", digits: $2}
           | "x{" brace_ws? hex_digits brace_ws? "}"         -> {type: "escape", kind: "hex", digits: $3}
hex_escape_short_payload = /([0-9A-Fa-f]{1,2})/

unicode_escape = "u{" hex_digits "}" -> {type: "escape", kind: "unicode", digits: $2}

# hex_digits rewritten from `hex_digit+` (multi-element chain) to a regex literal:
hex_digits = /([0-9A-Fa-f]+)/
```

**Consumer impact:** **breaking but correct** — hex and unicode escape atoms now read as field accesses:

| Source | Before | After |
|---|---|---|
| `\xF` | `["\\", ["x", "F", []]]` (3-elem chain) | `{type:"escape", kind:"hex", digits:"F"}` |
| `\xFF` | `["\\", ["x", "F", [["F"]]]]` | `{type:"escape", kind:"hex", digits:"FF"}` |
| `\x{1F}` | `["\\", ["x{", [], <hex_digits chain>, [], "}"]]` | `{type:"escape", kind:"hex", digits:"1F"}` |
| `\x{1F600}` | similar | `{type:"escape", kind:"hex", digits:"1F600"}` |
| `\u{41}` | `["\\", ["u{", <hex_digits chain>, "}"]]` | `{type:"escape", kind:"unicode", digits:"41"}` |
| `\u{1F600}` | similar | `{type:"escape", kind:"unicode", digits:"1F600"}` |

**`digits` is a string, not an int.** Consumers parse with `usize::from_str_radix(digits, 16)`. Int decoding via the rule's `@transform` would require extending the transform machinery (currently `@transform` is hard-coded to `str::parse::<TYPE>().unwrap_or(DEFAULT)`-style; `from_str_radix` needs a different signature). Saved as a separate codegen-feature slice.

**`hex_digits` rewrite** (was `hex_digit+`, now `/([0-9A-Fa-f]+)/`) cascades to clean strings everywhere `hex_digits` is used (the braced hex/unicode forms above). The standalone `hex_digit` rule is retained for any rule that still uses it directly.

**Atom subtree progress:** 5/25 alternatives directly typed; 5/7 escape_unit branches typed (single_byte, simple, control, hex, unicode). Remaining escape_unit branches: octal_escape, property_escape.

**Note on `\u` validator rejection:** PGEN's host-side compile validator currently rejects `\u{...}` escapes ("unsupported regex escape `\u`"). The annotation works correctly when the validator allows the escape through; for inputs the validator rejects, no AST is produced. That validator behavior is pre-existing and out of scope for this slice.

**Contract section:** "Release 1.1.45 / Contract 1.1.47 Highlights".

### 1.1.44 / Contract 1.1.46 — Atom subtree slice 14: escape subtree starts (simple/single_byte/control)

**What changed:** the `escape` rule and 3 of its 7 sub-rules got typed annotations:

```ebnf
escape         = "\\" escape_unit -> $2
single_byte_escape = "C" -> {type: "escape", kind: "single_byte"}
simple_escape  = any_char -> {type: "escape", kind: "shorthand", char: $1}
control_escape = "c" any_char -> {type: "escape", kind: "control", char: $2}
```

`escape` is a transparent wrapper — each sub-rule constructs its own typed `{type:"escape", kind:<form>, ...}` object, and `escape` passes through via `-> $2`.

**Consumer impact:** **breaking but correct** — most common escape forms now emit typed objects:

| Source | Kind | Output |
|---|---|---|
| `\d` `\w` `\s` `\.` `\\` `\n` (any letter/symbol) | `shorthand` | `{type:"escape", kind:"shorthand", char:<c>}` |
| `\C` | `single_byte` | `{type:"escape", kind:"single_byte"}` |
| `\cA` `\cZ` `\cz` | `control` | `{type:"escape", kind:"control", char:<C>}` |
| `\xFF` `\x{1F}` | `hex` (still raw) | un-typed — follow-up slice |
| `\u{1F600}` | `unicode` (still raw) | un-typed — follow-up slice |
| `\377` `\o{777}` | `octal` (still raw) | un-typed — follow-up slice |
| `\p{Lu}` `\PL` | `property` (still raw) | un-typed — follow-up slice |

Pre-fix, every escape produced a deeply-nested chain `["\\", [[[[[<char>]]]]]]` (4-5 levels of un-annotated wrappers around the simple_escape's any_char Terminal). Post-fix, `\d` is a single field read on `obj.kind` + `obj.char`. The chain disappears entirely for the 3 typed branches.

**Limitation — 4 escape_unit branches still raw.** `\xFF`, `\u{...}`, `\377`/`\o{...}`, `\p{...}`/`\PL` still emit their pre-fix raw shapes. Each requires digit-decoding (hex/unicode/octal) or property-name extraction. Follow-up slices will type them one by one.

**Atom subtree progress:** 5/25 alternatives directly typed (anchor, posix_class, posix_word_boundary_alias, backreference, escape outer). 3/7 escape_unit branches typed (single_byte, simple, control).

**Contract section:** "Release 1.1.44 / Contract 1.1.46 Highlights".

### 1.1.43 / Contract 1.1.45 — Atom subtree slice 13: signed_digits typing (backref family fully typed end-to-end)

**What changed:** the `signed_digits` rule now produces a typed `{sign, value}` object:

```ebnf
signed_digits = sign? digits -> {sign: $1, value: $2}
```

`sign` is `"+"`/`"-"` when the optional sign matched, or `[]` (the un-matched `Quantified-?` slot) when no sign was present. `value` is the typed integer from `digits`.

**Consumer impact:** **breaking but correct** — numeric subroutine refs now read `obj.ref.sign` and `obj.ref.value` as named fields:

| Source | Before | After |
|---|---|---|
| `\g<1>` | `ref: [[], 1]` | `ref: {"sign":[], "value":1}` |
| `\g<-2>` | `ref: ["-", 2]` | `ref: {"sign":"-", "value":2}` |
| `\g<+5>` | `ref: ["+", 5]` | `ref: {"sign":"+", "value":5}` |
| `\g{42}` | `ref: [[], 42]` | `ref: {"sign":[], "value":42}` |
| `\g+1` | `ref: ["+", 1]` | `ref: {"sign":"+", "value":1}` |
| `\g-3` | `ref: ["-", 3]` | `ref: {"sign":"-", "value":3}` |

**Backreference family — typed end-to-end:**
- `kind:"numeric"` → `index:<int>`.
- `kind:"named"` / `kind:"named_braced"` → `ref:<string>`.
- `kind:"subroutine"` → `ref:<string>` (named) OR `ref:{sign:..., value:...}` (numeric, typed).

Consumer code is now field reads end-to-end — no more dispatching on `is_string()` vs `is_array()` for the numeric form.

**Atom subtree progress:** 4/25 alternatives directly typed; backreference family fully typed end-to-end.

**Contract section:** "Release 1.1.43 / Contract 1.1.45 Highlights".

### 1.1.42 / Contract 1.1.44 — Atom subtree slice 12: subroutine_ref cleanup (closes backref family)

**What changed:** the `subroutine_ref` rule's 4 branches each got per-branch annotations to drop the angle/quote/brace delimiters and surface the inner `signed_digits_or_name` directly. `braced_subroutine_ref` annotated similarly:

```ebnf
subroutine_ref = braced_subroutine_ref          -> $1
              | "<" signed_digits_or_name ">"   -> $2
              | "'" signed_digits_or_name "'"   -> $2
              | signed_digits                   -> $1
braced_subroutine_ref = "{" brace_ws? signed_digits_or_name brace_ws? "}" -> $3
```

**Consumer impact:** **breaking but correct** — `\g<...>` family backreferences now surface clean inner values:

| Source | Before | After |
|---|---|---|
| `\g<name>` | `ref: ["<", "name", ">"]` (slice 11 cleaned inner) | `ref: "name"` |
| `\g'name'` | `ref: ["'", "name", "'"]` | `ref: "name"` |
| `\g{name}` | `ref: ["{", _, "name", _, "}"]` | `ref: "name"` |
| `\g<1>` | `ref: ["<", [[], 1], ">"]` | `ref: [[], 1]` |
| `\g<-2>` | `ref: ["<", ["-", 2], ">"]` | `ref: ["-", 2]` |
| `\g{42}` | `ref: ["{", _, [[], 42], _, "}"]` | `ref: [[], 42]` |
| `\g+1` | `ref: ["+", 1]` (already raw signed_digits) | `ref: ["+", 1]` (unchanged) |

**Backreference family closed for naming:**
- `kind:"named"` / `kind:"named_braced"` → `ref` is a clean string.
- `kind:"numeric"` → `index` is a typed integer.
- `kind:"subroutine"` → `ref` is either a clean string (named form) or `[<sign?>, <digit-int>]` (numeric form).

**Limitation — `signed_digits` still raw.** `signed_digits = sign? digits` is still un-annotated, so `\g<1>` produces `ref: [[], 1]` — a 2-element array `[<sign?-Quantified>, <typed integer>]`. Consumer dispatches by:
- `obj.ref.is_string()` → it's a name.
- `obj.ref.is_array()` → it's signed_digits; walk `[<sign>, <int>]`.

A future sub-slice will type `signed_digits` to `{sign: <"+"|"-"|null>, value: <int>}` for cleaner consumer ergonomics.

**Atom subtree progress:** 4/25 alternatives directly typed; named-ref + subroutine-ref family fully cleaned (closes backreference deep typing modulo the signed_digits sub-slice).

**Contract section:** "Release 1.1.42 / Contract 1.1.44 Highlights".

### 1.1.41 / Contract 1.1.43 — Atom subtree slice 11: named-ref cleanup (clean name strings)

**What changed:** three grammar changes that surface clean name strings everywhere a name appears:

1. `name` rewritten as a regex literal:
   ```ebnf
   name = /((?:[A-Za-z_]|[^\x00-\x7F])(?:[A-Za-z0-9_]|[^\x00-\x7F])*)/
   ```
   Pre-rewrite the rule was `name_start name_continue*` (multi-element body) which produced a deeply-nested `[first_char, [rest_chars]]` Sequence requiring consumers to concatenate. Post-rewrite the rule emits a Terminal of the matched name string directly.

2. `name_ref` (used by `\k<name>` / `\k'name'` backreferences) annotated to extract just the name:
   ```ebnf
   name_ref = "<" name ">" -> $2
            | "'" name "'" -> $2
   ```

3. `braced_name_ref` (used by `\k{name}`) annotated similarly:
   ```ebnf
   braced_name_ref = "{" brace_ws? name brace_ws? "}" -> $3
   ```

**Consumer impact:** **breaking but correct** — every consumer that walked the raw inner chain to recover a name string now reads the clean string directly:

| Source | Before | After |
|---|---|---|
| `\k<foo>` | `ref: ["<", ["f", ["o", "o"]], ">"]` | `ref: "foo"` |
| `\k{foo}` | `ref: ["{", [], ["f", ["o", "o"]], [], "}"]` | `ref: "foo"` |
| `(?<bar>x)` | atom contains name as `["b", ["a", "r"]]` | atom contains name as `"bar"` |
| `(?P<bar>x)` | similar raw chain | `"bar"` |
| `(?P=bar)` | similar raw chain | `"bar"` |
| `\g<name>` | `ref: ["<", ["n", ["a", "m", "e"]], ">"]` | `ref: ["<", "name", ">"]` (subroutine_ref still un-annotated; inner name now clean) |

**Cascading scope:** this slice affects every grammar rule that references `name` directly or through a wrapper — named groups (5 forms), backreferences (`\k<...>`/`\k{...}`), python named back-refs (`(?P=name)`), subroutine targets (`\g&name`/`\g<name>`/`\g{name}`), conditions (`(?(name)...)` / `(?(R&name)...)`), property escapes' `prop_name` is unrelated and not affected.

**Limitation — `subroutine_ref` still un-annotated.** `\g<name>` etc. still emit the raw `["<", <inner>, ">"]` shape, but the inner now carries a clean name string instead of a character chain. Follow-up slice will type `subroutine_ref` to drop the angle/brace/quote wrappers.

**Atom subtree progress:** 4/25 alternatives directly typed; the named-ref family now emits clean string values everywhere `name` is used.

**Contract section:** "Release 1.1.41 / Contract 1.1.43 Highlights".

### 1.1.40 / Contract 1.1.42 — PGEN-RGX-0077: `[$1**]` flatten-spread peels `Alternative`

**Bug** (RGX bug report PGEN-RGX-0077): every multi-char `\Q...\E quantifier?` source produced one extra wrapping layer at `pattern[0][0]` — `[[<N pieces>]]` (1-element array containing the pieces array) instead of the documented flat `[<N pieces>]`. The piece data was correct; the bug was purely structural wrapping. Adjacent regression to PGEN-RGX-0075 on a different codegen path.

**Root cause:** the `[$1**]` flatten-spread codegen in `rust/src/ast_pipeline/ast_return_transform.rs` did not peel `Alternative` wrapping before inspecting child content for the unwrap decision. The codegen wraps Or-rule and rule-reference branch results in `Alternative(boxed_inner)`; for `concatenation = piece+ -> [$1**]`, each piece node arrives as `Alternative(piece_inner_node)`. Pre-fix, the inner `match node.content` saw `Alternative` and fell into the "push as-is" arm, wrapping the whole inner Sequence-of-pieces (from `piece_quoted_run_quantified -> [$2**, ...]`) as a single element instead of spreading.

**Fix:**
1. Peel `Alternative` recursively in the FlattenSpread codegen before the unwrap decision. Now `Alternative(inner)` → look at `inner.content` to decide how to spread.
2. Add a `ParseContent::Json(Value::Array(_))` arm (preventative — guards against the same family of regressions for any future annotation that builds typed-Json arrays).

**Consumer impact:** every multi-char `\Q...\E quantifier?` source now produces flat pieces at `pattern[0][0]`. Empirical:

| Source | Before | After |
|---|---|---|
| `\Qab*\E{2,}` | `[[3 pieces]]` | `[a, b, *{2,}]` |
| `\Qabc\E?` | `[[3 pieces]]` | `[a, b, c?]` |
| `\Qabcdef\E+` | `[[6 pieces]]` | `[a, b, c, d, e, f+]` |
| `\Qab\E{3}` | `[[2 pieces]]` | `[a, b{3}]` |

Single-char (`\Qa\E{3}`) and empty (`\Q\E{2}`) cases unaffected — they hit the atom-fallback path, not `piece_quoted_run_quantified`.

**Regression-lock test:** `regex_parser_pgen_rgx_0077_quoted_run_quantified_pieces_flat_in_concatenation` in `rust/src/embedding_api.rs` pins the family-table coverage from the bug report (9 multi-char `\Q...\E quantifier?` shapes). Asserts piece count + atom values + quantifier-attached-to-last-piece + no-quantifier-on-inner-pieces.

**Contract section:** "Release 1.1.40 / Contract 1.1.42 Highlights".

### 1.1.39 / Contract 1.1.41 — Atom subtree slice 10: typed `backreference` shape

**What changed:** the `backreference` rule's 4 branches each got per-branch typed annotations:

```ebnf
backreference  = "\\" backreference_digits  -> {type: "backreference", kind: "numeric",      index: $2}
               | "\\k" name_ref             -> {type: "backreference", kind: "named",        ref:   $2}
               | "\\k" braced_name_ref      -> {type: "backreference", kind: "named_braced", ref:   $2}
               | "\\g" subroutine_ref       -> {type: "backreference", kind: "subroutine",   ref:   $2}
```

`backreference_digits` rewritten as a regex literal `/([1-9][0-9]*)/` with `@transform: str::parse::<usize>().unwrap_or(0)` so branch 0's `index` field is a typed integer directly (mirrors how `digits` was typed in slice 1).

**Consumer impact:** **breaking but correct** — consumers walking backreference atoms must update from `["\\", <digits>]` 2-element array dispatch to typed `obj.kind` lookup:

| Source | Before | After |
|---|---|---|
| `\1` | `["\\", ["1"]]` | `{"type":"backreference","kind":"numeric","index":1}` |
| `\23` | `["\\", ["2", ["3"]]]` | `{"type":"backreference","kind":"numeric","index":23}` |
| `\k<foo>` | `["\\k", ["<", <name>, ">"]]` | `{"type":"backreference","kind":"named","ref":[..raw name_ref..]}` |
| `\k{foo}` | `["\\k", ["{", _, <name>, _, "}"]]` | `{"type":"backreference","kind":"named_braced","ref":[..raw braced_name_ref..]}` |
| `\g{2}` | `["\\g", [..]]` | `{"type":"backreference","kind":"subroutine","ref":[..raw subroutine_ref..]}` |

**Limitation:** for branches 1-3 (`named` / `named_braced` / `subroutine`), the `ref` field carries the inner sub-rule's RAW shape — `name_ref`, `braced_name_ref`, and `subroutine_ref` are still un-annotated as of this slice. Consumers walking the name string need to descend the raw chain. A follow-up slice will type those rules so `ref` becomes `{name: <str>}` for named refs and `{kind: <numeric|named|signed_numeric>, value: ...}` for subroutine refs.

**Atom subtree progress:** 4 of 25 alternatives annotated (anchor, posix_class, posix_word_boundary_alias, backreference).

**Contract section:** "Release 1.1.39 / Contract 1.1.41 Highlights".

### 1.1.38 / Contract 1.1.40 — Atom subtree slice 9: typed `posix_word_boundary_alias` (closes anchor family)

**What changed:** the `posix_word_boundary_alias` rule's 2 branches each annotated to emit the same typed anchor shape as the `anchor` rule:

```ebnf
posix_word_boundary_alias = "[[:<:]]" -> {type: "anchor", kind: "posix_word_start"}
                          | "[[:>:]]" -> {type: "anchor", kind: "posix_word_end"}
```

PCRE2's POSIX-style word-boundary aliases (`[[:<:]]` / `[[:>:]]`) are anchors despite the character-class-looking syntax. They now join the typed anchor family — consumers can dispatch uniformly on `obj.type == "anchor"` regardless of whether the source used `\b` (regular) or `[[:<:]]`/`[[:>:]]` (POSIX-style).

**Consumer impact:** **breaking but correct** — consumers walking the `posix_word_boundary_alias` atom must update from `atom.as_str() == "[[:<:]]"` to `atom.get("kind").as_str() == "posix_word_start"` (and similar for end). After this slice, the consumer-side `classify_anchor` recipe in [Examples: Anchors](examples-anchors.md) covers all 11 anchor variants with no fallback paths.

**Anchor family closed:** all 11 anchor variants — 9 from `anchor` (slice 7) + 2 from `posix_word_boundary_alias` (this slice) — emit the same typed `{type:"anchor", kind:<name>}` shape:

| Source | `kind` |
|---|---|
| `^` | `start_of_line` |
| `$` | `end_of_line` |
| `\A` | `start_of_input` |
| `\Z` | `end_of_input_or_before_last_newline` |
| `\z` | `end_of_input` |
| `\b` | `word_boundary` |
| `\B` | `non_word_boundary` |
| `\G` | `match_start` |
| `\K` | `keep_out` |
| `[[:<:]]` | `posix_word_start` |
| `[[:>:]]` | `posix_word_end` |

**Atom subtree progress:** 3 of 25 alternatives annotated (anchor, posix_class, posix_word_boundary_alias). Note: anchor and posix_word_boundary_alias are siblings under `atom`, but together they close the anchor-shaped family (3 grammar atoms emit the same typed shape).

**Contract section:** "Release 1.1.38 / Contract 1.1.40 Highlights".

### 1.1.37 / Contract 1.1.39 — PGEN-RGX-0076: typed `posix_class` shape (slice 8)

**Bug**: RGX bug report PGEN-RGX-0076 — every POSIX class inside a character class collapsed to the literal string `"[:"` in the typed shape. The grammar had a placeholder annotation `posix_class = "[:" posix_negation? posix_name ":]" -> $1` which extracted only the FIRST element (the `"[:"` opener), silently discarding the matched POSIX name and any negation marker.

**Fix**:
```ebnf
posix_class = "[:" posix_negation? posix_name ":]"
-> {type: "posix_class", name: $3, negated: $2}

posix_negation = "^" -> true
```

**Codegen fixes** in the same commit:
- `BooleanLiteral` codegen at the rule-level scalar path (`generate_transform`) was emitting `ParseContent::Terminal(<bool_str>)` — a string Terminal `"true"`/`"false"` — instead of a typed JSON boolean. Surfaced when `posix_negation -> true` produced `"true"` (string) instead of `true` (bool).
- `NumberLiteral` codegen at the same path had the analogous bug. Both now emit `ParseContent::Json(serde_json::Value::Bool/Number(...))` mirroring the value-extraction path.

**Consumer impact:** **breaking but correct**. Every POSIX class inside `[...]` now emits a typed object:

| Source | Before | After |
|---|---|---|
| `[[:alpha:]]` | `class_body[0] = "[:"` | `class_body[0] = {"type":"posix_class","name":"alpha","negated":[]}` |
| `[[:^alpha:]]` | `class_body[0] = "[:"` | `class_body[0] = {"type":"posix_class","name":"alpha","negated":true}` |
| `[[:alpha:][:digit:]]` | `class_body = ["[:", "[:"]` (both truncated identically) | `class_body = [{type:posix_class,name:alpha,negated:[]}, {type:posix_class,name:digit,negated:[]}]` |

Consumers walking the typed shape can drop any source-span fallback they had for POSIX class name recovery — the typed object preserves `name` and `negated` directly.

**`negated` convention:** `true` (matched `^`) or `[]` (un-matched `posix_negation?` slot — map to `false`). Same convention as `quantifier.greediness`. A future coalesce-operator slice will let the rule emit a bare `false` directly.

**Contract section:** "Release 1.1.37 / Contract 1.1.39 Highlights".

### 1.1.36 / Contract 1.1.38 — Atom subtree slice 7: typed `anchor` shape

**What changed:** the `anchor` rule's 9 branches each got `-> {type: "anchor", kind: "<name>"}` annotations. Piece atoms for `^`/`$`/`\A`/`\Z`/`\z`/`\b`/`\B`/`\G`/`\K` now emit typed objects with semantic kind names instead of raw escape strings.

```ebnf
anchor = "^"   -> {type: "anchor", kind: "start_of_line"}
       | "$"   -> {type: "anchor", kind: "end_of_line"}
       | "\\A" -> {type: "anchor", kind: "start_of_input"}
       | "\\Z" -> {type: "anchor", kind: "end_of_input_or_before_last_newline"}
       | "\\z" -> {type: "anchor", kind: "end_of_input"}
       | "\\b" -> {type: "anchor", kind: "word_boundary"}
       | "\\B" -> {type: "anchor", kind: "non_word_boundary"}
       | "\\G" -> {type: "anchor", kind: "match_start"}
       | "\\K" -> {type: "anchor", kind: "keep_out"}
```

**Consumer impact:** **breaking but correct** — consumers dispatching on the raw escape text must switch to the typed `obj.kind` field. The kind names are stable identifiers and won't change if PCRE2 syntax evolves. See [Examples: Anchors and Boundaries](examples-anchors.md) for the full migration recipe.

| Source | Before | After |
|---|---|---|
| `^` | `"atom": "^"` | `"atom": {"type":"anchor","kind":"start_of_line"}` |
| `\b` | `"atom": "\\b"` | `"atom": {"type":"anchor","kind":"word_boundary"}` |
| `\K` | `"atom": "\\K"` | `"atom": {"type":"anchor","kind":"keep_out"}` |

**Note:** the POSIX word-boundary aliases (`[[:<:]]` and `[[:>:]]`, handled by the `posix_word_boundary_alias` rule) still emit raw 7-char terminals. They will join the typed family in a follow-up slice.

**Atom subtree campaign progress:** 1 of 25 alternatives annotated. Next focus areas: `dot`, `literal`, `backreference`, `quoted_literal`, `escape`, `posix_word_boundary_alias`, `char_class`, group family, etc.

**Contract section:** "Release 1.1.36 / Contract 1.1.38 Highlights".

### 1.1.35 / Contract 1.1.37 — Quantifier subtree closure (slice 6/N)

**What changed:** the final two rules in the quantifier subtree got their typed annotations:

- `quant_base` reshaped from per-branch `-> $1` (heterogeneous: string for shorthand, object for counted) to per-branch typed `{min, max}` for every alternative:

  ```ebnf
  quant_base = "*"                -> {min: 0, max: null}
             | "+"                -> {min: 1, max: null}
             | "?"                -> {min: 0, max: 1}
             | counted_quantifier -> $1
  ```

- `quantifier` rule annotated:

  ```ebnf
  quantifier = quant_base quant_suffix?
  -> {type: "quantifier", min: $1.min, max: $1.max, greediness: $2}
  ```

**Consumer impact:** **breaking but correct** — the piece's `quantifier` field is now a fully typed `{type, min, max, greediness}` object instead of a `[<base>, <suffix>]` 2-tuple. Empirical:

| Input | Before | After |
|---|---|---|
| `a*` | `["*", []]` | `{"type":"quantifier","min":0,"max":null,"greediness":[]}` |
| `a+?` | `["+", "lazy"]` | `{"type":"quantifier","min":1,"max":null,"greediness":"lazy"}` |
| `a{2,5}` | `[{"min":2,"max":5}, []]` | `{"type":"quantifier","min":2,"max":5,"greediness":[]}` |

Pieces with NO quantifier still have `"quantifier": []` (empty `quantifier?` slot — unchanged).

`greediness: []` is the un-matched `quant_suffix?` slot — interpret as PCRE2's greedy default. Consumers map `[]` → `"greedy"`. This will be removed when the annotation language gains a coalesce operator and `quantifier`'s annotation can emit the literal string `"greedy"` directly.

**Quantifier-subtree campaign closed:** all six rules (`digits`, `quant_suffix`, `counted_quantifier_body`, `counted_quantifier`, `quant_base`, `quantifier`) are now annotated. Consumer-side `extract_quantifier` walker collapses to a six-line typed-field read.

**Contract section:** "Release 1.1.35 / Contract 1.1.37 Highlights".

### 1.1.34 / Contract 1.1.36 — PGEN-RGX-0075 typed-shape correctness for multi-piece concatenation

**What changed:** The `$N` PositionalRef codegen no longer peels `elements[0]` from a `Quantified` base when the rule body has a single capture position. With this fix, `concatenation = piece+ -> [$1**]` correctly resolves `$1` to the whole `Quantified` (every piece), so `**` flattens all of them into the array.

Compensating grammar change: `regex = pattern -> ...` (was `regex = pattern? -> ...`). The inner `alternative = concatenation?` already handles the empty-input case, so the outer `?` was redundant and only existed to prop up the buggy auto-peel behaviour.

**Consumer impact:** **breaking but in the right direction** — anyone walking `regex.pattern[0][0]` for a multi-piece concatenation now sees every piece. Pre-fix, `"abc"` produced `pattern[0][0] == [piece_a]` (1 piece, buggy); post-fix, `pattern[0][0] == [piece_a, piece_b, piece_c]` (3 pieces, correct). The top-level pattern shape `[<head_alt>, <tail>]` is unchanged. RGX caught this when `Regex::compile("abc")` matched only `"a"` instead of `"abc"`.

**Bug history:** the `\Q...\E` family table covered `\Qab*\E{2,}` (3 pieces — passed incidentally because `piece_quoted_run_quantified`'s annotation pre-built a multi-element Sequence that `**` happened to flatten correctly), but no test asserted plain `"abc"` → 3 pieces in the typed output. PGEN-RGX-0074's empirical evidence focused on `\Q...\E` cases, which masked the underlying `$1`-extraction bug. Fixed in this release with a regression-lock test (`regex_parser_pgen_rgx_0075_multi_piece_concatenation_surfaces_all_pieces`) pinning the empirical shape for `"a"`, `"ab"`, `"abc"`, `"hello"`.

**Contract section:** "Release 1.1.34 / Contract 1.1.36 Highlights".

### Post-1.1.33 main — Task #38 fix: parens-grouped-Or trailing-annotation broadcast

**What changed:** the codegen now correctly applies a trailing return annotation on a parens-grouped Or to **every** alternative inside the group, not just the first. Affects both `extract_rule_annotations` (rust/src/ast_pipeline/mod.rs) and the cross-checker `extract_declared_annotations_from_json` (rust/src/ast_shape_contract.rs).

Pre-fix behaviour: `RULE = (A | B | C) -> ann` applied `ann` to branch 0 only; branches 1, 2 silently fell through to raw passthrough. Documented in `parse_string_literal` of the return_annotation grammar — single-quoted strings produced raw `Sequence` instead of the typed `{type:"string", value:...}` that double-quoted strings produced.

Post-fix behaviour: when a return annotation immediately follows a `group_close`, the annotation broadcasts to every branch that was inside the just-closed group. Per-branch annotations on un-grouped Or rules (`A | B -> ann`, where `-> ann` binds to B only per PEG precedence) still work as before.

**Consumer impact for return_annotation grammar:** single-quoted strings now produce `Json({"type":"string", "value":"..."})` — same as double-quoted. Anyone relying on the buggy raw-Sequence shape needs to update walking code.

**Consumer impact for regex grammar:** none directly from the bugfix. But `quant_base` was refactored to the now-supported factored form `( "*" | "+" | "?" | counted_quantifier ) -> $1` (was per-branch `-> $1` four times) — same JSON output.

**Contract section:** pending bump (will land with the slice that closes `quantifier`).

### Post-1.1.33 main — quant_base annotated (slice 5/N)

**What changed:** `quant_base = "*" | "+" | "?" | counted_quantifier` got per-branch `-> $1` annotations on every alternative. After task #38 fix landed (subsequent commit), the rule was refactored to the factored form `quant_base = ( "*" | "+" | "?" | counted_quantifier ) -> $1` — semantically identical, more elegant.

**Consumer impact:** **none** — JSON output is byte-identical to pre-slice-5. Empirical: `a*` still emits `quantifier: ["*", []]`; `a{2,5}` still emits `quantifier: [{"min":2,"max":5}, []]`. The change is to the rule's emission status (from "raw envelope via codegen default" to "annotated, Tier-2 stable").

**Contract section:** pending bump (will land with the slice that closes `quantifier`).

### Post-1.1.33 main — counted_quantifier typed (slice 4/N)

**What changed:** `counted_quantifier` rule got `-> $3` annotation, lifting `counted_quantifier_body`'s typed `{min, max}` straight through and dropping the surrounding `{`/`}`/whitespace tokens. The brace tokens carry no semantic information beyond "this is a counted quantifier" — context the surrounding `quant_base` already conveys.

**Consumer impact:** the `quant_base` position (visible inside `quantifier`'s `[<base>, <suffix>]` shape) now carries either a bare string `"*"`/`"+"`/`"?"` OR a typed `{min, max}` object directly. No more digging through a 5-element Sequence wrapper to reach the body's typed shape. See [Quantifier Subtree](rules-quantifier.md).

**Contract section:** pending bump (will land with the slice that closes `quantifier`).

### Post-1.1.33 main — counted_quantifier_body typed + null literal (slice 3/N)

**What changed:** restructured `counted_quantifier_body` from 2 branches (with 4 logical cases compressed inside an optional sub-group) into 4 explicit branches each with its own per-branch `-> {min, max}` annotation. Added the `null` literal to the return-annotation language so the unbounded `{n,}` form can encode `max:null` directly.

**Consumer impact:** the body now emits a typed `{min, max}` object regardless of which `{n}`/`{n,}`/`{n,m}`/`{,m}` source form matched. `min` is always a typed integer; `max` is a typed integer OR `null` (only `null` for the unbounded form). See [Quantifier Subtree](rules-quantifier.md), [Quantifiers](examples-quantifiers.md).

**Contract section:** pending bump (will land with the slice that closes `quantifier`).

### 1.1.33 / Contract 1.1.35 — quant_suffix typed (slice 2/N)

**What changed:** `quant_suffix` rule's two branches each got a return annotation: `"?"` → `"lazy"`, `"+"` → `"possessive"`. The `quant_suffix?` slot inside `quantifier` now carries either `[]` (greedy, no suffix matched) or `["lazy"]` / `["possessive"]` (1-element Quantified-?).

**Consumer impact:** consumers reading the quantifier shape can dispatch on the string instead of inferring from which branch matched. See [Quantifier Subtree](rules-quantifier.md).

**Contract section:** "Release 1.1.33 / Contract 1.1.35 Highlights".

### 1.1.32 / Contract 1.1.34 — digits typed (slice 1/N)

**What changed:** `digits` rule got `@transform: str::parse::<usize>().unwrap_or(0)` annotation. The rule now emits a typed integer (`Json(Value::Number(usize))`) instead of a `Terminal` of digit characters.

**Consumer impact:** consumers reading counted-quantifier bounds get the integer directly. See [Quantifier Subtree](rules-quantifier.md).

**Contract section:** "Release 1.1.32 / Contract 1.1.34 Highlights".

### 1.1.31 / Contract 1.1.33 — PGEN-RGX-0074 `\Q...\E` correctness

**What changed:** introduced the `piece_quoted_run_quantified` rule + the `**` flatten-spread operator. Restructured `concatenation` to `piece+ -> [$1**]`. The `\Q...\E quantifier` shape now emits per-char piece array with the trailing piece carrying the quantifier — matching PCRE2's "quantifier binds to last char of \Q...\E" semantics.

**Consumer impact:** **breaking** — any consumer that walked the pre-1.1.31 whole-block-quantified shape will see a different AST. The new shape is what PCRE2 says it should be. Drop any pre-existing `\Q...\E`-quantifier-attachment workaround. See [\Q...\E Quoted Literals](examples-quoted-literal.md).

**Contract section:** "Release 1.1.31 / Contract 1.1.33 Highlights".

### 1.1.30 / Contract 1.1.32 — PGEN-RGX-0073 perf closure

**What changed:** primary focus was performance — closing all 8 patterns under p50 < 50µs. Schema-affecting changes were minor: implicit `-> $1` default tightened to exclude `Quantified` bodies (so single-Quantified rules without an explicit annotation no longer collapse to the inner item). Several rule rewrites of slow Or-of-chars rules using `/.../` regex literals (perf-only, no shape change).

**Consumer impact:** if a consumer relied on the implicit-collapse behavior for a Quantified body, they need to add an explicit `-> $1` annotation to that rule (or update their walker to handle the un-collapsed shape). Vanishingly rare in practice — the regex grammar didn't have any rules in this position. See contract for full details.

**Contract section:** "Release 1.1.30 / Contract 1.1.32 Highlights".

## Earlier releases (pre-1.1.30)

Pre-1.1.30 the regex parser used the recursive-envelope shape exclusively — no `Json` carrier, every rule produced `Sequence` / `Alternative` / `Quantified` / `Terminal`. Consumers who built against pre-1.1.30 should read [Migration from the Recursive Envelope](migration-from-recursive-envelope.md) before reading the rest of this book.

For shape-affecting changes within the pre-1.1.30 envelope era, see the contract document — its release sections go back to the parser's earliest releases.

## Future releases

Upcoming slice campaign work (task #40 — "Annotate regex.ebnf for full AST usability"):

| Slice | Target rule | Expected AST change |
|---|---|---|
| 6 | `quantifier` | Combine quant_base + quant_suffix? into typed `{type:"quantifier", min, max, greediness}` |
| 7+ | atom subtree, char class, group family, escape subtree, ... | One rule per slice |

Each slice will:

- Bump the contract version.
- Add a row to this index.
- Get a CHANGES.md entry.
- Get a corresponding chapter update where applicable.

The eventual schema 1.0.0 milestone will land when every regex.ebnf rule is either annotated or has a deliberate "remain raw envelope" decision documented.

## Looking up a specific input's behavior

If you need to know "what does the parser output for input X across release Y?," the workflow is:

1. Check `git log --oneline grammars/regex.ebnf` to see when the rule for X was last touched.
2. Check the contract section for that release.
3. Check the ledger if there was a bug fix involved.
4. As last resort, check out the relevant tag and run `parseability_probe --parse-dump-ast-pretty regex 'X'` against it.

For consumer-side regression detection, the recommended pattern is the snapshot test described in [Schema Versioning](schema-versioning.md).
