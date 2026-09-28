---
id: a-store-epoch-names-content-not-time-so-a-rollback-restores-it
title: A store epoch names CONTENT, not time — a rollback restores the checkpoint's epoch, and a memoized success stamped before the branch is answered again
answers:
  - "why was a memoized parameter list re-parsed after a speculative branch rolled back"
  - "how does the semantic runtime's write_epoch validate store-tainted memo entries"
  - "is it sound to restore the write epoch on rollback and how are epoch collisions avoided"
  - "why does a mutation take a fresh epoch from an allocator instead of incrementing the current one"
  - "what did ENGINE-UNIVERSAL-SERVICES.111 change and what did it cost on the pinned sample"
tags: [engine, memo, semantic-store, soundness, parse-cost, rollback]
date: 2026-09-28
status: current
evidence: "ENGINE-UNIVERSAL-SERVICES.111 (PGEN-ENGINE-UNIVERSAL-SERVICES-0163, release 1.0.246): the engine change alone on grammar 1.0.245 — entries 409,344,527 -> 409,155,485 (-0.05 %), memo_hits -0.07 %, committed unchanged, generated parser byte-identical, semantic suite 387/387; with the grammar's module-scope escapes the pinned sample read entries -0.57 % / memo_hits -0.67 %; 10 022 / 10 022 / 2 283 accepted corpus trees byte-identical against the 1.0.245 probe, zero corpus movers, the two module-scope pins flipped to ACCEPT"
reverify: "cd rust && cargo test --lib write_epoch_is_a_content_identity_restored_by_rollback"
---

**A store-tainted memo entry is valid exactly when the store's CONTENT is what the entry saw. The write
epoch is that content's name: every mutation takes a value nobody has ever been handed, and a rollback
that puts the content back puts the name back. A counter that a rollback advances names time, not
content, and evicts entries whose every consulted fact still stands.**

## The defect this replaces

Through grammar `1.0.245` a discarding rollback did `write_epoch += 1`. A branch that parsed an
instantiation head's parameter list on the class route (a store-tainted success, stamped `e`), then
emitted a fact for its declarator, then failed at the `(` rolled the store back to the checkpoint — and
bumped the epoch to `e+1`. The parameter list's entry, stamped `e`, was stale by that reading although
the store it had consulted was byte-for-byte the store again; the instantiation route re-parsed the
same list at the same position (`SV-CORPUS-GRAD.28`'s first candidate: `named_parameter_assignment`
entries 51 → 102 in `aes_core.sv`, entries +5.18 %), and the grammar CONFINED its escape to class
bodies, type arguments and typedefs to avoid the cost — leaving two module-scope shapes refused.

## The mechanism

- `next_epoch` is a monotone allocator; every mutation (emit, import, scope open/close, delta apply)
  sets `write_epoch = ++next_epoch`. Two different contents never share a value.
- A discarding rollback sets `write_epoch = checkpoint.write_epoch`: facts, arena and chain are the
  checkpoint's again, so the name is too. Entries stamped with that value — inserted before the branch,
  or inside it before its first mutation — are answered again; entries stamped inside the branch after
  a mutation carry values the allocator never re-issues, so they stay unanswerable. The zero-change
  rollback fast path (`epoch unchanged ∧ deferred length unchanged ⇒ nothing to undo`) is untouched:
  equality still means identity, because a name that comes back only comes back with its content.
- The eviction the `sem_memo_*` sound anchors pin — a zero-width emission after a tainted win must
  evict it — is a mutation, and a mutation still takes a fresh name.

## What is NOT claimed

Adding a fact after the rollback still evicts every tainted entry (a fresh name), even those whose
consulted facts did not change: that axis is still the coarse global one. The change is confined to the
rollback axis, where the content provably returns. Related: [[a-fallback-arm-goes-inside-the-shared-rule-as-an-ordered-last-alternative]]
(why the escape's cost had to be measured before the ceremony).
