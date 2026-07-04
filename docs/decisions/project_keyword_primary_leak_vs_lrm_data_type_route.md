# A keyword-reservation "leak" fix: build the oracle in a data_type-free position; residual `data_type` accepts are LRM-legit, not leaks

- Category: `project`
- Established: 2026-07-04 (`SV-KEYWORD-PRIMARY-FIDELITY.2`, `PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0002`)

## Context

`SV-0035` = a reserved *type* keyword (`integer`/`real`/`reg`/`logic`/…) parsing as a bare
expression **primary** under every SV profile. The fix routes the expression-primary bare-`identifier`
name carriers (`specparam_identifier`, `genvar_identifier`, `hierarchical_identifier`-trailing-name)
through the keyword-excluding `non_keyword_identifier`. Two non-obvious traps surfaced during the
"re-probe after each fix" moved-leak discipline.

## Decision / durable facts

1. **The completeness oracle for a keyword-as-primary fix MUST be a position with NO `data_type`
   alternative.** Use `assign w = <kw>;` (continuous-assign RHS is a pure `expression`) or
   `localparam p = <kw> + 1;` (the `+ 1` forces a primary operand — `data_type + 1` is not an
   expression). A bare `localparam p = <kw>;` or `reg q [<kw>];` is a **dual-purpose** position and
   is the wrong oracle (see 2).

2. **Not every residual accept is a leak.** After the fix, `localparam p = integer;` still ACCEPTs —
   but via the **LRM-faithful** `constant_param_expression ::= constant_mintypmax_expression |
   data_type | $` route (`grammars/systemverilog.ebnf:1380`; `integer` genuinely IS a `data_type`,
   IEEE 1800 A.2.1.1). Likewise `reg q [integer];` under the SV profiles is the legitimate
   `associative_dimension ::= [ data_type ]`. These are **correct SV syntax** — the type/value
   distinction is an elaboration-time semantic check, not a parse-time one. **Do NOT "finish closing
   SV-0035" by making these reject** — that would remove legal IEEE 1800 syntax (violates
   [[project_ebnf_is_single_source_of_truth]]). This is also why one must NOT delete these
   alternatives (cf. [[feedback_no_rule_deletion_without_lrm_proof]]).

3. **Carrier-routing and reserved-list-completeness are independent axes.** Routing carriers through
   `non_keyword_identifier` closes the leak only for keywords IN the profile's reserved list.
   `reserved_non_keyword_identifier_sv` (`:386`) enumerates only data-type + control-flow words, so
   net-type/gate/structural SV keywords (`wire`/`and`/`always`/…) still leak — a *separate* defect
   (`SV-0036`, a reserved-LIST-completeness gap). The `_v2005` list is the complete Annex B, so
   `verilog_2005` was already clean on that axis.

4. **Cert signature of a carrier-routing fix = no delta.** Tightening a name rule to reject reserved
   keywords cannot reduce valid-sample witnessing (the stimuli generator never emits a keyword as a
   name for a valid sample), so both canonical and profiled certs stayed byte-identical — unlike a
   whole-rule **profile-gate** fix (`.6.9`/`.6.10`/`.6.11`), which de-witnesses leak-earned false
   witnesses because the gated rule leaves the profile universe. Expect no cert re-pin from carrier
   routing; expect a witness drop from a profile gate.

## Consequences

Guides `SV-KEYWORD-PRIMARY-FIDELITY.3` (the `SV-0036` reserved-list extension) and any future
keyword-reservation work: pick the data_type-free oracle, leave the `data_type`-route accepts alone,
and don't expect a cert delta from a carrier swap. Reinforces
[[feedback_tools_first_no_guessing]] and [[feedback_be_alert_root_cause_fishy_immediately]].
