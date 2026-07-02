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
  Status: `done` (2026-07-02, `PGEN-VERILOG-2005-PROFILE-0003`) — CODE leaf: **registered the
  `verilog_2005` profile end-to-end + gated the first whole-rule SV-only family
  (`class_declaration`) + admitted `verilog_2005` to the `sv_2017` baseline of the shared core
  `module_declaration_sv_2017`**, accept/reject proven both ways. See "`.2` Findings" for the
  tools-first design correction (the `.1` Option-B "core rules are un-annotated" premise had a hole:
  shared core constructs are profile-SPLIT into `_sv_2017`/`_sv_2023` variants, so a brand-new
  profile that matches neither parses the EMPTY language until it is admitted to the `sv_2017`
  baseline variant). Scope (minimal, provable — as executed):
  - Register the profile: `GrammarProfile::Verilog2005` variant + `as_str`,
    `FromStr` aliases `verilog_2005`/`1364-2005`/`ieee1364-2005`/`ieee_1364_2005`,
    `validate_profile_match` SystemVerilog arm, the `systemverilog` `profile_matrix` /
    `supported_profiles` binding, and `EMBEDDING_API_VERSION` `1.2.0`→`1.3.0` (backward-compatible
    supported-profile addition; schema stays `2`) in `rust/src/embedding_api.rs`; the runtime alias
    normalizer arm in `normalize_generated_grammar_profile` (`rust/src/parser_registry.rs`).
  - Gate ONE whole-rule SV-only family: add `@profiles: ["sv_2017", "sv_2023"]` above the
    `class_declaration` umbrella (`grammars/systemverilog.ebnf`) — excludes `verilog_2005` → reject.
  - **CORRECTED MECHANISM (`.2` finding):** admit `verilog_2005` to the `sv_2017` BASELINE variant of
    the shared core `module_declaration_sv_2017` (`@profiles: ["sv_2017"]`→`["sv_2017","verilog_2005"]`)
    — WITHOUT this, `module m; endmodule` rejects at position 0 under `verilog_2005` because the core
    `module_declaration` umbrella dispatches only to `_sv_2017`/`_sv_2023` gated variants (empty
    language). This is the load-bearing correction to the `.1` Option-B design. → regen SV parser.
  - PROVE both ways (tools-first): `class C; endclass` **REJECTS** under `--profile verilog_2005`
    (rc=1) and still **PARSES** under `sv_2017`/`sv_2023` (rc=0); a minimal Verilog-2005 module
    (`module m; endmodule`) **PARSES** under `verilog_2005`/`sv_2017`/`sv_2023` (rc=0); aliases
    `ieee1364-2005`/`1364-2005` normalize correctly. Acceptance checklist enforced (code leaf):
    ROOT CAUSE / ADDRESSED / NO REGRESSION — see the "Acceptance Checklist (`.2`)" section.

- ID: `VERILOG-2005-PROFILE.2.1`
  Status: `done` (2026-07-02, `PGEN-VERILOG-2005-PROFILE-0004`) — LOCKSTEP leaf, **PURE-DOCS
  (main platform mdBook + tracker docs only; NO `grammars|rust/src|generated|ast_shape_contract`
  change → not a code change per the mechanical classifier)**. Closes the main-platform-book drift
  the `.2` lockstep omitted: `.2` updated the SV *sub-book* + `EMBEDDING_API_CONTRACT.md`, but the
  top-level `docs/book/` never mentioned the now-live `verilog_2005` profile nor the embedding API
  `1.3.0` surface — a HIGH/MEDIUM drift found in the fresh-session startup mdBook currency audit
  (a book↔codebase drift is a tracked correctness defect per the no-drift doctrine). Scope:
  - `docs/book/src/parser-families.md` (SystemVerilog section): add the `verilog_2005` (IEEE
    1364-2005) dialect-profile bullet — rides the `sv_2017` baseline of shared core constructs,
    rejects the SV-only surface (`module m; endmodule` accepts / `class C; endclass` rejects under
    `--profile verilog_2005`), incrementally hardening under tree `VERILOG-2005-PROFILE`.
  - `docs/book/src/embedding-and-downstream-integration.md` (Embedding API section): state the
    current API version `1.3.0` + the family→profile list (SV: `sv_2017`/`sv_2023`/`verilog_2005`;
    VHDL: `vhdl_1076_2019`; regex: `regex_default`), pointing at `EMBEDDING_API_CONTRACT.md` as the
    authoritative versioned list. Values confirmed tools-first from `rust/src/embedding_api.rs`
    (`EMBEDDING_API_VERSION`@29; `GrammarProfile::as_str`@230-234; `systemverilog_profiles`@412-414).
  - Proof: `make -C rust SHELL=/bin/bash mdbook_docs_gate` GREEN (book builds). No oracle re-run
    needed (no code/grammar/generated change; the 6 fully-certified grammars + SV inert by
    construction; clippy N/A). Full COMMIT.md lockstep (tree + TASK_TREE + LIVE + CHANGES +
    DEVELOPMENT_NOTES + MEMORY). SV family status UNCHANGED (`Mostly Done`).
