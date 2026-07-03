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
  - **DIRECTOR CONFIRMED (2026-07-02): proceed with the build-to-coherence campaign.** Execution
    deferred to a FRESH SESSION for full focus — a ~40+-directive delicate grammar surgery with leak
    risk, not a tail-of-session task (fresh-session discipline).
  - **EFFICIENCY RECIPE (no wasted regens):** `--lint-grammar` runs on the `.ebnf` directly (no parser
    regen), so iterate EDIT → `ast_pipeline grammars/systemverilog.ebnf --lint-grammar` → adjudicate
    remaining orphans → repeat until **0 `verilog_2005` orphans**, THEN do ONE `make -C rust
    focus_systemverilog` + ONE full no-regression suite. Do NOT regen per edit.
  - **CANDIDATE CLASSIFICATION (tools-first; the fresh session MUST re-verify via the orphan detector +
    a reject-corpus — leaks are INVISIBLE to the orphan detector, so a reject-corpus is mandatory):**
    - ADMIT (`["sv_2017"]`→`["sv_2017","verilog_2005"]`) — oracle-CONFIRMED core (present in
      `verilog_2005_lrm_extracted.ebnf`): `blocking_assignment` `constant_primary` `event_control`
      `event_trigger` `full_edge_sensitive_path_description` `function_declaration` `gate_instantiation`
      `list_of_parameter_assignments`(+base) `local_parameter_declaration` `named_port_connection`
      `net_declaration` `parallel_edge_sensitive_path_description` `param_assignment`
      `parameter_declaration` `parameter_value_assignment` `primary` `task_declaration` `udp_declaration`
      (all `_sv_2017`).
    - ADMIT — core refactorings (oracle name-absent because SV renamed the 1364-2005 BNF, but the
      construct IS Verilog-2005 and its SV-only children are gated — VERIFY no leak each):
      `block_data_declaration` `data_declaration` `delay` `net_port_type` `statement_item`
      `module_common_item` `tf_port_direction` `parameter_port_declaration` (all `_sv_2017`).
    - GATE (`["sv_2017","sv_2023"]`) — SV-only reachable from admitted dispatchers, NOT in Verilog-2005:
      the 16 whole-rule umbrellas from the `.3` trial PLUS `type_declaration` (typedef) `struct_union`
      `integer_atom_type` `case_inside_item` `open_range_list` `open_value_range` `pattern`
      `tagged_union_expression` `class_*` `covergroup_*` `constraint_*` `cross_*` `prop_*`/`property_expr`
      `production`/`rs_*` `checker_*` `clocking_*` `dist_item` `boolean_abbrev`
      `non_consecutive_repetition` `uniqueness_constraint` `sequence_actual_arg` `method_call_receiver`
      `type_reference` `net_type_*` `weight_specification`.
    - The orphan detector's "DERIVED minimal fix" always suggests GATING — correct for SV-only, WRONG for
      core (a core orphan is fixed by ADMITTING its children). Judge by the oracle, not the hint.

