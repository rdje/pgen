# DONE-BAR: `Done` means SOTA / SIGNOFF-LEVEL done — a user can blindly trust the parser: stimuli-generator proof AND all gates AND all external corpus, and the trust that establishes

## Metadata

- Tree ID: `DONE-BAR`
- Status: `active` (opened 2026-07-29, session #221)
- Family / slice-id prefix: `PGEN-DONE-BAR-<NNNN>`
- Created: `2026-07-29`
- Owner: repo-local workflow
- Director directive: [[feedback_done_bar_is_first_tier_only]]
- Frontier: **`.5c`** (the silent-success sentinel gate — the last unbuilt `.5` gate before `.5d`/`.6`) — `.5b` DONE 2026-07-29 (`PGEN-DONE-BAR-0013`, the ledger criterion in the status gates); `.5a` DONE 2026-07-29 (`PGEN-DONE-BAR-0012`, the 14th enforced doctrine); `.2a`+`.2b` DONE 2026-07-29 (`PGEN-DONE-BAR-0010`/`-0011`); `.1` DONE 2026-07-29 (`PGEN-DONE-BAR-0002`); `.3a` (ANVIL) remains `in progress`

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

### `.1a` — the audit reported a SUPERSEDED gate failure as a current verdict (`done`, 2026-07-30 session #227, `PGEN-DONE-BAR-0022`)

- **Found by USING the instrument, not by being told**: while pulling the live-status snapshot for the
  `.5d` commit report, `bash scripts/audit_done_bar.sh` printed, three lines apart:

```
regex   tracker: In Progress   (not a `Done` claim — reported for context)
  ⛔ regex_parser_family_status_gate RAN AND FAILED (log …): error: regex tracker alignment
     mismatch: computed 'In Progress' but tracker says 'Done'
```

  ⇒ the audit contradicted itself: it read the tracker as `In Progress` and, immediately below, quoted
  a gate error complaining the tracker says `Done`.

##### ROOT CAUSE — an asymmetry, not an omission, and the audit's own comment describes it

- `scripts/audit_done_bar.sh` has **two** ways to learn a family-status gate's verdict:
  - the **artifact** path (`find_artifact` → a `summary.*`), which at `:427` applies
    `mtime < newest_input_mtime(family)` and reports `⚠️ STALE — older than its own inputs`, with the
    rationale *"a verdict resting on an artifact older than the inputs it judged is not proof"*;
  - the **failed-gate** path (`gate_ran_and_failed`, `:275-289`), which read the log's first `error:`
    line and returned **no vintage at all**.
- ⭐⭐ **The unguarded branch is the one MOST exposed to staleness**: a gate that failed wrote no
  summary, so the only evidence left is a log of arbitrary age. And the leg-2 site's own comment
  already states the governing rule — *"The status gates assert tracker alignment, so a tracker edit
  after the run leaves that alignment unproven even when the recorded answer was `true`"* — which is
  **exactly as true of a recorded FAILURE as of a recorded pass**, and was applied to only one of them.
- **MEASURED instance:** the quoted log's mtime is **2026-07-29 13:49** (epoch `1785325790`);
  `DONE-BAR.2b` (`b704e1ab`, 2026-07-29 **18:37:42**) moved that tracker row to `In Progress`
  **4 h 48 m later**. The complaint was TRUE when written and FALSE when read.

##### FIX

`gate_ran_and_failed` now returns `(rel, error, mtime)` and **both** call sites apply the artifact
path's staleness rule: a failure older than its own inputs is reported WITH its vintage, the recorded
error is still quoted (it is evidence, just not current), and the verdict becomes **UNPROVEN, not
UNMET** — *"the failure may already be fixed; re-run the gate"*. ⭐ Downgrading rather than dropping
keeps the refusal polarity the audit is built on: `UNPROVEN` does not satisfy the bar either, so no row
can be promoted by this change.

##### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `bash scripts/audit_done_bar.sh` printed `regex   tracker: In Progress`
      and, 3 lines below, `⛔ … error: regex tracker alignment mismatch: computed 'In Progress' but
      tracker says 'Done'` — a self-contradiction in one report.
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family: `grep -n 'gate_ran_and_failed'
      scripts/audit_done_bar.sh` → definition `:275` returning a 2-tuple with no mtime, consumed at
      `:417` (leg 1) and `:467` (leg 2), while the sibling artifact path at `:427` computes
      `mtime < newest_input_mtime(family)` and reports staleness. Vintage measured with
      `os.path.getmtime` = `1785325790` (2026-07-29 13:49) against `git log --format=%ad` for
      `b704e1ab` = 2026-07-29 18:37:42 ⇒ the artifact predates its own input by 4 h 48 m.
- [x] **FIX** — declarative tier: return the mtime and apply the rule the file already contains at
      both call sites. No new mechanism, no new dependency.
- [x] **ADDRESSED (verified)** — before→after **REPLAYED from `git show HEAD:`**, not described.
      AFTER: leg 1 reads `RAN AND FAILED (log …, mtime 1785325790, ⚠️ STALE — older than its own
      inputs)` + `that failure PREDATES its own inputs ⇒ UNPROVEN, not UNMET — re-run the gate`; leg 2
      reads `⚠️ … but that run PREDATES its own inputs` ⇒ `green-NOW is UNPROVEN, not UNMET`.
      `bash docs/tasks/artifacts/done_bar/run_audit_stale_failure_probes.sh` → **16 passed, 0 failed**
      after the `-0023` probe rebuild (it read 9/0/3-UNJUDGEABLE at `-0022`, on a driver with two
      defects of its own — see the correction below).
- [x] **NO REGRESSION** — ⭐ **CTRL-1 is the arm that matters**: the fix must not make the audit blind
      to REAL current failures. With the same log touched NEWER than every input, the verdict flips
      back to `⛔ … RAN AND FAILED` with **no** staleness claim on either leg — so a fresh failure is
      still UNMET. The audit still exits **0** on the live tree, control calibration still reproduces
      (no `MISCALIBRATED`), and `:371`'s `gate_ran_and_failed(_g)[1]` index-1 use is unaffected by the
      3-tuple. No `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` ⇒ all 11 generated parsers
      byte-identical BY CONSTRUCTION. `bash scripts/check_doctrines.sh` → ALL 14 PASS.
- [x] **LOCKSTEP** — `README.md` (the audit's advertised properties — corrected, see below), this
      tree, `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`. No release / schema / ledger / contract
      movement.

##### ⛔⛔ CORRECTION (2026-07-30, `PGEN-DONE-BAR-0023`) — MY OWN FIRST WRITE-UP OF THIS LEAF'S PROBE WAS WRONG

**What I first recorded here, and what is actually true:**

| first recorded | MEASURED |
|---|---|
| the `BEFORE-1` arms were `UNJUDGEABLE` because the retired script *"REFUSED … while `sota_exit_gate` run 4 was executing"* ⇒ **transient contention with a concurrent `make -C rust`** | ⛔ **WRONG.** `audit_done_bar.sh:44` derives `ROOT="$(dirname "${BASH_SOURCE[0]}")/.."` **from its own location**, and the probe ran a COPY from `rust/target/done_bar_audit/stale_failure_probe/` ⇒ `ROOT` resolved to `rust/target/done_bar_audit`, so the reachability precondition could not be found. **Decisive control: the byte-identical retired content run from `scripts/` exits 0, with no aggregate running.** |

⭐ **A DETERMINISTIC PATH BUG WAS MISDIAGNOSED AS FLAKINESS, AND THE "FIX" WAS A 3-RETRY LOOP** — i.e. I
papered over a reproducible defect by retrying it. The retry loop is **removed**; the retired script is
now staged inside `scripts/` so its `ROOT` resolves correctly. ⛔ The concurrency claim is **withdrawn**,
not softened: nothing measured supports it.

⭐ **SECOND PROBE DEFECT, exposed by run 4 itself:** the driver depended on a **REAL** failed-gate log as
its fixture. Run 4 made that gate PASS, overwriting the log, so the RED arms silently lost the failure
they existed to detect (4 passed / 5 failed on the next run). ⇒ **a probe whose fixture is a transient
artifact of the last run is not a probe** — the `.5f` custody lesson, now paid twice. The fixture is
**SYNTHETIC** and built by the driver (it also moves the real summary aside so the failed-gate branch is
reachable at all, restoring both from a trap), so the driver is reproducible on any tree.

⛔⛔ **AND THE ORIGINAL DEFECT IN THAT BLOCK STANDS AS RECORDED:** its first cut asserted an ABSENT string
and therefore **PASSED VACUOUSLY over a 171-byte refusal message** — an arm reaching the right verdict
for the wrong reason (`CI-PARITY-GATE-ROT.4`). Every arm now requires a real verdict before asserting.

**Probes after the rebuild: 16 passed, 0 failed, 0 unjudgeable** — and `BEFORE-1` now genuinely replays
the retired form, reproducing the self-contradiction verbatim (`tracker: In Progress` above an error
claiming the tracker says `Done`), which the earlier vacuous arms never demonstrated.

##### ⛔⛔ TWO FURTHER FINDINGS, ROUTED TO `.1b` RATHER THAN FIXED HERE

1. **The audit's advertised "no make" property is FALSE.** `README.md` described `audit_done_bar.sh` as
   *"read-only and cheap (no cargo, no make, no network)"*. But its hard precondition
   `scripts/check_gate_reachability.sh` executes
   `subprocess.run(["make","-C","rust","--no-print-directory","-s",f"print-{name}"], timeout=30)` at
   `:222-225` — so the audit **does** invoke `make -C rust`. Verified at the source, independently of
   anything else in this leaf; README corrected. ⛔⛔ **CORRECTED 2026-07-30 (`-0023`): the claim first
   written here — that this is *"why it is flaky"* and that the audit *"cannot be relied on while a
   `make -C rust` gate is running"* — is WITHDRAWN.** The refusal that motivated it was a path-depth bug
   in this leaf's own probe (see the correction above), and **no measurement supports a concurrency
   problem.** What remains is a documentation/behaviour drift, now fixed, plus finding (2).
2. ⭐⭐ **A LATENT SILENT-DEGRADATION worse than the flakiness**: that call is wrapped in
   `except Exception: _var_cache[name] = []` (`:227-228`) — a timeout or any failure yields an **EMPTY
   make-variable expansion, silently**. An empty prereq expansion **drops edges**, which is exactly
   calibration defect **(6)** from `CI-PARITY-GATE-ROT.2`'s six-wrong-answers list (*"a prereq list in
   a make variable"*), the one fixed by *"ASK MAKE, DO NOT RE-IMPLEMENT MAKE"*. ⇒ **the fix's failure
   path degrades back into the blindness it was written to cure.** Today that surfaced as a loud
   refusal (the ratchet noticed untriaged orphans), but if the lost edges touch only
   already-dispositioned orphans the census would come out **quietly different and still exit 0** —
   the reachability instrument's 8 ground-truth controls are the only thing standing between that and
   a wrong number.
   ⚠️ **NOT PROVEN, STATED AS AN INFERENCE:** the failing invocation's own log was overwritten by my
   later standalone re-runs before I read it, so *"the 30 s timeout fired"* is the strong reading, not
   a measurement. `.1b` must capture the failing invocation before diagnosing.

### `.1b` — the audit's reachability precondition swallowed its own make failures (`done`, 2026-07-30 session #227, `PGEN-DONE-BAR-0024`)

- **Status: `done`** (2026-07-30, `PGEN-DONE-BAR-0024`) — routed from `.1a` with the evidence above,
  and closed in the same session because it is the one item on that findings list that was a genuine
  open flow defect (`docs/decisions/feedback_flow_findings_are_routed_not_worked.md`).

##### THE FIX — refuse instead of degrading, and the measurement that made it safe

`scripts/check_gate_reachability.sh`'s `expand_make_var` was:

```python
_var_cache[name] = out.stdout.split() if out.returncode == 0 else []
except Exception: _var_cache[name] = []
```

⇒ a timeout, a missing `make`, or a real make error all yielded an **empty expansion**, dropping the
prerequisite edges the function exists to supply and turning reachable targets into "orphans".

⭐ **MEASURED BEFORE CHANGING THE BEHAVIOUR — this is what made refusal safe rather than a source of
false positives:** `make -C rust print-<not-a-variable>` exits **0** with empty stdout. So the
legitimate *"this token is not a make variable"* case **never raises and never returns nonzero** ⇒ an
exception or a nonzero exit can only mean a real failure, and there is **no legitimate empty expansion
to preserve.** Both paths now print what failed and `sys.exit(2)`.

##### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `sed -n '215,230p' scripts/check_gate_reachability.sh` at the pre-fix
      revision shows `except Exception: _var_cache[name] = []` plus the `else []` on a nonzero exit.
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family: `scripts/check_gate_reachability.sh:222-229`,
      `expand_make_var`, reached from `:238` for every make-variable prerequisite. `make -C rust
      --no-print-directory -s print-NOT_A_VARIABLE_XYZ` → **exit 0, empty output** (so the swallow has no
      legitimate job); `make -C rust print-RUST_DIR` → `.` (so the success path is real).
- [x] **FIX** — declarative tier, reusing the script's OWN refusal idiom (`print(...) ; sys.exit(2)`,
      as at `:414-420`) rather than inventing one.
- [x] **ADDRESSED (verified)** — before→after **REPLAYED under an induced failure** (a stub `make` on
      `PATH` that exits 127), not described:
      **AFTER** → `gate-reachability: cannot expand make variable 'PARSER_BOOK_GATES' (… exited 127 …)
      Refusing rather than deriving edges from an empty expansion`, **exit 2**, no inventory offered.
      **BEFORE** (retired form from `git show HEAD:`, same induced failure) → **exit 1** reporting
      `json_parser_book_gate: expected reachable, got ORPHAN` — ⭐⭐ **a WRONG orphan census blaming the
      repository for a defect that was actually "make could not run"**. ⭐ And it was caught only because
      a ground-truth control happened to cover that variable — the control's own text predicted this
      exact failure (*"if this regresses, the make-variable expansion of PARSER_BOOK_GATES has
      broken"*), so the old behaviour depended on luck.
- [x] **NO REGRESSION** — GREEN run byte-unchanged: `gate-reachability: OK (124 targets; 93 reachable,
      30 orphan + 1 policy-only, all dispositioned; 8 ground-truth controls reproduced)`, exit 0.
      `bash scripts/check_doctrines.sh` → ALL 14 PASS (this script IS one of the 14). No
      `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` ⇒ all 11 parsers byte-identical BY
      CONSTRUCTION.
- [x] **LOCKSTEP** — this tree, `docs/decisions/feedback_flow_findings_are_routed_not_worked.md` (new),
      `docs/decisions/INDEX.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`. `README.md` needed no
      further change — `.1a`/`-0023` already corrected the audit's advertised properties.
- **Scope (narrowed 2026-07-30 by `-0023`, after the concurrency premise was refuted):** decide whether
  the `except Exception: _var_cache[name] = []` swallow should **REFUSE** instead of degrading to an
  empty expansion, per this repo's own rule that *a check that cannot see must say so*. ⛔ Two items
  from the original scope are **retired, not deferred**: *"capture the failing invocation"* (the cause
  is known — it was a probe path bug, not the make call) and *"consider whether the audit should refuse
  while an aggregate runs"* (no concurrency problem was ever measured). `README.md`'s *"no make"* claim
  was corrected in `-0022`/`-0023`.
- ⭐ **The remaining item is the real one and it is latent, not observed:** no run has yet been seen to
  hit the timeout. Its danger is that if it ever fires, the census comes out **quietly different** and
  still exits 0 unless one of the 8 ground-truth controls happens to cover the lost edge.
- ⭐ **Why this matters more than a flaky script:** every `Done`/`Provisional` adjudication in this
  tree rests on this audit, and `GATE-REACHABILITY` is one of the 14 enforced doctrines. A silent
  empty expansion in its edge derivation is a defect in the instrument that decides whether other
  instruments exist.

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

#### `.2b` — move the rows (`done`, 2026-07-29 session #224, `PGEN-DONE-BAR-0011`)

- **Status: `done`.** All five contested rows moved, and the instruments agree with every one of
  them — the order `.2a` established (*the gate states the truth, then the tracker agrees*) held:

  | row | was | now | agreed by |
  |---|---|---|---|
  | `vhdl` | `Done` | **`Provisional (corpus pending)`** | `vhdl_parser_family_status_gate` ✅ aligned (replay green) |
  | `systemverilog_preprocessor` | `Done` | **`Provisional (corpus pending)`** | `sv_parser_family_status_gate` ✅ aligned (replay green) |
  | `regex` | `Done` | **`In Progress`** | `regex_parser_family_status_gate` ✅ aligned (replay green) — BELOW `Provisional` because legs 1-2 are unmet (`final_targets=31`; debt owned by `REGEX-PCRE2-FIDELITY.ROUTED-IN-2`) |
  | `return_annotation` | `Done` | **`Mostly Done`** | no instrument computes it (that IS its leg-2 failure) — see the adjudication below |
  | `rtl_frontend` | `Mostly Done` (was `Done`) | **`Mostly Done`** | no instrument computes it; qualifier WITHHELD as owed (`.3`), per the tree's own no-defaulting rule |

- ⭐⭐ **ADJUDICATION RECORDED — the `.2b` charter table's `Provisional (ceiling)` column entry for
  `return_annotation` is NOT taken, and the reason is the tracker's own vocabulary**: the Status
  Rules define `Provisional` as *legs 1 and 2 MET*; the audit scored return_annotation's leg 2
  **UNMET** (nothing computes its status; its README-named formal gate is an ORPHAN). Writing
  `Provisional (ceiling)` — *a FINISHED row* — over a red leg 2 would be taking exactly "the
  comfortable label that closes the row". ⇒ `Mostly Done`, with the row stating that once leg 2
  closes (the gate wired into an invoked lane + a computed status), `Provisional (ceiling)` is its
  finished state. The charter table's column header says "qualifier the audit derives" — ownership
  taxonomy, which stands — not "the status `.2b` writes".
- **Consumer surfaces corrected forward in lockstep** (a stale claim is the exact disclosure rot
  `.5` exists to gate): `README.md` (return_annotation "formal `Done` gate" prose + the
  rtl_frontend "LIVE row is `Done`" bullet), `docs/book/src/parser-families.md` (regex "still
  computes `Done`" bullet + the rtl_frontend closure paragraph).
- **Instrument updates owned by this leaf, each adjudicated not papered over:**
  - `scripts/audit_done_bar.sh` **C8** pinned a ground truth this demotion legitimately moved ("at
    least one `Done` row"). Replaced by C8′ — *every register family derives from the tracker* (the
    converse of the existing roster⊆register refusal; catches a status-vocabulary change silently
    dropping rows) — and the zero-`Done` case is now stated EXPLICITLY in the verdict ("VACUOUSLY
    green … NOT evidence of parser quality"), never a silent pass.
  - `.1` probe arms: CTRL-1 re-pinned to the vacuous zero-`Done` statement (exit 0); **new CTRL-1b**
    keeps the old arm's essence — a re-promoted unproven `Done` row still FAILS (`1 of 1`); CTRL-2/3/4
    exit expectations updated (the substance they pin is unchanged). **12/12 pass.**
  - `run_demotion_impact_probe.sh` gains a HISTORICAL guard: with the `| Done |` rows gone its
    mutation targets no longer exist, so it states that the demotion landed and exits 0 instead of
    MISCALIBRATING for a reason that is the fix working.
  - `run_family_status_bar_probes.sh` replay arms re-pinned to the ALIGNED steady state (**24/24**);
    the `.2a`-era transitional capture is preserved at commit `09634838`.
  - `sv_parser_family_status_gate`'s leg-3 unmet arm now keeps `details[].detail == unmet[]` —
    its contract sibling asserts that element-wise parity, which regex/vhdl's contracts do not
    (caught by running all three status-contract gates against the aligned replay summaries).

##### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — with `.2a` landed and the tracker unmoved, the replays showed the
  measured misalignment: `computed 'Provisional (corpus pending)' but tracker says 'Done'` (vhdl),
  the sv sibling for svpp, and regex's standing `computed 'In Progress' but tracker says 'Done'`
  (`.2a`-era capture, commit `09634838`).
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family: the five tracker rows asserted statuses
  their own instruments compute differently (`DONE-BAR.1` audit, exit 1, all five rows) — inputs
  all `git ls-files`-tracked; `bash -n` clean across every edited script; the sv contract-gate
  failure was traced with `bash -x` to its `details[].detail == unmet[]` jq assertion, the one
  consumer invariant the three contracts do not share.
- [x] **FIX** — declarative-tier: five tracker-row moves with dated demotion statements, two
  consumer-doc corrections, and the instrument adjudications above. No grammar, no `rust/src/*`,
  no `generated/*`.
- [x] **ADDRESSED (verified)** — before: 5 rows contested by their instruments (audit exit 1,
  5/5). After, measured by re-runnable oracles: `bash docs/tasks/artifacts/done_bar/run_family_status_bar_probes.sh`
  → **24/24** with all three status gates ✅ GREEN AND ALIGNED on the moved rows;
  `bash scripts/audit_done_bar.sh` → exit **0**, `0 \`Done\` rows are claimed — nothing to judge,
  VACUOUSLY green` (stated, not silent); all three `*_parser_family_status_contract_gate.sh`
  replayed against the aligned summaries → ✅ pass.
- [x] **NO REGRESSION** — no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` staged ⇒ all 11
  generated parsers byte-identical BY CONSTRUCTION; `.1` probe arms → **12/12** (incl. the new
  CTRL-1b proving an unproven `Done` claim still fails); demotion probe → HISTORICAL exit 0;
  `bash scripts/check_doctrines.sh` → ALL 13 PASS; `mdbook_docs_gate` → green.
- [x] **LOCKSTEP** — tracker rows + tracker note (the snapshot CHANGED — five rows moved),
  `README.md`, `docs/book/src/parser-families.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`,
  `MEMORY.md`, this tree, `docs/TASK_TREE.md`. No release / schema / ledger / contract movement:
  no parser behavior changed.

##### Original charter (kept for the record)

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

#### ✅ `.4a` — BOTH PREREQUISITES ARE NOW DISCHARGED (`done`, 2026-07-30 session #227, `PGEN-DONE-BAR-0025`)

`.4` named two prerequisites and both are closed, so the leaf is unblocked for the first time:

1. ✅ **`CI-PARITY-GATE-ROT.7`** — the aggregate went green end-to-end (`PGEN-CI-PARITY-GATE-ROT-0024`).
2. ✅ **the escalated hosted-auto-trigger call** — **the director ruled on 2026-07-30: enable the three
   regeneration-free gates.** Executed here.

##### ⛔⛔ TWO OF MY OWN NUMBERS WERE WRONG AND WERE CORRECTED *BEFORE* SPENDING THE DIRECTOR'S MONEY

- **"the automatic tier is ZERO"** — imprecise as stated. `memory-architecture-gate.yml` already runs on
  `push`/`pull_request` **and** invokes the doctrine driver since `CI-PARITY-GATE-ROT.15`, so **all 14
  doctrines already had an automatic lane** (measured: driver runs in **2 s**, no cargo/build/network).
  What was at zero is the automatic lane over the **123 `make` gate targets** — a narrower and truer
  claim than the one the tree carried.
- **"cost is seconds of Actions per push"** — ⛔ **WRONG, and caught before enabling.** That was a LOCAL
  measurement with a warm cargo cache and `mdbook` already installed. On a bare hosted runner
  `mdbook-docs-gate` runs `cargo install mdbook --version 0.5.2 --locked` (a compile) and
  `fixed-point-gate` builds Rust. The repo's own budgets say so: **10 / 30 / 15** `timeout-minutes`.
  ⇒ the honest per-run cost is **minutes, not seconds**.
- ⭐ **What makes it cheap anyway is FREQUENCY, not per-run cost, and that had to be measured too:**
  this repository has **0 merge commits** in its entire history and **7** pushes total, against a
  director push cadence of **300 commits**. ⇒ the lane fires a handful of times a year, so worst-case
  ~55 Actions-minutes per push event. *The right denominator was the push rate, not the commit rate.*

##### WHAT SHIPPED

`push:` added to `branch-protection-contract-gate.yml`, `fixed-point-gate.yml`, `mdbook-docs-gate.yml`
— the three documented as needing **no** `generated/` regeneration, so they run on a bare checkout.
⛔ **`pull_request:` deliberately OMITTED**: 0 merge commits ever ⇒ it would fire never, and wiring that
makes nothing run is precisely the theatre `.6` rejected. The reason is written at each site so a future
session adds it deliberately rather than wondering.

##### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — derived census over `.github/workflows/*.yml` (`awk` on each `on:` block):
      **1 of 15** workflows auto-ran, and `.6` measured the AUTOMATIC tier over the 123 `make` gate
      targets at **ZERO** ⇒ every proof lane ran only when a human asked.
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family: `grep`/`awk` over the 15 tracked
      `.github/workflows/*.yml` `on:` blocks shows `workflow_dispatch:`-only on 14 of them, the hosted
      pause recorded in `README.md`. The blocker was never technical — it was a **billed-action decision
      reserved to the director** (`.6` escalated it; `MEMORY.md` records *"real-world side effects
      (push/external/billed) stay director-owned"*).
- [x] **FIX** — declarative tier: one `push:` trigger on each of the 3 regeneration-free workflows. No
      script, no Makefile, no new mechanism.
- [x] **ADDRESSED (verified)** — before→after on the derived census: auto-running workflows
      **1 → 4** (`memory-architecture-gate` + the 3). ⭐ **Viability checked BEFORE enabling, because a
      RED automatic lane is worse than none**: all three carry `gh api=0 secrets=0 curl/wget=0` and
      `branch_protection_contract_gate.sh` reaches no network, so they work on a bare runner; and both
      locally-runnable ones **PASS right now** (`make -C rust branch_protection_contract_gate` → PASS,
      `make -C rust fixed_point_gate` → PASS; `mdbook_docs_gate` ran GREEN earlier this session).
- [x] **NO REGRESSION** — `bash scripts/check_flow_integrity.sh --report` → `flow-integrity: OK (11
      workflow(s) regenerate, 3 measured-exempt, recipe has one home, PREPARE on, 0 standalone-default
      hand-offs, 0 requires-a-defect assertions, provenance ratchet 1/23, all 14 doctrines on the
      automatic lane via the driver, 0 guards testing an artifact they do not read)` — ⭐ invariant **(8)**
      still passes with **4** auto-triggered workflows instead of 1, and CTRL-3 of its own probe set
      (*a `workflow_dispatch`-only workflow naming an enforcer must still PASS*) is unaffected.
      `GATE-REACHABILITY` PASS. No `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` ⇒ all 11
      generated parsers byte-identical BY CONSTRUCTION.
- [x] **LOCKSTEP** — `README.md` (the hosted-pause paragraph), this tree, `CHANGES.md`,
      `DEVELOPMENT_NOTES.md`, `MEMORY.md`. No release / schema / ledger / contract movement.

##### ⚠️ THE HONEST BOUND ON THE GUARANTEE, SHARPENED RATHER THAN CLAIMED AWAY

`.4`'s standard is *"enforced at every commit AND re-proved by an automatic lane no contributor
controls."* What now holds: the doctrines are re-proved server-side **on every push** (2 s), and three
gate targets join them. ⛔ What still does not: **a push happens roughly every 300 commits here**, so
the automatic re-proof is *per push*, not *per commit* — and the other **120** gate targets, including
the 4 h 39 m aggregate, remain operator-invoked. ⇒ the defensible claim is **"enforced at every commit
by the hook, and re-proved server-side at every push over the doctrines + 3 cheap gate targets"** — not
*"the flow guarantees the bar 100%"*. Closing that gap means pricing a periodic (e.g. `schedule:`) lane
for the heavy aggregate, which is a fresh billed-action decision and stays the director's.

### `.5` — the missing CONSUMER-FACING gates: disclosure integrity (`todo`) ⭐ PREREQUISITE FOR SHIPPING `Provisional`

- **Status: `todo`** — ⛔ **PROMOTED 2026-07-29.** It was recorded as *"safely deferred, not on the
  critical path"* because no family reached it. **That reasoning does not survive the `Provisional`-ships
  directive**: if customers decide from published state, the published state must be true, and it is
  measurably not (guide `1.1.29`/`1.1.31` vs contract `1.1.104`/`1.1.106`, no gate reading either).
  *A disclosure nobody checks is a claim, not a disclosure.*
- **Four gates that did not exist** (measured: no gate referenced the ledger or the user guide),
  now SPLIT into sub-leaves:
  1. `.5c` — **no reachable silent-success path** for the family — ✅ **DONE** (see below).
     ⛔ **The inventory recorded here was wrong in TWO ways and the leaf corrects both**: it is
     **5** codegen sites, not 6; and those three literals
     (`<property_access>`/`<array_access>`/`<last_extraction>`) occur **0 times in every shipped
     artifact**, so a gate built to this bullet would have been **VACUOUSLY GREEN**. The shipped
     surface is the RUNTIME half — **3,702 arms in 10 of 11 artifacts**, with a ledgered
     consumer-visible corruption history;
  2. `.5a` — **published version currency** — ✅ **DONE** (see below);
  3. `.5b` — **zero open ledger entries** naming the family — ✅ **DONE** (see below);
  4. `.5d` — **documented acceptance boundary** where a consumer looks (`in-progress`; ⭐ the
     `.5d`-vs-`.6` overlap is **ADJUDICATED — the split STANDS**: `.5d` is *"what will it parse and
     where does it get that wrong?"* (a leg-2 gate criterion, moves with the GRAMMAR) while `.6` is
     *"how well is that proven?"* (a disclosure task, moves with the EVIDENCE), and they are
     empirically independent — the census finds the boundary documented by **1 of 9** contracts and
     that one documents no bar state. MEASURED: 8 of 9 carry `## Scope / Non-Goals`, **1 of 9**
     documents an acceptance boundary, **0 of 9** have it read by any gate. ⛔ `regex` — the most
     mature family — is the ONE with no scope section at all. The gate is deliberately deferred
     behind the in-flight `sota_exit_gate` run — see below).
  6. `.5f` — **158 GB of unread backtrack-trace logs, plus a 1.4 MB evidence-custody gap**: the
     aggregate's scratch tree is 198 GB, ~158 GB of it trace logs nothing reads, written because a
     promotion gate leaves tracing ON by default (`in-progress`; scope items 1+2 ✅ **DONE** —
     the default is now `none` and the cost is MEASURED at **84–86% of the stage's wall time**, so
     this turned out to be a SPEED finding and not only a hygiene one; item 3, the custody fallback,
     stays open — see below).
  7. `.5g` — **a bounded always-on liveness signal for long gate stages** (`todo`; split out of `.5f`
     because it is a new mechanism, not a re-pricing). Today the only liveness channel a long stage
     has is the trace level itself, and prior art was measured ABSENT — no progress flag, no
     `PGEN_*PROGRESS*` env var, no periodic generator trace line.
  5. `.5e` — **`.5c`'s BINDING half**: the sentinel gate must bind a family's TIER, not merely exist
     — ✅ **DONE** (see below). A gate whose verdict enters no status computation is a gate that can
     go red while every tracker row stays green.
- ⭐ Once these exist they are ordinary leg-2 gates. That is the point: **the bar stays three legs and
  stays fully mechanizable**, which is what *"the flow shall guarantee this 100%"* demands.

#### `.5e` — the sentinel gate BINDS a tier (`done`, 2026-07-30 session #226, `PGEN-DONE-BAR-0016`)

- **Charter (written by `.5c`'s own frontier note):** *"wire a `no_reachable_silent_success` criterion
  into the four family-status computations (the `.5b` shape), so the gate does not merely exist but
  BINDS a family's tier."*
- ⛔⛔ **THE MEASURED GAP — `.5c` SHIPPED AN INSTRUMENT NOTHING CONSULTS WHEN DECIDING A TIER.** The
  gate is green, reachable, and wired as a `sota_exit_gate` prerequisite, yet **all four family
  computations carry 0 references to it** (`rust/scripts/{regex,sv,vhdl}_parser_family_status_gate.sh`
  and `rust/scripts/lib/parser_family_status_bar.sh` → `grep -c silent_success` = **0/0/0/0**), and its
  only consumers repo-wide are **a `make` target and one aggregate prerequisite edge**
  (`rust/Makefile:1042,1043,1045,1055`). ⇒ if a sentinel became reachable in `vhdl` tomorrow, the
  aggregate would fail — and the `vhdl` row would still read `Provisional (corpus pending)` with
  **every one of its 12 criteria green**, because none of them can see a placeholder inside a
  successful parse. **This is exactly `.5b`'s shape**: *the fact is true and unguarded at the tier
  level.*
- **DESIGN — four decisions, each one recorded because the obvious alternative has already failed
  here:**
  1. ⭐ **PRODUCE the evidence, never consume an ambient artifact.** `CI-PARITY-GATE-ROT.7` measured
     the alternative end to end: a consumer was pointed at a sub-gate's STANDALONE default state dir
     and a **three-day-old** `summary.txt` was consumed as CURRENT proof by a "fresh" run. So the
     helper RUNS `silent_success_sentinel_gate.sh` into `STATE_DIR/silent_success_sentinel_gate` —
     the identical shape `family_done_bar_leg3` already uses for `check_gate_reachability.sh`. ⛔ **No
     `EXISTING_*_STATE_DIR` seam is added and the aggregate is deliberately NOT rewired**: an unused
     hand-off seam is an invitation to the `FLOW-INTEGRITY` (5) violation.
  2. ⭐ **Cache PER PROCESS, never per directory.** The sv gate computes TWO families, so the sweep
     must not run twice — but a cache keyed on *"the file is already there"* is precisely how stale
     evidence gets reused. The cache is a shell global, so it cannot outlive the run that produced it.
  3. ⭐ **The gate's exit 1 is a VERDICT, not a refusal.** Exit 1 means a sentinel was reached
     *somewhere*; the artifact is valid and its per-family rows are exactly what this criterion needs
     (so a defect in `regex` must not make `vhdl` unjudgeable). Only exit 2 — REFUSE / MISCALIBRATED —
     is unjudgeable, and then the status gate REFUSES too.
  4. ⭐ **Attribute the STATIC arm per family, not repo-wide.** A codegen placeholder is reported as
     `{literal: {artifact: count}}`, so the criterion reads `generated/<family>_parser.rs` by **exact
     filename** — `systemverilog` must not inherit `systemverilog_preprocessor`'s violations, which a
     substring match would hand it.
- ⛔ **REFUSAL POLARITY (exit 2), and the arm that matters:** a family **absent from the swept
  roster** REFUSES. Removing `vhdl` from the sentinel contract must never make its criterion read
  *"0 sentinels reached"* — that is the vacuity trap this very instrument was built to escape, one
  level up.
- **SCOPE NOTE (deliberate, recorded):** the gates' prose `DONE_RULE` strings are **not** extended to
  narrate this criterion or `.5b`'s. Per-family prose disclosure is `.6`'s work; the machine-readable
  `criteria` map plus the `unmet_closure_criteria_details[]` entry are this leaf's disclosure surface.

##### MEASURED RESULT

- **All four families bind clean and NO tracker row moved**, which is the expected outcome and was
  stated as a prediction before it was run: each status gate produced its own sweep — **9 families,
  225 samples, calibration 3/3, 0 sentinels reached** — and the criterion reads `true` for regex, sv,
  svpp and vhdl. Criteria totals moved **regex 10→11, sv 9→10, svpp 14→15, vhdl 12→13**; all three
  replays stay **green and tracker-aligned** (`regex` `In Progress`, `vhdl` + `svpp`
  `Provisional (corpus pending)`, `systemverilog` `Mostly Done`).
- **COST, MEASURED — and it CORRECTS `.5c`'s published figure**: one full sweep is **26.0 s**
  (`time bash rust/scripts/silent_success_sentinel_gate.sh`), not the *"~40 s"* `.5c` recorded ⇒ the
  delta is **+26 s per status gate, ≈ +78 s across the three**, against a ~5 h aggregate. ⭐ **The
  number was cross-checked rather than taken on trust**: the probe driver's total wall clock was
  **1 m 20.9 s** while three 40 s sweeps alone would have been ~120 s — a discrepancy worth stopping
  on. It reconciles exactly: the three sweep dirs were created **27 s and 26 s apart**, 3 × 26 = 78 s,
  and the 32 helper arms are milliseconds. Nothing was skipped (each artifact independently reports
  its own 9 families / 225 samples / 3-of-3 calibration).
- ⚠️ **MY OWN PREDICTED ARM COUNT WAS WRONG AND IS CORRECTED FORWARD, NOT BACK-DATED**: this leaf's
  first draft said *"+13 arms … expect 29 + 13 = 42"*. The real total is **52** — the new arms are
  **23** (15 helper + 8 replay), not 13; I under-counted my own additions. 6th instance in this tree's
  history of a plausible figure surviving until someone re-ran it, this time my own.

##### ⛔ A DEFECT FOUND BY THIS LEAF'S OWN RED ARM, ROUTED NOT FIXED — `CI-PARITY-GATE-ROT.17`

Proving the new pin is load-bearing meant making a contract gate's schema assertion FAIL — which
nobody had ever done. It fails with **`exit=1` and a `0`-byte log**: `…:184`/`:191`/`:239` end a
~60-line monolithic `jq -e '<conjunction>' … >/dev/null` under `set -euo pipefail` with no `||` guard
and no message, so the gate reports *that* the schema is wrong and never *which* of ~20 conjuncts.
⭐ Same family as `.14` (a consumer burying the real cause) and `.2a` (emit the summary BEFORE
exiting), one layer further in: **a check that fails must say what failed.** Class priced at **4
sites** (`regex_broader_corpus_proof_gate.sh` is the fourth — the first count of 3 was the gates in
hand, before the sweep was run). ⛔ Deliberately NOT fixed here: the honest fix is a decomposition in
one shared helper across 4 sites, and this leaf's scope was one criterion.

##### ROUTING EVIDENCE (for the `CI-PARITY-GATE-ROT.17` routing above)

1. **Does the finding reproduce OUTSIDE the family it is being routed to?** ⭐ **YES — and that is the
   whole basis of the routing.** The silent-assertion shape was found in the **vhdl** contract gate, so
   the tempting (and wrong) reading is *"a vhdl-family defect"*. Swept:
   `grep -rlE "^\s+' \".*\" >/dev/null\s*$" rust/scripts/*.sh` → **4 files across three families plus a
   non-family gate** — `vhdl_…`, `regex_…`, `sv_parser_family_status_contract_gate.sh` **and
   `regex_broader_corpus_proof_gate.sh`**. A defect present in every family's gate is nobody's family
   defect; it is a shared shell idiom on the PROOF SURFACE, which is precisely what
   `CI-PARITY-GATE-ROT` owns.
2. **What was MEASURED to place it there, not what makes it plausible?** The reproduction, not the
   reading: a green artifact with ONE pinned criterion deleted (and both count fields decremented so
   every arithmetic invariant stays satisfiable) drives the gate to **`exit=1` with `wc -c` = `0`** on
   its combined stdout+stderr. The mechanism was then read at the named lines
   (`…:184`/`:191`/`:239` — `jq -e '<conjunction>' … >/dev/null` under `set -euo pipefail`, no `||`,
   no message), so the WHY and the WHERE are both from tools rather than inference.
3. **What would have to be true for the routing to be WRONG, and was it checked?** It would have to be
   **specific to one family's gate** (⇒ a family tree owns it) or **caused by this leaf's own edit**
   (⇒ mine to fix here). Both checked and both false: the class is 4 sites, and `git diff` shows this
   leaf touched only the `expected_criteria` literal in those files — the unguarded `jq -e … >/dev/null`
   line is untouched and pre-dates it. ⚠️ **Honest limit:** the sweep matched one *specific* textual
   shape (a heredoc-style assertion closing with `' "$file" >/dev/null`). A gate expressing the same
   silent-assertion idiom differently would not have been counted, so **4 is a floor, not a census** —
   `.17` should re-derive it rather than inherit the number.

##### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — measured before any code: `grep -c silent_success` over the three
  `*_parser_family_status_gate.sh` **and** the shared helper → **0 / 0 / 0 / 0**; the sentinel gate's
  only consumers repo-wide are `rust/Makefile:1042,1043,1045` (the target) and `:1055` (the
  `sota_exit_gate` prerequisite edge). ⇒ the instrument ran and bound nothing.
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family. WHY: a silent success returns `Ok` with
  zero diagnostics, so every "did it parse?" criterion in a family computation is green on it BY
  CONSTRUCTION; `.5c` built the detector but its verdict entered no tier, so a reachable sentinel
  would fail the aggregate while the family's row kept every criterion green. WHERE: the four criteria
  sets in `rust/scripts/{regex,sv,vhdl}_parser_family_status_gate.sh` and the shared home
  `rust/scripts/lib/parser_family_status_bar.sh`. Tool-backed: `bash -n ` clean on all 8 edited shell
  surfaces; `git ls-files`-derived consumer census as above; `jq` over the gate's own
  `summary.json` confirms the per-family rows the criterion reads.
- [x] **FIX** — declarative-tier: one shared-helper function + criterion wiring + summary/JSON metrics
  + three pinned contract-gate rosters. **No `grammars/*.ebnf`, no `rust/src/*`, no `generated/*`.**
- [x] **ADDRESSED (verified)** — before: no family computation could disagree with its row over a
  placeholder handed back inside a successful parse. After, by re-runnable oracle
  `bash docs/tasks/artifacts/done_bar/run_family_status_bar_probes.sh` → **52/52** (was 29/29):
  `CTRL-S1` reproduces the shipped artifact's own vhdl numbers; **`RED-S1`** proves a reached sentinel
  is counted AND named (`reached=2 sentinels=3`); **`CTRL-S2a/b`** prove exact-filename attribution
  (svpp charged 4, `systemverilog` charged 0 for the same violation — a substring match would have
  demoted both); `RED-S2`..`RED-S6` all REFUSE (absent from roster / no report / unmeasured row / zero
  samples / missing artifact); **`CTRL-S4b`** is the cache arm — a stub gate counting invocations reads
  exactly **1** across two families; `RED-S7`/`RED-S8` separate REFUSE (exit 2) from VERDICT (exit 1).
  Plus a RED arm for the pinning itself: deleting one pinned criterion with every arithmetic invariant
  kept satisfiable makes the contract gate **exit 1** — the pin binds, it is not decorative.
- [x] **NO REGRESSION** — no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` staged ⇒ **all 11
  generated parsers byte-identical BY CONSTRUCTION**; all three
  `*_parser_family_status_contract_gate` pass against the fresh artifacts (exit 0/0/0) on the extended
  roster; `bash scripts/check_gate_reachability.sh --report` → **OK (124 targets; 93 reachable,
  30 orphan + 1 policy-only, all dispositioned; 8 ground-truth controls reproduced)** — unchanged;
  `bash scripts/audit_done_bar.sh` exit 0, unchanged; `bash scripts/check_doctrines.sh` ALL 14 PASS.
  No release / schema / ledger / contract movement, and **no tracker row moved**.
- [x] **LOCKSTEP** — this tree, `docs/tasks/CI-PARITY-GATE-ROT.md` (the routed silent-assertion
  defect, new `.17`), book `quality-and-closure-model.md` (status-gate + sentinel sections),
  `README.md` standard commands, `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`,
  `docs/TASK_TREE.md`.

#### `.5f` — the probe driver's replay arms are anchored to a 198 GB untracked scratch dir (`in-progress`)

- **Status: `in-progress`** — opened 2026-07-30 session #226 while pre-flighting
  `CI-PARITY-GATE-ROT.7`'s acceptance run, and recorded rather than mentioned in passing.
  **Scope items 1 and 2 are DISCHARGED (2026-07-30 session #227, `PGEN-DONE-BAR-0020`)**; item 3
  (the 1.4 MB custody fallback for the `.5e` replay driver) remains open, and the bounded
  always-on liveness signal is split out as **`.5g`**.
- **MEASURED:** `du -sh rust/target/sota_exit_gate/work` → **198 G**, holding **40** `summary.*` pairs
  (91 files, **1.4 M** of actual evidence). The tracked driver
  `docs/tasks/artifacts/done_bar/run_family_status_bar_probes.sh` reads **14** of those sub-dirs by
  literal path for its replay arms.
- ⛔⛔ **ROOT-CAUSED 2026-07-30 ON THE DIRECTOR'S QUESTION (*"Why so big?"* / *"what's in those log
  files?"*) — AND IT CORRECTS THIS LEAF'S OWN FIRST FRAMING.** The size is **not** evidence. It is
  **8 trace logs of ~20 G each ≈ 158 G** (`du` per sub-dir: `sv_parse_full_ratio_promotion_gate`
  **154 G**, `sv_stimuli_quality_gate` **35 G**, `sv_declared_shadow_promotion_gate` **8.9 G**; every
  other sub-dir ≤ 494 M), named
  `work/trial_{0..3}/logs/profile_{2017,2023}_closed_loop_replay_parseability_shadow.log`.
- **WHAT IS IN THEM — one line shape, ~50 million times per log** (20 G ÷ 398 B/line; a
  prefix-normalised census over the first 200 M shows **577,974 of 582,168 lines** are this shape):
  ```
  [PGEN][LOW] 🧭 [/Volumes/SSD/…/trial_0/work/systemverilog_parser.rs:7] [<pgen::ast_pipeline::VerbosityLogger as pgen::ast_pipeline::Logger>::log_error]   📍 /Volumes/SSD/…/trial_0/work/systemverilog_parser.rs:7
  ```
  i.e. **398 bytes to record "the parser backtracked at generated-parser line 7"**.
- **WHY (two compounding causes, both measured):**
  1. **Tracing is ON BY DEFAULT for a production gate stage.**
     `rust/scripts/sv_stimuli_quality_gate.sh:49` —
     `REPLAY_TRACE_VERBOSITY="${PGEN_SV_STIMULI_QUALITY_REPLAY_TRACE_VERBOSITY:-low}"`. Per
     `TOOLBOX.md` §2.1, `low` is the 🧭 **errors/backtracks** level — the highest-frequency event class
     that exists in a PEG engine. The toolbox's own warning (*"full trace on a real input is hundreds
     of MB — almost always scope it with `--trace-rules`"*) is aimed at interactive debugging; here it
     is the unscoped default of an 8-replay promotion gate.
  2. ⭐ **73% of every line is ONE repeated absolute path.** The 145-byte build path appears **twice**
     per line (290 of 398 bytes) ⇒ roughly **115 G of the 158 G is the same path string written ~400
     million times.** Note the generator's own trace line uses a RELATIVE path
     (`src/ast_pipeline/stimuli_generator.rs:5962`) while the GENERATED parser's lines carry the
     absolute one — so this also brushes the repo-root-relative-path policy, not just disk.
- ✅ **NOTHING READS THEM — verified, which is what makes the disposition safe:**
  `grep -rn 'closed_loop_replay_parseability_shadow' rust/scripts/*.sh scripts/*.sh | grep -iE '\.log|logs/'`
  → **zero hits**. The stage's CONSUMED artifact is the structured
  `--parseability-report-json` report, and `sv_stimuli_quality_gate.sh:2226` `require_nonempty_file`s
  the **JSON**, not the log. The `.5e` probe driver references `sv_parse_full_ratio_promotion_gate`
  **0** times. ⛔ Checked deliberately because `CI-PARITY-GATE-ROT.11` warns that 14 gates scrape prose
  log lines for values — this log is not one of them.
- ⛔ **THE CUSTODY PROBLEM IS REAL BUT MUCH SMALLER THAN FIRST STATED.** This leaf originally recorded
  *"198 G pinned by 1.4 M of evidence … cannot be reclaimed safely"*. **Corrected:** ~158 G is
  unread trace output and IS safely reclaimable; only the **1.4 M** of `summary.*` pairs is evidence a
  tracked oracle depends on, and only THAT carries the `OPS-MEMSAFE.3` custody lesson (*durable
  evidence does not live under `rust/target/`*) — a `cargo clean`, a sweep, or the next aggregate run
  removes it and the `.5e` replay arms then SKIP.
- **SCOPE, in the order that pays:**
  1. ✅ **DONE (`PGEN-DONE-BAR-0020`) — the replay trace now defaults to `none`, with the env override
     kept for triage.** ⛔ **The design tension was SETTLED BY EVIDENCE, not by fiat, and the evidence
     changed the shape of the fix:** the trace's purpose is recorded in
     `LIVE_ACHIEVEMENT_STATUS.md`'s own 2026-04-21 note, verbatim — *"long
     `profile_2017_closed_loop_replay` runs are now **tail-able by default** instead of leaving empty
     stage logs unless a developer remembered to opt in manually"* ⇒ the purpose is **LIVENESS**
     (*is this stage still working or is it hung?*), **not failure triage**. That reframes the call:
     switching it off wholesale would have silently dropped a real, documented, still-valid operator
     capability — the exact pattern this repo keeps punishing — so the fix keeps it as an **opt-in
     the gate itself advertises** rather than a fact buried in a 2026-04 changelog entry.
  2. ✅ **DONE — MEASURED, and the leaf's own "do not quote a number until it is measured" is honored
     by a tracked re-runnable driver** (`docs/tasks/artifacts/done_bar/run_replay_trace_cost_ab.sh`,
     capture `replay_trace_cost_ab.txt`). Per shadow replay (count=8, sv_2017, seed 700000; two runs):

     | arm | wall time | stage log bytes | log lines | backtrack-noise lines |
     |---|---|---|---|---|
     | `low` (retired default) | **31–35 s** | 1,892,694,365 (**1.76–1.89 GB**) | 9,333,543 | **9,331,148 = 99.97%** |
     | `none` (shipped default) | **5 s** | 1,082 | 6 | 0 |

     ⇒ **trace share of wall time 84–86%**, **7.0× slowdown**, **~1.75 million×** the log bytes,
     written at **~50–60 MB/s / ~270–300k lines/s**. ⭐ **The arms produce BYTE-IDENTICAL stimuli and
     an IDENTICAL parseability report** — asserted by the driver, which is what upgrades "the trace is
     expensive" to "the trace is pure overhead". ⇒ **this IS the SPEED doctrine, not only hygiene.**
     ⚠️ **HONEST BOUND, STATED NOT IMPLIED: the effect on the aggregate's ~5 h is NOT measured here.**
     What is measured is one invocation; `sv_parse_full_ratio_promotion_gate` and
     `sv_declared_shadow_promotion_gate` run the stage across trials × 2 profiles at full
     5,000-attempt scale, so the aggregate effect is plausibly large and deliberately unquoted.
     `CI-PARITY-GATE-ROT.7`'s next end-to-end run is what prices it in situ.
  3. 🔜 **STILL OPEN** — the 1.4 MB custody fallback for the `.5e` replay driver (below).
- ✅ **RECLAIM AUTHORIZED AND EXECUTED (2026-07-30) — 228 G, more than the 158 G first estimated.**
  The director's ruling was a STANDING one (*"delete reclaimable files on a regular basis"*, recorded as
  [[feedback_delete_reclaimable_artifacts_regularly]]), so the sweep is now routine rather than
  per-request. ⭐ **The census widened the class before acting**: the same log name also exists OUTSIDE
  the aggregate tree (the standalone `rust/target/sv_stimuli_quality_gate/logs/` and the
  `sv_declared_shadow_promotion_gate` trials) ⇒ **18 files, 228 G**, not the 8 files / 158 G the
  aggregate-only view showed. *The first number was scoped to where the question was asked, not to
  where the class lives.*

  | | before | after |
  |---|---|---|
  | `/Volumes/SSD` used | 383 G | **156 G** |
  | `rust/target/sota_exit_gate/work` | 198 G | **2.8 G** |
  | `summary.*` pairs / shadow JSON reports | 91 / 33 | **91 / 33** (verified intact) |

- ⛔ **DELIBERATELY NOT SWEPT, and that is the discipline the standing authorization does NOT relax:**
  ~40 G of `regen_*.log` (`generated_lint_correctness/`, `generated_logs/sentinel_probe/`) turned up in
  the same census and their readers were **never checked** ⇒ left in place. *An unverified file is not
  a reclaimable file.* Likewise `rust/target/debug/{deps,incremental}` (**64 G** measured) is
  reclaimable in the safety sense but forces a rebuild, and the debug binaries are hard preconditions
  of the toolbox and of `silent_success_sentinel_gate` — sweeping them before a 5 h acceptance run
  trades hours for disk that is no longer scarce.
- **Scope when taken up:** teach the driver a snapshot fallback (prefer the live run dir, else a
  small committed-or-cached snapshot of just the `summary.*` pairs), then the bulk becomes safely
  disposable. A 1.4 M snapshot already exists at `rust/target/done_bar_5e/run3_snapshot/` (taken this
  session as insurance before run 4 would have overwritten the originals) and is the shape to
  generalize. ⚠️ Do NOT simply commit the snapshot without pricing it — 1.4 M of gate output in git is
  a decision, not a detail.
- ⭐ **Consequence for `CI-PARITY-GATE-ROT.7`:** whoever launches acceptance run 4 should snapshot the
  summary pairs FIRST (one command, 1.4 M), because run 4 overwrites the run-3 artifacts the `.5e`
  replays depend on.

##### What shipped for items 1 + 2 (`PGEN-DONE-BAR-0020`, 2026-07-30 session #227)

- **`rust/scripts/sv_stimuli_quality_gate.sh:49`** — the default moves `low` → `none`, with the
  measured A/B and the *"do not flip this back without re-running that A/B"* prohibition written at
  the site, so the next author meets the evidence instead of the bare value.
- ⭐ **ONE HOME, and it was verified to be one:** six wrapper gates plus the aggregate invoke this
  script (`sv_parser_aggregate_contract_gate`, `sv_roundtrip_contract_gate`,
  `sv_failure_context_contract_gate`, `sv_declared_shadow_promotion_gate`,
  `sv_parse_full_ratio_promotion_gate`, `stimuli_cross_family_platform_gate`, `sota_exit_gate`), and
  **none of them sets the variable** — so a single-line default change covers the whole class rather
  than needing seven edits.
- **The opt-in is now printed where a triager looks**, not only in a code comment: a derived
  `REPLAY_TRACE_VERBOSITY_NOTE` (ONE home, consumed by both the startup banner and the final
  summary) emits `closed_loop_replay_trace_verbosity_note:` — naming the env var and its ~50–60 MB/s
  cost when quiet, and naming how to restore the default when an override is active. ⭐ This is the
  *"a check that cannot run must SAY SO"* discipline applied to a **capability**: the liveness signal
  is opt-in rather than silently dropped.
- ⚠️ **THE RISK THIS CHANGE ITSELF CREATED, CHECKED BEFORE SHIPPING — a new summary key can shadow a
  scraped one.** `sota_exit_gate.sh:792` scrapes the SV summary by key
  (`sed -nE "s/^KEY: (.*)$/\1/p" | tail -n 1`) and `regex_parser_family_contract_gate.sh:51` uses an
  **unanchored** `grep -F "KEY: "`; a key that is a prefix-extension of a scraped key would win the
  `tail -n 1`. Both real matchers are replayed against a two-line fixture in the probe driver
  (RED-1 / RED-1b) and both correctly read the value line, because `_note` precedes the colon.
  Verified separately that **no gate asserts an exact key set or line count** over that summary, and
  that the note lands only in `summary.txt` (inside the `} >"$SUMMARY_TXT"` block), never in the JSON.
- ⛔ **A BOUNDED ALWAYS-ON PROGRESS SIGNAL WAS DELIBERATELY *NOT* BUILT HERE — routed to `.5g`.** The
  obvious candidates (a background heartbeat loop in the gate; a periodic progress line from
  `ast_pipeline`) are **new mechanisms**: a stray background process inside a 5 h aggregate is an ops
  hazard this repo has already paid for, and an engine-side progress facility must pass the
  zero-cost/neutrality acceptance test. Per `DESIGN-PRIOR-ART` a search was run first and found **no
  existing bounded progress facility** (`--help` carries no `--progress`/`--interval`/heartbeat flag;
  zero `PGEN_*PROGRESS*` env vars in `rust/src/`; the stimuli generator emits **no** periodic
  `pgen_trace_*` line at any level) ⇒ the gap is real, and it deserves its own leaf with its own
  design review, not a drive-by in a hygiene slice.

##### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `du -sh rust/target/sota_exit_gate/work` → **198 G** with **1.4 M** of
      actual evidence; a surviving log measured at this leaf's own reduced scale
      (`rust/target/sv_failure_context_contract_gate/.../profile_2017_closed_loop_replay_parseability_shadow.log`)
      is **59,434,221 bytes / 164,942 lines, of which 163,395 (99.06%) are one repeated 🧭 backtrack
      line**, and `awk` per 20k-line block shows the noise share uniform (19,506–19,949 per 20,000)
      rather than confined to a warm-up phase.
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family:
      `grep -n 'REPLAY_TRACE_VERBOSITY' rust/scripts/*.sh` names the single site
      `rust/scripts/sv_stimuli_quality_gate.sh:49` resolving
      `PGEN_SV_STIMULI_QUALITY_REPLAY_TRACE_VERBOSITY` to `low` — the 🧭 errors/backtracks level per
      `TOOLBOX.md` §2.1 — forwarded into the two `run_logged` replay stages at `:2124`/`:2207`, whose
      only consumer is `run_logged`'s `tail -n 80` on failure (`:699-711`). The same `grep` proves
      VHDL's gates carry no such default, which is why their surviving shadow logs are **1,101 B** and
      **1,129 B** against SV's 59 MB — the class is one site, measured, not assumed.
      `git log -S`-equivalent provenance is the tracked 2026-04-21 note quoted above, which states the
      liveness intent verbatim. `bash -n` clean on the edited gate.
- [x] **FIX** — declarative tier: one default value + a derived note string with ONE home; no
      `rust/src/*`, no grammar, no codegen, no new mechanism.
- [x] **ADDRESSED (verified)** — before→after **REPLAYED, not described**:
      `bash docs/tasks/artifacts/done_bar/run_replay_trace_cost_ab.sh` → `low` **35 s /
      1,892,694,365 B / 9,333,543 lines (9,331,148 noise)** vs `none` **5 s / 1,082 B / 6 lines (0
      noise)**; *trace share of wall time 85.7%*, *7.0x slowdown*, *1749255x log-byte ratio*, and
      **`✅ stimuli BYTE-IDENTICAL` + `✅ parseability report IDENTICAL`** across the arms.
      `bash docs/tasks/artifacts/done_bar/run_replay_trace_default_probes.sh` → **13/13**, including
      **BEFORE-1**, which extracts the default-resolution block out of `git show HEAD:` and confirms
      the retired form resolved to `low` with **no** discoverable note.
- [x] **NO REGRESSION** — no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` staged ⇒ **all 11
      generated parsers byte-identical BY CONSTRUCTION**; the A/B driver additionally proves the
      judged artifacts (stimuli + parseability report) are byte-identical across the two trace
      levels, so no gate's verdict can move; `bash scripts/check_doctrines.sh` → **ALL 14 enforced
      doctrines PASS**; the two summary-scraper matchers replayed against a fixture (RED-1/1b) prove
      the new key cannot shadow the scraped one; probe arm **RED-2 re-derives** that **0** consumers
      read the replay stage logs rather than trusting the earlier census.
- [x] **LOCKSTEP** — `docs/book/src/stimuli-and-quality.md` (the *Replay Progress Tracing* section
      rewritten with the measured table and the reason the default moved),
      `LIVE_ACHIEVEMENT_STATUS.md` (new 2026-07-30 note + the 2026-04-21 note marked **SUPERSEDED in
      place**, its historical text kept), this tree, `CHANGES.md`, `DEVELOPMENT_NOTES.md`,
      `MEMORY.md`. No release / schema / ledger / contract movement: no published parser identity,
      AST shape or support boundary changed.

#### `.5g` — a bounded always-on liveness signal for long gate stages (`todo`)

- **Status: `todo`** — split out of `.5f` on 2026-07-30 (`PGEN-DONE-BAR-0020`) rather than absorbed,
  because it is a **new mechanism** and `.5f` was a hygiene/re-pricing slice.
- **The need is real and documented, not hypothetical.** `LIVE_ACHIEVEMENT_STATUS.md`'s 2026-04-21
  note records the operator complaint verbatim: a long replay stage leaves an **empty stage log**, so
  nothing distinguishes a working stage from a hung one. `.5f` re-priced the answer (the trace costs
  84–86% of the stage's wall time to serve it) but did not replace it: with the default now `none`,
  first-run liveness is served only by opting back into the firehose.
- ⭐ **PRIOR ART SEARCHED AND ABSENT** (`.5f`, measured): `ast_pipeline --help` exposes no
  `--progress` / `--interval` / heartbeat flag; there are **zero** `PGEN_*PROGRESS*` env vars in
  `rust/src/`; and `rust/src/ast_pipeline/stimuli_generator.rs` emits **no** periodic `pgen_trace_*`
  line at any verbosity. So today the ONLY liveness channel any long stage has is the trace level.
- **Scope when taken up — settle the tier first, then build:**
  1. adjudicate **where** it belongs: a gate-script heartbeat (cheap, but a background process inside
     a 5 h aggregate is an ops hazard — `OPS-MEMSAFE` territory) versus an engine-side bounded
     progress line (general, reusable by every family's gates, but must pass the
     zero-cost/neutrality acceptance test: **non-users pay ZERO**);
  2. the signal must be **bounded by time, not by event count** — the defect in `low` is that it is
     event-driven in a PEG engine, where the event rate is ~270–300k/s;
  3. ⛔ do **not** re-solve it with a log cap: a `head`-style cap loses the tail where a failure lives,
     and a tail ring buffer still pays the full per-event formatting cost `.5f` measured.
- ⚠️ **Cross-family, not SV-only:** the VHDL replay gates have the same empty-log-until-done shape
  (their surviving shadow logs are ~1.1 kB), they simply never turned the firehose on to compensate.
  Whatever lands here should serve any long gate stage, not just `sv_stimuli_quality_gate`.

#### `.5d` — the documented acceptance boundary where a consumer looks (`in-progress`)

- **Status: `in-progress`** — opened 2026-07-30 session #227 (`PGEN-DONE-BAR-0021`). **The
  adjudication `.5` asked for is DONE and the surface is MEASURED**; the gate itself is scoped below
  and deliberately not built in this slice (reason stated).
- **Charter, verbatim from this tree's own leg-4 resolution:** *"the family's acceptance boundary is
  documented where a consumer looks."*

##### ⭐⭐ THE `.5d` vs `.6` ADJUDICATION — they are two DIFFERENT facts that happen to live in one document

`.5` recorded the two leaves as overlapping and told whoever opened `.5d` to settle the split. They do
not overlap; they answer different consumer questions, and only one of them is a *gate*:

| | `.5d` — acceptance boundary | `.6` — bar state |
|---|---|---|
| the consumer's question | **"what will it parse, and where does it get that wrong?"** | **"how well is that proven?"** |
| the fact | the language surface: supported constructs, over-acceptance, over-rejection | the tier + which of the three legs are met + what is therefore unproven + the `Provisional` qualifier |
| changes when | the **grammar** changes | the **evidence** changes |
| kind of work | a **leg-2 GATE criterion** (mechanically checkable per family) | a **disclosure/publishing** task, gated by `.5` |

⭐ They are also **empirically independent, not two names for one thing**: the census below finds the
boundary documented by **1 of 9** contracts, and that one (`systemverilog`) documents **no** bar state
— so shipping either leaf would leave the other's fact missing. ⇒ **the split STANDS; `.5d` is not
absorbed into `.6`, and `.6` is not a superset of `.5d`.**

##### ⭐ THE MEASURED STARTING POINT (2026-07-30) — census `docs/tasks/artifacts/done_bar/run_acceptance_boundary_census.sh`

| family contract | `## Scope / Non-Goals` | boundary disclosure |
|---|---|---|
| PNR | 1 | 0 |
| **REGEX** | **0** | 0 |
| RETURN_ANNOTATION | 1 | 0 |
| RTL_CONST_EXPR | 1 | 0 |
| RTL_FRONTEND | 1 | 0 |
| SEMANTIC_ANNOTATION | 1 | 0 |
| **SYSTEMVERILOG** | 1 | **1** |
| SYSTEMVERILOG_PREPROCESSOR | 1 | 0 |
| VHDL | 1 | 0 |

- **8 of 9** carry the repo's standard `## Scope / Non-Goals` section; **1 of 9** documents an
  acceptance boundary (SV's `### Honest boundary (ledgered, each with its own fix leaf)`); **0 of 9**
  have any of it read by a gate — measured soundly as *zero scripts repo-wide name any
  boundary/scope heading*. The only contract CONTENT any gate reads is the regex Contract Identity
  pair (`PUBLISHED-VERSION-CURRENCY`) and the regex oracle-tuple anchors
  (`REGEX-ORACLE-ANCHOR-SYNC`) — neither is a boundary. ⇒ **`.5d`'s gap is real.**
- ⛔⛔ **THE FINDING THAT WAS NOT IN THE CHARTER: `regex` — the most mature family in the repo, ~100
  releases and a 2,189-case corpus — is the ONE contract with NO scope section at all.** Its headings
  are Purpose / Contract Identity / Current Trust Statement / Companion Documentation / a long
  chronology of Maintenance Updates and Release Highlights / Supporting Documents / Stable Integration
  Surface / Build requirements. ⚠️ Its `## Current Trust Statement` is the closest thing and is
  **deliberately NOT counted as a boundary**: it says the flavor is *"closure-grade and fit for
  downstream parser consumption"* and that this *"does not automatically cover every regex dialect or
  every future contract widening"* — that is a **disclaimer about extent**, not a statement of what is
  accepted or of where acceptance is wrong. ⭐ Note the direction: **the family with the most evidence
  discloses the least boundary**, which is the opposite of what a reader would assume.
- ⚠️ **AND SV's ONE DISCLOSURE IS MOSTLY HISTORY, NOT CURRENT STATE:** its `### Honest boundary` list
  is per-ledger-entry and nearly every entry reads **FIXED** (`SV-0021`..`SV-0034` …). It is an
  excellent audit trail and it is *not* a current answer to *"what does this parser get wrong today?"*.
  Its sibling `### Support boundary` is about the **bug-reporting protocol**, not acceptance.
  ⇒ whoever builds the gate must not treat "SV already has one" as a template without deciding whether
  the template should be current-state or historical.

##### ⛔⛔ THE TRAP THIS GATE MUST NOT FALL INTO — it is `.5c`'s vacuity trap, one document over

The naive gate is *"assert every family contract has a section named `Scope / Non-Goals`"*. That gate
would **pass today for 8 of 9 families** and could only ever fail on regex ⇒ it measures **heading
presence, not disclosure**. A section can exist and say nothing, exactly as `.5c`'s chartered
literals existed 0 times and would have made its gate vacuously green. ⭐ **A heading is not a
boundary**, and this census already demonstrated the failure mode twice in one sitting:

1. the first cut counted `^#+ .*(support|boundary|scope|…)` headings and reported **regex 12 /
   systemverilog 68** — almost all of them `## Release … Highlights` rows;
2. the second cut split gate references into "existence-only vs content-reading" per line and reported
   **16/32** — wrong, because `check_published_version_currency.sh` reads the contract through a
   VARIABLE on a later line, so a per-line "is there an extractor here" test is structurally blind.
   The split was **dropped, not re-baselined**, and CAL-6 replaced it with a claim that can be
   defended directly.

⇒ **both crude instruments were caught by cross-checking against a fact already known**, and the
census now pins **6 calibration facts** and REFUSES (exit 2) rather than reporting numbers if any
fails to reproduce.

##### Scope when the gate is built (NOT in this slice — reason stated, not implied)

- ⛔ **Why not now:** `sota_exit_gate` acceptance run 4 (`CI-PARITY-GATE-ROT.7`) is IN FLIGHT and reads
  `rust/scripts/*` and `rust/Makefile` continuously across its ~5 h. A new gate needs a script **and**
  Makefile wiring (`GATE-REACHABILITY` refuses an unwired target), and editing the Makefile mid-run is
  the **mixed-vintage hazard** this repo has already paid for — it makes every number the run produces
  unattributable. The measurement and adjudication are therefore landed now; the gate lands after the
  run reaches its verdict.
- The gate must assert something **substantive**, not a heading. Candidate shape, to be adjudicated:
  the boundary section must NAME the family's open ledger entries (a set the repo already derives, per
  `.5b`'s `family_open_ledger_entries`) and must not name a **closed** one as open ⇒ the disclosure is
  then joined to a machine-derived fact and cannot be satisfied by prose alone.
- ⭐ That candidate has a property worth keeping: today every family has **0 open ledger entries**
  (`.1`'s census, 168 rows), so a naive "names its open entries" assertion is **vacuously satisfiable**
  — which is precisely the trap above. The gate must therefore also make the **empty case explicit**
  (an affirmative *"no open defects known"* disclosure), so a consumer can tell *"nothing is wrong"*
  from *"nobody wrote anything down"*.
- The regex scope gap is a **content** fix owed by the regex family, and is routed as such rather than
  fixed here: adding a section to satisfy a gate that does not exist yet is backwards.

#### `.5c` — the silent-success sentinel gate (`done`, 2026-07-29 session #225, `PGEN-DONE-BAR-0014`)

- ⛔⛔ **THE CHARTER'S OWN INVENTORY WOULD HAVE SHIPPED A VACUOUSLY GREEN GATE — MEASURED FIRST,
  WHICH IS THE ONLY REASON IT WAS CAUGHT.** `.5` chartered this leaf against the CODEGEN half of the
  sentinel family (`<property_access>` / `<array_access>` / `<last_extraction>`). Those literals occur
  **0 times across all 11 shipped `generated/*.rs` artifacts** (`grep -coF`, re-measured 2026-07-29).
  ⇒ a gate asserting only their absence **passes today over an untouched tree and can never fail** —
  the exact trap `GENERATED-LINT-CORRECTNESS.3` named (*"a generated-code gate that lints nothing and
  exits 0"*). The charter's PURPOSE (*"no reachable silent-success path for the family"*) governs; its
  inventory does not.
- ⭐⭐ **THE SHIPPED SENTINEL SURFACE IS A DIFFERENT, LARGER FAMILY — the RUNTIME half:**
  **3,702 fallback arms across 10 of the 11 shipped artifacts** (`<invalid_sequence_access>` **3,016**
  in 10, `<invalid_extraction_base>` **343** in 5, `<not_quantified>` **343** in 5; only
  `scratch_parser.rs` is clean). This class is **not hypothetical**: it has a proven consumer-visible
  corruption history — 9 reachable-corrupt SV sites fixed by `SV-AST-SHAPE-FIDELITY`, ledgered
  `SV-0014`..`SV-0020`, `SVPP-0001`, `RTL-FE-0002`, `VHDL-0001`, `RTL-CE-0001`.
- ⭐ **SITE COUNT CORRECTED: 5, not 6.** `.5`'s bullet and `MEMORY.md` both said *"6 sentinel emission
  sites"*; `DOCTRINE-GAP-OWNERSHIP.3`'s table lists **5**, and re-measurement confirms 5
  (`unified_return_ast.rs:1872/1883/1903`, `return_annotation_handler.rs:370`,
  `unified_semantic_ast.rs:301`). Corrected forward, not back-dated.
- ⛔ **A NEW DEFECT FOUND WHILE READING THE SITES, ROUTED NOT FIXED** — see `DOCTRINE-GAP-OWNERSHIP.3`:
  `unified_return_ast.rs:1883` emits `ParseContent::Terminal("<array_access>"` — **an unclosed call**,
  while its two siblings at `:1872`/`:1903` are balanced. That path returns `Ok(...)` carrying
  **syntactically invalid Rust**, which is strictly worse than the silent placeholder `.3` catalogued
  (a placeholder at least compiles). ⛔ Deliberately NOT fixed here: it is a `rust/src/*` change owned
  by another tree's leaf, and the code-change doctrine requires the owning leaf first.
- **SHIPPED — `rust/scripts/silent_success_sentinel_gate.sh`** + helper
  `rust/scripts/lib/silent_success_sentinel_sweep.py` + contract
  `rust/test_data/grammar_quality/silent_success_sentinel_contract_v0.json`, **two arms**:
  - **STATIC** — the codegen placeholders must stay ABSENT from every shipped artifact (locks a
    currently-true fact so it cannot drift in);
  - **DYNAMIC** — each family's own stimuli proof surface is generated under
    `--validate-parseability`, parsed and AST-dumped; **any sentinel actually REACHED fails**, naming
    the family, the sample index and the input.
- ⭐⭐ **CALIBRATION IS PART OF THE CHECK, because a detector with no POSITIVE control cannot tell a
  clean sweep from a blind one — a zero reads as a pass either way.** Three pinned ground-truth facts
  must reproduce or the gate prints `MISCALIBRATED` and exits 2 **without offering a sweep verdict**:
  `CAL-1` the fixed `SVPP-0001` ledger repro reads **0**; ⭐ `CAL-2` **the arm that matters** — the
  `SV-AST-SHAPE-FIDELITY.2.4` LATENT site reads **1** under `--entry-rule constraint_primary_sv_2017`
  isolation; `CAL-3` that same site reads **0** from the canonical entry (pinning the LATENT
  classification itself — a nonzero there means a latent site became reachable).
- ⚠️ **HONEST BOUND, printed in the gate's own output rather than implied away:** the dynamic arm
  proves *"NOT REACHED by 25 validated samples at the pinned seed"*, which is **NOT a proof of
  unreachability**. Reachability is entry-relative — that is precisely what `CAL-2` demonstrates.
- ⚠️ **THE INSTRUMENT'S OWN FIRST CUT WAS WRONG TWICE, both caught by requiring it to reproduce known
  facts** (recorded, not hidden): (1) the artifact roster globbed `*_parser.rs` and therefore **silently
  skipped `generated/ebnf.rs`** — a shipped parser without that suffix — under-reporting
  `<invalid_sequence_access>` as **2,893 vs the ground-truth 3,016** (exactly its 123 arms); the roster
  is now DERIVED from every `generated/*.rs`. (2) A faithfulness oracle read the generator's
  `sample_successes` as a PARSE count and raised a **false** `EXTRACTOR_SUSPECT` on
  `semantic_annotation`; measurement showed that field counts **GENERATION attempts** (27/27 generated
  vs `--validate-parseability`'s *"accepted 25/25 … 2 rejected over 27 attempts"*). ⇒ generation now
  runs under `--validate-parseability`, which makes *"every emitted sample must parse here"* SOUND.
- ⚠️ **A THIRD near-miss worth recording: `rtl_const_expr` generated ZERO samples** at the default
  window on seeds 0/7/42 via BOTH the `.ebnf` and raw-AST-JSON paths. That is **not a new defect** —
  `PARSE-HARNESS.5.5` already characterised its ~16-level precedence cascade (fails ≤28, hangs ≥40)
  and recorded the tuned window **`--max-depth 32`**, which reproduces 25/25 at 95.8% rule coverage.
  The window is now contract-declared and load-bearing: without it that family **REFUSES** rather than
  reporting a green zero.
- **MEASURED RESULT TODAY:** all **9** families sweep clean — 25 samples each, all validated-accepted,
  all parsed, **0 sentinels reached**; static arm **0** codegen-placeholder violations over **11**
  artifacts. Gate ~**40 s**, peak **870 MB**.

- ⚠️ **FOLLOW-UP FIX SAME SESSION (`PGEN-DONE-BAR-0015`) — the gate had an AMBIENT-BUILD dependency
  and a regeneration exposed it.** The sweep fed `grammars/<g>.ebnf` to `ast_pipeline`, which requires
  `--features ebnf_dual_run`; the STANDARD build — including the one
  `make -C rust regenerate_generated_parsers` leaves behind — does **not** enable it, so minutes after
  the gate shipped green it began **REFUSING** for `semantic_annotation` and `systemverilog`
  (*"requires building with --features ebnf_dual_run"*). ⭐ **The gate behaved correctly** — it refused
  rather than reporting a green it could not justify, which is exactly what it was built to do — but a
  gate wired as a `sota_exit_gate` prerequisite that goes red depending on **which make target ran
  last** is not usable. ⇒ generation now reads **`generated/<g>.json`** (the raw-AST JSON), which is
  already a hard precondition of this gate and carries no feature dependency; `--grammar-profile`
  still applies (measured: sv_2017 1352 rules vs 1475 unprofiled). New **CTRL-3** fails any family
  whose generation input is not under `generated/`, so the dependency cannot come back. Probes **9/9**.
  ⭐ Worth stating plainly: **the gate's first real-world failure was caused by another gate's side
  effect on a shared binary** — the artifact hand-off hazard `CI-PARITY-GATE-ROT` catalogues, reached
  through the build tree instead of a state dir.

##### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — measured before any code: the three charter sentinels occur **0** times
  in `generated/` (so the chartered gate is vacuous), while the runtime half ships **3,702** arms; and
  **no gate anywhere read a parse dump for a sentinel** (`grep -rln` over `rust/scripts/`, `scripts/`,
  `.github/workflows/`, both Makefiles → the sole hit is a comment in
  `sv_failure_context_contract_gate.sh:265`).
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family. WHY: a silent success returns `Ok` with
  zero diagnostics, so **every "did it parse?" gate is green by construction** and no acceptance test
  can see it; WHERE: the class is emitted at `ast_return_transform.rs:193/438` (the positional-model
  fallthrough — sentinel requires a positional `$N`, N≥2, over a too-short `Sequence`) and reaches
  consumers through `generated/*.rs`. Tool-backed: `parseability_probe --parse-dump-ast-pretty`
  reproduces the LATENT site at **1** sentinel under `--entry-rule` isolation and **0** canonically —
  the discrimination that proves the detector works. `bash -n` clean on both new shell surfaces;
  `python3 -c ast.parse` clean on the helper.
- [x] **FIX** — declarative-tier: one gate script + one Python sweep helper + one tracked contract +
  one Makefile target/edge. **No `grammars/*.ebnf`, no `rust/src/*`, no `generated/*`.**
- [x] **ADDRESSED (verified)** — before: no instrument could distinguish a clean parse from one
  handing back a placeholder. After, by re-runnable oracle
  `bash docs/tasks/artifacts/done_bar/run_silent_success_sentinel_probes.sh` → **8/8**:
  **CTRL-1** proves the positive control reads 1 (a 0 there means the detector is BLIND and every
  clean sweep is worthless); **CTRL-2** proves the artifact roster is DERIVED (11 scanned == 11 on
  disk — the arm that would have caught the `ebnf.rs` miss); **RED-1** a codegen placeholder in an
  artifact ⇒ FAIL(1); **RED-2** a REACHED sentinel ⇒ FAIL(1) naming family + sample; **RED-3**
  calibration drift ⇒ `MISCALIBRATED`(2) with no verdict; **RED-4** absent artifacts ⇒ REFUSE(2), never
  a green over an empty room; **RED-5** a family yielding no samples ⇒ REFUSE(2), never a green zero.
  ⚠️ **Two arms failed on their first run and BOTH were the PROBE's fault, not the gate's** — RED-2 left
  calibration facts in (calibration fires first, by design) and RED-4 assumed `cd` moves the gate's
  root (it derives it from `BASH_SOURCE`, correctly, per the repo-root relative-path policy); a
  `make_fake_root` helper fixes the latter properly.
- [x] **NO REGRESSION** — no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` staged ⇒ **all 11
  generated parsers byte-identical BY CONSTRUCTION**; `bash scripts/check_doctrines.sh` → **ALL 14
  PASS**; `bash scripts/check_gate_reachability.sh --report` → **OK (124 targets; 93 reachable,
  30 orphan + 1 policy-only, all dispositioned; 8 ground-truth controls reproduced)** — the new gate is
  reachable, not an orphan; `bash scripts/audit_done_bar.sh` unchanged. No release / schema / ledger /
  contract movement.
- [x] **LOCKSTEP** — this tree, `docs/tasks/DOCTRINE-GAP-OWNERSHIP.md` (the routed `<array_access>`
  defect), book `quality-and-closure-model.md`, `README.md` standard commands, `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `docs/TASK_TREE.md`.

#### `.5b` — zero open ledger entries, as a status-gate criterion (`done`, 2026-07-29 session #224, `PGEN-DONE-BAR-0013`)

- ⭐ **DESIGN DECISION, recorded:** this is NOT a pre-commit doctrine. A genuinely open ledger entry
  is a legitimate repository state (a downstream found a defect; it stays open until released) — a
  doctrine failing every commit while a real bug is open would block unrelated work. What an open
  entry must block is the FAMILY'S TIER: it is a leg-2-class fact (*a known, still-open downstream
  defect means the family's proof surface missed something a consumer hit*). ⇒ implemented as
  criterion **`ledger_open_entries_zero`** in the family-status gates, via the shared helper.
- **Helper:** `family_open_ledger_entries` in `rust/scripts/lib/parser_family_status_bar.sh` —
  derives the state vocabulary from the ledger's OWN "State Meanings" section (`Released`/`Rejected`
  closed, exactly as `.1`'s census established), counts open rows per family, REFUSES (exit 2) when
  the ledger or its vocabulary cannot be read. Seam: `PGEN_FAMILY_STATUS_LEDGER`.
- **Wiring:** criterion + count/ids metrics in all four family computations (regex 9→10, sv 8→9,
  svpp 13→14, vhdl 11→12), joined to each ladder's core (`Mostly Done`/`Done`) conjunction — an
  open entry demotes below `Provisional`, which is leg 2 reading honestly. Measured TODAY: regex
  114 rows / vhdl 2 / systemverilog 44 / svpp 4 attributed, **0 open each** ⇒ every computed status
  UNCHANGED and all replays stay green-aligned (verified — no tracker movement in this slice).

##### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `.1`'s census (`run_ledger_open_census.sh`): 168 rows, 0 open — *"the
  fact is true and unguarded — nothing reads that file"*; measured: no gate referenced the ledger.
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family: `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
  was read by ZERO gate scripts (`git ls-files` + the `.1` audit's derived gate universe); the
  status gates' criteria sets simply had no ledger fact. `bash -n` clean on all edited scripts.
- [x] **FIX** — declarative-tier: one helper function + criterion wiring; no grammar, no
  `rust/src/*`, no `generated/*`.
- [x] **ADDRESSED (verified)** — before: no instrument could ever disagree with a family's row over
  an open ledger entry. After, by re-runnable oracle `bash docs/tasks/artifacts/done_bar/run_family_status_bar_probes.sh`
  → **29/29**: CTRL-9 pins the real ledger derivation (`vhdl rows=2 open=0`); **RED-L1** proves an
  injected OPEN row is counted AND NAMED (`open=1 ids=FAKE-0001`); **RED-L2** proves a ledger
  without a State Meanings section REFUSES rather than classifying against a guessed vocabulary;
  the replay arms prove the criterion is recorded in every family's summary with statuses
  UNCHANGED and gates still ✅ green-aligned.
- [x] **NO REGRESSION** — no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` staged ⇒ all 11
  generated parsers byte-identical BY CONSTRUCTION; all three `*_parser_family_status_contract_gate.sh`
  ✅ pass the extended schema; `bash scripts/audit_done_bar.sh` exit 0 unchanged; `.1` arms 12/12;
  `bash scripts/check_doctrines.sh` ALL 14 PASS.
- [x] **LOCKSTEP** — book (`quality-and-closure-model.md` status-gates section), this tree, tracker
  note (no row moves), `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`. No release / schema /
  ledger / contract movement: the ledger is READ, not changed.

#### `.5a` — published version currency (`done`, 2026-07-29 session #224, `PGEN-DONE-BAR-0012`)

- **The disclosure fix AND the gate that makes the fix un-losable, in one slice** (fixing the drift
  without gating it would just restart the ~77-release clock):
  - `PGEN_USER_GUIDE.md`'s regex **Public contract identity** block updated `1.1.29`/`1.1.31` →
    **`1.1.106`/`1.1.109`** (the contract's Contract Identity block, the authoritative declaration —
    the same source the `PGEN-RGX-0091` embedding-constants gate is specified against); its
    published **`family status:`** updated `Done` → **`In Progress`** (the `.2b` row) with the
    demotion dated and explained; the 2026-04-era operational numbers are now MARKED as the dated
    historical baseline; the second stale mention (the "hardening slice" alignment) is DATED as
    historical with a pointer to the gated identity block.
  - NEW doctrine **`PUBLISHED-VERSION-CURRENCY`** (`scripts/check_published_version_currency.sh`,
    registered in `scripts/check_doctrines.sh` + the `DOCTRINE_ENFORCEMENT.md` §10 mirror ⇒
    **enforced at EVERY COMMIT via the pre-commit driver — the AUTOMATIC lane leg 2 demands**):
    guide identity pair == contract Contract Identity, guide published status == live tracker row.
    An empty extraction FAILS loudly rather than comparing empty strings. Reuses the `.2a` shared
    tracker-row reader (single home).
- ⚠️ **Honest scope bound:** this gates the ONE per-family public-identity block the guide carries
  (regex — measured: the only `Parser Flavor` identity block in the file). If another family gains
  a published identity block, the check must grow with it; `.6`'s machine-readable per-contract
  disclosure is the general surface.

##### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the check run BEFORE the fix named all three live drift instances:
  `publishes parser release '1.1.29' but the contract's Contract Identity declares '1.1.106'`,
  the `1.1.31` vs `1.1.109` sibling, and `publishes regex family status 'Done' but
  LIVE_ACHIEVEMENT_STATUS.md says 'In Progress'` (exit 1).
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family: `PGEN_USER_GUIDE.md:3808,3810` (the
  identity block) and the `family status:` line published state NO gate read (`git ls-files`
  confirms both documents tracked; `bash -n` clean on the new check + driver); the drift mechanism
  is the one `DOCTRINE-GAP-OWNERSHIP.4` measured — every release bumped the contract identity and
  nothing compared the guide against it.
- [x] **FIX** — declarative-tier: guide content fix + one structural doctrine check + registry row.
- [x] **ADDRESSED (verified)** — before: check exit 1 naming three drifts. After: `bash
  scripts/check_published_version_currency.sh` → `OK (guide 1.1.106/1.1.109 == contract identity;
  published status 'In Progress' == tracker)` exit 0; probe arms `bash
  docs/tasks/artifacts/done_bar/run_published_version_currency_probes.sh` → **5/5** (RED-1 replays
  the historical stale pair verbatim; RED-3 proves empty extraction never compares empty strings;
  RED-4 proves the contract advancing alone re-fails).
- [x] **NO REGRESSION** — no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` staged ⇒ all 11
  generated parsers byte-identical BY CONSTRUCTION; `bash scripts/check_doctrines.sh` → **ALL 14
  enforced doctrines PASS** (meta-mirror green with the new §10 row).
- [x] **LOCKSTEP** — `PGEN_USER_GUIDE.md`, `DOCTRINE_ENFORCEMENT.md` §10, this tree, `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `MEMORY.md`. No release / schema / ledger / contract movement: the
  contract documents are untouched — the GUIDE moved to match them.

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

**`.5d`'s GATE** — the adjudication and the census are DONE (`PGEN-DONE-BAR-0021`, session #227): the
`.5d`-vs-`.6` split **STANDS** (boundary = *what will it parse*, moves with the grammar; bar state =
*how well is that proven*, moves with the evidence), and the surface is measured — **8 of 9** contracts
carry `## Scope / Non-Goals`, **1 of 9** documents an acceptance boundary, **0 of 9** have it read by
any gate, and ⛔ **`regex`, the most mature family, is the one with no scope section at all**. The gate
itself is deliberately deferred behind the in-flight `sota_exit_gate` run (it needs `rust/Makefile`
wiring, and editing that mid-run is the mixed-vintage hazard), and its central design constraint is
already named: **a heading is not a boundary** — the naive "assert the section exists" form would pass
8 of 9 today and is `.5c`'s vacuity trap one document over. ⭐ **`.5f` scope items 1+2 DONE** (`PGEN-DONE-BAR-0020`, session #227): the SV
replay trace now defaults to **`none`**, and the cost it was paying is MEASURED rather than suspected
— **84–86% of the stage's wall time and 1.8–1.9 GB of log per invocation, 99.97% of it one repeated
backtrack line, for a log nothing reads**, with byte-identical stimuli and an identical parseability
report across the two arms. ⛔ **The design tension was settled by finding the ORIGINAL INTENT
(liveness, not triage — the 2026-04-21 tracker note says so verbatim), so the capability is now an
opt-in the gate itself advertises rather than a silent drop**; the bounded always-on replacement is
split out as **`.5g`** with prior art measured absent. ⚠️ The aggregate-level saving is deliberately
UNQUOTED — `CI-PARITY-GATE-ROT.7`'s next end-to-end run prices it in situ, and it should now be run
AFTER this change, not before. `.5f` item 3 (the 1.4 MB custody fallback for the `.5e` replay driver)
remains open. ✅ **`.5e` DONE** (`PGEN-DONE-BAR-0016`): `.5c`'s sentinel gate now **BINDS a
tier** — `no_reachable_silent_success` is a criterion in all four family computations (totals regex
10→11, sv 9→10, svpp 14→15, vhdl 12→13), each status gate PRODUCES its own sweep rather than reading
an ambient artifact (the `.7` stale-evidence lesson), the sweep is cached per PROCESS not per
directory, and the three DONE-BAR criteria are now PINNED in the contract gates — they were computed
by the producer and required by nothing. Probes **52/52**; measured **+26 s** per status gate; **no
tracker row moved** (all four families sweep clean). ⛔ Its RED arm exposed a contract-gate schema
assertion that fails with **exit 1 and a 0-byte log**, priced at 4 sites and routed to
`CI-PARITY-GATE-ROT.17`. ✅ **`.5c` DONE** (`PGEN-DONE-BAR-0014`): the silent-success sentinel gate ships
with a STATIC arm (codegen placeholders absent from all 11 artifacts) and a DYNAMIC arm (no sentinel
REACHED on any family's own stimuli surface), calibrated against three pinned ground-truth facts so a
clean sweep cannot be confused with a blind detector. ⛔ Its charter's own inventory would have
produced a **vacuously green** gate — the three chartered literals occur 0 times in every shipped
artifact, while the RUNTIME half ships **3,702** arms; both corrections are recorded in the leaf, and
a newly-found unbalanced-emission defect is routed to `DOCTRINE-GAP-OWNERSHIP.3`. 🔜 Next in this
tree: wire a `no_reachable_silent_success` criterion into the four family-status computations (the
`.5b` shape), so the gate does not merely exist but BINDS a family's tier.
✅ `.5a` DONE (`PGEN-DONE-BAR-0012`): the guide's published identity/status drift
FIXED and GATED by the 14th doctrine `PUBLISHED-VERSION-CURRENCY`. ✅ `.5b` DONE
(`PGEN-DONE-BAR-0013`): `ledger_open_entries_zero` is now a criterion in all four family-status
computations (an open downstream defect demotes the family's tier; deliberately NOT a pre-commit
doctrine, since a genuinely open entry must block the TIER, not unrelated commits). Remaining:
**`.5d`** — the documented acceptance boundary (adjudicate its overlap with `.6` when opened). `.3a` (ANVIL for `rtl_frontend`)
continues in parallel; `.4` (the enforcement ratchet) stays blocked on `CI-PARITY-GATE-ROT.7` +
the hosted auto-trigger call. ⭐ Near-term GOAL unchanged: **every family to at least
`Provisional`** — `regex` needs its leg-1 debt re-closed — ⛔ **and that debt is LARGER than this tree recorded**: run 4
published `8 of 11` criteria satisfied with **3** unmet, whose PRIMARY is
`stimuli_regex_parseability_parser_rejections_total=40 > 0`, a figure appearing nowhere in the tracked
record (which documents only `final_targets=31`) ⇒ closing the 31 alone would NOT close leg 1
(`REGEX-PCRE2-FIDELITY.ROUTED-IN-2` + the new `ROUTED-IN-4`),
`return_annotation` / `rtl_frontend` need a computed status (the missing-status-gate lever) before
they can hold `Provisional` honestly.

⭐ **Sequencing note, now discharged:** `.2` landed before `CI-PARITY-GATE-ROT.7`'s next
`sota_exit_gate` acceptance run — that run will no longer burn ~5 h re-confirming the known regex
tracker-alignment blocker, because the tracker now states what the gate computes.

## Blockers

✅ **`.4`'s FIRST blocker is CLEARED (2026-07-30): `CI-PARITY-GATE-ROT.7` is DONE — `sota_exit_gate` is
green end-to-end for the first time** (`exit=0`, 32 of 32 stages, 4 h 39 m, nothing reused or skipped;
`PGEN-CI-PARITY-GATE-ROT-0024`). ⛔ **`.4` remains BLOCKED on the escalated director call about resuming
hosted auto-triggers** — and that blocker is now the *only* one, which sharpens rather than weakens it:
`.6` measured the AUTOMATIC tier over the 123 `make` gate targets at **ZERO**, so a green aggregate on
one machine at one commit is exactly the *"enforced at every commit AND re-proved by an automatic lane
no contributor controls"* gap the tree's own honest bound names. ⇒ **the guarantee is no longer waiting
on a broken flow; it is waiting on a cost decision that is the director's to make.**
✅ `.2` is fully discharged (`.2a` `PGEN-DONE-BAR-0010`, `.2b` `PGEN-DONE-BAR-0011`); the
`rtl_frontend` / `rtl_const_expr` **qualifier ruling** remains owed by `.3` (their rows hold
`Mostly Done` with the qualifier withheld, per the no-defaulting rule).

## Verification Log

### `.5d` — the adjudication + the acceptance-boundary census (2026-07-30, session #227, `PGEN-DONE-BAR-0021`)

| instrument | command | result |
|---|---|---|
| the census | `bash docs/tasks/artifacts/done_bar/run_acceptance_boundary_census.sh` | exit **0** — 9 contracts DERIVED from a glob; **8** carry `## Scope / Non-Goals`, **1** documents an acceptance boundary, **0** scripts name any boundary/scope heading; **calibration 6/6** |
| the missing-scope family | same run, `NO scope section:` line | **REGEX** — asserted as calibration fact CAL-3, so it cannot silently become a different family |
| the boundary-owning family | same run, CAL-4 + CAL-5 | **SYSTEMVERILOG** only (`### Honest boundary (ledgered, …)`) |
| gate readers of a boundary | `grep -rlE 'Honest boundary\|Scope / Non-Goals\|Unsupported\|Known limitation' rust/scripts/ scripts/` | **0** — the sound form of the claim, after a per-line "existence vs content" split was measured WRONG and dropped |
| doctrines | `bash scripts/check_doctrines.sh` | **ALL 14 PASS** |
| byte-identity | *(by construction)* | docs + one `docs/tasks/artifacts/*.sh` census driver only — no `grammars/*.ebnf`, no `rust/src/*`, no `rust/scripts/*`, no `generated/*` ⇒ all 11 generated parsers unchanged |

⚠️ **TWO OF THIS LEAF'S OWN INSTRUMENTS WERE WRONG BEFORE THEY WERE RIGHT, and both are recorded in
the leaf rather than quietly fixed** — a heading-regex census that counted `## Release … Highlights`
rows (regex 12 / sv 68) and a per-line reference-mode split (16/32) that could not see an extraction
performed through a variable. Each was caught by cross-checking against a fact already measured, which
is why the census now pins 6 calibration facts and REFUSES rather than reporting numbers.

⚠️ **What this leaf does NOT claim:** the gate. Nothing is enforced yet — this is a measured
adjudication, and `.5d` stays `in-progress` until the gate lands.

### `.5f` items 1+2 — the replay-trace default (2026-07-30, session #227, `PGEN-DONE-BAR-0020`)

| instrument | command | result |
|---|---|---|
| the A/B cost driver | `bash docs/tasks/artifacts/done_bar/run_replay_trace_cost_ab.sh` | `low` **35 s / 1,892,694,365 B / 9,333,543 lines (9,331,148 noise)** vs `none` **5 s / 1,082 B / 6 lines**; trace share of wall time **85.7%**, slowdown **7.0x**, byte ratio **1,749,255x**, rate **51.6 MB/s / 266,604 lines/s**; ✅ stimuli BYTE-IDENTICAL, ✅ parseability report IDENTICAL |
| the A/B, first run (same driver shape) | as above | `low` **31 s / 1,892,694,367 B**, `none` **5 s** ⇒ the reported figure is a **range (31–35 s, 84–86%)**, not a single point |
| probe arms | `bash docs/tasks/artifacts/done_bar/run_replay_trace_default_probes.sh` | **13/13** — GREEN-1 default→`none` + note names the opt-in and its cost; CTRL-1/2 the override still resolves (`low`, `debug`) and the note flips to "ENABLED by override"; **BEFORE-1** replays `git show HEAD:`'s retired block → `low`, note `<none>`; RED-1/1b both real summary matchers read the value not the note; RED-2 re-derives **0** log consumers; CTRL-3 `bash -n` |
| class scope | `grep -n 'REPLAY_TRACE_VERBOSITY' rust/scripts/*.sh rust/scripts/lib/*.sh` | **1 site** (`sv_stimuli_quality_gate.sh:49`); VHDL gates carry none, and their surviving shadow logs measure **1,101 B** / **1,129 B** against SV's **59,434,221 B** |
| callers | `grep -rn 'sv_stimuli_quality_gate' rust/Makefile rust/scripts/*.sh .github/workflows/*.yml` | 6 wrapper gates + `sota_exit_gate`; **none sets the variable** ⇒ one default change covers the class |
| summary-schema safety | `grep -rn 'wc -l.*summary\|expected_keys\|key_count' rust/scripts/sv_*.sh rust/scripts/sota_exit_gate.sh` | **0 hits** — no gate asserts an exact key set or line count over the SV summary |
| doctrines | `bash scripts/check_doctrines.sh` | **ALL 14 PASS** |
| byte-identity | *(by construction)* | no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` staged ⇒ all 11 generated parsers unchanged |

⚠️ **What this log does NOT claim:** the aggregate's ~5 h cost. One invocation was measured; the
promotion gates run this stage across trials × 2 profiles at 5,000-attempt scale, so the aggregate
effect is plausibly much larger and is left for `CI-PARITY-GATE-ROT.7`'s next end-to-end run to price.

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

- `PGEN-DONE-BAR-0021` (2026-07-30, session #227, leaf `.5d` opened + adjudicated; docs + 1 census
  driver only) — the `.5d`-vs-`.6` overlap `.5` left open is **SETTLED: the split STANDS.** `.5d` is
  *"what will it parse, and where does it get that wrong?"* — a leg-2 gate criterion that moves with the
  GRAMMAR; `.6` is *"how well is that proven?"* — a disclosure task that moves with the EVIDENCE. They
  are empirically independent: the boundary is documented by **1 of 9** contracts and that one documents
  no bar state, so shipping either would leave the other's fact missing. MEASURED (calibrated census,
  6/6): **8 of 9** contracts carry `## Scope / Non-Goals`, **1 of 9** (systemverilog) documents an
  acceptance boundary, **0 of 9** have it read by any gate. ⛔⛔ **NOT IN THE CHARTER: `regex` — the most
  mature family, ~100 releases and a 2,189-case corpus — is the ONE contract with no scope section at
  all**, and its `## Current Trust Statement` is a disclaimer about extent, not a boundary ⇒ the family
  with the most evidence discloses the least boundary. ⚠️ SV's one disclosure is also **mostly history**
  (nearly every `### Honest boundary` entry reads FIXED), so it is an audit trail rather than a current
  answer. ⛔ **The gate is deliberately NOT built here**: `sota_exit_gate` run 4 is in flight and reads
  `rust/scripts/*` + `rust/Makefile` continuously, and a new gate needs Makefile wiring —
  the mixed-vintage hazard. Its design constraint is named up front: **a heading is not a boundary**;
  the naive form passes 8 of 9 today and is `.5c`'s vacuity trap one document over. ⚠️ Two of this
  leaf's own instruments were wrong before they were right (a heading regex that counted release-
  highlight rows; a per-line reference-mode split blind to extraction via a variable) — both left
  visible, both caught by cross-checking a known fact.
- `PGEN-DONE-BAR-0020` (2026-07-30, session #227, leaf `.5f` items 1+2 done; `.5g` opened) — the SV
  closed-loop replay trace defaulted to `low` since 2026-04-21 and was costing **84–86% of the
  stage's wall time** plus **1.8–1.9 GB of log per invocation, 99.97% of it one repeated backtrack
  line**, for output **nothing reads** — with **byte-identical stimuli and an identical parseability
  report** at `none`, so it was pure overhead. Default moved to `none` at the one site that all 6
  wrapper gates and `sota_exit_gate` inherit. ⭐ **The design tension `.5f` flagged was settled by
  finding the original intent rather than by fiat**: the 2026-04-21 tracker note states the purpose
  was **liveness** (*"tail-able by default instead of leaving empty stage logs"*), not failure triage
  ⇒ the capability is kept as an opt-in the gate **advertises in its own banner and summary**, and the
  bounded always-on replacement is split out as `.5g` with prior art measured **absent** (no progress
  flag, no `PGEN_*PROGRESS*` env var, no periodic generator trace line). ⚠️ **A risk this change
  itself created was checked before shipping**: the new summary key could have shadowed a scraped one
  under `sota_exit_gate`'s `sed`-anchored reader or `regex_parser_family_contract_gate`'s
  **unanchored** `grep -F`; both real matchers are replayed against a fixture. Probes **13/13** with
  BEFORE-1 replaying the retired default out of `git show HEAD:`; A/B is a tracked re-runnable driver;
  **ALL 14 doctrines PASS**; **no tracker row moved**. ⛔ Aggregate-level saving deliberately
  UNQUOTED — `CI-PARITY-GATE-ROT.7`'s next run prices it, and should now run AFTER this change.
- `PGEN-DONE-BAR-0016` (2026-07-30, session #226, leaf `.5e` done) — `.5c`'s silent-success sentinel
  gate now **BINDS a family's tier** instead of merely existing. Measured first: the gate's verdict
  entered **0 of 4** family computations (`grep -c silent_success` → 0/0/0/0) and its only consumers
  were a make target and one aggregate prerequisite edge ⇒ a reachable placeholder would fail the
  aggregate while the family's row stayed green on every criterion. New shared-helper criterion
  `no_reachable_silent_success` (totals regex 10→11, sv 9→10, svpp 14→15, vhdl 12→13); each status
  gate **produces its own sweep** (the `.7` stale-evidence lesson) cached per PROCESS not per
  directory; codegen violations attributed by **exact filename** so one family is never demoted for
  another's; REFUSES when the sweep cannot be judged, while the gate's own exit 1 stays a VERDICT.
  Also **pinned the three DONE-BAR criteria** in the contract gates — they were computed by the
  producer and required by nothing. Probes **52/52**, contract gates 0/0/0, reachability + audit
  unchanged, **+26 s** per status gate (correcting `.5c`'s ~40 s), **no tracker row moved**.
  ⛔ Routed out: the contract gates' schema assertion fails with **exit 1 and a 0-byte log**
  (4 sites) → `CI-PARITY-GATE-ROT.17`. ⚠️ Two of my own numbers corrected forward: the arm count
  (13 → **23** new, 52 total) and the sweep cost.
- `PGEN-DONE-BAR-0013` (2026-07-29, session #224, leaf `.5b` done) — `ledger_open_entries_zero` is
  now a criterion in all four family-status computations (regex 10, sv 9, svpp 14, vhdl 12
  criteria), derived from the ledger's OWN State Meanings vocabulary via the shared helper
  (`family_open_ledger_entries`; REFUSES without a vocabulary). Deliberately a TIER gate, not a
  pre-commit doctrine. Measured today: 0 open for every family ⇒ statuses unchanged, replays stay
  green-aligned. Probes 29/29 (RED-L1: an injected open row is counted and NAMED; RED-L2: a
  vocabulary-less ledger refuses); all three contract gates pass the extended schema. Frontier →
  `.5c`.
- `PGEN-DONE-BAR-0012` (2026-07-29, session #224, leaf `.5a` done) — the published-state drift is
  FIXED and GATED in one slice: the guide's regex Public contract identity moved `1.1.29`/`1.1.31`
  → `1.1.106`/`1.1.109` and its published family status moved `Done` → `In Progress` (the `.2b`
  row), and the NEW 14th doctrine `PUBLISHED-VERSION-CURRENCY`
  (`scripts/check_published_version_currency.sh`, pre-commit AUTOMATIC lane) holds both equal to
  their authoritative sources forever — an empty extraction fails loudly. Probes 5/5 (RED-1 replays
  the historical stale pair verbatim). `.5` SPLIT into `.5a`-`.5d`; frontier `.5b`.
- `PGEN-DONE-BAR-0011` (2026-07-29, session #224, leaf `.2b` done) — the five contested rows MOVED,
  and every instrument agrees: `vhdl` + `systemverilog_preprocessor` → `Provisional (corpus
  pending)` (their status gates replay GREEN AND ALIGNED), `regex` → `In Progress` (below
  `Provisional`: legs 1-2 unmet), `return_annotation` → `Mostly Done` (NOT `Provisional (ceiling)`
  — the Status Rules define `Provisional` as legs 1-2 MET and its leg 2 is red; the ceiling is its
  finished state once a computed, invoked status lane exists), `rtl_frontend` → `Mostly Done`
  (qualifier withheld as owed, `.3`). Audit C8 adjudicated (ground truth legitimately moved) →
  C8′ register⊆roster + an EXPLICIT vacuous zero-`Done` verdict; probes 12/12 (new CTRL-1b: a
  re-promoted unproven `Done` still fails) + 24/24; all three status-contract gates green on the
  new schema; README + book corrected forward. Frontier → `.5`.
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
