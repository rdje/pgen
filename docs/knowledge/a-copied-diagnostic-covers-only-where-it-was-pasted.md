---
id: a-copied-diagnostic-covers-only-where-it-was-pasted
title: A diagnostic written as an inline block covers only the call sites someone pasted it into — verify coverage by COUNTING sites, never by reading the catalog entry
answers:
  - "the toolbox says a diagnostic is always on — how do I check that it actually is"
  - "why does this parse error carry furthest_position for one grammar but not another"
  - "why is a corpus rejection population impossible to cluster into defect classes"
  - "the reported failure position names nothing useful — where is the real defect"
  - "how do I make a per-family diagnostic reach every family at once"
  - "my diagnostic change breaks a pinned rejection-signature contract — rebaseline or not"
  - "should a failure signature include the byte offsets it happened to observe"
tags: [diagnostics, toolbox, parser-registry, corpus, signatures, root-cause, instrument-honesty]
date: 2026-08-08
status: current
evidence: rust/src/parser_registry.rs (`augment_error_with_furthest_position` and its 12 call sites, one per own-parser `parse_with_<family>_detail`); rust/src/ast_pipeline/stimuli_generator.rs (`normalize_rejection_signature`, the decoration strip); TOOLBOX.md §3.2; docs/book/src/parseability-probe-debug.md "Furthest-Position Error Diagnostic"; docs/tasks/CORPUS-GRAD-ALL.md leaf .2.0
reverify: "test $(grep -c augment_error_with_furthest_position rust/src/parser_registry.rs) -eq 13 && printf '{\"a\": [1, 2, ]}' > rust/target/km_bad.json && ./rust/target/debug/parseability_probe --parse json rust/target/km_bad.json 2>&1 | grep -q 'furthest_position=' && echo DIAGNOSTIC-UNIVERSAL"
---

**An instrument's documented coverage and its real coverage are independent variables.** PGEN's
`TOOLBOX.md` §3.2 was titled *"Furthest-position error diagnostic (always on)"* and said **every**
parse-failure error carried `furthest_position`. Measured, it reached **2 of the 12** own-parser
detail parse paths — and not even uniformly inside one family, since SystemVerilog's
`--library-in-dir` variant lacked what its plain path had. The claim had stood unchallenged because
nothing contradicts a document except a measurement.

**Two greps settle this class of question, and they are the right first move on any inherited
instrument:**

```bash
grep -n 'furthest_position()' rust/src/parser_registry.rs      # where the diagnostic is PRODUCED
grep -nE '^fn parse_with_.*_detail' rust/src/parser_registry.rs # where it SHOULD be
```

Two numbers that should match and don't *are* the diagnosis. Confirm the signal was always
available before blaming the engine: `grep -c 'pub fn furthest_position' generated/*_parser.rs`
returned 1 for all 11 generated parsers — nothing had to be computed, only reported.

⛔ **The root cause is a SHAPE, not an oversight.** The augmentation was born as an eleven-line
`map_err` block inline in one function and hand-copied once. A copied block has no single site to
extend, so the tenth family that needs it is exactly as far away as the third. The moment it is a
named function, "should this path report the deep locus?" stops being a decision anyone can forget.
⇒ **when a behaviour must hold for every member of a set, make it a function the set calls, not a
paragraph each member owns a copy of.**

**A shallow position is worst exactly where corpus work lives: flat item-list entries.** A rule like
`vhdl_file := design_unit*` cannot fail *inside* anything — when an item does not parse the
quantifier just stops, so the reported byte is where that item BEGAN. Measured on one OSVVM file:

| | byte | source | information |
|---|---|---|---|
| surface position | 1651 | `package Axi4ComponentPkg is` | none — where the item list stopped |
| `furthest_position` | 3852 | `AxiBus : view Axi4ManagerView of Axi4RecType ;` | the unsupported VHDL-2019 mode view indication |

58 source lines apart. Across a corpus the difference is categorical, not incremental: stuck-point
**clustering** keys on this byte, so with only the surface position a 9 689-file rejection population
collapses into two meaningless buckets (`package … is`, `architecture … of … is`), and with it the
same population sorts into ranked, nameable defect classes. A triage tool can be present and *inert*
because its input signal does not exist — check the signal before writing a second tool
([[feedback_read_prior_art_before_designing]]).

⭐ **When a diagnostic change threatens a pinned signature contract, ask whether the contract's KEY
should have been ignoring that field all along.** Appending ` [furthest_position=N, …]` would have
broken pinned `Parser did not consume full input at position #` signatures, and the reflex is to
rebaseline. But a rejection signature names a failure *CLASS*, and once digit runs collapse to `#`
the whole bracket is a **constant suffix that discriminates nothing**. Stripping it in the normalizer
costs zero dedup precision and makes signatures decoration-independent — so a 2 → 12 rollout moved
no proof surface at all. Rebaselining would have buried the question instead of answering it.

**State where a universal diagnostic stops, in the code.** Two boundaries here, both deliberate: a
bootstrap parse path that owns no parser object has no furthest position to report, and the regex
lane augments the *parse* error only — the PCRE2 compile-contract error that follows is an oracle
disagreement, and decorating it with a byte offset would invent a locus it does not have. An
unstated exclusion is indistinguishable from the bug this card describes.

See also [[feedback_instrument_needs_ground_truth]] (an instrument with no ground truth is a
confident guess) and [[feedback_systematically_use_debug_toolbox.md]] (reach for the tools first, and
build one when they cannot show WHY+WHERE).
