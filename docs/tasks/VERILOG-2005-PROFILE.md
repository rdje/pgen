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
  Status: `done` (2026-07-01, `PGEN-VERILOG-2005-PROFILE-0002`) — SCOPING / DESIGN leaf,
  **tools-first, NO code**. All four deliverables produced; results in the new
  "`.1` Findings" section below; concrete `.2`.. plan replaces the old `.2` placeholder.
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

- ID: `VERILOG-2005-PROFILE.2`
  Status: `open` (next frontier) — CODE leaf: **register the `verilog_2005` profile end-to-end +
  gate the first unambiguous whole-rule SV-only family (`class_declaration`)**, accept/reject
  proven both ways. Scope (minimal, provable):
  - Register the profile: `GrammarProfile::Verilog2005` variant + `as_str`
    (`rust/src/embedding_api.rs:186/225`), `FromStr` aliases
    `verilog_2005`/`1364-2005`/`ieee1364-2005`/`ieee_1364_2005` (`:282`), the
    `systemverilog` `profile_matrix` binding (`:439`), and the runtime alias normalizer arm in
    `normalize_generated_grammar_profile` (`rust/src/parser_registry.rs:117`).
  - Gate ONE whole-rule family: add `@profiles: ["sv_2017", "sv_2023"]` above
    `class_declaration` (`grammars/systemverilog.ebnf:962`) → regen SV parser.
  - PROVE both ways (tools-first): `class C; endclass` **REJECTS** under `--profile verilog_2005`
    and still **PARSES** under `--profile sv_2017`; a minimal Verilog-2005 module
    (`module m; endmodule`) **PARSES** under `verilog_2005`. Acceptance checklist enforced
    (code leaf): ROOT CAUSE / ADDRESSED / NO REGRESSION (SV corpus 14/14; 6 fully-certified
    byte-identical; `ast_shape_contract` GREEN; clippy clean).
- ID: `VERILOG-2005-PROFILE.3` … (subsequent implementation leaves, one family/cluster per leaf,
  in the order fixed by "`.1` Findings → Implementation order"). Whole-rule gates first (cascade
  automatically, low-risk), then the bare-keyword **branch-lift** gates (shape-preserving, each
  AST-shape-verified), then keyword re-admission, then the closure corpus/gate + LIVE + contract +
  book lockstep. Accept/reject proven tools-first per leaf.

## `.1` Findings (the oracle map + mechanism — tools-first, 2026-07-01)

### Deliverable 3 — gating mechanism (RESOLVED; this is the load-bearing finding)

Read the profile-gating codegen directly (`rust/src/ast_pipeline/ast_based_generator.rs`):
`rule_profiles(rule_name)` (`:7108`) reads a rule's `@profiles: [...]` directive into a
`Vec<String>`; `profile_guard` (`:2692`) emits a guard at rule ENTRY **only when that vec is
non-empty**; `rule_profile_is_enabled` (`:4903`) returns `true` for an empty allow-list, else the
active profile string must match one entry (case-insensitive), and `None` active profile ⇒ `true`.

Consequences that fix the design:
- **An UN-annotated rule is active in ALL profiles** (empty allow-list ⇒ no guard ⇒ always
  enabled). So the low-blast-radius model is **Option B**: annotate only the SV-only rules with
  the 2-profile allow-list `["sv_2017", "sv_2023"]`; the thousands of shared/core rules stay
  un-annotated and remain active under `verilog_2005`. (Option A — a 3-profile allow-list on every
  core rule — is rejected: huge blast radius.)
- **The profile is a RUNTIME selection, not a codegen fork.** `parser_registry.rs:1028` comment +
  `set_grammar_profile`: codegen emits the FULL grammar with runtime guards; the profile is chosen
  at parse time. So a new profile needs NO separate parser — just the annotations + the profile
  registration + a regen.
- **Guard granularity is the RULE, not the branch.** The guard is emitted once at method entry
  (`:2760`). A gated rule immediately returns `Err(Backtrack)`, so an ordered-choice parent simply
  falls through to its next alternative. This gives a **cascade** property (below).

### Deliverable 1 — the subset oracle map (SV-only families to gate)

Tools-first diff of `grammars/verilog_2005_lrm_extracted.ebnf` (509 productions) vs
`grammars/systemverilog.ebnf` (1433 rules). Two gating granularities fall out of the rule-guard
mechanism:

**(1) Whole-rule gates — add `@profiles: ["sv_2017", "sv_2023"]` above the rule (low-risk, no
shape change).** These are SV-only construct entry rules; each verified present at the cited line:

| Family | SV entry rule | line |
| --- | --- | --- |
| Classes | `class_declaration` | 962 |
| Packages | `package_declaration` | 3560 |
| Interfaces / modports | `interface_declaration` / `modport_declaration` | 2514 / 3026 |
| Programs | `program_declaration` | 4059 |
| Interface classes | `interface_class_declaration` | 2461 |
| Assertions (concurrent) | `assert_property_statement` / `assume_property_statement` / `cover_property_statement` / `expect_property_statement` | 547 / 590 / 1486 / 2055 |
| Properties / sequences | `property_declaration` / `sequence_declaration` | 4098 / 4646 |
| Covergroups | `covergroup_declaration` | 1515 |
| Constraints | `constraint_block` | 1405 |
| Bind | `bind_directive` | 614 |
| Nettype | `nettype_declaration` | 3396 (already `@profiles:["sv_2023"]`) |
| Jump | `jump_statement` (return/break/continue) | 2557 |
| Inc/dec | `inc_or_dec_expression` | 2334 |
| Final | `final_construct` | 2098 |
| SV-only integer atoms | `integer_atom_type` (byte/shortint/int/longint — ALL 4 SV-only) | 2433 |

- **CASCADE simplification (verified):** `statement_item_sv_2017` (`:4836`) reaches the SV-only
  statement types via CHILD RULES (`inc_or_dec_expression`@4841, `jump_statement`@4846,
  `procedural_assertion_statement`@4851, `clocking_drive`@4852, `randsequence_statement`@4853,
  `randcase_statement`@4854, `expect_property_statement`@4855). Whole-rule-gating those children
  makes each dispatcher branch backtrack automatically — **no surgery on the dispatcher itself.**
  Same for module/generate/block-item dispatchers that reference the gated entry rules.

**(2) Branch-lift gates — needed ONLY where the SV-only alternative is a BARE KEYWORD TOKEN with no
child rule to gate** (a branch guard does not exist, so the SV-only branch must first be LIFTED into
a named sub-rule that carries the return annotation + the `@profiles` gate — a shape-preserving
refactor, each verified with `--parse-dump-ast` + `ast_shape_contract`, schema unchanged):

| Rule | line | Keep (Verilog-2005) | Lift + gate (SV-only) |
| --- | --- | --- | --- |
| `always_keyword` | 491 | `always` | `always_comb` / `always_latch` / `always_ff` (492–494) |
| `loop_statement` | 2801 | forever/repeat/while/for | `do…while` (2809) / `foreach` (2811) |
| `integer_vector_type` | 2454 | `reg` (2456) | `bit` (2454) / `logic` (2455) |
| `non_integer_type` | 3416 | `real` / `realtime` | `shortreal` (3416) |
| `case_statement` / `conditional_statement` | — | plain forms | `unique`/`priority` qualifier + `case…matches` pattern arm (SV-only) |

Note the double duty of `bit`/`logic`/`byte`/…: gated OUT as **data types** here (axis 1), AND
un-reserved as **identifiers** in Deliverable 2 (axis 2). Both are required.

### Deliverable 2 — keyword-reservation delta (the non-subtractive axis)

Authoritative sets extracted tools-first: IEEE 1364-2005 Annex B keyword list
(`docs/verilog/2005/md/section-Annex_B-normative-list-of-keywords.md`, 123 keywords) vs the SV
`reserved_non_keyword_identifier` negative-lookahead regex (`grammars/systemverilog.ebnf:360`,
`non_keyword_identifier := !reserved_non_keyword_identifier identifier` @334).

~48 words the SV grammar reserves are **NOT** Verilog-2005 keywords, so they must parse as
IDENTIFIERS under `verilog_2005`: `assert assume bit break byte chandle checker class clocking
context continue do endchecker endclass endinterface endpackage endprogram endproperty endsequence
enum expect export foreach import int interface join_any join_none logic longint modport package
program property pure randcase randsequence return sequence shortint shortreal string struct type
typedef union void wait_order`. The ~33 that ARE Verilog-2005 keywords stay reserved (`begin case
casex casez disable else end endcase endfunction endmodule endtask event for forever fork function
generate if integer join localparam module parameter real realtime reg repeat signed task time
unsigned wait while`).

