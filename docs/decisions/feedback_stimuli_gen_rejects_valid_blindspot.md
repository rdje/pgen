# ⛔ Director observation (2026-07-21, session #188): the stimuli generator has a REJECTS-VALID blind spot — improvement REQUIRED, deferred to a future activity

## What the director said (verbatim intent)

> This matching error is embarrassing to me, because the stimuli gen shall have
> been able to uncover those type of errors. Meaning the stimuli gen needs some
> improvement. We will tackle that at another point in time, not now. I am just
> making observation which will lead to actions later.

Context: `PGEN-RGX-0089` — the `(?[\b])` extended-class backspace escape
regressed REJECTS-VALID at release `1.1.82` (`.3.13`, commit `40202cb0`) and
survived undetected through `1.1.105` until the DOWNSTREAM consumer (RGX) hit it
with its own shipped fixtures. Every upstream gate stayed green the whole time.

## Why the current apparatus is structurally blind to this class (tool-backed)

1. **The stimuli generator derives from OUR grammar.** It can only emit strings
   our grammar derives. A rejects-valid defect is precisely a derivation our
   grammar LACKS (`\b` inside `(?[...])` has no derivation post-`.3.13`), so
   grammar-derived generation can NEVER produce the witness. Cert-coverage,
   duality hunting, and spf all inherit this blindness: they check
   generator⊆parser and parser-side reachability — both are consistency checks
   against ourselves, not against PCRE2.
2. **The external oracle corpus is fixed.** The PCRE2 conformance corpus
   (`2189` patterns) simply has no `(?[\b])` cell (the RGX report verified
   this), and the oracle gate is bounds-based on that fixed corpus. A
   rejects-valid regression on a construct outside the corpus is invisible.
3. Ergo: accepts-invalid and duality breaks are well-covered today;
   **rejects-valid detection has NO generative instrument** — it currently
   relies on corpus luck or downstream consumers (exactly what happened).

## The action this observation will lead to (LATER — not now, per the director)

Candidate instruments for the future activity (to be adjudicated when opened):

- **Oracle-side generation**: generate stimuli from the ORACLE's description of
  the language (e.g. a PCRE2-syntax-derived generator, or pcre2test's own test
  corpora beyond the current snapshot) and require OUR parser to accept whatever
  the oracle accepts — the mirror image of today's generator⊆parser duality.
  This is the only shape that catches "our grammar lacks a derivation".
- **Corpus enrichment ratchet**: every fidelity slice that touches an escape /
  class / construct family must add its oracle matrix cells to the persistent
  conformance corpus so the gate's blind spots shrink monotonically (the
  `(?[\b])` cell would have been added by the 1.1.92 class-escape migration).
- **Differential fuzzing lane**: random/mutational pattern generation compiled
  against BOTH pcre2 and PGEN, diffing verdicts (catches both directions).

Cross-references: `STIMULI-SIGNOFF` (the generator-signoff vision tree — the
natural home for the future activity), `REGEX-PCRE2-FIDELITY.4.8.1` (the
accepts-invalid generator over-approximation leaf — the OTHER direction),
`docs/tasks/RGX-0089.md` (the incident that triggered this observation).

## Status

- Recorded 2026-07-21. **NO work opened** — explicit director instruction:
  observation now, action later. When the director opens the activity, start
  from this note.
