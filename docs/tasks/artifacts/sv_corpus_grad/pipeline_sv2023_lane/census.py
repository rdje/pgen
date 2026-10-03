#!/usr/bin/env python3
"""SV-CORPUS-GRAD.38 — the sv_2017 × sv_2023 census of the PREPROCESS→PARSE lane.

The pipeline lane (`stimuli/run_pipeline_lane.sh`) read every corpus file under `--profile sv_2017`
only. `PGEN_PIPELINE_PROFILE=sv_2023` runs the SAME rows, the same derived context and the same
library pass under `--profile sv_2023`, into dumps that carry the `_sv2023` suffix. This census
joins the two readings and publishes what the edition changes — the raw lanes' `.14a` census, one
level up:

  * the per-file matrix (pass / not-pass under each edition) and EVERY row whose verdict differs,
    with the text at its 2017 stop and the 2023 answer key's ruling for it;
  * the LIBRARY matrix: every package file of the design suites under each edition, the packages
    each reading publishes, and every package file whose verdict differs;
  * the UNIT reading: of the rows the tracked 2017 re-adjudication (`pipeline_readjudication.tsv`)
    leaves `unreached_in_unit`, the ones a unit that PASSES under sv_2023 includes — the members
    the 2023 reading reaches — and, the other way, any `accepted_in_unit` row whose every unit
    stops passing (a regression candidate).

The PRIMARY reading stays sv_2017: the verdict-coverage tuple under the pipeline is defined over it
and this census moves no tuple. It says where the 2023 profile adjudicates what 2017 cannot.

⛔ IDENTITY IS VERIFIED, NEVER ASSUMED. The 2017 reading's dumps must re-hash to the rows its
tracked report (`pipeline_characterization.md`) carries; the 2023 reading's dumps must re-hash to
the provenance sidecar its run wrote (`pipeline_results_sv2023.tsv.provenance`), which must name
profile `sv_2023`, a library pass, and the SAME parse binary digest as the 2017 report; the binary
on disk must be that one; and the two membership tables must be byte-identical (membership is a
property of the EXPANSION, which takes no profile — a difference means two different contexts).
Any mismatch REFUSES (exit 2).

⭐ A SECOND PAIR (SV-CORPUS-GRAD.43): `--other sv_2023_tolerant` joins the `sv_2023` reading with the reading
under the opt-in tolerance profile and writes `../pipeline_sv2023_tolerant_lane/census.{md,tsv}`. The cells are
named after the profile that passes alone (`tolerant_only_pass`: the closed list at work; `2023_only_pass`: the
SUPERSET broken — must stay 0); the unit reading is still taken against the PRIMARY (2017) re-adjudication.
Every reading other than `sv_2017` is identified by its own provenance sidecar; all share one parse binary.

Usage:
  python3 docs/tasks/artifacts/sv_corpus_grad/pipeline_sv2023_lane/census.py              # derive + write census.md/.tsv
  python3 docs/tasks/artifacts/sv_corpus_grad/pipeline_sv2023_lane/census.py --check      # re-derive, compare bytes (rc 1 on drift; an absent dump is NOT EVALUATED, rc 0)
  python3 docs/tasks/artifacts/sv_corpus_grad/pipeline_sv2023_lane/census.py --self-test  # the arithmetic + the refusals, no parser
"""
from __future__ import annotations

import argparse
import hashlib
import re
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
CH = ROOT / "stimuli/sv/characterization"
sys.path.insert(0, str(ROOT / "stimuli" / "sv"))
import pipeline_vs_raw as pv  # noqa: E402  (the lane's own readers: the join convention, the origin text, the code span)

SUFFIX = {"sv_2017": "", "sv_2023": "_sv2023", "sv_2023_tolerant": "_sv2023_tolerant"}
SHORT = {"sv_2017": "2017", "sv_2023": "2023", "sv_2023_tolerant": "tolerant"}


def reading_paths(profile: str) -> dict:
    sfx = SUFFIX[profile]
    return {"results": CH / f"pipeline_results{sfx}.tsv", "library": CH / f"pipeline_library{sfx}.tsv",
            "members": CH / f"pipeline_library_members{sfx}.tsv"}


def sidecar_path(profile: str) -> Path:
    return CH / f"pipeline_results{SUFFIX[profile]}.tsv.provenance"


