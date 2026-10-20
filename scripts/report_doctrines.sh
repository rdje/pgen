#!/usr/bin/env bash
# scripts/report_doctrines.sh — RENDER the enforced-doctrine registry as book prose.
#
# Provenance: `DOCTRINE-GAP-OWNERSHIP.20`, 2026-08-30, DIRECTOR-ORDERED. Verbatim: *"I think the
# doctrines shall be documented in the top level mdbook because you keep talking about these 33
# doctrines that are passing, so I asked myself, are these doctrines described, documented
# somewhere, then I thought about the top level mdbook."*
#
# ⛔⛔ THE DEFECT THIS CLOSES, AND IT IS A COVERAGE DEFECT IN A GREEN CHECK. `check_doctrines.sh`
# carries a meta-check `<meta:book-count>` holding the book's published doctrine count EQUAL to the
# registry size. It has been green on every run. It compares ONE INTEGER. Measured on the day this
# script was written: **14 of the 33 registered doctrines did not appear ANYWHERE in
# `docs/book/src/*.md`** -- registered, enforced on every commit, mirrored two-sided into
# `DOCTRINE_ENFORCEMENT.md` §10, and never named in the reader-facing book, with every meta-check
# green. Median debt **32 days**, max **69**, **410 doctrine-days**; three of them absent since the
# doctrine framework's own founding commit (2026-06-22), so it is an ORIGIN defect, not drift.
# ⇒ the book's NUMBER was gated and the book's CONTENT was not.
#
# ⭐⭐ DERIVED, NEVER HAND-WRITTEN, AND THAT IS THE WHOLE POINT. A hand-written chapter describing 33
# doctrines is 33 more copies to rot, and this repository has measured that rot repeatedly:
# `LIVE-MEANS-LIVE.1a` found 25 of 61 hand-maintained "last updated" fields simply wrong, and the
# book itself published **19** against a registry of **20** until `<meta:book-count>` was added. So
# the chapter is a RENDERING of the two tracked sources, on the exact `report_family_status.sh` /
# `FAMILY-STATUS-CURRENCY` shape this repository already runs on ten parser books.
#
# ⛔⛔ TWO SOURCES, AND THE SECOND ONE IS A MEASURED CORRECTION TO THIS LEAF'S OWN DESIGN. The leaf
# decided "derive the chapter from the registry, which already carries a full paragraph per
# doctrine". That premise was CHECKED before it was built on, and it is only PARTLY true:
#
#     registry description bytes : min 56   median 526   max 6235   total 35737
#     §10 "Proves" cell bytes    : min 48   median 1755  max 12478  total 90256
#     registry descriptions under 120 chars: 6 of 33
#       DIAG-SEVERITY+DOCPATH, EBNF-SOURCE-OF-TRUTH, REGEX-SELF-HOSTING, KNOWLEDGE-MAP,
#       REGEX-ORACLE-ANCHOR-SYNC, DESTRUCTIVE-TARGET-GUARD
#
# Rendering the registry alone would give those six a single clause each -- and FOUR of the six are
# among the 14 the census found absent. A one-clause entry is a MENTION, and the director's
# clarified bar is explicitly that a mention is not documentation: *"Whenever I ask for the codebase
# to be in sync, in lockstep with the top level mdbooks, I really mean that important topics,
# concept shall be full documented in the proper mdbook."* So the population, the IDs and the
# enforcer paths come from the REGISTRY (the source of truth), and the archetype and the description
# come from `DOCTRINE_ENFORCEMENT.md` §10, whose prose is the reviewed, 2.5x fuller form of the same
# claim. The two are already held ID-two-sided by `<meta:mirror>`, so the population stays closed by
# construction -- and this script REFUSES rather than renders if they ever disagree.
#
# ⭐ IT ALSO CROSS-CHECKS THE ENFORCER PATH, which `<meta:mirror>` does not: that check compares ID
# SETS only, so the registry and §10 could name different scripts for the same doctrine and both
# meta-checks would stay green. Measured before wiring it: 33 of 33 paths already agree, so this arm
# fires on NOTHING today -- the `SV-CORPUS-GRAD.13e.11` condition for a guard that is safe to make
# blocking (the fix precedes the blocking; here there is nothing to fix).
#
# ⛔ THE `\|` TRAP IS NOT RE-SOLVED HERE, IT IS IMPORTED. §10 is a markdown table and 3 of its 33
# rows carry a deliberate escaped `\|` -- two of them a shell pipe inside a code span. Splitting on
# every `|` mis-reads those rows, which is EXACTLY the trap `TABLE-ARITY-RATCHET` was founded on
# (its own naive splitter read a bug ledger 3.8x over-wide). `split_cells`/`normalise` are therefore
# imported from `docs/tasks/artifacts/live_means_live/table_pipe_repair.py`, the same module
# `check_table_arity.sh` imports, and this script REFUSES if that module is unreadable rather than
# falling back to a splitter of its own. ⭐ `normalise` (`\|` -> `|`) is the FAITHFUL transform when
# lifting a cell out of table context: GFM unescapes `\|` before parsing the cell's inline markdown,
# so `|` is what the table already renders -- leaving the backslash in would publish `\| xargs` in a
# code span, where a backslash is literal.
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict; explains on stderr;
# deterministic; does not mutate tracked state; path-agnostic; no cargo, no network.
#   0 = rendered, or (with --check) the published block equals a fresh derivation
#   1 = evaluated and BREACHED (the published block is out of sync)
#   2 = could not evaluate (a refusal, never a pass)
#
# usage:
#   bash scripts/report_doctrines.sh                  # human-readable roster on stdout
#   bash scripts/report_doctrines.sh --markdown       # the book block, for pasting/redirect
#   bash scripts/report_doctrines.sh --check FILE     # re-derive and DIFF the published block
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT" || exit 2

