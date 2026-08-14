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

The book itself is organized into **two clearly-separated parts** (see
[How To Use This Book](how-to-use-this-book.md)):

- **Part I · Using PGEN** — for everyone who uses PGEN. *What it does and how to use
  it*, with examples and commands, no internals.
- **Part II · Inside PGEN** — for contributors who modify the engine. *How PGEN works
  internally*, in depth, explained in prose (not pasted Rust). Normal users can skip it.

This split is deliberate: the deep "how it works" material is fenced into Part II so it
never gets in the way of someone who only wants to use PGEN, while still giving
would-be contributors a thorough, readable account of the internals before they reach
for the code.

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
- `docs/book/src/roadmap-and-live-status.md` (the published family-status view)
- `MEMORY.md`
- plus workflow docs such as `COMMIT.md` and `SESSION_BOOTSTRAP.md`

These files are intentionally operational. They preserve live state, implementation waves, commit workflow, and crash-recovery context.

They are important, but they are not the primary outward-facing documentation surface.

### 4. The README — a landing page, not a layer

`README.md` sits outside the three layers on purpose. It is the **stable landing page**:
the first thing a visitor reads, and the map to everything above. It is *not* a place
detail accumulates.

This is a governed contract, not a preference —
[`docs/reference/PGEN_README_STABILITY_POLICY.md`](../../reference/PGEN_README_STABILITY_POLICY.md),
adopted 2026-07-30 and enforced by the `README-STABILITY` doctrine
(`scripts/check_readme_stability.sh`) on every commit and every push.

**Keep in the README:** purpose, audience and top-level scope; prerequisites and one
minimal verified quick start; stable architecture at a glance; links to canonical
documentation; license and repository-level notices.

**Route everything else** to the home that owns it — gate recipes to
[The Gate Flow](gate-flow.md), operational procedure to
[Operations and Governance](operations-and-governance.md), the path inventory to
[Developer Architecture](developer-architecture.md), status to
`rust/test_data/grammar_quality/done_bar_family_register_v0.json`, history to `CHANGES.md`, rationale to `docs/decisions/`.

Change the README only when its **purpose**, **first-use path**, **top-level
architecture** or **canonical navigation** changes. Ordinary feature work updates the
canonical destination instead.

#### Why it is mechanically capped — and why *two* caps

The README reached **510 lines / 48,811 bytes** with nothing watching it, because both
guards that touched it looked elsewhere: one audits the doc *paths written inside* it,
the other audits which root markdown files *exist*. A README can triple in size with both
green. 55.6% of it had become a file inventory plus an operations manual, and one bullet
was **4,369 bytes on a single line**.

So the doctrine enforces a **line cap and a byte cap together** — they are complements,
not belt-and-braces. Neither wrapped prose nor very long lines can be allowed to bypass
the budget.

That is not a theoretical argument. This repository already had the line-only shape in
isolation, and it was measurably not binding: `scripts/check_memory_architecture.sh`
capped layer-A `MEMORY.md` at 60 lines *only*, and the file sat at **60 lines — passing,
exactly at the ceiling — and 138,403 bytes**. That is 2,306 bytes per line, with a single
line of 18,816 bytes: a *"bounded resume pointer"* that was a 138 KB document, green the
whole time. The same class appeared independently at `README.md:115` with no shared code
path, which is why this doctrine shipped with both caps on day one.