# The pairs this census knows, keyed by the OTHER reading. BASE is the reading the other is measured against.
PAIRS = {
    "sv_2023": {
        "base": "sv_2017", "dir": "pipeline_sv2023_lane", "leaf": "SV-CORPUS-GRAD.38", "tag": "PIPELINE-SV2023-CENSUS",
        "other_only_meaning": "text the 2017 profile rejects and the 2023 profile accepts — adjudicated where the standard admits it",
        "base_only_meaning": "a 2023-profile REGRESSION candidate — a parser question until adjudicated",
        "primary_note": "The PRIMARY reading is `sv_2017`: the verdict-coverage tuple under the pipeline is defined over it, and this census moves no tuple.",
    },
    "sv_2023_tolerant": {
        "base": "sv_2023", "dir": "pipeline_sv2023_tolerant_lane", "leaf": "SV-CORPUS-GRAD.43", "tag": "PIPELINE-TOLERANT-CENSUS",
        "other_only_meaning": "text the standard does not derive and the opt-in tolerance profile accepts — the closed list of tool tolerances at work",
        "base_only_meaning": "the SUPERSET broken: `sv_2023` accepts it and the tolerant profile does not — must stay 0",
        "primary_note": "The tolerant profile is OPT-IN (`docs/decisions/project_sv_opt_in_tolerance_profile.md`): the primary reading stays `sv_2017`, the verdict-coverage tuple is defined over it, and this census moves no tuple.",
    },
}
BASE = "sv_2017"
OTHER = "sv_2023"
READINGS = {BASE: reading_paths(BASE), OTHER: reading_paths(OTHER)}
REPORT_2017 = CH / "pipeline_characterization.md"
READJ_2017 = CH / "pipeline_readjudication.tsv"
KEY_2023 = CH / "adjudication_manifest_sv2023.tsv"
OUT_MD = HERE / "census.md"
OUT_TSV = HERE / "census.tsv"
LEAF = "SV-CORPUS-GRAD.38"
TAG = "PIPELINE-SV2023-CENSUS"


def select_pair(other: str) -> None:
    """Point the module at one pair (the default pair is selected at import, so the functions read globals)."""
    global BASE, OTHER, READINGS, OUT_MD, OUT_TSV, LEAF, TAG
    pair = PAIRS[other]
    BASE, OTHER = pair["base"], other
    READINGS = {BASE: reading_paths(BASE), OTHER: reading_paths(OTHER)}
    out_dir = HERE.parent / pair["dir"]
    OUT_MD, OUT_TSV = out_dir / "census.md", out_dir / "census.tsv"
    LEAF, TAG = pair["leaf"], pair["tag"]
ROW_RE = re.compile(r"^\| ([^|]+?) \| `([^`]+)` \| `([0-9a-f]{64})` \|")
TEXT_WIDTH = 96


def rel(p: Path) -> str:
    return str(Path(p).resolve().relative_to(ROOT))


def sha256_of(path: Path) -> str:
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def report_rows(text: str) -> dict:
    """label -> (repo-relative path, sha256) for every identity row of the 2017 report (first wins)."""
    rows = {}
    for line in text.splitlines():
        m = ROW_RE.match(line)
        if m and m.group(1).strip() not in rows:
            rows[m.group(1).strip()] = (m.group(2), m.group(3))
    return rows


def sidecar_rows(text: str) -> dict:
    out = {}
    for line in text.splitlines():
        if "\t" in line:
            k, v = line.split("\t", 1)
            out[k] = v
    return out


def verify_identity(report: dict, sidecars: dict, on_disk: dict, base: str = "sv_2017", other: str = "sv_2023") -> list:
    """Refusals (empty = sound). `sidecars` maps a reading to its provenance rows (a reading other than `sv_2017`
    is identified by its sidecar; `sv_2017` by the tracked report). `on_disk` maps a subject to the sha the tree
    has NOW: 'parse binary' and '<short>:results' / ':library' / ':members' per reading (short = 2017, 2023,
    tolerant)."""
    refusals = []
    needed = ["parse binary", "grammar raw ast"] + (["pipeline results (output)", "library pass dump"] if base == "sv_2017" else [])
    for label in needed:
        if label not in report:
            refusals.append(f"the 2017 report carries no `{label}` row")
    pb = report.get("parse binary")
    wants = []
    if base == "sv_2017":
        wants += [("2017:results", report.get("pipeline results (output)", ("", ""))[1]),
                  ("2017:library", report.get("library pass dump", ("", ""))[1])]
    for reading in (base, other):
        if reading == "sv_2017":
            continue
        short, sidecar = SHORT[reading], sidecars.get(reading) or {}
        if not sidecar:
            refusals.append(f"the {short} reading has no provenance sidecar — a dump nothing says which run wrote; re-run the lane under PGEN_PIPELINE_PROFILE={reading}")
            return refusals
        if sidecar.get("profile") != reading:
            refusals.append(f"the {short} sidecar names profile `{sidecar.get('profile', '')}`, not `{reading}`")
        if sidecar.get("library") == "off" or "library_sha256" not in sidecar:
            refusals.append(f"the {short} reading ran WITHOUT its library pass — a different experiment from the {SHORT[base] if reading == other else SHORT[other]} reading")
        if pb and sidecar.get("probe_sha256") and sidecar["probe_sha256"] != pb[1]:
            refusals.append(f"the two readings name different parse binaries ({pb[1][:12]}… vs {sidecar['probe_sha256'][:12]}…) — a census across two instruments describes nothing")
        wants += [(f"{short}:results", sidecar.get("dump_sha256", "")),
                  (f"{short}:library", sidecar.get("library_sha256", "")),
                  (f"{short}:members", sidecar.get("library_members_sha256", ""))]
    if pb and on_disk.get("parse binary") and on_disk["parse binary"] != pb[1]:
        refusals.append(f"the binary on disk ({on_disk['parse binary'][:12]}…) is not the one the 2017 report names ({pb[1][:12]}…)")
    for subject, want in wants:
        have = on_disk.get(subject)
        if want and have and have != want:
            refusals.append(f"{subject} on disk does not re-hash to the digest its own run recorded — a dump from another run")
    a, b = on_disk.get(f"{SHORT[base]}:members"), on_disk.get(f"{SHORT[other]}:members")
    if a and b and a != b:
        refusals.append("the two readings' unit-membership tables differ — membership is a property of the expansion, which takes no profile; the two runs had different contexts")
    return refusals


