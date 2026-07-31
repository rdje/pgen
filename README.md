# PGEN

**EBNF-driven parser and stimuli generation for serious language tooling.**

PGEN turns an EBNF grammar into a production-grade Rust parser *and* the stimuli needed
to prove that parser correct — with deterministic gates, coverage/gap analysis and
closed-loop replay standing behind every claim.

- **Audience** — language-tooling engineers who need a parser they can trust where
  parsing correctness materially affects downstream flows.
- **Status** — active, Rust-first. Live per-family status:
  [`LIVE_ACHIEVEMENT_STATUS.md`](LIVE_ACHIEVEMENT_STATUS.md).
- **License** — Apache-2.0, see [`LICENSE`](LICENSE).

> 📖 **This README is a landing page, deliberately.** Detail lives in the
> [live book](docs/book/) and the reference/contract docs. What belongs where — and the
> line/byte caps that keep it that way — is
> [`docs/reference/PGEN_README_STABILITY_POLICY.md`](docs/reference/PGEN_README_STABILITY_POLICY.md).

## Scope

PGEN's product is **parser quality**: generated parsers must be correct, fast,
predictable, observable and trustworthy in real systems.

Three doctrines define the deliverable. Each is enforced, not aspirational:

- **EBNF-backed** — every parser that counts as a PGEN deliverable is generated from an
  EBNF grammar. No exceptions. Handwritten parsers exist only as bootstrap scaffolding
  and never count as closure.
- **Annotation-shaped** — every generated parser returns an AST. *Return annotations*
  shape that AST; *semantic annotations* steer generation behaviour.
- **Proof-first** — a closure claim requires EBNF-backed generation, return-AST shaping,
  parser/stimuli roundtrip proof, parser coverage proof, stimuli coverage/gap proof, and
  repeatable machine-checkable gates. This bar is repository-wide and identical for every
  grammar family — SystemVerilog, VHDL, regex, the annotation grammars, Phase S and
  anything added later.

Near-term integration targets: **Nexsim** (SystemVerilog + VHDL), **RGX** (regex), and
**PNR** (a staged LEF / Liberty / DEF / structural-netlist / SDC / SPEF family).

The rationale behind these doctrines is in
[Quality and Closure Model](docs/book/src/quality-and-closure-model.md); the graded
per-family bar is in [`LIVE_ACHIEVEMENT_STATUS.md`](LIVE_ACHIEVEMENT_STATUS.md).

## Prerequisites

