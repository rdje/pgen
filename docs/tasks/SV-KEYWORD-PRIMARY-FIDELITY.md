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
- ID: `SV-KEYWORD-PRIMARY-FIDELITY.3` — Status: `split` (2026-07-04, session #29,
  `PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0003`, tools-first ATTEMPT → BLOCKED, NO code shipped): implementing
  the reserved-list extension (adding the `_v2005 \ _sv` delta of 91 words to
  `reserved_non_keyword_identifier_sv`) closes the SV-0036 primary leak but **regresses implicit-type
  ANSI ports** (`module m(input a);` / `module m(input [7:0] a);` flip ACCEPT→REJECT). Root-caused
  tools-first (stash + rebuild + trace): a **pre-existing latent bug** — `net_port_type_sv_2017` alt 1
  (`grammars/systemverilog.ebnf:3474`, `net_type_identifier -> {kind:"identifier"}`) uses the *ungated*
  `net_type_identifier := declaration_identifier` (`:3517`) which greedily eats the port NAME as a
  net-type; in `.2` this was MASKED because the reserved-list omitted `input`/`output`/`inout`, so the
  parser could fall back to a (wrong) parse treating the direction word as an identifier. Reserving the
  direction keywords removes that mask and exposes the greediness. Ledgered `SV-0037`. So SV-0036 has a
  PREREQUISITE. Split into `.3.1` (fix SV-0037 first) → `.3.2` (then reserve the keywords safely).
- ID: `SV-KEYWORD-PRIMARY-FIDELITY.3.1` — Status: `done` (2026-07-04, session #30,
  `PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0004`, CODE leaf, the PREREQUISITE): fixed `SV-0037` — routed
  `net_port_type_sv_2017` alt 1 (`grammars/systemverilog.ebnf:3474`, `net_type_identifier`) and
  `net_port_type_sv_2023` alt 1 (`:3479`, `nettype_identifier`) to the store-gated
  `checked_nettype_identifier` (`:3537`, the SV-PARSE-STRICT.2 declared-nettype gate) so an undeclared
  identifier can no longer match as a net-type and eat the port name. Tools-first trace confirmed the
  precise mechanism: `net_port_type_sv_2017` runs under the GLOBAL DEFAULT `branch_policy=longest_match`
  (not ordered-choice), so the ungated `net_type_identifier` matching a bare ` a` (2 bytes) out-consumed
  alt 0's empty implicit match (0 bytes) and ate the port name. After the fix the implicit-type path
  (alt 0) correctly wins and the name binds: `module m(input a);` flips `{kind:"nonansi"}` →
  `{kind:"ansi"}` under sv_2017/sv_2023/verilog_2005. SV release `1.0.162` → `1.0.163` (schema `15`
  unchanged — both `ansi`/`nonansi` shapes pre-exist; a mis-parse *correction*, not a new shape).
  See "`.3.1` Findings" + "Acceptance Checklist — `.3.1`" below. `SV-0037` CLOSED (`Released`); `.3.2`
  UNBLOCKED.