- ID: `VERILOG-2005-PROFILE.4.1`
  Status: `done` (2026-07-02, `PGEN-VERILOG-2005-PROFILE-0007`) — CODE leaf: **built the `verilog_2005`
  profile to wellformedness COHERENCE (orphans 170→0, `--lint-grammar` rc 1→0 — closes the `.2`
  regression) + profile-faithful keyword reservation** (clusters (a)+(b)+(d) of the `.4` plan; cluster
  (c) branch-lifts remain → `.4.2`). Grammar-directive-only surgery + 3 shape-preserving named rules;
  NO engine change. As executed (see "`.4.1` Findings" + "Acceptance Checklist (`.4.1`)"):
  - (a) ADMITS — 28 more `["sv_2017"]`→`["sv_2017","verilog_2005"]` (29 total with `.2`): the 18
    oracle-confirmed + 8 refactoring candidates from the `.4` classification (each RE-verified against
    `verilog_2005_lrm_extracted.ebnf` before edit) + the `list_of_parameter_assignments` base umbrella
    + `package_or_generate_item_declaration_sv_2017` (mis-named shared-core declarations dispatcher —
    adjudicated ADMIT against the detector's gate-hint; its SV-only children are individually gated).
  - (b) GATES — 103 new `["sv_2017","sv_2023"]`: 77 orphan-adjudicated rules across 3 lint iterations
    (170→75→35→3→0; every batch oracle-checked, all absent) + 25 LEAK rules invisible to the orphan
    detector (SV-only entry rules whose children are all core-satisfiable): `package_declaration`,
    `package_import_declaration`, `modport_declaration`, `interface_class_declaration`,
    `sequence_declaration`, `constraint_block`, `bind_directive`, `jump_statement`,
    `inc_or_dec_expression`, `final_construct`, `procedural_assertion_statement`,
    `immediate_assertion_statement`, `simple/deferred_immediate_assertion_statement`(+`_item`),
    `assertion_item`(+`_declaration`), `concurrent_assertion_item`, `net_alias`, `randcase_statement`,
    `timeunits_declaration`, `clocking_drive`, `dpi_import_export`, `let_declaration`,
    `checker_declaration`, `case_pattern_item`, `unique_priority`.
  - (d) KEYWORD RE-ADMISSION — the `.1` D2 split verbatim: `reserved_non_keyword_identifier :=
    reserved_non_keyword_identifier_sv | reserved_non_keyword_identifier_v2005`; `_sv` = the original
    curated regex gated `["sv_2017","sv_2023"]`; `_v2005` = the FULL IEEE 1364-2005 Annex B set
    (124 keywords, extracted mechanically from
    `docs/verilog/2005/md/section-Annex_B-normative-list-of-keywords.md`) gated `["verilog_2005"]`.
    Set algebra re-verified mechanically: 33 stay reserved / 48 un-reserved (exactly the D2 lists) /
    ~91 newly reserved under `verilog_2005` (`initial`, `always`, `assign`, `and`, …).
  - Named-lift `integer_atom_type_sv_only` (byte/shortint/int/longint; gate moved onto it) — corrects
    a `.1` D1-table error: the SV `integer_atom_type` has SIX branches; `integer`/`time` are
    Verilog-2005 core and stay in the un-gated umbrella (shape-preserving; found when the realistic
    accept-corpus module over-rejected at `integer i;`).

- ID: `VERILOG-2005-PROFILE.4.2`
  Status: `done` (2026-07-02, `PGEN-VERILOG-2005-PROFILE-0008`) — CODE leaf: **cluster (c)
  branch-lifts landed** — 10 shape-preserving named-lifts per the `integer_atom_type_sv_only` idiom
  + 1 whole-rule leak gate, grammar-directive-only, NO engine change. As executed (see "`.4.2`
  Findings" + "Acceptance Checklist (`.4.2`)"):
  - Named-lifts (each `@profiles`-gated; umbrella alternative order PRESERVED; all lifted return
    annotations verbatim): `always_keyword_sv_only` (`always_comb`/`always_latch`/`always_ff`),
    `loop_statement_sv_only` (`do…while`, `foreach`), `integer_vector_type_sv_only` (`bit`/`logic`),
    `non_integer_type_sv_only` (`shortreal`), `net_port_type_interconnect_sv_only` (shared by both
    `net_port_type_sv_2017`/`_sv_2023`), `interconnect_net_declaration_sv_only` (shared by both
    `net_declaration` variants; carries the branch's `@probe_sample` verbatim),
    `port_direction_sv_only` (`ref` — a DISCOVERED leak beyond the plan: `task t (ref integer a);`
    accepted under `verilog_2005`), `tf_port_direction_const_ref_sv_only` (`const ref`,
    `["sv_2017"]` since its only parent is the `_sv_2017` variant), `wait_statement_sv_only`
    (`wait fork;` + `wait_order(…)`), `event_trigger_control_sv_only` (the delay-control form,
    `["sv_2017"]` — see the SV-0023 transformation note below).
  - Whole-rule leak gate: `interface_port_header` `["sv_2017","sv_2023"]` (interface-typed ANSI
    ports are SV-only; discovered when `module m (interconnect w);` re-parsed as an interface port
    after the net_port_type lift — AST-dump-pinned).
  - **The planned "`event_trigger` `->>` branch" item TRANSFORMED under tools:** the literal `->>`
    appears NOWHERE in the grammar — the SV grammar encodes both LRM alternatives with `->` and
    mis-attaches the optional `delay_or_event_control` to the second `->` branch (so `-> #5 e;`
    wrongly ACCEPTS under sv_2017 and `->> e;` wrongly REJECTS). Ledgered **`SV-0023`** (own fix
    leaf; also covers the 1364-2005 event-array-trigger accept-gap `-> e[0];`). Within this slice
    the mis-encoded delay-form branch was lifted + gated (`event_trigger_control_sv_only`) so
    `-> #5 e;` correctly REJECTS under `verilog_2005`; its sv_2017 shape/labels kept verbatim.
  - **`module m (interconnect w);` (bare two-identifier port) is NOT rejectable in this slice:**
    after both interconnect gates + the interface_port_header gate it STILL accepts via the
    non-ANSI `port_expression` list branch `( port_reference ( comma port_reference )* )*` — a
    pre-existing LRM-extraction defect (the LRM's LITERAL concatenation braces extracted as
    meta-repetition; `module m (a b);` accepts under EVERY profile at HEAD). Ledgered **`SV-0024`**
    (own fix leaf, behavior-tightening all profiles). The conformance corpus pins the interconnect
    PORT reject case on the ANSI form `module m (input interconnect w);` until SV-0024 lands.

- ID: `VERILOG-2005-PROFILE.4.3`
  Status: `done` (2026-07-02, `PGEN-VERILOG-2005-PROFILE-0009`) — CLOSURE leaf (gate/test-data/docs
  only; NO `grammars|rust/src|generated|ast_shape_contract` change — zero grammar or Rust-source
  edits). As executed (see "`.4.3` Findings" + "Acceptance Checklist (`.4.3`)"):
  - **Corpus promoted**: the `.4.1`/`.4.2` scratch conformance sets (recovered from the prior
    sessions' `/private/tmp` scratchpads before they could be wiped) now live tracked at
    `rust/test_data/grammar_quality/verilog_2005_conformance/{accept,reject}/` — **12 accept + 34
    reject files** (the union of the `.4.1` 4-accept/19-reject and `.4.2` 8-accept/15-reject sets).
  - **Repo-standard gate**: `make -C rust SHELL=/bin/bash verilog_2005_conformance_gate` →
    `rust/scripts/verilog_2005_conformance_gate.sh` asserting the tracked contract
    `rust/test_data/grammar_quality/verilog_2005_conformance_contract_v0.json` (the
    `sv_cert_recognized_union_gate` template): (1) the `--lint-grammar` **0 `verilog_2005`
    profile-orphan lock** (mechanizes the `.3` standing sub-rule — headline `profile_orphans=` AND
    the per-rule orphan-error line count must both equal 0, lint rc must equal 0); (2) the **full
    per-file × per-profile accept/reject matrix** — 138 checks (every case × every profile in its
    expectation map) + 2 profile-alias normalization probes (`ieee1364-2005` accept,
    `1364-2005` reject); (3) the **profiled cert-coverage baseline** (below), all fields pinned,
    determinism asserted across seeds. Emits `summary.txt`/`summary.json` under
    `rust/target/verilog_2005_conformance_gate/`.
  - **Profiled cert baseline measured + pinned** (`--grammar-profile verilog_2005 --entry-rule
    systemverilog_file --count 40`): `total=1138 proof=2 witness=826 UNKNOWN=310
    (sample_parse_failures=0, proof_reverify_failures=0)`, byte-identical headline at seeds 0/7/42.
    Honest read (recorded in the contract's `baseline_note`): 277 of the 310 UNKNOWN are
    NO-reach-path dead-rule candidates UNDER THIS PROFILE — the gated SV-only surface being
    profile-unreachable BY DESIGN (the `.1` D4 "expected and acceptable" posture) — so the pin is a
    regression lock + ratchet floor, NOT a closure claim; the genuine-remainder ratchet is leaf `.6`.
  - **Downstream contract write-up** (deferred from `.2`): full § "Dialect Profile —
    `verilog_2005`" in `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`
    (selection surface, reliance surface, honest boundary `SV-0021`..`SV-0024`, support boundary);
    identity block gains the `verilog_2005` host profile + corrects the STALE embedding-API
    baseline `1.2.0`→`1.3.0` (a `.2`-era lockstep gap found during this leaf).
  - **LIVE promotion decision**: dialect-profiles block `In Progress` → **`Mostly Done`** — NOT
    `Done`, per the tracker's own rules: the proof surface is a curated corpus (rule: curated lists
    cannot earn `Done`) and the ledgered `SV-0024` is a known plausible leak in the strict-subset
    claim (bare `module m (interconnect w);` still accepts under `verilog_2005`).
  - Book lockstep: main book `parser-families.md` (gate + baseline + honest read),
    `parseability-probe-debug.md` (the `verilog_2005` alias set was MISSING from the recognized
    profile-names list — drift found in the `.4.3` startup book audit), SV parser book
    `glossary.md`; README standard-commands gains the gate.
  (The `sv_cert_recognized_union_gate` count re-baseline stays its OWN leaf — now `.5`; drift
  `1304→1324` after `.4.1`'s 3 + `.4.2`'s 10 accounted new rules — canonical `1324/2/1302`,
  union witness `1321`.)

- ID: `VERILOG-2005-PROFILE.5`
  Status: `done` (2026-07-02, `PGEN-VERILOG-2005-PROFILE-0010`) — re-baselined the
  `sv_cert_recognized_union_gate` count pins
  (`rust/test_data/grammar_quality/systemverilog_recognized_cert_union_contract.json`), which had
  been RED on COUNTS only since the `SV-AST-SHAPE-FIDELITY` campaign: pinned
  `expected_total=1304 / proof=1 / canonical_witness=1283 / union_witness=1302` → actual
  `1324 / 2 / 1302 / 1321` (semantic invariants INTACT throughout and UNCHANGED by this
  re-baseline: canonical `UNKNOWN=20`, union `UNKNOWN=1`, residual
  `["context_member_method_call"]`, `spf=0`, deterministic seeds 0/7/42). Root cause git-traced in
  `.2` Findings (contract pinned at `5d8801d6`/release `1.0.151`; `SV-0014`→`SV-0020` + the
  `.4.1` 3 + `.4.2` 10 named-lift rules landed without re-baselining — every one of the +20 rules
  individually accounted in its landing leaf's cert verification, so the new pins re-state
  already-proven totals, not new claims). Executed: 4 count pins updated (nothing else in the
  contract touched — same schema, same seeds, same union configs, same done_rule);
  `make -C rust SHELL=/bin/bash sv_cert_recognized_union_gate` re-run fresh → **GREEN**;
  stale-number mentions in `docs/book/src/grammar-wellformedness.md` + the SV integration
  contract's trust statement updated in lockstep (each annotated with the re-baseline provenance).
  One concern per commit — no other edits. NOTE (follow-up, not this leaf): the union-gate script
  retains its own ~5 GB `focus_systemverilog` stage log per run (the phenomenon `.4.3` fixed in
  the conformance gate); port the `prune_log` helper to it under its owning surface
  (`GRAMMAR-WELLFORMED.H.12.8.5.2`) — the scratch log was deleted manually this session.
  (Follow-up LANDED 2026-07-03 as `GRAMMAR-WELLFORMED.H.12.8.5.4`,
  `PGEN-GRAMMAR-WELLFORMED-0148`: `prune_log` ported to the union gate AND the new
  `rtl_const_expr_cert_gate` stage-log helpers.)

## Acceptance Checklist (`.6.6`, enforced)
- [x] **REPRODUCE / ISSUE** — the `.6.5`-designed gap (profile-gated residual carries no machine classification; manual adjudication drifts per-commit). Fresh BEFORE run reproduced the pins byte-identical: `CERTIFICATE-COVERAGE: grammar='systemverilog' entry='systemverilog_file' samples=40 total=1147 proof=4 witness=816 UNKNOWN=327 fully_certified=false (sample_parse_failures=0, proof_reverify_failures=0)` (seed 0, `PGEN_CERT_COVERAGE_DUMP_ALL=1`, 295 NO-reach enumerated; canonical+union BEFORE likewise `1341/2/1319/20` + union `1338/1`).
- [x] **ROOT CAUSE (WHY + WHERE)** — carried from `.6.5`, all three mechanisms tool-pinned to file:line: `reachable_rules`' unreferenced→secondary-root promotion (`grammar_wellformedness.rs:306-313`) defeats today's `UnreachableRule` proof for stranded rules; the profile-agnostic `declared_*_identifier` emitters (`systemverilog.ebnf:1018-5481`) defeat the kind-existence lint (unproducibility is reachability of producing contexts, a fixpoint); `reach_hops_pass`' missing-mandatory skip (`stimuli_generator.rs:6869`) vs the honesty in `mandatory_node_gated` (`:7120`) explains the class-B spurious reach paths.
- [x] **FIX** — engine tier, parser-agnostic, READ-ONLY: P1 `profile_entry_positively_live` + P2 `classify_profile_residual` (+ satisfiability/edge/gate/emitter/import helpers) in `rust/src/ast_pipeline/grammar_wellformedness.rs`; cert-report print behind `PGEN_CERT_RESIDUAL_CLASSIFICATION=1` in `rust/src/main.rs` (entry universe = canonical + `--cert-union-config` entries present in the active tree); `reach_hops_pass` and all generation paths untouched; 6 new unit tests (module total 39, all green).
- [x] **ADDRESSED (verified)** — the machine classification exists and reproduces the manual adjudication: on `verilog_2005` (universe = canonical + `sv_multi_entry_root`/`library_text`/`systemverilog_parseable_file`) it prints `profile_entry_unreachable (292)` / `store_unproducible_under_profile (17)` / `genuine (18)` = 327, md5-identical across seeds 0/7/42 (`d3d0914028df9c1bdfa8de6d589053ec` ×3); the arithmetic reconciles `.6.5` exactly (295 − 11 entry cohort + 7 class-B + `identifier_list` = 292); the library/entry cohort is all `genuine` (never branded dead); class-A = exactly the `known_unscoped_*`/`checked_*` 17 with per-rule kind/cascade reasons; BOTH machine-vs-manual discrepancies adjudicated tools-first in the machine's favor (see "`.6.6` Findings": `hierarchical_tf_identifier` stale `.6.4` note; `class_scoped_tf_call` negative-gated escape — probes + `--parse-dump-ast` carrier pins), yielding ledgered `SV-0031`/`SV-0032`/`SV-0033`.
- [x] **NO REGRESSION** — env var UNSET: v2005 seed-0 `DUMP_ALL` output byte-identical pre↔post (empty diff) and canonical+union seed-0 output byte-identical pre↔post; headlines reproduce the pins at seeds 0/7/42 (v2005 `1147/4/816/327`, `sample_parse_failures=0`); `sv_cert_recognized_union_gate` ✅ GREEN fresh (canonical `UNKNOWN=20`, union `UNKNOWN=1`, residual `["context_member_method_call"]`, deterministic seeds 0/7/42); `verilog_2005_conformance_gate` ✅ GREEN fresh (0-orphan lint lock + 192-check matrix + 3-seed cert pins); fully-certified SIX byte-identical (json `9/0/9/0`, regex `198/0/198/0`, svpp `74/0/74/0`, vhdl `216/0/216/0`, rtl_frontend `169/1/168/0` — all `fully_certified=true`, `sample_parse_failures=0`; `rtl_const_expr_cert_gate` ✅ `48/0/48/0` seeds 0/7/42); mdbook gate ✅; clippy source-clean (`clippy_on_rust_change` ✅; the generated-stage findings are the pre-existing generated-parser debt class).
- [x] **LOCKSTEP** — TOOLBOX.md §4.6 + quick-chooser row; book `docs/book/src/diagnosing-unknowns.md` (glance-table row + full section, mdbook gate ✅); KM card `cert-coverage-unknown-diagnostics` companion row (km-check OK); ledger rows `SV-0031`/`SV-0032`/`SV-0033` (`Root Caused`); LIVE dialect block (waiver set five); `docs/TASK_TREE.md` row + frontier → `.6.7`; CHANGES/DEVELOPMENT_NOTES/MEMORY. Release/schema unchanged (`1.0.161`/15 — parser behavior untouched by construction).

## Acceptance Checklist (`.6.3`, enforced)
- [x] **REPRODUCE / ISSUE** — `SV-0026` at HEAD pre-fix: `printf 'wire w;\n' | parseability_probe --parse systemverilog /dev/stdin --profile verilog_2005` → ACCEPTS (likewise `reg r;`, `localparam p = 1;`, `parameter p = 1;`); IEEE 1364-2005 A.1.2 allows only module/UDP/config at top level. Two carriers (ledger row).
- [x] **ROOT CAUSE (WHY + WHERE)** — `.6.1`/`.6.3` findings: `description`'s SV `$unit` `package_item` alternative + `source_text_item`'s direct top-level `local_parameter_declaration semi`/`parameter_declaration semi` alternatives are active under `verilog_2005` (un-gated shared-core branches; reach chain + AST dumps named both accepting paths; the `.2` baseline-admission wave admitted them un-audited).
- [x] **FIX** — grammar tier (the `.4.2` shape-preserving named-lift idiom): `description_unit_item_sv_only` + `source_text_item_unit_sv_only`, both `@profiles: ["sv_2017","sv_2023"]`, PEG order preserved, parents reference them as bare pass-through alternatives. Replay of the `.6.3`-recorded diffs, landable only after the `.6.3.2` engine fix.
- [x] **ADDRESSED (verified)** — 12/12 probes: all 4 top-level forms ACCEPT→REJECT under `verilog_2005`, ACCEPT unchanged under `sv_2017`/`sv_2023`; conformance matrix 150→162 checks / 0 mismatches (4 new reject-locks); lint `profile_orphans=0` rc 0 (census 1448→1450); regenerated parser carries both gated rules (grep 49 sites).
- [x] **NO REGRESSION** — THE decisive former blocker now green: canonical `sv_2017` cert `total=1328 proof=2 witness=1306 UNKNOWN=20 spf=0` at seeds 0/7/42, residual 20-rule set md5-identical to pre-lift (`known_unscoped_property_identifier` witnessed — the `.6.3.1` collateral gone); `sv_cert_recognized_union_gate` GREEN fresh (`unmet_criteria_count: 0`; canonical 20 / union `1325/1` / residual exact); `verilog_2005_conformance_gate` GREEN fresh (`gate_green: true`, cert `1138/2/809/327` seeds 0/7/42); `ast_shape_contract_gate` 18/18; `mdbook_docs_gate` ✅; clippy source strict-clean (generated debt pre-existing). Contract count pins re-baselined in the SAME commit (union `1326/2/1304/1323`→`1328/2/1306/1325`; v2005 `817/319`→`809/327` + 4 corpus rows).
- [x] **LOCKSTEP** — ledger `SV-0026`→`Released` (both-carrier fix record); SV integration contract (trust posture, matrix 162, cert pins, honest boundary — open waivers now `SV-0024`/`SV-0028`); book `parser-families.md` (pins + fixed-leak narrative; mdbook gate ✅); LIVE dialect block + session-#20 note; both contract JSONs with provenance; tree + `docs/TASK_TREE.md` frontier → `.6.4`. Release/schema unchanged (`1.0.158`/13 — SV-profile behavior byte-invariant).

## Acceptance Checklist (`.6.3.2`, enforced)
- [x] **REPRODUCE / ISSUE** — `.6.3.1` evidence: `[plannable-probe] rule='known_unscoped_property_identifier' … sample="sequence\foo ;…endsequence property\foo_0 ;…"` (wrong-family prelude at HEAD); post-lift 48/48 `parsed=true witnessed_target=false` (canonical `UNKNOWN` 20→21, union 1→2 — the `.6.3` blocker).
- [x] **ROOT CAUSE (WHY + WHERE)** — tool-named in `.6.3.1`: `OR branch failed: rule='assertion_item_declaration' path='root' branch=0 reason=Stimuli generation depth exceeded max_depth=64 while expanding rule 'number'` → `Selected OR branch: … branch=1` (silent sequence fallback) inside the injected prelude render; injection loop `generate_quantified:9880-9913` accepts any `Ok` with no armed-fact check; budgets `run_plannable_witness_pass:4089/4210` + `max_offpath_mandatory_sibling_depth:6626` never cover the prelude sub-path.
- [x] **FIX** — engine tier (declarative/grammar tiers inapplicable — parser-agnostic generator internals): `ReachPrelude.sub_hops` + hops-generic `max_offpath_mandatory_sibling_depth_along_hops` + armed-prelude integrity check (`store_name_for_gate`) with ONE depth-fresh retry (injection depth + budget + sub-path deepest mandatory off-path sibling) and a loud `Armed name-prelude integrity failure` error when still unsatisfied. Count-preludes and already-correct injections byte-identical by construction.
- [x] **ADDRESSED (verified)** — the witness sample flipped `"sequence\foo ;…endsequence property\foo_0 ;…"` → `"property\foo ;215.8_40endproperty property\foo_0 ;\foo_0 endproperty"` (right fact kind; robust to reach re-routing); plannable pass `1054→1055` witnessed, probe-parse-failures `68→60` (seed 0).
- [x] **NO REGRESSION** — canonical cert `1326/2/1304/UNKNOWN=20 spf=0` byte-identical (headline + full 20-rule residual + NO-reach lists, md5-compared) at seeds 0/7/42; `sv_cert_recognized_union_gate` GREEN fresh (canonical 20 / union 1 / witness 1323 / residual `context_member_method_call`, seeds 0/7/42, `unmet_criteria_count: 0`); `verilog_2005_conformance_gate` GREEN fresh (orphans=0, 150/150, aliases 2, cert `1138/2/817/319` seeds 0/7/42); `ast_shape_contract_gate` 18/18; clippy source strict-clean (generated-stage 182 errors = pre-existing tolerated debt, all in `generated/systemverilog_parser.rs`); fully-certified certs seed 0: json `9/0`, regex `198/0`, vhdl `216/0`, svpp `74/0`, rtl_frontend `169/proof=1/0`; rtl_const_expr canonical-cert timeout A/B-proven PRE-EXISTING (git-stash control: identical `TargetTimeout conditional_expr root/o1 budget=4000ms` on the pre-change binary) → spun off `CERT-GEN-BUDGET.3`, not caused here.
- [x] **LOCKSTEP** — book `docs/book/src/grammar-wellformedness.md` (new prelude-integrity increment in the semantic-prelude ladder; `mdbook_docs_gate` ✅); `CERT-GEN-BUDGET.md` reopened with `.3`; this tree + `docs/TASK_TREE.md`; CHANGES / DEVELOPMENT_NOTES / MEMORY / LIVE. No contract/ledger/schema/release change (generator-internal; parser byte-untouched).

## Acceptance Checklist (`.6.10`, enforced)

- [x] **REPRODUCE / ISSUE** — pre-fix parse-probe on HEAD release binary: `module m; reg q [*];`
  (associative wildcard) and `module m; reg q [$];` (queue) both **ACCEPT under `--profile
  verilog_2005`** (leak), and also ACCEPT under `sv_2017`/`sv_2023` (legal). Associative arrays (IEEE
  1800 §7.8) and queues (§7.10) are SystemVerilog-only; IEEE 1364-2005 array dimensions are always
  ranged (§4.9 / Annex A). No-over-gate control `reg [7:0] q; reg r [3:0];` correctly ACCEPTS under
  `verilog_2005`.
- [x] **ROOT CAUSE (WHY + WHERE)** — `associative_dimension := lbrack data_type rbrack | lbrack star
  rbrack` (`grammars/systemverilog.ebnf:627`) and `queue_dimension := lbrack kw_dollar (colon
  constant_expression)? rbrack` (`:4587`) are each *entirely* SV-only and referenced from EXACTLY ONE
  site — the corresponding `variable_dimension` alternative (`:5810` associative, `:5811` queue).
  `variable_dimension` stays v2005-reachable via its `unpacked_dimension` (ranged) alt. Same
  `.2`/`.4.1` baseline-admission family as `SV-0026`/`SV-0031`/`SV-0032`/`SV-0033`. **Tools-first
  correction:** the associative probe MUST be the wildcard `[*]` — `reg q [integer];` does NOT
  isolate `associative_dimension` (`unpacked_dimension` is tried first; the keyword `integer` parses
  there as a bare primary → the separate all-profile `SV-0035`).
- [x] **FIX** — declarative, grammar-only (tier: grammar; no engine/codegen change): two whole-rule
  `@profiles: ["sv_2017","sv_2023"]` tags (the `.6.9`/`uniqueness_constraint` idiom) on
  `associative_dimension` and `queue_dimension` — ZERO new rules. Validated tools-first on the static
  `--lint-grammar` 0-orphan check before the regen. Emitted source carries the guards
  (`parse_associative_dimension`/`parse_queue_dimension` profile checks in
  `generated/systemverilog_parser.rs`).
- [x] **ADDRESSED (verified)** — `reg q [*];` and `reg q [$];` flip ACCEPT→**REJECT** under
  `verilog_2005` on the regenerated release binary; still ACCEPT under `sv_2017`/`sv_2023`; the
  no-over-gate control `reg [7:0] q; reg r [3:0];` still ACCEPTS under `verilog_2005`; the sv_2017 AST
  is byte-identical (`{kind:"associative"}`/`{kind:"wildcard"}`/`{kind:"queue"}` intact, **zero**
  `_sv_only` wrapper keys); 2 new corpus reject-locks (`reject/assoc_array_dim.sv` = `reg q [*];`,
  `reject/queue_dim.sv`) hold in the 210-check matrix.
- [x] **NO REGRESSION** — `verilog_2005_conformance_gate` **GREEN** (lint orphans=0, corpus 210
  checks / 0 mismatches, aliases 2, cert `1144/4/813/327` deterministic seeds 0/7/42); the honest
  leak-fix delta = both rules LEAVE the profile universe (total 1146→1144) dropping the 2 false
  witnesses earned through the leak (witness 815→813), with UNKNOWN 327 and NO-reach 296 unchanged
  (set-diff proven, both absent from the seed-0 dump); `sv_cert_recognized_union_gate` **GREEN &
  BYTE-IDENTICAL** (canonical `1343/2/1321/UNKNOWN=20`, union `1343/2/1340/UNKNOWN=1`, residual
  `context_member_method_call`, seeds 0/7/42 — no new rule, SV tree untouched); `--lint-grammar` 0
  `verilog_2005` orphans (census 1465 UNCHANGED); `ast_shape_contract` GREEN 18/18; the 6
  fully-certified grammars byte-identical (SV-only regen); `clippy_on_rust_change` GREEN (source stage
  0 errors — grammar-only; generated-parser clippy is pre-existing non-strict debt); `mdbook_docs_gate`
  ✅.
- [x] **LOCKSTEP** — ledger `SV-0034` → `Released` (fix record + `1.0.161` no-bump) + new `SV-0035`
  row (`Root Caused`, ALL-PROFILE — a reserved type keyword parsing as a bare primary,
  `localparam p = integer;` accepts under every profile); conformance contract JSON re-pinned
  (`1144/4/813/327` + `.6.10` `baseline_note` hop + 2 case entries, associative case using `[*]`);
  book `parser-families.md`; SV integration contract; LIVE dialect block; tree + `docs/TASK_TREE.md`
  frontier; CHANGES / DEVELOPMENT_NOTES / MEMORY. Release/schema unchanged (`1.0.161`/15 — SV
  profiles AST-byte-invariant).
- [x] **ALERT-DISCIPLINE (BE-ALERT) + WRONG-CARRIER CATCH** — the associative fix's *assumed* test
  case (`reg q [integer];`) did NOT flip on the rebuilt binary; empirical verification (not trusting
  the assumed grammar path) revealed `[integer]` routes through `unpacked_dimension`
  (keyword-as-primary), a DISTINCT ALL-PROFILE leak now ledgered `SV-0035` (`Root Caused`) —
  `localparam p = integer;` accepts under `sv_2017`/`sv_2023`/`verilog_2005`. Kept OUT of this
  surgical fix; the associative surface re-probed + locked via the isolating `[*]`.

## Acceptance Checklist (`.6.9`, enforced)

- [x] **REPRODUCE / ISSUE** — pre-fix parse-probe on HEAD release binary: `module m; reg q [];
  endmodule` **ACCEPTS under `--profile verilog_2005`** (leak), and also ACCEPTS under
  `sv_2017`/`sv_2023` (legal IEEE 1800). IEEE 1364-2005 §4.9 / Annex A array dimensions are always
  ranged (`[msb:lsb]`) — the bare `[]` (unsized) dimension appears nowhere in 1364-2005; it is an
  IEEE 1800 §7.5 dynamic-array construct. The no-over-gate control `module m; reg [7:0] q; reg r
  [3:0]; endmodule` (ranged packed + unpacked dims) correctly ACCEPTS under `verilog_2005`.
- [x] **ROOT CAUSE (WHY + WHERE)** — `unsized_dimension := lbrack rbrack -> {kind:"unsized"}`
  (`grammars/systemverilog.ebnf:5731`) is the single *entirely* SV-only leaf rule, referenced from
  exactly three UNTAGGED (universal) contexts that all rode into the `verilog_2005` baseline
  un-audited (the `.2`/`.4.1` baseline-admission family — statically satisfiable, off-dialect,
  invisible to the orphan-coherence lint): (1) `packed_dimension`'s alt-2 (`:3823`); (2) the
  `variable_decl_assignment` dynamic-array branch (`:5796`, where `unsized_dimension` is a
  MANDATORY element, not an OR-alt); (3) `variable_dimension`'s alt-1 (`:5801`). Confirmed
  tools-first on the fresh release binary; the sv_2017 AST-dump names the `[]` path as
  `{kind:"unsized"}`.
- [x] **FIX** — declarative, grammar-only (fix-hierarchy tier: grammar; no engine/codegen change):
  a SINGLE whole-rule `@profiles: ["sv_2017","sv_2023"]` tag on `unsized_dimension` (the
  `uniqueness_constraint@:5724` idiom). Candidate A (whole-rule gate) chosen over candidate B
  (per-context `_sv_only` lifts) because `unsized_dimension` is entirely SV-only and each of its
  three carriers keeps a v2005-legal alternative when it is pruned — validated tools-first on the
  static `--lint-grammar` 0-orphan check BEFORE the regen (the `:5796` mandatory-element cascade
  prunes cleanly), so candidate B was never needed. ZERO new rules — cleaner than `.6.8`'s 2-lift.
  Emitted source carries the guard (`parse_unsized_dimension` profile check in
  `generated/systemverilog_parser.rs`).
- [x] **ADDRESSED (verified)** — `module m; reg q [];` flips ACCEPT→**REJECT** under `verilog_2005`
  on the regenerated release binary; still ACCEPTS under `sv_2017`/`sv_2023`; the no-over-gate
  control `reg [7:0] q; reg r [3:0];` still ACCEPTS under `verilog_2005`; the sv_2017 AST is
  byte-identical — `{kind:"unsized"}` intact, **zero** `_sv_only` wrapper keys in the dumped JSON
  (no wrapper rule exists — the whole-rule gate is transparent under the allowed profiles); the 2
  new corpus locks (`reject/dynamic_array_unsized.sv`, `accept/array_ranged_dim.v`) hold in the
  204-check matrix.
- [x] **NO REGRESSION** — `verilog_2005_conformance_gate` **GREEN** (lint orphans=0, corpus 204
  checks / 0 mismatches, aliases 2, cert `1146/4/815/327` deterministic seeds 0/7/42); the honest
  leak-fix delta = `unsized_dimension` LEAVES the profile universe (total 1147→1146) dropping the
  ONE false witness it earned through the leak (witness 816→815), with UNKNOWN 327 and NO-reach 296
  unchanged (set-diff proven: the rule left the universe rather than moving to NO-reach, unlike
  `.6.8`'s `kw_void`); `sv_cert_recognized_union_gate` **GREEN & BYTE-IDENTICAL** (canonical
  `1343/2/1321/UNKNOWN=20`, union `1343/2/1340/UNKNOWN=1`, residual `context_member_method_call`,
  seeds 0/7/42 — candidate A adds no rule and never touches the sv_2017/sv_2023 tree);
  `--lint-grammar` 0 `verilog_2005` orphans (census 1465 UNCHANGED, no new
  shadowing/non-terminating/unreachable); `ast_shape_contract` GREEN 18/18; the 6 fully-certified
  grammars byte-identical (SV-only regen); `clippy_on_rust_change` GREEN (source stage **0 errors** —
  this grammar-only change touches NO hand-written source, so the source-clippy result is
  byte-identical to HEAD's pre-existing style-warning baseline; the generated-parser clippy stage is
  the pre-existing non-strict debt tolerated by the gate — proven pre-existing by its 64 errors in
  `rtl_frontend_parser.rs`, a grammar this leaf never regenerated); `mdbook_docs_gate` ✅.
- [x] **LOCKSTEP** — ledger `SV-0033` → `Released` (fix record + `1.0.161` no-bump) + new `SV-0034`
  row (`Root Caused`: the sibling SV-only associative-array `[type]` / queue `[$]` dims of
  `variable_dimension` also leak, found tools-first while fixing `SV-0033`); conformance contract
  JSON re-pinned (`1146/4/815/327` + `.6.9` `baseline_note` hop + 2 case entries); book
  `parser-families.md` (`SV-0033` FIXED + cert re-pin + `SV-0034` note; mdbook gate ✅); SV
  integration contract (`SV-0033` fixed record + `SV-0031`/`SV-0034` open); LIVE dialect block
  (waiver set still four: `SV-0024`/`SV-0028`/`SV-0031`/`SV-0034`); tree + `docs/TASK_TREE.md`
  frontier; CHANGES / DEVELOPMENT_NOTES / MEMORY. Release/schema unchanged (`1.0.161`/15 — SV
  profiles AST-byte-invariant).
- [x] **ALERT-DISCIPLINE (BE-ALERT)** — probing `unsized_dimension`'s siblings in the same
  `variable_dimension` rule surfaced (tools-first) `SV-0034`: `module m; reg q [integer];`
  (associative) and `module m; reg q [$];` (queue) both ACCEPT under `verilog_2005` (a genuine
  separate leak — the `associative_dimension`/`queue_dimension` alternatives at `:5803-5804` are
  ungated SV-only). Ledgered `Root Caused` with WHY+WHERE, kept OUT of this surgical one-defect fix
  per the scope-boundary rule.

## Acceptance Checklist (`.6.8`, enforced)

- [x] **REPRODUCE / ISSUE** — pre-fix parse-probe on HEAD release binary: `module m; function
  void f; input a; f = a; endfunction endmodule` **ACCEPTS under `--profile verilog_2005`**
  (`parse_full passed`), and the `void'(…)` cast `module m; initial void'(f()); endmodule`
  **ACCEPTS under `verilog_2005`** too; both also ACCEPT under `sv_2017` (legal IEEE 1800).
  IEEE 1364-2005 §10.4 function return types are `[signed][range]`/`integer`/`real`/`realtime`/
  `time` — no `void`; the token `void` appears nowhere in 1364-2005 Annex A.
- [x] **ROOT CAUSE (WHY + WHERE)** — `grep -n kw_void grammars/systemverilog.ebnf` names exactly
  two ungated `kw_void_e9cede9b` surfaces: `data_type_or_void := … | kw_void_e9cede9b ->
  {kind:"void"}` (`:1838`, the return-type/member slot, reachable under `verilog_2005` via
  `function_data_type_or_implicit`/function prototypes/`struct_union_member`) and
  `subroutine_call_statement := … | kw_void_e9cede9b tick lparen function_subroutine_call rparen
  semi -> {kind:"void_cast"}` (`:5174`). Both rules are UNTAGGED (universal) → survived the
  `verilog_2005` profile filter and rode into the baseline un-audited (the `.2`/`.4.1`
  baseline-admission family: statically satisfiable, off-dialect, invisible to the
  orphan-coherence lint). AST-dump confirms the accepting path
  `… → function_declaration (sv_2017 variant, v2005-admitted) → data_type_or_void (void)`.
- [x] **FIX** — declarative, grammar-only (fix-hierarchy tier: grammar; no engine change): two
  shape-preserving `_sv_only` lifts per the `.6.3`/`.4.2` idiom —
  `data_type_or_void_sv_only := kw_void_e9cede9b -> {kind:"void"}` (`grammars/systemverilog.ebnf`,
  above `data_type_or_void`) and `void_cast_statement_sv_only := kw_void_e9cede9b tick lparen
  function_subroutine_call rparen semi -> {kind:"void_cast", body: $4}` (above
  `subroutine_call_statement`), both `@profiles: ["sv_2017","sv_2023"]`, referenced as bare
  pass-through alternatives (PEG order preserved). Emitted source carries the profile guard
  (`parse_data_type_or_void_sv_only` / `parse_void_cast_statement_sv_only` in
  `generated/systemverilog_parser.rs`).
- [x] **ADDRESSED (verified)** — both probes flip ACCEPT→**REJECT** under `verilog_2005` on the
  regenerated release binary (`function void f; …` and `initial void'(f());`); both still ACCEPT
  under `sv_2017`/`sv_2023`; the sv_2017 AST is byte-identical — `{kind:"void"}` and
  `{kind:"void_cast"}` present, **zero** `data_type_or_void_sv_only`/`void_cast_statement_sv_only`
  wrapper keys in the dumped JSON (bare-ref passthrough is transparent, no extra nesting); the 2
  new corpus reject-locks (`function_void.sv`, `void_cast.sv`) hold in the 198-check matrix.
- [x] **NO REGRESSION** — `verilog_2005_conformance_gate` **GREEN** (lint orphans=0, corpus 198
  checks / 0 mismatches, aliases 2, cert `1147/4/816/327` deterministic seeds 0/7/42 — hard pins
  byte-identical; `kw_void_e9cede9b` moved UNKNOWN-genuine → NO-reach, NO-reach 295→296);
  `sv_cert_recognized_union_gate` **GREEN** (canonical `1343/2/1321/UNKNOWN=20`, union
  `1343/2/1340/UNKNOWN=1`, residual `context_member_method_call`, seeds 0/7/42 — the 2 new sv_only
  rules witnessed under sv_2017, +2 total/witness, UNKNOWN invariants unchanged); `--lint-grammar`
  0 `verilog_2005` orphans (census 1463→1465, no new shadowing/non-terminating/unreachable); the 6
  fully-certified grammars byte-identical (SV-only regen; json `9/0` + regex `198/0` spot-checked
  `fully_certified=true`); `ast_shape_contract` GREEN; clippy source strict-clean.
- [x] **LOCKSTEP** — ledger `SV-0032` → `Released` (fix record + `1.0.161` no-bump); book
  `parser-families.md` (void rejects in worked examples + matrix 66/198 + NO-reach 296 +
  `.6.6`-surfaced-leaks note, `SV-0032` FIXED / `SV-0031`/`SV-0033` open; mdbook gate ✅); both
  cert contract JSONs re-pinned with provenance (union pins `1341→1343`; conformance
  `baseline_note` 295→296 + `.6.8` hop); LIVE dialect block (waiver set now FOUR); tree +
  `docs/TASK_TREE.md` frontier; CHANGES / DEVELOPMENT_NOTES / MEMORY. Release/schema unchanged
  (`1.0.161`/15 — SV profiles AST-byte-invariant).

## Acceptance Checklist (`.6.2`, enforced)

- [x] **REPRODUCE / ISSUE** — pre-fix parse-probe matrix on HEAD binaries: `wire #1step w;`,
  `wire #10ns w;` (SV-0025), `assign w = 10ns;`, `assign w = '0;` (SV-0027) all **ACCEPT under
  `--profile verilog_2005`** (each also ACCEPT under `sv_2017`/`sv_2023` — legal IEEE 1800);
  IEEE 1364-2005 A.2.2.3 / §literals have none of these forms. AST dumps name the accepting
  paths (`delay.body.body.kind="step"` / `kind:"time_literal"` via `delay_value`;
  `primary_literal → time_literal / unbased_unsized_literal`).
- [x] **ROOT CAUSE (WHY + WHERE)** — `PGEN_REACH_PATH_DUMP` named the live chain
  `net_declaration_sv_2017 → delay_control → delay_value ("root/o4")` for `kw_n_1step`
  (`.6.1` Step 2); `delay_value` (`grammars/systemverilog.ebnf:1872`) and `primary_literal`
  (`:4079`) are shared UN-SPLIT core rules, so `verilog_2005` rides ALL their branches — the
  SV-only alternatives (`time_literal`, `1step`; `time_literal`, `unbased_unsized_literal`)
  were never gated by the `.4.1`/`.4.2` waves. Carrier census tool-proven (grep + reach map):
  no other active carrier under the profile.
- [x] **FIX** — declarative, grammar-only (fix-hierarchy tier: grammar; no engine change): two
  shape-preserving gated named lifts per the `.4.2` idiom — `delay_value_sv_only` +
  `primary_literal_sv_only`, both `@profiles: ["sv_2017","sv_2023"]`, adjacent-branch,
  PEG-order-preserving; parent rules reference them as bare pass-through alternatives.
- [x] **ADDRESSED (verified)** — all 4 probes flipped ACCEPT→**REJECT** under `verilog_2005`
  on the regenerated parser (fresh release probe build verified by mtime — an initial stale-binary
  probe run was caught and discarded); the profile guard is in the emitted source
  (`rule_profile_is_enabled(&["sv_2017","sv_2023"])` in both `parse_*_sv_only` fns); the 4 new
  corpus reject-locks hold in the 150/150 matrix.
- [x] **NO REGRESSION** — the same 4 inputs still ACCEPT under `sv_2017`/`sv_2023`; 12/12
  before→after AST dumps **byte-identical** (4 leak + 2 control files × 2 SV profiles);
  `--lint-grammar` rc=0, `profile_orphans=0`, `ordered_choice_shadowing=0`, warnings unchanged
  (census 1446→1448 = the 2 lifts exactly); `verilog_2005_conformance_gate` **GREEN fresh**
  (`gate_green: true`, lint lock + 150/150 matrix + 2/2 aliases + cert `1138/2/817/319`
  byte-identical seeds 0/7/42, `spf=0`); `sv_cert_recognized_union_gate` **GREEN fresh**
  (canonical `1326/2/1304/UNKNOWN=20`, union `1323/UNKNOWN=1`, residual
  `["context_member_method_call"]` — semantic invariants byte-identical, only the +2 accounted
  counts moved); `ast_shape_contract` 18/18; `embedding_api` 51/0; realistic corpus direct-parse
  subset 478/478 (the 252 directive-carrying files need the preprocess lane — verified
  failing-only-for-preprocessing, 0 directive-free failures); external corpus non-uvm triage
  green (see Verification Log); clippy source clean.
- [x] **LOCKSTEP** — ledger `SV-0025`→`Released` + new `SV-0027` (`Released`, born-fixed);
  SV integration contract (corpus/matrix/cert pins + honest boundary + trust posture); main book
  `parser-families.md` (same pins + fixed/open boundary sentence); LIVE dialect block +
  session-#19 tracker note; conformance + union contract JSONs re-baselined with provenance
  notes; tree + `docs/TASK_TREE.md` frontier → `.6.3`; CHANGES / DEVELOPMENT_NOTES / MEMORY;
  release/schema unchanged (`1.0.158`/13).

## Acceptance Checklist (`.5`, enforced)

- [x] **REPRODUCE / ISSUE** — `sv_cert_recognized_union_gate` RED on counts at HEAD (session
  #17's fresh run + this session's re-read of its `union_gate.txt`): `canonical total=1324
  (expected 1304)` + proof/witness drift, while `canonical UNKNOWN=20`, `union UNKNOWN=1`,
  residual `["context_member_method_call"]` all MATCHED (counts-only drift).
- [x] **ROOT CAUSE (WHY + WHERE)** — git-traced (`.2` Findings): contract pinned at `5d8801d6`
  (release `1.0.151`); the `SV-0014`→`SV-0020` named-lifts (`1.0.152`→`1.0.158`, +7 rules) and
  the `verilog_2005` campaign's accounted rules (`.4.1` +3, `.4.2` +10) landed without
  re-baselining the count pins — a stale-pin lockstep gap, not a cert regression (every added
  rule witnessed/proven in its landing leaf).
- [x] **FIX** — the 4 count pins in `systemverilog_recognized_cert_union_contract.json`
  (`1304→1324`, `1→2`, `1283→1302`, `1302→1321`); zero semantic-pin changes.
- [x] **ADDRESSED (verified)** — `make -C rust SHELL=/bin/bash sv_cert_recognized_union_gate`
  RED→**GREEN**: `recognized_basis_green: true`, `unmet_criteria_count: 0`, canonical
  `1324/2/1302/UNKNOWN=20/spf=0`, union `witness=1321 UNKNOWN=1`, residual
  `["context_member_method_call"]`, byte-identical across seeds 0/7/42
  (`rust/target/sv_cert_recognized_union_gate/summary.{txt,json}`).
- [x] **NO REGRESSION** — contract-pin-only change (no grammar / Rust / generated /
  shape-manifest edit): the gate re-ran the real cert oracle fresh at seeds 0/7/42 with `spf=0`
  and the semantic invariants byte-identical to every recorded run since `-0147`;
  `mdbook_docs_gate` GREEN after the book edit; clippy N/A.
- [x] **LOCKSTEP** — book `grammar-wellformedness.md` + SV integration-contract trust statement
  re-baselined with provenance notes; tree + `docs/TASK_TREE.md` frontier advanced; LIVE dialect
  block left-to-close updated; CHANGES / DEVELOPMENT_NOTES / MEMORY.

- ID: `VERILOG-2005-PROFILE.6` — **frontier** (active umbrella, 2026-07-02 session #19): ratchet
  the `verilog_2005` profiled cert-coverage baseline (pinned `1138/2/826/310/spf=0`). Adjudicate
  the 310-UNKNOWN residual tools-first (the 3-step protocol): the 277 NO-reach-path candidates are
  expected profile-unreachable SV-only surface (candidates for per-profile `proof` accounting
  rather than `UNKNOWN` — an engine-accounting question, own design pass), and the ~33 genuine
  reach/witness gaps under the profile are the ratchet targets. Every improvement re-baselines
  `verilog_2005_conformance_contract_v0.json` in the same commit.

  - ID: `VERILOG-2005-PROFILE.6.1` — Status: `done` (2026-07-02, `PGEN-VERILOG-2005-PROFILE-0011`)
    — INVESTIGATION leaf (adjudication, ZERO code change): the full 310-UNKNOWN profiled residual
    is now classified with named tool evidence (see `.6.1` Findings below). Headline: the pinned
    baseline reproduced byte-for-byte (`1138/2/826/310/spf=0`); 33 genuine-gap candidates =
    310 − 277 NO-reach-path; every one of the 33 adjudicated via `[plannable-probe]` verdicts +
    reach-path dumps + parse probes + a scoped `--trace-rules` predicate trace into five
    mechanism classes (A store-gated SV-only use-sites ×17, B spurious-reach-through-gated-
    siblings ×8, C **two confirmed profile LEAKS** — ledgered `SV-0025` (`delay_value` `1step`/
    `time_literal` un-gated) + `SV-0026` (`description → package_item` `$unit` surface active) —
    ×1 rule, D in-profile witnessable ratchet targets ×6, E known canonical residual ×1).
    Fix-leaf plan spawned as `.6.2`–`.6.5`.

  - ID: `VERILOG-2005-PROFILE.6.2` — Status: `done` (2026-07-02, `PGEN-VERILOG-2005-PROFILE-0012`)
    — CODE leaf (grammar-only): closed the SV-only literal/delay leak surface under
    `verilog_2005` — `SV-0025` (delay forms) + the same-mechanism `SV-0027` found while pinning
    the fix locus (`primary_literal`'s adjacent SV-only branches `time_literal` +
    `unbased_unsized_literal`; probes `assign w = 10ns;` / `assign w = '0;` ACCEPTed at HEAD;
    single-carrier proven — `unbased_unsized_literal` referenced ONLY by `primary_literal`;
    `time_literal`'s third carrier `timeunits_declaration` already profile-dead via
    `kw_timeunit`/`kw_timeprecision`). TWO shape-preserving gated lifts per the `.4.2` idiom,
    both `["sv_2017","sv_2023"]`, PEG order preserved: `delay_value_sv_only`
    (`time_literal` | `kw_n_1step`) and `primary_literal_sv_only` (`time_literal` |
    `unbased_unsized_literal`). 4 corpus reject-locks added (matrix 138→150 checks); the
    `verilog_2005` contract cert pins re-baselined `826/310`→`817/319` (the 9 dropped witnesses
    are EXACTLY the leak-earned false ones: `time_literal`, `time_unit`,
    `unbased_unsized_literal`, the 6 time-unit keyword tokens — set-diff proven; NO-reach
    277→287 incl. `kw_n_1step` itself) and the union-gate pins re-baselined for the +2 lifted
    rules (`1326/2/1304/1323`; UNKNOWN invariants byte-identical). Both gates GREEN fresh.
    Release/schema unchanged (`1.0.158`/13 — SV-profile behavior + AST byte-invariant, 12/12
    dumps). See Acceptance Checklist (`.6.2`).

  - ID: `VERILOG-2005-PROFILE.6.3` — Status: `done` (2026-07-02, session #20,
    `PGEN-VERILOG-2005-PROFILE-0016` — REPLAYED on the `.6.3.2`-fixed engine and LANDED;
    `SV-0026` → `Released`): the recorded two-lift diffs applied verbatim; lint 0 orphans
    (census 1450); 12/12 probes (4 top-level forms × 3 profiles) flip exactly; canonical cert
    `1328/2/1306/UNKNOWN=20 spf=0` seeds 0/7/42 with the 20-rule residual set md5-identical to
    the pre-lift baseline — the `.6.3.1` collateral is GONE (the carrier-diversification pass's
    prelude now declares a PROPERTY per the `.6.3.2` integrity fix); union gate GREEN with
    re-pins earned (`1328/2/1306/20`, union `1325/1`, residual `context_member_method_call`);
    conformance gate GREEN 162/0 (4 new reject-locks: `top_level_{wire,reg,localparam,parameter}`)
    with cert re-pinned `817/319`→`809/327` (NO-reach 287→294 — honest leak-fix direction);
    shape 18/18; mdbook; clippy. Ledger/contract/book/LIVE lockstepped. Release/schema unchanged
    (`1.0.158`/13). See "Acceptance Checklist (`.6.3`)". Original blocked-checkpoint record
    `PGEN-VERILOG-2005-PROFILE-0013` retained below: CODE leaf — fix `SV-0026`. The fix was
    BUILT and PARSE-VERIFIED this session, then **REVERTED** because it cannot earn NO-REGRESSION
    yet (see `.6.3` Findings): (a) a SECOND carrier was found tools-first — `source_text_item`
    carries direct top-level `local_parameter_declaration semi` / `parameter_declaration semi`
    alternatives that bypass `description` (top-level `localparam p = 1;` / `parameter p = 1;`
    ACCEPT under `verilog_2005`); (b) the two shape-preserving gated lifts
    (`description_unit_item_sv_only` := `attribute_instance* package_item`;
    `source_text_item_unit_sv_only` := the two parameter alternatives; both
    `["sv_2017","sv_2023"]`, order-preserving) flipped all 4 top-level probes to REJECT under
    `verilog_2005` with `sv_2017`/`sv_2023` ASTs 12/12 byte-identical, lint 0 orphans (census
    1448→1450), 162/162 matrix, and `verilog_2005` cert `1138/2/809/327` deterministic; BUT
    (c) canonical `sv_2017` cert went `UNKNOWN` 20→**21** deterministically (seeds 0/7/42) —
    `known_unscoped_property_identifier` lost its witness in BOTH SV entry profiles → the pinned
    recognized-union invariant (canonical 20 / union 1) would break. Landing was therefore
    STOPPED per signoff discipline; grammar/corpus/contract reverted to the `.6.2` state
    (restoration verified). The lift diffs are recorded verbatim in the Findings for replay once
    `.6.3.1` restores the witness.

  - ID: `VERILOG-2005-PROFILE.6.3.1` — Status: `done` (2026-07-02, session #20, ZERO code —
    INVESTIGATION leaf, engine-side WHY+WHERE, tools-first): the witness-routing anomaly is
    **fully tool-named** (see "`.6.3.1` Findings"). WHY = the armed name-prelude's INJECTED
    construct must derive the producer's host `property_declaration` INCLUDING its deep
    mandatory sibling `property_spec`; the per-target witness budget never accounts for the
    prelude sub-path (tier-1 = `reach_prefix + min_subtree[target]`; tier-2 adds
    `max_offpath_mandatory_sibling_depth` over the MAIN hop chain only), so the forced
    `property_declaration` branch dies with `Stimuli generation depth exceeded max_depth=64
    while expanding rule 'number'`; `generate_or`'s forced-first-with-fallback then silently
    selects `sequence_declaration`, and the injection loop accepts ANY `Ok` render without
    verifying the armed `(kind, family)` fact was emitted — the prelude emits `sequence_name`
    instead of `property_name`. WHERE = `rust/src/ast_pipeline/stimuli_generator.rs`: injection
    loop in `generate_quantified` (`:9880-9913`, no fact-kind integrity), forced-order fallback
    in `generate_or` (`:8540-8555` ordering, `:8896..` Err fallback), budget tiers in
    `run_plannable_witness_pass` (`:4089-4101` tier-1, `:4210-4221` tier-2 via
    `max_offpath_mandatory_sibling_depth` `:6626` — main-chain-only). HEAD survives only because
    the main chain's host `property_declaration` self-emits `property_name` (s1
    `declared_property_identifier`) and the replay echoes it; the `.6.3` lifts re-roll the BFS
    to the top-level `bind` route (no property host on path) → the gate depends entirely on the
    wrong-kind prelude → 48/48 (= 4 attempts × 2 tiers × 2 entry-configs × 3 seeds). The direct
    `declared_property_identifier` probe is the control: NO prelude armed (0 spec lines), tier-1
    fails with the SAME depth error ×4 → tier-2 (deep sibling IS on the main chain there)
    rescues it — working as designed. Fix commissioned as `.6.3.2`.

  - ID: `VERILOG-2005-PROFILE.6.3.2` — Status: `done` (2026-07-02, session #20,
    `PGEN-VERILOG-2005-PROFILE-0015`): CODE leaf (ENGINE fix, general/parser-agnostic —
    `stimuli_generator.rs`): the armed name-prelude now honors its semantic contract ("the
    injected iteration exists to emit one fact of the armed `(kind, family)`"). As designed from
    the `.6.3.1` evidence: (1) `ReachPrelude` carries its body→producer `sub_hops`; (2) in the
    `generate_quantified` prelude-injection loop, after each injected render (and on a render
    `Err`), the generator verifies via `store_name_for_gate(arm)` that a matching fact now
    exists; on failure it rolls the discarded attempt back (store checkpoint + word-shape
    flags) and retries ONCE under a depth-fresh budget = injection depth + current budget + the
    SUB-path's deepest mandatory off-path sibling (the tier-2 measure via the new hops-generic
    `max_offpath_mandatory_sibling_depth_along_hops`, applied to the sub-path the tiers cannot
    see); a retry that still lacks the armed fact fails LOUDLY (`Armed name-prelude integrity
    failure: …`) instead of handing the gate an unusable store. Count-preludes
    (`name_gate=None`) and every already-correct injection are byte-identical by construction.
    VERIFIED: the property-gate witness now carries a PROPERTY prelude
    (`property\foo ;…endproperty property\foo_0 ;\foo_0 endproperty`); canonical
    `1326/2/1304/20` byte-identical (headline + full residual + NO-reach lists) at seeds
    0/7/42; plannable pass 1054→1055 witnessed with probe-parse-failures 68→60 (honest
    direction); union gate GREEN fresh (pins exact); `verilog_2005` gate GREEN fresh
    (`1138/2/817/319`, 150/150, 0 orphans); ast_shape_contract 18/18; clippy source
    strict-clean; 5/6 fully-certified grammars UNKNOWN=0 green and the 6th
    (rtl_const_expr) canonical-cert timeout PROVEN PRE-EXISTING by a git-stash A/B control
    (identical failure on the pre-change binary — spun off as `CERT-GEN-BUDGET.3`). See
    "Acceptance Checklist (`.6.3.2`)". `.6.3` is UNBLOCKED — replay next.

  - ID: `VERILOG-2005-PROFILE.6.4` — Status: `done` (2026-07-02, session #21,
    `PGEN-VERILOG-2005-PROFILE-0017`, ZERO code — INVESTIGATION/ADJUDICATION leaf, tools-first):
    the class-D ratchet as scoped is **not earnable**; the 6 targets fully re-adjudicated on the
    HEAD (post-`.6.3`) grammar/binaries (see "`.6.4` Findings"). (a) The identifier TRIO
    (`simple_identifier_no_scope`, `scope_free_identifier`, `ps_identifier`) is **already
    witnessed** — the `.6.3.2` carrier-diversification pass rescues them through the in-profile
    `module m(a,b);` carrier (`[carrier-div-probe] … parsed=true witnessed_target=true`); the
    pinned `809/327` baseline already contains those witnesses, so no delta is available from
    them. (b) `identifier_list` RECLASSIFIED class D→B: its SOLE reference sits inside
    `randomize_call`'s `with`-clause whose MANDATORY tail `constraint_block` is
    `@profiles ["sv_2017","sv_2023"]`-gated (`grammars/systemverilog.ebnf:4544`/`:1441`) —
    semantically unreachable under `verilog_2005`; the `.6.1` `randomize(a,b)` probe exercised
    `variable_identifier_list`, a different rule. → `.6.5` accounting. (c)
    `hierarchical_tf_identifier` RECLASSIFIED: its sole production MANDATES the `$root.` prefix
    (`:2372`); the `.6.1` `top.f(1)` probe never exercised it, and `$root` is SV-only → post-fix
    it is gated SV-only surface (class A-like), not a v2005 ratchet target. (d) `scalar_constant`
    BLOCKED on two tool-named grammar defects (digit-loss + PEG shadowing; ledgered `SV-0030`).
    MAJOR side discovery, ledgered + spawned: the **19 `kw_sv_dollar_*` keyword tokens match the
    literal text `sv_dollar_*` instead of the LRM `$*` spellings** (`SV-0029`; new tree
    `SV-DOLLAR-LRM-FIDELITY`). `verilog_2005` cert UNKNOWN stays `327` (= pins, headline
    reproduced byte-identical at seed 0); no code/grammar/generated change.

  - ID: `VERILOG-2005-PROFILE.6.5` — Status: `done` (2026-07-03, session #25,
    `PGEN-VERILOG-2005-PROFILE-0018`, DESIGN leaf — ZERO engine/grammar code; one contract
    PROSE fix): per-profile `proof` accounting DESIGNED and DECIDED — the three profile-gated
    residual classes ((i) NO-reach stranded rules, (ii) class-A store-gated use-sites with
    profile-unreachable producers, (iii) class-B spurious reach paths) ARE soundly provable and
    SHALL become `proof` entries, but STAGED: `.6.6` lands the read-only machine
    classification (headline-byte-identical, env-gated), `.6.7` promotes to certificates + full
    re-pin only after the classification reproduces the manual adjudication. Full design with
    tool-pinned WHY+WHERE for all three mechanisms in "`.6.5` Findings" below. Also landed: the
    deferred conformance-contract `baseline_note` prose fixes (stale "295 of the 326"→"of the
    327", a duplicated `$unit` phrase removed, the missing SV-DOLLAR-LRM-FIDELITY.4 re-pin hop
    appended) — pins untouched, gate re-run GREEN.

  - ID: `VERILOG-2005-PROFILE.6.6` — Status: `done` (2026-07-03, session #26,
    `PGEN-VERILOG-2005-PROFILE-0019`, CODE leaf — engine, parser-agnostic, READ-ONLY): the
    `.6.5`-designed residual classification LANDED — pure analyses
    `profile_entry_positively_live` (P1) + `classify_profile_residual` (P2 fixpoint) in
    `grammar_wellformedness.rs` (+ 6 unit tests), surfaced by the cert report behind
    `PGEN_CERT_RESIDUAL_CLASSIFICATION=1` (default output byte-identical — diff-proven at seed 0
    for BOTH the v2005 DUMP_ALL run and the canonical+union run; `reach_hops_pass` untouched).
    On `verilog_2005` the machine classes REPRODUCE the manual adjudication and reconcile the
    327 exactly (`292 P1 + 17 P2 + 18 genuine`; library/entry cohort all `genuine`, class-A =
    exactly the `known_unscoped_*`/`checked_*` 17 with per-rule kind reasons, class-B ×7 fall
    out as P1-unreachable, `identifier_list` = the `.6.4` D→B reading; md5-identical at seeds
    0/7/42) — and where machine and manual DISAGREED, the tools-first adjudication went to the
    MACHINE both times (`hierarchical_tf_identifier`: the `.6.4` note went stale at wave 3;
    `class_scoped_tf_call`: the negative-gated scoped escape parses in-profile), exposing THREE
    new ledgered `verilog_2005` boundary leaks (**`SV-0031`** `::` scope-resolution surface,
    **`SV-0032`** `void` return type, **`SV-0033`** dynamic-array `[]` — all `Root Caused`,
    own fix leaves). Full record in "`.6.6` Findings".
  - ID: `VERILOG-2005-PROFILE.6.7` — proposed (UNBLOCKED by `.6.6`): CODE leaf (engine + gate
    re-pins) — PROMOTION: new
    `WellformednessCertificate::{ProfileEntryUnreachable, ProfileUnproducibleGate}` variants
    + independent checkers; cert proof-gathering consumes them; ALL affected pinned gates
    re-baselined in the SAME slice (v2005 conformance cert pins, `sv_cert_recognized_union_gate`
    canonical pins, LIVE/book/contract lockstep). The leaf must enumerate the exact expected
    canonical-run delta from the `.6.6` machine output BEFORE landing. NOTE from `.6.6`: the
    v2005 conformance gate's cert invocation carries NO `--cert-union-config`, so promotion must
    first decide the gate-side entry-universe wiring (add union configs to the gate command, or
    plumb a dedicated entry-universe flag) — otherwise P1 under the gate's single-entry universe
    would prove the entry-relative library cohort dead, exactly what `.6.5` forbids.
  - ID: `VERILOG-2005-PROFILE.6.8` — Status: `done` (2026-07-03, session #27,
    `PGEN-VERILOG-2005-PROFILE-0020`, CODE leaf, grammar-only): **closed the `SV-0032`
    `verilog_2005`-only boundary leak — the SV-only `void` surface.** `void` is categorically absent from IEEE 1364-2005 (§10.4 lists function return
    types `[signed][range]`/`integer`/`real`/`realtime`/`time`; the token `void` appears nowhere
    in 1364-2005 Annex A), so EVERY `void` construct must REJECT under the strict profile. The
    `.6.6` machine classification's `genuine` verdict on `data_type_or_void`/`kw_void`, probed
    tools-first, exposed this leak (ledger `SV-0032`). Two ungated `kw_void_e9cede9b` surfaces
    rode into the `verilog_2005` baseline un-audited (the same `.2`/`.4.1` baseline-admission
    family as `SV-0026`/`SV-0031`, statically satisfiable but off-dialect → invisible to the
    orphan-coherence lint):
      1. `data_type_or_void`'s void return-type/member slot (`grammars/systemverilog.ebnf:1838`,
         `kw_void_e9cede9b -> {kind: "void"}`) — reachable under `verilog_2005` via
         `function_data_type_or_implicit` / function prototypes / `struct_union_member`.
      2. `subroutine_call_statement`'s `void'(…)` cast branch (`:5174`,
         `kw_void_e9cede9b tick lparen function_subroutine_call rparen semi -> {kind: "void_cast"}`).
    FIX (declarative, shape-preserving `_sv_only` lift idiom, cf. `.6.3` / `.4.2`): lift each void
    branch into a `@profiles: ["sv_2017","sv_2023"]`-gated named rule referenced bare (AST
    passthrough) — `data_type_or_void_sv_only` and `void_cast_statement_sv_only`. Under
    `sv_2017`/`sv_2023` the AST is byte-identical (bare-ref passthrough, `--parse-dump-ast`
    diff-proven); under `verilog_2005` the gated rules are pruned → `void` rejects. Corpus:
    `reject/function_void.v` + `reject/void_cast.v` (v2005 REJECT / sv_2017+sv_2023 ACCEPT).
    Re-pin surface (measured post-regen): the v2005 conformance cert pins + the
    `sv_cert_recognized_union_gate` canonical/union pins (the 2 new sv_only rules must earn a
    witness under `sv_2017`, else new UNKNOWN). Runs `--lint-grammar` (v2005 orphan count
    NON-INCREASING per the `.4.3` standing sub-rule). Ledger `SV-0032` → `Released`. See
    "Acceptance Checklist (`.6.8`)".
  - ID: `VERILOG-2005-PROFILE.6.9` — Status: `done` (2026-07-03, session #28, CODE leaf,
    grammar-only): **close the `SV-0033` `verilog_2005`-only boundary leak — the SV-only
    dynamic-array `[]` (unsized) dimension surface.** IEEE 1800 §7.5 dynamic arrays are
    SystemVerilog-only; IEEE 1364-2005 has no bare `[]` (unsized) dimension anywhere in Annex A —
    every array/vector dimension is a ranged `[msb:lsb]` or indexed form — so every `[]` construct
    must REJECT under the strict profile. The `.6.6` machine classification's `genuine` verdict on
    `dynamic_array_variable_identifier` / the `unsized_dimension` surface, probed tools-first,
    exposed this leak (ledger `SV-0033`, `Root Caused`). TOOLS-FIRST WHY+WHERE (session #28,
    fresh release binary): `module m; reg q []; endmodule` **ACCEPTS under `--profile
    verilog_2005`** (leak) and under `sv_2017` (legal); control `reg q [3:0];` (ranged, legal
    Verilog-2005) correctly still ACCEPTS under v2005. `unsized_dimension := lbrack rbrack`
    (`grammars/systemverilog.ebnf:5731`) is the single SV-only leaf, referenced from exactly three
    UNTAGGED (universal) contexts that all rode into the v2005 baseline un-audited (the same
    `.2`/`.4.1` baseline-admission family as `SV-0026`/`SV-0031`/`SV-0032` — statically satisfiable,
    off-dialect, invisible to the orphan-coherence lint): (1) `packed_dimension`'s alt-2
    (`:3823`, `unsized_dimension -> {kind:"unsized"}`); (2) `variable_decl_assignment`'s
    dynamic-array branch (`:5796`, `dynamic_array_variable_identifier unsized_dimension
    variable_dimension* ( assign dynamic_array_new )? -> {kind:"dynamic_array", …}` — a MANDATORY
    element, not an OR-alt); (3) `variable_dimension`'s alt-1 (`:5801`, `unsized_dimension ->
    {kind:"unsized", body:$1}`). FIX (declarative, grammar-only; fix-hierarchy tier: grammar):
    gate the SV-only `[]` surface out of `verilog_2005` via the established `@profiles:
    ["sv_2017","sv_2023"]` idiom — the minimal form that (a) keeps the `sv_2017`/`sv_2023` AST
    byte-identical, (b) rejects `reg q [];` under `verilog_2005`, and (c) preserves the
    `--lint-grammar` 0-`verilog_2005`-orphan lock — determined tools-first during implementation
    (candidate A: gate the whole SV-only rule `unsized_dimension` directly, precedent
    `uniqueness_constraint@:5724`; candidate B: per-context `_sv_only` lifts if A's
    mandatory-element cascade at `:5796` leaves an orphan). SCOPE BOUNDARY (per the surgical
    one-defect-per-commit rule): SV-0033 is the `[]` unsized dimension ONLY; the sibling SV-only
    dimension forms in `variable_dimension` — `associative_dimension` (`[data_type]`, `:627`) and
    `queue_dimension` (`[$…]`, `:4587`) — are a SEPARATE potential leak to verify + ledger
    independently (candidate `SV-0034`), NOT fixed here. Re-pin surface (measured post-regen): the
    v2005 conformance cert pins + the `sv_cert_recognized_union_gate` canonical/union pins (any new
    `_sv_only` rule must earn a witness under `sv_2017`, else new UNKNOWN); 2 corpus locks
    (`reject/dynamic_array_unsized.sv` REJECT-lock: v2005 REJECT / sv_2017+sv_2023 ACCEPT;
    `accept/array_ranged_dim.v` no-over-gate control: ranged dims all-profile ACCEPT). Ledger
    `SV-0033` → `Released`. See "Acceptance Checklist (`.6.9`)".
  - ID: `VERILOG-2005-PROFILE.6.10` — Status: `done` (2026-07-03, session #28, CODE leaf,
    grammar-only): **close the `SV-0034` `verilog_2005`-only boundary leak — the SV-only
    associative-array (`[data_type]`) and queue (`[$]`) dimension surfaces.** IEEE 1800 §7.8
    associative arrays and §7.10 queues are SystemVerilog-only; IEEE 1364-2005 has neither (§4.9 /
    A.2.5 dimensions are ranged `[msb:lsb]` only), so `reg q [integer];` (associative) and `reg q
    [$];` (queue) must REJECT under the strict profile. Surfaced tools-first while fixing `SV-0033`
    (`.6.9`) by probing the sibling alternatives of the same `variable_dimension` rule (ledger
    `SV-0034`, `Root Caused`). TOOLS-FIRST WHY+WHERE (session #28, fresh release binary): the queue
    form `module m; reg q [$]; endmodule` and the associative WILDCARD form `module m; reg q [*];
    endmodule` both **ACCEPT under `--profile verilog_2005`** (leak) and under `sv_2017`/`sv_2023`
    (legal). TOOLS-FIRST CORRECTION during implementation (the assumed carrier was wrong — verified,
    not trusted): the associative surface must be probed with the wildcard `[*]`, NOT `[data_type]`
    with a type keyword — `reg q [integer];` does NOT isolate `associative_dimension` because
    `variable_dimension` tries `unpacked_dimension` FIRST and the keyword `integer` parses there as a
    bare primary/`specparam` expression (`localparam p = integer;` likewise ACCEPTS under
    `verilog_2005`), a SEPARATE keyword-reservation/primary-leniency leak now ledgered `SV-0035`
    (`Root Caused`), NOT this dimension surface. `[*]` matches only the associative wildcard
    alternative, so it cleanly proves the gate. The two SV-only leaf rules —
    `associative_dimension := lbrack data_type rbrack` (`grammars/systemverilog.ebnf:627`) and
    `queue_dimension := lbrack kw_dollar ( colon constant_expression )? rbrack` (`:4587`) — are each
    referenced from EXACTLY ONE site: the corresponding `variable_dimension` alternative (`:5810`
    associative, `:5811` queue). `variable_dimension` itself stays v2005-reachable via its
    `unpacked_dimension` (ranged) alternative, so gating the two SV-only leaves prunes ONLY the
    associative/queue surface (same structure as `.6.9`'s `unsized_dimension`). FIX (declarative,
    grammar-only; tier: grammar): the whole-rule `@profiles: ["sv_2017","sv_2023"]` idiom (cf.
    `.6.9`/`uniqueness_constraint`) on BOTH single-carrier SV-only rules (`associative_dimension`,
    `queue_dimension`) — ZERO new rules. Re-pin surface (measured post-regen): v2005 conformance
    cert pins + `sv_cert_recognized_union_gate` pins (expected union byte-identical — no new rule,
    SV tree untouched); 2 corpus reject-locks (`reject/assoc_array_dim.sv`, `reject/queue_dim.sv`) +
    reuse `accept/array_ranged_dim.v` as the no-over-gate control. Ledger `SV-0034` → `Released`.
    See "Acceptance Checklist (`.6.10`)".

## `.6.1` Findings (tools-first, 2026-07-02 — the 310-UNKNOWN adjudication)

All evidence from the 3-step protocol on HEAD binaries (release `1.0.158`, schema `13`), profile
`verilog_2005`, entry `systemverilog_file`, `--count 40`:

- **Step 0 (`PGEN_CERT_COVERAGE_DUMP_ALL=1`, seed 0):** headline byte-identical to the contract
  pins — `total=1138 proof=2 witness=826 UNKNOWN=310 (spf=0, prf=0)`; 277 named NO-reach-path
  dead-rule candidates; set difference = 33 genuine-gap candidates.
- **Step 1 (`PGEN_CERT_COVERAGE_DEBUG_PROBES=1`):** 18/33 `parsed=false`, 15/33
  `parsed=true witnessed_target=false`; only 4/33 forced samples carry the off-profile
  `input logic a` scaffold (a witness-planner profile-awareness gap, noted for `.6.4`/`.6.5`).
- **Step 2 (`PGEN_REACH_PATH_DUMP=1` + scoped `--trace-rules` + parse/AST probes):** every
  candidate's mechanism named. Representative predicate trace (class A):
  `🚫 Rule 'checked_type_identifier' rejected by post predicate 'has_fact [type_name, "\foo"]'`
  with `↪ NEGATIVE: 3 facts of kind 'type_name' exist (none matched)` — under `verilog_2005` the
  `typedef` producers are profile-gated, so `type_name` facts are unproducible by construction.

**The classification (33 = 17 A + 8 B + 1 C + 6 D + 1 E):**

- **A — store-gated SV-only use-sites, producers profile-gated (17; unwitnessable by design):**
  typedef-fact family ×5 (`known_unscoped_block_type_identifier`,
  `known_unscoped_data_type_identifier`, `known_unscoped_block_class_type`,
  `provisional_unscoped_block_class_type`, `checked_type_identifier`); class-scope family ×7
  (`known_unscoped_class_scope_{class,interface_class,type_parameter}_identifier`,
  `known_unscoped_class_scoped_call_{class,interface_class,type_parameter}_identifier`,
  `class_scoped_tf_call`); covergroup ×2 (`known_unscoped_covergroup_type_identifier`,
  `known_unscoped_block_covergroup_identifier`); `known_unscoped_let_identifier`;
  `known_unscoped_sequence_identifier`; `checked_nettype_identifier`. Mechanism = the Step-2
  predicate trace above (fact kinds `type_name`/class/covergroup/let/sequence/nettype have no
  active producer under the profile). Accounting question → `.6.5`.
- **B — spurious reach paths through gated mandatory siblings (8; semantically dead):**
  `kw_packed`, `kw_rand`, `kw_randc`, `random_qualifier`, `struct_union_member` (reach hops
  route through `data_type`'s struct branch whose MANDATORY head `struct_union` is profile-gated
  — proven by `kw_struct`/`kw_union` being correctly NO-reach in the 277); `kw_type`
  (`localparam type …` branch — parse probe `module m; localparam type T = reg; endmodule`
  REJECTS, so unsatisfiable); `kw_inside`, `kw_matches` (reachable keyword PREFIX of
  `case_statement` branches whose gated arm rules make the branch unsatisfiable — probes
  `case (y) inside 0: …` and `case (y) matches 0: …` both REJECT). Reach-map soundness note →
  `.6.5` (iii).
- **C — confirmed profile LEAKS (over-acceptance defects; ledgered):** `kw_n_1step` — the reach
  path was REAL: `wire #1step w;` ACCEPTS under `verilog_2005` (AST names
  `net_declaration_sv_2017 → delay_control → delay_value` with `kind:"step"`); the sibling
  `time_literal` branch leaks too (`wire #10ns w;` ACCEPTS). Ledgered **`SV-0025`** → fix leaf
  `.6.2`. The adjudication also exposed **`SV-0026`** (not itself one of the 33): top-level
  `wire w;` / `reg r;` ACCEPT via `description → package_item` (`$unit` surface; IEEE 1364-2005
  A.1.2 allows only module/UDP/config) → fix leaf `.6.3`. Both forms correctly ACCEPT under
  `sv_2017` (legal IEEE 1800), so both leaks are `verilog_2005`-only.
- **D — in-profile witnessable ratchet targets (6):** `simple_identifier_no_scope`,
  `scope_free_identifier`, `ps_identifier` (probe `module m; wire #foo w; endmodule` ACCEPTS —
  the delay-identifier route is in-profile; the planner had routed them through the store-gated
  NETTYPE-headed `net_declaration` branch instead, sample `\foo #foo\foo ;` `parsed=false`);
  `hierarchical_tf_identifier` (probe `module m; initial top.f(1); endmodule` ACCEPTS —
  hierarchical task enables are legal 1364-2005); `identifier_list` (reached via
  `randomize_call`; probe `x = randomize(a);` ACCEPTS under `verilog_2005` — spec-correct, since
  `randomize` is NOT a reserved word in 1364-2005 and this is a plain function call; the SV-only
  `with {…}` form correctly REJECTS); `scalar_constant` (specify timing checks are core
  1364-2005 surface; forced sample doubly blocked — off-profile scaffold + malformed `1'b`
  literal). Ratchet leaf `.6.4`.
- **E — known canonical residual (1):** `context_member_method_call` — the same single union
  residual as under `sv_2017`; owned by `GRAMMAR-WELLFORMED.H.12.8.3.2`, NOT this tree.
- **The 277:** spot-checked consistent with profile-unreachable-by-design (SV-only roots and
  their `kw_*` tokens whose every referencing rule is gated; e.g. `kw_typedef`, `kw_struct`,
  `kw_class` all correctly NO-reach). Per-profile `proof` accounting → `.6.5`.

## `.6.3` Findings (tools-first, 2026-07-02 — the checkpoint record)

- **Second `SV-0026` carrier (tools-first):** the `.6.1` reach chains showed `kw_packed` routed
  `source_text_item → local_parameter_declaration` DIRECTLY (not via `description`); the rule
  read + probes confirmed `source_text_item` carries its own top-level
  `local_parameter_declaration semi` / `parameter_declaration semi` alternatives (both `$unit`-only
  per IEEE 1364-2005) and ALSO a bare `semi` alternative with **no LRM counterpart in EITHER
  standard** — the latter accepts a stray top-level `;` under every profile and is ledgered
  **`SV-0028`** (open, all-profile, own fix leaf; deliberately not bundled here).
- **The reverted fix (record for replay):** in `grammars/systemverilog.ebnf`, (1) above
  `description`: `@profiles: ["sv_2017", "sv_2023"]` +
  `description_unit_item_sv_only := attribute_instance* package_item -> {kind: "package_item", attributes: $1, body: $2}`,
  with `description`'s branch 6 replaced by the bare reference `description_unit_item_sv_only`;
  (2) above `source_text_item`: `@profiles: ["sv_2017", "sv_2023"]` +
  `source_text_item_unit_sv_only := local_parameter_declaration semi -> {kind: "local_parameter_declaration", body: $1} | parameter_declaration semi -> {kind: "parameter_declaration", body: $1}`,
  with the two inline branches replaced by the bare reference. Verified before revert: 4/4
  top-level probes (`wire w;`, `reg r;`, `localparam p = 1;`, `parameter p = 1;`) ACCEPT→REJECT
  under `verilog_2005` and ACCEPT unchanged under both SV profiles; `.6.2` locks + 5 controls
  unchanged; 12/12 before→after AST dumps byte-identical (both SV profiles); lint rc=0,
  `profile_orphans=0`, census 1448→1450; corpus matrix 162/162 with the 4 new reject rows;
  `verilog_2005` cert `total=1138 proof=2 witness=809 UNKNOWN=327 spf=0` byte-identical at seeds
  0/7/42.
- **The blocking collateral (deterministic, tool-named):** canonical `sv_2017` cert moved
  `1326/2/1304/UNKNOWN=20` → `1328/2/1305/UNKNOWN=21` at ALL of seeds 0/7/42; the residual diff
  names exactly one addition: `known_unscoped_property_identifier`. The sv_2023-entry config also
  lost it (present in its UNKNOWN list) ⇒ the sound multi-config union would go `UNKNOWN 1→2`,
  breaking the pinned recognized-union invariant ("SV is one rule from fully-certified").
  `PGEN_CERT_COVERAGE_DEBUG_PROBES`: 48/48 forced samples for the target are
  `parsed=true witnessed_target=false` with scaffolds declaring a **sequence**
  (`package \foo ; sequence \foo ; … endsequence endpackage bind …`) — the wrong fact kind for
  the `has_fact(property_name, $body)` gate. `PGEN_REACH_PATH_DUMP` names the use-site route
  (top-level `bind` → `checker_instantiation` → `property_actual_arg` → … →
  `property_instance`). Grammar census: `declared_property_identifier` (`:4401`) is the SOLE
  `property_name` producer, so a directed producer selection cannot legitimately pick a sequence
  — and the probe targeting `declared_property_identifier` ITSELF renders a sequence scaffold,
  meaning the anomaly is in the engine's prelude/forcing render, not the grammar edit (the edit
  merely re-rolled the routing). Engine WHY+WHERE = the new blocking leaf `.6.3.1`.
