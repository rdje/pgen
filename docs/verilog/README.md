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
- `txt/`: lightweight top-level companion slices for quick inspection
- `md/`: lightweight top-level companion markdown for quick inspection
- `grammar_catalog.txt`, `grammar_normalized.ebnf`, `grammar_clean.ebnf`, `grammar_report.json`: top-level companion grammar artifacts mirrored from the versioned workspace

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
