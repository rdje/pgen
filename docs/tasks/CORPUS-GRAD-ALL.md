# CORPUS-GRAD-ALL: every PGEN parser graduates from its external official/recognized corpora before `Done` — exhaustive per-family corpus discovery, audit, and gate wiring

## Metadata

- Tree ID: `CORPUS-GRAD-ALL`
- Status: **`active`** (created 2026-07-22, session #190, on the director's
  cross-family directives — verbatim: "All PGEN parser to be exersized against
  their external and official and recognized test corpus before claiming they
  are done." + "So you will need to do an exhaustive search for each parser and
  pin all the applicable external test corpus that will increase the confidence
  in the fact that the corresponding parser is really Done and is sota,
  signoff-grade." — [[project_all_parsers_done_requires_corpus_graduation]])
- Roadmap lane: the Nexsim delivery directive
  ([[project_nexsim_sv_signoff_delivery_focus]]) + the locked program + the
  external-corpus doctrine ([[project_external_corpus_doctrine]]).
- Created: `2026-07-22`
- Owner: repo-local workflow
- Cross-links: generalizes [[project_sv_done_requires_external_corpus_graduation]]
  (SV = the template campaign, tree `SV-CORPUS-GRAD`); supersedes the
  EXTERNAL-CORPUS "`.3+` not blocking all-parsers-Done" framing (they now DO
  block, per family); supersedes-by-GO the `REJECTS-VALID-DEFENSE` cross-family
  corpus clause (its non-corpus instruments stay parked); acquisition +
  characterization precedents = `EXTERNAL-CORPUS` (`regex_corpus_bundle/`,
  `json_corpus_bundle/`, `stimuli/sv/subs/`, `stimuli/vhdl/subs/`).

## Goal (the tree's single deliverable)

Every parser family's `Done` claim is backed by graduation from an
**exhaustively-discovered, pinned, frozen roster** of external official/
recognized corpora — adjudicated by each corpus's own answer key / governing
spec with **zero unexplained divergences**, proven by standing deterministic
per-family graduation gates wired into each family's `Done` computation — or an
honest, tracked **N/A-with-cause** adjudication where no external corpus can
exist. Never gamed ([[feedback_corpus_expected_from_spec_not_fix]]).

## Per-family state

⛔ **The `LIVE row` column is DERIVED, never stored here** — it is
`jq -r '.families | to_entries[] | "\(.key): \(.value.claimed_status)"' rust/test_data/grammar_quality/done_bar_family_register_v0.json`,
and the register is the authority (`docs/DERIVED_STATE_CONTAINMENT.md` R1/R3). The table below
originally stored a copy, and by `2026-08-08` **four of its rows had silently diverged** from the
register they claimed to report — `regex` and `ebnf` read `Done` here while the register says
`In Progress`, and `vhdl`/`systemverilog_preprocessor` read `Done` here while the register says
`Provisional (corpus pending)`. That is this tree's own instance of the staleness defect class it
exists to burn down ([[project_all_parsers_fully_pass_stimuli_and_external_corpora]] clause 4), so
the copies are removed rather than refreshed.

⭐ **What that correction changes about `.2`:** the VHDL row was demoted honestly at some point
after this tree was opened, so `.2` is no longer *deciding whether to demote* — it is discharging
the `(corpus pending)` clause the register already admits. The 29.4% below is likewise a
`2026-07-22`-vintage number carried for context only; `.2` re-measures before quoting.

| Family | Vendored today | Known gap / note |
|---|---|---|
| regex | PCRE2 `regex_corpus_bundle/` + live gates (2,189-cell oracle corpus, divergences tracked) | LIKELY already graduated — formal audit + roster check (candidates beyond PCRE2: cross-engine/Unicode suites — `.1` adjudicated applicability under the PCRE2-faithful contract) |
| systemverilog | `stimuli/sv/subs/` (sv-tests/verible/slang/verilator + designs) | campaign ACTIVE = `SV-CORPUS-GRAD` (roster additions, e.g. ivtest for `verilog_2005`, adjudicated there); the `403` unexplained divergences are STALE (last regenerated 2026-07-25) |
| systemverilog_preprocessor | (shares SV suites' preprocessor cases) | mapping pinned in `.1`; explicit slice pending |
| vhdl | `stimuli/vhdl/subs/` (ghdl incl. VESTS+gna / nvc / OSVVM / UVVM / vunit + designs; 29.4% at 2026-07-22, **not a HEAD measurement**) | **NOT graduated** — VESTS ≈2,042 compliant-but-rejected real gaps; the second campaign, `.2` (Nexsim also needs VHDL) |
| json (built-in) | `json_corpus_bundle/` (JSONTestSuite) | graduation inherently gated on the parked `JSON-RFC8259` commitment (`.4` conformance gate) |
| rtl_frontend / rtl_const_expr | — | internal subset grammars — N/A-with-cause (`.1`; possible mapped SV-subset slice) |
| return_annotation / semantic_annotation / ebnf | — | PGEN-defined DSLs — N/A-with-cause, ratified by the director (`.1`) |

## Leaves

### `.1` — EXHAUSTIVE per-family corpus discovery + pinning + `Done`-claim audit (research + read-only)

- **Status: `done`** (`PGEN-CORPUS-GRAD-ALL-0002`, session #190, 2026-07-22;
  read-only — research + banked evidence, zero code change).
- **Executed as four parallel research-grounded web sweeps** (primary sources,
  ~166 searches/fetches total, counts API-verified) + the repo-side vendored
  audit. Banked: `regex_corpus_discovery.md` (18 candidates),
  `sv_svpp_corpus_discovery.md` (33), `vhdl_corpus_discovery.md` (28),
  `json_corpus_discovery.md` (24), synthesized into
  **`frozen_rosters_v1.md`** — the per-family FROZEN graduation rosters
  (vendored tier / ADD v1 tier / rejects-with-cause / N/A-with-cause) + the
  honest `Done`-claim audits. All under `docs/tasks/artifacts/corpus_grad_all/`.
- **Headline discoveries:** ispras/sv-tests (LRM-clause-keyed
  POSITIVE/NEGATIVE suite — the "LRM extraction" instrument) + ivtest's keyed
  CE/gold regressions (and `regress-vlg.list` = THE verilog_2005 corpus);
  JSONTestSuite's un-vendored `test_transform` half + the JSON_checker
  license trap; VESTS has no live upstream (ghdl's copy is canonical, 2 files
  ahead) and NO public IEEE LRM-examples project exists (VASG Packages = the
  legal analogue); UTS#18 ships no conformance corpus; the only new
  VHDL negative-case corpus is vhdl-linter's; no large open preprocessor
  torture suite exists beyond the Verilator lineage (hdlConvertor's
  LRM-page-keyed sv_pp + verilog-perl's goldens are the adds).
- **Audit verdicts:** regex `Done` = corpus-backed today (formal graduation
  statement pending); svpp `Done` = backed via vendored preprocessor cases
  (explicit slice pending); vhdl `Done` = NOT corpus-backed (campaign `.2`);
  internal DSLs (annotation/ebnf/rtl_*) = N/A-with-cause — **RATIFIED by the
  director 2026-07-22 ("If there is none, fine, we will live with it"):** the
  mandate's purpose is maximal justified confidence; an honest exhaustive
  search that finds nothing satisfies it. These N/A rulings are final for v1.
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — the director's exhaustive-search mandate; baseline = only 4 families had any corpus mapping.
  - [x] **ROOT CAUSE (WHY + WHERE)** — N/A (research leaf; the "why" is the doctrine record).
  - [x] **FIX** — N/A (read-only).
  - [x] **ADDRESSED (verified)** — four discovery reports with primary-source citations + API-verified counts banked; rosters frozen v1 with every candidate adjudicated (ADD / reject-with-cause / N/A-with-cause).
  - [x] **NO REGRESSION** — read-only; zero code/grammar/generated change.
  - [x] **LOCKSTEP** — tree + MEMORY + CHANGES this commit; LIVE tracker note already carries the audit flags.

### `.2.0` — the `furthest_position` diagnostic is SV-ONLY: universalize it, so a cross-family corpus campaign can be triaged at all

- **Status: `done`** (`PGEN-CORPUS-GRAD-ALL-0003`, session #213, 2026-08-08).
- **Why this comes before `.2`.** The VHDL campaign's first act is to turn ~9 700 raw
  parse-fails into a *ranked defect-class worklist*. The existing instrument for exactly
  that is `stimuli/sv/cluster_rejects_valid.py`, which clusters a rejects-valid population
  **by stuck-point signature read out of `furthest_position`** (its `POS_RE` treats the
  bracket as optional). Without the bracket a VHDL cluster key degenerates to the surface
  position, and because `vhdl_file := design_unit*` that surface position is always the
  START of the failing design unit — so every one of the 9 689 failures would cluster as
  `package ID is` / `architecture ID of ID is`, which names no defect. ⇒ the campaign's
  triage tool is inert on VHDL until this lands. Prior art checked before building anything
  new (`DESIGN-PRIOR-ART`): the clusterer exists and is family-agnostic except for its
  keyword set — it needed no replacement, it needed its **input signal** to exist.
- **The instrument was over-claiming, which is why nobody noticed.** `TOOLBOX.md` §3.2 is
  titled *"Furthest-position error diagnostic (always on)"* and states *"**every**
  parse-failure error is augmented with `furthest_position`"*. Measured, that is false for
  10 of the 12 detail-parse paths over generated parsers — an instrument whose
  documentation claims coverage it does not have ([[feedback_instrument_needs_ground_truth]]
  applied to the diagnostic surface itself).
- **Scope: repo-wide, not VHDL-only.** Every family's `--parse` rejection now names the deep
  locus, so this same triage is available to `systemverilog_preprocessor`, `json`, `ebnf`,
  `rtl_*`, the annotation DSLs and `regex` without a per-family patch — the "prefer ONE
  shared mechanism over per-probe patches" direction of
  [[project_all_parsers_fully_pass_stimuli_and_external_corpora]].

- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — `./rust/target/debug/parseability_probe --parse vhdl stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/src/Axi4ComponentPkg.vhd` → `Error: parse_full rejected sample for grammar 'vhdl' … Parser did not consume full input at position 1651` — the augmentation `TOOLBOX.md` §3.2 calls "always on" is **absent**. The same probe on SV emits it.
  - [x] **ROOT CAUSE (WHY + WHERE)** — `grep -n 'furthest_position()' rust/src/parser_registry.rs` returns exactly **2** call sites — `1151` (`parse_with_systemverilog_detail_profile_entry`) and `1501` (`parse_with_scratch_detail_entry`) — while `grep -nE '^fn parse_with_.*_detail' rust/src/parser_registry.rs` returns **17** detail entry points, **12** of which own a parser. WHY: the augmentation was written per-site for SV (`SV-EXH-PROOF.3.3.4.b.6.2.25`) as an inline `result.map_err(|err| … parser.furthest_position() …)` block and hand-copied ONCE to `scratch`; being a copied code block rather than a shared helper, it never reached the other 10. WHERE: the `*_detail` bodies in `rust/src/parser_registry.rs`, each of which ends `…map_err(|err| err.to_string())` inside `with_rule_entry_count_dump!` and returns that string unaugmented — including SV's own `parse_with_systemverilog_detail_profile_with_library` (1183), so the gap is not even uniform *within* SV. Verified the signal itself was always available: `grep -c 'pub fn furthest_position' generated/*_parser.rs generated/ebnf.rs` = 1 for all **11** generated parsers — nothing had to be computed, only reported. The observable WHY, side by side on the same probe: SV emits `Parser did not consume full input at position 113637 [furthest_position=643297, +529660 bytes deeper than surface position]` while `--parse vhdl` emits `…at position 1651` with **no `furthest_position=` segment at all** — same binary, same flag, different family.
  - [x] **FIX** — declarative-over-engine tier does not apply (this is a reporting seam, not grammar behaviour): extract the 2 copied blocks into ONE shared helper `augment_error_with_furthest_position(err, furthest)` and call it from every detail path. Net effect is a strict superset of today's diagnostics; no parse decision changes.
  - [x] **ADDRESSED (verified)** — before→after on the reproducer: `…at position 1651` → `…at position 1651 [furthest_position=3852, +2201 bytes deeper than surface position]`. Byte 3852 is line 112, `AxiBus : view Axi4ManagerView of Axi4RecType ;` — a **VHDL-2019 mode view indication** (`view <name> of <type>`) that `grammars/vhdl.ebnf` has no rule for. The surface position 1651 is line 54, `package Axi4ComponentPkg is`, which names no defect at all: it is merely where `vhdl_file := design_unit*` gave up. **+2 201 bytes and 58 source lines of diagnostic distance recovered on one file, in one run.** Coverage before→after: `grep -nE '^fn parse_with_.*_detail'` = **17** entry points, of which 4 are pure delegating wrappers and 1 is the bootstrap path; that leaves **12 own-parser detail paths, augmented 2 → 12** (`grep -c augment_error_with_furthest_position` = 13 = 1 definition + 12 sites). Cross-family spot-check, every one showing a locus the surface position missed: `json` 0→13 (the trailing comma, surface said byte 0), `svpp` 0→6, `rtl_frontend` 0→9, `return_annotation` 0→11, `semantic_annotation` 12→14, `ebnf` 12→13, `regex` 1→4, `rtl_const_expr` 2→3. The honest exclusion is `builtin_semantic_annotation`, whose detail path is the bootstrap `UnifiedSemanticAST::parse_bootstrap` and owns no parser object — stated in the code, not silently skipped.
  - [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` GREEN (17/17 doctrines); `make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change` clean (strict source lint + generated-parser `clippy` correctness stage at 0); the 4 rejection-signature/position unit tests in `rust/src/main.rs` + `stimuli_generator.rs` pass, and the duality-hunt contract's pinned `Parser did not consume full input at position #` signatures stay **byte-identical** because `normalize_rejection_signature` now strips the diagnostic decoration before digit-normalizing (a signature names the failure CLASS; after digit collapse the bracket is a constant suffix carrying zero discriminating power) — so `rust/test_data/grammar_quality/duality_hunt_gate_contract_v0.json` needed no rebaseline.
  - [x] **LOCKSTEP** — `TOOLBOX.md` §3.2 corrected (the false "always on" claim → the measured per-family truth + the stated exclusion); `docs/book/src/diagnosing-unknowns.md` + `docs/book/src/parseability-probe-debug.md` mirrored; `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md` updated. No LIVE-status row changes (no family's closure claim moves).

### `.2` — VHDL graduation campaign opening (the second family campaign)

- **Status: `todo` — NEXT, and unblocked by `.2.0`** — the largest measured gap (29.4%; VESTS
  answer-key adjudication first: compliant[must-parse] / non_compliant[must-reject] /
  gna expected-fail drivers). Scoped from `.1`'s frozen VHDL roster; the VHDL `Done` row is
  re-adjudicated honestly. ⛔ Run inside THIS tree — `MEMORY.md` explicitly forbids opening
  another corpus tree; delegate to an own tree only if the burn-down outgrows a leaf list.
- **The standing 29.4% is NOT a measurement of HEAD, and that is provable, not suspected.**
  `git ls-files stimuli/vhdl/characterization/` lists **only** `characterization.md` — its own
  cited raw input, `results.tsv`, **is not tracked at all**; and every one of its 13 720 rows
  carries an ABSOLUTE path rooted at a *different checkout* — a home-directory clone on the boot
  volume, i.e. off the repository volume — which also breaches the repo-root-relative-paths rule
  (reproduce with `cut -f3 stimuli/vhdl/characterization/results.tsv | head -1`; the path is not
  quoted here because a tracked live doc may not carry one). So the first act of `.2` is a re-measure on HEAD,
  not a quote — exactly the discipline
  [[project_all_parsers_fully_pass_stimuli_and_external_corpora]] demands ("the first honest act
  is to re-measure them rather than to quote them"). ⇒ `stimuli/run_external_corpus.sh` must be
  fixed to emit repo-root-relative paths before its output can be tracked or diffed.
- **First named defect class, already located by `.2.0`'s instrument:** the **VHDL-2019 mode view
  indication** (`AxiBus : view Axi4ManagerView of Axi4RecType ;`) has no rule in
  `grammars/vhdl.ebnf`. Found at `furthest_position=3852` on
  `stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/src/Axi4ComponentPkg.vhd`, where the surface
  position named only `package … is`. Expect it to account for a large share of the OSVVM 8.5%
  row; the clustering pass sizes it rather than assuming it.
- **Method:** re-measure → cluster with `stimuli/sv/cluster_rejects_valid.py`'s stuck-point keying
  (family-agnostic apart from its keyword set) → rank classes → one leaf per class, expected
  verdicts derived from the LRM / suite metadata only ([[feedback_corpus_expected_from_spec_not_fix]]).
- ⚠️ **ROUTED OUT of this leaf (found while scoping it; director call needed, so NOT acted on):**
  a second checkout of this repository exists in the user's home directory on the BOOT volume — a
  real directory, not a symlink, on a different `df` filesystem from the repo's own volume, last
  touched 2026-07-25 — which is where the stale `results.tsv` paths point. Under
  [[project_data_locality_same_volume]] that is project data owed the copy/verify/use/delete
  treatment, but deleting a git checkout is destructive and outside this leaf's scope — it needs an
  explicit director decision and its own tracked unit. (Reproduce:
  `df -h "$HOME/Documents/github/pgen" .` — two different filesystems.)

### ROUTING EVIDENCE (for the off-volume-checkout finding routed out of `.2`)

1. **Does it reproduce OUTSIDE the family it is routed to?** It is not a parser-family finding at
   all, which is exactly why it is routed out rather than fixed here: it is a repository
   DATA-LOCALITY finding, governed by [[project_data_locality_same_volume]], and it would be
   equally true if PGEN had no VHDL grammar. Routing it *into* `.2` would have made a
   volume-hygiene decision look like a corpus decision.
2. **What was MEASURED to place it there, not what makes it plausible?** `df -h` on the two paths
   reports two different filesystems (the repo on its own multi-terabyte volume, the second
   checkout on the boot filesystem); `ls -ld` shows the second path is a real directory, not a
   symlink into the repo volume (so it is a genuine second copy, not an alias); and
   `cut -f3 …/results.tsv` shows the untracked VHDL measurement's rows point at that copy — which
   is how the finding surfaced at all.
3. **What would make this routing WRONG, and was it checked?** It would be wrong if the second
   path were a symlink to this volume (then there is no off-volume data) — checked with `ls -ld`,
   it is not; or if the second checkout were a required read-only toolchain input rather than a
   stray clone (then it is documented cross-volume access, not a breach) — NOT checked, because
   establishing that requires reading another checkout's state, which is beyond this leaf and is
   part of what the director decision must settle. That gap is stated rather than assumed away.
   ⛔ No deletion, move, or inspection of the other checkout was performed.

### `.2.1` — RE-MEASURE VHDL axis 2 on HEAD, and turn 9 689 raw fails into a RANKED defect-class worklist

- **Status: `done`** (`PGEN-CORPUS-GRAD-ALL-0004`, session #213, 2026-08-08).
- **The re-measure: HEAD is byte-for-byte the July number, and that is a finding either way.**
  `13 720` files · `pass 4 031` · `fail 9 689` · `timeout 0` · `crash 0` · **29.4 %**, and every one
  of the ten per-sub-corpus rows is identical to the `2026-07-22` report. So the standing number was
  stale *by construction* (unre-runnable, unre-run) while happening to remain true — the same shape
  as `SV-EXH-PROOF.7.4.6.19`'s replay universe. ⭐ **The re-measure took 71 seconds** (peak RSS
  270 MB). A 71-second measurement had gone 17 days and an unknown number of grammar commits
  without a re-run, because nothing asked it to — which is the mechanizable core of clause 4 of
  [[project_all_parsers_fully_pass_stimuli_and_external_corpora]], and it is now cheap enough that a
  gate is clearly the right remedy (routed there by the record, not assumed here).
- **The worklist (the deliverable):** `docs/tasks/artifacts/corpus_grad_all/vhdl_fail_clusters.md` —
  9 689 rows → **861** distinct 3-token stuck signatures, ranked, each with a resolving example path
  and its stuck source line. Produced in 26 s. Top classes with their grammar-verified cause:

  | rows | signature | example stuck line | grammar reality (`grammars/vhdl.ebnf`, 546 lines / 216 rules) |
  |---|---|---|---|
  | 387 | `until ID =` | `wait until nReset = '1' ;` | `wait_statement := kw_wait (kw_for expression)? semi` — the LRM §10.2 **sensitivity (`on`) and condition (`until`) clauses do not exist**; only the `for` timeout does |
  | 357 | `range NUM to` | `type FREQ is range 0 to integer'high units` | `type_definition` has 3 of the LRM's classes (enumeration / array / record); **no scalar range, no physical type** — `kw_units` count 0 |
  | 256 | `after NUM ID` | `Clk <= not Clk after 10 ns ;` | **`after` appears nowhere in the grammar** — the LRM §10.5.2.1 waveform element's `after` time expression is unimplemented |
  | 219 | `file of ID` | `type T_PICFILE is file of character;` | no file type definition in `type_definition` |
  | 174 | `attribute ID :` | `attribute KEEP : boolean;` | **`kw_attribute` count 0** — no attribute declaration or specification |
  | 149 | `access ID ;` | `type CALL_PATH_VECTOR_PTR is access CALL_PATH_VECTOR ;` | **`kw_access` count 0** — no access type definition |
  | 86 | `shared variable ID` | `shared variable OperationFifo : … ;` | **`kw_shared` count 0** |
  | 71 | `alias ID :` | `alias Last : std_logic is ResultParam(0) ;` | `alias_declaration` accepts ONLY the VHDL-2019 `X is Y'converse` mode-view form — the general object alias is absent |

  ⇒ the honest headline is not "the VHDL parser has bugs" but **the VHDL grammar is an explicit
  SEED SUBSET** (its own header says so: *"Initial VHDL seed grammar focused on executable frontend
  readiness"*), and `.2` is a grammar-GROWTH campaign whose order is now measured rather than guessed.
- **Instrument work (why not a new tool):** `stimuli/sv/cluster_rejects_valid.py` already implements
  the whole engine and is family-neutral apart from its keyword set, so it was **parameterized, not
  forked** — practising the lesson `.2.0` recorded. Added: a `FAMILIES` profile table (keywords +
  multi-char operators + case-folding), a `--results` raw-fail input lane for a family with no
  adjudication yet, and `--grammar`/`--profile`/`--family`. VHDL needs case-folding: it is a
  case-insensitive language, so without it `ENTITY`/`Entity`/`entity` are three clusters and none
  shows its true size. SV defaults are untouched, so the SV invocation is byte-identical.
- ⭐ **Two DEFECTS IN THE INSTRUMENT were found and fixed before its output was trusted** — an
  instrument whose illustration contradicts its own key is a confident guess
  ([[feedback_instrument_needs_ground_truth]]):
  1. **the example line could name a different construct than the signature.**
     `furthest_position` often lands on trailing whitespace/newline; the tokenizer skips newlines and
     took its first token from the NEXT line, while the excerpt was read at the raw byte. Measured:
     cluster `alias ID :` was illustrated by `constant USER_RIGHT : integer := 1 ;` and `shared
     variable ID` by a `subtype` declaration. `signature_at` now also returns the offset of the first
     token it consumed, and the excerpt is anchored there — after the fix those two clusters show
     `alias Last : std_logic is …` and `shared variable OperationFifo : …`.
  2. **the example path was a nonsense doubled path** in the raw-results lane
     (`Compliance-Tests/stimuli/vhdl/subs/Compliance-Tests/…`), because `rel` is already
     repo-root-relative there and the suite was prefixed anyway. Now one path that resolves.
- **`stimuli/run_external_corpus.sh` column 3 is now REPO-ROOT-RELATIVE.** Verified
  backward-compatible before changing it: both consumers key on the `/subs/<suite>/` infix and split
  there, which a relative path still contains. The clusterer additionally **REFUSES** an absolute row
  with the regeneration command rather than silently failing to resolve it.
- **Tracked vs not, stated (no silent caps):** the ranked worklist `.md` is tracked. The per-row
  `vhdl_fail_clusters.tsv` (1.29 MB) and `results.tsv` (1.09 MB) are NOT: `.gitignore:412` excludes
  the raw dumps under a reviewed `EXTERNAL-CORPUS.3.1` policy ("regenerable; track
  characterization.md only"), and that policy is *stronger* now than when it was written, since the
  run is 71 s from portable paths. ⛔ Reversing a reviewed ignore policy is not a side effect of a
  measurement slice. Regenerate with the two commands in this leaf.
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — the standing 29.4 % could not be re-run: `git ls-files stimuli/vhdl/characterization/` lists only `characterization.md`, and `cut -f3 stimuli/vhdl/characterization/results.tsv | head -1` showed every row rooted at a foreign checkout.
  - [x] **ROOT CAUSE (WHY + WHERE)** — `git ls-files stimuli/vhdl/characterization/` (one file, no raw results) + the absolute column-3 emitted by `parse_one` in `stimuli/run_external_corpus.sh`, which printed `"$f"` (the absolute find result) instead of a `$ROOT`-relative path. WHY the worklist could not be built regardless: `grep -c 'furthest_position' stimuli/sv/cluster_rejects_valid.py` shows the clusterer keys on the deep locus via `furthest_position=`, which no VHDL rejection carried until `.2.0`; the surface position for `vhdl_file := design_unit*` is always the failing design unit's first byte, so all 9 689 rows would have collapsed into two classes.
  - [x] **ADDRESSED (verified)** — re-measure on HEAD: `13720 parsed — pass=4031 fail=9689 timeout=0 crash=0 (29.4% pass)` in 71 s, `results.tsv` column 3 now `stimuli/vhdl/subs/…`; clustering: `rows: 9689  clusters: 861` in 26 s, with the 8 classes above each confirmed against `grammars/vhdl.ebnf` by keyword-count (`kw_until`/`kw_on`/`kw_after`/`kw_access`/`kw_attribute`/`kw_units`/`kw_shared` all **0**). Instrument fixes verified before→after on the two named clusters.
  - [x] **NO REGRESSION** — SV lane defaults unchanged (`--grammar systemverilog --profile sv_2017`, `SV_KEYWORDS`, `SV_OPERATORS`, the manifest input path), so the SV invocation is byte-identical; no SV artifact regenerated in this slice. `bash -n stimuli/run_external_corpus.sh` and `python3 -c "ast.parse(...)"` clean; `scripts/check_doctrines.sh` GREEN (17/17); zero Rust, zero grammar, zero generated change ⇒ clippy and the cert/`shape-contract` oracles are untouched by construction.
  - [x] **LOCKSTEP** — tree + `docs/TASK_TREE.md` frontier + `TOOLBOX.md` (the clusterer documented as family-neutral, with the VHDL invocation) + `CHANGES.md` + `DEVELOPMENT_NOTES.md` + `MEMORY.md`. LIVE-status register unchanged: `vhdl` stays `Provisional (corpus pending)` — this slice measures and ranks, it fixes no grammar.

### `.3` — Per-family graduation-gate wiring (code; per-family leaves)

- **Status: `todo`** — each family's `Done` computation gains its
  corpus-graduation criterion (the SV `.5` pattern generalized: deterministic
  gate over pinned corpus commits; family-status gates / LIVE-row oracles).

### `.4` — Lockstep

- **Status: `todo`** — LIVE tracker per-row re-adjudication from the audits,
  books (top-level corpus doctrine chapter + per-parser books), integration
  contracts.

## Delegations (owned elsewhere, tracked here)

- SV → `SV-CORPUS-GRAD` (active; axis 2 of the SV `Done` bar).
- JSON → `JSON-RFC8259` (parked; director-gated).
- The non-corpus REJECTS-VALID instruments → `REJECTS-VALID-DEFENSE` (parked).

## Acceptance Criteria (tree)

1. Every family has a FROZEN, exhaustively-researched, pinned graduation roster
   (or a tracked N/A-with-cause adjudication).
2. Every family's `Done` claim is either backed by a green graduation gate over
   its roster or honestly re-adjudicated (no silent demotions, no gamed passes).
3. Gates are standing, deterministic, wired into each family's `Done`
   computation; full lockstep.
