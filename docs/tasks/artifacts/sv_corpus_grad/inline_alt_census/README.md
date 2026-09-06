# The inline-alternation census (`SV-CORPUS-GRAD.3.42`, TOOLBOX §1.5d)

`inline_alt_census.py <grammar.ebnf>` names every inline alternation `( a | b … )` that sits inside a group
(bare, `?`, `*`, `+`) of a rule carrying at least one return template, with the branch indices a template of
that rule would TYPE (`shadowed=[…]`). It is the mechanical form of the `ENGINE-UNIVERSAL-SERVICES.96` census:
spelled inline, the group's branch N wears the enclosing rule's template N wherever one exists, so the shipped
tree carries the host's template wrapped around the branch — `<invalid_sequence_access>` when that template names
a `$n` the branch lacks.

```bash
python3 docs/tasks/artifacts/sv_corpus_grad/inline_alt_census/inline_alt_census.py grammars/systemverilog.ebnf
# INLINE-ALT-CENSUS: grammar=… annotated_rules_with_inline_alternation_sites=N exposed(…)=E
#   EXPOSED / EXPOSED(raw-envelope) / unreferenced / no_template_at_branch_index — one line per site
python3 docs/tasks/artifacts/sv_corpus_grad/inline_alt_census/inline_alt_census.py --self-test
# replays the seven tracked synthetics under docs/tasks/artifacts/engine_universal_services/es96_inline_alt_template/
# (four that reproduce the class must read exposed=1, three lifted twins exposed=0); a miss exits 7
```

⛔ **Why it exists — the class definition, not the grep, bounds a census.** The `.96` textual census (2026-09-04)
defined the shape as *"an inline alternation inside an annotated MULTI-alternative rule"* and found 3 sites. Two
five-line synthetics changed the definition — a ONE-template host shadows branch 0 (`optional_alt_single.ebnf`), an
UNANNOTATED alternative of an annotated host keeps the corrupted element in its raw envelope (`unannot_alt.ebnf`) —
and this census then read **16 sites in 12 rules** on the same grammar (release 1.0.214) and **0** after the lift
(release 1.0.215). Five of the sixteen emitted corrupted nodes on ordinary SystemVerilog in accepting files.

⚠️ **Honest bound:** text-level triage on the `.ebnf`. It names the sites a template CAN type; whether a shadowed
branch ever WINS a final tree is Protocol D (`--trace-rules` + `🏁`) and the liveness census / AST identity sweep in
`../inline_alt_lift/`. An `unreferenced` site (the alternative has a template that does not name the group's `$n`)
drops the corrupted value and is harmless. A grammar with 0 sites can still carry the class in a shape this parser
does not recognise (a group nested two levels down, for instance) — run the synthetics if in doubt.

Census over every tracked grammar at SV release 1.0.214: `systemverilog` 16 (fifteen lifted into eight helpers, one alternative respelled), `ebnf` 35,
`semantic_annotation` 14, `vhdl` 2, `rtl_frontend` 1, the two 1800 LRM extractions 1 + 2, the rest 0 — the non-SV
sites are routed to `ENGINE-UNIVERSAL-SERVICES.96` (the engine fix removes the class in every family at once).
