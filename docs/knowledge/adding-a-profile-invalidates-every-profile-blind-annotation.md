---
id: adding-a-profile-invalidates-every-profile-blind-annotation
title: A profile NARROWS the accepted language, so the moment you add one, every literal already written into an annotation becomes a candidate defect — and nothing re-checks them
answers:
  - "I am adding a new @profiles dialect to an existing grammar — what silently breaks"
  - "a rule PARSES but its certificate says UNKNOWN and the parser did not change"
  - "the witness generator produced a sample and the sample does not parse"
  - "why is my @sample literal wrong now when it was right when I wrote it"
  - "a gated optional made an enclosing rule stop being witnessed — is that a generator bug"
  - "how do I census every @sample in a grammar against the profiles it is live in"
  - "four different reproducers all fail at the same byte offset — what does that mean"
  - "how do I prove a grammar change is accept-set-neutral instead of arguing it"
tags: [profiles, annotations, sample, stimuli-generation, certificate-coverage, latent-defect, census, sv-corpus-grad, engine-universal]
date: 2026-08-25
status: current
evidence: "SV-CORPUS-GRAD.13e.9, PGEN-SV-CORPUS-GRAD-0296. Four @sample annotations in grammars/systemverilog.ebnf were written in IEEE 1800 — module_ansi_header:3824 and program_ansi_header:4976 spelled their port `input logic a`, function_body_declaration alternatives 0 and 1 (:2775/:2777) spelled their return type `int`. Neither rule carries @profiles, so both are live under verilog_2005, where those strings parse as nothing. Measured with PGEN_CERT_COVERAGE_DEBUG_PROBES=1 at seed 0: all seven targeted rules parsed=false with ZERO parsed=true witnessed_target=false, and over the whole log 5608 of the 5776 parsed=false probe samples carried `input logic` with 0 of those 5608 ever parsing. All four distinct forced samples rejected at the IDENTICAL furthest_position=20, the byte after `logic` in the shared carrier. Dated by git: the sample was written 2026-04-22 (cd8770cd), four months of commits back (DERIVED by git rev-list --count cd8770cd..HEAD, never stored -- it read 1912 when measured and 1913 one commit later), when only the SV profiles existed — it was CORRECT. The verilog_2005 profile was registered 2026-07-02 (3d398364), which made it wrong retroactively; nothing re-checked it for 54 days, and two of the ten damaged rules were already UNKNOWN before the commit that surfaced the rest. Repair: four tokens (logic->wire, int->integer), each measured legal under all three profiles in both directions. verilog_2005 cert 1147/356/769/22 -> 1147/356/774/17, union UNKNOWN 11 -> 2, identical at seeds 0/7/42; generated parser BYTE-IDENTICAL (sha256 8bc4746aeb0363e92b13a59d6a3e473f6992a73ba90ac0d75435303b6ecb23cc for the fixed grammar, its predecessor, and the shipped artifact)."
reverify: "ast_pipeline <grammar> --dump-gen-ast ga.json && ast_pipeline <grammar> --dump-rule-profiles rp.json   # then for each (sample, profile in that rule's satisfiable_under): ast_pipeline <grammar> --interpret-parse s.txt --interpret-entry-rule <R> --grammar-profile <P>. On SystemVerilog: 52 samples, 115 checks, 2m41s, and every reject must be a branch that is genuinely dead under that profile."
---

# Adding a profile retroactively invalidates every profile-blind annotation already in the grammar

**Question it answers:** I am adding a second (or third) `@profiles` dialect to a grammar that
already works. What breaks silently?

**Answer:** every literal already baked into an annotation. `@sample` holds **one string**;
`@profiles` narrows **which language a rule is live in**; the two features do not talk to each
other. A profile is a *narrowing*, so every literal written against the wider language becomes a
candidate defect the instant the narrower profile exists — and nothing re-checks them.

## Measured

`module_ansi_header` carried `@sample: "module m(input logic a);"` and no `@profiles` at all, so it
is live under every profile. `logic` is IEEE-1800-only. Under `verilog_2005` that string is not a
sample of anything, and every witness probe that short-circuits through it emits a file the profile
must reject — before the target construct is ever reached.

| | count |
|---|---|
| `[plannable-probe]` lines, seed 0 | 10 110 |
| `parsed=false` | **5 776** |
| … whose sample contains `input logic` | **5 608** |
| samples containing `input logic` that EVER parsed | **0 of 5 608** |

