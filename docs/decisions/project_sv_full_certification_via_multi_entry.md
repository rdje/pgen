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

## ✅ `.8.4.3` LANDED — entry-aware cert verification drives the union `14 → 3` (2026-06-29, `GRAMMAR-WELLFORMED.H.12.8.4.3`, `PGEN-GRAMMAR-WELLFORMED-0141`, **CODE**)

The `-0140` REAL fix is implemented and the SV multi-config cert union now reaches **`UNKNOWN=3`**. Cert
witness VERIFICATION honors the configured entry: the codegen (`ast_based_generator.rs`) emits a
`parse_from(entry)` / `parse_full_from(entry)` per-rule dispatch (default arm = canonical ⇒ single-entry
grammars byte-identical), `parser_registry.rs` (`ParseAndCoverFn` + 7 closures + generic `parse_and_cover`)
takes `entry: Option<&str>` and verifies via `parse_full_from`, and `main.rs` threads the per-config entry
into the 6 cert verification sites.

- **Measured:** union with `--cert-union-config systemverilog_file:sv_2023 + sv_multi_entry_root:sv_2017 +
  library_text:sv_2017 + systemverilog_parseable_file:sv_2017` ⇒ `CERTIFICATE-COVERAGE-UNION: … witness=1300
  UNKNOWN=3` (canonical `UNKNOWN=22` byte-identical), **deterministic at seeds 0/7/42**. All **11
  entry-relative rules witness**; `PGEN_CERT_COVERAGE_DUMP_ALL` confirms the union residual is EXACTLY the
  **3 canonical reach-gaps** (`context_member_method_call`, `known_unscoped_class_scoped_call_interface_class_identifier`,
  `known_unscoped_class_scoped_call_type_parameter_identifier`) — owned by `.8.3`.
- **Why `14 → 3`, not `≤6` (the `.8.4.2`/`.8.4.1` prediction):** `kw_file_path_spec := /file_path_spec\b/`
  is a literal keyword, so the generator emits `include file_path_spec;` AND the parser accepts that literal —
  the sample is self-consistent. The ONLY thing that ever blocked the cohort was the verification *entry*
  (`systemverilog_file` cannot parse `include …;`). Once verification runs from `library_text`, all 11
  witness. So **`.8.4.4` (`file_path_spec` → LRM path char-class) is reframed from witnessing-blocker to an
  optional LRM-fidelity cleanup**; it is NOT needed to reach `UNKNOWN=0`.
- **Residual to literal `UNKNOWN=0`:** ONLY the **3 reach-gaps** (`.8.3`). `.8.2` (`kw_n_*`) was subsumed by
  the `-0138` union; the 11 entry-relative are now witnessed. So the SV `UNKNOWN→0` lane is, via the sound
  multi-config union, down to the single `.8.3` cohort.
- **No regression:** SV canonical byte-identical `22` (seeds 0/7/42); the 6 fully-certified grammars
  byte-identical `fully_certified=true`; `cargo test --lib` 771/0; clippy source-clean; SV external corpus
  14/14. Parser-agnostic, inert for the certified roster by construction (their default entry = canonical).
- **Carve-out:** `parseability_probe --entry-rule` → `.8.4.3.1` (same `parse_full_from` dispatch via the
  detail-parse path; a tool-build slice, kept out of this commit for reviewability).

---

## ✅ `.8.1.1` IMPLEMENTED — the shipped union delivers `22 → 14`, NOT `16` (2026-06-29, `GRAMMAR-WELLFORMED.H.12.8.1.1`, `PGEN-GRAMMAR-WELLFORMED-0138`, **CODE**)

The opt-in `--cert-union-config <entry>[:<profile>]` flag is implemented and verified. The measured SV
union is **`UNKNOWN 22 → 14`** (deterministic seeds 0/7/42), one BETTER than the `-0137` design's
predicted `16` — and the difference was root-caused tools-first, not accepted:

- The flag prints the byte-identical canonical `CERTIFICATE-COVERAGE:` line (`UNKNOWN=22`) plus
  `CERTIFICATE-COVERAGE-UNION: … witness=1289 UNKNOWN=14`. Isolation: **sv_2023-only union ⇒ 14**;
  **`sv_multi_entry_root:sv_2017`-only union ⇒ 22** (adds 0, as `-0136` predicted). So `sv_2023` is the
  sole contributor and it certifies **8**, not 6.
- **The `-0137`/`-0136` premise that `kw_n_29`/`kw_n_48` are profile-filtered OUT of `sv_2023` was
  TOOL-DISPROVEN** (`--dump-gen-ast --grammar-profile sv_2023` shows `kw_n_29`×3, `kw_n_48`×5 PRESENT
  in the sv_2023 rule set; `sv_2023` cert reports `UNKNOWN=17` with neither in it ⇒ both are genuinely
  WITNESSED under `sv_2023`, `sv_2023` proof=1 so it is witness not proof). The prior session almost
  certainly read the `sv_2023` cert UNKNOWN list (where they are absent *because covered*) and
  mis-concluded "absent from the rule_order". The SOUNDNESS rule still holds — the union credits them
  because a config POSITIVELY covers them (a witness), exactly as designed; nothing is certified via
  "not-UNKNOWN-in-some-config" (the lock test `certificate_coverage_union_is_over_positively_covered_sets`
  guards that). This is therefore a sound improvement, and **`.8.2` (the `kw_n_29`/`kw_n_48` artifacts)
  is SUBSUMED by the union** — they are cert-covered. (The grammar-fidelity question — should those
  spurious clause-number tokens exist at all — is orthogonal and may still be tidied later, but it no
  longer blocks `UNKNOWN=0`.)
- **Corrected residual to `0`:** `14` = **11 entry-relative** (`.8.4`: `sv_multi_entry_root`,
  `systemverilog_parseable_file`, `parseable_source_item`, `include_statement`,
  `library_declaration`/`_description`/`_text`, `kw_file_path_spec`/`kw_incdir`/`kw_include`/`kw_library`)
  **+ 3 canonical reach-gaps** (`.8.3`: `context_member_method_call` + the two `…scoped_call…` cousins).
- **No regression:** SV canonical byte-identical seeds 0/7/42; the 6 fully-certified grammars
  byte-identical `fully_certified=true`; `cargo test --lib` 669 passed; `clippy_on_rust_change` source
  clean. Parser-agnostic, inert for the certified roster by construction (they never pass the flag).

Implementation shape (as designed below, with the helper extraction): `gather_cert_covered_sets(...
emit_diagnostics) -> CertCoveredSets` (canonical verbose/byte-identical; union quiet), `Clone` on
`LoadedGrammar`, classify canonical `rule_order` against `⋃(proof ∪ witness)`. The `22 → 16` figure in
the sections below is the DESIGN-TIME prediction; the IMPLEMENTED, tool-verified figure is **`22 → 14`**.

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

---

## ✅ `.8.1` IMPLEMENTATION DESIGN (2026-06-29, `GRAMMAR-WELLFORMED.H.12.8.1` DESIGN, `PGEN-GRAMMAR-WELLFORMED-0137`, PURE-DOCS)

A tools-first code survey of `run_certificate_coverage_report` (`rust/src/main.rs:2349-2850`) + a
fresh this-session re-derivation of the union facts on the Jun-25 binary turn the "head start" design
facts above into a concrete, reviewable implementation plan for the **opt-in multi-config cert union**.
The IMPLEMENT slice (`.8.1.1`) is to be coded against THIS design.

