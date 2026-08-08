---
id: audit-a-reports-key-against-its-own-illustration
title: A report that publishes both a KEY and an ILLUSTRATION must be checked that the two name the same thing — the counts can be right while the human-readable half lies
answers:
  - "how do I audit a clustering or triage report before I plan work from it"
  - "my cluster signature and its example line describe different constructs — why"
  - "why does anchoring an excerpt to furthest_position give a misleading source line"
  - "how do I port a corpus triage tool to a case-insensitive language like VHDL"
  - "why did my cluster counts fragment into many small classes that should be one"
  - "should I fork a family-specific analysis tool or parameterize it"
  - "a corpus pass-rate looks stale but re-measuring reproduces it — is anything wrong"
tags: [instrument-honesty, corpus, clustering, triage, vhdl, measurement, root-cause]
date: 2026-08-08
status: current
evidence: stimuli/sv/cluster_rejects_valid.py (`signature_at` returning `first_token_pos`, the `FAMILIES` keyword/operator/fold-case table, the `--results` raw-fail lane); docs/tasks/artifacts/corpus_grad_all/vhdl_fail_clusters.md; docs/tasks/CORPUS-GRAD-ALL.md leaf .2.1; stimuli/run_external_corpus.sh (repo-root-relative column 3)
reverify: "python3 stimuli/sv/cluster_rejects_valid.py --results stimuli/vhdl/characterization/results.tsv --grammar vhdl --jobs 8 --out rust/target/km_vhdl_clusters.tsv --summary rust/target/km_vhdl_clusters.md && awk -F'|' '/`alias ID :`/ {print $4}' rust/target/km_vhdl_clusters.md | grep -q alias && echo KEY-MATCHES-ILLUSTRATION"
---

**A ranked report is two claims, not one.** The *key* (the cluster signature, the bucket name) and the
*illustration* (the example row, the excerpted source line) are computed by different code, and only
the key is usually tested. So the failure mode is asymmetric and quiet: **the counts stay correct
while the illustration points somewhere else** — and the illustration is the half a human reads to
decide what to build.

Measured instance. A stuck-point clusterer keyed on `furthest_position` and excerpted the source line
at the same byte. But `furthest_position` frequently lands on trailing whitespace or a newline, and
the tokenizer that builds the signature *skips whitespace, newlines included* — so the signature's
first token came from the **next** line while the excerpt was read at the raw byte:

| cluster key | illustration published | what the row actually was |
|---|---|---|
| `alias ID :` | `constant USER_RIGHT : integer := 1 ;` | `alias Last : std_logic is ResultParam(0) ;` |
| `shared variable ID` | `subtype OperationSlvType is std_logic_vector(0 downto 0) ;` | `shared variable OperationFifo : … ;` |

Nothing in the counts hinted at it. The fix is to have the tokenizer **return the offset of the first
token it consumed** and anchor the excerpt there, so key and illustration are derived from one
position by construction rather than from two positions that usually agree.

⇒ **Audit rule: before planning work from a clustered report, pick two or three of its top rows and
check that the illustration is an instance of the key.** If it is not, the report's ranking may still
be usable but its examples are not, and examples are what leaves get cut from
([[feedback_instrument_needs_ground_truth]]).

⛔ **Case sensitivity is a CLUSTERING property, not a lexing detail.** Porting such a tool to a
case-insensitive language (VHDL) without folding case makes `ENTITY`, `Entity` and `entity` three
classes, each looking a third as important as the one real class — the ranking silently
under-reports its own top item. A family profile therefore has to carry the language's *identity
rules* (fold-case), its keyword set, AND its multi-character operator set: a missing operator entry
splits `<=` into `<` and `=`, merging classes that should be distinct. All three are part of "what
language is this", not three unrelated options.

⭐ **Parameterize the analysis tool; do not fork it per family.** In the measured case the engine —
probe, read the deep locus, tokenize N tokens, rank by member count — was entirely family-neutral;
only a keyword table, an operator table and a fold-case flag were not. Forking would have reproduced
the copied-block defect described in
[[a-copied-diagnostic-covers-only-where-it-was-pasted]] one layer up, in the tool built to consume
that very diagnostic. Adding a family should cost a table entry.

⚠️ **A stale measurement that reproduces exactly is still a broken process.** Re-running a 17-day-old
corpus characterization returned byte-identical totals and per-sub-corpus rows. That is not
reassurance: the report cited a raw input git did not track, whose every row named a foreign checkout,
so it could be neither re-run nor diffed — correctness was luck. What converts luck into a property is
**pricing the re-measure**: at 71 seconds for 13 720 files, freshness stops being a discipline problem
and becomes a gate nobody has written yet. Re-pricing a measurement changes which remedy is correct —
when it is expensive you argue about storing its output, when it is seconds you argue about scheduling
it.

⭐ **And read what the measurement actually says about CAUSE before naming the work.** A 29.4 %
pass-rate invites "the parser has bugs". A keyword census over the grammar said the top classes were
*absent language* — seven reserved words with zero occurrences in a grammar whose own header calls
itself a seed subset. Same number, different campaign: grammar growth rather than debugging. Check
whether a low pass-rate measures defects or scope before cutting leaves
([[absence-in-the-corpus-is-not-a-property-of-the-rule]]).
