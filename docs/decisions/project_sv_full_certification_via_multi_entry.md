---
name: project-sv-full-certification-via-multi-entry
description: DIRECTOR DECISION (2026-06-25) — SystemVerilog WILL be driven to fully_certified UNKNOWN=0; the chosen vehicle is a MULTI-ENTRY/MULTI-PROFILE certification accounting (certify a rule if witnessed-or-proven in ANY supported (entry, profile) config; UNKNOWN only if UNKNOWN in ALL). TOOL-BACKED CORRECTION (2026-06-29, H.12.8.0): the union delivers 22->16 NOT 22->5 — only the 6 profile-relative rules witness (via sv_2023); the 11 entry-relative rules GAIN REACH under sv_multi_entry_root but do NOT witness (genuine generation gaps: trivial-alternative routing + file_path_spec literal-name) and become new sub-leaf .8.4. Residual to 0 = 11 entry-relative (.8.4) + 2 extraction artifacts kw_n_29/kw_n_48 (.8.2) + 3 canonical reach-gaps (.8.3). Owned by the SV UNKNOWN->0 lane (GRAMMAR-WELLFORMED.H.12.8).
metadata:
  node_type: memory
  type: project
  created: 2026-06-25
  owning_tree: GRAMMAR-WELLFORMED
  decided_by: director
---

## ⚠️ TOOL-BACKED CORRECTION TO THE UNION PREMISE (2026-06-29, `GRAMMAR-WELLFORMED.H.12.8.0`, `PGEN-GRAMMAR-WELLFORMED-0136`)

**The director's GOAL stands (SV → `fully_certified` `UNKNOWN=0`); the union PREMISE below is
corrected by the toolbox.** A tools-first re-derivation of the design facts before any code
(canonical + alt-config `--report-certificate-coverage` runs at seeds 0/7/42 + `PGEN_CERT_COVERAGE_DEBUG_PROBES=1`)
shows the multi-entry/multi-profile union delivers SV **`22 → 16`**, NOT the `22 → 5` this record
originally implied:

- **`sv_2023` profile run** (`(systemverilog_file, sv_2023)` → `UNKNOWN=17`, seeds 0/42): the **6
  profile-relative** rules (`class_constructor_super_args`, `declared_interface_class_identifier`,
  `interface_class_declaration`, `interface_class_item`, `interface_class_method`, `union_modifier`)
  DO witness here — set-diff vs the canonical residual = exactly those 6. ✅ The union certifies these 6.
- **`sv_multi_entry_root` entry run** (`(sv_multi_entry_root, sv_2017)` → `UNKNOWN=22` UNCHANGED,
  seeds 0/7/42): changing the entry collapses `no_path` 19→8 (the 11 entry-relative rules GAIN REACH)
  but **witnesses NONE of them** — `UNKNOWN` stays 22. `PGEN_CERT_COVERAGE_DEBUG_PROBES=1` shows WHY:
  the 11 split into **routing gaps** (`library_text` / `library_description` / `parseable_source_item`
  / `systemverilog_parseable_file` — the entry `Or` picks a trivial `""`/`";"` alternative, never
  forcing the target: `parsed=true witnessed_target=false`) and **malformed-forced-sample gaps**
  (`library_declaration` / `include_statement` / `kw_include` / `kw_incdir` / `kw_library` /
  `kw_file_path_spec` — the forced sample emits the LITERAL text `file_path_spec` instead of expanding
  it to a path token, so it does not parse: `parsed=false`). ❌ The union does NOT certify these 11.

**Root cause of the original over-claim:** the `H.12.7` adjudication (`-0134`) correctly said the 11
entry-relative rules *gain reach* under `sv_multi_entry_root` (`no_path 19→8`); the `-0135` decision
record then conflated *gained reach* with *witnessed*. The tools prove reach ≠ witness for this family.

**Corrected lane shape (`UNKNOWN=0` is still achievable, but the union is a MINOR contributor — 6 of 22):**
- `.8.1` (union accounting) — sound + the director's chosen vehicle, but its true yield is **`22 → 16`**
  (certifies the 6 profile-relative via `sv_2023`; the `sv_multi_entry_root` run adds 0 to the canonical
  denominator). Open design Q for the implementation slice: an explicit union *mode* vs. an auto-union
  default (auto-union ≈ triples the canonical-report runtime); either way it must stay inert +
  byte-identical for the 6 fully-certified grammars (their alternate-config set is empty).
