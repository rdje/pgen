---
id: a-furthest-position-names-a-region-not-a-token
title: "`furthest_position` is the deepest byte CONSUMED, not the offending token's offset — compare it to a source offset without skipping layout and you manufacture false positives at scale"
answers:
  - "why does my instrument say the parse failed before the construct that obviously caused it"
  - "how do I map furthest_position onto the token that actually defeated the parse"
  - "can I compare furthest_position directly against an offset I found with a regex"
  - "my positional audit found a huge misclassification rate on its first run — is it real"
  - "how do I check whether a corpus row is really blocked by the preprocessor"
tags: [parseability-probe, furthest-position, corpus-adjudication, instrument-soundness, systemverilog, toolbox]
date: 2026-08-10
status: current
evidence: docs/tasks/artifacts/sv_corpus_grad/explained_svpp_audit/{audit.tsv,summary.md} (full census, 1459 rows); stimuli/sv/audit_explained_svpp.py:stuck_offset_of() (the reference implementation); stimuli/sv/subs/Surelog/tests/PPComment/dut.sv (the canonical 6-byte gap — furthest_position=13, offending backtick at 19); docs/tasks/SV-CORPUS-GRAD.md leaf .12 ("THE FIRST CUT ... 504 ... WAS AN ARTEFACT") and leaf .12a (the trap named before it is stepped in)
reverify: "./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/subs/Surelog/tests/PPComment/dut.sv --profile sv_2017 2>&1 | tail -1 && python3 -c \"from pathlib import Path; r=Path('stimuli/sv/subs/Surelog/tests/PPComment/dut.sv').read_bytes().decode('latin-1'); print('consumed:', repr(r[:13])); print('choked on offset', min(i for i in range(13,len(r)) if not r[i].isspace()), repr(r[19:27]))\""
---

`parseability_probe` reports `furthest_position=N` on every reject — the deepest byte any branch
reached, even one that later backtracked. It is the single most useful number the toolbox emits, and
it is routinely read as *"the parse died at N, so look at what is at N."*

**That reading is off by the layout.** `N` is where consumption *stopped*; the token that defeated
the parse begins at the next non-whitespace, non-comment byte. Whitespace, newlines and comments sit
in the gap, and in real source that gap is frequently several bytes and occasionally hundreds.

Surelog `tests/PPComment/dut.sv` is the clean shape:

```
module top();\n  \n  `define USE_BOOTMODE_TEST
^0           ^13       ^19
```

The parser consumes `module top();` — 13 bytes — and chokes on the backtick at **19**. It reports
`furthest_position=13`. An instrument comparing that 13 against "the first backtick is at 19"
concludes *died 6 bytes before any preprocessor construct*, which is the exact opposite of the truth:
it died **on** one.

## Why this is worth a record rather than a comment

`SV-CORPUS-GRAD.12` audited whether 1 459 corpus rows labelled "blocked by the preprocessor" really
were. The first instrument made exactly this comparison and reported **504 of 1 459 misclassified** —
a 34.5 % rate, a headline finding, and entirely an artefact. Every row spot-checked in that 504 was
healthy. Resolving the gap first took the finding from **504 to 26**.

The failure mode is nasty in three specific ways:

- **It fails in the alarming direction.** The artefact invents defects, so the number is big,
  plausible, and *interesting* — which is precisely when a result stops getting checked.
- **It is invisible to spot-checking the instrument.** The code is correct; one operand means
  something other than what the comparison assumes.
- **It scales with how tidy the source is.** Well-formatted code with blank lines and indentation
  produces *larger* gaps, so the better the corpus, the worse the artefact.

## The fix, and the reference implementation

Resolve the offending token before judging:

```python
def stuck_offset_of(stripped, furthest):
    """First non-whitespace, non-comment byte at or after `furthest`."""
    for i in range(furthest, len(stripped)):
        if not stripped[i].isspace():
            return i
    return None          # consumed everything and still failed
```

`stripped` must be the source with comments and strings **blanked in place** — replaced by
equal-length runs of spaces, newlines preserved — never collapsed. A collapsing `re.sub(" ", …)` is
fine for "is there one?" and silently wrong for "where is it?", because every offset after the first
comment shifts.

`stimuli/sv/audit_explained_svpp.py` is the working version, proven across 1 459 rows and
byte-deterministic under `--jobs 8` vs `--jobs 4`. Reuse it rather than re-deriving it.

## The general rule

**A failure position names a REGION — `[furthest_position, next real token)` — not a token.** Any
oracle that joins a parser position to a source offset (a regex hit, a line number, a construct
census, a cluster key) must normalize across the layout first, in whichever direction it reads.
When such an oracle returns a large number on its first run, that is the tell to re-derive the
operands, not to write it up.