- **Restoration verified after revert:** grammar/contract/corpus back to the `.6.2` state;
  regen + both binaries rebuilt; canonical cert back to `1326/2/1304/20` and `verilog_2005` cert
  back to `1138/2/817/319` (seed 0), 150/150 matrix — see the Verification Log.

## `.6.3.1` Findings (tools-first, 2026-07-02 — the engine witness-routing anomaly, WHY+WHERE)

All evidence from HEAD binaries (release `1.0.158`, schema `13`), canonical config: profile
`sv_2017`, entry `systemverilog_file`, `--count 40 --seed 0`. Baseline reproduced byte-identical
first (`CERTIFICATE-COVERAGE: … total=1326 proof=2 witness=1304 UNKNOWN=20 spf=0 prf=0`;
`known_unscoped_property_identifier` NOT in the UNKNOWN list at HEAD). Decisive run =
`PGEN_TRACE_VERBOSITY=debug PGEN_CERT_COVERAGE_DEBUG_PROBES=1 PGEN_REACH_PATH_DUMP=1 …
--report-certificate-coverage …` piped through a scoped grep (name-prelude / semantic-prelude /
`assertion_item_declaration` / probe / reach-path lines).

- **The prelude arms CORRECTLY** (`stimuli_generator.rs:3138` spec trace):
  `STORE-AWARE-GEN.4b name-prelude spec: gated_rule='known_unscoped_property_identifier'
  kind='property_name' family=None producer='declared_property_identifier' clean_path=true
  site=('source_text','root') body='source_text_item'` — producer selection is right (the sole
  `property_name` emitter), site/body are right.
