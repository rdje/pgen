# SV-EXH-PROOF.3.3.4.a — MVP-0 design: parser-agnostic compilation-artifact library

**Owner leaf:** `SV-EXH-PROOF.3.3.4.a` (a sub-leaf of `.3.3.4`). Code-Change Doctrine: this memo is the design; the leaf will own the implementation commit when it lands.

**Strategic line (user, 2026-05-20):** EBNF + the AST-pipeline engine acquire the parser-agnostic features needed to cleanly and elegantly parse all real HDL corpora. The original concern — "the user supplies an ordered HDL file list with packages-before-importers" — generalizes to a per-compilation-artifact library used by every HDL.

This MVP-0 is the smallest concrete first step. Future increments are explicitly enumerated in §Out-of-Scope.

---

## 1. Problem statement

PGEN's SV parser today parses single files in isolation. Real HDL is multi-file: `import el2_pkg::*;` in `el2_lsu.sv` references types declared in `el2_def.sv` (separate file). The use-site `@predicate has_fact(type_name, X)` correctly fires (`.3.3.3` made it reliable) but correctly evaluates to false because the type-emitting file was never parsed in the same session.

The architecturally correct fix — matching every commercial HDL tool — is **per-file compilation that writes a compact on-disk artifact** containing the exported facts of each scope-creating entity (package, module, interface, …), with **on-demand library lookup** when an importer references that entity.

**MVP-0 narrows this** to packages only, JSON artifacts, user-supplied file order. Future increments grow the kind set, add `$unit`/CU semantics, optimize the format. Each increment is leaf-owned and corpus-driven.

## 2. What MVP-0 ships

### 2.1 Two new parser-agnostic engine annotations

```ebnf
@export_to_library: { kind: <static-kind>, name_from: <field-or-positional-expr> }
@import_from_library: { kind: <static-kind>, name_from: <field-or-positional-expr> }
```

Semantics (engine, in the generator-emitted `with_semantic_runtime_rule_transaction`):

- **`@export_to_library`** fires on a rule's **successful commit** (the `.3.3.3` IIFE makes this reliable). The engine snapshots the facts the rule emitted within its own scope (the delta against the rule's entry checkpoint), filters to a small set of "exportable" fact kinds (v0: `type_name` only; future: `class_name`, `package_name`, `function_name`, …), and writes them as a JSON artifact under the configured library-out directory. Atomic write (temp file + rename) so a failed parse never leaves a half-written artifact.
- **`@import_from_library`** fires on rule **entry** (pre-predicate phase). The engine reads the artifact for the named entity from the configured library-in directory and merges its facts into the current `SemanticRuntimeState`. Missing artifact = `Err` with a clear "library lookup failed: kind=<K>, name=<X>" message; the existing IIFE+restore makes that error path clean.

Neither annotation is SV-specific. Both are usable by VHDL (`@export_to_library: {kind:entity, …}` on `entity_declaration`), Verilog (`{kind:module, …}` on `module_declaration`), or any future HDL grammar.

### 2.2 Artifact format (v0)

One JSON file per exported entity, path: `<lib-dir>/<kind>/<name>.facts.json`.

```json
{
  "format_version": 1,
  "kind": "package",
  "name": "el2_pkg",
  "facts": [
    { "kind": "type_name", "name": "el2_trigger_pkt_t",
      "attributes": [{ "key": "declaration_family", "value": "typedef" }] },
    { "kind": "type_name", "name": "el2_lsu_pkt_t", "attributes": [] }
  ]
}
```

Properties:
- Human-readable, debuggable, easy to diff during development.
- Atomic write (`temp_name` then `rename`).
- Format-version field so future increments can evolve the shape with a compatibility check.
- Tiny: a few hundred entries per package, a few bytes each → trivial memory and disk footprint.

### 2.3 Parser CLI (`parseability_probe`)

Two new optional flags, both parser-agnostic:

```
parseability_probe --parse <grammar> <input> [--lib-out <dir>] [--lib-in <dir>]
```

- `--lib-out <dir>`: when set, `@export_to_library` directives write artifacts under `<dir>`. Mkdir-p as needed.
- `--lib-in <dir>`: when set, `@import_from_library` directives read artifacts from `<dir>`. Missing dir = `Err`.

Neither flag changes single-file parse behaviour when absent.

### 2.4 SV grammar additions (the MVP-0 grammar half)

Just two annotation sites in `grammars/systemverilog.ebnf`:

- On `declared_package_identifier` (where the package name is already emitted as a fact by `.3.3.2`'s `@emit_fact {kind: package_name, name: $body}`): add `@export_to_library: { kind: package, name_from: $body }`. The artifact is named for the package, contains every `type_name` fact emitted while parsing the package body.
- On `package_import_item` (whose first child is `package_identifier`): add `@import_from_library: { kind: package, name_from: $package_identifier.body }`. The fact-merge happens before the importer's later use-site `has_fact` checks evaluate.

No other grammar change. No removal of any existing `has_fact` gate — the gate is correct; the artifact mechanism is what lets it succeed.

### 2.5 Triage gate refactor

Manifest extension — each case may declare an ordered `bootstrap_files` array:

```json
{
  "name": "veer_el2_lsu",
  "path": "stimuli/sv/subs/Cores-VeeR-EL2/design/lsu/el2_lsu.sv",
  "bootstrap_files": [
    "stimuli/sv/subs/Cores-VeeR-EL2/design/include/el2_def.sv"
  ],
  …
}
```

Gate script changes:
1. Per-case lib-dir: `<work>/case_<name>_<profile>.lib/`.
2. For each `bootstrap_files[i]`: preprocess (same as the main file) then parse with `--lib-out <case-lib-dir>` (this writes the artifacts for any packages declared inside).
3. Parse the main `path` with `--lib-in <case-lib-dir>` (the importers can now resolve).

User supplies the file order. The user's order is the transitive-dependency contract — commercial-tool convention. The engine does not auto-resolve dependencies in MVP-0.

## 3. Out of scope for MVP-0 (explicitly deferred; each will be a future leaf when a corpus case demands it)

| Deferred capability | When to revisit |
|---|---|
| `module`/`interface`/`program`/`primitive`/`config`/`checker` artifacts | Next corpus case that fails because of cross-file references of those kinds |
| Compilation-unit concept (`$unit`, `-mfcu`/`-sfcu`, CU-scope visibility) | When a corpus case has top-level declarations outside any entity that need cross-file CU visibility |
| Automatic dependency resolution (scan for imports, topo-sort, cycle detection) | When user-supplied lists become impractical |
| Content-hash-based incremental cache invalidation | When recompile costs become a measured concern |
| Binary artifact format | When JSON size or parse time becomes a measured concern |
| Cross-kind name collision handling (e.g. `module foo` and `package foo`) | When a corpus case demands it (lib-dir kind subdir already segregates) |
| Multi-library / library search path | When multi-project corpora demand it |
| Elaborator (cross-file type checking, design build) | Out of PGEN's remit entirely — that's a downstream consumer's job |

## 4. Verification plan

**Pre-implementation invariants the design preserves**:
- `.3.3.3` exception-safety: `@export_to_library` and `@import_from_library` are effect directives that go through the same IIFE path; an error during library write/read is `Err`-propagated and the `if result.is_err() { restore }` fires cleanly.
- Single-file parse with neither flag set: byte-identical behaviour to today (the new directive applies are no-ops when both flags are absent).
- RGX downstream: regex has no `@export_to_library`/`@import_from_library` usages and no semantic predicates → the change is dead code for that parser → conformance gate must stay at 44/0.

**Empirical acceptance** (the gate before commit):
1. Minimal repro stays green: `module m; typedef int my_t; my_t [3:0] x; endmodule` — PASS (already does, with `.3.3.3`).
2. New cross-file repro: a tiny package file + a tiny module file using its types parses correctly with `--lib-out` on the first, `--lib-in` on the second.
3. **veer_el2_lsu_{2017,2023}** corpus cases — PASS. Triage gate corpus moves `6/14 → 8/14`.
4. scr1 ×4 + friscv_pipeline ×2 — still PASS (no regression).
5. uvm_pkg / uvm_compat_pkg / friscv_rv32i_core — unchanged (their failures are separate concerns).
6. SV shape-contract — GREEN, samples=3 aligned=3 drift=0 regression_lock_failures=0.
7. RGX broader corpus / conformance — **44/0**, unchanged.
8. Annotation inventory: +2 for the new directive-bearing sites in SV; no other shift.

If any step regresses, the implementation does not commit; we triage and amend the design — per "manage one issue at a time."

## 5. Implementation plan (leaf-owned `.3.3.4.a`)

Five sub-steps. Each is small and verifiable in isolation:

1. **Library I/O module** (`rust/src/ast_pipeline/library.rs`): JSON serde for the artifact format; atomic write; read with format-version check. ~150 lines + a unit test.
2. **Two new directive variants** in `rust/src/ast_pipeline/semantic_runtime.rs`: `SemanticRuntimeDirective::ExportToLibrary(spec)` and `ImportFromLibrary(spec)`, plus parsers for the new annotation forms. ~80 lines + unit tests.
3. **Wire-up in the generator** (`rust/src/ast_pipeline/ast_based_generator.rs`): apply `@import_from_library` at rule entry (pre-predicate phase); apply `@export_to_library` at successful commit (effect-directive phase). Inside the existing IIFE, so exception-safety is inherited. ~50 lines.
4. **CLI flags** in `parseability_probe`: `--lib-out`/`--lib-in`. Thread the configured paths into the generated parser's state via a new method on the parser struct. ~30 lines.
5. **SV grammar additions**: two annotation lines. ~5 lines.
6. **Triage gate refactor**: per-case `bootstrap_files` + per-case lib-dir. ~30 lines in the shell script + a manifest field on the veer case.

Lockstep at the end (same-commit): engine + library module + grammar + triage manifest + triage script + SV integration contract bump (1.0.121 → 1.0.122) + SV book changelog/schema-versioning + regenerated tracked HTML + CHANGES + LIVE + TASK_TREE + SV-EXH-PROOF.md (`.3.3.4.a` DONE).

**No push** — absolute override. Restore tag `checkpoint/sv-exh-proof-3.2-clean` @ 75afb3c7 still anchors the pre-`.3.3.3` state if a hard reset is ever needed.
