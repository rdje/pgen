#!/usr/bin/env bash
# FAMILY-STATUS-CURRENCY — every parser book's published status block stays a DERIVATION of the
# tree, and the producer that derives it can still REFUSE.
#
# Provenance: `DONE-BAR-VISIBILITY.4`, 2026-08-30. The tree exists because a director said, verbatim:
# *"I have no idea for each parser the amount of work to reach full completion (Done) … there seems
# to be no end in sight."* `.1`/`.2`/`.3` built the derivation and `.9` published it into all ten
# parser books. This doctrine is what stops those ten pages becoming ten more hand-maintained
# numbers — which is not a hypothetical here: `LIVE-MEANS-LIVE.1a` measured **25 of 61**
# hand-maintained "last updated" fields simply wrong, and `SV-CORPUS-GRAD.13c.2z` found the
# SystemVerilog book publishing release `1.0.183` against a shipped `1.0.192`.
#
# ⛔⛔ AND A PUBLISHED CHECK THAT NOTHING INVOKES IS THE DEFECT, NOT THE FIX. `.9` shipped ten pages
# whose `--check` was invoked by a human; `GATE-REACHABILITY`'s founding measurement is that a gate
# nothing invokes is indistinguishable from a gate that does not exist. This is the invoker.
#
# THREE TIERS, cheapest first, each blind to what the next one sees:
#
#   A  STRUCTURAL, no derivation (~0.05 s). The population is CLOSED and TWO-SIDED: every family in
#      the done-bar register has a published page, every published page belongs to a registered
#      family, each page carries EXACTLY ONE well-formed DERIVED block, and that block's BEGIN
#      marker names the family whose book it sits in. A page holding another family's block is a
#      copy-paste this tier catches for free. ⭐ The ELEVENTH published surface — the cross-family
#      view in the platform book (`DONE-BAR-VISIBILITY.5`) — is held to the same three questions,
#      and its marker must name `--cross-family`: a per-family block pasted onto the roster page
#      would otherwise sit there reading like the whole picture while describing one family.
#   B  THE RED CONTROL (~0.01 s). `report_family_status.sh <fam> --check <path that does not exist>`
#      must exit 2. A green verdict from an instrument that says green to everything is worth
#      nothing, and this repository has shipped exactly that: `GRAMMAR-CERT-STATUS.1b`'s producer
#      kept its `--check` FLAG after a rewrite deleted the implementation, so four different inputs
#      — including a nonexistent path — all exited 0 for a whole commit.
#   C  THE ORACLE (~3.4 s). `--check-all` re-derives all ten blocks AND the cross-family view, and
#      DIFFS them against what is published. It uses the producer's own all-families mode
#      deliberately: the two audit runs are family-independent and are ~2.3 s of the cost, so ten
#      separate invocations would pay for them ten times and the doctrine would be too slow to run
#      on every commit — which is how a check ends up behind a flag nobody sets. The cross-family
#      view rides the same pass for the same reason, and because a roster page checked by a second
#      invocation is a roster page that can be forgotten.
#
# ⚠️ HONEST LIMIT, stated rather than hidden (DOCTRINE_ENFORCEMENT.md §3): this holds the PUBLISHED
# block equal to what the producer derives TODAY. Whether the producer's own inputs are right is
# their doctrines' business — `SV-CORPUS-DENOMINATOR` for the corpus census, `GATE-REACHABILITY` for
# the gate universe, and the producer's own refusals for the register and schema.
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict; explains on stderr;
# deterministic; does not mutate tracked state; path-agnostic; no cargo, no network.
#   0 = every published block equals a fresh derivation
#   1 = evaluated and BREACHED
#   2 = could not evaluate (a refusal, never a pass)
#
# Usage: bash scripts/check_family_status.sh
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT" || exit 2

PRODUCER="scripts/report_family_status.sh"
REGISTER="rust/test_data/grammar_quality/done_bar_family_register_v0.json"
TAG="family-status"

for f in "$PRODUCER" "$REGISTER"; do
  [ -f "$f" ] || { echo "$TAG: REFUSED (2) - missing $f" >&2; exit 2; }
done

# --- tier A: the closed, two-sided population + one well-formed block per page -------------------
# ⛔ PGEN_FAMILY_STATUS_PAGES_ROOT is a TESTABILITY SEAM on the audit_done_bar.sh pattern, defaulting
# to the repository so an ordinary run reads exactly what ships. It exists so the probe driver can
# CONSTRUCT the states this tier must refuse — a missing page, a duplicated block, another family's
# block — without mutating a tracked page and hoping the restore runs. A control that has never been
# seen to FAIL is an untested control, and mutating tracked files to test a gate is how a crashed
# probe leaves a dirty tree.
python3 - <<'PY' || exit $?
import glob, json, os, re, sys
TAG = "family-status"
PAGES_ROOT = os.environ.get("PGEN_FAMILY_STATUS_PAGES_ROOT") or "."
reg = json.load(open("rust/test_data/grammar_quality/done_bar_family_register_v0.json"))
fams = set(reg["families"])
pages = {}
for path in sorted(glob.glob(os.path.join(PAGES_ROOT, "docs/*_parser_book/src/status-and-distance.md"))):
    fam = os.path.basename(os.path.dirname(os.path.dirname(path)))[: -len("_parser_book")]
    pages[fam] = path