MODE=text; CHECK=""
while [ $# -gt 0 ]; do
  case "$1" in
    --markdown) MODE=markdown ;;
    --check) CHECK="${2:-}"; MODE=markdown
             [ -n "$CHECK" ] || { echo "report_doctrines: REFUSED (2) - --check needs a FILE" >&2; exit 2; }
             shift ;;
    -h|--help) sed -n '/^# usage:/,/^set -uo/p' "$0" | sed 's/^# \{0,1\}//;$d'; exit 0 ;;
    *) echo "report_doctrines: REFUSED (2) - unknown arg '$1'" >&2; exit 2 ;;
  esac
  shift
done

# ⛔ VALIDATE --check's SUBJECT BEFORE SPENDING THE DERIVATION, and answer both questions
# separately: a missing page and a page with no DERIVED block are different operator errors and a
# single message sends half the readers to the wrong fix.
if [ -n "$CHECK" ]; then
  [ -f "$CHECK" ] || { echo "report_doctrines: REFUSED (2) - no published page at $CHECK" >&2; exit 2; }
  if ! grep -q '<!-- BEGIN DERIVED' "$CHECK" || ! grep -q '<!-- END DERIVED -->' "$CHECK"; then
    echo "report_doctrines: REFUSED (2) - $CHECK carries no <!-- BEGIN DERIVED --> ... <!-- END DERIVED -->" >&2
    echo "                  Publish one with: bash scripts/report_doctrines.sh --markdown" >&2
    exit 2
  fi
fi

RENDERED="$(RD_MODE="$MODE" python3 - <<'PY'
import importlib.util, os, re, sys

TAG = "report_doctrines"
MODE = os.environ["RD_MODE"]

def refuse(msg):
    print(f"{TAG}: REFUSED (2) - {msg}", file=sys.stderr)
    sys.exit(2)

# --- the shared splitter, IMPORTED, never re-declared -------------------------------------------
TPR = "docs/tasks/artifacts/live_means_live/table_pipe_repair.py"
if not os.access(TPR, os.R_OK):
    refuse(f"the shared GFM cell splitter {TPR} is unreadable, so §10's 3 escaped-pipe rows would "
           f"be mis-split. A splitter of our own here is exactly the duplication TABLE-ARITY-RATCHET "
           f"was founded on; refusing instead of guessing.")
