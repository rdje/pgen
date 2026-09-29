#!/usr/bin/env bash
# SV-KEYWORD-VERSIONS doctrine enforcer (SVPP-EXPANSION.8): the preprocessor's IEEE 1800-2017 §22.14
# per-version reserved-keyword tables (rust/src/families/systemverilog/keyword_versions.rs) equal the tables the in-repo
# LRM text carries — derived on every run, both directions, and cross-checked against IEEE 1364-2005
# §19.11's own full tables, IEEE 1800-2023 §22.14 and Annex B Table B.1 as the union. Thin wrapper —
# the instrument is stimuli/sv/check_sv_keyword_versions.py, whose docstring carries the derivation.
set -euo pipefail
cd "$(dirname "$0")/.."
python3 stimuli/sv/check_sv_keyword_versions.py --self-test
python3 stimuli/sv/check_sv_keyword_versions.py "$@"
