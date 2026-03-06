# SystemVerilog LRM Workspace

This folder stores versioned conversion artifacts for SystemVerilog LRM processing.

Source PDFs:
- `/Users/richarddje/Documents/github/SystemVerilog-LRM-IEEE-1800-2017.pdf`
- `/Users/richarddje/Documents/github/SystemVerilog-LRM-IEEE-1800-2023.pdf`

## Directory layout
- `2017/txt/`: section-based plain-text slices from IEEE 1800-2017 PDF
- `2017/md/`: markdown generated from `2017/txt/`
- `2023/txt/`: section-based plain-text slices from IEEE 1800-2023 PDF
- `2023/md/`: markdown generated from `2023/txt/`
- `*/grammar_catalog.txt`: raw extracted grammar catalog
- `*/grammar_normalized.ebnf`: normalized grammar output
- `*/grammar_clean.ebnf`: cleaned full extracted EBNF
- `*/grammar_report.json`: extraction report (counts, diagnostics)

## Quick start
Generate IEEE 1800-2017 artifacts:
```bash
python3 tools/ieee_lrm_converter.py \
  --pdf /Users/richarddje/Documents/github/SystemVerilog-LRM-IEEE-1800-2017.pdf \
  --out-root docs/systemverilog/2017 \
  --document "SystemVerilog Language Reference Manual" \
  --standard "IEEE 1800-2017" \
  --domain "SystemVerilog" \
  --clause-depth 1 \
  --toc-max-level 6 \
  --include-annex \
  --extract-grammar
```

Generate IEEE 1800-2023 artifacts:
```bash
python3 tools/ieee_lrm_converter.py \
  --pdf /Users/richarddje/Documents/github/SystemVerilog-LRM-IEEE-1800-2023.pdf \
  --out-root docs/systemverilog/2023 \
  --document "SystemVerilog Language Reference Manual" \
  --standard "IEEE 1800-2023" \
  --domain "SystemVerilog" \
  --clause-depth 1 \
  --toc-max-level 6 \
  --include-annex \
  --extract-grammar
```

Tips:
- Use `--clause-depth 2` for finer section granularity.
- Use `--limit N` for fast smoke tests.
