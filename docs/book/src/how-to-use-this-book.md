# How To Use This Book

This book is ordered from broad orientation to deeper implementation detail.

## Suggested Reading Path

1. `Platform Overview`
2. `Getting Started`
3. `User-Facing Surfaces`
4. `CLI and Workflows`
5. `Annotation System`
6. `Embedding and Downstream Integration`
7. `Parser Families`
8. `Roadmap and Live Status`
9. `Stimuli and Quality`
10. `Contracts and Support`
11. `Developer Architecture`
12. `Operations and Governance`

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

The target state is not static documentation. The target state is an always-current mastery surface.

## Maintenance Rule

When a change affects a surface already represented by the book, the book should be updated in the same implementation wave.

The goal is:

- no stale "book says X but repo does Y" drift,
- no treating the book as optional polish,
- no replacing curated chapters with unstructured note dumps.

In repository workflow terms, the maintained proof surface for this book is:

```bash
make -C rust SHELL=/bin/bash mdbook_docs_gate
```
