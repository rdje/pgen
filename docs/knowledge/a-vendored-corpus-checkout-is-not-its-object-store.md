---
id: a-vendored-corpus-checkout-is-not-its-object-store
title: A vendored corpus checkout is not its object store — whether a reference tool can be built from its submodule is measured, never read off the directory (`SV-SPEED`)
answers:
  - "can I build slang from the vendored submodule"
  - "can I build verilator from stimuli/sv/subs/verilator"
  - "why does stimuli/sv/subs/slang have no source directory"
  - "which vendored submodules hold their full source"
  - "is a sparse submodule missing objects"
tags: [corpus, submodules, reference-tools, speed, provenance]
date: 2026-10-18
provenance: SV-SPEED (PGEN-SV-SPEED-0001), measuring the reference-tool builds the speed directive needs
reverify: "for d in stimuli/sv/subs/*/; do echo \"$(basename $d) $(GIT_NO_LAZY_FETCH=1 git -C $d rev-list --objects --missing=print HEAD | grep -c '^?')\"; done   # missing objects per submodule; slang and verilator must read 0"
---

# A vendored corpus checkout is not its object store

**The fact.** Ten of the thirteen SystemVerilog corpora under `stimuli/sv/subs/` are SPARSE-checked-out to their test
directories (`Cores-VeeR-EL2`, `friscv` and `scr1` are full checkouts) — `stimuli/sv/subs/slang/` shows `tests/` and
the top-level files, no `source/`. Whether the tool's own source is
nonetheless present is a property of the submodule's OBJECT STORE, and the store differs per submodule: some are
shallow clones holding the whole tree at the pinned commit, others are blob-filtered partial clones (`promisor=true`)
that hold only the sparse paths' blobs. Measured 2026-10-18, read-only, with `GIT_NO_LAZY_FETCH=1` so the
measurement cannot fetch:

| submodule | objects at `HEAD` | missing | promisor | whole tree at `HEAD` present |
|---|---:|---:|---|---|
| `slang` | 733 | 0 | none | ✅ |
| `verilator` | 7,469 | 0 | none | ✅ |
| `sv-tests` | 1,237 | 0 | none | ✅ |
| `verible` | 1,113 | 0 | none | ✅ |
| `Cores-VeeR-EL2` | 7,789 | 0 | none (full clone) | ✅ |
| `friscv` | 4,361 | 0 | none (full clone) | ✅ |
| `scr1` | 1,132 | 0 | none (full clone) | ✅ |
| `ispras-sv-tests` | 1,336 | 0 | true | ✅ (a partial clone, yet nothing is missing) |
| `black-parrot` | 391 | 67 | true | ❌ |
| `sv2v` | 1,147 | 86 | true | ❌ |
| `iverilog` | 6,342 | 610 | true | ❌ |
| `opentitan` | 14,493 | 4,522 | true | ❌ |
| `Surelog` | 26,101 | 20,127 | true | ❌ |

`stimuli/sv/subs/PROVENANCE.md` records ONE acquisition recipe (`--depth 1 --filter=blob:none --sparse`); the stores
show it was not applied uniformly, so the recipe is not evidence of what a given store holds.

**Why it matters.** The speed directive ([[project_sv_speed_on_par_with_slang_and_verilator]]) needs slang and
Verilator binaries at known revisions. Both stores are complete, so `git archive` from the submodule (read-only) gives
the exact vendored revision's source with no network and no byte written inside the submodule. For a partial clone the
same command would trigger a LAZY FETCH — a write into another repository's object store, which the repository
boundary forbids — so the measurement runs with lazy fetch disabled, and an export from a `❌` row is refused, not
attempted.

**How I got it wrong first.** Asked about the reference tools, I told the director slang *"is already vendored … the
benchmark leaf can build it from there"* having seen only that the submodule existed. The directory listing then showed
`tests/` alone, which looked like a refutation; only the object census settled it, and in slang's favour. Neither the
first claim nor the apparent refutation was a measurement.

**How to apply.** Before planning to build, diff or read a tool's source from a vendored submodule, run the
`reverify` line above and read that submodule's row; never read the working tree and never the provenance recipe.