- Rust **1.95** or newer (the maintained Cargo packages declare this MSRV).
- `bash`, `make`, and `git`.
- [`mdbook`](https://rust-lang.github.io/mdBook/) if you want to build the books locally.

## Quick Start

`generated/` is **not tracked in git**, so a fresh clone must produce it before the
crate can build with generated parsers:

```bash
git config core.hooksPath .githooks          # once per clone — activates the doctrine hooks
make -C rust SHELL=/bin/bash regenerate_generated_parsers
```

That seeds `generated/ebnf.rs`, emits the annotation parser pair, then the grammar
families — roughly four minutes from a bare tracked tree. Then prove the tree is sound:

```bash
bash scripts/check_doctrines.sh              # every mechanizable doctrine, seconds
make -C rust SHELL=/bin/bash mdbook_docs_gate
```

To generate a parser from your own grammar:

```bash
grammars/foolang.ebnf  ->  generated/foolang_parser.rs   # via: make -C rust focus_foolang
```

⛔ **Before debugging anything** — an `UNKNOWN`, a rejected parse, a hang, a reach gap, a
"why isn't this witnessed" — read [`TOOLBOX.md`](TOOLBOX.md) and run the debug tools
first. Never eyeball a grammar or guess a root cause. This is a standing directive and it
is *mechanically enforced*: a code change cannot land without tool-backed WHY+WHERE
diagnosis and measured before→after verification in its owning task leaf.

## Architecture At A Glance

The canonical flow, both directions from one grammar:

```
grammars/foolang.ebnf ──► raw_ast/json ──► generated/foolang_parser.rs
                      └─► in-memory stimuli and/or generated/foolang_stimuli.rs
```

- **Two stimuli-delivery modes** — default in-memory generation (`--generate-stimuli`)
  and optional generated modules (`--generate-stimuli-module`). When both exist, parity
  between them is part of the contract.
- **Annotation parsers bootstrap themselves out of a cycle.**
  `grammars/builtin_return_annotation.ebnf` and
  `grammars/builtin_semantic_annotation.ebnf` are bootstrap-safe contracts that let the
  annotation parsers be generated without depending on themselves. Every other grammar
  uses the normal path.
- **Every parser has its own live mdBook** — the canonical AST-integration reference for
  that family — alongside a downstream contract under `docs/contracts/`.

The full repository layout is
[Developer Architecture § Repository Layout](docs/book/src/developer-architecture.md);
the gate machinery is [The Gate Flow — Reference](docs/book/src/gate-flow.md).

## Running The Gates

Gates are the proof surface. Two you will use constantly:

```bash
bash scripts/check_doctrines.sh                        # all mechanizable doctrines
make -C rust SHELL=/bin/bash sota_exit_gate            # the flagship aggregate (hours)
```

⚠️ Heavy or background jobs must run under the memory guard:

```bash
scripts/run_with_memory_guard.sh --budget-mb 12288 --timeout-s 7200 -- \
  make -C rust SHELL=/bin/bash sota_exit_gate
```

The complete gate catalogue, exit-code contracts, artifact hand-off protocol and the
eight ways this flow has failed are in
[The Gate Flow — Reference](docs/book/src/gate-flow.md). Operational posture — host-RAM
governance, hosted-Actions policy, workflow parity — is in
[Operations and Governance](docs/book/src/operations-and-governance.md).

## Documentation

The [**live book**](docs/book/) is the primary documentation surface. Build or serve it:

```bash
mdbook build docs/book
mdbook serve docs/book --open
make -C rust SHELL=/bin/bash mdbook_docs_gate    # gate it
```

Documentation is split three ways, described in
[Documentation Model](docs/book/src/documentation-model.md):

| Layer | Home | Purpose |
|---|---|---|
| Book | [`docs/book/`](docs/book/) + per-parser books | public mastery surface, with rationale |
| Contracts & reference | [`docs/contracts/`](docs/contracts/), [`docs/reference/`](docs/reference/) | deep authoritative detail behind the book |
| Continuity | `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`, [`docs/tasks/`](docs/tasks/) | internal session/continuity surfaces |

## Read In This Order

New here — human or AI agent — follow this path:

1. This README.
2. [`SESSION_BOOTSTRAP.md`](SESSION_BOOTSTRAP.md) — session entry point.
3. [`docs/book/`](docs/book/) — the live book. Start with
   [Platform Overview](docs/book/src/platform-overview.md) and
   [Getting Started](docs/book/src/getting-started.md).
4. [`TOOLBOX.md`](TOOLBOX.md) — the diagnostic & debug toolbox. **Read before debugging.**
5. [`QUICKSTART_AI_ONBOARDING.md`](QUICKSTART_AI_ONBOARDING.md) and
   [`PGEN_USER_GUIDE.md`](PGEN_USER_GUIDE.md).
6. [`docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`](docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md)
   and [`LIVE_ACHIEVEMENT_STATUS.md`](LIVE_ACHIEVEMENT_STATUS.md) — direction and status.
7. [`MEMORY_ARCHITECTURE.md`](MEMORY_ARCHITECTURE.md) and
   [`DOCTRINE_ENFORCEMENT.md`](DOCTRINE_ENFORCEMENT.md) — how continuity and rule
   enforcement work here.
8. [`MEMORY.md`](MEMORY.md) → the active task-tree frontier
   ([`docs/TASK_TREE.md`](docs/TASK_TREE.md)).

## Contributing

- **Activate the hooks once per clone:** `git config core.hooksPath .githooks`. The
  pre-commit hook runs [`scripts/check_doctrines.sh`](scripts/check_doctrines.sh); CI runs
  the same. These are git-level and harness-agnostic.
- **Nothing changes without a task-tree first.** All work is tracked under
  [`docs/tasks/`](docs/tasks/) (index [`docs/TASK_TREE.md`](docs/TASK_TREE.md)); durable
  facts and decisions go in `docs/decisions/`.
- **Commit per [`COMMIT.md`](COMMIT.md)**, naming the work-unit id in the subject.
- **All paths are relative to the repository root**, and all project-owned data stays on
  the repository's own volume.
- A code change must pass the acceptance checklist — root cause, addressed, no regression,
  each evidence-backed — in its owning task leaf.
