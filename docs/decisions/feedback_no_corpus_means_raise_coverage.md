# When a parser cannot be matched with an external test corpus, raise coverage by other means

**Category:** `feedback` (standing discipline)
**Stated:** 2026-07-29 (session #223), director.
**Status:** ⚠️ **the RULE is director-stated and binding. The (a)/(b)/(c) split, the
independence-or-derivation guard, and the ANVIL assessment below are the ENGINEER'S PROPOSED
REFINEMENT, recorded so they are not lost, and NOT yet confirmed.** Anything marked *proposed* must
not be cited as settled.

## The directive (verbatim)

> *"if no corpus is found for `rtl_frontend`, we should find ways to increase the coverage if need
> be. That should be the rule whenever a parser can't be matched with external test corpus."*

⇒ **A family with no external corpus does not get to sit at `Provisional (corpus pending)` doing
nothing.** Leg 3 being unreachable obliges us to raise coverage another way; it is not a licence to
stop. This closes the loophole in the `Done` bar that would otherwise make `(corpus pending)` a
permanent, comfortable parking space.

## ⚠️ THE GUARD (proposed) — or the rule quietly lowers the bar it was meant to raise

Leg 3 does not buy **volume**. It buys **independence**: evidence we did not author and did not
curate to pass. Every one of `rtl_frontend`'s 130 curated samples was written by someone who already
knew what the parser accepts — which is precisely the blind spot leg 3 exists to cover. If *"increase
coverage"* is satisfied by *"write more of our own examples"*, we get more volume with the same blind
spots and leg 3 becomes unfalsifiable.

**Proposed bar: a leg-3 substitute must restore INDEPENDENCE or DERIVATION.**

| kind | definition | counts toward leg 3? |
|---|---|---|
| **Independent** | material authored for a purpose other than testing this parser — real-world source, another project's suite, or a second implementation to differentially compare against | ✅ |
| **Derived** | mechanically enumerated from the language definition, so it cannot inherit the author's blind spots (stimuli generator, certificate coverage) | ✅ |
| **Hand-written examples** | more of what we already have | ⛔ legitimate work, but it must not close leg 3 |

⭐ **The property that matters is independence FROM THE ARTIFACT UNDER TEST, not third-party
authorship.** A corpus written by us but derived from a model that owes nothing to
`grammars/<family>.ebnf` is independent in the way that counts; a third-party corpus curated by us
until it passes is not.

## ⚠️ THREE SITUATIONS, NOT ONE (proposed) — one rule flattens them

| case | situation | families | the right move |
|---|---|---|---|
| **(a)** | a corpus exists for the **exact** language | `systemverilog`, `vhdl`, `regex`, `json`, `verilog_2005` | **not a corpus problem — a WIRING problem.** Measured 2026-07-29: every corpus-facing gate is a `make` orphan and two of regex's run by nothing at all. Wire what we already own; no new rule needed |
| **(b)** | a corpus exists for a **superset** | `rtl_frontend`, `rtl_const_expr` | **partition, don't abandon** — 16,388 real SV/V files are already vendored under `stimuli/sv/subs/`; split in-subset vs out-of-subset mechanically and hold the in-subset part to a pass. ⭐ **Superseded in practice by ANVIL, below** |
| **(c)** | **no corpus can exist** — PGEN authored the language | `return_annotation`, `semantic_annotation`, `ebnf` | **the only case where the director's rule is the sole option.** Strongest substitute: *dogfooding* — PGEN's own 17 tracked `grammars/*.ebnf` were written to define parsers, not to test the `ebnf` parser, so they are independent in the way that matters |

⚠️ **`rtl_frontend` is case (b), NOT case (c)** — so it is the wrong family to found the rule on.
*"We could not find a corpus"* was never true for it.

## ⭐⭐⭐ ANVIL — the director's proposal for case (b), assessed against the guard above

`/Volumes/SSD/Documents/github/anvil` (same volume ⇒ the storage policy is satisfied). A **random
by-construction generator of synthesizable SystemVerilog RTL**, whose own
`CODEBASE_ANALYSIS.md:243` states Phases 7-9 were delivered against *"the user's `rtl_const_expr` /
`rtl_frontend` style request"*: `anvil --artifact <dut|microdesign|frontend>`, binary built.

**Why it PASSES the independence guard — measured, not assumed:**

1. ⭐⭐ **Architecturally independent of grammars, BY EXPLICIT DESIGN.** ANVIL's book carries a
   chapter *"Why Not a Grammar?"* recording that an annotated-EBNF walk was considered **and
   rejected** in favour of circuit-cone recursion over a typed circuit graph. ⇒ it **cannot** inherit
   `grammars/rtl_frontend.ebnf`'s blind spots. This is the independence leg 3 wants, obtained
   structurally rather than by third-party authorship.
2. **Transitively anchored to four third-party tools** — Yosys, Verilator, Icarus, slang all appear
   as validation surfaces in its README. Its notion of *valid synthesizable SV* is adjudicated
   **outside both projects**.
3. **It ships an ANSWER KEY** — per-artifact *expected-facts* JSON manifests. Leg 3 requires corpora
   adjudicated by the corpus's own answer key; ANVIL emits one.
4. **Reproducible by seed** — byte-identical output for a given `(seed, knobs)` pair ⇒ a vendored
   corpus snapshot is **re-derivable**, not an opaque blob.
5. It targets exactly the **synthesizable envelope** `rtl_frontend` claims — a far closer match than
   raw VeeR/friscv source, which is full of legitimately out-of-subset material.

**Honest limits — the guard is applied, not waived:**

1. ⛔ **ANVIL is NOT third-party.** The `Done` bar says *"an officially-recognized third-party
   corpus"*. ANVIL does not satisfy that **wording**. ⇒ either the bar is **deliberately amended**
   (director's call) or ANVIL closes something we name differently. **It must not be quietly
   reinterpreted as satisfying leg 3** — silent relabelling is the exact failure this tree exists to
   stop. *Engineer's recommendation: amend it.* A rule that admits an arbitrary third-party repo but
   rejects an externally-adjudicated, grammar-independent generator is optimising the wrong variable.
2. **Generated ≠ real-world.** Random-by-construction RTL and human-written RTL fail differently.
   ANVIL should **complement** the case-(b) vendored partition, not replace it.
3. ⛔ **Circularity must be a MECHANICALLY GUARDED non-goal.** If ANVIL is ever tuned to emit only
   what PGEN accepts, independence evaporates **silently** and nothing would notice. A promise is not
   a guard.
4. **Coverage is ANVIL's model's coverage** — which `rtl_frontend` constructs it actually emits is
   **unmeasured**.

### ⭐⭐ THE CALIBRATION CONTROL, BEFORE ANVIL CERTIFIES ANYTHING

**The first test is whether ANVIL reproduces the `**` gap already measured** (`**` has 0 occurrences
in `grammars/rtl_frontend.ebnf`; `logic [7:0] ram [2**8-1:0];` is rejected by `rtl_frontend` and
accepted by the full-LRM `systemverilog` parser). **A corpus that cannot find a bug we know is there
is not ready to certify absence of bugs.** This is [[feedback_instrument_needs_ground_truth]] applied
to a corpus instead of an instrument, and it is non-negotiable before any leg-3 claim rests on ANVIL.

### ⚠️ A finding for the director, raised because the book is the review surface

ANVIL's mdBook documents the **DUT lane** thoroughly (`why-not-grammar`, `algorithm`, `ir`,
`synthesizability`, the motif catalogue) but has **no chapter for the `microdesign` or `frontend`
lanes** — the two lanes built for PGEN. Two of three artifact lanes are invisible in the surface the
director reviews. Owned by ANVIL, not PGEN; recorded here so it is not lost.

## Related

- [[feedback_done_bar_is_first_tier_only]] — the three-leg bar this refines
- [[feedback_instrument_needs_ground_truth]] — the calibration control above
- `docs/tasks/DONE-BAR.md` `.3` — the leaf that owns closing the external-corpus gap
