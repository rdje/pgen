---
id: derive-the-comparison-key-from-the-artifact-not-from-the-caller
title: A helper that takes the key from its caller cannot fix a caller-supplied-key defect — it just relocates it, and if one spelling is a SUBSTRING of the other it fails silently in the passing direction
answers:
  - "two generated artifacts differ by N bytes and I normalised the path away — why do they still differ"
  - "my normalisation reported success but the files still do not match"
  - "should a comparison helper take the path to normalise as an argument"
  - "how do I write a shared helper so the third caller cannot repeat the first caller's mistake"
  - "a site count came out 0 and nothing failed — what did I do wrong"
  - "why is a relative and an absolute spelling of the same path not interchangeable in a string replace"
  - "how do I stop the same measurement trap firing a fourth time after documenting it three times"
tags: [instruments, measurement, generated-parsers, normalisation, codegen, evidence, refusal]
date: 2026-08-17
status: current
evidence: |
  ENGINE-UNIVERSAL-SERVICES.25 slice 1. Three independent copies of "normalise the embedded `-o`
  path before comparing two generated parsers" existed; all three took the path FROM THE CALLER.
  Measured on `generated/systemverilog_parser.rs`: normalising with the short spelling
  `generated/systemverilog_parser.rs` (a SUBSTRING of the embedded
  `../generated/systemverilog_parser.rs`) leaves **36 346 `"../<TOKEN>"` residues and 0 clean
  sites** — success reported, nothing normalised. `run_guard_ab_structural.sh`'s `row()` used its
  file argument as BOTH the file to read and the spelling to normalise, so a relocated artifact
  reported `path_sites=0` with unnormalised bytes: a silent zero in the passing direction. And its
  published site count `43 615` is `36 346 × 42 ÷ 35` — the right byte delta over the wrong
  character width, i.e. a numerator from one spelling and a denominator from another, inside the
  script whose own comment warns about exactly this.
reverify: "python3 scripts/compare_generated_parsers.py --self-test   # 8/8, four RED-by-design; then `for f in generated/*_parser.rs; do python3 scripts/compare_generated_parsers.py --sites $f; done` must equal `grep -oF <derived spelling> $f | wc -l` for all 11 artifacts"
---

Every parser PGEN generates writes its own `-o` destination into the emitted source, once per
diagnostic site — **36 346** times in the SystemVerilog parser, **63 186** across the eleven shipped
artifacts. So *the size of a generated parser is a function of its own output path*, and comparing
two of them requires normalising that path away first.

This repository learned that lesson three times, wrote it down each time, and grew **three
independent implementations of the fix**. All three shared one design decision, and it is the wrong
one:

```python
def normalise(src, path):           # <- `path` comes from the caller
    return src.replace(path, "<OUT>")
```

## Why taking the key from the caller is not a fix

The defect being repaired is *"the caller compared two artifacts written through different `-o`
spellings."* A helper that asks the caller which spelling to normalise **has not removed that
judgement from the caller** — it has moved it one function deeper, where it is harder to see.

Here the two spellings in play are `../generated/systemverilog_parser.rs` (36 chars, what
`rust/Makefile` passes from `rust/`) and `generated/systemverilog_parser.rs` (33 chars, what an
ad-hoc run from the repo root passes). ⛔ **The short one is a SUBSTRING of the long one**, so the
failure is not an exception or an empty result:

```text
src.replace("generated/systemverilog_parser.rs", "<OUT>")   # over a LONG-spelling artifact
  ->  "../<OUT>"   × 36 346        # residues
  ->  "<OUT>"      × 0             # actually normalised
```

Both sides then "normalise" with their own spelling, still differ by 3 bytes per site, and the
verdict reads *"something other than the path moved"* — a false finding, in the direction that
invents work. That exact false verdict founded a task leaf on two hypotheses that were both wrong.

## The two silent-zero variants

**1 — the file's location is not the file's key.** `row()` used its argument as both *the file to
read* and *the spelling to normalise*, which is true only while the artifact still sits at the path
it was generated to. Copy it once and:

```text
original row(), relocated artifact   path_sites=0        norm_bytes=144237303   # unnormalised
derived  row(), same artifact        path_sites=36346    norm_bytes=142674425   # from either location
```

`path_sites=0` is not an error, prints no warning, and flows into every downstream figure.

**2 — a numerator from one spelling over a denominator from another.** A derived count
`(len(src) - len(norm)) // (len(path) - len(TOKEN))` is only sound when the `path` in the numerator
and the `path` in the denominator are the same string. When they are not, it yields a plausible
integer: `36 346 × 42 ÷ 35 = 43 615`, which was published and carried for sessions.

## What to do instead — read the key out of the artifact

Every generated parser contains **exactly one** distinct string literal ending in `.rs`, and its
occurrence count equals the embedded-site count. Measured **11/11 exact** against an independent
`grep -oF`. So the key is *in the file*, and the caller never needs to know it:

```python
distinct = set(re.findall(r'"([^"\n\\]*\.rs)"', src))
if len(distinct) != 1:
    raise Refusal(...)          # 0 -> not a generated parser; >1 -> ambiguous. REFUSE, never guess.
spelling = distinct.pop()
```

⭐ **Refusing on ambiguity is half the value.** The historical failures all produced a *number* —
`0`, `43 615`, "still differs" — that a reader could act on. An exit-2 refusal cannot be
misread.

⭐ **Derive the rule from the PRODUCER, not from a description of it.** The reason "exactly one
`.rs` literal" is trustworthy is that it was measured over all eleven emitted artifacts, not
inferred from the emitter's documentation — the discipline in
[[deriving-a-control-from-the-producer-is-not-the-same-as-observing-it]].

## ⛔ And prove the helper can be wrong

The self-test carries four RED-by-design arms, and the suite was run against three **mutants of the
live module** to prove it can fail: guessing instead of refusing on ambiguity → 7/8; `normalise()`
made a no-op → 6/8; and `derive_spelling()` returning the short caller-style spelling — *the exact
historical defect* — → 5/8. A control never observed failing is not known to work
([[a-check-whose-inputs-all-pass-has-not-been-tested]]).

## The meta-lesson: three prose warnings did not transfer

TOOLBOX 5.6, the leaf's own routing evidence, and a block comment in the runner all told the reader
to normalise first. A fourth instance of the trap was still live **inside the script carrying the
warning**. ⇒ when the same defect recurs after being written down, the remedy is not a clearer
sentence — it is to make the wrong call *unavailable*, by removing the parameter that carries it.
Same shape as [[a-two-lifecycle-file-needs-a-guard-at-the-boundary]]: the header already said "edit
the body below", and was overwritten anyway.

⚠️ **Honest bound.** Equal normalised sha256 proves the `-o` path was the ONLY difference. It does
**not** prove the two arms came from the same grammar: two artifacts with different rule counts
normalise to different bytes, which is a real difference and is reported as one.
