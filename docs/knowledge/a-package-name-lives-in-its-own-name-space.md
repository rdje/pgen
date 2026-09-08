---
id: a-package-name-lives-in-its-own-name-space
title: A package name lives in its own name space — a scope gate that consults the type name space to refuse a package name refuses legal text; gate the scope on the fact of the thing it names, and keep the exclusion for when that fact is absent
answers:
  - "why did PGEN reject T::VALUE when a typedef named T was visible beside package T before release 1.0.225"
  - "can a package and a typedef share a name in SystemVerilog — IEEE 1800-2017 3.13 name spaces"
  - "does import p::* make a typedef T shadow package T — 26.3 potentially locally visible"
  - "what does non_typedef_package_scope gate in the SystemVerilog grammar and why is it an ordered choice"
  - "how was the package-scope fix kept free on the parse-cost sample — branch-local predicates instead of a dispatcher"
  - "is std::randomize(obj.x) legal — 18.12 Syntax 18-11 variable_identifier_list"
  - "is extern function f(in); legal without a return type — A.2.6 function_prototype"
  - "is a virtual function prototype without a body legal in a class — A.1.9 class_method pure virtual or extern"
  - "is a trailing comma legal in a port connection list — A.4.1.1 list_of_port_connections"
tags: [systemverilog, grammar, semantic-store, packages, name-spaces, annex-a, answer-key, sv-corpus-grad]
date: 2026-09-08
status: current
evidence: "SV-CORPUS-GRAD.3.54 (PGEN-SV-CORPUS-GRAD-0384, release 1.0.225, ledger SV-0098). On the 1.0.224 probe: `package T; parameter int VALUE = 23; endpackage module test; initial if (T::VALUE !== 23) $stop; endmodule` parse_full passed; the same after `package p; typedef int T; endpackage import p::*;` furthest_position=132 (the VALUE after T::); after a unit-level `typedef int T;` furthest_position=95; `typedef int T;` with no package furthest_position=45 (correct). Predicate trace (--trace-rules ps_parameter_identifier,scoped_package_parameter_identifier,non_typedef_package_scope,class_scope): non_typedef_package_scope rejected by post predicate lacks_fact_attribute_equals [type_name, T, declaration_family, typedef], then every class-scope arm refused. Corpus census by token: 1 failing SV-lane file with a package and a typedef of one name (iverilog sv_type_identifier_package_name.v), 0 accepted files; 15 accepted files use a typedef name as a X:: prefix."
reverify: "./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_package_scope_beside_imported_typedef.sv --profile sv_2017 2>&1 | grep -q 'parse_full passed' && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/invalid_typedef_name_as_package_scope.sv --profile sv_2017 2>&1 | grep -q 'furthest_position=45'"
---

**The fact.** IEEE 1800-2023 §3.13 lists the language's name spaces and gives packages one of their own: "The
package name space unifies all the package identifiers defined among all compilation units"
(`docs/systemverilog/2023/md/section-3-design-and-verification-building-blocks.md:420-432`); 1800-2017 says the same sentence at `docs/systemverilog/2017/md/section-3-park-avenue.md:432` — a vendored file whose only headings are the
address page, so it is cited by line, not by clause. A.9.3 `package_scope ::=
package_identifier :: | $unit ::` (`docs/systemverilog/2017/md/section-41-data-read-api.md:3390`) reads `T::VALUE`
as a package scope whenever `package T` is declared. A typedef named `T` — declared in the compilation unit, or only
*potentially* visible through `import p::*` (§26.3, `docs/systemverilog/2017/md/section-26-packages.md:206-210`) —
is another name space's business and cannot take the scope away. iverilog's `sv_type_identifier_package_name.v` is
the language's own statement of this: "Check that package names can shadow visible type identifiers."

**The defect.** Through release `1.0.224` the SystemVerilog grammar's package arm — `non_typedef_package_scope`,
reached from every `pkg::` site (18 call sites; 7.2 M attempts on the parse-cost sample) — carried ONE gate: "the
name is no typedef" (`lacks_fact_attribute_equals [type_name, name, declaration_family, typedef]`, `phase: post`).
That exclusion is right for a typedef that names no package (`typedef int T; T::VALUE` with no `package T` is no
scope on any reading — it stays refused, pinned) and wrong the moment the package exists: the scope refused, and every
class-scope arm refused after it (`T` is no class). The predicate trace (TOOLBOX §2.2 + §2.4) names the gate in one
line; the bisection — the corpus text minus one thing at a time until the stop moves — names the variable.

**The fix shape.** The exclusion becomes branch-LOCAL (`phase: branch, view: shaped`, BRANCH-PREDICATE-LOCALITY.2)
on the rule's first alternative, and a SECOND alternative admits the scope by `has_fact [package_name, name]` — the
fact `declared_package_identifier` emits at the declaration — under `@branch_policy: ordered`: the second alternative
is tried only when the first fails — a typedef-named scope, or far more often no `identifier ::` at all — and the
engine answers its `package_scope` from the memo: no frame is added, and what the ratchet records is a replay
(`package_scope` +721,482 entries = +721,482 memo hits, committed conserved, on the pinned sample), the second shape of
`lookahead_replay_occurrences`. Both alternatives carry `-> {body: $1}`; every accepted tree is byte-identical. A
dispatcher around the rule (the `.3.50` / `.3.53` shape) would have cost +7.2 M entries — the shape was chosen by the
cost table, not by habit.

**The certificate found the planner's flattening.** BRANCH-PREDICATE-LOCALITY.2 made the parser evaluate an inline
`phase: branch` predicate on its branch alone; the certificate's planner (`compute_name_gates` in
`stimuli_generator.rs`) still merged every branch's annotations into the rule's, so the new alternative's
`has_fact [package_name, …]` made the whole rule a package-gated consumer no prelude could satisfy, and thirteen rules
witnessed only through a `pkg::` prefix went UNKNOWN in the canonical basis (seven in the union). The planner is
branch-local since `1.0.225`: a positive gate on one alternative gates the rule only when every alternative carries
one (`BRANCH-PREDICATE-LOCALITY.3`).

**The reading rule.** A gate that consults one name space to refuse a name from another refuses legal text. Gate a
scope on the fact of the thing it names, and keep the exclusion for when that fact is absent. And "contextual" is a
hypothesis until the variable is found: of seven rows that passed in isolation, one was this gate, five were text no
edition derives (a `std::randomize` argument that is a select — 18.12 Syntax 18-11 takes identifiers; an `extern
function` prototype with no return type — A.2.6 `function_prototype` takes `data_type_or_void`; a bodiless `virtual`
method — A.1.9 admits a prototype only behind `pure virtual` or `extern`), and one hid a trailing comma in a
port-connection list (A.4.1.1) behind a class no file of its fixture declares.
