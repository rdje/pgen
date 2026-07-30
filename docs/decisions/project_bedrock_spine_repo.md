# project — `bedrock`: PGEN's discipline spine lives in a separate public repo

**Category:** `project` (standing fact + porting discipline)
**Established:** session #202. **Recorded here:** 2026-07-30, session #230, by `README-POLICY.2`.

## ⛔ Why this record exists at all — the link was dangling

Layer-A `MEMORY.md` cited this fact as `[[project_bedrock_spine_repo]]`, and **that record did not
exist**. Measured during the `README-POLICY.2` trim: the string `project_bedrock_spine_repo` appeared
in **exactly one place in the entire repository — `MEMORY.md` itself** (0 hits anywhere else,
including `docs/decisions/`).

⇒ This was the single layer-A entry with **no durable home in any layer**. Every other entry in the
"Current state" block resolved either to a commit subject (layer D) or a task tree (layer B); this one
resolved to nothing. Pruning it without writing this file would have deleted the only record that a
sibling repository exists and that PGEN doctrine must be ported to it.

⭐ The general lesson, which is the whole reason the trim ran an ownership census first: **a `[[link]]`
is a promise, not a proof.** A dangling one marks content that is *less* safe to prune, not more.

## The fact

PGEN's neutral, project-agnostic **discipline spine** — the portable architectures this repo has
extracted (task-trees, memory-architecture, doctrine-enforcement, the toolbox contract) — is
maintained as a **separate public repository**, `github.com/rdje/bedrock`, set up as a GitHub template
with CI green. Genesis commit `021e685`.

The spine is what `MEMORY_ARCHITECTURE.md` and `DOCTRINE_ENFORCEMENT.md` each describe as
*"project-agnostic … drop it into any repository"*. `bedrock` is where that claim is actually kept
honest, because a second consumer is what proves a standard is portable rather than merely
described as portable.

## The porting discipline

When a **general** doctrine or structural improvement lands in PGEN, port it to `bedrock`:

1. **Strip the domain nouns.** Nothing SystemVerilog-, regex-, EBNF- or parser-specific survives the
   port. If a rule cannot be stated without a PGEN noun, it is not spine material.
2. Route the port through bedrock's own **`BEDROCK-MAINTENANCE`** task tree — the spine repo runs the
   same task-tree discipline it ships.
3. Keep `update_scaffold` **neutral**.
4. **Bump `DOCTRINE_VERSION`** so consumers can tell what they have.

## The boundary — both directions

- ⛔ **`bedrock` stays out of PGEN's git.** It is a separate repository, not a submodule or a vendored
  copy.
- ⛔ **PGEN specifics stay out of `bedrock`.** A domain noun leaking into the spine silently converts
  a portable standard into a PGEN fork.

⚠️ Consequence for cross-repo references: `bedrock` is a *separate checkout*, so PGEN must never
reference a path inside it. This is the same rule that applied when the README Stability Policy was
adopted from a sibling repo — the policy was **copied in** and tracked at
`docs/reference/PGEN_README_STABILITY_POLICY.md` rather than referenced across a boundary. See
[[project_data_locality_same_volume]].

## Honest limit

Nothing mechanically checks that a general PGEN doctrine change was ported. The porting discipline
above is a convention with no gate behind it, and this record does not claim otherwise. Whether the
two repositories have actually drifted is **unmeasured** — establishing that would need a comparison
run against the `bedrock` checkout, which is outside this repository.
