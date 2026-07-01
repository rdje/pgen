# VERILOG-2005-PROFILE: a strict IEEE 1364-2005 (Verilog) parsing profile on the SystemVerilog grammar

## Metadata

- Tree ID: `VERILOG-2005-PROFILE`
- Status: `active`
- Roadmap lane: dialect-profile surface for the EBNF-driven SV grammar (Verilog ⊂ SystemVerilog)
- Created: `2026-07-01`
- Owner: repo-local workflow
- Director ask (2026-07-01): "Verilog LRM IEEE 1364-2005 being a subset of SystemVerilog LRM
  IEEE 1800-2017/2023, we will need another profile to restrict the parsing to Verilog … will
  you need any extra PDF … or are the SV 2017/2023 enough?" — scope as a task-tree.
- Related: `docs/tasks/PNR-AUX-READERS.md` (weighs reusing `verilog_2005_lrm_extracted.ebnf`
  for gate-level-netlist reading), `docs/tasks/VERILOG-AMS.md` (notes the shared
  `verilog_2005`/`systemverilog` lexical model + a reuse-vs-fork decision),
  `grammars/systemverilog.ebnf` (the mature dual-profile SV grammar this profile extends).

## Goal

Add a strict `verilog_2005` (IEEE 1364-2005) parsing profile so PGEN can accept **only** the
Verilog-2005 language and **reject** SystemVerilog-only constructs, consistent with the way the
project already gates dialects (the `@profiles: [...]` semantic-annotation surface +
`rust/src/embedding_api.rs` `GrammarProfile`). Verilog-2005 is very nearly a strict syntactic
subset of IEEE 1800, so the natural, lowest-cost, architecture-consistent realization is a
**third profile on the existing closure-grade `grammars/systemverilog.ebnf`**, using
`grammars/verilog_2005_lrm_extracted.ebnf` as the authoritative in-subset **oracle** — NOT a
separate forked parser (see Decisions). The value of the profile is enforceability: plain
Verilog already parses under `sv_2017` because it is a subset, so the profile's job is to
REJECT the SV-only surface (and correctly re-admit the SV-only keywords that are legal
Verilog-2005 identifiers).

## Non-Goals

- NOT Verilog-AMS (analog/mixed-signal — its own tree `VERILOG-AMS`).
- NOT a gate-level structural netlist subset (that reuse question is `PNR-AUX-READERS`).
- NOT a separate standalone `verilog_2005` grammar/parser forked from the raw extracted EBNF
  (rejected in Decisions — duplicates the mature shared SV core + re-runs the whole
  annotation/closure campaign the SV grammar already passed).
- NOT IEEE 1364-1995 (an even older subset); scope is 1364-2005 only.

## Source materials — ALREADY IN THE REPO (no extra PDF needed)

Tools-first inventory (2026-07-01) — the authoritative delineator of the Verilog-2005 boundary
is present; the SV 2017/2023 LRMs alone are **insufficient** because the merged IEEE 1800 Annex A
grammar does not mark which productions/keywords are inherited from 1364-2005 vs added by SV.

- `docs/verilog/2005/Verilog-LRM-IEEE-1364-2005.pdf` — the IEEE 1364-2005 LRM (6.5 MB; the
  authoritative source for the production subset AND the reserved-keyword set / Annex B).
- `docs/verilog/2005/md/` + `docs/verilog/2005/txt/` — the same LRM extracted to markdown/text
  (same pipeline as the SV LRMs under `docs/systemverilog/2017|2023/`).
- `grammars/verilog_2005_lrm_extracted.ebnf` — a raw LRM-extracted grammar skeleton
  (1529 lines, 476 productions, BNF-form `::=` + `[ … ]` optionals; NOT the active-grammar
  `:=`+annotation form). This is the **oracle** for which constructs are in-subset.

## Acceptance Criteria

