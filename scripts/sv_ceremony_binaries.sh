#!/usr/bin/env bash
# scripts/sv_ceremony_binaries.sh — the SystemVerilog release ceremony's measured binaries are KEPT, not re-derivable.
#
# SV-CORPUS-GRAD.53 (2026-10-09). THE FINDING, MEASURED ON RELEASE 1.0.252: the ceremony builds the debug `ast_pipeline`,
# the debug `parseability_probe` and the lowmem release `parseability_probe` once, publishes them at
# `rust/target/{debug,release}/…`, and every lane, adjudication, sweep and pipeline reading names the bytes it measured
# with (the `parse binary` row of each tracked report). Later chain steps REBUILD those paths — `verilog_2005_conformance_gate`
# rebuilds the release probe (the release path held `b7ce38c4…` from 12:19 while every raw lane report named `6af036ea…`),
# the parity / equivalence gates and the lib suite rebuild the debug binaries (the debug probe the three pipeline readings
# named, `5a60442e…`, was gone by 17:53) — and a rebuild is NOT byte-reproducible on this host (the identical tree rebuilt
# to `c4df243c…`, same parser fingerprint, different bytes). A tracked identity row then names a binary that no longer
# exists anywhere: the adjudicator's store-gate REFUSED (correctly) until the lowmem copy was put back by hand, and the
# three pipeline readings had to be RE-RUN on a rebuilt probe (`.52`). The debug probe had no copy at all.
#
# THE RULE: copy the three binaries aside the moment they are built, put them BACK at their published paths before every
# measuring step that follows a rebuilding gate and as the LAST step of the chain, and VERIFY at the end that the copies,
# the published paths, every tracked report and the store-gate memo all name the same bytes. The copies are the ceremony's
# recovery point too (the 2026-10-08 host reboot resumed from `rust/target/sv48/` — with sha_final.txt, not with bytes).
#
# ⛔ Why RESTORE rather than pointing every tool at the copies: the lane runner RECORDS the probe path it measured with
# in the tracked report's identity row and adopts that path on its next run (`stimuli/run_external_corpus.sh`, the
# `.3.27` provenance reconcile); a tracked report naming `rust/target/svNN/bin/…` would send the next ceremony's lanes to
# a path the §8 artifact cleanup deletes within a week. The published paths are the ONLY paths a tracked artifact may
# name; the copies guarantee what those paths hold.
#
# VERBS
#   snapshot <scratch-dir>   copy rust/target/release/parseability_probe, rust/target/debug/parseability_probe and
#                            rust/target/debug/ast_pipeline into <scratch-dir>/bin/{release,debug}/, warm the two probes
#                            once (`--parser-fingerprint`; the first launch of a fresh 83 MB inode costs ~43 s here),
#                            refuse when the two probes carry different systemverilog fingerprints (a stale build), and
#                            write <scratch-dir>/bin/identity.tsv (full digests + fingerprints, plus the generated
#                            parser's and the grammar's digests as RECORDED rows) and <scratch-dir>/sha_final.txt (the
#                            short form the recovery procedure reads)
#   restore  <scratch-dir>   put every copy back at its published path when the path holds other bytes (or nothing);
#                            REFUSES when a copy itself no longer matches identity.tsv
#   verify   <scratch-dir>   the RED arm — (1) every copy matches identity.tsv; (2) every published path a tracked row
#                            names holds its copy — except the UPLIFTED debug ast_pipeline, a NOTE when it differs: cargo
#                            re-uplifts it on every build of its feature set (every doctrines run, the pre-commit hook) after
#                            the chain's last restore, so it cannot be held at commit time; SV-CORPUS-GRAD.56);
#                            (3) every tracked lane report's `parse binary` row names the published path of its role and
#                            the copy's digest (raw lanes → release, pipeline readings → debug); (4) the store-gate memo
#                            names each lane report's binary (`store_gate_explanations.py --check-identity`); (5) the
#                            generated parser on disk is the recorded one (NOT EVALUATED when absent)
#   --self-test              nine arms on a scratch tree (rc 1 on any miss)
#
# CONTRACT: exit 0 holds, 1 a breach (named on stderr), 2 refused (a subject is missing); deterministic; `snapshot` and
# `restore` write only under <scratch-dir> and the three published paths; `verify` writes nothing. Repo-relative paths.
# SEAMS (for the self-test): PGEN_SV_CEREMONY_ROOT = the tree to inspect; PGEN_SV_CEREMONY_NO_WARM=1 skips the warm.
set -uo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$REPO" || exit 2
export PGEN_SV_CEREMONY_REPO="$REPO"
exec python3 - "$@" <<'PYEOF'
import hashlib, json, os, shutil, subprocess, sys, tempfile
from pathlib import Path

