---
id: when-your-delimiter-is-also-legal-content-anchor-the-parse
title: A census that splits on a delimiter goes blind on exactly the rows whose content contains that delimiter — and those are usually the rows the census was built for
answers:
  - "I am writing a regex to extract rule bodies from an EBNF file — what will it miss"
  - "my extractor truncated a value and I did not notice"
  - "how do I count occurrences of an operator in a spec document without false negatives"
  - "is grep -c a safe way to ask whether a standard defines this token"
  - "my census number changed when I fixed the parser that produced it — which number was right"
  - "why did my instrument report a token as present in a document that does not define it"
  - "how do I test the refusal path of a check I just wrote"
  - "my check exited 1 with a traceback instead of its refusal code — does that matter"
tags: [census, instruments, parsing, regex, false-negatives, controls, refusal, claim-verification, sv-corpus-grad]
date: 2026-08-25
status: current
evidence: "SV-CORPUS-GRAD.13e.10 (a)+(b), PGEN-SV-CORPUS-GRAD-0298, building stimuli/sv/v2005_punctuation_faithfulness_census.py. THREE independent instances inside one instrument, each caught only by checking a result that looked fine. (1) DELIMITER-IN-CONTENT: a PGEN terminal is `name := trivia <literal> -> {...}`, so the extractor split each body on the return-annotation arrow `->`. Terminals whose LITERAL CONTAINS that arrow were truncated — `iff_arrow := trivia \"<->\"` read as `\"<` and `implies := trivia \"->\"` read as `\"`. Both are operators, the entire subject of the census, so it was silent exactly where it mattered; the population read 66 instead of 68. (2) SUBSTRING-COUNT ORACLE: asking `annex_a.count('--')` reported the decrement operator PRESENT twice in IEEE 1364-2005, which has no decrement operator. Both hits are the YAML front-matter delimiter `---` on lines 1 and 9 of the tracked file. Substring counting can only err toward FALSE NEGATIVES, so its zeros are sound and its non-zeros are worthless; a set-vs-set comparison over a tokenized BNF recovered `--` and `'`, moving the candidate set from 17 to 19. (3) REFUSAL PATH: the missing-oracle red control died with a ValueError traceback and exit 1 instead of the published refusal code 2, because the refusal MESSAGE formatted its path with Path.relative_to, which raises on a path outside the repo root. The only call site was inside a refusal — the least-exercised branch in the file."
reverify: "python3 stimuli/sv/v2005_punctuation_faithfulness_census.py   # must print 68 reachable non-keyword string-literal terminals and 19 candidates including '--', '<->' and \"'\" — the three members every earlier cut of the instrument dropped. Then drive its four refusal arms and require rc=2, never a traceback."
---

# When your delimiter is also legal content, anchor the parse — don't split

**Question it answers:** I am writing a census over structured text — rule bodies, spec productions,
task-leaf headings. What will my extractor silently miss?

**Answer:** the rows whose *content* contains your *delimiter*. And because delimiters are usually
chosen from the same small alphabet the interesting content uses, those rows are disproportionately
the ones the census exists to find.

## Three instances, one instrument, one afternoon

### 1. The delimiter was inside the literal

A PGEN terminal reads `name := trivia <literal> -> {…}`. Splitting the body on `->` is the obvious
move, and it truncates every terminal whose literal *contains* that arrow:

```
iff_arrow := trivia "<->"     →  extracted as  "<
implies   := trivia "->"      →  extracted as  "
```

Both are **operators** — the entire subject of a punctuation census. The population read **66**
instead of **68**, and the two missing members were the most interesting ones on the list.

⇒ Parse the literal **anchored** (`^"((?:[^"\\]|\\.)*)"`), then require what remains to be empty or
an annotation. Never `split()` on a token the grammar also permits inside a value.

### 2. The oracle was a substring count

*"Does IEEE 1364-2005 define this operator?"* answered as `annex_a.count("--")` returns **2**. The
standard has no decrement operator. Both hits are the **YAML front-matter delimiter `---`** on lines
1 and 9 of the tracked markdown.

⭐ Note the direction: a substring count can report a token PRESENT when it is absent, but never
absent when it is present. So its **zeros are sound and its non-zeros are worthless** — which is
precisely backwards from how such a census gets read. Tokenize the document into its actual
alphabet and compare **set to set**. That recovered `--` and `'`, moving 17 candidates to 19.

### 3. The refusal path itself raised

Driving the red controls, the missing-oracle arm died with a `ValueError` **traceback and exit 1**
instead of the published refusal code **2** — because the refusal *message* formatted its path with
`Path.relative_to`, which raises on a path outside the repo root. The only call site in the file was
inside that message: the least-exercised branch in the program.

⇒ **A check that tracebacks is indistinguishable from a check that is broken.** The formatting of a
refusal must not be able to fail, and you only find that out by *running* the refusal.

## The common shape

All three are the same defect wearing different clothes: **the instrument's own grammar decides what
its census can see**, and every one of them failed in the direction that makes the tree look
cleaner. None was caught by reading the code. Each was caught by a result that was slightly too
tidy — a population that felt short, a count of 2 in a document that should have had 0, a control
that "failed" with the wrong number.

## The checks that catch it

```bash
# does the extractor's count agree with a second, independent count?
grep -c '@sample' grammars/systemverilog.ebnf     # minus occurrences in prose = the AST count

# do the members you most expect actually appear?
... | grep -E "iff_arrow|implies|'"               # spot-check the hardest rows BY NAME

# does every refusal arm exit with the refusal code — not a traceback?
for arm in missing truncated unterminated empty; do run_control "$arm"; done   # require rc=2
```

## See also

- [[a-heading-census-is-only-as-good-as-the-heading-grammar]] — the same failure one level up: a
  heading census whose *pattern* was wrong reported a confident, wrong population, and the
  hand-check reused the same pattern so it could not disagree.
- [[an-x-is-checked-by-nothing-claim-is-a-census-claim]] — the claim-side twin: reading a few
  implementations is sampling, not a census.
