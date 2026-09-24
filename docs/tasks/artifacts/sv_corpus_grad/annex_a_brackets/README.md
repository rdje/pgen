# `annex_a_brackets/` — the Annex A LITERAL-BRACKET census (`SV-CORPUS-GRAD.21`)

IEEE prints literal tokens in **bold** and EBNF optionality in plain brackets. The PDF-to-text extraction
under `docs/systemverilog/{2017,2023}/md/` and `docs/verilog/2005/md/` keeps only the characters, so a bold
`[` and an optional `[` are the same byte — and the class bit the grammar twice before this census
(`.3.30`, the UDP table symbols; `.3.63`, `edge [ … ]` and `terminal_identifier [ … ]`), each time transcribed
as `( … )?`. This instrument makes the population known and pins every member.

## What `census.py` does

1. Enumerates EVERY bracket pair of every alternative of every production in the three Annex A texts
   (IEEE 1800-2017, 1800-2023, 1364-2005), nesting-aware — `census.tsv`, one row per pair.
2. Classifies each pair LITERAL or OPTIONAL: a GENERIC rule where the characters alone decide (an empty
   `[ ]`; an operator-led `[$`, `[*`, `[+`, `[->`, and `[=` inside a repetition production), a CURATED
   table where only the clause decides (dimensions, selects, ranges, `new [ ]`, enum ranges, bins, value
   ranges, the sequence and property operators, `with [ ]`, the specify terminals, `edge [ ]`), and the
   default OPTIONAL — EBNF optionality — for everything else.
3. For every LITERAL pair cites the clause's own example that spells the brackets (a regex over the
   edition's section texts, never Annex A itself; where no example carries the bracket the provenance of the
   literal reading is named instead), reports whether the production's grammar rule reaches an `lbrack`
   token within three references (a hint), and runs the production's REPRODUCER under `probes/` on the
   edition's profile — ACCEPT with the brackets is the proof, a REJECT is a finding.

```bash
python3 docs/tasks/artifacts/sv_corpus_grad/annex_a_brackets/census.py            # rewrite census.tsv + summary
python3 docs/tasks/artifacts/sv_corpus_grad/annex_a_brackets/census.py --check    # refuse if the tracked census is stale
```

## Result (2026-09-24, grammar `1.0.245`, `a701ba7cd`)

| | 1800-2017 | 1800-2023 | 1364-2005 | total |
| --- | ---: | ---: | ---: | ---: |
| bracket pairs | 461 | 513 | 204 | 1 178 |
| LITERAL | 33 | 38 | 17 | 88 |
| OPTIONAL | 428 | 475 | 187 | 1 090 |

88 literal pairs over 59 edition × production combinations, every one with clause evidence or a named
provenance (`delayed_data` / `delayed_reference`: no clause example spells the bracket; the reading is
`.3.63`'s), every one probed: **33 / 38 / 17 ACCEPT on sv_2017 / sv_2023 / verilog_2005, zero REJECT** —
the grammar renders every literal bracket the three standards carry, and the two known members are the
only members the grammar ever missed.

⭐ The probes also read one profile DIFFERENCE that is the standards', not the grammar's: `$width` with
two arguments ACCEPTS on `verilog_2005` and REJECTS on both SV profiles, because IEEE 1364-2005 A.7.5.1
spells `$width ( controlled_reference_event , timing_check_limit [ , threshold [ , notifier ] ] )` while
IEEE 1800-2017 and 1800-2023 A.7.5.1 spell `… , timing_check_limit , threshold [ , [ notifier ] ] )` —
the threshold is MANDATORY under 1800 — and the grammar carries exactly that twin
(`sv_dollar_width_timing_check_no_threshold_v2005`). Not a defect; recorded so the next reader does not
re-derive it (`probes/width_2arg.sv`).

## The reproducers (`probes/`)

Every file is a minimal construct whose brackets are literal in the standard, pinned in
`stimuli/sv/adjudication_repros/MANIFEST.tsv` as ACCEPT rows so the release ceremony's pins step reads them
on every release. A construct that parses through the wrong production would still ACCEPT here — the
rule-fire partition (`.22`) is the instrument that reads the arm, and the census's `grammar` hint names the
rule that carries the bracket token.