- ID: `VERILOG-2005-PROFILE.3`
  Status: `done` (2026-07-02, `PGEN-VERILOG-2005-PROFILE-0005`) — **INVESTIGATION leaf, PURE-DOCS**
  (task-tree + tracker docs only; the grammar gate edits trialed during this leaf were REVERTED, so
  NO `grammars|rust/src|generated|ast_shape_contract` change lands → not a code change). Records the
  tools-first discovery that **reshapes the `.4`+ implementation** and surfaces a pre-existing `.2`
  wellformedness debt. See "`.3` Findings". Headline (tool-proven via `ast_pipeline … --lint-grammar`):
  - The wellformedness linter has a **`verilog_2005`-profile-aware ORPHAN detector**
    (ANNOTATION-COMPOSITION.4): a rule active under `verilog_2005` whose every production references a
    rule ABSENT under `verilog_2005` is an unsatisfiable ORPHAN = a wellformedness ERROR.
  - **`.2` (committed HEAD) already has 170 such `verilog_2005` orphans** — `.2` created the profile
    but admitted only `module_declaration_sv_2017`, so ~170 core rules (`conditional_expression`,
    `concatenation`, `conditional_statement`, …) are active-but-unsatisfiable under `verilog_2005`.
    `.2` shipped this undetected because its verification did not run `--lint-grammar`. Under the
    grammar-wellformedness contract ([[project_grammar_wellformedness_contract]]) an orphan is a
    DEFECT, not a residual — so `.2` introduced a 170-orphan wellformedness regression.
  - A **gates-first** `.3` (the originally-planned 16 whole-rule SV-only gates) WORSENS this to **256
    orphans** (verified) — the gates remove more rules from `verilog_2005`, orphaning more parents. A
    gates-only slice therefore CANNOT pass the NO-REGRESSION box (it is a lint regression). REVERTED.
  - Root mechanism: the `verilog_2005` profile is orphan-clean **only when built to COHERENCE** — all
    Verilog-2005-core rules admitted to `verilog_2005` AND all SV-only rules gated out, so every
    `verilog_2005`-active rule is satisfiable. Any half-built intermediate (gates-only or admit-only)
    leaves orphans. The orphan detector is the exact **closure oracle** for the profile (drive to 0).
  - CONSEQUENCE for `.4`: the implementation must be a **build-to-coherence campaign driven by
    `--lint-grammar` (target: 0 `verilog_2005` orphans)**, not independent gates-only / admit-only
    slices. This also subsumes `.2`'s 170-orphan debt (completion resolves it). Flagged to the
    director (scope/strategy) since it touches the wellformedness contract + reveals a `.2` regression.
