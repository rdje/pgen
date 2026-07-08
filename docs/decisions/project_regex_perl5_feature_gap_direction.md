# Regex feature roadmap — PCRE2 parity FIRST, then a Perl5-only feature arc (PROPOSED direction)

- Category: `project`
- Created: `2026-07-09` (session #73)
- Status: **PROPOSED — my take in response to the director's mid-turn prompt; pending ratification.**
- Related: [[project_regex_pcre2_faithful_by_default]] · [[project_regex_validator_deletion_blocked_load_bearing]] ·
  [[reference_pcre2_unsupported_escape_oracle]] · [[project_vision_and_discipline]]

## Context (director prompt, 2026-07-09)

The director flagged, mid-session, three related points (explicitly **not** a request to pivot now):
1. **PGEN-RGX-0078** (a regex *speed* issue from RGX) still exists and is already captured as a tree — but
   regex must be **feature-complete before** tackling 0078.
2. Right now the regex work is **PCRE2 parity** (tree `REGEX-PCRE2-FIDELITY`). What about **Perl5 regex
   features PCRE2 doesn't support at all** — worth closing that gap too?
3. A Perl5 arc **will need Perl5 regex test data / an external corpus**.
4. **But complete PCRE2 parity first — we are not there yet.**

## My take (the analysis)

**PCRE2 is deliberately a near-superset of Perl5** ("Perl Compatible Regular Expressions"), so the genuine
Perl-only *pattern-syntax* gap is a **small, specific set** — not a large surface. The tractable Perl-only
features, tool-verifiable against `perl` itself:

| Perl-only feature | PCRE2 status | Parser tractability |
|---|---|---|
| `(?{ CODE })` / `(??{ CODE })` embedded/dynamic Perl code | none (PCRE2 `(?C n)` callouts invoke a HOST callback, not Perl code) | **Parse-shape ONLY** — accept the syntax, emit an opaque `perl_code_block` atom; NEVER execute (PGEN has no Perl interpreter and shouldn't). The flagship Perl-only feature and the one with the hardest boundary. |
| `\b{wb}` `\b{sb}` `\b{gcb}` `\b{lb}` + `\B{…}` Unicode boundary types (Perl 5.22+) | not supported | **Cleanly syntactic + AST-expressible** — the best first candidate. |
| Variable-length lookbehind (Perl 5.30+ experimental) | bounded only | **A relaxation** of the PCRE2 bounded-lookbehind rule — synergy with the PCRE2-parity leaf `.4.9` (same rule, opposite direction). |
| `\p{IsUserSub}` user-defined property (backed by a Perl `sub`) | standard props only | Parse-shape-able (accept the name), not resolvable. |
| `(?[ … ])` extended bracketed class set-ops (Perl 5.18+ experimental) | verify current PCRE2 10.4x support first — may already be parity | Syntactic if PCRE2 lacks it. |

Everything else people call "Perl regex" — named captures `(?<n>…)`, atomic groups, possessive quantifiers,
`\K`, `\R`/`\X`/`\h`/`\v`/`\N`, `(?^…)` reset modifiers, `\g{…}`/`\k<…>` — is in **both** and is parity work,
not a Perl-only gap.

### Architecture fit — clean, zero-engine

The project already has the machinery: the ratified **"PCRE2-faithful by default + `relaxed` opt-out"**
doctrine + the `@profiles` directive. A Perl5 arc is the **natural third profile** — tag each Perl-only rule
`@profiles: ["perl5"]`, exactly the `relaxed` precedent. Grammar-declarative, no engine change. The default
stays strict PCRE2; a caller opts into `perl5` the same way it opts into `relaxed`.

### Oracle + corpus (the director's point 3)

Mirror the PCRE2 lane: PCRE2 parity uses `pcre2test` as the executable oracle + a PCRE2 corpus; a Perl5 arc
uses **`perl` itself** as the authoritative oracle (compile a pattern via `qr//` under `use re 'strict'` and
capture the compile verdict/error) + **Perl's own regex regression suite** as the external corpus (`t/re/*`
— `re_tests`, `pat.t`, …), snapshotted immutably like `regex_corpus_bundle/`. This satisfies the
external-corpus doctrine (every parser proven by BOTH the stimuli generator AND an officially-recognized
external corpus).

## Recommended sequencing (agrees with the director's "PCRE2 parity first")

1. **FINISH PCRE2 parity** (current `REGEX-PCRE2-FIDELITY`): `.4.1`✓ → `.4.6`/`.4.4`/`.4.2` (structural) →
   the hard/primitive families (`.4.3`/`.4.5`/`.4.7`/`.4.8`/`.4.9`/`.4.10`/`.4.11`) → final `.4` validator
   deletion → `.5` verification. This is the committed contract; the near-term integration targets
   (RGX/Nexsim/PNR) are PCRE2/POSIX-shaped.
2. **THEN a Perl5 feature arc** (a new tree, e.g. `REGEX-PERL5-FEATURES`): (a) scope `perlre` vs PCRE2 to a
   frozen feature inventory; (b) stand up the `perl` oracle + Perl `re_tests` corpus bundle; (c) add a
   `perl5` profile; (d) encode per-feature, parse-shape-only for the code blocks.
3. **THEN PGEN-RGX-0078** (speed) — after feature-complete, per the director's own gate.

## Honest caveats to flag before investing

- `(?{…})`/`(??{…})` are fundamentally about *executing* Perl — a parser can only accept-and-shape them
  (opaque code atom), never execute. This boundary must be set explicitly and stated in the contract.
- **Downstream demand check first.** The value depends on whether RGX's users actually write Perl-only
  patterns; most EDA-/hardware-domain regex is PCRE2/POSIX-shaped. Worth a quick demand check with RGX
  before committing the arc.
- Perl5 and PCRE2 occasionally **disagree** on the same syntax at the edges — a `perl5` profile must be
  oracle-pinned to `perl`, never assumed to be "PCRE2 minus a few checks."

## Open questions for the director

- Ratify the sequencing (parity → Perl5 arc → 0078)?
- Is there confirmed RGX demand for Perl-only features, or is this exploratory?
- Acceptable to make `(?{…})`/`(??{…})` **parse-shape-only** (accept syntax, opaque atom, no execution)?