- **The injected render dies on DEPTH, then silently degrades family** (trace, injected
  construct): `Reach-plan branch forcing: rule='assertion_item_declaration' path='root'
  forced_local=0 forced_global=0` (property branch correctly forced first) →
  `OR branch failed: rule='assertion_item_declaration' path='root' branch=0
  reason=Stimuli generation depth exceeded max_depth=64 while expanding rule 'number'` →
  `Selected OR branch: … branch=1 output_len=38` (= `sequence\foo ;6309.412_E+39endsequence`).
  The injection loop (`generate_quantified` `:9880-9913`) accepts the `Ok` render with NO check
  that the armed `(kind, family)` fact was emitted → the prelude registers `sequence_name`, not
  `property_name`.
- **Why HEAD still witnesses (the accidental rescue):** the MAIN chain hosts the target inside
  `property_declaration` (hop `("property_declaration","root/s5")` → `property_spec` → … →
  `property_instance` → `ps_or_hierarchical_property_identifier o1`), whose own s1
  `declared_property_identifier` self-emits `property_name=\foo_0`; the replay echoes it
  (`C2 semantic-prelude replay: rule='known_unscoped_property_identifier' depth=39
  render='\foo_0 '`) → `[plannable-probe] … parsed=true witnessed_target=true
  sample="sequence\foo ;6309.412_E+39endsequence property\foo_0 ;\foo_0 endproperty"`. The
  wrong-family prelude is dead weight at HEAD; the host supplies the fact.
- **Why the `.6.3` lifts break it:** the lifts re-roll the BFS shortest path to the top-level
  `bind` route (`bind → checker_instantiation → property_actual_arg → … → property_instance`,
  per the `.6.3` reach dump) — NO `property_declaration` host on path → the gate depends
  entirely on the injected prelude → wrong-kind fact every time → 48/48
  `parsed=true witnessed_target=false` (= 4 attempts × 2 tiers × 2 entry-configs × 3 seeds).
- **Control (direct probe of the producer, same run):** `declared_property_identifier` has NO
  prelude armed (0 spec lines in its segment — it is excluded from `gen_name_gate` as a
  self-satisfying producer, `compute_name_gates` `:7039-7043`). Tier-1 (budget 64): 4× the SAME
  `OR branch failed … max_depth=64 … rule 'number'` → falls to sequence →
  `witnessed_target=false` ×4. Tier-2 (deep mandatory off-path sibling `property_spec` IS on
  the MAIN chain at `("property_declaration","root/s1")`) → `Selected OR branch: … branch=0` →
  `property\foo ;2737.2682endproperty` → witnessed. Tier-2 works when the deep sibling is
  visible to `max_offpath_mandatory_sibling_depth(entry, target)` (`:6626`); the PRELUDE
  sub-path's siblings are invisible to it — that is the budget blind spot.
- **Mechanism summary (the defect, engine, parser-agnostic):** (1) per-target witness budgets
  (tier-1 `:4089-4101`, tier-2 `:4210-4221`) never cover the armed prelude's injected construct
  (producer-host + its mandatory siblings, here `property_spec`'s ~25-level expression chain to
  `number`); (2) `generate_or`'s forced-first-with-fallback (`:8540-8555`, Err path `:8896..`)
  plus the integrity-blind injection loop (`:9880-9913`) convert that depth failure into a
  SILENT wrong-family prelude instead of a visible prelude failure. Blast radius of the fix =
  armed name-preludes only (99 spec lines / 10 distinct gated rules in the canonical run).
