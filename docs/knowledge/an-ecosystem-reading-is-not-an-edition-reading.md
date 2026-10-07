---
id: an-ecosystem-reading-is-not-an-edition-reading
title: An ecosystem reading is not an edition reading — before a spelling is called a tolerance, every in-repo edition's own syntax box is read, because the newer edition may derive what the older one forbade
answers:
  - "a spelling every simulator accepts and the LRM does not derive — is it a tolerance"
  - "why was the leading-zero sized literal 01'b0 moved from the tolerant profile to the sv_2023 reading"
  - "does IEEE 1800-2023 derive a size with leading zeros"
  - "what must be checked before admitting a spelling to the opt-in tolerance profile"
  - "how did a release design get refuted by its own pins' citations"
  - "which editions read size ::= non_zero_unsigned_number and which read size ::= unsigned_number"
tags: [systemverilog, lrm, editions, tolerance-profile, citations, release-ceremony, pre-registration]
date: 2026-10-07
status: current
evidence: "SV-CORPUS-GRAD.47 (`PGEN-SV-CORPUS-GRAD-0432`, release 1.0.251). The v1 design admitted `01'b0` to the tolerant profile's closed list on an ecosystem reading (Verilator 5.050 and Icarus 13.0 accept it; `.46`) and a citation copied from 1800-2017 (`IEEE 1800-2023 5.7.1 the same`). Checking the pins' citations against the 2023 text: docs/systemverilog/2023/md/section-5-lexical-conventions.md:213 and section-Annex_A-normative-formal-syntax.md:3200 read `size ::= unsigned_number`, and `grep -rn non_zero_unsigned_number docs/systemverilog/2023/md/` finds nothing — the edition dropped the production; its 5.7.1 prose (:258) keeps the size nonzero. The tolerance chain was stopped at its AST sweeps and the release re-run as an edition twin of integral_number gated to sv_2023; ledger SV-0120 (release 1.0.235) had claimed the three editions agree and is corrected forward."
reverify: "grep -n 'size ::=' docs/systemverilog/2023/md/section-5-lexical-conventions.md docs/systemverilog/2017/md/section-1024-characters-if-an-identifier-exceeds-the-implementation-specific-length-limit-an-error-shall-be-reported.md docs/verilog/2005/md/section-3-lexical-conventions.md   # 2023: unsigned_number; 2017 and 2005: non_zero_unsigned_number. Then: rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_lit_size_leading_zero_sv2023.sv --profile sv_2023   # parse_full passed; --profile sv_2017 rejects"
---

**A tolerance is a spelling NO edition derives.** The opt-in profile's closed list exists for text the big
simulators accept and the standard's text speaks against (`docs/decisions/project_sv_opt_in_tolerance_profile.md`).
"No edition" is three editions here — 1364-2005, 1800-2017 and 1800-2023, all in the repository — and the
check is three syntax boxes, not one plus an assumption. `SV-CORPUS-GRAD.46` measured `01'b0` against two tools
and found both accept it; the pins then cited 1800-2017 §5.7.1 and wrote *"IEEE 1800-2023 5.7.1 the same"*. It is
not the same: 1800-2023 reads `size ::= unsigned_number` in both its §5.7 syntax box and its Annex A, and the
production `non_zero_unsigned_number` does not exist in that edition. The spelling is standard text under 2023.

- **The consequence is not cosmetic.** A tolerance lives in the opt-in profile; an edition difference lives in the
  strict profile of its edition. Shipping the first for a spelling that is the second leaves `sv_2023` rejecting
  legal 2023 text — a REJECTS-VALID defect wearing a feature's name — and publishes a book sentence ("no edition
  derives it") that is false. Release `1.0.235` had done the narrower version of the same thing: it refused the
  spelling on every profile and recorded that the three editions agree.
- **Where it was caught, and why there.** Not by any gate — every strict instrument was green on the tolerance
  bytes, because a tolerance moves nothing the strict profiles measure. It was caught while the pins' citations were
  being verified before the release documents were written: a citation is a claim about a LINE, and reading the
  line it named in the 2023 file showed a different production. The pre-release citation check is where an
  edition-blind reading surfaces; the LRM-CLAUSE-CITATION census checks that a clause number encloses its cited
  line, not that the sentence quoted is the same in another edition.
- **The rule.** Before a spelling is proposed for the tolerant list — or refused under every profile — read the
  production in each edition's own syntax box and grep each edition for the production's NAME. A production
  that is absent from an edition is the strongest signal that the edition changed the rule. A prose sentence
  that survives (here: "a nonzero unsigned decimal number") may keep a SEMANTIC bound the syntax no longer
  spells (a zero size), and the parser keeps that bound on the prose.
- **The ecosystem reading still earns its place.** Two tools accepting the spelling is why it was looked at;
  the editions' text is what decides what it IS. An instance of
  [[an-unmeasured-honest-bound-is-an-unverified-claim-wearing-a-humble-face]] and of the pre-registration discipline: the
  design was written before the measurement, the measurement refuted the design, and the record says so.