- ID: `VERILOG-2005-PROFILE.4` … (implementation — the CORRECTED plan): **build the `verilog_2005`
  profile to wellformedness COHERENCE**, using `ast_pipeline grammars/systemverilog.ebnf --lint-grammar`
  `verilog_2005`-orphan count as the closure oracle (drive 170 → 0). One coherent cluster per leaf,
  each ending with a NON-INCREASING orphan count and accept/reject proven tools-first: (a) admit
  `verilog_2005` to the `sv_2017` baseline of the oracle-adjudicated Verilog-2005-core rules
  (declaration / statement / expression / dispatcher rules present in `verilog_2005_lrm_extracted.ebnf`
  or known SV-refactorings whose SV-only children are gated); (b) gate the SV-only umbrella rules
  (the 16 from the `.3` trial + the Deliverable-1 table) `["sv_2017","sv_2023"]`; (c) the bare-keyword
  branch-lift gates (`always_comb/latch/ff`, `do…while`/`foreach`, `bit`/`logic`, `shortreal` —
  shape-preserving, AST-verified); (d) keyword re-admission (Deliverable 2). Closure = `--lint-grammar`
  0 `verilog_2005` orphans + a realistic Verilog-2005 module ACCEPTS + the SV-only reject corpus
  REJECTS under `verilog_2005` while ACCEPTing under sv_2017/sv_2023 + cert/corpus/shape-contract
  no-regression. Then LIVE + contract + book lockstep + a `verilog_2005` cert/corpus closure gate. The
  `sv_cert_recognized_union_gate` count-drift re-baseline (its own ownership) also lives in this band.

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

## `.2` Findings (tools-first, 2026-07-02)

### The load-bearing correction to the `.1` Option-B design (tool-proven)

The `.1` design's Deliverable 3 concluded "an un-annotated rule is active in ALL profiles, so annotate
only the SV-only rules (Option B) and the shared/core rules stay active under `verilog_2005`." A
tools-first BEFORE-state probe of the un-registered `verilog_2005` profile disproved the load-bearing
half of that premise:

```
$ parseability_probe --parse systemverilog module_min.sv --profile verilog_2005
Parser did not consume full input at position 0 [furthest_position=0, +0 bytes deeper]   # rc=1
```

`module m; endmodule` — pure Verilog-2005 — REJECTED at position 0 under `verilog_2005`. A scoped
`PGEN_TRACE_VERBOSITY=high … --trace-rules systemverilog_file` showed `module_declaration` taking
`Backtrack { position: 0 }`. WHY+WHERE: the shared core constructs are **profile-SPLIT** —
`module_declaration_sv_2017` (`@profiles:["sv_2017"]`) and `module_declaration_sv_2023`
(`@profiles:["sv_2023"]`), dispatched by an un-gated `module_declaration` umbrella. There are **101
`["sv_2017"]` + 126 `["sv_2023"]`** such gates and **zero** `["sv_2017","sv_2023"]` core gates. A
brand-new profile string that matches NEITHER list disables every profile-split core rule ⇒
`verilog_2005` parses the **empty language**, not "core active + SV-only gated." Gating
`class_declaration` alone (the `.1` first-slice plan) would therefore have shipped a profile that
accepts nothing.

### Corrected mechanism (declarative, no new engine primitive)

Because **Verilog-2005 ⊂ SV-2017 ⊂ (2017 ∪ 2023)**, `verilog_2005` rides the **`sv_2017` baseline
variant** of each shared core rule: `@profiles:["sv_2017"]` → `["sv_2017","verilog_2005"]`. Genuinely
SV-only roots stay gated `["sv_2017","sv_2023"]` (excluding `verilog_2005`). This uses ONLY the
existing `@profiles` allow-list + ordered choice — no engine change. `.2` applies this to the single
core rule on the minimal `module m; endmodule` path (`module_declaration_sv_2017`; its children
`module_ansi_header`/`module_keyword`/`module_identifier`/`non_port_module_item` are un-annotated ⇒
already active). `.3`.. extends the baseline admission to the remaining shared core rules per-family,
tools-verified, alongside the SV-only gates (the Deliverable-1 table) and keyword re-admission
(Deliverable 2).

### Pre-existing finding (NOT this slice; flagged for a follow-up leaf)

