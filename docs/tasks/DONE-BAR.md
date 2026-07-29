# DONE-BAR: `Done` means SOTA / SIGNOFF-LEVEL done — a user can blindly trust the parser: stimuli-generator proof AND all gates AND all external corpus, and the trust that establishes

## Metadata

- Tree ID: `DONE-BAR`
- Status: `active` (opened 2026-07-29, session #221)
- Family / slice-id prefix: `PGEN-DONE-BAR-<NNNN>`
- Created: `2026-07-29`
- Owner: repo-local workflow
- Director directive: [[feedback_done_bar_is_first_tier_only]]
- Frontier: **`.2b`** (move the rows — the gates now compute the truth the tracker must state) — `.2a` DONE 2026-07-29 (`PGEN-DONE-BAR-0010`); `.1` DONE 2026-07-29 (`PGEN-DONE-BAR-0002`); `.2` SPLIT into `.2a` → `.2b` (`PGEN-DONE-BAR-0005`)

## Goal

Every parser family that claims `Done` in `LIVE_ACHIEVEMENT_STATUS.md` meets **all three legs**,
currently and simultaneously:

1. **stimuli-generator proof** — the family's own generated samples close the loop, zero residual
   actionable-target debt;
2. **all our gates** — every gate covering the family is green *now*, and is actually invoked by
   something;
3. **all the external test corpus** — an officially-recognized third-party corpus, **passing** —
   not triaged, not characterized.

**The purpose those three serve** — *"sota, signoff level … user can now blindly trust the parser"*
(director, verbatim) — is not a fourth leg. It is what leg 2 must GROW to cover: the consumer-facing
checks that no gate performs today (silent-success paths, published-version currency, open ledger
entries, documented boundaries). See "LEG 4 REFRAMED" below; they are owned by `.5`.

Rows that do not meet all three are **demoted**, visibly, with the unmet leg named — to `Provisional` when legs 1-2 hold, otherwise lower.

## ⭐⭐⭐ THE FLOW SHALL GUARANTEE THIS 100% (director 2026-07-29, verbatim) — this is the tree's GOVERNING requirement

> *"So the flow shall guarantee this 100%"*

⇒ **the bar shall not be a checklist someone remembers to apply; it SHALL BECOME an invariant the flow enforces.** ⚠️ **It is NOT enforced today** — the measured blocker is the next paragraph. A
`Done` row that does not meet all three legs must be **impossible to hold**, because a gate fails
while it is held. This reorders the tree: `.4` (enforcement) is not the tidy-up at the end, it is the
point, and `.1`-`.3` are what make enforcement possible.

⛔ **AND THE FLOW CANNOT DELIVER THAT GUARANTEE TODAY — measured, not asserted.** Three findings from
`CI-PARITY-GATE-ROT`, all already in the record, compose into a hard blocker:

| # | measured fact | consequence for the guarantee |
|---|---|---|
| 1 | `sota_exit_gate` — the aggregate that would prove the families — **has never completed green**; best run 2026-07-29 is 32 gates entered / 30 ok / 1 fail at 5 h 05 m | there is no single green proof surface to enforce against |
| 2 | **the AUTOMATIC tier over all 123 gate targets is ZERO** (`.6`): 14 of 15 workflows are `workflow_dispatch`-only, and the one that auto-runs invokes no `make` target | nothing re-proves a `Done` row unless a human asks |
| 3 | the family status gates are reachable **only** through that aggregate | a row can rot for months undetected — exactly how `regex` held `Done` against a gate that disagreed |

⇒ ⭐ **"the flow guarantees the Done bar" is BLOCKED ON `CI-PARITY-GATE-ROT.7` (aggregate green) AND
on the escalated director call about resuming hosted auto-triggers** (`.6`, which spends Actions
minutes and was deliberately not taken unilaterally). Those are not separate projects from this one —
they are this one's prerequisites, and that is now written down rather than discovered later.

⚠️ **HONEST BOUND ON THE WORD "100%", stated up front rather than discovered in an audit.** A
pre-commit hook is bypassable (`--no-verify`), and a locally-run gate proves only the machine that
ran it. The strongest honest guarantee available is: **enforced on every commit through the hook AND
re-proved by an automatic CI lane that no contributor controls.** Leg 2 of the bar already says a
gate nothing invokes does not count as green — the same standard applied to the enforcement of the
bar itself. Anything less gets *stated*, never quietly claimed as 100%.

## ⭐⭐ LEG 4 REFRAMED (2026-07-29, on the director's challenge *"I don't really understand (4). Why do we need it?"*)

**The challenge was right and the original framing was wrong.** Leg 4 was written as *"a downstream
team could build on it without re-verifying"* — a **goal statement, not a criterion**. It is
unmeasurable as phrased, and the director had just required that **the flow guarantee the bar 100%**.
⛔ An unmeasurable leg cannot be guaranteed by a flow. Those two statements were in conflict, and the
conflict was mine.

**But the thing it was pointing at is real, and it is MEASURED in this repo.** Legs 1-3 all ask *does
the parser behave correctly*. They cannot see failures that are invisible to the producer's own tests
and land on the consumer:

| measured failure class | why legs 1-3 cannot see it | evidence |
|---|---|---|
| **silent success** — codegen paths that return `Ok` with **zero diagnostics**, emitting `<property_access>` / `<array_access>` sentinels | the parse SUCCEEDS, so every "did it parse" gate is green while the consumer receives a placeholder node | **6 sentinel emission sites** measured; `DOCTRINE-GAP-OWNERSHIP.3` |
| **stale published contract** | no gate reads the user-facing docs | `PGEN_USER_GUIDE.md:3808,3810` publishes regex `1.1.29`/`1.1.31` as CURRENT; the contract says `1.1.104`/`1.1.106` — **~75 releases stale**; `DOCTRINE-GAP-OWNERSHIP.4` |
| **known open defect found downstream** | by definition our gates missed it, which is why it reached the ledger | released-parser bug ledger |

⇒ **measured: NO gate references the released-parser bug ledger or the user guide.** Zero coverage.

### ⭐⭐⭐ THE RESOLUTION — leg 4 is not a fourth kind of proof; it is a LIST OF GATES WE HAVE NOT WRITTEN

Every item above is a concrete, checkable fact about a *consumer-facing* artifact:

1. zero sentinel/silent-success paths reachable for the family;
2. the published version pair matches the released parser (⇒ a gate that reads the user guide);
3. zero open ledger entries naming the family;
4. the family's acceptance boundary is documented where a consumer looks.

**Written as gates, they collapse into leg 2** (*all gates covering the family are green now and
actually invoked*). ⇒ **the bar is THREE legs, all measurable, all gateable** — which is exactly what
*"the flow shall guarantee this 100%"* requires, and which the four-leg form could not deliver.

⇒ leg 4 is **retired as a leg** and re-entered as `.5`: *the missing consumer-facing gates*.

### ✅ AND IT IS NOT ON THE CRITICAL PATH — measured, which is why deferring it is safe

| family claiming `Done` | leg 1 | leg 3 (external corpus) |
|---|---|---|
| `systemverilog_preprocessor` | — | ⛔ **none** |
| `return_annotation` | — | ⛔ **none** |
| `rtl_frontend` | — | ⛔ **none** |
| `vhdl` | — | ⚠️ triage only |
| `regex` | ⛔ `final_targets=31` | ✅ 3 gates |

**Every one of the five fails legs 1-3 already.** Nothing reaches the consumer-facing checks. ⇒ they
can be built **after** legs 1-3 without ever minting a 2nd-tier `Done`, because no row can be
promoted on legs 1-3 alone while `.5` is open — a row that clears 1-3 is marked
**`PROVISIONAL — consumer-facing gates unbuilt`**, never `Done`.

## ⭐⭐⭐ `Provisional` IS A SHIPPING TIER, AND DISCLOSURE IS LOAD-BEARING (director 2026-07-29)

> *"It means we need to push all the parsers forwards to reach Provisional Done. If we can't find an
> external corpus for some parsers, they will stay as provisional but this should not prevent them
> from being used by downstream customers. The customers should know the state of what they are
> getting, then based on that information, it is up to them to use them or not."*

**This changes the campaign's shape and one of its conclusions.**

- ⭐ **The near-term goal is not `Done` for everything — it is `Provisional` for EVERYTHING.** Legs 1
  and 2 (stimuli-generator loop closed with zero residual debt; every covering gate green *now* and
  actually invoked) are the achievable bar for every family. Leg 3 promotes `Provisional` → `Done`.
- ⭐ **`Provisional` ships.** It does not block downstream use. The consumer is told exactly what is
  proven and what is not, and decides.
- ⛔⛔ **AND THAT MAKES DISCLOSURE LOAD-BEARING, WHICH REVERSES THIS TREE'S EARLIER CONCLUSION.**
  `.5` (the consumer-facing gates) was recorded as *"safely deferred, not on the critical path"*
  because no family reached it. **That reasoning does not survive this directive.** If a `Provisional`
  parser ships and the customer decides from its published state, then the published state being
  TRUE is the thing the whole arrangement rests on — and it is **measurably not true today**:
  `PGEN_USER_GUIDE.md:3808,3810` publishes regex `1.1.29`/`1.1.31` against the contract's
  `1.1.104`/`1.1.106`, ~75 releases stale, with **no gate reading either document**.
  ⇒ **`.5` is promoted from deferred to a PREREQUISITE for shipping `Provisional` honestly.**
  *A disclosure nobody checks is a claim, not a disclosure.*

### ⭐⭐ A DISTINCTION THAT MAKES THE MODEL COHERENT: which families CAN reach `Done` at all

Leg 3 requires an *officially-recognized third-party* corpus. That is available only where the
language is externally standardized:

| family | language owned by | leg 3 reachable? | attainable tier |
|---|---|---|---|
| `systemverilog`, `systemverilog_preprocessor`, `vhdl`, `regex`, `json`, `verilog_2005` | **IEEE / PCRE2 / ECMA / JSONTestSuite** — external standards | ✅ yes, a corpus exists in the world | `Done` (today: `Provisional (corpus pending)`) |
| `return_annotation`, `semantic_annotation`, `ebnf` | ⭐ **PGEN itself** | ⛔ **no — by construction** | ⭐ **`Provisional (ceiling)` — a FINISHED row** |
| `rtl_frontend`, `rtl_const_expr` | a *subset* PGEN defines of SystemVerilog | ⚠️ partially — SV corpora exist but exercise far more than the subset | to be adjudicated in `.3` |

### ⛔⛔ `ceiling` IS DELIBERATELY HARD TO CLAIM — the gradient runs the wrong way

`(ceiling)` says *finished*; `(corpus pending)` says *you still owe work*. **The comfortable label is
the one that closes the row**, so without a rule every awkward family drifts into `ceiling`. The rule:

- `ceiling` requires the language be **defined by PGEN itself** — no external standards body, no
  widely-recognized reference implementation defines it.
- ⛔ ***"We could not find a corpus" is NEVER a ceiling.*** Absence of search is not absence of
  existence. If an external standard defines the language, the row is `(corpus pending)` — however
  long it stays there, and however unlikely the corpus looks.
- ⇒ **exactly three families are expected to qualify**, all three because PGEN authored their
  language. A fourth claim should be treated as suspicious until it names the standard that does not
  exist.
- ⭐ **Mechanizable, and it should be** (`.4`): a `(ceiling)` row must name a `grammars/*.ebnf` PGEN
  authored and carry a justification entry in a tracked register — the `gate_reachability_register_v0.json`
  shape, where an untriaged claim FAILS rather than being reported.

⇒ **three families are permanently `Provisional` and that is CORRECT, not a failure.** Recording this
stops them being re-litigated every audit, and stops the tracker looking like it carries three
never-closing rows. ⚠️ It is a first-pass classification from language ownership, to be confirmed in
`.3` — not a finding.

## Non-Goals

- Lowering the bar to fit the current state. The directive is explicit: *"no 2nd or 3rd tier Done."*
- Silently rewriting history. Dated tracker notes that were correct when written stay; the **current**
  row is what moves.
- Deciding regex's specific residual debt here — that is `REGEX-PCRE2-FIDELITY.ROUTED-IN-2`. This
  tree owns the **bar**, not each family's remediation.

## Acceptance Criteria

- A re-runnable audit reports, per family, the state of each of the three legs, with its own ground
  truth so it says `MISCALIBRATED` rather than reporting a comfortable number.
- Every `Done` row either survives the audit with evidence, or is demoted with the unmet leg named.
- The bar is **enforced**, not just documented, so a row cannot drift back to `Done` without the
  three legs — mechanized once the audit's shape is known (⛔ price before mechanizing:
  `GENERATED-LINT-CORRECTNESS.4`'s rule).

## ⭐ THE MEASURED STARTING POINT (2026-07-29, before any remediation)

Taken at directive time. **Leg 3 is the one the repo is furthest from.**

**Corpus-facing gate targets, all 6 of them:**

| gate | family | kind |
|---|---|---|
| `regex_broader_corpus_proof_gate` | regex | proof |
| `regex_pcre2_textsafe_corpus_gate` | regex | proof |
| `regex_corpus_bundle_contract_gate` | regex | contract |
| `verilog_2005_conformance_gate` | verilog_2005 profile | conformance |
| `sv_external_corpus_triage_gate` | systemverilog | ⚠️ **triage** |
| `vhdl_external_corpus_triage_gate` | vhdl | ⚠️ **triage** |

**External corpus bundles that exist:** `regex_corpus_bundle/`, `json_corpus_bundle/` — and the
JSON one is self-described as *"a characterization, not a conformance gate"*.

⇒ **first-pass reading, to be confirmed by `.1`:** of the families currently claiming `Done`
(`systemverilog_preprocessor`, `vhdl`, `regex`, `return_annotation`, `rtl_frontend`), **only `regex`
has a genuine external-corpus lane — and `regex` is the family currently FAILING its own status
gate.** `systemverilog_preprocessor`, `return_annotation` and `rtl_frontend` appear to have **no
external corpus at all**; `vhdl`'s is triage.

⚠️ **That is a first-pass reading from gate names and directory listings, not a per-family audit.**
It is recorded as the hypothesis `.1` must confirm or refute — ⛔ **not** as a finding. This session
has already produced three instrument defects from exactly this kind of shortcut (a redirect counted
as a read, `systemveriLOG` matching `log`, and a family grep that assumed the `systemverilog_` prefix
when the gate is named `sv_`).

### ⛔⛔ ADJUDICATED BY `.1` (2026-07-29) — the hypothesis above was RIGHT about four families and **WRONG about `regex`, in `regex`'s favour**

Left above verbatim; corrected here rather than back-dated.

| hypothesis | `.1`'s measurement | verdict |
|---|---|---|
| *"only `regex` has a genuine external-corpus lane"* | its two genuine lanes (`regex_pcre2_textsafe_corpus_gate`, `regex_corpus_bundle_contract_gate`) are invoked by **NOTHING**; the lane that runs, `regex_broader_corpus_proof_gate`, reads `rust/test_data/regex/stress_tests.json` — **44 repo-authored cases** | ⛔ **REFUTED** |
| *"`vhdl`'s is triage"* | confirmed **and priced**: 8 declared cases against **13,720** vendored files = **0.058%** | ✅ confirmed, now quantified |
| *"`systemverilog_preprocessor` / `return_annotation` / `rtl_frontend` appear to have no external corpus at all"* | confirmed — zero corpus roots, zero corpus-facing gates for all three | ✅ confirmed |
| *"the 24 `open` tokens may be open defects"* | **168 ledger rows, 0 open** against the ledger's own state vocabulary | ⛔ REFUTED (as suspected) |

⭐ **THE SHAPE OF THE ERROR IS THE LESSON**: the first-pass reading counted **gate NAMES** — three
gates with `corpus` in the name, therefore a corpus lane. Leg 3 asks three further questions no name
answers: *does anything RUN it, does it read the EXTERNAL corpus, and is it a conformance gate or a
triage sample?* All three regex gates fail at least one. **A gate roster is not a proof surface.**

### ⚠️ A LEG-4 SIGNAL, RECORDED AS A HYPOTHESIS

`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` contains **24 occurrences of the token `open`**
(against 102 `fixed`, 72 `closed`, 18 `resolved`). ⛔ **That is a token tally, not a count of open
defects** — `open` occurs in prose too. It is recorded because a released-parser ledger with genuinely
open entries is a direct leg-4 blocker for whichever families they name, and `.1` must resolve it to
actual per-family open-defect counts rather than leaving it as an impression.

## Leaves

### `.1` — audit every `Done` claim against the three legs (`done`, 2026-07-29 session #223)

- **Status: `done`** — `PGEN-DONE-BAR-0002`. Instrument `scripts/audit_done_bar.sh` + register
  `rust/test_data/grammar_quality/done_bar_family_register_v0.json` + 3 tracked drivers and their
  captures under `docs/tasks/artifacts/done_bar/`.
  ⛔ **No grammar, no `rust/src/*`, no `generated/*`, no `rust/scripts/*` ⇒ all 11 generated parsers
  BYTE-IDENTICAL BY CONSTRUCTION**; no release / schema / ledger / contract movement.

#### ⭐⭐⭐ THE HEADLINE: **5 of 5 `Done` rows DO NOT MEET THE BAR** — and every one fails all three legs

Derived, not hand-listed: **7** parser families are on the tracker (`LIVE_ACHIEVEMENT_STATUS.md`
table rows joined against `grammars/*.ebnf`), **5** claim `Done`.

| family | tracker | leg 1 | leg 2 | leg 3 | verdict |
|---|---|---|---|---|---|
| `regex` | `Done` | ⛔ UNMET | ⛔ UNMET | ⛔ UNMET | DOES NOT MEET BAR |
| `vhdl` | `Done` | ⚠️ UNPROVEN | ⚠️ UNPROVEN | ⛔ UNMET | DOES NOT MEET BAR |
| `systemverilog_preprocessor` | `Done` | ⚠️ UNPROVEN | ⚠️ UNPROVEN | ⛔ UNMET | DOES NOT MEET BAR |
| `return_annotation` | `Done` | ⚠️ UNPROVEN | ⛔ UNMET | ⛔ UNMET | DOES NOT MEET BAR |
| `rtl_frontend` | `Done` | ⚠️ UNPROVEN | ⛔ UNMET | ⛔ UNMET | DOES NOT MEET BAR |
| `systemverilog` | `Mostly Done` | ⚠️ UNPROVEN | ⚠️ UNPROVEN | ⛔ UNMET | *(context — not a `Done` claim)* |
| `rtl_const_expr` | `Mostly Done` | ⚠️ UNPROVEN | ⛔ UNMET | ⛔ UNMET | *(context — not a `Done` claim)* |

⚠️ **`UNPROVEN` is not a softer `UNMET`; it is the audit refusing to score a leg it cannot see.** It
does not satisfy the bar. The charter said *"expect the audit to fail several rows — that is the
directive working"*; it failed all of them, and the reasons are specific, not systemic pessimism.

#### ⭐⭐⭐ FINDING 1 — the tree's own first-pass reading was WRONG **in `regex`'s favour**

The starting-point table recorded `regex` leg 3 as **"✅ 3 gates"**, the one family with a genuine
external-corpus lane. **Measured, that is refuted:**

- `regex_pcre2_textsafe_corpus_gate` and `regex_corpus_bundle_contract_gate` **read the real PCRE2
  corpus and are invoked by NOTHING** — orphan `make` targets whose scripts no gate calls. (Their
  only references are `ci_workflow_local_gate.sh`'s `assert_tracked` lines: a **mention**, not an
  invocation — the exact defect `CI-PARITY-GATE-ROT.2` had to fix, replayed here as probe RED-5.)
- The one regex corpus lane that **does** run, `regex_broader_corpus_proof_gate`
  (`… <- regex_formal_exhaustive_closure_gate <- regex_parser_family_status_gate`), reads
  **`rust/test_data/regex/stress_tests.json` — 44 repo-authored cases.** It is not an external
  corpus at all.

⇒ **`regex` has NO external-corpus proof in any lane anything runs**, while the 2,189-case PCRE2
oracle corpus sits vendored and unwired. The tracker's *"Broader-corpus proof remains green at
44/44"* reads like external evidence and is not.

#### ⭐⭐⭐ FINDING 2 — `vhdl`'s `Done` rests on a criterion named `external_corpus_backed_proof_surface` that a gate named `_triage_` satisfies, over **0.058%** of the corpus

`vhdl_formal_exhaustive_closure_gate.sh:187-211` sets `external_corpus_backed_proof_surface_present`
from `vhdl_external_corpus_triage_gate`, and asserts the producing gate's identity is *literally*
`vhdl_external_corpus_triage_gate`. Priced (`run_corpus_scale_census.sh`):

| family | declared cases | vendored source files | corpora referenced | coverage |
|---|---|---|---|---|
| `vhdl` | **8** | **13,720** `.vhd`/`.vhdl` | 5 of 10 | **0.058%** |
| `systemverilog` | **7** | **16,388** `.sv`/`.svh`/`.v`/`.vh` | 4 of 14 | **0.043%** |

The bar says *ALL the external test corpus, **passing***, and *"a TRIAGE gate is not a conformance
gate."* `parse_pass_total == cases_declared` is a curated sample reporting itself green.

#### ⭐⭐ FINDING 3 — **2 of the 5 `Done` families have no family-status gate at all**

`return_annotation` and `rtl_frontend` are computed by nothing: no `*_parser_family_status_gate`
emits a status for them, so **no instrument can ever disagree with their tracker row.** The three
that do have one (`sv_` covering two families, `vhdl_`, `regex_`) are exactly the three whose rows
have ever been contested. ⭐ And `README.md` names `return_annotation_support_gate` *"the formal
`Done` gate for the tracked return-annotation claim"* — **it is an ORPHAN**, invoked by nothing.

#### ⭐⭐ FINDING 4 — the `Done` rows rest on artifacts that PREDATE the tracker they assert alignment with

`sv_parser_family_status_gate` and `vhdl_parser_family_status_gate` both assert
`tracker_alignment_ok`. Their newest artifacts are from aggregate run 3 at **09:55** and **13:46**;
`LIVE_ACHIEVEMENT_STATUS.md` was last written at **14:49** the same day. ⇒ the alignment they
recorded is **no longer proven**, so the audit reports green-NOW as `UNPROVEN`, not `MET`. This is
`CI-PARITY-GATE-ROT.5`'s stale-artifact class (*a check that reuses evidence it did not produce,
without checking whether it still applies*) — now measured on the `Done` bar's own inputs, and the
reason the audit compares every artifact against the newest mtime among the inputs it judged.

