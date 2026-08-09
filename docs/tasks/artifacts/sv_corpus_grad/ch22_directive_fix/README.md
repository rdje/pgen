# `SV-CORPUS-GRAD.3.14b` — in-scope compiler directives: the grammar fix, measured

Banked 2026-08-09 (session #217). Every number here was measured at `HEAD` with the
instrument identified below; nothing is quoted from `.3.13`/`.3.14a`, and where a banked
claim was **refuted** the refutation is stated rather than absorbed.

## 0. What the leaf had to build

`.3.13` routed 8 rows as a real grammar gap. `.3.14a` then ruled — against its own
premise — that the routed 8 are **not uniform**: clause 22, read directive by directive,
says **tolerate 5 and keep rejecting 3**. This leaf is the grammar change that delivers
exactly that split, plus the adjudication pins that make the 3 rejections a *claim of
correctness with a cite* instead of a quiet deferral.

## 1. ⭐ Reading the clause end to end refuted `.3.14a`'s table too

`.3.14a`'s ruling table enumerated only the six directive names the routed population
happened to contain. Reading IEEE 1800-2017 clause 22 **and** IEEE 1364-2005 clause 19 in
full turned up **two further placement-restricted families the table did not cover**:

| directive | 1800-2017 | 1364-2005 | verbatim placement rule |
|---|---|---|---|
| `` `unconnected_drive `` / `` `nounconnected_drive `` | §22.9 | §19.9 | "These directives **shall be specified outside the design element declarations**." |
| `` `begin_keywords `` / `` `end_keywords `` | §22.14 | §19.11 | "can **only be specified outside a design element**" |

Had the whitelist been derived from the `.3.14a` table rather than from the clause, those
four names would have been tolerated in-scope — an over-acceptance regression against
`feedback_sv_strict_lrm_compliance_default`, landed inside a leaf whose entire purpose was
to prevent that exact mistake. **Sampling a clause is not reading it.**

The two LRMs agree name for name. `` `celldefine ``/`` `endcelldefine `` read the other
way and were kept TOLERATED on the same evidence: "These directives **may appear anywhere
in the source description**, but it is recommended that the directives be specified
outside any design elements" (§22.10 / §19.1) — a recommendation, not a rule.

## 2. The rule that shipped

> Tolerate a compiler directive wherever the LRM does not restrict its placement **and**
> the directive neither hides nor rewrites the source text that follows it.

| decision | names | why |
|---|---|---|
| **TOLERATED** | `` `celldefine `` `` `endcelldefine `` `` `undef `` `` `timescale `` `` `pragma `` `` `line `` (+ `` `undefineall ``, SV profiles only) | no placement restriction; the following text is untouched |
| **KEPT REJECTING** — placement | `` `resetall `` `` `default_nettype `` `` `unconnected_drive `` `` `nounconnected_drive `` `` `begin_keywords `` `` `end_keywords `` | clause text forbids it inside a design element |
| **KEPT REJECTING** — text-hiding | `` `include `` `` `define `` `` `ifdef `` `` `ifndef `` `` `else `` `` `elsif `` `` `endif `` | the file is not honest parser input; already routed to `explained_svpp_*`. `` `define `` also continues across lines with a trailing `\`, so a single-line carrier would MIS-CONSUME it |
| **NOT DIRECTIVES** | `` `__FILE__ `` `` `__LINE__ `` | §22.13 predefined text MACROS — the `.3.13` ruling |

`` `undefineall `` is **absent from IEEE 1364-2005** (grep: 0 hits in clause 19), so it is
the one name gated to `sv_2017`/`sv_2023` via the existing `*_sv_only` idiom. Everything
else is profile-neutral because both clauses say the same thing.

⛔ **Why a name WHITELIST and not the existing blanket `compiler_directive` rule.** Two
independent reasons, each a defect class this repo has already paid for:
1. **Placement is not uniform across clause 22** (§1 above) — a blanket rule widens
   acceptance into six LRM-forbidden constructs.
2. **A backtick line is not necessarily a directive.** `` `MY_MACRO(x) `` at an item
   position is an unexpanded §22.13-class MACRO use — a *preprocessing dependency*. A
   blanket rule swallows the line, drops the items the macro expands to, and converts a
   correct reject into a silent pass. This is the same conflation `.3.13` found in the
   adjudicator's directive allowlist.

`DESIGN-PRIOR-ART` was discharged by `.3.14a` (the `trivia` arm is REFUSED — it would make
the existing `source_text_item` → `compiler_directive` branch ENGINE-SHADOWED-DEAD;
`source_text_item:241` itself reaches only file top level, which IS the defect).

## 3. ⛔ The fix silently did nothing on the first attempt — and why that matters

First landing measured **no change at all**: all 8 rows still rejected after a full
regenerate + rebuild. Toolbox, not eyeballing, found it — the new rule was present in
`generated/systemverilog_parser.rs` (65 references) but had **no caller**, and
`cascade_match_non_port_module_item` carried **8 alternatives, not 9**.

**Root cause: a `#` comment at COLUMN 0 inside an alternation list TERMINATES the rule,
and every `|` arm below it is dropped from the generated parser with no diagnostic.** The
commentary had been placed between the arms.

Discriminating pair, measured both ways:

| form | site | arm in generated parser? |
|---|---|---|
| comment at **column 0** between arms | the `.3.14b` first attempt | **DROPPED** (no caller emitted) |
| comment **indented** to the continuation column | `net_declaration_sv_2017:3585` etc. | **LIVE** (`checked_nettype_identifier`, `wildcard_escape_nettype_identifier`, `interconnect_net_declaration_sv_only` all present) |

**Repo-wide audit — 17 grammars scanned: 8 sites have a comment directly above a `|` arm,
all 8 INDENTED, all 8 verified live; COLUMN-0 occurrences: 0.** So nothing shipped is
damaged. But the hazard is real, silent, and unguarded: it fails in the *accepting*
direction (a dropped arm just narrows the language), which is the failure mode no
pass-rate ever reveals. Routed as `.3.14c`.

## 4. Instrument identity

| input | repo-root-relative path | sha256 |
|---|---|---|
| parse binary | `rust/target/release/parseability_probe` | `780b9f06fe076b54724f7c5aeaf232a910d4224d2e8de5fb780bbef32860aecb` |
| grammar | `grammars/systemverilog.ebnf` | `77a55719dc77831c0242195408ae2d3cfcf1bfeaf44ab42952fbc46036c355ef` |
| generated parser | `generated/systemverilog_parser.rs` | `6bf175d27b02dcf09e67a1b4f9bf04c2c90005f3287031fc2f3721f92669537b` |

Corpus command, identical to the tracked baseline's:
`stimuli/run_external_corpus.sh sv 60 8 0` and `… sv2005 60 8 0`, under
`scripts/run_with_memory_guard.sh --budget-mb 12288`.

## 5. The control matrix (`matrix.sh`, output in `matrix_result.txt`)

**33 rows, 0 misses.** Every whitelist decision, every exclusion, both profile gates and
the `\b` name-prefix guard are covered — tolerated names in module AND class bodies, all
six placement-restricted names, all seven text-hiding names, user/UVM/`__FILE__`/`__LINE__`
macro uses, top-level behaviour unchanged, and `` `undefineall `` accepted under
`sv_2017`/`sv_2023` but REJECTED under `verilog_2005`.

⚠️ **The first version of this harness was VACUOUS and is preserved as a lesson.** It used
`printf '%s'`, which writes a literal `\n`: every case collapsed to one line, everything
rejected, and the entire `expect=REJECT` half "passed" while measuring nothing. The
shipped harness pins a positive AND a negative control and **REFUSES to report** on a miss
([[feedback_instrument_needs_ground_truth]]). The tell was a contradiction with a known-good
reproducer — not a suspicious-looking number.

## 6. Measured effect

### 6a. The 8 routed rows — exactly the designed split

| row | before | after | ruling |
|---|---|---|---|
| sv-tests `5.6.4--compiler-directives-pragma.sv` | REJECT | **PASS** | §22.11 tolerate |
| ispras `ieee-1800-2012/34/34.03.01_01.sv` | REJECT | **PASS** | §22.11 tolerate |
| sv-tests `5.6.4--compiler-directives-debug-line.sv` | REJECT | **PASS** | §22.12 tolerate |
| sv-tests `class_test_48.sv` | REJECT | **PASS** | §22.5.2 tolerate (class body) |
| sv2v `test/core/time.sv` | REJECT | **PASS** | §22.7 tolerate |
| verilator `t_lint_implicit_def_bad.v` | REJECT | **REJECT** | §22.8 — correct, now pinned `must_reject` |
| verilator `t_lint_implicit_func_bad.v` | REJECT | **REJECT** | §22.8 — ditto |
| verilator `t_lint_implicit_type_bad.v` | REJECT | **REJECT** | §22.8 — ditto |

### 6b. Corpus, both lanes (full row list in `delta.txt`)

| lane | files | pass before → after | pass→fail | pass→timeout | pass→crash |
|---|---|---|---|---|---|
| `sv_2017` bulk | 16 336 | 9 712 → **9 720** (+8) | **0** | **0** | **0** |
| `verilog_2005` | 2 459 | 2 180 → **2 181** (+1) | **0** | **0** | **0** |

The `.3.4` LAW holds: the only per-file transition in either lane is `fail → pass`.

| lane | unexplained before → after | rejects-valid | accepts-invalid |
|---|---|---|---|
| `sv_2017` | **360 → 352** | 339 → 331 | **21 → 21 (unchanged)** |
| `verilog_2005` | **75 → 74** | 61 → 60 | **14 → 14 (unchanged)** |

⛔ **ACCEPTS-INVALID IS UNCHANGED IN BOTH LANES. That is the over-acceptance control**, and
it is the number that would have moved had the whitelist been wrong.

### 6c. ⛔ Attribution — the two halves are NOT both yield

Measured separately, by running the adjudicator alone against the **unchanged** baseline
`results.tsv` before any corpus re-run:

| cause | sv_2017 unexplained | what it is |
|---|---|---|
| the `VERILATOR_PINNED` §22.8 pins | 360 → **357** (−3) | **ADJUDICATION CORRECTION** — parse verdict `fail → fail`, unchanged. Never report as burn-down. |
| the grammar change | 357 → **352** (−5) | **REAL YIELD** — 5 files that rejected now parse |

The pin-only run moved exactly those 3 rows and nothing else (`diff` on the full manifest),
and left the `verilog_2005` arm byte-identical. Determinism: re-running the adjudicator
reproduced the manifest byte-identically (`cmp`).

### 6d. Three heals the routing did not predict

Beyond the 5 routed rows, three more files flipped `fail → pass`:

| file | lane effect |
|---|---|
| Surelog `tests/PragmaProtect/pp.top.sv` | stays `deferred:chained_only` |
| Surelog `tests/PragmaProtect/svpp_all/top.sv` | stays `deferred:chained_only` |
| iverilog `ivltests/no_timescale_in_module.v` | stays `deferred:v2005_profile_lane` in the bulk lane; **is the +1 in the v2005 lane** |

⭐ `no_timescale_in_module.v` is the file `.8c.2` pinned `must_accept` on the reading
*"directive placement is unrestricted … the parser must tolerate it"*. That banked pin
predicted this heal and is now **vindicated by the parser** rather than by argument.

Honest reading: these three are genuine parser improvements that move **no bar**, because
their rows are deferred. They are counted as pass-rate, never as burn-down.

## 7. AST invariance — exhaustive, and without the pre-change binary

`ast_invariance_check.py`. The grammar delta adds exactly ONE production emitting exactly
one node shape, so an AST can differ from its pre-change self **only** by containing a node
from that arm. The discriminator is the CONTAINER, measured on the two reproducers in
`repro/`:

```
top level (source_text_item, UNCHANGED)  $/content/Json/source_text[0]
in scope  (the NEW arm)                  $/content/Json/source_text[0]/body/body/body/items[0]
```

Both emit the byte-identical node `{"body": "`timescale 1ns / 1ps", "kind": "compiler_directive"}`
— which is also why the schema does not change.

Population: **all 287** previously-PASSING corpus files containing a whitelisted directive
token at line start (i.e. every file whose AST could possibly move; the other 9 425 cannot
match the arm's regex at all).

```
AST dumped                               : 287 / 287
top-level compiler_directive nodes seen  : 595   (pre-existing path, unchanged)
DEEP (in-scope, NEW-ARM) directive nodes : 0 files
VERDICT: AST byte-invariant on previously-passing files
```

Controls pinned inside the script; it refuses to report if either reproducer's path shape
moves. This is also *a priori* true — a whitelisted directive at an item position REJECTED
before the change, so no previously-passing file can contain one — and the measurement
confirms the argument instead of substituting for it.

## 8. Honest bounds (stated, not silently capped)

1. **Item positions only.** An item-list alternative tolerates a directive *between items*,
   never between two tokens of one statement. That is what the measured population needs
   (all 8 rows parse once directive LINES are removed at item positions —
   `../ch22_directive_split/strip_probe/probe.txt`); a directive mid-statement still rejects.
2. **Two hosts, not all of them.** `non_port_module_item` (module bodies) and `class_item`
   (class bodies) — the two the population needs. Generate / interface / program / package /
   checker bodies are the stated residue → routed to `.3.14d`.
3. **The 2 §34 protected-envelope rows are still rejects** and correctly so — they stop at
   their base64 payload, not at a directive. `.3.13` already reclassified them
   `explained_svpp_protected_envelope`.
4. **`` `pragma `` is tolerated as a line**, not parsed as a structured §22.11 pragma. Its
   *effect* is not modelled; only its presence stops being a parse error.

## 9. Files

| path | what |
|---|---|
| `matrix.sh` / `matrix_result.txt` | the 33-row control matrix + its pinned controls |
| `ast_invariance_check.py` | the AST-invariance instrument (controls pinned, refuses on miss) |
| `repro/min_top.sv`, `repro/min_mod.sv` | the two minimal reproducers, and the invariance controls |
| `before_8rows.txt` | the pre-change verdicts of the 8 routed rows, at `HEAD` |
| `before/`, `after/` | both lanes' characterization + adjudication summaries, whole |
| `delta.txt` | every row that moved in either lane, with parse/expected/class transitions |
