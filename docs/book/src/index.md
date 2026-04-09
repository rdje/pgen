# Welcome

This book is the live, curated mastery surface for PGEN.

PGEN is a production-focused parser and stimuli generation platform built around EBNF-driven parser generation, AST shaping through return annotations, semantic steering, deterministic quality gates, and cross-family proof surfaces. The goal of this book is to make that platform learnable in layers:

1. understand what PGEN is and how to use it,
2. learn the user-facing commands, contracts, and parser families,
3. dive into the stimuli, coverage, and quality model,
4. move into the Rust architecture and operational rules used to evolve the system.

This book is intentionally a live document. As PGEN evolves, its chapters, sections, and learning path are expected to evolve too. The book is not meant to freeze the project; it is meant to keep the project understandable while it changes.

## What This Book Covers

- the mental model of the platform,
- the main CLI and workflow surfaces,
- the annotation system and why it is core to the platform,
- the embedding and downstream integration model,
- the currently relevant parser families,
- the roadmap and live-status model used to steer and evaluate the project,
- the stimuli-generation and proof doctrine,
- downstream contracts and support expectations,
- architecture and implementation guidance for contributors,
- operational governance for sessions, commits, and release-quality work.

## What Remains Outside The Book

Some documents are still best treated as operational continuity sources rather than book chapters:

- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `LIVE_ACHIEVEMENT_STATUS.md`
- `MEMORY.md`

Those files remain authoritative for live progress, continuity, and recent implementation waves. The book complements them; it does not replace them.
