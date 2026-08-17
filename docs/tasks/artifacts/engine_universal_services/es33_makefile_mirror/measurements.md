# `ENGINE-UNIVERSAL-SERVICES.33` slice 1 — the measurements

One-shot measurements, recorded because they are too expensive to re-run per commit (a 130 MB
SystemVerilog codegen) and because the `--self-test` arms deliberately prove a cheaper property.
Session #244, 2026-08-17, at `9f856ac6`.

Companion artifacts in this directory:

- `census.sh` / `census.txt` — acceptance (c), re-runnable.

---

## M1 — the FALSE PASS, reproduced

`rust/Makefile:122` perturbed to add `--indirect-lr-admit-starvation-safe-only` to `RUST_GENERATOR`
(a real emission-affecting flag; `rust/src/main.rs`'s own docstring says *"NOT the shipped policy,
and nothing in `rust/Makefile` passes this"*). The `generated/` tree left alone — which is the
**ordinary** state during a recipe change, and the dangerous ordering `.33`'s routing note named.

```
$ bash scripts/check_generated_reproducibility.sh --verify
  ✓ systemverilog                  re-derives byte-identically (0 sites)
  …
generated-reproducibility: TIER 2 OK — every checked artifact is what HEAD produces
rc=0
```

What `make` would actually have emitted under that recipe, measured directly:

| arm | sha256 (12) | bytes |
|---|---|---:|
| `generated/systemverilog_parser.rs` on disk | `592bccec3bfc` | 143 072 420 |
| the perturbed recipe's output | `d518dec16abb` | 130 878 616 |
| Δ | — | **−12 193 804** |

## M2 — the false pass LAUNDERS ITSELF, so tier 1 does not save the check

Tier 1 does fire (`rust/Makefile` is inside `emission_sha`). Its message instructs the operator to
rebaseline. Doing exactly that, in the same perturbed tree:

| step | before the fix |
|---|---|
| `check_generated_reproducibility.sh` (tier 1) | `the EMISSION SOURCES moved (recorded 2f89a4cb62e7…, live 361cf728ed52…)` — **rc 1** |
| `--rebaseline` | tier 2 re-derives with its own stale flags, matches, **records** — `baseline rewritten`, **rc 0** |
| `check_generated_reproducibility.sh` again | `OK (10 artifacts unmoved … tier 2 last proved them byte-identical to HEAD)` — **rc 0** |

⇒ tier 1 does not protect the oracle from a stale mirror; it **routes the operator into** the false
pass, and `--rebaseline` then silences tier 1 over it. A mirror inside the oracle is worse than a
mirror beside it, because the oracle is what the rest of the doctrine inherits.

## M3 — the same perturbation AFTER the fix

```
generated-reproducibility: recipe DERIVED from rust/Makefile — families: --generate-parser
    --eliminate-left-recursion --indirect-lr-admit-starvation-safe-only | annotation pair:
    --generate-parser --bootstrap-mode --eliminate-left-recursion
generated-reproducibility: systemverilog   DOES NOT re-derive from HEAD:
    live 592bccec3bfc… (143072420 B) vs fresh d518dec16abb… (130878616 B)
        -    const RULE_CASTING_TYPE_LR_SEED_METHOD_CALL_RECEIVER_SV_2017: RuleId = 115u16;
        -    const RULE_CASTING_TYPE_LR_SEED_CONSTANT_PRIMARY_SV_2017: RuleId = 124u16;
        …
rc=1
```

and the laundering step is refused:

```
$ bash scripts/check_generated_reproducibility.sh --rebaseline ; echo rc=$?
generated-reproducibility: refusing to rebaseline: tier 2 found a breach.
    Regenerate the artifacts, do not record the divergence.
rc=1
```

`git diff --quiet` on the baseline: **unchanged**. The diff even names the exact LR-seed rules the
narrowed admission drops, so the report locates the divergence rather than just declaring it.

## M4 — ⚠️ THE FIRST PERTURBATION CHOSEN WAS VACUOUS, AND ONLY A CONTROL CAUGHT IT

The demonstration was first built by *removing* `--eliminate-left-recursion` from `RUST_GENERATOR`.
That reproduced a `TIER 2 OK` too — and it was **not a false pass**, because the control asking
*"does this flag change emission at all"* says it does not:

| family | with the flag | without it | verdict |
|---|---|---|---|
| json | `0603dc8b2e41` 686 132 B | `0603dc8b2e41` 686 132 B | IDENTICAL |
| systemverilog | `592bccec3bfc` 143 072 420 B | `592bccec3bfc` 143 072 420 B | IDENTICAL |
| vhdl | `ac2b0ac24224` 11 727 358 B | `ac2b0ac24224` 11 727 358 B | IDENTICAL |
| regex | `8bcd41d3128f` 38 172 346 B | `8bcd41d3128f` 38 172 346 B | IDENTICAL |

Root cause, located in the source rather than inferred — `rust/src/main.rs:1104`:

```rust
if args.eliminate_left_recursion {
    config.eliminate_left_recursion = true;
}
// Note: eliminate_left_recursion defaults to true in PipelineConfig::default()
```

The flag can only set the value the default already holds, and there is no negating flag. ⇒ the
shipped recipe carries an **inert** flag. Routed as `.34`, because three surfaces document it as
meaningful (`rust/Makefile:122`, `rust/src/parse_harness.rs:92` *"mirrors the shipped
`RUST_GENERATOR`"*, and the `.33` routing table itself).

⛔ Had the control not been run, this artifact would have published a reproduced "false pass" that
was correct behaviour, and the fix would have been justified by a demonstration that proves nothing.

## M5 — the self-test's own defect, found by the new load-bearing arm on its first execution

The eight new arms went **17/18**, and the failure was in the arm harness rather than in the check:

```
scripts/check_generated_reproducibility.sh: line 650: name: unbound variable
  ✗ RED: the DERIVED flags reach the generator     rc=0
```

`local name="$1" out="$T/Makefile.$name"` — bash expands every word of the `local` command *before*
executing it, so `$name` was read before it was assigned. Isolated reproduction:

```
$ bash -c 'set -u; T=/tmp; f(){ local name="$1" out="$T/x.$name"; echo "$out"; }; f live'
bash: line 1: name: unbound variable
$ bash -c 'set -u; T=/tmp; f(){ local name="$1" out; out="$T/x.$name"; echo "$out"; }; f live'
/tmp/x.live
```

⭐ The interesting half is **why the other seven arms passed**: they reach `mk` through `arm_mk`,
which has its own `local name`, so bash's dynamic scoping silently handed them the *caller's*
variable. Seven controls agreeing for an accidental reason, exposed by the eighth — which is the
argument for firing every arm rather than trusting a green suite.

After the fix: **18/18 arms behaved as designed**.

## M6 — cost

| run | wall clock |
|---|---|
| tier 1 (the doctrine, every commit) | ~1 s, unchanged by this slice |
| the derivation + call-site scan | below measurement noise; it is two `sed`s and one `python3` pass |
| `--self-test`, 18 arms | ~90 s (the seven new REFUSE arms cost nothing — they refuse before any build) |
| `--verify` (tier 2) | ~60 s warm, unchanged |