def matrix(res_base: dict, res_other: dict, base_short: str = "2017", other_short: str = "2023") -> dict:
    """The join. A key in one reading only is a population breach: counted, never silently dropped."""
    keys = set(res_base) | set(res_other)
    only_base = sorted(k for k in keys if k not in res_other)
    only_other = sorted(k for k in keys if k not in res_base)
    both = sorted(k for k in keys if k in res_base and k in res_other)
    base_only, other_only = f"{base_short}_only_pass", f"{other_short}_only_pass"
    cells, pairs, moved = Counter(), Counter(), []
    for k in both:
        a, b = res_base[k], res_other[k]
        pairs[(a, b)] += 1
        pa, pb = a == "pass", b == "pass"
        cell = ("both_pass" if pa and pb else "both_nonpass" if not pa and not pb else
                base_only if pa else other_only)
        cells[cell] += 1
        if cell in (base_only, other_only):
            moved.append((cell, k, a, b))
    return {"rows": len(both), "only_in_base": only_base, "only_in_other": only_other,
            "cells": dict(cells), "status_pairs": dict(pairs), "moved": moved,
            "base_only": base_only, "other_only": other_only}


def units_of(library_rows: list, member_rows: list) -> dict:
    """(suite, member suite-relative path) -> [(package file, unit status)] for every unit including it —
    the re-adjudication's own reading (`pipeline_readjudication.read_units`), without its import weight."""
    verdict = {r[2]: r[1] for r in library_rows}
    units: dict = {}
    for suite, pkg, member in member_rows:
        if pkg in verdict:
            units.setdefault((suite, pv.suite_relpath(suite, member)), []).append((pkg, verdict[pkg]))
    return units


def published_packages(library_rows: list) -> list:
    """The packages a reading's library pass PUBLISHED: a file's own exports (column 9) plus every package
    several files declare whose variants all passed and agreed (`variants_agree:<pkg>` in the notes, column 10 —
    published once, from no single file's export column)."""
    out = set()
    for r in library_rows:
        out.update(r[8].split())
        out.update(n.split(":", 1)[1] for n in r[9].split(";") if n.startswith("variants_agree:"))
    return sorted(out)


def unit_reading(readj_rows: list, units_base: dict, units_other: dict) -> dict:
    """Of the PRIMARY (2017) re-adjudication's rows: `unreached_in_unit` rows a unit PASSING under the other
    reading includes (gained), and `accepted_in_unit` rows with no passing unit under it (lost)."""
    gained, lost, by_verdict = [], [], Counter()
    for r in readj_rows:
        key, verdict = (r["suite"], r["relpath"]), r["verdict"]
        passing = sorted(u[0] for u in units_other.get(key, []) if u[1] == "pass")
        if verdict.endswith("unreached_in_unit"):
            by_verdict["unreached_in_unit"] += 1
            if passing:
                gained.append((key, verdict, passing[0]))
        elif verdict.endswith("accepted_in_unit"):
            by_verdict["accepted_in_unit"] += 1
            if not passing:
                lost.append((key, verdict, r.get("unit", "")))
    return {"gained": gained, "lost": lost, "by_verdict": dict(by_verdict)}


def clip(text: str | None) -> str:
    if text is None:
        return ""
    t = " ".join(text.split())
    return t if len(t) <= TEXT_WIDTH else t[: TEXT_WIDTH - 1] + "…"


def cell_text(text: str) -> str:
    """A table cell holding source text: a code span that can hold its own backticks, with every `|`
    spelled `¦` (a pipe inside a code span still splits a GFM table row — TABLE-ARITY-RATCHET)."""
    return pv.code(text.replace("|", "¦")) if text else "-"


def read_readj(path: Path) -> list:
    lines = path.read_text(encoding="utf-8").splitlines()
    header = lines[0].split("\t")
    return [dict(zip(header, (l.split("\t") + [""] * len(header))[: len(header)])) for l in lines[1:] if l.strip()]


def read_key(path: Path) -> dict:
    lines = path.read_text(encoding="utf-8").splitlines()
    header = lines[0].split("\t")
    out = {}
    for l in lines[1:]:
        if l.strip():
            r = dict(zip(header, l.split("\t")))
            out[(r["suite"], r["relpath"])] = r
    return out


def absent_inputs() -> list:
    return [rel(p) for spec in READINGS.values() for p in spec.values() if not p.exists()]


