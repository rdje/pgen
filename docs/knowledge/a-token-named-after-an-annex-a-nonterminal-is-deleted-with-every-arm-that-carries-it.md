---
id: a-token-named-after-an-annex-a-nonterminal-is-deleted-with-every-arm-that-carries-it
title: A token named after an Annex A nonterminal is deleted with every arm that carries it — never gated or guarded — and the census over the whole kw_ inventory is what says it was the only one
answers:
  - "how do I find whether the grammar reserves a word that IEEE 1800 Annex B does not"
  - "why did the parser accept y = class_qualifier m; and y = null class_qualifier:=local::43;"
  - "what is kw_class_qualifier_fa08937d and why was it deleted"
  - "which kw_ tokens of the SystemVerilog grammar are not Annex B keywords"
  - "is randomize a reserved keyword in SystemVerilog"
  - "why do the sv_2017 and sv_2023 type-scope twins ps_type_identifier_scope_sv_2017 / _sv_2023 both exist"
  - "how did the grammar extractor weld a production definition line onto the previous production"
  - "when is a schema bump owed for removing a branch that no legal input ever produced"
tags: [sv-corpus, grammar-extraction, lrm-fidelity, over-acceptance, keywords, schema-versioning]
date: 2026-09-23
status: current
evidence: "SV-CORPUS-GRAD.13c.2m (PGEN-SV-CORPUS-GRAD-0413, release 1.0.244, schema 33 -> 34): the markdown the extractor read prints `class_qualifier := [ local ::43 ] …` with ONE colon (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3097; one such head per edition against 717 / 734 correct `::=` heads), so the head was appended to `primary`'s `null` alternative, the footnote superscript became a token and `class_qualifier` a literal keyword carried by three scope-prefix copies; nine pinned arms accepted the LRM's own production text through 1.0.243 (`accepts_invalid_*` pins, flipped to `invalid`). The census over the 318 `kw_<name>_<hash>` tokens against the three keyword annexes read line-wise from their *Reserved keywords* heading (1800-2017 = 1800-2023 = 248 names, 1364-2005 126): 248 keywords + 68 LRM literals + two residual names, `class_qualifier` (deleted) and `randomize` (A.8.4 randomize_call's own literal, not reserved; `int randomize;` parses — pinned `control_randomize_as_identifier.sv`). docs/tasks/SV-CORPUS-GRAD.md `.13c.2m`, grammars/systemverilog.ebnf `primary_hier_scope_prefix` / `ps_type_identifier_scope_sv_2017`."
reverify: "grep -c 'kw_class_qualifier\\|kw_n_43\\|kw_n_48' grammars/systemverilog.ebnf | sed 's/^/non-comment refs must be 0 among: /'; grep -nE '^kw_(class_qualifier|n_43|n_48)_' grammars/systemverilog.ebnf   # prints nothing: the three tokens are gone; then python3 stimuli/sv/run_adjudication_repros.py --verbose 2>&1 | grep -E 'accepts_invalid_class_qualifier_as_prefix|control_randomize_as_identifier'"
---

**When a `kw_<name>` token's name is an Annex A NONTERMINAL rather than an Annex B keyword, the
token is an extraction artifact: delete it and every arm that references it, render the
nonterminal's real production where the LRM puts it, and let the schema bump if a branch went.**

## The mechanism, and how to see it

- The extractor splits productions on `::=`. A source line whose head prints `:=` (one lost colon)
  is not a head, so its text is appended to the PREVIOUS production's last alternative, and every
  word in it that is not already a rule becomes a literal token — including a footnote
  superscript glued to `::`. Two editions renumber the footnote (`43` → `48`), which is why the
  `43` form parsed only under `sv_2017` and the `48` form only under `sv_2023`: the LRM itself
  supplies the discriminator that proves the weld.
- The instance was found by reading a rendering (`.13c.2j`); the CLASS is found by a census: every
  `kw_<name>` name against the keyword annexes, read line-wise from *Reserved keywords* (a loose
  word roster over the 2017 markdown reads Annex A itself as keywords, because those section files
  are mis-titled — restrict the roster to the table). The residual is the answer, and here it was
  two names: the weld, and `randomize` — an Annex A literal that is NOT reserved and that the
  identifier rule does not exclude (measured: `int randomize;` parses).

## Why delete, not gate

Every arm that carried the token could match only the LRM's production text, never SystemVerilog:
there is nothing to keep, and a predicate or a profile gate would keep paying the arm's asks at
every primary. Deleting the literal left `primary_hier_scope_prefix` unsatisfiable under
`verilog_2005` (a profile orphan); the lint's DERIVED minimal fix — tag the rule SV-only — is
correct because its hosts read an empty optional there, as on every legal 1364-2005 text before.

## The dual, at every position the production reaches

The same weld made the LRM's `[ local :: ]` type qualifier (A.9.3) dead: the type-scope twins
demanded the superscript. `.3.37` had rendered `local::` at the primary; this leaf renders it at
the type name (`local::T'(1)` inside `randomize() with { … }`) with the node the primary's arm
already renders (`{kind: "local_qualifier"}`), the package / class scopes passed through. The
twins stay two rules although their text is now identical, so their hosts' shapes do not move.

## Schema

A branch removed is a bump by the SV parser book's rule — even when no legal input ever produced
the kind (`null_class`, `null_class_assign`, `class_qualifier`). The bump is honest about the
contract; the byte-identity of every accepted tree is the measurement, not the argument.
Related: [[a-widening-arm-probes-its-sibling-sites-before-the-corpus-does]],
[[a-fallback-arm-goes-inside-the-shared-rule-as-an-ordered-last-alternative]].
