# How To Use This Book

This book is the main surface the outside world reads to understand PGEN. It is
written for **two different readers**, and it is split into two clearly-marked
parts so each reader knows exactly where to look — and what they can safely skip.

## The two parts

### Part I · Using PGEN — *for everyone*

This is for you if you want to **build, run, and consume** PGEN parsers: write a
grammar, add annotations, generate a parser, debug it, embed it, and read its
output. It explains *what PGEN does and how to use it* in plain terms, with
examples and commands — no engine internals, no Rust.

**If you only use PGEN, Part I is all you need. You can stop at the end of it.**

### Part II · Inside PGEN — *for those who modify the internals*

This is for the smaller audience that is **not afraid to change how PGEN itself
works** — the parsing engine, the code generator, the performance and termination
machinery, the quality gates. It goes *all-in* on **how PGEN works internally**,
in much more depth than Part I, so a contributor can reason about the engine
before touching it.

Part II deliberately explains mechanisms in prose and diagrams rather than pasting
Rust: if you want the last 100% of detail, the code is the source of truth and the
[Source Map](source-map.md) points you straight to it. A reader who just wants to
*use* PGEN does not need to read Part II at all — the "scary internals" live here
on purpose, fenced off, so they never get in a normal user's way.

> **Rule of thumb:** if a sentence would only matter to someone editing PGEN's own
> source, it belongs in Part II. If it helps someone *use* PGEN, it belongs in Part I.

## Suggested reading path

**Using PGEN (Part I):** Platform Overview → Getting Started → User-Facing Surfaces
→ CLI and Workflows → Debugging With `parseability_probe` → Annotation System → The
Semantic Store → Stimuli and Quality → Parser Families → Embedding and Downstream
Integration → Contracts and Support → Roadmap and Live Status.

**Inside PGEN (Part II):** Developer Architecture → Inside the Parser: Termination &
Performance → Academic Foundations → Parser Hooks → Quality and Closure Model →
Operations and Governance → Source Map.

## Live-Document Rule

This book should evolve with the project. When a major user-facing surface, contract, roadmap, or architecture seam changes, the relevant chapter should be updated so the curated learning path remains truthful.

The target state is not static documentation. The target state is an always-current, outward-facing, comprehensive documentation system.

## Maintenance Rule

When a change affects a surface already represented by the book, the book should be updated in the same implementation wave.

The goal is:

- no stale "book says X but repo does Y" drift,
- no treating the book as optional polish,
- no hiding important behavior or rationale in internal continuity docs when it belongs in the public book,
- no leaving important project domains permanently outside the book just because deeper reference docs also exist,
- no replacing curated chapters with unstructured note dumps.

In repository workflow terms, the maintained proof surface for this book is:

```bash
make -C rust SHELL=/bin/bash mdbook_docs_gate
```
