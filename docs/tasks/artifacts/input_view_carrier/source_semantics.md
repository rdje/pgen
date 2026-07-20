# Immutable input-view feasibility and ownership audit

## Source contract

The source-pinned generated parser owns exactly one `input: &'input str` field,
initialized by `new`. There is no `self.input =` or `parser.input =` assignment
anywhere in the generator or fused emitter. The input allocation and UTF-8
validity therefore remain fixed for the parser's `'input` lifetime; only the
current byte position and other parser state mutate.

The fused match emitter has three `parser.input.len()` templates and four
`parser.input.as_bytes()` templates. Grammar expansion and fat-LTO replicate
them across the two preserved hot regions. Build-side string slicing is a
different lifetime/content mechanism and is excluded.

## Complete machine and dynamic ownership

The full preserved-region disassembly contains **93** direct input-length
carrier loads and **77** direct input-data carrier loads after accepted-range
subtraction. Every length value feeds a bounds comparison. Every data value
feeds one or more byte/halfword loads, directly or after an address add. The
bounds comparisons, conditional branches, and byte loads are required and are
not priced; only the two parser-field carrier loads are candidates.

Of those 170 static sites, 50 length and 14 data PCs receive samples. Their
dynamic counts are length **226/205/297** and data **4/4/11**, reproducing the
prior input-view row **230/209/308** exactly. The complete static set has zero
overlap with accepted mechanisms and held expansions through `-0191`.

## Scalar shortcut refuted

Passing only `input_len: usize` looks attractive because length loads own 97.7%
of the gross sampled carrier time. The source-pinned Rust probe disproves it.
After an explicit `position < input_len` comparison, safe indexing through
`parser.input.as_bytes()[position]` still reloads the parser field's real
length and performs its own bounds check. The compiler cannot assume an
arbitrary scalar equals the slice length. Removing that reload would require
unchecked indexing or another proof mechanism, neither of which is admitted.

## Safe complete-view carrier

A full shared byte slice is feasible in safe Rust:

1. A fused sub-root orchestrator copies `parser.input.as_bytes()` once.
2. `cascade_match_*` functions accept `input: &'input [u8]` beside
   `&mut self`; internal match calls forward that view.
3. The three match templates use `input.len()` and `input[...]`.
4. Build functions and the public parser API remain unchanged.

The reference is `Copy` and points at external immutable input, so copying it
out of the parser does not keep the parser field borrowed and can coexist with
`&mut self`. Safe indexing uses the forwarded slice's actual length. The probe
shows one orchestrator `ldp x1, x2, [x0]`, while the callee reads parser
position only and indexes through the forwarded x1/x2 data/length pair. No
unsafe read, UTF-8 relaxation, input cap, or public lifetime change is needed.

## Replacement-aware price

Current sampled length loads price **12.029766185 ns** and data loads
**0.286389190 ns**, for a **12.316155375 ns gross current-carrier ceiling**.
That is not net savings. The replacement retains one entry `ldp` per fused
sub-root invocation, forwards a two-register fat slice through the match graph,
and may create register pressure or spills in the already-large fused regions.
None of those costs exists as an attributable equivalent in the current
binary, so strict net contribution is **0 ns** rather than pretending they are
free.

Added only as a gross ceiling to the prior strict bundle, 30% capture leaves
**5.278746322 ns** above the 28.8 ns noise floor. That margin has not paid the
entry/ABI/register costs. The safe design is banked, but implementation remains
HOLD.
