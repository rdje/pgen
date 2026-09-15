---
id: a-spelling-test-standing-in-for-a-property-test
title: A text census that matches how a call is SPELLED, not where its argument RESOLVES, is wrong in both directions — and it is wrong quietly
answers:
  - "my grep-based census reports far more breaches than are real — what did the classifier get wrong"
  - "how do I tell a repo-rooted mktemp template from a bare mktemp"
  - "why did my regex read a quoted argument as the end of the call"
  - "is it safe to decide a path's location from the NAME of the variable holding it"
  - "how do I stop a census classifier from silently drifting back to a spelling list"
  - "what do I pin in a census self-test so a wrong cut cannot come back"
  - "my census got cleaner after I changed the rule — is that progress or is it looking less"
tags: [instruments, census, classifiers, evidence, blind-spots, data-locality, doctrines]
date: 2026-09-15
status: current
evidence: DATA-LOCALITY.1 (2026-09-15, PGEN-DATA-LOCALITY-0001). Three successive classifiers for one question — does this call create scratch off the repository volume — were each wrong in a different way. Cut 1 matched the CALL (`\bmktemp\b(?![^\n]*-p)`) and counted every repo-rooted TEMPLATE as a breach: 52 reported, 4 real. Cut 2 required a terminator after the flags and matched the template's opening QUOTE, so `mktemp "$ROOT/…"` still read as bare. Cut 3 decided by the NAME of the variable a template used and reported three CORRECT derived-path sites. The fourth resolves a variable against its own assignments in the same file and asks where the path lands. Fourth recorded occurrence of the class in this repository: the look-around check (does the pattern COMPILE), the separator check (can it carry non-whitespace text), the comment-introducer guard (is this terminal AN introducer vs does it BEGIN with one), and this one.
reverify: "python3 scripts/data_locality_census.py --self-test   # PASS — 10 classifier arms incl. the three wrong cuts, + a REACH arm that fails if the frozen population reads zero"
---

**The question a census asks is a PROPERTY. What a regex matches is a SPELLING.** Whenever those two
come apart the census is wrong in both directions at once — it flags correct code and it misses real
breaches — and neither shows up as an error, because a census's output is a number and a number
always looks like an answer.

`mktemp` is a clean demonstration: one call, four spellings, and only the argument decides.

```text
mktemp                                   -> OFF-volume  (no template, no -p: $TMPDIR)
mktemp -d                                -> OFF-volume
mktemp "${TMPDIR:-/tmp}/x.XXXXXX"        -> OFF-volume  (an explicit off-volume template)
mktemp "$ROOT/rust/target/x.XXXXXX"      -> ON-volume   (a repo-rooted template)
mktemp -d -p "$ROOT/rust/target"         -> ON-volume
scratch="$ROOT/rust/target/x"            -> ON-volume   (and only the ASSIGNMENT says so)
mktemp "$scratch/y.XXXXXX"
```

Three cuts failed before one worked, each at a different altitude:

| cut | rule | what it got wrong |
|---|---|---|
| 1 | match the CALL, exclude `-p` | every repo-rooted TEMPLATE became a breach — **52 reported, 4 real** |
| 2 | require a terminator after the flags | matched the template's opening **quote**, so `mktemp "$ROOT/…"` still read as bare |
| 3 | allowlist the variable NAMES that mean "repo" | three CORRECT derived-path sites reported as breaches |
| 4 | **RESOLVE** the variable against its own assignments, then ask where the path lands | — |

⭐ **Cut 3 is the instructive one, because it looks like the fix.** It has a list, the list is
explicit, and it is still a spelling test — it just spells variables instead of calls. The tell is
that the rule can be broken by *renaming something*, which no property can be.

## How to keep it honest

1. **Write the property as a sentence first** — *"where does the path this returns RESOLVE"* — and
   check the implementation answers that sentence rather than a proxy for it.
2. **Pin every wrong cut as a self-test arm, by name.** All three above are arms of
   `scripts/data_locality_census.py --self-test`, so the classifier cannot regress into any of them.
3. **Add a REACH arm.** A census that gets cleaner because it stopped looking is the failure mode
   that looks most like success, so one arm fails if a population that must be non-empty reads zero.
4. **Count both populations.** When part of the tree is deliberately exempt, report its size on every
   run — the exemption is then a number rather than a silence.

⛔ This is the fourth recorded occurrence in this repository. The others: *"does the pattern
COMPILE"* standing in for *"can this look-around ever match"*; *"can this separator carry
non-whitespace text"*; and *"is this terminal AN introducer"* standing in for *"does it BEGIN with
one"* (`GRAMMAR-WELLFORMED.H.16.2b`, [[the-accept-set-can-only-narrow-when-the-guard-is-a-superset]]
if present). The class is not a regex problem — it is the habit of testing what code *looks like*
instead of what it *does*.