#### ✅ FINDING 5 — the ledger hypothesis is RESOLVED and REFUTED: **zero** open entries

The tree recorded *"24 occurrences of the token `open`"* as a hypothesis and required `.1` to resolve
it to real per-family counts. Read against the ledger's **own** state vocabulary (derived from its
"State Meanings" section, not a list in the script): **168 rows, 0 open**, across all 7 families.
The 24 was prose, exactly as suspected. ⛔ **But nothing reads that file**, so the fact is *true and
unguarded* — it is `.5`'s third gate, not a leg that passes.

#### ⚠️ TWO DEFECTS IN THIS INSTRUMENT'S OWN FIRST CUT — caught by controls, recorded not hidden

1. **Family-status gates were attributed by NAME prefix**, so `sv_parser_family_status_gate` (prefix
   `sv_`) went to `systemverilog` and **`systemverilog_preprocessor` was reported as having no status
   gate at all** — worse than it measures, when the run-3 artifact plainly records
   `systemverilog_preprocessor_status: Done`, 12/12 criteria satisfied, `final_targets: 0`.
   Attribution is now derived from what each gate **EMITS**; control **C9** pins it and probe
   **RED-4** proves C9 fires.
2. **A gate that RAN AND DIED was reported as merely `UNPROVEN`.** `regex_parser_family_status_gate`
   left a **0-byte** `summary.txt` (the `CI-PARITY-GATE-ROT.14` shape) beside a log naming the real
   cause. The audit now recovers it — *`error: regex tracker alignment mismatch: computed 'In
   Progress' but tracker says 'Done'`* — and reports `UNMET` with the cause, not `UNPROVEN`.
   Control **C10** holds it, conditionally so a clean `rust/target` cannot fail it.

