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

- the book is the curated mastery surface,
- the continuity docs are the live operational truth,
- the contracts and reference docs are the deep authoritative details behind the book.

That split keeps the repository teachable without losing high-signal live state.
