# LANG-CAPABILITY-AUDIT: which real-language constructs can PGEN's EBNF actually express?

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

- **Status: `todo`** — **run before trusting either number.** Establish the lint's
  entry-universe semantics (`--lint-grammar` unreachable computation), re-run the
  closure over the same universe, and adjudicate: instrument bug, semantics
  mismatch, or my probe's entry choice. Toolbox-first; no verdict without tool output.

### `.3` — The ECMA-262 expressiveness pass

- **Status: `todo`** — the pass the director greenlit. A **paper exercise**: no
  grammar written, no parser built. For each canonical JS parsing construct, is it
  expressible in PGEN's EBNF *today*? Verdict per row: **expressible** /
  **expressible-but-unwired** / **gap**.
- Seed rows (from the `.1` families + the known-hard JS list, all **UNMEASURED** until
  this leaf runs): grammar parameters `[In, Yield, Await]`; ASI `[no LineTerminator here]`
  (→ [`LEX-ADJACENCY`](LEX-ADJACENCY.md)); cover grammars / arrow-params-vs-parenthesized-expr;
  regex-vs-division; contextual keywords; lookahead-set restrictions; template
  literals with nested `${}`; Unicode ID_Start/ID_Continue; strict-mode directive
  prologue; the Early Errors layer.
- ⚠️ Do **not** assume PGEN's scannerless PEG handles the lexical ones for free —
  measure, per this session's standing lesson.

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