⭐ Both defects moved a verdict in the **flattering** direction for one family and the **harsh**
direction for another. That is why the controls are the deliverable, not the number.

#### ⛔ WHAT THIS LEAF DELIBERATELY DID NOT DO

- **No row is demoted.** `.2` owns that, with this evidence attached — audit first, adjudicate
  second, as the charter requires.
- **No gate was run.** The audit is read-only and cheap (no cargo, no make, no network); it can
  never manufacture the green it is auditing. Everything it could not see it reports `UNPROVEN`.

- **Scope (as chartered):**
  1. Enumerate every row claiming `Done` in the parser-family tables of
     `LIVE_ACHIEVEMENT_STATUS.md` — **derived from the file**, not hand-listed (a hand-list cannot
     see a row nobody added to it).
  2. For each, establish per leg:
     - **leg 1**: the family's residual actionable-target debt and certificate-coverage state, from
       the gates' own artifacts — not from prose;
     - **leg 2**: which gates cover it, whether each is currently green, and whether anything
       actually invokes it (reuse `check_gate_reachability.sh`'s tier model);
     - **leg 3**: whether an officially-recognized external corpus exists, whether it is asserted as
       a **pass** rather than triaged/characterized, and what it currently reports.
  3. Ship it as a **re-runnable, self-calibrating** instrument, with pinned ground-truth facts that
     make it print `MISCALIBRATED` and exit nonzero rather than report a number it cannot back.
  4. Report, per family, `MEETS BAR` / `DOES NOT MEET BAR (leg N unmet: …)`; a row clearing legs 1-3 while `.5` is open is `PROVISIONAL`, never `Done`.
- ⛔ **Do NOT demote rows inside `.1`.** Audit first, adjudicate second — this session has twice paid
  for the opposite order. Demotions land in `.2` with the evidence attached.
- ⚠️ **Expect the audit to fail several rows.** That is the directive working. The instrument must
  not be tuned until the answer is comfortable.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — no instrument existed that could answer *"does this `Done` row meet the
  bar?"*; the tree's own starting point was recorded as a hypothesis from gate names and directory
  listings. `bash scripts/audit_done_bar.sh` now answers it: `audit-done-bar: 5 of 5 `Done` rows DO
  NOT meet the bar` (capture `docs/tasks/artifacts/done_bar/audit_report.txt`).
- [x] **ROOT CAUSE (WHY + WHERE)** — each unmet leg is located, not asserted. `git ls-files`-derived
  gate universe via `scripts/check_gate_reachability.sh --json`; `vhdl_formal_exhaustive_closure_gate.sh:187-211`
  sets `external_corpus_backed_proof_surface_present` from a gate whose identity it asserts is
  `vhdl_external_corpus_triage_gate`; `rust/test_data/grammar_quality/regex_broader_corpus_v0.json`
  names `source_file: rust/test_data/regex/stress_tests.json`, `expected_case_count: 44`;
  `rust/target/sota_exit_gate/logs/regex_parser_family_status_gate.log` carries
  `error: regex tracker alignment mismatch: computed 'In Progress' but tracker says 'Done'` beside a
  **0-byte** `summary.txt`.
- [x] **FIX** — declarative-tier: a read-only audit + a tracked register, no engine or grammar
  change. The roster is DERIVED (tracker × `grammars/*.ebnf`); the register supplies only what a
  name cannot (gate prefixes, corpus roots) and an unregistered family REFUSES rather than being
  skipped.
- [x] **ADDRESSED (verified)** — before: no instrument, and a first-pass reading that credited
  `regex` with a working external-corpus lane. After, measured: **7 families derived, 5 `Done`, 5/5
  do not meet the bar**; `regex` leg 3 refuted (its two external lanes run by nothing, its running
  lane reads a 44-case repo fixture); `vhdl` corpus coverage **8 cases / 13,720 files = 0.058%**, SV
  **7 / 16,388 = 0.043%**; ledger open entries **24 (token tally) → 0 (real, over 168 rows)**.
  Re-runnable oracles: `bash scripts/audit_done_bar.sh` (exit 1),
  `bash docs/tasks/artifacts/done_bar/run_corpus_scale_census.sh`,
  `bash docs/tasks/artifacts/done_bar/run_ledger_open_census.sh`.
- [x] **NO REGRESSION** — no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*`, no
  `rust/scripts/*` ⇒ **all 11 generated parsers byte-identical BY CONSTRUCTION** (the emitting
  inputs are untouched, so cert seeds 0/7/42, the fully-certified grammars' byte-identity, the
  external corpus and `ast_shape_contract` cannot move); `bash scripts/check_doctrines.sh` →
  **ALL 13 enforced doctrines PASS**; probe arms **11/11** with every RED arm flipping and every
  CTRL arm holding (`docs/tasks/artifacts/done_bar/probe_arms.txt`).
- [x] **LOCKSTEP** — `README.md` (Standard Commands), `docs/book/src/quality-and-closure-model.md`,
  `LIVE_ACHIEVEMENT_STATUS.md` (tracker note; ⛔ **no row moves — `.2` owns demotion**), `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `MEMORY.md`. No release / schema / ledger / contract movement: nothing
  executable changed.

### `.2` — demote what does not meet the bar, with the unmet leg named (`todo`, **SPLIT into `.2a` → `.2b`**)

- **Status: `todo`** — ✅ **UNBLOCKED: `.1` is done and every row it must act on is measured**
  (`docs/tasks/artifacts/done_bar/audit_report.txt`). `.2` now has five rows to adjudicate, not one.

#### ⛔⛔ `.2` CANNOT BE A TRACKER EDIT — MEASURED, AND IT REVERSES THE OBVIOUS ORDER (2026-07-29, `PGEN-DONE-BAR-0005`)

`.2` was chartered as *"demote what does not meet the bar"*. Executed literally — as a tracker edit —
**it turns three family-status gates RED, two of which pass today.** Measured by replaying the LIVE
alignment logic, not by reading it
(`docs/tasks/artifacts/done_bar/run_demotion_impact_probe.sh`, capture `demotion_impact_probe.txt`):

| family | the gate COMPUTES | tracker AFTER an honest demotion | alignment |
|---|---|---|---|
| `regex` | `In Progress` | `Provisional (corpus pending)` | ⛔ MISMATCH ⇒ `exit 1` |
| `vhdl` | `Done` | `Provisional (corpus pending)` | ⛔ MISMATCH ⇒ `exit 1` (**passes today**) |
| `systemverilog_preprocessor` | `Done` | `Provisional (corpus pending)` | ⛔ MISMATCH ⇒ `exit 1` (**passes today**) |

**WHY (root cause, located):** the family-status gates implement the **OLD** bar. Their entire
computable vocabulary is `Done` / `Mostly Done` / `In Progress` / `Not Started` — derived from the
live scripts — with **`Provisional` appearing in ZERO gate scripts repo-wide**; they carry no leg-3
criterion at all; and each compares tracker-vs-computed by **exact string equality** then `exit 1`
(`regex_parser_family_status_gate.sh:391-399` and its two siblings).

⇒ **The status an honest demotion must write is a status no gate can COMPUTE.** Demoting first does
not record the truth — it manufactures a disagreement, and blocks `CI-PARITY-GATE-ROT.7` *harder*
than the blocker `.2` was sequenced to clear.

⭐ **THE INVERSION IS THE FINDING: the tracker is not the thing that is wrong.** `regex`'s row was
contested because a gate disagreed with it; here **the gates would disagree with a CORRECT row**.
The bar moved; the instruments that compute against it did not. Fixing the tracker without fixing
the instruments just relocates the lie.

⚠️ **THE PROBE'S OWN FIRST CUT REPORTED `✅ aligns` FOR ALL THREE ROWS — FROM EMPTY STRINGS.** A
broken field split left both sides of the comparison empty, and `"" == ""` is `true`, so the probe
printed the comfortable answer having compared **no data at all**. It was caught because the output
table rendered three blank columns, not because the logic was re-read. ⇒ the probe now **REFUSES**
(`MISCALIBRATED`, exit 3) if either side of a comparison is empty or if it parses other than 3 rows.
**Two empty strings are never evidence of agreement** — the same vacuous-green class as
`CI-PARITY-GATE-ROT.3`'s mistyped filter that replayed zero workflows and printed ✅.

#### `.2a` — teach the family-status gates the new bar (`done`, 2026-07-29 session #224, `PGEN-DONE-BAR-0010`)

- **Status: `done`.** Delivered exactly as chartered, via ONE shared helper
  (`rust/scripts/lib/parser_family_status_bar.sh`, sourced by all three status gates — the `.10`
  duplication lesson applied: `markdown_table_status_for_row` now has a single home):
  1. **`Provisional` is computable**, always qualified, with the qualifier derived from the
     register's `language_owner` exactly as `.1` derives it (`pgen` ⇒ `(ceiling)`,
     `external-standard` ⇒ `(corpus pending)`); an `unadjudicated` owner (rtl_frontend,
     rtl_const_expr) makes the helper **REFUSE (exit 2)** rather than guess — probed (RED-2).
  2. **The leg-3 criterion exists**: `external_corpus_conformance_pass`, added to every family's
     criteria set (regex 8→9, sv 7→8, svpp 12→13, vhdl 10→11). It is met only by a
     register-declared `leg3_surface` `{gate, summary_json, pass_query}` passing the three tests
     `.1` measured every corpus-named gate failing: **conformance not triage** (a `*triage*` name
     REFUSES — RED-3), **external-backed** (the gate script must read a declared corpus root, else
     REFUSE — RED-4), **actually invoked** (reachable per `scripts/check_gate_reachability.sh`; an
     orphan surface leaves leg 3 UNMET with the cause named — CTRL-5). The artifact must satisfy
     the declared pass assertion (CTRL-7). ⛔ NOT a hard-coded false: a fully valid surface scores
     `met=true` (CTRL-6), so `Done` stays honestly reachable the moment `.3` wires a lane.
  3. **`Done` is unreachable while leg 3 is unmet** — the ladder cap (`family_apply_done_bar_status`)
     turns a would-be `Done` into the qualified `Provisional`; statuses below `Done` pass through.
  4. **A misaligned gate now STATES its verdict before failing**: the full summary pair
     (computed status, leg-3 verdict, qualifier, detail) is emitted, THEN `exit 1` — closing this
     family's own instance of the 0-byte-summary shape (`CI-PARITY-GATE-ROT.14`; the `.1` audit had
     to recover this very gate's verdict from its log).
- **Register extension** (`done_bar_family_register_v0.json`): per-family `leg3_surface` (null for
  all 7 today) + a `policy.leg3_surface` entry documenting the declared shape and the three tests.
  The `.1` audit is UNAFFECTED — re-run byte-identical vs the tracked capture, probe arms 11/11.
- **Measured AFTER (replays against the aggregate run-3 artifacts, the same set `.1` audited):**

  | family | run 3 computed (old bar) | now computes | alignment vs tracker |
  |---|---|---|---|
  | `vhdl` | `Done` | **`Provisional (corpus pending)`** | ⛔ exit 1 vs `Done` — correct until `.2b` |
  | `systemverilog_preprocessor` | `Done` | **`Provisional (corpus pending)`** | ⛔ exit 1 vs `Done` — correct until `.2b` |
  | `systemverilog` | `Mostly Done` | `Mostly Done` (cap only affects `Done`) | ✅ aligned |
  | `regex` | `In Progress` | `In Progress` (legs 1-2 unmet ⇒ below the cap) | ⛔ exit 1 vs `Done` — the known `.2b` row |

  ⇒ exactly the chartered outcome: *the gate states the truth, then the tracker agrees with it.*
  ⚠️ The three status gates (and their contract gates) are deliberately RED against today's tracker
  until `.2b` moves the rows — same-session work, not a parked breakage.
- **Consumer surface priced before mechanizing:** the status CONTRACT gates assert criteria-count /
  unmet-array / detail-mapping consistency, not pinned totals — all invariants verified holding on
  every replay summary (jq consistency probe in the verification log). The combined-telemetry gates
  and `sota_exit_gate` copy values (same-run parity), so the new status strings propagate. The
  audit's `STATUS_KEY_RE` derivation and controls C9/C10 verified unaffected.
- `run_demotion_impact_probe.sh` is marked **superseded as a live instrument** (its step-2 column is
  the run-3 snapshot under the OLD logic; its own "Provisional anywhere" count moved 0 → 4). The
  live AFTER instrument is `run_family_status_bar_probes.sh`.

##### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the demotion-impact probe's measurement (capture
  `docs/tasks/artifacts/done_bar/demotion_impact_probe.txt`): status vocabulary derived from the
  live scripts = `Done/In Progress/Mostly Done/Not Started` only, `Provisional` in **zero** gate
  scripts, and an honest demotion ⇒ `⛔ MISMATCH ⇒ gate exit 1` for 3 of 3 rows.
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family: the gates compare tracker-vs-computed by
  exact string equality then exit (`regex_parser_family_status_gate.sh:391-399` pre-change, and the
  sv/vhdl siblings), with no leg-3 criterion in any gate; verified against the live scripts by the
  probe's own extraction (`sed -n '/^markdown_table_status_for_row/…'` from the gate, `bash -n`
  clean on all five edited scripts; `git ls-files`-tracked inputs only).
- [x] **FIX** — declarative-tier: one shared shell helper + a tracked-register field + three gate
  rewires; no grammar, no `rust/src/*`, no `generated/*`, no Rust build.
- [x] **ADDRESSED (verified)** — before: `Provisional` computable by 0 gates, leg-3 criterion in 0
  gates, misalignment ⇒ 0-byte summary. After, measured by the re-runnable oracle
  `bash docs/tasks/artifacts/done_bar/run_family_status_bar_probes.sh` → **23/23 arms pass**
  (capture `family_status_bar_probes.txt`): vhdl replay `computed 'Provisional (corpus pending)'
  but tracker says 'Done'` with `vhdl_status: Provisional (corpus pending)` IN the emitted
  summary; svpp likewise; sv `Mostly Done` aligned; regex reproduces run-3's `computed 'In
  Progress'` failure now WITH a non-empty summary pair; CTRL-6 proves `met=true` on a valid
  declared surface.
- [x] **NO REGRESSION** — no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` staged ⇒ all 11
  generated parsers byte-identical BY CONSTRUCTION; `bash scripts/audit_done_bar.sh` re-run
  **byte-identical** to the tracked capture (`diff` = 0 lines) over the extended register;
  `.1` probe arms `bash docs/tasks/artifacts/done_bar/run_done_bar_probes.sh` → **11/11**;
  `bash -n` clean on every edited script; `bash scripts/check_doctrines.sh` → ALL PASS.
- [x] **LOCKSTEP** — book (`docs/book/src/quality-and-closure-model.md` — "The family-status gates
  compute the bar"), tracker note (⛔ no row moves — `.2b` owns that), `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `MEMORY.md`, this tree. No release / schema / ledger / contract movement:
  no parser behavior changed.

#### `.2b` — move the rows (`todo`, blocked on `.2a`)

- Only after `.2a` can a demotion be *recorded* rather than *asserted against the instruments*.
- ⭐⭐ **THE AUDIT CHANGED `.2`'s SHAPE. It was written expecting to move `regex`; it must move all
  five, and three of them for reasons that were not on the table when this leaf was written:**

  | row | unmet legs | qualifier the audit derives | what `.2` must state |
  |---|---|---|---|
  | `regex` | 1, 2, 3 | `Provisional (corpus pending)` | residual target debt 31 ≠ 0 (already adjudicated); **and** no external-corpus lane that anything runs |
  | `vhdl` | 3 (1-2 UNPROVEN) | `Provisional (corpus pending)` | its corpus proof is a TRIAGE gate over **0.058%** of the vendored corpus |
  | `systemverilog_preprocessor` | 3 (1-2 UNPROVEN) | `Provisional (corpus pending)` | **no external corpus at all**, though IEEE 1800 cl. 22 defines the language |
  | `return_annotation` | 2, 3 (1 UNPROVEN) | `Provisional (ceiling)` | leg 3 unreachable by construction — but its named `Done` gate is an **ORPHAN** and nothing computes its status |
  | `rtl_frontend` | 2, 3 (1 UNPROVEN) | ⛔ **cannot be issued yet** | language ownership is UNADJUDICATED (`.3` owes it); ⛔ `ceiling` may **not** be taken by default |

- ⛔⛔ **`rtl_frontend` BLOCKS ON `.3`, AND THAT IS THE RULE WORKING.** It is a subset PGEN delimits
  of an IEEE-standardized language. `(ceiling)` would close the row; `(corpus pending)` would keep it
  open. The tree's own rule — *"the comfortable label is the one that closes the row"*, *"absence of
  search is not absence of existence"* — forbids defaulting. ⇒ `.2` demotes it off `Done` with the
  qualifier **withheld and named as owed**, rather than picking the convenient one. Same for
  `rtl_const_expr` when its row next moves.
- ⭐ **A SECOND, CHEAPER LEVER FALLS OUT, AND IT IS NOT LEG 3.** `return_annotation` and
  `rtl_frontend` fail leg 2 because **nothing computes their status** — no `*_parser_family_status_gate`
  exists for either. That is a missing gate, not a missing corpus: strictly less work than `.3`, and
  it is what makes their rows *checkable at all*. Recommend `.2` route it to a new leaf rather than
  leaving it inside the demotion note.
- Each demoted row states which leg is unmet and what would close it. The dated historical notes are
  left intact; the **current** row is what moves ([[feedback_done_bar_is_first_tier_only]]).
- Each demoted row states which leg is unmet and what would close it. The dated historical notes are
  left intact; the **current** row is what moves ([[feedback_done_bar_is_first_tier_only]]).
- ⭐⭐⭐ **ONE ROW IS ALREADY ADJUDICATED AND WAITING — `regex`** (2026-07-29 session #222,
  `PGEN-REGEX-PCRE2-0051`; full evidence in `docs/tasks/REGEX-PCRE2-FIDELITY.md` `ROUTED-IN-2`).
  `regex_parser_family_status_gate` computes `In Progress` against a `Done` tracker row
  (`final_targets == 0` required, **31** measured). The pending adjudication — genuine coverage debt
  vs scope drift — is **settled: scope drift, measured**. `grammars/regex.ebnf` went **88 → 276
  productions (+214%)** since the claim; **9 of the 10** named residual rules did not exist then; and
  the one that did, `backreference`, went from **2 branches to 7+**, so *no* residual corresponds to
  a target that was in scope. ⛔⛔ **That does NOT defend the row — it is the definition of a stale
  claim**, and this tree's own rule decides it: *`Done` is not a snapshot; a grown universe must be
  re-earned.* The two readings were filed as having *opposite* fixes; they **converge**. ⇒ `regex`
  moves off `Done` when `.2` runs, unmet **leg 1** (residual target debt 31 ≠ 0), and the row's
  qualifier is **`Provisional (corpus pending)` at best** — it cannot be `(ceiling)`, since PCRE2 is
  precisely an external standard with a recognized reference implementation, and
  `regex_corpus_bundle/` already vendors it.
- ⭐ **Sequencing consequence, measured:** `CI-PARITY-GATE-ROT.7`'s next `sota_exit_gate` acceptance
  run would burn **5 hours** to re-confirm this same blocker. The gate is right and the tracker is
  stale ⇒ **`.2` should land before that run**, or the run is waste.

### `.3` — close the external-corpus gap (`todo`)

#### `.3a` — wire ANVIL as the independent generator for `rtl_frontend` / `rtl_const_expr` (`in progress`)

- **Director instruction (2026-07-29):** *"we can use ANVIL"* + *"you will need to create a git
  submodule for it. Here is the github link https://github.com/rdje/anvil"*.
- **Why it qualifies** (full assessment: [[feedback_no_corpus_means_raise_coverage]]): ANVIL is a
  random by-construction generator of **synthesizable** SystemVerilog whose validity model is
  **architecturally independent of any grammar** — its book chapter *"Why Not a Grammar?"* records
  that an annotated-EBNF walk was considered and **rejected** for circuit-cone recursion ⇒ it cannot
  inherit `grammars/rtl_frontend.ebnf`'s blind spots. It is anchored to Yosys/Verilator/Icarus/slang,
  ships **expected-facts answer-key manifests**, and is **byte-reproducible per `(seed, knobs)`** so a
  vendored snapshot is re-derivable rather than an opaque blob. Its own `CODEBASE_ANALYSIS.md:243`
  states Phases 7-9 were delivered for *"the user's `rtl_const_expr` / `rtl_frontend` style request"*
  (`anvil --artifact <dut|microdesign|frontend>`).
- **Placement decision:** `stimuli/generators/anvil`. ⛔ **NOT** `stimuli/sv/subs/` — all 23 existing
  submodules there are **vendored corpora**, and ANVIL is a **generator**. Filing a generator among
  the corpora would invite exactly the category error this tree keeps finding (a triage sample read as
  a corpus proof). A separate `generators/` bucket makes the distinction self-documenting, and it is
  language-agnostic because ANVIL's lanes serve two PGEN families.
- ⛔ **SEQUENCED BEHIND A CALIBRATION CONTROL, NOT AHEAD OF IT.** Before ANVIL backs any leg-3 claim
  it must **reproduce the `**` gap already measured** (0 occurrences in the grammar;
  `logic [7:0] ram [2**8-1:0];` rejected here, accepted by the full-LRM parser). *A corpus that cannot
  find a bug we know is there cannot certify the absence of bugs* —
  [[feedback_instrument_needs_ground_truth]] applied to a corpus instead of an instrument.
- ⛔ **AND IT DOES NOT SATISFY LEG 3 AS THE BAR IS WORDED** (*"officially-recognized third-party
  corpus"*): ANVIL is not third-party. Either the bar is amended deliberately by the director, or what
  ANVIL closes is named as something else. **It must not be quietly reinterpreted** — silent
  relabelling is the failure this tree exists to stop.
#### `.3a` FIRST-CONTACT RESULTS (2026-07-29, `PGEN-DONE-BAR-0008`)

Submodule added at **`stimuli/generators/anvil`**, pinned **`ecda0e78`**. ⛔ **Placed in a new
`generators/` bucket, NOT `stimuli/*/subs/`** — all 23 existing submodules there are *vendored
corpora*; ANVIL is a *generator*, and filing it among the corpora would invite the exact category
error this tree keeps finding (a triage sample read as a corpus proof).

| lane | rtl_frontend result | structural diversity |
|---|---|---|
| `--artifact frontend` (built FOR this) | **10 / 10 parse** | **10 distinct shapes / 10 seeds** |
| `--artifact dut` (the rich lane), default knobs | **0 / 5** | 5 / 5 |
| `--artifact dut`, case-family knobs at 0 | **2 / 5** | — |

⭐⭐ **ANVIL FOUND A REAL GAP ON FIRST CONTACT — and it is the second confirmed `rtl_frontend` gap.**
`case`/`endcase` have **0 occurrences** in `grammars/rtl_frontend.ebnf`; a `case` inside
`always_comb` is REJECTED by `rtl_frontend` and **ACCEPTED by the full-LRM `systemverilog` parser`**
⇒ valid, unambiguously synthesizable SV the parser cannot read. One DUT artifact uses `case` **25
times**. ⭐ This is **stronger than the calibration control demanded**: the bar was *"reproduce the
known `**` gap"*, and ANVIL instead surfaced one that was **not already known**. A corpus that only
reproduces known bugs adds nothing.

⛔⛔ **AND THE FIRST KNOB TURN IS THE CIRCULARITY TRAP, LIVE.** Zeroing `case_mux_prob` /
`casez_mux_prob` / `priority_encoder_prob` moves the DUT lane 0/5 → 2/5. **That must NOT be how the
corpus is configured.** `case` is core synthesizable RTL; silencing the generator to raise the pass
rate would hide a real gap behind a green number. ⇒ **the rule for knob selection: a knob may be
zeroed only where the construct is outside `rtl_frontend`'s DECLARED subset — never because zeroing
it makes the parser pass.** That distinction is the whole of ANVIL's value and it needs a mechanical
guard, not a promise.

#### ⛔ CORRECTION (2026-07-29, `PGEN-DONE-BAR-0009`) — ANVIL DID NOT MISBEHAVE; I DISCARDED ITS DIAGNOSTIC

The note below says a partial `--config` made ANVIL emit **0 bytes**. That is true and **materially
incomplete**, and as written it could be read as an ANVIL defect. **Re-measured with stderr kept:**

```
$ anvil --artifact dut --seed 0 --config partial.json
exit=1
Error: missing field `seed` at line 1 column 42
```

⇒ **ANVIL exited 1 and named the exact problem, at the exact column.** My invocation used
`2>/dev/null` and never checked the exit code, so I threw the diagnostic away and then scored the
empty output. ⭐ **The fault was 100% mine, and the tool's error handling was exemplary.** The
documented contract is unambiguous too (`knobs.md` "Knob serialization": `--dump-config > knobs.json`
then `--config knobs.json`) — a full config, not a partial one.

⭐ **THE REAL LESSON IS ONE LEVEL UP FROM "READ THE DOCS": I SUPPRESSED THE CHANNEL THE ANSWER CAME
ON.** A probe that redirects stderr to `/dev/null` and ignores `$?` has blinded itself to every
diagnostic the tool offers, and will then attribute its own mistake to the tool. ⇒ **a probe must
capture stderr and check the exit code before scoring anything** — and it must never report a defect
against another project without re-running with diagnostics kept.

✅ **VERIFIED POSITIVE, so the record is even-handed:** ANVIL's non-negotiable reproducibility
guarantee holds — byte-identical output for the same `(seed, knobs)` across **all three lanes**
(`dut` 108,771 B, `frontend` 1,148 B, `microdesign` 682 B), with a control confirming different seeds
genuinely differ. **No ANVIL defect was found in this slice.**

⚠️⚠️ **THREE PROCESS FAILURES IN THIS SLICE, RECORDED BECAUSE THEY ARE THE LESSON.** (1) I ran ANVIL
with default knobs and drew a conclusion about the DUT lane **before reading its documentation** —
the director had to say so twice. (2) I hand-wrote a partial `--config` JSON instead of the
documented *dump → edit → replay* flow; ANVIL emitted **0 bytes**, and my checker scored **five empty
files as PASS**. ⭐ **A vacuous green, in my own probe, in the very session whose subject is vacuous
greens** — caught only because the loop printed byte counts. The probe now refuses to score a file
under 100 bytes. (3) I suspected the frontend lane was one template with randomised constants; the
digit-stripped shape hash **refuted** it (10/10 distinct). ⇒ **read the tool's docs before measuring
it, and never let a checker score an artifact it has not confirmed is non-empty.**

⭐ **WHAT READING THE DOCS CHANGED.** ANVIL exposes **91 knobs**, and its capability gating is
rules-first by architecture: `--sv-version <2012|2017|2023>` is a *valid-by-construction capability
gate* that **down-gates** (targeting a lower standard, the emitter never emits a newer construct),
and every block knob is gated at construction time rather than generate-then-filter. ⇒ **scoping
ANVIL's output to a declared subset is a first-class supported operation, not a hack** — which is
what makes the case-(b) "partition" idea unnecessary here: ANVIL can be *told* to stay in the subset
at generation time. The book also ships a recipe for exactly this use — *"I want to test my parser
only, not synthesis"* (crank structural diversity: `--max-depth 8 --max-width 64 --count 1000`).

- **Steps:** (1) add the submodule ← *this slice*; (2) run the calibration control; (3) measure which
  `rtl_frontend` constructs ANVIL actually emits (currently **unmeasured**); (4) build the guarded
  consumption lane — with circularity as a **mechanically guarded** non-goal, since tuning ANVIL to
  emit only what PGEN accepts would destroy the independence silently.

#### ⭐⭐⭐ `rtl_frontend` MEASURED AGAINST REAL RTL FOR THE FIRST TIME (2026-07-29, director question, `PGEN-DONE-BAR-0006`)

The director asked the right question, and it is not the one this leaf was framed around:
*"is `rtl_frontend` sufficiently tested? Is it ok to release it as is?"* — **not** which
`Provisional` qualifier the taxonomy assigns. Measured
(`docs/tasks/artifacts/done_bar/run_rtl_frontend_real_world_probe.sh`):

| what | measured |
|---|---|
| the proof surface `Done` currently rests on | **130 curated samples, 35,123 bytes total**, mean **270 B**, largest **687 B** — 98 accepts / 32 rejects, **every one authored by this project** |
| real vendored design files parsed | **2 of 20** (`dmi_mux.v`, `dmi_wrapper.v` — the two smallest, simplest muxes in VeeR-EL2) |
| a genuine subset gap, with a cross-parser control | **`**` (power operator) has 0 occurrences in `grammars/rtl_frontend.ebnf`**; `logic [7:0] ram [2**8-1:0];` is REJECTED by `rtl_frontend` and **ACCEPTED by the full-LRM `systemverilog` parser** ⇒ valid SV, not invalid input |
| a CORRECT rejection, not a gap | **`initial` has 0 occurrences** — deliberately out of a *synthesizable* subset ⇒ part of the 18 rejections is legitimate |

⭐⭐ **THE ANSWER TO THE DIRECTOR'S QUESTION: not "it is bad" — "the question had never been asked."**
No gate points `rtl_frontend` at real RTL; this probe is the first time it met a design file. It
holds `Done` on 35 KB of examples this project wrote itself, and on first contact with real
hardware source it parsed the 2 simplest files of 20. ⛔ **Recommendation: it is NOT `Done`, and it
should NOT be released as `Done`.** It is usable — `Provisional` — provided the published boundary
says what is actually proven: *validated against 130 curated samples totalling 35 KB; never
validated against any third-party RTL.*

⭐ **AND THIS SETTLES THE QUALIFIER BY MEASUREMENT, NOT TAXONOMY: `(corpus pending)`, never
`(ceiling)`.** `ceiling` requires that no corpus exist. **16,388 real SV/V files are already
vendored in this repository** under `stimuli/sv/subs/` — the corpus was never absent, it was never
pointed at this parser. *"We could not find a corpus"* was never true here.

⛔⛔ **WHAT IS NOT ESTABLISHED, STATED SO IT IS NOT INFERRED:** the **split** between deliberate
subset boundaries (`initial`, testbench constructs) and genuine gaps (`**`). A rejection rate cannot
be read as a defect count. Attributing per-file causes is `.3`'s work and cannot be guessed.

⚠️ **THREE FALSE FINDINGS WERE PRODUCED AND WITHDRAWN WHILE MEASURING THIS — recorded because the
withdrawals are the method working.** (1) *"0 of 20 parse"* — an unsorted `find` sampled a different
file set; sorted, it is **2 of 20**. (2) *"a module with no port list is rejected"* — the input
`module top` + `endmodule` is **invalid SV** (no semicolon); the parser was right. (3) *"unpacked
memory arrays are rejected"* — `logic [7:0] ram [255:0];` **parses**; the rejection was the `**`
inside the bound. ⭐ Each was killed by a **cross-parser control** — *does the full-LRM parser accept
this same input?* — which is now built into the probe. **Without that control a rejection cannot be
told apart from invalid input**, and all three would have shipped as defects.


- **Status: `todo`** — blocked on `.1`'s per-family leg-3 result.
- Likely the largest body of work in this tree: acquiring/vendoring an officially-recognized corpus
  for families that have none, and promoting `triage`/`characterization` surfaces into asserted
  conformance where the parser can actually meet them.
- ⚠️ Where a family genuinely cannot have an external corpus, that needs a **recorded,
  director-visible justification** — absence is an unmet leg, not an inapplicable one.

### `.4` — THE GUARANTEE: the flow enforces the bar, so a false `Done` cannot be held (`todo`)

- **Status: `todo`** — the tree's **highest-value leaf** per the director's *"the flow shall guarantee
  this 100%"*; sequenced after `.1`/`.2` only because enforcement needs the audit's shape first.
- **Requirement:** a `Done` row that does not meet all three legs makes a gate FAIL. Not a report, not
  a warning — the `GATE-REACHABILITY` / `FLOW-INTEGRITY` ratchet shape, which this repo has now
  shipped twice and which is the worked template.
- ⛔ **Prerequisites, measured and named above, not discovered later:** `CI-PARITY-GATE-ROT.7` (the
  aggregate must be able to go green) and the escalated hosted-auto-trigger call (`.6`) — without an
  automatic lane, the enforcement is itself in the OPERATOR tier, which is the exact rot `.8` was
  opened to end (*the flow had been fixed with checks that could themselves rot*).
- ⚠️ State the residual honestly: hook-bypassable + machine-local unless a CI lane re-proves it.
  ⛔ Price before mechanizing (`GENERATED-LINT-CORRECTNESS.4`'s rule).

### `.5` — the missing CONSUMER-FACING gates: disclosure integrity (`todo`) ⭐ PREREQUISITE FOR SHIPPING `Provisional`

- **Status: `todo`** — ⛔ **PROMOTED 2026-07-29.** It was recorded as *"safely deferred, not on the
  critical path"* because no family reached it. **That reasoning does not survive the `Provisional`-ships
  directive**: if customers decide from published state, the published state must be true, and it is
  measurably not (guide `1.1.29`/`1.1.31` vs contract `1.1.104`/`1.1.106`, no gate reading either).
  *A disclosure nobody checks is a claim, not a disclosure.*
- **Four gates that do not exist today** (measured: no gate references the ledger or the user guide):
  1. **no reachable silent-success path** for the family — 6 sentinel emission sites measured
     (`<property_access>` / `<array_access>` / `<last_extraction>`), where a parse returns `Ok` with
     zero diagnostics and a placeholder node ⇒ green gates, garbage handed to the consumer;
  2. **published version currency** — the user-facing docs' version pair must match the released
     parser (measured drift: guide `1.1.29`/`1.1.31` vs contract `1.1.104`/`1.1.106`);
  3. **zero open ledger entries** naming the family;
  4. **documented acceptance boundary** where a consumer looks.
- ⭐ Once these exist they are ordinary leg-2 gates. That is the point: **the bar stays three legs and
  stays fully mechanizable**, which is what *"the flow shall guarantee this 100%"* demands.

### `.6` — publish each family's bar state where a consumer looks (`todo`)

- **Status: `todo`** — the disclosure surface the `Provisional`-ships model requires.
- **Requirement:** every family's downstream integration contract (`docs/contracts/PGEN_*_PARSER_INTEGRATION_CONTRACT.md`,
  9 of them) and its per-parser mdBook state, in a fixed place: the family's tier (`Done` /
  `Provisional` / lower), which legs are met, **what is therefore unproven**, and — for `Provisional` —
  **which qualifier applies** — `(ceiling)` = finished, leg 3 unreachable by construction; `(corpus pending)` =
  a recognized corpus exists and wiring it is outstanding. ⭐ A consumer reading a contract must be able to
  tell *"this is as good as it gets"* from *"this is not finished yet"* — that distinction is the single
  most decision-relevant fact on the page, and an unqualified `Provisional` withholds it.