TAG = "SV-CEREMONY-BINARIES"
REPO = Path(os.environ["PGEN_SV_CEREMONY_REPO"]).resolve()            # where this script and the memo module live
ROOT = Path(os.environ.get("PGEN_SV_CEREMONY_ROOT") or REPO).resolve()  # the tree inspected (a seam for the self-test)
WARM = os.environ.get("PGEN_SV_CEREMONY_NO_WARM") != "1"
MEMO_MODULE = "stimuli/sv/store_gate_explanations.py"
CHAR = "stimuli/sv/characterization"
# role -> (published path, the tracked reports whose identity row of the given LABEL must name that path and the copy's digest)
# SV-CORPUS-GRAD.56: the debug ast_pipeline IS named by a tracked row — the rejects-own-file census's `expansion binary` —
# so arm (3) holds that row to the ceremony's copy; arm (2) reports the published path's drift as a NOTE for this role only,
# because cargo re-uplifts the binary on every build of its feature set (every doctrines run, the pre-commit hook) AFTER
# the chain's last restore, so the path cannot be held to the ceremony's bytes at commit time (measured on 1.0.253).
ROLES = {
    "release": ("rust/target/release/parseability_probe",
                [(f"{CHAR}/characterization.md", "parse binary"), (f"{CHAR}/characterization_v2005.md", "parse binary"),
                 (f"{CHAR}/characterization_sv2023.md", "parse binary")]),
    "debug": ("rust/target/debug/parseability_probe",
              [(f"{CHAR}/pipeline_characterization.md", "parse binary"), (f"{CHAR}/pipeline_readjudication.md", "parse binary")]),
    "pipeline": ("rust/target/debug/ast_pipeline",
                 [("docs/tasks/artifacts/sv_corpus_grad/rejects_own_file_census/census.md", "expansion binary")]),
}
UPLIFTED = {"pipeline"}   # roles whose published path cargo re-uplifts on every build of its feature set: arm (2) notes, never breaches
RECORDED = ["generated/systemverilog_parser.rs", "grammars/systemverilog.ebnf"]   # digested at snapshot time, not copied
PROBES = ("release", "debug")                                                     # the two that answer --parser-fingerprint


