# LRM Conversion Workflow

This workflow adapts the earlier IEEE 1800-2017 scripts into reusable tooling for:
- IEEE 1800-2023 (SystemVerilog)
- IEEE 1076-2019 (VHDL)

## Scripts
- `tools/split_sections.py`
  - TOC-driven PDF section extraction to `section-*.txt` + `sections_manifest.json`.
- `tools/txt_to_md_converter.py`
  - converts `section-*.txt` to `section-*.md` with metadata frontmatter.
- `tools/extract_grammar.py`
  - extracts raw `::=` rules from markdown into catalog text.
- `tools/extract_grammar_v2.py`
  - dedupes/normalizes catalog into `grammar_normalized.ebnf` + JSON report.
- `tools/create_clean_grammar.py`
  - writes sorted clean EBNF from normalized input.
- `tools/ieee_lrm_converter.py`
  - end-to-end orchestrator for the full pipeline.

## SystemVerilog (1800-2023)
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

## VHDL (1076-2019)
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

## Notes
- `docs/systemverilog/.gitignore` and `docs/vhdl/.gitignore` are configured to keep generated artifacts untracked.
- Use `--limit N` for quick smoke tests.
- Use higher `--clause-depth` for finer section slicing.
