---
id: a-cat-glob-whose-output-matches-its-own-glob-fabricates-a-record
title: An aggregate whose path matches its own input glob reads back its own output on every re-run — and piping it defeats the one guard that would have caught it, so the corpus silently gains a mangled record that then fails whatever you score it with
answers:
  - "my corpus gained one row that no generator produced"
  - "a re-run of my harness finds a failure the first run did not"
  - "where did this truncated line in my concatenated file come from"
  - "why is my aggregate file not reproducible across runs"
  - "is it safe to write cat dir/prefix_*.txt > dir/prefix_all.txt"
  - "my instrument reports one defect I cannot reproduce from any source file"
  - "does cat protect me from reading the file I am writing"
  - "why did adding a pipe to my command change the output"
tags: [instruments, harnesses, shell, corpora, reproducibility, claim-verification]
date: 2026-08-23
status: current
evidence: |
  GRAMMAR-WELLFORMED.H.16.6e (`PGEN-GRAMMAR-WELLFORMED-0182`). A `reverify:` one-liner built a
  16-seed stimuli corpus with

      for s in …; do ast_pipeline … --seed $s -o rust/target/kmr_$s.txt; done
      cat rust/target/kmr_*.txt | grep -v '^[[:space:]]*$' | awk '!s[$0]++' > rust/target/kmr_own.txt

  `kmr_own.txt` matches `kmr_*.txt`. Run against a clean directory it is correct — 3 198 unique rows,
  **0 self-rejects**, agreeing with the leaf's 40-seed result of 0 over 7 994. From the SECOND run on
  it reads **3 199** rows and **1 self-reject**, on a row that appears in the aggregate and in **no**
  per-seed file: a 352-character arithmetic stimulus cut mid-token, ending on a trailing `.`, which
  of course fails to parse.

  Two-arm control, `rust/target/kmg/x_{1,2}.txt` holding 2 lines each:

  | arm | run 1 | run 2 | run 3 | stderr from run 2 |
  |---|---|---|---|---|
  | `cat x_*.txt > x_all.txt` | 4 | 4 | 4 | `cat: x_all.txt: input file is output file` |
  | `cat x_*.txt \| cat > x_all.txt` | 4 | **8** | **4 or 8** | *(silent)* |

  ⚠️ Run 3 of the piped arm is **not stable** — it is 4 on one execution and 8 on the next, because
  how much of the truncated aggregate `cat` manages to read back is a race. Non-reproducibility
  across runs of the *same* command is itself the signature.

  ⇒ BSD `cat` DOES carry a guard for exactly this, and it fires — but only when its own stdout is the
  file. Put any filter in between (`| grep`, `| awk`, `| sort`) and stdout is a pipe, the guard
  cannot see the collision, and the aggregate quietly eats itself. The same two-arm control on the
  real corpus: output inside the glob reads 3 198 / 3 199 / 3 199; output moved outside the glob
  reads 3 198 / 3 198 / 3 198.
reverify: "rm -rf rust/target/kmg && mkdir -p rust/target/kmg; printf 'alpha\\nbravo\\n' > rust/target/kmg/x_1.txt; printf 'charlie\\ndelta\\n' > rust/target/kmg/x_2.txt; echo 'ARM A direct:'; for i in 1 2 3; do cat rust/target/kmg/x_*.txt > rust/target/kmg/x_all.txt 2>rust/target/kmg/e; echo \"  run $i: $(wc -l < rust/target/kmg/x_all.txt) lines $(head -1 rust/target/kmg/e)\"; done; rm -f rust/target/kmg/x_all.txt; echo 'ARM B piped:'; for i in 1 2 3; do cat rust/target/kmg/x_*.txt | cat > rust/target/kmg/x_all.txt 2>rust/target/kmg/e; echo \"  run $i: $(wc -l < rust/target/kmg/x_all.txt) lines $(head -1 rust/target/kmg/e)\"; done"
---

The pattern is everywhere in throwaway harnesses:

```bash
cat out/part_*.txt | grep -v '^$' | awk '!seen[$0]++' > out/part_all.txt   # ⛔ part_all.txt matches part_*.txt
```

It looks correct, and on a **clean** directory it is. The trap opens on the *second* run, once the
aggregate exists to be globbed — which is precisely when a `reverify:` command, a CI step or a rebuilt
baseline runs it.

## What actually happens

1. The shell expands `part_*.txt`; last run's aggregate is now in the argument list.
2. The shell performs the redirection, truncating the aggregate to zero.
3. `cat` walks its arguments while the *tail* of the pipeline appends to that same file.
4. Reaching that argument, `cat` reads whatever has already been written — usually a **partial
   line**, because the read races the append.

So the corpus gains a record no producer emitted: a real record, cut at an arbitrary byte. Feed that
to a parser, a schema validator, a checksum or a type-checker and it fails — *plausibly*, because it
is a well-formed prefix of something real.

## The part that makes it survive review

**The one safety net does not apply.** BSD `cat` compares its inputs against its own stdout and
refuses with `input file is output file`. That check sees a **file**; in a pipeline it sees a pipe.
So the version everyone actually writes — with a `grep` or an `awk` in the middle — is exactly the
version with no protection, and it fails silently rather than loudly.

And the symptom has every property of a genuine finding: freshly generated data (so not a stale
artifact), a single row (so a rare edge case, not a harness fault), **reproducible** on every later
run, and absent from run 1 (so it reads as a regression introduced by whatever changed in between).
Here it briefly contradicted a 7 994-row measurement that had just read zero — it argued a correct
conclusion was wrong.

## The tell, and the check

**The tell:** the offending record is in the aggregate and in **no** source file. One `grep -c` across
the parts versus the aggregate settles it in seconds, and it is the first thing to run when an
aggregate produces a finding no single input reproduces.

## The practice

1. **Never let an output path match its own input glob.** Generate parts into their own directory
   (`out/parts/s_*.txt`) and write the aggregate somewhere no glob of the parts can reach.
2. `rm -rf` the parts directory at the top of the builder, so a re-run inherits nothing.
3. Prefer a builder **script** to an inline one-liner — a script can hold both guards; a one-liner
   copied into a `reverify:` field cannot.
4. Treat "run 1 was clean and the re-run was not" as a statement about the **harness** until proven
   otherwise ([[an-all-red-result-from-a-new-harness-is-a-suspicion-about-the-harness]]).

Related: [[a-control-that-cannot-fail-is-not-a-control]] ·
[[a-corpus-generated-from-the-artifact-under-test-is-part-of-the-measurement]] ·
[[an-inherited-residual-may-have-died-with-the-fix-that-came-before-it]]