def sha(p: Path) -> str:
    h = hashlib.sha256()
    with open(p, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def copy_path(scratch: Path, role: str) -> Path:
    published = Path(ROLES[role][0])
    return scratch / "bin" / published.parent.name / published.name


def fingerprint(binary: Path) -> str:
    """The probe's own `parsers.systemverilog` fingerprint, or '' when it cannot answer."""
    try:
        out = subprocess.run([str(binary), "--parser-fingerprint"], capture_output=True, text=True, timeout=600)
        return json.loads(out.stdout).get("parsers", {}).get("systemverilog", "") if out.returncode == 0 else ""
    except (OSError, ValueError, subprocess.TimeoutExpired):
        return ""


def read_identity(scratch: Path) -> dict:
    """role -> {published, sha256, fingerprint}; RECORDED rows under role 'recorded:<path>'."""
    f = scratch / "bin" / "identity.tsv"
    if not f.is_file():
        raise SystemExit(f"{TAG}: REFUSED (2) — no {f}; run `snapshot` first")
    rows = {}
    for line in f.read_text(encoding="utf-8").splitlines()[1:]:
        role, published, digest, fp = line.split("\t")
        rows[role] = {"published": published, "sha256": digest, "fingerprint": fp}
    return rows


def report_row(report: Path, label: str = "parse binary"):
    """(path cell, digest) of the report's identity row carrying `label`, or None."""
    sys.path.insert(0, str(REPO / "stimuli/sv"))
    import importlib.util
    spec = importlib.util.spec_from_file_location("sg", REPO / MEMO_MODULE)
    sg = importlib.util.module_from_spec(spec); spec.loader.exec_module(sg)
    if not report.is_file():
        return None
    for line in report.read_text(encoding="utf-8").splitlines():
        m = sg.IDENTITY_ROW_RE.match(line)
        if m and m.group(1).strip() == label:
            return m.group(2).strip(), m.group(3)
    return None


def snapshot(scratch: Path) -> int:
    scratch = Path(scratch)
    for role, (published, _) in ROLES.items():
        if not (ROOT / published).is_file():
            print(f"{TAG}: REFUSED (2) — {published} is not built; the ceremony's build step comes first", file=sys.stderr)
            return 2
    rows, fps = [], {}
    for role, (published, _) in ROLES.items():
        src, dst = ROOT / published, copy_path(scratch, role)
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dst); dst.chmod(dst.stat().st_mode | 0o111)
        d = sha(dst)
        if d != sha(src):
            print(f"{TAG}: REFUSED (2) — the copy of {published} does not hash as its source", file=sys.stderr)
            return 2
        fp = fingerprint(dst) if (WARM and role in PROBES) else ""
        fps[role] = fp
        rows.append((role, published, d, fp))
        print(f"{TAG}: copied {published} -> {dst.relative_to(scratch)} sha256={d[:12]}…" + (f" systemverilog={fp[:16]}" if fp else ""))
    if WARM and fps["release"] and fps["debug"] and fps["release"] != fps["debug"]:
        print(f"{TAG}: REFUSED (2) — the release and debug probes carry different systemverilog fingerprints "
              f"({fps['release'][:16]} vs {fps['debug'][:16]}): one of them is a stale build", file=sys.stderr)
        return 2
    for rel in RECORDED:
        p = ROOT / rel
        rows.append((f"recorded:{rel}", rel, sha(p) if p.is_file() else "(absent)", ""))
    ident = scratch / "bin" / "identity.tsv"
    ident.write_text("role\tpublished_path\tsha256\tsystemverilog_fingerprint\n"
                     + "".join("\t".join(r) + "\n" for r in rows), encoding="utf-8")
    # the short form the crash-recovery procedure reads (generated parser first, then the three binaries)
    short = [(sha((ROOT / RECORDED[0])) if (ROOT / RECORDED[0]).is_file() else "(absent)", RECORDED[0])]
    short += [(r[2], r[1]) for r in rows[:3]]
    (scratch / "sha_final.txt").write_text("".join(f"{d[:17]}  {p}\n" for d, p in short), encoding="utf-8")
    print(f"{TAG}: OK snapshot copies=3 recorded={len(RECORDED)} -> {ident}")
    return 0


def restore(scratch: Path) -> int:
    scratch = Path(scratch); ident = read_identity(scratch)
    restored, unchanged = 0, 0
    for role, (published, _) in ROLES.items():
        cp = copy_path(scratch, role)
        if not cp.is_file() or sha(cp) != ident[role]["sha256"]:
            print(f"{TAG}: REFUSED (2) — the copy {cp} is absent or no longer hashes as identity.tsv records "
                  f"({ident[role]['sha256'][:12]}…); nothing restored", file=sys.stderr)
            return 2
        dst = ROOT / published
        have = sha(dst) if dst.is_file() else ""
        if have == ident[role]["sha256"]:
            unchanged += 1; print(f"{TAG}: unchanged {published} ({have[:12]}…)")
            continue
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(cp, dst); dst.chmod(dst.stat().st_mode | 0o111)
        restored += 1
        print(f"{TAG}: restored {published} ({(have[:12] + '…') if have else 'absent'} -> {ident[role]['sha256'][:12]}…)")
    print(f"{TAG}: OK restore restored={restored} unchanged={unchanged}")
    return 0


