---
name: feedback_regex_fix_geomean_ratchet_and_fresh_session
description: Director 2026-07-20 — accept a PGEN regex performance fix only when the canonical PCRE2 external-corpus geomean strictly decreases; after one fully implemented fix, commit cleanly and require a brand-new session before another fix
metadata:
  node_type: memory
  type: feedback
---

**Standing director directive (2026-07-20):** every PGEN regex performance fix
is accepted only if the canonical PCRE2 external-corpus parse-time geomean is
strictly lower than its immediate baseline. Equal or higher means the product
change is rejected and reverted. A benchmark-only or modeled improvement is
insufficient. Correctness identity and the existing settled-MAX
non-regression gate remain additional mandatory conditions.

**Fresh-session directive (same date, non-negotiable):** after a single fix has
been fully implemented, verified, corpus-measured, and accepted or reverted,
commit its durable result, restore a clean handoff-ready tree, and stop. The
next implementation fix starts only in a brand-new session. This overrides
PNT/batch continuation between implementation fixes and exists to protect
focus and signoff quality.

**Operational meaning:** one task leaf owns one optimization and one
accept/reject result. If the geomean stays still or grows, only the evidence and
rejection record may commit; no candidate product source remains. Noise may
qualify confidence but cannot reverse the strict numerical direction rule.

The full execution contract is tracked by `PGEN-RGX-0078-0197` in
`docs/tasks/artifacts/held_carrier_batch/execution_contract.md`.

**AMENDED 2026-07-22 (director, post-campaign-closure, verbatim): the achieved
regex speed numbers are a PERMANENT floor.** "PGEN regex parser speed numbers
achieved during the last optimization campaign shall not regress under no
circumstance." Operational meaning: the closure-vintage floor of record —
corpus geomean **1,004.4 ns** (confirmed 991.67 ns raw sub-1 µs on the
preserved probe `preserved_probes/regex_perf_probe_carrier48_8d392176`) and
the settled corpus-MAX bound **425,000 ns** — binds EVERY future change that
could touch regex parse time, not just perf-campaign work: any perf-touching
change must re-prove the floor (paired A/B against the preserved probe, flips
0, MAX within bound, geomean not above the drift-adjudicated floor) or be
reverted. This elevates the campaign-era ratchet from campaign discipline to
standing product law; the "milk on fire" continuous-monitoring directive
([[feedback_correctness_before_speed]]) is its monitoring arm.
