# Schema Versioning

This chapter explains how the regex parser's AST shape is versioned, what guarantees consumers can rely on, and how to pin to a known shape.

## Two versioning axes

The regex parser carries **two** version numbers:

1. **Parser release version** — e.g. `1.1.31`. Tracks the parser library's release identity. Bumped on every functional change to the parser, including bug fixes, perf work, and grammar changes.
2. **Schema version** — e.g. `0.7.x`. Tracks the AST output shape. Bumped only when the output shape changes in a way consumers may need to adapt to.

A single parser release can carry the same schema version as the previous release (no shape change) or a bumped schema version (shape changed). The two version numbers move independently.

The contract document `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` is the authoritative source for both numbers per release.

## What "shape change" means

Any of these triggers a schema version bump:

- A new return annotation lands on a previously-unannotated rule (e.g. slice 1's `digits → integer`, slice 2's `quant_suffix → enum string`).
- An existing return annotation is restructured (e.g. PGEN-RGX-0074's piece-array reshape).
- A grammar rule changes shape in a way that's user-visible (new branch added, branch removed, sub-rule renamed in a way that affects shape).
- The default fall-through behavior of unannotated rules changes (e.g. when `body_has_single_element` was tightened to exclude `Quantified`).

These do NOT trigger a schema bump:

- Pure performance optimizations that produce the same AST.
- Internal codegen reorganization that doesn't reach the output.
- Parser-side bug fixes that produce the same shape consumers were already relying on.

The slice campaign for task #40 ("Annotate regex.ebnf for full AST usability") will produce many small schema bumps as rules are annotated one-by-one. Each slice gets its own contract-version row.

## Byte-equivalence guarantee

For any input the parser accepts, this equality holds:

```rust
parse_full_regex(input).unwrap().content.to_json_value()
    == parse_regex_typed(input).unwrap()
```

That is — walking the typed `Json` content via `to_json_value()` produces the same `serde_json::Value` as the typed parser entry point produces directly. This is a stability invariant we maintain across **all** future shape changes.

In practice this means:

- Consumers can use either entry point (`parse_full_regex` for envelope-with-spans, `parse_regex_typed` for plain `serde_json::Value`).
- Consumers writing JSON snapshots from one entry point can later switch to the other without re-baselining.

If you ever encounter a case where the equality fails, that's a parser bug — please report.

## Stability tiers

The regex parser's behavior is divided into three tiers by stability guarantee:

### Tier 1 — Stable surface (contract-bound)

| Item | Stability |
|---|---|
| `crate::parse_full_regex(input) -> Result<ParseNode<'_>, ParseError>` | Stable signature. Function does not move or rename. |
| `crate::parse_regex_typed(input) -> Result<serde_json::Value, ParseError>` | Stable signature. |
| `ParseNode { rule_name, content, span }` field set | Stable. New fields may be added (additive); existing fields not removed. |
| `ParseContent` six-variant set | Stable. Variants not removed without major version bump and migration window. |
| Schema version field in CHANGES.md | Always present per release. |

### Tier 2 — Annotated rule shapes

Each rule listed in [The Json Carrier](json-carrier.md)'s annotated-rule table is part of the schema once annotated. Removing or substantially changing the typed shape requires:

1. A schema version bump.
2. A contract document update with a "before/after" entry.
3. A `MIGRATION.md` entry for affected consumers.

We commit to NOT silently changing typed shapes. Consumers can rely on a given annotation's output shape across all releases that share the schema version.

### Tier 3 — Unannotated rule shapes

Rules NOT in the annotated list emit raw `Sequence` / `Alternative` / `Quantified` envelopes. Their shape is determined by the codegen's default fall-through and may evolve as part of:

- Grammar reorganization (e.g. sub-rule extraction or merging).
- Implicit `-> $1` default policy changes (unlikely; tightened in PGEN-RGX-0073-era).
- Slice campaign annotation (the rule moves from Tier 3 to Tier 2).

The expected lifecycle of a Tier 3 rule is to be annotated and promoted to Tier 2, after which its shape is locked.

If you walk a Tier 3 rule today, your walker may need to adapt when that rule is annotated. The recommended approach is to centralize Tier 3 walking in one place per rule and update it slice-by-slice.

## Pinning policy

For consumer projects, pin the PGEN parser release in `Cargo.toml`:

```toml
[dependencies]
parseability = { git = "https://github.com/RichardSamWell/pgen.git", tag = "1.1.31" }
```

The release tags are stable git references; once published, they don't move. Pin to a specific tag to lock the AST shape. To upgrade, read the contract changelog for the target version, adjust your walker if needed, and re-pin.

Branch-based pinning (`branch = "main"`) is **not recommended** for consumer projects — it makes shape changes appear without notice. For development experimentation, branch-pinning is fine.

## How to detect schema changes in CI

The recommended consumer-side CI guard is a regex-AST snapshot test:

```rust
#[test]
fn ast_snapshot_for_canonical_inputs() {
    let inputs = [
        r"a", r"a*", r"\d+", r"[a-z]", r"\Qab\E{3}", /* ... */
    ];
    for input in inputs {
        let actual = parseability::parse_regex_typed(input).unwrap();
        let expected = read_snapshot(input);
        assert_eq!(actual, expected, "AST shape changed for input: {input}");
    }
}
```

When you upgrade PGEN, this test will fail loudly on any shape change. Re-baselining the snapshots gives you the diff to inspect.

## Schema version timeline

| Schema version | First parser release | Notable changes |
|---|---|---|
| 0.5.x | pre-1.1.30 | Pure recursive envelope. |
| 0.6.0 | 1.1.30 | First annotated rules: `regex`, `pattern`, `concatenation`, `piece`. |
| 0.7.0 | 1.1.31 | PGEN-RGX-0074 fix: `\Q...\E` per-char piece array. New `**` flatten-spread semantics. |
| 0.7.1 | 1.1.32 | `digits` → integer (slice 1). Within-version-line shape addition (additive). |
| 0.7.2 | 1.1.33 | `quant_suffix` → enum string (slice 2). |
| 0.8.0 | post-1.1.33 main | `counted_quantifier_body` → typed `{min, max}` (slice 3). New `null` literal in the annotation language. |
| 0.8.1 | post-1.1.33 main | `counted_quantifier` → `-> $3` lifts body's typed shape (slice 4). |

(Numbers above match the contract document at the time this book was written. The contract is authoritative for the current state — consult it for the live version.)

## Future major version

A schema 1.0.0 milestone will land when the task #40 annotation campaign completes — that is, when every rule in `regex.ebnf` carries either a return annotation or a deliberate decision to remain raw envelope. At that point all shape definitions move to Tier 2 (locked) and no further default fall-through changes are expected.

Pre-1.0 schema versions (0.x.y) follow semver-ish convention — minor bumps for additive changes, patch for purely-additive within-shape additions, breaking changes are explicitly called out and may bump the minor digit.

Post-1.0, breaking schema changes become major-version events with a deprecation cycle.
