---
id: a-ceremonys-measured-binary-is-kept-not-re-derived
title: A ceremony's measured binary is kept, not re-derived — copy it aside at build time, restore the published path before every measuring step and last, verify at the end; and a memo written by a scratch run lives beside the scratch manifest, never in the tracked file
answers:
  - "the release ceremony's binaries do not survive the ceremony — which steps rebuild them"
  - "a tracked report names a parse binary that no longer exists at its path — what happened"
  - "why is a rebuild of the identical tree not byte-reproducible on this host"
  - "the store-gate memo names a binary the lane report does not — how"
  - "why point the ceremony's tools at the published paths instead of at the copies"
  - "what is scripts/sv_ceremony_binaries.sh and when does the chain call snapshot, restore and verify"
  - "the adjudicator's store-gate cache: where does a scratch run write it"
  - "what does the store-gate bank's A13 arm check"
tags: [systemverilog, ceremony, release, identity, store-gate, provenance, ops]
date: 2026-10-09
provenance: SV-CORPUS-GRAD.53 (PGEN-SV-CORPUS-GRAD-0440); SV-CORPUS-GRAD.52 (the finding), .48 (release 1.0.252, the ceremony it happened in)
reverify: "bash scripts/sv_ceremony_binaries.sh --self-test   # SV-CEREMONY-BINARIES-SELF-TEST: pass=7 fail=0; python3 stimuli/sv/store_gate_explanations.py --check-identity   # STORE-GATE-IDENTITY: OK"
---

# A ceremony's measured binary is kept, not re-derived

**The finding.** Release `1.0.252`'s ceremony built its three binaries once (`rust/target/sv48/sha_final.txt`:
the release probe `6af036ea…`, the debug probe `5a60442e…`) and every lane, adjudication and pipeline
reading named them. Then `verilog_2005_conformance_gate` rebuilt the release probe at the published path
(`b7ce38c4…`, 12:19) and the parity / equivalence gates and the lib suite rebuilt the debug pair (17:53).
A rebuild of the identical tree is **not byte-reproducible** on this host (the identical sources rebuilt to
`c4df243c…` — the same parser fingerprint, different bytes), so by the end of the chain the tracked identity
rows named binaries that existed nowhere. The adjudicator's store-gate REFUSED (correctly); the lowmem copy
saved the release identity; the debug probe had no copy and the three pipeline readings were re-run (`.52`).

**The second half, found by reading the commit.** The release commit's tracked store-gate memo carried
`b7ce38c4…` on its 54 `sv_2017` rows beside a lane report naming `6af036ea…` — and nothing read the column.
`git diff-tree -p 90e09d983 -- stimuli/sv/characterization/store_gate_cache.tsv` shows exactly those 54 rows;
the writer was `rust/scripts/sv_external_corpus_conformance_gate.sh` (inside the SV formal umbrella, 16:52),
which re-measures the sv lane into a SCRATCH report with whatever binary sits at the release path when it
runs, then runs the adjudicator with scratch `--report` / `--out-manifest` paths — and the memo path defaulted
to the TRACKED file. A scratch run mutated a tracked artifact, silently, with an identity the tracked report
contradicts.

**The rule, in three parts.**
1. **Copy aside at build time** — `bash scripts/sv_ceremony_binaries.sh snapshot rust/target/svNN` the
   moment the build chain publishes; `bin/identity.tsv` records each copy's digest and parser fingerprint
   (the two probes must agree, or the snapshot refuses), `sha_final.txt` is the short form the crash-recovery
   procedure reads.
2. **Restore the published paths, never point tools at the copies** — `… restore rust/target/svNN` before
   every measuring step that follows a rebuilding gate and as the LAST step of the chain. The lane runner
   RECORDS the probe path it measured with and adopts it on its next run; a tracked report naming
   `rust/target/svNN/bin/…` would send the next ceremony to a path the §8 cleanup deletes within a week. The
   published paths are the only paths a tracked artifact may name; the copies guarantee what they hold.
3. **Verify at the end** — `… verify rust/target/svNN`: the copies against `identity.tsv`, the published
   paths against the copies, every tracked lane report's `parse binary` row against the copy of its role (raw
   lanes → release, pipeline readings → debug), and the store-gate memo against the lane reports
   (`store_gate_explanations.py --check-identity`, also the store-gate bank's A13 arm on every commit, with a
   flipped-row RED control).

**And the memo follows the manifest it publishes.** `adjudicate_external_corpus.py` resolves its memo path from
`--out-manifest`: the tracked manifest → the tracked memo; anything else → a memo BESIDE that manifest, seeded
from the tracked one so agreeing rows stay hits, the tracked file untouched, and a line on stderr saying so.
The conformance gate passes the sibling path explicitly as well.

**How to apply.** The next ceremony chain (`.50`'s) calls `snapshot` right after its build chain, `restore`
before A1's lanes, before A2's pipeline readings and after the family umbrellas, `restore` + `verify` as the
last steps of B2 before the commit chain. `rust/target/sv53/` holds the `1.0.252` snapshot taken when this
rule landed (the only copy of the debug probe the tracked pipeline readings name).
