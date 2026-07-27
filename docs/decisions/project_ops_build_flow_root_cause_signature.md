---
name: project-ops-build-flow-root-cause-signature
description: DOCTRINE EXTENSION (2026-07-27, session #216, `GENERATED-LINT-CORRECTNESS.4`) — the chartered "fifth family = controlled differential" is REFUTED by measurement (it would admit 2 of 304 boxes); the real gap is 94% PLACEMENT, group 2 named the wrong profilers, and the genuine fifth family is OPS/BUILD-FLOW — a family the corpus itself requested in a hand-written waiver note nobody read.
metadata:
  node_type: memory
  type: project
---

**What this record settles.** `GENERATED-LINT-CORRECTNESS.3` routed an open question to `.4`:
*is a "source citation plus a controlled experiment" a FIFTH diagnosis-signature family, or is it
prose that should be required to cite a tool?* The director delegated the decision
(*"You know the objective of the project, the north star too, so decide"*). This is the decision,
and it is **not** the one the charter proposed.

## ⛔ The charter's own hypothesis is REFUTED — measured, not argued

`.3` proposed making the fifth group a CONJUNCTION: a **SITE** (`file.ext:NNN`) **AND** a
**CONTRAST** (a quoted arm-vs-arm outcome difference). Priced against every ticked ROOT CAUSE box
in `docs/tasks/` (driver:
`docs/tasks/artifacts/generated_lint_correctness/run_root_cause_box_census.sh`, which sources
`DIAGNOSIS_SIG` and `ROOT_KW` live out of the enforcer so the census can never measure a different
rule than the gate applies):

| candidate | admits |
|---|---|
| **SITE ∧ CONTRAST — the charter's hypothesis** | **2 of 304 (0.7%)** |
| CONTRAST alone | 15 of 304 (4%) |
| SITE alone (⛔ forbidden — "cite a line number") | 134 of 304 (44%) |

⇒ the conjunction is not a fifth family; it is **one sample generalised into a rule**. `.3` picked
`QUANT-PLUS-ITER.md:723` and labelled it *"the cleanest example"* — designing from the cleanest
sample instead of the corpus, the exact failure
[[feedback_read_prior_art_before_designing]] names. Measuring first caught it.

`.3`'s second hypothesis — that `docs/tasks/RGX-0078.md`'s **144** unbacked boxes meant *either*
"SPEED leaves diagnose by differential not profiler" *or* "group 2's tokens do not match how
profiler work is written here" — is **also** refuted in its stated form: only **6 of those 144**
mention `/usr/bin/sample` at all, and only **15 of 304** state any contrast. Neither.

## ⭐⭐ The real shape of the gap: 94% is PLACEMENT, not signature

For every unbacked box, does its own FILE carry a diagnosis signature somewhere else?

| | count |
|---|---|
| **OUT-OF-BOX** — the leaf DOES hold tool evidence, just not inside the ticked bullet | **288 of 304 (94%)** |
| **NO-EVIDENCE** — the whole leaf file carries no diagnosis signature at all | 16 of 304 (5%) |

⇒ the gap `.3` opened is the **direct and correct consequence of box-scoping**. `.3` moved the
requirement from *"the leaf shows tool output"* to *"the ticked bullet quotes tool output"*, and
94% of the historical corpus satisfies the former, not the latter. **No token set can close a
placement gap** — which is precisely why every candidate priced at 0.7–3.6%. Adding a fifth family
to fix this would have been solving the wrong problem with the wrong instrument.

⇒ **The 288 are left AS-IS and deliberately NOT back-filled.** The rule binds new commits only;
back-filling 49 historical records to satisfy a rule that did not exist when they were written is
back-dating the record. The precedent is `LEX-ADJACENCY.1`, whose PRIOR ART was retro-fitted **and
marked as such** so the failure stays visible.

## ✅ What DID change (two narrow, non-weakening corrections)

**(1) GROUP 2's tool vocabulary was factually wrong for this repository.** It named
`cargo flamegraph` / `self-time` — a generic Rust-profiler vocabulary. Measured: group 2 backed
exactly **2** boxes repo-wide, while the SPEED tree itself (153 such boxes) root-causes with macOS
`/usr/bin/sample`, `otool -tV` annotated disassembly, `spindump` / `filtercalltree`, a process-local
`ITIMER_PROF` sampler, and PGEN's own `--dump-rule-outcome-counts-json`. Those are verbatim tool
invocations on exactly the same footing as `cargo flamegraph`, so naming them is a **correction of a
bar aimed at the wrong tools, not a relaxation of it**. Left uncorrected, a SPEED leaf using this
repo's real profiler is forced to waive — which is how a gate teaches authors to bypass it.

**(2) A genuine FIFTH family — OPS / BUILD-FLOW.** The 16 NO-EVIDENCE boxes are dominated by
defects in the repository's **own operational surface**: a Makefile recipe that swallows a nonzero
exit (`RGX-0090`), a version gate that passes vacuously (`RGX-0091`), an `execve` argument list
overflowing `ARG_MAX` (`SV-REPLAY-DEBT`), awk array auto-vivification in the memory guard
(`OPS-MEMSAFE`), untracked-residue hygiene (`REPO-HYGIENE`). Such a defect has **no parse to trace,
no run to sample, no compiler error (it is shell/make, not Rust) and no codegen emission** — the
*identical* argument the record already used to admit groups 3 and 4
([[project_build_integrity_compiler_root_cause_signature]],
[[project_codegen_emission_root_cause_signature]]).

⭐⭐ **This family was requested BY THE CORPUS ITSELF, in prose, and nobody read it.**
`docs/tasks/RGX-0090.md:131` carries a hand-written waiver note *inside the ticked box*, verbatim:
*"like RGX-0091 this is a BUILD-FLOW defect — the parse/perf diagnosis-toolbox signatures do not
apply"*. `docs/tasks/RGX-0091.md:119` wrote *"Diagnosis tool signatures: `grep -n …`"* and got no
credit. An author telling the gate it does not model their defect class is the strongest possible
evidence for a missing family, and it sat unread for months.

*Why these tokens.* Same discipline as groups 3 and 4 — verbatim invocations and errno names, so
quoting one means the tool was actually run: `git ls-files|log -S|rev-list|fsck|reflog|diff-tree|
merge-base|cat-file`, `shellcheck`, `bash -n `, `make -n `/`make --dry-run`, `E2BIG`/`ENOSPC`/
`EACCES`/`ARG_MAX`, and the memory guard's own marker output (`guard.<pid>.marker`,
`reason=rss-budget|free-floor|disk-floor|timeout|none`).

