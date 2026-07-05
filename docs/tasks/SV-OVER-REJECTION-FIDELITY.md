# Task Tree: SV-OVER-REJECTION-FIDELITY

Owner tree for the three remaining **over-REJECTION** defects that bound the SystemVerilog /
`verilog_2005` LRM-fidelity claim — spec-valid input that `grammars/systemverilog.ebnf` wrongly
REJECTS (the worst defect class: a parser rejecting known-valid input, per
`feedback_fix_parser_bugs_asap_highest_priority` / `feedback_uvm_is_valid_sv`). Surfaced
2026-07-02 by the `VERILOG-2005-PROFILE.4.1`/`.4.2` conformance-corpus probes and ledgered
`SV-0021`/`SV-0022`/`SV-0023`; carved into their own focused tree 2026-07-04 (session #34) from the
standing SV/`verilog_2005` FINALIZATION lane, mirroring the sibling all-profile SV-correctness trees
(`SV-KEYWORD-PRIMARY-FIDELITY`, `SV-LRM-SHAPE-FIDELITY`, `SV-DOLLAR-LRM-FIDELITY`). One concern per
leaf, one leaf per commit.

- **`SV-0021`** — mixed ANSI port list: an implicit-type ANSI port followed by an explicitly-typed
  one — `module m (input a, output reg b);` — wrongly REJECTED under **every** profile. Valid IEEE
  1800 (A.1.3) and valid IEEE 1364-2005 (ANSI port declarations with a `reg` type). **Already FIXED
  as a consequence of `SV-0037`** (`SV-KEYWORD-PRIMARY-FIDELITY.3.1`, `1.0.162`→`1.0.163`): the
  greedy ungated `net_type_identifier` in `net_port_type_sv_2017/2023` was eating the implicit
  port's name under the global `longest_match` policy, corrupting the first port into a `nonansi`
  shape so a following typed ANSI port had no valid continuation → over-rejection. This tree's `.1`
  leaf ADJUDICATES `SV-0021` closed and adds the missing regression lock.
- **`SV-0022`** — `bind` double-`semi`: a spec-valid single-`semi` `bind … ;` never parses under any
  SV profile while the invalid `;;` form accepts. `bind_directive` (`grammars/systemverilog.ebnf:621`)
  requires a trailing `semi` after `bind_instantiation`, but every `bind_instantiation` alternative
  already consumes its own `;`. Over-rejection **+** over-acceptance pair. *(open — grammar fix)*
- **`SV-0023`** — event triggers: the IEEE 1800 §9.4.4 NONBLOCKING trigger `->>` is unsupported (the
  literal `->>` is absent from the grammar); the optional `[delay_or_event_control]` is mis-attached
  to a second `->` branch (`-> #5 e;` wrongly accepts); and the 1364-2005 event-array trigger
  `-> e[0];` rejects. Under-support + over-acceptance complex; the swapped `kind` labels are
  consumer-visible → schema-bump candidate. *(open — grammar fix)*

## Acceptance Criteria (tree)

- Each spec-valid form ACCEPTS in every profile whose LRM admits it, at ALL carriers (re-probe after
  each fix for sibling leaks, cf. `VERILOG-2005-PROFILE.6.11`'s cascade discipline).
- Each spec-invalid sibling (e.g. the `bind` `;;` form) REJECTS where the LRM forbids it.
- Every fixed / adjudicated form is pinned by a re-runnable regression lock in the
  `verilog_2005_conformance` corpus + contract so it cannot silently regress.
- Full no-regression where a parser/grammar change is made: canonical/union cert (seeds 0/7/42),
  `ast_shape_contract`, the external corpus, the `verilog_2005` conformance matrix + cert,
  `--lint-grammar` 0 profile-orphans, the 6 fully-certified grammars byte-identical, clippy
  source-clean. For an adjudication-only leaf (no parser change), the conformance gate is the
  necessary + sufficient oracle and the parser-behavior gates are inert by construction (proven by a
  grammar/generated-free diff).
- Release/schema per `PGEN_RELEASE_POLICY`: an accept↔reject behavior change on shipped SV profiles →
  release bump; schema unchanged unless a NEW AST envelope shape is introduced (the `SV-0023` label
  correction is the schema-bump candidate).

## Current Frontier

- ID: `SV-OVER-REJECTION-FIDELITY.1` — Status: `done` (2026-07-04, session #34,
  `PGEN-SV-OVER-REJECTION-FIDELITY-0001`, CORPUS/adjudication leaf — no parser change) — closes
  `SV-0021`. The ledgered all-profile over-rejection `module m (input a, output reg b);` no longer
  reproduces on HEAD (`1.0.165`); tools-first proved it was fixed as a consequence of `SV-0037`
  (see "`.1` Findings" below). This leaf adds the missing regression lock
  (`accept/ansi_implicit_then_typed_port.v`, accept under all 3 profiles) + adjudicates the ledger
  row `Root Caused`→`Released`. No grammar/parser/schema/release change. See "Acceptance Checklist —
  `.1`" below.
- ID: `SV-OVER-REJECTION-FIDELITY.2` — Status: `done` (2026-07-04, session #34,
  `PGEN-SV-OVER-REJECTION-FIDELITY-0002`, CODE leaf — grammar-only) — closed `SV-0022`
  (`bind` double-`semi`). Removed the trailing `semi` from both `bind_directive` alternatives
  (`grammars/systemverilog.ebnf:668`/`:670`) so a spec-valid single-`;` `bind` parses. SV release
  `1.0.165` → `1.0.166` (schema 15 unchanged — `semi` was the uncaptured last position). `bind … c1 ();`
  (top-level + module-scope) flips REJECT→ACCEPT under `sv_2017`/`sv_2023`, stays REJECT under
  `verilog_2005`; `bind_dir.sv` corpus rows flipped to SV-accept. All gates GREEN seeds 0/7/42. See
  "`.2` Findings" + "Acceptance Checklist — `.2`" below.

## `.2` Findings (tools-first, 2026-07-04 session #34 — SV-0022 bind double-semi)

**REPRODUCE (HEAD `1.0.165` release binary — SV-0022 STILL reproduces, unlike SV-0021):**
```
printf 'module m; endmodule bind m my_checker c1 ();\n'  | probe --parse … --profile sv_2017  → REJECT (pos 20, furthest_position=44)   [WRONG — valid IEEE 1800]
printf 'module m; endmodule bind m my_checker c1 ();;\n' | probe --parse … --profile sv_2017  → ACCEPT   [the double-semi mechanism confirmation]
printf 'module m; bind m my_checker c1 (); endmodule\n'  | probe --parse … --profile sv_2017  → REJECT (furthest_position=34)          [WRONG]
```

**ROOT CAUSE (WHY + WHERE):** `bind_directive` (`grammars/systemverilog.ebnf:668`) is
`kw_bind bind_target_scope ( colon bind_target_instance_list )? bind_instantiation semi` (and the
second alt `:670` `kw_bind bind_target_instance bind_instantiation semi`). But
`bind_instantiation` (`:673`) is just `program_instantiation | module_instantiation |
interface_instantiation | checker_instantiation`, and EVERY one of those already consumes its own
trailing `semi` (`module_instantiation:3280` `… ( comma hierarchical_instance )* semi`;
`program_instantiation:4310`; `interface_instantiation:2677`; `checker_instantiation:928`). So
`bind_directive` requires a SECOND `;` that the LRM does not: the spec-valid `bind … c1 ();` is
consumed through the instantiation's `;`, then `bind_directive`'s trailing `semi` finds nothing →
REJECT; only the invalid-looking `c1 ();;` parses (instantiation eats `;`, bind_directive's `semi`
eats the second).

**LRM ADJUDICATION (from the tracked markdown, per `feedback_corpus_expected_from_spec_not_fix`):**
IEEE 1800-2017 §23 (`docs/systemverilog/2017/md/section-23-modules-and-hierarchy.md:753`):
`bind_directive ::= bind bind_target_scope [: bind_target_instance_list] bind_instantiation ;`, and
`bind_instantiation ::=` (`:772`) is the 4 instantiation alternatives with NO `;` of its own, while
`module_instantiation ::=` (`:833`) ends in `;`. The literal published BNF therefore expands to a
redundant DOUBLE `;` (`bind … inst() ; ;`) — a known LRM redundancy that every commercial simulator
(VCS/Questa/Xcelium/Verilator/slang) resolves as a SINGLE `;`. (The earlier ledger row's "IEEE
1800-2017 A.1.4 puts no extra `;` after `bind_instantiation`" was imprecise: the `;` is in
`bind_directive`; the point is it is REDUNDANT with the instantiation's own `;`.) The grammar
over-specified by keeping BOTH `;`s.

**FIX (declarative, grammar-only):** remove the trailing `semi` from both `bind_directive`
alternatives; the instantiation's own `;` is the terminator. `semi` is the last, UNCAPTURED position
(`$5`/`$4`), so the return-annotation `$` refs (`instantiation: $4` / `$3`) are unchanged → NO schema
change. Release bump (accept↔reject behavior change on shipped SV profiles) per `PGEN_RELEASE_POLICY`.

## Acceptance Checklist (`.2`, enforced)

- [x] **REPRODUCE / ISSUE** — `bind m my_checker c1 ();` REJECTed (`furthest_position=44`, top-level + module-scope); `c1 ();;` ACCEPTed. Confirmed on HEAD `1.0.165` release binary.
- [x] **ROOT CAUSE (WHY + WHERE)** — `bind_directive` (`:668`/`:670`) required a trailing `semi` after `bind_instantiation`, whose alternatives (`module_instantiation:3280`, `program_instantiation:4310`, `interface_instantiation:2677`, `checker_instantiation:928`) each already consume their own `;`; the IEEE 1800-2017 §23 BNF (`section-23…:753`) has a redundant double-`;` resolved as single by every commercial tool.
- [x] **FIX** — declarative/grammar tier: removed the trailing `semi` from both `bind_directive` alternatives; positionally safe (uncaptured last token `$5`/`$4` → no schema change).
- [x] **ADDRESSED (verified, release binary post-regen)** — `bind … c1 ();` REJECT→**ACCEPT** under `sv_2017`/`sv_2023` (top-level + module-scope), REJECT under `verilog_2005`; `bind_dir.sv` flips to SV-accept; the `;;` form is now a legitimate `bind … ();` + empty package item under SV (cf. SV-0028), REJECT under verilog_2005.
- [x] **NO REGRESSION (seeds 0/7/42)** — `sv_cert_recognized_union_gate` GREEN (canonical `1343/2/1321/20` byte-identical, union `UNKNOWN=1` residual `context_member_method_call`, witness 1340); `verilog_2005_conformance_gate` GREEN (orphans 0, matrix `231/0` with `bind_dir.sv` flipped, cert `1117/4/773/340` byte-identical); `ast_shape_contract` 18/18 (1 test, shapes byte-identical); external corpus 14/14; `--lint-grammar` census 1465 / profile_orphans 0; the 6 fully-certified grammars byte-identical (SV-only regen, mtimes); clippy source-clean (grammar-only, flow skipped).
- [x] **LOCKSTEP** — ledger `SV-0022` → `Released`; SV integration contract (version `1.0.165`→`1.0.166`, SV-0022 waiver→FIXED, remaining-bound framings, new Release 1.0.166 section); conformance corpus `reject/bind_dir.sv` rows flipped to SV-accept; SV parser book changelog `1.0.166` + regenerated HTML; top book `parser-families.md`; `docs/TASK_TREE.md`; release bump `1.0.166`; LIVE/CHANGES/DEVELOPMENT_NOTES/MEMORY.
- ID: `SV-OVER-REJECTION-FIDELITY.3` — Status: `done` (2026-07-05, session #35,
  `PGEN-SV-OVER-REJECTION-FIDELITY-0003`, CODE leaf — grammar-only + schema/release bump) — closed
  `SV-0023` (event-trigger complex). Added the `->>`
  nonblocking-trigger token, split the event-trigger block into three profile-faithful rules, move
  the optional `[delay_or_event_control]` onto the `->>` (nonblocking) branch, correct the swapped
  consumer-visible `kind` labels (`->` is BLOCKING, `->>` is NONBLOCKING per IEEE 1800 §15.5), and
  give the `verilog_2005` branch a `bit_select` so the IEEE 1364-2005 event-array trigger accepts.
  SCHEMA BUMP `15`→`16` (restructured event-trigger return annotations + new/removed branches) +
  release bump `1.0.166`→`1.0.167` (accept↔reject on shipped SV profiles). See "`.3` Findings" +
  "Acceptance Checklist — `.3`" below.

## `.3` Findings (tools-first, 2026-07-05 session #35 — SV-0023 event-trigger complex)

**REPRODUCE (HEAD `1.0.166` release binary — all four SV-0023 facets reproduce):**
```
initial ->> e;    (sv_2017) → REJECT (furthest_position=29)   [WRONG — IEEE 1800 §9.4.4/A.6.5 ->> nonblocking trigger unsupported; literal `->>` absent from the grammar]
initial ->> #5 e; (sv_2017) → REJECT (furthest_position=29)   [WRONG — ->> with [delay_or_event_control] unsupported]
initial -> #5 e;  (sv_2017) → ACCEPT                          [WRONG — per A.6.5 the optional control belongs to ->> ONLY; the grammar mis-attached it to a second `->` branch]
initial -> e[0];  (verilog_2005) → REJECT (furthest_position=37) [WRONG — IEEE 1364-2005 A.6.5 `-> hei { [expression] } ;` array trigger; branch has no bracket-select]
initial -> e;     (sv_2017) → ACCEPT but AST {kind:"non_blocking"}  [WRONG LABEL — `->` is the BLOCKING trigger per §15.5.3]
initial -> #5 e;  (sv_2017) → AST {kind:"blocking", control:{kind:"delay"}}  [WRONG LABEL + over-accept]
```
AST-dump-pinned label swap confirmed via `--parse-dump-ast-pretty`.

**ROOT CAUSE (WHY + WHERE):** the event-trigger block `grammars/systemverilog.ebnf:2152-2168`:
- The `->>` token appears NOWHERE in the grammar (grep `->>` = 0 hits) → the nonblocking trigger is
  unsupported (facet 1/2).
- The optional `( delay_or_event_control )?` is attached to `event_trigger_control_sv_only` (`:2153`,
  `implies ( delay_or_event_control )? hierarchical_event_identifier semi`) — a SECOND `->` (`implies`)
  branch — instead of to `->>`; so `-> #5 e;` over-accepts (facet 3).
- The consumer-visible `kind` labels are SWAPPED vs IEEE 1800 §15.5: the plain `-> hei ;` branch
  (`:2157`) is labeled `{kind:"non_blocking"}` (LRM: `->` is BLOCKING, §15.5.3) and the
  control-carrying branch (`:2154`) is labeled `{kind:"blocking"}` (LRM: `->>` is NONBLOCKING,
  §15.5.4) — the schema-bump driver.
- `event_trigger_sv_2017` (`:2156`) is gated `["sv_2017","verilog_2005"]` with a bare
  `implies hierarchical_event_identifier semi` (no bracket-select), so the IEEE 1364-2005 array
  trigger `-> e[0];` (A.6.5 `-> hierarchical_event_identifier { [ expression ] } ;`) has no `[…]`
  path and rejects under `verilog_2005` (facet 4).

**LRM ADJUDICATION (spec-first, per `feedback_corpus_expected_from_spec_not_fix`):**
- IEEE 1800-2017 A.6.5 (`docs/systemverilog/2017/md/section-15…:287-288`):
  `event_trigger ::= -> hierarchical_event_identifier ; | ->> [ delay_or_event_control ] hierarchical_event_identifier ;`.
- IEEE 1800-2023 Annex A (`docs/systemverilog/2023/md/section-Annex_A…:2192-2193`): same with a
  trailing `nonrange_select` on each alternative.
- IEEE 1364-2005 (`grammars/verilog_2005_lrm_extracted.ebnf:370-371`):
  `event_trigger ::= "->" hierarchical_event_identifier { [ expression ] } ;` — no `->>`, with
  bracket array-selects.
- Semantics: §15.5.3 `->` = blocking named-event trigger; §15.5.4 `->>` = nonblocking trigger and the
  ONLY form carrying the optional `[ delay_or_event_control ]`.

**FIX (declarative, grammar-only, fix-hierarchy tier 1):** add `nonblocking_implies := trivia "->>"`
(gated `["sv_2017","sv_2023"]`, modeled on the existing 3-char operator tokens
`arithmetic_shift_left`/`iff_arrow`); split the event-trigger block into three profile-faithful
rules — `event_trigger_sv_2017` (2017 BNF, no select), `event_trigger_sv_2023` (2023 BNF, trailing
`nonrange_select`), `event_trigger_verilog_2005` (1364-2005 BNF, trailing `bit_select`); each has a
`->` blocking branch and (SV only) a `->> [ delay_or_event_control ]` nonblocking branch; the top
`event_trigger` dispatches to all three. Positional `$` refs verified. `event_trigger_control_sv_only`
is removed (its only reference was `:2159`). Census `1465`→`1466` (−1 `event_trigger_control_sv_only`,
+1 `nonblocking_implies`, +1 `event_trigger_verilog_2005`). No new EBNF construct (`trivia "…"` is
existing syntax) → no `ebnf.ebnf` port.

## Acceptance Checklist (`.3`, enforced)

- [x] **REPRODUCE / ISSUE** — HEAD `1.0.166` release binary: `->> e;` / `->> #5 e;` REJECT
  (`furthest_position=29`), `-> #5 e;` wrongly ACCEPTs, `-> e[0];` REJECTs under `verilog_2005`
  (`furthest_position=37`); `-> e;` AST mislabeled `{kind:"non_blocking"}`, `-> #5 e;` AST
  `{kind:"blocking", control:{kind:"delay"}}` (pinned via `--parse-dump-ast-pretty`).
- [x] **ROOT CAUSE (WHY + WHERE)** — event-trigger block `grammars/systemverilog.ebnf:2152-2168`:
  `->>` token absent; optional control mis-attached to a second `->` branch
  (`event_trigger_control_sv_only:2153`); `kind` labels swapped vs IEEE 1800 §15.5.3/15.5.4; the
  shared `sv_2017`/`verilog_2005` branch has no bracket-select for the 1364-2005 array trigger.
- [x] **FIX** — declarative/grammar tier: add `nonblocking_implies := trivia "->>"`; three
  profile-faithful event-trigger rules; optional control on the `->>` branch only; corrected labels;
  `bit_select` on the `verilog_2005` branch.
- [x] **ADDRESSED (verified, release binary post-regen)** — `->> e;` / `->> #5 e;` REJECT→**ACCEPT** under
  `sv_2017`/`sv_2023` (REJECT under `verilog_2005`); `-> #5 e;` **ACCEPT→REJECT** under every profile;
  `-> e[0];` REJECT→**ACCEPT** under `verilog_2005` + `sv_2023` (REJECT under `sv_2017` per the strict 2017
  BNF — no select); `-> e;` AST label `non_blocking`→**`blocking`** and `->> #5 e;` → `{kind:"non_blocking",
  control:{kind:"delay"}}` (pinned via `--parse-dump-ast-pretty`). Baseline `-> e;` stays ACCEPT all profiles.
  All flips deterministic (3×). Census `1465`→`1466`.
- [x] **NO REGRESSION (seeds 0/7/42)** — `sv_cert_recognized_union_gate` **GREEN** (canonical `1343/2/1321/20`
  byte-identical, union UNKNOWN=1 residual `context_member_method_call`, witness 1340 — the sv_2017/union tree
  net census change is 0: −`event_trigger_control_sv_only` +`nonblocking_implies`); `verilog_2005_conformance_gate`
  **GREEN** (`gate_green:true`, orphans 0, matrix `240/0` with 3 new locks + the `event_trigger_delay.sv`
  re-adjudication, cert `1117/4/773/340` byte-identical seeds 0/7/42 — v2005 tree net census change 0:
  −`event_trigger_sv_2017` from the v2005 universe +`event_trigger_verilog_2005`); `ast_shape_contract_gate`
  **18/18** (no sample exercises event triggers → inert); `sv_external_corpus_triage_gate` **14/14**;
  `--lint-grammar` census 1466 / `non_terminating=0` / `ordered_choice_shadowing=0` / `unreachable_rules=0` /
  `profile_orphans=0`; the 6 fully-certified grammars byte-identical (SV-only regen — mtimes prove the other 6
  generated parsers untouched); clippy source strict-clean (grammar-only, generated regen only).
- [x] **LOCKSTEP** — ledger `SV-0023` `Root Caused`→`Released` (Fix/Fixed-in `1.0.167`/Status filled);
  integration contract (version `1.0.166`→`1.0.167`, schema `15`→`16` note prepended, dialect trust-posture
  bullet, `SV-0023` waiver→FIXED, new § "Release 1.0.167"); conformance corpus (`accept/event_trigger_array.v`,
  `reject/event_trigger_nonblocking.sv`, `reject/event_trigger_nonblocking_delay.sv` added +
  `reject/event_trigger_delay.sv` re-adjudicated + `accept/event_trigger.v` note); SV parser book
  (`schema-versioning` current-schema `13`→`16` drift fixed + schema-16 prose/table row, `changelog-index`
  1.0.167 entry, `json-carrier` event_trigger 3-branch + corrected labels + new verilog_2005 row) + regenerated
  HTML; `docs/TASK_TREE.md` frontier; LIVE/CHANGES/DEVELOPMENT_NOTES/MEMORY.

## `.1` Findings (tools-first, 2026-07-04 session #34 — SV-0021 mixed ANSI port list)

**REPRODUCE (ledger repro, now on HEAD release binary `1.0.165`):**
```
printf 'module m (input a, output reg b); endmodule\n' \
  | parseability_probe --parse systemverilog /dev/stdin --profile sv_2017   → parse_full passed (rc0)
                                                         --profile sv_2023   → parse_full passed (rc0)
                                                         --profile verilog_2005 → parse_full passed (rc0)
```
The exact ledgered failing case (which historically REJECTED at `furthest_position=25` on
`1.0.0`–`1.0.158`) now ACCEPTS on all three profiles, deterministically (3×). A broad 20-case
mixed-ANSI permutation battery (implicit→typed, typed→implicit, ranged, `inout`, `logic`/`wire`/`reg`
typed) is 20/20 ACCEPT across `sv_2017`/`sv_2023`. The AST is a coherent `ansi` header (port `a`
implicit `net_or_interface`, port `b` typed with `data_type → reg`).

**ROOT CAUSE (WHY + WHERE) — fixed as a consequence of `SV-0037`:** git provenance pins the fixing
commit `19fc7ed9` (`PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0004`, `SV-KEYWORD-PRIMARY-FIDELITY.3.1`,
`1.0.162`→`1.0.163`). Its tools-first trace showed `net_port_type_sv_2017` (`grammars/systemverilog.ebnf:3474`,
alt 1 `net_type_identifier`) / `net_port_type_sv_2023` (`:3479`) run under the GLOBAL DEFAULT
`branch_policy=longest_match` (not ordered-choice), so the ungated `net_type_identifier` matching a
bare implicit-port name (2 bytes) out-consumed alt 0's empty implicit match (0 bytes) and ATE the
port name — corrupting the first implicit-type ANSI port into a `nonansi` shape. When the next port
was explicitly typed (`output reg b`), the `nonansi` framing had no valid continuation → the whole
ANSI header over-rejected (`SV-0021`). The `.3.1` fix routed both alts to the store-gated
`checked_nettype_identifier` (`:3537`, the `SV-PARSE-STRICT.2` declared-nettype gate) so alt 0's
implicit `ansi` path correctly wins and the name binds; the mixed implicit-then-typed header then
parses. `SV-0021` and `SV-0037` share the identical mechanism and rule seam; `SV-0021` is the
over-rejection surface of the same defect, and both were closed by the one `.3.1` change.

**FIX (this leaf):** no parser change — the behavior is already correct. Add the missing regression
lock so the mixed-ANSI-header acceptance cannot silently regress:
`rust/test_data/grammar_quality/verilog_2005_conformance/accept/ansi_implicit_then_typed_port.v`
(`module m (input a, output reg b); endmodule`) + its contract case (accept/accept/accept, alongside
the sibling port locks `accept/port_concat.v` / `reject/port_bare_multi_id.sv`).

## Acceptance Checklist (`.1`, enforced)

- [x] **REPRODUCE / ISSUE** — ledger repro `module m (input a, output reg b);` now `parse_full passed`
  on `sv_2017`/`sv_2023`/`verilog_2005` (rc0), deterministic 3×; 20/20 mixed-ANSI battery ACCEPT. The
  historical over-rejection (`furthest_position=25`) is gone.
- [x] **ROOT CAUSE (WHY + WHERE)** — fixed as a consequence of `SV-0037`: the ungated
  `net_type_identifier` (`net_port_type_sv_2017/2023`, `grammars/systemverilog.ebnf:3474`/`:3479`)
  ate the implicit port name under `longest_match`, corrupting the first port to `nonansi` so a
  following typed ANSI port had no continuation. Fixing commit `19fc7ed9`
  (`SV-KEYWORD-PRIMARY-FIDELITY.3.1`, routed both alts to store-gated `checked_nettype_identifier`
  `:3537`).
- [x] **FIX** — declarative/test tier: add the `verilog_2005_conformance` accept-lock
  `accept/ansi_implicit_then_typed_port.v` + contract case; no grammar/parser/schema/release change
  (parser already correct at `1.0.165`).
- [x] **ADDRESSED (verified)** — `verilog_2005_conformance_gate` GREEN (`gate_green: true`, 0 unmet):
  the new lock `accept/ansi_implicit_then_typed_port.v` parses `observed=accept` under all 3 profiles
  (`verilog_2005`/`sv_2017`/`sv_2023`), matrix `228`→`231` file×profile checks / **0 mismatches**.
- [x] **NO REGRESSION** — grammar/generated-free diff (only `rust/test_data/…` corpus+contract + docs
  touched → parser behavior byte-identical → canonical/union cert, `ast_shape_contract`, external
  corpus, the 6 fully-certified grammars all inert by construction). `verilog_2005_conformance_gate`
  re-verified GREEN seeds 0/7/42: cert `1117/4/773/340` byte-identical (corpus-independent), matrix
  `231/0`, `--lint-grammar` orphans 0. *(pasted below.)*
- [x] **LOCKSTEP** — ledger `SV-0021` `Root Caused`→`Released`; LIVE dialect block, `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `docs/TASK_TREE.md`, the conformance contract `note`, and the
  matrix-count mentions (`228`→`231`) updated. No user-facing parser-behavior change (already shipped
  at `1.0.163`) → no book chapter drift.

## Decisions

- `2026-07-04`: `SV-0021` is closed by ADJUDICATION + a regression lock, not a new fix, because
  tools-first proof (git provenance + the 20/20 battery) shows it was already resolved as a
  consequence of `SV-0037`. Owning it in its own leaf keeps the ledger honest and pins the
  acceptance so it cannot silently regress.
- `2026-07-04`: the tree is carved separately from `SV-LRM-SHAPE-FIDELITY` (over-ACCEPTANCES) and
  `SV-KEYWORD-PRIMARY-FIDELITY` (keyword-as-primary) because over-REJECTIONS are a distinct defect
  class (spec-valid input rejected) with distinct mechanisms; grouping `SV-0021`/`SV-0022`/`SV-0023`
  matches the `VERILOG-2005-PROFILE` LIVE framing of "the remaining all-profile over-REJECTION
  complex."

## Open Questions

- None blocking the frontier. `SV-0022`/`SV-0023` (`.2`/`.3`) are genuine open grammar fixes deferred
  to their own leaves.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-04` | `SV-OVER-REJECTION-FIDELITY.1` | ledger repro + 20-case mixed-ANSI battery (release `1.0.165`) | `20/20 ACCEPT` on `sv_2017`/`sv_2023`; ledger repro `parse_full passed` on all 3 profiles, deterministic 3× |
| `2026-07-04` | `SV-OVER-REJECTION-FIDELITY.1` | `verilog_2005_conformance_gate` (rebuild + matrix + cert seeds 0/7/42) | **GREEN** (`gate_green: true`, 0 unmet): new lock ACCEPT all 3 profiles, matrix `231`/0 mismatches, lint `profile_orphans=0` (1465 rules), cert `1117/4/773/340` spf=0 byte-identical seeds 0/7/42 |
| `2026-07-04` | `SV-OVER-REJECTION-FIDELITY.2` | bind flip (release binary post-regen) | `bind … c1 ();` top-level + module-scope REJECT→**ACCEPT** on `sv_2017`/`sv_2023`, REJECT on `verilog_2005`; lint 1465 / orphans 0 |
| `2026-07-04` | `SV-OVER-REJECTION-FIDELITY.2` | no-regression suite (seeds 0/7/42) | `sv_cert_recognized_union_gate` **GREEN** (canonical `1343/2/1321/20`, union `1`, witness 1340 byte-identical); `verilog_2005_conformance_gate` **GREEN** (matrix `231/0`, cert `1117/4/773/340` byte-identical, orphans 0); `ast_shape_contract` 1/1; external corpus 14/14; 6 grammars byte-identical; clippy source-clean |
| `2026-07-05` | `SV-OVER-REJECTION-FIDELITY.3` | facet flips (release binary post-regen) | `->> e;`/`->> #5 e;` REJECT→**ACCEPT** (sv_2017/sv_2023), REJECT (verilog_2005); `-> #5 e;` **ACCEPT→REJECT** (all); `-> e[0];` REJECT→**ACCEPT** (verilog_2005/sv_2023), REJECT (sv_2017); `-> e;` label `non_blocking`→`blocking`; all deterministic 3×; lint census 1466 / orphans 0 |
| `2026-07-05` | `SV-OVER-REJECTION-FIDELITY.3` | no-regression suite (seeds 0/7/42) | `sv_cert_recognized_union_gate` **GREEN** (canonical `1343/2/1321/20` byte-identical, union `1` residual `context_member_method_call`, witness 1340); `verilog_2005_conformance_gate` **GREEN** (matrix `240/0` with 3 new event-trigger locks + `event_trigger_delay.sv` re-adjudication, cert `1117/4/773/340` byte-identical, orphans 0); `ast_shape_contract` 18/18; external corpus 14/14; 6 grammars byte-identical (mtimes); clippy source-clean |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SV-OVER-REJECTION-FIDELITY.1` | `PGEN-SV-OVER-REJECTION-FIDELITY-0001 (SV-OVER-REJECTION-FIDELITY.1): SV-0021 CLOSED — mixed implicit-then-typed ANSI port list adjudicated Released (fixed as consequence of SV-0037) + regression-locked` | Corpus/adjudication leaf — no grammar/parser/schema/release change |
| `SV-OVER-REJECTION-FIDELITY.2` | `PGEN-SV-OVER-REJECTION-FIDELITY-0002 (SV-OVER-REJECTION-FIDELITY.2): SV-0022 CLOSED — spec-valid single-semi bind no longer over-rejects; SV 1.0.165->1.0.166` | Grammar-only; release bump `1.0.166` (schema 15 unchanged) |
| `SV-OVER-REJECTION-FIDELITY.3` | `PGEN-SV-OVER-REJECTION-FIDELITY-0003 (SV-OVER-REJECTION-FIDELITY.3): SV-0023 CLOSED — event-trigger ->>/delay/array LRM complex restored; SV 1.0.166->1.0.167, schema 15->16; TREE COMPLETE` | Grammar-only; release bump `1.0.167` + schema bump `15`→`16` (restructured event-trigger annotations + new/removed branches) |

## Changelog

- `2026-07-04`: Created task tree (session #34) for the `SV-0021`/`SV-0022`/`SV-0023` over-rejection
  complex; opened `.1` (SV-0021 adjudication + regression lock).
- `2026-07-04`: `.1` done (SV-0021 CLOSED, `PGEN-SV-OVER-REJECTION-FIDELITY-0001`, commit `96bba68d`).
- `2026-07-04`: `.2` done (SV-0022 CLOSED, `PGEN-SV-OVER-REJECTION-FIDELITY-0002`) — grammar-only,
  removed `bind_directive`'s redundant trailing `semi`; SV `1.0.165`→`1.0.166` (schema 15); all gates
  GREEN seeds 0/7/42. Remaining leaf: `.3` (SV-0023 event-trigger complex).
- `2026-07-05`: `.3` done (SV-0023 CLOSED, `PGEN-SV-OVER-REJECTION-FIDELITY-0003`) — grammar-only,
  rewrote the event-trigger block per IEEE 1800 §15.5/A.6.5 + IEEE 1364-2005 A.6.5: added the `->>`
  (`nonblocking_implies`) token, split into three profile-faithful rules, moved the optional
  `[delay_or_event_control]` onto the `->>` (nonblocking) branch, corrected the swapped
  `blocking`/`non_blocking` labels, added the `verilog_2005` array bracket-select. SV `1.0.166`→`1.0.167`,
  schema `15`→`16`; census `1465`→`1466`; all gates GREEN seeds 0/7/42 (union `1343/2/1321/20`,
  conformance matrix `240/0` + cert `1117/4/773/340`, ast_shape 18/18, external corpus 14/14).
  **TREE COMPLETE** — all three all-profile over-REJECTION defects `SV-0021`/`SV-0022`/`SV-0023` closed.