Running the authoritative `make -C rust sv_cert_recognized_union_gate` shows it is **RED on COUNTS
only**: `canonical UNKNOWN=20`, `union UNKNOWN=1`, `union residual=["context_member_method_call"]` all
MATCH the contract, but `total=1312`/`witness` are `+8` over the pinned `expected_total=1304`. Root
cause (git-traced): the contract `systemverilog_recognized_cert_union_contract.json` was pinned at
`5d8801d6` on release `1.0.151`; the subsequent `SV-AST-SHAPE-FIDELITY` campaign (`SV-0014`→`SV-0020`,
`1.0.152`→`1.0.158`) landed *"shared named-lift"* fixes (each adds named rules) **without
re-baselining this contract**. This drift is **pre-existing** and **union-neutral to this slice** (my
diff is directive-only; `verilog_2005` is not a union config; the `+8` is identical with/without it —
proven: rule-def count `1433=1433` HEAD↔working, and the semantic union invariants are unchanged).
Recommend a separate task-tree leaf to re-baseline the union contract counts (its own ownership +
verification), not folded here (one concern per commit; a cert-oracle re-baseline deserves its own
proof).

## Acceptance Checklist (`.2`, enforced)

- [x] **REPRODUCE / ISSUE** — `parseability_probe --parse systemverilog {class_min,module_min}.sv
  --profile verilog_2005`: BEFORE, `class` rejected at pos 0 AND `module m; endmodule` rejected at
  `furthest_position=0` (verilog_2005 = empty language, un-registered profile).
- [x] **ROOT CAUSE (WHY + WHERE)** — `PGEN_TRACE_VERBOSITY=high … --trace-rules systemverilog_file`
  shows `module_declaration` `Backtrack { position: 0 }`; WHERE = the profile-split shared core rules
  (`module_declaration_sv_2017`@`["sv_2017"]` / `_sv_2023`@`["sv_2023"]`; 101+126 such gates, 0
  three-profile) exclude any new profile; `--report-certificate-coverage --grammar-profile sv_2017`
  used for the coverage baseline. `furthest_position=0` cited.
- [x] **FIX** — declarative (fix-hierarchy tier: grammar + profile registration, no engine primitive):
  admit `verilog_2005` to `module_declaration_sv_2017`'s `sv_2017` baseline; gate `class_declaration`
  `["sv_2017","sv_2023"]`; register `GrammarProfile::Verilog2005` end-to-end + registry normalize +
  `EMBEDDING_API_VERSION` `1.3.0`.
- [x] **ADDRESSED (verified)** — before→after, `--parse … --profile`: `class C; endclass`
  verilog_2005 REJECT (rc=1) / sv_2017 ACCEPT (rc=0) / sv_2023 ACCEPT (rc=0); `module m; endmodule`
  verilog_2005 **ACCEPT (rc=0, was reject@0)** / sv_2017 (rc=0) / sv_2023 (rc=0); aliases
  `ieee1364-2005` (module ACCEPT) / `1364-2005` (class REJECT) normalize correctly.
- [x] **NO REGRESSION** — SV cert seeds 0/7/42 (sv_2017): canonical `UNKNOWN=20`, union `UNKNOWN=1`,
  residual `["context_member_method_call"]`, `sample_parse_failures=0`, deterministic (semantic
  invariants preserved; the `total`/`witness` `+8` is the pre-existing `SV-AST-SHAPE-FIDELITY`
  count-drift above, union-neutral to this slice). `ast_shape_contract_gate` GREEN (18/18);
  embedding_api lib tests 51 passed/0 failed; clippy SOURCE clean (generated debt pre-existing,
  non-strict); realistic corpus 239/239 non-preprocessor `.sv` parse under sv_2017 (0 real fails; 126
  fails are preprocessor-directive files, by-design not raw-SV-parseable); the 6 fully-certified
  grammars byte-identical (only SV regenerated).
