# VHDL LRM Workspace

This folder stores local conversion artifacts for VHDL LRM processing.

Recommended source PDF:
- `/Users/richarddje/Documents/github/ieee-1076-2019.pdf`

## Directory layout
- `txt/`: section-based plain text slices extracted from PDF TOC
- `md/`: section-based markdown derived from `txt/`

## Quick start
```bash
python3 tools/ieee_lrm_converter.py \
  --pdf /Users/richarddje/Documents/github/ieee-1076-2019.pdf \
  --out-root docs/vhdl \
  --document "VHDL Language Reference Manual" \
  --standard "IEEE 1076-2019" \
  --domain "VHDL" \
  --clause-depth 1 \
  --extract-grammar
```

Tip:
- Increase `--clause-depth` for subsection-level markdown granularity.
