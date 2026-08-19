#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.37 — prove the NEW `PARSE-COST-RATCHET` invariant can FAIL.
#
# ⛔ A CHECK WHOSE INPUTS ALL PASS HAS NOT BEEN TESTED. `accepted_rises.tsv` is not a waiver file:
# a row names an exact from/to plus an invariant that this gate RE-EVALUATES on the fresh numbers.
# That guarantee is worth exactly as much as the predicate's ability to say NO. So each refusal
# branch of `_unmatched_terminal_alternatives` is exercised here against the SHIPPED source text —
# the function is exec'd out of `scripts/check_parse_cost_ratchet.sh` itself, never re-typed, so a
# probe cannot pass against a copy that has drifted from the gate.
#
#   bash docs/tasks/artifacts/engine_universal_services/unmatched_terminal_alternatives/probe.sh
#   # UNMATCHED-TERMINAL-ALTERNATIVES: N/N as declared
#
# The totals are DERIVED from the arms that ran — never stored.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../../.." && pwd)"
cd "$ROOT" || exit 2
[ -f scripts/check_parse_cost_ratchet.sh ] || { echo "probe: not at the repo root (got $ROOT)"; exit 2; }

python3 - <<'PY'
import re, sys

SRC = "scripts/check_parse_cost_ratchet.sh"
text = open(SRC, encoding="utf-8").read()

# ⭐ Lift the predicate OUT OF THE GATE, so this probe tests the code that actually runs.
m = re.search(r"^def _unmatched_terminal_alternatives\(.*?(?=^def |^INVARIANTS)", text,
              re.S | re.M)
if not m:
    print(f"probe: could not find _unmatched_terminal_alternatives in {SRC} — REFUSING", file=sys.stderr)
    sys.exit(2)
ns = {}
exec(m.group(0), ns)                                   # noqa: S102 — the point is to run the shipped text
fn = ns["_unmatched_terminal_alternatives"]

# ⛔ It must also be REGISTERED. A predicate the gate never looks up is a predicate that cannot
# accept anything, and `accepted_rises.tsv` would refuse every row naming it.
registered = re.search(r"INVARIANTS = \{(.*?)\}", text, re.S)
is_registered = bool(registered and '"unmatched_terminal_alternatives"' in registered.group(1))

def tot(entries, committed, memo):
    return {"entries": entries, "committed": committed, "memo_hits": memo}

passed = total = 0
def arm(name, got, want):
    global passed, total
    total += 1
    if got == want:
        passed += 1
        print(f"  ✓ {name:<58} {got}")
    else:
        print(f"  ✗ {name:<58} got {got}, wanted {want}")

print("UNMATCHED-TERMINAL-ALTERNATIVES — ENGINE-UNIVERSAL-SERVICES.37")
print()

BASE = tot(417_009_457, 7_123_491, 187_512_221)

# A1 GREEN (the control): the REAL measured rise this invariant was written for.
#     4 net-new terminal alternatives x 71,988 attempted positions x (1 body + 1 memoized trivia).
holds, _ = fn(BASE, tot(417_585_361, 7_123_491, 187_800_173))
arm("A1 the real measured rise HOLDS", holds, True)

# A2 RED: committed moved — an added alternative actually MATCHED, so this is no longer
#     unmatched speculation and the acceptance's justification is gone.
holds, _ = fn(BASE, tot(417_585_361, 7_123_492, 187_800_173))
arm("A2 committed moved is REFUSED", holds, False)

# A3 RED: entries rose but memo hits did not — a rise with no memoized `trivia` companion is not
#     the shape this invariant describes.
holds, _ = fn(BASE, tot(417_585_361, 7_123_491, 187_512_221))
arm("A3 zero memo movement is REFUSED", holds, False)

# A4 RED: entries != 2 x memo hits — the `pure_memo_lookups` shape (1:1) must NOT be swallowed by
#     this invariant, or the two acceptances stop meaning different things.
holds, _ = fn(BASE, tot(417_594_709, 7_123_491, 187_800_173))
arm("A4 the 1:1 pure-memo shape is REFUSED", holds, False)

# A5 RED: memo hits FELL while entries rose — a negative companion is not an added alternative.
holds, _ = fn(BASE, tot(417_585_361, 7_123_491, 187_400_000))
arm("A5 falling memo hits is REFUSED", holds, False)

# A6 the registration arm, without which every row naming this invariant is refused as unknown.
arm("A6 the invariant is REGISTERED in INVARIANTS", is_registered, True)

print()
print(f"UNMATCHED-TERMINAL-ALTERNATIVES: {passed}/{total} as declared")
sys.exit(0 if passed == total else 1)
PY
