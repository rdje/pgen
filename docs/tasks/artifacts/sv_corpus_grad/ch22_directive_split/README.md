# `SV-CORPUS-GRAD.3.13` — the ch22 family SPLIT BY DIRECTIVE NAME, with an LRM cite per name

Banked 2026-08-08 (session #216). Read-only diagnosis + the spec-derived adjudication
correction it licenses. Every number here was re-measured at `HEAD`; nothing is quoted
from the `.3.13` diagnosis note, and where the two differ the difference is stated.

## 0. Why a split was mandatory before any reclassification

`.3.13`'s banked diagnosis warned that reclassifying rows lowers `unexplained` **without
fixing a parser defect**, which is the exact shape of gaming the tree forbids
(`feedback_corpus_expected_from_spec_not_fix`), and that the ch22 family is **not
homogeneous**. Both held. The family splits cleanly in two, and the two halves take
**opposite verdicts**.

## 1. The population, re-measured (not quoted)

Source: `docs/tasks/artifacts/sv_corpus_grad/rejects_valid_clusters.tsv` at the
post-`.3.12` vintage (372 rejects-valid rows, 368 clustered). Selector = every row whose
stuck-point signature begins with a backtick.

| | rows |
|---|---|
| banked in the `.3.13` diagnosis note | 34 |
| **re-measured here** | **37** |

⚠️ **The banked 34 was 3 rows short, and the correction is recorded rather than silently
absorbed.** The note summed four clusters (`` ` ID ) `` 15 + `` ` ID , `` 11 +
`` ` ID ID `` 6 + `` ` ID NUM `` 2); two further clusters carry the same family —
`` ` ID wire `` (2) and `` ` ID ; `` (1). Suite split re-measured:
iverilog 22 / verilator 8 / sv-tests 4 / sv2v 2 / ispras 1 = **37**.

## 2. The split, by directive name

| `` ` `` name | rows | IEEE 1800-2017 | verdict |
|---|---|---|---|
| `` `__LINE__ `` | 15 | **§22.13** | **MACRO** — adjudicator hole |
| `` `__FILE__ `` | 12 | **§22.13** | **MACRO** — adjudicator hole |
| `` `pragma `` | 4 | §22.11 / §34 | 2 in-scope-directive gap · 2 §34 protected envelope |
| `` `default_nettype `` | 3 | §22.8 | in-scope-directive gap |
| `` `line `` | 1 | §22.12 | in-scope-directive gap |
| `` `undef `` | 1 | §22.5.2 | in-scope-directive gap |
| `` `timescale `` | 1 | §22.7 | in-scope-directive gap |

### 2a. The MACRO half (27 rows) — §22.13, verbatim

> `__FILE__ expands to the name of the current input file, in the form of a string
> literal. […] `__LINE__ expands to the current input line number, in the form of a
> simple decimal number.
> — IEEE 1800-2017 §22.13 (`docs/systemverilog/2017/md/section-22-compiler-directives.md:823`)

A directive **steers** the compiler and leaves the surrounding text parseable; a
predefined text macro **expands to a value**, and the expression around it is not
parseable until it does. `KNOWN_DIRECTIVES` held both names, so `preproc_dependency()`
reported *no* preprocessing dependency for a file whose only dependency was one of them,
and the row fell through to `unexplained` = defect signal. Spec-derived and mechanical:
the fix is the removal of two names from one constant.

### 2b. The IN-SCOPE DIRECTIVE half (8 rows) — a real grammar gap, NOT reclassified

Tool-pinned discriminator (`repro/`, release probe at `HEAD`):

| reproducer | verdict |
|---|---|
| `` `default_nettype none `` **before** `module m;` (`A_directive_top_level.sv`) | **PASS** — AST carries `{"kind":"compiler_directive"}` |
| the same directive **inside** the module (`B_directive_in_scope.sv`) | REJECT, `furthest_position=9` |
| `` `timescale 1ns / 10ps `` inside a module (`D_timescale_in_scope.sv`) | REJECT, `furthest_position=9` |
| `` `undef EVIL `` inside a class (`E_undef_in_class.sv`) | REJECT, `furthest_position=8` |

Same text, two placements, two verdicts ⇒ not a lexical or directive-spelling defect.
**WHERE:** `grammars/systemverilog.ebnf:257` defines
`compiler_directive := trivia /`[^\r\n]*/` and `:241` makes it an alternative of
**`source_text_item` only** — the top level of the file. No in-scope item list
(`module_common_item`, `class_item`, `generate_item`, …) can reach it. `:229` carries the
same alternative **commented out** in `parseable_source_item`.

### 2c. The §34 PROTECTED-ENVELOPE pair (2 rows) — a third verdict, found by measurement

`strip_probe/` answers "would directive tolerance ALONE fix this row?" mechanically: blank
every line whose first non-space character is a backtick (exactly the text a
`compiler_directive` alternative swallows) and re-parse.

```
PASS   `default_nettype  verilator   t_lint_implicit_def_bad.v
PASS   `default_nettype  verilator   t_lint_implicit_func_bad.v
PASS   `default_nettype  verilator   t_lint_implicit_type_bad.v
REJECT `pragma           verilator   t_lint_pragma_protected.v      furthest_position=429
REJECT `pragma           verilator   t_lint_pragma_protected_bad.v  furthest_position=695
PASS   `pragma           sv-tests    5.6.4--compiler-directives-pragma.sv
PASS   `pragma           ispras      ieee-1800-2012/34/34.03.01_01.sv
PASS   `line             sv-tests    5.6.4--compiler-directives-debug-line.sv
PASS   `undef            sv-tests    class_test_48.sv
PASS   `timescale        sv2v        test/core/time.sv
```

8 of 10 PASS ⇒ the in-scope-directive gap is real and bounded. The 2 that still REJECT
stop at their base64 payload (`'==\n…IEV2ZXJ5b25l…'` at both furthest positions), not at a
directive:

> The data_block and key_block pragma expressions introduce the encrypted data or keys and
> will always be found within a begin_protected–end_protected envelope.
> — IEEE 1800-2017 §34.5 (`…/section-34-protected-envelopes.md:263`)

§34.3 has the decrypting tool replace "each decryption envelope with the decrypted source
text from the data_block" *before* compilation, so the raw text is not parser input —
a preprocessing dependency of the same kind as `` `include ``, and strictly stronger (it
hides arbitrary text, including the other dependency markers). Hence the new
`divergence:explained_svpp_protected_envelope` class, tested **first** in
`preproc_dependency()`.

⭐ The detector keys on `key_block`/`data_block`/`digest_block`, not on
`` `pragma protect `` — which is why ispras `34.03.01_01.sv` (`enctype="raw"`, payload is
plain text) correctly stays in the **gap** half while its sibling `34.03.01_02.sv`
(encoded) moves. A blanket `` `pragma protect `` rule would have swallowed 2 real defect
rows.

## 3. Measured effect (the reclassification, in both directions)

Ground-truth control first: rebuilding the manifest **before** any edit reproduced the
tracked artifact **byte-identically** (`cmp`), so every row that moves below is caused by
the edit and by nothing else. Determinism after the edit: `cmp` ×2 byte-identical.

| lane | before | after |
|---|---|---|
| `sv_2017` unexplained | **393** (372 rejects-valid + 21 accepts-invalid) | **360** (339 + 21) |
| `sv_2017` explained | 1 430 | 1 463 |
| `sv_2017` match / deferred | 5 737 / 8 776 | **unchanged** |
| `verilog_2005` unexplained | 76 | **75** |

42 rows changed class; full list in `delta.txt`:

| movement | rows | reading |
|---|---|---|
| `unexplained_rejects_valid` → `explained_svpp_macro_use` | 31 | §22.13 correction |
| `unexplained_rejects_valid` → `explained_svpp_protected_envelope` | 2 | §34 correction |
| `explained_svpp_conditional` → `explained_svpp_macro_use` | 7 | label refinement *inside* explained — no effect on the bar |
| `explained_svpp_macro_use` / `_include` → `_protected_envelope` | 2 | ditto |

**Net effect on the graduation bar: −33 sv_2017, −1 v2005. This is an ADJUDICATION
CORRECTION and must never be reported as burn-down yield.** Zero parser, grammar,
generator or generated-artifact bytes were touched by it.

## 4. ⚠️ The honest cost of any explained-class label (4 rows)

An explained label says *this file is not honest parser input*; it does **not** say the
parser handled it. Of the 33 rows leaving `unexplained`, **29 were stuck exactly at the
`` `__FILE__ ``/`` `__LINE__ ``/protected-envelope token**. The other **4** carry a
`__FILE__`/`__LINE__` use *somewhere else in the file* and were stuck at an unrelated
construct, so the correct label now hides a genuine stuck point:

| suite | file | stuck at | family |
|---|---|---|---|
| iverilog | `ivltests/br_gh782b.v` | `/* comment */ 1 /* comment */` | comment/number lexical |
| iverilog | `ivltests/sv_type_identifier_package_name.v` | `T::VALUE !== 23` | scoped-name expression |
| verilator | `t/t_randomize_within_func.v` | `randomize(m_2) with {…}` | F3 constraint/randomize |
| verilator | `t/t_vams_basic.v` | `wreal wr;` | Verilog-AMS (likely lane-external) |

Recorded here so the signal is routed, never lost — `feedback_every_finding_must_be_fixed_not_logged`:
routing decides WHEN, never WHETHER. These 4 are not re-adjudicable back into the bar
(the files genuinely need expansion); they are worked as **crafted minimal cases** in the
`.9` gap loop, where the construct — not the vendored file — is the unit.

## 5. Files

| path | what |
|---|---|
| `repro/` | the 5 minimal reproducers behind §2b |
| `strip_probe/probe.txt` + inputs | the "would directive tolerance alone fix it?" measurement |
| `before/summary*.md`, `after/summary*.md` | the two manifest summaries, whole |
| `delta.txt` | every row that changed class, plus the §4 stuck-point audit |

Large intermediates (the two 3 MB manifests) were deleted after the diff was taken; both
are reproducible in ~1 s by `python3 stimuli/sv/adjudicate_external_corpus.py`.
