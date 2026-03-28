# PGEN_RELEASED_PARSER_BUG_LEDGER.md

## Purpose
Track every bug reported against a released PGEN parser family until it is either fixed, proven invalid, or explicitly deferred with a documented reason.

This is a live operational ledger, not an archival narrative.

GitHub is optional. This ledger should be the canonical parser-side tracker inside the PGEN git repo while any number of downstream consumer repos may keep their own local tracking references.

## Tracking Rule
- Every downstream bug report against a released parser family must receive a stable report ID.
- Every accepted report must link back to a reproducible artifact bundle captured using `PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.
- A bug is not considered fully closed until:
  - the root cause is identified,
  - the fix is landed,
  - executable proof exists,
  - and the ledger entry is updated with the fix provenance.

## Per-Parser Indexing Rule
- The primary index for this ledger is `Parser Family/Profile`.
- Report IDs should carry a parser-family prefix whenever practical, for example:
  - `REGEX-0001`
  - `VHDL-0003`
  - `SV-0012`
- If multiple downstream consumers report the same parser-side root cause, prefer one canonical ledger row with multiple downstream tracking refs instead of losing that linkage across separate isolated notes.
- If two reports look similar but differ in parser profile, reproducer class, or root cause, keep separate rows and cross-reference them in `Notes`.

## Required Fields
- `Report ID`
- `Parser Family/Profile`
- `Reported Against Parser Release`
- `Reported Against Contract Version`
- `Downstream Consumer(s)`
- `First Reported`
- `Current State`
- `Downstream Tracking Refs`
- `Reproducer Bundle`
- `Root Cause`
- `Fix Proof`
- `Fixed In`
- `Notes`

## State Meanings
- `Reported`
  - issue has been received but not yet reproduced locally
- `Reproduced`
  - PGEN can reproduce it from the supplied bundle
- `Root Caused`
  - the mechanism is understood, but the fix is not landed yet
- `Fix In Progress`
  - an implementation is underway
- `Fixed Pending Release`
  - a fix is landed and validated, but downstream release/adoption is still pending
- `Released`
  - the fix is landed, validated, and available to downstream consumers
- `Rejected`
  - not actually a PGEN bug, duplicate, or outside the contracted parser surface
- `Deferred`
  - acknowledged but intentionally postponed with a documented reason

## Closure Rule
Each `Released` row should point at:
- the reproducer artifact bundle
- the commit and/or parser release version containing the fix
- the regression test or gate proving the bug stays closed

## Live Ledger

| Report ID | Parser Family/Profile | Reported Against Parser Release | Reported Against Contract Version | Downstream Consumer(s) | First Reported | Current State | Downstream Tracking Refs | Reproducer Bundle | Root Cause | Fix Proof | Fixed In | Notes |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| _none yet_ | | | | | | | | | | | | No released-parser bug reports have been logged in this ledger yet. |
