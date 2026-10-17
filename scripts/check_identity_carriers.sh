#!/usr/bin/env bash
# scripts/check_identity_carriers.sh
#
# DOCTRINE `IDENTITY-CARRIER-CURRENCY` (structural) — SV-CORPUS-GRAD.13i.
#
#   An artifact that tells its reader "re-hash these inputs; if any differs this report no longer
#   describes your tree" is making a checkable promise — and a promise nobody checks is a claim.
#
# ⭐ WHY THIS IS A DOCTRINE, MEASURED. Tracked artifacts under docs/tasks/artifacts and
# stimuli/*/characterization open with an `## Instrument identity` table naming their inputs and each
# input's sha256, in their own words instructing the reader to re-hash them. `.13i` measured (2026-08-14)
# that SIX live carriers existed and exactly ONE was gate-checked — and the one with a gate was the one
# that had never been stale; the other five had been stale for days. By 2026-09-03 three doctrines had
# grown readers for five of them (`CORPUS-CENSUS-CURRENCY`, `PARSE-COST-RATCHET`, `SV-CORPUS-DENOMINATOR`)
# and the last un-gated pair, `dark_chained_only/{adjudication,summary}.md`, had sat stale for 21
# grammar revisions (16 days) — re-derived, they moved zero verdict classes and six failure positions:
# STALE, not WRONG, and nothing could have said which without a re-run.
#
# WHAT IT ASSERTS, over a population DERIVED on every run — never hand-listed (`.13j` measured the
# hand list rotting inside two weeks):
#   population = every tracked `*.md` under docs/tasks/artifacts/ and stimuli/*/characterization/ that
#                carries an `## Instrument identity` heading followed by a table of sha256 rows.
#   FROZEN     = a carrier whose path has a `/before/` or `/after/` component: a dated snapshot of a
#                measurement that WAS true, kept as evidence. Reported, never judged (a guard fires on
#                live provenance, not on history — BASELINE-IDENTITY's own rule).
#   per row, the SUBJECT is the LABEL, not the path cell (`.13j`(ii): the path is a reference):
#     label matches `raw ast`   -> the semantic digest of that grammar (`--digest ebnf_raw_ast`), HARD
#                                  on mismatch; if bytes moved but this row holds, that is a NOTE
#                                  (a comment-only edit — the language is unchanged, .13r)
#     label `parse binary`      -> NOTE only: the shared probe moves whenever ANY family's parser does
#     label `generated parser`  -> NOTE only: engine drift is budgeted by CORPUS-CENSUS-CURRENCY arm (5)
#     any other label           -> bytes, HARD on mismatch (the grammar, a manifest, a corpus list)
#     a path that does not exist here (generated/, a submodule) -> NOT EVALUATED, loudly
#   STALE vs WRONG: this gate proves the report describes a DIFFERENT tree. Whether its numbers moved
#   is the producer's re-run to tell, and the message says so rather than crying "wrong".
#
# Seam: `PGEN_IDENTITY_CARRIERS` = a newline-separated file list replaces the derivation (for the bank).
# Usage:
#   bash scripts/check_identity_carriers.sh              # the doctrine
#   bash scripts/check_identity_carriers.sh --report     # every carrier, every row, its verdict
#   bash scripts/check_identity_carriers.sh --self-test  # the verdict function on literals, every cell
# Deterministic; reads the tree; mutates nothing; ~1 s.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT" || exit 2
exec python3 - "$@" <<'PYEOF'
import os, re, subprocess, sys
ROOT = os.getcwd(); TAG = "identity-carriers"; DIGESTER = "scripts/check_baseline_identity.sh"
SHA = re.compile(r"^[0-9a-f]{64}$")
ROW3 = re.compile(r"^\|\s*([^|]+?)\s*\|\s*`([^`]+)`\s*\|\s*`([0-9a-f]{64})`\s*\|\s*$")
ROW2 = re.compile(r"^\|\s*`([^`|]+)`\s*(\([^|]*\))?\s*\|\s*`([0-9a-f]{64})`\s*\|\s*$")
HEADING = re.compile(r"^## Instrument identity\b")
FROZEN = re.compile(r"/(before|after)/")

