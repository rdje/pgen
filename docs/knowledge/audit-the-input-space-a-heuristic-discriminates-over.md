---
id: audit-the-input-space-a-heuristic-discriminates-over
title: A census of recorded reasoning inherits every blind spot of that reasoning — to find a heuristic's hole, enumerate the input space it discriminates over, not the verdicts it produced
answers:
  - "how do I audit a classification heuristic for holes rather than for self-consistency"
  - "my audit of the recorded verdicts found nothing — does that mean the classifier is right"
  - "two sibling test files got opposite expected verdicts — how do I find out why"
  - "I am classifying rows by the reason field that was recorded for them — what can that miss"
  - "how do I decide whether to match a message prefix or enumerate the exact strings"
  - "I saw two error messages sharing a prefix — is it safe to match the prefix"
  - "is it safe to key an expected-verdict heuristic off an upstream tool's error text"
  - "how do I keep a stage-classification heuristic from going stale when a submodule is re-vendored"
  - "should I fix this with a per-file pin or by widening the rule"
  - "how do I size how much of a defect backlog is really mis-adjudicated expectations"
tags: [instrument-honesty, heuristics, corpus, adjudication, enumeration, staleness, verification]
date: 2026-08-09
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaf .3.24 (the stage-claim census + the verilator lexical-vocabulary leg); docs/tasks/artifacts/sv_corpus_grad/adjudicator_hole_sizing/stage_claim_census.py (LEG 1 vocabulary guard + LEG 2 basis census, both refusing) and stage_claim_census.txt; stimuli/sv/adjudicate_external_corpus.py (VERILATOR_PARSE_STAGE_RE, the enumerated allowlist replacing SYNTAX_ERR_RE in VerilatorIndex); docs/tasks/SV-CORPUS-GRAD.md .3.14b (the compiler-directive whitelist, same shape)
reverify: "python3 docs/tasks/artifacts/sv_corpus_grad/adjudicator_hole_sizing/stage_claim_census.py | grep -q 'every spelling is adjudicated' && grep -q 'VERILATOR_PARSE_STAGE_RE' stimuli/sv/adjudicate_external_corpus.py && echo VOCABULARY-GUARD-LIVE"
---

**A census of recorded reasoning classifies the *derivation*, so a row derived wrongly is classified
confidently — and correctly — as whatever the author of the derivation believed.** It can tell you
what the current rules imply. It cannot tell you the rules are wrong.

Measured: a corpus adjudicator recorded, per row, *why* each expected verdict was chosen. Auditing
those recorded reasons across 284 defect-signal rows returned a clean bill of health — 0 rows
mis-classified, every reason recognized. Auditing the **input space the heuristic discriminates
over** — every error message the upstream tool can actually print, enumerated across 1 427 tracked
golden logs — found the hole immediately:

| file | construct | upstream golden says | expected verdict |
|---|---|---|---|
| `t_parse_eof_str_bad.v` | unterminated string | `syntax error` **and** `EOF in unterminated string` | `must_reject` ✔ |
| `t_parse_eof_qqq_bad.v` | unterminated triple-quoted string | only `EOF in unterminated … string` | `must_accept` ✘ |
| `t_parse_eof_attr_bad.v` | unterminated `(*` | only `EOF in (*` | `must_accept` ✘ |
| `t_fuzz_eof_bad.v` | both at once | `syntax error` **and** `EOF in (*` | `must_reject` ✔ |

One lexical class, two verdicts, separated by nothing but whether the message happened to contain
two particular words — because the whole stage test was `re.compile(r"syntax error")`, and the
tool's **lexer never emits those words**. Every one of those four rows had an impeccable recorded
reason. The census read all four exactly as they claimed to be.

⭐ **The rule of thumb: auditing a heuristic's OUTPUTS tests self-consistency; auditing its INPUT
SPACE tests whether it discriminates on the right thing.** The first is cheap and reassuring. Only
the second finds a hole. Ask "what are all the values this predicate can see?" and enumerate them —
if that set is unbounded, the heuristic is keyed on the wrong field.

⛔ **And once you have enumerated, do not generalize — rule.** Having seen `EOF in (*` and
`EOF in unterminated """ string`, the obvious widening is to match `EOF in `. The enumeration
refutes it: four of the ten spellings are `EOF in define argument list`,
`Unterminated ( in define formal arguments.`, `EOF in unterminated preprocessor expression` and
`Unterminated /* comment inside -f file.` — three preprocessor errors and one about a command-file
list, none a statement about the source text at all. The prefix rule would have moved four rows into
the wrong lane while fixing three. **The tempting rule is stated over the SHAPE of the thing; the
correct rule is stated over an ENUMERATED SET of the things that exist.** Third instance of this in
one tree, after a compiler-directive whitelist and a fused-token sweep.

⚠️ **A heuristic hole and a per-file pin have different lifetimes, and that decides the fix.** A pin
covers one file forever. A heuristic re-opens its hole on every re-vendoring, when a new upstream
message lands that nobody has ruled on. So pinning the three affected files would have been the
smaller change and the wrong one. The guard that matters is not *"are these files right today"* but
*"does the vocabulary still cover everything upstream prints"* — which means the instrument must
**refuse** on an unruled spelling rather than report a clean run, and its refusal path must be
exercised on every invocation rather than assumed
([[a-check-whose-inputs-all-pass-has-not-been-tested]]).

⭐ **Finally: a negative answer is a result.** The census's headline finding was that **0 %** of the
remaining backlog was mis-adjudicated by the metadata channel, refuting the hypothesis the whole
audit was opened on (that files named `_bad` or living under `test/error/` were negatives waiting to
be reclassified — they are overwhelmingly *elaboration* failures, which parse fine). Acting on that
hypothesis would have injected over-acceptance across dozens of files. A measurement that closes off
a tempting shortcut is worth as much as one that opens a lane.
