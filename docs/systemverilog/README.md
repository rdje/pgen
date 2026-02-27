# SystemVerilog LRM Workspace

This folder stores local conversion artifacts for SystemVerilog LRM processing.

Recommended source PDF:
- `/Users/richarddje/Documents/github/1800-2023.pdf`

## Directory layout
- `txt/`: section-based plain text slices extracted from PDF TOC
- `md/`: section-based markdown derived from `txt/`

## Quick start
```bash
python3 tools/ieee_lrm_converter.py \
  --pdf /Users/richarddje/Documents/github/1800-2023.pdf \
  --out-root docs/systemverilog \
  --document "SystemVerilog Language Reference Manual" \
  --standard "IEEE 1800-2023" \
  --domain "SystemVerilog" \
  --clause-depth 1 \
  --extract-grammar
```

Tip:
- Use `--clause-depth 2` for finer section granularity.
- Use `--limit N` for fast smoke tests.
