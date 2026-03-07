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
- `../../grammars/systemverilog.ebnf`: active flattened single-file dual-profile grammar used by the HDL flow
- `profiled_generation_report.json`: staged single-grammar dual-profile synthesis report (`sv_2017`, `sv_2023`)
- `../../grammars/systemverilog_lrm_profiled_generated.ebnf`: generated dual-profile grammar fragment retained for regeneration/reporting
- `../../grammars/systemverilog_lrm_profiled_wrapper.ebnf`: wrapper scaffold retained for reference/debugging

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

Profile-aware synthesis:
```bash
python3 tools/extract_systemverilog_lrm_profiles.py \
  --md-2017 docs/systemverilog/2017/md/section-41-data-read-api.md \
  --md-2023 docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md \
  --output-ebnf grammars/systemverilog_lrm_profiled_generated.ebnf \
  --output-active-ebnf grammars/systemverilog.ebnf \
  --output-report docs/systemverilog/profiled_generation_report.json
```

Note:
- the active `grammars/systemverilog.ebnf` is now the promoted flattened dual-profile grammar used by the normal HDL flow;
- `grammars/systemverilog_lrm_profiled_generated.ebnf` and `grammars/systemverilog_lrm_profiled_wrapper.ebnf` are retained as synthesis/debug artifacts;
- the active file is flattened because the current Perl `ebnf_to_json.pl` frontend does not expand `include(...)`.