- `.8.4` (NEW) — the **11 entry-relative library/parseable-fragment generation gaps**: genuine
  reach/generation work (fix the trivial-alternative routing + the `file_path_spec` literal-name
  expansion), NOT free accounting. Tool-proven they do not witness even from the multi-entry root.
- `.8.2` (2 extraction artifacts `kw_n_29`/`kw_n_48`) + `.8.3` (3 canonical reach-gaps) — unchanged.

The lane's center of gravity therefore shifts from *accounting* (the union, 6 rules) to
*generation/grammar fixes* (the 11 entry-relative + 3 reach-gaps = 14 rules, + 2 artifacts). All
subsequent `.8.x` implementation is designed on THESE corrected facts. The original director decision
(below) is retained verbatim — only the union's reach was over-stated, not the goal.

---

**The decision (director, 2026-06-25).** SystemVerilog — the one shipped grammar still
`Mostly Done` — **WILL be driven to `fully_certified`, `UNKNOWN=0`. This is achievable and we
will achieve it.** When offered the post-`H.12.7` fork (1: adopt a multi-entry/multi-profile
cert accounting; 2: attack the deferred reach-gaps), the director chose **(1) as the vehicle**,
explicitly to reach `UNKNOWN=0`. The honest-limit *deferral* on the 3 reach-gaps is therefore
**LIFTED** — they are now in-scope to close, not to document-and-defer.

**Context (tool-backed, `H.12.7` / `PGEN-GRAMMAR-WELLFORMED-0134`).** SV cert under the single
canonical config (`--grammar-profile sv_2017 --entry-rule systemverilog_file`) reports
`UNKNOWN=22` (deterministic seeds 0/7/42), fully adjudicated into:
- **11 entry-relative** rooted under the LRM `library_text` / parseable-fragment start symbols
  (IEEE 1800-2017 §33 / Annex A.1.1) — witness under `--entry-rule sv_multi_entry_root`
  (`no_path` collapses 19→8);
- **6 profile-relative** genuine IEEE 1800-2023 features (interface-class family +
  `class_constructor_super_args` + `union_modifier`) — witness under `--grammar-profile sv_2023`
  (`UNKNOWN 22→17`);
- **2 LRM-extraction artifacts** `kw_n_29_7719a1c7` (`/29\b/`) / `kw_n_48_64e095fe` (`/48\b/`):
  spurious literal clause-numbers leaked into productions during LRM extraction
  (`systemverilog.ebnf:1492` covergroup-extends `… semi kw_n_29 coverage_spec_or_option* …`;
  `:2869`/`:3926`/`:4328` `( kw_local scope_resolution kw_n_48 )?` = a bogus `local::48`). They
  make those OR-branches require a number that never appears in real SV ⇒ dead ⇒ `no_path` under
  every entry/profile;