spec = importlib.util.spec_from_file_location("table_pipe_repair", TPR)
tpr = importlib.util.module_from_spec(spec)
spec.loader.exec_module(tpr)

# --- source 1: THE REGISTRY (population, ids, enforcer paths) ------------------------------------
DRIVER = "scripts/check_doctrines.sh"
try:
    driver_src = open(DRIVER, encoding="utf-8").read()
except OSError as e:
    refuse(f"cannot read the registry at {DRIVER}: {e}")
m = re.search(r"^DOCTRINES=\(\n(.*?)^\)$", driver_src, re.S | re.M)
if not m:
    refuse(f"{DRIVER} carries no DOCTRINES=( ... ) array; the population cannot be established. A "
           f"check that cannot inspect its subject must SAY SO, not render an empty chapter.")
registry = []
for line in m.group(1).splitlines():
    s = line.strip()
    if not s.startswith('"'):
        continue
    row = s[1:-1] if s.endswith('"') else s[1:]
    parts = row.split("|")
    if len(parts) != 3:
        refuse(f"a registry row does not split into exactly 3 fields (id|proves|check): {row[:70]}…")
    registry.append((parts[0], parts[1], parts[2]))
if not registry:
    refuse(f"{DRIVER}'s registry array is EMPTY. Rendering nothing would let --check pass by "
           f"construction against an empty block.")
reg_ids = [i for i, _, _ in registry]
if len(set(reg_ids)) != len(reg_ids):
    dupes = sorted({i for i in reg_ids if reg_ids.count(i) > 1})
    refuse(f"the registry names a doctrine twice: {', '.join(dupes)}")

# --- source 2: THE §10 MIRROR (archetype + the reviewed, fuller description) ---------------------
MIRROR = "DOCTRINE_ENFORCEMENT.md"
try:
    mirror_src = open(MIRROR, encoding="utf-8").read()
except OSError as e:
    refuse(f"cannot read the mirror at {MIRROR}: {e}")
sec = re.search(r"^## 10\. The live PGEN instance.*?^## 11\.", mirror_src, re.S | re.M)
if not sec:
    refuse(f"{MIRROR} §10 could not be located (section renamed, or §11 removed). The archetype and "
           f"the reviewed description live only there; rendering the registry alone would publish a "
           f"single clause for the 6 doctrines whose registry description is under 120 chars, and a "
           f"mention is not documentation.")
mirror = {}
for line in sec.group(0).splitlines():
    if not line.startswith("| `"):
        continue
    cells = [c.strip() for c in tpr.split_cells(line)]
    if len(cells) != 4:
        refuse(f"a §10 row has {len(cells)} cells, want 4 (Doctrine|Archetype|Check|Proves): "
               f"{line[:70]}…")
    did = cells[0].strip("`")
    if did in mirror:
        refuse(f"{MIRROR} §10 names {did} twice")
    mirror[did] = (cells[1], cells[2].strip("`"), tpr.normalise(cells[3]))

# --- the population is held TWO-SIDED before ANYTHING is rendered --------------------------------
# ⛔ A partial render is the failure mode this whole leaf exists to close: 14 doctrines were absent
# from the book while every meta-check stayed green. Refusing beats publishing a roster that is
# quietly short.
only_reg = sorted(set(reg_ids) - set(mirror))
only_mir = sorted(set(mirror) - set(reg_ids))
if only_reg or only_mir:
    msgs = []
    if only_reg:
        msgs.append(f"registered but absent from {MIRROR} §10: {', '.join(only_reg)}")
    if only_mir:
        msgs.append(f"in {MIRROR} §10 but not registered: {', '.join(only_mir)}")
    refuse("the registry and its §10 mirror disagree, so the chapter's population is not "
           "established — " + "; ".join(msgs) + ". Fix the mirror (<meta:mirror> gates this too), "
           "do not render a partial roster.")

# ⭐ AND THE ENFORCER PATH, which <meta:mirror> does NOT compare — it holds the ID sets only, so the
# two sources could name different scripts for one doctrine with every meta-check green.
path_diff = [(i, s, mirror[i][1]) for i, _, s in registry if s != mirror[i][1]]
if path_diff:
    refuse("the registry and §10 name DIFFERENT enforcers for the same doctrine — "
           + "; ".join(f"{i}: registry {a} vs §10 {b}" for i, a, b in path_diff)
           + ". <meta:mirror> compares ID sets only and cannot see this.")