- ⭐ Machine-readable too, not only prose: the family status gates already compute a status string, so
  the tier and its evidence should be emitted in their `summary.json` for a downstream to check
  mechanically rather than parse marketing text.
- ⛔ Gated by `.5`, or the disclosure can go stale exactly like the version pair did.

## Current Frontier

**`.2b`** — move the rows. ✅ **UNBLOCKED by `.2a` (`PGEN-DONE-BAR-0010`)**: the three family-status
gates now COMPUTE the qualified `Provisional` and carry the leg-3 criterion, and the replays against
the run-3 artifacts already state the statuses the tracker must adopt — `vhdl` and
`systemverilog_preprocessor` ⇒ `Provisional (corpus pending)`, `regex` ⇒ `In Progress` (legs 1-2
unmet, so it sits BELOW `Provisional` until its 31-target debt closes;
`REGEX-PCRE2-FIDELITY.ROUTED-IN-2` owns that), `systemverilog` unchanged `Mostly Done`.
⚠️ The status gates (and their contract gates) are deliberately RED against today's tracker until
`.2b` lands — the gate states the truth, then the tracker agrees. ⛔ `rtl_frontend` /
`rtl_const_expr` qualifiers remain WITHHELD pending `.3` (their rows are demoted off `Done` with the
qualifier named as owed). ⭐ Near-term GOAL: **every family to at least `Provisional`**. The tree's
PURPOSE is `.4` (the flow guaranteeing the bar); `.5`/`.6` (disclosure integrity) are prerequisites
for shipping `Provisional` honestly.