def population():
    env = os.environ.get("PGEN_IDENTITY_CARRIERS")
    if env is not None:
        return [f for f in env.split("\n") if f.strip()]
    p = subprocess.run(["git", "ls-files", "docs/tasks/artifacts/*.md", "docs/tasks/artifacts/**/*.md",
                        "stimuli/*/characterization/*.md"], capture_output=True, text=True, cwd=ROOT)
    out = []
    for f in sorted(set(p.stdout.split())):
        try:
            with open(f, encoding="utf-8") as fh:
                if any(HEADING.match(l) for l in fh):
                    out.append(f)
        except OSError:
            pass
    return out

def rows_of(path):
    """(label, subject_path, digest) per identity row; the 2-column shape uses its input cell as both."""
    rows, inside = [], False
    with open(path, encoding="utf-8") as fh:
        for l in fh:
            if HEADING.match(l): inside = True; continue
            if inside and l.startswith("## "): break
            if not inside: continue
            m = ROW3.match(l)
            if m: rows.append((m.group(1).strip(), m.group(2).strip(), m.group(3))); continue
            m = ROW2.match(l)
            if m:
                note = (m.group(2) or "")
                label = m.group(1).strip() + (" raw ast" if "ebnf_raw_ast" in note else "")
                rows.append((label, m.group(1).strip(), m.group(3)))
    return rows

def digest(kind, path):
    p = subprocess.run(["bash", DIGESTER, "--digest", kind, path], capture_output=True, text=True, cwd=ROOT)
    out = (p.stdout or "").strip().split()
    return out[0] if p.returncode == 0 and out and SHA.match(out[0]) else None

def verdict(label, recorded, live_bytes, live_sem, exists):
    """Pure function of the row and the tree — the self-test drives every cell on literals.
    -> (kind, message) with kind in pass|note|fail|unevaluated."""
    lab = label.lower()
    if not exists:
        return "unevaluated", f"`{label}` names a file absent here (untracked build product or submodule)"
    if "raw ast" in lab:
        if live_sem is None:
            return "unevaluated", f"`{label}` is a semantic digest and the digester cannot run here (build rust/target/debug/ast_pipeline)"
        if live_sem == recorded:
            return "pass", f"`{label}` re-hashes equal (semantic)"
        return "fail", f"`{label}` MOVED semantically: report {recorded[:12]}… vs tree {live_sem[:12]}… — a production, terminal or annotation changed"
    if live_bytes == recorded:
        return "pass", f"`{label}` re-hashes equal"
    if lab == "parse binary":
        return "note", f"`{label}` moved ({recorded[:12]}… vs {live_bytes[:12]}…) — the shared probe moves whenever ANY family's parser does; provenance, not staleness"
    if lab == "generated parser":
        return "note", f"`{label}` moved ({recorded[:12]}… vs {live_bytes[:12]}…) — engine drift, budgeted by CORPUS-CENSUS-CURRENCY arm (5)"
    return "fail", f"`{label}` MOVED: report {recorded[:12]}… vs tree {live_bytes[:12]}…"

def owning_oracles(path):
    """Registered oracles (the registry's script column) that name this carrier — its repo-relative PATH, or its
    BASENAME together with its parent directory as a path component (`/parse_cost_ratchet` + `cost.md`): the
    doctrines that already judge it under their OWN definition of its rows. ⛔ Never the basename alone
    (SV-CORPUS-GRAD.58c): `census.md` is a common name, and under the basename rule four live census carriers
    were "deferred" to `check_corpus_key_integrity.sh`, which names `corpus_key_audit/census.md` and never opens
    theirs — the two profile censuses sat three answer-key re-derivations stale behind that note (.58b). Honest
    limit: this is a TEXT match; an oracle that builds the path from a variable the match cannot see is not
    an owner, and a mismatch on such a carrier FAILS here — the breach it always was, re-stamped by its producer."""
    p = subprocess.run(["awk", r'/^DOCTRINES=\(/{p=1;next} p&&/^\)/{exit} p&&/^ *"/{n=split($0,a,"|"); s=a[n]; sub(/".*$/,"",s); print s}', "scripts/check_doctrines.sh"], capture_output=True, text=True, cwd=ROOT)
    rel = os.path.relpath(path, ROOT) if os.path.isabs(path) else path
    base = os.path.basename(rel); parent = "/" + os.path.basename(os.path.dirname(rel)); out = []
    for o in p.stdout.split():
        if o.endswith("check_identity_carriers.sh") or not os.path.isfile(o): continue
        try:
            t = open(o, encoding="utf-8").read()
        except OSError:
            continue
        if rel in t or (base in t and parent in t): out.append(os.path.basename(o))
    return out