- A `verilog_2005` profile is registered end-to-end: `GrammarProfile` variant +
  `parse_grammar_profile` aliasing (`verilog_2005` / `ieee1364-2005` / …), and the SV grammar's
  `@profiles: [...]` gating admits the Verilog-2005 core while excluding the SV-only surface.
- STRICTNESS proven both ways on a curated conformance corpus:
  - every Verilog-2005 construct in the corpus PARSES under `verilog_2005`;
  - every SV-only construct (class/package/interface/program, `logic`/`bit`/`int`/…,
    `always_ff`/`_comb`/`_latch`, assertions, covergroups, `++`/`--`, `do-while`, …) is REJECTED
    under `verilog_2005` while still parsing under `sv_2017`/`sv_2023`.
- KEYWORD-RESERVATION divergence handled: SV-only keywords that are legal Verilog-2005
  identifiers (`logic`, `bit`, `byte`, `int`, `do`, `final`, `bind`, …) parse as identifiers
  under `verilog_2005`.
- NO REGRESSION to `sv_2017`/`sv_2023`: SV external corpus 14/14; SV cert unchanged; the 6
  fully-certified grammars byte-identical; `cargo test --lib` green; clippy source-clean.
- Full COMMIT.md lockstep: SV integration contract (new profile), the SV parser book
  (a `verilog_2005` profile chapter/section), CHANGES/DEVELOPMENT_NOTES/MEMORY/LIVE, and a
  `verilog_2005` cert-coverage / corpus gate as the closure surface.

## Task Tree

- ID: `VERILOG-2005-PROFILE.1`
  Status: `in_progress` (2026-07-01) — SCOPING / DESIGN leaf, **tools-first, NO code**.
  Goal: produce the design that the implementation leaves execute. Deliverables:
  1. **Subset-boundary derivation (the oracle map).** Systematically diff/map
     `grammars/verilog_2005_lrm_extracted.ebnf` (476 productions) against the active
     `grammars/systemverilog.ebnf` rule set to classify every SV rule/branch as
     `verilog_2005-core` (keep) vs `sv-only` (gate out) vs `divergent` (needs adaptation).
     Tools-first (a mapping script / the linter / grep), NOT eyeballed. Record the SV-only
     construct families to gate.
  2. **Keyword-reservation delta.** From the 1364-2005 LRM Annex B (keyword list) vs the SV
     keyword set, enumerate the SV-only reserved words that are legal Verilog-2005 identifiers,
     and design how the `verilog_2005` profile re-admits them (profile-gated keyword rules /
     the existing `non_keyword_identifier` exclusion mechanism — audit `non_keyword_identifier`
     + the `kw_*` fused-token rules).
  3. **Gating mechanism decision.** Confirm the `@profiles: [...]` allow-list model can express
     "core rule active in `{sv_2017, sv_2023, verilog_2005}`; SV-only rule active in
     `{sv_2017, sv_2023}` only" — verify how an UN-annotated rule's default profile membership
     behaves (all-profiles?) so gating is additive and low-risk. Decide whether the core rules
     get an explicit 3-profile allow-list or whether SV-only rules get an explicit 2-profile
     allow-list (minimize edits + blast radius).
  4. **Closure bar + first slice.** Define the `verilog_2005` cert-coverage/corpus closure
     surface (a curated 1364-2005 accept corpus + an SV-only reject corpus) and pick the first
     narrow implementation slice (likely: register the profile + gate ONE unambiguous SV-only
     family, e.g. `class_declaration`, and prove accept/reject both ways).
  Output: the Decisions + a concrete `.2`.. implementation-leaf plan appended to this tree.

- ID: `VERILOG-2005-PROFILE.2` … (implementation leaves)
  Status: `open` — defined by `.1`'s output. Expected shape: register the profile
  (`GrammarProfile` + aliases) → gate SV-only families in dependency order → keyword re-admission
  → curated conformance corpus + cert/corpus gate → LIVE + contract + book lockstep. One family
  (or small cluster) per leaf, accept/reject proven tools-first per leaf.

