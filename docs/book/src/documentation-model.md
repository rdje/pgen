# Documentation Model

PGEN uses more than one documentation surface, but they do not all serve the same purpose.

This chapter explains the intended split so readers, contributors, and future sessions do not confuse:

- what the world should read to understand PGEN,
- what deep technical detail exists behind that public surface,
- and what internal continuity artifacts exist only to keep repo work resumable.

## The Three Documentation Layers

### 1. The Book

The book under `docs/book/` is the primary public documentation surface.

Its job is to explain:

- what PGEN does,
- how to use it,
- how it works,
- why it is designed the way it is,
- how parser families, proof lanes, and quality doctrine fit together,
- and how users and contributors should navigate the platform.

The book is not meant to be a thin welcome page. It is intended to become the comprehensive outward-facing documentation system for the project.

### 2. Contracts and Reference Docs

The maintained docs under `docs/contracts/`, `docs/reference/`, and selected `rust/docs/` files are the deep authoritative detail behind the book.

They are where PGEN keeps:

- parser-family integration contracts,
- support and bug-reporting rules,
- normative specs,
- roadmap steering,
- matrices,
- deep implementation references,
- and detailed Rust/API contracts.

These docs are authoritative, but they are not the main teaching surface. The book should guide readers into them when exact detail is needed.

### 3. Continuity Docs

The continuity docs exist for internal session recovery and live repo-state continuity:

- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `LIVE_ACHIEVEMENT_STATUS.md`
- `MEMORY.md`
- plus workflow docs such as `COMMIT.md` and `SESSION_BOOTSTRAP.md`

These files are intentionally operational. They preserve live state, implementation waves, commit workflow, and crash-recovery context.

They are important, but they are not the primary outward-facing documentation surface.

## What Belongs In The Book

Anything that an external reader needs in order to genuinely understand or master PGEN should eventually have first-class representation in the book.

That includes:

- platform concepts,
- CLI and workflow entrypoints,
- parser-family surfaces,
- annotation and semantic steering behavior,
- stimuli-generation capabilities,
- proof and closure doctrine,
- downstream integration,
- architecture at the level contributors need to reason well,
- and the rationale behind major design choices.

If a surface matters often enough that users or contributors keep needing to rediscover it, it belongs in the book.

## What Does Not Belong In The Book

The book should not become:

- a raw changelog,
- an implementation scratchpad,
- a crash-recovery notebook,
- a dump of every temporary investigation,
- or an uncurated mirror of the repository tree.

The book should stay readable and structured. But readability is not an excuse to hide important project behavior in internal notes.

## Coverage Closure Rule

The target state is not “a nice overview plus many scattered docs.”

The target state is:

- the book explains every important aspect of the project,
- deep contract/reference docs provide exact supporting detail,
- continuity docs preserve internal operational state,
- and the three layers stay aligned.

In practice, that means:

- if a covered surface changes, update the relevant book chapter in the same wave,
- if a new important surface appears, add a new section or chapter,
- if a topic is still only explained in continuity docs, that is a sign the public book likely still has a gap.

## Reader Guidance

If you want to understand PGEN from the outside in:

1. start with the book,
2. follow the links and source-map references into contracts/reference docs for deeper detail,
3. consult continuity docs only when you specifically need live internal state or repo workflow continuity.

That is the intended documentation model for the project.