⛔ **Deliberately EXCLUDED as too loose: a bare `grep` mention.** Measured: it matches **16** real
boxes on prose such as *"verified by grep"* — a claim, not tool output. Admitting it would be the
"cite a line number" degradation in another costume.

## Calibration — is a 6-box family too small to be worth it?

What backs the boxes that pass today, per group:

Measured on the corpus **at `2eed59b6`**, in a pristine worktree so this leaf's own checklist box
cannot inflate its own justification:

| group | backs |
|---|---|
| 1 correctness | 48 |
| 2 performance — **before** the vocabulary correction | **2** |
| 2 performance — **after** | **14** |
| 3 build-integrity | **2** |
| 4 codegen-emission | **1** |
| 5 ops/build-flow (new) | **7** |

⇒ group 3 was admitted on the strength of ONE leaf and group 4 on ONE. The corrected group 2 goes
**2 → 14**, and the new ops family backs **7** — the third-largest family in the gate, and more
than groups 3 and 4 combined. The precedent is met several times over.

## Verification (RED / GREEN / CONTROL, with a real before→after)

Driver: `docs/tasks/artifacts/generated_lint_correctness/run_diag_evidence_family5_probes.sh`,
**13/13**. It accepts `PGEN_DIAG_CHECK_OVERRIDE` so the identical arms replay against the older
enforcer — the before→after is produced, not asserted:

| | RED (must BLOCK) | GREEN (must ALLOW) | CTRL |
|---|---|---|---|
| enforcer at `2eed59b6` (before) | 4/4 block | **0/5 — all five BLOCKED** | 4/4 |
| enforcer after `.4` | 4/4 block | 5/5 allow | 4/4 |

⭐ **The RED and CONTROL arms are byte-identical in both runs** — that is the non-weakening proof.
RED-2 (a bare `file.rs:NNN` citation) and RED-3 (*"verified by grep"*) are refused before and
after; RED-4 proves box-scoping still binds the new family. GREEN-3/4/5 quote the **real** box text
from `RGX-0090` / `SV-REPLAY-DEBT` / `REPO-HYGIENE` rather than synthetic prose, because the claim
under test is *"the corpus this family was designed for now passes"*.

Measured effect on the corpus: backed boxes **52 → 68**; the NO-EVIDENCE residual **16 → 9**.

## ⚠️ Honest limits, stated rather than discovered later

1. **The gate cannot check that the signature MATCHES the defect class.** A parser leaf can satisfy
   it with `git ls-files`, exactly as it can already with `error[E0308]`. This is the pre-existing
   documented limit, widened by one family, not a new hole.
2. **The census over-counts relative to what the gate actually polices.** `check_diagnosis_evidence.sh`
   only demands a checklist when a `code_changed` path is staged, so "304 unbacked boxes" is *not*
   304 commits that would have been blocked. It is the box corpus, measured; several of those boxes
   live in docs-only slices the gate never ran on.
3. **Box-scoping has almost no operational history.** It landed at `5c5a0ca0`, two commits before
   this one, so the 94% placement figure describes a corpus written under the OLD rule.
4. ⛔ **A separate, larger defect was MEASURED here and deliberately NOT fixed** — see below.

## ⛔ Routed, not fixed: the gate does not look where the rot is

While pricing the ops family, `code_changed` was measured across all **2,618** commits:

- **521** commits touched the ops surface (`scripts/`, `rust/scripts/`, Makefiles, `.githooks/`,
  `.github/workflows/`, `rust/build.rs`);
- **397 of those 521 (76%)** staged **no** `code_changed` path ⇒ the acceptance checklist was
  **never required** for them.

That includes `5c5a0ca0` itself, which shipped a new gate, a tracked contract, Makefile lanes, a
CI workflow and a change to the doctrine enforcer — with the checklist not required.

⇒ **the acceptance gate is blind to precisely the surface where this session found three separate
rot classes** (`ast_dump_contract_gate` red and unrun; `PGEN_CLIPPY_GENERATED_STRICT` set by
nothing; `ci_workflow_local_gate` unable to complete for 1,371 commits). This is the same family as
[[project_codegen_emission_root_cause_signature]]'s *"a check that cannot run, or cannot see, must
SAY SO, not return green"* — here it is *cannot see*. Widening `code_changed` has real blast radius
(it newly binds ~397 commits' worth of change classes) and belongs to its own leaf, so per the
`DOCTRINE-GAP-OWNERSHIP` rule it is **owned here and fixed there**: `GENERATED-LINT-CORRECTNESS.5`.

**Ratification pending.**
