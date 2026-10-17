---
id: an-option-file-is-the-command-line-read-it-as-the-tool-does
title: An option file is the command line — read it as the tool does, and withhold what its rules do not settle (`SVPP-EXPANSION.4c.5`)
answers:
  - "why did the derived context report a verilator -f list as not read, and what does it read now"
  - "how does pipeline_context.py read a .vc option file — comments, nested -f / -F, $TEST_ROOT, -v, +define+A+B"
  - "a +define+ in an option file has a quoted value — why is it withheld instead of applied"
  - "which files of a verilator unit get predecessors — only the top, or every listed member"
  - "what is a -v library file's position in the compilation unit"
  - "t//x.v in an option file — a comment or a doubled slash"
tags: [systemverilog, pipeline, context, verilator, derivation, option-file]
date: 2026-10-17
provenance: SVPP-EXPANSION.4c.5 (PGEN-SVPP-EXPANSION-0050); SVPP-EXPANSION.4c.2 (the unit lists; -f reported); SV-CORPUS-GRAD.58 (the key reads the same tokens); SV-CORPUS-GRAD.58a (the two refused rows that asked for it)
reverify: "python3 stimuli/sv/pipeline_context.py --self-test   # PIPELINE-CONTEXT-SELF-TEST: checks=39 failed=0; python3 stimuli/sv/pipeline_context.py --check   # PIPELINE-CONTEXT: OK; grep -c verilator stimuli/sv/pipeline_context_units.tsv   # 216"
---

# An option file is the command line

**The rule.** `SVPP-EXPANSION.4c.2` derived a file's unit predecessors from the harness's own declarations and
reported a verilator `-f` list as *not read*, because Verilator's option-file syntax has forms a regex over the
driver would guess at. A guess was never the alternative to skipping the file; reading it with the tool's rules was.
`read_option_file` in `stimuli/sv/pipeline_context.py` reads a `.vc` the way Verilator does: whitespace-separated
options; `//`, `/* */` and leading-`#` comments; nested `-f` (paths relative to the run directory) and `-F` (relative
to the file's own directory); `$TEST_ROOT` / `${TEST_ROOT}` the test's directory and `$TEST_REGRESS` the run directory,
a doubled slash collapsed; `-v X` a LIBRARY file; `+define+A+B=V` plus-args, every `+` a new name. A `//` opens a
comment only where a token starts — the upstream test `t_flag_f.vc` writes `${TEST_REGRESS}/t//t_flag_f__3.v` on
purpose, and the first cut lost the file to a comment rule before the self-test pinned the shape.

**What is withheld, and why.** Derived, never guessed: an option file absent from the tree, an unterminated comment
(`t_flag_f_bad_cmt.vc`, whose driver expects the failure), an environment name the harness sets at run time, an
option file naming itself, and a `+define+` whose VALUE carries a quote or a backslash — `t_flag_define.vc`'s
`STRING1="\"New String\""` and `LIT2=32\'h600D600D` depend on the option file's quoting rules, which this reader does
not derive. Each is reported in the deriver's `unit` lines and contributes nothing; the eight plain plus-args of the
same file are read.

**Who gets predecessors.** A verilator unit is `[the driver's sources in command order, the top last]` —
`driver.py:1099` puts `*flags` before `top_filename` — and every listed member reads after the files before it, not
only the top (the `.4c.2` arm stopped there; the slang arm had the per-member reading from the start). A `-v`
library file is parsed on demand once every listed file has been, so its predecessors are all of them, the top
included, and it precedes nothing. The quoted items of a flag list are tokenized in order, so a string naming several
files yields its unit. Measured on the tree: `pipeline_context_units.tsv` 44 → 222 rows on 17 → 46 files; the lane
re-run under three readings, **exactly 2 movers on each of the three readings, both pre-registered** (`subB.v` `pp_fail → pass`, `subsub.v` `pp_fail → fail` on the `param_t` declaration barrier inside its included `.vh`), none other.

**What stays routed.** An include fragment's own-row context (the includer's text before the `include` line) is a
lever measured larger than the leaf — 266 refused-context rows are include targets — and is `.4c.6`; `-y` library
members resolved by module name are 19 files, all passing, so nothing there to read.

**Related.** [[a-pick-among-an-ips-real-builds-is-still-a-guess]] (the same rule for a build list);
[[a-delay-on-a-member-select-needs-parentheses]] (the two rows that asked for this).
