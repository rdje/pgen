# Regex feature roadmap — feature-completeness FIRST (PCRE2 parity + code hooks), then SPEED, plus a Perl5-only matrix

- Category: `project`
- Created: `2026-07-09` (session #73); **corrected same session** (the `(?{...})` embedded-code mechanism was
  mischaracterized in the first draft — see the ⚠️ Correction below).
- Status: **partially ratified** — the director confirmed the ordering (features → speed) and elevated SPEED
  as THE critical issue; the Perl5-only matrix scope stays PROPOSED / future.
- Owns/links the tracked trees: `RGX-0078` (speed) · `REGEX-CODE-HOOKS` (`(?{...})`/`(??{...})`) ·
  `REGEX-PCRE2-FIDELITY` (PCRE2 parity, active).
- Related: [[project_regex_pcre2_faithful_by_default]] · [[project_regex_validator_deletion_blocked_load_bearing]] ·
  [[project_rgx_0078_regex_slowness_followup]] · [[reference_pcre2_unsupported_escape_oracle]] · [[project_vision_and_discipline]]

## Context (director prompt, 2026-07-09)

The director flagged, mid-session, three related points (explicitly **not** a request to pivot now):
1. **PGEN-RGX-0078** (a regex *speed* issue from RGX) still exists and is already captured as a tree — but
   regex must be **feature-complete before** tackling 0078.
2. Right now the regex work is **PCRE2 parity** (tree `REGEX-PCRE2-FIDELITY`). What about **Perl5 regex
   features PCRE2 doesn't support at all** — worth closing that gap too?
3. A Perl5 arc **will need Perl5 regex test data / an external corpus**.
4. **But complete PCRE2 parity first — we are not there yet.**

## My take (the analysis)

### ⚠️ Correction (2026-07-09, tool-verified) — embedded code is ALREADY a PGEN extension, NOT a Perl-only gap

My first draft called `(?{ CODE })` the "flagship Perl-only feature, parse-shape only, never execute." **That
was wrong** — corrected after the director pointed at `regex.ebnf` and I read the actual grammar. PGEN
**already** supports `(?{...})` and the language-tagged `(?{lang: ...})` form (`code_block` family,
`regex.ebnf:1668-1671`), with `code_lang = lua | js | javascript | rhai | native | wasm`. It is a **PGEN/RGX
extension** — broader than Perl (which runs *Perl* code): PGEN tags a BACKEND language and **RGX executes the
hook via that backend** (grammar contract: "runtime execution semantics remain out of scope" for PGEN; the
consumer executes). So embedded code is not a gap at all. What's LEFT is the postponed/dynamic sibling
`(??{...})` — added via the **same mechanism** (same `code_lang` + `code_content`). That is now a concrete
tracked leaf: **`REGEX-CODE-HOOKS.2`** ([docs/tasks/REGEX-CODE-HOOKS.md](../tasks/REGEX-CODE-HOOKS.md)).

### The genuine Perl5-only gap (needs an exhaustive support/postpone matrix — future)

**PCRE2 is deliberately a near-superset of Perl5** ("Perl Compatible Regular Expressions"), so the genuine
Perl-only *pattern-syntax* gap is **small**. A future arc must build an **exhaustive matrix** (every `perlre`
construct × PCRE2-support × PGEN-support × support/postpone decision), tool-verified against `perl` itself.
The tractable candidates (NOT the embedded-code family — that is already covered above):

| Perl-only feature | PCRE2 status | Parser tractability |
|---|---|---|
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

## Recommended sequencing (director-ratified emphasis, 2026-07-09: SPEED is the real priority)

The director was emphatic this session: **the feature set is "really, really good" — the critical issue for
the PGEN regex parser is SPEED, not features.** So features are the near-term gate to *unblock* the speed
work, and speed is the headline goal:

1. **FINISH regex feature-completeness** (the gate):
   - **PCRE2 parity** — `REGEX-PCRE2-FIDELITY`: `.4.1`✓ → `.4.6`/`.4.4`/`.4.2` (structural) → the
     hard/primitive families (`.4.3`/`.4.5`/`.4.7`/`.4.8`/`.4.9`/`.4.10`/`.4.11`) → final `.4` validator
     deletion → `.5`. **This is what we continue working on now.**
   - **RGX code-hook family** — `REGEX-CODE-HOOKS.2`: add `(??{...})` via the existing `(?{...})` mechanism.
2. **THEN `RGX-0078` — SPEED (the headline priority).** [docs/tasks/RGX-0078.md](../tasks/RGX-0078.md).
   Close the PCRE2-relative gap: geomean(PGEN-parse / PCRE2-compile) `< 5×` (baseline ~360× no-JIT / ~85×
   JIT). Profile-first; likely needs structural work (arena/`ParseNode`/memo strategy), not micro-tuning.
   This is the thing that determines whether RGX can use PGEN in its compile hot-path.
3. **Perl5-only feature matrix (future / parallel-planning).** The exhaustive `perl`-oracle matrix + a
   `perl5` `@profiles` axis + a Perl `t/re` external corpus. Modest scope (the gap is small); scheduled by
   the director relative to the speed work.

## Honest caveats to flag before investing

- **Code hooks are already handled by design** (corrected): PGEN parses+tags `(?{lang:...})` and RGX
  executes it via the backend (`lua`/`js`/`rhai`/`native`/`wasm`) — "runtime execution out of scope" for
  PGEN is the intended split, not a limitation. `(??{...})` follows the same model (`REGEX-CODE-HOOKS.2`).
- **Downstream demand check first.** The value of the Perl5-only matrix depends on whether RGX's users
  actually write Perl-only patterns; most EDA-/hardware-domain regex is PCRE2/POSIX-shaped. Worth a quick
  demand check with RGX before committing that arc.
- Perl5 and PCRE2 occasionally **disagree** on the same syntax at the edges — a `perl5` profile must be
  oracle-pinned to `perl`, never assumed to be "PCRE2 minus a few checks."

## Open questions for the director

- Sequencing **RATIFIED** (features → speed; speed = the priority). Remaining: schedule the Perl5-only matrix
  relative to the speed work (`RGX-0078`) — before, after, or parallel?
- For `REGEX-CODE-HOOKS.2` (`(??{...})`): confirm the carrier `kind` (proposed `dynamic_code_block`) and
  whether it should be default-accepted or `@profiles`-gated (it is a PGEN extension, not PCRE2).
- Is there confirmed RGX demand for the Perl5-only feature set, or is that matrix exploratory?