- [x] **LOCKSTEP** — `rust/docs/EMBEDDING_API_CONTRACT.md` (version + `GrammarProfile` list), the SV
  parser book (`verilog_2005` profile note), CHANGES / DEVELOPMENT_NOTES / MEMORY /
  LIVE_ACHIEVEMENT_STATUS updated; downstream SV integration contract full `verilog_2005` write-up
  deferred to the closure leaf (profile still incrementally hardening) per the tree's closure plan.

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
- 2026-07-02 (`.2`, tool-proven correction): **`verilog_2005` RIDES the `sv_2017` baseline variant of
  each shared, profile-SPLIT core rule** (`["sv_2017"]`→`["sv_2017","verilog_2005"]`), while genuinely
  SV-only roots stay gated `["sv_2017","sv_2023"]` (excluding `verilog_2005`). This SUPERSEDES the
  `.1` Option-B premise that "core rules are un-annotated → active in all profiles" — false for the
  101 `["sv_2017"]` + 126 `["sv_2023"]` version-split core rules; a new profile matching neither
  parses the empty language (proven: `module m; endmodule` rejected at pos 0). Declarative, no new
  engine primitive. Because Verilog-2005 ⊂ SV-2017, the `sv_2017` variant is the correct baseline.
- 2026-07-02 (`.2`): **Introduce the profile in the embedding contract now (version `1.3.0`), not at
  closure.** Adding a supported profile is a backward-compatible contract addition (schema stays `2`);
  the profile is registered + selectable immediately and its subset enforcement hardens across `.3`..
  The downstream SV integration-contract full write-up stays deferred to the closure leaf (honest: the
  strict subset is still being completed) — but the code-owned `rust/docs/EMBEDDING_API_CONTRACT.md`
  is updated in lockstep because the code const changed.
- 2026-07-02 (`.3`, tool-proven, SUPERSEDES the `.1`/`.2` "incremental gates-first/one-family-per-leaf"
  framing for the ordering): **the `verilog_2005` profile must be built to wellformedness COHERENCE
  (0 `--lint-grammar` orphans), not shipped half-built.** An incomplete profile makes core rules
  active-but-unsatisfiable (ORPHANS) — a wellformedness DEFECT per
  [[project_grammar_wellformedness_contract]]. `.2` already left 170 such orphans (undetected — `.2`
  did not run `--lint-grammar`); a gates-only step worsens it (256). The corrected `.4`+ plan drives
  the orphan count to 0 using `--lint-grammar` as the closure oracle. **NEW STANDING SUB-RULE for
  this tree:** every future `verilog_2005` code leaf MUST run `--lint-grammar` and end with a
  NON-INCREASING `verilog_2005` orphan count (the profile-orphan lint is part of acceptance now).
  Deferred to the director: confirm the build-to-coherence scope (a larger coordinated campaign than
  a single 16-rule gate slice) before the `.4` code work begins.

## Open Questions

- ~~Exact default-profile-membership semantics of an UN-annotated rule~~ — **RESOLVED (`.1`)**:
  un-annotated ⇒ active in ALL profiles (`rule_profile_is_enabled` returns `true` on an empty
  allow-list). Gating is therefore additive (Option B). **AMENDED (`.2`, tool-proven):** the premise
  is only half the story — shared CORE constructs are profile-SPLIT into `_sv_2017`/`_sv_2023` gated
  variants (101+126 of them), so they are NOT un-annotated and a new profile matching neither is
  excluded from them ⇒ empty language. `verilog_2005` must be positively ADMITTED to the `sv_2017`
  baseline of each shared core rule (see `.2` Findings + Decisions). Option B (gate SV-only roots) is
  correct for the subtractive axis; the additive baseline-admission axis is the `.2` correction.
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
- 2026-07-02 (`.2`, CODE — full verification): see "Acceptance Checklist (`.2`)" for the earned
  boxes. Headlines: accept/reject proven both ways for `class`/`module` × `{verilog_2005, sv_2017,
  sv_2023}` + aliases; SV cert seeds 0/7/42 semantic invariants preserved (`UNKNOWN=20`, union `1`,
  residual `context_member_method_call`, `spf=0`); `ast_shape_contract` 18/18; embedding_api 51/0;
  clippy source clean; realistic corpus 239/239 non-preprocessor under sv_2017. Regenerated SV only
  (other 6 grammars byte-identical). Pre-existing `sv_cert_recognized_union_gate` count-drift
  discovered + git-traced (not this slice; follow-up leaf recommended).
