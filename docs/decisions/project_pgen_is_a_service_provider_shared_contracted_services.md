---
name: project-pgen-is-a-service-provider-shared-contracted-services
description: DIRECTOR DIRECTION (2026-10-19, verbatim) — PGEN is to become a very high quality parsing, compilation and/or elaboration SERVICE PROVIDER for downstream consumers; its parsers share services like lego blocks, and those shared services are well-defined, well-contracted, and usable by downstream projects too. The structure is a SERVICE REGISTER (contract, stability axes, neutrality tier, consumers, currency gate per service); the first block is the svpp expander (SVPP-EXPANSION.9).
id: project-pgen-is-a-service-provider-shared-contracted-services
title: "PGEN is a service provider: its parsers share contracted services, and downstream projects may use those services too"
date: 2026-10-19
evidence: "measured 2026-10-19: the engine holds neutral services (preprocessing_stage, the semantic store, the parse harness and dedicated parse stack, the stimuli generator, the certificate, the AST/JSON carrier, the registry-derived embedding API) — every one internal; only the parsers are contracted (docs/contracts/PGEN_*_PARSER_INTEGRATION_CONTRACT.md); the one service contract set (PGEN_SEMANTIC_STORE_*) has no published downstream surface; a downstream project cannot reach the preprocessing stage, a source map or the stimuli generator through a versioned API (SVPP-EXPANSION.9's measurement)"
reverify: "ls docs/contracts | grep -c -v -i 'parser_integration\\|ledger\\|reporting\\|revision'   # the non-parser contracts (4 semantic-store documents at the time of writing); and: ls docs/tasks/PGEN-SERVICES.md"
answers:
  - "what is PGEN for, beyond shipping parsers"
  - "may a downstream project use a PGEN internal service such as preprocessing, the semantic store or stimuli generation"
  - "what does well-contracted mean for a PGEN service"
  - "why does the svpp expander live behind a neutral socket instead of inside the SystemVerilog family"
  - "is adding a shared service a governance lane or feature work"
---

# PGEN is a service provider: its parsers share contracted services, and downstream projects may use them too

- **Date:** 2026-10-19 · **By:** the director (verbatim below) · **Recorded by:** `PGEN-SV-CORPUS-GRAD-0468`
- **Tree:** `PGEN-SERVICES` (the program); the first block is `SVPP-EXPANSION.9`
  ([[project_svpp_expansion_is_a_service_over_the_svpp_ast]]). Extends [[project_north_star]]'s parser-neutrality
  constraint from *the engine names no language* to *the engine's services are published blocks*.

## The direction (verbatim)

> "for all of its present and future parsers, I would like PGEN become a truly a very high quality parsing, or
> compilation and/or elaboration service provider for downstream consumers, projects. If PGEN parsers can share
> services other PGEN parsers, and become like lego blocks where some of this shared services can themselve be
> leveraged by downstream consumers too, that's a win on many account. I am for this kind of structure where common
> parts can be turn into services allowing PGEN parsers to share services and other projects to use them should they
> also need those well defined, well contracted PGEN services."

> "I really want PGEN to become like the Swiss-Army-Knife of all things parsing, compilation and eloboration, made of
> very well contracted parts or services other downstream projects can seamlessly leverage for their own benefit."

## What was true when it was given (measured)

The engine already holds neutral services behind the family components — the preprocessing stage socket, the semantic
store, the parse harness and dedicated parse stack, the stimuli generator, the certificate, the AST/JSON carrier, the
registry-derived embedding API — and `ENGINE-UNIVERSAL-SERVICES` moved every family residue out of the engine. Every
one of them is internal: only the parsers are contracted and reachable from outside; the semantic store has contract
documents but no downstream surface; a downstream project cannot obtain preprocessed text, a source map or generated
stimuli through a versioned API.

## The decision

1. **PGEN is a service provider**, not only a parser vendor: parsing, and in time compilation and elaboration, offered
   as well-defined, well-contracted services.
2. **Shared services are lego blocks**: a common part of two parsers becomes a service both consume through one
   contracted boundary — never a copy in each family.
3. **A block downstream can use**: a service with a contract, stability axes (service release + output schema), a
   neutrality tier, named consumers and a currency gate is published; a service lacking any of these is a component,
   not a block, and is listed as such.
4. **The register is the structure**: `PGEN-SERVICES` holds the inventory of services, each with those five columns,
   measured before it is claimed; the first block built under it is the svpp expander.

## Why

- Shared, contracted parts are how quality compounds: a service proven on one parser's corpus and certificate is
  proven for every consumer, and a defect fixed once is fixed everywhere.
- Downstream projects integrate what they can reach; an internal service, however good, is not a product.

## How to apply

Adding a shared service is FEATURE work, not a governance lane ([[feedback_prefer_feature_work_over_governance_lanes]]
is unaffected): it lands with its contract, its axes, its gate and a consumer, in one slice. A family capability climbs
the neutrality ladder first ([[feedback_neutrality_ladder_before_a_family_capability]]): its neutral part becomes the
block, the family keeps only what diverges by language. The parsers' own contracts stay the model for every service
contract.
