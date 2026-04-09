# How To Use This Book

This book is ordered from broad orientation to deeper implementation detail.

It is also intended to become the main documentation surface that the outside world reads to understand PGEN.

## Suggested Reading Path

1. `Platform Overview`
2. `Documentation Model`
3. `Getting Started`
4. `User-Facing Surfaces`
5. `CLI and Workflows`
6. `Annotation System`
7. `Embedding and Downstream Integration`
8. `Parser Families`
9. `Roadmap and Live Status`
10. `Quality and Closure Model`
11. `Stimuli and Quality`
12. `Contracts and Support`
13. `Developer Architecture`
14. `Operations and Governance`

## Reader Modes

### If you are new to PGEN

Focus first on:

- the platform model,
- the standard commands,
- the active parser families,
- the user-facing guide and roadmap.

### If you already use PGEN

Use this book to:

- revisit exact contract boundaries,
- understand how new proof lanes fit into the broader doctrine,
- discover related surfaces you may have been using only indirectly.

### If you are developing PGEN itself

Use this book as the high-level entrypoint before diving into the deeper reference docs and code. It is designed to keep future sessions from starting from scratch on the same conceptual questions.

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
