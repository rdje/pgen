# QUANT-PLUS-ITER: a `+` over a rule reference iterates ONCE in the scratch shape — and VHDL's identical idiom does not

> **RESOLVED (session #212): `+` was never defective — the quantifier's rule was never
> the entry rule.** The title states the charter's symptom, kept for continuity; read
> the banner below for the actual finding.

## Metadata

- Tree ID: `QUANT-PLUS-ITER`
- Status: `active` (opened 2026-07-26 session #211; **`.1` + `.3` DONE session #212** —
  the tree's premise is retired; **`.2` is the only open leaf**, and its second half
  carries a director-visible surface call)
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
failed on the remainder, at position 2, every time. Starting the parse at `scratch`
instead accepts `a;`, `a;a;`, `a;b;` and `b;a;b;a;` — and a three-arm control pins the
start symbol as the *sole* cause, with the execution graph held fixed.

Everything in the "What was MEASURED" section below is **accurate and was correctly
reasoned** — it is retained verbatim as the record. It is also, in its entirety,
about the machinery of a loop that never ran. The question never asked was
***"is the rule I am testing the rule being run?"***

The residual work is real and is NOT about `+`:
- **`.2`** — no instrument says which rule became the start symbol, and the start
  symbol **cannot be declared in the EBNF at all** (measured: 0 hits across the
  meta-grammar, the directive registry, and every tracked grammar).
- ✅ **`.3` DONE** — the trace-changes-the-engine caveat is banked in `TOOLBOX.md`, and
  measurement found a **second** routing hazard the charter did not know about:
  `--entry-rule` switches the engine too (all ten generated parsers).

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

##### ⚠️ THE ENGINE CONFOUND IN THAT TABLE — FOUND, MEASURED, ELIMINATED (same session)

The two-arm flip above varied **two** things, not one. While banking `.3`,
`parse_from` was measured to set `self.bare_parse = false;` **unconditionally** —
verified across **all ten** generated parsers (1/1 each) — so `--entry-rule` always
takes the PROTOCOL graph while a default `--parse` takes the fused `cascade_*` graph.
Recorded as a real weakness in the evidence as first committed, not quietly repaired.

The missing control, on one freshly built binary:

| input | arm1 default (bare, `stmt`) | arm2 `--entry-rule stmt` (protocol, `stmt`) | arm3 `--entry-rule scratch` (protocol, `scratch`) |
|---|---|---|---|
| `a;` | rc 0 ACCEPT | rc 0 ACCEPT | rc 0 ACCEPT |
| `a;a;` | rc 1 @ position 2 | rc 1 @ position 2 | ✅ rc 0 ACCEPT |
| `a;b;` | rc 1 @ position 2 | rc 1 @ position 2 | ✅ rc 0 ACCEPT |
| `b;a;b;a;` | rc 1 @ position 2 | rc 1 @ position 2 | ✅ rc 0 ACCEPT |

- **arm1 vs arm2** — same start symbol, **different engine graph** ⇒ verdicts
  **identical** ⇒ the engine graph is *not* the discriminator.
- **arm2 vs arm3** — **same engine graph**, different start symbol ⇒ REJECT flips to
  ACCEPT ⇒ **the start symbol is isolated as the sole cause.**

⇒ the `.1` conclusion is **unchanged and now rests on a controlled comparison.** It
also supplies `.3`'s measured data point: the two graphs **agreed** on all four
inputs, so the routing hazard is a *reasoning* hazard (you may not be observing the
path that produced the verdict), not a known correctness difference.

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

### `.2` — declare the entry rule in the EBNF: `@entry` (increment 1) (**step A `done`**; steps B/C open)

> ⭐⭐⭐ **DIRECTOR-APPROVED, 2026-07-26 session #212.** Raised by the director on
> reading `.1` (verbatim: *"ok do you need a way to explicitly indicate the entry rule
> in the EBNF file ?"* → *"If you need such feature, please explain me the rationale
> for that"* → ⭐ *"Aah maybe is because, rules in an EBNF shall not be ordered, is
> that right ?"*), then decided: **"Just increment 1 for now. Agreed with `@entry`"**.
> ⇒ scope is **exactly one `@entry` per grammar**; multi-entry is explicitly OUT.

#### The rationale, as accepted

⭐ **The director's own formulation is the governing one: in EBNF a grammar is a SET
of productions — rule order is presentation, not semantics.** PGEN honours that
everywhere except one hidden place: the start symbol is `rule_order[0]`. So swapping
two definitions — a pure formatting change by every EBNF convention — silently
changes the accepted language, and **both versions lint clean with exit 0** (measured,
`.1`). A hidden exception to an otherwise-clean invariant is the worst kind: everyone
correctly generalises "order doesn't matter" and is bitten exactly once. It cost
session #211 an entire task-tree.

Supporting grounds (all measured in `.1`):
1. **#208 compliance.** *"every user-controllable feature MUST be declared IN THE
   EBNF — no runtime flag, no engine table"*. The start symbol is today controlled by
   file position + the `--entry-rule` CLI flag. This is not a new capability; it is an
   unaudited violation of a directive already in force.
2. **It makes a diagnostic possible at all.** Today the linter cannot warn about a
   mis-rooted grammar *even in principle* — the first rule **is** the entry by
   definition, so there is no declared intent to disagree with.
3. **Zero cost, parser-neutral** — compile-time only (see the acceptance test below).

#### PRIOR ART

Searched in the `DESIGN-PRIOR-ART` authority order before proposing any surface
([[feedback_read_prior_art_before_designing]]); every claim below is a measurement
taken this session, not a recollection.

| # | Authority | Searched for | Result |
|---|---|---|---|
| 1 | `grammars/ebnf.ebnf` | `entry` / `start` / `root` / `goal` / `axiom` / `main` / `top` | **0 hits** — no surface exists for an author to write. |
| 1b | `grammars/ebnf.ebnf` | `whitespace_sensitive` / `default_profile` / `profile_alias` | **0 hits each** ⇒ ⭐ grammar-level directive syntax is **GENERIC**; a new directive needs **NO meta-grammar change**. |
| 2 | `rust/src/.../semantic_directive_registry.rs` | a registered entry/start directive | **0** — but the **family to join** is registered and live: `whitespace_sensitive`, `default_profile`, `profile_alias`. |
| 3 | `docs/decisions/` | entry rule / start symbol / multi-entry | `project_sv_full_certification_via_multi_entry.md` — entries are already modelled as `(entry, profile)` pairs via `--cert-union-config`, and `parse_from(entry)` / `parse_full_from(entry)` dispatch already ships (`GRAMMAR-WELLFORMED.H.12.8.4.3`). |
| 4 | every tracked `grammars/*.ebnf` | `@entry` / `@start` | **0 uses.** |
| 6 | `grammars/semantic_annotation.ebnf` | the normative annotation-language spec | ⭐ **DIRECTOR-POINTED** — I had stopped at `ebnf.ebnf`, found `semantic_annotation` referenced-but-undefined, and wrongly concluded the meta-grammar *"cannot tell me what payload syntax is legal"*. The spec is one file over (20,206 B) and settles the spelling outright (`":" annotation_value` is **mandatory**). |
| 7 | `grammars/builtin_semantic_annotation.ebnf` | the bootstrap-safe annotation contract | ⭐ **DIRECTOR-POINTED** — 23 rules, and it is the composition-safe twin. |
| 5 | `rust/src/parser_registry.rs` | `entry_rule` | **0** — registered grammars carry no entry config either. |

⭐⭐ **THE FIND — a shipped grammar already documents the intended fix, and it is the
WRONG SHAPE.** `grammars/systemverilog.ebnf` header, section (2), verbatim:

> *"`sv_multi_entry_root` — synthetic multi-entry root … a closure-gate analysis aid
> that lets the gate's single-entry static reachability analyzer see all three real
> top-level entries at once. **The runtime parser does NOT use it as an entry.**
> Long-term tooling fix: extend `ast_pipeline --entry-rule` and the closure-gate
> contract to accept a list of entries; **once that lands, `sv_multi_entry_root` can
> be removed.**"*

⇒ a shipped grammar carries a **synthetic rule that exists only to work around the
single-entry analyzer** (measured: **1** `rust/src` reference vs **7** gate/contract
references). Its recorded remedy is **CLI-and-contract shaped — it predates #208** and
would put the entry set in the engine, which #208 now forbids. ⭐ **This proposal
supersedes that plan in the EBNF direction and eventually lets `sv_multi_entry_root`
be deleted, which its own author already anticipated** — but only at increment 2
(multi-entry), which the director has explicitly deferred.

⛔ **Conclusion of the search: no existing surface covers this.** The `@entry` name is
chosen to match PGEN's OWN established vocabulary rather than bison's `%start` — the
flag is `--entry-rule`, the field is `entry_rule`, the API is `parse_full_from(entry)`,
certificate coverage prints `entry='…'`, and the books say "canonical entry".

#### Design (increment 1) — ⭐ CORRECTED BY THE DIRECTOR MID-DESIGN

⛔ **My first proposal was `@entry: <rule_name>` as a grammar-level directive. The
director rejected the SHAPE** (verbatim: *"Is `@entry: name` is appropriate ? because
usually semantic annotations occurs before or inside a rule."*) — and the measurement
backs it up. PGEN has **two** annotation patterns:

| pattern | members | attachment |
|---|---|---|
| **rule-level** (dominant) | `@profiles`, `@emit_fact`, `@recover`, `@precedence`, `@transform`, `@priority`, … | the annotation steers **the rule it precedes** |
| **grammar-level** (only 3) | `@whitespace_sensitive`, `@default_profile`, `@profile_alias` | measured: all three `flat_map` over every rule and **discard the attachment key** (`(_, list)`); the book tells authors to write them *"directly above a rule — conventionally the entry rule"*, so attachment is syntactically required but semantically **ignored** |

*"This rule is the start symbol"* is intrinsically a statement **about a rule**, so it
belongs to the dominant pattern. ⇒ **`@entry` ATTACHES TO THE ENTRY RULE.** The
attachment name is available for free: `Annotations.semantic_annotations` is
`HashMap<String, Vec<SemanticAnnotation>>` — the rule name **is the map key**; this
directive simply stops discarding the key the other three throw away.

⭐ **The corrected shape is strictly better, in ways the payload form was not:**
1. **An entire error class vanishes by construction** — you cannot name a rule that
   does not exist, so the "declared-but-undefined entry ⇒ hard error" check I had
   designed becomes *unrepresentable* rather than merely checked.
2. **DRY + rename-safe** — the rule name appears once; renaming cannot desynchronize.
3. No typo can silently select a different *existing* rule.

##### Spelling — settled by measurement, not preference

The normative annotation grammar (`grammars/semantic_annotation.ebnf:16`) makes the
payload **mandatory**:

```ebnf
semantic_annotation := "@" /\s*/ annotation_name /\s*/ ":" /\s*/ annotation_value
```

and the frontend agrees — but *silently*, which is the important part. Measured on the
real frontend (`--emit-raw-ast-json`, three probe grammars):

| form | `--lint-grammar` | recorded in the raw AST? |
|---|---|---|
| bare `@entry` | **exit 0, clean** | ⛔ **0 mentions — SILENTLY DROPPED** |
| `@entry: true` | exit 0, clean | ✅ captured |
| `@entry: program` | exit 0, clean | ✅ captured |

⇒ **bare `@entry` is not a viable spelling** — it lints clean and vanishes. ⚠️ That is
a NEW measured cell of the `ANNOTATION-PLACEMENT` silent-drop family (not mid-sequence
this time, but **payload-less**) — routed there, not absorbed here.

⇒ ✅ **CANONICAL SPELLING: `@entry: true`**, attached to the entry rule — exactly the
shape of the nearest existing analogue, `@recover: true`.

```ebnf
statement := "a" ";"          # helper rules may live wherever reads best

@entry: true
program := statement+         # ← "this rule is the entry"
```

<!-- superseded draft retained for the record:
```ebnf
@entry: program

statement := "a" ";"
program   := statement+
```
-->

| situation | behaviour |
|---|---|
| directive **absent** | `rule_order[0]` — today's behaviour exactly; generated parser **byte-identical** |
| directive **present** | that rule is the entry, wherever it sits in the file |
| names a rule the grammar does not define | **hard error at load**, naming the rule (the `undefined_references` class) |
| declared **twice with the same payload** | allowed (mirrors `@default_profile`/`@profile_alias`) |
| declared **twice with different payloads** | **hard error** — declare the entry exactly once |
| `--entry-rule` **also** given | **CLI wins** — it is the probing / entry-relative override, documented as such |

⭐⭐ **THE IMPLEMENTATION IS A NORMALIZATION, NOT A THREADING — and that is the whole
design.** `rule_order[0]` is consulted by **~10 sites** (`main.rs` ×7,
`grammar_wellformedness.rs` ×2, the interpreter, plus every `--entry-rule`-less
consumer). Threading an `Option<String>` through all of them would be exactly the
per-call-site fragility `LANG-CAPABILITY-AUDIT.7` rejected for `include()`. Instead:
**resolve `@entry` ONCE at the single grammar-load chokepoint and reorder `rule_order`
so the declared rule is first.** Every consumer then inherits it **structurally**, with
zero call-site changes and no way for a future consumer to forget.

The chokepoint already exists and is already labelled as one — `main.rs:2209`,
verbatim: *"PARSE-SOTA.8.1 (A1): the SINGLE grammar-load chokepoint every build path
goes through (`--generate-parser`/`-stimuli`, `make focus_*`, `--lint-grammar`)"*. The
new normalization goes immediately **before** `check_grammar_wellformed(&grammar)?`
so the linter sees the normalized order.

⭐ **The invariant `rule_order[0] == the entry` therefore SURVIVES INTACT.** We are not
weakening it; we are letting the author choose which rule occupies that slot instead of
it being an accident of file layout. This is precisely the director's point — file
order stops mattering, because the loader normalizes it from declared intent.

#### Zero-cost / neutrality acceptance test (the #208 standing test)

1. **Non-users pay ZERO** — no directive ⇒ no reorder ⇒ **byte-identical parser**, to
   be PROVEN not asserted, by the pinned byte-identity method (`.5` /
   `BIN-BUILD-INTEGRITY.3`: input **and** output paths pinned, because codegen embeds
   the output path).
2. **Users pay at CODEGEN time** — the directive only changes which `parse_X()` the
   emitted `parse()` dispatches to. No runtime branch, no engine table.
3. **No runtime cost at all**, so criterion (3) is vacuous here.

#### Migration doubles as the regression proof

All 10 tracked grammars already define their entry first. Adding `@entry: <that same
rule>` to each is therefore a **no-op reorder** ⇒ the regenerated parsers must come out
**byte-identical**. That makes intent explicit everywhere AND proves inertness with the
strongest available oracle.

⭐ It also **fixes the scratch slot's unenforced contract for free**: the fixture
declares `@entry: scratch`, so a probe author may overwrite the body and put helper
rules anywhere without silently re-rooting the slot — which is exactly the accident
that produced this tree.

#### ⭐⭐⭐ DIRECTOR REQUIREMENT (2026-07-26, late #212): `@entry` IS MANDATORY, EXACTLY ONCE

First stated as *"At least one `@entry: true` shall be defined in the main EBNF"*, then
**corrected by the director the same session** — verbatim: *"sorry, maybe I shouldn't
have used 'at least' … you suggested that maybe we could support more than one
`@entry: true`, that's why. So please replace 'at least', by **'one and only one'**."*

⇒ **ONE AND ONLY ONE `@entry: true` in the main EBNF.** The correction is a deliberate
narrowing: "at least one" would have left a door open to the multi-entry increment,
which the director has explicitly deferred. This pins increment 1 as strictly
single-entry.

✅ **The "and only one" half is ALREADY IMPLEMENTED AND VERIFIED** by this leaf —
`compile_entry_rule` rejects a second declaration with a hard error naming both rules
(measured: `@entry: true` on rules `a` and `b` ⇒ `rc=1`, *"declared on more than one
rule ('a' and 'b'). A grammar has exactly ONE entry rule; declare it once."*). What
remains is the **mandatory** half: zero declarations must stop being legal.

This supersedes the "absent ⇒ positional default" compatibility story above: a grammar
with no `@entry: true` becomes an **error**, and the declaration must live in the
**MAIN** file — not in an included one (which is what stops an include from re-rooting
a grammar, the hazard `LANG-CAPABILITY-AUDIT.7` guarded against by ordering; `.9`'s
per-file rule-ownership tracking already provides the provenance needed to enforce
"main file" specifically).

⚠️ **BLAST RADIUS — MEASURED BEFORE COMMITTING TO A SEQUENCE, not assumed:**

| population | count | declaring `@entry` today |
|---|---|---|
| tracked `.ebnf` files | **78** | **0** |
| ⤷ of which `grammars/` | 17 | 0 |
| ⤷ `tests/`, `test/`, `test_includes/`, corpora, artifacts | 61 | 0 |
| synthetic grammars embedded in the parse-harness Rust oracles | ~200+ rule defs across 5 modules (27 combinator + 36 semantic cases) | n/a |

⛔ **Flipping to mandatory in one step would red-line every gate in the repository at
once — including the parse-harness combinator/semantic/equivalence suites, which are
the very oracles that prove this change safe.** Removing the safety net at the moment
it is needed is not an acceptable way to satisfy the requirement.

⇒ **SEQUENCE (recommended, director decision pending):**

- **Step A — this leaf, DONE:** `@entry: true` is recognized, honoured, and outranked
  by `--entry-rule`; positional fallback still works. Proven byte-inert.
- **Step B:** migrate all 78 tracked grammars (+ the inline synthetic grammars) to
  declare `@entry: true`. Every one of them already defines its entry first, so each is
  a **no-op reorder** and the regenerated parsers must stay byte-identical — the
  migration is its own proof.
- **Step C:** flip to MANDATORY (missing `@entry` in the main EBNF = hard error). After
  A+B this is a small change with zero breakage, and it is provable rather than hoped.

Doing C before B is the only ordering that cannot be verified.

#### Explicitly OUT of scope (director-deferred)

- ⛔ **Multi-entry / a list of entries** — deferred to increment 2 by director decision.
  `sv_multi_entry_root` therefore stays exactly as it is; nothing in SV changes.
- ⛔ Deleting `sv_multi_entry_root`, touching `--cert-union-config`, or changing the
  closure-gate contract.
- ⛔ Making the positional default an error or a warning. It stays the default
  **forever** — it is what all 10 tracked grammars rely on today.

#### Step A — LANDED and VERIFIED (session #212)

| behaviour | verdict |
|---|---|
| `@entry: true` on the **last-defined** rule | ✅ codegen emits `self.parse_scratch()` — file order is now irrelevant |
| no declaration | ✅ `self.parse_stmt()` — positional default unchanged |
| **`--entry-rule stmt` + `@entry` on `scratch`** | ✅ **`self.parse_stmt()` — the CLI outranks the declaration (director rule)** |
| `--entry-rule nosuch` | ✅ hard, named error (it was a *silent* fallback before) |
| `@entry: true` on two rules | ✅ error naming **both** rules ("one and only one" — already enforced) |
| `@entry` inline in a rule body | ✅ error naming the placement (branch-start AND mid-sequence) |
| `@entry: "scratch"` (rule-name payload) | ✅ error — attachment already identifies the rule |
| `@entry: false` | ✅ explicit no-op |
| `--lint-grammar` | ✅ now prints `[info] entry rule 'X' — DECLARED via @entry: true` / `— POSITIONAL (…)` |

⚠️ **Found rather than assumed: `--entry-rule` was ALREADY silently ineffective on
`--generate-parser`** — `generate_parser_ast_based` receives only `&grammar.rule_order`
and has never been passed the CLI entry. Pre-existing, not introduced by `@entry`; the
director's precedence rule is what prompted the check. Fixed through the SAME reorder,
so there is one mechanism for "which rule is the entry", not two.

⭐ **`.1`'s driver correctly went RED and was RE-PINNED, not deleted.** It asserted
*"lint mentions the resolved entry rule (0 = never)"* — the very gap `.2` closes. The
assertion now pins the new truth (`1`, plus the `POSITIONAL` wording), so the record
shows the gap closing rather than the evidence being quietly edited.

**Verification:**
- `run_entry_directive_probes.sh` — **14 declared-verdict cases, exit 0, 0 divergences**.
- `run_entry_rule_probes.sh` (`.1`'s driver, re-pinned) — **17 cases, exit 0, 0 divergences**.
- ⭐ **NO REGRESSION — all 10 generated parsers BYTE-IDENTICAL** against a HEAD-vintage
  binary with input **and** output paths pinned (`.5`/`BIN-BUILD-INTEGRITY.3` method):
  json 13,707 · regex 520,694 · vhdl 196,512 · systemverilog 1,840,414 · svpp 51,988 ·
  rtl_frontend 164,187 · rtl_const_expr 35,777 · return_annotation 37,809 ·
  semantic_annotation 200,236 · ebnf 156,863 — **3,218,187 lines, 0 differences**
  (`byte_identity.sh` + `byte_identity_capture.txt`).
- clippy source-strict **0 errors**; the one finding on new code (a redundant closure)
  fixed. Generated-stage debt is the tracked non-strict baseline, untouched.
- `ebnf_parser_book_gate` GREEN (tracked HTML re-rendered).

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `.1`: reordering two rule definitions flips `a;a;` from
      ACCEPT to REJECT@2 while `--lint-grammar` reports `unreachable_rules=0`, exit 0.
- [x] **ROOT CAUSE (WHY + WHERE)** — `.1`, tool-pinned: `ast_based_generator.rs:616-621`
      `entry_rule = self.entry_rule.or_else(|| rule_order.first())`; three-arm control
      isolates the start symbol with the execution graph held fixed.
- [x] **FIX** — declarative tier (highest in the fix hierarchy): a grammar-level
      `@entry` directive normalized at the single grammar-load chokepoint.
- [x] **ADDRESSED (verified)** — a grammar whose entry is declared but defined LAST
      parses the multi-statement input; declared-but-undefined entry is a named hard
      error; `--lint-grammar` reports the resolved entry and whether it was declared.
- [x] **NO REGRESSION** — all 10 generated parsers **byte-identical** with the
      directive absent (input+output paths pinned); byte-identical again after the
      no-op `@entry` migration; cert seeds 0/7/42; `ast_shape_contract`; clippy.
- [x] **LOCKSTEP** — grammar-author book (`grammar-file-structure.md` +
      `semantic-annotations.md`), `TOOLBOX.md`, `docs/book/`.

### `.3` — Record the trace-changes-the-engine trap in `TOOLBOX.md` (`done`)

> **Slice `PGEN-QUANT-PLUS-ITER-0002` (session #212) — docs-only.** The charter asked
> for one caveat; measurement found **two**, and the second is what confounded `.1`'s
> own first evidence table.

Measured, from the emitted source of every generated parser:

```rust
self.bare_parse = !self.coverage_enabled && !self.logger_enabled
    && !self.counters_observed.get()
    && !crate::ast_pipeline::report_memo_stats_enabled();
```

1. **Tracing switches the engine** (the charter's ask) — a logger sets
   `logger_enabled`, so a traced run leaves the fused `cascade_*` graph for the
   PROTOCOL graph. ⇒ *a traced run is not necessarily the run that produced the
   untraced verdict.*
2. ⭐ **`--entry-rule` switches the engine too** (NEW this session) — `parse_from`
   sets `self.bare_parse = false;` **unconditionally**, measured across **all ten**
   generated parsers (json, regex, return_annotation, rtl_const_expr, rtl_frontend,
   scratch, semantic_annotation, systemverilog, systemverilog_preprocessor, vhdl —
   1/1 each). ⇒ **an `--entry-rule X` vs default comparison varies TWO things.** The
   remedy is stated with the caveat: pass `--entry-rule` on *both* arms, including
   the canonical entry, so only the rule name differs.

Both are the same observability-twin routing already documented for memo-stats
(§3.3) and the counter dumps (§3.4/§3.5) — but neither trace nor `--entry-rule`
carried a routing note, and both *feel* passive, which is precisely why they mislead.

⭐ **Honest scope of the hazard, measured not assumed:** `.1`'s three-arm control ran
the same grammar and inputs on both graphs and they **agreed on all four inputs**. So
this is a *reasoning* hazard — say which graph you observed — **not** a known
correctness difference; the two graphs are held byte-identical by the
equivalence/AST oracles (`TOOLBOX.md` §1.6).

**LANDED in `TOOLBOX.md`:**
- §2.1 — a `⚠️⚠️ ROUTING — TRACING CHANGES WHICH ENGINE RUNS` bullet with the verbatim
  `bare_parse` computation.
- §1.1 — a `⚠️⚠️ ROUTING — --entry-rule ALSO CHANGES WHICH ENGINE RUNS` bullet with
  the both-arms remedy, **plus** a `⭐ FIRST QUESTION ON ANY "THIS RULE MISBEHAVES"`
  bullet (*is the rule you are testing the rule being run?*) carrying the
  entry-is-`rule_order[0]` rule, the lint-is-silent warning, and the one
  cert-coverage command that answers it.
- The **Quick chooser** table gains a first row routing
  *"a rule misbehaves / a quantifier iterates once / my rule seems ignored"* to that
  check — so the question that would have closed `.1` in one command is the first
  thing a reader meets.

### `.4` — residual entry-visibility items not covered by `.2` (`todo`)

Carved out of the original `.2` so `.2` stays exactly the director-approved scope.

- The **orphan report** already routed to `LANG-CAPABILITY-AUDIT.2`/`.4` (T-9,
  *"N rules are unreferenced roots"*). `.1` is its second independent motivation (the
  first was `ebnf.ebnf:18`'s dangling include). ⛔ Keep it **separate from** the hard
  `unreachable_rules` error: treating an unreferenced rule as a root is CORRECT design
  and is what protects SV's `library_text` / `*_multi_entry_root` secondary entries
  from false rejection.
- Whether `--report-certificate-coverage`'s *"adjudicate via the linter"* hand-off
  should name an instrument that can actually confirm it (`.1`'s measured instrument
  disagreement).


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
| `PGEN-QUANT-PLUS-ITER-0001` | `.1` | `ce71ffaa` (session #212, 2026-07-26) |
| `PGEN-QUANT-PLUS-ITER-0002` | `.3` (+ `.1` engine control) | see git log (session #212, 2026-07-26) |
