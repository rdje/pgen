---
id: a-guard-after-a-greedy-regex-atom-cannot-shorten-the-match
title: A guard placed after a greedy regex terminal filters where the match ENDED — it is not a lever on where it ends, so a terminal that matches too much must be constrained INSIDE the terminal
answers:
  - "my terminal is eating the delimiter that should follow it, how do I stop it"
  - "can I add a negative lookahead after a regex terminal to stop it over-matching"
  - "does a !(...) guard make a regex backtrack to a shorter match"
  - "how do I stop a URL or path terminal swallowing a closing bracket"
  - "my lookahead guard made the grammar worse, why"
  - "where should the constraint live when a terminal is too greedy"
  - "how do I let a delimiter appear inside an unquoted literal"
tags: [grammar-authoring, terminals, regex, ordered-choice, pgen-engine, grammar-wellformedness]
date: 2026-08-23
status: current
evidence: |
  GRAMMAR-WELLFORMED.H.16.6c → H.16.6d (`PGEN-GRAMMAR-WELLFORMED-0180`, `-0181`).
  `grammars/semantic_annotation.ebnf` spelled four value terminals as `[^\s]` — any run of
  non-whitespace. `]`, `}`, `)` and `,` are not whitespace, so an unquoted path or URL abutting a
  collection's closing delimiter consumed it: `ftp://98eS]` parsed as a WHOLE `annotation_value`
  while `[ftp://98eS]` was rejected, and `[ftp://98eS ]` accepted on one added space.

  The obvious repair is to leave the class alone and guard the terminal:
  `url_reference := /(https?|ftp|file):\/\/[^\s,\]\}\)]+/ !(/\s*/ "=>")`. Measured on a scratch arm,
  it fails in BOTH directions at once:

    http://PYJ=>http://aFC        ACCEPTED  — the regex had already eaten the arrow, so the guard
                                             looked past the end of input and found nothing to object to
    { http://PYJ => http://aFC }  REJECTED  — here the guard DID fire, so a URL could no longer be a
                                             map key at all

  Strictly worse than the un-guarded control, and it never entered the scored set. The shipped fix
  puts the constraint inside the class instead — exclude the delimiters and `>`, forbid a TRAILING
  `=` (so `=` stays legal inside a URL and query strings still parse), and admit a backslash escape:
  `url_reference := /(https?|ftp|file):\/\/([^\s,\]\}\)>\\]|\\.)*([^\s,\]\}\)>=\\]|\\.)/`.
  Result: the grammar's self-rejection of its own generated stimuli went 15 → 0 over 3 200 samples.
reverify: "python3 -c \"import pathlib; s=pathlib.Path('grammars/semantic_annotation.ebnf').read_text(); a=r'url_reference := /(https?|ftp|file):\\\\/\\\\/([^\\\\s,\\\\]\\\\}\\\\)>\\\\\\\\]|\\\\\\\\.)*([^\\\\s,\\\\]\\\\}\\\\)>=\\\\\\\\]|\\\\\\\\.)/'; assert s.count(a)==1, 'anchor moved — re-derive the arm by hand'; pathlib.Path('rust/target/kmv_guard.ebnf').write_text(s.replace(a, r'url_reference := /(https?|ftp|file):\\\\/\\\\/[^\\\\s,\\\\]\\\\}\\\\)]+/ !(/\\\\s*/ \\\"=>\\\")'))\" && for i in 'http://PYJ=>http://aFC' '{ http://PYJ => http://aFC }'; do printf '%s' \"$i\" > rust/target/kmv_g.txt; ./rust/target/debug/ast_pipeline rust/target/kmv_guard.ebnf --interpret-parse rust/target/kmv_g.txt --interpret-entry-rule annotation_value >/dev/null 2>&1 && echo \"ACCEPT $i\" || echo \"reject $i\"; done   # the guarded arm accepts the one it must reject and rejects the one it must accept"
---

A negative lookahead written *after* a terminal feels like it should tighten the terminal. It does
not. The regex atom runs to completion first and commits to its longest match; the guard is then
evaluated at whatever position that left. Its only two outcomes are *"fine, carry on"* and *"reject
this whole alternative"*. **Nothing in that sequence can hand bytes back.**

This makes the guard useless in exactly the case you reached for it, and actively harmful in the
neighbouring one:

| you wanted | what the guard does |
|---|---|
| stop the terminal before `=>` so the arrow survives | the regex already ate the `=>`; the guard sees clean input and passes |
| leave a well-formed `x => y` alone | the regex stopped at the space; the guard fires and kills the whole alternative |

So the arm loses the case it was built for and breaks a case that previously worked — which is why
it is worth *building and measuring* rather than reasoning about. Two probes settle it in a minute.

## Where the constraint has to live

Inside the terminal. Concretely, three techniques, in order of how often they are enough:

**1. Narrow the character class.** If the terminal must not cross a delimiter, say so:
`[^\s,\]\}\)]` instead of `[^\s]`. This is usually the whole fix.

**2. Constrain the LAST character separately** when the forbidden thing is a *two-character* sequence
you cannot simply exclude. To stop `http://x=>y` eating the arrow, excluding `=` outright would also
kill query strings (`?a=1&b=2`) — measured, that cost 13 of 33 legitimate values. Excluding only `>`
does not help either, since the terminal then stops at `>` and the `=>` is already broken across two
tokens. What works is *allow `=` internally, forbid it as the final character*, which is a plain
regex, no look-around required:

```ebnf
url_reference := /(https?|ftp|file):\/\/[^\s,\]\}\)>]*[^\s,\]\}\)>=]/
```

The regex engine backtracks *within the atom* to satisfy the final class — which is precisely the
shortening a following guard cannot do.

**3. Give the excluded characters an escape** so the narrowing costs nothing:

```ebnf
([^\s,\]\}\)>\\]|\\.)*        # "not a delimiter and not a backslash, OR a backslash and anything"
```

⭐ Check whether your grammar already has this shape before designing one — a string-literal rule
almost certainly does, for its own quote character. See
[[a-trade-off-may-be-a-consistency-question-already-answered-in-the-same-artifact]].

## The general form

**A trailing guard is a predicate on the parse state after the atom. If your problem is the atom's
extent, no predicate on its aftermath can fix it.** The same reasoning applies to any committed
token: a lexer's maximal munch, a `+`/`*` quantifier over a sub-rule, a scanner's longest-match rule.
Guards compose with *decisions already made*; they do not re-open them.

Related: [[a-containment-test-over-the-input-text-is-a-test-about-the-text]] ·
[[a-corpus-generated-from-the-artifact-under-test-is-part-of-the-measurement]]