fails = []
for fam in sorted(fams - set(pages)):
    fails.append(f"{fam} is a registered family with NO published status page at "
                 f"{os.path.join(PAGES_ROOT, f'docs/{fam}_parser_book/src/status-and-distance.md')}")
for fam in sorted(set(pages) - fams):
    fails.append(f"docs/{fam}_parser_book publishes a status page but '{fam}' is not a family in the "
                 f"done-bar register — the population is closed in BOTH directions")

BEGIN = re.compile(r"<!-- BEGIN DERIVED: scripts/report_family_status\.sh (\S+) --markdown -->")
for fam, path in sorted(pages.items()):
    text = open(path, encoding="utf-8").read()
    n_begin, n_end = len(BEGIN.findall(text)), text.count("<!-- END DERIVED -->")
    if n_begin != 1 or n_end != 1:
        fails.append(f"{path} carries {n_begin} BEGIN and {n_end} END DERIVED marker(s); exactly one "
                     f"of each is required or the check cannot inspect its subject")
        continue
    named = BEGIN.search(text).group(1)
    if named != fam:
        fails.append(f"{path} publishes the block for '{named}' — that is another family's block in "
                     f"{fam}'s book")

# ⭐ THE CROSS-FAMILY VIEW, held to the same three questions as a family page. Its own seam, so the
# constructed pages-root above (which mirrors only the ten family books) leaves it pointing at the
# real page rather than reporting it missing for every family-page probe arm.
cross_page = os.environ.get("PGEN_FAMILY_STATUS_CROSS_PAGE") or "docs/book/src/roadmap-and-live-status.md"
if not os.path.isfile(cross_page):
    fails.append(f"the cross-family view is not published at {cross_page} — the roster page is the "
                 f"one surface that answers 'how far is the PROJECT', and its absence is a "
                 f"disclosure loss, not a missing nicety (DONE-BAR-VISIBILITY.5)")
else:
    text = open(cross_page, encoding="utf-8").read()
    n_begin, n_end = len(BEGIN.findall(text)), text.count("<!-- END DERIVED -->")
    if n_begin != 1 or n_end != 1:
        fails.append(f"{cross_page} carries {n_begin} BEGIN and {n_end} END DERIVED marker(s); "
                     f"exactly one of each is required or the check cannot inspect its subject")
    elif BEGIN.search(text).group(1) != "--cross-family":
        fails.append(f"{cross_page} publishes the block for '{BEGIN.search(text).group(1)}' — a "
                     f"single family's block on the page that claims to show the whole roster")

if fails:
    print(f"{TAG}: {len(fails)} structural breach(es):", file=sys.stderr)
    for f in fails:
        print(f"  ✗ {f}", file=sys.stderr)
    sys.exit(1)
print(f"{TAG}: tier A OK ({len(pages)} pages + the cross-family view at {cross_page}, population "
      f"closed both ways, one block each)")
PY

# --- tier B: the producer's own refusal must still fire ------------------------------------------
# ⛔ Driven BEFORE the oracle is believed. A check that cannot go red is not a check.
red_out="$(bash "$PRODUCER" systemverilog --check "$ROOT/rust/target/family_status_no_such_page.md" 2>&1)"
red_rc=$?
if [ "$red_rc" -ne 2 ] || [[ "$red_out" != *"no published page at"* ]]; then
  echo "$TAG: REFUSED (2) - the producer's own refusal arm did not fire: --check on a nonexistent" >&2
  echo "                  page exited $red_rc (want 2). Its green verdicts cannot be trusted until" >&2
  echo "                  it is able to refuse. Output was:" >&2
  printf '%s\n' "$red_out" | sed 's/^/                    /' >&2
  exit 2
fi
echo "$TAG: tier B OK (the producer still refuses a page that does not exist)"

# --- tier C: derive all ten and DIFF ------------------------------------------------------------
out="$(bash "$PRODUCER" --check-all 2>&1)"; rc=$?
printf '%s\n' "$out" >&2
if [ "$rc" -ne 0 ]; then
  echo "$TAG: the published status block(s) above are OUT OF SYNC with the tree." >&2
  echo "  Re-derive and republish each one:" >&2
  echo "    bash scripts/report_family_status.sh <family> --markdown" >&2
  exit "$rc"
fi
echo "$TAG: OK (all published family status blocks equal a fresh derivation)"