## Decisions

- 2026-07-01: **A profile is the correct mechanism** (not a bespoke validator, not a fork). This
  matches the project's existing dialect-gating architecture (`@profiles:` + `GrammarProfile`)
  and the [[project_ebnf_is_single_source_of_truth]] doctrine — the subset boundary lives IN the
  EBNF via profile-gating annotations, not in out-of-band code.
- 2026-07-01: **Profile-on-the-SV-grammar, not a separate parser.** The SV grammar is mature /
  closure-grade with a shared lexer + full infra; Verilog IS a subset, so a third profile reuses
  all of it. A fork from `verilog_2005_lrm_extracted.ebnf` would duplicate the shared core and
  re-incur the entire annotation/closure/cert campaign. The extracted EBNF is retained as the
  in-subset **oracle**, not wired as a parser. (Consistent with the reuse lean already noted in
  `PNR-AUX-READERS` / `VERILOG-AMS`.)
- 2026-07-01: **No extra document required.** The IEEE 1364-2005 LRM PDF + extracted md/txt +
  extracted EBNF are already in the repo (see "Source materials"). The SV 2017/2023 LRMs alone
  would NOT suffice to draw the boundary (the merged Annex A doesn't mark 1364-2005 provenance).
- 2026-07-01: **It is not pure subtraction.** Two axes: (a) gate OUT SV-only constructs
  (additive `@profiles:` gating — the easy axis); (b) UN-reserve the SV-only keywords that are
  legal Verilog-2005 identifiers (the non-subtractive axis — the 1364-2005 keyword annex is the
  authority). Both are required for a faithful profile.

## Open Questions

- Exact default-profile-membership semantics of an UN-annotated rule (all-profiles vs none) —
  resolved in `.1` step 3 by reading the profile-gating codegen, not assumed.
- Whether any 1364-2005 construct is NOT a clean subset of IEEE 1800 (rare true divergences,
  e.g. a construct legal in Verilog but removed/reinterpreted in SV) — surface via the `.1`
  oracle diff and classify as `divergent`.
- Corpus sourcing: which real Verilog-2005 designs (and SV-only negative cases) form the
  conformance corpus (the repo already vendors SV corpora under `stimuli/sv/`; a Verilog-2005
  accept corpus + an SV-only reject corpus need curating).

## Blockers

- none (all source materials present; composes with the active SV work, no dependency on the
  open `SV-AST-SHAPE-FIDELITY` leaves).

## Verification Log

- 2026-07-01 (`.1`, tools-first inventory): confirmed the 1364-2005 LRM PDF
  (`docs/verilog/2005/Verilog-LRM-IEEE-1364-2005.pdf`), extracted md/txt, and
  `grammars/verilog_2005_lrm_extracted.ebnf` (1529 lines / 476 productions) are all present;
  confirmed the profile surface (`@profiles: ["sv_2017"]`/`["sv_2023"]` in `systemverilog.ebnf`;
  `GrammarProfile::{Sv2017,Sv2023,Vhdl1076_2019,RegexDefault}` in `embedding_api.rs`; the
  `systemverilog_profiles` vec at `:405`). `verilog_2005` is referenced only in planning docs
  (`PNR-AUX-READERS`, `VERILOG-AMS`) — no active profile yet.

## Commit Log

- 2026-07-01: tree created + `.1` scoping opened (`PGEN-VERILOG-2005-PROFILE-0001`, PURE-DOCS —
  scoping only, no code/grammar/generated change).

## Changelog

- 2026-07-01: Tree created to scope a strict `verilog_2005` (IEEE 1364-2005) parsing profile on
  the SV grammar; `.1` design leaf opened with the source-material inventory + the
  profile-on-SV-grammar-with-oracle decision; implementation leaves (`.2`..) deferred to `.1`'s
  boundary-derivation output.
