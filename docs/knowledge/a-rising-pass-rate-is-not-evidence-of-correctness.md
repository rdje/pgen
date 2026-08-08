---
id: a-rising-pass-rate-is-not-evidence-of-correctness
title: A grammar change that makes MORE input parse can still be wrong — rank your checks by what they cannot see, and verify the negative axis in the same leaf
answers:
  - "my grammar change made the corpus pass-rate go up — is that enough to land it"
  - "how do I tell an over-accepting grammar fix from a correct one"
  - "which checks can miss an over-acceptance defect in a grammar change"
  - "I added a rule and nothing errored but behaviour is wrong — what should I check"
  - "why did my new rule silently become an extra alternative of an existing rule"
  - "how do I detect an accidental duplicate rule name in a large EBNF grammar"
  - "I fixed a 387-row defect class but only 51 more files pass — did the fix fail"
  - "where do I get negative test cases for a parser without writing fixtures"
  - "my new alternative never fires even though the input parses — why is the branch dead"
  - "a reserved word is being matched as an identifier and my keyword branch is unreachable"
  - "the input passed before and after my change — how do I tell if it passed for the right reason"
  - "did my scope deferral hold, or did I quietly leave part of the production out"
tags: [grammar, over-acceptance, corpus, verification, vhdl, lrm, burn-down, instrument-honesty]
date: 2026-08-08
status: current
evidence: grammars/vhdl.ebnf (`wait_statement` + `wait_sensitivity_clause`/`condition_clause`/`timeout_clause`, and the comment recording the name collision); docs/tasks/CORPUS-GRAD-ALL.md leaf .2.2; rust/src/ast_shape_contract.rs (the declared-annotation crosscheck that caught it); docs/tasks/LANG-CAPABILITY-AUDIT.md .9 (within-file header merging is the documented idiom)
reverify: "test -z \"$(grep -oE '^[a-z_]+ :=' grammars/vhdl.ebnf | sort | uniq -d)\" && ./rust/target/debug/ast_pipeline grammars/vhdl.ebnf --lint-grammar | head -1 | grep -q 'vhdl' && echo NO-DUPLICATE-RULE-NAMES"
---

**The check that fails is worth less than the check that *could* fail.** A VHDL `wait`-statement fix
had four cheap signals and all four were green while the change was wrong:

| signal | verdict | why it could not see the defect |
|---|---|---|
| reproducers REJECT→PASS | all 4 flipped | asks "does the new syntax parse", not "does ONLY the right syntax parse" |
| `--lint-grammar` | 0 errors | asks "is the grammar well-formed"; the defect was well-formed |
| certificate coverage | `fully_certified`, every branch witnessed | the bogus branch was witnessed *because it was real* |
| corpus pass count | went **UP** | over-acceptance raises it too |

Only an instrument comparing **two independent derivations of the same fact** caught it: the
declared-annotation crosscheck, where the pipeline's inventory-emit path and its raw_ast walk
disagreed on a branch index. Before trusting a set of green checks, name what each one is blind to.

⛔ **A rising pass-rate is not evidence of correctness, and it can be priced exactly.** The buggy
grammar scored **4 086** corpus passes; the corrected grammar scores **4 082**. Four files
"improved" by being accepted when the LRM requires rejecting them. On any campaign whose headline
metric is a pass count, that is the failure mode to design against — so **verify the negative axis in
the same leaf as the positive one**, never as a follow-up.

⭐ **The corpus usually adjudicates its own negative axis — read its directory names before writing
fixtures.** The rows that survived the fix were all under
`vests/vhdl-93/billowitch/non_compliant/analyzer_failure/`, and each was a clause sequence in the
wrong LRM order (`wait for 60 ns on i;`). The parser refusing them is a *correctness result*, obtained
free from the suite's own answer key ([[feedback_corpus_expected_from_spec_not_fix]]).

⛔ **A duplicate rule NAME does not error, because within-file header merging is a deliberate
feature.** Repeating a rule header merges the clauses into ALTERNATIVES of one rule — legal by
design, with only cross-file collisions made a hard error. So the frontend cannot distinguish an
accidental collision from the idiom it must support, and in a several-hundred-line grammar the author
has no cheap warning. The consequence is over-acceptance in *both* directions: the new clause joins
the old rule, and the old clause joins the new rule's use site. Two spec violations from one name.
The free defence is to grep before defining, and as a closing check:

```bash
grep -oE '^[a-z_]+ :=' grammars/<g>.ebnf | sort | uniq -d      # must be EMPTY
```

⭐ **Check the arithmetic when a change adds N named things.** The linter prints a rule count. Five
definitions were added and it read **216 → 220**; after the collision was fixed, **221**. A silent
merge is exactly the discrepancy simple arithmetic detects, and it was on screen a full verification
cycle before the expensive gate found the same thing. Count deltas are the earliest and cheapest
detector of "something merged that shouldn't have".

⚠️ **Burn-down does not equal class size — expect that, and prove where the rest went.** Killing a
387-row stuck-point class bought **51** fully-passing files, because a file fails at its FIRST gap and
~330 of them simply moved DEEPER to their next one. Re-cluster after the fix and show the classes that
GREW (`for ID :` 110 → 338, `downto NUM =>` 67 → 101): that turns "only +51" from an apparent
shortfall into a measured explanation. A burn-down leaf reporting only its own class's collapse is
reporting half the result.

⛔ **A DEAD BRANCH passes every verdict-shaped check, because the input still parses — just via the
wrong branch.** If a grammar lets reserved words be matched as identifiers, then a permissive
`expression` alternative placed BEFORE a reserved-word alternative makes the reserved-word branch
unreachable. Measured twice in two consecutive leaves of one campaign:

| input | before | after | what a verdict check saw |
|---|---|---|---|
| `s <= unaffected;` | `{kind: "function_call", name: "unaffected"}` | `{kind: "unaffected"}` | **PASS both times** |

⇒ **when a change is about WHICH BRANCH WINS, the verdict is not evidence — dump the tree**
(`--parse-dump-ast-pretty`). And state the rule positionally, because knowing it is not the same as
applying it: **a reserved-word branch goes first at EVERY level where it competes with a permissive
branch**, not just the innermost one. The leaf that recorded this rule in a comment on its inner rule
still got the ordering wrong on the enclosing rule.

⭐ **Certificate coverage is a second, independent detector of exactly this class — use it
deliberately on any ordering change.** A dead branch is a rule that is never exercised, so `UNKNOWN`
rises and `fully_certified` drops. `total=N witness=N UNKNOWN=0` is therefore doing double duty: the
no-regression number *and* proof that no branch is shadowed. ⚠️ Do not expect the linter to cover it:
`--lint-grammar` reported `ordered_choice_shadowing=0` both before and after the reorder above
(plausibly it models syntactic prefix shadowing, not overlap reached through a chain of rules).

⭐ **A deferred scope is a claim; the RESIDUAL is where it gets audited.** A leaf that declares part
of a production out of scope should check, after the fix, that what survives is the deferred part.
Measured: after implementing the waveform production but deferring `delay_mechanism`, 29 of 362
`after` rows survived and essentially all were `transport …` — the deferral made visible in the data
instead of asserted in prose. It is the cheapest test of whether a scope decision was honest, and it
names the next leaf for free.

See also [[audit-a-reports-key-against-its-own-illustration]] (audit the instrument before ranking
work from it) and [[feedback_done_bar_is_first_tier_only]] (a pass-rate movement is not a status
change).