Four such annotations cost **ten** rules their certificate. The parser was never involved.

## The timeline is the whole lesson

| when | what |
|---|---|
| **2026-04-22** (`cd8770cd`) | the sample is written. Only the SV profiles exist. **It is correct.** |
| **2026-07-02** (`3d398364`) | the `verilog_2005` profile is registered. **The sample becomes wrong retroactively**, for a profile that did not exist when anyone wrote it. |
| **2026-08-24** (`ac2aa012`) | an unrelated gating change removes the route that was masking it; seven rules fall through to the profile-blind carrier and go `UNKNOWN`. |

Four months of commits (`git rev-list --count cd8770cd..HEAD` — DERIVED, never stored: it read
1 912 when measured and 1 913 one commit later). 54 days in which the defect was already costing published certificate coverage — two
of the ten rules were `UNKNOWN` before the commit that surfaced the rest. ⇒ **the commit that
surfaces a latent defect is not the commit that caused it**, and `git show <commit>^:<file>` +
`git log -S` settle which is which in seconds.

## The diagnostic tell

Four *structurally different* forced samples — a function declaration, an event control, two
lifetimes — all rejected at `furthest_position=20`. A defect in four different constructs does not
land on one byte offset; a defect in the string they **share** does.

⭐ **When independent reproducers agree to the byte, stop looking at the reproducers and look at
what they have in common.**

## The census, and the trap inside it

```bash
ast_pipeline <grammar> --dump-gen-ast ga.json        # samples, from the FRONTEND
ast_pipeline <grammar> --dump-rule-profiles rp.json  # which profiles each rule is live in
ast_pipeline <grammar> --interpret-parse s.txt \
  --interpret-entry-rule <R> --grammar-profile <P>   # ~1.3 s per (sample, profile)
```

- ⛔ Read the samples out of the **gen-AST**, never by grepping the `.ebnf`. An annotation binds to
  the *next* rule, so a regex census miscounts exactly where it matters.
  (Cross-check the count anyway: `grep -c '@sample'` minus the occurrences in prose comments should
  equal the gen-AST count. On SystemVerilog: 54 − 2 = **52**, exact.)
- ⚠️ `--dump-rule-profiles`' `satisfiable_under` is **rule**-level, while most `@sample`s are
  **branch**-level. A branch can be dead under a profile while its rule is live, so the raw join
  **over-reports by construction** — 7 of 11 hits here were honest samples on dead branches.
  Adjudicate every hit by parse: substitute the other profile's spelling and require the repair to
  **accept**. Still rejects ⇒ the branch is dead there and the sample was fine.
- ⭐ The TOOLBOX 5.11 warning that `satisfiable_under` is not reachability does **not** bite here,
  and it is worth knowing why: the question asked is *can rule R derive this string under P*, which
  is exactly the bottom-up question `satisfiable_under` answers.

## Two controls that were nearly wrong

1. **A repair control must repair the SAME branch.** The first adjudicator called `case_statement`
   alternative 1 a defect because the "repair" it substituted was **alternative 0's sample**. That
   arm went green by testing a different alternative. ⇒ substituting a sibling's text tests the
   sibling.
2. **Compare runs made with the same instrument settings.** A `WARNING … NO reach path` list looked
   like it had shrunk between two runs. It had not: the list caps at 10 without
   `PGEN_CERT_COVERAGE_DUMP_ALL=1`, which only one of the two runs set. The header count — 11 in
   both — was right there and disagreed.

## Prove accept-set neutrality; do not argue it

`@sample` steers stimuli generation and nothing else, so the accept set *cannot* move. That is still
worth **hashing** rather than asserting:

```bash
ast_pipeline <grammar>     --generate-parser -o after.rs      # 26 s
git show HEAD:<grammar> > before.ebnf && ast_pipeline before.ebnf --generate-parser -o before.rs
cmp before.rs after.rs && cmp after.rs generated/<family>_parser.rs
```

Three identical digests convert an argument into a fact — and they are what let the change be
recorded as a non-release.

## See also

- [[an-x-is-checked-by-nothing-claim-is-a-census-claim]] — the *"nothing checks `@sample` against
  profiles"* half of this finding is itself a census claim, and was discharged with a named command
  rather than asserted.
- `docs/book/src/stimuli-and-quality.md` — the reader-facing version, under
  *"A `@sample` Is One String, But A Rule Can Be Live In Several Profiles"*.
