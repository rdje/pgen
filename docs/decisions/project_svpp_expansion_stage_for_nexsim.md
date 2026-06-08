---
name: project-svpp-expansion-stage-for-nexsim
description: DIRECTIVE (director 2026-06-08) — PGEN must deliver the SystemVerilog preprocessing/EXPANSION stage (macro substitution + conditional resolution + `include` inlining) that composes the `systemverilog_preprocessor` directive parse with the `systemverilog` parser into a usable front-end for downstream consumers (NEXSIM first). The SV + SVPP front-end stack is NOT release-complete for NEXSIM without it — a parse tree of un-expanded SV (macros opaque, conditionals unresolved, includes missing) is not a usable design representation. The svpp + sv parsers remain valid deliverables AS PARSERS; the expansion stage is a SEPARATE deliverable (a transformation over the svpp directive AST, not a parser). Tracked by tree SVPP-EXPANSION. SEQUENCED AFTER the locked program (all existing parsers cert-coverage clean + UNKNOWN=0) — handled later.
metadata:
  node_type: memory
  type: project
  director_directive: true
  created: 2026-06-08
  owning_tree: SVPP-EXPANSION
---

**THE DIRECTIVE (director, 2026-06-08).** "Something in the roadmap and task-trees mentions PGEN aims to be
a service for downstream consumers like linters, compilers, elaborators, so its feature surface shall grow
to accommodate that ultimate goal. For NEXSIM, PGEN ought to deliver the whole package — I mean the
expansion stage. Track that in a task-tree and we will handle that later. We can't release the SV and SVPP
parsers without this expansion stage." Follow-up sequencing (same session): "All the existing parsers shall
be cert-coverage clean first — everything wired and no UNKNOWN — before pivoting to this expansion stage."

**THE GAP (tool-verified, 2026-06-08).** PGEN's `systemverilog_preprocessor` (svpp) parser is a directive-
STRUCTURE parser: it parses `` `define `` / `` `ifdef `` / `` `include `` / macro bodies / condition exprs
into a typed AST and passes non-directive code through as `non_directive_text`. It does **not** expand
macros, resolve conditionals, or inline includes — there is no substitution engine in the grammar. The
`systemverilog` (sv) parser parses SV source directly; it *tolerates* a directive appearing inline via a
`compiler_directive` rule (`` `[^\r\n]* `` — matched as an opaque item) but does **not** expand it either
(`systemverilog.ebnf:178` is explicit: "does not expand `` `include(...) ``"). So **neither parser expands**,
and there is **no PGEN stage that feeds svpp's output into sv**. They are two independent parsers for two
different jobs. To realize the classic preprocess→parse pipeline a consumer expects, an EXPANSION stage
must sit between them — which PGEN does not currently provide.

**WHY IT IS A NEXSIM RELEASE PREREQUISITE.** Real SystemVerilog cannot be elaborated without preprocessing:
macros expand into arbitrary token streams the SV grammar then parses; `` `ifdef `` conditionals select which
code is *live*; `` `include `` pulls in the package/interface/typedef definitions the body depends on. A
parse tree of un-expanded SV is not a usable design representation for a simulator/elaborator like NEXSIM
(the primary near-term SV+VHDL integration target). PGEN's stated mission — be a trustworthy front-end
service for downstream consumers (the linter / compiler-elaborator enablement + "front-end workbench"
direction: hand downstream tools "a stronger front-end product than just a parse tree") — squarely includes
this. Shipping svpp + sv to NEXSIM without expansion hands NEXSIM two pieces that do not compose into what
it needs.

**THE DECISION.** PGEN shall deliver a SystemVerilog **preprocessing / expansion stage** — the transformation
that composes the svpp directive AST with (1) macro substitution (object-like + function-like macros with
arguments, `` `" `` stringize, `` ` ` `` token-paste, nested/recursive expansion with a cycle guard),
(2) conditional resolution (`` `ifdef `` / `` `ifndef `` / `` `elsif `` / `` `else `` / `` `endif ``, nested),
and (3) `` `include `` inlining (path resolution + recursion guard), plus the supporting directives
(`` `undef ``, `` `line ``, predefined/built-in macros where in scope) — producing expanded, sv-parseable
source. The expanded output must re-parse cleanly through the `systemverilog` parser; closure expects
round-trip + coverage proof, parser-agnostic where the svpp directive model is general. Tracked by tree
**SVPP-EXPANSION**.

**SEQUENCING (binding, director 2026-06-08).** This is owned-now / **build-later**. It is sequenced strictly
AFTER the current **locked program** — every existing parser cert-coverage WIRED + clean (`sample_parse_failures`
0) and `UNKNOWN` = 0. PGEN does not pivot to building the expansion stage until that is achieved.

**CONSEQUENCES.**
- The svpp + sv parsers stay valid, independently-proven deliverables AS PARSERS (their grammar-closure proof
  is unaffected). What changes is the **release framing**: the SV *front-end* deliverable for NEXSIM is not
  release-complete until the expansion stage lands. When it lands, the SV + SVPP integration contracts /
  release policy are updated in lockstep to state the front-end pipeline (svpp parse → expand → sv parse).
- The expansion stage is a TRANSFORMATION (a service over the svpp AST), consistent with PGEN's "parsers from
  EBNF" doctrine — it is not a new parser and does not weaken that doctrine.
- VHDL is out of scope (no C-style preprocessor); this is SV-specific.

**Cross-links.** Tree [SVPP-EXPANSION](../tasks/SVPP-EXPANSION.md). Composes the svpp parser (directive AST) +
the sv parser (expansion's output target). Aligns with `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`
and `docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md` (the front-end-workbench / service direction). Disciplines:
[[feedback_always_signoff_decisions]], [[feedback_research_grounded_sota_no_trial_and_revert]] (survey IEEE 1800
§22 + slang/Verible/Verilator preprocessing models before building), [[feedback_tools_first_no_guessing]].
