# Verilog LRM Workspace

This folder stores versioned conversion artifacts for Verilog LRM processing.

Source PDF:
- `/Users/richarddje/Documents/github/Verilog-LRM-IEEE-1364-2005.pdf`

## Directory layout
- `2005/txt/`: section-based plain-text slices extracted from the IEEE 1364-2005 PDF
- `2005/md/`: section-based markdown derived from `2005/txt/`
- `2005/grammar_catalog.txt`: raw extracted grammar catalog
- `2005/grammar_normalized.ebnf`: normalized grammar output
- `2005/grammar_clean.ebnf`: cleaned full extracted EBNF
- `2005/grammar_report.json`: extraction report (counts, diagnostics)
- `../../grammars/verilog_2005_lrm_extracted.ebnf`: canonical extracted Verilog 2005 grammar snapshot promoted from `2005/grammar_clean.ebnf`
- `txt/`: lightweight top-level companion slices for quick inspection
- `md/`: lightweight top-level companion markdown for quick inspection
- `grammar_catalog.txt`, `grammar_normalized.ebnf`, `grammar_clean.ebnf`, `grammar_report.json`: top-level companion grammar artifacts mirrored from the versioned workspace

## Current promotion note
- The extracted IEEE 1364-2005 grammar snapshot is now tracked in `grammars/verilog_2005_lrm_extracted.ebnf`.
- The current extracted surface is now materially stronger:
  - `rules_extracted=943`
  - `rule_count=508`
  - `0` remaining `(From A.x.y)` placeholders in the normalized/clean grammar outputs
  - both `2005/grammar_normalized.ebnf` and `2005/grammar_clean.ebnf` now pass the normal `EBNF -> JSON` frontend path unchanged
- An active `grammars/verilog.ebnf` has still not been promoted yet because the remaining blocker has shifted from syntax-annex placeholders to unresolved bare terminals/lexical shorthand in the extracted LRM surface. Current frontend-derived unresolved-reference count is `339`, led by Verilog keywords like `module`, `endmodule`, `input`, `output`, `reg`, `signed`, `config`, and lexical-class fragments like `a`, `zA`, `Z0`, and `_`.

## Quick start
```bash
python3 tools/ieee_lrm_converter.py \
  --pdf /Users/richarddje/Documents/github/Verilog-LRM-IEEE-1364-2005.pdf \
  --out-root docs/verilog/2005 \
  --document "Verilog Hardware Description Language Reference Manual" \
  --standard "IEEE 1364-2005" \
  --domain "Verilog" \
  --clause-depth 1 \
  --toc-max-level 6 \
  --include-annex \
  --extract-grammar
```

Tip:
- Increase `--clause-depth` for subsection-level markdown granularity.