**Design (uses only existing primitives — profile guard + ordered choice):** split the reserved-word
lookahead into two profile-gated variants and select by profile —
`reserved_non_keyword_identifier_sv` (full SV set, `@profiles:["sv_2017","sv_2023"]`) and
`reserved_non_keyword_identifier_v2005` (reduced set, `@profiles:["verilog_2005"]`), with
`reserved_non_keyword_identifier := reserved_non_keyword_identifier_sv |
reserved_non_keyword_identifier_v2005`. Traced through the negative lookahead: under `verilog_2005`
the SV variant backtracks (gated) and the v2005 variant (which omits `logic`, `bit`, …) fails to
match `logic` ⇒ `!reserved…` SUCCEEDS ⇒ `logic` is an identifier; for `module` the v2005 variant
matches ⇒ correctly reserved. Under `sv_2017` the SV variant matches the full set; the v2005 variant
is gated/never reached. No new engine primitive. (The SV-only `kw_*` fused tokens — 289 of them —
need no individual gating: they are only reachable via the construct roots gated in Deliverable 1.)

### Deliverable 4 — closure bar + first slice

- **Closure surface:** (a) a curated `verilog_2005` **conformance corpus** — an ACCEPT set (real
  1364-2005 designs) + a REJECT set (SV-only snippets that MUST fail under `verilog_2005` while
  still parsing under `sv_2017`), run through `parseability_probe --parse systemverilog f.v
  --profile verilog_2005` (the primary accept/reject-both-ways proof, mirroring the existing SV
  external-corpus triage); plus (b) `ast_pipeline … --report-certificate-coverage --grammar-profile
  verilog_2005 --entry-rule systemverilog_file` as the trustworthiness number for the profiled
  grammar (SV-only rules become unreachable/proof under the profile — expected and acceptable).
- **First slice = `.2`** (register the profile + gate `class_declaration`, accept/reject both ways)
  — the minimal end-to-end proof that the mechanism works before scaling to every family.
- **Implementation order (`.3`..):** whole-rule gates in dependency order (cascade, low-risk) →
  bare-keyword branch-lift gates (shape-preserving, AST-verified) → keyword re-admission
  (Deliverable 2 split) → curated corpus + `verilog_2005` cert/corpus gate → LIVE + SV integration
  contract (new profile) + SV parser book (`verilog_2005` chapter) lockstep.

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
- 2026-07-01 (`.1`): **Gate with Option B (2-profile allow-list on SV-only rules), not Option A.**
  Tool-proven: an un-annotated rule is active in all profiles (`rule_profile_is_enabled` empty ⇒
  true), so annotating only the SV-only rules with `["sv_2017", "sv_2023"]` is additive and
  minimal-blast-radius; annotating every core rule with a 3-profile list is rejected.
- 2026-07-01 (`.1`): **Two gating granularities, and prefer whole-rule (cascade) over branch.** The
  profile guard fires at RULE entry, so gating an SV-only entry rule makes every ordered-choice
  parent that references it fall through automatically (verified for the `statement_item_sv_2017`
  dispatcher). Branch-lift (lift an inline `|` alternative into a named gated sub-rule) is used
  ONLY where the SV-only alternative is a bare keyword token with no child rule to gate
  (`always_comb/latch/ff`, `do…while`/`foreach`, `bit`/`logic`, `shortreal`), and MUST be
  shape-preserving (AST verified) per the SV-AST-SHAPE-FIDELITY inline-alt-`$N` cautions.
- 2026-07-01 (`.1`): **Keyword re-admission via a profile-gated ordered-choice split of the reserved
  lookahead** (`reserved_non_keyword_identifier_sv | reserved_non_keyword_identifier_v2005`) — no
  new engine primitive; traced correct through the `!reserved…` negative lookahead both ways.
- 2026-07-01 (`.1`): **Profile is a runtime selection, no parser fork.** Codegen emits the full
  grammar with runtime guards (`parser_registry.rs:1028`); `verilog_2005` needs only the EBNF
  annotations + `GrammarProfile`/alias registration + a regen.

## Open Questions

- ~~Exact default-profile-membership semantics of an UN-annotated rule~~ — **RESOLVED (`.1`)**:
  un-annotated ⇒ active in ALL profiles (`rule_profile_is_enabled` returns `true` on an empty
  allow-list). Gating is therefore additive (Option B).