- 2026-07-02 (`.2.1`, PURE-DOCS book lockstep): `make -C rust SHELL=/bin/bash mdbook_docs_gate`
  GREEN (`mdbook_build` pass) after adding the `verilog_2005` profile bullet to
  `docs/book/src/parser-families.md` (SystemVerilog section) and the "Current API surface"
  subsection (version `1.3.0` + family→profile list) to
  `docs/book/src/embedding-and-downstream-integration.md`. Documented values confirmed tools-first
  from `rust/src/embedding_api.rs` (`EMBEDDING_API_VERSION="1.3.0"`@29; `GrammarProfile::as_str`
  `sv_2017`/`sv_2023`/`verilog_2005`/`vhdl_1076_2019`/`regex_default`@230-234; SV
  `systemverilog_profiles` vec@412-414). No code/grammar/generated change ⇒ no oracle re-run
  (cert / corpus / the 6 fully-certified grammars inert by construction; clippy N/A).
- 2026-07-02 (`.3`, INVESTIGATION — tools-first): `ast_pipeline <grammar> --lint-grammar`. On HEAD
  (committed `.2`): **170 `verilog_2005` profile-orphan ERRORS** (`git show HEAD:grammars/systemverilog.ebnf`
  → temp → lint). With the trial 16-gate `.3` edit: **256 orphans** (170→256, a lint regression) →
  the gate edits were REVERTED (`git checkout -- grammars/systemverilog.ebnf`; gate census back to the
  single `.2` `["sv_2017","sv_2023"]` class gate). Oracle adjudication that fed the finding:
  `verilog_2005_lrm_extracted.ebnf` (509 prod) presence check — Verilog-2005-core (present → admit):
  `net_declaration`/`blocking_assignment`/`statement`/`event_control`/`delay_control`/`primary`/
  `constant_primary`/`module_item`/`parameter_declaration`/`param_assignment`/`function_declaration`/
  `task_declaration`/`udp_declaration`/`gate_instantiation`/…; SV-only (absent → gate):
  `class`/`covergroup`/`constraint`/`property_expr`/`production`/`tagged_union`/`interface`/`program`.
  Observed leak: `package p; endpackage` currently PARSES under `--profile verilog_2005` (un-gated
  `package_declaration` reachable from `source_text_item`); `interface`/`program` already reject
  (profile-split dispatch). No grammar/code/generated change lands (all trial edits reverted).

## Commit Log

- 2026-07-02 (`.3`, INVESTIGATION, PURE-DOCS, `PGEN-VERILOG-2005-PROFILE-0005`): tools-first
  `--lint-grammar` run exposed the `verilog_2005` profile-orphan wellformedness debt (HEAD/`.2`: 170
  orphans; trial gates-first: 256 → REVERTED as a lint regression). Reclassified `.3` as an
  investigation leaf + rewrote the `.4`+ plan to a build-to-coherence campaign (orphan count → 0 via
  `--lint-grammar` as the closure oracle). No grammar/code/generated change lands (all trial gate
  edits reverted; grammar restored to HEAD). Flagged the strategy/scope + the pre-existing `.2`
  regression to the director. SV family status UNCHANGED (`Mostly Done`).
- 2026-07-02 (`.2.1`, PURE-DOCS book lockstep, `PGEN-VERILOG-2005-PROFILE-0004`): closed the
  main-platform-book drift `.2` omitted (found in the fresh-session startup mdBook currency audit) —
  the top-level `docs/book/` now documents the live `verilog_2005` profile (in `parser-families.md`)
  and the embedding API `1.3.0` surface + family→profile list (in
  `embedding-and-downstream-integration.md`, pointing at `EMBEDDING_API_CONTRACT.md` as authoritative).
  `mdbook_docs_gate` GREEN. No `grammars|rust/src|generated|ast_shape_contract` change → not a code
  change; SV family status UNCHANGED (`Mostly Done`). Frontier stays `.3`.