- ID: `SV-KEYWORD-PRIMARY-FIDELITY.3.2` — Status: `done` (2026-07-04, session #31,
  `PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0005`, CODE leaf — grammar + generator prerequisite): `SV-0036`
  CLOSED. SV release `1.0.163` → `1.0.164` (schema `15` unchanged). See "`.3.2` Findings" + "Acceptance
  Checklist — `.3.2`" below. This is the LAST leaf of the tree; the all-profile reserved-keyword-as-primary
  family (`SV-0035`+`SV-0036`) is fully closed. Closed `SV-0036` —
  extend `reserved_non_keyword_identifier_sv` (`:386`) by the `_v2005 \ _sv` delta (91 words: the full
  IEEE 1364-2005 Annex B net-type/gate/structural/config keywords SV also reserves — `always`/`and`/
  `assign`/`wire`/`config`/`input`/`output`/… — making `_sv` ⊇ `_v2005`, LRM-complete) so they can no
  longer match as expression primaries under `sv_2017`/`sv_2023`. Only safe AFTER `.3.1` removes the
  net-type greediness. All 91 words empirically leak as primaries on the `.3.1` baseline (verified,
  91/91 → REJECT after the reservation). The `\b` word-boundary makes alternation order irrelevant and
  protects keyword-prefixed identifiers (`input_data`/`wire_en`/`and_gate` — verified still ACCEPT).
  The `_v2005` list is already complete, so `verilog_2005` already rejects these.
  **Closing `.3.2` required a co-landed GENERATOR prerequisite (tools-first discovery — see "`.3.2`
  Findings" below): the SV stimuli generator out-generated its own parser** — a seed-0 diverse sample
  emitted `import or::…` (`or` is one of the 91 newly-reserved words) which the parser now correctly
  REJECTS, bumping the canonical-cert `sample_parse_failures` 0→1 (the `sv_cert_recognized_union_gate`
  hard-asserts `spf=0`). Root-caused to the RTL-FE-CLOSURE.6 keyword-exclusion collector only
  recognizing the `!kw_A … !kw_Z TERM` FIXED-literal shape, never SV's `!reserved_non_keyword_identifier
  identifier` (regex whole-word-list) shape → the generator never excluded reserved words there. Fix
  (general, parser-agnostic): extend the exclusion collector to also read a negative-lookahead target
  that resolves to a whole-word REGEX alternation `trivia? /(?:w1|…)\b/` (incl. via an `Or` of
  profile-gated sub-rules). The existing RNG-neutral `_`-prefix repair then makes `or`→`_or` in place
  (no RNG cascade), so `spf` returns to 0 and all other samples stay byte-identical.

## `.3.2` Findings (tools-first, 2026-07-04 session #31 — the reservation + the generator prerequisite)

**REPRODUCE (HEAD `.3.1` release binary, `sv_2017`+`sv_2023`):** the 91 net/gate/structural keywords
(`wire`/`and`/`always`/`assign`/`initial`/`posedge`/`nand`/`supply0`/…) ACCEPT as bare expression
primaries (`assign w = <kw>;`) — the SV-0036 leak. Keyword-prefixed identifiers (`wire_en`/`input_data`/
`and_gate`/`always_on`) ACCEPT (the `\b` guard — must stay accepting).

**FIX 1 — the reservation (grammar):** appended the `_v2005 \ _sv` delta (exactly 91 words, computed
tools-first with `comm`) to `reserved_non_keyword_identifier_sv` (`grammars/systemverilog.ebnf:386`),
preserving the proven-good `_v2005`-native ordering, so `_sv` (81→172 words) ⊇ `_v2005` (verified: 0
`_v2005` words missing from `_sv`). After regen: all 91 REJECT as primaries under `sv_2017`/`sv_2023`
(arith/initial/localparam/if-cond positions — moved-leak re-probe clean); keyword-prefixed identifiers
still ACCEPT; SV-0035 type family still REJECTs (10/10); the SV-0037 implicit-ANSI port stays
`{kind:"ansi"}` (all profiles); a realistic design ACCEPTs (no over-rejection).

**THE FISHY RESULT (stopped + root-caused per the alert doctrine):** with the reservation alone the
canonical cert stayed `total=1343 proof=2 witness=1321 UNKNOWN=20` at all seeds — BUT
`sample_parse_failures` went **0→1 at seed 0** (0 at seeds 7/42). A generator emitting a parser-rejected
sample is a gen⟷parse duality break; the `sv_cert_recognized_union_gate` HARD-asserts `canonical spf==0`
(`rust/scripts/sv_cert_recognized_union_gate.sh:241`), so this would fail the gate.

**ROOT CAUSE (WHY + WHERE, tools-first):** the failing seed-0 diverse sample (dumped by the cert's own
`SAMPLE-PARSE FAILURES` block, `rust/src/main.rs:3018`) is a top-level `import or::ihoiK;` (comments
interspersed) — `or` is one of the 91 newly-reserved words. The generator emitted `or` as a package
identifier; the parser now correctly REJECTS it (`import or::x;`→REJECT, `import foo::x;`→ACCEPT,
verified). WHERE: the RTL-FE-CLOSURE.6 keyword-exclusion collector
`collect_identifier_keyword_exclusions` (`rust/src/ast_pipeline/stimuli_generator.rs:9471`) only
recognizes a leading `!X` whose target `X` is a FIXED keyword-literal rule (`kw_if := /if\b/`, via
`rule_fixed_keyword_literal`). SV's `non_keyword_identifier := !reserved_non_keyword_identifier
identifier` (`:359`) targets a rule whose body is an `Or` of two REGEX whole-word-list rules
(`reserved_non_keyword_identifier_sv | _v2005`), so the collector extracted NOTHING → the generator was
never keyword-aware for SV's identifier exclusion (a latent gap, masked until a word the generator emits
— `or` — became reserved). The `.3.2` reservation exposed it.

**FIX 2 — the generator prerequisite (engine, general/parser-agnostic):** extend the exclusion
collector to ALSO treat a negative-lookahead target that resolves to a whole-word REGEX alternation
`trivia? /(?:w1|…)\b/` (directly, or via an `Or` of such profile-gated sub-rules) as an exclusion set —
returning every reserved whole-word spelling. Conservative (returns nothing unless EVERY branch is a
pure identifier-shaped word list, so no other rule shape is ever misread; the words are precomputed once
per generator from the already-profile-filtered tree). The EXISTING RNG-neutral `_`-prefix repair
(`:9716`) then repairs the colliding identifier `or`→`_or` IN PLACE, consuming no RNG, so every other
sample stays byte-identical and the cert stream is not perturbed. **Safety invariant:** the generator
excludes a word in an identifier position ONLY where a `!<rule-matching-word> identifier` sequence
exists — exactly where the parser rejects it — so generator and parser agree by construction, and the 6
fully-certified grammars (all `spf=0` today ⇒ no current collision ⇒ repair never fires) stay
byte-identical. This honors RTL-FE-CLOSURE.6's design ("the closed loop must not out-generate its own
parser"), extended from the `!kw_A…!kw_Z` shape to the `!reserved-regex` shape.

## Acceptance Checklist (enforced) — `SV-KEYWORD-PRIMARY-FIDELITY.3.2`

- [x] **REPRODUCE / ISSUE** — HEAD `.3.1` release binary: `assign w = <kw>;` for the 91 net/gate/structural
  keywords (`wire`/`and`/`always`/`assign`/`initial`/`posedge`/`nand`/`nor`/`supply0`/…) ACCEPTs as a bare
  expression primary under `sv_2017`/`sv_2023` (the SV-0036 leak); keyword-prefixed identifiers (`wire_en`/
  `input_data`/`and_gate`/`always_on`) ACCEPT (the `\b` guard — must stay accepting).
- [x] **ROOT CAUSE (WHY + WHERE)** — (1) `reserved_non_keyword_identifier_sv` (`grammars/systemverilog.ebnf:386`)
  omitted the net-type/gate/structural Verilog keywords SV also reserves, so `non_keyword_identifier` (`:359`)
  let them through as primaries. (2) The spf regression: the cert's own `SAMPLE-PARSE FAILURES` dump
  (`rust/src/main.rs:3018`) named the seed-0 diverse sample `import or::ihoiK;` (`or` newly reserved) which the
  parser now correctly REJECTS → canonical `sample_parse_failures` 0→1; root-caused to `collect_identifier_keyword_exclusions`
  (`rust/src/ast_pipeline/stimuli_generator.rs:9471`) recognizing only the fixed-literal `!kw_A…!kw_Z` shape and
  not SV's `!reserved_non_keyword_identifier identifier` regex-word-list shape (the target rule's `Or` body has a
  profile-pruned `_v2005` branch — verified via a temporary env-gated map dump: `reserved_non_keyword_identifier`
  resolved to 172 words including `or` only after the `Or`-leniency fix for the pruned branch).
- [x] **FIX** — (grammar) append the `_v2005 \ _sv` delta (91 words) to `reserved_non_keyword_identifier_sv`
  (fix-hierarchy tier: grammar; `_sv` 81→172, `_sv` ⊇ `_v2005`); (engine prerequisite) extend the RTL-FE-CLOSURE.6
  exclusion collector to resolve a whole-word REGEX alternation (incl. via an `Or` of profile-gated sub-rules), so
  the existing RNG-neutral `_`-prefix repair handles SV. General/parser-agnostic; ZERO new grammar rules.
- [x] **ADDRESSED (verified)** — all 91 delta keywords REJECT as primaries under `sv_2017`/`sv_2023`
  (arith/initial/localparam/if-cond positions — moved-leak re-probe clean, 0 residual carriers); keyword-prefixed
  identifiers still ACCEPT (0 false rejects); SV-0035 type family still REJECTs (10/10); SV-0037 implicit-ANSI port
  stays `{kind:"ansi"}` (all profiles); realistic design ACCEPTs; canonical cert `sample_parse_failures` 1→0 at seed 0.
- [x] **NO REGRESSION** — seeds 0/7/42: canonical SV cert `1343/2/1321/UNKNOWN=20` spf=0 byte-identical;
  `sv_cert_recognized_union_gate` GREEN (canonical UNKNOWN=20, union UNKNOWN=1, residual `context_member_method_call`);
  `verilog_2005_conformance_gate` GREEN (orphans 0, matrix 219/0, cert `1117/4/773/340` — byte-identical, `_sv` change
  inert under v2005); `ast_shape_contract_gate` 18/18; `sv_external_corpus_triage_gate` 14/14 parse_pass / 0 fail;
  `--lint-grammar` 1465 rules / profile_orphans=0 / always_matches_shadowing=8; the 6 fully-certified grammars
  byte-identical (json `9/0`, regex `198/0`, vhdl `216/0`, rtl_const_expr `48/0`, rtl_frontend `168/0`, svpp `74/0`
  — the generator change measured byte-identical against a git-stash baseline); `clippy_on_rust_change` source strict-clean.
- [x] **LOCKSTEP** — ledger `SV-0036` → `Released` (fixed `1.0.164`); SV integration contract release/contract
  `1.0.163` → `1.0.164` (schema 15) + § "Release 1.0.164"; SV parser book `changelog-index.md` `1.0.164` entry;
  top book `parser-families.md` SV-0036 → FIXED; `stimuli-and-quality.md` Mechanism 5 extended to the
  `!reserved-regex` shape; `docs/TASK_TREE.md` frontier; live docs (`MEMORY.md`/`CHANGES.md`/`DEVELOPMENT_NOTES.md`/
  `LIVE_ACHIEVEMENT_STATUS.md`) + this task file (`.3.2` done + checklist).

## `.3` Findings (tools-first, 2026-07-04 — the blocked attempt + the SV-0037 discovery)

**ATTEMPT:** added the 91-word `_v2005 \ _sv` delta to `reserved_non_keyword_identifier_sv`, regenerated,
rebuilt. Result: all 91 residual keywords correctly REJECT as primaries (SV-0036 leak closed), AND
keyword-prefixed identifiers (`input_data`/`wire_en`/`and_gate`) still ACCEPT (the `\b` works) — BUT
`module m(input a);` and `module m(input [7:0] a);` flipped ACCEPT→REJECT.

**ROOT CAUSE of the regression (SV-0037, pre-existing, tools-first):** stash + rebuild proved `.2`
ACCEPTED `module m(input a);`; the `.2` trace showed it only accepted because `port_identifier` matched
`input` (9→14) as an identifier in a fallback parse — the "correct" parse (direction=`input`,
implicit-type, name=`a`) was ALREADY broken because `net_port_type_sv_2017` alt 1
(`net_type_identifier := declaration_identifier`, bare) greedily eats `a` as a net-type (longest-match
across the 3 alts), leaving nothing for `port_identifier` at the `)`. Reserving `input`/`output`/`inout`
removed the fallback mask and exposed the greediness. This is `net_port_type`'s ungated
`net_type_identifier` — a latent grammar bug distinct from the reserved-list-completeness of SV-0036.

**DECISION:** do NOT ship `.3` with the port regression (correctness before speed). SV-0036's fix has a
prerequisite; split into `.3.1` (fix SV-0037) → `.3.2` (reserve keywords). No code shipped in `.3`.

## `.3.1` Findings (tools-first, 2026-07-04 — the SV-0037 fix + the precise longest_match mechanism)

**REPRODUCED (HEAD `.2` release binary, all profiles):** `module m(input a);` ACCEPTs but the AST-dump is
`{kind:"nonansi"}` with a port-expression list `[input, a]` — mis-parsed as a NON-ANSI port list, NOT the
correct ANSI port (direction=`input`, implicit type, name=`a`). (`--parse-dump-ast-pretty`.)

**ROOT CAUSE sharpened by trace (`--trace-rules ansi_port_declaration,net_port_type_sv_2017,net_type_identifier_sv_2017,port_identifier`
at `PGEN_TRACE_VERBOSITY=debug`):** the trace line
`Rule 'net_port_type_sv_2017' selected branch 2/3 consuming 2 chars (… branch_policy=longest_match)` +
`Rule 'net_type_identifier' successfully parsed from 14 to 16 (consumed 2 bytes: ' a')` proves the exact
mechanism — `net_port_type_sv_2017` runs under the **global default `longest_match`** branch policy (NOT
ordered-choice; comments `grammars/systemverilog.ebnf:484`,`:5604`). So alt 0 `( net_type )?
data_type_or_implicit` matches EMPTY (implicit, 0 bytes) but alt 1's ungated `net_type_identifier`
(`:3517`, bare `declaration_identifier`) matches ` a` (2 bytes) and, being longer, WINS — eating the port
NAME. With no name left, the ANSI port fails and the module backtracks to the wrong `nonansi` branch.

**FIX (declarative grammar, store-consulting — fix-hierarchy tier: grammar):** routed both carriers to the
store-gated `checked_nettype_identifier` (`:3537`, `@predicate fact_attribute_equals(type_name, $body,
declaration_family, nettype)`, profile-agnostic). An undeclared identifier now fails alt 1, so alt 0
(implicit) wins and the name binds. A DECLARED nettype still routes through alt 0's
`known_unscoped_data_type` (declared nettypes emit a `type_name` fact), so the legitimate nettype-port
shape is preserved (`myNet a` stays `{kind:"typed", data_type:{known_unscoped_data_type}}`, tie-dominated
by alt 0 under longest_match). ZERO new rules (census `1465` unchanged); `net_type_identifier`/
`nettype_identifier` stay referenced by the nettype-DECLARATION rules (`:3510`/`:3559`), so no orphan.
AST shape `{kind:"identifier", name:$1}` on alt 1 preserved (`{body:X}` both sides). This honors
`feedback_grammar_rules_must_consult_store` — a rule claiming a bare identifier is category `nettype`
MUST consult the store.

## Acceptance Checklist (enforced) — `SV-KEYWORD-PRIMARY-FIDELITY.3.1`

- [x] **REPRODUCE / ISSUE** — HEAD `.2` release binary: `printf 'module m(input a); endmodule\n' | parseability_probe
  --parse-dump-ast-pretty systemverilog … --profile sv_2017` → ACCEPTS but AST `{kind:"nonansi"}` (mis-parse;
  `input`/`a` consumed as a non-ANSI port-expression list) instead of the correct ANSI port.
- [x] **ROOT CAUSE (WHY + WHERE)** — `--trace-rules` at `debug`: `net_port_type_sv_2017` (`grammars/systemverilog.ebnf:3474`)
  under the global default `branch_policy=longest_match` selects alt 1 (`net_type_identifier` → ungated
  `declaration_identifier`, `:3517`) because it consumes ` a` (2 bytes) vs alt 0's empty implicit match (0 bytes),
  eating the port name → ANSI fails → `nonansi` fallback. sv_2023 twin: `:3479` (`nettype_identifier`, `:3567`).
- [x] **FIX** — declarative grammar (tier: grammar): re-point both alt-1 carriers to the store-gated
  `checked_nettype_identifier` (`:3537`). Store-consulting, AST-shape-preserving, ZERO new rules.
- [x] **ADDRESSED (verified)** — `module m(input a);` flips `{kind:"nonansi"}` → `{kind:"ansi"}` (direction=input,
  name=a) under sv_2017 / sv_2023 / verilog_2005; 14-case port harness OK on sv_2017 + sv_2023 (implicit A1–A5,
  ranged B1–B2, typed C1–C3, declared-nettype D1–D2 shape-preserved, typedef E1, nonansi F1 preserved); v2005
  implicit/ranged/wire → ansi, nonansi preserved.
- [x] **NO REGRESSION** — canonical SV cert `1343/2/1321/UNKNOWN=20` (seeds 0/7/42, spf=0, byte-identical);
  `ast_shape_contract_gate` 18/18; `sv_cert_recognized_union_gate` GREEN (canonical=20, union=1, residual
  `context_member_method_call`, seeds 0/7/42); `verilog_2005_conformance_gate` GREEN (orphans 0, matrix 219/0, cert
  `1117/4/773/340` seeds 0/7/42); `sv_external_corpus_triage_gate` 14/14 parse_pass / 0 fail; `--lint-grammar`
  1465 rules, profile_orphans=0, always_matches_shadowing=8 (byte-identical to HEAD baseline); json `9/0` + regex
  `198/0` `fully_certified=true` (fully-certified-6 byte-identical — SV-only regen).
- [x] **LOCKSTEP** — SV integration contract release/contract `1.0.162` → `1.0.163` (schema `15` unchanged) + §
  "Release 1.0.163" + last-updated; SV parser book `changelog-index.md` `1.0.163` entry; top book `parser-families.md`
  SV-0037 fixed + SV-0036 prerequisite cleared; ledger `SV-0037` → `Released` + `SV-0036` UNBLOCKED; live docs
  (`MEMORY.md`/`CHANGES.md`/`DEVELOPMENT_NOTES.md`/`LIVE_ACHIEVEMENT_STATUS.md`) + this task file.

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
- 2026-07-04 (`.3` ATTEMPT → BLOCKED, `PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0003`, session #29, NO code
  shipped): added the 91-word `_v2005 \ _sv` delta to `reserved_non_keyword_identifier_sv`, regenerated,
  rebuilt. All 91 keywords REJECT as primaries (leak closed) + keyword-prefixed identifiers still
  ACCEPT — BUT `module m(input a);` / `module m(input [7:0] a);` regressed ACCEPT→REJECT. Tools-first
  root cause (git stash + rebuild `.2` baseline + `--trace-rules ansi_port_declaration`): a PRE-EXISTING
  latent bug — `net_port_type_sv_2017` alt 1's ungated `net_type_identifier := declaration_identifier`
  (`:3517`) greedily eats the port name; masked in `.2` by the reserved-list omitting the direction
  keywords. Ledgered `SV-0037`. REVERTED the grammar change (working tree back to clean `.2`); split
  `.3` → `.3.1` (fix SV-0037) → `.3.2` (SV-0036 reservation, blocked on `.3.1`). NO regression shipped.
- 2026-07-04 (`.3.1` CODE, `PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0004`, session #30): `SV-0037` fixed +
  regenerated + re-verified. Tools-first trace named the exact mechanism (`net_port_type_sv_2017`
  `branch_policy=longest_match` → alt 1's ungated `net_type_identifier` out-consumes alt 0's empty
  implicit match and eats the port name). Both alt-1 carriers (`:3474` sv_2017, `:3479` sv_2023) routed to
  the store-gated `checked_nettype_identifier` (`:3537`). ADDRESSED: `module m(input a);`
  `{kind:"nonansi"}` → `{kind:"ansi"}` under all 3 profiles (14-case harness OK sv_2017/sv_2023;
  v2005 implicit/ranged/wire ansi, nonansi preserved; declared-nettype port shape preserved). NO
  REGRESSION (all measured, seeds 0/7/42): canonical cert `1343/2/1321/20` byte-identical; ast_shape
  18/18; union canonical=20/union=1; v2005 conformance orphans 0 / matrix 219/0 / cert `1117/4/773/340`;
  external corpus 14/14; lint 1465 rules profile_orphans=0 always_matches_shadowing=8 byte-identical;
  json 9/0 + regex 198/0 fully_certified. Release `1.0.162` → `1.0.163`, schema `15`. `SV-0037` CLOSED
  (`Released`); `SV-0036`/`.3.2` UNBLOCKED.
