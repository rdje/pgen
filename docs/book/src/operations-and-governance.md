# Operations and Governance

PGEN relies on disciplined operational docs, not just code.

## Continuity Docs

These four files are the live continuity spine:

- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `LIVE_ACHIEVEMENT_STATUS.md`
- `MEMORY.md`

## Session and Commit Workflow

Two operational docs matter especially for future sessions:

- `SESSION_BOOTSTRAP.md`
- `COMMIT.md`

They capture:

- how a fresh session should ramp up,
- which docs must be reviewed before commit,
- what must be included in post-commit reporting,
- how live-status communication stays consistent.

## Documentation Governance

The intended split is:

- the book is the primary public documentation surface,
- the continuity docs are internal continuity and crash-recovery surfaces,
- the contracts and reference docs are the deep authoritative details behind the book.

That split keeps the repository teachable without losing high-signal live state.

## Book Maintenance Doctrine

The book is not a one-time scaffold. It is part of the maintained repo surface.

That means:

- if a change affects a user-facing or developer-facing surface already covered by the book, update the relevant chapter in the same wave,
- if a new important surface appears often enough to matter, add a new section or chapter,
- use the book to curate and teach, not to mirror every raw implementation note,
- but treat the book as the place where the world should be able to understand what PGEN does, how it works, and why it is designed that way.

The maintained proof lane for this doctrine is:

```bash
make -C rust SHELL=/bin/bash mdbook_docs_gate
```
