#!/usr/bin/env bash
# SV-RELEASE-DISTANCE-CURRENCY (SV-CORPUS-GRAD.25, 2026-09-17): the doctrine wrapper. The enforcer runs a
# registered script by PATH with no arguments, so this wrapper fixes the subject: the SV parser book's
# status page. It re-derives the release-distance block from the tracked tree and refuses (rc 2) when the
# published block, or the history's last row, disagrees with the tree. RED control: a missing page → rc 2.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"
exec bash scripts/report_sv_release_distance.sh --check docs/systemverilog_parser_book/src/status-and-distance.md
