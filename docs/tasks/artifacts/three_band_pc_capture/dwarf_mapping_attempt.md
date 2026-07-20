# DWARF source mapping attempt — stopped without an attribution claim

`PGEN-RGX-0078-0182`, leaf `RGX-0078.5.j.4`, session #170,
2026-07-20. This is the rejected high-memory route for attaching generated
source lines to the custody-pinned sampled instructions.

## Why a separate debug twin was required

The captured probe is the preserved C1 artifact at SHA-256
`1d3fa0eebb4de19ff003ace38721d80d71bd94653f8ea3801c0f29ca9581f9b8`.
It has no usable DWARF. The current generated regex source is a later G1-A
vintage (`e4924024a4bf7a91a75bb8b0dfb8460c71f19ef83e08313661fbd5d4e1ad590f`),
so addresses cannot be transferred by a constant offset. The preserved target
functions contain 6,845 `piece` and 19,215 `atom_closure` instructions; the
current release artifact contains 6,794 and 19,079. Exact normalized-instruction
alignment covers only 4,623/6,845 = **67.5383%** and 6,778/19,215 =
**35.2745%** statically. Across the sampled target population that is only
1,907/2,986, 1,901/2,987, and 2,570/3,868 exact matches by band. This is useful
custody evidence, but not permission to infer source locations for unmatched
instructions.

`map_source_lines.py` therefore requires an independently built, DWARF-bearing
current-vintage probe, aligns only equal normalized instructions, and refuses
to transfer any other address.

## Build attempt and stop condition

The guarded release+fat-LTO+DWARF build completed the 1.1 GiB library/LLVM
object work, but its binary link was externally terminated. A link-only retry
was also externally terminated after 60 seconds with peak resident memory
**14,902 MiB**, below the configured 16,384 MiB guard budget; the guard itself
reported `reason=none`. The durable marker is:

```text
status=completed
reason=none
exit=101
budget_mb=16384
floor_pct=10
disk_floor_gb=8
timeout_s=7200
peak_rss_mb=14902
elapsed_s=60
```

Marker SHA-256 before cleanup:
`64b916df0e20730820d413f789b34dfc6f00ec7dce493c891b64006004afb0ad`.

A narrower fallback tried extracting and internally pruning just the regex
parser from the completed LLVM 22 bitcode with the toolchain's own `opt`. That
process was externally terminated after 50 seconds with exit 143; again the
memory guard reported `reason=none` rather than a resource-floor kill:

```text
status=completed
reason=none
exit=143
budget_mb=16384
floor_pct=10
disk_floor_gb=8
timeout_s=7200
peak_rss_mb=0
elapsed_s=50
```

Marker SHA-256 before cleanup:
`75067f65661cdd8b0538d44a0c29802f26b1e188d7892418d24fe02563bce1b7`.
The installed Homebrew LLVM 20 cannot read the Rust toolchain's LLVM 22
bitcode, so changing `opt` implementations is not a valid fallback.

## Adjudication

The high-memory route stops here. No process remains, and the 5.0 GiB scratch
target plus both completion markers are deleted after this record captures the
failure. **No source-line attribution is claimed.** The accepted mechanism
analysis instead uses exact half-open instruction ranges in the captured
probe, machine-structure assertions, zero-overlap checks, and a complete
target-sample re-sum. That lower-bound route requires neither DWARF nor address
transfer across artifact vintages.
