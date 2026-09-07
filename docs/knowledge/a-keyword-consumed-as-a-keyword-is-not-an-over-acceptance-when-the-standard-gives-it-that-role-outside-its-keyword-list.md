---
id: a-keyword-consumed-as-a-keyword-is-not-an-over-acceptance-when-the-standard-gives-it-that-role-outside-its-keyword-list
title: A keyword the falsifier proves CONSUMED AS A KEYWORD is still not a dialect over-acceptance when the profile's own standard gives the word that role outside its reserved-word list — IEEE 1364-2005 clause 19.8 spells `timescale's time_unit as s / ms / us / ns / ps / fs and Annex B lists no directive's arguments — and the census earns such a row with a fourth mechanical leg, never with a note
answers:
  - "the verilog_2005 keyword-faithfulness census refuses with fs / ms / ns / ps / us reachable and no witness — are the time units an over-acceptance"
  - "is `ns` a reserved word in IEEE 1364-2005 — why does `timescale 1 ns / 1 ps parse if Annex B does not list it"
  - "leg 3 says the keyword was consumed as a keyword (the substituted witness rejects) — when is that verdict wrong"
  - "what is leg 4 of the keyword-faithfulness census — the identifier witness and the backtick-line rule"
  - "what does the fourth column of v2005_keyword_faithfulness/witnesses/MANIFEST.tsv mean"
  - "what does verdict directive_argument mean in census.tsv / census.md"
  - "how do I prove a keyword is reserved only inside a compiler directive and stays an identifier elsewhere"
  - "why are s / ms / us / ns / ps / fs reachable under verilog_2005 only since 1.0.217"
  - "does `wire ns;` parse under verilog_2005 — and should it"
tags: [systemverilog, verilog-2005, profiles, keywords, census, directives, instruments, adjudication]
date: 2026-09-07
status: current
evidence: "SV-CORPUS-GRAD.3.39 (release 1.0.217, ledger SV-0090). Typing the compiler directives made time_unit reachable under verilog_2005 through timescale_directive alone (time_literal stays SV-only), and stimuli/sv/v2005_keyword_faithfulness_census.py refused: 5 reachable IEEE-1800-only keyword(s) have no witness: ['fs', 'ms', 'ns', 'ps', 'us'] (rust/target/s339/step_kw_v2005.log). Release probe, verilog_2005: `timescale 1 ns / 1 ps + module m; endmodule ACCEPTS; with ns swapped for zzq_census_ident REJECTS (leg 3: consumed as a keyword); module m; wire ns; wire ps, fs, ms, us; assign ns = ps; endmodule ACCEPTS, and still accepts after the swap (the parser has not reserved the word). IEEE 1364-2005 docs/verilog/2005/md/section-19-compiler-directives.md:557-575 (`timescale time_unit / time_precision; the unit symbols) and Annex B (no directive argument listed). Leg 4 coded in the census (VERDICT_DIRECTIVE_ARGUMENT, the manifest's optional fourth column, refusals for a word used off a backtick-led line, a missing or load-bearing identifier witness); five witness pairs under docs/tasks/artifacts/sv_corpus_grad/v2005_keyword_faithfulness/witnesses/; probe.sh section [9] fires each refusal and shows the verdict column MOVING to over_acceptance when the identifier witness rejects."
reverify: "python3 stimuli/sv/v2005_keyword_faithfulness_census.py | grep -o 'over_acceptances=[0-9]* .*directive_argument=[0-9]*'   # over_acceptances=0 … directive_argument=5; bash docs/tasks/artifacts/sv_corpus_grad/v2005_keyword_faithfulness/probe.sh | tail -1   # pass=18 fail=0"
---

**The verdict leg 3 gives is right, and the conclusion drawn from it can still be wrong.** The census
adjudicates every IEEE-1800-only keyword a `verilog_2005` parse can reach on three legs: the witness is
accepted under the profile, accepted under `sv_2017` (so it is well-formed SystemVerilog), and REJECTED
once the keyword is swapped for a fresh identifier — the falsifier that proves the parser consumed the
word as a keyword. For `` `timescale 1 ns / 1 ps `` all three hold: swap `ns` and the directive is
malformed. Leg 3 is telling the truth. What it cannot know is whether the profile's standard gives the
word exactly that role somewhere Annex B does not look. IEEE 1364-2005 clause 19.8 does: the `time_unit`
of `` `timescale `` "shall be" one of `s`, `ms`, `us`, `ns`, `ps`, `fs`, and a 1364-2005 parser that
accepted `` `timescale 1 xyz / 1 ps `` would be the defective one. Annex B is a list of reserved WORDS;
a directive's arguments are not words the language reserves — `wire ns;` is legal 1364-2005 — so the
keyword census's oracle is silent on them, and reachability alone would have called five faithful
tokens over-acceptances.

**Why a note is not enough.** The manifest already carries a note column, and `tx_path_delay_expression`
shows the temptation: one sentence explaining that the LRM spells a nonterminal where PGEN has a
terminal. That row is safe because leg 3 refutes it mechanically (the word is an identifier there). A
row that leg 3 CONFIRMS cannot be waived by prose without turning the census into a list of exceptions.
So the claim "a keyword only inside a compiler directive" is decomposed into two facts the probe can
falsify, and the row is EARNED: (4a) every occurrence of the keyword in the directive witness sits on
a backtick-led line — clause 19 territory and nowhere else — and (4b) a SECOND witness, named in the
manifest's optional fourth column, uses the same word as an ordinary identifier, ACCEPTS under
`verilog_2005`, and still accepts after the same substitution — the word sat in an identifier slot, so
the parser has not reserved it. A second witness that rejects keeps the row an over-acceptance (the
verdict column moves, and `probe.sh` shows it moving); a word used off a directive line, an identifier
witness in which the word is load-bearing, or a missing file REFUSES the census outright.

**The general rule.** A profile's faithfulness oracle is only as wide as the list it is keyed on. When a
grammar change makes a token reachable through a construct the oracle's list does not cover — a
directive argument, a system-task name, a pragma keyword — the right move is not to widen the note
column but to add the leg that distinguishes "reserved everywhere" from "meaningful here, an identifier
elsewhere", and to make the distinguishing witness a file the probe can fail. The five rows cost two
lines each and one new verdict; the alternative was five over-acceptances that were not, or five
exceptions nobody could re-check.
