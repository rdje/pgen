---
name: feedback_prefer_a_vetted_dependency_over_reimplementing_it
description: "STANDING DIRECTIVE (director 2026-08-17, session #242, ruling on the sha2 build-dependency added by ENGINE-UNIVERSAL-SERVICES.24 slice 2): when a well-known, vetted implementation of a SOLVED problem exists, take the dependency — do not hand-roll it to keep a dependency count low. 'Recreating the wheel is rarely a good solution.' Dependency-count minimalism is not a quality argument, and a hand-rolled standard algorithm is a durable review cost every future reader must re-audit. This does NOT license casual dependency growth: the rule applies to SOLVED, SPECIFIED problems with a canonical implementation, and the cost is still stated and measured before it lands."
id: feedback-prefer-a-vetted-dependency-over-reimplementing-it
title: A solved, specified problem with a canonical implementation is not worth re-implementing to save a dependency — price the dependency, do not refuse it on count alone
date: 2026-08-17
evidence: "Director ruling 2026-08-17 session #242 on ENGINE-UNIVERSAL-SERVICES.24 slice 2 (PGEN-ENGINE-UNIVERSAL-SERVICES-0066): sha2 0.10.9 landed as the crate's first [build-dependencies] entry, 8 new crates in the build graph, against an offered ~90-line self-contained SHA-256 with NIST vectors. rust/Cargo.toml carries both the dependency rationale and the [profile.*.build-override] measurement (5.5 s -> 0.39 s, 14.1x) that the decision had not anticipated."
reverify: "grep -q '^sha2 = ' rust/Cargo.toml && grep -q 'build-override' rust/Cargo.toml && echo VETTED-DEP-RULING-LIVE"
answers:
  - "should I add a crate for this or write it myself"
  - "is it worth hand-rolling a hash / encoder / small algorithm to avoid a dependency"
  - "how do I decide between a vetted dependency and a self-contained implementation"
  - "is adding a dependency a good enough objection on its own"
  - "what do I have to state before adding a new crate to this project"
  - "when is a dependency NOT acceptable here"
metadata:
  node_type: memory
  type: feedback
  director_directive: true
  created: 2026-08-17
---

**Director, verbatim (2026-08-17, session #242):**

> *"I am ok with the sha2 dependency … Recreating the wheel is rarely a good solution."*

## What was asked

`ENGINE-UNIVERSAL-SERVICES.24` slice 2 needed a sha256 computed in `rust/build.rs` that a
Python consumer re-computes with `hashlib` and compares byte for byte. Two options were live and
the engineer surfaced the trade rather than deciding it silently:

| | `[build-dependencies] sha2` | ~90 lines of self-contained SHA-256 in `build.rs` |
|---|---|---|
| correctness argument | the RustCrypto reference implementation | NIST FIPS-180-4 vectors asserted at build time + the live cross-check against `hashlib` |
| new crates in the build graph | **8** (`sha2` + 7; `cfg-if` and `libc` were already present) | 0 |
| durable review cost | none | every future reader must satisfy themselves the hand-rolled hash is right |

The engineer took the dependency and offered to swap it. **The director ruled: keep it.**

## The rule

> **A solved, specified problem with a canonical implementation is not worth re-implementing to
> save a dependency.** Take the vetted one.

Dependency-count minimalism reads as discipline and usually is not: it trades a one-time,
externally-audited cost for a permanent, locally-audited one. The hand-rolled version does not
become correct because it is small — it becomes *your* correctness argument, forever, re-litigated
by every reader who meets it.

⛔ **This is not a licence for casual dependency growth.** It applies where all of these hold:

1. The problem is **solved and specified** — a published algorithm or standard, not a
   project-shaped behaviour. (SHA-256 qualifies. "A small helper that happens to exist on
   crates.io" does not.)
2. There is a **canonical / obviously-dominant implementation**, so the choice is not itself a
   research task.
3. The cost is **stated and measured** before it lands, not waved at: which crates, how many are
   genuinely new, and what it does to build time. `.24` slice 2 counted 8 new crates and then
   measured a cost the decision had NOT anticipated — cargo builds build scripts at
   `opt-level = 0` in *every* profile, so hashing 236 MB took **5.5 s** until
   `[profile.*.build-override] opt-level = 2` made it **0.39 s**.

Where those do not hold, the existing posture is unchanged: the crate's runtime dependency list is
deliberately small and each entry carries its rationale in `rust/Cargo.toml`.

## How to apply

- When a slice needs a standard algorithm (hashing, encoding, compression, date arithmetic,
  parsing a published format), **reach for the vetted crate first** and price it in the leaf.
- Do not raise "it adds a dependency" as an objection on its own. Pair it with the concrete cost —
  crate count, build time, cold-clone/offline impact, licence — or it is not an argument.
- Do raise it when the dependency would be **load-bearing at runtime for consumers**: a library
  must not impose global state on embedders, which is why `mimalloc` is optional and
  non-default here. That is a different question from "how many crates are in the graph".
- ⭐ **Surfacing the trade was correct even though the answer was "keep it".** A dependency
  decision is a director call by default ([[feedback_always_signoff_decisions]]); what this record
  removes is the need to ask *again* for the same shape.
- Related: [[feedback_read_prior_art_before_designing]] (the same instinct one level up — do not
  invent a surface the project already has), [[feedback_answer_your_own_technical_questions]]
  (this ruling converts a recurring question into an answered one),
  [[a-one-sided-gate-rewards-the-instrument-that-under-reports]] (the leaf this arose from).
