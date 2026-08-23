---
id: a-trade-off-may-be-a-consistency-question-already-answered-in-the-same-artifact
title: Before scoring design alternatives, ask what the neighbouring constructs in the same artifact already do — a trade-off is often a consistency question that was settled long ago, five lines up
answers:
  - "how do I choose between several fixes that all work"
  - "I scored four alternatives and none is clearly best, what now"
  - "should I invent a new escape or quoting convention for this grammar"
  - "do I need a prior-art search for this design decision"
  - "how do I know if my design is consistent with the rest of the file"
  - "my fix narrows the language, is there a way to avoid the cost"
  - "what is the cheapest question to ask before building candidate arms"
tags: [design, prior-art, grammar-authoring, decision-making, claim-verification]
date: 2026-08-23
status: current
evidence: |
  GRAMMAR-WELLFORMED.H.16.6d (`PGEN-GRAMMAR-WELLFORMED-0181`).
  Four terminals in `grammars/semantic_annotation.ebnf` were spelled `[^\s]`, so an unquoted path or
  URL swallowed the delimiter that should have closed its enclosing collection. Any fix narrows the
  language, so I built three candidate formulations, scored them against four corpora, ruled for the
  best of them, edited the shipped grammar, regenerated the parser and rebaselined reproducibility.

  The ruling was defensible and it was not the best answer. The director then proposed — tentatively,
  "maybe that's a dumb idea" — that the delimiters be made ESCAPABLE with a backslash rather than
  simply forbidden. Checking it surfaced this, at lines :158 and :165 of the same file, five lines
  above the rules I had been editing:

      double_quoted_string := /"([^"\\]|\\.)*"/
      single_quoted_string := /'([^'\\]|\\.)*'/

  The grammar already solved this exact problem — "the literal must not contain its own delimiter,
  unless escaped" — with this exact shape, for the quote character. The four path/URL terminals were
  simply the ones that had never been given the treatment.

  It also DOMINATED the arm I had ruled for: same containment (0 self-rejects over 3 200 samples,
  against the control's 15), and less than half the expressiveness cost — 4 narrowed legitimate
  values instead of 9, each recoverable IN PLACE with one backslash rather than by re-quoting the
  whole value.
reverify: "grep -nE '^(double_quoted_string|single_quoted_string) :=' grammars/semantic_annotation.ebnf && grep -nE '^(absolute_path|relative_path|home_path|url_reference) :=' grammars/semantic_annotation.ebnf   # the same ([^X\\\\]|\\\\.) shape, now in both families — it was in the first one all along"
---

Building candidate arms and scoring them against real corpora is the right method when a design
genuinely trades one thing against another. The failure mode is reaching for it **first**, because a
scored comparison feels like rigour and therefore never prompts the cheaper question:

> **What do the neighbouring constructs in this same artifact already do about this?**

That question costs one `grep`. In this case it would have produced the winning formulation before
any arm was built, and it also dissolved the governance overhead — a convention already used six
times in the file is not a new surface needing a prior-art search, it is *consistency repair*.

## Why the scored comparison did not surface it

Because every arm I built was a variation on **my own** framing of the problem. I had decided the
question was *"which characters should this terminal refuse?"*, and all three candidates answered
that question at different strengths. The escape mechanism is an answer to a different question —
*"how does a literal in this language carry a character that would otherwise end it?"* — and no
amount of scoring within the first framing reaches outside it. Arms explore a space; they do not
question its boundaries.

⇒ **scoring alternatives tests strength, not framing.** Consistency with sibling constructs is a
check on the framing, which is why it has to come first.

## The practice

1. Before generating candidates, `grep` the artifact for constructs that face the same structural
   problem. In a grammar: how do the string literals delimit themselves? In a schema: how do sibling
   fields express optionality? In an API: how do neighbouring endpoints paginate?
2. If a sibling already solves it, your default is *"do what the sibling does"*, and the burden of
   proof shifts onto deviating rather than onto conforming.
3. Say so explicitly in the write-up. *"This is the shape `double_quoted_string` already uses"* is a
   stronger justification than any score table, because it also buys the reader a mental model they
   already have.
4. When someone offers a suggestion hedged as naive, check it before you weigh it. The hedge is about
   their confidence, not about the idea's merit — this one arrived as *"maybe that's a dumb idea"*
   and was better than a ruling backed by four scored arms.

## The residue worth keeping

I had already **landed** the inferior arm — grammar edited, parser regenerated, reproducibility
rebaselined — before this came to light. Re-landing cost one more regeneration and one more gate run,
and the reproducibility gate immediately caught its own now-stale baseline from the first attempt. ⇒
the cost of correcting a design ruling late is bounded by how mechanised the lockstep is; if reversing
a landed decision is expensive, that is a fact about the pipeline, not a reason to keep the decision.

Related: [[a-guard-after-a-greedy-regex-atom-cannot-shorten-the-match]] ·
[[a-containment-test-over-the-input-text-is-a-test-about-the-text]]
