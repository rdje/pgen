# GRAMMAR-WELLFORMED.H.12.8.4 — entry-relative cohort WHY+WHERE + LRM-grounded fix design (`file_path_spec` artifact is the dominant root cause)

Tools-first root-cause map of the **11 entry-relative** SV certificate-coverage residual
rules (the `library_text` / parseable-fragment cohort), reproduced this session on the
current cert binary, with the LRM-grounded fix design for the dominant blocker. This is the
diagnosis+design half of the `.8.4` leaf flagged by `H.12.8.0`/`.8.1.1`
([[project_sv_full_certification_via_multi_entry]]); the implement half is `.8.4.2`.

> Slice `PGEN-GRAMMAR-WELLFORMED-0139` (**PURE-DOCS** — tracker + continuity lockstep; NO
> code/grammar/generated/release/schema/ledger change). Status: `done`.
> Reads with [[feedback_systematically_use_debug_toolbox]], [[feedback_why_and_where_before_solution]],
> [[feedback_tools_first_no_guessing]], [[feedback_no_codebase_change_without_tool_backed_facts]],
> [[feedback_no_rule_deletion_without_lrm_proof]], [[feedback_research_grounded_sota_no_trial_and_revert]],
> [[project_ebnf_is_single_source_of_truth]], [[feedback_always_signoff_decisions]].
> Parent: the SV `UNKNOWN`→0 lane `GRAMMAR-WELLFORMED.H.12.8`; precedent `H.12.7` (`-0134`,
> the 22-residual adjudication that first named the 11 entry-relative cohort) and `.8.1.1`
> (`-0138`, the opt-in `--cert-union-config` that brought the union to `UNKNOWN=14`).

## Why now (the frontier)

`.8.1.1` (`-0138`) shipped the multi-config cert union and proved the SV union residual is
**`UNKNOWN=14`** = **11 entry-relative** + **3 reach-gaps** (`.8.3`). The director-committed
endgame ([[project_sv_full_certification_via_multi_entry]]) is to drive that `14 → 0`. The 11
entry-relative rules gain *reach* under `sv_multi_entry_root` but do **not witness** — `.8.4`
owns turning that reach into a real witness. This slice pins the exact mechanism + source
location of each of the 11 (the WHY+WHERE), proves one shared root cause dominates, and
designs the minimal LRM-faithful fix; `.8.4.2` implements it.

## THE EVIDENCE (tools-first, reproduced this session)

Canonical + union baseline (`ast_pipeline --features "generated_parsers ebnf_dual_run"`, default parser paths, seed 0):

```
PGEN_CERT_COVERAGE_DUMP_ALL=1 ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf \
  --report-certificate-coverage --grammar-profile sv_2017 --entry-rule systemverilog_file \
  --count 40 --seed 0 \
  --cert-union-config systemverilog_file:sv_2023 --cert-union-config sv_multi_entry_root:sv_2017
# CERTIFICATE-COVERAGE:       … total=1304 proof=1 witness=1281 UNKNOWN=22 fully_certified=false (spf=0)
# CERTIFICATE-COVERAGE-UNION: … witness=1289 UNKNOWN=14 fully_certified=false
# UNION UNKNOWN rules (14 of 14): sv_multi_entry_root, systemverilog_parseable_file,
#   parseable_source_item, include_statement, library_declaration, library_description,
#   library_text, context_member_method_call, kw_file_path_spec_c26c9dc9, kw_incdir_e08adf20,
#   kw_include_d3ecb0d8, kw_library_00299a40,
#   known_unscoped_class_scoped_call_interface_class_identifier,
#   known_unscoped_class_scoped_call_type_parameter_identifier
```

**Per-rule `[plannable-probe]` verdicts under `--entry-rule sv_multi_entry_root` (where the 11 gain reach)** — `PGEN_CERT_COVERAGE_DEBUG_PROBES=1 … --entry-rule sv_multi_entry_root --count 40 --seed 0`:

| Rule | `parsed` | `witnessed` | forced sample | Mechanism |
|---|---|---|---|---|
| `sv_multi_entry_root` | true | false | `""` | B — routing (trivial alt) |
| `systemverilog_parseable_file` | true | false | `""` | B — routing (trivial alt) |
| `parseable_source_item` | true | false | `";"` | B — routing (trivial alt) |
| `library_text` | true | false | `""` | B — routing (`*` ⇒ zero reps) / downstream-blocked by A |
| `library_description` | true | false | `";"` | B — routing / downstream-blocked by A |
| `include_statement` | **false** | false | `"include file_path_spec;"` | **A — `file_path_spec` literal** |
| `library_declaration` | **false** | false | `"library\\foo file_path_spec;"` | **A — `file_path_spec` literal** |
| `kw_file_path_spec_c26c9dc9` | **false** | false | `"library\\foo file_path_spec;"` | **A — `file_path_spec` literal** |
| `kw_incdir_e08adf20` | **false** | false | `"library\\foo file_path_spec-incdir file_path_spec;"` | **A — `file_path_spec` literal** |
| `kw_include_d3ecb0d8` | **false** | false | `"include file_path_spec;"` | **A — `file_path_spec` literal** |
| `kw_library_00299a40` | **false** | false | `"library\\foo file_path_spec;"` | **A — `file_path_spec` literal** |

Deterministic: the canonical/union counts (`22`/`14`) reproduce at seeds 0/7/42 (`-0138`).

## THE ROOT CAUSE (WHY + WHERE)

**Mechanism A — `file_path_spec` is an LRM-extraction artifact (the DOMINANT root cause).**
`grammars/systemverilog.ebnf:5736`:

```
kw_file_path_spec_c26c9dc9 := trivia /file_path_spec\b/
```

This token rule matches the **literal word** `file_path_spec`. It is referenced by exactly two
productions — `include_statement` (`:2313`) and `library_declaration` (`:2591`). The stimuli
generator therefore faithfully emits the literal text `file_path_spec` where a real file path
belongs (`"include file_path_spec;"`, `"library \foo file_path_spec;"`), and those samples do
not witness the target. This is the **same defect class as `kw_n_29`/`kw_n_48`**: an LRM
nonterminal flattened into a literal-keyword terminal during markdown extraction.

**LRM ground truth (IEEE 1800-2017 §33.3.1, Annex A.1.1** — `docs/systemverilog/2017/txt/section-33-configuring-the-contents-of-a-design.txt:117-140`**):**
`file_path_spec` is a **file-system path token**, not a keyword — *"file_path_spec uses
file-system-specific notation to specify an absolute or relative path to a particular file or
set of files,"* with documented wildcards `?` (single char), `*` (multi char), `...`
(hierarchical), `..` (parent), `.` (current), and `/` separators. Annex A gives it **no formal
lexical production** (it is described textually), so the LRM-faithful encoding is a token rule
matching a path-like lexeme.

**Mechanism A blocks 8 of the 11**, directly and transitively:
- direct (6, `parsed=false`): `include_statement`, `library_declaration`,
  `kw_file_path_spec_c26c9dc9`, `kw_incdir_e08adf20`, `kw_include_d3ecb0d8`, `kw_library_00299a40`.
- transitive (2, mechanism-B routing rules whose only content path runs through
  `library_declaration`): `library_text := library_description*` (`:2601`) and
  `library_description := library_declaration` (`:2594`) cannot witness their body because the
  only `library_description` alternative, `library_declaration`, is itself unparseable while A
  holds. Fixing A is a prerequisite for these two.

**Mechanism B — multi-entry scaffolding routing (3, genuinely routing-only).** The remaining
three are the analysis-only umbrella rules whose generation legitimately picks a trivial/empty
alternative:
- `sv_multi_entry_root := systemverilog_file | library_text | systemverilog_parseable_file`
  (`:203`) — the planner picks the empty-capable `library_text`/`systemverilog_parseable_file`
  branch ⇒ `""`.
- `systemverilog_parseable_file := trivia parseable_source_item* trivia` (`:209`) — `*` ⇒ zero reps ⇒ `""`.
- `parseable_source_item := semi` (`:214`) — its non-`semi` alternatives are commented out
  (`:216-218`), so only `";"` is reachable; the witness needs the rule to be entered under a
  forcing path. These are routing/forcing gaps, handled after A (some may fall out once
  `library_text` witnesses, since `sv_multi_entry_root` can then route through a non-empty
  `library_text`).

## THE FIX DESIGN (`.8.4.2`, CODE — minimal, LRM-faithful, AST-shape-neutral)

**Correct the `file_path_spec` lexeme at `systemverilog.ebnf:5736`** — keep the rule *name and
arity* (so the positional `$N` references in `include_statement`/`library_declaration` are
unchanged and **no AST shape / schema change** results), replace only the regex with an
LRM-faithful path lexeme. Proposed (final regex validated against the EBNF char-class/escape
conventions — `[...]` classes, `\/` for slash, trailing `-` literal in a class, like
`escaped_identifier := /\\[!-~]+/` and `compiler_directive := /\`[^\r\n]*/`):