def derive() -> dict:
    bs, os_ = SHORT[BASE], SHORT[OTHER]
    report = report_rows(REPORT_2017.read_text(encoding="utf-8"))
    sidecars = {r: sidecar_rows(sidecar_path(r).read_text(encoding="utf-8")) if sidecar_path(r).is_file() else {}
                for r in (BASE, OTHER) if r != "sv_2017"}
    probe = ROOT / report["parse binary"][0] if "parse binary" in report else None
    on_disk = {"parse binary": sha256_of(probe) if probe and probe.is_file() else ""}
    for reading in (BASE, OTHER):
        for k, p in READINGS[reading].items():
            on_disk[f"{SHORT[reading]}:{k}"] = sha256_of(p)
    refusals = verify_identity(report, sidecars, on_disk, BASE, OTHER)
    if refusals:
        for r in refusals:
            print(f"{TAG}: REFUSED — {r}", file=sys.stderr)
        raise SystemExit(2)

    lane = {t: pv.read_tsv(spec["results"], 7) for t, spec in READINGS.items()}
    lib = {t: pv.read_tsv(spec["library"], 10) for t, spec in READINGS.items()}
    mem = {t: pv.read_tsv(spec["members"], 3) for t, spec in READINGS.items()}
    by_key = {t: {(r[0], pv.suite_relpath(r[0], r[2])): r for r in rows} for t, rows in lane.items()}
    m = matrix({k: r[1] for k, r in by_key[BASE].items()}, {k: r[1] for k, r in by_key[OTHER].items()}, bs, os_)
    key23 = read_key(KEY_2023)
    readj = read_readj(READJ_2017)
    readj_by_key = {(r["suite"], r["relpath"]): r for r in readj}

    detail = []
    for cell, k, a, b in m["moved"]:
        rb, ro = by_key[BASE][k], by_key[OTHER][k]
        stop = rb if cell == m["other_only"] else ro              # the reading that rejects names the stop
        origin = stop[6].replace(str(ROOT) + "/", "")
        kr = key23.get(k, {})
        detail.append({"kind": "row", "cell": cell, "suite": k[0], "relpath": k[1], BASE: a, OTHER: b,
                       "stop_origin": origin, "stop_text": clip(pv.origin_line_text(origin)) if stop[1] == "fail" else "",
                       "readjudication_2017": readj_by_key.get(k, {}).get("verdict", "-"),
                       "key_2023": f"{kr.get('expected', '(not in the 2023 key)')} / {kr.get('adjudication', '')}".rstrip(" /"),
                       "unit": ""})

    lib_b = {r[2]: r for r in lib[BASE]}
    lib_o = {r[2]: r for r in lib[OTHER]}
    lm = matrix({k: r[1] for k, r in lib_b.items()}, {k: r[1] for k, r in lib_o.items()}, bs, os_)
    published = {t: published_packages(rows) for t, rows in lib.items()}
    for cell, k, a, b in lm["moved"]:
        stop = lib_b[k] if cell == lm["other_only"] else lib_o[k]
        origin = stop[7].replace(str(ROOT) + "/", "")
        suite = lib_b[k][0]
        detail.append({"kind": "package", "cell": cell, "suite": suite, "relpath": pv.suite_relpath(suite, k),
                       BASE: a, OTHER: b, "stop_origin": origin,
                       "stop_text": clip(pv.origin_line_text(origin)) if stop[1] == "fail" else "",
                       "readjudication_2017": "-", "key_2023": "-", "unit": " ".join(lib_b[k][3].split())})

    units_b, units_o = units_of(lib[BASE], mem[BASE]), units_of(lib[OTHER], mem[OTHER])
    ur = unit_reading(readj, units_b, units_o)
    for key, verdict, unit in ur["gained"]:
        detail.append({"kind": "member", "cell": f"{os_}_reaches", "suite": key[0], "relpath": key[1],
                       BASE: by_key[BASE].get(key, ["", "-"])[1], OTHER: by_key[OTHER].get(key, ["", "-"])[1],
                       "stop_origin": "", "stop_text": "", "readjudication_2017": verdict, "key_2023": "-",
                       "unit": pv.suite_relpath(key[0], unit)})
    for key, verdict, unit in ur["lost"]:
        detail.append({"kind": "member", "cell": f"{os_}_loses", "suite": key[0], "relpath": key[1],
                       BASE: by_key[BASE].get(key, ["", "-"])[1], OTHER: by_key[OTHER].get(key, ["", "-"])[1],
                       "stop_origin": "", "stop_text": "", "readjudication_2017": verdict, "key_2023": "-",
                       "unit": unit})
    order = {"row": 0, "package": 1, "member": 2}
    detail.sort(key=lambda d: (order[d["kind"]], d["cell"], d["suite"], d["relpath"]))

    identity = [("parse binary", report["parse binary"][0], report["parse binary"][1]),
                ("grammar raw ast", report["grammar raw ast"][0], report["grammar raw ast"][1]),
                (f"{BASE} pipeline results", rel(READINGS[BASE]["results"]), on_disk[f"{bs}:results"]),
                (f"{BASE} library pass dump", rel(READINGS[BASE]["library"]), on_disk[f"{bs}:library"]),
                (f"{OTHER} pipeline results", rel(READINGS[OTHER]["results"]), on_disk[f"{os_}:results"]),
                (f"{OTHER} library pass dump", rel(READINGS[OTHER]["library"]), on_disk[f"{os_}:library"]),
                ("unit membership (both readings, byte-identical)", rel(READINGS[OTHER]["members"]), on_disk[f"{os_}:members"]),
                ("sv_2017 re-adjudication table", rel(READJ_2017), sha256_of(READJ_2017)),
                ("sv_2023 answer key", rel(KEY_2023), sha256_of(KEY_2023))]
    params = {k: sidecars[OTHER].get(k, "") for k in ("timeout_s", "jobs", "files")}
    return {"matrix": m, "library": lm, "published": published, "units": ur, "detail": detail,
            "identity": identity, "params": params,
            "lane_counts": {t: dict(Counter(r[1] for r in rows)) for t, rows in lane.items()}}


