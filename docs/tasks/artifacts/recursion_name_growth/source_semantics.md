# Recursion name-stack growth caller audit

## Why child attribution is required

`PGEN-RGX-0078-0188` priced exact instructions inside four generated inlined
name-stack pushes and the name half of outlined `RecursionGuard::enter_id`, but
deliberately excluded time interrupted inside `RawVec::grow_one` and its
allocator descendants. Flat samples for that `Vec<24>` monomorphization cannot
repair the omission: the same symbol has unrelated build, memo, and arena
callers.

The preserved binary contains exactly five relevant calls to name-stack
`RawVec::grow_one::h1ea4d81f4bfacb92`:

- `piece` call PCs `0x10005a6f4` and `0x10005c4d4`;
- sampled `atom` closure call PCs `0x100070644` and `0x10007a4a4`; and
- outlined `RecursionGuard::enter_id` call PC `0x10008df58`.

The adjacent required ID-stack arm calls the distinct
`RawVec::grow_one::h84ff3e56869fffcc` at `0x10008df84`. It remains excluded.
The classifier verifies all six `bl` targets against the full-SHA-pinned probe
before reading a profile.

## Exact call-tree ownership

macOS `sample` records a callee beneath its caller's return address. The four
inlined call PCs therefore map to exact `piece` offsets 6616/14264 and `atom`
offsets 22868/63412; the outlined call maps to `enter_id +148`. For every
appearance of those frames, the classifier sums only immediate child subtrees.
LLVM sometimes tail-elides the outer `RawVec` frame, leaving
`RawVecInner::finish_grow::h0ad14a629d25f3f4` directly beneath the same exact
return address; that form is admitted because the uniquely qualified parent
still proves ownership. Any other direct child is a hard refusal.

This yields complete parent-owned child counts **33/43/34** with zero residual.
No flat/aggregate `h1ea` row, required ID growth, or other `Vec<24>` consumer is
admitted. Profile log weights cover 99.3% of the corpus; the excluded tail is
priced at zero.
