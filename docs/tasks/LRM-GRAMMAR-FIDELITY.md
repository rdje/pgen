# LRM-GRAMMAR-FIDELITY: make the LRM-markdown→EBNF extraction SOTA + a standing guardrail so silent construct-drops (the `|->`/`|=>` class) can never recur

## Metadata

- Tree ID: `LRM-GRAMMAR-FIDELITY`
- Status: **`active`** (created 2026-07-23, session #198, on the director's
  directive — verbatim: *"What do you [think] about task-tree tracking this
  'the LRM-markdown→EBNF extractor' and at a later point in time to make this
  extractor sota, top-notch and put in place all the necessary guard rails for
  this problem to never happen again. Does it make any sense, your take?"* —
  raised mid-`SV-CORPUS-GRAD.3.3` after that leaf found the SVA `|->`/`|=>`
  operators had been silently dropped by the extractor; see
  [[project_lrm_extractor_sota_and_guardrails]])
- Roadmap lane: cross-family grammar-fidelity infrastructure — serves the
  **Nexsim SV delivery** (the `.9` 100%-LRM-coverage mandate,
  [[project_sv_corpus_100pct_lrm_coverage_mandate]]), the **VHDL campaign**
  ([[project_all_parsers_done_requires_corpus_graduation]]), and the
  **horizon goal** (parse any precisely-described language,
  [[project_horizon_universal_parser]]). Sibling of `SV-CORPUS-GRAD` (which
  finds extraction drops REACTIVELY, one corpus family at a time); this tree
  finds and prevents them PROACTIVELY, at the source.
- Created: `2026-07-23`
- Owner: repo-local workflow

## Why this tree exists — the `|->`/`|=>` signature

`SV-CORPUS-GRAD.3.3` (`SV-0039`) found that the two most fundamental SVA
operators — overlapped `|->` and non-overlapped `|=>` (IEEE 1800-2017 A.2.10) —
had been **entirely absent from `grammars/systemverilog.ebnf` since the
beginning** (`git log -S` empty). Root cause: the LRM-markdown→EBNF extractor
split the `|`-prefixed operators on `|`, which is the EBNF **alternation
metacharacter**, leaving the mangled `implies` (`->`) / `or_assign` (`|=`)
remnants in `prop_primary`.

This is a **class**, not a one-off. Every LRM operator/construct whose literal
collides with an EBNF metacharacter is suspect the same way. SV operators that
collide with `| ( ) * ? [ ] { }`: `|->`, `|=>`, `[*N]`, `[->N]`, `[=N]`, `(* … *)`
(attributes), `->>`, `##`. The current burn-down (`SV-CORPUS-GRAD.3.x`) finds
these **reactively** — O(defects), one corpus family at a time. A proactive
audit + a standing guardrail is **O(1)** for the whole class.

### Second confirmed instance — the `ref` reserved-word omission (`SV-CORPUS-GRAD.3.4`, `SV-0040`, 2026-07-23)

A wider face of the same LRM-fidelity gap surfaced during `SV-CORPUS-GRAD.3.4`:
`ref` — a genuine SV `port_direction` (`port_direction_sv_only := kw_ref`) and
an **IEEE 1800 Table B.1 reserved keyword** — was absent from
`reserved_non_keyword_identifier_sv`, so it wrongly matched `port_identifier`.
It fell through the `SV-0036` net (which appended only the
`verilog_2005`-reserved delta to the SV list; `ref` is SV-*only*-reserved, so it
was never in that delta). This is the same class — an LRM-mandated terminal
(here a reserved keyword) missing from the shipped grammar — surfacing
**reactively** only because a modport fix happened to expose it. The `.1`
coverage audit's scope should therefore include not just Annex A **productions**
but the **reserved-keyword table (Annex B)** cross-checked against the
grammar's `reserved_non_keyword_identifier_sv`/`_v2005` lists, per profile — a
bounded, high-yield check that would have named `ref` (and any siblings)
proactively.

## The sharpening caveat (the honest architecture note)

The extractor is **no longer on the live regeneration path.** The shipped
`grammars/systemverilog.ebnf` is a hand-annotated flattened *synthesis*
(`@profiles`, return annotations, store predicates) that a re-extraction would
destroy. So "make the extractor SOTA" *alone* does not fix the shipped grammar.
That splits the work into two pieces with different leverage:

