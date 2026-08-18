# Scope-prefix rendering drift — `SV-CORPUS-GRAD.13c.2j`

IEEE 1800-2017/2023 A.8.4 writes one production for the prefix of a hierarchical name:

```text
primary ::= … | [ class_qualifier | package_scope ] hierarchical_identifier select | …
class_qualifier ::= [ local :: ] [ implicit_class_handle . | class_scope ]
```

PGEN renders it in **four** places. One is a named rule, `primary_hier_scope_prefix`; the other
three are hand-spelled inline groups, because `split_hierarchical_callable_receiver`,
`method_call_receiver_sv_2017` and `method_call_receiver_sv_2023` exist to cut the cycle
`primary → call_primary → method_call → primary` and therefore cannot simply say `primary`.

A copy of a normative production drifts. This one did: `SV-EXH-PROOF.3.3.4.b.6.2.37.3` added the
`class_scope` branch to the named rendering for a measured defect, and none of the three copies
received it — so `p::base::m.g()` had no derivation while `p::base::m` alone parsed. Nothing in
the repository compared the four renderings, which is why the divergence survived.

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

**The attribution** is re-derived, not asserted. `.13c.2j` first restored `class_scope` at all
three copies; the parse-cost ratchet REFUSED that (`entries` +0.47 %, `memo_hits` +1.05 %) and a
per-site measurement showed `split_hierarchical_callable_receiver` alone carried the whole
accept-set gain. Costs are rejected here, not traded, so only that site keeps the alternative and
the other two are PINNED divergences. On every run the sweep strips `class_scope` from all three
renderings in a scratch copy, proves each reproducer REJECTS there, then restores it at one site
at a time — 12 measurements, each falsifiable in its own direction:

```text
attribution control: with `class_scope` at NO site, all 2 reproducers REJECT on every profile
      split_hierarchical_callable_receiver alone -> …_method_receiver.sv [sv_2017] accept  (as specified)
      method_call_receiver_sv_2017 alone         -> …_method_receiver.sv [sv_2017] reject  (as specified)
```

The day one of those pinned sites becomes reachable for a class-scoped receiver, its entry goes
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

`sweep_after.txt` is this tree (`sites=4 drift=0 pinned=2 attribution_broken=0`, exit 0).
`sweep_before.txt` is the same sweep against the pre-fix grammar via `--grammar`, and it is the
proof the instrument can go RED (`drift=1`, exit 1, naming the un-pinned site at its pre-fix line).

## What this does not decide

Only the alternative SET, plus the reachability of each pinned omission over the reproducers the
spec names. It does NOT decide what FOLLOWS the prefix: `primary_sv_2017` writes
`hierarchical_identifier select`, the copies write `hierarchical_identifier` alone. That difference
is deliberate and measured — `.13c.2c` restored `select` to both receivers and it moved **zero rows
in either direction**, because `hierarchical_identifier`'s own possessive
`( identifier constant_bit_select dot )*` has consumed the selects before `select` is entered
(`LANG-CAPABILITY-AUDIT.10.18`). Each site's trailing context is printed so the divergence stays
visible, and the sweep does not fail on it.

⛔ One member of the canonical set is itself a defect, recorded here so the sweep is not read as a
fidelity claim it cannot make: `kw_class_qualifier_fa08937d` is `trivia /class_qualifier\b/` — a
literal keyword standing in for an LRM **nonterminal**, owned by `.13c.2m`. The sweep asserts that
the four renderings AGREE; it does not assert that what they agree on is right.