⭐ **Sequencing, measured and unchanged:** `.2` should land **before** `CI-PARITY-GATE-ROT.7`'s next
`sota_exit_gate` acceptance run, or that run burns ~5 h to re-confirm a known, routed, unfixed
blocker (`regex` tracker alignment).

## Blockers

⛔ **`.4` — the guarantee itself — is BLOCKED on `CI-PARITY-GATE-ROT.7` (aggregate green) and on the
escalated director call about resuming hosted auto-triggers.**
⛔ **`.2` is blocked ONLY for `rtl_frontend`'s and `rtl_const_expr`'s qualifier**, which `.3` owes;
the demotions themselves are not blocked.

## Verification Log

### `.1` — the audit (2026-07-29, session #223, `PGEN-DONE-BAR-0002`)

| instrument | command | result |
|---|---|---|
| the audit | `bash scripts/audit_done_bar.sh` | exit **1** — 7 families derived, 5 `Done`, **5/5 do not meet the bar**; 10 ground-truth controls reproduced |
| probe arms | `bash docs/tasks/artifacts/done_bar/run_done_bar_probes.sh` | **11/11** — 6 RED arms all flip (refuse / MISCALIBRATED), 5 CTRL arms all hold |
| corpus scale | `bash docs/tasks/artifacts/done_bar/run_corpus_scale_census.sh` | `vhdl` 8 cases / 13,720 files = **0.058%**; `systemverilog` 7 / 16,388 = **0.043%**; regex PCRE2 **2,189** cases wired to nothing |
| ledger | `bash docs/tasks/artifacts/done_bar/run_ledger_open_census.sh` | 168 rows, **0 open** (the `24` was a prose token tally — hypothesis REFUTED) |
| doctrines | `bash scripts/check_doctrines.sh` | **ALL 13 PASS** |
| byte-identity | *(by construction)* | no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*`, no `rust/scripts/*` staged ⇒ all 11 generated parsers unchanged |

**Probe arms, each asserting an exit code AND a substring of the message it expects** (comparing only
pass/fail was measured to hide arms reaching the right verdict for the wrong reason,
`CI-PARITY-GATE-ROT.4`):

| arm | injected defect / healthy state | must produce |
|---|---|---|
| CTRL-1 | untouched tree | a VERDICT (exit 1), never a refusal |
| CTRL-2 | a `Mostly Done` row | reported as context, **not** counted as a failing `Done` row |
| CTRL-3 | a tracker row with a backticked NON-grammar token | roster stays 7 — the `gate-level` trap replayed |
| CTRL-4a/b | ground truth for the instrument | `regex`'s failure recovered verbatim; `vhdl`'s lane reported TRIAGE |
| RED-1 | a family on the tracker, absent from the register | **REFUSE** (exit 2) — a skip would let a new `Done` row hide |
| RED-2 | an empty derived roster | **REFUSE** — not exit 0 (the vacuous-green class) |
| RED-3 | longest-prefix attribution broken | **MISCALIBRATED** via C4 |
| RED-4 | status-gate coverage lost (this instrument's own defect) | **MISCALIBRATED** via C9 |
| RED-5 | a MENTION counted as an INVOCATION | **MISCALIBRATED** via C7 |
| RED-6 | the register contradicted by the derived gate set | **MISCALIBRATED** — the register is checkable, not trusted |

⚠️ **HONEST LIMITS, stated in the audit's own output rather than discovered later.**
(1) The audit **never runs a gate**, so leg 1 and "green NOW" rest on whatever artifacts exist on the
machine; where none exist it prints `UNPROVEN` and the command that would produce one. (2) Its
"actually invoked" ceiling is **OPERATOR** for every family, because the AUTOMATIC tier over all 123
`make` gate targets is **zero** — so leg 2 can never read better than *runs when a human asks*.
(3) `scripts/audit_done_bar.sh` is a proof-surface script that the `TASK-ACCEPTANCE` code-change glob
(`scripts/check_*.sh`) does **not** cover; the checklist above is carried **voluntarily**. That gap
closes when `.4` registers the enforcement check as `scripts/check_done_bar.sh`.

## Commit Log

- `PGEN-DONE-BAR-0010` (2026-07-29, session #224, leaf `.2a` done) — the three family-status gates
  now compute the NEW bar through one shared helper (`rust/scripts/lib/parser_family_status_bar.sh`):
  qualified `Provisional` in the computable vocabulary (qualifier derived from register language
  ownership, `unadjudicated` REFUSES), the leg-3 criterion `external_corpus_conformance_pass`
  (conformance-not-triage / external-backed / actually-invoked, met only by a register-declared
  surface whose artifact passes), `Done` unreachable while leg 3 is unmet, and misaligned gates now
  emit their full summary pair BEFORE exiting 1 (the 0-byte-summary fix). Probes 23/23
  (`run_family_status_bar_probes.sh`); replays: vhdl + svpp now compute `Provisional (corpus
  pending)` against run-3 artifacts. Audit re-run byte-identical; `.1` arms 11/11. No row moved —
  `.2b` next.
- `PGEN-DONE-BAR-0001` (2026-07-29, session #221) — tree opened on the director's standing
  directive; decision record `feedback_done_bar_is_first_tier_only.md`; measured starting point
  recorded as a hypothesis, not a finding.
- ⚠️ **SLICE-ID COLLISION, CORRECTED FORWARD AND LEFT VISIBLE.** Commit **`94454e30`** (leaf `.1`
  done, this session) was labelled `PGEN-DONE-BAR-0002` — an id already consumed by **`598038a8`**
  (session #221, *"leaf DONE-BAR.5 opened"*). History is deliberately **not** rewritten (COMMIT.md
  forbids destructive git operations without an explicit request, and the correction belongs in the
  record rather than behind it): **refer to the `.1` slice by its SHA `94454e30`.** This session's
  subsequent slices resume at `0005`, since `0003`/`0004` were also consumed in session #221.
  ⇒ **ROUTED: nothing checks slice-id uniqueness.** `.githooks/commit-msg` requires an
  identifier-shaped work-unit id in the subject and never asks whether that id is already taken —
  a one-line structural check over `git log` would have blocked this. Recorded as a candidate
  doctrine; ⛔ price it against the whole corpus before mechanizing
  (`GENERATED-LINT-CORRECTNESS.4`'s rule — this is the first known occurrence, which is below that
  bar today).
- `PGEN-DONE-BAR-0006` (2026-07-29, session #223, `.3` seeded with measurement) — on the director's
  question *"is `rtl_frontend` sufficiently tested? Is it ok to release it as is?"*: pointed it at
  real vendored RTL for the first time. **2 of 20** real design files parse; its `Done` rests on
  **130 curated samples totalling 35 KB** that this project authored. `**` is a genuine gap (0
  occurrences in the grammar, rejected here, accepted by the full-LRM parser); `initial` is absent
  BY DESIGN. ⇒ recommend NOT `Done`; `Provisional (corpus pending)` — settled by measurement, since
  **16,388 real SV/V files are already vendored in this repo**, so `(ceiling)` was never available.
  Three false findings produced and withdrawn en route; the cross-parser control that killed all
  three is now built into the probe. Also records the append-only git-history directive
  ([[feedback_git_history_is_append_only]]).
- `PGEN-DONE-BAR-0005` (2026-07-29, session #223, `.2` SPLIT into `.2a` → `.2b`) — measured, by
  replaying the LIVE alignment logic, that an honest demotion would turn **3 of 3** family-status
  gates RED, two of which pass today: `Provisional` appears in **zero** gate scripts, the gates carry
  no leg-3 criterion, and each compares tracker-vs-computed by exact string equality then `exit 1`.
  ⇒ the instruments must learn the new bar BEFORE the tracker states it, or the demotion
  manufactures a disagreement instead of recording the truth. The probe's own first cut reported
  `✅ aligns` from empty strings and now refuses on an empty side.
- `PGEN-DONE-BAR-0002` (2026-07-29, session #223, leaf `.1` done) — the audit exists and **5 of 5
  `Done` rows fail it**. The tree's own first-pass reading is corrected in `regex`'s disfavour (its
  external-corpus lanes run by nothing; the lane that runs reads a 44-case repo fixture);
  `vhdl`'s corpus proof priced at **0.058%** of its vendored corpus; two `Done` families found to
  have **no family-status gate at all**; the ledger hypothesis resolved to **0** open entries.
  Two defects in the instrument's own first cut caught by controls and left visible.
