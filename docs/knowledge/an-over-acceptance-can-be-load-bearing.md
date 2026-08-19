---
id: an-over-acceptance-can-be-load-bearing
title: An over-acceptance can be LOAD-BEARING — closing one is an excavation, not a local edit, and the constructs it was propping up are invisible to every check that only asks "did it parse"
answers:
  - "why did fixing an over-acceptance suddenly break things that always worked"
  - "how do I find defects that are hidden behind a different defect of the opposite sign"
  - "is a green corpus pass-rate enough to say a construct is supported"
  - "should my parser test suite pin WHICH production parsed the input, not just the verdict"
  - "how do I safely tighten a grammar rule that many other rules reference"
  - "my strictness fix regresses the corpus — is the fix wrong"
  - "what does a keyword leaking into an identifier position actually cost me"
tags: [parsers, grammars, over-acceptance, oracles, test-design, evidence, sweeps]
date: 2026-08-19
status: current
evidence: SV-CORPUS-GRAD.13c.2k/.13c.2q/.13c.2r (PGEN-SV-CORPUS-GRAD-0233). PGEN's SystemVerilog grammar guarded reserved keywords only in DECLARED names and the FINAL component of a hierarchical path; 46 other references spelled the raw `identifier` rule. Moving the exclusion into `identifier` itself — correct per IEEE 1800-2017 §5.6.2 + A.9.3 — made TEN gate/switch primitives stop parsing. None of them was collateral damage: `bufif0`/`bufif1`/`notif0`/`notif1`/`tranif0`/`tranif1`/`rtranif0`/`rtranif1` had been extracted with their trailing digits DROPPED (`/bufif\b/` cannot match `bufif0` — `\b` demands a word boundary and `0` is a word character), and `buf`/`not` sat behind a greedy `( comma output_terminal )* comma input_terminal` whose star ate its own mandatory tail whenever the final terminal could start a net lvalue — every ordinary gate instantiation, though not literally every input (a literal, a parenthesised expression or a system-function call left the star nothing to take, and the tracked rule-coverage artifact recorded those 4 corpus files all along). All ten had been reaching `udp_instantiation` with a reserved keyword as the UDP type name. The corpus pass count, the two-sided reproducer ratchet and the syntax-closure gates were green across all of it, for years, because the VERDICT never moved — only the arm did.
reverify: "python3 stimuli/sv/run_adjudication_repros.py   # the four `fixed_gate_*` rows carry arm claims of the form `gate_instantiation>enable>bufif0,!udp_instantiation`; the `!` clause is the whole guard, and a verdict-only row cannot express it"
---

**A parser can get the right ANSWER through the wrong PRODUCTION, and when it does, the wrong
production is holding up real work.** Remove the flaw and the constructs riding on it fall over —
not because the fix is wrong, but because the fix is the first thing that ever asked those
constructs to be parsed properly.

The shape is specific and worth recognising. An over-acceptance widens some rule beyond its
specification. Somewhere else, a *different* defect makes the production that should carry a
construct unreachable. As long as the widened rule can absorb that construct, the two defects
cancel at the level of the verdict: input goes in, `accept` comes out, and no instrument keyed on
the verdict can tell the difference. They cancel *only* at that level. The AST is wrong, anything
downstream that reads it is wrong, and the moment either defect is fixed alone the cancellation
stops.

⛔ **So the risk runs in the opposite direction from the one you brace for.** Tightening a rule
feels like a narrowing, and the thing to watch for feels like "did I reject something legal". The
real hazard is that you are about to *discover* a backlog: every construct that was quietly
depending on the flaw now needs its own production to actually work. In the measured case the
backlog was ten gate primitives and two independent root causes, neither of which had anything to
do with keywords or identifiers.

⭐⭐ **The instrument lesson: pin the ARM, not just the verdict.** A reproducer that records
`accept` has recorded almost nothing about a grammar with ordered choice and a permissive catch-all.
A reproducer that records *which alternative won* — as a chain of AST node kinds, with the ability
to assert that a particular alternative did **not** win — is the only kind that can see this class.
The negative half is what does the work: `!udp_instantiation` beside `gate_instantiation>enable>bufif0`
turns "it parsed" into "it parsed the way the standard says it must".

⭐ **And budget for the excavation.** A sweep that closes an over-acceptance should expect to be
interrupted by defects it did not open, price that in before it starts, and adjudicate every
resulting movement against the specification one at a time — because the movements will not all be
regressions and they will not all be improvements. In the measured case, ten corpus files moved:
six were newly CORRECT rejections (three of those merely reflecting which dialect profile was being
applied), and one was a genuine rejects-valid gap that had been hiding under the same flaw. The
right response to that one was to hold the fix and open a leaf for the gap — landing a narrowing
beside a known rejects-valid regression trades one defect class for another.

⚠️ **The count itself needs an instrument too.** The sweep that found all this opened with "43
sites", a number produced by a `grep -c` nobody re-ran; `grep -c` counts LINES, and five lines
carried two or three references each. The real population was 47 references across 40 rules, and
the classification — not the count — is what decided the fix, because exactly one of those sites
sat under a negation, where the same tightening makes the grammar strictly more permissive
([[a-hand-count-of-a-class-is-a-claim]]).