def verify(scratch: Path) -> int:
    scratch = Path(scratch); ident = read_identity(scratch)
    breaches, notes = [], []
    # (1) the copies
    for role in ROLES:
        cp = copy_path(scratch, role)
        if not cp.is_file():
            breaches.append(f"(1) the copy {cp} is ABSENT")
        elif sha(cp) != ident[role]["sha256"]:
            breaches.append(f"(1) the copy {cp} hashes {sha(cp)[:12]}…, identity.tsv records {ident[role]['sha256'][:12]}… — the copy MOVED")
    # (2) the published paths — a breach, except for an UPLIFTED role (SV-CORPUS-GRAD.56): cargo re-uplifts the debug
    # ast_pipeline on every build of its feature set — every doctrines run, the pre-commit hook — after the chain's last
    # restore, so that path cannot be held to the ceremony's bytes at commit time; the tracked row that names it is held
    # by arm (3) to the ceremony's copy, and `restore` still puts the bytes back before any measuring step that needs them
    for role, (published, _) in ROLES.items():
        p = ROOT / published
        have = sha(p) if p.is_file() else ""
        if have != ident[role]["sha256"]:
            msg = (f"(2) {published} holds {(have[:12] + '…') if have else 'NOTHING'}, the ceremony measured with "
                   f"{ident[role]['sha256'][:12]}… — run `restore`")
            if role in UPLIFTED:
                notes.append(msg + " (an uplifted path: cargo re-uplifts it on every build of its feature set after the last restore — a note, not a breach; the tracked row naming it is held by arm (3))")
            else:
                breaches.append(msg)
    # (3) the tracked reports
    reports = 0
    for role, (published, report_list) in ROLES.items():
        for rel, label in report_list:
            rp = ROOT / rel
            if not rp.is_file():
                breaches.append(f"(3) the tracked report {rel} is ABSENT"); continue
            row = report_row(rp, label); reports += 1
            if row is None:
                breaches.append(f"(3) {rel} carries no `{label}` identity row"); continue
            path_cell, digest = row
            if path_cell != published:
                breaches.append(f"(3) {rel} names `{path_cell}`, not the {role} probe's published path `{published}`")
            if digest != ident[role]["sha256"]:
                breaches.append(f"(3) {rel} names {digest[:12]}…, the ceremony's {role} probe is {ident[role]['sha256'][:12]}… — "
                                f"the reading was taken with other bytes (or never re-run this ceremony)")
    # (4) the store-gate memo against the lane reports
    memo = subprocess.run([sys.executable, str(REPO / MEMO_MODULE), "--check-identity",
                           "--cache", str(ROOT / CHAR / "store_gate_cache.tsv"), "--characterization-dir", str(ROOT / CHAR)],
                          capture_output=True, text=True)
    memo_line = (memo.stdout.strip().splitlines() or [""])[-1]
    if memo.returncode != 0:
        breaches.append(f"(4) {memo_line or memo.stderr.strip()[-200:]}")
    # (5) the recorded generated parser
    for key, rec in ident.items():
        if not key.startswith("recorded:"):
            continue
        p = ROOT / rec["published"]
        if rec["sha256"] == "(absent)" or not p.is_file():
            notes.append(f"(5) {rec['published']}: NOT EVALUATED ({'not recorded' if rec['sha256'] == '(absent)' else 'absent now'})")
        elif sha(p) != rec["sha256"]:
            breaches.append(f"(5) {rec['published']} hashes {sha(p)[:12]}…, the ceremony recorded {rec['sha256'][:12]}… — regenerated since")
    for n in notes:
        print(f"{TAG}: note {n}")
    if breaches:
        for b in breaches:
            print(f"{TAG}: ✗ {b}", file=sys.stderr)
        print(f"{TAG}: ✗ verify breaches={len(breaches)} (copies / published paths / reports / memo / recorded — see stderr)")
        return 1
    print(f"{TAG}: OK verify copies={len(ROLES)} published={len(ROLES)} reports={reports} memo=OK "
          f"recorded={sum(1 for k in ident if k.startswith('recorded:'))} notes={len(notes)}")
    return 0


