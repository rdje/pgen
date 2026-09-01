---
id: a-missing-alternative-is-invisible-to-every-instrument-that-reads-only-what-exists
title: A dropped member of a keyword enumeration is invisible to every instrument that reads only what the grammar DECLARES — the lint resolves every name, the certificate witnesses every rule that exists, the corpus sees it only where a file spells the word; only a census against the standard's own enumeration can see it
answers:
  - "lint, certificates and the corpus are all green, so can a keyword alternation still be missing a member"
  - "why did nothing catch net_type shipping ten of the LRM's twelve alternatives"
  - "how do I check every LRM keyword enumeration against the shipped grammar at once"
  - "what does LRM-KEYWORD-ENUMERATION measure and what does it deliberately not score"
  - "is a trailing digit on an LRM production name a footnote marker or part of the name"
  - "why does the LRM extractor merge strength0 and strength1 into one rule"
  - "an instrument reports a shipped rule reaches keywords the LRM enumeration lacks, is that an over-acceptance"
tags: [grammar-fidelity, lrm, keywords, census, oracles, diagnosis, systemverilog, extractor]
date: 2026-09-01
status: current
evidence: "LRM-GRAMMAR-FIDELITY.1e (PGEN-LRM-GRAMMAR-FIDELITY-0001), discharging SV-CORPUS-GRAD.13n(e). net_type shipped 10 of the LRM's 12 alternatives for the whole life of the flattened grammar (ledger SV-0075) with --lint-grammar green, the certificate pass green and 9 corpus rows failing that no guard attributed. The census stimuli/sv/lrm_keyword_enumeration_census.py joins the vendored Annex A markdown (read with the extractor's own scraping primitives, pinned equal to extract_rules on 734/747/381 rules) to the shipped grammar's raw_ast envelope under each edition's profile: 109 enumerations, 97 name-matched, 289 keyword members, 0 MISSING at HEAD; a RED control that removes the two net_type branches from a copy of the shipped grammar reads MISSING 6 of 6 with exactly six rows moving. Reusing the extractor's canonicalize_rule_name reported strength0 missing strong and weak, members the LRM never wrote: that canonicaliser strips trailing digits unconditionally and merges strength0/strength1 and delay2/delay3 in all three editions (grammars/systemverilog_lrm_profiled_generated.ebnf:1860 carries strength := kw_supply | kw_strong | kw_pull | kw_weak). The extra column's first customer was edge in 1364-2005 edge_identifier ::= posedge | negedge, confirmed a verilog_2005 over-acceptance by three probe legs (SV-CORPUS-GRAD.13o)."
reverify: "python3 stimuli/sv/lrm_keyword_enumeration_census.py --grammar grammars/systemverilog.ebnf | head -1"
---

**Three instruments were green while `net_type` lacked two of its twelve LRM members, and each one
was doing exactly its job.** `--lint-grammar` holds that every referenced name resolves — a branch
that does not exist references nothing. The certificate pass holds that every rule that EXISTS is
witnessed — it cannot demand a witness for a rule nobody declared. The corpus lanes hold that every
vendored file gets a verdict — the defect is visible only where a file happens to spell the missing
word, which nine files did and nothing attributed. A missing alternative in a keyword-bearing
alternation is therefore the quietest defect shape a grammar has: the text lexes as a keyword and
then has no production to land in, and the parse dies with no signal that a production is missing.

**The only instrument that can see it reads the STANDARD's enumeration, not the grammar's.** The
census takes every LRM production whose alternatives are all bare tokens and that names at least
one reserved keyword — reserved per IEEE's own Annex B list for the edition, never inferred from a
spelling — and asks, member by member, whether the shipped rule of that name reaches the member's
terminal through bare alternatives live under the edition's profile. Three verdicts: `present`,
`MISSING` (a terminal spells the word, the rule cannot reach it), `MISSING_NO_TERMINAL` (nothing
spells it — the shape `net_type` shipped in).

**Two things it prints and deliberately does not score.** A production with no shipped rule by
name is `unmatched`: the shipped grammar is a hand-annotated synthesis, so a lifted production is an
adjudication row, not a defect (12 at HEAD; 9 outside the source language, 3 that parse). And
`extra` — the shipped closure reaching a keyword the enumeration does not name — is a worklist,
because *"the annex does not derive it"* is a measured known-bad over-acceptance classifier: the
`assignment_pattern_key` extras `string`/`chandle`/`event` are legal by §10.9.2's own example. The
column still earned its keep on the first run — `edge` in `edge_identifier ::= posedge | negedge`
became, three probe legs later, a confirmed `verilog_2005` over-acceptance.

**A trailing digit is a footnote only when nothing uses that spelling.** IEEE prints footnote
numbers on definitions (`class_item_qualifier10 ::=`) and on uses (`assignment_pattern_expression37`)
but never spells one production with the same digits in both places; a real name — `strength0`,
`delay2`, every keyword such as `tri0` — is defined and used verbatim. A canonicaliser that strips
unconditionally merges `strength0`/`strength1` and `delay2`/`delay3`, and the census's first cut,
built on the extractor's, reported members the LRM never wrote. Keep a keyword always; strip a
defined name only when nothing references it with its digits; strip a used name when the stripped
spelling is known.
