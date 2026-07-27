#!/usr/bin/env bash
# docs/tasks/artifacts/doctrine_gap_ownership/run_gap_ownership_sweep.sh
# DOCTRINE-GAP-OWNERSHIP.1 — entry point for the gap-ownership census + triage ratchet.
#
# WHY: the box-scoping defect was found, written down on 2026-07-22, and then sat for 58
# commits because it lived in a decision record's prose instead of a task-tree leaf.
# `docs/decisions/` is a MEMORY surface, not a WORK QUEUE.
#
# ⛔ A CENSUS ALONE REPEATS THE DISEASE ONE LEVEL UP. A one-shot count of "140 orphans"
# is itself a recorded-but-inert fact: nobody re-runs it, and the next session re-triages
# from zero. So this is now a RATCHET, not a report — the residue is joined against the
# TRACKED register `gap_triage_register.tsv`, and anything UNTRIAGED exits nonzero.
#
# The engine is Python (gap_ownership_sweep.py) because the original bash implementation
# had three measured defects that each produced a CONFIDENTLY WRONG census:
#   1. it matched gap phrases inside ``` fenced code blocks (4 walker examples);
#   2. its owner window was 3 lines, so a real closure annotation 5 lines above its hit
#      was reported as an orphan (project_every_parser_per_parser_book.md);
#   3. `sort` aborted on non-UTF-8 byte sequences in the TSV, truncating any tally.
# This wrapper keeps the tracked entry-point path stable.
#
# Usage:
#   run_gap_ownership_sweep.sh            # report
#   run_gap_ownership_sweep.sh --check    # ratchet: exit 1 on any UNTRIAGED residue
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
exec python3 "$ROOT/docs/tasks/artifacts/doctrine_gap_ownership/gap_ownership_sweep.py" "$@"
