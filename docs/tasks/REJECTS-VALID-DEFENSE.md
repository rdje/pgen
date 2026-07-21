# REJECTS-VALID-DEFENSE: close the completeness (rejects-valid) verification gap — grammar, stimuli, corpus, spec

## Metadata

- Tree ID: `REJECTS-VALID-DEFENSE`
- Status: `parked` (director 2026-07-21, session #188, verbatim: "I hope you log the
  outcome of all this exchange in a task-tree if not done already. Keep it
  inactivate for me now because we have a lot of other things to do.")
- Unlock condition: an explicit director GO opens the activity; work then starts
  at leaf `.1` in the recorded priority order.
- Roadmap lane: verification-apparatus thoroughness (parser-agnostic where possible;
  regex = the first concrete family). Sibling lanes: `STIMULI-SIGNOFF` (the
  generator-signoff vision), `REGEX-PCRE2-FIDELITY` (the fidelity campaign whose
  `.3.13` slice produced the motivating incident).
- Created: `2026-07-21` (session #188)
- Owner: repo-local workflow

## Background (the motivating incident + the director's exchange)

`PGEN-RGX-0089` / ledger `REGEX-0115` (fixed 2026-07-21, tree `RGX-0089`): release
`1.1.82` (`REGEX-PCRE2-FIDELITY.3.13`) narrowed the shared `simple_escape` rule on a
context-local "behavior-invisible" argument — sound at pattern-body level, fatal in
the extended class `(?[...])`, where `\b` lost its only derivation. The rejects-valid
regression survived **~23 releases** and was found by the downstream consumer (RGX),
not by any PGEN gate.

Director's driving questions (this exchange, session #188): *"PGEN should have been
able to spot this issue. Why did we miss it? grammar? stimuli? how to prevent this
from happening again?"* and *"what is possible to catch or uncover the grammar
error? … I just want to make sure we are thorough (grammar, stimuli, …)"*.

**Adjudicated root understanding (recorded verdict of the exchange):**

1. The grammar edit was the TRIGGER (a shared-rule narrowing whose "invisible" claim
   was verified in one referencing context only). This error class will recur — no
   process makes it impossible; instruments must make it short-lived.
2. The 23-release SURVIVAL is an apparatus property: every generative instrument
   (cert coverage, duality hunt, spf) samples FROM our own grammar, so it proves
   soundness/self-consistency but is STRUCTURALLY incapable of testing completeness
   (a rejects-valid defect = a derivation our grammar lacks = unreachable by
   self-referential generation). The only completeness instrument — the frozen
   2,189-cell PCRE2 corpus — is tight in the false-reject direction (ceiling ≤48)
   but only ON ITS CELLS, and it had no `(?[\b])` cell.
3. KEY INSIGHT for the grammar-error question: this was a REGRESSION, and
   regressions are catchable DETERMINISTICALLY at edit time, because the PRE-edit
   grammar still derives the lost pattern. The current apparatus never exploits the
   previous vintage generatively. From-birth gaps (never-derived constructs) are
   reachable only by oracle-side means (grid enumeration, mutation-differential).
4. Historical bias worth naming: the fidelity campaign's threat model was
   accepts-invalid (burning false-accepts 285→262), so tooling grew one-sided;
   0089 came from the uninstrumented direction.

Durable companions: `docs/decisions/feedback_stimuli_gen_rejects_valid_blindspot.md`
(the director's original observation; now points here) and the
`PGEN-RGX-0089-0001` DEVELOPMENT_NOTES entry (the shared-rule-narrowing discipline).

## The charter (the exchange's outcome — priority-ordered)

Three complementary time-horizons; they are NOT redundant:

- **`.1` catches grammar errors AT THE MOMENT THEY ARE MADE** (regressions,
  deterministically — the direct answer to "what can catch the grammar error");
- **`.2` + `.3` close the KNOWN context-dual family FOREVER** (cells, once present,
  are protected by the already-tight false-reject ceiling);
- **`.4` sweeps for UNKNOWN-UNKNOWNS over time** (the only instrument class that
  reaches from-birth gaps).

## Task Tree

- ID: `REJECTS-VALID-DEFENSE`
  Status: `parked`
  Goal: `Give the verification apparatus its missing COMPLETENESS direction so a rejects-valid defect (regression OR from-birth gap) is caught upstream of downstream consumers.`
  Children: `.1` `.2` `.3` `.4` `.5`

- ID: `.1`
  Status: `proposed` (FIRST on GO — the only leaf that guards the NEXT `.3.13`
  regardless of which construct it hits; mostly assembly of existing instruments)
  Goal: `The GRAMMAR-EDIT DIFFERENTIAL GATE: every grammar edit produces an oracle-adjudicated verdict-flip ledger.`
  Sketch (from the exchange): (a) static accept-set/terminal-set diff of the edited
  rules (e.g. ".3.13 shrank simple_escape's letter set by {A,B,G,K,Z,b,z}");
  (b) the reverse-reachability CONE of each edited rule (reach tooling exists) —
  the mechanical context enumeration the `.3.13` slice lacked; (c) TARGETED probes:
  the diffed terminals/branches exercised through EVERY referencing context,
  generated from the BASE (pre-edit) vintage — for `.3.13` that is 7 letters × the
  cone, deterministically containing `(?[\b])`; (d) base-vs-candidate verdict diff
  over the targeted probes + the accumulated corpus + base-vintage-sampled stimuli;
  EVERY ACCEPT↔REJECT flip must carry a pcre2test receipt (intended flips recorded;
  an unadjudicated flip = RED). Precedent: the perf ratchet's flips-0/2,189 custody
  is this concept, minus the generative breadth — components all exist (two-vintage
  builds, stimuli generator, parseability probe, pcre2test, flip-diff plumbing).
  Limits (honest): sampling/targeting-based (language equivalence is undecidable);
  catches regressions deterministically for enumeration-style edits, NOT from-birth
  gaps.
- ID: `.2`
  Status: `proposed` (with `.3`, the immediate exhaustive patch — one bounded slice)
  Goal: `The CONTEXT-DUAL GRID: enumerate context-sensitive tokens × syntactic contexts, verdict every cell with pcre2test, fold into the persistent corpus.`
  Sketch: ~40 tokens (\b \B \A \Z \z \G \K \Q \E \N \R \X \C, digits/backrefs,
  - ^ ] [ | $ . * + ? { } …) × ~9 contexts (body, [...], [^...], (?[...]),
  nested-in-extended, \Q…\E, (?#…), x-mode body/class) ≈ 350 machine-generated
  cells. `(?[\b])` is exactly one cell of this grid; the whole family the director
  is worried about closes permanently at trivial ongoing cost.
- ID: `.3`
  Status: `proposed`
  Goal: `The CORPUS-ENRICHMENT RATCHET: every fidelity slice's frozen oracle-matrix cells JOIN the persistent conformance corpus same-commit (doctrine-enforced), + a one-time retroactive harvest of all banked matrices from the ledger/trees.`
  Honest caveat (recorded): this alone would NOT have caught 0089 (no slice ever
  wrote a `(?[\b])` cell) — it encodes "every context anyone ever verified stays
  verified forever", which is necessary but not sufficient.
- ID: `.4`
  Status: `proposed` (the one genuinely NEW instrument)
  Goal: `The MUTATION-DIFFERENTIAL LANE: generate → MUTATE OFF our grammar's support (cross-context escape swaps, (?[...]) wraps, construct splices) → compile with pcre2, parse with PGEN, diff verdicts BOTH directions → minimize → every confirmed divergence becomes a permanent corpus cell (feeds .3).`
  Sketch: the duality hunter already generates + diffs — it seeds only from our
  grammar; a mutation stage + the pcre2 verdict oracle is the smallest delta to an
  existing instrument. Would have found 0089 in minutes (`(?[\a])` is in-support;
  the one-letter mutation `a→b` is the witness).
- ID: `.5`
  Status: `proposed` (OPTIONAL — only after .1–.4; real value, real maintenance cost)
  Goal: `SPEC-CLAIMS TRACEABILITY: map normative pcre2pattern statements → owned corpus cells ("inside a class, \b means backspace" ⇒ at least one cell per class context, including future ones).`

Cross-cutting (already durable, no leaf): the shared-rule-narrowing design
discipline — any slice narrowing a shared rule's accept set must dump the rule's
reverse-reachability cone and re-verify the "invisible" claim per referencing
context (DEVELOPMENT_NOTES `PGEN-RGX-0089-0001`; `.1(b)` mechanizes the cone).

## What-catches-what (the exchange's adjudication table)

| Instrument | Catches `.3.13`-class regressions? | Catches from-birth gaps? | Cost |
|---|---|---|---|
| `.1` edit-time differential gate | **Yes — deterministically** | No | Moderate; assembly of existing parts |
| `.2` context-dual grid | **Yes** (owns the cell) | Only within the grid | Small, bounded, one slice |
| `.3` corpus ratchet | No (cell never existed) | No | Trivial; permanent |
| `.4` mutation-differential | Yes, near-certainly | **Yes** | The one new instrument |
| Current cert / duality / spf | No — structurally | No | — |

## Cross-family generalization (captured 2026-07-22, session #189 — director question "at which point do you plan to exercise all the parsers with official and aggressive external test corpus?")

The charter above is REGEX-scoped (pcre2test oracle, pcre2pattern spec, the PCRE2
external corpus). The director's 2026-07-22 question makes the CROSS-FAMILY form
explicit, and it is CAPTURED HERE as this tree's generalization clause: on the
director's GO, each lane generalizes per family with that family's official
oracle/suite — SystemVerilog: an official/aggressive external suite beyond the
standing 14-design real-world corpus (candidate: the CHIPS-Alliance `sv-tests`
suite; the LRM stays the spec authority); VHDL: an official suite beyond the
realistic corpus (candidates: VESTs / the GHDL test suite); JSON: the
spec-derived gate owned by the parked `JSON-RFC8259.4` (+ the JSONTestSuite
class of aggressive corpora as the external complement). External suite
ACQUISITION (downloads/licenses) is a real-world side effect and stays
director-owned — each family's suite is provisioned on explicit director
approval. No cross-family leaf is opened while this tree is parked.

> ⭐ **SV clause SUPERSEDED-BY-GO (2026-07-22, session #190):** the director's
> bar-amending directive ("No Done with the SV parser graduating from those
> external official and recognized SV corpus") activates the SV lane — owned by
> tree **`SV-CORPUS-GRAD`** ([[project_sv_done_requires_external_corpus_graduation]]),
> building on the ALREADY-vendored `stimuli/sv/subs/` suites
> (`EXTERNAL-CORPUS.3.1`, 2026-06-17). The VHDL / JSON / cross-family clauses
> above remain parked here awaiting their own GO.

## Non-Goals (while parked)

- NO implementation, NO gate changes, NO corpus changes until the director's GO.
- Not a reopening of `RGX-0089` (fixed + closed) nor of `.3.13` (its body-level fix
  was correct and is preserved by the 0089 fix).

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | (tree parked) | `parked` | Director 2026-07-21: log + keep inactive; other priorities first. On GO: start `.1`, then `.2`+`.3`, then `.4`; `.5` optional. |

## Decisions

- `2026-07-21`: Priority order adjudicated in the exchange: `.1` first (guards the
  NEXT regression regardless of construct), `.2`+`.3` next (cheap + permanent),
  `.4` as the standing sweep, `.5` optional. Rationale: prevention must be
  instrument-side — the trigger error class (context-local reasoning on shared
  rules) cannot be made impossible, only short-lived.
- `2026-07-21`: Parked at creation on the director's explicit instruction.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-21` | (tree creation) | Charter derived from the session-#188 director exchange (three questions + the layered answer); incident facts cross-checked against `docs/tasks/RGX-0089.md` + ledger `REGEX-0115`. | `recorded — no work opened (parked).` |

## Commit Log

| Slice | Commit ID | Note |
| --- | --- | --- |
| Tree creation (parked) | `PGEN-RVDEF-0001` | Docs-only: tree + decisions-note re-point + index row + MEMORY. No push (director-owned). |

## Changelog

- `2026-07-21`: Created PARKED per the director's instruction; charter = the full
  outcome of the "why did we miss it / how to be thorough" exchange.