- **Fix leaf:** `.6.3.2` (integrity check + depth-fresh retry at the injection site). `.6.3`
  replays after it.

## `.6.4` Findings (tools-first, 2026-07-02 — the class-D ratchet re-adjudication)

All evidence from HEAD binaries (SV release `1.0.158`, schema `13`), grammar at the `.6.3`
state (census 1450), profile `verilog_2005`, entry `systemverilog_file`, `--count 40 --seed 0`:
`PGEN_CERT_COVERAGE_DUMP_ALL=1 PGEN_CERT_COVERAGE_DEBUG_PROBES=1 … --report-certificate-coverage …`.
Headline reproduced the contract pins byte-for-byte:
`CERTIFICATE-COVERAGE: … total=1138 proof=2 witness=809 UNKNOWN=327 (sample_parse_failures=0,
proof_reverify_failures=0)`.

- **The identifier trio is ALREADY WITNESSED** (`simple_identifier_no_scope`,
  `scope_free_identifier`, `ps_identifier` are NOT in the 327-rule UNKNOWN dump). Every
  plannable/target-own/store-free probe still fails on the off-profile
  `module m(input logic a);` carrier (the `module_ansi_header` rule-level `@sample` at
  `grammars/systemverilog.ebnf:3137` — `logic` is SV-only), but the `.6.3.2`-era
  carrier-diversification pass re-rolls the header and lands each of the three:
  `[carrier-div-probe] rule='simple_identifier_no_scope' parsed=true witnessed_target=true
  sample="module m(a,b);\foo #foo(\foo ,1.9E07);endmodule"` (same shape for the other two).
  The `.6.3` re-pin (`809/327`) already counts these witnesses — the leaf's stale
  `UNKNOWN 310→≤304` target text predated `.6.2`/`.6.3`.
