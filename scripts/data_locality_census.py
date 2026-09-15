#!/usr/bin/env python3
"""data_locality_census.py — every tracked site that writes scratch data OFF the repository volume.

`DATA-LOCALITY.1`. The director's standing policy (project instruction §13, [[project_north_star]]):
*"All data owned by the project — including generated outputs, build artifacts, caches, logs,
runtime-created test fixtures, and temporary workspaces — must reside on the same filesystem volume
as the repository … tools must derive absolute paths at runtime from the current root and must never
default to /private/tmp, /tmp, user-home caches, or another off-volume location."*

  census (default)   print the LIVE and FROZEN populations; exit 0
  --detail           one line per live site
  --check            exit 1 if the LIVE population is not empty (the gate)
  --self-test        the classifier's own refusal arms

⛔⛔ THIS INSTRUMENT'S CLASSIFIER WAS WRONG TWICE BEFORE IT WAS WRITTEN DOWN, AND BOTH CUTS ARE
PINNED AS SELF-TEST ARMS. A text census is only as good as its ability to tell one spelling from
another, and `mktemp` has four of them:

    mktemp                                   -> OFF-volume (no template, no -p: $TMPDIR)
    mktemp -d                                -> OFF-volume
    mktemp "${TMPDIR:-/tmp}/x.XXXXXX"        -> OFF-volume (an explicit off-volume template)
    mktemp "$ROOT/rust/target/x.XXXXXX"      -> ON-volume  (a repo-rooted template)
    mktemp -p "$ROOT/rust/target"            -> ON-volume

The first cut matched `\\bmktemp\\b(?![^\\n]*-p)` and so counted every repo-rooted TEMPLATE as a
breach — 52 shell "hits" of which most were already correct. The second cut tried to require a
terminator after the flags and matched the opening QUOTE of the template, which reads `mktemp
"$ROOT/…"` as bare. ⇒ the rule is not *"does it mention mktemp"* but *"where does the path it
returns RESOLVE"*, and the decision is made on the ARGUMENT, not on the call.

⭐ TWO POPULATIONS, AND THE SECOND IS NOT A BACKLOG. `docs/tasks/artifacts/**` holds one-shot PROBE
scripts kept as the evidence of a measurement that was already taken. Rewriting them would falsify
the record of what was actually run, so they are COUNTED and CLASSIFIED and never edited — reported
on every run so the exemption is a number rather than a silence.
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FROZEN_PREFIX = "docs/tasks/artifacts/"

# ⛔⛔ THE THIRD CUT, AND THE FIRST ONE THAT IS NOT A SPELLING LIST. Cut 3 decided a template by the
# NAME of the variable it used (`$ROOT` yes, `$scratch_dir` no) — which is the same
# spelling-versus-property error as cuts 1 and 2 one level up: it reported three CORRECT repo-rooted
# sites as breaches purely because their variable was not on an allowlist. The rule is *where does
# the path RESOLVE*, so a variable reference is RESOLVED against its own assignments in the same
# file (up to `RESOLVE_DEPTH` levels) before the verdict is taken.
REPO_ROOT_ANCHOR = re.compile(
    r"git\s+(?:-C[^\n]*?)?rev-parse\s+--show-toplevel"      # the git way
    r"|dirname\s+\"?\$\{BASH_SOURCE"                          # the script's own location
    r"|\bCARGO_MANIFEST_DIR\b"
    r"|(?<![\w/])(?:rust|stimuli|docs|generated|knowledge-map|scripts|grammars)/"  # a repo-relative path
)
OFF_VOLUME_ANCHOR = re.compile(r"TMPDIR|(?<![\w.])/tmp/|/private/tmp/|~/|\$HOME")
RESOLVE_DEPTH = 4

MKTEMP = re.compile(r"\bmktemp\b((?:\s+-[-\w]+)*)\s*(.*)$")
RS_TEMP_DIR = re.compile(r"std::env::temp_dir\s*\(")
RS_TEMPFILE_DEFAULT = re.compile(r"tempfile::(?:tempdir|TempDir::new|NamedTempFile::new)\s*\(\s*\)")
PY_TEMPFILE_NO_DIR = re.compile(
    r"tempfile\.(?:mkdtemp|TemporaryDirectory|NamedTemporaryFile|mkstemp)\s*\(\s*\)"
)
LITERAL_TMP = re.compile(r"(?<![\w/])/(?:private/)?tmp/")


VAR_REF = re.compile(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?")


def resolve(text: str, source: str, depth: int = RESOLVE_DEPTH) -> str:
    """Expand `$VAR` references against assignments in the SAME file, `depth` levels deep.

    Text-level and deliberately shallow: enough to decide whether a template is anchored at the
    repository, which is the only question here. An unresolvable variable expands to nothing, so an
    otherwise-unanchored template stays unanchored and is judged on what remains.
    """
    for _ in range(depth):
        names = VAR_REF.findall(text)
        if not names:
            break
        grew = False
        for name in set(names):
            assign = re.search(rf"^\s*(?:local\s+|readonly\s+|export\s+)?{re.escape(name)}=(.*)$",
                               source, re.M)
            if not assign:
                continue
            value = assign.group(1).strip()
            if VAR_REF.fullmatch(value.strip('"\'')) and value.strip('"\'') == f"${name}":
                continue
            text = re.sub(rf"\$\{{?{re.escape(name)}\}}?", value, text)
            grew = True
        if not grew:
            break
    return text


def shell_site(line: str, source: str = "") -> str | None:
    """`off` / `on` / None for the mktemp call on this line — decided by where its ARGUMENT RESOLVES."""
    match = MKTEMP.search(line)
    if not match:
        return None
    flags, rest = match.group(1), match.group(2).strip()
    if "-p" in flags.split() or "--tmpdir=" in line:
        target = rest
    else:
        target = rest.lstrip('"\'')
        if not target or target[0] in ")|;&":
            return "off"  # no template and no -p: $TMPDIR
    resolved = resolve(target, source)
    if OFF_VOLUME_ANCHOR.search(resolved):
        return "off"
    if REPO_ROOT_ANCHOR.search(resolved):
        return "on"
    if resolved.lstrip('"\'').startswith("/"):
        return "off"  # an absolute template with no repository anchor is not ours
    return "on"  # relative: the enforcers all `cd` to the repository root


def is_comment(path: str, line: str) -> bool:
    stripped = line.lstrip()
    if path.endswith((".sh", ".py")) or "Makefile" in path:
        return stripped.startswith("#")
    if path.endswith(".rs"):
        return stripped.startswith("//") or stripped.startswith("*")
    return False


def scan():
    listing = subprocess.run(["git", "ls-files"], cwd=ROOT, capture_output=True, text=True)
    live, frozen = [], []
    for rel in listing.stdout.split():
        if not (rel.endswith((".rs", ".py", ".sh")) or "Makefile" in rel):
            continue
        if rel == "scripts/data_locality_census.py":
            continue  # the census names every spelling it classifies
        try:
            text = (ROOT / rel).read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        bucket = frozen if rel.startswith(FROZEN_PREFIX) else live
        for number, line in enumerate(text.split("\n"), 1):
            if is_comment(rel, line):
                continue
            kinds = []
            if rel.endswith(".sh") or "Makefile" in rel:
                if shell_site(line, text) == "off":
                    kinds.append("sh_mktemp_off_volume")
            if rel.endswith(".rs"):
                if RS_TEMP_DIR.search(line):
                    kinds.append("rs_env_temp_dir")
                if RS_TEMPFILE_DEFAULT.search(line):
                    kinds.append("rs_tempfile_default")
            if rel.endswith(".py") and PY_TEMPFILE_NO_DIR.search(line):
                kinds.append("py_tempfile_no_dir")
            if LITERAL_TMP.search(line) and "TMPDIR" not in line:
                kinds.append("literal_tmp_path")
            for kind in kinds:
                bucket.append((rel, number, kind, line.strip()[:110]))
    return live, frozen


def headline(live, frozen) -> str:
    def by_kind(rows):
        counts: dict[str, int] = {}
        for _, _, kind, _ in rows:
            counts[kind] = counts.get(kind, 0) + 1
        return " ".join(f"{k}={v}" for k, v in sorted(counts.items())) or "none"

    return (
        f"DATA-LOCALITY: live={len(live)} sites in {len({r for r, *_ in live})} file(s) [{by_kind(live)}]; "
        f"frozen={len(frozen)} sites in {len({r for r, *_ in frozen})} probe artifact(s) under "
        f"{FROZEN_PREFIX}** [{by_kind(frozen)}] — counted, never edited (they are the record of a "
        f"measurement already taken)"
    )


def self_test() -> int:
    fails = []
    cases = [
        ('tmp="$(mktemp)"', "off", "a bare mktemp is $TMPDIR"),
        ('tmp="$(mktemp -d)"', "off", "a bare mktemp -d is $TMPDIR"),
        ('tmp="$(mktemp "${TMPDIR:-/tmp}/x.XXXXXX")"', "off", "an explicit off-volume template"),
        ('tmp="$(mktemp /tmp/x.XXXXXX)"', "off", "a literal /tmp template"),
        # ⛔ the two cuts that were WRONG, pinned so they cannot come back
        ('tmp="$(mktemp "$ROOT/rust/target/x.XXXXXX")"', "on",
         "a repo-rooted TEMPLATE is not a breach — cut 1 counted it as one"),
        ('fake="$(mktemp -d "$ROOT/rust/target/y.XXXXXX")" || return 2', "on",
         "flags plus a repo-rooted template — cut 2 read the opening quote as the end of the call"),
        ('out="$(mktemp "$scratch/digest.XXXXXX.json")"', "on", "a derived repo-rooted scratch dir"),
        ('d="$(mktemp -d -p "$ROOT/rust/target")"', "on", "an explicit -p inside the repo"),
    ]
    # ⛔ cut 3: a template using a DERIVED variable is on-volume, and deciding it by the variable's
    # NAME reported three correct sites as breaches. The verdict follows the ASSIGNMENT.
    derived = (
        'scratch_dir="$ROOT/rust/target/knowledge_map_gate"\n'
        'ids_file="$(mktemp "$scratch_dir/ids.XXXXXX")"\n'
    )
    if shell_site('ids_file="$(mktemp "$scratch_dir/ids.XXXXXX")"', derived) != "on":
        fails.append("a template using a variable DERIVED from the repo root was called a breach "
                     "(cut 3: the verdict was taken from the variable's NAME)")
    derived_off = (
        'scratch_dir="${TMPDIR:-/tmp}/x"\n'
        'ids_file="$(mktemp "$scratch_dir/ids.XXXXXX")"\n'
    )
    if shell_site('ids_file="$(mktemp "$scratch_dir/ids.XXXXXX")"', derived_off) != "off":
        fails.append("a template whose variable resolves OFF-volume was not refused")
    for line, want, why in cases:
        got = shell_site(line, "")
        if got != want:
            fails.append(f"{why}: expected {want}, got {got} — {line}")
    live, frozen = scan()
    if not frozen:
        fails.append("the frozen probe-artifact population reads ZERO, which would mean the scan "
                     "stopped reaching docs/tasks/artifacts/** — a census that gets clean by looking "
                     "less is the failure mode that looks most like success")
    print(f"data-locality self-test: {'PASS' if not fails else 'FAIL'} ({len(fails)} failing, "
          f"{len(cases) + 2} classifier arms + 1 reach arm)")
    for fail in fails:
        print("  - " + fail)
    return 1 if fails else 0


def main(argv) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--detail", action="store_true")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args(argv)
    if args.self_test:
        return self_test()
    live, frozen = scan()
    print(headline(live, frozen))
    if args.detail or (args.check and live):
        for rel, number, kind, line in sorted(live):
            print(f"  {rel}:{number}  [{kind}]  {line}")
    if args.check and live:
        print("data-locality: the LIVE surface must write inside the repository volume. Derive the "
              "path from the repo root at run time (a `$ROOT/rust/target/...` template, `-p` inside "
              "the repo, `tempfile::Builder::new().tempdir_in(...)`, `dir=` for Python).",
              file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
