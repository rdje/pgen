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
