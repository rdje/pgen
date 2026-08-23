---
id: a-minimal-core-is-a-witness-only-for-the-vintage-it-was-shrunk-under
title: A minimal reproducer is minimised AGAINST ONE ARTIFACT — carried to another vintage it can flip while the row it came from does not, so a core is a witness for the vintage it was shrunk under and for no other
answers:
  - "my minimal repro behaves differently on an older commit than the full input does"
  - "can I use a shrunk test case to bisect across versions"
  - "the reduced case says the fix regressed but the original input says it did not"
  - "is a delta-debugged core a faithful stand-in for its input"
  - "how do I attribute a rejected row to a specific change"
  - "why does my minimised case disagree with the ledger"
  - "should I re-shrink a reproducer when I change the artifact under test"
tags: [measurement, reproducers, delta-debugging, bisection, attribution, claim-verification, grammar-wellformedness]
date: 2026-08-23
status: current
evidence: |
  GRAMMAR-WELLFORMED.H.16.6c → H.16.6e (`PGEN-GRAMMAR-WELLFORMED-0180`, `-0182`).

  `H.16.6c` shrank a rejected 192-byte stimulus, row `#0875`, to the 48-byte core
  `@handles : { W => %RXCy => 72e10 }`, and routed it to `H.16.6e` as evidence that an arrow in a
  map VALUE position is admissible only through narrow routes.

  Worked one leaf later, that core was probed across grammar vintages and came back
  **accepted by the pre-`map_key` grammar (`dcc2e2d8`) and rejected at HEAD** — which reads exactly
  like the intervening `H.16.6b` having NARROWED the accept set, contradicting its published and
  gate-checked *"25 newly accepted, 0 newly rejected"*.

  It had not. The ROW the core came from is rejected by **both** vintages, and the per-input ledger
  re-derived over the rebuilt 1 000-row corpus reads `control=pre_a_plus arm=post_b`
  **`widen=10 narrow=0`** — the published claim, reproduced blind.

  The mechanism is ordinary and general. Shrinking removes context until the verdict stops changing
  UNDER THE ARTIFACT YOU ARE SHRINKING AGAINST. The discarded context is exactly what the reduction
  proved irrelevant *there*; another version can make some of it decisive again. Here the full row
  contained four other map entries, one of which fails under the old grammar for an unrelated
  reason — so the old grammar rejects the row for a cause the core no longer carries, while it
  accepts the core outright (under the old rule `map_entry := annotation_value "=>" annotation_value`
  the key `W => %RXCy` is itself a legal lambda, which the new `map_key` excludes).

  One step from publishing a false regression against a closed, gate-checked leaf.
reverify: "git show dcc2e2d8:grammars/semantic_annotation.ebnf > rust/target/kmv_pre.ebnf; printf '%s' '@handles    :   { FyT  (  ) => 0O527,   #{    } =>  {  _LdSh   ,    qcM2    , ECbN }   =>   (   ) , W  =>   %RXCy   =>    72e10   ,    Ep(  )  =>GvDx.O7qtt  (     )   ,disabled  =>    0xBF   }' > rust/target/kmv_row.txt; printf '%s' '@handles : { W => %RXCy => 72e10 }' > rust/target/kmv_core.txt; for g in rust/target/kmv_pre.ebnf grammars/semantic_annotation.ebnf; do for i in kmv_row kmv_core; do ./rust/target/debug/ast_pipeline $g --interpret-parse rust/target/$i.txt >/dev/null 2>&1 && r=accept || r=reject; echo \"$(basename $g) $i -> $r\"; done; done   # the CORE flips across vintages, the ROW does not"
---

A minimal reproducer feels like a distilled version of its input. It is not. It is the **residue of a
search that ran against one specific artifact**, and every byte it dropped was dropped because *that*
artifact did not need it. Move to another version and some of those bytes can matter again.

## What went wrong

| input | pre-`map_key` grammar | HEAD |
|---|---|---|
| the full 192-byte row `#0875` | reject | reject |
| the 48-byte core shrunk from it | **accept** | reject |

Read the core row alone and the conclusion is *"the intervening change narrowed the language"* — a
regression in a leaf that had published, and gate-checked, `0 newly rejected`. Read the full row and
there is nothing to explain: it was rejected before and after.

Both facts are true. They are about different inputs.

## Why it is systematic, not bad luck

Shrinking is a search for a **locally minimal** input with the same verdict. Locality is defined by
the artifact under test. Concretely, here:

- the full row carries five map entries; under the OLD grammar one of the *other* entries fails, so
  the row is rejected for a cause the core does not contain;
- the core, isolated, is *accepted* by the old grammar, because the old
  `map_entry := annotation_value "=>" annotation_value` lets the key itself be a lambda (`W => %RXCy`),
  a reading the new `map_key` deliberately excludes.

So the reduction dropped the very entry that explained the old verdict. Nothing about that is a
mistake in the shrinker — it is what minimisation *means*.

## The practice

1. **Attribute on the ROW, not on the core.** A per-input accept-set ledger (`widen`/`narrow`, rows
   named) is the instrument for "did this change move the language". A core is an aid to
   *understanding* one verdict, never evidence about a delta.
2. **Stamp a core with the artifact vintage it was shrunk under**, the same way a derived corpus is
   stamped ([[a-corpus-generated-from-the-artifact-under-test-is-part-of-the-measurement]]).
3. **Re-shrink when the artifact moves.** Carrying a core forward across a fix is reusing the output
   of a search whose inputs changed.
4. When a core and its row disagree across two versions, **the row wins** — and the disagreement is
   itself worth recording, because it usually names the second, unrelated cause the reduction removed.

Related: [[a-control-that-clears-your-hypothesis-has-not-cleared-the-symptom]] ·
[[a-corpus-generated-from-the-artifact-under-test-is-part-of-the-measurement]] ·
[[an-inherited-residual-may-have-died-with-the-fix-that-came-before-it]]
