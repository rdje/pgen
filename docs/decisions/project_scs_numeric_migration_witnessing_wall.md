# scan-substring NUMERIC ref migration is blocked on a WITNESSING WALL (not just forward `+N`)

**Category:** project · **Established:** 2026-07-11 (`PGEN-REGEX-PCRE2-0047`, `REGEX-PCRE2-FIDELITY.4.7.b`, session #89) · tools-first (cert-coverage `UNKNOWN` list + 60k-stimuli generation sweep + `stimuli_generator.rs:1190`).

## Fact

Migrating the scan-substring NUMERIC capture references (`(*scs:(N))` absolute,
`(*scs:(-N))` backward, `(*scs:(+N))` forward) from the out-of-band validator into
`grammars/regex.ebnf` — by splitting `scs_capture_number` into sign-homogeneous rules so
each can carry its own store predicate (absolute → `fact_count_at_least phase:final`,
backward → `phase:post`) — **cannot keep BOTH `fully_certified` (cert `UNKNOWN=0`) AND
`spf=0`**, and is therefore DEFERRED (all scs numeric refs stay validator-owned) until a new
forward/suffix-count primitive exists. This is broader than the previously-scoped `.4.7.c`
"forward `+N` needs a suffix count": the *whole numeric split* is blocked.

## Why (the mechanism)

Sound scs generation is **absolute-only**. The `@gen_predicate fact_count_at_least(
regex_capture_group, $value)` on `scs_capture_number` compiles to a VALUE-DRAW `IndexUpTo`
that **replaces the rule's WHOLE render** with an integer in `1..=prior_count`
(`stimuli_generator.rs:1190`), never descending into sign branches — empirically **0 signed
scs refs across 60k stimuli**. Given that:

- **Whole-render-replace kept** ⇒ the split's `scs_capture_number_backward`/`_forward` rules
  are never generated ⇒ cert `UNKNOWN=2 fully_certified=false` (**measured**, seed 0). The
  reach-driver cannot rescue them: the whole-render-replace blocks descent, and forward `+N`
  probes fail to re-parse (no trailing group) ⇒ `SelectedButFailed`.
- **Generation DESCENT (leaf value-draw)** ⇒ backward `-N` becomes sound + witnessed (literal
  `-` + int≤prior_count), but the forward `+N` branch draws N≤prior_count and renders `+N` —
  **unsound** (no group at-or-after this position) ⇒ the validator rejects it ⇒ **spf spike**
  (the [[project_gen_side_no_lacks_fact_branch_prune]] / `.4.8` A′ class).

Backward `-N` ALONE is soundly generatable, but cannot be isolated from forward: there is no
generation branch-suppression mechanism (`@gen_weight`/`@gen_never` do not exist), and any
scs-specific signed rule loses the shared-`signed_digits`-via-subroutines witness the current
(unsplit) design relies on. Forward `+N` valid ⟺ `prior_count + N ≤ full_count` ⟺ "≥ N groups
defined AT-OR-AFTER this position" — a SUFFIX/forward count no predicate expresses and the
reach-driver cannot synthesize.

## Decision / how to apply

- **The scs numeric migration is a UNIT deferred to `.4.7.c`**, to land when a parser-agnostic
  **forward/suffix-count** primitive exists (which ALSO unblocks sound forward generation +
  witnessing). Do not attempt a partial (absolute-only / absolute+backward) split — each still
  creates a scs-specific signed rule that the generator cannot witness soundly.
- **General rule:** before splitting a rule that is generated via a whole-render-replace
  `@gen_predicate` value-draw (`IndexUpTo`/`NameOf`), check whether the split's sub-rules can be
  SOUNDLY GENERATED. If a sub-rule's only sound value depends on the SUFFIX (later input), it is
  un-witnessable and the split will regress `fully_certified` or `spf`. Sound generation is the
  gate on migrating a validator check into a per-branch grammar predicate.
- **Corollary:** a validator→grammar migration that carries ZERO correctness value (the
  validator is already correct) is not worth a `fully_certified`/`spf` regression. Fix the real
  bug at the current owner (here: the `(*scs:(+0))`/`(*scs:(-0))` relative-zero accepts-invalid,
  fixed with one validator guard, ledger `REGEX-0113`) and defer the migration.

Reinforces [[feedback_correctness_before_speed]] (fully_certified is a correctness/coverage
floor, not tradeable) and [[project_gen_side_no_lacks_fact_branch_prune]] (gen-side
over-generation is the recurring trap). See `REGEX-PCRE2-FIDELITY.4.7.b`/`.4.7.c`.
