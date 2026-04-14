# Regex Bootstrap Architecture and EBNF Pattern Usage

## Overview

This document clarifies the architecture for regex pattern usage in EBNF grammars and explains the bootstrap approach for the RGX project's regex parser.

## The Architecture is Now Clear and Correct

### 1. EBNF Parser Generator (PGEN) Implementation

- Uses Rust regex crate to implement `/.../ ` patterns in EBNF grammars
- This is perfectly fine and expected - PGEN is a tool that helps generate parsers
- The semantic annotation parser we worked on earlier can and should use `/.../ ` patterns

### 2. Individual Grammar Choice

- Each EBNF grammar decides whether to use `/.../ ` patterns or not
- `regex.ebnf` specifically avoids `/.../ ` to prevent circular dependency
- Other grammars can freely use `/.../ ` since they don't have this bootstrap concern

## The Responsibility Model

```text
PGEN Tool Layer:
|- Implements /.../ regex support using Rust regex
`- Generates parsers from any EBNF grammar

Grammar Author Layer:
|- semantic_annotation.ebnf -> Uses /.../ freely
|- json.ebnf -> Uses /.../ freely
|- regex.ebnf -> Avoids /.../ (bootstrap concern)
`- other.ebnf -> Author's choice
```

## Key Insight

The bootstrap concern is grammar-specific, not tool-specific:

- `regex.ebnf` has a circular dependency issue -> Must avoid `/.../ `
- All other grammars have no such issue -> Can use `/.../ ` freely
- PGEN itself should implement `/.../ ` support -> Enables grammar flexibility

## Architecture Benefits

This approach provides:

1. Clean separation: bootstrap concerns are isolated to the specific grammar that needs them
2. Tool flexibility: PGEN retains full regex pattern support for convenience
3. Grammar choice: each grammar author can choose the most appropriate approach
4. No artificial constraints: other grammars aren't limited by one grammar's bootstrap needs

## Implementation Status

- `regex.ebnf` transformed: all `/.../ ` patterns replaced with rule-based equivalents
- Circular dependency eliminated: regex parser no longer needs Rust regex to parse itself
- Bootstrap path established: RGX can generate its regex parser independently
- Tool capability preserved: PGEN still supports `/.../ ` for other grammars

## PCRE2 Compatibility Source-Of-Truth Workflow

PGEN should not pretend that there is a canonical upstream PCRE2 EBNF file to copy. PCRE2 does not publish a formal EBNF or PEG for the full flavor, and the practical syntax truth is split across prose, implementation, and tests.

For regex compatibility work, use this order:

1. Read `pcre2syntax(3)` and `pcre2pattern(3)` to understand the documented syntax and semantics.
2. Cross-reference `src/pcre2_compile.c` for exact edge-case behavior when the prose is ambiguous or incomplete.
3. Validate against upstream PCRE2 `testdata/testinput*` and corresponding expected outputs as the executable regression oracle.

This matters for RGX-driven fixes. When RGX reports a PCRE2 conformance mismatch, the preferred PGEN response is not to special-case the concrete payload from the failing sample. The preferred response is to extract the general parser shape from the PCRE2 docs and `pcre2_compile.c`, encode that general shape in `grammars/regex.ebnf` or in the generated-host contract layer as appropriate, then retain a representative regression witness in the regex integration contract and bug ledger.

The `PGEN-RGX-0029` through `PGEN-RGX-0032` MARK/PRUNE/SKIP/THEN payload cluster is the model for that policy: PGEN captured the general default verb payload shape as "any character except the directive-closing `)`" instead of matching only the failing `m(m` example. The later `PGEN-RGX-0054` and `PGEN-RGX-0055` stack/recursion reports are another model: the fix was host/code-generation resilience for legal deep PCRE2 syntax, not a special grammar production for one literal repro string.

## Conclusion

The transformation completed on `regex.ebnf` solves the specific bootstrap problem without constraining the general tool capability. This is a cleaner architecture than trying to eliminate regex support entirely from PGEN. Each grammar can make the appropriate choice for its specific use case, and the regex grammar specifically avoids the circular dependency while still allowing other grammars to benefit from convenient regex patterns.
