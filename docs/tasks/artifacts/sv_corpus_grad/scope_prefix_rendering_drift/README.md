# Scope-prefix rendering drift — `SV-CORPUS-GRAD.13c.2j` (instrument rebuilt by `.40`, 2026-10-03)

IEEE 1800-2017/2023 A.8.4 writes one production for the prefix of a hierarchical name:

```text
primary ::= … | [ class_qualifier | package_scope ] hierarchical_identifier select | …
class_qualifier ::= [ local :: ] [ implicit_class_handle . | class_scope ]
```

PGEN renders it in **four** places. One is the canonical named rule, `primary_hier_scope_prefix`; one
is the named helper a CALLABLE receiver reaches, `split_callable_receiver_scope` (an inline group until
`.3.42` lifted it, release 1.0.215); and two are the receiver copies `method_call_receiver_sv_2017` /
`method_call_receiver_sv_2023`, whose `hierarchical_identifier` arm renders NO prefix at all. The
three copies exist to cut the cycle `primary → call_primary → method_call → primary` and therefore
cannot simply say `primary`.

A copy of a normative production drifts. This one did: `SV-EXH-PROOF.3.3.4.b.6.2.37.3` added the
`class_scope` branch to the named rendering for a measured defect, and none of the three copies
received it — so `p::base::m.g()` had no derivation while `p::base::m` alone parsed. Nothing in
the repository compared the four renderings, which is why the divergence survived.

⛔ **The instrument drifted too, and in the passing direction's neighbour: it could not run.** Its
extractor admitted only `( a | b )?` inline groups; `.3.42` lifted those into named rules, `.13c.2m`
deleted the `class_qualifier` literal from its vocabulary, `.3.37` / `.39` added a `local ::` arm it did
not name — and nothing invoked it, so from 2026-09-06 to 2026-10-03 it read `CONTROL FAILED` on every
grammar and nobody saw (`.40`). It now reads three forms — NAMED (`NAME?` before the prefixed name,
the rule body an alternation over the vocabulary), INLINE (the old form, kept) and EMPTY (a receiver copy
the spec names, whose bare arm must stay bare) — its vocabulary is the three spellings of the
`[ local :: ] class_scope | package_scope` half, and the SV syntax-closure gate runs it on every pass.

## The instrument

`sweep_scope_prefix_renderings.py` extracts the alternative SET of every rendering and compares
each to the canonical one. Controls run before any verdict prints; two are worth naming.

**The mechanism control** is the pair of synthetic grammars beside it:

| file | shape | `p::b::m` |
|---|---|---|
| `inline_alt_giveback.ebnf` | `( short \| long )? tail` — the inline group, both alternatives | **accept** |
| `inline_alt_noLong.ebnf` | the same grammar with one alternative removed | **reject** |
| `ruleref_alt_giveback.ebnf` | the same alternation lifted into a rule reference | **accept** |

That pair is `.13c.2j` in five rules, and it answers the design question the leaf opened with:
PGEN gives back across an **inline** alternation exactly as it does across a rule's branches
(`branch_policy=longest_match`), so restoring the missing alternative in place was sufficient and
lifting the copies into a shared rule was **not** required for the accept-set fix. That lift is
routed separately (see the leaf) because it changes the `qualifier`/`scope` field of a
contract-documented AST from a bare alternative to `primary_hier_scope_prefix`'s `{kind, body}`
envelope — a downstream-visible change that belongs with a contract bump, not with a
parse-correctness fix.

**The attribution** is re-derived, not asserted, one entry per vocabulary member. `.13c.2j` first
restored `class_scope` at all three copies; the parse-cost ratchet REFUSED that (`entries` +0.47 %,
`memo_hits` +1.05 %) and a per-site measurement showed the callable receiver's rendering alone carried
the whole accept-set gain. Costs are rejected here, not traded, so only that rendering keeps the
alternative and the receiver copies keep NO prefix — PINNED divergences. `.39` (release 1.0.248) then
put the `local ::` arm at the same one rendering, for the same reason, and the sweep carries that as a
second member. On every run it strips a member from all four renderings in a scratch copy, proves each
reproducer REJECTS there, then restores it at one site at a time — for an EMPTY site by widening its
bare arm to `( member )? hierarchical_identifier` — 24 measurements, each falsifiable in its own
direction:

```text
attribution control [class_scope]: with `class_scope` at NO site, all 2 reproducer(s) REJECT on every profile
      [class_scope] split_callable_receiver_scope alone -> …_method_receiver.sv [sv_2017] accept  (as specified)
      [class_scope] method_call_receiver_sv_2017 alone  -> …_method_receiver.sv [sv_2017] reject  (as specified)
attribution control [local]: with `local` at NO site, all 1 reproducer(s) REJECT on every profile
      [local] split_callable_receiver_scope alone -> defect_local_qualified_method_call.sv [sv_2017] accept  (as specified)
      [local] primary_hier_scope_prefix alone    -> defect_local_qualified_method_call.sv [sv_2017] reject  (as specified)
```

The day one of those pinned sites becomes reachable for a prefixed receiver, its entry goes
`ATTRIBUTION BROKEN` rather than staying quietly true.

⭐ Both bugs this instrument has had were caught by its own controls, which is the argument for
writing controls before verdicts: length-changing comment blanking shifted every byte offset (the
attribution control refused, *"this alternative is not what decides these rows"*), and a two-pass
rewrite compounded its own offset error (the load-bearing site reported inert, which its `flips`
expectation refused).

## Reading the output

```bash
python3 docs/tasks/artifacts/sv_corpus_grad/scope_prefix_rendering_drift/sweep_scope_prefix_renderings.py
python3 …/sweep_scope_prefix_renderings.py --fast     # textual pass only; prints NOT RE-DERIVED
```

`sweep_2026-10-03.txt` is this tree after `.40` (`sites=4 drift=0 pinned=2 members=2
attribution_broken=0 attribution_rederived=yes`, exit 0, 89 s) and `sweep_2026-10-03_red_control.txt` its
built-in RED control (`--red-control`: a scratch copy with `class_scope` stripped from
`split_callable_receiver_scope` reads `DRIFT`, rc 1). `sweep_after.txt` is the 2026-08-18 tree under the
first extractor (`sites=4 drift=0 pinned=2 attribution_broken=0`, exit 0) and `sweep_before.txt` the same
sweep against the pre-`.13c.2j` grammar via `--grammar`, the first proof the instrument could go RED
(`drift=1`, exit 1, naming the un-pinned site at its pre-fix line). Since `.40` the sweep is a step of
`rust/scripts/sv_syntax_closure_gate.sh` (`make -C rust sv_syntax_closure_gate`), red control first, so a
control that cannot run is never again mistaken for a passing one.

## What this does not decide

Only the alternative SET, plus the reachability of each pinned omission over the reproducers the
spec names. It does NOT decide what FOLLOWS the prefix: `primary_sv_2017` writes
`hierarchical_identifier select`, the copies write `hierarchical_identifier` alone. That difference
is deliberate and measured — `.13c.2c` restored `select` to both receivers and it moved **zero rows
in either direction**, because `hierarchical_identifier`'s own possessive
`( identifier constant_bit_select dot )*` has consumed the selects before `select` is entered
(`LANG-CAPABILITY-AUDIT.10.18`). Each site's trailing context is printed so the divergence stays
visible, and the sweep does not fail on it.

⛔ The `implicit_class_handle .` half of `class_qualifier` is OUTSIDE the compared vocabulary. The grammar
renders it as rule ARMS, not as members of these alternations — the canonical rule's
`implicit_class_handle_prefix_sv_only` arm (`.29`) and the callable host's own
`implicit_class_handle dot callable_method_call_body` alternative — and removing it from the canonical
rule alone cannot make a `this.`-rooted call reject, so the attribution control has no purchase on it.
The sweep prints the canonical rule's arms outside the vocabulary and excludes a `NAME?` whose body steps
outside it, naming the alternative that excluded it (`implicit_handle_or_class_scope` renders A.1.10's
`constraint_primary` prefix, a different production). The `class_qualifier` LITERAL the first vocabulary
carried — an LRM nonterminal welded into a keyword — was deleted by `.13c.2m` (release 1.0.244). The
sweep asserts that the four renderings AGREE on the compared half; it does not assert that what they
agree on is right.
