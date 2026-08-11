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
- **Current frontier (session #213): `.10.2`** — make `semantic_annotation.ebnf`
  composable and restore `ebnf.ebnf`'s delegated annotation sub-language. Then `.10.3`
  (retire the allowlist entry; **strictly after** `.10.2`). ✅ `.10.4` (`true`/`false`
  zero-width builtins) is **done** — it was independent and landed first. Other open
  leaves: `.2.1`, `.3c`, `.6`.

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
| 1 | **Parameterized productions** — `Expression[In, Yield, Await]`; one production family instantiated per parameter set | JS/ECMA-262 (pervasive), Ada, C++ | 🕳️ declared, **0 consumers** — ⛔ **and `.4` measured WORSE than unwired: the declared surface `expr[In, Yield]` compiles to `expr (In Yield)?` (an optional group) and LINTS CLEAN**, because `[` is the optional-element form (`ebnf.ebnf:288`). Cannot be built at the declared syntax ⇒ needs a surface decision |
| 2 | **Significant indentation / offside rule** — synthesize INDENT/DEDENT/NEWLINE from column state | Python, Haskell, YAML, Nim, F#, Raku heredocs | ❌ `@whitespace_sensitive` is grammar-global, not an offside primitive |
| 3 | **Lexer modes / sub-languages** — switch lexical rules mid-parse and nest them | Ruby `#{}`, Python f-strings, JS templates `${}`, shell/Perl/Raku heredocs, embedded SQL/regex | 🕳️ `lexer_mode` declared, **0 consumers** |
| 4 | **Lexical adjacency / no-layout boundaries** — "no white space (or no newline) between these two elements" | JS ASI `[no LineTerminator here]`, SV fn 44, Ruby `foo?`/`a +b`, Raku | ⚠️ notation exists (`[>! /\s/]`) but **generator-only**; inline per-seam form designed, unbuilt → [`LEX-ADJACENCY`](LEX-ADJACENCY.md) |
| 5 | **Case-insensitive keywords** — `BEGIN`/`begin`/`Begin` one token, identifiers still case-preserving | **VHDL**, Ada, SQL, Fortran, Pascal | ⚠️ **EXPRESSIBLE TODAY — `.4` re-measured**: `(?i:…)` inside a regex terminal works and is used **69 times across 216 rules** in `grammars/vhdl.ebnf`. So this is an **ERGONOMIC** gap (69× boilerplate), not an expressive one. The declared `case_control` alternative (`~i"begin"`) *misparses* — the `~` is dropped and `i` becomes a rule reference |
| 6 | **Error recovery / resync** — a spec whose *recovery* is normative | HTML (WHATWG defines it), any IDE-grade parser | ⛔⛔ **THE VERDICT WAS WRONG — `.4` measured that recovery SHIPS.** The meta-grammar productions really do have 0 consumers, but the capability exists under different names: registered directives `@recover`/`@sync`/`@panic_until` + 3 budgets (`semantic_directive_registry.rs:253/269/273`) that codegen turns into `parser.recover_with_hints(…)` (`ast_based_generator.rs:4222-4257`). ⛔ **But `@recover: true` CRASHES CODEGEN** unless all three budgets are also set ⇒ unusable in practice. → [`.8`](#8--codegen-recover-true-emits-invalid-rust-when-a-budget-is-unset-todo) |
| 7 | **Declaration-sensitive parsing** (the "lexer hack") — `T * x;` is a decl or a product depending on a *prior declaration* | C/C++, **SV** (`type_identifier`), VHDL | ✅ **PGEN's core strength** — semantic store `has_fact`/scopes, proven at SV scale |
| 8 | **Contextual / soft keywords** — a word is a keyword only in some positions | Python (`match`, `case`, `type`), JS (`let`, `async`, `of`), SV, C# | ✅ **HAVE — `.3b`-measured, and with NO keyword-exclusion tax.** All three readings of `match` parse against a deliberately *un-taxed* identifier token (4/4) |
| 9 | **Cover grammars / delayed disambiguation** — parse one shape, reinterpret later | JS `(a,b)` arrow-params vs paren-expr; C++ most-vexing-parse | ✅ **HAVE — `.3b`-measured** (4/4), including a disambiguator sitting past an arbitrarily deep ambiguous prefix |
| 10 | **Balanced / user-chosen delimiters** — closing delimiter determined by the opening one, incl. mirrored pairs | **Raku** `q//`/`qq{}`/`«»`, Perl `q{}`, Rust `r#""#` | ⚠️ **PARTIAL — `.3b` OVERTURNED the "suspected gap".** The OPEN delimiter set is expressible TODAY via a store-driven dynamic guard (6/6). The bound is **fact LIFETIME**, not lexical reach: repeated quotes collide on the monotone parse-global store (2 of 3 wrong) |
| 11 | **Here-documents** — terminator named *now*, body starts on the *next line* | Ruby, Perl, Raku, shell, PHP | ⚠️ **PARTIAL — `.3b` OVERTURNED the "suspected gap".** A single heredoc is fully correct (5/5) *including* the discriminating body-line-that-looks-like-another-tag case; the repeated-heredoc bound is **identical to row 10's** |
| 12 | **Operator-precedence declaration** — a precedence/associativity table instead of a hand-rolled cascade | VHDL, SV, C, most expression languages | ⚠️ **CONFIRMED + sharpened by `.3b`:** `@priority`/`@precedence` take an **integer** payload and rank the alternatives **inside one rule**; there is no cross-rule ladder. The ~16-level `rtl_const_expr` cascade is **measured** un-generatable within the bounded ladder (PARSE-HARNESS `.5.5`) ⇒ a real, already-felt pain |
| 13 | **Unicode identifier classes + normalization** | Python (PEP 3131, **NFKC**), JS ID_Start/ID_Continue, Raku | ✅ **(a) classes HAVE** — `.3b`-measured: `\p{XID_Start}`/`\p{XID_Continue}` compile and discriminate (5/5, incl. `café`/`变量`/reject `1abc`). ❌ **(b) NFKC normalization ABSENT** — 0 normalization crates, 0 NFKC/NFC/NFD mentions in `rust/src` |
| 14 | **Preprocessor / macro phase** | C/C++, **SV** `` `define ``, Rust macros | ⚠️ svpp is a *separate grammar*, not a composable phase |
| 15 | **Grammar composition** — import/extend another grammar | large LRMs, layered dialects | ⛔⛔ **THE VERDICT WAS WRONG — `.4` measured a REGRESSION, not a gap.** `import_statement`/`grammar_inheritance` are indeed unwired, but `include()` **ships, is recognized, and is then SILENTLY DISCARDED** (`ebnf_frontend.rs:152-155`) — while the Perl frontend implemented it (`perl/AST/Transform.pm:3234`) and the shipped author book documents it as working. Tracked-grammar damage measured: the SV profiled wrapper loads **3 of 1400 rules**; `ebnf.ebnf:18` has a **dangling** include. ✅ **REPAIRED by `.7`** (director-ordered): includes resolve for real at the single frontend chokepoint, the linter honours the graph, an unresolvable include is a hard error, and the SV wrapper now loads **1403** rules. The `import`/`extends` half stays unwired by design — `include()` is the canonical mechanism. |
| 16 | **Parse-time-mutable grammar** — the program *extends its own syntax* | **Raku** (slangs, custom operators), Perl 5 (`BEGIN`, prototypes) | ⛔ **HARD BOUND — out of scope by design.** The horizon record already scopes this: the realistic target is the *precise static subset*. Recording it keeps the boundary honest rather than pretending "any language" includes self-modifying ones. |

#### Reading the matrix

- ⛔ **"🕳️ declared-unwired is the dominant class (7 of 16 rows)" — RETRACTED by `.4`.**
  Three of those seven moved on re-measurement: rows **6** and **15** are shipped
  capabilities (one broken, one silently discarded) and row **5** is expressible today at
  69× boilerplate. The reading was wrong because it ranked on a **consumer count for the
  meta-grammar PRODUCTION NAME**, which `.4` measured lying in both directions — 0
  consumers does not mean the capability is absent (row 6), and a non-zero count can be a
  substring false positive (`parameter_list`, `named_capture`). **A name census is not a
  capability inventory.** The surviving declared-unwired rows are 1, 3, and the
  `import`/`extends` half of 15.
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

### `.4` — Prioritized primitive roadmap (`done`)

- **Status: `done`** (`PGEN-LANG-CAPABILITY-AUDIT-0009`, session #210). **Docs-only.**
  No Rust, no codegen, no tracked grammar, no generated artifact, no contract touched.
  Every probe grammar and every generated artifact was written to a `mktemp -d` the
  driver removes on exit — the repo is only READ.
- Rank by (coverage × tractability × **ergonomic distance**), attach a **cost model** to
  each per [[project_capability_growth_is_zero_cost_and_neutral]], and feed the horizon
  record's living axes list. Seeded by `.3b` with **fact retraction / instance-scoped
  fact lifetime** and by `.2` with the informational orphan report.

#### ⭐⭐⭐ THE HEADLINE: pricing the roadmap RE-MEASURED the matrix, and FOUR rows were wrong

`.4`'s job is to price gaps. A price computed from `.3`'s prose verdicts would inherit
their errors, so [[feedback_read_prior_art_before_designing]]'s **"RE-MEASURE before
citing engine behaviour"** clause was applied to every row before it was ranked. Four
rows moved — and **two of them are not gaps at all but SHIPPED capabilities**, one of
which is *broken* and one of which is *silently discarded*.

Re-run: `bash docs/tasks/artifacts/lang_capability_audit/run_primitive_pricing_probes.sh`
(exit 0, **0 declared-verdict divergences**, byte-identical on re-run).

| row | `.3`/`.1` said | `.4` MEASURED | why it matters |
|---|---|---|---|
| **15** composition | 🕳️ `import_statement` declared, 0 consumers | ⛔ **`include()` SHIPS, is RECOGNIZED, and is then DISCARDED** — silently, exit 0 | corrupts **tracked** grammars today |
| **6** error recovery | 🕳️ declared, 0 consumers; *"NOT yet in PGEN"* | ⛔ **recovery SHIPS as registered semantic directives — and `@recover: true` CRASHES CODEGEN** | an entire matrix row was written off as absent |
| **1** parametric | 🕳️ declared, 0 consumers | ⛔ **the declared surface `expr[In, Yield]` silently miscompiles to `expr (In Yield)?` and LINTS CLEAN** | the surface cannot be built as declared |
| **5** case-insensitive | 🕳️ declared, 0 consumers | ⚠️ **expressible today — at 69 hand-written `(?i:…)` in one grammar**; the declared `~i"…"` *misparses* | an ERGONOMIC gap, not an expressive one |

⇒ **the "declared-unwired consumer count" that `.1` and `.3` ranked on is a NAME census,
never a capability inventory**, and this leaf measured it lying in *both* directions:

- **FALSE NEGATIVE** — every error-recovery production reads 0 consumers while the
  capability ships under completely different names (`@recover`/`@sync`/`@panic_until`).
- **FALSE POSITIVE** — `parameter_list` read **2** and `named_capture` read **29** on a
  bare substring grep; word-anchored both are **0** (`parse_macro_parameter_list` in
  `sv_preprocessor.rs:1079`; the *regex* grammar's `named_captures` in
  `stimuli_generator.rs`). `.1`'s verdict survives, its **method** does not — the driver
  now prints both columns so the lesson cannot be lost.

#### ⛔ FINDING 1 — `include()` is recognized and then thrown away, and it reaches TRACKED grammars

`rust/src/ebnf_frontend.rs:152-155`, verbatim — the authoritative hand-written frontend:

```rust
if is_include_directive(trimmed) {
    idx += 1;
    continue;
}
```

Recognized (`is_include_directive`, `:246`, covering `include(` / `include_file(` /
`include_dir(` / `file(` / `dir(`) and **skipped**. Nothing is resolved, nothing is
spliced. Measured on a two-file probe: **1 rule loaded, the included rule absent, exit 0,
zero diagnostics.**

⭐ **It is not a greenfield gap — it is a REGRESSION.** The capability was *implemented*
in the retired Perl frontend (`perl/AST/Transform.pm:3234` `process_ast_includes`, with
`resolve_include_files`, `resolve_include_directory`, recursive processing and cycle
handling) and is documented across **two live surfaces**: `docs/EBNF_INCLUDE_SYSTEM.md`
(12.8 KB) and — far worse — a whole chapter of the **shipped grammar-author book**,
`docs/ebnf_parser_book/src/includes.md`, which states verbatim that the directives *"are
recognized by the EBNF frontend (`rust/src/ebnf_frontend.rs`) and **resolved into a single
combined grammar before code generation**"* and then tells the author (`:78-80`) that
includes — not `import`/`extends` — are *"the supported mechanism"*.

⛔ **Measured blast radius on TRACKED grammars:**

| grammar | loaded | should be | loss |
|---|---|---|---|
| `grammars/systemverilog_lrm_profiled_wrapper.ebnf` (body = one `include`) | **3 rules** | 1400 | **1397 (99.8%)**, exit 0, no diagnostic |
| `grammars/ebnf.ebnf:18` — `include(semantic_annotations)` | — | — | ⛔ **the target file does not exist**; a dangling include nothing can ever detect |

⚠️ **And the diagnostic actively misleads.** With the include dropped, the linter reports:

> *"rule 'start' references UNDEFINED rule 'digit' … **or fix the reference (likely a
> typo)**"*

The author wrote a correct include and is sent hunting for a typo — the `.3.10` law
(*an innocent rule gets named*) reappearing at frontend scale.

⭐⭐ **DIRECTOR RULING (2026-07-26, session #210, on seeing this measurement), verbatim:**
*"The linter should honour EBNF include() graph trees, of course."* ⇒ the adjudication is
decided in substance: **includes are to be made real, not deleted and not documented
away** — and the linter specifically must resolve the include GRAPH before it reports
undefined references, so the misattributed diagnostic disappears with the root cause
rather than being reworded. Routed to new leaf [`.7`](#7--make-include-real-end-to-end-and-make-the-linter-honour-the-include-graph-todo).

#### ⛔ FINDING 2 — error recovery SHIPS, and its enabling annotation crashes codegen

Row 6 was written off quoting the horizon record's *"NOT yet in PGEN"*. Measured, the
capability is **built, registered and wired**:

- **declared directives** — `semantic_directive_registry.rs:253` `recover`, `:269` `sync`,
  `:273` `panic_until` (+ `recover_budget` / `recover_parse_budget` /
  `recover_global_budget`), resolved by `effective_rule_recovery_enabled` (`:805`);
- **consumed by codegen** — `rule_recovery_hints` (`ast_based_generator.rs:9505`) feeds
  the multi-branch failure path (`:4222-4257`), which emits
  `parser.recover_with_hints(rule, start, sync, panic_until, budgets…)` and a
  `🛟 Rule '…' recovered from branch failure` trace;
- **pinned by a unit test** — `:12381` asserts `sync_tokens == [";", "end"]`.

⛔ **And `@recover: true` does not compile.** Control matrix, all five cells measured:

```
  codegen=ok    control: no recovery annotations
  codegen=ok    @sync alone                          (inert without @recover)
  codegen=FAIL  @recover: true, NO budgets                    <- DEFECT
  codegen=FAIL  @recover: true + 1 of 3 budgets                <- DEFECT
  codegen=ok    @recover: true + ALL 3 budgets       (the only usable form)
```

**ROOT CAUSE (WHY + WHERE), from the emitted token dump — not inferred:**

```
recover_with_hints ("stmt" , parse_start , & [] , & [] , , , ,)
                                                      ^^^^^^^ three empty slots
```

`recover_budget` / `recover_parse_budget` / `recover_global_budget` are `Option<usize>`
interpolated straight into `quote!` at `ast_based_generator.rs:4240-4252`. `quote`'s
`ToTokens for Option<T>` emits **nothing** for `None`, so the call collapses to invalid
Rust and the pipeline aborts with *"Failed to parse generated TokenStream: expected an
expression"* pointing at **byte 0** — a location that names nothing. The emitted helper's
own signature takes `usize`, not `Option<usize>`, so there is no "no budget" value to
emit either. Routed to new leaf [`.8`](#8--codegen-recover-true-emits-invalid-rust-when-a-budget-is-unset-todo).

⇒ **the practical state of row 6: PGEN has panic-mode-with-sync-tokens recovery, with
per-rule / per-parse / global budgets, and no grammar can switch it on** unless the
author happens to also set all three budgets. Zero tracked grammars do.

#### ⭐ FINDING 3 — the ZERO-COST acceptance test, measured rather than asserted

[[project_capability_growth_is_zero_cost_and_neutral]] requires *non-users pay ZERO* —
byte-identical, not "negligible". Both P0/P1 families were measured against it:

| capability | non-user cost | measured |
|---|---|---|
| error recovery | the `fn recover_with_hints` **helper** is emitted unconditionally, but **0 call sites** in a non-recovering parser (1 in an opted-in one) | inert — nothing on any parse path |
| semantic store (rows 10/11) | `generated/json_parser.rs`: **0** store hits across **13,707** lines (vs 24 in the SV parser) | pay-per-use — a fact-free grammar emits no store code at all |

⇒ a fact-lifetime primitive and a recovery repair both inherit criterion (1) **by
construction**; neither needs a new inertness argument.

#### ⛔ FINDING 4 — the declared `parametric_rule` surface cannot be built as declared

`ebnf.ebnf:595` declares `parametric_rule := rule_name "[" parameter_list "]"` — i.e.
the author writes ECMA-262's `Expression[In, Yield]`. Measured, that input compiles to:

```
[['rule','start'], ['rule_reference','expr'],
 ['group_open','('], ['rule_reference','In'], ['rule_reference','Yield'], ['group_close',')'],
 ['operator','?']]                                   #  expr ( In Yield )?
```

**`undefined_references=0` — LINT-CLEAN.** The comma is swallowed and the parameter list
becomes an *optional group*, because `[` is the optional-element form (`ebnf.ebnf:288`).
This is **exactly `.5`'s silent `digit := [0-9]` defect**, on the single most structural
JS construct: the highest-value declared-unwired surface in the whole audit is ambiguous
by construction with a core existing one. ⇒ row 1 cannot be implemented at the declared
syntax; it needs a director-level surface decision, in the same family as `.6`.

#### The roadmap — ranked and priced

Ranked by (**coverage** × **tractability** × **ergonomic distance**). Every cost model
states the three acceptance-test answers in order: *non-user cost / user cost / runtime
cost*. ⛔ **P0 outranks every new primitive: a shipped capability that does not work is
worse than an absent one, because the audit itself was misled by both of them.**

| # | item | row(s) | coverage | tractability | ergonomic distance | cost model | owner |
|---|---|---|---|---|---|---|---|
| **P0-1** | **make `include()` real + linter honours the include graph** | 15 | composition (**director-confirmed in scope** #209); multi-file LRMs, layered dialects; partially unblocks row 14 | **MEDIUM** — semantics designed AND once implemented (Perl); the frontend already *recognizes* the directives, it must resolve+splice instead of skip | **∞ today, and NEGATIVE** — the book's recommended mechanism is a no-op and the diagnostic blames the wrong rule | frontend/**codegen-time** splice ⇒ **zero** for a grammar with no include (identical IR by construction) / author pays parse-time of the included files / **zero** runtime | [`.7`](#7--make-include-real-end-to-end-and-make-the-linter-honour-the-include-graph-todo) — ⭐ **DIRECTOR-ORDERED** |
| **P0-2** | **fix `@recover: true` codegen crash** | 6 | HTML-class recovery, IDE-grade parsers — the whole of row 6 | ⭐ **HIGHEST** — one `quote!` interpolation of three `Option<usize>`, one call site | **∞ today** (unusable unless all 3 budgets set; failure is an internal crash at byte 0) | **zero** (measured: 0 call sites when unused) / codegen-time / runtime only on an opted-in rule's failure path | [`.8`](#8--codegen-recover-true-emits-invalid-rust-when-a-budget-is-unset-todo) |
| **P1-3** | **fact retraction / instance-scoped fact lifetime** | 10, 11 | Raku delimiters, heredocs, and every "closer named by the opener" form (Rust `r#""#`, Lua `[==[…]==]`) | **MEDIUM-HIGH** — bounded by `.3b` Finding 2's four source facts; needs a retraction directive *or* an instance-keyed (not depth-keyed) scope index (`semantic_runtime.rs:2170`) | **HIGH** — a 4-rule store-guard idiom that is *wrong on the second instance* (2 of 3 repeated cases) | **zero** (measured: store is pay-per-use) / grammar-author writes the directive / runtime — but the context-sensitivity is **semantically unavoidable**, which criterion (3) explicitly admits | `.4` seed → new leaf when scheduled |
| **P1-4** | **declarative case-insensitive keywords** | 5 | VHDL, Ada, SQL, Fortran, Pascal — a whole language family | ⭐ **HIGHEST of the new surfaces** — pure codegen-time lowering to `(?i:…)`, which already works | **69 hand-written `(?i:` across 216 rules** in `grammars/vhdl.ebnf` — measured, the clearest awkwardness number in the audit | **byte-identical** for a grammar without the directive / codegen-time lowering / **zero** runtime | unscheduled — ⛔ see the surface warning below |
| **P2-5** | **parameterized productions** | 1 | JS/ECMA-262 (pervasive), Ada, C++ | **LOW-MEDIUM** — ⛔ blocked on a **surface decision**: the declared `[…]` syntax miscompiles lint-clean (Finding 4) | manual expansion; up to 2^k rules per k parameters (*derived*, not measured) | **monomorphize at codegen** ⇒ zero for non-users / user pays codegen time + grammar size / **zero** runtime | unscheduled — needs a director surface call |
| **P2-6** | **cross-rule precedence ladder** | 12 | VHDL, SV, C, most expression languages — **already-felt pain** | **MEDIUM** | the ~16-level `rtl_const_expr` cascade, measured un-generatable within the bounded ladder (PARSE-HARNESS `.5.5`) | codegen-time expansion into the cascade PGEN already emits ⇒ zero-cost by construction | unscheduled |
| **P2-7** | **offside rule + lexer modes** | 2, 3 (with 4) | Python/Haskell/YAML/Nim; Ruby `#{}`, f-strings, JS `${}` | **LOW** — the one place `.3`'s "parse state feeds the lexical layer" hypothesis is **still unrefuted** | ❌ / 🕳️ | to be priced — the only family where a *runtime* channel may be unavoidable ⇒ price against criterion (2) **before** designing | gated on `LEX-ADJACENCY.2` (row 4 is the same seam, owned there) |
| **P3-8** | **NFKC identifier normalization** | 13b | Python PEP 3131 only, and only for identifier *equality* | **HIGH** (a crate + one pass) but **LOWEST coverage** | ❌ absent (0 crates, 0 mentions) | opt-in pass ⇒ zero for non-users | unscheduled |
| **T-9** | **informational unreferenced-root ("orphan") report** | — | tooling, not a primitive | **HIGH** — extend `detect_unreachable_rules` (`grammar_wellformedness.rs:308`), kept **separate** from the hard `unreachable_rules` error so multi-entry safety survives | — | lint-only, opt-in | `.2` seed — ⭐ **newly motivated**: it is the only instrument that would have surfaced `ebnf.ebnf:18`'s dangling include and the decorative productions |

⛔ **DELIBERATE NON-COMMITMENTS — on the roadmap as *excluded*, so the boundary stays honest:**

- **Extensibility.** Director #209 recorded it as an explicit NON-commitment. ⛔ Nothing
  above may be cited as authorization for it, and the composability mandate (P0-1) is
  **not** a back door — the director scoped composability IN and extensibility OUT in the
  same sentence.
- **Row 16 — parse-time-mutable grammar** (Raku slangs, Perl 5 `BEGIN`, Prolog `op/3`).
  HARD BOUND, out of scope by design.
- **The declared syntaxes `~i"…"` (`case_control`), `[a-z]` (`character_class`), and
  `import`/`extends`.** All three are superseded by a canonical existing form — `(?i:…)`
  and `/.../` per the `.6` director steer, `include()` per P0-1 — and all three
  *misparse* today. ⇒ ⭐ **`.6` should widen from `character_class` alone to the whole
  decorative-syntax set**, with one carve-out: `parametric_rule` is a capability the
  roadmap **wants** (P2-5), so it must be **re-surfaced, never deleted**.

#### ⚠️ What this leaf does NOT claim

- The 2^k figure for P2-5 is **derived from the parameter count, not measured** — no
  ECMA-262 grammar is vendored in this repo. Marked as such in the table.
- P2-7 is the one row still carrying `.3`'s unrefuted hypothesis; it is **ranked, not
  priced**, and says so.
- Nothing here re-measures rows 7/8/9/13a/14/16 — they keep their `.3`/`.3b` verdicts.
- **No claim that recovery WORKS end-to-end.** Measured: codegen emits the call with all
  three budgets set. Whether `recover_with_hints` then recovers *correctly* on a
  malformed input is unmeasured and belongs to `.8`.

- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — the tree's own acceptance criterion is *"every gap
    carries a cost model before it becomes a work item"*, and `.4` was the last leaf
    blocking the director's #209 "build capability" directive.
  - [x] **ROOT CAUSE (WHY + WHERE)** — per priced row, tool output + source location:
    include drop `ebnf_frontend.rs:152-155` (+ blast radius 3 vs 1400 rules measured on a
    tracked grammar); recovery crash `ast_based_generator.rs:4240-4252` proven by the
    emitted token dump's three empty argument slots, isolated by a 5-cell control matrix;
    `parametric_rule` collision `ebnf.ebnf:595` vs `:288` proven by the compiled IR;
    case-insensitivity 69 `(?i:` in `grammars/vhdl.ebnf`; store pay-per-use 0/13,707 in
    `generated/json_parser.rs`.
  - [x] **FIX** — N/A (roadmap leaf). Output is the ranked+priced roadmap, four corrected
    matrix rows, and leaves `.7`/`.8`. ⚠️ **One exception, declared:** the author book's
    include chapter and `docs/EBNF_INCLUDE_SYSTEM.md` carry a measured-false claim on a
    **shipped, gated** surface. Leaving them until `.7` lands would be exactly the book
    drift the director forbids, so both gain a dated status admonition — the *claim* is
    corrected, the *design* is untouched and stays `.7`'s to deliver.
  - [x] **ADDRESSED (verified)** — `run_primitive_pricing_probes.sh`, every probe
    carrying its declared verdict: **exit 0, 0 divergences, byte-identical on re-run**.
    Two of `.1`'s consumer counts re-measured word-anchored and the false-positive
    mechanism printed in the capture rather than silently corrected.
  - [x] **NO REGRESSION** — docs-only. No `rust/`, no codegen, no generated artifact, no
    contract, no tracked grammar. Probe grammars and generated parsers all live in a
    `mktemp -d` removed by the driver's `trap`; `git status` shows only intended docs.
    No release/schema/ledger movement.
  - [x] **LOCKSTEP** — `.3` matrix rows 1/5/6/15 rewritten with their measured verdicts
    (not appended-to — the wrong verdicts are struck); the horizon record gains the
    director's include ruling; `.6` widening recommended with the `parametric_rule`
    carve-out stated; both defects routed to `.7`/`.8` rather than fixed in a docs leaf.

#### ⭐⭐⭐ DIRECTOR STEER (2026-07-26, session #209) — the bar is EAGERNESS, not sufficiency

**Verbatim:** *"PGEN EBNF support shall so powerful, flexible that we should be eager to
handle the creation of even more tricky languages"* — clarified moments later: *"… more
tricky languages parsers."* Banked in full at [[project_horizon_universal_parser]]. Three
consequences that bind this leaf:

1. ⭐ **Expressive AWKWARDNESS becomes a defect class.** The old bar ("a correct grammar must
   be WRITABLE") is met by anything merely *possible* to write. This one is not: a
   capability reached only through a multi-rule workaround is not something an author takes
   on **eagerly**. ⇒ **`.3b`'s rows 10 and 11 do NOT close on their ✅.** They are expressible
   — via a four-rule store-guard idiom carrying a monotone-store caveat — and under this bar
   that keeps them open as *ergonomic* gaps even though the *expressive* gap is answered.
   ⚠️ Whoever runs `.4` must not read `.3b`'s "OVERTURNED — expressible today" as "done".
2. ⭐ **"Powerful AND flexible" are two properties, and flexibility is the thinner one.** A
   capability that exists only at the wrong GRANULARITY counts as a gap. This is the third
   time the director has made this point in two sessions (per-SEAM not per-rule
   [[project_ebnf_steers_the_engine_at_full_granularity]]; every user-controllable feature
   declared in the EBNF [[project_ebnf_is_single_source_of_truth]]; now flexibility itself).
3. ⭐ **The matrix must keep GROWING** — "even more tricky languages" is an instruction not to
   close the audit at `.3`'s 16 rows. Each new notoriously-hard language is a column that may
   surface a row nothing else does. ⇒ new leaf `.3c` below.
- ⛔ Unchanged and still binding: duality-completeness, and the zero-cost/neutrality
  acceptance test — eagerness never buys an exemption from "non-users pay ZERO".

### `.7` — Make `include()` real end-to-end, and make the linter honour the include graph (`done`)

- **Status: `done`** (`PGEN-LANG-CAPABILITY-AUDIT-0010`, session #210). **Code change** —
  `rust/src/ebnf_frontend.rs` + `grammars/ebnf.ebnf` (a tracked grammar).
- ⭐⭐ **DIRECTOR-ORDERED (2026-07-26, session #210).** First, verbatim: *"The linter should
  honour EBNF include() graph trees, of course."* Then, on reading `.4`: *"Full support for
  include() must be reinstated for every logic that consume EBNF, not sure why this wasn't
  detected earlier and acted upon."* ⇒ **two deliverables: the fix, AND an account of the
  detection failure.** Both below. Includes are made real — not deleted, not documented
  away, not replaced by a new surface.

#### ⭐⭐⭐ WHY THIS WAS NEVER DETECTED — three independent maskings, all measured

The answer is not "nobody looked." A gate exists whose whole purpose is to catch exactly
this class, and **three separate things had to line up for it to stay green**.

**1. The one instrument that could see it was aimed at the single grammar where the bug
cancels itself out.** `ebnf_frontend_dual_run_diff_gate.sh` diffs the **Perl** frontend
against the **Rust** frontend over `("ebnf" "json" "regex")`, comparing `perl_rule_count`
vs `rust_rule_count` and the raw_ast rule-name sets. `grammars/ebnf.ebnf` carried
`include(semantic_annotations)`, and Perl *does* resolve includes — so this gate should
have gone red the moment the Rust frontend stopped.

It did not, because **the include target does not exist**. Perl's `resolve_include_files`
pushes a path only `if -f $full_path`; no match ⇒ empty list ⇒ Perl contributes **zero**
rules, exactly like the Rust frontend that never looked. Both report 131, parity, green.
⇒ **the dangling include masked the dropped include** — two unrelated defects cancelling
in the one gate designed to catch the class.

**2. Even a valid target would not have resolved on the Perl side.** From Perl's own trace,
the search path it actually builds under `tools/ebnf_to_json.pl` is:

```
🔍 Include search paths: <EBNF_INCLUDES entries>, .
```

— **no base directory.** `load_ebnf_spec_from_content` (`Transform.pm:3509`) does call
`process_ast_includes` (`:3542`), but the CLI passes no base dir, so the documented *"base
directory (containing the main grammar file)"* rule was inert. A `grammars/…ebnf` target
would have been sought in the repo root and missed. Proof the wiring is otherwise live:
with `EBNF_INCLUDES` pointed at a probe directory, Perl emits
`📁 Included file: …/common.ebnf` and returns **2 rules** where Rust returned 1.

**3. The grammar actually being destroyed is in no gate at all.**
`grammars/systemverilog_lrm_profiled_wrapper.ebnf` — losing 1,397 of 1,400 rules — has
**zero** consumers across `rust/`, `scripts/` and `.github/` (measured). Nothing loads it,
so nothing could notice.

⚠️ **And no gate anywhere loaded a multi-file grammar.** The include system had no positive
test of any kind; its only tracked users were one grammar in no gate and one dangling
directive. ⇒ this leaf ships `run_include_resolution_probes.sh` as that missing coverage.

⭐ **The generalizable lesson** (routed to layer C): **a differential gate proves only that
two implementations AGREE, never that either is RIGHT.** Both sides were wrong in the same
direction and the diff was clean. A differential needs at least one case whose expected
value is asserted **independently of both implementations** — which is exactly what the new
driver's declared verdicts are. Same family as `ANNOTATION-PLACEMENT`'s principle: *a check
that cannot see a defect class must say so, not return green.*
- **The defect, measured** (`.4` Finding 1): `ebnf_frontend.rs:152-155` recognizes every
  include directive and `continue`s past it. Nothing resolves, nothing splices, exit 0,
  zero diagnostics. Blast radius already on tracked grammars —
  `systemverilog_lrm_profiled_wrapper.ebnf` loads **3 of 1400 rules (99.8% lost)** and
  `grammars/ebnf.ebnf:18` carries `include(semantic_annotations)` pointing at **a file
  that does not exist**.
- **PRIOR ART (already searched — do not redo, extend):** the semantics are *designed and
  once implemented*. `perl/AST/Transform.pm:3234` `process_ast_includes` +
  `resolve_include_files` (`:3381`) + `resolve_include_directory` +
  `process_ast_includes_from_content` (`:3473`) cover search paths, recursion and cycle
  handling; `docs/EBNF_INCLUDE_SYSTEM.md` is the exhaustive reference;
  `docs/ebnf_parser_book/src/includes.md` is the author-facing contract this leaf must
  make true. `is_include_directive` (`ebnf_frontend.rs:246`) already fixes the accepted
  spellings (`include(`, `include_file(`, `include_dir(`, `file(`, `dir(`).
#### The fix — one chokepoint, so "every logic that consumes EBNF" is structural, not per-tool

⭐ The director's *"every logic that consume EBNF"* is satisfied **by construction**, and
that is worth stating precisely: `rust/src/ebnf_frontend.rs` exposes exactly **two** public
functions, and every EBNF consumer in the repository goes through them —
`main.rs` (CLI: codegen / lint / stimuli / raw-AST export), `ast_pipeline/stimuli_generator.rs`,
`parse_harness_interpreter.rs` and `parse_harness_equivalence.rs`. Resolution was therefore
placed in `parse_ebnf_text_to_raw_ast_envelope`, **above** `scan_top_level_rules`, so no
caller can opt out and no future caller can forget.

- **`scan_rules_with_includes`** — scans the main file's rules, then appends the rules its
  includes contribute.
- **`collect_included_rules`** — resolves each directive, recursing, carrying a `visited`
  set of canonical paths.
- ⭐ **Rules are spliced, not text.** Each file is scanned *independently* and the rule
  lists concatenated, so a trailing `@annotation` or `[> …]` directive at the end of one
  file cannot bind the first rule of the next — a bug text concatenation would have had.
- ⭐ **Main-file rules come FIRST.** Downstream reachability and linting treat
  `rule_order[0]` as the canonical entry; letting an included file supply rule 0 would
  silently re-root the grammar. Measured and pinned (`top, l, shared, r`).
- Search path: including file's own dir → the dirs of its includers → `EBNF_INCLUDES` →
  `EBNFLIB` → `.`, deduped in order so the nearest definition wins. `.ebnf` appended when
  absent; absolute paths bypass the search; `include_dir()` takes every `*.ebnf`
  alphabetically; both quoted and **bare** specs accepted (both spellings occur in tracked
  grammars).

**Two deliberate departures from the Perl behaviour, both recorded rather than silently
inherited:**

1. **The including file's own directory is always on the search path.** Perl documented
   this and never did it (masking #2 above).
2. ⛔ **An unresolvable include is a HARD ERROR** naming the spec, the directive and the
   search path tried. Perl returned an empty match list and carried on — the precise
   behaviour that let a dangling include hide in the meta-grammar. A silent partial resolve
   would have preserved the disease.

Also: the author book has always promised *"circular includes are handled gracefully —
each file is processed once"*. The Perl implementation it documents has **no such guard**
(measured: no visited set anywhere in the recursion). The guarantee is therefore **new,
not restored**, and it is now tested.

#### `grammars/ebnf.ebnf:18` — adjudicated on measurement, then removed

> ⛔⛔ **THIS SUBSECTION IS WRONG AND IS SUPERSEDED BY `.10.1`. Retained verbatim
> below as the record — do not act on it.** Two of its three bullets do not survive
> measurement: the target file **does** exist under the singular spelling, and
> `ebnf.ebnf` is **not** self-contained (`undefined_references=0` is produced by a
> codegen allowlist, not by soundness). Read `.10` / `.10.1` before touching this
> line again. Corrected in place per `.10` deliverable 4.

The stale `include(semantic_annotations)` is deleted (replaced by a comment recording why).
Decided on evidence, not convenience:

- ⛔ **FALSE (`.10.1`)** — ~~`grammars/semantic_annotations.ebnf` **has never existed**~~;
  true of the **plural** spelling only. The singular `grammars/semantic_annotation.ebnf`
  exists, defines `semantic_annotation`, and its own header advertises the plural
  spelling (`# semantic_annotations.ebnf`, `# Usage: include(semantic_annotations)`) ⇒
  a naming-drift typo for a file that is right there, not a reference to nothing.
- ⛔ **FALSE (`.10.1`)** — ~~`ebnf.ebnf` lints `undefined_references=0` at 131 rules ⇒ it
  is **self-contained** and needs nothing composed in~~; the grammar has **3 live
  dangling references** (`grammar_file`, `annotation_list`, `inline_semantic_annotation`
  all name `semantic_annotation`, which no rule defines). The lint reads 0 because
  `"semantic_annotation"` is hard-coded into `NATIVE_UNRESOLVED_REFERENCE_BUILTINS`,
  the const the linter consumes as its allowlist. **The evidence cited here was
  produced by the very masking this leaf was supposed to be removing.**
- ✅ **STANDS (`.10.1`, re-measured)** — pointing it at the real
  `semantic_annotation.ebnf` would import 112 rules that **collide on 15 names**
  already defined in `ebnf.ebnf`. `.10.1` sharpens it: **11 of the 15 are genuine
  semantic CONFLICTS**, not duplicate spellings, so the composition is a real design
  problem — but "actively harmful" was the wrong conclusion to draw from it. The right
  one is that `semantic_annotation.ebnf` **is not composable as it stands**, which is a
  reason to make it composable (`.10.2`), not a reason to delete the delegation.

- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — `.4` measured the silent drop; this leaf reproduced it on
    a 2-file probe (1 rule, exit 0, no diagnostic) and on two tracked grammars.
  - [x] **ROOT CAUSE (WHY + WHERE)** — `ebnf_frontend.rs:152-155`: `is_include_directive`
    matched, then `idx += 1; continue`. No resolution existed anywhere on the Rust path.
    The *detection* failure is separately root-caused above, with all three maskings
    measured (dual-run parity via a dangling target; Perl's base-dir-less search path
    printed from its own trace; the SV wrapper's zero consumers).
  - [x] **FIX** — resolution at the single frontend chokepoint + the stale meta-grammar
    directive removed. Fix-hierarchy tier: **engine capability restored**, no workaround.
  - [x] **ADDRESSED (verified)** — `run_include_resolution_probes.sh`, **17 declared-verdict
    cases, exit 0, 0 divergences**: two-file compose 1→**2** rules with
    `undefined_references` 1→**0** (⭐ the linter now honours the graph — the director's
    specific requirement, and the misattributed *"likely a typo"* is gone with its cause);
    SV profiled wrapper **3 → 1403** rules; unresolvable include **exit 0 → exit 1** with a
    named diagnostic; cycle terminates; diamond composes the shared file exactly once;
    depth-3 + bare spec + relative subdir; `include_dir()` alphabetical; entry rule pinned;
    and all four consumer surfaces re-measured end-to-end (codegen emits `fn parse_digit`,
    stimuli generate digits, raw-AST export shows `start,digit`).
  - [x] **NO REGRESSION** — five no-include grammars pinned unchanged (json 9 / regex 269 /
    vhdl 216 / rtl_frontend 169 / rtl_const_expr 48). ⭐ **The meta-grammar edit is proven
    CODEGEN-INERT**: a genuine HEAD baseline was built (HEAD `ebnf_frontend.rs` + HEAD
    `ebnf.ebnf`), the parser generated, then the change restored and regenerated **with the
    input path AND the output path pinned** per `.5`/`BIN-BUILD-INTEGRITY.3` —
    **byte-identical, sha256 `c0f26ff7eeb92b03e0424a3740b1f8a13fd8445c55071a06ac65b99e1e124fee`,
    156,863 lines both sides.** `ebnf_frontend_dual_run_gate` GREEN with `ebnf` at
    **131/131 Perl-vs-Rust parity**; `ebnf_parser_book_gate` GREEN (tracked HTML
    re-rendered); clippy strict source stage **0 errors and 0 hits on the changed file**
    (the generated-stage debt is entirely in `systemverilog_parser.rs` 158 /
    `rtl_frontend_parser.rs` 132 / +2, none regenerated here); all 9 doctrines PASS. No
    release, schema, ledger or contract movement — the frontend composes rules *before*
    codegen, so no shipped parser's AST changes.
  - [x] **LOCKSTEP** — both book admonitions `.4` planted are **removed and replaced with
    accurate descriptions** (this leaf's declared definition of done), including the new
    hard-error rule, the rule-order guarantee and the corrected search path;
    `docs/EBNF_INCLUDE_SYSTEM.md` gains an ACTIVE status note that flags the two departures
    from its Perl-era text; `.3` matrix row 15 updated; the differential-gate lesson routed
    to the horizon record.

⚠️ **Left open deliberately, and named rather than absorbed:** cross-file **rule-name
collision** detection. The book warns about it (`includes.md`) and the combined grammar
does not yet lint for it — a duplicate definition across two files currently resolves
silently. It is a real gap in the same family this tree keeps finding, and it is **not**
what the director ordered here. → new leaf `.9`.

### `.8` — Codegen: `@recover: true` emits invalid Rust when a budget is unset (`done`)

- **Status: `done`** (`PGEN-LANG-CAPABILITY-AUDIT-0012`, session #211, 2026-07-26).
  **Code change**: `rust/src/ast_pipeline/ast_based_generator.rs` + 1 new book chapter.
  No release/schema/ledger/contract movement — **all 11 generated parsers are
  byte-identical** (proof below), because no tracked grammar uses `@recover`.

#### ⭐⭐⭐ THE HEADLINE: the defect was WORSE than the leaf recorded — `@recover` NEVER worked, in ANY configuration

`.4` measured **codegen** and honestly claimed nothing beyond it ("`+ all 3` → OK"
meant *the TokenStream parsed*). This leaf measured the **next layer down**, and the
"only usable form" is not usable either:

| `@recover` configuration | `.4` verdict (codegen) | `.8` verdict (**does the parser COMPILE?**) |
|---|---|---|
| no annotations | ok | ✅ compiles |
| `@sync` alone (inert) | ok | ✅ compiles |
| `@recover: true`, no budget | **FAIL** — "expected an expression" | never reached codegen |
| `+ 1 of 3` budgets | **FAIL** | never reached codegen |
| `+ ALL 3` budgets — *"the only usable form"* | ok | ⛔ **`error[E0308]` ×3 — DOES NOT COMPILE** |

⇒ **row 6 was a 100% dead shipped capability**, not a partially-broken one. The
`ToTokens for Option<T>` trap is wrong in *both* arms: `None` emits **nothing**
(empty argument slot ⇒ codegen abort) and `Some(4)` emits the **bare payload**
`4usize` against a parameter declared `Option<usize>` (⇒ rustc E0308). A single
`Option` interpolation cannot be right for any value.

**The verbatim compiler diagnostic (the WHY+WHERE for the second layer):**

```text
error[E0308]: arguments to this method are incorrect
    --> src/../../generated/scratch_parser.rs:2086:42
2086 |                       .recover_with_hints(
note: expected `Option<usize>`, found `usize`
2091 |                       4usize,
     = note: expected enum `std::option::Option<usize>`
                found type `usize`
```

⚠️ **Why this survived review:** the unit test
`semantic_usage_codegen_extracts_recovery_hints` asserted `rendered.contains("2usize")`
— it **pinned the buggy emission as correct**. A render-level assertion cannot see a
type error. The test now asserts `Some (2usize)`, and a new test covers the `None`
arm that had no coverage at all.

#### The fix

`optional_usize_expr(Option<usize>) -> TokenStream` spells both arms explicitly
(`Some(#limit)` / `None`). **No signature change was needed** — `recover_with_hints`
already declared `Option<usize>` and already implemented "unbounded" as `None`
(`:7247`/`:7261`/`:7274`), so the leaf's suggested "widen the helper" half was
already done; only the *emission* was wrong.

#### ⭐ Recovery is now measured END-TO-END: it really does RECOVER

The leaf's open question ("does `recover_with_hints` resynchronize, or merely
compile?") is answered **YES**, through the **PARSE-HARNESS scratch slot**
(authoritative by construction — real register→codegen→compile→drive pipeline):

```text
🛟 Recovery for rule 'stmt': moved parser from 0 to 2 using sync token at 1
🛟 Rule 'stmt' recovered from branch failure using sync=[;] panic_until=[]
   budget(rule=unbounded, parse=unbounded, global=unbounded)
```

Differential (same grammar, `@recover` removed = the control): input `X;` →
**rejected** without recovery, **accepted** with it. The `budget(rule=unbounded…)`
line is the previously-unreachable no-budget form working for the first time.
⇒ **row 6 closes as ✅.**

#### ⚠️ A SECOND sharp edge found and DOCUMENTED (not silently absorbed)

`@recover` is emitted only into the **branch-tournament failure path**, which
exists only for a multi-branch rule. On a single-branch rule it is accepted,
lints clean, and emits **nothing**:

```text
stmt := "a" ";"                 branch-count=1   recover CALL-SITES=0
stmt := "a" ";" | "b" ";"       branch-count=2   recover CALL-SITES=1
```

This is the `ANNOTATION-PLACEMENT` family again (*a check that cannot see a defect
class must say so, not return green*). It is **documented in the new book chapter**
as a known limit; making it a diagnostic is left to a follow-up rather than
smuggled into this leaf.

#### The byte-0 diagnostic is fixed too (the leaf's "if cheap" item)

**Root cause of the useless location:** `locate_syn_parse_boundary` bisects **byte
prefixes** and asks whether each still parses as a whole `syn::File`. A prefix cut
at an arbitrary byte almost never does, so `last_good` stayed **0** and every report
pointed at the file's first token. Measured before:

```text
approximate failure byte: 0
context: ...<<HERE@0>>use std :: collections :: HashMap ; ...
```

Replaced by `locate_syn_parse_failure`, which asks a question that *can* be
answered: split the stream into top-level item chunks, `syn`-parse each, and
descend one level into a failing `impl`/`trait`/`mod` to name the **member**.
Because a PGEN parser emits one method per rule, **the answer names the offending
grammar rule**. 5 new unit tests cover it (including the exact `, , ,` shape, the
healthy-sibling non-blame case, and attribute chunking).

#### Bug-class sweep (no silent second instance)

A repo-wide sweep for the same trap — every `Option`-typed binding, **including
tuple destructures**, that is interpolated into a `quote!` — reports **0** across
all of `rust/src`. The sweep was **validated against the pre-fix commit**, where it
flags exactly the 3 known instances:

```text
⚠️  ast_based_generator.rs:4220  #recover_budget : Option<usize>   (from rule_recovery_hints)
⚠️  ast_based_generator.rs:4220  #recover_parse_budget : Option<usize>
⚠️  ast_based_generator.rs:4220  #recover_global_budget : Option<usize>
PRE-FIX Option slots destructured AND interpolated: 3
```

⇒ a sweep that finds nothing proves nothing until it is shown to catch the known
instance. This one is.

#### ACCEPTANCE CHECKLIST

- [x] **REPRODUCE** — `.4`'s 5-cell control matrix re-run on the shipping binary at
  HEAD, all 5 cells matching their declared verdicts; then the *new* layer: the
  all-budgets parser compiled through the scratch slot and rejected by rustc.
- [x] **ROOT CAUSE (WHY + WHERE)** — two layers, both tool-backed. Layer 1 (codegen):
  the token dump shows `recover_with_hints ("stmt" , parse_start , & [] , & [] , , , ,)`
  and `--lint-grammar`-clean grammars still abort. Layer 2 (compile): verbatim
  `error[E0308]` at `generated/scratch_parser.rs:2086:42`, *"expected `Option<usize>`,
  found `usize`"*. WHERE = `ast_based_generator.rs:4249-4251` (emission), NOT the
  helper at `:7237` (already correct).
- [x] **FIX** — `optional_usize_expr` + 3 call sites; `locate_syn_parse_failure`
  wired into the codegen error path; 1 test corrected, 6 added.
- [x] **ADDRESSED (verified, before→after)** — `@recover: true` alone: codegen
  `rc=1` → `rc=0`, emitting `None, None, None`; all-budgets: `4usize` → `Some(4usize)`;
  the generated parser **compiles** (`cargo build --features generated_parsers` green,
  was `error[E0308]`); and recovery **fires at runtime** (`🛟` trace, reject→accept
  differential vs the no-`@recover` control).
- [x] **NO REGRESSION** — all **11** generated parsers regenerated with the fixed
  codegen through the canonical Makefile invocation (cwd `rust/`, `../generated/...`
  paths pinned so the embedded filename is identical) and **BYTE-IDENTICAL**:
  json, regex, vhdl, systemverilog, systemverilog_preprocessor, rtl_const_expr,
  rtl_frontend, return_annotation, semantic_annotation, ebnf. Zero tracked grammars
  use `@recover` (measured: 0 files), so the fix is codegen-inert by construction.
  `cargo test --lib --features generated_parsers`: **984 passed / 0 failed / 21 ignored**.
- [x] **LOCKSTEP** — new book chapter `docs/book/src/error-recovery.md` + `SUMMARY.md`
  entry. This is the FIRST author-facing documentation of recovery in either book
  (the leaf measured 0 prior hits for `@recover`/`@sync`/`@panic_until`).

#### ⛔ Deliberately NOT done in this leaf

- Making `@recover`-on-a-single-branch-rule a **diagnostic** — documented as a known
  limit, routed to a follow-up. Adding a new linter verdict is its own change.
- The `+`-quantifier anomaly surfaced while probing (see the new `QUANT-PLUS-ITER`
  tree) — unrelated to recovery, proven so by the control, and NOT absorbed here.

#### Original charter (retained for provenance)

> ⚠️ **One charter claim was WRONG and is corrected above:** *"The emitted helper's
> signature takes `usize`, so there is no 'unbounded' value to emit."* It does not —
> `recover_with_hints` declares `Option<usize>` and already handles `None` as
> unbounded (`:7243-7245`, `:7247`/`:7261`/`:7274`). That error mattered: taken at
> face value it would have sent the fix into a needless signature migration, and it
> is also what masked the E0308 layer (a charter that believes the parameter is
> `usize` cannot notice that `Some(4)` emitting `4usize` is a type error). Re-measuring
> the cited source before acting is exactly [[feedback_read_prior_art_before_designing]]'s
> RE-MEASURE clause, and it paid here for the second leaf running.

- **ROOT CAUSE, already established (WHY + WHERE) — do not re-diagnose:**
  `ast_based_generator.rs:4240-4252` interpolates `recover_budget`,
  `recover_parse_budget` and `recover_global_budget` — all `Option<usize>` from
  `rule_recovery_hints` (`:9505`) — directly into `quote!`. `ToTokens for Option<T>`
  emits **nothing** for `None`, so the emitted call is
  `recover_with_hints("stmt", parse_start, &[], &[], , , ,)` (verbatim from the token
  dump) and the pipeline aborts with *"expected an expression"* at **byte 0**. The
  emitted helper's signature takes `usize`, so there is no "unbounded" value to emit.
- **Control matrix already measured** (`.4`, 5 cells): `@recover: true` alone → FAIL;
  `+1 of 3` budgets → FAIL; `+ all 3` → OK; `@sync` alone → OK but inert; no annotations
  → OK. Re-runnable via `run_primitive_pricing_probes.sh`.
- **Scope:** give the three budgets a representable "unbounded" form end-to-end — either
  emit `Option<usize>` and widen `recover_with_hints`, or emit an explicit sentinel —
  then make `@recover: true` alone generate and run. ⛔ Do **not** "fix" it by requiring
  all three budgets; that is the bug, not the contract.
- ⚠️ **The failure mode is itself a defect worth fixing:** a codegen `TokenStream` parse
  error reports **byte 0** and names no rule, so a grammar author gets a location that
  points at nothing. Worth a diagnostic improvement in the same leaf if cheap — this is
  the `ANNOTATION-PLACEMENT` family again (*a check that cannot say where must not
  pretend it can*).
- **Then, and only then, measure whether recovery actually RECOVERS.** `.4` deliberately
  claims nothing beyond codegen success. The end-to-end question — does
  `recover_with_hints` resynchronize on `@sync` tokens and produce a usable AST? — is
  this leaf's, and it decides whether row 6 closes as ✅ or reopens as ⚠️.
- **Book obligation:** error recovery is documented **nowhere** in either book (measured:
  0 hits for `@recover`/`@sync`/`@panic_until` across `docs/book/src` and
  `docs/ebnf_parser_book/src`) — a shipped steering surface with no author-facing
  documentation at all. Once it works, it needs a chapter.

### `.9` — Rule-definition uniqueness across a composed grammar (`done`)

- **Status: `done`** (`PGEN-LANG-CAPABILITY-AUDIT-0011`, session #210). **Code change** —
  `rust/src/ebnf_frontend.rs`, plus the two book surfaces.
- ⭐⭐ **DIRECTOR RULING (2026-07-26), verbatim:** *"when loading a given EBNF file, any rule
  definition shall be unique and any rule reference shall have one and only one rule
  definition."* ⇒ the error-vs-warning question `.7` left open is answered: **error**, and
  the *deliberate override* idiom is rejected outright. No override mechanism exists.

#### ⭐⭐ THE MEASUREMENT THAT SHAPED THE FIX — "unique" cannot mean per-clause

The charter was going to enforce uniqueness per rule header. **Measuring first stopped that
from breaking two shipped grammars.** Repeating a rule header is an existing, working PGEN
idiom: the clauses **merge into alternatives of one rule**. Measured on the shipping binary —
`start := "a"` + `start := "b"` produces two raw_ast entries and `--lint-grammar` reports
**1 rule**.

And it is not incidental; two **tracked** grammars are built on it:

| grammar | loads as | uses it for |
|---|---|---|
| `grammars/json.ebnf` | 19 raw_ast entries → **9 rules** | seven `value :=` clauses, each with **its own return annotation** — far more readable than one `\|` chain of seven annotated branches |
| `grammars/rtl_const_expr.ebnf` | 56 entries → **48 rules** | the precedence cascade, one clause per production (`unary_expr` ×5, `primary_expr` ×3, `conditional_expr`/`literal` ×2) |

⇒ **the unit of uniqueness is the FILE, not the clause.** Those clauses are *one definition
written across several lines*, so a reference still resolves to exactly one rule — the
director's invariant holds. Enforcing per-clause uniqueness would have hard-failed the load
of `json` and `rtl_const_expr` on the first run.

⚠️ **The idiom was documented NOWHERE.** The only mention of repeated rule names anywhere in
the grammar-author book was the includes chapter warning that it is a *"well-formedness
concern"* — i.e. the book's sole statement on the subject described a shipped, load-bearing
feature as a problem. Fixed here (see LOCKSTEP).

#### The fix

`register_rule_definitions` records, per **file**, the distinct rule names it defines, and
rejects a name already owned by a *different* file. Threaded through `scan_rules_with_includes`
and `collect_included_rules`, so it covers the whole composed graph at load — every EBNF
consumer inherits it through the same chokepoint `.7` established.

⭐ **Zero risk to single-file grammars by construction:** the check can only fire when two
distinct files are involved, so a grammar with no includes cannot be affected no matter how
many clauses it repeats.

The diagnostic names the rule **and both files**, because the provenance is the entire value
of the check:

```
duplicate rule definition: 'value' is defined in BOTH 'main.ebnf' and 'other.ebnf'.
A rule reference must resolve to exactly one definition, so the same rule name may not be
defined by two different files. Rename one, or remove the duplicate include.
(Repeating a header WITHIN one file is fine — those clauses merge into alternatives of a
single rule.)
```

- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — `.7` made composition real, which made cross-file collisions
    *reachable*: two files defining one name merged silently, one file adding alternatives to
    another file's rule with no diagnostic. New exposure, not pre-existing debt.
  - [x] **ROOT CAUSE (WHY + WHERE)** — composition in `scan_rules_with_includes` concatenated
    rule lists with no name bookkeeping, and the linter has no duplicate-definition check, so
    the merge was indistinguishable from the intentional within-file idiom.
  - [x] **FIX** — `register_rule_definitions` (`ebnf_frontend.rs`), file-scoped ownership,
    hard error naming both files.
  - [x] **ADDRESSED (verified)** — driver extended to **22 declared-verdict cases, exit 0, 0
    divergences**: cross-file collision **exit 1** with both files named; within-file
    multi-clause still merges to 1 rule; `json.ebnf` **9** and `rtl_const_expr.ebnf` **48**
    unchanged; ⭐ a **diamond does not false-positive** (the shared file is visited once, so
    its names are registered once).
  - [x] **NO REGRESSION** — every tracked grammar re-measured through the real frontend for
    duplicate definitions before the change, so the blast radius was known, not assumed: only
    `json`, `rtl_const_expr` and the throwaway `scratch` slot carry repeats, **all
    within-file**, hence all unaffected. `.7`'s full probe bank re-run green (SV wrapper 1403,
    two-file compose 2, five no-include grammars pinned). clippy strict source **0 errors, 0
    hits on the changed file**; `ebnf_frontend_dual_run_gate` GREEN; `ebnf_parser_book_gate`
    GREEN; all 9 doctrines PASS. No release, schema, ledger or contract movement.
  - [x] **LOCKSTEP** — `includes.md`'s vague *"well-formedness concern"* line replaced with
    the actual hard-error contract, the quoted diagnostic, and an explicit *"there is
    deliberately no override mechanism"*; and `rules-and-expressions.md` gains a new
    **Multi-clause definition** section documenting the previously-undocumented idiom, with
    the same-file constraint stated and cross-linked both ways.

#### ✅ THE ONE OPEN READING — CLOSED BY THE DIRECTOR, SAME SESSION

The leaf shipped with one question deliberately routed rather than absorbed: under a *literal*
reading of the ruling ("any rule definition shall be unique"), the within-file multi-clause
idiom would also be forbidden and two shipped grammars would need rewriting into `|` chains.
This leaf took the reading under which the ruling's stated *purpose* holds — a reference
resolves to exactly one definition — because the clauses **are** one definition.

⭐ **The director confirmed that reading immediately, verbatim (2026-07-26, session #210):**
*"You are right, I forgot this `RuleA := Branch_A | Branch_B | Branch_C` is the same as 3 rules
productions like `RuleA := Branch_A` then `RuleA := Branch_B` and `RuleA := Branch_C`"*

⇒ **no follow-up leaf is needed and no shipped grammar is rewritten.** The equivalence is
confirmed at the source: the two spellings are the same rule, so file-scoped uniqueness is the
correct and complete enforcement of the ruling. ⚠️ Worth noting *why* the question arose at all
— the idiom was **documented nowhere**, so even its author had to be reminded it existed. That
is the strongest possible argument for the book section this leaf adds, and one more instance of
the tree's recurring finding: **an undocumented capability is one nobody can defend, including
the person who specified it.**

### `.3c` — Keep widening the matrix with further notoriously-hard languages (`todo`)

- **Status: `todo`**, opened by the `.4` steer above. `.3`'s 16 rows came from a fixed
  candidate list (JS, Python, Ruby, Raku, VHDL, SV, C/C++, HTML, …). The directive says the
  audit does not close there.
- Method is `.3b`'s, unchanged: a capability is a ROW, languages are EVIDENCE, and no row
  earns a verdict without a probe. Candidate columns not yet examined — each chosen because
  it stresses an axis the current 16 rows may not cover: **Forth / PostScript**
  (parse-time-definable words — check against the row-16 hard bound), **TeX** (catcode
  reassignment = lexical classes mutable mid-document), **COBOL** (column-sensitive fixed
  format + `COPY … REPLACING`), **Fortran** (fixed-form + significant-whitespace-free
  tokenization, `DO 10 I=1,10` vs `DO10I=1.10`), **APL/J** (glyph tokenization, no reserved
  words), **Makefile** (tab-significant, recursive macro expansion), **Nix/Dhall** (string
  interpolation nesting), **Prolog** (user-declared operators via `op/3` — likely another
  row-16 relative), **Wolfram/M4** (macro-expansion-time syntax).
- ⚠️ Expect some to land on `.3`'s row 16 **HARD BOUND** (parse-time-mutable grammar, out of
  scope by design). That is a legitimate outcome and must be recorded as such — the bound is
  what keeps "any language" honest. The value is in the rows that do NOT.

### `.10` — re-open `.7`: the `include()` was a TYPO, and the linter was silenced to hide it (`active` — SPLIT into `.10.1` ✅ / `.10.2` ✅ / `.10.3` / `.10.4` ✅ / `.10.5` ✅ / `.10.6` ◐ / `.10.7`)

> **Container as of session #228.** Opened in a fresh session per the director's ruling
> below. `.10.1` (done) settled the target, audited the whole allowlist, and corrected
> `.7`'s record; it also **overturned this charter's recommended target** and found a
> second defect class the charter did not know about. `.10.4` (done) then removed that
> second defect — the `true`/`false` zero-width builtins — with all 11 generated parsers
> byte-identical. `.10.5` (done) removed a **third**: the meta-grammar's
> `single_quoted_string` never closed its quote, so the self-hosting parser swallowed
> arbitrary text and its published **12/12** was a false green (now an honest **11/12**).
> ✅ **`.10.2` LANDED AS OPTION B** (session #228), after the director challenged the
> chartered premise before any code was written — *"why even wanting to reinstate it"* —
> and measurement upheld both challenges: the include had **never once worked** in the
> repo's history, nothing consumes the meta-parser's annotation AST, and the charter's cost
> warning was **inverted** (it budgeted against `ebnf`, which has no contract; the
> contract-bearing parser is `semantic_annotation`). The director then chose B. Result:
> `ebnf.ebnf` defines `semantic_annotation` itself as an opaque-payload delimiter,
> **self-hosting is 12/12** and `ebnf_frontend_dual_run_gate` is **GREEN**. **`.10.3` is
> now unblocked** — the references resolve, so the allowlist entry can be retired.
> ◐ `.10.6` part 1 (the de-Perl) is DONE; its part 2 — the frontend↔meta-parser envelope
> differential — remains `todo`.
> Read `.10.1` before acting on anything in this charter — two of its statements below
> are annotated as superseded.

> **Director, 2026-07-26 session #212**, on being shown `.7`'s account: *"The reference
> in ebnf.ebnf had a typo."* → *"But we have grammars/semantic_annotation.ebnf"* →
> *"The builtin semantic annotation EBNF is grammars/builtin_semantic_annotation.ebnf"*.
> Asked whether to fold the repair into session #212 alongside `@entry`, the director
> ruled: **"In a new fresh session."** ⇒ **DO NOT START THIS IN A SESSION THAT IS
> ALREADY CARRYING OTHER WORK.**

⛔ **`.7` GOT THIS ONE WRONG, and the record must say so.** `.7` deleted
`include(semantic_annotations)` from `grammars/ebnf.ebnf:18` as *"a stale include
naming a file that has never existed"*. That is true **of the plural spelling only** —
and the singular file is right there:

| fact | measured |
|---|---|
| the deleted line (git `754d1a5e`) | `-include(semantic_annotations)` — **plural** |
| `grammars/semantic_annotation.ebnf` | **exists**, 20,206 B, 112 rules — the normative annotation-language spec named by `README.md` |
| `grammars/builtin_semantic_annotation.ebnf` | **exists**, 4,580 B, 23 rules — the bootstrap-safe twin |
| `ebnf.ebnf` references to `semantic_annotation` | **3 live, non-comment** (`:30`, `:79`, `:134`) — none defined |

⇒ it was a **one-character typo** (plural/singular) for a file that exists, not a
reference to nothing. `.7` removed the symptom and certified the grammar
*"self-contained"*.

⭐⭐⭐ **AND THE REASON NOTHING CAUGHT IT IS THE REAL DEFECT — a FOURTH masking layer
on top of the three `.7` itself found.** `--lint-grammar` reports
`undefined_references=0` on a grammar with 3 undefined references, because the name is
**hard-coded into a rule-NAME allowlist** (`ast_based_generator.rs`,
`NATIVE_UNRESOLVED_REFERENCE_BUILTINS`):

```rust
pub const NATIVE_UNRESOLVED_REFERENCE_BUILTINS: &'static [&'static str] = &[
    "builtin_any_char", "builtin_ascii_char", "false",
    "semantic_annotation",     // <- NOT a builtin: a real rule in another grammar file
    "true",
];
```

`builtin_any_char`/`builtin_ascii_char` are genuine codegen builtins and `true`/`false`
are literals; **`semantic_annotation` is neither.** Its presence there is a workaround
for the broken include that silenced the one instrument that would have named it. ⇒
this is an **`EBNF-SOURCE-OF-TRUTH` breach of the exact class director #208 named**
(*"no hard-coded rule-NAME `matches!` arm — that last disguise is exactly what hid the
no-layout capability for a whole session"*), and `.7`'s `undefined_references=0`
evidence rested on it.

#### ⚠️ Fixing the typo is NECESSARY BUT NOT SUFFICIENT — measured, so the next session does not start optimistic

Neither annotation grammar is a drop-in replacement:

| candidate target | rules | collisions with `ebnf.ebnf`'s 131 | defines `semantic_annotation`? |
|---|---|---|---|
| `semantic_annotation.ebnf` (full) | 112 | **15** ← the number `.7` cited to justify deletion | ✅ yes (`:16`) |
| `builtin_semantic_annotation.ebnf` | 23 | **3** — `boolean_literal`, `identifier_literal`, `null_literal` | ⛔ no — its entry rule is `builtin_semantic_annotation` |

⇒ the builtin is far the better composition target (**3 collisions, not 15**) but does
not supply the referenced NAME; the full one supplies the name but collides 5× worse.
⭐ And since `.9`, a cross-file name collision is a **hard error**, so the 3 must be
resolved, not tolerated.

#### Deliverables

1. Decide the target (**recommend `builtin_semantic_annotation`**, the bootstrap-safe
   twin — the meta-grammar has the same chicken-and-egg property the builtin grammars
   exist to break) and align the referenced rule name.
   — ⛔ **RECOMMENDATION OVERTURNED BY `.10.1` ON SHAPE. See `.10.1`.**
2. Resolve the 3 name collisions.
   — re-scoped by `.10.1`: the target is the full grammar, so it is **11 conflicts of 15
   collisions**, not 3.
3. ⭐ **Retire `"semantic_annotation"` from `NATIVE_UNRESOLVED_REFERENCE_BUILTINS`** so
   the undefined-reference check can see that class again — and re-run it to confirm it
   now reports 0 *because the grammar is sound*, not because the name is allowlisted.
   — `.10.1` **sequences** this: it MUST land after the grammar is sound (`.10.3`).
4. ✅ **DONE by `.10.1`** — correct `.7`'s record in place: its deletion and its
   *"self-contained"* conclusion were both wrong, and the reasons are above.
5. Byte-identity proof with input AND output paths pinned (`.5` / `BIN-BUILD-INTEGRITY.3`).
   — ✅ done for `.10.1`'s own edit; still required for `.10.2`/`.10.3`.

⚠️ **Audit the other allowlist members while there** — the same question ("is this
actually a codegen builtin, or a silenced defect?") has not been asked of them.
— ✅ **DONE by `.10.1`, and it found a second defect class.**

---

### `.10.1` — adjudicate the target, audit the whole allowlist, correct `.7`'s record (`done`)

- **Status: `done`** (`PGEN-LANG-CAPABILITY-AUDIT-0013`, session #213, 2026-07-26).
  **Grammar comment block + docs + driver only** — the sole non-doc edit is a comment
  block in `grammars/ebnf.ebnf`, proven codegen-INERT below. No Rust source, no
  generated artifact, no contract, no release/schema/ledger movement.

`.10` as chartered bundled an unresolved policy choice (which file to compose) with a
multi-grammar merge, an allowlist retirement, and a record correction. Measurement
overturned the choice, so per the splitting rules `.10` is now a container and this leaf
banks what measurement settled.

#### ⭐⭐⭐ Finding 1 — `.10`'s recommended target is the WRONG SHAPE, and the collision count was the wrong criterion

`.10` recommended `builtin_semantic_annotation.ebnf` because it collides 3× instead of
15×. **Collision count ranks the candidates; it does not tell you whether either one
means the right thing.** Measured entry-rule shapes:

| candidate | entry rule | matches the `@`? | verdict |
|---|---|---|---|
| `semantic_annotation.ebnf` | `semantic_annotation := "@" /\s*/ annotation_name /\s*/ ":" /\s*/ annotation_value` | ✅ **yes** | the `@name: value` **line** — what `ebnf.ebnf` needs |
| `builtin_semantic_annotation.ebnf` | `builtin_semantic_annotation := ws* semantic_payload ws*` | ⛔ **no** | the **payload** only — the text *after* the `:` |

⇒ the builtin grammar is not a smaller version of the same object; it is a **different
object**. It is the inferred behavioural spec of the hand-written bootstrap parser
(its own header, verbatim: *"intentionally permissive"*, *"never hard-fails on syntax"*),
and its fallback is `raw_payload := any_text` where `any_text := /(.|\n)*/` — an
**always-succeeding, everything-swallowing** rule. Splicing that into `ebnf.ebnf`'s
`grammar_file := (include_directive | semantic_annotation | grammar_rule | comment |
whitespace)*` alternation would put an always-succeeding alternative inside the
meta-grammar's top-level loop. **Cheaper by collision count, catastrophic by shape.**

⇒ **DECIDED: the target is the full `grammars/semantic_annotation.ebnf`.** This also
matches the director's own steer (*"But we have grammars/semantic_annotation.ebnf"*);
`.10` read the third director statement (*"The builtin semantic annotation EBNF is
grammars/builtin_semantic_annotation.ebnf"*) as target selection when it was
disambiguation of which file is which.

#### ⭐⭐ Finding 2 — the 15 collisions are 11 genuine CONFLICTS, so the composition is a real design problem

`.10` (and `.7` before it) treated "15 collisions" as one number. Measured per rule
(definition text compared with comments and whitespace normalized out):

- **4 duplicates** (identical definitions): `decimal_literal`, `identifier_literal`,
  `numeric_literal`, `scientific_literal`.
- **11 genuine conflicts** (same name, different language): `binary_literal`,
  `block_comment`, `boolean_literal`, `double_quoted_string`, `hexadecimal_literal`,
  `integer_literal`, `line_comment`, `null_literal`, `octal_literal`,
  `single_quoted_string`, `whitespace`.

Two that show why this cannot be resolved by renaming alone:

| rule | `ebnf.ebnf` | `semantic_annotation.ebnf` |
|---|---|---|
| `boolean_literal` | `("true" \| "false")` | adds `yes`/`no`/`on`/`off`/`enabled`/`disabled`/`active`/`inactive` |
| `line_comment` | `("#" \| "//") comment_content` | `"//" /[^\r\n]*/` — **no `#`** |

⇒ `semantic_annotation.ebnf` is **not a composable fragment**: it is a standalone
grammar carrying its own private lexical layer that disagrees with the meta-grammar's.
Since `.9` a cross-file collision is a hard error, so all 15 must be resolved. That is
`.10.2`'s job and it is a design leaf, not a one-line repair.

#### ⭐⭐⭐ Finding 3 — the allowlist audit found a SECOND defect class: `true` and `false` synthesize ALWAYS-SUCCEEDING ZERO-WIDTH matchers

The `.10` charter asked whether the other allowlist members are genuine builtins. They
are not one class — they are three, and one of them is a trap:

| member | emitted matcher | consumes? | referenced by any tracked grammar? | verdict |
|---|---|---|---|---|
| `builtin_any_char` | one Unicode scalar, `self.position = end_pos` | ✅ yes | `regex.ebnf` | ✅ **genuine builtin** |
| `builtin_ascii_char` | one ASCII byte | ✅ yes | `regex.ebnf` | ✅ **genuine builtin** |
| `semantic_annotation` | `@` … to end of line | ✅ yes | `ebnf.ebnf` (dangling) | ⛔ **masked defect** — Finding 4 |
| `true` | unconditional `Ok`, `Span::new(start_pos, start_pos)` | ⛔ **NO** | **none** | ⛔⛔ **dead surface AND a trap** |
| `false` | unconditional `Ok`, `Span::new(start_pos, start_pos)` | ⛔ **NO** | **none** | ⛔⛔ **dead surface AND a trap** |

Emitted verbatim (capture §A5):

```rust
pub fn parse_true(&mut self) -> ParseResult<ParseNode<'input>> {
    let start_pos = self.position;
    Ok(ParseNode {
        rule_name: &"true",
        content: ParseContent::Terminal("true"),
        span: Span::new(start_pos, start_pos),   // <- zero width. Nothing consumed. Never fails.
    })
}
```

⭐ **The decisive contrast, measured behaviourally through the scratch slot** (one
grammar, one build, 7 declared verdicts, 0 divergences) — same allowlist, opposite
behaviour:

| input | grammar fragment | verdict | what it proves |
|---|---|---|---|
| `TT` | `probe_true := "T" true "T"` | **ACCEPT** | ⛔ `true` matched **EMPTY** |
| `TtrueT` | same | REJECT | `true` is zero-width, **not** a literal matcher — the `Terminal("true")` payload is a lie about the input |
| `FF` | `probe_false := "F" false "F"` | **ACCEPT** | ⛔ `false` matched **EMPTY** |
| `CzC` | `probe_any := "C" builtin_any_char "C"` | ACCEPT | ✅ a genuine builtin **consumes** |
| `CC` | same | REJECT | ✅ …and **refuses to match empty** — the contrast |

⇒ this is the **`.5` defect class exactly** (`digit := [0-9]` lints clean while
compiling to an always-succeeding empty optional). A grammar author who writes a rule
named `true` — an entirely natural name for a boolean-literal rule — gets a silently
always-succeeding empty match **and** `undefined_references=0`. Nothing in the pipeline
says a word. Neither name is referenced by any tracked grammar, so this is dead surface
that exists only to be stepped on. Routed to `.10.4`.

#### ⭐⭐ Finding 4 — the masking is structural, and the const's own rationale is false

The `.10` charter said `undefined_references=0` is produced by the allowlist. Measured
both directions (capture §A4), on a probe grammar with four dangling references:

- probe naming `true`, `false`, `semantic_annotation`, `builtin_any_char` →
  `undefined_references=0`;
- probe naming one **non-member** → `undefined_references=1`.

⇒ the check works; it is the allowlist that makes the four invisible. And the const's
doc comment claims `semantic_annotation` is *"the native `@…`-line matcher used by the
annotation grammars"* — **measured false**: `generated/ebnf.rs` is the ONLY generated
parser carrying the native form. `semantic_annotation.ebnf` carries `parse_semantic_annotation`
from its **own definition**, not from the fallback, and `builtin_semantic_annotation.ebnf`
mentions the name only in a comment. The meta-grammar is the sole consumer, **and it
consumes it because its include is broken** — precisely the workaround `.10` alleged,
now measured rather than asserted.

Two independent instruments were made to agree explicitly (capture §A3b): a text screen
over the tracked grammars and the generated-parser census. The screen only became
truthful after stripping three things that spell these names without referencing a
rule — annotation lines (`@entry: true`), quoted/regex terminals (`("true" | "false")`),
and return-annotation literals (`-> {negated: false}`). Before that it reported `true`
as referenced by **16** grammars. **That screen is in the driver with the reason
written next to it**, so the next reader does not re-derive the false number.

#### ⭐⭐ Finding 5 — the grammar-author book documented a built-in that does not exist, in 5 chapters

Asking "what are the real members of the const?" immediately convicted the book PGEN
ships for grammar authors. `docs/ebnf_parser_book/` stated in **five** chapters
(`terminals.md`, `glossary.md`, `codegen-model.md`, `lookaheads.md`, `quantifiers.md`)
that `any_char` / `ascii_char` are native built-ins, equivalent to the `builtin_`-prefixed
forms — `terminals.md` billing itself as *"the authoritative list of what the codegen
actually matches"*. Measured, they are not on the const, and:

```
$ ast_pipeline --lint-grammar <(printf '@entry: true\nstart := "C" any_char "C"\n')
undefined_references=1
[error] … rule 'start' references UNDEFINED rule 'any_char' — codegen emits a
never-matching stub for it, so every path through the reference ALWAYS fails
```

`any_char` is an **ordinary rule defined in exactly one grammar** — `grammars/regex.ebnf:2239`,
`any_char = letter | digit | whitespace | special_char | unicode_char` — which is why the
un-prefixed spelling reads as idiomatic throughout that file. The book had generalized
one grammar's private rule into a platform primitive, and **four of its example snippets
were uncopyable**: paste them anywhere else and you get a hard error (or, in a grammar
that happens to define its own `any_char`, a silently different matcher).

⭐ **No code defect here — the un-prefixed names SHOULD NOT exist.** The `builtin_`
prefix is the director's 2026-06-07 namespacing rule precisely so a primitive can never
be shadowed by a same-named grammar rule, and `regex.ebnf`'s `any_char` is exactly the
rule it protects against. Withholding the alias is the correct design; only the
documentation was wrong. All five chapters corrected, examples included, plus a residual
sweep (the one surviving un-prefixed mention is `grammar-file-structure.md`, where the
line is explicitly quoting `regex.ebnf`'s own idiom to illustrate the `=` operator).

⇒ this is the same shape as `.5` (*"the meta-grammar's own comments were contradicting
the book PGEN ships for authors"*) and `.9` (*"an undocumented capability is one nobody
can defend"*) — **a mis-documented capability is worse than an undocumented one**,
because a reader has no reason to doubt it.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `.7` certified `ebnf.ebnf` *"self-contained"* on
  `undefined_references=0`. Reproduced: the grammar has 3 live references to an
  undefined `semantic_annotation` and still lints 0.
- [x] **ROOT CAUSE (WHY + WHERE)** — `ast_based_generator.rs:1198-1204`,
  `NATIVE_UNRESOLVED_REFERENCE_BUILTINS`. `grammar_wellformedness.rs:333` consumes that
  same const as the linter's allowlist (deliberately, so linter and codegen cannot
  drift), so allowlisting a name for codegen **necessarily** blinds the linter to it.
  The dispatch arm is `ast_based_generator.rs:1315-1343`. Tool-backed by the two-arm
  lint differential (§A4) and the emitted-source extraction (§A5).
- [x] **ADDRESSED** — for this leaf's own scope: the target is decided on shape
  evidence, the allowlist is audited member-by-member, the false record is corrected in
  both places it was written (`.7`'s leaf above and the comment block in
  `grammars/ebnf.ebnf`), and the grammar-author book is corrected in all five chapters
  that documented a built-in which does not exist (Finding 5 — that one is fully closed
  here, no follow-up leaf, because the correct design is to withhold the alias). The
  remaining repairs are `.10.2`/`.10.3`/`.10.4`.
- [x] **NO REGRESSION** — the `grammars/ebnf.ebnf` edit is a comment block and is proven
  **codegen-INERT**: generated twice **to the same pinned output path** from the same
  pinned input path, HEAD vs edited, sha256
  `8b91ef9097f5dc3143b3ab3d7ca85a61c01d89db6c0e98a0767051d3a880fbed` **both arms**,
  `cmp` clean; the 229-entry return-annotation inventory is byte-identical too. Lint is
  unchanged (131 rules, all counters 0). The scratch slot was restored to its committed
  fixture **and** `parseability_probe` rebuilt against it (verified behaviourally:
  `hello, pgen!` ACCEPT, `hello, mars!` REJECT at position 7).

⚠️ **Method note banked for the next leaf.** The first byte-identity attempt reported a
3,056-hunk diff for a comment-only edit. Root cause: the generated parser **embeds its
own output path** as a `filename_str` logging literal, so the two arms differed only by
`ebnf_before/` vs `ebnf_after/`. This is what *"input AND output paths pinned"* in `.5`
and `BIN-BUILD-INTEGRITY.3` actually protects against — **generating to two different
paths makes byte-identity structurally unachievable and the diff looks alarming.**
Recorded because the failure mode reads as a codegen defect and is not one.

---

### `.10.2` — define `semantic_annotation` LOCALLY as an opaque-payload delimiter (OPTION B) (`done`)

- **Status: `done`** (`PGEN-LANG-CAPABILITY-AUDIT-0021`, session #228). ⛔ **The chartered
  approach (option A, "restore the delegation") was NOT implemented — its premise did not
  survive measurement.** The leaf was first put `on hold` by DIRECT DIRECTOR ORDER
  (2026-07-30): *"postpone `.10.2` for later, because I do not understand why implementing
  `.10.2` would have so much effect of breaking things … apparently
  `include(semantic_annotation)` is not actually needed, because so far, it was discarded
  all along, so why even wanting to reinstate it."* **Both challenges were correct**
  (measured below). The director then chose **option B**: *"then do (2) .10.2 option B
  (your recommendation)"*.
- ⇒ the original charter — *"restore the delegation"* — was a **design proposal wearing the
  costume of a repair**, and the more expensive of the two ways to meet the requirement.
  Everything below the options table is retained as the reasoning that settled it.

#### ⭐⭐⭐ ANSWER 1 — "why would this break things?" The composition mechanics

An `include(X)` **splices X's rules into the grammar** (`ebnf_frontend.rs`
`scan_rules_with_includes`). `semantic_annotation.ebnf` is not a fragment: it is a
standalone 112-rule grammar carrying its **own complete lexical layer** — `whitespace`,
`line_comment`, `block_comment`, `boolean_literal`, `integer_literal`,
`double_quoted_string`, … — and `ebnf.ebnf` defines its own rules with **15 of the same
names**. Since `.9`, a cross-file duplicate rule name is a **hard error**, so the splice
cannot simply happen. Worse, 11 of the 15 are not duplicates but genuine **conflicts** —
the two files *disagree about what the token means*:

| rule | `ebnf.ebnf` says | `semantic_annotation.ebnf` says |
|---|---|---|
| `line_comment` | `("#" \| "//") …` | `"//" …` — **no `#`** |
| `boolean_literal` | `("true" \| "false")` | adds `yes`/`no`/`on`/`off`/`enabled`/`disabled`/`active`/`inactive` |

So the leaf must first **rename on one side or extract a shared fragment** (surgery on
both files) — and only then discovers that the payload language it imported **still does
not parse what PGEN actually writes**: `.10.5` measured all 148 distinct annotation lines
in every tracked grammar against that grammar and **9 are rejected**, including the entire
`@transform: str::parse::<usize>().unwrap_or(0)` family that ships in `return_annotation.ebnf`
and `regex.ebnf`. ⇒ *compose two grammars that disagree about their lexical layer, in order
to import a payload spec that is measurably wrong, and then extend it anyway.* **That is
why it breaks things.**

#### ⭐⭐⭐ ANSWER 2 — "why reinstate it at all?" You should not. Three measured facts

**Fact 1 — the include has NEVER once worked, in the entire history of the repo.** The
deleted line was `include(semantic_annotations)`, **plural**, naming a file that has never
existed. Before `.7` the frontend **discarded include directives entirely** (`.4`
FINDING 1), so it was inert; after `.7` made includes real, an unresolvable include is a
**hard error**, so the same line could only have failed. It has never contributed a single
rule. ⇒ *"restore the delegation"* is not restoration — **there is nothing to restore.**

**Fact 2 — nothing consumes the meta-parser's annotation AST.** Every consumer of the
generated `EbnfParser` uses `parse_full_grammar_file()` for a **verdict**, never for a
payload:

| consumer | what it takes |
|---|---|
| `parser_registry.rs:627` `parse_with_ebnf` | `.is_ok()` — a bool |
| `parser_registry.rs:633` `parse_with_ebnf_detail` | `Result<(), String>` — a bool + message |
| `ebnf_frontend.rs:70` | `if let Err(..)` — a **soft** cross-check, warn-only unless `PGEN_EBNF_FRONTEND_REQUIRE_GENERATED_VERIFY=1` |
| `ebnf_dual_run_diff.rs:157` | the diagnostic report |
| `parser_registry.rs:648` `parse_with_ebnf_ast_json` | the ONLY AST consumer — and its only caller is `parse_harness_equivalence.rs`, the interpreter-vs-generated **self-consistency oracle** |

The **real** EBNF frontend is the hand-written `scan_top_level_rules`; the generated
meta-parser is a *proof surface*, not a production path. ⇒ **structuring the annotation
payload inside `ebnf.ebnf` buys no consumer anything.** The payload is already parsed —
properly, by the annotation backend (`generated/semantic_annotation_parser.rs`) — and that
path is untouched by any of this.

**Fact 3 — ⛔ the charter's cost warning is INVERTED.** Deliverable 4 below warns that the
work *"changes a shipped parser's behaviour … budget for release/schema/ledger + book
lockstep"* and points at the **`ebnf`** parser. Measured:

| parser | integration contract | `ast_shape_contract` manifest |
|---|---|---|
| `ebnf` | **none** | **none** |
| `semantic_annotation` | `PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md` | `semantic_annotation_v1.json` |

`ebnf` is **contract-free** — there is no published surface to move. The
**contract-bearing** parser is `semantic_annotation`, the one the charter treated as
merely *"cheaper to write"* — and its manifest pins the very names in the collision set
(`boolean_literal` ×10, `null_literal` ×5, `integer_literal`, `single_quoted_string`). ⇒
**the charter budgeted the cost against the wrong parser, and the option it called cheap
is the expensive one.**

#### The requirement, restated with the include removed

Strip the mechanism away and only two things are actually required:

- **R1 — `ebnf.ebnf` must be a sound grammar** (every reference resolves), so
  `"semantic_annotation"` can be dropped from `NATIVE_UNRESOLVED_REFERENCE_BUILTINS`
  (`.10.3`) and the linter stops being structurally blind to that name for **every**
  grammar. This is the `EBNF-SOURCE-OF-TRUTH` breach the director named in #208 —
  *"no hard-coded rule-NAME `matches!` arm"*.
- **R2 — the meta-grammar must recognize every annotation form tracked grammars use**,
  including the multi-line brace payload, so self-hosting is 12/12 for the right reason
  and `ebnf_frontend_dual_run_gate` is green because the parser understands the bytes.

Neither requires the include. Neither requires the payload to be **structured** — only
**delimited**.

#### The three options, priced on the measurements above

| | option | what it does | cost | meets R1 | meets R2 |
|---|---|---|---|---|---|
| **A** | restore the include *(the charter as written)* | splice `semantic_annotation.ebnf` into `ebnf.ebnf` | resolve 11 lexical conflicts by renaming a **contract-bearing** parser or extracting a shared fragment; **then** extend `annotation_value` to accept the 9 rejected payload forms; two books + a contract + a manifest in lockstep | ✅ | only after the extension |
| **B** ⭐ | **define `semantic_annotation` locally in `ebnf.ebnf`** as a *delimiter* rule — `@` + name + `:` + an **opaque** payload spanning balanced braces/brackets or to end of line | one self-contained rule in a **contract-free** grammar | no cross-file composition, no collisions, no contract/manifest movement, no duplicated payload spec | ✅ | ✅ |
| **C** | do nothing | keep the native `@`-to-EOL slurp | zero work, but: the allowlist entry stays ⇒ the linter stays blind ⇒ `.10.3` can never land; self-hosting stays 11/12; `ebnf_frontend_dual_run_gate` stays RED | ⛔ | ⛔ |

**Recommendation: B.** It is strictly smaller than what is chartered, touches only the
parser with no published contract, keeps payload validation in the one place that already
does it correctly, and is the only option that meets both requirements without importing a
spec that is already known to be wrong. ⭐ It also matches what the meta-grammar's job
actually is: *parse grammar FILES*, not validate annotation payloads.

#### ✅ OPTION B IS MEASURED, NOT PROPOSED — `run_option_b_probe.sh`, exit 0, 0 divergences

⛔ **An earlier revision of this leaf left "can PGEN's EBNF express an opaque balanced-brace
payload?" as an open question for the director.** That was wrong twice over: the answer was
already in the repository, and it is the engineer's to obtain. Director, session #228:
*"you are the elite coder of PGEN … you should be able to respond to your own question.
Your question sounds as if you are a stranger to PGEN Rust code."* Correct. It is answered
below, and the method is now a Knowledge-Map card so the next reader does not re-ask it.

**Both constructs already ship — in the two files this rule sits between:**

| construct | already in `ebnf.ebnf` | already in `builtin_semantic_annotation.ebnf` |
|---|---|---|
| a regex terminal spanning **newlines** | `block_comment_content := /((?:[^*]\|\*(?!\/))*)/` | `any_text := /(.\|\n)*/` |
| **recursive balanced-bracket** nesting | `grouped_expression`, `object_return` | `structured_object := "{" … "}"` via `structured_payload` |

⭐ **And the builtin grammar settles the DESIGN, not just the feasibility** (director's
pointer, session #228 — *"the builtin parsers are to avoid the chicken and egg dilemma"*).
`rust/src/ast_pipeline/unified_semantic_ast.rs:286` `parse_bootstrap` is **TOTAL**:
`TransformExpr` if the text carries both `::parse::<` and `>().unwrap_or(`, else
`Structured` if the structured parser succeeds, else `Raw { content }` — **no error path**.
PGEN's own annotation contract is therefore *"structured if possible, opaque otherwise,
never reject"*, and `builtin_semantic_annotation.ebnf` is its written spec. Option B gives
the meta-grammar the **same** bootstrap-safe shape, sized to what a meta-grammar needs
(delimit) rather than what a validator needs (structure). ⇒ `.10`'s original instinct —
*"the meta-grammar has the same chicken-and-egg property the builtin grammars exist to
break"* — was **right**, even though `.10.1` correctly rejected including that file (its
entry rule is the payload, and its `raw_payload := any_text` would put an
always-succeeding alternative in `grammar_file`'s top-level loop).

**Measured end-to-end** — `docs/tasks/artifacts/lang_capability_audit/run_option_b_probe.sh`,
zero repo edits (the probe is built in an untracked `rust/target/` scratch dir):

| | shipped meta-grammar | option-B probe |
|---|---|---|
| rules / lint | 131, all counters 0 | **138, all counters 0** (no shadowing, no always-succeeds, no unreachable, no nullable-repetition) |
| `parse_semantic_annotation` | **25** lines — the synthesized native `@`-to-EOL fallback | **939** lines — generated from the rule; all 6 helpers emitted |
| `regex.ebnf` | **REJECT @ byte 55,925** | **ACCEPT** |
| self-hosting | 11/12 tracked *(14/15 by the driver's wider set)* | **12/12** *(driver: **15/15**)* |

*(The book's denominator is 12 — the tracked grammars, excluding `scratch.ebnf` and the two
`systemverilog_lrm_profiled_*` intermediates the driver also sweeps. Both numbers describe
the same result: `regex.ebnf` is the one that flips, and nothing else moves.)*

⇒ **R1 satisfied** — `semantic_annotation` resolves to a real rule, so
`"semantic_annotation"` can leave `NATIVE_UNRESOLVED_REFERENCE_BUILTINS` and `.10.3` is
unblocked. **R2 satisfied** — the multi-line payload parses, self-hosting is 12/12 for the
right reason, and `ebnf_frontend_dual_run_gate` re-greens.

⭐⭐⭐ **AND IT IS FORWARD-COMPATIBLE WITH THE SELF-HOSTING ENDGAME — measured, after the
director asked whether `generated/ebnf.rs` is meant to REPLACE the hand-written frontend
(session #228).** It is: `README.md:28` — *"Handwritten parsers exist only as bootstrap
scaffolding and never count as closure"*. So the question that matters for this leaf is
whether an opaque payload would have to be undone when that replacement happens. **It
would not**, because the code being replaced already does exactly this:

```rust
// rust/src/ebnf_frontend.rs:1410 — the AUTHORITATIVE frontend
fn parse_semantic_annotation_text(annotation: &str) -> Option<(String, String)> {
    let content   = trimmed.strip_prefix('@')?.trim();
    let separator_idx = find_top_level_colon(content)?;   // brace/bracket/paren-depth AND quote aware
    let name    = content.get(..separator_idx)?.trim();
    let payload = content.get(separator_idx + 1..)?.trim();   // <- an OPAQUE String
    Some((name.to_string(), payload.to_string()))
}
// …emitted as: ["semantic_annotation", [name, payload]]
```

⇒ the authoritative frontend **delimits the annotation and passes the payload through
untouched**, handing it to the annotation backend. Option B models that; option A would
model something the real frontend does not do. `find_top_level_colon` also confirms the
right delimiter semantics — depth-aware over `()`/`[]`/`{}` and quote-aware — which is the
specification the option-B rule should ultimately match if it is ever tightened.

⚠️ **The honest trade, stated rather than sold.** With an opaque payload, `ebnf.ebnf` does
**not** specify annotation-payload syntax. That is the correct division of labour — the
payload spec belongs to the annotation parsers, and it is *already* opaque today, just
hidden inside a codegen allowlist instead of written in the grammar where a reader can see
it. But it means the self-hosting number stays a **recognition** claim, which is all it has
ever been (`docs/knowledge/ebnf-self-hosting-what-it-means.md`). If a future requirement
needs the meta-grammar to validate payload *structure*, that is a new decision, not an
extension of this one.

⛔ **Everything below this line is the ORIGINAL charter, retained for provenance. Its
deliverables 1, 2 and 4 are superseded by the analysis above** — do not execute them
without a director decision on A/B/C.

#### ⭐⭐⭐ MEASURED INPUT FROM `.10.5` — read this before designing, it moves the target

`.10.5` was opened by this leaf's own first measurement and closed three questions that
this charter (written before any of it was measured) could not have known. All three are
**constraints on the design**, not colour:

**1. The acceptance bar is now exact, and it is ONE construct.** With the meta-grammar's
swallow removed, the self-hosting corpus reads **11 of 12** and the single gap is
`regex.ebnf` at byte 55,925 = **line 843, the first `@dispatch: {`**. ⇒ `.10.2` has a
sharp, re-runnable success criterion it did not have before: *the definition it installs
for `semantic_annotation` must parse a **multi-line, brace-delimited annotation payload**,
and doing so returns the published count to 12/12 and re-greens
`ebnf_frontend_dual_run_gate`.* Run `run_metagrammar_quote_probes.sh` ARM C to re-measure;
its `multiline_payload` probe is the minimal reproducer.

**2. ⛔ The delegation target CANNOT parse the payload language PGEN actually uses — 9 of
148.** Measured by feeding every distinct annotation line in every tracked grammar (148
after dedup) through the shipped `semantic_annotation` parser
(`parseability_probe --parse semantic_annotation`): **139 ACCEPT, 9 REJECT.** The nine are
three classes, and two of them ship in grammars that are already fully certified:

| class | example | where it ships |
|---|---|---|
| **Rust-code payloads** (4) | `@transform: str::parse::<usize>().unwrap_or(0)` | `return_annotation.ebnf` ×3, `regex.ebnf` ×1 |
| **expression payloads** (3) | `@generate: "^" if $1 else ""`, `@generate: "ch == '" + escape_char($1) + "'"` | `regex.ebnf` |
| **multi-line payloads** (2) | `@dispatch_table: {` … `}` | `regex.ebnf` ×2 (`:1052`, `:1195`) — the same construct as (1) |

⇒ **restoring `include(semantic_annotation)` as-is would make the meta-grammar REJECT
annotation forms that tracked grammars ship today.** The composition problem is therefore
strictly larger than the 15 name collisions `.10.1` scoped: `annotation_value` must also
grow a payload form that accepts an opaque balanced-brace / to-end-of-line body, or the
delegation must be to a *fragment* whose value language is deliberately permissive at the
top. Adjudicate on measurement — the 148-line corpus is the oracle, and it is cheap to
re-derive:
```bash
grep -hE '^[[:space:]]*@[a-zA-Z_]' grammars/*.ebnf | sed 's/^[[:space:]]*//' | sort -u
```

**3. There are TWO swallow surfaces, and `.10.2` removes the second one.** Besides the
quote (fixed), the native `@`-to-end-of-line matcher codegen substitutes for the undefined
`semantic_annotation` accepts `@@@` as a well-formed inline annotation — measured,
`r = 'a' @@@` ⇒ ACCEPT. It is pinned as a declared-`ACCEPT` probe
(`native_slurp_swallows_at_text`) precisely so that when `.10.2` installs a real
definition, **that probe must flip to `REJECT`** — which is the cleanest single signal
that the delegation is genuinely doing the work rather than the fallback still being hit.
⚠️ Update the driver's declared verdict in the same leaf, or the instrument will report a
divergence for a fix working correctly.

⚠️ **`.10.5` also fixed `grammars/semantic_annotation.ebnf:146`**, so the two files no
longer disagree about what a single-quoted string is — one collision class fewer to
adjudicate here.

`ebnf.ebnf` delegates its annotation sub-language by design — it defines no
`annotation_name`/`annotation_value` and never has. The include was the mechanism.
Restore it for real:

1. Resolve the 15 collisions (4 duplicates, 11 conflicts) between `ebnf.ebnf` and
   `semantic_annotation.ebnf`. The two live options, to be adjudicated on measurement,
   not preference:
   - **extract a fragment** — a `semantic_annotation` core (the rule plus
     `annotation_name`/`annotation_value` and only the literals they need, under
     non-colliding names) that BOTH `semantic_annotation.ebnf` and `ebnf.ebnf` include.
     Fragments are an established shape here (`test_includes/*`); note the
     `QUANT-PLUS-ITER.2` constraint that **a fragment must not declare `@entry`**, or
     the spliced grammar declares two.
   - **rename on one side** — cheaper to write, but it moves a shipped parser's rule
     names and therefore its AST shape.
2. Restore `include(semantic_annotation)` (**singular**) in `ebnf.ebnf`.
3. Fix the misleading header in `grammars/semantic_annotation.ebnf` — it still says
   `# semantic_annotations.ebnf` and `# Usage: include(semantic_annotations)`. That
   header is what the typo was copied from; leaving it invites the same bug back.
4. ⚠️ **This changes a shipped parser's behaviour**: `ebnf.ebnf`'s `semantic_annotation`
   stops being an `@`-to-EOL slurp and becomes a structured `@name: value` parse.
   Expect AST-shape movement on the `ebnf` parser and budget for release/schema/ledger
   + `docs/ebnf_parser_book/` + `EBNF-BOOK` lockstep. Measure before→after; do not
   assume byte-identity.
   — ⛔ **MEASURED FALSE, see Fact 3 above.** The `ebnf` parser has **no** integration
   contract and **no** `ast_shape_contract` manifest, so there is no published surface to
   move; and nothing consumes its annotation AST (Fact 2). The contract-bearing parser is
   `semantic_annotation`, which this deliverable does not mention. The budget warning
   points at the wrong file.

#### What landed (option B)

`grammars/ebnf.ebnf` gains **7 rules** in a new, heavily-commented section, and
`grammars/semantic_annotation.ebnf`'s misleading header — `# semantic_annotations.ebnf` and
`# Usage: include(semantic_annotations)`, **the text the original typo was copied from** — is
corrected so the same bug cannot be copied out of it again.

```ebnf
semantic_annotation := "@" annotation_key ":" annotation_payload
annotation_payload  := (braced_payload | line_payload)   # brace-balanced, may span lines
braced_payload      := "{" brace_body "}"
brace_body          := (braced_payload | brace_text)*    # recursion = the nesting
```

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `ebnf_dual_run_diff --input grammars/regex.ebnf` reported
  `parse_full.ok=false`, unconsumed at **byte 55,925** (line 843, the first `@dispatch: {`),
  i.e. **35.55%** of the file consumed; `make -C rust ebnf_frontend_dual_run_gate` RED on its
  `regex` row; self-hosting **11/12**.
- [x] **ROOT CAUSE (WHY + WHERE)** — `grammars/ebnf.ebnf` referenced `semantic_annotation`
  from three sites (`grammar_file`, `annotation_list`, `inline_semantic_annotation`) and
  **defined it nowhere**; codegen's `NATIVE_UNRESOLVED_REFERENCE_BUILTINS` allowlist
  synthesized an `@`-to-END-OF-LINE matcher, which stops at the first newline and therefore
  cannot express a multi-line brace payload. Measured directly: the emitted
  `parse_semantic_annotation` was a **25-line** synthesized fallback, and the isolated
  `multiline_payload` probe REJECTed.
- [x] **FIX** — declarative tier, option B: define the rule locally as a **delimiter with an
  opaque payload**. Option A (restore `include(semantic_annotation)`) was rejected on
  measurement — see the three facts above.
- [x] **ADDRESSED (verified)** — measured before → after:

  | measurement | before | after |
  |---|---|---|
  | `regex.ebnf` under the meta-parser | **REJECT @ 55,925** (35.55% consumed) | **ACCEPT, 100.00%** ✅ |
  | self-hosting over the tracked corpus | 11/12 *(driver set 14/15)* | **12/12** *(driver **15/15**)* ✅ |
  | `make ebnf_frontend_dual_run_gate` | **RED** | **GREEN** — ebnf/json/regex all pass ✅ |
  | emitted `parse_semantic_annotation` | 25 lines (synthesized fallback) | **939 lines, generated from the rule**; all 6 helpers emitted ✅ |
  | `--lint-grammar` | 131 rules, all counters 0 | **138 rules, all counters 0** (no shadowing, no always-succeeds, no unreachable, no nullable-repetition) ✅ |
  | probe `native_slurp_swallows_at_text` (`r = 'a' @@@`) | ACCEPT (the fallback ate it) | **REJECT** ✅ — the single cleanest signal that the real definition, not the fallback, is in force |
  | probe `multiline_payload` | REJECT | **ACCEPT** ✅ |
  | the frontend's soft cross-check on `regex.ebnf` | `warning: … verification skipped` | **silent — it now passes** ✅ |

- [x] **NO REGRESSION** — `parse_harness_equivalence_gate` **GREEN 4/4** (`ebnf` is a
  CERTIFIED grammar and this leaf changed it, so this is the load-bearing oracle);
  `mdbook_docs_gate` green; `run_metagrammar_quote_probes.sh` **exit 0, 0 divergences** after
  its two predicted verdict flips; **`json`/`regex`/`return_annotation`/`semantic_annotation`
  parsers all BYTE-IDENTICAL** across a forced regeneration — the annotation backend did not
  move (`c5e41a8c…` unchanged), so the other grammars' codegen inputs are identical;
  `check_doctrines.sh` **ALL 15 PASS**.
- [x] **LOCKSTEP** — `docs/book/src/grammar-wellformedness.md` (**11/12 → 12/12**, with the
  full down-then-up account and the reason), the `.10.5` probe driver's declared verdicts
  (two predicted flips, plus its ARM A line numbers made **derived** after they went stale
  within one leaf), `CHANGES.md`, `LIVE_ACHIEVEMENT_STATUS.md`, `MEMORY.md`. **No
  contract/manifest movement** — `ebnf` has neither, which is Fact 3 above.

#### ✅ `.10.3` IS NOW UNBLOCKED

`.10.3`'s stated blocker was *"retiring the entry first would leave `ebnf.ebnf` with 3
references the linter now reports as a hard error and codegen compiles into never-matching
stubs"*. That is resolved: the references resolve to a real rule, so
`"semantic_annotation"` can leave `NATIVE_UNRESOLVED_REFERENCE_BUILTINS` and the linter can
be shown to report `undefined_references=0` **because the grammar is sound** rather than
because the name is allowlisted — the distinction `.7` could not make.


### `.10.3` — retire `"semantic_annotation"` from `NATIVE_UNRESOLVED_REFERENCE_BUILTINS` (`done`)

- **Status: `done`** (`PGEN-LANG-CAPABILITY-AUDIT-0023`, session #229, 2026-07-31).
  **CODE CHANGE** — `ast_based_generator.rs` (const + dispatch arm + oracle test),
  `first_set.rs` and `grammar_wellformedness.rs` (comment lockstep), 2 book chapters, the
  `.10.1` audit driver. **Every generated artifact BYTE-IDENTICAL**, so no
  release/schema/ledger/contract movement. Executed under the standing director approval
  of 2026-07-31 (*"If you need my greenlight, you have it"*), not re-asked —
  [[feedback_routine_decisions_are_not_escalations]].
- The blocker was discharged by `.10.2`: it read *"retiring the entry first would leave
  `ebnf.ebnf` with 3 references the linter reports as a hard error and codegen compiles
  into never-matching stubs"*. `.10.2` defined the rule, so the references now resolve.

#### What landed

The allowlist is **3 → 2 members**, and every one of the three names ever removed from it
was hiding a defect rather than expressing a primitive:

| member | removed by | what it actually was |
|---|---|---|
| `true` / `false` | `.10.4` | unconditional **zero-width** matchers — `"T" true "T"` accepted `TT` |
| `semantic_annotation` | **`.10.3`** | an `@`-to-END-OF-LINE slurp standing in for `ebnf.ebnf`'s broken `include` — and a *correctness ceiling*, since a line-bounded matcher cannot express `@dispatch: { … }` |
| `builtin_any_char` / `builtin_ascii_char` | — | ✅ the genuine, `builtin_`-namespaced primitives |

Concretely: the const entry (`ast_based_generator.rs:1222`), the 28-line dispatch arm
(`:1322`), and the false doc-comment rationale all go; the removals are *recorded in place*
so the next reader learns why each name may not come back. The
`native_unresolved_builtins_const_matches_dispatch` oracle drops 3 → 2 and its
absence-pin list grows to all three retired names, each with its own reason string, so
re-adding one while dropping another cannot slip through on an unchanged length.

⭐ **`first_set.rs` needed no arm removal** (the `.10.3` prep flagged this as a thing to
check, correctly): `semantic_annotation` was never in `native_builtin_first_set` because
its matcher skipped leading layout before requiring `@`, which makes first-byte peeking
unsound. Its doc bullet now records that the same conclusion holds for a stronger reason —
the name is an ordinary undefined reference, not a builtin that happens to be unsummarizable.

#### ⭐⭐ THE SHARP ORACLE HELD — `generated/ebnf.rs` is BYTE-IDENTICAL

`.10.2` proved the fallback was no longer *reached*; this leaf proves it was no longer
*there*. Regenerating the meta-parser before and after the removal:

```
7ce6578f799c36c79343f98c1e441feb70b453eee25be860ae64824f751938e5  before/generated/ebnf.rs
7ce6578f799c36c79343f98c1e441feb70b453eee25be860ae64824f751938e5  after/generated/ebnf.rs
7ce6578f799c36c79343f98c1e441feb70b453eee25be860ae64824f751938e5  generated/ebnf.rs   (the shipped artifact)
```

Had one byte moved, `.10.2` would have been incomplete — a far sharper signal than *"the
lint still says 0"*.

⚠️ **`.10.1`'s output-path trap is real and it fired here first.** The generated parser
**embeds its own `-o` path as a string literal, 3,231 times**
(`let filename_str = "generated/ebnf.rs";` and its error-reporting siblings), so a naive
`-o /tmp/probe.rs` regeneration diffs against the shipped artifact in 6,462 lines and
proves nothing. The protocol that works: run each arm from a scratch working directory
containing its own `generated/`, with the *identical* relative `-o generated/ebnf.rs`. The
input JSON path is **not** embedded, so only `-o` needs pinning. A **positive control** ran
first — the before-arm reproduces the shipped artifact byte-for-byte — so a later match is
evidence rather than coincidence ([[feedback_instrument_needs_ground_truth]]).

#### The un-masking, measured both directions

The point of the leaf is not the deletion; it is that the check can now see the name:

| probe | before | after |
|---|---|---|
| `probe := "C" semantic_annotation "C"` (undefined) | `undefined_references=0` — **invisible** | **`=1`**, and the linter NAMES the rule ✅ |
| the `.10.1` 4-dangling-reference probe grammar | `=2` | **`=3`** ✅ |
| `grammars/ebnf.ebnf` (138 rules) | `=0` *(because allowlisted)* | **`=0` — because the grammar is SOUND** ✅ |
| emitted `parse_semantic_annotation` on a dangling reference | the 28-line `@`-slurp | **the bare `Err(Backtrack)` stub** ✅ |
| scratch-slot behaviour on `A@name: value` | ACCEPT | **REJECT** ✅ |

That last row is the whole leaf in one line: the third column of the `.7` claim — *"reports
0 **because the grammar is sound**"* — is finally a statement the instrument can make about
itself. Every other tracked grammar was swept too: **no tracked grammar has a dangling
`semantic_annotation`** (only `ebnf.ebnf` and `semantic_annotation.ebnf` name it, and both
define it), which is why nothing else could move.

#### ⚠️ Two stale artifacts found while working this leaf

1. **`.10.2` left the `.10.1` audit driver RED and did not notice.** Defining
   `semantic_annotation` in `ebnf.ebnf` made it a 16th name colliding with
   `semantic_annotation.ebnf` and a 12th genuine conflict, but the driver still declared
   `15`/`11` — so `run_native_builtin_audit.sh` reported **2 divergences** from
   `PGEN-LANG-CAPABILITY-AUDIT-0021` until this leaf re-declared them. Both numbers now
   carry the reason they moved, so the movement reads as the fix it is. ⭐ The general
   shape: **a leaf that changes a grammar must re-run every declared-verdict driver that
   measures that grammar**, not only the gates.
2. `grammar_wellformedness.rs:178` justified its include-handling with *"e.g. ebnf.ebnf's
   `annotation_list := semantic_annotation+`"* — which was never an include, it was the
   dangling reference. Re-pointed at the one tracked grammar that genuinely uses
   `include(...)`, `systemverilog_lrm_profiled_wrapper.ebnf` (measured: 1 of 18).

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the allowlist masks the name. Measured on two one-rule probe
  grammars through `ast_pipeline --lint-grammar`: `probe := "C" semantic_annotation "C"` →
  `undefined_references=0 (error)`, while the identical grammar naming a non-member →
  `undefined_references=1 (error)` with the `[error] … references UNDEFINED rule` line.
  Same instrument, same shape of grammar, opposite verdicts — the allowlist is the only
  difference.
- [x] **ROOT CAUSE (WHY + WHERE)** — `--lint-grammar` consumes
  `AstBasedGenerator::NATIVE_UNRESOLVED_REFERENCE_BUILTINS`
  (`grammar_wellformedness.rs:332-343`) as its allowlist, and `"semantic_annotation"` was
  its third member (`ast_based_generator.rs:1222`), paired with a 28-line native-matcher
  dispatch arm at `:1322` that slurped `@`→end-of-line. WHERE it was consumed:
  `--report-certificate-coverage`-style census over `generated/*.rs` shows the emitted
  `parse_semantic_annotation` came from the grammars' OWN definitions —
  `grep -c "as_bytes()\[start_pos\] != b'@'" generated/ebnf.rs` → **0**, i.e. the
  synthesized matcher was present in **no** shipped parser. Dead surface holding a
  diagnostic hostage.
- [x] **FIX** — engine tier, deletion only: drop the const member and its dispatch arm; no
  grammar and no declarative surface moves (`.10.2` had already done the declarative half).
- [x] **ADDRESSED (verified)** — the un-masking table above, plus the byte-identical
  oracle. Re-runnable: `bash docs/tasks/artifacts/lang_capability_audit/run_native_builtin_audit.sh`
  → **`✅ 0 divergences`** across arms A+B, and `PGEN_AUDIT_RUN_SLOT_ARM=1` → **7/7 OK**
  through the PARSE-HARNESS scratch slot (authoritative by construction), with
  `A@name: value` flipping **ACCEPT → REJECT**. Capture:
  `docs/tasks/artifacts/lang_capability_audit/native_builtin_audit.txt`.
- [x] **NO REGRESSION** — **every generated artifact byte-identical** across a canonical
  per-family regeneration (`make -C rust focus_{json,regex,systemverilog,systemverilog_preprocessor,vhdl,rtl_const_expr,rtl_frontend}`
  + `annotation_parsers`, plus the path-pinned `generated/ebnf.rs` arm above): all 11
  `*_parser.rs` + `ebnf.rs` unchanged; the `*.json` raw-AST envelopes differ only in their
  `generated_at` stamp, which is not content. Unit oracles green:
  `native_unresolved_builtins_const_matches_dispatch` (the const↔dispatch lock, now pinned
  at 2) and the three `detect_undefined_reference_*` tests. `parse_harness_equivalence_gate`
  **4/4 GREEN** (`certified_grammars_are_byte_identical` + 3 classification tests, 36.4 s).
  `make -C rust clippy_on_rust_change` **exit 0** (peak 9,243 MB / 201 s): source-strict
  `clippy_source_all_targets` ok, `generated_clippy_correctness_policy` **10/10 artifacts + 68/68
  pinned lints still in `clippy::correctness`**, and the deny-by-default
  `clippy_generated_all_targets` stage **pass**. Independently, every clippy finding in the three
  edited files (`ast_based_generator.rs` :25/:813/:1300/:3731/:5464/:5597/:5602/:5884/:6218,
  `grammar_wellformedness.rs:264`, `first_set.rs` none) sits **outside** the edited regions and
  its construct is present verbatim at `HEAD` — checked, not assumed.
  `mdbook_docs_gate` (10/10 per-parser books + the live book) and `ebnf_parser_book_gate` both
  GREEN. `scripts/check_doctrines.sh` **ALL 15 PASS** against the real staged diff, so the four
  staged-scope doctrines bound rather than passing vacuously.
- [x] **LOCKSTEP** — `docs/book/src/grammar-wellformedness.md` (the allowlist is two names;
  the full three-removal account and why `undefined_references=0` now means something
  different), `docs/ebnf_parser_book/src/terminals.md` (*"The complete built-in list"* —
  three names → two, with the author-facing reason the list being short is what makes the
  check trustworthy), the `.10.1` driver + its capture, `CHANGES.md`,
  `LIVE_ACHIEVEMENT_STATUS.md`, `MEMORY.md`. No contract/manifest movement — nothing
  generated moved.

### `.10.4` — `true` / `false`: always-succeeding zero-width builtins nobody uses (`done`)

- **Status: `done`** (`PGEN-LANG-CAPABILITY-AUDIT-0014`, session #213, 2026-07-26).
  **CODE CHANGE** — `ast_based_generator.rs` + `first_set.rs` + 2 book chapters + the
  `.10.1` driver. **All 11 generated parsers BYTE-IDENTICAL** ⇒ no release/schema/ledger/
  contract movement. Opened by `.10.1`'s Finding 3; independent of `.10.2`/`.10.3`.

#### The decision, and why option 2 lost

`.10.1` left two options. **Option 1 (delete) is taken**, on the standing directive
rather than on preference: the `builtin_` prefix exists (director 2026-06-07) precisely
so a primitive can never be shadowed by an ordinary grammar rule, and `true`/`false` are
un-prefixed. Option 2 (*"make them honest"* — emit a real matcher consuming the literal
text) would have **entrenched a directive violation** to deliver a capability that has
no user: no tracked grammar references either name, and every grammar that wants those
words already writes them as quoted terminals (`("true" | "false")`), a path that never
touched this code. Nothing measured argued for keeping them.

#### What changed

| site | change |
|---|---|
| `ast_based_generator.rs` const | `true` / `false` removed ⇒ 5 members → **3** |
| `ast_based_generator.rs` dispatch | both `quote!` arms removed (a comment records what stood there) |
| `first_set.rs` ×3 | the `"true" \| "false"` arms in `native_builtin_first_set`, `native_builtin_second_byte`, `native_builtin_prefix_trie` — all three modelled the names as nullable/epsilon |
| tests ×3 | const-length 5→3 **plus a new absence assertion**; the `first_set` test now requires both names to read *unresolved*; the codegen test rewritten (below) |
| books ×2 | `docs/ebnf_parser_book/src/terminals.md` (the list is now 3, with the removal as history) and `docs/book/src/grammar-wellformedness.md` (the allowlist enumeration **plus** the sharp edge `.10.1` found) |

⭐ **`first_set.rs` is why the byte-identity sweep was necessary rather than ceremonial.**
FIRST-set facts are compiled into emitted prune guards, so this change touched a path
that *can* reach codegen. Only a full regeneration could prove it did not.

#### ⚠️ A THIRD test was pinning the bug — the same pattern `.8` found

`.10.1`'s allowlist sweep did not find `unresolved_reference_codegen_emits_semantic_and_boolean_fallbacks`,
because it greps `rust/tests/` and the test is **inline in the 14k-line generator**. It
asserted:

```rust
assert!(rendered.contains("\"true\""),
        "expected parse_true fallback to materialize boolean content");
```

That passes on an unconditional zero-width matcher, because the emitted payload literal
`Terminal("true")` is all it looks at — **a render-level `contains` cannot see that the
matcher never consumes input.** This is exactly `.8`'s finding (`contains("2usize")`
pinning a type error) in a second place. Renamed to
`unresolved_reference_codegen_emits_semantic_fallback_and_stubs_boolean_names` and
rewritten to assert *structure*: `parse_true` must contain `Backtrack` and must **not**
contain `Ok(ParseNode`. ⇒ **banked lesson: a `contains` assertion over rendered tokens
tests spelling, not behaviour** — third occurrence, and the rule-of-three is now met.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `.10.1` measured it through the scratch slot:
  `probe_true := "T" true "T"` **ACCEPTED `TT`**, i.e. the reference matched empty,
  while `--lint-grammar` reported `undefined_references=0`.
- [x] **ROOT CAUSE (WHY + WHERE)** — `ast_based_generator.rs`, the `"true"`/`"false"`
  arms of `generate_unresolved_reference_method`: `Ok(ParseNode{ …,
  span: Span::new(start_pos, start_pos) })` — unconditional success, zero-width span, no
  `self.position` advance. Invisible to the linter because
  `detect_undefined_references` (`grammar_wellformedness.rs:333`) consumes the same const
  as its allowlist.
- [x] **ADDRESSED — measured before → after**, one probe grammar, one build:

  | measurement | before | after |
  |---|---|---|
  | `"T" true "T"` on `TT` | **ACCEPT** | **REJECT** ✅ |
  | `"F" false "F"` on `FF` | **ACCEPT** | **REJECT** ✅ |
  | `"T" true "T"` on `TtrueT` | REJECT | REJECT (unchanged) |
  | `undefined_references` on that grammar | **0** | **2** ✅ |
  | linter names `true` / `false` | no | **yes, both** ✅ |
  | emitted `parse_true` | unconditional `Ok`, zero-width | bare `Err(Backtrack)` stub ✅ |
  | `"A" semantic_annotation` on `A@name: value` | ACCEPT | ACCEPT (unchanged) |
  | `"C" builtin_any_char "C"` on `CzC` / `CC` | ACCEPT / REJECT | ACCEPT / REJECT (unchanged) |

- [x] **NO REGRESSION** — **all 11 generated parsers BYTE-IDENTICAL** across a
  before→after sweep running the *identical* script in both arms (`focus_json`,
  `focus_scratch`, `focus_rtl_const_expr`, `focus_systemverilog_preprocessor`,
  `focus_rtl_frontend`, `focus_vhdl`, `focus_regex`, `focus_systemverilog`,
  `return_annotation_parser`, `semantic_annotation_parser`, and `ebnf` via the direct
  route with its output path pinned). SV (150 MB) regenerated for real in both arms.
  Unit tests green (45 passed across `first_set` / `native_unresolved` /
  `undefined_reference` / `unresolved_reference`); driver re-run **exit 0, 0
  divergences**; clippy source-strict pass green.

⚠️ **Two sweep-methodology traps banked** (both silently produce a *false* green):
1. **`make` no-ops.** The first baseline arm finished in 91 s at 2.2 GB without
   rebuilding SystemVerilog — the artifacts were already up to date, so the "baseline"
   was untouched files, and the after arm (forced to rebuild by the source edit) would
   have been compared against them. The sweep script now `touch`es the two sources in
   **both** arms so both genuinely regenerate.
2. **The `focus_*` targets rebuild `ast_pipeline` with THEIR feature set**, dropping
   `ebnf_dual_run` — so the `ebnf` step died with *"requires building with --features
   ebnf_dual_run"* mid-sweep. The script now rebuilds the binary with the feature
   *after* the make targets, in both arms.

#### Residual, deliberately not absorbed

`always_succeeds_alternatives` still cannot see through the allowlist: a rule reference
resolving to an unconditional zero-width `Ok` is exactly what that note exists to
report, and it would not have fired here. With `true`/`false` gone, the only remaining
allowlist member that could exhibit it is `semantic_annotation`, which is slated for
retirement by `.10.3` — so the gap has no live instance and is recorded rather than
fixed speculatively. **Re-open if a new member is ever proposed.**


### `.10.5` — `single_quoted_string` never closed its quote, so the meta-parser's green verdict was a FALSE GREEN (`done`)

- **Status: `done`** (`PGEN-LANG-CAPABILITY-AUDIT-0015`, session #228, 2026-07-30).
  **CODE CHANGE** — two grammar files, one character each. Opened by `.10.2`'s own
  measurement and it **blocked `.10.2`**: the regression oracle `.10.2` was going to
  verify against ("the tracked grammars still parse") did not mean what it says while
  this defect stood.

#### ⭐⭐⭐ THE HEADLINE — an instrument that could not fail

`.10.2` needs one number before it can move: *which tracked grammars does the
self-hosting meta-parser accept today?* Measured through `ebnf_dual_run_diff` (the tool
that runs `generated/ebnf.rs` directly, with none of the frontend's soft-skip gating):
**15 of 18 ACCEPT.** That looked like a healthy baseline. It is not a baseline at all.

```
grammars/ebnf.ebnf:237  double_quoted_string := /"([^"\\]|\\.)*"/     ← closing quote present
grammars/ebnf.ebnf:240  single_quoted_string := /'([^'\\]|\\.)*/      ← ⛔ NO CLOSING QUOTE
```

One character. The rule opens a `'`, consumes everything that is not a quote or a
backslash — **including newlines** — and then simply ends. So a `'` swallows the rest of
the file up to the next `'` or `\`, and the meta-parser reports a full, clean parse.

⭐ **The decisive contrast, measured** (`run_metagrammar_quote_probes.sh` ARM B, 10
declared verdicts, ground truth pinned inside the instrument):

| probe | input | before | what it proves |
|---|---|---|---|
| `sq_unterminated` | `r = 'a` | **ACCEPT** ⛔ | an unterminated quote parses clean |
| `sq_swallows_garbage` | `r = 'a' ]]]` | **ACCEPT** ⛔ | text no EBNF construct can claim is swallowed |
| `sq_swallows_block` | a `'` before a multi-line annotation | **ACCEPT** ⛔ | the swallow crosses newlines |
| `dq_unterminated` | `r = "a` | REJECT ✅ | **the twin one line above behaves correctly** |

The negative control is what makes this un-arguable: the double-quoted rule shares every
code path with the single-quoted one *except the one character*, and it rejects. This is
a typo, not a design.

#### ⛔ WHY IT MATTERS MORE THAN A TYPO — `regex.ebnf` "passes" for the wrong reason

The meta-grammar genuinely **cannot** parse a multi-line annotation payload — measured in
isolation, probe `multiline_payload`, `REJECT`:

```ebnf
@dispatch_table: {          ← the native `@`-to-END-OF-LINE slurp stops here
    "x": "y"                ← and NOTHING in grammar_file's alternation matches this line
}
```

`grammars/regex.ebnf` contains two such blocks (`:1052`, `:1195`) and nevertheless
reports `parse_full.ok = true` over all 157,319 bytes. Bisected to the minimal trigger:

| probe | preceding rule | verdict |
|---|---|---|
| `pF` | `r = "a" \| "b"` | **REJECT** — the honest answer |
| `pJ` | `r = '"' \| "b"` | **ACCEPT** ⛔ |

`regex.ebnf:1048` is `class_safe_special = '[' | '!' | … | '"' | "'" | …` — **29
single-quoted terminals**, not one of which the broken rule could close. The parse
desynchronizes there and the dangling quote eats the annotation block 200 lines later.
(What the bisect establishes is that a *single-quoted terminal* before the block flips the
verdict — `r = "a" | "b"` REJECTs, `r = '"' | "b"` ACCEPTs; it does not single out which of
the 29 desynchronizes first, and nothing depends on that.) ⇒ the
meta-parser's `ACCEPT` over `regex.ebnf` is **not evidence that it understood those
bytes** — it is evidence that it stopped looking. A `parse_full.ok = true` produced this
way cannot be the oracle for `.10.2`, and it had been the oracle for every prior claim
about the meta-parser's reach.

⇒ Same family as the rest of this container, now for the **fifth** time: `.7`'s
`undefined_references=0`, `.10`'s allowlist masking, `.10.1`'s `true`/`false` zero-width
matchers, `.10.4`'s `contains`-assertion test — and now a grammar rule that makes the
parser itself unfalsifiable. **An instrument that cannot fail is not reporting; it is
agreeing.**

#### The fix, and the bug-class sweep

Add the closing quote. Both instances, in one leaf — the sweep (ARM A) enumerates every
quote-opening regex terminal in every tracked PGEN-authored grammar and adjudicates each:

| site | verdict |
|---|---|
| `grammars/ebnf.ebnf:240` `single_quoted_string` | ⛔ **DEFECT** — fixed here |
| `grammars/semantic_annotation.ebnf:146` `single_quoted_string` | ⛔ **DEFECT**, byte-identical twin — fixed here |
| `ebnf.ebnf:237` / `semantic_annotation.ebnf:143` `double_quoted_string` | ✅ closing quote present |
| `ebnf.ebnf:244` `raw_quoted_string`, `semantic_annotation.ebnf:149` `raw_string`, `:152` `multiline_string`, `json.ebnf:46` `string` | ✅ closing delimiter present |
| `builtin_semantic_annotation.ebnf:87/88` `dq_char`/`sq_char` | ✅ N/A — per-CHARACTER rules; the quotes are matched by their wrappers (`sq_string := "'" sq_char* "'"`) |
| `return_annotation.ebnf:132/133` `string_content_*` | ✅ N/A — content-only rules inside an explicit quoted wrapper |
| `systemverilog.ebnf:453` `integral_number` | ✅ N/A — the SV sized-literal apostrophe (`8'hFF`), not a delimiter |

**Exactly two instances.** No silent second instance.

⚠️ **The twin is not optional scope.** `.10.2` composes these two files into one grammar;
fixing one and leaving the other would carry the defect straight into the composition it
is meant to make sound.

#### ⚠️ What this leaf deliberately does NOT fix — routed, not absorbed

Closing the quote makes the multi-line-annotation gap **visible** rather than swallowed.
That gap is not this leaf's to close: `ebnf.ebnf` has no definition for
`semantic_annotation` at all — it is the dangling reference this whole container exists
to repair, and defining it is **`.10.2`'s** charter. Routed there with the measurement
attached, and the driver pins the newly-visible verdict (`multiline_payload REJECT`) so
it cannot regress to a swallow again.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `ebnf_dual_run_diff --input <probe> --output <json>` on
  `r = 'a` (an unterminated quote) reports `parse_full.ok = true`. Driver ARM B, before
  arm: the 3 declared-`REJECT` defect probes all measured `ACCEPT`
  (`sq_unterminated`, `sq_swallows_garbage`, `sq_swallows_block`), while both
  ground-truth controls read correctly — so the instrument was honest and the grammar
  was not.
- [x] **ROOT CAUSE (WHY + WHERE)** — `grammars/ebnf.ebnf:240`, the `single_quoted_string`
  regex terminal `/'([^'\\]|\\.)*/`: the closing `'` is absent, so the match runs to the
  next `'`/`\` across newlines. Its twin at `grammars/semantic_annotation.ebnf:146` is
  byte-identical. Located by bisection through `ebnf_dual_run_diff`'s
  `unconsumed_start`, pinned by the `pF`/`pJ` differential (`r = "a" | "b"` before the
  block ⇒ `REJECT`; `r = '"' | "b"` before the same block ⇒ `ACCEPT`) and by the correct
  `double_quoted_string` twin one line above.
- [x] **FIX** — declarative tier, the top of the fix hierarchy: one character per file,
  in the grammar. No engine, codegen, or Rust change.
- [x] **ADDRESSED (verified)** — `run_metagrammar_quote_probes.sh` **exit 0, 0
  divergences**, 10 declared verdicts, ground truth pinned inside the instrument
  (`metagrammar_quote_probes.txt`). `generated/ebnf.rs`
  `bc9c4750…` → `6088b8d6…`; the emitted matcher now carries `'([^'\\]|\\.)*'`.

  | probe | before | after |
  |---|---|---|
  | `sq_unterminated` — `r = 'a` | **ACCEPT** ⛔ | **REJECT** ✅ |
  | `sq_swallows_garbage` — `r = 'a' ]]]` | **ACCEPT** ⛔ | **REJECT** ✅ |
  | `sq_swallows_block` — odd `'` before a multi-line annotation | **ACCEPT** ⛔ | **REJECT** ✅ |
  | `quoted_ok` — `r = 'abc'` (POSITIVE control) | ACCEPT | ACCEPT (unchanged) |
  | `dq_unterminated` — `r = "a` (NEGATIVE control) | REJECT | REJECT (unchanged) |
  | `sq_in_dq` / `dq_in_sq` — a quote inside the other quote | ACCEPT | ACCEPT (unchanged) |
  | `single_line_annotation` / `rust_code_annotation` | ACCEPT | ACCEPT (unchanged) |

- [x] **NO REGRESSION** — five independent legs, every one a named re-runnable oracle:

  1. **`make -C rust SHELL=/bin/bash parse_harness_equivalence_gate` GREEN** — 4/4 tests,
     all **11 CERTIFIED grammars byte-identical** interpreter-vs-generated-parser. This
     is the certifying oracle for *both* changed grammars: `semantic_annotation` and
     `ebnf` are both on the CERTIFIED list.
  2. **9 of 11 generated parsers BYTE-IDENTICAL** across a real regeneration (every
     grammar source `touch`ed first, so no `make` no-op could fake it — the `.10.4` trap):
     `json`, `scratch`, `rtl_const_expr`, `rtl_frontend`, `vhdl`,
     `systemverilog_preprocessor`, `return_annotation`, **`regex`** (the grammar carrying
     the affected annotations) and **`systemverilog`** (150 MB, regenerated for real).
     The only two that moved are the two intended: `generated/ebnf.rs`
     `bc9c4750…`→`6088b8d6…` and `generated/semantic_annotation_parser.rs`
     `acccd1e7…`→`c5e41a8c…`.
  3. **The annotation corpus is unmoved: 139/148 ACCEPT in BOTH arms, and the 9-line
     reject set is `diff`-identical.** `semantic_annotation_parser.rs` is the annotation
     backend every other grammar's codegen runs on, so this is the leg that matters most.
     ⭐ It has an analytical companion that explains *why* it could not move: of the 148
     distinct annotation lines in every tracked grammar, only **2** contain a single quote
     at all, both with an **even** count, and both are already in the reject set ⇒ no
     ACCEPTED annotation line exercises `single_quoted_string`.
  4. **`make -C rust SHELL=/bin/bash generated_clippy_correctness_gate` GREEN** —
     `GENERATED-CLIPPY-CORRECTNESS: findings: total=0 (expected 0)` across 10 required +
     1 optional generated artifacts, 68 pinned lints all still in `clippy::correctness`.
  5. **`bash scripts/check_doctrines.sh` — ALL 15 doctrines PASS**, plus the
     `<meta:mirror>` check.

  ⛔ **The one thing that DID change is stated as its own finding below rather than
  buried here** — `ebnf_frontend_dual_run_gate`'s `regex` row. It is a false green
  removed, not a capability lost, and the loading path does **not** hard-fail: the
  equivalence-gate run shows the frontend's soft cross-check reporting it exactly as
  designed — `warning: Rust EBNF generated-parser verification skipped for 'regex':
  Parser did not consume full input at position 55925`.
- [x] **LOCKSTEP** — `docs/book/src/grammar-wellformedness.md` (the published
  self-hosting count corrected 12/12 → 11/12, with the reason, the exact locus, and the
  general lesson); `LIVE_ACHIEVEMENT_STATUS.md` dated tracker note; `CHANGES.md`;
  `DEVELOPMENT_NOTES.md`; `MEMORY.md`. No contract/release/schema/ledger movement — the
  meta-grammar ships no downstream parser contract.

#### ⛔⛔ THE CONSEQUENCE, STATED PLAINLY — a gate turns RED, and it should

`make -C rust ebnf_frontend_dual_run_gate` gates exactly `ebnf`/`json`/`regex` on
`parse_full.ok`. **`regex` now fails it.** That is not a regression this leaf introduced;
it is the gate's `regex` row reporting for the first time. The full before→after over the
tracked corpus (driver ARM C):

| grammar | before | after |
|---|---|---|
| `regex.ebnf` | ACCEPT (**false green** — the payload blocks were swallowed) | **REJECT @ byte 55,925 = line 843, the first `@dispatch: {`** |
| the other 11 tracked grammars | ACCEPT | ACCEPT — **all unchanged** |
| the 3 IEEE-LRM extraction snapshots | REJECT | REJECT — unchanged (not tracked self-hosting artifacts) |

⇒ published self-hosting **12/12 → 11/12**, and the 12/12 was never true. The single
remaining gap is one thing, precisely located: the meta-grammar cannot express a
multi-line annotation payload, because `ebnf.ebnf` defines no `semantic_annotation` and
codegen substitutes an `@`-to-end-of-line slurp. **That is `.10.2`'s charter**, it is this
container's declared frontier, and closing it is what earns 12/12 back — honestly this
time.

⭐ **Why the red is the right outcome rather than something to work around.** The
`FLOW-INTEGRITY` doctrine already carries the invariant *"no assertion requires a defect
to pass"*, adopted after a gate was found that passed only when the parser FAILED. This is
that shape exactly, one layer deeper: the assertion did not require a defect, it was
*satisfied* by one. Preserving the green would have meant preserving the swallow.

#### ⚠️ A declared verdict of mine was wrong, and the instrument caught it

The first `sq_swallows_garbage` probe was `r = 'a' THIS IS GARBAGE @@@ !!! (((`, declared
`REJECT`. After the fix it still measured `ACCEPT`, and the fix was not at fault — the
probe was confounded two ways: `THIS IS GARBAGE` are **legal non-terminal references** in
EBNF, and `@@@ !!! (((` is eaten by the *native `@`-to-end-of-line slurp*, a second and
entirely separate swallow surface. Decomposed with the tool: `r = 'a' @@@` ⇒ ACCEPT,
`r = 'a' !!!` ⇒ REJECT@16, `r = 'a' (((` ⇒ REJECT@8, `r = 'a' ]]]` ⇒ REJECT@8. The probe
now uses `]]]`, which no EBNF construct can claim, and the native-slurp behaviour is
**pinned as its own declared-`ACCEPT` probe** (`native_slurp_swallows_at_text`) so that
defect — owned by `.10.2`/`.10.3` — cannot change silently either. ⇒ banked: **two
independent swallow surfaces existed, and a probe that does not separate them measures
neither.**

---

### `.10.6` — the "dual run" diffs the wrong pair, and a guard PINS the Perl frontend the project believes it retired (`done` — ✅ PART 1 de-Perl, ✅ PART 2 envelope differential)

- **Status: `done`.** Opened session #228 from two director statements, both of which
  measurement partly confirmed and partly refuted. **Part 1 — the de-Perl — is DONE**
  (`PGEN-LANG-CAPABILITY-AUDIT-0020`), on the director's explicit order: *"Do this first
  (1) .10.6 finish the de-Perl, then do (2) .10.2 option B"*. **Part 2 — the
  frontend↔meta-parser envelope differential — is DONE** (`PGEN-LANG-CAPABILITY-AUDIT-0029`),
  and it found two live `ebnf.ebnf` defects on its first run, routed to `.10.13` / `.10.14`.
  Owning machinery is the flow/CI gates + `GRAMMAR-WELLFORMED` (H.13/H.14 own the meta-grammar
  lockstep); recorded here because `.10` found it and it conditions `.10.2`.

#### Statement 1 — *"ebnf_dual_* runs both parsers and compares their output live"*

**Right that both run live; wrong that the outputs are compared.** Three pairs run, and
none of them is frontend-output ↔ meta-parser-output:

| pair | what is compared | where |
|---|---|---|
| hand-written frontend ↔ **generated `ebnf.rs`** | **verdict only** (`Ok`/`Err`); **soft** — warns unless `PGEN_EBNF_FRONTEND_REQUIRE_GENERATED_VERIFY=1` | **live, every grammar load** — `ebnf_frontend.rs:65-79` |
| **Perl** `ebnf_to_json.pl` ↔ hand-written frontend | **rule-NAME sets** — `sorted(set(names))` over `["rule", name]` heads; not bodies, not tokens | `ebnf_frontend_dual_run_diff_gate.sh:224-260` |
| interpreter ↔ generated `ebnf.rs` | byte-identical AST ✅ | `parse_harness_equivalence` (`ebnf` CERTIFIED) |

⇒ the raw-AST differential that would prove `ebnf.ebnf` ready to **replace** the
hand-written frontend — the stated endgame (`README.md:28`, *"handwritten parsers exist
only as bootstrap scaffolding"*) — **has never been built**. `12/12` says the spec still
*reads* every grammar we ship; nothing says it reads them the *same way*. The gate's output
diff is measuring a **migration that is already finished** (Perl → Rust), not the one that
is live.

#### Statement 2 — *"we ditched any Perl EBNF parser long ago"* ⛔ MEASURED FALSE

⚠️ **Correction to this leaf's own first revision**, which called the Perl arm *"a frozen
legacy reference, not a live second opinion"* on the strength of its mtime. That was an
over-claim from a weak signal, and running it refuted it: Perl still parses, and agrees
**exactly** with the Rust frontend on `ebnf` (131/131) and `json` (9/9).

The truth is the opposite of retired — **Perl is mechanically pinned in place**:

| site | role | measured |
|---|---|---|
| `ebnf_stimuli_quality_gate.sh:582` · `ebnf_frontend_readiness_gate.sh:127` | selectable frontend branch | ✅ **already migrated** — `FRONTEND_IMPL="${PGEN_EBNF_FRONTEND_IMPL:-rust}"`, so the **Rust** path is the default and the Perl branch is dead unless someone opts in |
| ⛔ `ebnf_stimuli_quality_gate.sh:593` (+ `require_file` at `:555`) | **THE ONE MISSED SITE** — hard-coded to Perl, **not** routed through the script's own `run_frontend_to_json()` helper, so it fires whenever `require_ebnf_parseability` is on **regardless of `FRONTEND_IMPL`**; the `require_file` makes the gate **refuse to start** without the Perl script | an invoked target (`make -C rust ebnf_stimuli_quality_gate`) |
| `ebnf_frontend_dual_run_diff_gate.sh:110,167` | the differential reference arm — an oracle built to de-risk the Perl→Rust migration | that migration is **complete**, so the oracle's purpose has **expired**; it under-reports **25 of 276** rules on `regex.ebnf` and the gate **passes** that as `perl_under_reports` |
| ⛔⛔ `ci_workflow_local_gate.sh:810-823` | **`assert_file_contains` on all three scripts** | **removing the Perl calls FAILS the gate** — this is what froze the other rows in place |

⇒ **the answer to "why do we even need it" is: we do not.** Nothing requires Perl
functionally. It survives on exactly three things — **one line missed by an otherwise
complete migration** (`:593`), **one expired migration oracle**, and **one guard that
asserts its presence**. The director's de-Perl campaign did land; it stopped one line
short, and then a parity gate cemented the remainder.

⇒ the de-Perl guard `assert_file_not_contains_uncommented "$repo_file" "ebnf_to_json.pl"`
(`:804`) keeps Perl out of *other* files, while the very next lines **positively require**
it in these three. **A guard that pins the thing it was built to remove.** Meanwhile the
book tells users *"No Perl is required"* (`docs/regex_parser_book/src/quickstart.md:28`)
and the regex contract says it is *"retained but not the recommended path"*.

⚠️ **And the carve-out's own justification is stale**: it cites *"README.md `perl/:
legacy/frontend EBNF-to-JSON path … still used in hybrid flow`"* — **`README.md` no longer
mentions Perl at all**, that inventory having been trimmed out by `README-POLICY.1`. The
pin now rests on a citation to a line that does not exist.

⭐ The 25 rules Perl cannot see in `regex.ebnf` are the **modern** surface — the whole
`code_*` family (`code_block`, `code_balanced_braces`, `code_string_*`, …) plus
`any_char`/`digit`/`letter`/`whitespace`/`unicode_char`. So the arm is blind precisely where
the language has grown since it was written, and the gate is configured to accept that.

#### What this leaf should do — one coherent piece of work

1. **Build the differential that matters**: diff the **token envelopes**
   (`["rule", …]`, `["semantic_annotation", [name, payload]]`, `["group_open","("]`, …)
   between the hand-written frontend and the `ebnf.ebnf`-derived parser, not just the
   verdict. ⚠️ Expect RED on day one — `ebnf.ebnf`'s return annotations emit
   `{type: "grammar_file", elements: […]}` while the frontend emits the raw_ast token
   envelope. **Closing that shape gap IS the replacement work**, and this instrument sizes
   it; today nobody knows how big it is.
2. **Then retire the Perl arm — it is a 3-step finish, not a campaign.**
   (a) route `ebnf_stimuli_quality_gate.sh:593` through the script's **own**
   `run_frontend_to_json()` helper and drop the `require_file` at `:555` — that alone ends
   the last unconditional Perl execution; (b) re-point
   `ebnf_frontend_dual_run_diff_gate.sh` at the new frontend↔meta-parser pair, retiring an
   oracle whose migration finished; (c) invert the `ci_workflow_local_gate.sh:810-823` pins
   from *"assert Perl is present"* to *"assert Perl is absent"*, so the parity gate enforces
   the intent instead of the residue. ⚠️ Sequence matters — (c) before (a)/(b) turns the
   parity gate red.
3. **Fix the stale citation** in the guard's rationale either way.

#### ⛔⛔⛔ BLAST RADIUS — Perl is a REQUIRED stage of the flagship aggregate

The gates above are not operator-only curiosities. Measured in `sota_exit_gate.sh`:

- `:1110` — `run_check "ebnf_frontend_dual_run_gate" "required" "strict Perl-vs-Rust EBNF dual-run differential"`
- `:33` — `POLICY_REQUIRED_CHECKS="… ebnf_stimuli_quality_gate …"` (the stage carrying the
  hard-coded Perl call at `:593`)

⇒ **`perl tools/ebnf_to_json.pl` executes inside `sota_exit_gate`, in TWO required stages.**
The 4 h 39 m run recorded as *"green end-to-end for the first time ever"*
(`CI-PARITY-GATE-ROT.7`) ran Perl. A component the director describes as ditched — *"the
Perl EBNF parser has flaws I couldn't fix"* — is a **required dependency of the
release-grade proof**.

#### ⚖️ DIRECTOR QUESTION — *"the gate is useless then, so shall be ditched too?"*

**Half of it. Not the whole gate — and the distinction is load-bearing.**

| arm | verdict |
|---|---|
| Perl ↔ Rust-frontend **rule-name diff** | ⛔ **Ditch.** Its purpose (de-risking the Perl→Rust migration) expired when the migration finished; it is blind to 25 of 276 rules on `regex.ebnf`; and the gate is configured to *pass* that blindness. |
| generated `ebnf.rs` **`parse_full` per grammar** | ✅ **Keep — it is the ONLY self-hosting instrument in the repo.** It produces the 12/12 number, and it is what caught `.10.5`'s false green. Delete the gate wholesale and nothing would notice `ebnf.ebnf` drifting away from the language again — which is precisely the six-gap rot the lockstep doctrine was written for. |

⇒ **rename + re-point, don't delete.** Drop the Perl arm, retire the misleading
*"dual run"* name (the duality it describes has ended), and upgrade the surviving arm from
*verdict-only* to the raw-AST differential in step 1 — which turns a confusing vestige into
the instrument the replacement endgame actually needs.

⚠️ **Sequencing constraint:** both gates are `required` stages of `sota_exit_gate`, so a
rename or removal must land in the same wave as the aggregate's stage list, or the flagship
breaks.

⭐ **The director's premise is right and the cost is already measurable.** *"It is confusing
and shall avoid confusion at all cost"* — in this session alone the name caused **me** to
write a false statement into a Knowledge-Map card (*"a frozen legacy reference"*, refuted by
simply running it), and it left the tracked record describing a Perl↔Rust duality that no
longer exists. A name that misdescribes what a gate compares is not cosmetic debt.

#### Acceptance Checklist (enforced) — PART 1, the de-Perl

- [x] **REPRODUCE / ISSUE** — `git grep -n ebnf_to_json.pl -- 'rust/scripts/*.sh'` showed the
  Perl EBNF frontend executing in **three** tracked gate scripts, and
  `sota_exit_gate.sh:1110` / `:33` ran two of them as **required** stages. `perl
  tools/ebnf_to_json.pl` on `grammars/regex.ebnf` returned **251** rules against the Rust
  frontend's **276** — blind to 25, which the gate passed as `perl_under_reports`.
- [x] **ROOT CAUSE (WHY + WHERE)** — three independent causes, each located:
  (1) `ebnf_stimuli_quality_gate.sh:593` was **hard-coded** to `"$EBNF_TO_JSON"` and
  bypassed the `run_frontend_to_json()` helper 10 lines above it, so it ran Perl regardless
  of `FRONTEND_IMPL` (already defaulting to `rust`) — and the *reason* it needed a second
  frontend at all was `cargo build --features generated_parsers` **without**
  `ebnf_dual_run`, leaving the binary unable to read `.ebnf`;
  (2) `ebnf_frontend_dual_run_diff_gate.sh:110,167` kept Perl as a migration oracle whose
  migration had completed;
  (3) `ci_workflow_local_gate.sh:810-823` **`assert_file_contains`**'d the Perl invocations,
  so removing them FAILED that gate — a guard pinning the thing it was built to remove,
  justified by a `README.md` citation that `README-POLICY.1` had already deleted.
- [x] **FIX** — ops/build-flow tier. Build `ast_pipeline` with `--features "generated_parsers
  ebnf_dual_run"` up front (the cross-check is cfg-gated on `has_generated_ebnf_parser`, so
  this compiles on a cold clone with no `generated/ebnf.rs` — the breaker the Makefile's own
  `regex_parser_bootstrap` already relies on), then read `.ebnf` directly. Perl arm deleted;
  `PGEN_EBNF_FRONTEND_IMPL` now **rejects** `perl` loudly rather than silently defaulting;
  pins inverted to `assert_file_not_contains_uncommented`.
- [x] **ADDRESSED (verified)** — measured before → after:

  | measurement | before | after |
  |---|---|---|
  | uncommented `ebnf_to_json.pl` / `EBNF_TO_JSON` in the 3 gate scripts | 3 scripts, 6 sites | **0 / 0 in all three** ✅ |
  | `perl` invocations in `ebnf-frontend-dual-run-diff.yml` / `sota-exit-gate.yml` | 1 each (`Verify Perl runtime` steps) | **0 / 0** ✅ |
  | `require_tool perl` in `ebnf_stimuli_quality_gate.sh` | present (+ a `perl -e` percentage one-liner) | **removed**; the one-liner is now `awk` ✅ |
  | `make ebnf_stimuli_quality_gate` | passed **via Perl** | **passed, no Perl** — 5/5 grammars ✅ |
  | `make ebnf_frontend_readiness` | passed | **passed, no Perl** — 3/3 grammars ✅ |
  | `make ebnf_frontend_dual_run_gate` | passed, tolerating `perl_under_reports` | **runs with no Perl**; `ebnf` pass, `json` pass, **`regex` FAILS honestly** at 35.55% consumed ⬅ the `.10.5` gap, and exactly what `.10.2` option B closes |
  | `ci_workflow_local_gate` pins | `assert_file_contains` Perl **present** | `assert_file_not_contains_uncommented` — Perl **absent** ✅ |

- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` **ALL 15 doctrines PASS**
  (incl. `GATE-REACHABILITY` and `FLOW-INTEGRITY`, the two that govern gate wiring);
  `bash -n` clean on all four edited scripts; two gates re-run green end-to-end with Perl
  gone; **no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` touched ⇒ all 11 generated
  parsers byte-identical BY CONSTRUCTION**. ⚠️ `ebnf_frontend_dual_run_gate`'s `regex` row is
  RED — that is `.10.5`'s already-recorded true-red, not a regression introduced here, and
  `.10.2` option B (measured, exit 0) closes it.
- [x] **LOCKSTEP** — `rust/Makefile` help text (two targets no longer described as
  "Perl-vs-Rust"), `sota_exit_gate.sh` stage descriptions ×2, the stale Makefile bootstrap
  comment (it claimed the seed goes "via the Perl frontend"; the recipe has used the Rust
  frontend for a long time), `CHANGES.md`, `MEMORY.md`.

#### ⚠️ Deliberately NOT done in part 1 — needs a director call

`tools/*.pl` (5 files) and `perl/` (34 tracked files) still EXIST. All *usage* is gone, but
deleting 39 tracked files is a separate, hard-to-reverse decision and is not implied by
"stop using Perl" — git history would retain them either way. ⇒ **raised, not assumed.**


⭐ **Same family as `.10.5`, twice over**: `.10.5` found a gate green *because of* a defect;
here a gate promises a comparison it never makes, **and** a guard enforces the survival of
a component the project's own docs describe as gone. All three were believed — by the
record, by the director, and by me — until measured.

#### ✅ PART 2 — the envelope differential, and what it found on its first run

`pgen::ebnf_envelope_differential` (new module, 10 unit tests) projects arm 2's typed AST into
arm 1's `raw_ast` token vocabulary and diffs them token by token. Both arms run **in one
process** (`ebnf_dual_run_diff --envelope-differential`), so the two sides can never be
compared across stale artifacts.

⭐⭐⭐ **THE HEADLINE — the gap is SMALL, and six grammars are already exactly equivalent.**
The leaf predicted *"expect RED on day one"* and *"today nobody knows how big it is"*. It is
now measured, at **34 014 compared token positions across 14 grammars** (1 933 divergences in total, of which 1 399 are the one include-asymmetry grammar):

| result | grammars |
|---|---|
| ✅ **ENVELOPE-EQUIVALENT — 0 divergences** | `builtin_return_annotation` (159 tok), `builtin_semantic_annotation` (167), **`ebnf` (913)**, `rtl_const_expr` (242), `rtl_frontend` (1355), `vhdl` (1693) |
| 96–99 % agreement, divergences located | `json` 97.06 %, `semantic_annotation` 99.50 %, `return_annotation` 96.89 %, `regex` 98.86 %, `systemverilog` 98.92 %, `systemverilog_lrm_profiled_generated` 96.47 %, `systemverilog_preprocessor` 96.26 % |
| not comparable — arms describe different rule sets | `systemverilog_lrm_profiled_wrapper` (arm 1 RESOLVES `include(…)` and splices 1398 rules; arm 2 stops at the directive) |

⭐ **`grammars/ebnf.ebnf` itself is envelope-equivalent over all 913 tokens.** The meta-grammar
is self-hosting at OUTPUT level, not merely at verdict level. That is a materially stronger
statement than the `12/12` this gate used to make, and nothing in the repo could make it before.

#### ⛔⛔ TWO LIVE `ebnf.ebnf` DEFECTS, found by the instrument on its first run

Both are invisible to every existing instrument, because both parse **`Ok`**. Routed, not
worked ([[feedback_flow_findings_are_routed_not_worked]]) — neither blocks a gate, and both are
`grammars/*.ebnf` changes whose blast radius is the whole seed chain. → **`.10.13`**, **`.10.14`**.

#### Acceptance Checklist (enforced) — PART 2, the envelope differential

- [x] **REPRODUCE / ISSUE** — the gate compared a *verdict*, never an *output*. Measured with
  the two arms side by side on `grammars/json.ebnf`: arm 1
  (`ast_pipeline --emit-raw-ast-json`) emits a flat token envelope
  `[["rule","json"],["rule_reference","value"],…]`; arm 2 (`ebnf_dual_run_diff --emit-ast-json`,
  new flag) emits `{type:"grammar_file", elements:[…]}`. **Nothing in the repo compared them**,
  so `12/12` said the meta-parser *reads* every grammar and nothing said it reads them the
  *same way*.
- [x] **ROOT CAUSE (WHY + WHERE)** — for each of the two defects the instrument found, with a
  minimal repro:

  **(1) `.10.13` — `regex_flags` matches ACROSS trivia and eats the next token's leading
  characters.** `grammars/ebnf.ebnf:254-260`:
  `regex_pattern := "/" regex_content "/" regex_flags?` with `regex_flags := /([gimsuyx]*)/`.
  Minimal repro `a := /x/ <ident> /y/`, run through the generated meta-parser — the verdict is
  `Ok` in every row:

  | source identifier | leading char ∈ `[gimsuyx]` | meta-parser's `non_terminal` name | chars eaten |
  |---|---|---|---|
  | `zebra` | no | `zebra` | 0 ✅ |
  | `alpha` | no | `alpha` | 0 ✅ |
  | `gamma` | **yes** | `amma` | 1 ⛔ |
  | `members` | **yes** | `embers` | 1 ⛔ |
  | `xylophone` | **yes, twice** | `lophone` | **2** ⛔ |

  `xylophone` losing exactly two characters is the decisive evidence: it is the greedy
  `[gimsuyx]*` run, consumed after trivia-skipping put the cursor on the identifier. ⚠️ A
  SECOND defect sits on the same two lines — the return annotation is
  `{type: "regex", pattern: $2, flags: $3}`, but `$3` is the closing `"/"` literal and the flags
  are `$4`. Census confirms it: **all 186 `regex` nodes across the tracked grammars carry
  `flags: "/"`.**

  **(2) `.10.14` — a leading `@annotation` binds to the PREVIOUS rule.** Minimal repro, verdict
  `Ok`:

  ```
  first := "a"

  @transform: some_fn()
  second := "b"
  ```

  Arm 1 attaches `@transform` to `second`. Arm 2 gives `second.annotations == []` and puts the
  annotation inside **`first`'s expression** as a trailing sequence element. WHERE:
  `grammars/ebnf.ebnf:139` `sequence := sequence_element+` with `:143-147`
  `sequence_element := (inline_semantic_annotation | quantified_element | primary_element)` and
  `:155` `inline_semantic_annotation := semantic_annotation` — the greedy `+` with the inline
  annotation as its FIRST alternative lets a rule's expression run past the blank line and
  swallow the next rule's annotation, so `grammar_rule := annotation_list? rule_definition`
  never gets it. Measured blast radius: **47 rebound annotations** across the tracked grammars,
  including `@transform` on `positive_integer`/`float`/`integer` in `return_annotation.ebnf` and
  `@branch_policy: priority_first` in the SystemVerilog profile grammars.
- [x] **FIX** — instrument tier (no grammar or engine change; `grammars/*` and `generated/*`
  untouched). New `pgen::ebnf_envelope_differential`: an explicit, auditable projection from
  arm 2's node types onto arm 1's 14 envelope kinds, plus a differ that classifies every token
  position as match / payload-not-comparable / payload-divergence / kind-divergence. Two
  normalizations are declared rather than assumed — arm 2's quoted literals are decoded
  **through arm 1's own `decode_quoted_literal_body`** (one implementation of the escape rules,
  not two), and a semantic annotation's opaque payload is compared modulo its `{ … }` delimiter
  and the whitespace immediately inside it. Everything unmapped is **counted and named**, and
  defeats equivalence rather than being dropped.
- [x] **GROUND TRUTH — the instrument refuses rather than guesses**
  ([[feedback_instrument_needs_ground_truth]]). Both controls run against the LIVE arms before
  any report is emitted, and a miss aborts:
  - **positive** — a synthetic grammar exercising every projected construct must project to an
    exactly-identical envelope. Measured **27 token positions, 0 divergences**. A miss means the
    PROJECTION is broken and every number is meaningless.
  - **negative** — one token of arm 1's envelope is mutated; the differ must report **exactly
    one** divergence at **exactly** that rule/token index. Measured: **1**. A clean verdict
    there means the DIFFER is blind and its parity claims are worthless.
- [x] **ADDRESSED (verified)** — measured before → after:

  | measurement | before | after |
  |---|---|---|
  | what the gate compares between the arms | **verdict only** (`Ok`/`Err`) | **34 014 token positions** across 14 grammars ✅ |
  | grammars covered by the gate | 3 (`ebnf`, `json`, `regex`) | **14** — every tracked grammar arm 1 accepts ✅ |
  | grammars provably ENVELOPE-EQUIVALENT | **unknown — nothing measured it** | **6**, incl. `ebnf.ebnf` itself at 913/913 ✅ |
  | live `ebnf.ebnf` defects the flow could see | 0 | **2**, each with a minimal repro + file:line ✅ |
  | `make ebnf_frontend_dual_run_gate` (strict) | passed on 3 grammars, verdict only | **passed on 14 with the envelope ratchet** ✅ |
  | instrument ground-truth controls | none | positive **27/27**, negative **1/1**, both refusing on a miss ✅ |
- [x] **NO REGRESSION** — `cargo test --features ebnf_dual_run --lib ebnf_envelope_differential`
  **10/10 pass** (including two regression pins for projection bugs found and fixed during
  construction — see below); `rustfmt --check` clean and `cargo clippy` clean on both new/edited
  Rust files; `bash -n` clean on the gate script; `PGEN_EBNF_DUAL_RUN_STRICT=1` gate **green
  end-to-end on all 14 grammars**; **no `grammars/*.ebnf`, no `generated/*`, no engine source
  touched ⇒ all 11 generated parsers byte-identical BY CONSTRUCTION**.
- [x] **THE RATCHET IS TWO-SIDED, AND I PROVED IT FIRES** — `.10.9` had to replace a floor that
  could never fire again, so this one was tested in **both** directions before it was trusted,
  not merely written:

  | experiment | result |
  |---|---|
  | ceiling lowered below the measured count (simulated regression) | `overall=fail`, *"envelope divergences REGRESSED: 38 > ceiling 10"*, **strict-mode exit 1** ✅ |
  | ceiling raised above the measured count (unlocked-in gain) | `overall=fail`, *"envelope divergences IMPROVED to 38 (ceiling 99) — lower the ceiling"*, **exit 1** ✅ |
  | ceilings at the measured values | gate **green** ✅ |

  ⇒ a ceiling cannot be satisfied vacuously, and a gain cannot be left unlocked. A grammar with
  no declared ceiling fails too (`*) echo -1`).
- [x] **LOCKSTEP** — `rust/Makefile` help text for both targets (no longer "across ebnf/json/regex"),
  the gate script's header block, `docs/book/src/gate-flow.md`, `TOOLBOX.md`, `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `MEMORY.md`.

#### ⚠️ TWO PROJECTION BUGS I FOUND IN MY OWN INSTRUMENT — reported, not quietly fixed

The first sweep reported ~100 more divergences than were real. Both causes were **mine**, and
both are now pinned by a regression test so they cannot come back:

1. **Synthesized `|` separators.** I emitted the alternation separator *between* alternatives.
   Arm 2 stores a per-branch return annotation (`a := x -> $1 | y -> $2`) at the **head of the
   next alternative, ahead of the `|`** — so every such rule was one token out of step.
   Measured effect of the fix: `vhdl` **37 divergences → 0** (and it is now envelope-equivalent),
   `rtl_frontend` **31 → 0**, `systemverilog` 92.27 % → 98.63 %. The separator is now projected
   where arm 2 actually carries it.
2. **Braced annotation payloads.** Arm 1 keeps the `{ … }`; arm 2 hands back the interior. Raw
   comparison reported a representational difference as a divergence — `regex` **32 payload
   divergences → 0**, `systemverilog` 41 → 1.

⭐ **The lesson is the leaf's own**: an instrument's first RED is as likely to be the instrument
as the subject. The two ground-truth controls are what make that distinguishable — and the
positive control is exactly the thing that would have caught bug 1 had it existed first.

#### ⚠️ HONEST BOUNDS, stated rather than implied

- **Return-annotation payloads are compared by KIND only.** Arm 1 carries the raw source text;
  arm 2 carries it already parsed. Recovering the text would need a pretty-printer — a new
  trusted surface whose own bugs would surface as false divergences. The report counts these
  under `payload_not_comparable` (never folded silently into either column), and
  `agreement_ratio` counts them as agreement **on the kind, which is all that was checked**.
- **Arm 2 is known-blind to `[> … ]` lexical annotations** — it reads all 12 in
  `systemverilog_preprocessor.ebnf` as character classes. Pre-existing and already documented in
  `ebnf_frontend.rs`; the instrument now *measures* it under `unmapped_arm2_constructs`.
- **Arm 1's inline / rule-level annotation distinction is not represented by arm 2** (12
  occurrences, `systemverilog` only), so those report as kind divergences.
- **`systemverilog_lrm_profiled_wrapper`'s 1.69 % is not a defect rate.** Arm 1 resolves the
  include and describes 1401 rules; arm 2 describes 3. The report says so on its own via
  `unresolved_include_directives`.
- **Not re-run end-to-end**: `sota_exit_gate` runs this gate as a `required` stage. The stage
  itself was run strict-green here, but the 4 h 39 m aggregate was not re-run.

#### ⚠️ ROUTED, not worked — a published tracker cell still quotes the deleted Perl telemetry

`LIVE_ACHIEVEMENT_STATUS.md`'s `regex` row states *"The latest fresh validated family summary
records … dual-run overall `pass` with `raw_ast_status=perl_under_reports`,
`perl_rule_count=104`, `rust_rule_count=194`, `raw_ast_missing_on_perl_count=90`"*. Those four
keys were **deleted** by `.10.9`; no gate emits them, and `rust_rule_count` for `regex` is now
**276**, not 194. The sentence reads as current and is not.

⚠️ **Deliberately NOT rewritten here.** The cell sits inside a block explicitly prefixed
*"Historical record of the landed proof follows"*, and `.10.7`/`.10.10` both established the
standing rule that **history surfaces are not rewritten** — a dated true-at-the-time statement
stands. Whether *this* cell is history (it is prefixed as such) or a live claim (it is phrased
as *"the latest fresh validated summary"*) is a **judgement about the published surface, not an
engineering call**, so it is surfaced rather than assumed. ⇒ **director call.** If it should be
corrected, the numbers to replace it with are now measurable in one command
(`make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_gate`), which was not true before this
leaf.

⭐ Note the connection: `.10.9` recorded that it *"deliberately left the regex contract with only
a one-sided rule-count ratchet"* because the second arm did not exist. **It exists now** — the
envelope differential is that arm, and `regex` measures 98.86 % over 3 338 token positions. Whether
to wire it into `regex_parser_family_contract_gate` is a separate, priced decision and is NOT
assumed here.


### `.10.13` — `regex_flags` matches across trivia and EATS the next token's leading characters (`todo`)

- **Status: `todo`** — opened 2026-07-31 by `.10.6` part 2, which found it on the envelope
  differential's first run. ⛔ **Every affected parse returns `Ok`**, so no verdict-level
  instrument can see it; it took an output-level differential.
- **WHY + WHERE**: `grammars/ebnf.ebnf:254-260`. `regex_flags := /([gimsuyx]*)/` is reached
  after trivia-skipping, so following a regex terminal it consumes the leading `[gimsuyx]` run
  of the next identifier. Full repro table in `.10.6` part 2's checklist — `xylophone` →
  `lophone` (two characters) is the decisive row.
- **Second defect, same two lines**: `-> {type: "regex", pattern: $2, flags: $3}` — `$3` is the
  closing `"/"` literal; the flags are `$4`. Census: **all 186 `regex` nodes** across the
  tracked grammars carry `flags: "/"`, so no grammar has ever had its regex flags read
  correctly by the meta-parser.
- **Measured blast radius**: 8 divergences the differential can currently see
  (`json` ×2, `semantic_annotation` ×6) plus an unknown share of the divergences that sit behind
  an earlier kind-divergence in the larger grammars.
- ⛔ **Consequence if `ebnf.ebnf` replaced the hand-written frontend today**: rule references
  would silently resolve to names that do not exist (`members` → `embers`), and regex flags
  would be `"/"` everywhere.
- **When taken up**: the fix is a `grammars/ebnf.ebnf` change, so it regenerates
  `generated/ebnf.rs` and the whole seed chain — that is why it is a leaf of its own and not an
  aside in `.10.6`. Lower `json` and `semantic_annotation` in
  `envelope_divergence_ceiling()` as it lands; the gate FAILS if a gain is left unlocked.

### `.10.14` — a leading `@annotation` binds to the PREVIOUS rule, not the one it precedes (`todo`)

- **Status: `todo`** — opened 2026-07-31 by `.10.6` part 2. ⛔ Also parses `Ok`.
- **WHY + WHERE**: `grammars/ebnf.ebnf:139` `sequence := sequence_element+`, `:143-147`
  `sequence_element := (inline_semantic_annotation | quantified_element | primary_element)`,
  `:155` `inline_semantic_annotation := semantic_annotation`. The greedy `+` with the inline
  annotation as its FIRST alternative lets a rule's expression run past the blank line and
  swallow the following rule's leading `@annotation`, so
  `grammar_rule := annotation_list? rule_definition` never receives it. Minimal repro in
  `.10.6` part 2's checklist.
- **Measured blast radius**: **47 rebound annotations** across the tracked grammars — including
  `@transform` on `positive_integer` / `float` / `integer` in `grammars/return_annotation.ebnf`
  and `@branch_policy: priority_first` in the SystemVerilog profile grammars. Annotation COUNTS
  match in both arms (verified per grammar), so nothing is lost — everything is **misrouted**,
  which is worse, because a lost annotation is a loud failure and a misrouted one is silent.
- ⛔ **Consequence if `ebnf.ebnf` replaced the hand-written frontend today**: every semantic
  annotation written on its own line above a rule would steer the WRONG rule.
- **When taken up**: needs a boundary the expression cannot cross — the same class of fix as an
  anchored rule terminator. ⚠️ Prove the repair against the differential, not against a parse
  verdict: the defect is `Ok` on both sides today.


### `.10.15` — ⛔ `.10.3` REMOVED THE EMISSION AND LEFT `.10.4`'s ASSERTION BEHIND — a RED lib test, 146 commits old (`todo`, found 2026-08-11 by `ENGINE-UNIVERSAL-SERVICES.8`'s confirmatory sweep)

**SYMPTOM.** `cargo test --features "generated_parsers ebnf_dual_run" --lib -- --skip deep_nesting`
→ **1074 passed / 1 failed**:

```
---- ast_pipeline::ast_based_generator::semantic_usage_tests::unresolved_reference_codegen_emits_semantic_fallback_and_stubs_boolean_names ----
panicked at src/ast_pipeline/ast_based_generator.rs:14841:
expected semantic_annotation fallback to detect '@' directives
```

**ROOT CAUSE (WHY + WHERE), and the two leaves are siblings in THIS tree.** `.10.3` (`cc0cbffe`,
`PGEN-LANG-CAPABILITY-AUDIT-0023`, 2026-07-31) deliberately retired `"semantic_annotation"` from
`NATIVE_UNRESOLVED_REFERENCE_BUILTINS`, deleting the arm that emitted
`if start_pos >= self.input.len() || self.input.as_bytes()[start_pos] != b'@'` — correctly, since
`semantic_annotation` is a real rule in a real grammar, not a primitive. `.10.4`'s test
(last touched at `779dca06`, **82+ commits EARLIER**) still asserts
`rendered.contains("starts_with") || rendered.contains("b'@'")`. `.10.3` did not update it.

⛔ **Unsatisfiable ever since, not intermittently**: `git log cc0cbffe..HEAD -S "b'@'"` and
`-S "starts_with"` over `rust/src/ast_pipeline/ast_based_generator.rs` both return **nothing**, so no
commit re-added either token. `git rev-list --count cc0cbffe..HEAD` = **146**.

⭐⭐ **WHY NOBODY NOTICED — and it is the same finding as the nesting ceiling.** The lib suite is
effectively never run to completion: measured at **>2h 27m** before being abandoned, with
`/usr/bin/sample` (twice, hours apart, identical stacks) showing the only remaining threads were
`parser_embedding_systemverilog_deep_nesting_…` and `parser_embedding_vhdl_deep_nesting_…` on a
2000-deep paren nest. Every other test had finished. ⇒ a red test can sit in the tracked suite for
146 commits because the suite's wall time is set by two stress tests unrelated to it. That is the
developer-flow cost named in `ENGINE-UNIVERSAL-SERVICES.3`'s bounded-refusal gap entry, now with a
dated casualty. **The two findings should be read together.**

**VERDICT — the CODE is right, the TEST is stale.** `.10.3`'s whole point is that an undefined
reference to `semantic_annotation` must emit the bare never-matching `Backtrack` stub like any other,
so the linter's undefined-reference error is the single diagnostic. The fix is to flip that one
assertion to the stub form the SAME test already applies to `parse_true` a few lines below — i.e.
make both halves assert the same post-`.10.3` truth.

⚠️ **Do not "fix" it by re-adding the `@` matcher**: that would undo `.10.3` and re-introduce a
builtin the tree deliberately retired.

⛔ **PARKED behind the SV lane lock** — routed, not worked: it is a codegen/test defect in another
family, not an SV-release blocker. ⭐ Named re-open trigger: the lane lock lifting, OR the next
session that runs the lib suite and needs a green baseline.

**Acceptance:** the assertion is corrected to the stub form (not the emission restored), and the test
passes.

✔ **And it is the ONLY one — that is a census, not a floor.** `cargo test` runs every test and
reports the complete tally rather than stopping at the first failure, so `1074 passed; 1 failed;
29 ignored; 4 filtered out` covers the whole suite (the 4 filtered are the `--skip deep_nesting`
pair × their two families). Nothing else is red behind it.

### `.10.7` — delete the retired Perl tree (`done` — ✅ SLICE 1 + ✅ SLICE 2)

- **Status: `in-progress`, carrying explicit director approval (2026-07-31): *"you have my
  approval, go, go, go"*.** ⛔ **That approval was given on a number I got wrong, so the
  scope is restated here before anyone acts on it.** `.10.6` removed all Perl *usage*; this
  leaf removes the *files*.
- ✅ **SLICE 1 DONE** (`PGEN-LANG-CAPABILITY-AUDIT-0024`, session #229) — the leaf's own
  step 1, *"re-point or retire the executing referrers (they are the only breakage)"*.
  **Measured after: zero tracked files execute an in-scope Perl artifact**; every remaining
  non-doc mention is a comment. Slice 2 is the `git rm` + the published-doc sweep.

#### ✅ Slice 1 — the four executing referrers, each adjudicated on measurement

The charter named three; there are **four**. Each was measured before deciding, and the two
verdicts differ for a stated reason.

| referrer | what it ran | verdict |
|---|---|---|
| `tests/bootstrap_tests/run_bootstrap_tests.sh` | `tools/ebnf_to_json.pl` | ⛔ **RETIRED** — see the three convictions below |
| `tests/bootstrap_tests/run_simple_tests.sh` | `tools/ebnf_to_json.pl` | ⛔ **RETIRED** — same family |
| `testing/automated_test_framework.py` | `transform_ast.pl` + `ebnf_to_json.pl` (the latter shared by **all six** language arms) | ✅ **RE-POINTED** to the Rust frontend |
| `examples/workflow_examples.py` | `perl tools/transform_ast.pl` | ✅ **Perl example removed**, rest kept |

⭐ **Why two of them were retired rather than re-pointed — the Rust frontend IS a drop-in
replacement, so this was a choice, not a limitation.** Measured on the fixtures themselves:
`ast_pipeline <fixture>.ebnf --emit-raw-ast-json out.json` then `--generate-parser` both
return `rc=0`. Re-pointing was rejected because measurement convicted the harness three ways
— it would have preserved an instrument that lies:

1. ⛔⛔ **IT REPORTED SUCCESS WHILE ITS OWN OUTPUT SAID EVERY TEST FAILED.** Verbatim from
   the run: `Total tests: 4`, `Passed: 0`, three `❌` lines — then
   `🎉 ALL TESTS BEHAVED AS EXPECTED!` and **exit 0**. WHERE: the verdict is
   `exit $UNEXPECTED_RESULTS` (`run_bootstrap_tests.sh:194`), gated on that one counter
   (`:171`), while the `❌ UNCLEAR` path increments **nothing** and `❌ FAILED` increments
   `FAILED_TESTS` (`:47`/`:55`) — a counter the verdict never reads.
2. ⛔ **It only ever reached 4 of the 15 fixtures.** `set -e` (`:6`) plus `run_test`'s
   `return 1` aborts each category at its first failing case — measured per category:
   1 of 5, 1 of 4, 1 of 3, 1 of 3.
3. ⛔ **It wrote generated artifacts INTO the tracked fixture directories** — `bash -x`
   caught `-o …/tests/bootstrap_tests/return_annotation/success/….json`, and a single run
   left **16** `.json`/`_parser.rs`/`.log` files next to the `.ebnf` fixtures, against the
   repository's artifact-locality policy. Its sibling `run_simple_tests.sh` had been fixed to
   use `$ARTIFACT_ROOT` under `rust/target/`; this one never was.

⇒ a repaired-but-still-lying instrument is worse than no instrument
([[feedback_instrument_needs_ground_truth]]). Nothing tracked invoked either script — no
gate, no `make` target, no workflow (measured: the only referrers are prose) — which is the
only reason a harness in that state never misled anyone.

**The 15 `.ebnf` fixtures were KEPT.** They are grammar files, unaffected by any of the
above, and the behaviour they describe is still worth gating; `tests/`(71) is explicitly a
separate director call, so deleting a corpus by side effect would exceed this approval. They
carry a new `tests/bootstrap_tests/README.md` recording exactly why they have no runner and
what a correct one must fix, and the decision is routed to **`.10.8`** rather than settled
here.

⚠️ **A THIRD pre-existing defect found while working slice 1, and deliberately not fixed
here**: `examples/workflow_examples.py` **is not valid Python and never was in this state** —
from line 171 its whole body is one line of literal `\n`-escaped text
(`python3 -m py_compile` on `git show HEAD:` fails identically, so this is not my edit). The
Perl-ectomy landed cleanly on it (0 Perl mentions, 18,166 → 14,796 bytes) and the file is
exactly as broken as it was. Routed to `.10.8`.

#### Acceptance Checklist (enforced) — SLICE 1, the executing referrers

- [x] **REPRODUCE / ISSUE** — four tracked files execute a Perl artifact this leaf deletes.
  `git grep -nE "tools/(ebnf_to_json|transform_ast|…)\.pl"` over the non-doc surface named
  exactly four call sites: `tests/bootstrap_tests/run_bootstrap_tests.sh:44`,
  `run_simple_tests.sh:45`, `testing/automated_test_framework.py:105/431/440/504`,
  `examples/workflow_examples.py`. Deleting the tree without them leaves four files calling a
  file that is not there.
- [x] **ROOT CAUSE (WHY + WHERE)** — two separable causes, both located. **(i)** The two
  bootstrap harnesses are built entirely around the retired frontend AND are broken
  independently of it: `bash -x tests/bootstrap_tests/run_bootstrap_tests.sh` traces
  `+ tools/ebnf_to_json.pl … -o …/tests/bootstrap_tests/return_annotation/success/….json`
  (artifacts into the tracked fixture tree), the run prints `Total tests: 4 / Passed: 0` with
  three `❌` lines and still `exit 0`, and WHERE is `:194 exit $UNEXPECTED_RESULTS` gated at
  `:171` — a verdict that reads one of three counters, with `❌ UNCLEAR` incrementing none and
  `❌ FAILED` incrementing `FAILED_TESTS` (`:47`/`:55`) which the verdict never consults; the
  4-of-15 truncation is `set -e` at `:6` meeting `run_test`'s `return 1`. **(ii)** For the two
  Python harnesses the Perl step is *incidental* — in
  `testing/automated_test_framework.py` `ebnf_to_json.pl` is the shared EBNF→JSON step for
  **all six** language arms, so it had to be re-pointed, not removed. `git ls-files` +
  `git grep -l` over every tracked file proves **no gate, `make` target or workflow invokes
  any of the four** — the only referrers are prose.
- [x] **FIX** — declarative/ops tier. Retire the two harnesses (`git rm`); re-point the shared
  frontend step to `ast_pipeline --emit-raw-ast-json` (an explicit output path, where Perl
  wrote the envelope to stdout); delete `_execute_perl` and the `'perl'` entries from the
  executor map, the `--language` choices and both default language lists; remove
  `example_3_perl_integration` and its summary bullet. Keep the 15 fixtures + a `README.md`
  that records why they have no runner.
- [x] **ADDRESSED (verified)** — measured before → after on the census itself: tracked files
  executing an in-scope Perl artifact **4 → 0**; every remaining non-doc mention is a comment
  (`git grep` over the whole tree returns 7 hits, all comment lines, two of which slice 2
  owes). `python3 -m py_compile testing/automated_test_framework.py` **OK**. The replacement
  frontend step is proven on the real fixtures — `ast_pipeline
  tests/bootstrap_tests/return_annotation/success/single_scalar.ebnf --emit-raw-ast-json`
  `rc=0`, then `--generate-parser` `rc=0`. `git ls-files tests/bootstrap_tests` **17 → 16**
  (2 runners out, 1 README in), all **15** `.ebnf` fixtures intact.
- [x] **NO REGRESSION** — nothing in `rust/src/*`, no grammar, no `generated/*` in the diff ⇒
  **all 11 generated parsers byte-identical by construction**; confirmed by
  `git diff --cached --name-only` showing zero paths under `rust/src/`, `grammars/` or
  `generated/`. `scripts/check_doctrines.sh` **ALL 15 PASS** against the real staged diff.
  `make -C rust mdbook_docs_gate` GREEN. The 16 stray artifacts my own diagnostic run left in
  the fixture tree were removed and `git status` is clean — verified with `git ls-files` plus a
  `find` sweep for `*.json`/`*.log`/`*_parser.rs` under `tests/bootstrap_tests`.
- [x] **LOCKSTEP** — new `tests/bootstrap_tests/README.md` (the corpus's own record), new leaf
  `.10.8` (routing both dead harnesses), `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`,
  `docs/TASK_TREE.md`. No book/contract movement — none of the four files is a published
  surface, and `PGEN_USER_GUIDE.md`'s Perl command is slice 2's.

#### ✅ Slice 2 — the deletion, and the four published surfaces that were still lying

**40 tracked files deleted**, exactly the approved scope: `tools/*.pl` (5),
`tools/generators/perl_parser_gen` (1), `perl/` (34). ⛔ **Plus 6 UNTRACKED leftovers** the
charter did not know about — `.gitignore`'s `*_parser.pm` and `perl/*parser.rs` patterns had
been hiding generated Perl parser outputs inside `perl/`, so a `git rm` alone would have left
the directory standing with 178 KB of orphans in it. They are **not in git history**, so their
names, sizes and sha256s are recorded before removal in
`rust/target/lang_cap_audit_10_7/perl_untracked_manifest.txt` and here:

| file | bytes | sha256 (16) |
|---|---|---|
| `perl/Ultimate_return_annotation_parser.pm` | 15,100 | `65e4dfbe0de1d3f4` |
| `perl/generated_parser.rs` | 7,200 | `273380feefbd8696` |
| `perl/Parser/Ultimate_return_annotation_parser.pm` | 41,486 | `2b860c2a2315f9f5` |
| `perl/Parser/Development/working_return_annotation_parser.pm` | 5,346 | `79304a473a45d379` |
| `perl/Parser/Development/new_return_annotation_parser.pm` | 65,182 | `01a12fd6e6359ff1` |
| `perl/Parser/Development/return_annotation_parser.pm` | 44,256 | `793ad00d642ec648` |

Each was confirmed unreferenced by any tracked file first (the single `grep` hit for
`generated_parser.rs` is `CHANGES.md` prose about a *log filename string*, not the file).

⭐⭐ **THE SWEEP FOUND FOUR PUBLISHED SURFACES MAKING FALSE PRESENT-TENSE CLAIMS — and the
worst was a downstream CONTRACT.** `.10.6` retired the frontend but corrected almost none of
what documents it:

| surface | the claim, verbatim | why it mattered |
|---|---|---|
| `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md:2455` | *"The Perl-based fallback `tools/ebnf_to_json.pl` **is retained** … Don't reach for it unless you have no Rust toolchain."* | ⛔ a **published promise to RGX** of a fallback that no longer exists — and one whose "known feature-coverage limitations" were, measured, far worse than the hedge implied (blind to 25 of `regex.ebnf`'s 276 rules) |
| `PGEN_USER_GUIDE.md:119` | the Perl command was the **first** EBNF→JSON recipe, Rust shown as *"alternative"* | the user-facing quick path was the deleted one |
| `PGEN_USER_GUIDE.md:2394` | *"`PGEN_EBNF_FRONTEND_IMPL=perl` **remains the default**."* | ⛔ **inverted** — measured, the default is `rust` and `=perl` now fails loudly on purpose (`ebnf_frontend_readiness_gate.sh:20/36`, `ebnf_stimuli_quality_gate.sh:21/41`) |
| `docs/knowledge/ebnf-self-hosting-what-it-means.md:66` | listed the Perl↔frontend pair as one of three **live** comparisons | a retrieval-indexed knowledge card teaching a retired mechanism |

All four corrected, plus the two dangling code comments
(`rust/scripts/ast_dump_contract_gate.sh:127`, which told a reader to *run* the deleted file,
restated against the Rust frontend; `rust/src/ebnf_frontend.rs:273`, whose
`perl/AST/Transform.pm:3234` provenance citation now says it resolves only in git history),
three more user-guide gate descriptions, `docs/regex_parser_book/src/quickstart.md`, and the
vestigial `.gitignore` pattern (`perl/*parser.rs`, measured 0 matches; `*_parser.pm` **kept**,
it still matches 2 files under `tests/artifacts/`).

⚠️ **Deliberately NOT swept**: `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md:1620-1621`
carries `perl tools/ebnf_to_json.pl --validate-only …` inside a dated **`Progress
(2026-03-18)`** entry. That is a historical record of what was run then, and history is not
rewritten here.

⭐ **This is where `.10.9` was found** — sweeping for dangling references surfaced four gate
scripts still consuming Perl telemetry the producer no longer emits. See that leaf; it is
`.10.6`'s residue, it is separable from this deletion, and it is RED right now.

#### Acceptance Checklist (enforced) — SLICE 2, the deletion

- [x] **REPRODUCE / ISSUE** — `git ls-files` counted the approved scope at **40** tracked
  files (`tools/*.pl` 5 + `tools/generators/perl_parser_gen` 1 + `perl/` 34), against the
  charter's *"≈35"*; and a `find` over `perl/` after `git rm` showed the directory still
  standing with **6 untracked files** the tracked census could not see.
- [x] **ROOT CAUSE (WHY + WHERE)** — the files implement a frontend `.10.6` retired: WHERE,
  `git grep` over every tracked non-doc file returns **zero** executing references after
  slice 1, and the only guards naming them are `.10.6`'s INVERTED pins
  (`ci_workflow_local_gate.sh:810/824/825 assert_file_not_contains_uncommented`), which the
  deletion makes *more* true, not less. The 6 survivors are hidden by `.gitignore:232-233`
  (`perl/*parser.rs`, `*_parser.pm`) — generated outputs of the very generator being deleted.
- [x] **FIX** — `git rm` the 40 tracked files; `rm -rf perl` for the 6 gitignored generated
  leftovers after recording their manifest; sweep the four false published claims, the two
  dangling code comments and the dead ignore pattern.
- [x] **ADDRESSED (verified)** — before → after: tracked Perl-tree files **40 → 0**;
  `perl/` **exists → absent** (`ls -d perl` → *No such file or directory*);
  `git grep` for the deleted paths over every tracked file except the history surfaces
  (`CHANGES.md`/`DEVELOPMENT_NOTES.md`/`LIVE_ACHIEVEMENT_STATUS.md`/`docs/tasks/`/`MEMORY.md`/
  the built `*-html/`) returns **only past-tense retirement notes and one dated 2026-03-18
  progress entry** — no instruction to run a deleted file survives.
- [x] **NO REGRESSION** — `make -C rust ebnf_frontend_dual_run_gate` **GREEN with the Perl
  tree gone**: `ebnf`/`json`/`regex` all `pass`, self-hosting `consumed_pct`
  100.00/99.90/100.00, `✅ EBNF frontend dual-run differential gate passed`, `EXIT=0`.
  `make -C rust ebnf_stimuli_quality_gate` **`✅ … passed`, `EXIT=0`**. This slice touches
  one `rust/src` file (a comment in `ebnf_frontend.rs`), so it was proven **codegen-inert by
  the `.10.3` path-pinned oracle**: regenerating the meta-parser afterwards yields
  `7ce6578f799c36c79343f98c1e441feb70b453eee25be860ae64824f751938e5` for
  `generated/ebnf.rs` — **the identical hash `.10.3` pinned**, so no generated artifact moved.
  `bash -n rust/scripts/ast_dump_contract_gate.sh` OK. `scripts/check_doctrines.sh` **ALL 15
  PASS** against the real staged diff. `mdbook_docs_gate` + `ebnf_parser_book_gate` GREEN.
- [x] **LOCKSTEP** — ⭐ the **downstream contract**
  `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` (the retained-fallback promise),
  `PGEN_USER_GUIDE.md` (6 corrections incl. the inverted default),
  `docs/knowledge/ebnf-self-hosting-what-it-means.md`,
  `docs/regex_parser_book/src/quickstart.md`, `.gitignore`, `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `LIVE_ACHIEVEMENT_STATUS.md`, `MEMORY.md`, `docs/TASK_TREE.md`.
  No schema/ledger/release movement — nothing generated moved.

#### ⛔ CORRECTION — I said "39 files". It is 142, and most of them are not what "go" meant

| area | tracked `.pl`/`.pm` | verdict |
|---|---|---|
| `tools/` | **5** (`ebnf_to_json.pl`, `generate_parser.pl`, `transform_ast.pl`, `ebnf_generator.pl`, `ebnf_input_generator.pl`) + `tools/generators/perl_parser_gen` | ✅ **IN SCOPE** — this *is* the Perl EBNF frontend |
| `perl/` | **30** (`AST/Transform.pm`, `LinkedSpec.pm`, the codegen `.pm` family, debug scripts) | ✅ **IN SCOPE** — the library the frontend loaded |
| `fx/` | **34 `.pl`/`.pm` — but 267 TRACKED FILES in total** | ⛔ **OUT OF SCOPE HERE, and a finding in its own right** — see below |
| `tests/` | **71** (`run_tests.pl`, generators, `experimental/`) | ⚠️ **ADJACENT, separate call** — Perl, and measured to be invoked by **nothing** tracked in `rust/`, `scripts/` or `.github/`; but it is a test corpus, not the frontend |
| `legacy/` | **2** | ⚠️ adjacent; explicitly legacy |

⇒ **the approved deletion is `tools/` + `perl/` ≈ 35 files.** The rest are separate
decisions that were never put to the director, and must not ride along on a "go".

#### ⭐ `fx/` — the director's instinct is right, their premise is not (measured 2026-07-31)

Director: *"`fx/` wasn't git tracked and I do not think there is anything in PGEN that uses
or references `fx/`."*

| claim | measured |
|---|---|
| *"wasn't git tracked"* | ⛔ **FALSE — `git ls-files fx/` returns 267 TRACKED files**, and `git check-ignore` reports it is **not** ignored |
| *"nothing in PGEN uses or references it"* | ✅ **TRUE** — zero references from `rust/`, `scripts/` or `.github/`; the only tracked mentions are prose in `LIVE_ACHIEVEMENT_STATUS.md`, the SOTA roadmap and `tests/TEST_GUIDE.md` |
| how it got here | `git log --diff-filter=A` ⇒ **the initial commit `b579dc8a`** — swept in when the repository was created, never deliberately added |

⇒ **`fx/` is 267 tracked files of dead weight that nothing in PGEN consumes.** That makes
the case for removing it stronger than for anything else in this leaf — but it is a
*different* subsystem (`fx/bin`, `fx/cgi`, `fx/plugin`, `fx/ruby`, `fx/specs`, Verilog and
FSM material), it is 267 files rather than the 34 I first reported, and the approval on
record was given for *"the Perl EBNF parser"*. ⇒ **put it to the director as its own
one-line decision with these numbers**, then execute. Nothing is lost either way: it is in
git from the first commit onward.

#### ⚠️ 43 tracked files reference these paths — deletion is not `git rm` alone

Non-doc referrers that would **dangle**, measured:
`tests/bootstrap_tests/run_bootstrap_tests.sh`, `tests/bootstrap_tests/run_simple_tests.sh`,
`testing/automated_test_framework.py`, `examples/workflow_examples.py`, `go/ast_pipeline.go`,
`julia/ast_pipeline.jl`, `zig/src/ast_pipeline.zig` + `zig/src/main.zig`,
`rust/src/bin/pgen_ast.rs`, `rust/src/ebnf_frontend.rs`, `rust/Makefile`, four
`rust/scripts/*.sh` (comments only — `.10.6` already cleared their executable lines),
`PGEN_USER_GUIDE.md`, `tests/*.md`, `test/grammars/*.json`, plus the `fx/`/`legacy/` cluster.

Most are **comments or doc prose** and are free to update. The ones that would genuinely
**break** are the two `tests/bootstrap_tests/*.sh` harnesses and
`testing/automated_test_framework.py`, which *execute* `tools/ebnf_to_json.pl` — measured to
be invoked by no tracked gate, so they are dead harnesses, but they must be retired or
re-pointed in the same wave rather than left calling a deleted file.

#### Order of work

1. ✅ **DONE (slice 1)** — re-point or retire the executing referrers (they are the only
   breakage). ⛔ **There were FOUR, not three**: the charter missed
   `examples/workflow_examples.py`.
2. `git rm` `tools/*.pl`, `tools/generators/perl_parser_gen`, and `perl/`. ⛔ **Measured
   count: 40 files, not the charter's "≈35"** — `tools/*.pl` (5) +
   `tools/generators/perl_parser_gen` (1) + `perl/` (**34** tracked files, of which 30 are
   `.pl`/`.pm`; the rest are `test.ebnf`, `test_input.txt`,
   `test_grouped_quantifiers.ebnf`, `json_parser_clean.rs`). The charter's 30 counted only
   the Perl sources; the deletion is the whole directory.
3. Sweep the doc/comment referrers (`PGEN_USER_GUIDE.md` still documents
   `tools/ebnf_to_json.pl --verbosity debug …` as a user-facing command — that one is a
   published surface and must be corrected, not just deleted around), plus the two comments
   named under *"What slice 2 still owes"*.
4. Re-run `check_doctrines.sh`, `ebnf_frontend_dual_run_gate`, `ebnf_stimuli_quality_gate`,
   `mdbook_docs_gate`.
5. ⭐ Nothing is lost: git history retains every deleted file.

---

### `.10.10` — delete `fx/`: 267 tracked files nothing in PGEN consumes (`done`)

- **Status: `done`** (`PGEN-LANG-CAPABILITY-AUDIT-0026`, session #229, 2026-07-31).
  **✅ DIRECTOR-APPROVED on the corrected numbers** — the call was escalated by `.10.7`
  precisely because the approval on record covered *"the Perl EBNF parser"*, and `fx/` is a
  different subsystem an order of magnitude larger. Director, 2026-07-31, shown the measured
  figures: *"Go for [it]"*.
- **Why it was a separate call and not a ride-along on `.10.7`**: the director's premise was
  *"`fx/` wasn't git tracked"* — measured **false** (267 tracked files, not ignored, present
  since the initial commit `b579dc8a`, 2025-08-30). Executing a "go" given on a wrong premise
  would have been the failure mode, not the caution.

#### Measured before deleting — and the `perl/` lesson was applied

`.10.7` slice 2 learned that a tracked-file census is not a directory census: `git rm` left
`perl/` standing with 6 gitignored generated files inside it. `fx/` was checked the same way
**before** the deletion, not after:

| measure | `fx/` |
|---|---|
| tracked | **267** |
| files on disk | **267** |
| untracked | **0** |
| gitignored | **0** ⇒ no hidden leftovers, unlike `perl/` |
| bytes | **1,509,317** |
| executable / config references from anywhere outside `fx/` | **0** |

⇒ `git rm -r fx` is a complete deletion here, verified by `find` afterwards rather than
assumed.

#### The reference sweep — 8 files mention it, and only ONE was live

`git grep` over every tracked file outside `fx/`:

- **0** hits in any executable or config surface — no `rust/`, no `scripts/`, no `.github/`,
  no `Makefile`, no `Cargo.toml`. This is what makes it deletable at all.
- **5 history surfaces** (`CHANGES.md`, `DEVELOPMENT_NOTES.md`, `LIVE_ACHIEVEMENT_STATUS.md`,
  `MEMORY.md`, `docs/tasks/`) — untouched; history is not rewritten.
- **2 dated progress entries** — `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md:4152` (`2026-02-19`)
  and `:4205` (`2026-03-19`), and `REPO-HYGIENE.md:74` (a past hygiene sweep, which also names
  the already-deleted `perl/AST/`). True statements about their dates ⇒ left standing, the
  same rule `.10.7` applied to the roadmap's Perl commands.
- ⚠️ **1 genuinely live doc**: `tests/TEST_GUIDE.md`'s *"Directory Structure"* block. ⭐ It was
  **already wrong before this leaf, in a more interesting way**: it is rooted at **`afx/cursor/`**
  — a path that does not exist in this repository at all — and it bills `fx/perl/LinkedSpec.pm`
  as the *"Core parser generator"*, which PGEN has not been for as long as the Rust stack has
  existed. It documents a **pre-PGEN project layout**, not this one. Corrected rather than
  merely de-`fx/`'d.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `git ls-files fx/` → **267** tracked files, `git check-ignore`
  → not ignored, `git log --diff-filter=A` → added by the **initial commit** `b579dc8a`
  (2025-08-30), i.e. swept in at repository creation rather than deliberately added; and
  `git grep` over every tracked file outside `fx/` finds **0** executable/config references.
  1.5 MB of a different subsystem (`fx/bin`, `fx/cgi`, `fx/plugin`, `fx/ruby`, `fx/specs`,
  Verilog + FSM material) that nothing in PGEN consumes.
- [x] **ROOT CAUSE (WHY + WHERE)** — not a defect but dead weight, and the WHERE is the point:
  `git log --diff-filter=A -- fx/` names the **initial commit** as its only addition, so no
  later change ever adopted it. Confirmed by the reference census above — every surviving
  mention is prose, and `git ls-files --others [--ignored]` over `fx/` returns **0/0**, so
  unlike `perl/` there is no gitignored residue to strand.
- [x] **FIX** — `git rm -r fx` (267 files), plus the one live doc correction.
- [x] **ADDRESSED (verified)** — before → after: tracked files **267 → 0**; `find fx` →
  *No such file or directory*; `git grep` for `fx/` outside the history surfaces returns only
  the two dated progress entries deliberately kept. Repo tracked-file count and working tree
  shrink by 1,509,317 bytes.
- [x] **NO REGRESSION** — nothing in `rust/src/`, `grammars/` or `generated/` is in the diff
  (`git diff --cached --name-only` → 0 such paths) ⇒ **all 11 generated parsers byte-identical
  by construction**; `fx/` is in no build graph (`git grep` → 0 hits in `Makefile`/`Cargo.toml`/
  `.github/`), so no gate consumes it. `scripts/check_doctrines.sh` **ALL 15 PASS** against the
  real staged diff; `mdbook_docs_gate` GREEN.
- [x] **LOCKSTEP** — `tests/TEST_GUIDE.md` (the stale `afx/cursor/` structure block),
  `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `LIVE_ACHIEVEMENT_STATUS.md`, `MEMORY.md`,
  `docs/TASK_TREE.md`. No book/contract movement — `fx/` appears in neither.
- ⭐ **Nothing is lost**: all 267 files remain in git history from `b579dc8a` onward.

---

### `.10.12` — the `DOCPATH` doctrine only sees absolute paths that contain `/pgen/` (`todo`)

- **Status: `todo`** — opened 2026-07-31 session #229 by `.10.10`, which hit 13 checkout-specific
  absolute paths in a file it was already editing and asked why an enforced doctrine had not
  caught them.
- ⭐ **The doctrine is doubly blind, and the PATTERN half is the interesting one.**
  `scripts/check_diagnostics_and_docpaths.sh:51-53`:

  ```bash
  absolute="$(git grep -nIE '/Users/[^ )`]*/pgen/' -- \
    'docs/book/src/**' 'docs/contracts/**' 'PGEN_USER_GUIDE.md' 'README.md' \
    'docs/tasks/**' 'docs/decisions/**' 'KNOWLEDGE_MAP.md' 'docs/knowledge/**' 'LIVE_ACHIEVEMENT_STATUS.md' …
  ```

  1. **PATTERN** — the regex requires the absolute path to contain **`/pgen/`**. An absolute
     home path pointing anywhere *else* is invisible. ⇒ the doctrine catches
     "your own checkout" and misses every other leaked local path.
  2. **SCOPE** — the pathspec omits `tests/`, `tools/`, `rust/`, `grammars/`, `scripts/` and
     the vendor corpora.

- ⭐⭐ **THE DECIDING PROOF — an IN-SCOPE file with violations the pattern cannot see.**
  `docs/contracts/**` *is* in the pathspec, and
  `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` carries **3** absolute paths
  (`:1766`, `:1820`, `:1893` — `/Users/richarddje/Documents/github/rgx/pgen-issues/PGEN-RGX-00NN.yaml`).
  Pattern-visible: **0**. The doctrine **passes**. ⇒ this is not a scope oversight that a wider
  pathspec would fix; the pattern itself under-specifies the rule it enforces. It is also a
  *published downstream contract* leaking a local home directory.

#### The size of the class, classified honestly — this is NOT a 260-file sweep

`git grep -lIE '/Users/'` over tracked files (excluding built `*-html/`):

| bucket | files | what it is | verdict |
|---|---|---|---|
| vendor LRM extraction (`docs/systemverilog/**`, `docs/verilog/**`, `docs/vhdl/**`) | **199** | one provenance line each naming the **source PDF** (`/Users/…/SystemVerilog-LRM-IEEE-1800-2017.pdf`) | ⚠️ **external-input provenance, not a repo-internal path** — CLAUDE.md §12 governs repo-internal paths. Adjudicate deliberately; do NOT bulk-rewrite |
| append-only history (`CHANGES.md`, `DEVELOPMENT_NOTES.md`, `docs/tasks/**`) | ~43 | records of past runs | ⇒ out of scope by the same rule that keeps history intact |
| **everything else** | **18** | incl. the published contract above, `KNOWLEDGE_MAP.md`, 3 `docs/decisions/`, `grammars/systemverilog*.ebnf`, `rust/src/bin/*.rs`, `rust/scripts/ci_workflow_local_gate.sh`, `tools/` | ⇒ **the real work item** |

⚠️ ⭐ The enforcer **itself** is in that last bucket
(`scripts/check_diagnostics_and_docpaths.sh` — its own doc comment quotes the pattern), which
is legitimate but means a naive fix will flag its own source. Handle that deliberately rather
than by an exclusion added in a hurry.

- ⚠️ **ROUTED, not worked** ([[feedback_flow_findings_are_routed_not_worked]]): no gate is
  blocked and no verdict is untrustworthy — the paths are cosmetic/portability leaks, not
  behaviour. But the doctrine currently **certifies** a property it does not check, which is
  the shape this repo treats most seriously. ⭐ Measured to reproduce **outside** the file that
  surfaced it (the published contract, 3 hits, in-scope, invisible) ⇒ it is a pattern defect,
  not a `tests/` defect.
- **Scope when taken up:** widen the pattern to any `/Users/<user>/` or `/home/<user>/`
  repo-internal path, decide the vendor-provenance question explicitly (a documented carve-out
  is fine; silence is not), extend the pathspec, then fix the 18. ⛔ Do not widen the pattern
  without first adjudicating the 199 — a doctrine that turns red on 199 legitimate provenance
  lines gets an exclusion bolted on and stops meaning anything.
- ✅ **Already fixed in passing by `.10.10`:** `tests/TEST_GUIDE.md`'s **13** occurrences of
  `/Users/richarddje/Downloads/AFX/fsm/afx/cursor/…` — a checkout-specific path into a
  directory that is not even this repository — now `0`.

---

### `.10.11` — the Perl INTERPRETER is a wanted dependency that 7 gate scripts never declared (`done` — director-ruled; 2 fixed, 5 routed)

- **Status: `done`** (`PGEN-LANG-CAPABILITY-AUDIT-0027`, session #229, 2026-07-31). Opened by
  the director's question *"why was Perl still used for those parts that are now failing
  loudly?"*, then **settled by their ruling the same session**.
- ⭐⭐ **DIRECTOR RULING, verbatim (2026-07-31): *"About the perl interpreter being called, I
  think we can leave it there. I was referring to removing references, uses to and of the Perl
  EBNF frontend, not the shell script calling the perl interpreter. So we can reinstate, if
  that's still possible use of the Perl interpreter. I just do not want the Perl EBNF
  frontend."*** ⇒ the scope of `.10.6`/`.10.7` was always **the EBNF frontend**, never Perl the
  language. The interpreter **stays**, and the fix is to **DECLARE** it, not to port away from
  it. Option (b) of this leaf's original scope (porting `-MJSON::PP` → `jq`, `-MTime::HiRes` →
  `date`, arithmetic → `awk`) is **REJECTED by ruling**, not deferred.

#### ⛔ TWO CORRECTIONS TO MY OWN OPENING NUMBERS — both were wrong, and measurably

This leaf was opened on a count I did not verify carefully enough. Corrected before acting:

| my claim | measured | why I was wrong |
|---|---|---|
| *"17 gate scripts execute Perl"* | **16** | `ebnf_stimuli_quality_gate.sh`'s only hit is a **comment** (`# … was a perl -e one-liner; awk is …`). Excluding comment lines is the whole difference. |
| *"~48 sites"* | **47** | same cause |
| *"`.10.6` deleted `require_tool perl` ⇒ the flow now needs Perl without declaring it"* | ⛔ **the attribution is FALSE** | `.10.6` removed that line from **exactly one** script — `ebnf_stimuli_quality_gate.sh` — and in the *same* commit ported that script's only `perl -e` to `awk`. **So that removal was correct**: the script genuinely stopped needing Perl. `git log -S"require_tool perl"` over `sota_exit_gate.sh` and `hdl_frontend_readiness_gate.sh` returns **empty** ⇒ they **never** declared it. The gap is **PRE-EXISTING**, not `.10.6`'s doing. |

⇒ I reported a regression where there was a long-standing hole. The hole is real; the blame was not.

#### The measured state, and what "declared" splits into

16 scripts, 47 real (non-comment) execution sites — `perl -e` arithmetic and file sizes,
`-MJSON::PP` JSON validation, `-MTime::HiRes` millisecond clocks, `-0777` slurp-mode text
processing, `-0ne` NUL-separated `git ls-files` filtering:

| class | scripts | verdict |
|---|---|---|
| declares `require_tool perl` | **9** | ✅ already correct |
| **has a preflight, but skips perl** | **2** — `sota_exit_gate.sh`, `hdl_frontend_readiness_gate.sh` | ⛔ the sharpest case: the mechanism is right there and the tool is not in it ⇒ **FIXED HERE** |
| **has no preflight at all** | **5** — `branch_protection_contract_gate.sh`, `fixed_point_bootstrap_gate.sh`, `sv_declared_shadow_promotion_gate.sh`, `sv_parse_full_ratio_promotion_gate.sh`, `vhdl_strict_promotion_gate.sh` | ⚠️ **ROUTED** — see below |

⭐ `sota_exit_gate.sh` is the one that matters most: it runs `perl -MJSON::PP` inside the
**required** `differential_baseline_contract` stage (`:989`) and already required `jq` two lines
away. A host without Perl failed deep inside that stage rather than at the preflight, by name.

#### ⚠️ Why the remaining 5 are ROUTED and not fixed here

They have no `require_tool()` at all, so declaring the dependency means **introducing the
function** — and `git grep -c "^require_tool() {"` finds **60 existing copies** of the identical
6-line body across `rust/scripts/`. Adding 5 more is a direct, knowing contribution to the
duplicated-shell-helper problem that **`CI-PARITY-GATE-ROT.10`** already owns and explicitly
prices as *"introducing a shared-library convention across 91 scripts, which must be decided on
its own, not smuggled in behind a blocker fix"*. ⇒ the 5 belong with that decision. Cross-linked
there; ⛔ do not resolve them by pasting a 61st copy.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `git grep -lE "(^|[^a-zA-Z_])perl[[:space:]]+-"` over
  `rust/scripts/*.sh` + `scripts/*.sh`, with comment lines excluded, gives **16 scripts / 47
  execution sites**; cross-tabulating against `grep -cE "^[[:space:]]*require_tool[[:space:]]+perl"`
  shows **7 of 16 never declare the dependency**, including the flagship aggregate.
- [x] **ROOT CAUSE (WHY + WHERE)** — not a regression: `git log --oneline -S"require_tool perl" --
  rust/scripts/sota_exit_gate.sh rust/scripts/hdl_frontend_readiness_gate.sh` returns **no
  commits**, so neither ever declared it. WHERE the mechanism exists but is incomplete:
  `sota_exit_gate.sh:843` and `hdl_frontend_readiness_gate.sh:96` each call `require_tool jq`
  and stop, while `sota_exit_gate.sh:989` executes `perl -MJSON::PP` in a required stage.
- [x] **FIX** — declarative, 2 lines of code plus the reason: `require_tool perl` added at both
  preflight sites, each annotated with the director's frontend-vs-interpreter distinction so the
  next de-Perl sweep does not delete it again.
- [x] **ADDRESSED (verified)** — scripts that execute Perl without declaring it: **7 → 5**, and
  both fixed scripts are the ones that had the mechanism already. The preflight is proven to
  work in **both directions** on the real function body (`feedback_instrument_needs_ground_truth`):
  positive control `require_tool perl` → passes silently on this host (`/usr/bin/perl`, 5.34.1);
  negative control `require_tool definitely_not_a_tool` → `error: required tool
  'definitely_not_a_tool' is not available in PATH` + `exit 1`. `bash -n` clean on both files.
- [x] **NO REGRESSION** — shell-only, no `rust/src/`, no grammar, no `generated/*` in the diff ⇒
  **all 11 generated parsers byte-identical by construction**. `bash -n` on both edited scripts.
  `scripts/check_doctrines.sh` **ALL 15 PASS** against the real staged diff — including
  `FLOW-INTEGRITY`, which is the doctrine that owns gate-flow invariants.
- [x] **LOCKSTEP** — this leaf (incl. the two self-corrections), the `CI-PARITY-GATE-ROT.10`
  cross-link, `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `LIVE_ACHIEVEMENT_STATUS.md`, `MEMORY.md`,
  `docs/TASK_TREE.md`. No book/contract movement — no published surface documents the
  interpreter dependency either way, which is itself the point of declaring it in the preflight.

---

### `.10.9` — ⛔⛔ `.10.6` LEFT FIVE GATES READING PERL TELEMETRY THAT NO LONGER EXISTS — the regex family contract gate is RED, and it is a required stage of the flagship aggregate (`done`)

- **Status: `done` (2026-07-31, session #229f).** ⛔ The opening headline said FOUR gates and
  three breakages; both were low — see the two corrections below. Opened 2026-07-31 session #229 by
  `.10.7` slice 2, which found it while sweeping for dangling references. ⚠️ **This is
  `.10.6`'s residue, not `.10.7`'s**: the readers broke the moment the Perl ARM was removed,
  one commit before any file was deleted. Deleting the files does not make it worse, which is
  why it is separable — but it must not wait.
- ⛔ **`make -C rust regex_parser_family_contract_gate` cannot pass**, and
  `rust/scripts/sota_exit_gate.sh` invokes it ⇒ **the flagship aggregate is RED**. `.7`
  reported `sota_exit_gate` green end-to-end on 2026-07-30 (32/32 stages, 4 h 39 m); `.10.6`
  landed immediately after, so that green is the last one this configuration can have
  produced.

#### WHY + WHERE — measured from the live artifact, three separate breakages in one gate

The dual-run gate's regex entry carries exactly these keys today (read from
`rust/target/ebnf_frontend_dual_run_gate/summary.json`):

```
['artifacts','consumed_pct','grammar','input_bytes','notes','overall',
 'raw_ast_status','rust_parse','rust_parse_end','rust_parse_full','rust_report','rust_rule_count']
```

⇒ **no `perl_*` key exists at all**, and `raw_ast_status` is now `exported`. Against that,
`rust/scripts/regex_parser_family_contract_gate.sh`:

| # | site | what it does | outcome now |
|---|---|---|---|
| 1 | `:217` | `assert_equal "dual-run regex perl_ebnf_to_json" "pass" "$…"` | the key is absent ⇒ `jq -r` yields `null` ⇒ **FAILS** (`"pass"` ≠ `"null"`) |
| 2 | `:222-228` | `case "$raw_ast_status" in parity\|perl_under_reports)` … `*) exit 1` | actual is **`exported`** ⇒ **`exit 1`**, message: *"raw_ast_status must be 'parity' or 'perl_under_reports' but found 'exported'"* |
| 3 | `:230` | `if (( dual_run_regex_rust_rule_count < dual_run_regex_perl_rule_count ))` | `jq` yields the bare word `null`, which bash arithmetic reads as an **unset name = 0** ⇒ `276 < 0` is false ⇒ **passes VACUOUSLY**. Confirmed directly: `bash -c 'set -euo pipefail; a=276; b=""; if (( a < b )); then echo LT; else echo GE; fi'` → `GE`, `rc=0` |

⭐ **Breakages 1 and 2 are loud; 3 is the dangerous one.** It is a *numeric floor* — "the Rust
frontend must not report fewer rules than Perl did" — that now compares against a constant
zero and can never fire again. Fixing 1 and 2 without noticing 3 would restore a green gate
with a dead assertion inside it, which is the `.10.5` shape exactly.

#### The same telemetry is read in three more places

`git grep` over `rust/scripts/`, all still consuming keys the producer no longer emits:

- `rust/scripts/sota_exit_gate.sh:3048/3050` → publishes `regex_family_dual_run_perl_rule_count`
  and `…_raw_ast_missing_on_perl_count` in its summary (`:3903/:3905`), pre-seeded `<missing>`
  at `:3001/:3003` — so the aggregate reports a value for a measurement that no longer exists.
- `rust/scripts/regex_parser_family_status_gate.sh:185`
- `rust/scripts/regex_combined_telemetry_contract_gate.sh:232/234`

#### Scope when taken up

Decide what the regex contract should assert now that there is only one frontend arm — that
is a **design** question and it belongs with `.10.6` part 2 (the raw-AST differential), which
is the thing that would give the contract a real second arm to compare against. Until then the
honest minimum is: delete the three Perl assertions rather than weaken them, replace the
`raw_ast_status` allowlist with the values the producer actually emits, and **state in the gate
that it has lost its cross-frontend floor** rather than letting `(( … < null ))` imply one is
still there. ⛔ Do not simply add `exported` to the `case` allowlist and move on — that
converts a broken gate into a quiet one.

⭐ **ROUTING EVIDENCE — does it reproduce outside the regex family?** Measured: **yes, and it
is not regex-specific.** The producer dropped the keys for *all three* grammars
(`ebnf`/`json`/`regex` — the summary artifact shows the same key set for each), and three of
the four consumers are regex-family scripts only because the regex family is the only one with
a dual-run contract surface. The defect class is *"a consumer outlived its producer's schema"*,
i.e. the `CI-PARITY-GATE-ROT.11` stale-metric family, arrived at from the opposite direction —
there the reader scraped a prose log, here the reader queries a structured key that was
deleted. Cross-linked accordingly.

#### ✅ WHAT LANDED — and two corrections to this leaf's own opening numbers

⛔ **CORRECTION 1 — it is FOUR loud breakages in that gate, not three.** The opening table
missed `:221`, `assert_equal "dual-run regex raw_ast_missing_on_rust_count" "0" "$…"`. That
key is absent from the producer exactly like the other three, so it too compares `"0"` against
`"null"` and fails. It was invisible in the reproduction because `:217` fails first and the
gate exits — a reminder that "the first error a gate prints" is a lower bound on its breakage,
never the census.

⛔ **CORRECTION 2 — it is FIVE consuming scripts, not four.** The opening census listed
`regex_parser_family_contract_gate.sh`, `sota_exit_gate.sh`,
`regex_parser_family_status_gate.sh` and `regex_combined_telemetry_contract_gate.sh`. A
whole-tree `git grep` also finds **`regex_parser_family_status_contract_gate.sh`**, which pins
the criterion and metric NAMES as a schema (`expected_criteria` / `expected_metrics`) — so it
is a consumer of the vocabulary even though it never reads a Perl value. Renaming a criterion
without it would have left that gate asserting a key nothing emits.

**The root-cause fix, not just the four call sites.** The reason one missing value produced
three *different* outcomes is that `jq -r '… | .absent_key'` prints the bare word `null` and
the caller cannot distinguish "measured null" from "no longer emitted". Extraction now goes
through `dual_run_entry_value`, which aborts on an absent key, a null value, or anything but
exactly one matching entry — naming the key. That is what makes the *next* schema change loud
in every consumer rather than loud in some and silent in others.

**What replaced each dead assertion** (deleted, never defaulted):

| was | now |
|---|---|
| `perl_ebnf_to_json == "pass"` | *deleted* — no Perl stage exists to report on |
| `raw_ast_missing_on_rust_count == "0"` | *deleted* — a count relative to a frontend that is gone |
| `raw_ast_status ∈ {parity, perl_under_reports}` | `assert_equal "exported"` — an **exact** match on the producer's only success value (`skip` = arm-1 export FAILED), not the old allowlist with `exported` appended |
| `(( rust_rule_count < perl_rule_count ))` | `require_int` + `assert_int_ge … "$RULE_COUNT_FLOOR"` (pinned 276, env-overridable) |

⭐ **The floor is labelled for what it is.** It is a one-sided regression **ratchet**, not a
cross-implementation check, and the gate publishes `dual_run_regex_cross_frontend_floor:
retired` into its own summary — carried through the family-status gate and the flagship
aggregate — so no downstream reader can infer a second frontend from the presence of a numeric
floor. A constant right-hand side also cannot go null, which is precisely how its predecessor
died. Restoring a real second arm remains `.10.6` part 2.

**Criterion rename, priced across all five scripts:** `dual_run_raw_ast_missing_on_rust_zero`
→ `dual_run_raw_ast_exported`. The closure-criteria count is unchanged at **11**, so the
family-status arithmetic and the status-contract schema pin stay coherent.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the gate at the parent commit, run against the live artifact:
  `rc=1`, `error: dual-run regex perl_ebnf_to_json mismatch: expected 'pass' but found 'null'`.
  Pinned as arm A of `run_dual_run_telemetry_probes.sh` (`PARENT_REV=1362b375`), so it keeps
  reproducing after this fix commits.
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow signature (family 5). `bash -n` clean on
  all five scripts. The mechanism, measured in bash itself:
  `bash -c 'set -euo pipefail; a=276; b=""; if (( a < b )); then echo LT; else echo GE; fi'`
  → **`GE`, rc=0** — `(( ))` reads the absent operand as an unset name, i.e. **0**, so the
  floor at `regex_parser_family_contract_gate.sh:230` compared `276 < 0` and passed vacuously.
  WHERE: the producer `rust/scripts/ebnf_frontend_dual_run_diff_gate.sh` emits exactly
  `artifacts,consumed_pct,grammar,input_bytes,notes,overall,raw_ast_status,rust_parse,rust_parse_end,rust_parse_full,rust_report,rust_rule_count`
  for every one of `ebnf`/`json`/`regex`; all four `perl_*`/`raw_ast_missing_on_*` keys read
  `null`.
- [x] **FIX** — declarative tier: no `rust/src/`, no grammar, no `generated/*`. Extraction
  helper + deletion of the four dead consumers + an exact-match status assert + a pinned
  ratchet, propagated across five scripts.
- [x] **ADDRESSED (verified)** — **REJECT → PASS** on the named re-runnable oracle
  `regex_parser_family_contract_gate.sh`: `rc=1` → `rc=0`, `✅ Regex parser-family contract
  gate passed`, publishing `dual_run_regex_rust_rule_count: 276`,
  `dual_run_regex_rust_rule_count_floor: 276`, `dual_run_regex_cross_frontend_floor: retired`,
  `dual_run_regex_raw_ast_status: exported`.
- [x] **NO REGRESSION** — the downstream chain re-run on the new schema, each exit 0:
  `regex_formal_exhaustive_closure_gate` (broader corpus **44/44**),
  `regex_parser_family_status_gate` (`regex_status: In Progress`, closure criteria
  **8/11 satisfied, 3 unsatisfied** — unchanged, total still **11**,
  `regex_dual_run_raw_ast_exported: true`), `regex_parser_family_status_contract_gate`
  (schema pins accepted). ⭐ The two gates that require a 4 h 39 m `sota_exit_gate` run are
  covered by a **mechanical producer→consumer census** instead of a claim: **226 reads across
  6 script→summary edges, 0 unresolved**, and the census carries a ground-truth control (an
  injected unresolvable read that it MUST report — it does). Guard controls, all firing:
  key deleted → `rc=5` naming the key; key null → `rc=5`; `raw_ast_status=skip` → `rc=1`;
  rule count 275 → `rc=1 expected >= 276` (⭐ the direct counter-proof that the ratchet is
  **not** vacuous, which is exactly what the old floor could no longer do); POSITIVE control
  untouched → `rc=0`. Driver exit 0, **0 divergences**.
- [x] **LOCKSTEP** — book `gate-flow.md` §7 gains failure mode **8, "a consumer that outlived
  its producer's schema"** (the count and the closing principle updated from seven→eight, and
  the README pointer with it); the roadmap's three present-tense false claims corrected
  (`:1000`, `:1001`, and `:4660`'s *"`perl_ebnf_to_json` remains reserved for the dual-run
  differential gate"*, which had been false since `.10.6`); `CHANGES.md`, `MEMORY.md`,
  `LIVE_ACHIEVEMENT_STATUS.md`, `DEVELOPMENT_NOTES.md` updated.

⚠️ **Honest bound, stated rather than implied.** `regex_combined_telemetry_contract_gate` and
`sota_exit_gate` are verified by static key-coherence, **not** executed end-to-end here — a
`sota_exit_gate` run is 4 h 39 m. The census proves every key they read is a key its producer
emits; it does **not** prove their value-level parity assertions still agree. That re-proof
rides on the next full aggregate run, and is the same standing gap the open director call on a
priced `schedule:` lane addresses.

---

### `.10.8` — two dead harnesses left behind by `.10.7`: decide, don't leave them dangling (`todo`)

- **Status: `todo`** — opened 2026-07-31 session #229 by `.10.7` slice 1, which measured both
  while retiring the Perl referrers and deliberately did not widen its slice to them.
- **(a) 15 bootstrap fixtures with no runner.** `tests/bootstrap_tests/` keeps 15 `.ebnf`
  fixtures encoding accept/reject behaviour for the two annotation surfaces; their two runner
  scripts were retired (three convictions, recorded in `.10.7`). ⭐ **The Rust frontend is a
  measured drop-in replacement for the step that died**, so writing a correct runner is cheap
  — but a correct one must fix all three defects, above all that **its exit code be a function
  of every failure path it prints**, not of one counter out of three. Decide: gate them, or
  delete the corpus. ⚠️ Whichever way, `GATE-REACHABILITY` applies — a runner nothing invokes
  is the same as no runner.
- **(b) `examples/workflow_examples.py` does not parse.** From line 171 its body is one line
  of literal `\n`-escaped text; `python3 -m py_compile` fails on it at `HEAD` and has for as
  long as it has been in this state. It bills itself as *"both documentation and integration
  testing"* and is neither. Decide: repair the escaping, or delete it. Same question for
  `testing/automated_test_framework.py` (807 lines, re-pointed by `.10.7` slice 1, invoked by
  nothing tracked, and needing go/julia/zig toolchains to exercise).
- ⚠️ **ROUTED, not worked** ([[feedback_flow_findings_are_routed_not_worked]]): neither makes
  any verdict untrustworthy — nothing tracked invokes either, which is exactly why they rotted
  unnoticed. They cost credibility, not correctness. Measured to **not** reproduce outside
  these two files: the census over every tracked non-doc referrer of the Perl tree found no
  third harness in this state.
- ⭐ **The general shape, worth stating once**: all three defects — a false green, a 4-of-15
  sample, and artifacts written into a tracked fixture tree — survived because **nothing ran
  the thing**. An instrument nobody invokes does not stay neutral; it rots, and it rots in the
  direction of reporting success.

---

### `.11` — THE HORIZON REGISTER: every commitment in `project_horizon_universal_parser` is task-tree OWNED or explicitly PARKED — never merely described (`todo`, opened 2026-08-10 by director order)

- **Status: `todo` — ⛔ PARKED under the SV lane lock (director 2026-08-10).** Register seeded
  below; the leaf closes when every row has an owner or a dated parked disposition, and a check
  keeps it that way. ⛔ **Do not pick this up before SV is RELEASED**, and the same holds for
  every capability row it registers — director, verbatim: *"We should not pivot to
  `LANG-CAPABILITY-AUDIT.*` now because those features does not seem to be useful to bring the SV
  parser to release status."* ⭐ The seeded register IS the deliverable the order asked for (the
  commitments are now tracked and reachable); what remains — the structural check — is
  scheduled, not urgent. **Being greenlit is not being next**
  ([[feedback_capability_work_is_greenlit_by_standing_authorization]] §1: authorization and
  priority are separate axes).
- **THE DIRECTIVE (director, 2026-08-10, verbatim):** *"So, everything that was decided in
  `docs/decisions/project_horizon_universal_parser.md` shall be task-tree owned, tracked if we
  want to achieve whatever was decided there."* Followed immediately by the scope bound:
  *"I am not saying to implement anything there now, but they shall clearly remain on the list
  of things EBNF and the AST pipeline shall one day full support."*
  ⇒ **this leaf is TRACKING, not implementation.** Landing a capability is out of scope here.
- **⛔ THE DEFECT IT EXISTS TO FIX — measured 2026-08-10.** `.4` priced nine roadmap items and
  then closed as `done`. Six of them (**P1-3, P1-4, P2-5, P2-6, P2-7, P3-8**) carry the owner
  value *"unscheduled"* or *"`.4` seed → new leaf when scheduled*. So they exist **only as rows
  in a table inside a CLOSED leaf**. No open leaf references them, no frontier names them, and
  the resume pointer cannot reach them. A priced, greenlit capability that no tree owns is
  indistinguishable from one nobody ever thought of — which is exactly the failure mode the
  horizon record itself diagnosed (*"the audit is PRESCRIBED but has never been RUN"*), one
  level up.
- ⭐ **And it is the same disease this tree has now named three times**: a surface that exists
  on paper but not in the engine (`.1`'s 27 decorative productions), a capability shipped under
  a different name than the census looked for (`.4`'s *"a name census is not a capability
  inventory"*), and now a **commitment recorded in a decision record with no work-tracking
  edge**. All three are *the record and the reality disagreeing*; only the third is invisible
  to every instrument the repo owns, because layer C has no reachability check into layer B.

#### THE REGISTER (seed — owners are `grep`-verified, 2026-08-10)

**A. Capability axes** (horizon §3 living list + the two added at #208):

| # | axis | owner | state |
|---|---|---|---|
| A1 | context-sensitivity via the semantic store | `SCOPE-CONTEXT-PREDICATE`, `FINAL-PHASE-PREDICATE`, `STORE-AWARE-GEN`, `MEMO-STORE-SOUNDNESS`, `RULE-SPAN-VALUE-CONSTRAINT` | ✅ owned, strong |
| A2 | forward / deferred obligations | `FINAL-PHASE-PREDICATE` (`phase:final`, landed) + `REGEX-PCRE2-FIDELITY` `.4.7.c` (forward/suffix-count) | ✅ owned |
| A3 | layout / indentation sensitivity (INDENT/DEDENT, offside) | `WS-DIRECTIVE` covers `@whitespace_sensitive` only; the **offside primitive** is `.4` P2-7, gated on `LEX-ADJACENCY.2` | ⚠️ **partial — no leaf owns the offside primitive** |
| A4 | lexer/parser feedback & ambiguity (JS ASI, C++ `>>`) | tie policies shipped (`ordered`/`longest_match`/`priority_first`, pinned in `PARSE-HARNESS.6.1`); the *feedback* channel has no owner | ⚠️ **partial** |
| A5 | error recovery | `.8` fixed the `@recover: true` codegen crash and is `done`; ⛔ `.4` states explicitly *"No claim that recovery WORKS end-to-end"* | ⚠️ **crash owned, capability UNOWNED** |
| A6 | preprocessor / macro expansion | `SVPP-EXPANSION`, `SVPP-CONTRACT-BODY` | ✅ owned (SV-specific) |
| A7 | encoding / Unicode (identifier classes, normalization) | `.4` P3-8, unscheduled | ⛔ **UNOWNED** |
| A8 | lexical adjacency / no-layout boundaries | `LEX-ADJACENCY` (+ `-design`) | ✅ owned |
| A9 | steering-surface GRANULARITY (gates every other axis) | `ANNOTATION-PLACEMENT`, `INLINE-ACTIONS`, `LEXICAL-ANNOTATIONS` | ✅ owned |

**B. Priced roadmap rows** (`.4`) — the six with no owning leaf are the core of this leaf:

| row | item | owner | state |
|---|---|---|---|
| P0-1 | `include()` real + linter honours the graph | `.7` → re-opened as `.10` (`active`) | ✅ owned |
| P0-2 | `@recover: true` codegen crash | `.8` | ✅ `done` |
| P1-3 | fact retraction / instance-scoped fact lifetime | — | ⛔ **UNOWNED** |
| P1-4 | declarative case-insensitive keywords | — | ⛔ **UNOWNED** |
| P2-5 | parameterized productions | — (needs a director SURFACE call; see below) | ⛔ **UNOWNED** |
| P2-6 | cross-rule precedence ladder | — | ⛔ **UNOWNED** |
| P2-7 | offside rule + lexer modes | gated on `LEX-ADJACENCY.2`, no leaf | ⛔ **UNOWNED** |
| P3-8 | NFKC identifier normalization | — | ⛔ **UNOWNED** |
| T-9 | unreferenced-root ("orphan") report | `.2` seed | ⚠️ seed only |

**C. Structural commitments and standing bounds** (these are *invariants/bounds*, and the
register's job is to record that they are deliberately not work items — so a later reader does
not mistake "no tree" for "forgotten"):

| # | commitment | disposition |
|---|---|---|
| C1 | duality-completeness — no primitive is done at parse-only | invariant; binds every capability leaf. ⚠️ **no gate enforces it** — candidate for a doctrine check |
| C2 | zero-cost / neutrality acceptance test | [[project_capability_growth_is_zero_cost_and_neutral]]; `PARSER-NEUTRALITY` tree |
| C3 | composability IS in scope (#209) | `.7`/`.10` |
| C4 | extensibility is a deliberate NON-commitment | ⛔ intentionally untracked — needs a precise mechanism agreed FIRST |
| C5 | row 16 parse-time-mutable grammar | ⛔ declared HARD BOUND, out of scope by design |
| C6 | *"flexible" = friction removal*, 🔜 to be defined more precisely later | ⚠️ **open director definition, no owner** — the one row whose *scope* is still undefined |
| C7 | eagerness bar — expressive AWKWARDNESS is a defect class | ranking input, applied by `.4`; must bind future rows too |
| C8 | the matrix must keep GROWING (don't close at 16 rows) | `.3c` (`todo`) |

#### What closing this leaf requires

1. **An owner per ⛔ row** — a leaf id, or a dated *parked* disposition naming why. A row may
   legitimately be parked; it may not be silent.
2. **A pointer FROM the decision record TO this register**, so layer C names its layer-B
   tracker and the edge is discoverable from either end.
3. **⭐ A check, or this rots exactly like the thing it fixes.** The natural shape is a
   `DOCTRINE_ENFORCEMENT.md` §3 *structural* check: every capability row in this register
   resolves to a live tree/leaf id or carries a parked disposition — the same
   derive-and-reconcile shape `GATE-REACHABILITY` uses for gates and `LIVE-DOC-CURRENCY` uses
   for route closure. ⛔ Without it this table is prose, and prose is what failed.
4. ⚠️ **Do NOT let this leaf drift into implementation.** The director bounded it explicitly.
   Any row that becomes real work opens its own leaf.

#### Blocked-on-director (carried here so it is visible from the register, not buried)

- **P2-5 surface call.** No parametric-rule syntax has ever been decided; `ebnf.ebnf:627`'s
  `parametric_rule := rule_name "[" parameter_list "]"` is an unratified placeholder that
  silently miscompiles (`expr[In, Yield]` → `expr ( In Yield )?`) and lints clean, because
  `[ … ]` is already the optional-element form (`:288`). Capability greenlit
  ([[feedback_capability_work_is_greenlit_by_standing_authorization]]); **notation open**.
- **C6** — the precise definition of *"flexible / friction removal"*, which the director
  deferred ("we can define that more clearer later").

## Acceptance Criteria (tree)

- A measured, re-runnable expressiveness matrix — not prose.
- Every gap carries a cost model before it becomes a work item.
- The `ebnf.ebnf` unreachable set is adjudicated: wire it, or delete it, or document
  it as reserved — **not left as decorative surface** that misleads grammar authors.
- ⭐ Every commitment recorded in [[project_horizon_universal_parser]] is task-tree OWNED or
  carries an explicit parked disposition, and a check keeps that true (`.11`).

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
- `docs/tasks/artifacts/lang_capability_audit/run_primitive_pricing_probes.sh` — `.4`
  driver (declared-verdict pricing probes for rows 1/5/6/15 + the two zero-cost tests +
  the word-anchored consumer census; every probe grammar and generated parser lives in a
  `mktemp -d` the driver removes on exit)
- `docs/tasks/artifacts/lang_capability_audit/primitive_pricing_probes.txt` — `.4`
  capture (exit 0, 0 divergences, byte-identical on re-run)
- `docs/tasks/artifacts/lang_capability_audit/run_include_resolution_probes.sh` — `.7`
  driver (17 declared-verdict cases: compose, tracked-grammar repair, hard error, cycle,
  diamond, depth-3, bare spec, `include_dir`, entry-rule pinning, 5 no-include regression
  pins, and all four consumer surfaces)
- `docs/tasks/artifacts/lang_capability_audit/include_resolution_probes.txt` — `.7`
  capture (exit 0, 0 divergences)
- `docs/tasks/artifacts/lang_capability_audit/run_native_builtin_audit.sh` — `.10.1`
  driver. Three arms: **A** static (const membership, the rule-reference screen with its
  three strip rules, the generated-parser census, the two-arm lint-masking differential,
  and the emitted matcher per member), **B** shape (collision sets + conflict/duplicate
  split + entry-rule shapes of both include candidates), **C** behaviour (7
  declared-verdict parses through the scratch slot; **opt-in** via
  `PGEN_AUDIT_RUN_SLOT_ARM=1` because it overwrites the slot and rebuilds — it restores
  the fixture, its artifact, **and** the `parseability_probe` binary on every exit path).
- `docs/tasks/artifacts/lang_capability_audit/native_builtin_audit.txt` — `.10.1`
  capture, all three arms, **exit 0, 0 divergences**.
- `docs/tasks/artifacts/lang_capability_audit/run_metagrammar_quote_probes.sh` — `.10.5`
  driver. Three arms: **A** static (both defect sites + the bug-class sweep, with every
  non-instance adjudicated and its reason written next to it), **B** behaviour (10
  declared verdicts through the REAL generated meta-parser via `ebnf_dual_run_diff`,
  regenerating `generated/ebnf.rs` from the current grammar with **input AND output paths
  pinned**), **C** corpus (the tracked-grammar acceptance map, each `REJECT` mapped to its
  source line so the report explains itself). ⭐ Ground truth is pinned **inside** the
  instrument — a POSITIVE control (a legal single-quoted string must still parse) and a
  NEGATIVE control (the already-correct double-quoted twin must still reject); the script
  REFUSES rather than reporting a verdict if either moves.
- `docs/tasks/artifacts/lang_capability_audit/run_dual_run_telemetry_probes.sh` — `.10.9`
  driver. Three arms: **A** reproduce (the gate as it stood at the PINNED parent
  `1362b375`, run against the live artifact, plus the bash-arithmetic measurement showing
  `(( 276 < null ))` is vacuously satisfied), **B** controls (one fabricated artifact per
  guard — key deleted / key null / `raw_ast_status=skip` / rule count below the ratchet —
  each asserted to FIRE with its exact message, then a POSITIVE control on the untouched
  artifact), **C** chain coherence (every `KEY` a downstream gate reads out of a producer
  summary must be a key that producer emits, over 6 script→summary edges). ⭐ Ground truth
  is pinned **inside** the instrument twice: arm B's positive control, and arm C's injected
  unresolvable read which the census MUST report — the driver reports a divergence rather
  than a verdict if either moves. Its own A2 verdict caught a harness bug on first run (the
  parent gate staged at the wrong depth re-roots its `ROOT_DIR`).
- `docs/tasks/artifacts/lang_capability_audit/dual_run_telemetry_probes.txt` — `.10.9`
  capture, all three arms, **exit 0, 0 divergences**, 226 producer→consumer reads checked.
- `docs/tasks/artifacts/lang_capability_audit/metagrammar_quote_probes.txt` — `.10.5`
  capture, **exit 0, 0 divergences**.