def self_test() -> int:
    fails = []

    def check(name, cond):
        print(f"  {'PASS' if cond else 'FAIL'}  {name}")
        if not cond:
            fails.append(name)

    def run(*args, root):
        env = dict(os.environ, PGEN_SV_CEREMONY_ROOT=str(root), PGEN_SV_CEREMONY_NO_WARM="1")
        p = subprocess.run(["bash", str(REPO / "scripts/sv_ceremony_binaries.sh"), *args], capture_output=True, text=True, env=env)
        return p.returncode, p.stdout + p.stderr

    with tempfile.TemporaryDirectory(dir=str(REPO / "rust/target")) as td:
        root = Path(td) / "tree"; scratch = Path(td) / "svNN"
        for role, (published, _) in ROLES.items():
            p = root / published; p.parent.mkdir(parents=True, exist_ok=True); p.write_bytes(f"binary {role}\n".encode())
        (root / RECORDED[0]).parent.mkdir(parents=True, exist_ok=True); (root / RECORDED[0]).write_text("parser\n")
        rc, out = run("snapshot", str(scratch), root=root)
        ident = read_identity(scratch) if rc == 0 else {}
        check("snapshot copies the three binaries, records their digests and the generated parser's, writes sha_final.txt",
              rc == 0 and all(copy_path(scratch, r).is_file() for r in ROLES) and len(ident) == 3 + len(RECORDED)
              and ident["release"]["sha256"] == sha(root / ROLES["release"][0]) and (scratch / "sha_final.txt").is_file()
              and ident[f"recorded:{RECORDED[1]}"]["sha256"] == "(absent)")
        # the tracked reports and the memo, agreeing
        (root / CHAR).mkdir(parents=True)
        for role, (published, reports) in ROLES.items():
            for rel, label in reports:
                (root / rel).parent.mkdir(parents=True, exist_ok=True)
                (root / rel).write_text(f"| {label} | `{published}` | `{ident[role]['sha256']}` |\n")
        memo_rows = ["lane\tsuite\trelpath\tfile_sha256\tbinary_sha256\tinstrument_sha256\tverdict\tnames\tforms",
                     f"sv_2017\ts\tx.sv\t{'1' * 64}\t{ident['release']['sha256']}\t{'2' * 64}\texplained\tT\tclass"]
        (root / CHAR / "store_gate_cache.tsv").write_text("\n".join(memo_rows) + "\n")
        rc, out = run("verify", str(scratch), root=root)
        check(f"verify is GREEN when copies, published paths, every report and the memo agree ({out.strip().splitlines()[-1]})",
              rc == 0 and "OK verify copies=3 published=3 reports=6 memo=OK" in out and "note (5) grammars/systemverilog.ebnf: NOT EVALUATED" in out)
        # a rebuilt published path: RED, then restore, then GREEN
        (root / ROLES["release"][0]).write_bytes(b"rebuilt by a later gate\n")
        rc1, out1 = run("verify", str(scratch), root=root)
        rc2, out2 = run("restore", str(scratch), root=root)
        rc3, out3 = run("verify", str(scratch), root=root)
        check("a published path rebuilt over is a (2) breach, `restore` puts the copy back (restored=1 unchanged=2), verify is GREEN again",
              rc1 == 1 and "(2) rust/target/release/parseability_probe holds" in out1 and rc2 == 0 and "restored=1 unchanged=2" in out2 and rc3 == 0)
        # a role no tracked row names (the debug ast_pipeline) rebuilt over: GREEN with a note, restored on demand
        (root / ROLES["pipeline"][0]).write_bytes(b"rebuilt by a doctrines run\n")
        rc1, out1 = run("verify", str(scratch), root=root)
        rc2, out2 = run("restore", str(scratch), root=root)
        check("a re-uplifted debug ast_pipeline is a NOTE (its tracked row still holds under arm (3)), verify stays GREEN, and `restore` still puts it back (restored=1)",
              rc1 == 0 and "note (2) rust/target/debug/ast_pipeline holds" in out1 and "an uplifted path" in out1 and rc2 == 0 and "restored=1 unchanged=2" in out2)
        # the census row naming other bytes for the uplifted role IS a (3) breach
        cens = root / ROLES["pipeline"][1][0][0]; keep_c = cens.read_text()
        cens.write_text(f"| expansion binary | `{ROLES['pipeline'][0]}` | `{'8' * 64}` |\n")
        rc1, out1 = run("verify", str(scratch), root=root)
        check("the rejects-own-file census naming other bytes for the debug ast_pipeline is a (3) breach", rc1 == 1 and "(3) docs/tasks/artifacts/sv_corpus_grad/rejects_own_file_census/census.md names 888888888888…" in out1)
        cens.write_text(keep_c)
        # a copy that moved: verify RED on (1), restore REFUSES
        cp = copy_path(scratch, "debug"); keep = cp.read_bytes(); cp.write_bytes(b"the copy moved\n")
        rc1, out1 = run("verify", str(scratch), root=root)
        rc2, out2 = run("restore", str(scratch), root=root)
        check("a copy that no longer matches identity.tsv is a (1) breach and `restore` REFUSES (rc 2) rather than publish it",
              rc1 == 1 and "(1) the copy" in out1 and "MOVED" in out1 and rc2 == 2 and "REFUSED" in out2)
        cp.write_bytes(keep)
        # a report naming other bytes / another path
        rel = ROLES["debug"][1][0][0]
        (root / rel).write_text(f"| parse binary | `rust/target/release/parseability_probe` | `{'9' * 64}` |\n")
        rc, out = run("verify", str(scratch), root=root)
        check("a tracked report naming another path and other bytes is named twice under (3)",
              rc == 1 and f"(3) {rel} names `rust/target/release/parseability_probe`, not the debug probe's" in out
              and f"(3) {rel} names 999999999999…" in out)
        (root / rel).write_text(f"| parse binary | `{ROLES['debug'][0]}` | `{ident['debug']['sha256']}` |\n")
        # the memo naming other bytes
        (root / CHAR / "store_gate_cache.tsv").write_text("\n".join(memo_rows[:1] + [memo_rows[1].replace(ident["release"]["sha256"], "f" * 64)]) + "\n")
        rc, out = run("verify", str(scratch), root=root)
        check("a memo row naming other bytes than its lane report is a (4) breach, worded by the memo module",
              rc == 1 and "(4) STORE-GATE-IDENTITY: MISMATCH (sv_2017: rows=1 MISMATCH" in out)
        # a tree with nothing built
        rc, out = run("snapshot", str(Path(td) / "svXX"), root=Path(td) / "empty")
        check("snapshot on a tree with nothing built REFUSES (rc 2) and names the first missing binary", rc == 2 and "is not built" in out)
    print(f"{TAG}-SELF-TEST: pass={9 - len(fails)} fail={len(fails)}")
    return 1 if fails else 0


def main(argv):
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__ or "usage: sv_ceremony_binaries.sh snapshot|restore|verify <scratch-dir> | --self-test"); return 2 if not argv else 0
    if argv[0] == "--self-test":
        return self_test()
    if len(argv) != 2 or argv[0] not in ("snapshot", "restore", "verify"):
        print("usage: scripts/sv_ceremony_binaries.sh snapshot|restore|verify <scratch-dir> | --self-test", file=sys.stderr); return 2
    verb, scratch = argv
    return {"snapshot": snapshot, "restore": restore, "verify": verify}[verb](Path(scratch))


sys.exit(main(sys.argv[1:]))
PYEOF
