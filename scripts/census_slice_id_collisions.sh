#!/usr/bin/env bash
# DOCTRINE-GAP-OWNERSHIP.18 — the PRODUCER for this repo's slice-id uniqueness census.
#
# `docs/TASK_TREE.md` calls `PGEN-<FAMILY>-<NNNN>` "the unit of commit traceability", and nothing
# derives, allocates or checks it. This script measures how far that has drifted, so the number in
# the leaf is RE-DERIVED on demand instead of re-typed from a run that has scrolled away.
#
# ⛔⛔ THE DENOMINATOR IS THE TRAP, AND IT CAUGHT THIS CENSUS'S OWN FIRST TWO PASSES.
#   A leading-token extractor (regex or split, they agree) reads only the FIRST token of the subject
#   and therefore MISSES the 361 subjects that carry their id later in the line. Two methods agreeing
#   is not corroboration when they share the blind spot. This script scans the WHOLE subject.
#
# ⛔ NOT A GATE and deliberately not one: 155 existing commits collide, git history here is
#   append-only, and a blocker firing on all of them would teach bypass (`DOCTRINE-GAP-OWNERSHIP.15`).
#   Exit is 0 whatever it finds. `--next <FAMILY>` is the forward-only half: it prints the next
#   unused id for a family, which is the allocator whose absence is the actual defect.
#
# USAGE
#   census_slice_id_collisions.sh                 # the census
#   census_slice_id_collisions.sh --list          # census + every colliding id and its commits
#   census_slice_id_collisions.sh --next FAMILY   # next free id, e.g. --next ENGINE-UNIVERSAL-SERVICES
set -uo pipefail
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$ROOT" || exit 1

MODE="${1:-census}"; FAMILY="${2:-}"

git log --format='%H%x1f%s' | python3 -c '
import sys, re, collections
mode = sys.argv[1]; family = sys.argv[2] if len(sys.argv) > 2 else ""
# A family segment is UPPERCASE; the numeric tail must not be followed by another digit/letter/dash,
# so `PGEN-RGX-0078-0102` yields the full id and not the `-0078` prefix.
pat = re.compile(r"PGEN-[A-Z][A-Z0-9-]*?-\d{3,4}(?![\dA-Za-z-])")
ids = collections.defaultdict(list); noid = 0
for line in sys.stdin:
    line = line.rstrip("\n")
    if "\x1f" not in line: continue
    h, s = line.split("\x1f", 1)
    m = pat.search(s)
    if m: ids[m.group(0)].append(h)
    else: noid += 1
dup = {k: v for k, v in ids.items() if len(v) > 1}
carried = sum(len(v) for v in ids.values())

if mode == "--next":
    if not family:
        print("--next needs a FAMILY, e.g. --next ENGINE-UNIVERSAL-SERVICES"); raise SystemExit(2)
    pre = "PGEN-%s-" % family
    nums = [int(k[len(pre):]) for k in ids if k.startswith(pre) and k[len(pre):].isdigit()]
    if not nums:
        print("%s0001   (no existing id for family %s)" % (pre, family)); raise SystemExit
    print("%s%04d   (highest existing = %s%04d over %d ids)" % (pre, max(nums)+1, pre, max(nums), len(nums)))
    raise SystemExit

print("commits in history                        : %d" % (carried + noid))
print("  carrying a PGEN-<FAMILY>-<NNNN> id      : %d" % carried)
print("  carrying NO id (older convention)       : %d" % noid)
print("distinct ids                              : %d" % len(ids))
print("ids naming MORE THAN ONE commit           : %d" % len(dup))
print("commits NOT uniquely addressable by id    : %d  (%.1f%% of id-bearing commits)"
      % (sum(len(v) for v in dup.values()), 100.0 * sum(len(v) for v in dup.values()) / carried if carried else 0.0))
if mode == "--list":
    print("--- every colliding id, worst first ---")
    for k, v in sorted(dup.items(), key=lambda x: (-len(x[1]), x[0])):
        print("  %-44s x%d" % (k, len(v)))
        for h in v:
            print("      %s  %s" % (h[:8], __import__("subprocess").run(
                ["git","log","-1","--format=%ad","--date=short",h],
                capture_output=True, text=True).stdout.strip()))
' "$MODE" "$FAMILY"