# --- render --------------------------------------------------------------------------------------
n = len(registry)
arche_counts = {}
for i, _, _ in registry:
    a = mirror[i][0]
    arche_counts[a] = arche_counts.get(a, 0) + 1
arche_line = ", ".join(f"{k} **{v}**" for k, v in sorted(arche_counts.items()))

# --- the chapter REPORTS ITS OWN DEPTH ------------------------------------------------------------
# ⛔⛔ A ONE-CLAUSE ENTRY IS A MENTION, AND THE DIRECTOR'S BAR SAYS A MENTION IS NOT
# DOCUMENTATION. Shipping 33 sections without saying which of them are a single clause would repeat
# this leaf's founding defect one level in: a surface that LOOKS complete because it is structurally
# complete. So the block derives its own depth and SAYS SO.
#
# ⭐ THE THRESHOLD IS READ OFF THE DISTRIBUTION, NOT INVENTED. Measured 2026-08-30 over all 33
# §10 descriptions: 48 58 62 70 130 210 233 349 368 | 975 1367 1516 ... 12477. The widest relative
# gap in the lower half is 368 -> 975 (2.65x), so 500 bytes separates the one-clause class from the
# explained class cleanly, with nothing near the boundary. 9 of 33 fell below it, and all 9 are
# registry positions 2-12 — depth is a function of registration ERA, not of importance.
#
# ⚠️ HONEST LIMIT, stated rather than discovered later: this measures LENGTH, and length is a
# PROXY for depth. A 600-byte description can be vacuous and a 300-byte one can be complete. It
# catches the class the census actually found — the single clause — and nothing finer. Depth stays a
# review judgement (`DOCTRINE-GAP-OWNERSHIP.21` says the same about the general concept census), and
# this line is deliberately NOT a gate: a byte floor that blocks commits would be enforcing prose
# length, which is not the property anyone wants.
THIN_BYTES = 500
lens = sorted(len(mirror[i][2]) for i, _, _ in registry)
thin = [i for i, _, _ in registry if len(mirror[i][2]) < THIN_BYTES]
thinnest = min(registry, key=lambda r: len(mirror[r[0]][2]))[0]
if thin:
    depth_line = (f"⚠️ **Depth, derived:** median **{lens[len(lens) // 2]}** bytes per description, "
                  f"thinnest **{lens[0]}** (`{thinnest}`). **{len(thin)} of {n}** are still a single "
                  f"clause under {THIN_BYTES} bytes and are therefore *mentioned* here rather than "
                  f"*explained*: "
                  + ", ".join(f"`{i}`" for i in thin) + ". "
                  f"Enriching them is tracked, not forgotten — the description these sections render "
                  f"is `DOCTRINE_ENFORCEMENT.md` §10's, so the fix lands there and this chapter "
                  f"inherits it on the next commit.")
else:
    depth_line = (f"**Depth, derived:** median **{lens[len(lens) // 2]}** bytes per description, "
                  f"thinnest **{lens[0]}** (`{thinnest}`) — every one of the {n} clears the "
                  f"{THIN_BYTES}-byte one-clause floor. ⚠️ Length is a proxy for depth, not a "
                  f"measure of it; that this floor is clear does not make any single description "
                  f"complete.")

out = []
if MODE == "markdown":
    out.append("<!-- BEGIN DERIVED: scripts/report_doctrines.sh --markdown -->")
    out.append("")
    out.append(f"**{n} doctrines are registered and enforced.** By archetype: {arche_line}.")
    out.append("")
    out.append(depth_line)
    out.append("")
    out.append("Rows are in **registration order**, oldest first — so the bottom of this table is "
               "what the repository learned most recently.")
    out.append("")
    out.append("| # | Doctrine | Archetype | Enforcer |")
    out.append("|---:|---|---|---|")
    for k, (i, _, s) in enumerate(registry, 1):
        anchor = i.lower().replace("+", "").replace(".", "")
        out.append(f"| {k} | [`{i}`](#{anchor}) | {mirror[i][0]} | `{s}` |")
    out.append("")
    for i, _, s in registry:
        out.append(f"### `{i}`")
        out.append("")
        out.append(f"*{mirror[i][0]}* · enforced by `{s}`")
        out.append("")
        out.append(mirror[i][2])
        out.append("")
    out.append("<!-- END DERIVED -->")
