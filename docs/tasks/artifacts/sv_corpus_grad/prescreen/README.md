# The grammar-candidate pre-screen (`SV-CORPUS-GRAD.3.38`, first used by `.3.41`)

Two instruments that measure a SystemVerilog grammar CANDIDATE before a byte is generated, using the
interpreter (`ast_pipeline --interpret-parse`, TOOLBOX §1.5b) on a COPY of the debug binary taken before
the slice's builds, HEAD grammar vs candidate:

| script | population | per arm | headline |
|---|---|---|---|
| `prescreen_pins.py` | every reproducer pin × its profiles (`stimuli/sv/adjudication_repros/MANIFEST.tsv`) | verdict + typed-AST sha | `PRESCREEN-PINS: jobs=N verdict_moved=V ast_moved=A` |
| `prescreen_corpus.py` | a caller-supplied list of corpus files × the SV profiles | verdict + furthest position | `PRESCREEN-CORPUS: files=N arms=M` + every moved arm |

⛔ **The corpus candidate list must contain the rows the slice was opened on, by name, before any grep
adds to it.** `.3.38` measured a broad ugrep pre-screen that returned 33 files and missed 5 of the 6 known
rows (a `\\` alternative inside a group made the regex fail on `reg [] illegal;` itself); the interpreter
read *zero movers* over 168 files while the two rows that had to move were not in the list. A non-empty
list looks tested and is not.

Typical use (`sNNN` = the slice's scratch directory under `rust/target/`, on the repository volume):

```bash
mkdir -p rust/target/sNNN && cp grammars/systemverilog.ebnf rust/target/sNNN/g_head.ebnf \
  && cp rust/target/debug/ast_pipeline rust/target/sNNN/ast_pipeline_interp
# … edit grammars/systemverilog.ebnf …
python3 docs/tasks/artifacts/sv_corpus_grad/prescreen/prescreen_pins.py --head rust/target/sNNN/g_head.ebnf \
  --new grammars/systemverilog.ebnf --bin rust/target/sNNN/ast_pipeline_interp --out rust/target/sNNN --jobs 5
python3 docs/tasks/artifacts/sv_corpus_grad/prescreen/prescreen_corpus.py --head rust/target/sNNN/g_head.ebnf \
  --new grammars/systemverilog.ebnf --bin rust/target/sNNN/ast_pipeline_interp --out rust/target/sNNN \
  --files rust/target/sNNN/corpus_candidates.txt --jobs 4
```

Both run under `scripts/run_with_memory_guard.sh` in practice (measured peaks: pins 235 MB, corpus 3 379 MB
over 336 arms). Neither reads `generated/`; the parser under measurement is the grammar, through the
interpreter, whose byte-identity to the generated parser is TOOLBOX §1.6's gate.
