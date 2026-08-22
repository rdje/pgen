---
id: a-heading-census-is-only-as-good-as-the-heading-grammar
title: A grep-shaped census over hand-formatted documents measures your pattern, not the corpus — and the hand-check that confirms it usually reuses the same pattern
answers:
  - "are there task-tree leaves named in the index with no leaf section"
  - "is CI-PARITY-GATE-ROT.40 / .41 / .42 missing a leaf section"
  - "how do I census headings across markdown files without over-reporting"
  - "my grep says N documents are missing a section — how do I know that is real"
  - "why did my missing-section count keep changing as I fixed the regex"
  - "how do I verify a census before publishing its number"
  - "what defines a task-tree leaf in this repository"
  - "my hand-check agreed with my script — is that confirmation"
  - "is it safe to anchor a markdown heading pattern right after the hashes"
tags: [evidence, census, instruments, markdown, task-trees, controls, false-positives, doctrine-gap-ownership]
date: 2026-08-22
status: current
evidence: "DOCTRINE-GAP-OWNERSHIP.8, opened by GRAMMAR-WELLFORMED.H.17.1 (session #256). A census of `<TREE>.<leaf>` ids named in docs/TASK_TREE.md against docs/tasks/*.md reported 38 missing, then 42, then 17 — the drops caused only by learning more of the real heading grammar, nothing in the tree changing. The first version reported CI-PARITY-GATE-ROT.40/.41/.42 as missing and that was PUBLISHED to the director; all three are present at docs/tasks/CI-PARITY-GATE-ROT.md lines 249/146/186. The pattern `^### \\`\\.40\\`` anchors the backtick immediately after the hashes, and those headings carry a status emoji first."
reverify: "python3 docs/tasks/artifacts/doctrine_gap_ownership/dangling_leaf_id_census.py   # exit code = MISSING count. Then the falsifier that matters: grep -nE '^#{2,6} [^\\`]*\\`\\.(40|41|42)\\`' docs/tasks/CI-PARITY-GATE-ROT.md  -> three hits; the naive '^### \\`\\.40\\`' -> zero. Same corpus, different pattern, opposite finding."
---

**A census written as a grep does not measure the corpus. It measures the intersection of the corpus
and your pattern** — and it reports the difference as a defect in the corpus.

The trap is not that the pattern is wrong. It is that a wrong pattern produces a *plausible, specific,
actionable* number, in the direction that makes you look diligent: a list of things that are missing.
Nothing about the output says "this is what I could see."

## The measured case

Auditing whether every task-tree leaf id named in the index has an owning section, the count went:

```text
38 missing   ->   42 missing   ->   17 missing
```

Nothing in the repository changed between those runs. Each drop was one more heading convention
learned. There were three in live use:

```markdown
### `.1` — the INVENTORY: ...                     (A) backticked id, bare
### ⛔⛔ `.40` NEW `todo` — **THE COLD-CLONE ...   (A) with a status emoji BEFORE the id
## 23. PARSE-HARNESS.11 — the blessed scratch ... (B) numbered prose, id NOT backticked
- ID: `SV-EXH-PROOF.3.3.4.b.6.2.15`               (C) a body FIELD, not a heading at all
```

The first pattern was `^### \`\.40\``, which requires the backtick immediately after the hashes.
Convention (A)-with-emoji, (B) and (C) are all invisible to it. It reported
`CI-PARITY-GATE-ROT.40/.41/.42` as unowned. All three are present, at lines 249/146/186.

**That false finding was published** before it was checked.

## The part that makes it dangerous

The hand-check run to confirm the script *used the same pattern*:

```bash
# the "independent" verification
for n in 40 41 42; do grep -c "^### \`\.$n\`" docs/tasks/CI-PARITY-GATE-ROT.md; done   # 0 0 0
```

Two agreeing measurements, one shared blind spot, zero information. This is
[[a-control-that-cannot-fail-is-not-a-control]] one level up: there, the control could not move the
artifact the instrument read; here, the check could not disagree with the pattern under test.

## The habit

**Falsify a census from the positive side before publishing the negative side.** A missing-things
census is a claim that a set is empty; the cheap falsifier is to go find one member of it by hand,
by a *different* route.

1. **Take three names the census calls MISSING and open the file.** Not grep it — open it. Three is
   enough; the conventions cluster.
2. **Enumerate the real grammar first.** `grep -oE '^#{2,6}.{0,40}' <file> | sort -u` shows what
   headings actually look like before you write a pattern that assumes.
3. **Never anchor at a fixed offset in hand-written text.** Prefixes are free text — emoji, status
   words, numbering. Match the *first token of the shape you want*, not its position.
4. **Publish the number with the grammar it assumed.** "At most 17, under the three conventions the
   census knows" is a claim. "17 are missing" is not — the next convention silently changes it.

## The finding under the finding

When a census count is a strong function of the pattern, the corpus has **no single convention** —
and that, not the count, is the thing worth reporting. Here the real defect was never "N leaves are
missing"; it was that *"is this leaf owned?"* is not mechanically answerable while three spellings
coexist. **That** is why no gate had ever read it.

A number that moves when you look harder is not a measurement yet. It is a description of how hard
you have looked.