else:
    out.append(f"{n} registered doctrines ({arche_line.replace('**', '')})")
    out.append("")
    out.append(f"description depth: median {lens[len(lens) // 2]} B, thinnest {lens[0]} B "
               f"({thinnest}); {len(thin)} under {THIN_BYTES} B")
    out.append("")
    w = max(len(i) for i in reg_ids)
    for k, (i, _, s) in enumerate(registry, 1):
        out.append(f"{k:>3}. {i:<{w}}  {mirror[i][0]:<30}  {s}")

sys.stdout.write("\n".join(out) + "\n")
PY
)" || exit $?

if [ -z "$RENDERED" ]; then
  echo "report_doctrines: REFUSED (2) - the derivation is EMPTY; refusing rather than emitting a" >&2
  echo "                  block that --check would then pass against by construction." >&2
  exit 2
fi

# --- --check: derive, then DIFF ------------------------------------------------------------------
# ⛔⛔ THE FLAG AND ITS IMPLEMENTATION LIVE OR DIE TOGETHER. `GRAMMAR-CERT-STATUS.1b` measured a
# sibling producer that kept its `--check` FLAG after a rewrite deleted the diff: four different
# inputs, including a path that does not exist, all exited 0 for a whole commit while three surfaces
# advertised the capability. The RED controls for this block are driven by the meta-check in
# `check_doctrines.sh` BEFORE its verdict is believed, which is what makes this paragraph a claim.
if [ -n "$CHECK" ]; then
  # CI-PARITY-GATE-ROT.121 — the fresh derivation reaches python through a FILE on the repo volume, never as an
  # argv/envp string: the page is 160 KB today and Linux refuses any single argv/envp string over MAX_ARG_STRLEN
  # (131 072 B) — `python3: Argument list too long` — which the first hosted run in 56 days turned into a false
  # "OUT OF SYNC with the registry". macOS has no per-string limit, so every local run had passed.
  rd_fresh_file="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/rust/target/report_doctrines/fresh_derivation.md"
  mkdir -p "$(dirname "$rd_fresh_file")"
  printf '%s\n' "$RENDERED" > "$rd_fresh_file"
  python3 - "$CHECK" "$rd_fresh_file" <<'PYCHECK'
import difflib, os, pathlib, re, sys
page = sys.argv[1]
fresh = pathlib.Path(sys.argv[2]).read_text(encoding="utf-8").strip()
if not fresh:
    print("report_doctrines: REFUSED (2) - the fresh derivation is EMPTY, so there is nothing to "
          "compare against; comparing would pass by construction.", file=sys.stderr)
    sys.exit(2)
m = re.search(r"<!-- BEGIN DERIVED.*?<!-- END DERIVED -->",
              pathlib.Path(page).read_text(encoding="utf-8"), re.S)
if not m:
    print(f"report_doctrines: REFUSED (2) - {page} carries no DERIVED block", file=sys.stderr)
    sys.exit(2)
published = m.group(0).strip()
if published != fresh:
    print("report_doctrines: FAILED (1) - the published doctrine chapter is OUT OF SYNC with the "
          "registry.\n  Re-derive and republish it:\n"
          "    bash scripts/report_doctrines.sh --markdown", file=sys.stderr)
    print("\n".join(difflib.unified_diff(published.split("\n"), fresh.split("\n"),
                                         f"published:{page}", "fresh derivation", lineterm="")),
          file=sys.stderr)
    sys.exit(1)
print(f"report_doctrines: OK (published chapter equals a fresh derivation, {len(fresh)} bytes)")
PYCHECK
  exit $?
fi

printf '%s\n' "$RENDERED"
