---
name: feedback-a-named-call-site-is-a-category-of-call-sites
description: DISCIPLINE (2026-08-11, SV-CORPUS-GRAD.12c.1) — when a defect is diagnosed at a NAMED call site, the site is an instance and the fix scope is a CATEGORY. Enumerate every call of the same API and classify each by what the file IS, because the classification decides which sites must NOT change as much as which must. Measured: 46 text readers in one crate — 7 read user source (fix), 4 read grammar text (route), 35 read files PGEN itself writes (⛔ changing those would HIDE corruption). The enumeration also exposed two readers of the same concern holding OPPOSITE behaviours for their whole life, and a second unrelated defect one layer down.
metadata:
  node_type: memory
  type: feedback
id: feedback-a-named-call-site-is-a-category-of-call-sites
title: A defect named at one call site is a defect of a CATEGORY of call sites — enumerate and classify before you fix the named one
date: 2026-08-11
answers:
  - "the bug report names a file and line — is fixing that line the fix?"
  - "should I sweep every call of this API when one of them is wrong?"
  - "how do I decide which call sites of a fixed API to change"
  - "why did a two-line fix at the named site turn out to be wrong scope"
  - "how can two parts of one codebase disagree about the same thing for years"
  - "is a find-and-replace over an API a safe way to apply a fix"
reverify: cat docs/tasks/artifacts/sv_corpus_grad/source_text_readers/enumeration.md   # the categorised census; regenerate its raw input with the two greps at the top
---

**The founding case.** `SV-CORPUS-GRAD.12c` diagnosed a real defect and named its location exactly:
`rust/src/bin/parseability_probe.rs:504` and `:556` call `std::fs::read_to_string`, which refuses
any file that is not valid UTF-8 — so thirteen ISO-8859-1 files of the SystemVerilog corpus (twelve
carrying a single `0xA9`, the `©` in a copyright comment) produced **no parse verdict at all**. The
diagnosis was correct and the location was correct. Fixing those two lines would still have been
the wrong change.

The leaf insisted the enumeration of source-text readers come **first**, as a deliverable in its own
right. It found 46 text-reading sites in `rust/src/`, in three categories:

| category | what it reads | sites | correct action |
|---|---|---:|---|
| A | USER SOURCE TEXT handed to a parser | 7 | **fix** — a non-UTF-8 byte is the input's property |
| B | GRAMMAR text (`.ebnf`) | 4 | **route** — same defect, different family, different risk |
| C | files PGEN itself WRITES (JSON, generated Rust, reports, dumps) | 35 | ⛔ **must NOT change** |

Category C is the one that matters. Those files are UTF-8 *by construction* — PGEN wrote them — so
a decode failure there is not "the input used another encoding", it is **corruption or a truncated
write**, and it has to stay loud. A blanket `read_to_string` → tolerant-reader sweep, which is what
"fix the reader" invites, would have made 35 sites silently swallow corrupted artifacts to fix a
comment byte.

## The two things the enumeration found that no amount of care at the named site would have

1. **Two readers of the same concern, holding OPPOSITE behaviours, for their whole life.**
   `parseability_probe` REFUSES a non-UTF-8 file. `sv_preprocessor.rs` had been decoding it
   *lossily* since it was written — U+FFFD per bad byte, behind a warning. The same thirteen files
   were unreadable on one path and silently mangled on the other, and **nothing in the repository
   said so**, because nothing had ever put the two side by side. Fixing only the named site would
   have left the mangling in place and widened the divergence.

2. **A second, unrelated defect one layer down, found by wiring the second reader.** The new test
   asserted a Latin-1 `©` survives the preprocessor. It failed — `©` arrived as `Â©` — and the
   reader was exonerated immediately because the same corruption reproduces on input that is
   *valid UTF-8*. Root cause: eight scanners walking the line as bytes and re-emitting with
   `out.push(bytes[i] as char)`, a Latin-1 promotion rather than a UTF-8 decode. Pre-existing,
   invisible for its whole life, because nothing downstream ever asserted on the *content* of the
   preprocessed text.

## The discipline

> **A named call site is an instance. The fix scope is the category, and the category boundary must
> be written down before the first line changes — because it names the sites that must NOT change
> just as much as the ones that must.**

Concretely, when a defect is diagnosed at `file.rs:NNN` in a call to some shared API:

1. **Enumerate every call of that API**, mechanically (`grep -rn`), and publish the count.
2. **Classify each site by what the FILE IS**, not by what the call looks like — "who authored the
   bytes this reads?" is the question, and it is invisible at the call site.
3. **State the disposition per category**, including the *do-not-touch* one and why touching it
   would be harmful. That sentence is the deliverable, not a formality.
4. **Route the categories you will not fix now to CREATED leaves** — naming an owner is not routing
   (`DOCTRINE-GAP-OWNERSHIP`).
5. **Expect the enumeration to find something.** Twice out of two here, the enumeration — not the
   review, not the diagnosis — was what surfaced the divergence.

⚠️ **Honest limit:** this is a discipline, not a gate. Nothing mechanically requires an enumeration
before a shared-API fix. The tripwire is the shape of the instruction: *"the bug is at file:line"*
is a report about an instance, and the scope question has not been asked yet.

Sibling of [[feedback_classify_referents_by_requirement]], which says `grep -l` yields the candidate
set and never the *cost*; this one says the same grep yields the candidate set and never the *fix
scope* — and adds the direction that record does not cover: **classification is what stops a sweep
from doing harm**. Also a sibling of [[feedback_enumerating_instrument_must_refuse]] (total
classification, refuse on the unmatched) applied to call sites instead of to an instrument's input.
