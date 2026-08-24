---
id: an-answer-key-that-records-why-it-decided-can-audit-itself
title: An oracle's own record of WHY it decided beats any inference about it — above all one drawn from the system it judges
answers:
  - "how do I audit a corpus answer key without trusting it"
  - "which upstream message actually keyed this expected verdict"
  - "can I use the parser's furthest_position to interpret its own answer key"
  - "how do I attribute a corpus row to its deciding evidence"
  - "my contradiction census reports candidates that are all artifacts — why"
  - "how do I tell a clause-cited expectation from one resting on tool testimony"
  - "why did my defect census over-report by 100 percent"
  - "how do I state the power bound of a clean audit result"
  - "my message-class census missed `syntax error` — how"
tags: [oracles, answer-key, corpus, adjudication, instruments, circularity, false-positives, power-bound, corpus-key-audit]
date: 2026-08-24
status: current
evidence: "CORPUS-KEY-AUDIT.1(a)-(d) (PGEN-CORPUS-KEY-AUDIT-0002). The census attributed every message in an iverilog golden to the row's single expected verdict and reported 2 contradictory classes; both were artifacts, a 100% false-positive rate. The leaf's own plan — narrow to the message the parser's furthest_position lands on — fails twice: br_gh1087b is pinned at line 3 while the only message in its golden is at line 6 (positional narrowing finds nothing), and it asks the system under test to interpret its own answer key. The manifests' `basis` column already records the deciding evidence: 9 rows cite an IEEE clause, 11 quote the message verbatim as PARSE-stage refusal ('...'), 479 are must_accept claims about the whole golden. Reading it takes contradictions 2 -> 0. Separately, the census's message regex required a literal `error:`/`sorry:` tag while iverilog emits `./ivltests/br_gh79.v:6: syntax error` untagged, so the most parse-relevant class was absent from the measured vocabulary (168 -> 176 classes, +8, 0 dropped)."
reverify: "python3 docs/tasks/artifacts/corpus_key_audit/key_contradiction_census.py   # rows=499 messages=176 contradictory_classes=0 (naive=2) key_integrity_findings=0 -- the naive number is the before. Then the controls: python3 docs/tasks/artifacts/corpus_key_audit/key_contradiction_census.py --self-test   # arms=5 failed=0, arm 2 proves the CLAUSE exclusion is what turns it green. Then the two facts by hand: grep -h 'br_gh1087b\\.v\\|pr1704726a\\.v' stimuli/sv/characterization/adjudication_manifest_v2005.tsv   # both bases cite IEEE 1364-2005; cat stimuli/sv/subs/iverilog/ivtest/gold/br_gh79.gold   # untagged `syntax error`"
---

An answer key is an instrument. When you set out to audit one, the first instinct is to build a
*second* oracle that infers what the key meant. Check first whether the key already **says** what it
meant — most well-kept ones do, in the field nobody reads mechanically.

## The measured case

A contradiction census over an external SystemVerilog corpus asks: *do two rows keyed from the same
upstream evidence carry opposite expected verdicts?* The first cut attributed **every** message in a
file's golden log to that file's single expected verdict, and reported two contradictory message
classes. Both were artifacts — a file pinned for one reason whose golden also reports several
unrelated errors. **A 100 % false-positive rate**, 2 of 2.

The obvious refinement — narrow each row to the message the parser's own `furthest_position` lands
on — was written into the task leaf as the plan. Measuring it before building it killed it twice:

```text
br_gh1087b.v   pinned at line 3   (`wire bool [7:0] b;` gives a NET a data type)
br_gh1087b.gold  its only message is at line 6  (a multiple-driver ELABORATION error)
```

1. **It does not reach the candidates.** No golden message sits at line 3, so positional narrowing
   finds nothing to attribute and falls back to the whole golden. The false positive survives.
2. **It is circular.** The census exists to audit the answer key; the answer key is what tells the
   parser whether it is right. Using one to interpret the other is not evidence about either.

## What the key already said

The manifest's `basis` column had recorded the deciding evidence all along, in three mechanical
shapes:

| provenance | the basis text | what the row should attribute |
|---|---|---|
| clause-cited | `IEEE 1364-2005 A.2.1.3 net_declaration …` | **nothing** — the clause decided it |
| quoted decider | `PARSE-stage refusal ('Missing task/function port direction.')` | only the messages containing that fragment |
| whole-golden | a `must_accept` key | every message — the claim *is* that each is post-parse |

One row's basis states the ranking outright: *"spec outranks the ivtest driver key"*. Reading the
column takes the census from 2 contradictions to 0, needs no parse, and costs 0.24 s over the whole
population. It also buys a second check for free — **a basis quoting evidence its own golden does
not contain is a key contradicting itself**.

## The trap under the trap

The same session's throwaway sanity probe — *does each quoted fragment actually appear in its
golden?* — came back **hit 5 / miss 6**. Six rows quoted `'syntax error'` against goldens that
visibly contain it:

```text
./ivltests/br_gh79.v:6: syntax error
./ivltests/br_gh79.v:6: error: Malformed statement
```

The key was fine. The census's own message regex required a literal `error:`/`sorry:` tag, and the
upstream emits its **bare parse refusal untagged**. So the one class that most directly answers a
parse-stage question was absent from the vocabulary the founding number was measured over. Corrected:
176 classes instead of 168, **+8, none dropped**, and the integrity check stopped reporting instrument
blindness as key defects. This is
[[a-heading-census-is-only-as-good-as-the-heading-grammar]] in a different corpus: a census measures
the intersection of the corpus and your pattern, and reports the difference as someone else's defect.

⇒ **A "fishy" side-result is the finding.** Six misses could have been written off as *"the key
quotes loosely"*, and that sentence would have been the end of it.

## Publish the power bound with the zero

After attribution the reject side of the comparison is **4 message classes** wide against **124** on
the accept side. `0 contradictions` over a 4-class reject side is a far weaker statement than `0`
alone implies, and a reader cannot reconstruct the difference. The artifact prints the ratio and the
per-class table beside the headline, so the strongest check inside it stays legible: *does any
`must_accept` row's golden carry the upstream's own bare `syntax error`?* — **0 of 479**, a question
that was **unaskable** rather than answered one commit earlier.

## The habit

1. **Before inferring why an oracle decided, grep its own justification field.** A basis, a comment,
   a commit message — a key worth auditing usually records itself.
2. **Never let the system under test adjudicate the key that judges it.** If your attribution needs
   the parser, the result is about the parser.
3. **Treat a plan written in a task leaf as a hypothesis.** Toolbox-first is usually invoked against
   guessing a *root cause*; it applies identically to a proposed *fix*. Ten minutes of measurement
   retired this one.
4. **Publish the reach beside the result.** A clean number whose power is unstated reads as a much
   stronger clean bill than it is — which is exactly the flattering-direction failure an
   answer-key audit exists to catch.
