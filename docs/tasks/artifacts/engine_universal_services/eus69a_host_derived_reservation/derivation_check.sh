#!/bin/bash
# `ENGINE-UNIVERSAL-SERVICES.69`(a) SLICE 2 — the derivation, read from the SHIPPED BINARY
# and cross-checked against an instrument this repository did not build.
#
# Every column here is DETERMINISTIC and load-independent: byte counts the process already
# holds at startup, and a `sysctl` reading. Nothing here is a wall-clock or a peak-RSS
# number, which matters because this slice's measurements were taken on a host at load 4.3
# (layer A: never measure peak RSS on a busy machine).
#
# THE THREE ARMS:
#   1. the resolved reservation + its provenance, from `--stack-reservation`
#   2. the SAME host quantity from `sysctl -n hw.memsize` (macOS) or /proc/meminfo (Linux)
#      -- a SECOND instrument, so the reading is corroborated rather than replicated
#   3. the operator-override arm, proving the three provenances are distinguishable
#
# Usage:  bash <this script> <path-to-parseability_probe>
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 1
probe="${1:?usage: derivation_check.sh <probe>}"

printf '## ARM 1 — the reservation this host resolved, from the binary itself\n'
env -u PGEN_PARSE_STACK_BYTES "$probe" --stack-reservation || exit 1

printf '\n## ARM 2 — the same host quantity from an instrument this repository did not build\n'
if [ "$(uname -s)" = "Darwin" ]; then
  printf 'sysctl_hw_memsize=%s\n' "$(sysctl -n hw.memsize)"
else
  printf 'proc_meminfo_memtotal_bytes=%s\n' \
    "$(( $(awk '/^MemTotal:/{print $2}' /proc/meminfo) * 1024 ))"
fi

printf '\n## ARM 3 — the operator-override arm stays distinguishable\n'
PGEN_PARSE_STACK_BYTES=134217728 "$probe" --stack-reservation | \
  grep -E '^(reservation_bytes|source|host_memory_bytes)='

printf '\n## ARM 4 — an UNPARSEABLE override is not an operator statement\n'
PGEN_PARSE_STACK_BYTES=not-a-number "$probe" --stack-reservation | \
  grep -E '^(reservation_bytes|source)='

printf '\n## ARM 5 — an override BELOW the usability floor is raised to it, and still reads as an override\n'
PGEN_PARSE_STACK_BYTES=1 "$probe" --stack-reservation | \
  grep -E '^(reservation_bytes|source)='
