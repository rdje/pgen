# Roadmap and Live Status

PGEN uses two different but connected planning surfaces:

- the living roadmap,
- the live status tracker.

Understanding the difference between them is important, because they answer different questions.

## The Roadmap Answers “What Are We Building Toward?”

The roadmap is the forward-looking contract for the project. It captures:

- mission and doctrine,
- execution preferences,
- retained strategic decisions,
- deferred backlog ordering,
- parser-family steering,
- the current preferred next moves.

In other words, the roadmap is where PGEN explains how it wants to advance.

## The Live Tracker Answers “Where Are We Right Now?”

The live tracker is the authoritative status surface for current closure. It uses only four labels:

- `Done`
- `Mostly Done`
- `In Progress`
- `Not Started`

It is deliberately stricter than casual status language. A row should not be marked `Done` unless the project has a machine-checkable proof surface strong enough to justify it.

## Why Both Surfaces Exist

If PGEN only had the roadmap, it would be hard to know what is actually closed today.

If PGEN only had the live tracker, it would be hard to understand the direction, doctrine, and queued platform work.

The roadmap provides direction. The live tracker provides present-tense truth.

## A Critical Distinction: Phase Progress vs Family Maturity

The live tracker separates:

- major roadmap phases,
- parser-family maturity.

That matters because a phase can be advancing even while a parser family touched by that phase is not yet closed, and a closed family can stay closed while broader roadmap work continues elsewhere.

This is especially important in PGEN because the same proof-first quality doctrine applies across families, but they are not all at the same proof depth yet.

## How To Read Current PGEN Status

When you want the honest current picture:

1. read `LIVE_ACHIEVEMENT_STATUS.md` for the actual tracked snapshot,
2. read `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md` for the rationale and next steps behind that snapshot,
3. use the family-specific contracts and gates when the question is about one concrete parser surface.

For platform lanes that are important but broader than one parser family, the main roadmap may also point to dedicated side roadmaps. The linter-enablement lane and the compiler-and-elaborator workbench lane are current examples: both are tracked in the main roadmap, but their doctrine, API shape, and milestone order live in maintained reference documents of their own.

## How Book Readers Should Use These Surfaces

The book should explain the model and help readers navigate it. But the exact current truth still lives in the live tracker and roadmap documents themselves.

So the right reading pattern is:

- use this chapter to understand the system,
- use the live tracker for exact current status,
- use the roadmap for exact current steering.

## Primary Source Docs

- `LIVE_ACHIEVEMENT_STATUS.md`
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
- `docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md`
- `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`
- `README.md`
