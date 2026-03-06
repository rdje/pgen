# VHDL LRM Workspace

This folder stores versioned conversion artifacts for VHDL LRM processing.

Recommended source PDF:
- `/Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf`

## Directory layout
- `2019/txt/`: section-based plain-text slices extracted from PDF
- `2019/md/`: section-based markdown derived from `2019/txt/`
- `2019/grammar_catalog.txt`: raw extracted grammar catalog
- `2019/grammar_normalized.ebnf`: normalized grammar output
- `2019/grammar_clean.ebnf`: cleaned full extracted EBNF
- `2019/grammar_report.json`: extraction report (counts, diagnostics)

## Quick start
```bash
python3 tools/ieee_lrm_converter.py \
  --pdf /Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf \
  --out-root docs/vhdl/2019 \
  --document "VHDL Language Reference Manual" \
  --standard "IEEE 1076-2019" \
  --domain "VHDL" \
  --clause-depth 1 \
  --extract-grammar
```

Tip:
- Increase `--clause-depth` for subsection-level markdown granularity.
