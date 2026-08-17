# ENGINE-UNIVERSAL-SERVICES.32 — the three measurements the fix rests on

⛔ These are ONE-SHOT measurements, deliberately not wired into the self-test. Each costs a real
build or a hand-constructed tree state, and re-running them per commit would buy nothing: the
per-run guard is `--self-test`'s refusal arm (9/9), and what is recorded here is *why that arm is
the right arm*. Recorded 2026-08-17, session #243.

---

## M1 — the false pass, constructed and observed

**State constructed:** `generated/`'s annotation PAIR at HEAD's emission (correct); the eight family
artifacts + `ebnf.rs` left at the PREVIOUS emission (per-site `-o` literals, pre-`.31` slice 2); and
`rust/target/debug/ast_pipeline` replaced by the generator that produced them.

**Command:** `bash scripts/check_generated_reproducibility.sh --verify`

```text
generated-reproducibility: TIER 2 — re-deriving every artifact from HEAD
building ast_pipeline_bootstrap from HEAD…
  ✓ return_annotation              re-derives byte-identically (1 sites)
  ✓ semantic_annotation            re-derives byte-identically (1 sites)
  ✓ json                           re-derives byte-identically (208 sites)
  ✓ regex                          re-derives byte-identically (11371 sites)
  ✓ systemverilog                  re-derives byte-identically (34738 sites)
  ✓ systemverilog_preprocessor     re-derives byte-identically (821 sites)
  ✓ vhdl                           re-derives byte-identically (2850 sites)
  ✓ rtl_const_expr                 re-derives byte-identically (557 sites)
  ✓ rtl_frontend                   re-derives byte-identically (2339 sites)
  ✓ scratch                        re-derives byte-identically (69 sites)
generated-reproducibility: TIER 2 OK — every checked artifact is what HEAD produces
GATE-EXIT=0
```

HEAD's emitter produces **1** embedded `-o` site per artifact. Eight rows carry 69–34 738 and every
one passed. ⭐ **The run's own output contains the disproof of its headline**: the two PAIR rows read
`1 sites` because their generator is built from HEAD inside the gate; the eight family rows do not,
because theirs was taken from disk unverified.

⛔ An earlier attempt at this demonstration left the **pair** stale too, and the gate correctly
REFUSED (exit 2, *"embedded -o sites live=640 fresh=1 … any verdict would measure the PATH"*). That
refusal is the pair's protection working, and it is why the demonstration has to leave the pair
CORRECT: the finding is about the family cohort specifically, and a run that aborts on the pair
never reaches it.

---

## M2 — cargo detects the real staleness mechanism

The mechanism that produces a stale generator in practice is `make` skipping the rebuild on GNU Make
3.81's whole-second mtime comparison (`CI-PARITY-GATE-ROT.37`) — i.e. **an emission source is newer
than the last cargo build, and cargo was never invoked**. Reproduced by perturbing one emission
source and running the gate:

```bash
printf '\n// one-shot currency probe\n' >> rust/src/ast_pipeline/ast_based_generator.rs
bash scripts/check_generated_reproducibility.sh --verify
grep -E 'Compiling pgen|Finished' rust/target/generated_reproducibility/pipeline_build.log
```

```text
   Compiling pgen v1.0.0 (/Volumes/SSD/Documents/github/pgen/rust)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 41.66s
…
generated-reproducibility: TIER 2 OK — every checked artifact is what HEAD produces      (rc 0)
```

⇒ cargo **rebuilt** rather than reusing the binary, and the gate then verified all ten artifacts
against a current generator. The perturbation was a comment, so the emission is unchanged and the
artifacts stay byte-identical — which is the point: the gate now re-derives with a **current** tool
whether or not the change was semantic. Source reverted and rebuilt (40.6 s) immediately after.

---

## M3 — the price of the currency step

| state | cost |
|---|---|
| binary already current (the ordinary case) | **0.8 s** |
| one emission source moved, incremental rebuild | **41.7 s** |
| candidate (A), cold build into the gate's own `CARGO_TARGET_DIR` | rejected: full ~216 MB cold build every run |

⚠️ A first measurement of the "no-op" case read **5 m 32 s** and was **wrong as a price for this
step**: the tree's binary had been hand-`cp`'d during M1's construction, so cargo's fingerprint was
genuinely stale and it did real work. The honest no-op figure is the second measurement, taken on a
tree cargo itself had just built. ⇒ *price a no-op only on a tree the tool itself last touched.*

---

## Why not the fingerprint candidate — the decision, not just the outcome

Candidate (B) was to have `build.rs` publish an `emission_sha` into the binary, mirroring
`ENGINE-UNIVERSAL-SERVICES.24`'s parser fingerprint, and compare it against the digest tier 1
already computes. It is **rejected on design rather than on price**: the gate derives `emission_sha`
from `git ls-files`, and a `build.rs` cannot, so the two would be **separate implementations of one
digest that must agree**. This repository has paid for that class four times — the `2.741`
classifier written from its own design's prose, the carried `43 615`, four stale prose copies of one
number, and the `128`-vs-`127` denominator. Cargo has no second implementation to drift.
