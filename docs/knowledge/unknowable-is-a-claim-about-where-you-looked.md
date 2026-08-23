---
id: unknowable-is-a-claim-about-where-you-looked
title: A recorded "cannot be determined" is a fact about the artifact you inspected, not about the world — and a well-written one is the hardest kind to overturn
answers:
  - "a tool says the information is not available — should I believe it"
  - "how do I check whether a documented limitation is really a limitation"
  - "the config file does not specify the default — where is the default"
  - "how do I tell a genuine unknown from a wrong place to look"
  - "why did an honest, specific, falsifiable note stay wrong for months"
  - "what is a positive control when auditing a classification"
  - "is finding a key the same as having a verdict"
tags: [evidence, instruments, classification, corpus, defaults, audit, tooling]
date: 2026-08-23
status: current
evidence: "SV-CORPUS-GRAD.13e (PGEN-SV-CORPUS-GRAD-0281). 147 vendored corpus rows were classified `no_sv_key` — contributing nothing to a signoff claim — with the basis `vvp_tests descriptor(s) without an explicit generation flag - dialect unresolved (the upstream default generation is not encoded in the descriptor)`. True of the descriptor; the default generation is a property of the compiler, and that compiler is vendored in the same corpus: stimuli/sv/subs/iverilog/compiler.h declares GN_DEFAULT = 4 in an enum whose GN_VER2005 = 4, and main.cc:108 initialises generation_flag = GN_DEFAULT. The positive control that made it publishable rather than plausible: the primary key regress-sv.list encodes a generation EXPLICITLY in 907 of its 922 entries, so explicit labelling is the norm precisely because the default is something else."
reverify: "grep -A12 'enum generation_t' stimuli/sv/subs/iverilog/compiler.h | grep -E 'GN_VER2005|GN_DEFAULT'; grep -c 'g2005-sv\\|g2009\\|g2012\\|g2017\\|g2023' stimuli/sv/subs/iverilog/ivtest/regress-sv.list   # GN_DEFAULT aliases GN_VER2005; 907 of 922 primary-key entries label the dialect explicitly"
---

**When a tool, a manifest or a past self records *"this cannot be determined"*, that is a finding
about the artifact they inspected.** Whether it is also a finding about the world depends entirely on
whether the artifact is where the answer lives — and nothing in the record tells you that, because
the person writing it had already decided it was.

The founding case reads like this:

> `vvp_tests descriptor(s) without an explicit generation flag — dialect unresolved
> (the upstream default generation is not encoded in the descriptor)`

Every clause is correct. The descriptor genuinely carries no dialect flag. And the sentence is
answering *"which language dialect does this test compile under?"* — which is a property of the
**compiler**, not of the descriptor. The compiler was vendored in the same corpus, two directories
away, and states its answer in eleven characters:

```c
enum generation_t { … GN_VER2005 = 4, … GN_DEFAULT = 4 };   /* compiler.h */
generation_t generation_flag = GN_DEFAULT;                  /* main.cc     */
```

## Why the *good* version of this note is the durable one

A vague "unclear, skipping" invites someone to look again. A specific, falsifiable, honestly-bounded
note does the opposite: a reader checks the descriptor, confirms there is indeed no flag, agrees, and
moves on. **The verification it invites is of the part it already got right.** Precision about the
wrong artifact is more resistant to review than sloppiness about the right one.

⇒ The question to ask a recorded unknown is never *"is this statement true?"*. It is **"is this the
artifact that would know?"**

## The positive control is what turns a plausible reading into a verdict

Reading two lines of C++ and concluding *"the default is Verilog-2005"* is a plausible reading, and
plausible readings of someone else's build system are wrong often enough to be dangerous. What made
it publishable was checking what the same corpus does **when it does know the answer**: the primary
key list encodes a dialect flag explicitly in **907 of its 922** entries. Universal explicit
labelling is what you would expect precisely *because* the default is something else — and if that
ratio had come back near zero, the finding would have been the opposite and much bigger.

⭐ Whenever you overturn a recorded unknown, find the population where the same system states the
answer out loud, and check that your verdict explains why *those* are explicit.

## Then guard the premise, because it is usually one edit away

This verdict depends on a `force_gen()` helper not running by default. Three separate facts hold it
up — a guard condition, a flag defaulting off, and a Makefile target not passing that flag — and each
is a one-line upstream change that would silently invert the conclusion. A paragraph noting this is
worth little; a check that re-derives all three on every run and **refuses** is worth the whole
finding. Drive each refusal once, against a mutated copy of the inputs, so the guard is known to fire
rather than assumed to.

## And a key is not a verdict

Overturning the unknown produces *access* to an answer, not the answer. In the same audit, 37 further
rows turned out to be named by some other test list — and reading those lists' status columns as
parse verdicts would have manufactured defects, because they belong to different back ends. One of
them contains a compile-error entry for a construct that is perfectly legal in the language under
test: the error is the back end refusing, not the parser. ⇒ **enumerate the keys first, adjudicate
what each key MEANS second.** Only the first step is cheap, and collapsing them is how a
classification fix becomes a defect report.

⚠️ **Bound.** This says a recorded unknown deserves the *"is this the artifact that would know?"*
question. It does not say such notes are usually wrong — most are exactly right, which is why the
question has to be asked deliberately rather than triggered by suspicion.