✅ **That bypass is now closed.** `README-POLICY.2` trimmed layer A to **39 lines /
5,720 bytes** and gave the `MEMORY-ARCH` doctrine the same two caps — see
[Layer A is capped the same way](#layer-a-is-capped-the-same-way) below.

⛔ **A cap is never raised to land new content.** Move the detail to its canonical home.
Raising one requires an explicit reviewed decision recorded in
`docs/tasks/README-POLICY.md` that the landing-page contract itself expanded.

#### Layer A is capped the same way

`MEMORY.md` is layer A of the [memory architecture](../../../MEMORY_ARCHITECTURE.md) — the
**resume pointer**: where we are *now*, the next action, anything in flight. Nothing else.
Like the README, it is a surface that decays by accretion rather than by error, so it is
governed the same way and by the same shape of guard.

What went wrong is worth naming precisely, because it is the failure mode to watch for:
the block headed *"Current state (OVERWRITE this block each update — do not append)"* had
accumulated **18 distinct sessions** and **81.3%** of the file. The instruction was right
there in the heading, and the file grew anyway — a rule with no check is a suggestion.

Before pruning anything, an ownership census resolved every entry against a durable layer,
because *a canonical home named from a plausible title is not a verified destination*:

| what the census found | count |
|---|---|
| entries whose slice id resolves to a commit subject (layer D) with the owning tree present (layer B) | 20 |
| entries with no slice id, resolved individually against a named destination | 14 |
| entries with **no durable home anywhere** — written to `docs/decisions/` *before* removal | 2 |

Those last two are the reason the census existed: a standing storage directive that lived
only in overwrite-only layer A plus an untracked harness-home file, and a dangling
`[[project_bedrock_spine_repo]]` link whose target did not exist. Both are now layer-C
records. One entry was deliberately **kept** — the regex oracle tuple, because the
`REGEX-ORACLE-ANCHOR-SYNC` doctrine anchors on it.

⭐ The defect was in the **portable standard**, not only in this deployment:
`MEMORY_ARCHITECTURE.md` §9's own reference check prescribed the line-only cap, so every
project adopting it inherited the same bypass. §6, §9 and §9.1 were corrected together, and
the neutral policy now sits at the repository root as
[`README_POLICY.md`](../../../README_POLICY.md) beside the other portable standards, with
this repository's *instance* of it under `docs/reference/`.

#### The layer-A byte cap was raised to 32,768 — and why that is not the anti-pattern

On 2026-08-14 the layer-A byte cap moved **7,168 → 32,768** by director ruling
(`README-POLICY.8`). The rule one paragraph above still stands, so the distinction matters:

> **A cap is never raised to LAND CONTENT. A cap may be raised to RESTORE HEADROOM, by an
> explicit reviewed decision, taken while the file is passing.**

`README-POLICY.2` set 7,168 as the post-trim size (5,720 B) plus ~25% headroom. That
headroom was spent. Measured over layer A's own git history at the time of the ruling:

| what was measured | value |
|---|---|
| last 40 layer-A commits — min / median / max bytes | 6,795 / 7,079 / **7,168 = the cap exactly** |
| commits sitting at ≥ 97% of the cap | **36 of 40** |
| commits within 68 bytes of the cap | 18 of 40 |
| headroom remaining at the ruling | **117 bytes** |
| updates whose net size change alone exceeded 117 bytes | 3 of the last 20 |

The telling one is not any single number, it is `git diff --numstat` over nine consecutive
commits: **`4 4`** — the same four lines rewritten in place, never grown. The cap had
stopped bounding the *layer* and started editing the *prose*. Authors were shaving bytes to
fit rather than deciding what belongs in layer A, which is the opposite of what the cap is
for, and it is exactly the state this project's own lesson card
`a-cap-with-no-headroom-is-a-cap-about-to-be-raised` predicts: *"the moment a cap blocks you
is the worst possible moment to decide policy about it."* The ruling was taken at the calm
moment that card asks for — layer A was **passing**, not blocked.

⚠️ **The honest structural cost, stated up front.** At 50 lines / 32,768 bytes the byte axis
permits ~655 B/line, so across the 7–32 KB band the **line cap is the only binding axis**,
and the two-axis design degrades toward the single-axis form `README-POLICY.2` replaced. The
byte cap keeps its original job — making the 138,403-byte outcome impossible — but it is no
longer co-binding at pointer shape.

⭐ **The mitigation is to report the metric that actually predicts failure.** The enforcer
now prints headroom on every passing run:

```text
memory-arch: OK (layer A 7051/32768 bytes = 21% of cap, 30/50 lines = 60% of cap)
```

Layer A sat at ≥ 97% of its byte cap for 36 consecutive commits and **nothing said so**,
because a passing gate printed the same three characters at 5,720 bytes as at 7,168. This is
reporting only — it adds no failure path and cannot change a verdict; the caps remain the
sole gate. What it changes is that "how much room is left?" is now answered on every commit
instead of only when the gate finally fires.

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
