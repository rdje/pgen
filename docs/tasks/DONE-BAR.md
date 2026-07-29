# DONE-BAR: `Done` means SOTA / SIGNOFF-LEVEL done — a user can blindly trust the parser: stimuli-generator proof AND all gates AND all external corpus, and the trust that establishes

## Metadata

- Tree ID: `DONE-BAR`
- Status: `active` (opened 2026-07-29, session #221)
- Family / slice-id prefix: `PGEN-DONE-BAR-<NNNN>`
- Created: `2026-07-29`
- Owner: repo-local workflow
- Director directive: [[feedback_done_bar_is_first_tier_only]]
- Frontier: **`.1`** (the audit)

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

⇒ **the bar is not a checklist someone remembers to apply; it is an invariant the flow enforces.** A
`Done` row that does not meet all four legs must be **impossible to hold**, because a gate fails
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

| family | language owned by | leg 3 reachable? | ceiling |
|---|---|---|---|
| `systemverilog`, `systemverilog_preprocessor`, `vhdl`, `regex`, `json`, `verilog_2005` | **IEEE / PCRE2 / ECMA / JSONTestSuite** — external standards | ✅ yes, a corpus exists in the world | `Done` |
| `return_annotation`, `semantic_annotation`, `ebnf` | ⭐ **PGEN itself** | ⛔ **no — by construction**: there is no third-party corpus for PGEN's own annotation/meta languages | **`Provisional` is the honest ceiling** |
| `rtl_frontend`, `rtl_const_expr` | a *subset* PGEN defines of SystemVerilog | ⚠️ partially — SV corpora exist but exercise far more than the subset | to be adjudicated in `.3` |

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

### ⚠️ A LEG-4 SIGNAL, RECORDED AS A HYPOTHESIS

`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` contains **24 occurrences of the token `open`**
(against 102 `fixed`, 72 `closed`, 18 `resolved`). ⛔ **That is a token tally, not a count of open
defects** — `open` occurs in prose too. It is recorded because a released-parser ledger with genuinely
open entries is a direct leg-4 blocker for whichever families they name, and `.1` must resolve it to
actual per-family open-defect counts rather than leaving it as an impression.

## Leaves

### `.1` — audit every `Done` claim against the three legs (`todo`)

- **Status: `todo`** — the tree's frontier. Nothing started; no partial state.
- **Scope:**
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

### `.2` — demote what does not meet the bar, with the unmet leg named (`todo`)

- **Status: `todo`** — blocked on `.1`.
- Each demoted row states which leg is unmet and what would close it. The dated historical notes are
  left intact; the **current** row is what moves ([[feedback_done_bar_is_first_tier_only]]).

### `.3` — close the external-corpus gap (`todo`)

- **Status: `todo`** — blocked on `.1`'s per-family leg-3 result.
- Likely the largest body of work in this tree: acquiring/vendoring an officially-recognized corpus
  for families that have none, and promoting `triage`/`characterization` surfaces into asserted
  conformance where the parser can actually meet them.
- ⚠️ Where a family genuinely cannot have an external corpus, that needs a **recorded,
  director-visible justification** — absence is an unmet leg, not an inapplicable one.

### `.4` — THE GUARANTEE: the flow enforces the bar, so a false `Done` cannot be held (`todo`)

- **Status: `todo`** — the tree's **highest-value leaf** per the director's *"the flow shall guarantee
  this 100%"*; sequenced after `.1`/`.2` only because enforcement needs the audit's shape first.
- **Requirement:** a `Done` row that does not meet all four legs makes a gate FAIL. Not a report, not
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
  whether leg 3 is unmet or **unreachable by construction** (a PGEN-owned language has no third-party
  corpus, and saying so is more useful than an open TODO).
- ⭐ Machine-readable too, not only prose: the family status gates already compute a status string, so
  the tier and its evidence should be emitted in their `summary.json` for a downstream to check
  mechanically rather than parse marketing text.
- ⛔ Gated by `.5`, or the disclosure can go stale exactly like the version pair did.

## Current Frontier

`.1` — the audit. ⭐ Near-term GOAL: **every family to at least `Provisional`**. The tree's PURPOSE is `.4` (the flow guaranteeing the bar); `.5`/`.6` (disclosure integrity) are prerequisites for shipping `Provisional` honestly.

## Blockers

`.1` is read-only and can start immediately. ⛔ **`.4` — the guarantee itself — is BLOCKED on `CI-PARITY-GATE-ROT.7` (aggregate green) and on the escalated director call about resuming hosted auto-triggers.**

## Verification Log

_(empty — `.1` not started)_

## Commit Log

- `PGEN-DONE-BAR-0001` (2026-07-29, session #221) — tree opened on the director's standing
  directive; decision record `feedback_done_bar_is_first_tier_only.md`; measured starting point
  recorded as a hypothesis, not a finding.