- **3 reach-gaps** (the genuine canonical-entry residual): `context_member_method_call`
  (store-gated declaration-hosting carrier the generator can't yet synthesise) +
  `known_unscoped_class_scoped_call_interface_class_identifier` +
  `known_unscoped_class_scoped_call_type_parameter_identifier` (`T::method()` ambiguity above
  the rule's alternative).

**The path to `UNKNOWN=0` (the committed plan — design-first, tools-first, NOT YET STARTED).**
1. **Multi-entry/multi-profile cert accounting** (parser-agnostic proof-tooling): certify a rule
   if witnessed-or-proven in ANY officially-supported `(entry, profile)` config — the supported set
   being at least `(systemverilog_file, sv_2017)`, `(systemverilog_file, sv_2023)`, and the
   `library_text` / `sv_multi_entry_root` entries. This certifies the **17** entry/profile-relative
   rules where they actually witness. DESIGN-FIRST: read the cert-coverage accounting code
   (`run_certificate_coverage_report` + the reach/BFS pass + the result struct) and decide whether
   the union is a new cert mode, a multi-run merge, or a multi-entry reach graph — tool-backed
   WHY+WHERE before any edit.
2. **The 2 extraction artifacts** `kw_n_29`/`kw_n_48`: LRM-ground the affected productions
   (covergroup-extends; `ps_type_identifier`/`class_scope` `local::` forms), correct the spurious
   number token at source per the no-deletion policy ([[feedback_no_rule_deletion_without_lrm_proof]]),
   OR certify them as a verified unreachability PROOF if LRM-proven dead — released-SV ceremony if
   accept-changing.
3. **The 3 reach-gaps**: close them for real — `context_member_method_call` via the
   STORE-AWARE-GEN declaration-hosting carrier (the name-coupled gen-time `variable_binding` prelude,
   `H.12.5.5.3.3.4.2.1.2.2.3` design, blocked on `STORE-AWARE-GEN.4b` value-selection); the two
   `…scoped_call…` cousins via the grammar tightening that excludes type-parameter/interface-class
   heads from the generic scoped-call alternative (the deferral is now lifted).

**Discipline (binding).** Each step is its own task-tree leaf under `GRAMMAR-WELLFORMED.H.12.8`,
tools-first WHY+WHERE before any code, decisive A/B with GLOBAL cert + `spf` at seeds 0/7/42, the
6 fully-certified grammars byte-identical, SV external corpus 14/14, and the full released-SV
lockstep ceremony for any accept-changing grammar edit. The cert-accounting change is
parser-agnostic and must be inert for the 6 already-`fully_certified` grammars. Composes with
[[project_grammar_wellformedness_contract]] (literal-0 is the consequence of well-formedness),
[[project_certcoverage_measures_structural_not_validator]], [[feedback_no_rule_deletion_without_lrm_proof]],
[[feedback_research_grounded_sota_no_trial_and_revert]], and [[feedback_no_codebase_change_without_tool_backed_facts]].

**DESIGN FACTS for `.8.1` (tool-mapped this session via a read-only code survey — a HEAD START, re-verify before coding).**
The cert-accounting code is already shaped for a clean union:
- The classifier is `certificate_coverage(all_fragments, proof_covered, witness_covered) -> CertificateCoverageReport`
  (`rust/src/ast_pipeline/grammar_wellformedness.rs:1685`; struct at `:1669` = `total` / `covered_by_proof`
  / `covered_by_witness` / `unknown`). A rule is PROOF if in `proof_covered`, WITNESS if in `witness_covered`
  (and not proof), else UNKNOWN.
- **PROOF side is entry-INDEPENDENT (structural):** `gather_verified_proof_covered_rules` (`:1723`) =
  `detect_unreachable_rules` (`:268`) ∪ `detect_lookahead_only_rules` (`:426`), each re-checked by the
  independent `verify_wellformedness_certificate` (`:1496-1585`). `reachable_rules()` is already DUAL-ROOT
  (canonical entry `rule_order[0]` + every unreferenced rule as a secondary root, `:284-324`).
- **WITNESS side is entry/profile-SPECIFIC but union-safe:** witnesses come from the 5 sequential passes
  (`main.rs` ~`2417`/`2489`; plannable `generate_plannable_rule_witnesses` stimuli_generator.rs`:3809`,
  store-free `:3837`, carrier-div `:3961`, target-own `:4264`), each verified through
  `parser_registry::parse_and_cover(grammar, sample, profile)` and **union-only** (passes only ADD to the
  covered set). `--grammar-profile` filters `rule_order`/`grammar_tree` via `apply_grammar_profile_filter`
  (`main.rs:2212`) before the report runs.
- **UNION VERDICT (the agent's conclusion — verify, don't trust):** unioning `covered_by_proof` and
  `covered_by_witness` across runs for distinct `(entry, profile)` configs is reportedly feasible with no
  structural obstacle (rule names are profile-agnostic strings; proof is structural; witness is union-safe)
  ⇒ a rule is certified iff witnessed-or-proven in ANY config, UNKNOWN iff UNKNOWN in ALL. So `.8.1` is
  plausibly a thin multi-run merge (or a new cert mode that loops the configs and unions) rather than a deep
  rewrite — but this MUST be re-derived tools-first and proven byte-identical-inert for the 6 fully-certified
  grammars before any edit. There is currently NO stored "blessed decomposition / unreachability
  certificate" — proof is recomputed each run, which is where `.8.2`'s `kw_n_*` proof-of-unreachability (if
  that route is chosen) would plug in.