1. **Highest-leverage, extractor-independent:** a standing **LRM-Annex-A ⟷
   shipped-grammar coverage gate** — prove every LRM production/operator is
   reachable in `systemverilog.ebnf`. This finds all the remaining
   `|->`-siblings in one pass and prevents recurrence, regardless of how the
   grammar was authored. It is the proactive twin of `SV-CORPUS-GRAD.7a`'s
   rule-coverage instrument, closing the loop from the *other* direction
   (LRM→grammar, not corpus→grammar), and directly feeds the `.9` mandate.
2. **Systemic capstone:** the SOTA extractor + an extraction-fidelity gate —
   valuable as (a) a re-derivable *oracle* to diff the shipped grammar against,
   and (b) the faithful path for future LRM-based families (VHDL; the horizon
   goal). The generalizable guardrail: any markdown→EBNF extractor must
   escape/detect operator literals that collide with EBNF metachars, with a
   round-trip check that the emitted grammar's operator terminals ⊇ the LRM's
   operator table.

## Ground rules

- **TOOLBOX-first + task-acceptance on every landing leaf** (root cause +
  addressed + no regression, evidence-backed).
- **Never game coverage:** N/A-with-cause only for LRM clauses/productions with
  no parse surface (the ratified principle, cf. `SV-CORPUS-GRAD.9`).
- **Heavy runs under `scripts/run_with_memory_guard.sh`**.

## Leaves

### `.1` — The LRM-Annex-A ⟷ shipped-grammar coverage audit (the recommended first leaf)

- **Status: `todo`** — a read-only, tool-backed sweep that enumerates every
  IEEE 1800-2017/2023 Annex A production and operator terminal (from the in-repo
  LRM markdown `docs/systemverilog/2017`/`2023`) and checks each is
  **reachable/representable** in `grammars/systemverilog.ebnf`. Output: the
  list of LRM constructs with NO grammar derivation path (the proactive
  `|->`-sibling worklist) + the metachar-collision operators specifically
  audited (`|->`, `|=>`, `[*]`, `[->]`, `[=]`, `(* *)`, `->>`, `##`). Each gap
  becomes an `SV-CORPUS-GRAD.3.x`-style fix leaf. This is the highest-yield,
  bounded first step — it surfaces the rest of the class before sessions are
  spent finding them one corpus row at a time.

### `.2` — The standing coverage gate

- **Status: `todo`** — promote `.1`'s audit into a deterministic `make` gate
  (`lrm_grammar_coverage_gate`) that fails when a required LRM production/
  operator has no grammar reachability, wired so the class cannot silently
  recur. Complements `SV-CORPUS-GRAD.7a` (corpus→grammar) with the LRM→grammar
  direction.

### `.3` — SOTA the extractor + an extraction-fidelity gate (the systemic capstone)

- **Status: `todo`** — make `tools/` LRM-markdown→EBNF extraction faithful for
  metachar-colliding operator literals (escape/detect `| ( ) * ? [ ] { }`), and
  add a fidelity gate: a fresh extraction's operator-terminal set ⊇ the LRM's
  operator table, with a round-trip proof. Valuable as a re-derivable oracle
  and the faithful path for future LRM families (VHDL; the horizon goal). Own
  scope; sequenced after the SV delivery push per the director's "at a later
  point in time".

### `.4` — Generalize to the other LRM-based families

- **Status: `todo`** — apply the `.1`/`.2` audit+gate to VHDL (IEEE 1076) and
  verilog_2005 (IEEE 1364-2005), then any future LRM-based grammar. The
  guardrail is language-agnostic (the metachar-collision class is universal).

## Sequencing (recorded, director-gated)

The director framed the SOTA-extractor work as "at a later point in time" —
after the current SV delivery push. My recorded engineering recommendation
(see the decision record) is that **the `.1` coverage audit is worth pulling
forward**, because it will surface the rest of the `|->`-class proactively
rather than reactively. `.3` (the extractor rewrite) can wait. The director
decides when each leaf opens. This session created the tree (the "task-tree
track this" directive) and did NOT start any leaf — the SV `.3.x` burn-down
remains the active frontier.

## Acceptance Criteria (tree)

1. A tool-backed audit proving, per LRM production/operator, whether it is
   reachable in the shipped grammar (gaps named + routed to fix leaves).
2. A standing deterministic gate (`.2`) that fails on a missing required LRM
   construct, so the silent-drop class cannot recur.
3. (Capstone) A SOTA extractor + fidelity gate handling metachar-colliding
   operator literals, usable as a re-derivable oracle and for future families.
4. Full lockstep (books/contracts/tracker) and every landing leaf through the
   TOOLBOX acceptance checklist.
