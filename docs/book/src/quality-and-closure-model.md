# Quality and Closure Model

One of the most important things to understand about PGEN is that it is not satisfied with "the parser exists."

PGEN is built around a proof-first closure model.

## The Core Idea

A parser family is not considered mature just because:

- the grammar parses,
- the generated parser compiles,
- a few happy-path samples work.

Instead, PGEN aims to close the gap between "it seems to work" and "we have machine-checkable reasons to trust it."

## What Closure Means In Practice

The exact proof surface varies by family and maturity, but the general doctrine includes:

- EBNF-backed parser generation,
- generated artifacts that are reproducible and tracked,
- stimuli generation,
- parser/stimuli round-trip or comparable proof where applicable,
- coverage and gap analysis,
- deterministic replay,
- family-specific contracts and support boundaries,
- executable gates backing published claims.

This is why the repository talks so much about gates, contracts, and tracked evidence. They are not auxiliary paperwork; they are part of the product.

## Why PGEN Works This Way

PGEN targets domains where parser behavior materially affects downstream tooling and trust:

- HDL tooling,
- regex engines,
- annotation-driven parser platforms,
- future high-rigor language integrations.

In those environments, parser novelty is not enough. Predictability, observability, and repeatable proof matter.

## Closure Is Normalized Across Families

PGEN does not use different quality philosophies for different parser families.

The doctrine is the same across EBNF-based families:

- regex,
- VHDL,
- SystemVerilog,
- annotation grammars,
- Phase S grammars,
- future families.

What differs is not the quality bar, but how much of the proof surface has already landed.

## Why Status Labels Stay Conservative

This is why `LIVE_ACHIEVEMENT_STATUS.md` can keep a family at `Mostly Done` even when it already looks strong to a casual reader. The status labels are meant to reflect proof depth, not enthusiasm.

Likewise, a family can remain `Done` while still receiving maintenance releases or syntax widening, as long as the published closure doctrine for that family remains satisfied.

## How To Read PGEN Claims

When PGEN says something is closed or production-ready, the right next question is:

"What executable proof surface backs that claim?"

That is the correct lens for:

- gates,
- contracts,
- aggregate reports,
- closure rows,
- maintenance releases.

## Primary Source Docs

- `README.md`
- `LIVE_ACHIEVEMENT_STATUS.md`
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
- `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`
