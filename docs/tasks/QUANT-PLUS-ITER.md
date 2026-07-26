# QUANT-PLUS-ITER: a `+` over a rule reference iterates ONCE in the scratch shape — and VHDL's identical idiom does not

## Metadata

- Tree ID: `QUANT-PLUS-ITER`
- Status: `active` (opened 2026-07-26, session #211; **`.1` RESOLVED session #212** —
  the tree's premise is retired, `.2`/`.3` remain)
- Family / slice-id prefix: `PGEN-QUANT-PLUS-ITER-<NNNN>`
- Created: `2026-07-26`
- Owner: repo-local workflow
- Opened by: `LANG-CAPABILITY-AUDIT.8`, which surfaced this **while probing something
  else** and deliberately did NOT absorb it (recovery was proven not to be the cause).

## ⭐⭐⭐ RESOLVED (session #212, leaf `.1`) — READ THIS BEFORE THE CHARTER BELOW

**`+` is not defective. The quantifier was never executed.**

PGEN's canonical entry rule is `rule_order[0]` — the rule **defined first in the
file**. The probe grammar below defines `stmt` *above* `scratch`, so the generated
parser was entered at `stmt`; `stmt` consumed exactly one `a;` and the parse then
failed on the remainder, at position 2, every time. The `--entry-rule scratch` flip
(one binary, one grammar, one variable) accepts `a;`, `a;a;`, `a;b;` and `b;a;b;a;`.

Everything in the "What was MEASURED" section below is **accurate and was correctly
reasoned** — it is retained verbatim as the record. It is also, in its entirety,
about the machinery of a loop that never ran. The question never asked was
***"is the rule I am testing the rule being run?"***

The residual work is real and is NOT about `+`:
- **`.2`** — no instrument says which rule became the start symbol, and the start
  symbol **cannot be declared in the EBNF at all** (measured: 0 hits across the
  meta-grammar, the directive registry, and every tracked grammar).
- **`.3`** — the trace-changes-the-engine caveat, still worth banking.

Full diagnosis, evidence and the flip table: leaf `.1` below.

---

## Why this tree exists

While building the end-to-end recovery probe for `LANG-CAPABILITY-AUDIT.8`, a
minimal scratch-slot grammar whose entry rule quantifies a rule reference
**consumed exactly one occurrence** and then stopped. This is a potential
parser-correctness defect in the most basic construct PGEN has, so it gets its own
tree rather than a footnote.

⛔ **It is NOT "PGEN's `+` is broken".** The single most important measurement in
this file is the VHDL control, which shows the same idiom working at gate scale.
Whoever takes this must start from that tension, not from the alarming half.

## What was MEASURED (all tool-backed, session #211)

### The failing shape

```ebnf
stmt := "a" ";"
      | "b" ";"

scratch := stmt+
```

| input | verdict |
|---|---|
| `a;` | ACCEPT |
| `a;a;` | ⛔ reject — *did not consume full input at position 2* |
| `a;b;` | ⛔ reject — *at position 2* |
| `b;a;b;a;` | ⛔ reject — *at position 2* |

Position 2 = exactly one `stmt`. The quantifier stops after one iteration.

### Four shape variants — the behaviour does NOT move

| variant | result |
|---|---|
| `scratch := stmt+` (quantifier IS the entry rule's whole body) | one iteration |
| `list := stmt+` / `scratch := list` (one level below entry) | one iteration |
| `stmt` **single-branch** (`stmt := "a" ";"`) | one iteration |
| `scratch := stmt+ "!"` (quantifier has a **follower**) | one iteration — and even `a;!` rejects at position 2 |

⇒ independent of entry-rule position, of nesting depth, and of branch count.

### It is NOT caused by `@recover` (the control that matters for `.8`)

The same grammar with **all recovery annotations removed** rejects `a;a;` at
position 2 **identically**. Recovery is exonerated — which is why
`LANG-CAPABILITY-AUDIT.8` could close.

### It is NOT a cascade-vs-memoized duality break

`bare_parse` (`generated/*_parser.rs:295`) selects the **cascade** engine when
tracing is off and the **memoized** engine when a logger is enabled — so an
untraced run and a traced run execute *different code*. Both were compared
directly on the same inputs:

```text
input      CASCADE (untraced)     MEMOIZED (traced)
a;!        reject                 reject
a;a;!      reject                 reject
a;a;a;!    reject                 reject
```

Both engines agree. (⚠️ **Toolbox note worth propagating regardless:** tracing
changes which engine runs. Anyone diagnosing with `PGEN_TRACE_VERBOSITY` is not
necessarily observing the path that produced the untraced verdict. This trap cost
real time in session #211 and belongs in `TOOLBOX.md`.)

### Things ruled OUT by reading the generated source

- **Memo key** — `memoized_call` keys on `(rule_id, self.position)` captured at
  ENTRY (`:4057`); correct. The trace line *"Memoized successful result for rule 0
  at position 2"* prints `self.position` (the END) and is merely **cosmetically
  misleading**, not evidence of a mis-keyed memo.
- **`try_parse` backtracking** — the `Err` arm restores `self.position = saved_pos`
  correctly.
- **Codegen structure** — the emitted `+` loop is present and well-formed
  (`SAFETY_LIMIT`, zero-length guard, `results.push`, `iteration_count`), and for
  `stmt+ "!"` the **follower is emitted too** (`match_lit_ascii("!")`) inside a
  proper `ParseContent::Sequence`. The raw AST is also correct:
  `[["rule","scratch"],["rule_reference","stmt"],["operator","+"],["quoted_string","!"]]`.
- **Layout sensitivity** — adding whitespace between items (`a; a; !`) changes nothing.

### ⭐⭐ THE CONTROL THAT REFRAMES EVERYTHING: VHDL does this correctly

`grammars/vhdl.ebnf:208` — `record_type_definition := kw_record record_element_declaration+ kw_end kw_record`
— is a bare rule-reference `+`, and the **shipped** VHDL parser iterates it fine:

```text
v1 (1 record element)   rc=0  passed for grammar
v2 (3 record elements)  rc=0  passed for grammar
```

Same for `view_element+` (`:133`) and `case_statement_alternative+` (`:320`).

⇒ **the defect is shape-specific, and the distinguishing factor is NOT YET
IDENTIFIED.** Candidate differences between the two, none confirmed:
the scratch grammar declares no trivia/layout rule at all; VHDL's quantified rule is
reached from a much deeper context; VHDL's followers are keyword tokens rather than
a bare literal; the scratch grammar is whitespace-**sensitive** by inference while
VHDL is not (`layout_sensitivity().terminals`, which also gates FIRST-set predictive
dispatch at `ast_based_generator.rs:4287`).

Note also that `grammars/json.ebnf` expresses repetition by **recursion**
(`members := pair /,/ members`), not by `+` — so JSON's healthy iteration is not
evidence about `+` either way.

## Leaves

### `.1` — Identify the distinguishing factor between the scratch shape and the VHDL shape (`done`)

> **Slice `PGEN-QUANT-PLUS-ITER-0001` (session #212, 2026-07-26) — docs-only, read-only.**
> No grammar, Rust, generated artifact, contract, release, schema or ledger byte
> changed. The scratch slot was loaded for the flip and **restored to its committed
> fixture** (`git checkout`), with `generated/scratch_parser.rs` regenerated from
> the restored fixture and both binaries rebuilt.

#### ⭐⭐⭐ THE DISTINGUISHING FACTOR IS NAMED — AND IT IS NOT A PROPERTY OF `+` AT ALL

**The failing grammar's quantifier was never executed. The parse never entered the
rule that contains it.**

PGEN's canonical entry rule is `rule_order[0]` — **the rule DEFINED FIRST in the
file** (`rust/src/ast_pipeline/ast_based_generator.rs:616-621`, verbatim):

```rust
let entry_rule = self
    .entry_rule
    .as_ref()
    .map(|s| s.clone())
    .or_else(|| rule_order.first().cloned())
    .ok_or_else(|| anyhow::anyhow!("No entry rule found"))?;
```

The charter's probe grammar defines `stmt` **above** `scratch`. So the generated
parser's entry is `stmt`, and every row of the failing table is one `stmt`
consuming exactly one `a;`/`b;` and the parse then failing on the unconsumed
remainder — at position 2, every time.

⛔ **`+` IS CORRECT.** The four "shape variants" never moved the behaviour because
none of them changed the variable that matters: **all four kept `stmt` defined above
the quantifying rule**, so all four were entered at `stmt`. The loudest row — the
follower case, where even `a;!` rejects at position 2 — is the same single fact:
the parse ran `stmt` alone, consumed `a;`, and stopped; the `"!"` lives in a rule
that was never entered.

#### THE FLIP (the tree's Acceptance Criterion: demonstrated, not hypothesized)

ONE binary, ONE grammar in the slot, ONE variable changed (`--entry-rule`):

| input | DEFAULT entry (= `stmt`) | `--entry-rule scratch` |
|---|---|---|
| `a;` | ACCEPT | ACCEPT |
| `a;a;` | ⛔ REJECT @ position 2 | ✅ **ACCEPT** |
| `a;b;` | ⛔ REJECT @ position 2 | ✅ **ACCEPT** |
| `b;a;b;a;` | ⛔ REJECT @ position 2 | ✅ **ACCEPT** |

The DEFAULT column reproduces the charter's failing table row for row; the second
column is the flip — `+` iterates 1, 2 and 4 occurrences correctly the moment the
parse actually starts at the rule that owns it. (Exit codes captured from the binary
itself — an earlier harness of mine read `tail`'s status through a pipeline and
reported a uniform "ACCEPT"; corrected before any conclusion was drawn.)

#### CORROBORATION AT THE CODEGEN LAYER (cheap, no rebuild)

Same two rules, **only the definition ORDER differs**:

| variant | emitted dispatch | emitted alias |
|---|---|---|
| `stmt` defined first | `self.parse_stmt()` | `pub fn parse_full_stmt` |
| `scratch` defined first | `self.parse_scratch()` | `pub fn parse_full_scratch` |

and the generator's own trace line on the canonical `make focus_scratch` path
(`ast_based_generator.rs:623`, DBG verbosity, **line 275,924** of the generation log):

```
[PGEN][DBG] 🧠 [src/ast_pipeline/ast_based_generator.rs:623]   Entry rule determined: 'stmt'
```

#### ⭐ THE VHDL TENSION IS RESOLVED, NOT MERELY RESTATED

`grammars/vhdl.ebnf:11` defines `vhdl_file := design_unit*` **first**, so
`record_element_declaration+` (`:208`) is genuinely reached and genuinely iterates
(re-verified at HEAD on a 3-element record, rc 0). The committed scratch fixture
likewise defines `scratch` first — **which is exactly why the slot's own integration
test never exposed this.** There is no shape-specificity and no second engine
behaviour: one rule about definition order explains both sides.

⇒ The charter's six exonerations (`@recover`, cascade-vs-memoized duality, memo key,
`try_parse` backtracking, emitted loop structure, layout) were all **correct and all
irrelevant** — they interrogated the machinery of a loop that never ran. The question
never asked was *"is the rule I am testing the rule being run?"*

#### ⚠️ THIS WAS A TOOLBOX-**USE** GAP, NOT A TOOLBOX GAP

Step 0 of the standing 3-step `UNKNOWN` protocol answers it **in one command, with
no flags**, on the slot-loaded grammar:

```
CERTIFICATE-COVERAGE: grammar='scratch' entry='stmt' samples=40 total=2
    proof=0 witness=1 UNKNOWN=1 fully_certified=false
  WARNING plannable-rule reach pass: 1 UNKNOWN rules have NO reach path from the
    entry (dead-rule candidates — adjudicate via the linter): ["scratch"]
  UNKNOWN rules (1 of 1 shown): ["scratch"]
```

It names the entry on its headline **and** names the orphan. Recorded as a
use-failure, not softened into a tool gap.

#### ⛔ BUT A REAL, MEASURED DEFECT REMAINS — AND IT IS AN INSTRUMENT DISAGREEMENT

That warning's own advice is **"adjudicate via the linter"** — and the linter then
declares the grammar clean:

| surface (default verbosity) | says |
|---|---|
| `--lint-grammar` | exit **0**, `unreachable_rules=0`, entry rule **never named** |
| `--generate-parser` | 2 lines of output, entry rule **never named** |
| `--report-certificate-coverage` | `entry='stmt'`, `scratch` has **NO reach path** |

`scratch` escapes the unreachable check **by construction**: it is referenced by
nothing, and `reachable_rules` (`grammar_wellformedness.rs:377-389`) treats every
unreferenced rule as a **ROOT**. That is `LANG-CAPABILITY-AUDIT.2`'s named blind
spot — *a declared-and-never-referenced production can never be flagged* — here with
a concrete measured cost: **a whole task-tree was opened on a phantom `+` bug.**
The `ANNOTATION-PLACEMENT` family law applies verbatim: **a check that cannot see a
defect class must SAY SO, not return green** — and here it must not hand off to a
blind instrument as though it were the authority.

#### ⭐⭐⭐ THE FINDING THAT OUTRANKS THE BUG — THE START SYMBOL IS NOT EBNF-DECLARABLE

**PRIOR ART search** (`DESIGN-PRIOR-ART` authority order), all three measured **ZERO**:

| authority | searched for | hits |
|---|---|---|
| `grammars/ebnf.ebnf` | `entry` / `start_rule` / `start_symbol` / `@start` | **0** |
| `semantic_directive_registry.rs` | a registered entry/start directive | **0** |
| every tracked `grammars/*.ebnf` | `@entry` / `@start` | **0** |

⇒ **PGEN has no way to declare a grammar's start symbol in the EBNF.** The single
most fundamental user-controllable property of a grammar is steered by an *implicit
positional convention* (first rule defined wins), overridable only out-of-band by a
CLI flag, and reported by no default-verbosity surface. `parser_registry.rs` carries
no `entry_rule` either, and `make focus_scratch` passes none.

This is a direct instance of the director's standing #208 directive —
*"EVERY user-controllable feature MUST be declared IN THE EBNF … no runtime flag, no
engine table"* ([[project_ebnf_is_single_source_of_truth]]) — and the slot's own
documented contract (*"the entry rule must be named `scratch`"*,
`grammars/scratch/scratch.ebnf` header) is **unenforced**: `focus_scratch` happily
generated a parser entered at `stmt`. **Routed to `.2` as a director-visible surface
call, NOT implemented here.**

#### ⚠️ Ops note re-confirmed (not new, but it bit again)

- `make focus_scratch` **overwrites `rust/target/debug/ast_pipeline` with a
  SINGLE-feature build** — the standing dual-feature trap, hit twice this leaf. The
  driver now carries its own `--report-feature-surface` tripwire and refuses with the
  exact rebuild command (verified firing).
- The release `parseability_probe` relink **breached the default 12,288 MB guard
  budget at 12,362 MB** (an earlier identical build peaked at 12,101 MB — it sits on
  the line). Use the banked **16,384 MB** heavy-run budget.

#### Verification

- Driver `docs/tasks/artifacts/quant_plus_iter/run_entry_rule_probes.sh`:
  **16 declared-verdict cases, exit 0, 0 divergences, BYTE-IDENTICAL on re-run**
  (`cmp` clean). It installs nothing and rebuilds nothing.
- Capture `entry_rule_probes_capture.txt` (driver output) +
  `entry_rule_flip_capture.txt` (the slot-loaded flip + cert-coverage transcript +
  the full reproduction recipe).
- ⛔ **Honest bound, no silent cap:** the cert-coverage section is *not* asserted
  mechanically by the driver, because certificate-coverage verifies witnesses through
  the REAL generated parser and therefore only runs for a **registered** grammar name,
  which a temp-dir probe grammar never is. The driver asserts **that refusal** so the
  reason is itself mechanized, and points at the capture for the slot-loaded run.
- Tree clean; scratch fixture restored and its artifact regenerated from it.

#### `.1`'s original charter (kept for the record — and one of its steers was WRONG)

- **Start from the tension, not from the failure.** The deliverable is the ONE
  property that differs and flips the behaviour — bisect the scratch grammar
  *towards* the VHDL shape (add a layout rule; wrap the quantified rule deeper; swap
  the follower literal for a keyword-ish token) until it starts iterating, one
  variable at a time.
- ⛔ Do NOT begin by reading the `+` codegen again — `.0` already read it and it is
  structurally correct. The cause is upstream of the loop or in a guard around it.
- Suspect to check FIRST (cheapest, and the only one with a known conditional
  emission): the FIRST-set predictive dispatch prune guard, emitted only when
  `top_level && self.layout_sensitivity().terminals` — a condition the tiny
  whitespace-insensitive scratch grammar satisfies and VHDL does not.
- ⚠️ Each variant costs a `make -C rust focus_scratch` + `parseability_probe` rebuild
  (~3.5 min). Batch the variants into as few rebuilds as the question allows.

⛔ **The named FIRST suspect was measurably FALSE and would have cost a rebuild
each.** `layout_sensitivity()` is compiled *only* from a grammar-level
`@whitespace_sensitive:` directive and otherwise returns
`LayoutSensitivity::default()` (all facets `false`, `semantic_runtime.rs:204-217`).
**Neither** grammar declares it — `grep -c whitespace_sensitive` is **0** in both
`grammars/vhdl.ebnf` and `grammars/scratch/scratch.ebnf` — so the FIRST-set prune
guard is emitted for *neither*, and the charter's "the tiny grammar satisfies it and
VHDL does not" was inverted-and-irrelevant. The claim was a **plausible reading of
source, never measured** — the same failure mode `feedback_read_prior_art_before_designing`
names, and the reason `.1` opened by *measuring* the emitted dispatch rather than
following the steer. ⭐ Two seconds of `grep` retired a hypothesis budgeted at
multiple 19-minute rebuilds.

### `.2` — Make the start symbol visible, and adjudicate whether it becomes EBNF-declarable (`todo`, **re-scoped by `.1`**)

`.1` unblocked this and changed its subject. There is **no `+` bug to fix**. Two
distinct deliverables remain, and the second needs a director call.

**(a) VISIBILITY — the cheap, zero-risk half.** No default-verbosity surface names
the resolved entry rule, and the one instrument that does see the problem
(`--report-certificate-coverage`) hands off to `--lint-grammar`, which reports
`unreachable_rules=0` and exits 0. Candidates, in fix-hierarchy order:

- `--lint-grammar` states the resolved entry rule by name (one informational line).
  Purely additive, no verdict changes, would have made this instantly visible.
- The **orphan report** already routed to `LANG-CAPABILITY-AUDIT.2`/`.4` (T-9:
  *"N rules are unreferenced roots"*) — the general instrument for this class. `.1`
  is now its second independent motivation (the first was `ebnf.ebnf:18`'s dangling
  include). ⛔ Keep it **separate from** the hard `unreachable_rules` error: the
  unreferenced-rule-is-a-root rule is CORRECT design and protects SV's
  `library_text` / `*_multi_entry_root` secondary entries from false rejection.
- ⚠️ Whatever is built must not turn a legitimate multi-entry grammar red. The
  measured safe shape is *informational*, not an error class.

**(b) DECLARABILITY — the director-visible surface call.** Measured in `.1`: the
start symbol cannot be declared in the EBNF (0 hits in `grammars/ebnf.ebnf`, 0 in
`semantic_directive_registry.rs`, 0 across every tracked grammar). It is steered by
an implicit positional convention and overridable only by an out-of-band CLI flag —
squarely against the standing #208 directive that *every user-controllable feature
MUST be declared in the EBNF*. ⛔ **Do not implement a new annotation on the strength
of this leaf**: `DESIGN-PRIOR-ART` requires the surface be agreed first, and this one
interacts with SV's multi-entry model (`--cert-union-config`, `parse_full_from`,
`library_text`) which already treats "the entry" as plural. The deliverable here is
the *adjudication*, with the zero-cost/neutrality acceptance test applied
(compile-time only; non-users pay ZERO).

**(c) The scratch slot's own contract is unenforced.** Its header says *"the entry
rule must be named `scratch`"*; `make focus_scratch` generated a parser entered at
`stmt` without a word. Cheapest possible guard, independent of (a)/(b): have
`focus_scratch` assert `rule_order[0] == "scratch"` and fail loudly otherwise — the
toolbox's own probe surface should not be able to silently answer a different
question than the one asked.

### `.3` — Record the trace-changes-the-engine trap in `TOOLBOX.md` (`todo`)

- Independent of the outcome above and useful immediately: `bare_parse` means
  enabling a trace switches the parser from the cascade engine to the memoized one.
  A diagnostic session can therefore observe a path that did not produce the verdict
  being investigated. This belongs in the toolbox as a standing caveat.

## Acceptance Criteria (tree)

- ✅ **MET by `.1`** — the distinguishing factor is **named** (`rule_order[0]`: the
  rule defined first in the file) and **demonstrated by a flip**, not hypothesized:
  `--entry-rule scratch` turns every REJECT into an ACCEPT in one binary.
- ⏳ Either a fix with before→after evidence, or a diagnostic that refuses the shape.
  — re-scoped by `.1`: there is no `+` bug to fix; the open half is the missing
  *entry-rule diagnostic* and the *EBNF-declarability* call (`.2`).
- ✅ No shipped parser's behaviour changes without byte-identity evidence — `.1`
  changed no shipped parser at all (docs-only; the scratch slot was restored to its
  committed fixture and its artifact regenerated from it).

## Evidence

- `docs/tasks/artifacts/quant_plus_iter/run_entry_rule_probes.sh` — the re-runnable
  declared-verdict driver (**13 cases, exit 0, 0 divergences, byte-identical on
  re-run**). Installs nothing, rebuilds nothing; carries its own dual-feature
  tripwire.
- `docs/tasks/artifacts/quant_plus_iter/entry_rule_probes_capture.txt` — driver output.
- `docs/tasks/artifacts/quant_plus_iter/entry_rule_flip_capture.txt` — the
  slot-loaded behavioural flip, the certificate-coverage transcript, the codegen
  mechanism, and the full reproduction recipe.
- Session #211's measurements are reproduced inline above; every one was taken
  through the PARSE-HARNESS scratch slot (`grammars/scratch/scratch.ebnf`, restored
  to its committed fixture afterwards) and the shipped `vhdl` registry parser.
- Companion context: `docs/tasks/LANG-CAPABILITY-AUDIT.md` leaves `.2` (the
  unreferenced-root blind spot this lands in) and `.8` (which opened this tree).

## Commit log

| slice | leaf | commit |
|---|---|---|
| `PGEN-QUANT-PLUS-ITER-0001` | `.1` | see git log (session #212, 2026-07-26) |