def columns() -> list:
    return ["kind", "cell", "suite", "relpath", BASE, OTHER, "stop_origin", "stop_text",
            "readjudication_2017", "key_2023", "unit"]


def render_tsv(d: dict) -> str:
    cols = columns()
    lines = ["\t".join(cols)]
    for r in d["detail"]:
        lines.append("\t".join(str(r[c]).replace("\t", " ") for c in cols))
    return "\n".join(lines) + "\n"


def render_md(d: dict) -> str:
    bs, os_, pair = SHORT[BASE], SHORT[OTHER], PAIRS[OTHER]
    m, lm, ur = d["matrix"], d["library"], d["units"]
    cells, lcells = m["cells"], lm["cells"]
    base_only, other_only = m["base_only"], m["other_only"]
    rows = [r for r in d["detail"] if r["kind"] == "row"]
    pkgs = [r for r in d["detail"] if r["kind"] == "package"]
    members = [r for r in d["detail"] if r["kind"] == "member"]
    script = f"docs/tasks/artifacts/sv_corpus_grad/pipeline_sv2023_lane/census.py" + ("" if OTHER == "sv_2023" else f" --other {OTHER}")
    out = []
    out.append(f"# The {BASE} × {OTHER} census of the preprocess→parse lane ({LEAF})\n")
    out.append(f"> DERIVED. Re-run: `python3 {script}`; `--check` re-derives and diffs.\n")
    out.append("Both readings run the SAME rows — every `.sv` / `.svh` / `.v` file under `stimuli/sv/subs` plus the folded `uvm-core` tree — "
               "through `parseability_probe --parse systemverilog <file> --profile <edition> --preprocess <the file's derived context> --lib-in <the reading's own library>`, "
               "with the same binary and the same context; only `--profile` differs (the library pass included). "
               + pair["primary_note"] + "\n")
    out.append("## Instrument identity\n")
    out.append("> Re-hash the rows; if any differs, this census no longer describes your tree — re-run it, never quote it.\n")
    out.append("| input | repo-root-relative path | sha256 |\n|---|---|---|")
    for label, path, sha in d["identity"]:
        out.append(f"| {label} | `{path}` | `{sha}` |")
    out.append("")
    out.append(f"The {os_} run's own parameters (its provenance sidecar): `timeout_s={d['params']['timeout_s']}` `jobs={d['params']['jobs']}` `files={d['params']['files']}`.\n")
    out.append("## The per-file matrix\n")
    out.append("| reading | " + " | ".join(f"`{s}`" for s in pv.STATUSES) + " |\n|---|" + "---:|" * len(pv.STATUSES))
    for t in (BASE, OTHER):
        out.append(f"| `{t}` | " + " | ".join(str(d["lane_counts"][t].get(s, 0)) for s in pv.STATUSES) + " |")
    out.append("")
    out.append(f"Rows joined: **{m['rows']}**; only in the {BASE} reading: **{len(m['only_in_base'])}**; only in the {OTHER} reading: **{len(m['only_in_other'])}** (a non-zero here is a population breach, not a verdict).\n")
    out.append("| cell | rows | meaning |\n|---|---:|---|")
    out.append(f"| `both_pass` | {cells.get('both_pass', 0)} | parses under both {'editions' if OTHER == 'sv_2023' else 'profiles'} |")
    out.append(f"| `both_nonpass` | {cells.get('both_nonpass', 0)} | parses under neither |")
    out.append(f"| `{other_only}` | {cells.get(other_only, 0)} | {pair['other_only_meaning']} |")
    out.append(f"| `{base_only}` | {cells.get(base_only, 0)} | {pair['base_only_meaning']} |")
    out.append("")
    out.append(f"### Status pairs ({BASE}, {OTHER})\n")
    out.append(f"| {BASE} | {OTHER} | rows |\n|---|---|---:|")
    for (a, b), n in sorted(m["status_pairs"].items(), key=lambda kv: (-kv[1], kv[0])):
        out.append(f"| `{a}` | `{b}` | {n} |")
    out.append("")
    out.append("### Every row whose verdict differs\n")
    if not rows:
        out.append(f"(none — the two {'editions' if OTHER == 'sv_2023' else 'profiles'} agree on every row)\n")
    else:
        by_suite = Counter((r["cell"], r["suite"]) for r in rows)
        out.append("| cell | suite | rows |\n|---|---|---:|")
        for (cell, suite), n in sorted(by_suite.items()):
            out.append(f"| `{cell}` | `{suite}` | {n} |")
        out.append("")
        out.append("The stop is the REJECTING reading's own origin and the source line there; the 2017 re-adjudication verdict and the 2023 key's ruling ride beside it.\n")
        out.append("| cell | suite | relpath | stop (origin) | text at the stop | 2017 re-adjudication | 2023 key |\n|---|---|---|---|---|---|---|")
        for r in rows:
            origin = r["stop_origin"].rsplit("/", 1)[-1] if r["stop_origin"] else "-"
            out.append(f"| `{r['cell']}` | `{r['suite']}` | `{r['relpath']}` | `{origin}` | {cell_text(r['stop_text'])} | `{r['readjudication_2017']}` | `{r['key_2023']}` |")
        out.append("")
    out.append("## The library pass\n")
    out.append(f"Package files joined: **{lm['rows']}**; only in one reading: **{len(lm['only_in_base']) + len(lm['only_in_other'])}**. "
               f"Packages published: **{len(d['published'][BASE])}** under `{BASE}`, **{len(d['published'][OTHER])}** under `{OTHER}`.\n")
    out.append("| cell | package files |\n|---|---:|")
    for c in ("both_pass", "both_nonpass", other_only, base_only):
        out.append(f"| `{c}` | {lcells.get(c, 0)} |")
    out.append("")
    only_o = sorted(set(d["published"][OTHER]) - set(d["published"][BASE]))
    only_b = sorted(set(d["published"][BASE]) - set(d["published"][OTHER]))
    out.append(f"Published under `{OTHER}` only: " + (" ".join(f"`{p}`" for p in only_o) if only_o else "(none)") + ".\n")
    out.append(f"Published under `{BASE}` only: " + (" ".join(f"`{p}`" for p in only_b) if only_b else "(none)") + ".\n")
    if pkgs:
        out.append("| cell | suite | package file | packages | stop (origin) | text at the stop |\n|---|---|---|---|---|---|")
        for r in pkgs:
            origin = r["stop_origin"].rsplit("/", 1)[-1] if r["stop_origin"] else "-"
            out.append(f"| `{r['cell']}` | `{r['suite']}` | `{r['relpath']}` | `{r['unit']}` | `{origin}` | {cell_text(r['stop_text'])} |")
        out.append("")
    out.append("## The unit reading\n")
    out.append("A member file's verdict under the pipeline is its package unit's (`SVPP-EXPANSION.6d.2`). Of the rows the tracked 2017 re-adjudication reads:\n")
    gained = [r for r in members if r["cell"] == f"{os_}_reaches"]
    lost = [r for r in members if r["cell"] == f"{os_}_loses"]
    out.append(f"| 2017 reading | rows | under `{OTHER}` |\n|---|---:|---|")
    out.append(f"| `unreached_in_unit` | {ur['by_verdict'].get('unreached_in_unit', 0)} | **{len(gained)}** are included by a unit that PASSES — reached |")
    out.append(f"| `accepted_in_unit` | {ur['by_verdict'].get('accepted_in_unit', 0)} | **{len(lost)}** have no passing unit — a regression candidate |")
    out.append("")
    if gained:
        by_unit = Counter((r["suite"], r["unit"]) for r in gained)
        out.append(f"The members the {os_} reading reaches, by the passing unit that includes them (the per-member rows are in `census.tsv`, kind `member`):\n")
        out.append("| suite | unit (package file) | members reached |\n|---|---|---:|")
        for (suite, unit), n in sorted(by_unit.items(), key=lambda kv: (-kv[1], kv[0])):
            out.append(f"| `{suite}` | `{unit}` | {n} |")
        out.append("")
    if lost:
        out.append(f"The members that LOSE their passing unit under `{OTHER}`:\n")
        for r in lost:
            out.append(f"- `{r['suite']}` `{r['relpath']}` (2017 unit `{r['unit']}`)")
        out.append("")
    if m["only_in_base"] or m["only_in_other"]:
        out.append("## Population breaches\n")
        for k in m["only_in_base"][:50]:
            out.append(f"- only in {BASE}: `{k[0]}` `{k[1]}`")
        for k in m["only_in_other"][:50]:
            out.append(f"- only in {OTHER}: `{k[0]}` `{k[1]}`")
        out.append("")
    return "\n".join(out)