def check_carrier(path, report):
    rows = rows_of(path)
    if not rows:
        return [("fail", f"{path}: carries an `## Instrument identity` heading and NO parsable sha256 row — a promise with nothing to check")]
    out, byte_moved, sem_ok = [], False, False
    sem_by_path = {}
    for label, subj, rec in rows:
        exists = os.path.isfile(subj)
        lb = digest("bytes", subj) if exists else None
        ls = digest("ebnf_raw_ast", subj) if (exists and "raw ast" in label.lower()) else None
        k, msg = verdict(label, rec, lb, ls, exists)
        # ⛔ A ROW THIS GATE CANNOT RE-DERIVE IS NOT A ROW THIS GATE MAY FAIL. `cost.md`'s `sample inputs`
        # digests the manifest ORDER plus every sampled file's bytes (its own prose says so); hashing the
        # path cell reads "moved" while PARSE-COST-RATCHET, which owns that definition, reads fresh — the
        # third instance of `.13j`(ii): the path is a reference, not always the hashed subject. So a
        # byte MISMATCH on a carrier some registered oracle already judges is DEFERRED to that oracle
        # (a note naming it), and this gate stays what `.13i` asked for: the guard on carriers NOBODY
        # else checks. A mismatch on an un-owned carrier is the breach it always was.
        if k == "fail" and "raw ast" not in label.lower():
            owners = owning_oracles(path)
            if owners:
                k, msg = "note", f"`{label}` differs from a byte re-hash of its path cell ({rec[:12]}… vs {(lb or '')[:12]}…) — deferred to {', '.join(owners)}, which judges this carrier under its own row definitions"
        if k == "fail" and "raw ast" not in label.lower() and subj.endswith(".ebnf"): byte_moved = True
        if k == "pass" and "raw ast" in label.lower(): sem_by_path[subj] = True
        out.append((k, f"{path}: {msg}"))
    # .13r: a moved grammar BYTE row beside an EQUAL semantic row for the same grammar is a NOTE, not a breach
    if byte_moved and sem_by_path:
        out = [("note", m.replace("MOVED:", "moved by BYTES only (semantic digest unchanged — comment/whitespace, no re-measure owed):")) if (k == "fail" and any(sp in m for sp in sem_by_path)) else (k, m) for k, m in out]
    return out

