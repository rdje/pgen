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
- ID: `SV-KEYWORD-PRIMARY-FIDELITY.2` — proposed (CODE leaf): route the expression-primary-reachable
  bare-`identifier` name rules through the existing keyword-excluding `non_keyword_identifier`
  (`grammars/systemverilog.ebnf:359`, `!reserved_non_keyword_identifier identifier -> {body:$2.body}`),
  carrier-complete per the `.1` map, with the "re-probe after each fix to catch a moved leak"
  discipline; regen + full corpus/cert/shape verification; release bump; ledger `SV-0035` → `Released`.

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