- 2026-07-02 (`.2`, tools-first CODE): confirmed BEFORE-state via `parseability_probe --parse`:
  `class C; endclass` and `module m; endmodule` BOTH rejected at pos 0 under `--profile verilog_2005`
  (empty language). Root cause pinned by `--trace-rules systemverilog_file` (`module_declaration`
  `Backtrack{position:0}`) → shared core rules profile-split, no 3-profile gate. Grammar: rule-def
  count `1433=1433` (directive-only diff). AFTER regen (`focus_systemverilog`, 47s) + release-probe
  rebuild: class REJECT@verilog_2005 / ACCEPT@sv_2017,sv_2023; module ACCEPT@all-three; aliases
  normalize. Cert seeds 0/7/42: canonical `UNKNOWN=20`, union `UNKNOWN=1`, residual
  `["context_member_method_call"]`, `spf=0`, deterministic. `ast_shape_contract_gate` 18/18;
  embedding_api tests 51/0; clippy source clean; realistic corpus 239/239 non-preprocessor `.sv`
  under sv_2017. Discovered + git-traced the pre-existing `sv_cert_recognized_union_gate` count-drift
  (`total 1304→1312` from `SV-0014`→`SV-0020` named-lifts; contract pinned at `5d8801d6`/`1.0.151`) —
  flagged for a follow-up re-baseline leaf, union-neutral to this slice.
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
- 2026-07-02: `.2` DONE (`PGEN-VERILOG-2005-PROFILE-0003`, CODE) — registered the `verilog_2005`
  profile end-to-end (`GrammarProfile::Verilog2005` + aliases + `validate_profile_match` +
  `profile_matrix` + `EMBEDDING_API_VERSION 1.3.0`; registry normalize) and gated the first SV-only
  family (`class_declaration` → `["sv_2017","sv_2023"]`). Tools-first surfaced + corrected the `.1`
  Option-B hole: shared core constructs are profile-SPLIT, so `verilog_2005` was admitted to the
  `sv_2017` baseline of `module_declaration_sv_2017` (else the profile parses the empty language).
  Accept/reject proven both ways; no-regression green (cert semantic invariants, shape-contract,
  embedding tests, clippy, realistic corpus). Discovered the pre-existing `sv_cert_recognized_union_gate`
  count-drift (SV-AST-SHAPE-FIDELITY lockstep debt) — flagged for a follow-up leaf. Frontier → `.3`
  (extend baseline admission + SV-only gates per-family).
- 2026-07-02: `.2.1` DONE (`PGEN-VERILOG-2005-PROFILE-0004`, PURE-DOCS book lockstep) — closed the
  main-platform-book drift the `.2` lockstep omitted (found in the fresh-session startup mdBook
  currency audit): `docs/book/src/parser-families.md` now documents the `verilog_2005` dialect
  profile in the SystemVerilog section, and `docs/book/src/embedding-and-downstream-integration.md`
  now states the embedding API version `1.3.0` + the family→profile list (SV
  `sv_2017`/`sv_2023`/`verilog_2005`; VHDL `vhdl_1076_2019`; regex `regex_default`), deferring the
  authoritative versioned list to `EMBEDDING_API_CONTRACT.md`. `mdbook_docs_gate` GREEN; no
  code/grammar/generated change; SV family status UNCHANGED (`Mostly Done`). Frontier stays `.3`.
- 2026-07-02: `.3` DONE (`PGEN-VERILOG-2005-PROFILE-0005`, INVESTIGATION, PURE-DOCS) — tools-first
  `--lint-grammar` discovery: the `verilog_2005` profile is only wellformed (0 orphans) when built to
  COHERENCE; `.2` already left 170 `verilog_2005` profile-orphan defects (undetected — linter not run
  in `.2`), and a gates-only step worsens it to 256 (a lint regression), so the trial 16-gate edit was
  REVERTED. Reclassified the `.4`+ implementation as a build-to-coherence campaign driven by the
  orphan detector as the closure oracle (→ 0 orphans), with a new standing sub-rule that every
  `verilog_2005` code leaf must run `--lint-grammar` and never increase the orphan count. Flagged the
  scope + the pre-existing `.2` regression to the director. Frontier → `.4` (director scope-confirm
  pending). SV family status UNCHANGED (`Mostly Done`).
