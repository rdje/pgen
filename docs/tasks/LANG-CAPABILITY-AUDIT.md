# LANG-CAPABILITY-AUDIT: which real-language constructs can PGEN's EBNF actually express?

> ⭐⭐ **SCOPE AMENDED (director, 2026-07-26, session #208):** *"LANG-CAPABILITY-AUDIT
> shall not focus solely on Javascript, but on features that would be necessary for
> PGEN EBNF to support cleanly that would allow PGEN to parse them, like Raku,
> Python, Ruby, even VHDL, SysVerilog — I mean any language you know of, that are
> well-known to be extremely painful to parse. PGEN shall be able to parse any
> language like it is a walk in a park."*
>
> ⇒ The audit is **capability-first, cross-language**. ECMA-262 is ONE column, not
> the subject. The unit of work is a **capability** (a thing an EBNF must be able to
> say); languages are the *evidence* that the capability is needed.

## Metadata

- Tree ID: `LANG-CAPABILITY-AUDIT`
- Status: `active` (opened 2026-07-26, session #208, by **direct director order**:
  *"please execute this capability-gap audit since it will make PGEN much, much,
  much stronger by uncovering gaps. and we do not want gaps in PGEN, right?"*)
- Family / slice-id prefix: `PGEN-LANG-CAPABILITY-AUDIT-<NNNN>`
- Roadmap lane: the **universal-parser horizon** —
  [[project_horizon_universal_parser]] §3 has prescribed a capability-gap audit
  since 2026-07-11 and it had **never been run**; both capability axes found in
  session #208 were found *reactively*.
- Created: `2026-07-26`
- Owner: repo-local workflow
- Governed by [[project_capability_growth_is_zero_cost_and_neutral]]: every gap this
  audit prices must carry a cost model — non-users pay zero, users pay at codegen
  time where semantics permit.

## PRIOR ART (per [[feedback_read_prior_art_before_designing]])

| source | searched | found |
|---|---|---|
| `docs/tasks/` | capability / audit / gap / horizon / ecma / javascript | ⭐ **[`PARSE-SOTA`](PARSE-SOTA.md) already produced "a citation-backed capability-gap audit"** — but on a **different axis**: it weighs *our implementation* against the **academic literature** ("is the parser-generator path SOTA?"). This tree's axis is **language expressiveness** ("can our EBNF express what real languages need?"). Complementary, not duplicate — and `PARSE-SOTA`'s frontier is a director review of its adoption backlog, so it is not blocked by this. |
| `docs/decisions/` | capability | [[project_horizon_universal_parser]] holds the *living capability-axes list* this tree operationalizes; [[project_ebnf_steers_the_engine_at_full_granularity]] and [[project_no_layout_primitive_is_undeclarable]] are two axes already added. |
| `grammars/ebnf.ebnf` | the whole file | ⭐⭐ **see the `.1` finding below — the meta-grammar itself already contains an unfinished capability wishlist.** |
| `docs/book/` | roadmap / academic-foundations | `academic-foundations.md`, `roadmap-and-live-status.md` — narrative, no expressiveness matrix. |

⇒ No existing tree owns the language-expressiveness axis. This tree is new work.

## Why this tree exists (the meta-finding it answers)

[[project_horizon_universal_parser]] §3 says *"pursue it via a capability-gap audit,
not language-by-language."* In practice every capability gap so far was found
**reactively**, from a tracked language already hurting. JS / Ruby / Raku are not
tracked, so their gaps are **structurally invisible**. Two axes surfaced by accident
in one afternoon (session #208) is the evidence that a deliberate pass pays.

## Leaves

### `.1` — Audit PGEN's OWN meta-grammar for declared-but-unwired capability

- **Status: `done`** (`PGEN-LANG-CAPABILITY-AUDIT-0001`, session #208). Read-only.
- **Why start here, not with ECMA-262:** the cheapest possible first pass. Before
  asking "what does JavaScript need that we lack?", ask "what have we *already
  written down* that we lack?" — and the answer turned out to be substantial.

#### ⭐⭐ FINDING: 27 of 131 productions in `grammars/ebnf.ebnf` are UNREACHABLE

Re-run: `bash docs/tasks/artifacts/lang_capability_audit/metagrammar_reachability.sh`

```
entry='grammar_file'  rules=131  reachable=104  UNREACHABLE=27
```

They are not noise — they cluster into **named capability families**, and several map
directly onto the horizon record's own gap list:

| family | unreachable productions | horizon axis it maps to |
|---|---|---|
| **Error recovery** | `error_production`, `error_recovery_action`, `panic_mode`, `sync_to`, `skip_to` | ⭐ the record's *"Error recovery — HTML/'tag soup' … **NOT yet in PGEN**"* — the productions were written and never wired |
| **Parameterized productions** | `parametric_rule`, `parameter_list` | ⭐⭐ **exactly ECMA-262's `Expression[In, Yield, Await]`** — the single most structural feature of the JS grammar |
| **Lexer modes** | `lexer_mode` | template literals `${…}`, heredocs, Raku quoting |
| **Generics / templates** | `template_instantiation`, `type_argument`, `type_argument_list` | parameterized rule instantiation |
| **Semantic predicates (inline form)** | `semantic_predicate`, `predicate_content` | (the `@predicate` *directive* exists; this inline surface does not) |
| **Grammar composition** | `import_statement`, `grammar_inheritance`, `exception_rule`, `exception_list` | multi-file / layered grammars |
| **Misc** | `action_block`, `action_content`, `named_capture`, `case_control`, `case_modifier`, `rule_modifier`, `optimization_hint`, `optimization_directive`, `optimization_parameter(s)` | — |

⇒ **PGEN's meta-grammar has been quietly documenting its own capability gaps.**
Each of these is a surface a grammar author might reasonably try to write, that
parses as far as the meta-grammar is concerned, and that no engine code consumes.
`parametric_rule` was confirmed at **zero** consumers across all of `rust/src/`.

⚠️ **This is the same disease already named twice this session** — a surface that
exists on paper but not in the engine ([[project_no_layout_primitive_is_undeclarable]]),
and a placement accepted then silently dropped (`ANNOTATION-PLACEMENT`). Here it is
at meta-grammar scale: **20.6% of our own EBNF language is decorative.**

#### ⚠️ SECOND FINDING (unexplained — needs its own root-cause, NOT claimed as a bug yet)

`--lint-grammar` on the same file reports **`unreachable_rules=0`**, while an
independent closure over the `--dump-gen-ast` IR finds **27**. One of the two is
wrong, and I have **not** established which.

Candidate explanations, none verified:
- the lint may quantify over a **declared entry universe** (several roots), not the
  single `grammar_file` entry my closure used — cf. the `PGEN_CERT_RESIDUAL_CLASSIFICATION`
  entry-universe semantics, which exist precisely because "unreachable from ONE entry"
  is not the same claim;
- the lint may run on a **pre-filter / unfiltered** rule set with different edges;
- or the linter genuinely under-reports on this grammar.

⛔ **Explicitly NOT concluded:** that the linter is broken. `feedback_certifying_linter_trustworthiness`
makes the linter the fulcrum of sign-off, so a soundness claim against it requires the
3-step toolbox protocol and an entry-semantics check first — **that is leaf `.2`.**
Recorded as *unexplained*, not as a defect, per [[feedback_be_alert_root_cause_fishy_immediately]].

- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — the audit had never been run despite being prescribed
    since 2026-07-11; both #208 axes were found reactively.
  - [x] **ROOT CAUSE (WHY + WHERE)** — measured: `grammars/ebnf.ebnf` 131 rules,
    104 reachable from `grammar_file`, **27 unreachable**, named and clustered above;
    `parametric_rule` has **0** consumers in `rust/src/` (`grep -rn "parametric"`).
  - [x] **FIX** — N/A (read-only audit leaf). Output is the gap inventory + `.2`/`.3`.
  - [x] **ADDRESSED (verified)** — re-runnable driver banked
    (`metagrammar_reachability.sh`); reachability computed from the `--dump-gen-ast`
    IR that codegen itself consumes, not from text.
  - [x] **NO REGRESSION** — no `grammars/`, `rust/`, codegen, generated artifact or
    contract touched.
  - [x] **LOCKSTEP** — tree registered in `docs/TASK_TREE.md`; horizon record and
    `MEMORY.md` updated; the unexplained lint discrepancy routed to `.2` rather than
    silently absorbed.

### `.2` — Root-cause the lint-vs-closure reachability discrepancy

- **Status: `done`** (`PGEN-LANG-CAPABILITY-AUDIT-0002`, session #208). Read-only.

#### ✅ RESOLVED — the linter is NOT wrong, and neither is the closure

`rust/src/ast_pipeline/grammar_wellformedness.rs:299-306` documents the root set
verbatim:

> *"Roots = the canonical entry (`rule_order[0]`) **PLUS every rule that NOTHING
> references** (a secondary entry…). Multi-entry-SAFE (an unreferenced top is a root,
> never a false "unreachable") and **conservative** (an unreferenced dead orphan is
> treated as a root → **not flagged**; only referenced-but-unreachable dead ISLANDS
> are caught — **false negatives are safe**, false positives would wrongly reject a
> good grammar)."*

⇒ The two numbers answer **different questions**, and both are correct:

| instrument | question | answer |
|---|---|---|
| `--lint-grammar` | is any rule a *referenced-but-unreachable dead island*? | **0** — sound, deliberately conservative |
| the `.1` closure | is any rule unreachable from the *single canonical entry*? | **27** — all 27 are unreferenced orphans, which the lint classifies as roots |

⛔ **No linter defect. The `.1` "second finding" is withdrawn as a suspicion** — and
the design reason is good: PGEN grammars legitimately carry secondary entries (SV's
`library_text`, `*_multi_entry_root`), so treating an unreferenced top as a root is
what prevents false rejections of correct grammars.

#### ⭐ BUT — the conservatism has a NAMED BLIND SPOT, and it is exactly the `.1` class

Because *unreferenced ⇒ root ⇒ reachable*, a production that is **declared and never
referenced by anything** can **never** be flagged by this check — not now, not ever.
That is precisely the `parametric_rule` class: dead surface no gate can see.

⇒ **Recommendation (routed to `.4`, not actioned here):** an **informational,
opt-in** orphan report — *"N rules are unreferenced roots"* with names — separate
from the hard `unreachable_rules` error so multi-entry safety is preserved. Dead
surface becomes visible without false-failing legitimate grammars.

⚠️ **Note the recurring shape — third time today:** a check that *structurally
cannot see* a class of defect and reports green (cf. the self-hosting cross-check
standing down, and mid-sequence annotations dropped silently). The unifying
principle stays [`ANNOTATION-PLACEMENT`](ANNOTATION-PLACEMENT.md)'s: **a check that
cannot run — or cannot see — must say so, not return green.**

- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — `.1` measured lint `unreachable_rules=0` vs a closure
    of 27 on the same file; one had to be wrong.
  - [x] **ROOT CAUSE (WHY + WHERE)** — `grammar_wellformedness.rs:299-306`
    (`detect_unreachable_rules` / `reachable_rules`): roots = `rule_order[0]` ∪
    every unreferenced rule. `parametric_rule` is referenced by nothing ⇒ it IS a
    root ⇒ trivially reachable ⇒ correctly not flagged.
  - [x] **FIX** — none required; the suspicion is withdrawn. Output is the
    adjudication + the named blind spot routed to `.4`.
  - [x] **ADDRESSED (verified)** — the doc-comment states the semantics and the
    accepted false-negative class explicitly; the 27 are all unreferenced orphans,
    matching that class exactly.
  - [x] **NO REGRESSION** — read-only; nothing touched.
  - [x] **LOCKSTEP** — `.1`'s "second finding" re-labelled RESOLVED (not a defect);
    blind-spot recommendation carried into `.4`; `MEMORY.md` corrected.

<details>
<summary>Original charter (superseded by the resolution above)</summary>

- **Status: `todo`** — **run before trusting either number.** Establish the lint's
  entry-universe semantics (`--lint-grammar` unreachable computation), re-run the
  closure over the same universe, and adjudicate: instrument bug, semantics
  mismatch, or my probe's entry choice. Toolbox-first; no verdict without tool output.

</details>

### `.3` — The cross-language capability matrix (director-scoped; ECMA-262 is one column)

- **Status: `in_progress`** — first cut below. Rows are **capabilities**; the
  languages are evidence. Verdicts: **✅ have** / **⚠️ partial** / **🕳️ declared-unwired**
  (the surface exists in `ebnf.ebnf` with zero engine consumers — measured) /
  **❌ gap** / **❓ unmeasured**.

#### ⭐⭐ The corroboration that shapes this leaf

The `.1` orphan set is **not random**. Cross-referencing it against the notorious-language
feature list, the dead productions in our own meta-grammar land almost one-to-one on the
hardest capabilities. Someone enumerated the right list and never wired it.
Measured engine consumers (`grep -rc … rust/src/`): `case_control` **0**,
`case_modifier` **0**, `lexer_mode` **0**, `error_production` **0**, `panic_mode` **0**,
`import_statement` **0**, `parametric_rule` **0**.

#### The matrix (first cut)

| # | capability — *what the EBNF must be able to say* | languages that force it | PGEN today |
|---|---|---|---|
| 1 | **Parameterized productions** — `Expression[In, Yield, Await]`; one production family instantiated per parameter set | JS/ECMA-262 (pervasive), Ada, C++ | 🕳️ `parametric_rule`/`parameter_list` declared, **0 consumers** |
| 2 | **Significant indentation / offside rule** — synthesize INDENT/DEDENT/NEWLINE from column state | Python, Haskell, YAML, Nim, F#, Raku heredocs | ❌ `@whitespace_sensitive` is grammar-global, not an offside primitive |
| 3 | **Lexer modes / sub-languages** — switch lexical rules mid-parse and nest them | Ruby `#{}`, Python f-strings, JS templates `${}`, shell/Perl/Raku heredocs, embedded SQL/regex | 🕳️ `lexer_mode` declared, **0 consumers** |
| 4 | **Lexical adjacency / no-layout boundaries** — "no white space (or no newline) between these two elements" | JS ASI `[no LineTerminator here]`, SV fn 44, Ruby `foo?`/`a +b`, Raku | ⚠️ notation exists (`[>! /\s/]`) but **generator-only**; inline per-seam form designed, unbuilt → [`LEX-ADJACENCY`](LEX-ADJACENCY.md) |
| 5 | **Case-insensitive keywords** — `BEGIN`/`begin`/`Begin` one token, identifiers still case-preserving | **VHDL**, Ada, SQL, Fortran, Pascal | 🕳️ `case_control`/`case_modifier` declared, **0 consumers** ⇒ VHDL's case-insensitivity is currently handled *inside* the grammar, not declared |
| 6 | **Error recovery / resync** — a spec whose *recovery* is normative | HTML (WHATWG defines it), any IDE-grade parser | 🕳️ `error_production`/`error_recovery_action`/`panic_mode`/`sync_to`/`skip_to` declared, **0 consumers**; the horizon record says "NOT yet in PGEN" |
| 7 | **Declaration-sensitive parsing** (the "lexer hack") — `T * x;` is a decl or a product depending on a *prior declaration* | C/C++, **SV** (`type_identifier`), VHDL | ✅ **PGEN's core strength** — semantic store `has_fact`/scopes, proven at SV scale |
| 8 | **Contextual / soft keywords** — a word is a keyword only in some positions | Python (`match`, `case`, `type`), JS (`let`, `async`, `of`), SV, C# | ⚠️ likely expressible via store + predicates + `@profiles`; **unmeasured as a general pattern** |
| 9 | **Cover grammars / delayed disambiguation** — parse one shape, reinterpret later | JS `(a,b)` arrow-params vs paren-expr; C++ most-vexing-parse | ❓ tournament + backtracking *may* cover it; **unmeasured** |
| 10 | **Balanced / user-chosen delimiters** — closing delimiter determined by the opening one, incl. mirrored pairs | **Raku** `q//`/`qq{}`/`«»`, Perl `q{}`, Rust `r#""#` | ❓ needs parse-state feeding the matcher; **unmeasured — suspected gap** |
| 11 | **Here-documents** — terminator named *now*, body starts on the *next line* | Ruby, Perl, Raku, shell, PHP | ❓ non-context-free interaction of line structure + a named terminator; **suspected gap** |
| 12 | **Operator-precedence declaration** — a precedence/associativity table instead of a hand-rolled cascade | VHDL, SV, C, most expression languages | ⚠️ `@priority`/`@associativity` exist per-rule; the ~16-level `rtl_const_expr` cascade is **measured** un-generatable within the bounded ladder (PARSE-HARNESS `.5.5`) ⇒ a real, already-felt pain |
| 13 | **Unicode identifier classes + normalization** | Python (PEP 3131, **NFKC**), JS ID_Start/ID_Continue, Raku | ❓ regex-class support unmeasured; normalization almost certainly absent |
| 14 | **Preprocessor / macro phase** | C/C++, **SV** `` `define ``, Rust macros | ⚠️ svpp is a *separate grammar*, not a composable phase |
| 15 | **Grammar composition** — import/extend another grammar | large LRMs, layered dialects | 🕳️ `import_statement`/`grammar_inheritance` declared, **0 consumers** |
| 16 | **Parse-time-mutable grammar** — the program *extends its own syntax* | **Raku** (slangs, custom operators), Perl 5 (`BEGIN`, prototypes) | ⛔ **HARD BOUND — out of scope by design.** The horizon record already scopes this: the realistic target is the *precise static subset*. Recording it keeps the boundary honest rather than pretending "any language" includes self-modifying ones. |

#### Reading the matrix

- **🕳️ declared-unwired is the dominant class (7 of 16 rows).** The capability was
  *designed* and never built. That is a far cheaper starting position than "absent" —
  and it is exactly why `.1` (audit our own meta-grammar) was the right first pass.
- **Rows 2, 10, 11 are the true "extremely painful" cluster** — Python indentation,
  Raku delimiters, heredocs. All three need **parse state feeding the lexical layer**,
  which is the same architectural seam as row 3 (lexer modes) and row 4 (adjacency).
  ⇒ Hypothesis for `.4`: **one primitive family may unlock 2/3/4/10/11 together.**
- **Row 7 is where PGEN already beats most toolkits** — the store makes the lexer
  hack declarative. Worth stating in the book: it is the strongest existing answer to
  "extremely painful to parse".
- ⚠️ **Six rows are ❓ unmeasured.** They are *assessments*, not measurements, and are
  marked as such — no row here may be cited as fact until `.3b` probes it.

### `.3b` — Measure the ❓ rows

- **Status: `todo`.** Rows 8, 9, 10, 11, 13 (+ row 12's generality) are engineering
  judgement, not tool output. Each needs a probe grammar through the existing harness
  (`parse_harness` / scratch slot) before it earns a verdict. **Do not let this
  matrix harden into fact without that pass** — this session produced four separate
  instances of a plausible claim surviving because nobody re-ran it.

### `.4` — Prioritized primitive roadmap

- **Status: `todo`**, blocked on `.2`+`.3`. Rank by (coverage × tractability), attach
  a **cost model** to each per [[project_capability_growth_is_zero_cost_and_neutral]],
  and feed the horizon record's living axes list.

## Acceptance Criteria (tree)

- A measured, re-runnable expressiveness matrix — not prose.
- Every gap carries a cost model before it becomes a work item.
- The `ebnf.ebnf` unreachable set is adjudicated: wire it, or delete it, or document
  it as reserved — **not left as decorative surface** that misleads grammar authors.

## Evidence

- `docs/tasks/artifacts/lang_capability_audit/metagrammar_reachability.sh` — driver
- `docs/tasks/artifacts/lang_capability_audit/metagrammar_reachability.txt` — capture