### Tool-verified union facts (this session, seeds re-confirmable)
- **Canonical** `(systemverilog_file, sv_2017)` count 40 seed 0 → `total=1304 proof=1 witness=1281 UNKNOWN=22 spf=0` (reproduces the `-0136`/`H.12.7` baseline exactly).
- **`sv_2023`** `(systemverilog_file, sv_2023)` count 40 seed 0 → `total=1324 proof=1 witness=1306 UNKNOWN=17 spf=0`. The **6 profile-relative** rules (`class_constructor_super_args`, `declared_interface_class_identifier`, `interface_class_declaration`/`_item`/`_method`, `union_modifier`) are GENUINELY WITNESSED here (present in `sv_2023`'s `rule_order`, absent from its UNKNOWN + `no_path` lists).
- **⚠️ kw_n_29/kw_n_48 SUBTLETY (root-caused, not assumed).** A naïve set-diff `canonical_UNKNOWN − sv_2023_UNKNOWN` returns **8** rules (the 6 + `kw_n_29`/`kw_n_48`), which would over-claim `22 → 14`. Tools prove the 2 artifacts are **ABSENT from `sv_2023`'s output entirely** — they are profile-filtered OUT of `sv_2023`'s `rule_order` (their sv_2017 host productions — covergroup-extends `systemverilog.ebnf:1492`, the `local::48` sites `:2869`/`:3926`, and the `sv_2017` `ps_type_identifier` form — are profile-variant; `sv_2023` instead surfaces its OWN artifacts `kw_n_43`/`kw_function_declaraton`/`dot_star` as `no_path`). A profile-filtered-out rule is **neither covered NOR UNKNOWN** in that config, so it must contribute NOTHING to the union.

### THE SOUND UNION SEMANTICS (the load-bearing correctness rule)
> Classify the **canonical (base) config's fragment set** (`rule_order` — the headline denominator, 1304) against `⋃_configs (proof_covered ∪ witness_covered)`. A rule is **CERTIFIED iff POSITIVELY covered (proof OR witness) in SOME config; UNKNOWN iff covered in NONE.**
>
> The union is over the **POSITIVELY-covered sets, NEVER over "not-UNKNOWN-in-some-config."** Subtracting per-config UNKNOWN sets would falsely certify a rule merely because a profile filtered it out (the `kw_n_29`/`kw_n_48` trap). This is the exact reason `certificate_coverage(all_fragments, proof_covered, witness_covered)` (`grammar_wellformedness.rs:1685`) takes the COVERED sets as inputs — it is already the right primitive; the union only feeds it bigger covered sets and the same canonical `all_fragments`.

With the two configs `[(systemverilog_file, sv_2023), (sv_multi_entry_root, sv_2017)]`: the `sv_2023` run adds the 6 profile rules to `witness_covered`; the `sv_multi_entry_root` run adds **0** today (the 11 entry-relative rules gain reach but don't witness — that's `.8.4`'s job, and the framework is ready to credit them once it lands). → **union `UNKNOWN 22 → 16`**, residual = 11 entry-relative + 2 artifacts (`kw_n_29`/`kw_n_48`) + 3 reach-gaps (NOT the 6 profile rules).

### DECIDED design question (director "make the technical decisions"): EXPLICIT OPT-IN CLI, not auto-union
A new repeatable flag **`--cert-union-config <entry>[:<profile>]`**. Each value names an additional `(entry, profile)` config whose verified covered sets union into the canonical accounting. **Empty (default) ⇒ exactly today's single-config behavior, byte-identical for EVERY grammar.** Rejected the auto-union default because it (a) ~3×'s the canonical-report runtime (the cert is ~24 s/config for SV), and (b) would require baking a per-grammar config set into the binary (un-parser-agnostic). Opt-in keeps the supported-config set a property the CALLER declares (a gate invocation / the cert command), and makes inertness for the 6 fully-certified roster provable by construction (they never pass the flag). SV's invocation:
```
ast_pipeline grammars/systemverilog.ebnf --report-certificate-coverage \
  --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0 \
  --cert-union-config systemverilog_file:sv_2023 \
  --cert-union-config sv_multi_entry_root:sv_2017
```

### The refactor (helper extraction — the bulk of the IMPLEMENT work)
Extract the per-config covered-set computation (`run_certificate_coverage_report` body `~2385–2754`: the diverse PASS-1 + proof gathering + constructive-reach + plannable + target-own + store-free + carrier-div passes — NO early returns, accumulates `witness_covered`/`proof_covered`) into:
```
fn gather_cert_covered_sets(
    grammar: &LoadedGrammar, entry_rule: &str, samples: usize, seed: u64,
    profile: Option<&str>, max_depth: usize, emit_diagnostics: bool,
) -> Result<CertCoveredSets>
```
returning a new `struct CertCoveredSets { proof_covered: HashSet<String>, witness_covered: HashSet<String>, sample_parse_failures, reach_pass_parse_failures, plannable_* tallies, no_path: Vec<String> }`.
- **Canonical** call: `emit_diagnostics=true` ⇒ prints the per-pass diagnostic lines (`store-free reach pass: …`, `carrier-diversification reach pass: …`, plannable summary, `WARNING … NO reach path …`) + honors `PGEN_CERT_COVERAGE_DEBUG_PROBES` EXACTLY as today ⇒ **byte-identical canonical output**.
- **Union-config** calls: `emit_diagnostics=false` ⇒ quiet (no per-pass spam; DEBUG_PROBES still allowed for union-config diagnosis if useful).

`run_certificate_coverage_report` then: gather canonical (verbose) → if `--cert-union-config` present, for each: `apply_grammar_profile_filter(bundle.clone(), cfg.profile)` then `gather_cert_covered_sets(quiet)` → union into `proof_union`/`witness_union` (seeded from canonical) → `certificate_coverage(&canonical.rule_order, &proof_union, &witness_union)` → print the canonical `CERTIFICATE-COVERAGE:` line (byte-identical single-config numbers + `sample_parse_failures` from the canonical run) AND, when union configs are present, an additional **`CERTIFICATE-COVERAGE-UNION:`** line (union proof/witness/UNKNOWN/`fully_certified`) + the union DUMP_ALL residual.

### Plumbing
`apply_grammar_profile_filter` CONSUMES `LoadedGrammar` by value and `LoadedGrammar` is not `Clone`. To re-filter per union config, **derive `Clone` on `LoadedGrammar`** (`String` + `HashMap<String,ASTNode>` + `Vec<String>` + `Option<Annotations>` — all `Clone`; confirm `Annotations: Clone` at IMPLEMENT) and thread the UNFILTERED bundle (or a clone) into `run_certificate_coverage_report`. The caller at `main.rs:962` already has the unfiltered `load_grammar_bundle(...)` result before it is consumed by `apply_grammar_profile_filter`.

### Inert-by-default proof + verification matrix (the IMPLEMENT acceptance checklist)
- **NO REGRESSION (canonical byte-identity):** with NO `--cert-union-config`, the function takes the canonical-only path ⇒ `gather_cert_covered_sets(emit_diagnostics=true)` reproduces the exact pre-refactor computation + output. PROVE empirically: SV `total=1304 … UNKNOWN=22 spf=0` byte-identical at **seeds 0/7/42**; the **6 fully-certified grammars byte-identical** `fully_certified=true` (json/regex/vhdl/svpp/rtl_frontend/rtl_const_expr) at their canonical entries; SV external corpus 14/14; `ast_shape_contract` GREEN; `cargo test --lib` green; `clippy_on_rust_change` source-clean.
- **ADDRESSED (union):** SV union (the 2 configs) ⇒ `UNKNOWN 22 → 16`, deterministic seeds 0/7/42, residual a strict subset = 11 entry-relative + `kw_n_29`/`kw_n_48` + 3 reach-gaps (the 6 profile rules certified, NOT in residual). Add a `cargo test` lock asserting (a) the union certifies the 6 profile rules and (b) `kw_n_29`/`kw_n_48` do NOT leak in via profile-filtering (the soundness guard).
- **Parser-agnostic:** no grammar names in code; the union is driven entirely by the CLI configs.

This design supersedes the "open Q = explicit union mode vs auto-union" note in the `H.12.8` row (decided: explicit opt-in CLI) and the "8 certified" naïve-set-diff reading (corrected: 6, via the sound covered-set union). The `22 → 16` figure stands.