- Divergences (`.1` oracle diff): the SV-vs-Verilog differences found are **structural, not true
  "legal-in-Verilog-illegal-in-SV" divergences** — SV ADDS `unique`/`priority` qualifiers +
  `case…matches` pattern arms to `case_statement`/`conditional_statement` (branch-lift), and a
  spelling difference (`procedural_continuous_assignment` singular in SV vs
  `procedural_continuous_assignments` plural in the 1364-2005 skeleton — same construct). No
  construct was found that is legal Verilog-2005 yet removed/reinterpreted in IEEE 1800; if one
  surfaces during implementation it is classified `divergent` in its leaf.
- Corpus sourcing (OPEN, for `.2`+): which real Verilog-2005 designs (accept set) and SV-only
  snippets (reject set) form the `verilog_2005` conformance corpus. The reject set is cheap to
  author from the Deliverable-1 SV-only family table; the accept set needs a small curated set of
  pure-1364-2005 modules (plus targeted keyword-as-identifier cases like `reg logic; wire bit;`).

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
- 2026-07-01 (`.1`, DESIGN closure — tools-first, NO code): completed all four deliverables.
  Mechanism (D3) proven by direct code read: `rule_profiles` (`ast_based_generator.rs:7108`),
  `profile_guard` (`:2692/2760`), `rule_profile_is_enabled` (`:4903`) — empty allow-list ⇒ active
  everywhere; explicit list ⇒ off-profile Backtrack at rule entry; profile is a runtime selection
  (`parser_registry.rs:1028`). Oracle map (D1) from `verilog_2005_lrm_extracted.ebnf` (509 prod) vs
  `systemverilog.ebnf` (1433 rules); every cited SV-only entry rule spot-verified present at its
  line (`class_declaration`@962, `package_declaration`@3560, `interface_declaration`@2514,
  `program_declaration`@4059, `bind_directive`@614, `jump_statement`@2557, `inc_or_dec_expression`
  @2334, `final_construct`@2098, …); branch shapes verified for `always_keyword`@491,
  `loop_statement`@2801, `integer_atom_type`@2433, `integer_vector_type`@2454, `non_integer_type`
  @3416, `statement_item_sv_2017`@4836 (cascade confirmed). Keyword delta (D2) from IEEE 1364-2005
  Annex B (123 kw, `section-Annex_B-…md`) vs `reserved_non_keyword_identifier`@360 →
  ~48 SV-only reserved words to un-reserve. Closure bar + first slice (D4) fixed → `.2`. No
  grammar / code / generated / release change (design-only).

## Commit Log

- 2026-07-01: tree created + `.1` scoping opened (`PGEN-VERILOG-2005-PROFILE-0001`, PURE-DOCS —
  scoping only, no code/grammar/generated change).
- 2026-07-01: `.1` DESIGN closed (`PGEN-VERILOG-2005-PROFILE-0002`, PURE-DOCS) — all four
  deliverables landed (mechanism / oracle map / keyword delta / closure bar), Decisions +
  concrete `.2` first slice + `.3`.. ordering appended; `.1` → `done`, frontier → `.2`.

## Changelog

- 2026-07-01: Tree created to scope a strict `verilog_2005` (IEEE 1364-2005) parsing profile on
  the SV grammar; `.1` design leaf opened with the source-material inventory + the
  profile-on-SV-grammar-with-oracle decision; implementation leaves (`.2`..) deferred to `.1`'s
  boundary-derivation output.
- 2026-07-01: `.1` DONE (`PGEN-VERILOG-2005-PROFILE-0002`, PURE-DOCS scoping) — produced the
  design tools-first: (D3) gating mechanism = Option B 2-profile allow-list on SV-only rules,
  rule-granularity guard with cascade-through-dispatchers, profile is a runtime selection (no
  fork); (D1) oracle map of SV-only families split into whole-rule gates vs bare-keyword
  branch-lifts; (D2) ~48 SV-only reserved words to un-reserve via a profile-gated ordered-choice
  split of the reserved lookahead; (D4) closure bar (accept/reject conformance corpus + profiled
  cert) + first slice `.2` (register profile + gate `class_declaration`) + `.3`.. ordering.
  Frontier → `.2` (first CODE leaf).
