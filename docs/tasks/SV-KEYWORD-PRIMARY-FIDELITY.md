# Task Tree: SV-KEYWORD-PRIMARY-FIDELITY

Owner tree for `SV-0035` — the **all-profile** reserved-keyword-as-primary over-acceptance:
a reserved type keyword (`integer`, `real`, `time`, `reg`, `wire`, `logic`, `int`, `bit`, …)
parses as a bare **primary expression** under EVERY profile (`sv_2017`/`sv_2023`/`verilog_2005`),
e.g. `localparam p = integer;` and `assign w = integer;` both wrongly ACCEPT. Reserved words
cannot be identifiers (IEEE 1364-2005 Annex B / IEEE 1800 Annex B), so a reserved keyword must
never match an identifier-primary.

Created 2026-07-04 (session #29) from the `VERILOG-2005-PROFILE.6.10` tools-first discovery
(`PGEN-VERILOG-2005-PROFILE-0022`): `reg q [integer];` kept accepting after the associative-array
gate because `[integer]` routes through `unpacked_dimension`'s expression as a keyword-as-primary,
NOT the associative surface. This is an **all-profile** correctness defect (distinct from the
`verilog_2005`-only dialect-boundary family `SV-0025`..`SV-0034`), on shipped releases — so it
warrants its own focused tree (cf. the `SV-DOLLAR-LRM-FIDELITY` precedent for an all-profile SV
correctness family), design-first because the widest carrier (`hierarchical_identifier`) underpins
every name reference in the 1465-rule grammar.

## Acceptance Criteria (tree)

- Every reserved keyword (the `reserved_non_keyword_identifier_sv` / `_v2005` lists) REJECTS in every
  expression name-primary position under every profile, with the leak closed at ALL carriers (no
  sibling leak — the "leak moves to the next alternative" failure mode explicitly ruled out by
  re-probing after each fix, cf. `VERILOG-2005-PROFILE.6.11`'s cascade discipline).
- Legitimate (non-reserved) name references parse byte-identically (AST-shape-invariant).
- Full no-regression: canonical/union cert, `ast_shape_contract`, the external corpus, the
  realistic/uvm corpora (no over-rejection of legal names), the 6 fully-certified grammars.
- Release/schema handling per `PGEN_RELEASE_POLICY` (an accept→reject behavior change on shipped SV
  profiles → release bump; schema unchanged — no new AST shape, `{body:X}` preserved).

## Current Frontier

- ID: `SV-KEYWORD-PRIMARY-FIDELITY.1` — Status: `done` (2026-07-04, session #29,
  `PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0001`, DESIGN/AUDIT leaf — ZERO code): the complete tools-first
  carrier map + fix architecture + shape/corpus-risk analysis. See "`.1` Findings" below. Stages the
  implementation as `.2`.
- ID: `SV-KEYWORD-PRIMARY-FIDELITY.2` — Status: `done` (2026-07-04, session #29,
  `PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0002`, CODE leaf): routed the three confirmed expression-primary
  bare-`identifier` carriers (`specparam_identifier:5083`, `genvar_identifier:2375`, and the trailing
  name of `hierarchical_identifier:2412`) through the keyword-excluding `non_keyword_identifier`.
  SV-0035 (reserved *type* keyword as primary) CLOSED under every profile; SV release `1.0.161` →
  `1.0.162` (schema `15` unchanged). The "re-probe after each fix" discipline surfaced a *distinct,
  broader* residual — the reserved-list-completeness leak now owned by `.3` (ledger `SV-0036`). See
  "`.2` Findings" + "Acceptance Checklist" below.
- ID: `SV-KEYWORD-PRIMARY-FIDELITY.3` — proposed (CODE leaf): close `SV-0036` — extend
  `reserved_non_keyword_identifier_sv` (`grammars/systemverilog.ebnf:386`) to the full SV reserved
  keyword set so the 35 net/gate/structural Verilog keywords (`wire`/`wand`/`wor`/`tri*`/`supply*`/
  `uwire`/`and`/`or`/`not`/`nand`/`nor`/`xor`/`xnor`/`buf*`/`nmos`/`pmos`/`always`/`assign`/`initial`/
  `posedge`/`negedge`/`edge`/`macromodule`/`primitive`/`genvar`/`defparam`/`specparam`) — reserved in
  SV but absent from the `_sv` list — can no longer match as expression primaries under `sv_2017`/
  `sv_2023`. Wider blast radius than `.2` (the list feeds every `non_keyword_identifier` site incl.
  `declaration_identifier`/`callable_identifier`), so it warrants its own corpus/cert verification.
  The `_v2005` list is already complete (full Annex B), so `verilog_2005` already rejects these.

## `.2` Findings (tools-first, 2026-07-04 — implementation + the moved-leak discovery)

**FIX (3 carriers → `non_keyword_identifier`, shape-preserving `{body:X}`):**
- `specparam_identifier := identifier` → `:= non_keyword_identifier` (the `constant_primary` alt that
  matched `integer` first: AST `{kind:"specparam", name:{body:"integer"}}`).
- `genvar_identifier := identifier` → `:= non_keyword_identifier` (the next `constant_primary` alt —
  where the leak would move if only specparam were fixed).
- `hierarchical_identifier`'s trailing name `identifier` → `non_keyword_identifier` (the non-constant
  `primary_sv_2017:4082` carrier: AST `{kind:"hierarchical", name:{body:"integer"}}`).

**MOVED-LEAK RE-PROBE (the completeness oracle, exactly the `.6.11` discipline):**
1. After the fix, `assign w = <kw>;` / `initial x = <kw>;` REJECT the whole `_sv` reserved TYPE family
   (`integer`/`real`/`time`/`realtime`/`reg`/`logic`/`byte`/`int`/`bit`/`shortint`/`longint`/
   `shortreal`/`string`/`chandle`/`event`) under `sv_2017`/`sv_2023`, and the `_v2005` family
   (`integer`/`real`/`time`/`realtime`/`reg`/`event`/`wire`) under `verilog_2005`. SV-0035 CLOSED.
2. `localparam p = integer;` still ACCEPTs — but now via the **LRM-faithful** `constant_param_expression
   := … | data_type | …` route (`:1380`; AST `{kind:"integer_atom"}`), NOT a keyword-as-primary. The
   `+ 1` arithmetic oracle (`localparam p = <kw> + 1;`) confirms the *primary* is closed (REJECTs). And
   `reg q [integer];` under `sv_2017` is the legitimate `associative_dimension := [data_type]` route
   (rejects under `verilog_2005` — SV-0034 already gated it). Both residual accepts are correct syntax.
3. **Discovered (SV-0036, split to `.3`):** under `sv_2017`/`sv_2023`, 35 net/gate/structural Verilog
   keywords (`wire`/`and`/`always`/…) still match as primaries — they are reserved in SV but ABSENT
   from `reserved_non_keyword_identifier_sv` (the `_sv` list only ever covered data-type + control-flow
   words). A *reserved-list-completeness* defect (distinct mechanism, wider blast radius) — not a bare
   carrier — so it is its own leaf, keeping `.2` targeted. `.2` is a strict improvement (net leak
   reduction; the whole `_sv`/`_v2005` reserved family closed).

## Acceptance Checklist (enforced) — `SV-KEYWORD-PRIMARY-FIDELITY.2`

- [x] **REPRODUCE / ISSUE** — HEAD release binary: `assign w = integer;`, `initial x = integer;`,
  `localparam p = integer;` all ACCEPT under `sv_2017`/`sv_2023`/`verilog_2005`; AST-dump proved the
  carriers `{kind:"specparam", name:{body:"integer"}}` (const) and `{kind:"hierarchical",
  name:{body:"integer"}}` (non-const).
- [x] **ROOT CAUSE (WHY + WHERE)** — three name rules defined as bare `identifier` (which matches
  reserved keywords) feed expression primaries: `specparam_identifier` (`grammars/systemverilog.ebnf:5083`),
  `genvar_identifier` (`:2375`) in `constant_primary_sv_2017/_sv_2023`, and `hierarchical_identifier`
  (`:2412`, trailing name) in `primary_sv_2017:4082`. The keyword-excluding `non_keyword_identifier`
  (`:359`, `!reserved_non_keyword_identifier identifier -> {body:$2.body}`) already exists and is
  proven-safe under all profiles (`declaration_identifier`/`callable_identifier` use it).
- [x] **FIX** — declarative grammar fix (fix-hierarchy tier: grammar): re-point the 3 carriers to
  `non_keyword_identifier`. AST-shape-preserving by construction (both `identifier` and
  `non_keyword_identifier` emit `{body:X}`; only reserved keywords flip ACCEPT→REJECT). No new rules
  (census `1465` unchanged).
- [x] **ADDRESSED (verified)** — the whole `_sv` reserved TYPE family now REJECTs in pure
  value-expression-primary positions (`assign w = <kw>;`, `initial x = <kw>;`, `localparam p = <kw> + 1;`)
  under `sv_2017`/`sv_2023`; the `_v2005` family REJECTs under `verilog_2005`. Legal names byte-identical
  (10/10 realistic hierarchical/member/scope/method/genvar/specparam constructs still ACCEPT; the sole
  probe "reject" was a malformed test string — stray `;` after `endgenerate`, isolated + refuted).
- [x] **NO REGRESSION** — canonical SV cert `1343/2/1321/UNKNOWN=20` (seeds 0/7/42, spf=0, byte-identical);
  `verilog_2005_conformance_gate` GREEN (orphans=0, matrix 219 checks/0 mismatches, cert `1117/4/773/340`
  deterministic seeds 0/7/42); `sv_cert_recognized_union_gate` GREEN (canonical UNKNOWN=20, union
  UNKNOWN=1, residual `["context_member_method_call"]`, byte-identical); `ast_shape_contract_gate` 18/18;
  `sv_external_corpus_triage_gate` 14/14 parse_pass / 0 fail; `clippy_on_rust_change` source strict-clean
  (generated-parser stage = pre-existing non-strict debt tolerated by the gate); `--lint-grammar`
  profile_orphans=0; json `9/0` + regex `198/0` `fully_certified=true` (6 fully-certified grammars
  byte-identical — SV-only regen).
- [x] **LOCKSTEP** — SV integration contract release/contract `1.0.161` → `1.0.162` (schema `15`
  unchanged); SV parser book `changelog-index.md` `1.0.162` row; book `parser-families.md` SV-0035
  mention updated to CLOSED + SV-0036 noted; ledger `SV-0035` → `Released`, new `SV-0036` → `Root Caused`;
  live docs (`MEMORY.md`/`CHANGES.md`/`DEVELOPMENT_NOTES.md`/`LIVE_ACHIEVEMENT_STATUS.md`) + this task file.

## `.1` Findings (tools-first, 2026-07-04 — the carrier map + fix architecture)

**Reproduced (HEAD release binary, all profiles ACCEPT = leak):**
- constant expression: `module m; localparam p = integer; endmodule` ACCEPTs under
  `verilog_2005`/`sv_2017`/`sv_2023`; the whole reserved-type-keyword family
  (`integer`/`real`/`time`/`realtime`/`reg`/`wire`/`logic`/`byte`/`int`/`bit`) ACCEPTs under sv_2017.
- non-constant expression: `assign w = integer;`, `initial x = integer;`, `q = integer + 1;` all ACCEPT.

**Root cause (WHY + WHERE) — the leak has MULTIPLE carriers, all the same mechanism** (a name rule
defined as bare `identifier`, which matches reserved keywords, instead of the keyword-excluding
`non_keyword_identifier`):

1. **Constant primary** (`constant_primary_sv_2017`/`_sv_2023`, `grammars/systemverilog.ebnf:1387`/
   `:1423`): the ordered-choice alternatives `specparam_identifier ( … )?` (`:1391`/`:1427`) and
   `genvar_identifier` (`:1393`/`:1429`) — `specparam_identifier := identifier` (`:5083`) and
   `genvar_identifier := identifier` (`:2375`) are BARE. `integer` matches `specparam_identifier`
   first (AST-proven: `{kind:"specparam", name:{body:"integer"}}`); if only that is fixed the leak
   MOVES to `genvar_identifier` (the next alternative — both must be fixed). The sibling alternatives
   already use the keyword-excluding form: `formal_port_identifier`/`enum_identifier :=
   declaration_identifier := non_keyword_identifier`, `ps_parameter_identifier` needs a `class_scope`
   prefix — so they correctly reject `integer`.
2. **Non-constant primary** (`primary_sv_2017`, `:4082`): routes through
   `hierarchical_identifier := ( hierarchical_root_prefix_sv_only )? ( identifier constant_bit_select
   dot )* identifier -> {root:$1, scope:$2, name:$3}` (`:2412`) — BOTH the scope-group `identifier`
   and the trailing `name` `identifier` are bare, so `integer` matches the trailing `identifier` with
   zero scope repetitions.

**Candidate carrier universe** — `grep -nE '^[a-z_]+ *:= *identifier *$'` finds **22** rules defined
as bare `identifier`. Most are declaration-name / structured contexts where a preceding keyword or
structure disambiguates so the keyword-leniency never surfaces as a bare-primary leak (`cell_identifier`,
`library_identifier`, `udp_identifier`, `instance_identifier`, `attr_name`, `bin_identifier`,
`cover_point_identifier`, `cross_identifier`, `constraint_identifier`, `generate_block_identifier`,
`inout/input/output_port_identifier`, `terminal_identifier`, `member_identifier`, `let_identifier`,
`const_identifier`, `index_variable_identifier`, `signal_identifier`). `.2` MUST empirically
determine which are reachable as bare expression name-primaries (the "leak moves" re-probe after
fixing the 3 confirmed carriers is the completeness oracle), rather than assume the 3 known ones are
the whole set. Note the grammar is already INCONSISTENT: `net_identifier`/`variable_identifier :=
declaration_identifier` (keyword-excluding) while `signal_identifier`/`specparam_identifier`/… are bare.

**Fix architecture (staged for `.2`):** route each expression-primary-reachable bare-`identifier`
name rule through `non_keyword_identifier`. This is **AST-shape-preserving by construction**:
`identifier := … -> {body:$1}` emits `{body:X}` and `non_keyword_identifier := !reserved
identifier -> {body:$2.body}` ALSO emits `{body:X}` (`:348`/`:360`), so every `$N` that resolved to
an `identifier` node keeps its exact `{body:X}` shape — only reserved keywords flip ACCEPT→REJECT.
The reserved lists are profile-gated (`reserved_non_keyword_identifier_sv` `@profiles:
["sv_2017","sv_2023"]` at `:386`; `_v2005` `@profiles: ["verilog_2005"]` at `:389`), BOTH include
`integer`/`real`/`time`/`reg`/`wire`/`logic`/`bit`/`int`/`byte`/… — so `non_keyword_identifier`
excludes them under every profile. `declaration_identifier`/`callable_identifier`/`parameter_identifier`
already use `non_keyword_identifier` and work under all profiles, so the mechanism is proven-safe.

**Residual risk `.2` must clear (why this is design-first, not a rushed gate):**
- **Carrier completeness** — `hierarchical_identifier` underpins EVERY name reference; the fix must
  re-probe (`assign w = <kw>;`, `localparam p = <kw>;`, `q = <kw> + 1;`, `reg q [<kw>];`) for the
  whole reserved family after the swap, and chase any moved leak to a further bare-identifier carrier.
- **Corpus over-rejection** — a reserved keyword is never a legal name, so no legal design should
  break; but `.2` MUST prove it against the external corpus (14/14), the realistic corpus, and uvm
  (no NEW parse failures) before claiming closure — over-rejection on a foundational identifier rule
  would be a serious regression.
- **Release/schema** — accept→reject on shipped `sv_2017`/`sv_2023` ⇒ release bump per
  `PGEN_RELEASE_POLICY` + `embedding_api.rs` version consts + SV book changelog lockstep; schema
  unchanged (`{body:X}` preserved).

## Blockers

- none (all analysis in-repo; the fix mechanism `non_keyword_identifier` already exists and is
  proven under all profiles). `.2` is a careful multi-carrier implementation, not a research problem.

## Verification Log

- 2026-07-04 (`.1` DESIGN, `PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0001`, session #29, ZERO code): the
  carrier map + fix architecture established tools-first on the HEAD release binary (SV `1.0.161`):
  the 3 confirmed carriers (`specparam_identifier`/`genvar_identifier`/`hierarchical_identifier`) each
  parse-probed (`localparam p = integer;` / `assign w = integer;` ACCEPT), the AST route AST-dumped
  (`{kind:"specparam", name:{body:"integer"}}`), the 22-rule bare-`identifier` universe enumerated,
  and the shape-invariance of `identifier` ⟷ `non_keyword_identifier` (both `{body:X}`) proven by
  direct grammar read (`:348`/`:360`). No code/grammar/generated change.
- 2026-07-04 (`.2` CODE, `PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0002`, session #29): the 3-carrier fix
  landed + regenerated + re-probed. Leak CLOSED for the reserved family (proven above); all
  no-regression oracles GREEN (canonical `1343/2/1321/20`; v2005 `1117/4/773/340`; conformance
  219/0 + orphans 0; union canonical=20/union=1; ast_shape 18/18; external corpus 14/14; clippy
  source strict-clean; json 9/0 + regex 198/0). Release `1.0.161` → `1.0.162`, schema `15`.
  The moved-leak re-probe surfaced `SV-0036` (net/gate/structural keyword primary leak under the SV
  profiles) → split to `.3`.
