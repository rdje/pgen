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

- **Status: `done`** — the matrix landed here and its six ❓ rows were then
  measured by [`.3b`](#3b--measure-the--rows-done). Rows are **capabilities**;
  the languages are evidence. Verdicts: **✅ have** / **⚠️ partial** /
  **🕳️ declared-unwired** (the surface exists in `ebnf.ebnf` with zero engine
  consumers — measured) / **❌ gap** / **❓ unmeasured**.
- ⚠️ **Read the matrix together with `.3b`.** Every row below that once read ❓ now
  carries its measured verdict inline, and **two of them moved in the favourable
  direction** — rows 10 and 11 were logged as *suspected gaps* and are in fact
  expressible today. A row's `.3b` verdict supersedes its first-cut assessment.

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
| 8 | **Contextual / soft keywords** — a word is a keyword only in some positions | Python (`match`, `case`, `type`), JS (`let`, `async`, `of`), SV, C# | ✅ **HAVE — `.3b`-measured, and with NO keyword-exclusion tax.** All three readings of `match` parse against a deliberately *un-taxed* identifier token (4/4) |
| 9 | **Cover grammars / delayed disambiguation** — parse one shape, reinterpret later | JS `(a,b)` arrow-params vs paren-expr; C++ most-vexing-parse | ✅ **HAVE — `.3b`-measured** (4/4), including a disambiguator sitting past an arbitrarily deep ambiguous prefix |
| 10 | **Balanced / user-chosen delimiters** — closing delimiter determined by the opening one, incl. mirrored pairs | **Raku** `q//`/`qq{}`/`«»`, Perl `q{}`, Rust `r#""#` | ⚠️ **PARTIAL — `.3b` OVERTURNED the "suspected gap".** The OPEN delimiter set is expressible TODAY via a store-driven dynamic guard (6/6). The bound is **fact LIFETIME**, not lexical reach: repeated quotes collide on the monotone parse-global store (2 of 3 wrong) |
| 11 | **Here-documents** — terminator named *now*, body starts on the *next line* | Ruby, Perl, Raku, shell, PHP | ⚠️ **PARTIAL — `.3b` OVERTURNED the "suspected gap".** A single heredoc is fully correct (5/5) *including* the discriminating body-line-that-looks-like-another-tag case; the repeated-heredoc bound is **identical to row 10's** |
| 12 | **Operator-precedence declaration** — a precedence/associativity table instead of a hand-rolled cascade | VHDL, SV, C, most expression languages | ⚠️ **CONFIRMED + sharpened by `.3b`:** `@priority`/`@precedence` take an **integer** payload and rank the alternatives **inside one rule**; there is no cross-rule ladder. The ~16-level `rtl_const_expr` cascade is **measured** un-generatable within the bounded ladder (PARSE-HARNESS `.5.5`) ⇒ a real, already-felt pain |
| 13 | **Unicode identifier classes + normalization** | Python (PEP 3131, **NFKC**), JS ID_Start/ID_Continue, Raku | ✅ **(a) classes HAVE** — `.3b`-measured: `\p{XID_Start}`/`\p{XID_Continue}` compile and discriminate (5/5, incl. `café`/`变量`/reject `1abc`). ❌ **(b) NFKC normalization ABSENT** — 0 normalization crates, 0 NFKC/NFC/NFD mentions in `rust/src` |
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
  ⛔ **`.3b` PARTIALLY REFUTED this hypothesis — see its "the hypothesis was half
  wrong" section.** Rows 10 and 11 need no new lexical channel at all; the store
  ALREADY feeds the lexical decision. What they need is fact **lifetime**.
- **Row 7 is where PGEN already beats most toolkits** — the store makes the lexer
  hack declarative. Worth stating in the book: it is the strongest existing answer to
  "extremely painful to parse".
- ⚠️ **Six rows are ❓ unmeasured.** They are *assessments*, not measurements, and are
  marked as such — no row here may be cited as fact until `.3b` probes it.
  ✅ **DISCHARGED by `.3b`** — every ❓ row now carries a measured verdict.

### `.3b` — Measure the ❓ rows (`done`)

- **Status: `done`** (`PGEN-LANG-CAPABILITY-AUDIT-0004`, session #209).
  **Read-only** — no `grammars/` (except the blessed throwaway scratch slot, restored
  by the driver), no `rust/`, no codegen, no generated artifact, no contract touched.
- **Method:** two probe banks driven through the PARSE-HARNESS **scratch slot**
  (TOOLBOX 1.3 — authoritative *by construction*: the real register → codegen → drive
  pipeline), one bank per grammar-level layout policy, every row parsed in **isolation**
  via `--entry-rule` so the banks' top-level alternation cannot launder one row's
  verdict into another's. **43 cases, each with its required verdict declared in the
  driver** (`[want …]`), so a gap is a printed `⛔`, never a reading of prose.

#### ⭐ ROW 0 — the control that had to run first

Rows 8 and 9 both hinge on "the engine keeps trying alternatives", so their verdicts
are worthless until the DEFAULT selection semantics are *measured*. `a | a b` on
`"ab"` is the discriminator (TOOLBOX Protocol D, the A2.3 case):

```
  ACCEPT [want ACCEPT]    r0-default-ab          "ab"     # default policy
  REJECT [want REJECT]    r0-ordered-ab          "ab"     # @branch_policy: ordered
```

⇒ the default is a **longest-match tournament**, and `ordered` is the one that commits
to the first success. A **second, independent instrument agrees**: `--lint-grammar`
raises its policy-conditional shadowing error on `r00_ordered` **only** — *"the rule's
effective @branch_policy is 'ordered' (the choice commits to the first successful
alternative)"* — and stays silent on the byte-identical `r00_policy`.

#### The measured verdicts

| row | first-cut assessment | `.3b` verdict | cases |
|---|---|---|---|
| **8** contextual / soft keywords | ⚠️ "likely expressible … unmeasured" | ✅ **HAVE — and the identifier token pays NO keyword-exclusion tax** | 4/4 |
| **9** cover grammars | ❓ "may cover it" | ✅ **HAVE**, incl. an arbitrarily distant disambiguator | 4/4 |
| **10** user-chosen delimiters | ❓ **"suspected gap"** | ⚠️ **PARTIAL — the suspicion is OVERTURNED**: the open delimiter set is expressible TODAY | 6/6 single · 1/3 repeated |
| **11** here-documents | ❓ **"suspected gap"** | ⚠️ **PARTIAL — same**: a single heredoc is fully correct | 5/5 single · 1/3 repeated |
| **12** precedence declaration | ⚠️ per-rule only | ⚠️ **CONFIRMED + sharpened** (integer branch-rank inside ONE rule; no cross-rule ladder) | source + oracle |
| **13** Unicode identifiers | ❓ classes unmeasured; normalization "almost certainly absent" | ✅ **(a) classes HAVE** / ❌ **(b) NFKC ABSENT** | 5/5 · 0 crates |

#### ⭐⭐ FINDING 1 — rows 10 and 11 are NOT gaps. The open-set delimiter is expressible today.

The construction is a **store-driven dynamic guard**: register the opening delimiter
(`@emit_fact`), gate every body element `lacks_fact` against it, gate the closer
`has_fact`. Measured on the Raku `q`-string shape:

```
  ACCEPT  q/abc/     ACCEPT  q!abc!     ACCEPT  q#abc#      # any delimiter char
  ACCEPT  q/ab!c/    ACCEPT  q//                            # foreign delim in body; empty body
  REJECT  q/abc!                                            # closer must equal opener
```

and on the here-document shape — **including the case that separates a real heredoc
from a hack**, a body line that looks like a *different* tag:

```
  ACCEPT  <<END\nline1\nEOF\nline2\nEND\n     REJECT  <<EOF\nline1\n      (unterminated)
  ACCEPT  <<EOF\nEOF\n                        REJECT  <<EOF\nline1\nEND\n (wrong tag)
```

⇒ **two of the audit's three "extremely painful" rows were mis-classified as gaps.**
Recorded as a correction, not smoothed over: the first cut said *"needs parse-state
feeding the matcher — suspected gap"*, and the measurement says the parse state
already reaches the matcher.

#### ⭐⭐ FINDING 2 — the real bound is fact LIFETIME, and it is one named primitive

Both rows fail on the **same** thing, and the failure is identical in shape:

```
  REJECT [want ACCEPT] ⛔ r10-two-inner-slash    "q/abc/q!d/e!"     # stale delim blocks a legal body char
  ACCEPT [want REJECT] ⛔ r10-two-wrong-close    "q/abc/q!de/"      # stale delim closes the wrong quote
  REJECT [want ACCEPT] ⛔ r11-two-cross          "<<A\nx\nA\n<<B\nA\ny\nB\n"
  ACCEPT [want REJECT] ⛔ r11-two-wrong          "<<A\nx\nA\n<<B\ny\nA\n"
```

Root-caused, WHY + WHERE, four independent source facts:

1. the only NEGATIVE query is the parse-global `lacks_fact`
   (`semantic_runtime.rs:3963` = `!fact_index.any_with_name(…)`); the 15-name
   `ENGINE_BUILTIN_PREDICATE_NAMES` registry has **no `lacks_fact_in_current_scope`**
   (measured: 0 occurrences) though it *does* ship the positive
   `has_fact_in_current_scope`;
2. there is **no fact-retraction directive** (measured: 0 occurrences) — the directive
   set is `OpenScope`/`CloseScope`/`EmitFact`/`Predicate`/library ×2/`FactKind`;
3. `close_scope` (`:3689`) pops the chain and marks the arena node closed but **never
   retracts facts** — the code says so itself at `:2852` (*"index, which `close_scope`
   never retracts"*) and `:6152`;
4. even a scope-local negative query would not fix it: the scope-local index is keyed
   by scope **DEPTH**, not identity — `by_scope_and_name: FxHashMap<(usize,
   FactNameKey), Vec<usize>>` (`:2170`) — so two *sibling* quotes both sit at depth 1.

⭐ The `@open_scope`/`@close_scope` repair was **built and measured, not assumed**: its
four cases are byte-for-byte the unscoped verdicts. ⇒ the candidate primitive for `.4`
is **fact retraction / instance-scoped fact lifetime**, priced against (1)–(4).

#### ⭐ FINDING 3 — a general engine law the matrix never named: quantifiers are POSSESSIVE

```
  REJECT [want ACCEPT] ⛔ r10-bt-ab      "ab"      # "a"* "ab"
  REJECT [want ACCEPT] ⛔ r10-bt-aaab    "aaab"
```

`"a"*` takes the `a`, the loop stops, and `"ab"` is attempted at the leftover `b`; the
quantifier **never gives an iteration back**. Trace-confirmed (`❌ Terminal 'a' failed
at position 1 … 🔤 Attempting to match terminal 'ab' at position 1`) and located:
`generate_quantified_logic` (`ast_based_generator.rs:5692`) emits a maximal-munch loop
that breaks on the first failing iteration and enforces only a **min** count — no
emission path re-tries the loop shorter.

This is **consistent with the design**, not a defect: `docs/book/src/developer-architecture.md:19`
already says PGEN has *"greedy/possessive repetition"*. But it is the reason the naive
`body* close` delimiter shape cannot work, and the author-facing consequence — **write
`(!close body)* close`, and when `close` is not known statically, reach for the store
guard** — is stated nowhere. Measured working, both halves:

```
  ACCEPT  q/abc/  q//     via  r10_static := "q" "/" ( !"/" char )* "/"
```

#### ⚠️ FINDING 4 — TWO doc surfaces measured WRONG (routed, not absorbed)

- ⛔ **`docs/book/src/developer-architecture.md:21`** claims *"PGEN is a faithful PEG:
  **ordered choice**, `&`/`!`, greedy quantifiers."* The greedy-quantifier half is
  CORRECT (Finding 3). The **ordered-choice half is measurably false for the default
  policy** — ROW 0 above, with the linter agreeing. A reader is told the first matching
  alternative commits; the engine runs a longest-match tournament unless the grammar
  says otherwise. This is the most fundamental semantic in the book. ⇒ routed to `.5`.
- ⛔ **`grammars/ebnf.ebnf:688`**, inside the meta-grammar's own *"COMPLETE EBNF GRAMMAR
  EXAMPLES (Documentation)"* block, advertises `@precedence: {level: 5, associativity:
  "left"}`. The validator **rejects that payload** — `@priority/@precedence` expects
  `5` or `[1, 9, 2]` (`annotation_validator.rs:656`; parser
  `parse_semantic_numeric_list`, integers only) — and the engine's own test pins the
  `{level: 5}` shape as `W_SEM_INVALID_PRIORITY_PAYLOAD`
  (`annotation_validator.rs:2862`; **re-run this session: PASS**). ⇒ this is `.1`'s
  disease (decorative surface) landing in the examples a grammar author copies.
  Routed to `.5`.

#### ⭐ The `.3` hypothesis was half wrong — recorded as such

`.3` proposed: *"rows 2/3/4/10/11 ALL need PARSE STATE FEEDING THE LEXICAL LAYER ⇒ ONE
primitive family may unlock all five."* Measured: **rows 10 and 11 need no new lexical
channel** — a `lacks_fact` gate on a single-character rule already steers the lexical
decision, which is the very thing the hypothesis assumed missing. Their blocker is fact
lifetime (Finding 2), a different and far cheaper primitive. Rows 2/3/4 stay unmeasured
on that axis and keep the hypothesis; it may no longer be quoted for 10/11.

- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — `.3` shipped a 16-row matrix with **six ❓ rows** it
    explicitly forbade citing as fact; two of them (10, 11) were labelled *suspected
    gaps* and would have entered `.4`'s roadmap as work items that do not exist.
  - [x] **ROOT CAUSE (WHY + WHERE)** — per row, tool output + source location:
    the possessive-quantifier law (`ast_based_generator.rs:5692`, trace-confirmed);
    the fact-lifetime bound (`semantic_runtime.rs:3963` / `:2170` / `:2852` / `:3689`,
    plus 0-occurrence measurements for `lacks_fact_in_current_scope` and retraction);
    the precedence payload (`annotation_validator.rs:656` + oracle test `:2862`);
    NFKC absence (0 crates, 0 mentions in `rust/src`).
  - [x] **FIX** — N/A (read-only measurement leaf). Output is the six verdicts, the
    four findings, and the named `.4` primitive candidate.
  - [x] **ADDRESSED (verified)** — 43/43 cases run through the real
    register→codegen→drive pipeline; **10 printed `⛔` gaps, all inside the two
    families named above and none elsewhere**; re-runnable end-to-end by one command
    (`run_capability_probes.sh`, exit 0), the driver declaring each case's required
    verdict so the capture is self-checking. Both banks lint clean apart from the ONE
    deliberate `r00_ordered` shadowing error, which is itself a measurement.
  - [x] **NO REGRESSION** — nothing shipped was touched: no `rust/`, no codegen, no
    generated artifact, no contract, no tracked grammar. The scratch slot is
    overwritten and **restored by the driver's `trap` on every exit path** (verified:
    `git status` clean of `grammars/`). No release/schema/ledger movement.
  - [x] **LOCKSTEP** — the `.3` matrix rows rewritten with their measured verdicts;
    the `.3` "one primitive family" hypothesis explicitly half-refuted rather than left
    standing; the two measured doc defects routed to a new leaf `.5` instead of being
    fixed inline (they are outside a read-only leaf's mandate); book update owned by
    `.5`.

### `.5` — Repair the measured doc-surface defects (`done`)

- **Status: `done`** (`PGEN-LANG-CAPABILITY-AUDIT-0005`, session #209). Opened by
  `.3b` Finding 4 with the evidence already banked, so no re-diagnosis was needed —
  but the mandated sweep of the surrounding surface **found a third defect, and it is
  the worst of the three because it fails silently.**

#### ⭐ The sweep first: WHERE ELSE does the repo make these claims?

`.3b` found one bad sentence and one bad example. Both fixes are worthless if the same
claim is repeated elsewhere, so the whole documentation surface was swept before
editing anything — and the result reframes the defects:

| claim | book (`docs/book/`) | grammar-author book (`docs/ebnf_parser_book/`) | the meta-grammar's own comments |
|---|---|---|---|
| `\|` selection semantics | ⛔ **`developer-architecture.md:21` WRONG** — but `parse-harness.md:531-534` and `grammar-wellformedness.md:182/220/1475` are all **already correct** (the latter even carries a 2026-07-05 correction retracting the "PEG commits to the first success" reasoning) | ✅ **already correct and detailed** — `rules-and-expressions.md:20-41` states *"PGEN's `\|` is not a first-match commit by default"* with the exact `a \| a b` example | — |
| `[ … ]` is optional, NOT a character class | — | ✅ **already correct** — `terminals.md:89` calls it *"the single most important terminal rule to internalize"* and names the lowering (`ebnf_frontend.rs`), `rules-and-expressions.md:79-80` cross-references it | ⛔ **`ebnf.ebnf` WRONG** (found by this sweep) |
| `@precedence` payload shape | — | — | ⛔ **`ebnf.ebnf` WRONG** |

⇒ **both defects are isolated outliers in a documentation set that already agrees with
the measurement**, and the meta-grammar's own comment block was contradicting the very
book PGEN ships for grammar authors. That is a much better position than "the docs are
wrong about ordered choice" — and it is only knowable because the sweep ran.

#### ⛔ THE THIRD DEFECT — the block's "Character classes" section, and it fails SILENTLY

Every one of the block's example lines was run through `--lint-grammar`, not eyeballed:

```
  LINT-FAIL  letter  := [a-zA-Z]        undefined rule 'Z' (+2 more) — hard error
  LINT-FAIL  special := [!@#$%^&*()]    hard error
  LINT-OK    digit   := [0-9]           ⚠️ and LINT-OK is the problem
```

`digit := [0-9]` lints clean and compiles to
`Quantified{ element: Sequence{[]}, quantifier: "?" }` — **an always-succeeding empty
optional that matches nothing.** `[ … ]` is the *optional-element* form
(`ebnf.ebnf:288`), so a reader who copies the documented character-class syntax gets a
hard error twice and a silently-vacuous rule once. All 15 example lines in the block were
measured; the other 12 are correct.

#### The three edits

1. **`docs/book/src/developer-architecture.md:17-21`** — the PEG bullet now says PGEN
   takes PEG's *determinism* property but **selects the winner by a branch tournament,
   not first-match commit**; names the default `longest_match`, the `ordered` opt-in and
   the measured `"a" | "a" "b"` / `"ab"` discriminator; keeps the (correct)
   greedy/possessive-repetition and `&`/`!` claims; and points at the three surfaces
   that already had it right. A sub-bullet records *why* this is load-bearing rather
   than pedantic — two linter deadness verdicts were unsound for exactly this reason.
2. **`grammars/ebnf.ebnf`** — `@precedence: {level: 5, associativity: "left"}` →
   `@precedence: 5` + `@associativity: left`, with the rejected object payload named;
   and the "Character classes" examples → regex atoms (`/[a-zA-Z]/`, `/[0-9]/`,
   `/[!@#$%^&*()]/`) with the bare-bracket trap spelled out, including the silent
   `[0-9]` case. (A stray trailing space on the old `# digit := [0-9] ` line goes too.)
3. **`docs/ebnf_parser_book/src/quantifiers.md`** — new section *"Repetition is
   POSSESSIVE — a quantifier never gives an iteration back"*, the author-facing
   consequence `.3b` Finding 3 showed is documented nowhere: the measured `bt := "a"*
   "ab"` rejection, the rule **never write `body* closer` where `body` can match
   `closer`**, and both working idioms (static negative-lookahead guard; the store guard
   for a dynamic closer) — with `.3b`'s monotone-store bound stated up front so nobody
   adopts idiom 2 without knowing it.

#### ⚠️ A measurement of mine was wrong, and the tool caught it

The first no-regression run reported the regenerated `ebnf` parser as **DIVERGED**. It
had not: I generated the two artifacts to *different* `--output` paths, and codegen
embeds the output path in the emitted source, so all 12,224 diff lines were that string.
Re-run with **both** the input path (`git stash` the edit in place) and the output path
held constant: **byte-identical, sha256 `f56f907556e8e27c07c7fcd45749b83581a5b427070f636f2de9d35a1694342c`.**
Recorded rather than quietly corrected — it is the same trap `BIN-BUILD-INTEGRITY.3`
already named ("same output path so path-embedded strings are controlled"), and a
byte-identity claim is worthless unless every non-semantic input is pinned.

- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — `.3b` measured `developer-architecture.md:21`'s
    ordered-choice claim false under the default policy (ROW-0 control, with
    `--lint-grammar` independently agreeing) and `ebnf.ebnf`'s `@precedence` object
    payload rejected by the validator; this leaf's own sweep then measured the
    block's character-class examples 2× hard-error / 1× silently vacuous.
  - [x] **ROOT CAUSE (WHY + WHERE)** — the `|` claim: default `@branch_policy` is
    `longest_match`, `ordered` is opt-in (measured both ways). The `@precedence` claim:
    `parse_semantic_numeric_list` accepts integers/integer-lists only
    (`annotation_validator.rs:656`; oracle test `:2862`, re-run PASS). The bracket
    claim: `[` is `optional_element` (`ebnf.ebnf:288`) and the *declared*
    `character_class` (`:232`) has **zero** consumers attributable to the meta-grammar —
    all 14 `character_class` hits in `rust/src` belong to the **regex** grammar's class
    surface (validator + perf/census bench labels), enumerated per file in the capture.
  - [x] **FIX** — three documentation edits, fix-hierarchy tier **declarative/doc only**;
    no engine or grammar *behaviour* touched.
  - [x] **ADDRESSED (verified)** — the three corrected character-class examples now
    lint **OK** (measured, all three); the corrected `@precedence`/`@associativity` pair
    uses the payload shape the validator's own message prescribes; the book bullet now
    matches the three surfaces that were already right, so the documentation set is
    self-consistent for the first time.
  - [x] **NO REGRESSION** — `grammars/ebnf.ebnf` is a tracked grammar, so it is treated
    as **code**: the regenerated parser is **BYTE-IDENTICAL** with both input and output
    paths controlled (sha256 `f56f9075…`), i.e. the edit is codegen-inert; the edited
    meta-grammar lints clean (0 errors, the 2 pre-existing left-recursion `[info]` lines
    unchanged); `mdbook_docs_gate` GREEN; all 9 doctrines PASS. No release, schema,
    ledger or contract movement.
  - [x] **LOCKSTEP** — both books updated in the same commit as the grammar comment;
    the third defect's *deeper* question (a production the meta-grammar declares and the
    shipping frontend does not implement) routed to a new leaf `.6` rather than absorbed
    into a doc fix; measurement banked in the `.3b` static driver so it re-runs.

### `.6` — `character_class`: declared in the meta-grammar, unimplemented by the frontend (`todo`)

- **Status: `todo`**, opened by `.5`'s sweep. ⭐ **This is a THIRD deadness class, distinct
  from both earlier findings:** `.1` found productions unreachable from the entry, `.2`
  found unreferenced-root orphans that no lint can see — `character_class`
  (`ebnf.ebnf:232`) is **reachable AND referenced AND lint-clean**, and still has zero
  implementing consumers, because the authoritative hand-written frontend
  (`scan_top_level_rules` / `convert_scanned_rule`) lowers `[` to the optional form
  instead. So the meta-grammar and the shipping frontend **disagree about what `[ … ]`
  means**, and the disagreement resolves silently in favour of the frontend.
- ⭐⭐ **DIRECTOR STEER (2026-07-26, session #209, on reviewing `.5`), verbatim:** *"shouldn't
  `digit := [0-9]` be written `digit := /[0-9]/`, instead. `/.../` is how we use regexes in
  EBNF, so not sure why one would create a regex without enclosing it inside `/.../`"*
  ⇒ **this decides the adjudication in substance: `/.../` is the ONE canonical way to write a
  character class in PGEN's EBNF, so `character_class` has no reason to exist.** The default
  arm is therefore **DELETE the production** (with its 5 helper rules
  `character_class_negation` / `character_class_content` / `character_class_item` /
  `character_range_item` / `normal_character`, if nothing else references them), NOT "mark it
  reserved" — reserving a second syntax for something `/.../` already covers would keep the
  ambiguity alive against the director's stated model. ⛔ *"Implement the class"* is now OFF
  the table.
- Scope: adjudicate on measurement, don't assume — (a) confirm the divergence directly (the
  generated `EbnfParser` vs the hand-written frontend on the same input; `ebnf_dual_run_diff` /
  `ebnf_frontend_dual_run_gate` are the instruments), (b) enumerate every referrer of
  `character_class` and its helpers before deleting anything (some helpers may be shared —
  `escaped_character` is referenced by `special_character` too), (c) delete per the steer, with
  the byte-identity no-regression check `.5` established (input AND output paths pinned) plus
  the self-hosting dual-run gate, since removing a *reachable* production is a real
  meta-grammar change and NOT codegen-inert the way `.5`'s comment edit was.
- ⚠️ Note for whoever takes it: this sits squarely in the family
  `ANNOTATION-PLACEMENT` named — *a check that cannot see a defect class must say so,
  not return green*. `--lint-grammar` reports 0 errors on `digit := [0-9]`.

### `.4` — Prioritized primitive roadmap

- **Status: `todo`** — **UNBLOCKED** (`.2` and `.3`+`.3b` are `done`). Rank by
  (coverage × tractability), attach a **cost model** to each per
  [[project_capability_growth_is_zero_cost_and_neutral]], and feed the horizon record's
  living axes list. Seeded by `.3b` with one already-named, already-priced-out
  candidate: **fact retraction / instance-scoped fact lifetime**, whose absence is the
  *sole* measured blocker on rows 10 and 11 and which is bounded by the four source
  facts in `.3b` Finding 2. `.2`'s informational orphan report is the other seed.

## Acceptance Criteria (tree)

- A measured, re-runnable expressiveness matrix — not prose.
- Every gap carries a cost model before it becomes a work item.
- The `ebnf.ebnf` unreachable set is adjudicated: wire it, or delete it, or document
  it as reserved — **not left as decorative surface** that misleads grammar authors.

## Evidence

- `docs/tasks/artifacts/lang_capability_audit/metagrammar_reachability.sh` — `.1` driver
- `docs/tasks/artifacts/lang_capability_audit/metagrammar_reachability.txt` — `.1` capture
- `docs/tasks/artifacts/lang_capability_audit/run_capability_probes.sh` — `.3b` driver
  (43 declared-verdict cases through the scratch slot; restores the slot on every exit)
- `docs/tasks/artifacts/lang_capability_audit/capability_probes.txt` — `.3b` capture
- `docs/tasks/artifacts/lang_capability_audit/static_surface_measurements.sh` — `.3b`
  zero-build source/CLI measurements (predicate registry, normalization, precedence)
- `docs/tasks/artifacts/lang_capability_audit/static_surface_measurements.txt` — capture
- `docs/tasks/artifacts/lang_capability_audit/probes/probe_default_layout.ebnf` — rows
  0/8/9/10/13
- `docs/tasks/artifacts/lang_capability_audit/probes/probe_ws_sensitive.ebnf` — row 11
