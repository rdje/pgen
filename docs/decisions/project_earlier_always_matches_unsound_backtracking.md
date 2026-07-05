---
name: project-earlier-always-matches-unsound-backtracking
description: The A2 grammar-linter check `EarlierAlwaysMatches` (always-succeeds ordered-choice shadowing) is UNSOUND for PGEN's BACKTRACKING engine — it declares a proven-LIVE grammar fragment DEAD, violating the certifying-linter "never declares a live fragment dead" soundness contract. Its premise ("PEG commits to the earlier alternative") is false for PGEN, which backtracks. Belongs with the deliberately-excluded unsound heuristics; its A2.1 hard-gate promotion is RETIRED.
metadata:
  node_type: memory
  type: project
  created: 2026-07-05
---

**Finding (2026-07-05, session #37, tools-first, ZERO code — `GRAMMAR-WELLFORMED.A2.1-SOUNDNESS`).**
The grammar linter's `always_matches_shadowing` check (`ShadowingReason::EarlierAlwaysMatches`,
landed by `A2`/`PGEN-GRAMMAR-WELLFORMED-0006`) is **UNSOUND for PGEN's backtracking engine**. It
emits "alternative #N is unreachable — alternative #0 always matches (never fails) earlier than it
(**PEG commits to the earlier alternative**)". That premise is FALSE for PGEN: the engine BACKTRACKS
(enters ordered-choice alternatives in turn and keeps the one that leads to overall success), so an
earlier alternative that always-succeeds only by matching EMPTY does **not** shadow later ones.

This directly violates the foundational contract of [[feedback_certifying_linter_trustworthiness]]:
"the linter NEVER declares a live fragment dead." The check DOES.

**Decisive proof (existing shipped parser — NO regen).** Input `module m(interconnect p); endmodule`
(`--profile sv_2017`, ACCEPTS). The linter flags `net_port_type_sv_2017` alt #2
(`net_port_type_interconnect_sv_only`) UNREACHABLE. But:
1. `--parse-dump-ast-pretty` → the FINAL AST contains `{kind:"interconnect"}` on the route
   `… ansi > net_or_interface > sv_2017 > interconnect` — i.e. through `net_port_type` (the `sv_2017`
   wrapper is `net_port_type := net_port_type_sv_2017 -> {kind:"sv_2017"}`).
2. producer-grep → the ONLY `{kind:"interconnect"}` producer reachable in a PORT context is that alt #2
   (`grammars/systemverilog.ebnf:3490`); the other producer (`:3444`) is a `;`-terminated net
   *declaration*, impossible in a port list.
3. `--trace-rules net_port_type_sv_2017` → the parser ENTERS branch 1/3, 2/3, AND 3/3 (alt #0 logs
   `matched with zero length` 45×; branch 3/3 is entered and wins).
A branch that is entered and whose output is in the final parse tree is REACHED → "unreachable" is a
FALSE POSITIVE.

**Root cause.** `compute_always_succeeds` is CORRECT (`( net_type )? data_type_or_implicit` always-
succeeds, because `implicit_data_type := ( signing )? packed_dimension*` is nullable). The unsound step
is the CONCLUSION "⇒ later alternatives unreachable", which holds only under PEG-commit. PGEN's own
Phase `C1` ("defeat-earlier-branch crafting … so the parser SELECTS i on replay") already presumes
later branches are selectable — corroborating the backtracking model.

**Precise scope.** The sound property under backtracking is "can this alternative ever WIN" (a
language-difference / domination question — UNDECIDABLE in general, exactly the class the certifying-
linter decision EXCLUDES alongside general FIRST-domination and parse-order predicate reachability).
"Earlier alt is always-succeeds" is locally decidable but does NOT answer it. Empirically MIXED within
one rule: alt #2 (`interconnect`) WINS (live); alt #1 (`checked_nettype_identifier`) is entered-but-
never-wins for bare identifiers (alt #0 consumes any id as a `data_type` first) — "dominated" but still
not "unreachable". The check cannot separate these soundly.

**Consequences / disposition.**
- `A2.1`'s goal "promote `EarlierAlwaysMatches` to a hard gate" is **RETIRED** — promoting it would hard-
  reject grammars over LIVE branches. Even as a warning it makes an unprovable deadness claim (violates
  "no verdict without a checkable proof").
- The residual 8 SV `always_matches` warnings include FALSE POSITIVES (the port-header/net-type family
  fires via backtracking) — they are NOT dead branches to "clean".
- Item 4b store-gating is NOT an A2 deadness fix (the branches are reachable); any store-gating there is
  a separate AST-shape/fidelity question.
- **DISPOSITION IMPLEMENTED — `A2.2` DONE (2026-07-05, `PGEN-GRAMMAR-WELLFORMED-0150`, CODE).** Chosen
  of the three options: **demote to a non-verdict informational note** (there is NO sound sub-case —
  every always-succeeds form, empty-match or not, is defeated by the same backtracking argument; the only
  truly-redundant case is exact-DUPLICATE, already its own sound reason). Implemented in
  `rust/src/ast_pipeline/grammar_wellformedness.rs` + `rust/src/main.rs`: removed
  `ShadowingReason::EarlierAlwaysMatches` (the whole shadowing *verdict*) AND the unsound
  `UnreachabilityReason::EarlierArmAlwaysSucceeds` *certificate* variant (a certificate is a PROOF of
  deadness — which always-succeeds cannot supply); removed the now-vacuous `ShadowingReason::is_hard_gate`;
  added the non-verdict `WellformednessIssue::AlwaysSucceedsAlternative` + detector
  `detect_always_succeeds_alternatives`, whose message makes NO unreachability claim. `--lint-grammar`
  now reports `always_succeeds_alternatives=N (note)` (was `always_matches_shadowing=N (warning)`); SV's
  8 always-matches warnings → 6 informational notes; `ordered_choice_shadowing` stays a hard gate at 0;
  the proven-live `net_port_type_sv_2017 #2` interconnect branch is no longer flagged. NO grammar/parser
  regen (shadowing/note is lint-only; shadowing certs are branch-level, not in the rule-level cert-coverage
  proof pool, so canonical cert `1343/10/1321/12` is byte-identical at seeds 0/7/42). Book
  `docs/book/src/grammar-wellformedness.md` reconciled (the "As implemented (A2.2)" note + the
  "Where PGEN stands" section). `A2.3` (theoretical): `FixedTerminalPrefix` (`a | ab`) is *also* reachable
  under backtracking — its soundness deserves its own tool-proved re-audit; logged, not acted on here.

The earlier-fixed A2.1 families (boolean-abbrev, covergroup ranges, rs-prod, module-path, etc.) were
LRM-grounded DROPPED-DELIMITER extraction-artifact fixes (spurious `?`/lost `[ ]`/`{ }`) that are
correct on their own merits independent of any deadness claim — those fixes stand. What is retracted is
the deadness LABEL and the gate-promotion goal.

See `docs/tasks/GRAMMAR-WELLFORMED.md` (`A2.1-SOUNDNESS`, `A2.2`). Reinforces
[[feedback_certifying_linter_trustworthiness]] (sound-not-complete; honest UNKNOWN, never a wrong verdict)
and [[feedback_be_alert_root_cause_fishy_immediately]] (root-caused tools-first, not classified-and-routed).