def headline(d: dict) -> str:
    bs, os_ = SHORT[BASE], SHORT[OTHER]
    m, lm, ur = d["matrix"], d["library"], d["units"]
    c, lc = m["cells"], lm["cells"]
    members = [r for r in d["detail"] if r["kind"] == "member"]
    return (f"{TAG}: rows={m['rows']} both_pass={c.get('both_pass', 0)} both_nonpass={c.get('both_nonpass', 0)} "
            f"{m['other_only']}={c.get(m['other_only'], 0)} {m['base_only']}={c.get(m['base_only'], 0)} "
            f"population_breaches={len(m['only_in_base']) + len(m['only_in_other'])} | "
            f"library: files={lm['rows']} {lm['other_only']}={lc.get(lm['other_only'], 0)} {lm['base_only']}={lc.get(lm['base_only'], 0)} "
            f"published_{bs}={len(d['published'][BASE])} published_{os_}={len(d['published'][OTHER])} | "
            f"members: unreached_2017={ur['by_verdict'].get('unreached_in_unit', 0)} "
            f"reached_{os_}={sum(1 for r in members if r['cell'] == f'{os_}_reaches')} "
            f"lost_{os_}={sum(1 for r in members if r['cell'] == f'{os_}_loses')}")


def self_test() -> int:
    fails = []

    def check(name, cond):
        print(f"  {'PASS' if cond else 'FAIL'}  {name}")
        if not cond:
            fails.append(name)

    a = {("s", "x.sv"): "pass", ("s", "y.sv"): "fail", ("s", "z.sv"): "pass", ("s", "w.sv"): "pp_fail", ("s", "q.sv"): "fail"}
    b = {("s", "x.sv"): "pass", ("s", "y.sv"): "pass", ("s", "z.sv"): "fail", ("s", "w.sv"): "pp_fail", ("s", "q.sv"): "timeout"}
    m = matrix(a, b)
    check("the four cells partition the joined rows", sum(m["cells"].values()) == m["rows"] == 5)
    check("a 2023-only pass is classified and named", m["cells"].get("2023_only_pass") == 1 and ("2023_only_pass", ("s", "y.sv"), "fail", "pass") in m["moved"])
    check("a 2017-only pass is classified and named", m["cells"].get("2017_only_pass") == 1 and ("2017_only_pass", ("s", "z.sv"), "pass", "fail") in m["moved"])
    check("pp_fail and timeout are not-pass, and their pair is counted", m["cells"].get("both_nonpass") == 2 and m["status_pairs"].get(("fail", "timeout")) == 1)
    m2 = matrix(a, {**b, ("s", "extra.sv"): "pass"})
    check("a key in one reading only is a POPULATION BREACH, counted, never dropped", m2["only_in_other"] == [("s", "extra.sv")] and m2["rows"] == 5)
    mt = matrix(a, b, "2023", "tolerant")
    check("the cells are named after the pair's own profiles (the tolerant pair)", mt["cells"].get("tolerant_only_pass") == 1 and mt["cells"].get("2023_only_pass") == 1 and mt["other_only"] == "tolerant_only_pass")

    sha = "a" * 64
    rep = {"parse binary": ("rust/target/debug/parseability_probe", sha), "grammar raw ast": ("grammars/systemverilog.ebnf", "b" * 64),
           "pipeline results (output)": ("r", "c" * 64), "library pass dump": ("l", "d" * 64)}
    side = {"profile": "sv_2023", "probe_sha256": sha, "dump_sha256": "e" * 64, "library": "x", "library_sha256": "f" * 64,
            "library_members_sha256": "1" * 64}
    disk = {"parse binary": sha, "2017:results": "c" * 64, "2017:library": "d" * 64, "2017:members": "1" * 64,
            "2023:results": "e" * 64, "2023:library": "f" * 64, "2023:members": "1" * 64}
    S = lambda sc: {"sv_2023": sc}
    check("matching identities are sound", verify_identity(rep, S(side), disk) == [])
    check("a 2023 dump with NO sidecar REFUSES", any("no provenance sidecar" in r for r in verify_identity(rep, {}, disk)))
    check("a sidecar naming another profile REFUSES", any("not `sv_2023`" in r for r in verify_identity(rep, S({**side, "profile": "sv_2017"}), disk)))
    check("a 2023 reading without its library pass REFUSES", any("WITHOUT its library pass" in r for r in verify_identity(rep, S({"profile": "sv_2023", "probe_sha256": sha, "dump_sha256": "e" * 64, "library": "off"}), disk)))
    check("two readings naming different binaries REFUSE", any("different parse binaries" in r for r in verify_identity(rep, S({**side, "probe_sha256": "9" * 64}), disk)))
    check("a binary on disk that is not the report's REFUSES", any("binary on disk" in r for r in verify_identity(rep, S(side), {**disk, "parse binary": "9" * 64})))
    check("a 2017 dump that does not re-hash to its report's row REFUSES", any("2017:results" in r for r in verify_identity(rep, S(side), {**disk, "2017:results": "0" * 64})))
    check("a 2023 dump that does not re-hash to its sidecar REFUSES", any("2023:results" in r for r in verify_identity(rep, S(side), {**disk, "2023:results": "0" * 64})))
    check("membership tables that differ between the readings REFUSE", any("membership tables differ" in r for r in verify_identity(rep, S({**side, "library_members_sha256": "2" * 64}), {**disk, "2023:members": "2" * 64})))
    check("a report without an identity row REFUSES", any("no `library pass dump` row" in r for r in verify_identity({k: v for k, v in rep.items() if k != "library pass dump"}, S(side), disk)))
    # the tolerant pair: BOTH readings are identified by sidecars, and each must name its own profile
    tside = {**side, "profile": "sv_2023_tolerant", "dump_sha256": "3" * 64, "library_sha256": "4" * 64}
    tdisk = {"parse binary": sha, "2023:results": "e" * 64, "2023:library": "f" * 64, "2023:members": "1" * 64,
             "tolerant:results": "3" * 64, "tolerant:library": "4" * 64, "tolerant:members": "1" * 64}
    both = {"sv_2023": side, "sv_2023_tolerant": tside}
    check("the tolerant pair is sound when both sidecars hold", verify_identity(rep, both, tdisk, "sv_2023", "sv_2023_tolerant") == [])
    check("the tolerant pair REFUSES when the BASE reading has no sidecar", any("the 2023 reading has no provenance sidecar" in r for r in verify_identity(rep, {"sv_2023_tolerant": tside}, tdisk, "sv_2023", "sv_2023_tolerant")))
    check("the tolerant pair REFUSES a tolerant sidecar naming sv_2023", any("not `sv_2023_tolerant`" in r for r in verify_identity(rep, {"sv_2023": side, "sv_2023_tolerant": {**tside, "profile": "sv_2023"}}, tdisk, "sv_2023", "sv_2023_tolerant")))
    check("the tolerant pair REFUSES a tolerant dump from another run", any("tolerant:results" in r for r in verify_identity(rep, both, {**tdisk, "tolerant:results": "0" * 64}, "sv_2023", "sv_2023_tolerant")))

    lib17 = [["s", "fail", "stimuli/sv/subs/s/p_pkg.sv"], ["s", "pass", "stimuli/sv/subs/s/q_pkg.sv"]]
    lib23 = [["s", "pass", "stimuli/sv/subs/s/p_pkg.sv"], ["s", "fail", "stimuli/sv/subs/s/q_pkg.sv"]]
    mem = [["s", "stimuli/sv/subs/s/p_pkg.sv", "stimuli/sv/subs/s/a.sv"], ["s", "stimuli/sv/subs/s/q_pkg.sv", "stimuli/sv/subs/s/b.sv"],
           ["s", "stimuli/sv/subs/s/zz_pkg.sv", "stimuli/sv/subs/s/c.sv"]]
    u17, u23 = units_of(lib17, mem), units_of(lib23, mem)
    readj = [{"suite": "s", "relpath": "a.sv", "verdict": "chained:unreached_in_unit", "unit": ""},
             {"suite": "s", "relpath": "b.sv", "verdict": "accepted_in_unit", "unit": "stimuli/sv/subs/s/q_pkg.sv"},
             {"suite": "s", "relpath": "c.sv", "verdict": "chained:unreached_in_unit", "unit": ""},
             {"suite": "s", "relpath": "d.sv", "verdict": "chained:rejects_own_file", "unit": ""}]
    ur = unit_reading(readj, u17, u23)
    check("an unreached member whose unit passes under 2023 is REACHED, with the unit named",
          ur["gained"] == [(("s", "a.sv"), "chained:unreached_in_unit", "stimuli/sv/subs/s/p_pkg.sv")])
    check("an accepted member whose unit stops passing under 2023 is LOST", [g[0] for g in ur["lost"]] == [("s", "b.sv")])
    check("a member of a unit absent from the library dump is not credited", ("s", "c.sv") not in [g[0] for g in ur["gained"]])
    check("the unit reading counts only the two unit verdicts", ur["by_verdict"] == {"unreached_in_unit": 2, "accepted_in_unit": 1})
    check("a pipe in source text cannot split a table row, and a backtick is held by its span",
          "|" not in cell_text("a | b") and cell_text("`uvm_info(x)") == "`` `uvm_info(x) ``" and cell_text("") == "-")
    check("a long stop line is clipped to the published width", len(clip("x" * 400)) == TEXT_WIDTH and clip(None) == "")
    check("a package published through agreeing variants is counted once, beside the single-file exports",
          published_packages([["s", "pass", "a.sv", "p", "0", "0", "", "", "p", ""],
                              ["s", "pass", "b.sv", "q", "0", "0", "", "", "", "variants_agree:q"],
                              ["s", "pass", "c.sv", "q", "0", "0", "", "", "", "variants_agree:q;cycle"],
                              ["s", "fail", "d.sv", "r", "0", "1", "", "", "", "variants_incomplete:r"]]) == ["p", "q"])
    side_text = "profile\tsv_2023\nprobe_sha256\t" + sha + "\n"
    check("the sidecar reader takes key<TAB>value rows", sidecar_rows(side_text) == {"profile": "sv_2023", "probe_sha256": sha})
    rows = report_rows("| parse binary | `p` | `" + sha + "` |\n| parse binary | `x` | `" + "9" * 64 + "` |\n")
    check("the FIRST identity row of a label wins", rows["parse binary"][1] == sha)
    check("every pair names a known base and its own output directory",
          all(v["base"] in SUFFIX and k in SUFFIX and v["dir"] for k, v in PAIRS.items()) and len({v["dir"] for v in PAIRS.values()}) == len(PAIRS))
    print(f"PIPELINE-SV2023-CENSUS-SELFTEST: arms=30 failed={len(fails)}")
    return 1 if fails else 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--other", default="sv_2023", choices=sorted(PAIRS),
                    help="the reading compared against its base: sv_2023 (base sv_2017, the default) or sv_2023_tolerant (base sv_2023)")
    a = ap.parse_args()
    if a.self_test:
        return self_test()
    select_pair(a.other)
    gone = absent_inputs()
    if gone:
        if a.check:
            print(f"{TAG}: NOT EVALUATED — {', '.join(gone)} absent here (the dumps are .gitignored; run the lane under both profiles to re-derive)")
            return 0
        print(f"{TAG}: REFUSED — {', '.join(gone)} absent; run the lane under both profiles first", file=sys.stderr)
        return 2
    d = derive()
    md, tsv = render_md(d), render_tsv(d)
    if a.check:
        cur_md = OUT_MD.read_text(encoding="utf-8") if OUT_MD.is_file() else ""
        cur_tsv = OUT_TSV.read_text(encoding="utf-8") if OUT_TSV.is_file() else ""
        if cur_md != md or cur_tsv != tsv:
            print(f"{TAG}: STALE — {rel(OUT_MD)} / .tsv differ from a fresh derivation; re-run without --check", file=sys.stderr)
            print(headline(d))
            return 1
        print(headline(d) + " (current)")
        return 0
    OUT_MD.parent.mkdir(parents=True, exist_ok=True)
    OUT_MD.write_text(md, encoding="utf-8")
    OUT_TSV.write_text(tsv, encoding="utf-8")
    print(headline(d))
    print(f"wrote {rel(OUT_MD)} and {rel(OUT_TSV)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
