#!/usr/bin/env bash
# DATA-LOCALITY doctrine enforcer (DATA-LOCALITY.3): no tracked LIVE code path creates project-owned
# scratch outside the repository volume. Thin wrapper — the instrument is
# scripts/data_locality_census.py, whose docstring carries the classifier and its three wrong cuts.
set -euo pipefail
cd "$(dirname "$0")/.."
python3 scripts/data_locality_census.py --self-test
python3 scripts/data_locality_census.py --check "$@"