- **`identifier_list` is class B, not D** (misclassification in `.6.1`): grep proves its SOLE
  reference is `randomize_call`'s `with`-clause (`:4544` —
  `kw_with ( lparen ( identifier_list )? rparen )? constraint_block`), and the MANDATORY tail
  `constraint_block` (`:1441`) is `@profiles: ["sv_2017","sv_2023"]`-gated — any sample carrying
  `identifier_list` must also render a `constraint_block`, unsatisfiable under `verilog_2005`
  (consistent with `.6.1`'s own "the SV-only `with {…}` form correctly REJECTS" probe). The
  `.6.1` class-D justification (`x = randomize(a);` ACCEPTS) exercised
  `variable_identifier_list` (`:5679`), a different rule. Probes: plannable/store-free forced
  samples all degenerate to `module m; endmodule` (`parsed=true witnessed_target=false` — the
  planner cannot render the gated tail). → per-profile accounting is `.6.5` scope.
- **`hierarchical_tf_identifier` is `$root`-mandatory, not in-profile**: the rule (`:2372`) is
  `kw_sv_dollar_root_f65f0e67 dot ( identifier constant_bit_select dot )* callable_identifier` —
  the `$root.` anchor is MANDATORY. The `.6.1` class-D probe `module m; initial top.f(1);
  endmodule` never touches this rule (no `$root.`). AST-dump control: `$root.top.f(1);` under
  `sv_2017` parses with `kind:"system_tf"` only (zero `hierarchical`/`root` kinds) — `$root` is
  consumed by the generic `system_tf_identifier` token (`:449`), never this rule. Since `$root`
  is SV-only (no counterpart in IEEE 1364-2005), the rule belongs to the gated SV-only surface
  once `SV-0029` is fixed. Witness probes: `parsed=true witnessed_target=false` on the
  non-scaffold carrier-div forms (mis-routed through `hierarchical_identifier`'s OPTIONAL
  `$root.` prefix, `:2356`), `parsed=false` elsewhere (scaffold).
- **`scalar_constant` is blocked on two grammar defects (`SV-0030`)**: (1) EXTRACTION DIGIT
  LOSS — IEEE 1364-2005/1800 `scalar_constant ::= 1'b0 | 1'b1 | 1'B0 | 1'B1 | 'b0 | 'b1 | 'B0 |
  'B1 | 1 | 0` (faithfully present in `grammars/systemverilog_2017_lrm_extracted.ebnf:951`);
  the active rule (`:4713`) lost every trailing digit (`kw_n_1_tick_b_f4c81681 := trivia "1'b"`,
  `:6068`). (2) PEG SHADOWING — `scalar_timing_check_condition` (`:4720`) lists bare
  `expression` FIRST, so the `expression equal scalar_constant` branches can never win:
  probe `$…(d &&& e == 1'b0, …)` ACCEPTS via the bare-expression branch (LRM spelling never
  reaches `scalar_constant`), probe `… e == 1'b, …` REJECTS at the outer sequence (the committed
  bare-`expression` choice consumes `e`, then `==` breaks the sequence). Generator forced
  samples render `==1'b` (duality with the defective grammar) and correctly `parsed=false`.
  The rule is covered on the `sv_2017` side only through the recognized-UNION lane (union
  `UNKNOWN=1` = `context_member_method_call` only). No honest v2005 witness exists until
  `SV-0030` lands (order the eq/ne branches before bare `expression` + restore the digits).
- **MAJOR side discovery — the `sv_dollar_*` mangled-literal token family (`SV-0029`, new tree
  `SV-DOLLAR-LRM-FIDELITY`):** 19 keyword tokens (`grammars/systemverilog.ebnf:6248-6284`)
  match the literal TEXT `sv_dollar_*` instead of the LRM `$*` spellings (e.g.
  `kw_sv_dollar_setup_b58bdaae := trivia /sv_dollar_setup\b/`). Provenance: the extracted LRM
  snapshots carry the true `$` spellings (zero `sv_dollar` matches in BOTH
  `systemverilog_2017_lrm_extracted.ebnf` and `verilog_2005_lrm_extracted.ebnf`); the mangling
  entered via the profiled-synthesis name canonicalization
  (`tools/extract_systemverilog_lrm_profiles.py:315` — `$name` → `sv_dollar_name`, correct for
  RULE NAMES, leaked into the keyword-token LITERALS). No pre-parse rewrite exists anywhere
  (`grep -rn "sv_dollar" rust/src/` → zero hits), so the defect is live in the released parser.
  Probes (parseability_probe, `sv_2017` unless noted): LRM `$setup(d, posedge clk, 1);` in
  specify REJECTS (`furthest_position=26`) and also REJECTS under `verilog_2005` (timing checks
  are CORE 1364-2005 §15 surface); nonsense `sv_dollar_setup(…)` ACCEPTS; LRM `$unit::y`
  REJECTS / `sv_dollar_unit::y` ACCEPTS; LRM module-level `$fatal;` (IEEE 1800 §20.11
  elaboration severity task) REJECTS / `sv_dollar_fatal;` ACCEPTS; `$root.top.f(1);` ACCEPTS
  but mis-routed (`kind:"system_tf"`). Affected surfaces: 12 specify timing checks
  (`system_timing_check`, `:5095+`), `$root` (`:2356`/`:2372`), `$unit` (`:3762`), the 4
  elaboration/severity tasks (`:2019+`/`:4871+`), the bare `$` primary (`:2976`/`:2993`/
  `:4026`/`:4081`). External-corpus 14/14 never exercised these spellings (no timing checks /
  `$unit::` / module-level severity tasks in the corpus), which is why the family stayed
  invisible.

## `.6.5` Findings (DESIGN, 2026-07-03 — per-profile proof accounting)

Design leaf: heavy engine reading + a fresh evidence run; ZERO engine/grammar code. Fresh
baseline (HEAD binaries, SV release `1.0.161`, schema `15`):
`PGEN_CERT_COVERAGE_DUMP_ALL=1 … --report-certificate-coverage --grammar-profile verilog_2005
--entry-rule systemverilog_file --count 40 --seed 0` reproduced the contract pins
byte-for-byte — `total=1147 proof=4 witness=816 UNKNOWN=327 (spf=0, prf=0)`, 295 NO-reach rules
enumerated in full, `327 − 295 = 32` attempted-but-unwitnessed.

### The tool-pinned mechanisms (WHY + WHERE, all three classes)

- **(i) Stranded-rule class — why NO-reach rules can't earn today's `UnreachableRule` proof.**
  `apply_grammar_profile_filter` (`rust/src/main.rs:2237`) prunes only rules whose OWN
  `@profiles` tag excludes the active profile; an untagged (universal) rule SURVIVES even when
  every rule referencing it is pruned (e.g. `kw_typedef` under `verilog_2005`). The cert's
  proof side (`main.rs:2497` → `gather_verified_proof_covered_rules`,
  `grammar_wellformedness.rs:1723`) detects whole-rule deadness via `detect_unreachable_rules`
  (`:268`), whose `reachable_rules` (`:284`) promotes every UNREFERENCED rule to a secondary
  ROOT (`:306-313` — the multi-entry-awareness that keeps `library_text` legitimate). A
  profile-stranded rule is therefore its own root → never "unreachable" → no proof → UNKNOWN,
  with only the pass-3 warning (`main.rs:2926`) naming it a "dead-rule candidate — adjudicate
  via the linter" — and the linter has NO profile-entry-unreachability check to adjudicate
  with (`detect_profile_orphans`/`compute_sat_by_profile`, `grammar_wellformedness.rs:778`/
  `:671`, prove the DOWNWARD axis — "can this rule derive a string under P" — not the UPWARD
  axis "can any entry reach it under P").
- **(ii) Class-A store-gated use-sites — why the linter's unbound-kind check can't fire.** The
  `type_name` emitters live on profile-AGNOSTIC `declared_*_identifier` helper rules
  (`grammars/systemverilog.ebnf:1018/:1022/:1587/:3507/:5464-:5481`) that SURVIVE the profile
  filter, so `detect_unbound_fact_kinds` (`grammar_wellformedness.rs:958` — kind-existence over
  the annotation set) still sees emitters. The kinds are unproducible under `verilog_2005` only
  because every PRODUCING CONTEXT (the gated declaration hosts referencing those helpers) is
  entry-unreachable under the profile — i.e. class (ii) is class (i) lifted through the
  annotation surface, and deadness CASCADES (a dead producer's emissions vanish → more rules
  die), so the sound analysis is a fixpoint.
- **(iii) Class-B spurious reach paths — where the BFS loses profile truth.** `reach_hops_pass`
  (`rust/src/ast_pipeline/stimuli_generator.rs:6800`) discovers edges from EVERY
  rule-reference site; a reference to a rule missing from the active (profile-pruned) tree is
  simply SKIPPED (`:6869` `continue`) instead of invalidating the enclosing MANDATORY branch —
  so `kw_packed` is "discovered" through `data_type`'s struct branch whose mandatory head
  `struct_union` is not in the active tree. The exact honesty needed ALREADY EXISTS in
  `mandatory_node_gated` (`:7120` — "a reference to a rule MISSING from the active
  (profile-pruned) tree is treated as gated — it cannot be rendered, so it is no escape",
  `:7161-7164`) but is only consulted by the LAST-pass store-free edge deprioritization, never
  by the default BFS or the NO-reach classification.

### The load-bearing structural finding: the 295 is HETEROGENEOUS

The fresh NO-reach dump shows the 295 contains TWO populations that any sound proof MUST
separate: (a) the genuinely profile-stranded SV-only surface (`kw_class`,
`declared_class_identifier`, the covergroup/SVA/constraint families, …) and (b) the
ENTRY-RELATIVE infrastructure (`library_text`, `library_declaration`, `sv_multi_entry_root`,
`systemverilog_parseable_file`, `include_statement`, …) — rules rooted under OTHER declared
entries, partly core 1364-2005 §13 surface (library maps/config are legal Verilog-2005), which
are NOT dead under the profile. A proof keyed on the single cert entry would falsely brand (b)
dead. **Therefore the certificate must quantify over the DECLARED ENTRY UNIVERSE** (the cert
entry + the `--cert-union-config` entries present in the active filtered tree), not the single
`--entry-rule`.

### The current 32-rule attempted-but-unwitnessed residual (fresh, seed 0)

Class A (mandatory positive store-gate, producers profile-unreachable — 18):
`known_unscoped_class_scope_{class,interface_class,type_parameter}_identifier`,
`known_unscoped_block_type_identifier`, `known_unscoped_data_type_identifier`,
`known_unscoped_block_class_type`, `provisional_unscoped_block_class_type`,
`known_unscoped_covergroup_type_identifier`, `known_unscoped_block_covergroup_identifier`,
`known_unscoped_let_identifier`, `checked_nettype_identifier`,
`wildcard_escape_nettype_identifier`, `known_unscoped_sequence_identifier`,
`checked_type_identifier`, `known_unscoped_class_scoped_call_{class,interface_class,
type_parameter}_identifier`, `class_scoped_tf_call`. Class B (spurious reach through a gated
mandatory sibling — 7): `kw_packed`, `kw_randc`, `random_qualifier`, `struct_union_member`,
`kw_type`, `kw_inside`, `kw_matches`. Class E (canonical residual — 1):
`context_member_method_call`. Post-`.6.1` drift entries needing `.6.6` adjudication (6):
`data_type_or_void`, `kw_void`, `dynamic_array_variable_identifier`, `function_statement`,
`hierarchical_tf_identifier`, `identifier_list` (the SV-DOLLAR waves re-routed reach paths
since `.6.1`; `kw_rand` left the residual). That the manual snapshot drifted in ONE DAY of
grammar work is itself design evidence: prose adjudication goes stale per-commit; only a
machine classification stays current.

### The DESIGN (decided)

**Decision: the three classes SHALL become sound `proof` entries — per-profile proof
accounting is adopted as the end state — but STAGED behind a read-only machine classification
that must first reproduce the manual adjudication.** Leaving mechanizable truth as contract
prose contradicts `DOCTRINE_ENFORCEMENT.md` ("a doctrine that is not mechanically checked is
not enforced"); conversely, re-pinning every gate on the strength of a brand-new analysis with
no independent confirmation surface is the tournament-loser-leak mistake. Staging resolves
both.

**P1 — profile-entry-universe unreachability (covers classes i + iii).** New pure analysis in
`grammar_wellformedness.rs`: over the ACTIVE (profile-filtered) tree, the positively-reachable
set from the DECLARED ENTRY UNIVERSE, with SATISFIABILITY-HONEST edges — a reference site
contributes an edge only if its enclosing derivation within the referencing rule is
satisfiable under the profile (reusing the `node_satisfiable` algebra: Or = any alternative,
Sequence = all elements, min-0 quantifier = skippable, lookahead = no positive edge, a
reference to a missing rule = unsatisfiable — the `mandatory_node_gated` missing-rule half
lifted to the proof layer). A rule outside that set is `profile_entry_unreachable`: no parse
from any declared entry under P can positively enter it. Decidable, pure, independently
re-derivable. Class-B rules fall out automatically (their only "reach" was through an
unsatisfiable branch). The default reach BFS (`reach_hops_pass`) is deliberately NOT touched —
witness-landscape neutrality (the `.6.3` collateral lesson).

**P2 — profile-unproducible mandatory store-gate (covers class ii).** Fixpoint composed with
P1: fact-kind K is producible under P iff SOME P1-live rule carries an `@emit_fact` of kind K;
a rule whose MANDATORY descent (the `mandatory_node_gated` algebra) forces a POSITIVE
fact-query (`has_fact` / `fact_attribute_equals` / `fact_count_at_least` min≥1 — NEVER
`lacks_fact` or other negative forms: an unsatisfiable NEGATIVE gate makes a rule ALWAYS-LIVE,
not dead) on an unproducible K is dead under P → remove → its emissions vanish → iterate to
fixpoint. Sound-core boundaries stated explicitly: (a) KIND-level only (attribute-level
`declaration_family=…` refinement deliberately out — kind-level suffices for the observed
classes); (b) if ANY P1-live rule carries `@import_from_library`, the analysis degrades to
inert for kinds that library kind can supply (external artifacts can inject facts without an
in-parse emitter) — under `verilog_2005` the import surface is SV-only-gated so no
degradation; (c) the certificate is scoped "under profile P with no external fact libraries"
(the cert-coverage accounting configuration — witness verification runs lib-free today).

**Staging.** `.6.6` (CODE): P1+P2 as pure functions + a cert-report classification of the
residual UNKNOWN into `profile_entry_unreachable` / `store_unproducible_under_profile` /
`genuine`, printed ONLY under `PGEN_CERT_RESIDUAL_CLASSIFICATION=1` (default output
byte-identical; precedent: the gap report's read-only `reach_classification` field). Entry
universe = canonical entry + `--cert-union-config` entries present in the active tree. `.6.7`
(CODE, blocked on `.6.6`): promotion — `WellformednessCertificate::ProfileEntryUnreachable
{ rule, profile, entries }` + `ProfileUnproducibleGate { rule, profile, kind }` variants with
independent checkers (re-derive from scratch, never trust the detector), proof-gathering
consumes them, and EVERY affected pinned gate is re-baselined in the same slice with set-diff
evidence (v2005 conformance cert pins → UNKNOWN becomes the genuine residual; the recognized
union gate's canonical pins move too — the stranded sv_2023-feature rules under `sv_2017` earn
the same proof; the union RESULT is invariant because per-config proofs already extend the
union, `main.rs:3012`). Expected post-`.6.7` v2005 posture: UNKNOWN ≈ entry-relative library
cohort + `context_member_method_call` + whatever `.6.6` adjudicates genuine — each then
individually ownable, which is exactly what the pins-not-closure posture cannot offer today.

## `.6.6` Findings (CODE + adjudication, 2026-07-03 — the machine classification and what it corrected)

**The landed surface.** Pure, parser-agnostic analyses in
`rust/src/ast_pipeline/grammar_wellformedness.rs`: P1 `profile_entry_positively_live`
(positive reachability from the DECLARED entry universe over the active profile-filtered tree,
satisfiability-honest edges — an unsatisfiable Or-alternative, a pruned mandatory sibling, or a
lookahead contributes no edge; a rule missing from the active tree but present in the pre-filter
tree reads PRUNED=⊥, missing from both reads external=⊤; NO unreferenced→secondary-root
promotion) and P2 `classify_profile_residual` (kind-producibility over the P1-live set from
per-rule `@emit_fact` enumeration across all three annotation surfaces; a live rule whose
RULE-LEVEL `@predicate` REQUIRES a positive fact-query — And=union, Or=INTERSECTION,
Not/Compare/In=nothing, `fact_count_at_least` only with a literal count ≥1 — on an unproducible
kind is dead; deadness cascades through the satisfiability composition itself, since a mandatory
reference to a dead rule makes the referencer unsatisfiable — the `mandatory_node_gated` algebra
is the satisfiability algebra's dual, so the in-loop mandatory check proved DEAD CODE and the
mandatory-descent walk is used at classification time to ATTRIBUTE each cascade casualty;
`@import_from_library` on any live rule degrades P2 to inert). Surfaced by the cert report ONLY
under `PGEN_CERT_RESIDUAL_CLASSIFICATION=1` (`main.rs`, after the `UNKNOWN` list; entry universe
= canonical entry + the `--cert-union-config` entries present in the active tree); the analysis
does not even run otherwise. `reach_hops_pass` untouched. 6 new unit tests (stranded-rule
no-self-root + second-entry rescue; satisfiability-honest edges + lookahead; gate/cascade/escape/
negative-gate; live-emitter + import-degradation; Or-intersection + literal-count; stranding).

**The machine classification (v2005, seed 0, entry universe = canonical +
`sv_multi_entry_root`/`library_text`/`systemverilog_parseable_file`):**
`profile_entry_unreachable (292)` + `store_unproducible_under_profile (17)` + `genuine (18)`
= 327 ✓, and the arithmetic reconciles the `.6.5` manual snapshot EXACTLY:
295 NO-reach − 11 entry-relative cohort (now live via the declared universe) = 284, + the 7
class-B spurious-reach rules (`kw_packed`, `kw_randc`, `random_qualifier`,
`struct_union_member`, `kw_type`, `kw_inside`, `kw_matches` — all falling out automatically as
P1-unreachable, exactly as designed) + `identifier_list` (the `.6.4` D→B reading confirmed) =
292. The 17 store rules are precisely the `known_unscoped_*`/`checked_*` class-A family, now
with machine-precise reasons: `type_name` ×13, `let_name`, `sequence_name`,
`wildcard_import_open`, and two mandatory-descent cascade attributions
(`known_unscoped_block_type_identifier`/`known_unscoped_data_type_identifier` →
`checked_type_identifier`; `known_unscoped_block_covergroup_identifier` →
`known_unscoped_covergroup_type_identifier`). The 18 genuine = the 11 entry-relative
library/include/parseable cohort (NEVER branded dead — the `.6.5` load-bearing requirement) +
`context_member_method_call` (class E ✓) + `class_scoped_tf_call` + the 5 drift entries.
Classification block md5-identical at seeds 0/7/42.

**Machine-vs-manual discrepancies — BOTH adjudicated tools-first, BOTH in the machine's favor:**

- `hierarchical_tf_identifier` (manual `.6.4`: "sole production MANDATES `$root.`" → A-like):
  STALE — at HEAD the rule has TWO alternatives (`grammars/systemverilog.ebnf:2403`): the
  `$root.`-rooted one (whose mandatory head `hierarchical_root_prefix_sv_only` is
  profile-gated, `:2383-84`) AND the `anchored` `identifier constant_bit_select dot (…)*
  callable_identifier` alternative added by the SV-DOLLAR wave-3 split — plain `top.f(1)`
  hierarchical task enables, core 1364-2005 §12.5 surface. Machine `genuine` CORRECT; the
  manual note aged out in one day of grammar work — the exact staleness the `.6.5` design
  predicted prose adjudication would suffer.
- `class_scoped_tf_call` (manual `.6.1`: class-A unwitnessable): the machine declined the
  deadness claim because `class_scoped_call_prefix_head` (`:6641`) carries a FOURTH
  alternative, `scoped_class_scoped_call_prefix_identifier` (`:6630`), gated ONLY by
  NEGATIVE `lacks_fact_attribute_equals` predicates (×3 + `non_typedef_package_scope`'s
  `:2694`) — trivially satisfiable on an empty store. Probing it proved the machine right and
  exposed a REAL boundary leak: `initial p::f();`, `initial p::C::f();`, and the expression
  `wire w = p::X;` ALL ACCEPT under `verilog_2005` (AST-proven route
  `subroutine_call → scoped_or_hierarchical_with_args → package_scope → tf`), though IEEE
  1364-2005 has no `::` token anywhere in Annex A. Ledgered **`SV-0031`** (`Root Caused`,
  own fix leaf — NOT bundled into this read-only slice).

**The 4 remaining machine-`genuine` drift entries probed tools-first:** `function_statement`
is legal in-profile surface (task/function body statements — a witnessable ratchet target,
verdict stands). `data_type_or_void`/`kw_void` and `dynamic_array_variable_identifier` are
machine-CORRECT (`module m; function void f; … endfunction endmodule` and `module m; reg q [];
endmodule` both ACCEPT under `verilog_2005`) — and that in-profile parseability is itself
over-acceptance: `void` is not a 1364-2005 return type (§10.4) and dynamic-array `[]` is IEEE
1800 §7.5 only. Ledgered **`SV-0032`** (void return-type slot on the v2005-admitted
`function_declaration_sv_2017`) and **`SV-0033`** (dynamic-dimension alternative on the
v2005-admitted `data_declaration`), both AST-carrier-pinned, both `Root Caused`, own fix
leaves. Net: the classification's first run corrected two manual adjudications and surfaced
three ledgerable boundary leaks — the mechanization case made empirically.

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

## `.4.1` Findings (tools-first, 2026-07-02)

- **The reserved-word lookahead is a PARTIAL curated list — gated constructs re-parse as
  identifier-built constructs.** The pre-`.4.1` `reserved_non_keyword_identifier` regex (`:360`)
  omits `initial`/`always`/`assign`/… Under `verilog_2005` (assertions gated), `initial assert (1);`
  re-parsed as a **module_instantiation** — module type `initial`, instance `assert` — proven by
  `--parse-dump-ast-pretty` (the `.4.1` leak repro). Masked under `sv_2017` only because the
  keyword-led construct wins the PEG ordered choice first. The D2 split fixes BOTH keyword axes: the
  full Annex B set is reserved under `verilog_2005` (no identifier re-parse of gated constructs) and
  the 48 SV-only words become legal identifiers.
- **Task-enable re-reading (spec-derived corpus expectations, not fix-mirrors).** With `assert`/
  `return` legally un-reserved, `initial assert (1);` and bare `return;` are **syntactically valid
  IEEE 1364-2005** — task enables of tasks named `assert`/`return`. AST-verified: the accepted parse
  is `initial_construct` → `subroutine_call` (NOT the old instantiation leak). The reject corpus
  therefore uses non-reinterpretable forms (`assert (1) else $error(…)` — task enables take no action
  block; `return x;` — task enables take no bare argument), per
  [[feedback_corpus_expected_from_spec_not_fix]].
- **`wire logic;` PEG-commit over-rejection (deferred to `.4.2`, mechanism pinned):**
  `data_type_or_implicit` COMMITS to the un-gated `logic`-as-data-type branch (cluster (c) residual);
  the completed sub-rule cannot be re-asked for its implicit alternative, so `wire logic;` (net named
  `logic`, legal 1364-2005) rejects until `.4.2` gates the `bit`/`logic` type branches under
  `verilog_2005`. `reg bit;` / `integer int;` / `wire [3:0] class;` all ACCEPT (no type-keyword
  collision on their paths).
- **Two PRE-EXISTING sv_2017 defects found by the corpus (ledgered `SV-0021`/`SV-0022`, own fix
  leaves):** (1) mixed untyped→typed ANSI ports (`module m (input a, output reg b);` rejects,
  `furthest_position=25`; typed→typed and untyped→untyped accept); (2) `bind_directive`'s trailing
  `semi` (`:621`) duplicates the `semi` already consumed by `module_instantiation` (`:3174`) — the
  spec-valid single-`;` `bind` can never parse and the invalid `;;` form ACCEPTS (mechanism confirmed
  both ways). Proven pre-existing: this slice's grammar diff is directive-only (all changed/added
  lists still include `sv_2017`, so `sv_2017` rule activation is invariant).
- **uvm triage-gate posture on this 24 GB host (NOT a slice regression):** the 4 uvm rows of
  `sv_external_corpus_triage_gate` classify as `parse_fail` via the PARSE-TERMINATION.4 resource
  guard (debug probe + `ulimit -v` at 70% of host RAM); the compat rows are a CASCADE (the capped
  `--lib-out` uvm_pkg bootstrap dies → compat's first `uvm_object_registry` reference is
  store-unresolvable). Decisive A/B on the gate's own preprocessed uvm_pkg with the release probe:
  HEAD `15.66 GB` peak RSS / `58.9 s` vs slice `15.75 GB` / `62.8 s` (**+0.6 % RSS**); the slice
  parser COMPLETED `parse_full` under both `sv_2017` and `sv_2023` uncapped, and a HEAD single-case
  run was ALSO ambient-killed — environment-dependent host posture, documented in the gate script
  itself ("uvm cases may hit this cap on a small host — that is the HONEST state").

## `.4.2` Findings (tools-first, 2026-07-02)

- **REPRODUCE baseline (HEAD binary): 14 leaks + 1 over-rejection.** 12 of the 13 planned reject
  probes ACCEPTED under `verilog_2005` while ACCEPTING under `sv_2017` (valid SV — the leak
  signature): `always_comb`/`always_latch`/`always_ff`, `do…while`, `foreach` (statement-body form,
  non-reinterpretable per [[feedback_corpus_expected_from_spec_not_fix]]), `logic x;`/`bit b;`/
  `shortreal r;` (as TYPES), `interconnect` port + declaration, `const ref` tf-port, `wait fork;`,
  `wait_order(…) else …`; plus 2 DISCOVERED leaks: `ref` task-port direction (via the shared core
  `port_direction`'s `ref` branch) and `-> #5 e;` (the mis-encoded event-trigger delay form, legal
  neither in 1364-2005 — oracle `:370` has no control — nor on `->` in IEEE 1800). `wire logic;`
  reproduced the `.4.1`-diagnosed over-rejection (REJECT under all profiles; under sv_2017 that is
  spec-correct since `logic` is reserved there).
- **The 13th planned probe (`->> e;`) REJECTED under sv_2017 too** — not a profile item at all:
  the literal `->>` is absent from the entire grammar (ledgered `SV-0023`, see the leaf entry).
- **Second-path re-parses are the dominant failure mode of branch-gating — every reject case was
  AST-dump-adjudicated, not assumed.** Two inputs stayed accepted after the planned gates and each
  exposed a distinct pre-existing hole: (1) `module m (interconnect w);` re-parsed as an ANSI
  INTERFACE port (`kind:"net_or_interface"`/`"named"` — `interface_port_header`'s
  `interface_identifier` is a bare `declaration_identifier`) → whole-rule-gated
  `interface_port_header` `["sv_2017","sv_2023"]`; (2) after THAT gate it re-parsed AGAIN via the
  non-ANSI `port_expression` un-braced list branch (`kind:"list"`, module `kind:"nonansi"`) —
  pre-existing all-profile over-acceptance (`module m (a b);` accepts at HEAD), ledgered `SV-0024`;
  corpus case re-pinned to `module m (input interconnect w);`.
- **Spec-honest accepts preserved:** `wait_order (a, b);` under `verilog_2005` remains ACCEPT — with
  `wait_order` un-reserved (D2) it is a legal 1364-2005 TASK ENABLE; the reject case uses the
  `else`-action form task enables cannot take. Same doctrine as `.4.1`'s `initial assert (1);`.
- **Shape-preservation proven byte-exact:** 24 BEFORE/AFTER `--parse-dump-ast` dumps (all reject +
  accept corpus files under `sv_2017`, interconnect cases also under `sv_2023`) — `diff -r` clean;
  the lifted interconnect PORT branch additionally proven still live under sv_2017 via
  `module m (input interconnect [3:0] w);` → `kind:"interconnect"` (the un-dimmed form is
  tournament-shadowed by the `net_type_identifier` branch — pre-existing, lint-warned, unchanged).
- **Cert accounting exact:** the +10 canonical `total`/`witness` delta equals the 10 new named
  rules, all witnessed at seeds 0/7/42 (union witness +19→1321 includes the entry-relative
  configs' credits); UNKNOWN=20 canonical / union=1 residual byte-identical. The union-gate count
  pins (1304/1/1283+1302) remain the pre-existing stale-pin drift — now 1324/2/1302+1321 — owned by
  the standing re-baseline leaf (per `.4`).

## `.4.3` Findings (tools-first, 2026-07-02)

- **The scratch corpora survived, and the fresh matrix reproduces the recorded adjudications
  EXACTLY.** Both prior sessions' scratchpads were still present under
  `/private/tmp/claude-501/…/scratchpad/`; the 46 corpus files were backed up, promoted, and the
  full 46-file × 3-profile probe matrix re-measured on the HEAD binaries: 12/12 accepts under
  `verilog_2005`; 34/34 rejects under `verilog_2005`; every reject still ACCEPTS under
  `sv_2017`/`sv_2023` except `bind_dir.sv` (the ledgered `SV-0022` all-profile waiver, exactly as
  recorded); `keywords_as_identifiers.v` + `wire_logic.v` REJECT under the SV profiles
  (spec-correct — those words are reserved in IEEE 1800). Zero surprises vs the `.4.1`/`.4.2`
  checklists — the promoted contract pins measured behavior, not aspiration.
- **The profiled cert baseline is deterministic and self-explaining.** Three independent runs
  (seeds 0/7/42) produced byte-identical headlines: `total=1138 proof=2 witness=826 UNKNOWN=310
  (spf=0, prf=0)`. The run's own diagnostics classify 277/310 as `NO reach path from the entry`
  dead-rule candidates — under a strict-subset profile that IS the design (SV-only rules gated out
  of `verilog_2005` remain in the census but are correctly unreachable), matching the `.1` D4
  prediction "SV-only rules become unreachable/proof under the profile — expected and acceptable".
  The `total` drop `1324→1138` (sv_2017 vs verilog_2005 active-universe) is the gated surface.
- **A `.2`-era lockstep gap surfaced and closed**: the SV integration contract still declared
  embedding-API baseline `1.2.0` (the `.2` slice bumped the API to `1.3.0` and updated
  `EMBEDDING_API_CONTRACT.md` + the SV book, but not this contract's identity block — the full
  contract write-up had been deferred to `.4.3`, so the stale line rode along). Corrected with the
  write-up; the correction is annotated in-place in the contract.
- **Gate-log disk hygiene**: the `focus_systemverilog` regen stage streams a ~5 GB stdout log; the
  gate now prunes any successful-or-failed stage log above 10 MB down to its last 2000 lines (the
  failure-triage tail is what matters; the repo's disk-space doctrine forbids retaining
  multi-GB scratch logs). Same posture as `ci_workflow_local_gate`'s successful-run cleanup.

## Acceptance Checklist (`.4.3`, enforced)

- [x] **REPRODUCE / ISSUE** — the `.4.3` gap at HEAD: conformance corpus existed ONLY in wipeable
  `/private/tmp` session scratchpads (46 files located + backed up tools-first); NO repo-standard
  `verilog_2005` gate target existed (`make -C rust … verilog_2005_conformance_gate` was an
  unknown target); the profiled cert baseline had never been measured; the SV integration contract
  had ZERO `verilog_2005` mentions (grep-verified) and a stale `1.2.0` embedding-API baseline.
- [x] **ROOT CAUSE (WHY + WHERE)** — not a defect leaf: the closure surfaces were EXPLICITLY
  deferred here by design (`.2` Decisions: contract write-up deferred to the closure leaf; `.4`
  plan: corpus/gate/baseline = the `.4.3` band). The one genuine drift (stale `1.2.0`) is
  pinned to the `.2` lockstep having updated `EMBEDDING_API_CONTRACT.md` + SV book but not the SV
  integration contract's identity block.
- [x] **FIX** — closure artifacts, no grammar/engine change (fix-hierarchy tier: N/A — test-data +
  gate + docs): 46-file tracked corpus + contract JSON (138-cell matrix, every cell measured, notes
  spec-derived) + gate script (lint lock / matrix / alias probes / cert pins + determinism) +
  Makefile target + contract § + LIVE promotion + book/README lockstep.
- [x] **ADDRESSED (verified)** — `make -C rust SHELL=/bin/bash verilog_2005_conformance_gate`
  **GREEN end-to-end** (gate_green=true, unmet_criteria_count=0): lint rc=0 with 0 `verilog_2005`
  orphans (headline + per-rule line count both asserted), corpus 138/138 file×profile checks with 0
  mismatches, 2/2 alias probes, cert `1138/2/826/310/spf=0` byte-identical across seeds 0/7/42
  (`rust/target/verilog_2005_conformance_gate/summary.{txt,json}`).
- [x] **NO REGRESSION** — zero grammar / Rust-source / generated / shape-manifest change in this
  leaf (docs + test-data + shell + Makefile only), so the parser oracles are INERT BY CONSTRUCTION;
  the gate itself re-ran the heavy oracles fresh on HEAD binaries: cert seeds 0/7/42 deterministic
  `spf=0`, lint rc=0 warnings unchanged (8 A2-backlog), 138-check corpus matrix green;
  `mdbook_docs_gate` + `systemverilog_parser_book_gate` GREEN after the book edits; clippy N/A
  (no Rust change).
- [x] **LOCKSTEP** — SV integration contract (full profile §, identity block, trust statement);
  LIVE dialect block (`In Progress`→`Mostly Done` + why-not-Done); main book `parser-families.md`
  + `parseability-probe-debug.md` (alias-list drift closed); SV parser book `glossary.md` (+ gate
  rebuild); README standard commands; CHANGES / DEVELOPMENT_NOTES / MEMORY; tree +
  `docs/TASK_TREE.md` frontier → `.5`.

## Acceptance Checklist (`.4.2`, enforced)

- [x] **REPRODUCE / ISSUE** — HEAD-binary probe matrix over the authored `.4.2` corpus:
  14 SV-only constructs ACCEPT under `--profile verilog_2005` (each ACCEPTS under `sv_2017` — valid
  SV, so the acceptance is a profile leak), e.g. `always_comb q = 1'b0;` v2005 rc=0 /
  `task t (ref integer a);` v2005 rc=0 / `interconnect w;` v2005 rc=0; `wire logic;` REJECTS
  (rc=1, the `.4.1` over-rejection); `->> e;` REJECTS under sv_2017 (`furthest_position=33`) →
  re-scoped to `SV-0023`.
- [x] **ROOT CAUSE (WHY + WHERE)** — un-gated SV-only alternatives inside `verilog_2005`-active
  rules, each pinned by rule-text dump + AST evidence: `always_keyword:498`,
  `loop_statement:2866` (do@2874, foreach@2876), `integer_vector_type:2513`,
  `non_integer_type:3486`, `net_port_type_sv_2017:3379`/`_sv_2023:3384` (interconnect),
  `net_declaration_sv_2017:3340`/`_sv_2023:3352` (interconnect), `port_direction:3954` (`ref`),
  `tf_port_direction_sv_2017:5186` (`const ref`), `wait_statement:5671-5673`,
  `event_trigger_sv_2017:2090` (delay form); the two second-path re-parses pinned by
  `--parse-dump-ast-pretty` (`net_or_interface/named` → `interface_port_header:2615`;
  `list`/`nonansi` → `port_expression` un-braced star, HEAD `:3958`, oracle
  `verilog_2005_lrm_extracted.ebnf:982-983`).
- [x] **FIX** — declarative grammar-only (fix-hierarchy tier: grammar; NO engine change): 10
  shape-preserving named-lifts per the `integer_atom_type_sv_only` idiom (annotations verbatim,
  umbrella alternative order preserved, gates `["sv_2017","sv_2023"]` — or `["sv_2017"]` for the
  two whose only parent is the `_sv_2017` variant) + the `interface_port_header` whole-rule gate.
- [x] **ADDRESSED (verified)** — `.4.2` corpus: REJECT **15/15** under `verilog_2005` with every
  file still ACCEPTING under `sv_2017` (leak → clean profile boundary), ACCEPT **8/8** under
  `verilog_2005` including `wire logic;` **REJECT→ACCEPT** (the `.4.1` over-rejection CLOSED);
  `.4.1` standing corpus re-verified (module_min / realistic counter / generate ×3 profiles PASS;
  keywords-as-identifiers PASS under `verilog_2005`; 19/19 rejects hold, `bind_dir` sv_2017-waiver
  per `SV-0022` unchanged); `--lint-grammar` rc=0, `verilog_2005` orphans **0→0** (non-increase
  rule held), warnings byte-identical (8).
- [x] **NO REGRESSION** — cert seeds 0/7/42 (`sv_2017`, entry `systemverilog_file`, count 40):
  `total=1324 proof=2 witness=1302 UNKNOWN=20 (spf=0, prf=0)` DETERMINISTIC, canonical 20-rule
  residual IDENTICAL (`DUMP_ALL` diff: zero newly-unknown; the `+10 total/+10 witness` are exactly
  the 10 new lifted rules, all witnessed); `sv_cert_recognized_union_gate` semantic invariants
  INTACT (canonical `UNKNOWN=20`, union `UNKNOWN=1`, residual `["context_member_method_call"]`,
  union witness `1321`, deterministic ×3 seeds; unmet criteria = the pre-existing count pins only);
  24/24 BEFORE→AFTER AST dumps byte-identical (sv_2017 + sv_2023); `ast_shape_contract` tests
  18/18; embedding_api tests 51/0; realistic corpus `239` accepts / `126` directive-file fails
  (= baseline exactly); SV external corpus non-uvm **10/10 preprocess + 10/10 parse** via
  `sv_external_corpus_triage_gate` on the filtered manifest (uvm rows excluded per the documented
  24 GB-host mem-cap posture, unchanged scope); clippy source clean (`clippy_on_rust_change`;
  generated-stage debt pre-existing, non-strict); only the SV parser regenerated (mtime census —
  the other 6 generated parsers untouched); release `1.0.158` / schema `13` unchanged (sv_2017 and
  sv_2023 language + AST proven invariant; `verilog_2005` is the in-hardening profile).
- [x] **LOCKSTEP** — bug ledger `SV-0023`/`SV-0024` rows added; main-platform book
  `parser-families.md` remaining-hardening list updated to the post-`.4.2` state; SV parser book
  updated + both book gates GREEN; CHANGES / DEVELOPMENT_NOTES / MEMORY / LIVE_ACHIEVEMENT_STATUS
  updated; tree + `docs/TASK_TREE.md` frontier → `.4.3`.

## Acceptance Checklist (`.4.1`, enforced)

- [x] **REPRODUCE / ISSUE** — `ast_pipeline grammars/systemverilog.ebnf --lint-grammar` at HEAD:
  `Error: grammar 'systemverilog' has 170 profile-orphan rule(s)`, rc=1 (the `.2`/`.3` recorded
  wellformedness regression); plus the `.3` leak observation (`package p; endpackage` PARSED under
  `--profile verilog_2005`) re-confirmed and extended (`initial assert (1);` parsed as a module
  instantiation — AST dump cited in Findings).
- [x] **ROOT CAUSE (WHY + WHERE)** — the orphan detector names each present-but-unsatisfiable rule +
  profile (`[error] … rule 'X' is present under profile 'verilog_2005' but is NOT satisfiable there`,
  ANNOTATION-COMPOSITION.4); the leak axis pinned by `--parse-dump-ast-pretty` (module_instantiation
  with identifier-matched `initial`/`assert`) + the PARTIAL reserved regex at
  `grammars/systemverilog.ebnf:360`; the over-rejection axis pinned by `furthest_position` bisection
  (`integer i;` → the 6-branch `integer_atom_type` whole-rule gate error).
- [x] **FIX** — declarative grammar-only (fix-hierarchy tier: grammar; NO engine primitive): 28
  baseline admits + 103 SV-only gates + the D2 profile-split reserved lookahead + the
  `integer_atom_type_sv_only` shape-preserving named-lift.
- [x] **ADDRESSED (verified)** — `--lint-grammar` `verilog_2005` orphans **170 → 0**, exit rc
  **1 → 0**; conformance corpus: ACCEPT 10/10 checks (module_min / realistic parameterized counter /
  generate+genvar+localparam file × 3 profiles each, + keywords-as-identifiers under `verilog_2005`)
  and REJECT **19/19** SV-only files under `verilog_2005` while ACCEPTING under `sv_2017`
  (`bind_dir` waived to v2005-REJECT-only per pre-existing `SV-0022`); aliases
  `ieee1364-2005`/`1364-2005` normalize.
- [x] **NO REGRESSION** — cert seeds 0/7/42 (`sv_2017`, entry `systemverilog_file`, count 40):
  `total=1314 proof=2 witness=1292 UNKNOWN=20 (spf=0, prf=0)` DETERMINISTIC, canonical 20-rule
  residual IDENTICAL (19 `no_path` + `context_member_method_call`), zero newly-unknown — the
  `+2 total / +1 proof / +1 witness` are exactly the 3 new named rules (`_sv` = negation-only proof;
  `_v2005` profile-excluded from the sv_2017 universe; `integer_atom_type_sv_only` witnessed);
  `sv_cert_recognized_union_gate` semantic invariants INTACT (canonical `UNKNOWN=20`, union
  `UNKNOWN=1`, residual `["context_member_method_call"]`, union witness `1311`) with the count drift
  pre-existing (pinned `1304`, now `1314` — re-baseline leaf per `.4`); `ast_shape_contract_gate`
  18/18; embedding_api tests 51/0; realistic corpus `239` accepts / `126` directive-file fails
  (= the `.2` baseline exactly); lint warnings byte-identical to baseline (8 A2-backlog);
  uvm A/B `+0.6 %` RSS (Findings); clippy source clean (`clippy_on_rust_change`); only SV
  regenerated (`focus_systemverilog`) — the other 6 generated parsers untouched on disk.
- [x] **LOCKSTEP** — main-platform book `parser-families.md` + SV book `glossary.md` updated to the
  coherence state (`mdbook_docs_gate` + `systemverilog_parser_book_gate` both GREEN, regenerated
  tracked HTML staged); bug ledger `SV-0021`/`SV-0022` rows added; CHANGES / DEVELOPMENT_NOTES /
  MEMORY / LIVE_ACHIEVEMENT_STATUS updated; tree + `docs/TASK_TREE.md` frontier → `.4.2`.

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

- 2026-07-03 (`.6.5`): **Per-profile proof accounting ADOPTED as the end state, STAGED.** The
  three profile-gated residual classes are soundly provable (P1 entry-universe unreachability
  over the active tree with satisfiability-honest edges; P2 unproducible-mandatory-positive-
  store-gate fixpoint composed with P1) and SHALL become `proof` certificates — but only after
  a read-only, env-gated machine classification (`.6.6`) reproduces the manual adjudication;
  promotion + all gate re-pins land together in `.6.7`. Soundness boundaries fixed in the
  design: the certificate quantifies over the DECLARED ENTRY UNIVERSE (never the single cert
  entry — the 295 NO-reach set is heterogeneous and contains the entry-relative library
  cohort, which is legal 1364-2005 §13 surface and must NOT be branded dead); P2 counts only
  POSITIVE fact-queries (`lacks_fact` unsatisfiability makes a rule always-live, not dead),
  kind-level only, degrades to inert in the presence of a live `@import_from_library`; the
  default reach BFS (`reach_hops_pass`) is not touched at any stage (witness-landscape
  neutrality — the `.6.3` collateral lesson). See "`.6.5` Findings".
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
- 2026-07-02 (director decision): **CONFIRMED — proceed with the `verilog_2005` build-to-coherence
  campaign** (drive `--lint-grammar` orphans 170→0, closing the `.2` regression). Because `.4` is a
  ~40+-directive delicate grammar surgery with leak risk, execution is deferred to a FRESH SESSION for
  full focus (fresh-session discipline) — NOT abandoned; the candidate classification + lint-iterate
  recipe are recorded in the `.4` leaf so the fresh session verifies rather than re-derives them.

- 2026-07-02 (`.4.1`): **prefer an existing NAMED SV-only rule over a branch-lift.** `unique_priority`
  and `case_pattern_item` exist as named rules, so whole-rule gates cover the `unique`/`priority`
  qualifiers and `case…matches` arms — two fewer branch-lifts than the `.1` D1 plan anticipated.
- 2026-07-02 (`.4.1`): **corpus expected-values derive from IEEE 1364-2005 SYNTAX, not from
  "SV-shaped input must reject".** With the D2 un-reserve landed, `initial assert (1);` and bare
  `return;` are legal 1364-2005 task enables and the profile correctly ACCEPTS them (AST-verified
  `subroutine_call`); the reject corpus uses non-reinterpretable forms. Applies
  [[feedback_corpus_expected_from_spec_not_fix]] to the profile boundary.
- 2026-07-02 (`.4.1`): **keep the D2 umbrella split as designed; the "memo-lean" double-negation
  alternative is NOT pursued.** The A/B measurement (HEAD `15.66 GB`/`58.9 s` vs slice
  `15.75 GB`/`62.8 s` on the gate's preprocessed uvm_pkg) shows the umbrella costs ~0.6 % RSS — no
  tool-backed need for a redesign ([[feedback_no_codebase_change_without_tool_backed_facts]]).
  (Generated-code fact recorded: the profile guard fires BEFORE `memoized_call`, so an off-profile
  rule call is near-free and leaves no memo row.)

- 2026-07-02 (`.4.2`): **AST-dump-adjudicate every branch-gate reject case — second-path re-parses
  are the dominant failure mode.** Gating a branch does not make an input reject; it makes the PEG
  try every OTHER path, and under `verilog_2005`'s un-reserved SV keywords those paths are
  identifier-shaped and plentiful. Both `.4.2` second-path accepts exposed real pre-existing
  defects (`interface_port_header` bare-identifier leak; `SV-0024` un-braced `port_expression`).
  A reject expectation is EARNED by a rejecting run + an accept-side AST dump naming the path,
  never by "the branch is gated now".
- 2026-07-02 (`.4.2`): **single-parent variant lifts gate to the variant's own profile list**
  (`tf_port_direction_const_ref_sv_only` / `event_trigger_control_sv_only` → `["sv_2017"]`, not
  `["sv_2017","sv_2023"]`): the sv_2023 twins carry their own copies of those branches, so a
  2-profile gate would leave the lifted rule present-but-unreachable under sv_2023. Mirrors the
  existing `weight_specification_sv_2017` convention.
- 2026-07-02 (`.4.2`): **a planned gate item can dissolve into a defect ledger row** — the
  "`event_trigger` `->>` branch" plan item had no `->>` branch to gate (the grammar mis-encodes
  both LRM alternatives on `->`). The profile slice lifts/gates only what exists
  (`event_trigger_control_sv_only`); the LRM-fidelity repair (add `->>`, move the control, fix the
  swapped `kind` labels, array-trigger selects) is `SV-0023`'s own fix leaf — one concern per
  commit, per the `.4.1` `SV-0021`/`SV-0022` precedent.

- 2026-07-02 (`.4.3`): **LIVE promotion = `Mostly Done`, deliberately NOT `Done`.** Two
  tracker-rule-grounded reasons: (1) the proof surface is a curated corpus, and the tracker forbids
  `Done` on curated/manual construct lists where grammar-derived exhaustiveness is expected (the
  grammar-derived surface here would be profiled-cert witness parity — leaf `.6`); (2) `SV-0024` is
  a KNOWN plausible leak in the strict-subset claim (bare `module m (interconnect w);` accepts under
  `verilog_2005` at HEAD). Signoff honesty over promotion optics.
- 2026-07-02 (`.4.3`): **contract expectations pin MEASURED behavior with ledgered waivers named
  in-row** — every non-obvious matrix cell carries its spec-derivation or defect-waiver note
  (`SV-0022` bind reject-everywhere, `SV-0023` sv-side delay-form accepts, `SV-0024` ANSI-form pin,
  keyword-reservation divergences). When a ledgered defect lands, the gate FAILS by design and the
  contract row is re-adjudicated in the fixing commit — the waiver can never silently outlive its
  defect.
- 2026-07-02 (`.4.3`): **bulky gate stage-logs are pruned, not retained** — any stage log over
  10 MB keeps only its last 2000 lines (success AND failure; the triage tail is what the gate's own
  failure path prints). The `focus_systemverilog` stage otherwise retains ~5 GB per run, violating
  the disk-space doctrine.

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

- 2026-07-03 (`.6.6` CODE, `PGEN-VERILOG-2005-PROFILE-0019`, session #26): BEFORE captures on
  HEAD binaries reproduced the pins byte-identical (v2005 seed-0 `DUMP_ALL`
  `1147/4/816/UNKNOWN=327 spf=0` + 295 NO-reach; canonical+union seed-0 `1341/2/1319/20`,
  union `1338/1`, residual `context_member_method_call`). AFTER (env unset): both outputs
  **diff-proven byte-identical** pre↔post; v2005 headlines reproduced at seeds 7/42.
  Unit tests: `cargo test --lib ast_pipeline::grammar_wellformedness` → 39 passed / 0 failed
  (6 new: P1 no-self-root + second-entry rescue, satisfiability-honest edges + lookahead,
  gate/cascade/escape/negative, live-emitter + import-degradation, Or-intersection +
  literal-count, store-fixpoint stranding). Classification (env set, universe = canonical +
  `sv_multi_entry_root`/`library_text`/`systemverilog_parseable_file`): `292/17/18 = 327`,
  block md5 `d3d0914028df9c1bdfa8de6d589053ec` identical at seeds 0/7/42. Discrepancy
  adjudication probes: `initial p::f();` / `initial p::C::f();` / `wire w = p::X;` ACCEPT
  under `verilog_2005` (AST dump route `subroutine_call → scoped_or_hierarchical_with_args →
  package_scope → tf`) → `SV-0031`; `module m; function void f; …` ACCEPT (route
  `… → function_declaration` sv_2017 variant, universal `data_type_or_void` slot) → `SV-0032`;
  `module m; reg q [];` ACCEPT (route `data_declaration → data_type` with the universal
  dynamic-dimension alternative) → `SV-0033`; grammar reads pinned the negative-gate escape
  (`systemverilog.ebnf:6630/:6641/:2694`) and the wave-3 `anchored` alternative (`:2403`).
  Oracles re-run fresh: `sv_cert_recognized_union_gate` ✅ (canonical `UNKNOWN=20`, union
  `UNKNOWN=1`, deterministic seeds 0/7/42); `verilog_2005_conformance_gate` ✅ (lint 0-orphan
  lock + 192-check matrix + 3-seed cert pins); fully-certified six byte-identical (json
  `9/0/9/0`, regex `198/0/198/0`, svpp `74/0/74/0`, vhdl `216/0/216/0`, rtl_frontend
  `169/1/168/0`, `rtl_const_expr_cert_gate` ✅); `mdbook_docs_gate` ✅; km-check OK;
  `clippy_on_rust_change` ✅ (source strict-clean).
- 2026-07-03 (`.6.5` DESIGN, `PGEN-VERILOG-2005-PROFILE-0018`): fresh evidence run on HEAD
  binaries (SV `1.0.161`/schema 15): `PGEN_CERT_COVERAGE_DUMP_ALL=1 ./rust/target/debug/
  ast_pipeline grammars/systemverilog.ebnf --report-certificate-coverage --grammar-profile
  verilog_2005 --entry-rule systemverilog_file --count 40 --seed 0` → headline byte-identical
  to the contract pins (`total=1147 proof=4 witness=816 UNKNOWN=327`, `spf=0`, `prf=0`); 295
  NO-reach rules enumerated (heterogeneity confirmed: contains `library_text`/
  `sv_multi_entry_root`/`systemverilog_parseable_file`/`library_declaration`/
  `include_statement` — the entry-relative cohort — alongside the stranded SV-only surface);
  the 32-rule attempted-but-unwitnessed residual extracted by set-diff and classified (18 A +
  7 B + 1 E + 6 post-`.6.1` drift). Engine mechanisms pinned by direct code read (file:line
  in the findings): `main.rs:2237`/`2497`/`2926`, `grammar_wellformedness.rs:268`/`284`/
  `306-313`/`671`/`778`/`958`/`1723`, `stimuli_generator.rs:6800`/`6869`/`7120`/`7161-7164`,
  `grammars/systemverilog.ebnf:1018-5481` (profile-agnostic `type_name` emitters). Contract
  prose fix verified: `jq` parse clean, the four cert pins unchanged (`1147/4/816/327`), and
  `make -C rust SHELL=/bin/bash verilog_2005_conformance_gate` re-run GREEN end-to-end after
  the edit (lint 0-orphan lock + 192-check matrix + 3-seed cert pins).
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

- 2026-07-02 (`.4.1`, CODE — full verification): see "Acceptance Checklist (`.4.1`)" for the earned
  boxes. Headlines: `--lint-grammar` `verilog_2005` orphans 170→0 / rc 1→0 across 4 adjudicated edit
  batches (each batch oracle-checked against `verilog_2005_lrm_extracted.ebnf`; warnings byte-identical
  throughout); conformance corpus ACCEPT 10/10 checks + REJECT 19/19; cert seeds 0/7/42
  `UNKNOWN=20`/`spf=0` deterministic with the identical canonical residual and the 3 new rules
  accounted (`+2 total/+1 proof/+1 witness`); union-gate semantic invariants intact (counts drift
  pre-existing → re-baseline leaf); `ast_shape_contract_gate` 18/18; embedding_api 51/0; realistic
  corpus 239/126 (= `.2` baseline); uvm A/B `+0.6 %` RSS (HEAD↔slice, gate-preprocessed uvm_pkg,
  release probe); both book gates GREEN; clippy source clean. Two pre-existing sv_2017 defects
  discovered + ledgered (`SV-0021` mixed untyped→typed ANSI ports; `SV-0022` bind double-`semi`).

- 2026-07-02 (`.4.2`, CODE — full verification): see "Acceptance Checklist (`.4.2`)" for the earned
  boxes. Headlines: `.4.2` corpus REJECT 15/15 under `verilog_2005` (each still ACCEPT under
  sv_2017) + ACCEPT 8/8 (incl. `wire logic;` REJECT→ACCEPT); `.4.1` standing corpus holds; lint
  rc=0 / 0 orphans / warnings byte-identical; cert seeds 0/7/42 `total=1324 proof=2 witness=1302
  UNKNOWN=20 spf=0` deterministic, identical canonical residual, +10 = the 10 new rules all
  witnessed; union-gate semantic invariants intact (union `UNKNOWN=1`, witness `1321`; count pins
  pre-existing-stale → re-baseline leaf); 24/24 AST before→after byte-identical; shape-contract
  18/18; embedding 51/0; realistic 239/126; external non-uvm 10/10+10/10; clippy source clean;
  SV-only regen; release/schema unchanged (`1.0.158`/13). Two pre-existing defects ledgered
  (`SV-0023` event-trigger complex; `SV-0024` un-braced `port_expression`).

- 2026-07-02 (`.4.3`, CLOSURE — full verification): see "Acceptance Checklist (`.4.3`)" for the
  earned boxes. Headlines: 46-file corpus matrix re-measured on HEAD binaries BEFORE pinning
  (12 accept + 34 reject under `verilog_2005`; per-profile divergences exactly as adjudicated in
  `.4.1`/`.4.2`); profiled cert `--grammar-profile verilog_2005` measured 3× (seeds 0/7/42):
  `total=1138 proof=2 witness=826 UNKNOWN=310 spf=0 prf=0` byte-identical (277/310 self-classified
  NO-reach-path under the profile); `verilog_2005_conformance_gate` GREEN end-to-end
  (lint lock + 138/138 matrix + 2/2 aliases + cert pins + determinism);
  `mdbook_docs_gate` + `systemverilog_parser_book_gate` GREEN post-lockstep. No grammar / Rust /
  generated / shape-manifest change (docs + test-data + shell + Makefile only).

- 2026-07-02 (`.5`, RE-BASELINE — full verification): see "Acceptance Checklist (`.5`)". Headlines:
  4 count pins updated (`1304→1324`, `1→2`, `1283→1302`, `1302→1321`; semantic pins untouched);
  `sv_cert_recognized_union_gate` re-run fresh end-to-end → GREEN (`recognized_basis_green: true`,
  `unmet_criteria_count: 0`, seeds 0/7/42 byte-identical, `spf=0`); book + contract stale-number
  mentions re-baselined with provenance notes; `mdbook_docs_gate` GREEN.

- 2026-07-02 (`.6.1`, INVESTIGATION — adjudication verification, ZERO code change): Step 0
  `PGEN_CERT_COVERAGE_DUMP_ALL=1` seed-0 reproduced the contract pins byte-for-byte
  (`total=1138 proof=2 witness=826 UNKNOWN=310 spf=0 prf=0`; 277 NO-reach-path named); Step 1
  `PGEN_CERT_COVERAGE_DEBUG_PROBES=1` gave per-rule verdicts for all 33 genuine-gap candidates
  (18 `parsed=false`, 15 `parsed=true witnessed_target=false`); Step 2 `PGEN_REACH_PATH_DUMP=1`
  named every reach chain, and the scoped `--trace-rules known_unscoped_data_type_identifier`
  debug trace produced the class-A predicate rejection
  (`🚫 … rejected by post predicate 'has_fact [type_name, "\foo"]'`). Leak adjudication by
  parse-probe matrix under `verilog_2005` vs `sv_2017`: `wire #1step w;` ACCEPT/ACCEPT (leak →
  `SV-0025`), `wire #10ns w;` ACCEPT/ACCEPT (same row), top-level `wire w;` + `reg r;`
  ACCEPT/ACCEPT (leak → `SV-0026`, AST names `description → package_item`); benign probes:
  `case…inside`, `case…matches`, `localparam type`, `randomize(a) with {…}` all REJECT under
  `verilog_2005`; in-profile witnessability probes: `wire #foo w;`, `top.f(1);`,
  `x = randomize(a);` all ACCEPT under `verilog_2005` (positive control `wire #10 w;` ACCEPT).

- 2026-07-02 (`.6.2`, CODE — full verification): see "Acceptance Checklist (`.6.2`)" for the
  earned boxes. Headlines: 4 leak probes ACCEPT→REJECT under `verilog_2005` / ACCEPT unchanged
  under `sv_2017`+`sv_2023`; 5 in-profile controls unchanged (`#10`, `#foo`, hierarchical tf
  call, plain `randomize(a)` call, string literal); 12/12 AST byte-compares; lint
  `profile_orphans=0` rc=0 census 1446→1448; `verilog_2005_conformance_gate` GREEN
  (150/150 matrix, cert `1138/2/817/319` seeds 0/7/42 `spf=0`, the 9 dropped witnesses
  set-diff-proven = `time_literal`/`time_unit`/`unbased_unsized_literal`/6 time-unit kw tokens,
  NO-reach 277→287); `sv_cert_recognized_union_gate` GREEN (`1326/2/1304/20` canonical,
  `1323/1` union, residual unchanged); `ast_shape_contract` 18/18; `embedding_api` 51/0;
  realistic direct-parse 478/478 (252 directive-carrying files classified
  preprocessing-lane-only, 0 directive-free failures); non-uvm external triage green; clippy
  source clean; both gates' >10 MB stage logs pruned (disk doctrine — the union gate's ~5 GB
  regen log pruned manually again; the `prune_log` port to that script remains the
  `GRAMMAR-WELLFORMED.H.12.8.5.2` follow-up).

- 2026-07-02 (`.6.3`, CHECKPOINT — build/verify/revert record): the fix built and parse-verified
  (4/4 top-level ACCEPT→REJECT under `verilog_2005`; `sv_2017`/`sv_2023` ACCEPT unchanged; 12/12
  AST byte-compares; lint `profile_orphans=0` census 1448→1450; matrix 162/162; `verilog_2005`
  cert `1138/2/809/327` seeds 0/7/42); BLOCKED on the deterministic canonical collateral
  (`UNKNOWN` 20→21 all seeds; residual diff = `known_unscoped_property_identifier`; also absent
  from the sv_2023-entry config ⇒ union 1→2; DEBUG_PROBES 48/48 sequence-scaffold mis-render;
  REACH_PATH names the bind→checker→property_actual_arg use route; grammar census: sole
  `property_name` producer = `declared_property_identifier`). REVERTED and restoration verified
  fresh: canonical `1326/2/1304/20` (seed 0), `verilog_2005` `1138/2/817/319` (seed 0), matrix
  150/150, `.6.2` locks REJECT, SV-0026 top-level probes back to known-open ACCEPT, reverted
  rules absent from the regenerated parser (grep 0).

- 2026-07-02 (`.6.3.1`, INVESTIGATION — engine WHY+WHERE, ZERO code): baseline reproduced
  byte-identical first (canonical `1326/2/1304/20 spf=0 prf=0`, seed 0, HEAD binaries;
  `known_unscoped_property_identifier` witnessed at HEAD). Decisive evidence from ONE scoped
  debug-trace cert run (`PGEN_TRACE_VERBOSITY=debug PGEN_CERT_COVERAGE_DEBUG_PROBES=1
  PGEN_REACH_PATH_DUMP=1 … --report-certificate-coverage --grammar-profile sv_2017
  --entry-rule systemverilog_file --count 40 --seed 0`, stderr+stdout piped through a scoped
  grep — no multi-GB artifact): (1) prelude spec line proves CORRECT arming
  (`gated_rule='known_unscoped_property_identifier' kind='property_name'
  producer='declared_property_identifier' clean_path=true site=('source_text','root')`);
  (2) injected render: `Reach-plan branch forcing … forced_local=0` →
  `OR branch failed: rule='assertion_item_declaration' … branch=0 reason=Stimuli generation
  depth exceeded max_depth=64 while expanding rule 'number'` → `Selected OR branch: … branch=1`
  (the sequence fallback) — the tool-named WHY; (3) replay line
  `C2 semantic-prelude replay: … render='\foo_0 '` + the witness sample
  `"sequence\foo ;…endsequence property\foo_0 ;\foo_0 endproperty"` prove the HEAD witness is
  earned by the MAIN-chain self-emitting host, not the prelude; (4) control: the direct
  `declared_property_identifier` probe has NO prelude armed (0 spec lines), fails tier-1 4× with
  the SAME depth error, and IS rescued by tier-2 (`Selected OR branch: … branch=0`,
  `property\foo ;2737.2682endproperty`, witnessed) — the budget blind spot is specific to the
  PRELUDE sub-path. Arming census for fix blast-radius: 99 `name-prelude spec` lines across 10
  distinct gated rules in the canonical run. No grammar / code / generated / release change.

- 2026-07-02 (`.6.3` REPLAY, CODE — full verification): recorded diffs applied byte-verbatim
  (idiom cross-checked against `primary_literal_sv_only`); lint rc 0 / `profile_orphans=0` /
  census 1450; regen + BOTH binaries rebuilt fresh-mtime (stale-binary trap avoided); generated
  parser carries both rules (grep 49); 12/12 parse probes flip exactly (4 forms × 3 profiles);
  canonical cert seeds 0/7/42 all `1328/2/1306/20 spf=0` with residual md5 `9fe92e99…` =
  pre-lift baseline md5 (set-identical — the decisive former-blocker check); the witnessing
  sample for `known_unscoped_property_identifier` is
  `program p(input logic a);property\foo ;…endproperty assert property(\foo );endprogram`
  (carrier-diversification pass; the prelude declares a PROPERTY — the `.6.3.2` fix in action);
  `verilog_2005` cert seeds 0/7/42 all `1138/2/809/327 spf=0` (matches the `-0013` pre-revert
  record byte-for-byte); union gate GREEN fresh after re-pin (measured union witness 1325 =
  predicted +2); conformance gate GREEN fresh after re-pin (162 checks / 0 mismatches / 2
  aliases / cert deterministic); `ast_shape_contract_gate` 18/18; `mdbook_docs_gate` ✅ (after
  the book edits); `clippy_on_rust_change` ✅ (source strict-clean; generated-stage debt
  pre-existing, all in `generated/systemverilog_parser.rs`); union-gate 4.9 GB regen log pruned
  post-pass.

- 2026-07-02 (`.6.3.2`, CODE — engine fix, full verification): symptom flip MEASURED — the
  `known_unscoped_property_identifier` probe sample now renders a PROPERTY prelude
  (`property\foo ;215.8_40endproperty property\foo_0 ;\foo_0 endproperty`); canonical cert
  byte-identical `1326/2/1304/20 spf=0` at seeds 0/7/42 (headline + full residual list + NO-reach
  list md5-compared vs the pre-fix baseline); plannable-pass transparency moved honestly
  (1054→1055 witnessed; probe-parse-failures 68→60; target-own pass 25→24 targeted / 3→2
  witnessed — same final set). Gates GREEN fresh: `sv_cert_recognized_union_gate`
  (`unmet_criteria_count: 0`), `verilog_2005_conformance_gate` (`gate_green: true`),
  `ast_shape_contract_gate` (18/18), `mdbook_docs_gate`, `clippy_on_rust_change` (source
  strict-clean; generated-stage errors pre-existing, all in `generated/systemverilog_parser.rs`).
  Fully-certified sweep seed 0: 5/6 UNKNOWN=0 green; rtl_const_expr canonical cert
  (`--entry-rule conditional_expr --max-depth 32`) FAILS at HEAD with the deterministic
  step-budget timeout — git-stash A/B control run on the pre-change binary reproduced the
  IDENTICAL error, proving it pre-existing (intervening drift since `CERT-GEN-BUDGET.2`'s
  2026-06-25 A/B) → `CERT-GEN-BUDGET.3` spawned; not caused by this leaf. Union-gate 4.9 GB
  regen log pruned post-pass (the `H.12.8.5.2` `prune_log` port remains open).

- 2026-07-02 (`.6.4`, INVESTIGATION/ADJUDICATION — ZERO code): baseline reproduced
  byte-identical first (`1138/2/809/327 spf=0 prf=0`, seed 0, HEAD binaries, full
  `PGEN_CERT_COVERAGE_DUMP_ALL=1 PGEN_CERT_COVERAGE_DEBUG_PROBES=1` run). Trio verdicts: NOT in
  the 327-list; `[carrier-div-probe] … parsed=true witnessed_target=true` through the
  `module m(a,b);` carrier for all three (every earlier pass still `parsed=false` on the
  `module m(input logic a);` `@sample` scaffold, `systemverilog.ebnf:3137`). `identifier_list`:
  grep = sole reference inside `randomize_call`'s with-clause (`:4544`) with MANDATORY gated
  tail `constraint_block` (`:1441`, `@profiles ["sv_2017","sv_2023"]`) → class B; forced samples
  degenerate to `module m; endmodule` (`parsed=true witnessed_target=false`).
  `hierarchical_tf_identifier`: rule text `:2372` mandates `kw_sv_dollar_root … dot`; AST-dump
  control `$root.top.f(1);` under `sv_2017` → `kind:"system_tf"` only (mis-routed; zero
  hierarchical/root kinds). `scalar_constant`: probes `e == 1'b0` ACCEPT-via-expression /
  `e == 1'b` REJECT prove the eq-branches PEG-shadowed + the digit loss vs
  `systemverilog_2017_lrm_extracted.ebnf:951`. `SV-0029` family probes (all
  `parseability_probe`): LRM `$setup(…)` REJECT under `sv_2017` (`furthest_position=26`) AND
  under `verilog_2005`; mangled `sv_dollar_setup(…)` ACCEPT; `$unit::y` REJECT /
  `sv_dollar_unit::y` ACCEPT; module-level `$fatal;` REJECT / `sv_dollar_fatal;` ACCEPT.
  Provenance: zero `sv_dollar` in both extracted LRM snapshots; mangling site
  `tools/extract_systemverilog_lrm_profiles.py:315`; zero `sv_dollar` hits in `rust/src/`
  (no pre-parse rewrite). No grammar / code / generated / release change.

## Commit Log

- 2026-07-03 (`.6.6` CODE, `PGEN-VERILOG-2005-PROFILE-0019`): the READ-ONLY machine
  residual-classification LANDED (P1 `profile_entry_positively_live` + P2
  `classify_profile_residual` in `grammar_wellformedness.rs`, 6 unit tests; cert-report print
  behind `PGEN_CERT_RESIDUAL_CLASSIFICATION=1` in `main.rs`; `reach_hops_pass` untouched;
  default output diff-proven byte-identical; all pinned gates GREEN fresh). On `verilog_2005`
  the machine split `292/17/18 = 327` reconciles the manual adjudication exactly and CORRECTED
  it twice tools-first (`hierarchical_tf_identifier` stale note; `class_scoped_tf_call`
  negative-gated escape), surfacing THREE new ledgered boundary leaks `SV-0031` (`::` surface)
  / `SV-0032` (`void` return type) / `SV-0033` (dynamic-array `[]`). TOOLBOX §4.6 + book + KM
  + ledger + LIVE lockstep. Frontier → `.6.7` (promotion + re-pins; gate-side entry-universe
  wiring is its first decision), alternates = `SV-0021`..`SV-0024`/`SV-0028`/
  `SV-0031`..`SV-0033` fix leaves.

- 2026-07-03 (`.6.5` DESIGN, `PGEN-VERILOG-2005-PROFILE-0018`): per-profile proof accounting
  DESIGNED + DECIDED (staged adoption: `.6.6` read-only machine classification →
  `.6.7` certificate promotion + full re-pin); all three residual-class mechanisms tool-pinned
  to file:line (the stranded-rule secondary-root heuristic, the profile-agnostic fact-emitter
  helpers, the reach-BFS missing-mandatory skip); the 295 NO-reach heterogeneity finding
  (entry-relative cohort vs stranded SV-only surface) fixed the certificate's quantification
  (declared entry universe). Conformance-contract `baseline_note` prose corrected (stale
  "of the 326"→"of the 327", duplicated `$unit` phrase removed, the missing
  `SV-DOLLAR-LRM-FIDELITY.4` re-pin hop appended) — pins untouched, gate GREEN. ZERO
  engine/grammar code. Frontier → `.6.6` (classification CODE leaf), alternates =
  `SV-0021`..`SV-0024`/`SV-0028` fix leaves.

- 2026-07-02 (`.6.4` INVESTIGATION, `PGEN-VERILOG-2005-PROFILE-0017`): the class-D witness
  ratchet ADJUDICATED — no earnable `verilog_2005` UNKNOWN delta (ZERO code): the identifier
  trio is already witnessed inside the `809/327` pins (rescued by the `.6.3.2`
  carrier-diversification through the in-profile `module m(a,b);` carrier); `identifier_list`
  reclassified D→B (gated mandatory `constraint_block` tail); `hierarchical_tf_identifier` is
  `$root`-mandatory SV-only surface (the `.6.1` `top.f(1)` probe never exercised it);
  `scalar_constant` blocked on the new `SV-0030` (digit loss + PEG shadowing). MAJOR side
  discovery ledgered `SV-0029`: 19 `kw_sv_dollar_*` tokens match mangled `sv_dollar_*` literals
  instead of the LRM `$*` spellings (LRM `$setup`/`$unit::`/module-level-`$fatal` REJECT;
  nonsense spellings ACCEPT; `$root.` mis-routed) → new fix tree `SV-DOLLAR-LRM-FIDELITY`.
  Frontier → `SV-DOLLAR-LRM-FIDELITY.1` (design) then back here (`.6.5` remains proposed).

- 2026-07-02 (`.6.3` CODE — the replay, `PGEN-VERILOG-2005-PROFILE-0016`): `SV-0026` CLOSED
  (`Released`) — the recorded two-lift diffs (`description_unit_item_sv_only`,
  `source_text_item_unit_sv_only`) landed cleanly on the `.6.3.2`-fixed engine: 12/12 probes,
  lint 0 orphans (census 1450), canonical cert `1328/2/1306/20` with the residual set
  md5-identical to pre-lift at seeds 0/7/42 (the former collateral GONE), union gate re-pinned +
  GREEN (`1325/1`), conformance gate re-pinned + GREEN (162/0; cert `809/327`), shape 18/18,
  mdbook, clippy. Ledger/contract/book/LIVE lockstep. Release/schema unchanged. Frontier →
  `.6.4`.

- 2026-07-02 (`.6.3.2` CODE, `PGEN-VERILOG-2005-PROFILE-0015`): armed name-prelude fact-kind
  INTEGRITY + depth-fresh retry landed in `stimuli_generator.rs` (`ReachPrelude.sub_hops`,
  hops-generic `max_offpath_mandatory_sibling_depth_along_hops`, integrity check via
  `store_name_for_gate` + ONE retry sized by the sub-path's deepest mandatory off-path
  sibling, loud failure otherwise). Property-gate witness now robust (PROPERTY prelude);
  canonical/union/verilog_2005/shape/clippy/mdbook all green with pins byte-identical.
  Book lockstep: new prelude-integrity increment in `grammar-wellformedness.md`.
  Side discovery spun off: `CERT-GEN-BUDGET.3` (rtl_const_expr canonical-cert budget timeout
  at HEAD, A/B-proven pre-existing). `.6.3` UNBLOCKED → frontier (replay the recorded lifts).

- 2026-07-02 (`.6.3.1` INVESTIGATION, `PGEN-VERILOG-2005-PROFILE-0014`): the engine
  witness-routing anomaly fully tool-named (ZERO code) — the armed name-prelude's injected
  construct is invisible to both per-target depth-budget tiers, so the forced
  `property_declaration` branch dies (`depth exceeded max_depth=64 … rule 'number'`) and the
  forced-first-with-fallback Or + the integrity-blind injection loop silently degrade the
  prelude to a `sequence_declaration` (wrong fact kind); HEAD witnesses only via the main
  chain's self-emitting host, which the `.6.3` lifts route away from. Control probe (producer
  direct, no prelude) fails tier-1 identically and IS rescued by tier-2 — the blind spot is
  the PRELUDE sub-path. Fix leaf `.6.3.2` spawned (integrity check + depth-fresh retry at the
  injection site, general/parser-agnostic). Frontier → `.6.3.2`.

- 2026-07-02 (`.6.3` CHECKPOINT, `PGEN-VERILOG-2005-PROFILE-0013`): SV-0026 second carrier found
  (`source_text_item` direct top-level `localparam`/`parameter` branches) + `SV-0028` ledgered
  (bare top-level `;` accepts under EVERY profile — no LRM counterpart in either standard); the
  two-lift fix built, parse-verified, then REVERTED — landing it deterministically breaks the
  recognized-union invariant (canonical `UNKNOWN` 20→21; `known_unscoped_property_identifier`
  witness lost to an engine-side name-prelude mis-render, 48/48 sequence scaffolds for a
  property_name gate with a SOLE property producer). Blocking leaf `.6.3.1` spawned (engine
  WHY+WHERE); lift diffs recorded verbatim for replay. ZERO code change in the commit
  (ledger + tree + continuity docs only). Frontier → `.6.3.1`.

- 2026-07-02 (`.6.2`, CODE, `PGEN-VERILOG-2005-PROFILE-0012`): SV-only literal/delay leak
  surface CLOSED under `verilog_2005` — `SV-0025` + `SV-0027` fixed via two shape-preserving
  gated lifts (`delay_value_sv_only`, `primary_literal_sv_only`); 4 corpus reject-locks
  (matrix 138→150); cert pins re-baselined `826/310`→`817/319` (false-witness drop, honest
  direction) + union pins `1324/2/1302/1321`→`1326/2/1304/1323` (+2 accounted rules); both
  gates GREEN fresh; ledger/book/contract/LIVE lockstep; release/schema unchanged
  (`1.0.158`/13). Frontier → `.6.3` (`SV-0026` `$unit` gate).

- 2026-07-02 (`.6.1`, INVESTIGATION, `PGEN-VERILOG-2005-PROFILE-0011`): 310-UNKNOWN profiled
  residual fully adjudicated tools-first into 5 mechanism classes (17 store-gated SV-only
  use-sites + 8 spurious-reach + 2 confirmed LEAKS ledgered `SV-0025`/`SV-0026` + 6 in-profile
  ratchet targets + 1 canonical residual); fix leaves `.6.2`–`.6.5` spawned; ledger + book +
  SV-contract + LIVE lockstep. ZERO code change. Frontier → `.6.2` (SV-0025 fix).

- 2026-07-02 (`.5`, RE-BASELINE, `PGEN-VERILOG-2005-PROFILE-0010`): `sv_cert_recognized_union_gate`
  count pins re-baselined `1304/1/1283/1302` → `1324/2/1302/1321` (counts-only stale-pin gap from
  the `SV-0014`→`SV-0020` + `.4.1`/`.4.2` named-lift campaigns; semantic invariants unchanged);
  gate RED→GREEN; book/contract lockstep. Frontier → `.6` (proposed profiled-cert ratchet) or the
  `SV-0021`..`SV-0024` fix leaves via PNT.

- 2026-07-02 (`.4.3`, CLOSURE, `PGEN-VERILOG-2005-PROFILE-0009`): corpus promoted (46 tracked
  files), repo-standard `verilog_2005_conformance_gate` landed (contract JSON + script + Makefile
  target; lint lock + 138-check matrix + alias probes + profiled cert baseline pins, deterministic
  seeds 0/7/42), downstream SV-contract full `verilog_2005` write-up (+ stale `1.2.0`→`1.3.0`
  embedding-baseline fix), LIVE `In Progress`→`Mostly Done`, book/README lockstep. Frontier →
  `.5` (union-gate count re-baseline).

- 2026-07-02 (`.4.2`, CODE, `PGEN-VERILOG-2005-PROFILE-0008`): cluster (c) branch-lifts — 10
  shape-preserving `_sv_only` named-lifts (always/loop/vector-type/non-integer-type/interconnect
  port+decl/ref-direction/const-ref/wait/event-trigger-control) + the `interface_port_header`
  whole-rule leak gate; `wire logic;` over-rejection closed; SV-0023/SV-0024 discovered + ledgered.
  Frontier → `.4.3` (corpus promotion + `verilog_2005` gate + contract write-up + LIVE decision).

- 2026-07-02 (`.4.1`, CODE, `PGEN-VERILOG-2005-PROFILE-0007`): built the `verilog_2005` profile to
  wellformedness coherence — 28 baseline admits + 103 SV-only gates + the D2 profile-split reserved
  lookahead (full Annex B under `verilog_2005`; the 48 SV-only words un-reserved) + the
  `integer_atom_type_sv_only` named-lift (fixing the `.1` six-branch table error). Orphans 170→0,
  lint rc 1→0 (closes the `.2` regression). Conformance corpus accept/reject proven both ways;
  no-regression suite green (cert/shape/embedding/realistic/clippy; uvm A/B ≈ equal). Frontier →
  `.4.2` (branch-lifts).

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

- 2026-07-03: `.6.6` CODE leaf done (`PGEN-VERILOG-2005-PROFILE-0019`) — read-only machine
  residual-classification landed (env-gated, byte-identical default, all gates green); the
  classification's first run corrected the manual adjudication twice and surfaced the three
  new ledgered `verilog_2005` leaks `SV-0031`/`SV-0032`/`SV-0033`. `.6.7` UNBLOCKED;
  frontier → `.6.7`.
- 2026-07-03: `.6.5` DESIGN leaf done (`PGEN-VERILOG-2005-PROFILE-0018`) — per-profile proof
  accounting designed and decided (staged: `.6.6` read-only classification → `.6.7` certificate
  promotion + re-pins); conformance-contract `baseline_note` prose corrected (pins untouched,
  gate GREEN). New proposed leaves `.6.6`/`.6.7`; frontier → `.6.6`.
- 2026-07-02: `.6.3` BLOCKED-CHECKPOINT (`PGEN-VERILOG-2005-PROFILE-0013`, docs-only commit) —
  the `SV-0026` fix was built and parse-verified but deliberately NOT landed: it deterministically
  loses the `known_unscoped_property_identifier` witness in both SV entry profiles (canonical
  cert `UNKNOWN` 20→21 at seeds 0/7/42 → the pinned recognized-union invariant would break).
  Signoff decision = STOP + checkpoint: grammar/corpus/contract reverted to the `.6.2` state
  (restoration measured fresh), the lift diffs + full evidence recorded in `.6.3` Findings for
  verbatim replay, blocking leaf `.6.3.1` spawned (engine-side WHY+WHERE: the store-aware
  name-prelude renders a SEQUENCE scaffold for a `property_name` gate whose sole producer is
  `declared_property_identifier` — 48/48; also reproduces when targeting the producer itself at
  HEAD, so it is a pre-existing engine anomaly the reshape merely re-rolled). Second `SV-0026`
  carrier documented (`source_text_item` direct top-level `localparam`/`parameter` branches) +
  NEW open all-profile row `SV-0028` (bare top-level `;` accepted; no LRM counterpart in either
  standard). Frontier → `.6.3.1`. SV family status UNCHANGED (`Mostly Done`).

- 2026-07-02: `.6.2` DONE (`PGEN-VERILOG-2005-PROFILE-0012`, CODE — grammar-only) — the SV-only
  literal/delay leak surface under `verilog_2005` is CLOSED: `SV-0025` (`wire #1step w;` /
  `wire #10ns w;`) and the same-mechanism `SV-0027` found while pinning the fix locus
  (`assign w = 10ns;` / `assign w = '0;`) now REJECT under the strict profile via two
  shape-preserving gated lifts (`delay_value_sv_only`, `primary_literal_sv_only` — the `.4.2`
  idiom, PEG order preserved, SV-profile ASTs 12/12 byte-identical). 4 corpus reject-locks; cert
  pins re-baselined `826/310`→`817/319` (the 9 leak-earned FALSE witnesses dropped — honest
  direction for a leak fix); union pins re-baselined for the +2 lifted rules
  (`1326/2/1304/1323`, UNKNOWN invariants byte-identical); both gates GREEN fresh; ledger rows
  `SV-0025`/`SV-0027` → `Released`. Release/schema unchanged (`1.0.158`/13). Frontier → `.6.3`
  (the `SV-0026` `$unit` gate — the last OPEN `verilog_2005`-only leak alongside all-profile
  `SV-0024`). SV family status UNCHANGED (`Mostly Done`).

- 2026-07-02: `.6.1` DONE (`PGEN-VERILOG-2005-PROFILE-0011`, INVESTIGATION — adjudication +
  ledger/docs only, ZERO code change) — the `verilog_2005` profiled-cert 310-UNKNOWN residual is
  fully adjudicated tools-first (3-step protocol + reach-path dumps + parse/AST probes + a scoped
  predicate trace): 33 genuine-gap candidates classified into 5 mechanism classes (see `.6.1`
  Findings). TWO strict-subset over-acceptance LEAKS confirmed and ledgered — `SV-0025`
  (`delay_value` `1step`/`time_literal` branches un-gated: `wire #1step w;` / `wire #10ns w;`
  ACCEPT under `verilog_2005`) and `SV-0026` (`description → package_item` `$unit` surface
  active: top-level `wire w;` / `reg r;` ACCEPT). Fix/ratchet/design leaves `.6.2`–`.6.5`
  spawned; frontier → `.6.2`. SV family status UNCHANGED (`Mostly Done`); LIVE dialect block
  left-to-close extended with the two new ledger rows.

- 2026-07-02: `.5` DONE (`PGEN-VERILOG-2005-PROFILE-0010`, RE-BASELINE — contract-pin +
  docs only) — the `sv_cert_recognized_union_gate` count pins re-baselined to the proven actuals
  (`total 1304→1324`, `proof 1→2`, `canonical witness 1283→1302`, `union witness 1302→1321`);
  the gate is GREEN again end-to-end (semantic invariants byte-identical: canonical `UNKNOWN=20`,
  union `UNKNOWN=1`, residual `context_member_method_call`, `spf=0`, seeds 0/7/42). The
  counts-only drift was the stale-pin lockstep gap opened by the `SV-0014`→`SV-0020` and
  `verilog_2005` named-lift campaigns (+20 accounted rules), git-traced in `.2` Findings. Book +
  SV-contract stale mentions updated with provenance. Frontier → `.6` (proposed) / defect fix
  leaves via PNT. SV family status UNCHANGED (`Mostly Done`).

- 2026-07-02: `.4.3` DONE (`PGEN-VERILOG-2005-PROFILE-0009`, CLOSURE — gate/test-data/docs, no
  code-classified change) — the profile's machine-checkable closure surface landed: 46-file
  conformance corpus promoted into `rust/test_data/grammar_quality/verilog_2005_conformance/`;
  repo-standard `verilog_2005_conformance_gate` (tracked contract + script + Makefile target)
  locking the 0-orphan lint invariant, the 138-check 3-profile accept/reject matrix (+2 alias
  probes), and the newly measured profiled cert baseline `total=1138 proof=2 witness=826
  UNKNOWN=310 spf=0` (deterministic seeds 0/7/42; 277/310 = profile-unreachable-by-design SV-only
  surface); downstream SV integration-contract full `verilog_2005` write-up + stale embedding-API
  baseline `1.2.0`→`1.3.0` corrected; LIVE dialect block PROMOTED `In Progress`→`Mostly Done`
  (deliberately not `Done`: curated-corpus proof + the `SV-0024` known leak); main-book /
  probe-book alias-list / SV-book / README lockstep. SV family status UNCHANGED (`Mostly Done`).
  Frontier → `.5` (the `sv_cert_recognized_union_gate` count re-baseline); `.6` proposed
  (profiled-cert baseline ratchet).

- 2026-07-02: `.4.2` DONE (`PGEN-VERILOG-2005-PROFILE-0008`, CODE) — cluster (c) branch-lifts
  landed: 10 shape-preserving `_sv_only` named-lifts + the `interface_port_header` leak gate make
  the bare-keyword SV-only surface (`always_comb/latch/ff`, `do…while`/`foreach`, `bit`/`logic`,
  `shortreal`, `interconnect` port+decl, `ref`/`const ref` directions, `wait fork`/`wait_order`,
  the event-trigger delay form, interface-typed ports) REJECT under `verilog_2005`, and `wire
  logic;` (the `.4.1` over-rejection) ACCEPT. Lint 0 orphans held; cert +10 accounted, UNKNOWN=20
  invariant; full no-regression green; AST shapes byte-identical under sv_2017/sv_2023. Two more
  pre-existing sv_2017 defects found + ledgered (`SV-0023` event-trigger LRM complex, `SV-0024`
  un-braced `port_expression` list). SV family status UNCHANGED (`Mostly Done`). Frontier → `.4.3`
  (corpus promotion + repo-standard `verilog_2005` gate + profiled cert baseline + contract
  write-up + LIVE decision).

- 2026-07-02: `.4.1` DONE (`PGEN-VERILOG-2005-PROFILE-0007`, CODE) — the build-to-coherence campaign's
  whole-rule + keyword axes landed: `verilog_2005` profile-orphans 170→0 (`--lint-grammar` rc 1→0,
  closing the `.2` wellformedness regression), 28 core baseline admissions + 103 SV-only whole-rule
  gates + the D2 profile-split reserved lookahead (full IEEE 1364-2005 Annex B reserved under
  `verilog_2005`; the 48 SV-only reserved words legal as identifiers) + the shape-preserving
  `integer_atom_type_sv_only` lift (`integer`/`time` stay core). Accept/reject proven both ways on a
  22-file conformance corpus; full no-regression green; two pre-existing sv_2017 defects found +
  ledgered (`SV-0021`/`SV-0022`). SV family status UNCHANGED (`Mostly Done`). Frontier → `.4.2`
  (cluster (c) branch-lifts: `always_comb/latch/ff`, `do…while`/`foreach`, `bit`/`logic`,
  `shortreal`, `interconnect`, `->>`).

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