```
kw_file_path_spec_c26c9dc9 := trivia /[A-Za-z0-9_.\/?*~$+-]+/
```

This covers the §33 path alphabet (letters, digits, `_`, `.`, `/`, the `?`/`*` wildcards, `~`,
`$`, `+`, `-`) while stopping at whitespace, `,`, and `;` (the grammar's path delimiters), and
deliberately excludes `\` (no collision with `escaped_identifier`/library_identifier) and `"`
(no quoted form — bare paths per the LRM). Word-boundary `\b` is dropped (a path is not a word).

**Why this is the minimal correct fix (not a deletion, not a rename):** per
[[feedback_no_rule_deletion_without_lrm_proof]] the rule is NOT dead — it is referenced and
*should* exist; the LRM proves it must accept a path, not a literal. The `kw_` prefix + hash
suffix is the extraction-naming convention for terminals and is left intact (cosmetic; an
optional later tidy). Keeping the rule a single token at the same position guarantees the
typed carrier is unchanged.

**Expected effect (to be MEASURED in `.8.4.2`):** the 6 mechanism-A rules become `parsed=true`
and witness under `sv_multi_entry_root`; the 2 transitive routing rules (`library_text`,
`library_description`) then witness; `sv_multi_entry_root` may also witness once `library_text`
is non-empty. Target: union `UNKNOWN 14 → ≤ 6` (the 3 pure-routing scaffolding rules + the 3
`.8.3` reach-gaps; the pure-routing residue, if any, is a `.8.4.3` forcing slice). Canonical
`UNKNOWN=22` is expected **unchanged** (the library cohort is `no_path` from `systemverilog_file`
by LRM design — they only witness via the `sv_multi_entry_root` union config).

**Ceremony for `.8.4.2` (accept-changing flagship-SV grammar edit):** regen
`generated/systemverilog_parser.rs` + rebuild both binaries; verify cert canonical `22` +
union before→after at seeds 0/7/42 `spf=0`; the 6 fully-certified grammars byte-identical
`fully_certified=true`; SV external corpus 14/14; `ast_shape_contract` GREEN; `--lint-grammar`
clean; `clippy_on_rust_change` source-clean; released-SV release/ledger bump + per-parser SV
book / integration-contract / top-level book lockstep (the library-map-file path acceptance is
a user-visible accept change for the `library_text` analysis entry, even though the
`systemverilog_file` embedding entry is unaffected).

## Acceptance Checklist (PURE-DOCS — code-change boxes N/A)
- [x] **REPRODUCE / ISSUE** — union run above ⇒ `UNKNOWN=22` canonical / `14` union (seed 0, deterministic 0/7/42 per `-0138`); 14-rule union residual pasted.
- [x] **ROOT CAUSE (WHY + WHERE)** — `PGEN_CERT_COVERAGE_DEBUG_PROBES=1 --entry-rule sv_multi_entry_root` verdict table above splits the 11 into mechanism A (6 `parsed=false`, literal `file_path_spec`) + 2 transitive + 3 routing; pinned to `systemverilog.ebnf:5736` (`kw_file_path_spec_c26c9dc9 := /file_path_spec\b/`), LRM-grounded against §33.3.1 / Annex A.1.1.
- [x] **FIX** — N/A here (design only). Designed: replace the `:5736` lexeme with the LRM-faithful path char-class `/[A-Za-z0-9_.\/?*~$+-]+/`, rule name/arity unchanged (AST-shape-neutral); implement + measure in `.8.4.2`.
- [x] **ADDRESSED (verified)** — the 11-rule cohort is now fully mechanism-mapped tools-first; the dominant root cause (`file_path_spec` artifact, 8 of 11) is isolated + LRM-grounded; the implement slice + its verification matrix are scoped.
- [x] **NO REGRESSION** — PURE-DOCS ⇒ no code/grammar/generated/release/schema/ledger change ⇒ every deterministic gate inherits the `-0138` green state byte-identical (SV cert canonical `22` / union `14` seeds 0/7/42 `spf=0`; 6 fully-certified grammars `fully_certified=true`; SV external corpus 14/14; `ast_shape_contract` GREEN).
- [x] **LOCKSTEP** — tree `GRAMMAR-WELLFORMED.md` (new `.8.4`/`.8.4.1` frontier rows + Decisions entry); `MEMORY.md` / `CHANGES.md` / `DEVELOPMENT_NOTES.md`. No book/contract/ledger change (no behaviour change this slice; those land with `.8.4.2`). `LIVE_ACHIEVEMENT_STATUS.md` unchanged (SV stays `Mostly Done`).
