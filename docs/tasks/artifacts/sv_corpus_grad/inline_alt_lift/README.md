# The AST identity sweep and the liveness census (`SV-CORPUS-GRAD.3.42`, TOOLBOX §1.5e)

Two instruments that turn a shape-neutral claim about a grammar change into a measurement over the corpus.

| script | question | headline |
|---|---|---|
| `ast_identity_sweep.py --a <probe A> --b <probe B> --profile P --files <list>` | did ANY accepted tree move between two parser binaries? | `AST-IDENTITY-SWEEP: profile=P files=N both_accepted=… identical=… ast_moved=… verdict_moved=… both_nonaccepting=…` + one line per moved file |
| `liveness_census.py --probe <probe> --profile P --files <list> --rules r1,r2,…` | which corpus files make rule R COMMIT (appear in a successful derivation)? | `LIVENESS-CENSUS: profile=P files=N accepted=… rejected=… timeout=… nodump=… \| r1:entered_files=…,committed_files=… …` + one line per (rule, file) with `committed > 0` |

Both run the release probe per file in parallel (`--jobs`), write scratch files on the repository volume and delete
them after hashing / reading, and COUNT every `timeout` / `nodump` / `malformed` row in the headline rather than
dropping it. Both write a TSV beside the headline (`--out`).

```bash
# GREEN control first (must read ast_moved=0), then the RED control (two profiles on a file with an edition
# discriminator must read ast_moved), then the measurement:
python3 docs/tasks/artifacts/sv_corpus_grad/inline_alt_lift/ast_identity_sweep.py --a rust/target/sNNN/probe_head \
    --b rust/target/release/parseability_probe --profile sv_2017 --files rust/target/sNNN/all_sv_files.txt \
    --out rust/target/sNNN/ast_sweep_sv_2017.tsv --jobs 8
python3 docs/tasks/artifacts/sv_corpus_grad/inline_alt_lift/liveness_census.py --probe rust/target/release/parseability_probe \
    --profile sv_2017 --files rust/target/sNNN/all_sv_files.txt --rules r1,r2 --out rust/target/sNNN/liveness_sv_2017.tsv --jobs 8
```

⛔ **Read `committed` beside its verdict column.** `committed > 0` is C3-B (TOOLBOX §3.5): a tournament winner AND a
successful-but-losing branch both commit, so it is a SUPERSET of final-tree membership, and on a REJECTED row the
commits are speculation. `.3.42`'s first prediction of the sweep counted a rejected row as a mover for exactly that
reason (the sweep's 4 was right). `committed == 0` on every accepted file IS a dead rule on that corpus; the sweep
decides whether a tree moved.

⛔ **Pass the whole lane, not a candidate list, when the claim is corpus-wide.** First use (`.3.42`, release 1.0.215):
16 335 files × two SV profiles, `ast_moved=4 verdict_moved=0` on each profile in ~100 s — the four movers being
exactly the files whose text spells a shadowed branch, 9 776 / 9 797 accepted trees byte-identical; the liveness
census (~70 s per profile) named the eight lifted helpers' committing files, two of the eight committing on none.
