# Task Tree: SV-LRM-SHAPE-FIDELITY

Owner tree for the two remaining **all-profile** grammar-SHAPE over-acceptances that bound the
SystemVerilog / `verilog_2005` strict-subset & LRM-fidelity claim — both are "extraction-era
meta-notation" defects where `grammars/systemverilog.ebnf` accepts a form the IEEE LRM's
*meta-notation* forbids under EVERY profile (`sv_2017`/`sv_2023`/`verilog_2005`):

- **`SV-0024`** — un-braced multi-identifier `port_expression`: `module m (a b);` (two references,
  no comma, no concatenation braces) wrongly ACCEPTS. IEEE 1364-2005 A.1.3 / IEEE 1800 A.1.3:
  `port_expression ::= port_reference | { port_reference { , port_reference } }` — the multi-reference
  form requires LITERAL concatenation braces `{ … }`. The grammar extracted the LRM's *literal* (bold)
  outer braces as BNF meta-repetition, yielding an outer `*` with no `lbrace`/`rbrace` and no separator
  between iterations (same family as the historical "`[ X ]` optional extracted without `?`" class).
- **`SV-0028`** — stray top-level `;`: a bare `;` with no declaration wrongly ACCEPTS. Neither
  IEEE 1800-2017 A.1.2 (`source_text ::= [timeunits_declaration] { description }`; `description` has
  no null alternative) nor IEEE 1364-2005 A.1.2 admits a stray top-level semicolon — an extraction-era
  robustness extension (`source_text_item`'s `semi -> {kind:"semi"}` alternative) with no LRM
  counterpart.

Created 2026-07-04 (session #32) from the standing SV/`verilog_2005` FINALIZATION lane. These are the
last two open all-profile leaks after the `SV-KEYWORD-PRIMARY-FIDELITY` tree closed the
reserved-keyword-as-primary family (`SV-0035`/`SV-0036`/`SV-0037`). They warrant their own focused tree
(cf. the `SV-KEYWORD-PRIMARY-FIDELITY` / `SV-DOLLAR-LRM-FIDELITY` precedent for all-profile SV
correctness families, kept distinct from the `verilog_2005`-only dialect-boundary family
`SV-0025`..`SV-0034`). One concern per leaf, one leaf per commit.

## Acceptance Criteria (tree)

- The over-accepted form REJECTS in every profile where the LRM forbids it, at ALL carriers
  (no sibling-leak — re-probe after each fix, cf. `VERILOG-2005-PROFILE.6.11`'s cascade discipline).
- Legitimate forms parse byte-identically or newly-correctly per the LRM (AST-shape-faithful).
- Full no-regression: canonical/union cert (seeds 0/7/42), `ast_shape_contract`, the external corpus,
  the `verilog_2005` conformance matrix + cert, `--lint-grammar` 0 profile-orphans, the 6
  fully-certified grammars byte-identical, clippy source-clean.
- Release/schema per `PGEN_RELEASE_POLICY`: an accept↔reject behavior change on shipped SV profiles
  → release bump; schema unchanged unless a NEW AST envelope shape is introduced.

## Current Frontier

- ID: `SV-LRM-SHAPE-FIDELITY.1` — Status: `done` (2026-07-04, session #32,
  `PGEN-SV-LRM-SHAPE-FIDELITY-0001`, CODE leaf — grammar-only) — closed `SV-0024`. Restored the LRM shape
  `lbrace port_reference ( comma port_reference )* rbrace -> {kind:"list", body:[$2, $3::2*]}` for
  `port_expression`'s multi-reference alternative (`grammars/systemverilog.ebnf:4092`). SV release
  `1.0.164` → `1.0.165` (schema 15 unchanged). `module m (a b);` REJECTs under `verilog_2005` / ACCEPTs
  under `sv_2017`/`sv_2023` (interface port); `module m ({a, b});` ACCEPTs under every profile. All gates
  GREEN seeds 0/7/42. See "`.1` Findings" + "Acceptance Checklist — `.1`" below.
- ID: `SV-LRM-SHAPE-FIDELITY.2` — Status: `pending` — CODE leaf: close `SV-0028` (remove / strictly
  justify `source_text_item`'s `semi` alternative). Deliberately NOT bundled with `.1` (distinct
  carrier, distinct corpus/cert adjudication — one concern per commit).

## `.1` Findings (tools-first, 2026-07-04 session #32 — SV-0024 port_expression)

**REPRODUCE (HEAD `.3.2` release binary `1.0.164`):**
```
printf 'module m (a b); endmodule\n' | parseability_probe --parse systemverilog /dev/stdin --profile verilog_2005  → ACCEPT (rc0)  [WRONG]
                                                                                                 --profile sv_2017  → ACCEPT (rc0)
```
`a b` = two whitespace-separated references, NO comma, NO braces — invalid `port_expression` in both
IEEE 1364-2005 and IEEE 1800.

**ROOT CAUSE (WHY + WHERE, AST-dump-pinned):** the accepting path differs by profile —
- **verilog_2005 (non-ANSI):** `--parse-dump-ast-pretty … --profile verilog_2005` shows the port as a
  `port_expression` node `{kind:"list", body:[{name:a,select},{name:b,select}]}`. WHERE:
  `port_expression` second alternative (`grammars/systemverilog.ebnf:4092`)
  `( port_reference ( comma port_reference )* )*` — an outer `*`-repeated group with no `lbrace`/`rbrace`
  and no inter-iteration separator, so `a b` matches as iteration-1 `a` + iteration-2 `b`. The LRM's
  LITERAL concatenation braces were extracted as meta-repetition (it also admits the empty form).
- **sv_2017/sv_2023 (ANSI):** `--parse-dump-ast-pretty … --profile sv_2017` shows the port as
  `{kind:"net_or_interface", header:{kind:"named", name:a}, name:b}` — i.e. `a` parsed as an
  `interface_port_header` (`:2691`, `interface_identifier`), `b` as the `port_identifier`. This is a
  DIFFERENT path (`ansi_port_declaration`) exposed after the `SV-0037` fix (`.3.1`, `1.0.163`) store-gated
  `net_port_type`. Per IEEE 1800 A.1.3 `interface_port_header port_identifier` IS a valid ANSI interface
  port (the "`a` is not a declared interface" check is elaboration, not parse), so `module m (a b)` is
  legal IEEE 1800 syntax — the defect is `verilog_2005`-effective only.

**SPEC RE-ADJUDICATION (expected from the LRM, independent of the stale ledger repro).** The ledger row
(2026-07-02) pre-dates the `SV-0037` fix and asserted "all profiles show port_expression list". Post-
`.3.1` the sv_2017 path is now the legitimate ANSI interface path. Correct spec-faithful behavior:
| input | verilog_2005 | sv_2017/sv_2023 |
| --- | --- | --- |
| `module m (a b);` | **REJECT** (no interfaces; only `port_expression`, which needs braces) | ACCEPT (interface port — valid 1800 syntax) |
| `module m (a, b);` | ACCEPT (two single ports) | ACCEPT |
| `module m ({a, b});` | ACCEPT (one braced-concat port) | ACCEPT |
| `module m (a);` | ACCEPT | ACCEPT |

**Pre-fix measured (HEAD `1.0.164`):** `{a, b}` REJECTS under BOTH profiles (the braced-concat port is
NOT currently supported — the un-braced list alt never matched `{`), `a, b` ACCEPTS, `a` ACCEPTS. So the
un-braced alt only ever admitted invalid forms; the fix simultaneously ADDS the missing correct
braced-concat form and REMOVES the invalid un-braced form — a net LRM-fidelity gain.

**FIX (declarative — grammar-level, restores the exact LRM shape):**
```
port_expression := port_reference                                        -> {kind: "single", body: $1}
                | lbrace port_reference ( comma port_reference )* rbrace  -> {kind: "list",   body: [$2, $3::2*]}
```
Mirrors the established `lbrace X ( comma X )* rbrace -> [$_, $_::2*]` concatenation-list idiom already
used by `assignment_pattern` (`:604`). No new rule (census unchanged). `port_expression` is referenced
only at `port` (`:4066` named-port, `:4068` expression-port) — contained blast radius. NO
`ast_shape_contract` sample references `port_expression` (only `calibration_history` prose), so the
return-annotation reshape cannot break that gate; a sample locking the now-reachable braced-concat shape
is added in lockstep.

## Acceptance Checklist (`.1`, enforced)

- [x] **REPRODUCE / ISSUE** — `module m (a b);` ACCEPTS under verilog_2005 (and sv_2017/sv_2023) at
  release `1.0.164` (`parseability_probe --parse`, rc0); should REJECT under verilog_2005.
- [x] **ROOT CAUSE (WHY + WHERE)** — `port_expression` alt 2 (`grammars/systemverilog.ebnf:4092`)
  `( port_reference ( comma port_reference )* )*` — un-braced `*`-repeated group, no `lbrace`/`rbrace`/
  separator (AST dump `{kind:"list"}` under verilog_2005). LRM: `port_expression ::= port_reference |
  { port_reference { , port_reference } }` (literal braces). sv_2017 `a b` is a distinct, legitimate
  `interface_port_header` ANSI path (AST dump `{kind:"net_or_interface"}`) — correctly stays ACCEPT.
- [x] **FIX** — restored `lbrace port_reference ( comma port_reference )* rbrace -> {kind:"list", body:[$2, $3::2*]}`
  at `grammars/systemverilog.ebnf:4092` (declarative/grammar tier; the `assignment_pattern:604` idiom).
  ZERO new rules (`--lint-grammar` census `1465` unchanged, non_terminating=0, ordered_choice_shadowing=0,
  profile_orphans=0). SV parser regenerated (`focus_systemverilog`) + both binaries rebuilt.
- [x] **ADDRESSED (verified, release binary post-regen)** — `module m (a b)` verilog_2005 ACCEPT→**REJECT**;
  sv_2017/sv_2023 **ACCEPT** (interface port, AST `{kind:"net_or_interface"}`); `{a, b}` / `{a, b, c}`
  REJECT→**ACCEPT** (all profiles, `port_expression {kind:"list", body:[{name,select}…]}`); `a, b` / `a` /
  named `.p(a)` / ANSI ports unchanged ACCEPT. 21/21 profile×case matrix as designed.
- [x] **NO REGRESSION (seeds 0/7/42)** — `sv_cert_recognized_union_gate` GREEN (canonical `1343/2/1321/UNKNOWN=20`
  byte-identical, union `UNKNOWN=1` residual `context_member_method_call`); direct canonical cert seed-0
  `1343/2/1321/20` spf=0; `verilog_2005_conformance_gate` GREEN (lint orphans 0, matrix 225/0 — the 6 new
  cells for the 2 new lock files proven via scoped matrix replay 6/6 + full-gate re-run, cert `1117/4/773/340`
  byte-identical, `port_expression` still witnessed via braces); `ast_shape_contract_gate` 18/18;
  `sv_external_corpus_triage_gate` 14/14 (parse_pass 14 / fail 0); the 6 fully-certified grammars
  byte-identical (json `9/0`, regex `198/0` spot-checked `fully_certified=true`); clippy source strict-clean
  (grammar-only — no hand-written `rust/src` change).
- [x] **LOCKSTEP** — ledger `SV-0024` → `Released` (`1.0.165`); SV integration contract release/contract
  `1.0.164`→`1.0.165` (schema 15) + § "Release 1.0.165" + updated open-waiver line; SV parser book
  `changelog-index.md` `1.0.165` entry; top book `parser-families.md` (SV-0024→FIXED, open count three→two);
  conformance corpus + contract (2 new locks `reject/port_bare_multi_id.sv` + `accept/port_concat.v`, updated
  `interconnect_port` note); `docs/TASK_TREE.md` (new tree row + VERILOG-2005-PROFILE alternates note);
  LIVE_ACHIEVEMENT_STATUS (dialect block + session-#32 tracker note); CHANGES / DEVELOPMENT_NOTES / MEMORY.
  No ast_shape_contract sample references `port_expression` (gate GREEN as-is); the now-reachable braced-concat
  shape is behavior-locked by `accept/port_concat.v` + documented in the ledger/contract/book. Release bump
  (not SV-profile-byte-invariant: `{a,b}` REJECT→ACCEPT on sv_2017/sv_2023); schema 15 (no new envelope shape).

## Decisions

- `2026-07-04`: Own `SV-0024` + `SV-0028` in a dedicated all-profile tree (not `VERILOG-2005-PROFILE.6.x`),
  following the director-approved `SV-KEYWORD-PRIMARY-FIDELITY` split precedent — they are all-profile
  correctness defects, distinct from the `verilog_2005`-only dialect-boundary ratchet.
- `2026-07-04`: `SV-0024` scope is the non-ANSI `port_expression` shape ONLY. The sv_2017/sv_2023
  acceptance of `module m (a b)` is the legitimate `interface_port_header` ANSI interface-port path
  (valid IEEE 1800 syntax) — NOT a defect, and NOT gated here. Spec-faithful outcome: `a b` REJECTs under
  verilog_2005, ACCEPTs under the SV profiles.

## Open Questions

- Release/schema: the fix changes shipped-profile behavior (`{a,b}` REJECT→ACCEPT on sv_2017/sv_2023;
  `a b` ACCEPT→REJECT on verilog_2005) → release bump warranted; schema unchanged unless the braced-concat
  `{kind:"list"}` body is treated as a new envelope shape (it is not — still typed-JSON schema 15).
  Resolved at implementation from the measured AST.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-04` | `SV-LRM-SHAPE-FIDELITY.1` | reproduce + AST-dump root-cause (pre-code) | done |
| `2026-07-04` | `SV-LRM-SHAPE-FIDELITY.1` | union cert (canonical `1343/2/1321/20`, union `1/1340`) seeds 0/7/42 | GREEN |
| `2026-07-04` | `SV-LRM-SHAPE-FIDELITY.1` | verilog_2005 conformance (orphans 0, matrix 225/0, cert `1117/4/773/340`) | GREEN |
| `2026-07-04` | `SV-LRM-SHAPE-FIDELITY.1` | ast_shape_contract 18/18; external corpus 14/14; json 9/0 + regex 198/0 byte-identical | GREEN |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SV-LRM-SHAPE-FIDELITY.1` | `PGEN-SV-LRM-SHAPE-FIDELITY-0001` (SV-LRM-SHAPE-FIDELITY.1) | `SV-0024` CLOSED; SV `1.0.164`→`1.0.165`, schema 15; grammar-only |

## Changelog

- `2026-07-04`: Created task tree; owns `SV-0024` (leaf `.1`) + `SV-0028` (leaf `.2`, pending). `.1`
  reproduce + root cause pinned tools-first (pre-code).
- `2026-07-04`: `.1` DONE — `SV-0024` closed (restore LRM concatenation-brace `port_expression` shape); all
  no-regression gates GREEN seeds 0/7/42; full lockstep; committed as `PGEN-SV-LRM-SHAPE-FIDELITY-0001`.
