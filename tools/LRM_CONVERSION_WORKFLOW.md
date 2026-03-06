# LRM Conversion Workflow

This workflow adapts the earlier IEEE 1800-2017 scripts into reusable tooling for:
- IEEE 1800-2017 (SystemVerilog)
- IEEE 1800-2023 (SystemVerilog)
- IEEE 1076-2019 (VHDL)

## Scripts
- `tools/split_sections.py`
  - PDF section extraction to `section-*.txt` + `sections_manifest.json`.
  - supports two detection modes:
    - `pdf_toc` (when embedded PDF TOC exists),
    - `page_heading_fallback` (when TOC is absent).
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

## SystemVerilog (1800-2017)
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

## SystemVerilog (1800-2023)
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

## VHDL (1076-2019)
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

## Notes
- Section manifests report `detection_mode` so fallback extraction is explicit and auditable.
- Use `--limit N` for quick smoke tests.
- Use higher `--clause-depth` for finer section slicing.
