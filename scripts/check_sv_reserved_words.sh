#!/usr/bin/env bash
# SV-RESERVED-WORDS doctrine enforcer (SV-CORPUS-GRAD.3.37): the SV grammar's two reserved-word
# regexes equal IEEE 1800-2017/2023 Table B.1 and IEEE 1364-2005 Annex B, derived from the in-repo LRM
# text, both directions. Thin wrapper — the instrument is stimuli/sv/check_sv_reserved_keywords.py.
set -euo pipefail
cd "$(dirname "$0")/.."
python3 stimuli/sv/check_sv_reserved_keywords.py --self-test
python3 stimuli/sv/check_sv_reserved_keywords.py "$@"