def self_test():
    B0, B1, S0, S1 = "a" * 64, "b" * 64, "c" * 64, "d" * 64
    cells = [
        ("bytes equal -> pass",                      ("grammar", B0, B0, None, True), "pass"),
        ("bytes moved, ordinary label -> FAIL",       ("grammar", B0, B1, None, True), "fail"),
        ("parse binary moved -> NOTE",               ("parse binary", B0, B1, None, True), "note"),
        ("generated parser moved -> NOTE",           ("generated parser", B0, B1, None, True), "note"),
        ("raw ast equal -> pass (semantic)",         ("grammar raw ast", S0, B1, S0, True), "pass"),
        ("raw ast moved -> FAIL semantically",        ("grammar raw ast", S0, B1, S1, True), "fail"),
        ("raw ast underivable -> NOT EVALUATED",     ("grammar raw ast", S0, B1, None, True), "unevaluated"),
        ("absent file -> NOT EVALUATED",             ("generated parser", B0, None, None, False), "unevaluated"),
        ("2-col row shape parses",                    None, None),
        ("annotated (ebnf_raw_ast) 2-col row is semantic", None, None),
        ("frozen path rule",                          None, None),
        ("owning_oracles: cost.md owned, dark summary un-owned", None, None),
        ("owning_oracles: a basename alone owns nothing (.58c)", None, None),
        ("owning_oracles: the oracle naming the carrier's directory owns it (.58c)", None, None),
    ]
    misses = []
    for label, args, want in cells:
        if args is None: continue
        got, _ = verdict(*args)
        if got != want: misses.append(f"{label}: got {got}")
    if not ROW2.match("| `stimuli/x/results.tsv` | `" + "e" * 64 + "` |"): misses.append("2-col row shape does not parse")
    m2 = ROW2.match("| `grammars/x.ebnf` (ebnf_raw_ast) | `" + "e" * 64 + "` |")
    if not (m2 and "ebnf_raw_ast" in (m2.group(2) or "")): misses.append("annotated 2-col row (ebnf_raw_ast) does not parse as semantic")
    if owning_oracles("docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet/cost.md") == []: misses.append("owning_oracles finds no owner for cost.md (PARSE-COST-RATCHET names it)")
    if owning_oracles("docs/tasks/artifacts/sv_corpus_grad/dark_chained_only/summary.md") != []: misses.append("owning_oracles claims an owner for the un-gated dark_chained_only summary")
    # .58c: `census.md` is named by two oracles that never open THIS census — a basename alone must own nothing
    if owning_oracles("docs/tasks/artifacts/sv_corpus_grad/lrm_keyword_enumeration/census.md") != []: misses.append("owning_oracles defers lrm_keyword_enumeration/census.md by its basename alone (no oracle names its directory)")
    own_pipe = owning_oracles("docs/tasks/artifacts/sv_corpus_grad/pipeline_sv2023_lane/census.md")
    if "check_pipeline_lane_currency.sh" not in own_pipe: misses.append("owning_oracles misses PIPELINE-LANE-CURRENCY for pipeline_sv2023_lane/census.md (its arm 5)")
    if "check_corpus_key_integrity.sh" in own_pipe: misses.append("owning_oracles still defers pipeline_sv2023_lane/census.md to check_corpus_key_integrity.sh, which never opens it")
    if not (FROZEN.search("docs/tasks/artifacts/x/before/report.md") and not FROZEN.search("docs/tasks/artifacts/x/report.md")):
        misses.append("frozen path rule wrong")
    if misses:
        print(f"{TAG}: self-test FAILED — {misses}", file=sys.stderr); sys.exit(2)
    print(f"{TAG}: self-test OK — {len(cells)} controls, every verdict cell, both structural rules and the ownership rule observed")

args = sys.argv[1:]
if "--self-test" in args: self_test(); sys.exit(0)
report = "--report" in args
self_test()
pop = population()
if not pop:
    print(f"{TAG}: REFUSE — the derived population is EMPTY; a gate with nothing to check reports a clean tree for free", file=sys.stderr); sys.exit(2)
live = [f for f in pop if not FROZEN.search(f)]; frozen = [f for f in pop if FROZEN.search(f)]
fails, notes, unev, passes = [], [], [], 0
for f in live:
    for k, m in check_carrier(f, report):
        if k == "fail": fails.append(m)
        elif k == "note": notes.append(m)
        elif k == "unevaluated": unev.append(m)
        else: passes += 1
        if report: print(f"  {k:<11} {m}")
for m in unev: print(f"{TAG}: NOT EVALUATED — {m}", file=sys.stderr)
for m in notes: print(f"{TAG}: NOTE — {m}", file=sys.stderr)
if fails:
    print(f"{TAG}: {len(fails)} carrier row(s) describe a DIFFERENT tree (STALE — whether their numbers also moved is the producer's re-run to tell):", file=sys.stderr)
    for m in fails: print(f"  ✗ {m}", file=sys.stderr)
    sys.exit(1)
print(f"IDENTITY-CARRIERS: OK (live={len(live)} frozen={len(frozen)} rows_equal={passes} notes={len(notes)} not_evaluated={len(unev)})")
PYEOF
