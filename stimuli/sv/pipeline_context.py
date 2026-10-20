#!/usr/bin/env python3
"""Derive a corpus suite's COMPILATION CONTEXT for the preprocess→parse lane (SVPP-EXPANSION.4).

WHY THIS EXISTS
---------------
The raw corpus lane (`stimuli/run_external_corpus.sh`) parses every file ALONE and cannot judge a
file that `include`s a header it cannot find, so 4 157 UVM + OpenTitan rows carry no verdict at all
(`deferred:chained_only`). The pipeline lane runs the preprocessor first, and a preprocessor needs
an include search path. Nothing in this repository reads a build system's own context (`.core`,
`.f`, `+incdir+` — measured, SVPP-EXPANSION.1), so the context is DERIVED from the corpus itself:

    for every `` `include "NAME" `` in every source file of the suite,
      if `dirname(includer)/NAME` exists      -> LOCAL: the preprocessor already searches the
                                                 including file's own directory first; no dir needed
      else the directories D with `D/NAME`   -> exactly one: D is a SHARED header directory -> emitted
         (D/NAME may be ANY file the tree       -> several, ONE text (every copy byte-identical):
          holds — IEEE 1800-2017 §22.4 puts         IDENTICAL — the choice cannot change the expansion,
          no constraint on the filename, and        so it is resolved PER FILE (a `.4b` row: the nearest
          a corpus writes `.vi` / `.vams`             copy, the first on a tie) and the copy count is
          headers; `.4d`)                           reported — never as a suite directory (SVPP-EXPANSION.4d)
                                              -> several texts: AMBIGUOUS-NONLOCAL, reported, not guessed
                                              -> none:       ABSENT, reported
    and the SHADOW guard (`.4d`): a directory emitted for one name that also holds an AMBIGUOUS name
    would let the preprocessor resolve the ambiguous name silently — a guess by the back door. A SUITE
    directory in that position REFUSES the derivation (exit 4), naming the suite, the name and the
    directory; a PER-FILE directory in that position (the includer itself includes the ambiguous name)
    is WITHHELD for that includer and reported, and the includer keeps its honest refusal.

The emitted directories are ordered by how many includers need them (ties by path), which is the
order the preprocessor searches them in. ⛔ An ambiguous name is reported with every candidate and
is NOT resolved by picking one: the per-file context a build system would supply is `.4`(b)'s
question, and a guessed directory would turn an honest `preprocess_failed` into a silent wrong
expansion.

OUTPUT (deterministic; sorted; no timestamps)
    stimuli/sv/pipeline_context.tsv        suite<TAB>kind<TAB>value<TAB>needed_by<TAB>resolves
                                           kind = include_dir | define  (defines are hand-added rows)
    stimuli/sv/pipeline_context_files.tsv  suite<TAB>relpath<TAB>include_dir<TAB>name — the PER-FILE
                                           rows (SVPP-EXPANSION.4b): an ambiguous name resolved for ONE
                                           including file by the NEAREST candidate (the candidate
                                           directory sharing the longest path prefix with the includer —
                                           what a build system's per-top file list does for the
                                           `top_earlgrey` / `top_darjeeling` variants of one `seq_lib`).
                                           A tie stays ambiguous and is reported, never guessed.
    stdout                                 one PIPELINE-CONTEXT: line per suite + the ambiguous /
                                           absent names

USAGE
    python3 stimuli/sv/pipeline_context.py --write            # re-derive every registered suite
    python3 stimuli/sv/pipeline_context.py --check            # re-derive and diff against the tracked TSV (exit 1 on drift)
    python3 stimuli/sv/pipeline_context.py --self-test        # the synthetic tree; exit 7 on a miss
    python3 stimuli/sv/pipeline_context.py --suite opentitan  # report one suite, write nothing
    python3 stimuli/sv/pipeline_context.py --list-suites      # every registered suite, one per line (the lane's roster)
"""
from __future__ import annotations

import argparse
import collections
import hashlib
import os
import pathlib
import re
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
if not (ROOT / "grammars" / "systemverilog.ebnf").exists():
    sys.exit(f"pipeline_context: derived repo root {ROOT} holds no grammars/systemverilog.ebnf — refusing")

TSV = ROOT / "stimuli" / "sv" / "pipeline_context.tsv"
SOURCE_SUFFIXES = (".sv", ".svh", ".v", ".vh")
INCLUDE_RE = re.compile(r'^[ \t]*`include[ \t]+"([^"]+)"', re.M)

# suite name -> root directory, repo-root-relative. The names are the raw lane's own (the `sub`
# column of results.tsv: the directory under stimuli/sv/subs/, and `uvm-core` for the UVM tree), so
# the two lanes join on them. DERIVED from the tree (SVPP-EXPANSION.4b): every directory under
# stimuli/sv/subs/ is a suite, and a suite added by vendoring joins the lane without an edit here.
def registered_suites() -> dict[str, str]:
    suites = {"uvm-core": "stimuli/sv/uvm"}
    subs = ROOT / "stimuli" / "sv" / "subs"
    if subs.is_dir():
        for d in sorted(subs.iterdir()):
            if d.is_dir():
                suites[d.name] = f"stimuli/sv/subs/{d.name}"
    return suites


SUITES = registered_suites()
# The DESIGN suites: a file there is one member of a compiled design (packages, `import X::*`, `` `include ``d class
# files), so the suite's own convention — the design compiles — is the expectation (SVPP-EXPANSION.6c.1) and the
# library pass (.6d.1) parses their packages in dependency order. ONE definition; corpus_verdict_coverage.py imports it.
DESIGN_SUITES = {"Cores-VeeR-EL2", "friscv", "scr1", "opentitan", "black-parrot", "uvm-core"}
FILES_TSV = ROOT / "stimuli" / "sv" / "pipeline_context_files.tsv"
# SVPP-EXPANSION.4c.1 — per-file DEFINES a suite's own driver passes for the file: verilator's `t_x.py`
# (`+define+NAME[=VALUE]` / `-DNAME[=VALUE]` inside its flag lists) and sv-tests' `:defines:` header.
# Derived, never guessed: only a LITERAL value is emitted (a value the driver computes at run time —
# `"+define+SIM_CYCLES=" + str(test.cycles)`, an f-string — is REPORTED as computed); the SAME-STEM driver
# owns the file, and where only variant drivers (`top_filename = "t/t_x.v"`) target it, a single variant
# is used and several are REPORTED as a tie. ⛔ This is the test's OWN configuration, not a tool-identity
# predefine (`.6b.3` measured and declined those): the driver ran the file with exactly these.
DEFINES_TSV = ROOT / "stimuli" / "sv" / "pipeline_context_defines.tsv"
# SVPP-EXPANSION.4c.2 — per-file UNIT PREDECESSORS: the source files a suite's own harness compiles BEFORE the
# file in the same compilation unit, so their `define`s are in scope (IEEE 1800-2017 §22.5.1: a macro is
# unit-wide once defined). Declared by: verilator's `t_x.py` source-file arguments in its SV flag lists
# (the harness puts them before the top file — driver.py: `*flags, top_filename, *v_other_filenames`);
# slang's `tests/unittests/data/*.f` command files (`-F` expanded, `-v` library files skipped); sv2v's
# `test/*/run.sh` explicit multi-file `convert` / `runAndCapture` lines (sv2v lexes its inputs as one
# unit unless `--siloed`). Derived, never guessed: a `-f` list, a glob, a `--siloed` run and a file two
# declarations order differently are REPORTED, not applied. Applied by the lane as `--sv-unit-file`, in order.
UNITS_TSV = ROOT / "stimuli" / "sv" / "pipeline_context_units.tsv"
FRAGMENTS_TSV = ROOT / "stimuli" / "sv" / "pipeline_context_fragments.tsv"   # SVPP-EXPANSION.4c.6: an include fragment's own-row context
INCLUDE_LINE_RE = re.compile(r'^[ \t]*`include[ \t]+"([^"]+)"')
COND_LINE_RE = re.compile(r'^[ \t]*`(ifdef|ifndef|endif)\b')
PACKAGE_DECL_RE = re.compile(r"^[ \t]*package[ \t]+(?:automatic[ \t]+|static[ \t]+)?[A-Za-z_]", re.M)
MAX_FRAGMENT_CHAIN = 8
# Rows the derivation cannot produce: predefined macros a suite is compiled with. Hand-curated,
# each with its source. SVPP-EXPANSION.6d.4: OpenTitan's dvsim flow compiles every DV build with
# `+define+UVM` (hw/dv/tools/dvsim/common_sim_cfg.hjson:65, under "Standard UVM defines"); three DV
# files guard their `include "uvm_macros.svh"` on it (dv_utils_pkg.sv, dv_macros.svh, sw_logger_if.sv),
# and without the symbol every `uvm_*` / `DV_*` macro below the guard stays unexpanded — dv_utils_pkg.sv
# rejected at :215:24 on a raw `uvm_fatal, and 36 packages importing it rejected behind it.
HAND_DEFINES: dict[str, list[tuple[str, str]]] = {
    "opentitan": [("UVM", "stimuli/sv/subs/opentitan/hw/dv/tools/dvsim/common_sim_cfg.hjson:65 (+define+UVM, dvsim build_opts)")],
}
# Unit PREDECESSORS a suite's flow compiles before every file, hand-curated with their source
# (SVPP-EXPANSION.6d.6). IEEE 1800-2017 §22.5.1: a macro is unit-wide once defined, and OpenTitan's
# dvsim builds one compilation unit that starts with UVM (`{UVM_HOME}/src/uvm_pkg.sv`, which includes
# uvm_macros.svh — hw/dv/tools/dvsim/dsim.hjson:15) and compiles dv_utils_pkg.sv (dv_macros.svh) and the
# RTL (prim_assert.sv) before any DV env or test package — so those packages use `uvm_object_utils,
# `DV_CHECK_*, `gfn, `ASSERT_I … without including anything themselves. Measured: 15 of the 38 DV
# packages still rejecting after .6d.4 stopped at exactly such a raw macro; with the three headers as
# unit predecessors 4 of 5 probed packages parse (the fifth stops at a later construct). Applied by the
# lane as `--unit-file`, in this order, to every row of the suite (an RTL row sees three macro headers
# it already includes behind their own guards — no change there, measured by the lane's movers).
HAND_UNITS: dict[str, list[tuple[str, str]]] = {
    # SVPP-EXPANSION.6d.8: VeeR's flow compiles the generated configuration header FIRST — tools/Makefile:167
    # `defines  = $(BUILD_DIR)/common_defines.vh` (then el2_def.sv and el2_pdef.vh, :168-169), and :210 passes
    # it as the first source argument of the VCS build after `+incdir+${RV_ROOT}/design/include`. No source
    # file `include`s it by name (grep: none), so without it every `RV_*` / `TEC_RV_ICG` use is a raw backtick
    # at the parser (tb_top.sv:996, axi_lsu_dma_bridge.sv:82, beh_lib.sv:770). The two other `defines` entries
    # carry no directive (el2_def.sv's guard is commented out, 1 define; el2_pdef.vh is parameter text, 0) and
    # are listed for fidelity to the flow, in its order.
    "Cores-VeeR-EL2": [
        ("stimuli/sv/subs/Cores-VeeR-EL2/snapshots/default/common_defines.vh",
         "stimuli/sv/subs/Cores-VeeR-EL2/tools/Makefile:167 (defines = $(BUILD_DIR)/common_defines.vh) and :210 (the first source argument of the VCS build; the generated configuration header every VeeR build compiles first)"),
        ("stimuli/sv/subs/Cores-VeeR-EL2/design/include/el2_def.sv",
         "stimuli/sv/subs/Cores-VeeR-EL2/tools/Makefile:168 (defines += ${RV_ROOT}/design/include/el2_def.sv; its include guard is commented out, so it defines nothing)"),
        ("stimuli/sv/subs/Cores-VeeR-EL2/snapshots/default/el2_pdef.vh",
         "stimuli/sv/subs/Cores-VeeR-EL2/tools/Makefile:169 (defines += $(BUILD_DIR)/el2_pdef.vh; parameter text, no directive)"),
    ],
    "opentitan": [
        ("stimuli/sv/uvm/uvm-core-2020.3.1/src/uvm_macros.svh",
         "stimuli/sv/subs/opentitan/hw/dv/tools/dvsim/dsim.hjson:14-15 ({UVM_HOME}/src/uvm_pkg.sv compiled first; uvm_pkg.sv includes uvm_macros.svh)"),
        ("stimuli/sv/subs/opentitan/hw/dv/sv/dv_utils/dv_macros.svh",
         "stimuli/sv/subs/opentitan/hw/dv/sv/dv_utils/dv_utils_pkg.sv:11 (`include \"dv_macros.svh\" in the package every DV env imports, compiled before it)"),
        ("stimuli/sv/subs/opentitan/hw/ip/prim/rtl/prim_assert.sv",
         "stimuli/sv/subs/opentitan/hw/ip/prim/rtl/prim_assert.sv:9 (`ifndef PRIM_ASSERT_SV; included by the RTL the DV build compiles first; `ASSERT_I used by DV env packages)"),
        # SVPP-EXPANSION.6d.7: the CIP macro header — guarded, macro-only (13 defines), included INSIDE cip_base_pkg,
        # the package every CIP env imports and the build compiles before it; `DV_MUBI8_DIST in csrng_env_cfg.sv met a raw backtick.
        ("stimuli/sv/subs/opentitan/hw/dv/sv/cip_lib/cip_macros.svh",
         "stimuli/sv/subs/opentitan/hw/dv/sv/cip_lib/cip_base_pkg.sv:24 (`include \"cip_macros.svh\" inside the package every CIP env imports, compiled before it; cip_lib.core:23 lists it as an include file)"),
    ],
}
# Library trees every suite may include from, beyond its own tree. Measured (SVPP-EXPANSION.4):
# 231 OpenTitan files `include "uvm_macros.svh"`, which lives in the UVM suite — a cross-suite
# dependency the derivation resolves like any other shared header, and reports as such.
SHARED_LIBRARIES = ["stimuli/sv/uvm/uvm-core-2020.3.1/src"]


def source_files(base: pathlib.Path) -> list[pathlib.Path]:
    return sorted(p for p in base.rglob("*") if p.is_file() and p.suffix in SOURCE_SUFFIXES)


def all_files(base: pathlib.Path) -> list[pathlib.Path]:
    """Every regular file of the tree — the INCLUDE CANDIDATE population (`.4d`). An `include names a
    file by its path; the standard constrains neither the suffix nor the content, and the corpora write
    `.vi` (Berkeley HardFloat) and `.vams` (iverilog) headers that the source-suffix scan, which is right
    for the INCLUDER population, called absent while the tree held exactly one copy."""
    return sorted(p for p in base.rglob("*") if p.is_file())


def identical_text(paths: list[pathlib.Path]) -> bool:
    """True when every path holds the same bytes (two or more paths)."""
    digests = set()
    for p in paths:
        try:
            digests.add(hashlib.sha256(p.read_bytes()).hexdigest())
        except OSError:
            return False
    return len(digests) == 1


def derive(base: pathlib.Path):
    """Return (dirs, ambiguous, absent, stats) for the suite rooted at `base`.

    dirs: {dir (relative to ROOT): (needed_by_file_count, resolved_name_set)}
    ambiguous: {name: sorted candidate dirs} — non-local names resolvable in more than one dir
    absent: {name: includer_count}
    """
    files = source_files(base)
    # every file path relative to `base`, so a NAME with a subpath ("macros/x.svh") matches
    # `D/NAME` for every directory D whose subtree holds it; the shared library trees join the
    # candidate set under their own repo-relative prefix. The candidate population is EVERY file
    # of the tree (`.4d`), the includer population the source files.
    rel_paths = {p.relative_to(base).as_posix() for p in all_files(base)}
    lib_paths: dict[str, pathlib.Path] = {}
    for lib in SHARED_LIBRARIES:
        lib_base = ROOT / lib
        if lib_base.exists() and not lib_base.is_relative_to(base):
            for lp in all_files(lib_base):
                lib_paths[lp.relative_to(ROOT).as_posix()] = lp
    dirs: dict[str, set[str]] = collections.defaultdict(set)
    needed_by: collections.Counter = collections.Counter()
    ambiguous: dict[str, list[str]] = {}
    absent: collections.Counter = collections.Counter()
    identical: dict[str, int] = {}              # name -> copy count, every copy one text (`.4d`)
    per_file: list[tuple[str, str, str]] = []   # (includer relpath, include_dir, name) — nearest-candidate rows
    local = 0
    for p in files:
        try:
            text = p.read_text(errors="replace")
        except OSError:
            continue
        for name in INCLUDE_RE.findall(text):
            if (p.parent / name).exists():
                local += 1
                continue
            name_posix = pathlib.PurePosixPath(name).as_posix()
            candidates = sorted(
                {
                    (base.relative_to(ROOT) / rp[: -len(name_posix)]).as_posix().rstrip("/")
                    for rp in rel_paths
                    if rp == name_posix or rp.endswith("/" + name_posix)
                }
                | {
                    lp[: -len(name_posix)].rstrip("/")
                    for lp in lib_paths
                    if lp == name_posix or lp.endswith("/" + name_posix)
                }
            )
            if len(candidates) > 1 and identical_text([ROOT / c / name_posix for c in candidates]):
                # `.4d`: several copies, ONE text — whichever directory is searched, the expansion is
                # the same, so there is nothing to guess. Resolved PER FILE (a `.4b` row for this
                # includer: the nearest copy, the first on a tie), never as a suite directory: the
                # first derivation emitted the chosen directory suite-wide and the shadow guard refused
                # it on the real tree — OpenTitan's darjeeling seq_lib, chosen for pwrmgr_vseq_list.sv
                # (4 copies, one text), also holds pwrmgr_base_vseq.sv, which is ambiguous.
                identical[name] = len(candidates)
                includer = p.relative_to(ROOT).as_posix()
                chosen = nearest_candidate(includer, candidates) or candidates[0]
                per_file.append((includer, chosen, name))
                ambiguous.setdefault("__resolved_per_file__", []).append(name)
                continue
            if not candidates:
                absent[name] += 1
            elif len(candidates) == 1:
                dirs[candidates[0]].add(name)
                needed_by[candidates[0]] += 1
            else:
                # SVPP-EXPANSION.4b: the NEAREST candidate for THIS includer — the one sharing the
                # longest path prefix with it — is what a per-top build would hand it; a tie is
                # still ambiguous for this file and stays reported.
                includer = p.relative_to(ROOT).as_posix()
                nearest = nearest_candidate(includer, candidates)
                if nearest is not None:
                    per_file.append((includer, nearest, name))
                    dirs.setdefault("__per_file__", set())
                else:
                    ambiguous[name] = candidates
                if name not in ambiguous:
                    ambiguous.setdefault("__resolved_per_file__", []).append(name)
    dirs.pop("__per_file__", None)
    resolved_per_file = sorted(set(ambiguous.pop("__resolved_per_file__", [])))
    # `.4d` SHADOW guard: an emitted directory (a suite one, or a per-file one for THAT includer) that
    # holds an ambiguous name would resolve it silently — reported here, refused by the caller
    shadowed: list[tuple[str, str]] = []       # (ambiguous name, emitted dir holding it)
    for name, cands in ambiguous.items():
        name_posix = pathlib.PurePosixPath(name).as_posix()
        for d in sorted(dirs):
            if d in cands or (ROOT / d / name_posix).is_file():
                shadowed.append((name, d))
    # the per-file form of the same hazard is WITHHELD, not refused: a per-file directory handed to
    # includer I for name A that also holds an ambiguous name B that I includes would resolve B for I
    # silently — so none of I's rows for that directory is emitted and I keeps its honest refusal
    # (measured on the first real run: OpenTitan's ip_templates pwrmgr vseq list includes vseqs that
    # are one text across the tops AND pwrmgr_base_vseq.sv, which differs per top)
    per_file_includes: dict[str, set[str]] = collections.defaultdict(set)
    for includer, d, _ in per_file:
        per_file_includes[includer].add(d)
    withheld: dict[tuple[str, str], list[str]] = {}   # (includer, dir) -> the ambiguous names the dir holds
    for includer, pf_dirs in per_file_includes.items():
        try:
            names = INCLUDE_RE.findall((ROOT / includer).read_text(errors="replace"))
        except OSError:
            continue
        for name in names:
            if name in ambiguous:
                for d in sorted(pf_dirs):
                    if (ROOT / d / pathlib.PurePosixPath(name).as_posix()).is_file():
                        withheld.setdefault((includer, d), []).append(name)
    if withheld:
        per_file = [(i, d, n) for i, d, n in per_file if (i, d) not in withheld]
    stats = {"files": len(files), "local": local, "per_file_rows": len(per_file),
             "resolved_per_file_names": resolved_per_file, "identical": identical, "shadowed": shadowed,
             "withheld": withheld}
    return {d: (needed_by[d], dirs[d]) for d in dirs}, ambiguous, absent, stats, per_file


def common_prefix_len(a: str, b: str) -> int:
    """Path components the two repo-relative paths share from the root."""
    n = 0
    for x, y in zip(a.split("/"), b.split("/")):
        if x != y:
            break
        n += 1
    return n


def nearest_candidate(includer: str, candidates: list[str]) -> str | None:
    """The single candidate directory sharing the longest path prefix with the includer, or None on a tie."""
    scored = sorted(((common_prefix_len(includer, c), c) for c in candidates), key=lambda t: (-t[0], t[1]))
    if len(scored) >= 2 and scored[0][0] == scored[1][0]:
        return None
    return scored[0][1]


DEFINE_TOKEN = re.compile(r"""(?:\+define\+|(?<![A-Za-z0-9_])-D)([A-Za-z_][A-Za-z0-9_]*)(=?)([^"'\s\]\),]*)""")
COMPUTED_TAIL = re.compile(r"""^["']\s*\+|\{[^}]*\}""")
# spans whose -D flags reach the C++ compiler or iverilog, never the SV source under verilator:
# make_flags=[…] (CPPFLAGS_ADD=…), CPPFLAGS_*=…, -CFLAGS/--CFLAGS/-LDFLAGS + one quoted argument, iv_flags*=[…]
CXX_SPANS = re.compile(
    r"""make_flags\s*=\s*\[.*?\]|iv_flags2?\s*=\s*\[.*?\]|CPPFLAGS\w*=[^\]\n]*|--?(?:CFLAGS|LDFLAGS)['"]?\s*,?\s*['"][^,\]\n]*""",
    re.S,
)


UNDEF_TOKEN = re.compile(r"""(?<![A-Za-z0-9_])-U([A-Za-z_][A-Za-z0-9_]*)""")


def driver_defines(py_text: str) -> tuple[list[str], list[str]]:
    """(literal defines as `NAME` or `NAME=VALUE`, computed define names) found in a verilator driver.
    A name a later `-UNAME` removes is not a define the driver passes (`.4c.2`: `-DCMD_UNDEF -UCMD_UNDEF`)."""
    literal, computed = [], []
    py_text = CXX_SPANS.sub(" ", py_text)
    for m in DEFINE_TOKEN.finditer(py_text):
        name, eq, value = m.group(1), m.group(2), m.group(3)
        tail = py_text[m.end():m.end() + 12]
        if (eq and value == "" and COMPUTED_TAIL.match(tail)) or "{" in value:
            computed.append(name)          # the value is built at run time — not derivable here
            continue
        if any(u.group(1) == name and u.start() > m.start() for u in UNDEF_TOKEN.finditer(py_text)):
            continue                       # undefined again on the same command line
        literal.append(f"{name}={value}" if eq and value else name)
    return sorted(set(literal)), sorted(set(computed))


SRC_ARG = re.compile(r"""["']((?:t/|\$TEST_ROOT/)[A-Za-z0-9_/.\-]+\.(?:v|sv|vh|svh))["']""")
FLAG_LIST = re.compile(r"(?:v_flags2?|verilator_flags2?)\s*=\s*\[(.*?)\]", re.S)
F_LIST = re.compile(r"""(?<![A-Za-z0-9_])-[fF]\s+[^\s"']+""")
SRC_TOKEN = re.compile(r"^[A-Za-z0-9_./\-]+\.(?:sv|svh|v|vh)$")


QUOTED_ITEM = re.compile(r"""["']([^"']*)["']""")
VC_BLOCK_COMMENT = re.compile(r"/\*.*?\*/", re.S)
VC_LINE_COMMENT = re.compile(r"(?m)(?:^|(?<=\s))(?://|#).*$")   # a comment starts a token: `t//x.v` is a doubled slash, not a comment (t_flag_f.vc)
IDENT = re.compile(r"^[A-Za-z_]\w*$")


def subst_env(tok: str, test_root: str) -> str:
    """`$TEST_ROOT` / `${TEST_ROOT}` → the test's own directory; `$TEST_REGRESS` / `${TEST_REGRESS}` → the run
    directory (every path here is test_regress-relative); a doubled slash collapsed, a leading `./` dropped — the
    reading the key's `VerilatorIndex._owner_map` gives the same tokens (SV-CORPUS-GRAD.58)."""
    tok = tok.replace("${TEST_REGRESS}/", "").replace("$TEST_REGRESS/", "")
    tok = tok.replace("${TEST_ROOT}/", test_root + "/").replace("$TEST_ROOT/", test_root + "/")
    while "//" in tok:
        tok = tok.replace("//", "/")
    return tok[2:] if tok.startswith("./") else tok


def vc_defines(tok: str) -> tuple[list[str], list[str]]:
    """`+define+A+B=V+C` → (`["A", "B=V", "C"]`, withheld): Verilator's plus-arg form, every `+` starting a new
    name. ⛔ A value carrying a quote or a backslash (t_flag_define.vc's STRING1 … LIT3) depends on the option
    file's quoting rules, which this reader does not derive — the name is WITHHELD and reported, never emitted guessed."""
    out, withheld = [], []
    for part in tok[len("+define+"):].split("+"):
        name, eq, value = part.partition("=")
        if not IDENT.match(name):
            continue
        if any(c in value for c in "\"'\\"):
            withheld.append(name)
            continue
        out.append(f"{name}={value}" if eq else name)
    return out, withheld


def read_option_file(rel: str, regress: pathlib.Path, test_root: str, seen: frozenset, base_dir: str = ""):
    """Verilator's option file, named by `-f` (paths relative to the run directory) or `-F` (relative to the file's
    own directory) — SVPP-EXPANSION.4c.5. Whitespace-separated options; `//`, `/* */` and leading-`#` comments (the
    suite's own `.vc` files carry all three and verilator reads them); nested `-f` / `-F`; `-v X` a LIBRARY file
    (parsed on demand, after every listed source — Verilator's library semantics, the reading the slang arm gives
    `-v`); `+define+A+B=V` defines; environment names substituted. Returns (members [(rel, 'source' | 'library')] in
    command order, defines [NAME or NAME=VALUE], reported [notes]). ⛔ Derived, never guessed: a file absent from the
    tree, an unterminated comment, an environment name the harness sets at run time, an option file naming itself —
    each is REPORTED and contributes nothing."""
    p = regress / rel
    if not p.is_file():
        return [], [], [f"-f {rel} (an option file absent from the tree; not read)"]
    text = p.read_text(errors="replace")
    if text.count("/*") != text.count("*/"):
        return [], [], [f"-f {rel} (an unterminated comment in the option file; not read)"]
    text = VC_LINE_COMMENT.sub("", VC_BLOCK_COMMENT.sub(" ", text))
    members, defines, reported = [], [], []

    def resolve(raw: str) -> str:
        arg = subst_env(raw, test_root)
        if raw.startswith(("$", "/")) or not base_dir:
            return arg
        return pathlib.PurePosixPath(os.path.normpath(base_dir + "/" + arg)).as_posix()

    toks = text.split()
    i = 0
    while i < len(toks):
        t = toks[i]
        if t in ("-f", "-F") and i + 1 < len(toks):
            raw = toks[i + 1]; sub_rel = resolve(raw)
            if "$" in sub_rel:
                reported.append(f"{t} {raw} (an environment name the harness sets at run time; not read)")
            elif sub_rel in seen:
                reported.append(f"{t} {sub_rel} (an option file naming itself; not read twice)")
            else:
                m2, d2, r2 = read_option_file(sub_rel, regress, test_root, seen | {sub_rel},
                                              "" if t == "-f" else pathlib.PurePosixPath(sub_rel).parent.as_posix())
                members += m2; defines += d2; reported += r2
            i += 2
            continue
        if t == "-v" and i + 1 < len(toks):
            raw = toks[i + 1]; arg = resolve(raw)
            if "$" in arg:
                reported.append(f"-v {raw} (an environment name the harness sets at run time; not read)")
            elif SRC_TOKEN.match(arg):
                members.append((arg, "library"))
            i += 2
            continue
        if t.startswith("+define+"):
            d2, withheld = vc_defines(t)
            defines += d2
            for name in withheld:
                reported.append(f"+define+{name}=… in {rel} (a quoted value; the option file's quoting rules are not derived — withheld)")
            i += 1
            continue
        if t.startswith(("-", "+")):
            i += 1
            continue
        arg = resolve(t)
        if "$" in arg:
            reported.append(f"{t} (an environment name the harness sets at run time; not read)")
        elif SRC_TOKEN.match(arg):
            members.append((arg, "source"))
        i += 1
    return members, defines, reported


def driver_unit_members(py_text: str, top: str, test_root: str, regress):
    """The files a verilator driver's SV flag lists compile besides the top, in command order — `t/…` relative to
    test_regress: (members [(rel, 'source' | 'library')], defines [from `-f` option files], reported). The quoted
    items of each list are tokenized in order, so `-f t/x.vc -DFOO` and `"t/a.v"` read alike; `-f` / `-F` option
    files are READ (`read_option_file`, SVPP-EXPANSION.4c.5; `regress` None = not here, reported), `$TEST_ROOT` is
    the test's own directory, a `-v` file is a library member."""
    py_text = CXX_SPANS.sub(" ", py_text)
    members, defines, reported = [], [], []
    for m in FLAG_LIST.finditer(py_text):
        for item in QUOTED_ITEM.findall(m.group(1)):
            toks = item.split()
            i = 0
            while i < len(toks):
                t = toks[i]
                if t in ("-f", "-F") and i + 1 < len(toks):
                    raw = toks[i + 1]; arg = subst_env(raw, test_root)
                    if regress is None or "$" in arg:
                        reported.append(f"{t} {raw} (an option file; not read here)")
                    else:
                        m2, d2, r2 = read_option_file(arg, regress, test_root, frozenset({arg}),
                                                      "" if t == "-f" else pathlib.PurePosixPath(arg).parent.as_posix())
                        members += m2; defines += d2; reported += r2
                    i += 2
                    continue
                if t == "-v" and i + 1 < len(toks):
                    arg = subst_env(toks[i + 1], test_root)
                    if "$" not in arg and SRC_TOKEN.match(arg):
                        members.append((arg, "library"))
                    i += 2
                    continue
                if t.startswith(("-", "+")):
                    i += 1
                    continue
                arg = subst_env(t, test_root)
                if "$" not in arg and SRC_TOKEN.match(arg) and arg.split("/")[-1] != top and (arg, "source") not in members:
                    members.append((arg, "source"))
                i += 1
    return members, defines, reported


def driver_unit_files(py_text: str, top: str, test_root: str, regress=None) -> tuple[list[str], list[str]]:
    """(the top's predecessor SOURCE files in command order, reported items) — the `.4c.2` reading, over
    `driver_unit_members`; a `-v` library file is not a predecessor of the top (parsed on demand, after it)."""
    members, _defs, reported = driver_unit_members(py_text, top, test_root, regress)
    return [m for m, kind in members if kind == "source"], reported


def units_from_lists(units: list[tuple[str, list[str]]]):
    """units = [(source, [files in order])] → ({file: [predecessors in order]}, {file: conflict note}).
    A file's predecessors are the UNION, in declaration order, of the files listed before it in every
    unit that names it; a pair two declarations order differently is a conflict — reported, not applied."""
    preds: dict[str, list[tuple[str, str]]] = {}
    conflicts: dict[str, str] = {}
    for source, files in units:
        for i, f in enumerate(files):
            lst = preds.setdefault(f, [])
            for p in files[:i]:
                if p not in [x for x, _ in lst]:
                    lst.append((p, source))
    # a PAIR two declarations order differently is dropped from both sides and reported; an
    # unambiguous predecessor of the same file is kept (it is derived, not guessed)
    before = {f: [x for x, _ in lst] for f, lst in preds.items()}   # a snapshot: both sides are judged on it
    for f, lst in preds.items():
        for p, src in list(lst):
            if f in before.get(p, []):
                conflicts[f] = f"{p} precedes it in {src} and follows it in another declaration; the pair is not applied"
                lst.remove((p, src))
    return {f: lst for f, lst in preds.items() if lst}, conflicts


def derive_units(suite: str, base: pathlib.Path):
    """Per-file unit-predecessor rows. Returns (rows [(suite, relpath, order, unit_file, source)],
    reported {relpath: [notes]}, ties {relpath: [drivers]})."""
    rows, reported, ties = [], {}, {}
    if suite == "verilator":
        tdir = base / "test_regress" / "t"
        if not tdir.is_dir():
            return rows, reported, ties
        regress = base / "test_regress"
        by_target: dict[str, list[pathlib.Path]] = {}
        for py in sorted(tdir.glob("t_*.py")):
            text = py.read_text(errors="replace")
            m = re.search(r'top_filename\s*=\s*"t/([^"]+)"', text)
            target = m.group(1) if m else py.stem + ".v"
            if (tdir / target).is_file():
                by_target.setdefault(target, []).append(py)
        vunits: list[tuple[str, list[str]]] = []          # (driver, [members in command order, the top last])
        vlibs: list[tuple[str, str, list[str]]] = []      # (driver, -v library file, every listed file before it)
        for target, drivers in sorted(by_target.items()):
            same = [d for d in drivers if d.stem + ".v" == target]
            chosen = same[0] if same else (drivers[0] if len(drivers) == 1 else None)
            rel = (tdir / target).relative_to(ROOT).as_posix()
            if chosen is None:
                with_units = [d.name for d in drivers if driver_unit_files(d.read_text(errors="replace"), target, "t/" + d.stem, regress)[0]]
                if with_units:
                    ties[rel] = with_units
                continue
            test_root = "t/" + chosen.stem if (tdir / chosen.stem).is_dir() else "t"
            members, _vcdefs, notes = driver_unit_members(chosen.read_text(errors="replace"), target, test_root, regress)
            src = chosen.relative_to(ROOT).as_posix()
            sources = [m for m, kind in members if kind == "source"]
            present = []
            for pth in sources + ["t/" + target]:          # the top LAST: driver.py:1099 puts *flags before top_filename
                f = regress / pth
                if not f.is_file():
                    notes.append(f"{pth} (named by the driver, absent from the tree)")
                    continue
                present.append(f.relative_to(ROOT).as_posix())
            vunits.append((src, present))
            for lib in [m for m, kind in members if kind == "library" and m not in sources]:
                f = regress / lib
                if not f.is_file():
                    notes.append(f"-v {lib} (a library file named by the driver, absent from the tree)")
                    continue
                vlibs.append((src, f.relative_to(ROOT).as_posix(), list(present)))
            if notes:
                reported[rel] = notes
        # SVPP-EXPANSION.4c.5: every listed member — not only the top — reads after the files its own command line
        # compiles before it (one compilation unit; the same `units_from_lists` reading the slang arm has, a pair two
        # drivers order differently reported, not applied)
        preds, conflicts = units_from_lists(vunits)
        for f, lst in sorted(preds.items()):
            for order, (p, src) in enumerate(lst, 1):
                rows.append((suite, f, order, p, src))
        for f, note in conflicts.items():
            reported.setdefault(f, []).append(note)
        # a `-v` LIBRARY file is parsed on demand, once every listed file (the top among them) has been: its
        # predecessors are all of them, and it precedes nothing
        listed = {r[1] for r in rows}
        for src, lib, allp in vlibs:
            if lib in listed:
                reported.setdefault(lib, []).append(f"-v {lib} is also a listed source of a unit; the library reading is not applied")
                continue
            listed.add(lib)
            for order, p in enumerate(allp, 1):
                rows.append((suite, lib, order, p, src))
    elif suite == "slang":
        data = base / "tests" / "unittests" / "data"
        if not data.is_dir():
            return rows, reported, ties

        def expand(fpath: pathlib.Path, seen: frozenset) -> list[str]:
            files: list[str] = []
            toks = fpath.read_text(errors="replace").split()
            i = 0
            while i < len(toks):
                t = toks[i]
                if t in ("-F", "-f", "-C") and i + 1 < len(toks):
                    sub = data / toks[i + 1]
                    if sub.is_file() and sub not in seen:
                        files += expand(sub, seen | {sub})
                    i += 2
                    continue
                if t == "-v":            # a library file: parsed on demand, not a predecessor
                    i += 2
                    continue
                if t.startswith("-"):
                    i += 1
                    continue
                if SRC_TOKEN.match(t) and (data / t).is_file():
                    files.append((data / t).relative_to(ROOT).as_posix())
                i += 1
            return files

        units = [(f.relative_to(ROOT).as_posix(), expand(f, frozenset({f}))) for f in sorted(data.glob("*.f"))]
        preds, conflicts = units_from_lists([u for u in units if len(u[1]) >= 2])
        for f, lst in sorted(preds.items()):
            for order, (p, src) in enumerate(lst, 1):
                rows.append((suite, f, order, p, src))
        for f, note in conflicts.items():
            reported[f] = [note]
    elif suite == "sv2v":
        units = []
        for run in sorted((base / "test").glob("*/run.sh")):
            d = run.parent
            for ln, line in enumerate(run.read_text(errors="replace").splitlines(), 1):
                m = re.match(r"\s*(?:convert\s+\"[^\"]*\"\s+\S+|runAndCapture)\s+(.*)$", line)
                if not m:
                    continue
                toks = m.group(1).split()
                src = f"{run.relative_to(ROOT).as_posix()}:{ln}"
                if "--siloed" in toks:
                    reported.setdefault(src, []).append("--siloed: macros do not carry across the files; not a unit")
                    continue
                if any("*" in t for t in toks):
                    reported.setdefault(src, []).append("a glob names the unit; not applied")
                    continue
                files = [(d / t).relative_to(ROOT).as_posix() for t in toks if SRC_TOKEN.match(t) and (d / t).is_file()]
                if len(files) >= 2:
                    units.append((src, files))
        preds, conflicts = units_from_lists(units)
        for f, lst in sorted(preds.items()):
            for order, (p, src) in enumerate(lst, 1):
                rows.append((suite, f, order, p, src))
        for f, note in conflicts.items():
            reported.setdefault(f, []).append(note)
    rows.sort(key=lambda r: (r[0], r[1], r[2]))
    return rows, reported, ties


# SVPP-EXPANSION.4c.3 — opentitan: the IP's OWN dvsim configuration. A `*_sim_cfg.hjson` declares
# `build_modes: [{name, build_opts: ["+define+NAME[=VALUE]", …]}]`, enables some for every test
# (`en_build_modes`) or lets a test pick one (`build_mode:`), carries its own top-level `build_opts`,
# and imports a base cfg (`import_cfgs: ["{proj_root}/…"]`) whose modes it may enable. Each cfg and
# each enabled / picked mode is one CONFIGURATION the IP's harness compiles its dv tree under; the
# nearest cfg directory owns a file. Several configurations of one directory are VARIANTS: the lane
# applies the FIRST by name and reports the others (every value is a literal the cfg itself wrote;
# a templated value such as `{tl_aw}` is computed and reported, never emitted). The project-wide
# `hw/dv/tools/dvsim/common_sim_cfg.hjson` is not an IP's directory and is left to the suite rows.
SIM_CFG_GLOB = "*_sim_cfg.hjson"
HJSON_DEFINE = re.compile(r'"\+define\+([A-Za-z_]\w*)(?:=([^"\s]*))?"')
HJSON_QUOTED = re.compile(r'"([^"\n]*)"')


def _hjson_bracket(text: str, key: str, start: int = 0):
    """(inner text, end) of the first `key: [ … ]` at or after `start`, bracket-balanced; None if absent."""
    m = re.compile(r"(?<![\w])" + re.escape(key) + r"\s*:\s*\[").search(text, start)
    if not m:
        return None
    depth, i = 1, m.end()
    while i < len(text) and depth:
        depth += {"[": 1, "]": -1}.get(text[i], 0)
        i += 1
    return text[m.end():i - 1], i


def _hjson_modes(text: str) -> dict:
    """{mode name: [define tokens]} from a cfg's `build_modes` block."""
    blk = _hjson_bracket(text, "build_modes")
    if not blk:
        return {}
    modes = {}
    # a brace inside a quoted value (`"+define+W={tl_aw}"`) is text, not an entry boundary
    safe = re.sub(r'"[^"\n]*"', lambda m: m.group(0).replace("{", "\x01").replace("}", "\x02"), blk[0])
    for entry in re.split(r"\}\s*\{|\{|\}", safe):
        entry = entry.replace("\x01", "{").replace("\x02", "}")
        nm = re.search(r"(?<![\w])name\s*:\s*([^\s,]+)", entry)
        opts = _hjson_bracket(entry, "build_opts")
        if nm and opts:
            modes[nm.group(1)] = HJSON_DEFINE.findall(opts[0])
    return modes


def _hjson_names(text: str, key: str) -> list[str]:
    blk = _hjson_bracket(text, key)
    return HJSON_QUOTED.findall(blk[0]) if blk else []


def opentitan_sim_cfg_defines(base: pathlib.Path):
    """(rows [(suite, relpath, define, source)], computed {relpath: [names]},
    variants {dir: {name: {build: value}}}) — SVPP-EXPANSION.4c.3, the IP's own configurations, read from its
    `dv/*_sim_cfg.hjson`. A cfg nobody imports is a RUN cfg; its default build carries the top-level `build_opts`
    of its import chain plus the chain's `en_build_modes`, and every `build_mode:` a test of the chain picks is one
    more build on top of that; a cfg another cfg imports is a base, run only through its importers. A define is
    applied to the files the cfg directory owns only when EVERY build of the directory carries it with ONE value;
    a name the builds disagree on (another value, or absent from one build) is REPORTED per build, never applied —
    `.6d.1`'s rule, the `.6d.5` parking: a disagreement is reported, not resolved by guessing which build the row
    meant. A templated value (`{tl_aw}`) is computed: reported, never emitted."""
    rows, computed, variants = [], {}, {}
    cfgs = [c for c in sorted(base.rglob(SIM_CFG_GLOB)) if "hw/dv/tools/dvsim" not in c.as_posix()]
    texts = {c: c.read_text(errors="replace") for c in cfgs}

    def imported(c: pathlib.Path):
        out = []
        for ref in _hjson_names(texts[c], "import_cfgs"):
            p = base / ref.replace("{proj_root}/", "")
            if p in texts:
                out.append(p)
        return out

    importers: dict[pathlib.Path, list] = {}
    for c in cfgs:
        for b in imported(c):
            importers.setdefault(b, []).append(c)

    def chain(c: pathlib.Path, seen=()):
        """the cfg and every cfg it imports, bases first"""
        out: list = []
        for b in imported(c):
            if b not in seen and b not in out:
                out += [x for x in chain(b, seen + (c,)) if x not in out]
        out.append(c)
        return out

    def top_defs_of(c: pathlib.Path):
        text = texts[c]
        own_top = _hjson_bracket(text, "build_opts")
        # the FIRST build_opts at the cfg's own level: inside a build_modes block it belongs to a mode
        modes_blk = _hjson_bracket(text, "build_modes")
        if own_top and not (modes_blk and modes_blk[1] - len(modes_blk[0]) - 1 < own_top[1] <= modes_blk[1]):
            return HJSON_DEFINE.findall(own_top[0])
        return []

    by_dir: dict[pathlib.Path, list] = {}
    for c in cfgs:
        if c in importers:
            continue  # a base: run only through the cfgs importing it
        members = chain(c)
        modes: dict = {}
        for m in members:
            modes.update(_hjson_modes(texts[m]))
        top = [d for m in members for d in top_defs_of(m)]
        enabled: list[str] = []
        for m in members:
            enabled += [n for n in _hjson_names(texts[m], "en_build_modes") if "{" not in n and n in modes and n not in enabled]
        picked = sorted({n for m in members for n in re.findall(r"(?<![\w])build_mode\s*:\s*([^\s,}]+)", texts[m])
                         if "{" not in n and n in modes and n not in enabled})
        default = top + [d for n in enabled for d in modes[n]]
        stem = c.stem + (f"+{'+'.join(enabled)}" if enabled else "")
        confs = [(stem, default)] + [(f"{stem}+{n}", default + modes[n]) for n in picked]
        if any(defs for _, defs in confs):
            by_dir.setdefault(c.parent, []).extend((name, c, defs) for name, defs in confs)
    dirs = sorted(by_dir, key=lambda d: -len(d.as_posix()))     # the nearest cfg directory owns a file
    owned: dict[pathlib.Path, pathlib.Path] = {}
    for d in dirs:
        for f in d.rglob("*"):
            if f.suffix in (".sv", ".svh") and f not in owned:
                owned[f] = d
    for d, confs in by_dir.items():
        confs = sorted(confs, key=lambda t: t[0])
        per_build: dict[str, dict] = {}
        for name, _cfg, defs in confs:
            per_build[name] = dict(defs)  # a later +define+ of the same name wins, as on a command line
        agreed, comp, disagreed = [], [], {}
        for k in sorted({k for m in per_build.values() for k in m}):
            vals = {b: ("(absent)" if k not in m else ("(defined)" if not m[k] else m[k])) for b, m in per_build.items()}
            if len(set(vals.values())) == 1:
                v = next(iter(vals.values()))
                if "{" in v:
                    comp.append(k)
                else:
                    agreed.append((k, None if v == "(defined)" else v))
            else:
                disagreed[k] = vals
        rel_dir = d.relative_to(ROOT).as_posix()
        if disagreed:
            variants[rel_dir] = disagreed
        if not agreed and not comp:
            continue
        n = len(per_build)
        src = f"{confs[0][1].relative_to(ROOT).as_posix()} ({n} build{'s' if n > 1 else ''} agree: {' '.join(per_build)}; dvsim build_opts)"
        for f, od in owned.items():
            if od != d:
                continue
            rel = f.relative_to(ROOT).as_posix()
            for k, v in agreed:
                rows.append(("opentitan", rel, f"{k}={v}" if v else k, src))
            if comp:
                computed[rel] = comp
    return sorted(set(rows)), computed, variants


def derive_defines(suite: str, base: pathlib.Path):
    """Per-file define rows for the suites whose drivers carry them; plus what was REPORTED, not emitted.
    Returns (rows [(suite, relpath, define, source)], computed {relpath: [names]}, ties {relpath: [drivers]},
    variants {dir: {name: {build: value}}} — SVPP-EXPANSION.4c.3, the IP's own builds disagree on the name)."""
    rows, computed, ties, variants = [], {}, {}, {}
    if suite == "opentitan":
        rows, computed, variants = opentitan_sim_cfg_defines(base)
        return rows, computed, ties, variants
    if suite == "verilator":
        tdir = base / "test_regress" / "t"
        if not tdir.is_dir():
            return rows, computed, ties, variants
        by_target: dict[str, list[pathlib.Path]] = {}
        for py in sorted(tdir.glob("t_*.py")):  # a driver; `*_common.py` helpers are not drivers
            text = py.read_text(errors="replace")
            m = re.search(r'top_filename\s*=\s*"t/([^"]+)"', text)
            target = m.group(1) if m else py.stem + ".v"
            if not (tdir / target).is_file():
                continue
            by_target.setdefault(target, []).append(py)
        for target, drivers in sorted(by_target.items()):
            same = [d for d in drivers if d.stem + ".v" == target]
            chosen = same[0] if same else (drivers[0] if len(drivers) == 1 else None)
            rel = (tdir / target).relative_to(ROOT).as_posix()
            if chosen is None:
                with_defs = [d.name for d in drivers if any(driver_defines(d.read_text(errors="replace")))]
                if with_defs:
                    ties[rel] = with_defs
                continue
            text = chosen.read_text(errors="replace")
            lit, comp = driver_defines(text)
            for d in lit:
                rows.append((suite, rel, d, chosen.relative_to(ROOT).as_posix()))
            # SVPP-EXPANSION.4c.5: the `+define+` tokens of the driver's -f option files, a name the command line -U's again excluded
            _members, vcdefs, _notes = driver_unit_members(text, target, "t/" + chosen.stem if (tdir / chosen.stem).is_dir() else "t", base / "test_regress")
            undefs = {u.group(1) for u in UNDEF_TOKEN.finditer(CXX_SPANS.sub(" ", text))}
            for d in vcdefs:
                if d.split("=", 1)[0] not in undefs:
                    rows.append((suite, rel, d, chosen.relative_to(ROOT).as_posix() + " (-f option file)"))
            if comp:
                computed[rel] = comp
    elif suite == "sv-tests":
        for f in source_files(base):
            try:
                head = f.read_text(errors="replace")[:4096]
            except OSError:
                continue
            for m in re.finditer(r"^:defines:\s*(.+)$", head, re.M):
                for tok in m.group(1).split():
                    rows.append((suite, f.relative_to(ROOT).as_posix(), tok.strip(), f.relative_to(ROOT).as_posix() + " (:defines: header)"))
    return sorted(set(rows)), computed, ties, variants


def harness_tops(suite: str, base: pathlib.Path) -> set[str]:
    """The files a suite's harness compiles as TOPS (repo-relative): verilator — every `t_*.py` driver's top
    (`top_filename`, else the same-stem `.v`). A top has a context of its own (its driver's defines and unit list);
    when such a file is also `` `include ``d by another test, its own harness's reading wins (SVPP-EXPANSION.4c.6:
    `t_mailbox.v`, a test of its own AND the body `t_mailbox_class.v` includes under another `MAILBOX_T`)."""
    tops: set[str] = set()
    if suite == "verilator":
        tdir = base / "test_regress" / "t"
        if tdir.is_dir():
            for py in tdir.glob("t_*.py"):
                m = re.search(r'top_filename\s*=\s*"t/([^"]+)"', py.read_text(errors="replace"))
                target = tdir / (m.group(1) if m else py.stem + ".v")
                if target.is_file():
                    tops.add(target.relative_to(ROOT).as_posix())
    return tops


def derive_fragments(suite: str, base: pathlib.Path, dirs: dict, per_file: list, own_context: frozenset = frozenset()):
    """SVPP-EXPANSION.4c.6 — an INCLUDE FRAGMENT's own-row context: the file is read as its includer compiles it.
    Returns (rows [(suite, relpath, order, includer, line, close)], reported {relpath: [notes]}). For every source
    file of the suite that is `` `include ``d: the includer's text BEFORE the `include` line is the macro state the
    fragment is compiled in (the lane materializes that prefix and preprocesses it as a unit file: its definitions
    carry over, its text is discarded — the fragment's own guard is untouched), and the includer's own context
    (units, include dirs, defines) is the fragment's. `close` is the number of conditional regions (`ifdef` /
    `ifndef`) still open at the `include` line — the header-guard idiom puts nearly every include inside one — and
    the lane closes the prefix with that many `` `endif `` lines: the probe refuses a unit file whose conditional is
    left open (measured: rc 3 `unterminated`), and closing it preserves the macro state at the cut exactly. The
    chain goes up: an includer that is itself a fragment contributes its own includer's prefix first (order 1 = the
    root). ⛔ Derived, never guessed: a fragment several files include (their prefixes differ) is a TIE, reported; a
    cycle or a chain deeper than MAX_FRAGMENT_CHAIN is reported; a file that declares a `package` is a unit of its
    own, never a fragment; a file its own harness compiles as a TOP or lists in a unit (`own_context`: the harness's
    tops and the unit tables' listed files) keeps its own harness's reading — reported, not a fragment; an include name
    that resolves to no single candidate (`.4b`'s ambiguity) yields no row."""
    base_rel = base.relative_to(ROOT).as_posix()
    files = source_files(base)
    population = {p.relative_to(ROOT).as_posix() for p in files}
    pf_dir = {(i, n): d for i, d, n in per_file}
    suite_dirs = [d for d in dirs if not d.startswith("__")]
    includers: dict[str, list[tuple[str, int, int]]] = {}
    for p in files:
        try:
            text = p.read_text(errors="replace")
        except OSError:
            continue
        if "`include" not in text:
            continue
        rel = p.relative_to(ROOT).as_posix()
        depth = 0
        for lineno, line in enumerate(text.split("\n"), 1):
            m = COND_LINE_RE.match(line)
            if m:
                depth += 1 if m.group(1) != "endif" else -1
                continue
            mi = INCLUDE_LINE_RE.match(line)
            if not mi:
                continue
            name = mi.group(1)
            name_posix = pathlib.PurePosixPath(name).as_posix()
            target = None
            if (p.parent / name_posix).is_file():
                target = (p.parent / name_posix).resolve()
            elif (rel, name) in pf_dir:
                target = (ROOT / pf_dir[(rel, name)] / name_posix).resolve()
            else:
                hits = [d for d in suite_dirs if (ROOT / d / name_posix).is_file()]
                if len(hits) == 1:
                    target = (ROOT / hits[0] / name_posix).resolve()
            if target is None:
                continue
            try:
                trel = target.relative_to(ROOT.resolve()).as_posix()
            except ValueError:
                continue
            if trel in population:
                includers.setdefault(trel, []).append((rel, lineno, max(depth, 0)))
    rows, reported = [], {}
    single: dict[str, tuple[str, int, int]] = {}
    for trel, incs in sorted(includers.items()):
        try:
            head = (ROOT / trel).read_text(errors="replace")[:20000]
        except OSError:
            continue
        if PACKAGE_DECL_RE.search(head):
            continue                                   # a package is a unit of its own, never a fragment
        if trel in own_context:
            reported[trel] = [f"a file its own harness compiles (a test's top, or a listed unit member) — its own context wins; the `include by {sorted({i for i, _, _ in incs})[0]} is not applied"]
            continue
        distinct = sorted({i for i, _, _ in incs})
        if len(distinct) > 1:
            reported[trel] = [f"included by {len(distinct)} files ({' '.join(distinct[:3])}{' …' if len(distinct) > 3 else ''}); their prefixes differ — a tie, not applied"]
            continue
        inc, lineno, depth = incs[0]
        single[trel] = (inc, lineno, depth)
    for trel, (inc, lineno, depth) in sorted(single.items()):
        chain = [(inc, lineno, depth)]
        seen = {trel, inc}
        cur = inc
        ok = True
        while cur in single:
            up, upline, updepth = single[cur]
            if up in seen or len(chain) >= MAX_FRAGMENT_CHAIN:
                reported[trel] = [f"the include chain above it cycles or exceeds {MAX_FRAGMENT_CHAIN} levels at {up}; not applied"]
                ok = False
                break
            chain.append((up, upline, updepth)); seen.add(up); cur = up
        if not ok:
            continue
        chain.reverse()                                # order 1 = the root includer
        for order, (i, l, d) in enumerate(chain, 1):
            rows.append((suite, trel, order, i, l, d))
    return rows, reported


def rows_for(suite: str, base: pathlib.Path):
    dirs, ambiguous, absent, stats, per_file = derive(base)
    ordered = sorted(dirs.items(), key=lambda kv: (-kv[1][0], kv[0]))
    rows = [(suite, "include_dir", d, str(n), ",".join(sorted(names))) for d, (n, names) in ordered]
    for name, source in HAND_DEFINES.get(suite, []):
        rows.append((suite, "define", name, "hand", source))
    for unit_file, source in HAND_UNITS.get(suite, []):
        rows.append((suite, "unit_file", unit_file, "hand", source))
    file_rows = sorted({(suite, includer, d, name) for includer, d, name in per_file})
    define_rows, computed, ties, variants = derive_defines(suite, base)
    stats["define_rows"] = len(define_rows)
    stats["computed_defines"] = computed
    stats["define_ties"] = ties
    stats["define_variants"] = variants
    unit_rows, unit_reported, unit_ties = derive_units(suite, base)
    stats["unit_rows"] = len(unit_rows)
    stats["unit_reported"] = unit_reported
    stats["unit_ties"] = unit_ties
    own_context = frozenset(harness_tops(suite, base)) | frozenset(r[1] for r in unit_rows)
    fragment_rows, fragment_reported = derive_fragments(suite, base, dirs, per_file, own_context)
    stats["fragment_rows"] = len(fragment_rows)
    stats["fragment_files"] = len({r[1] for r in fragment_rows})
    stats["fragment_reported"] = fragment_reported
    return rows, ambiguous, absent, stats, file_rows, define_rows, unit_rows, fragment_rows


def report(suite: str, rows, ambiguous, absent, stats, out=sys.stdout):
    n_dirs = sum(1 for r in rows if r[1] == "include_dir")
    print(
        f"PIPELINE-CONTEXT: suite={suite} files={stats['files']} local_includes={stats['local']} "
        f"include_dirs={n_dirs} per_file_rows={stats['per_file_rows']} "
        f"names_resolved_per_file={len(stats['resolved_per_file_names'])} "
        f"ambiguous_nonlocal_names={len(ambiguous)} absent_names={len(absent)} "
        f"identical_names={len(stats.get('identical', {}))} shadowed={len(stats.get('shadowed', []))} withheld={len(stats.get('withheld', {}))} "
        f"define_rows={stats.get('define_rows', 0)} computed_defines={len(stats.get('computed_defines', {}))} "
        f"define_ties={len(stats.get('define_ties', {}))} define_variant_dirs={len(stats.get('define_variants', {}))} "
        f"define_variant_names={sum(len(v) for v in stats.get('define_variants', {}).values())} "
        f"unit_rows={stats.get('unit_rows', 0)} unit_reported={len(stats.get('unit_reported', {}))} "
        f"unit_ties={len(stats.get('unit_ties', {}))} "
        f"fragment_rows={stats.get('fragment_rows', 0)} fragment_files={stats.get('fragment_files', 0)} "
        f"fragment_reported={len(stats.get('fragment_reported', {}))}",
        file=out,
    )
    for name in sorted(ambiguous):
        print(f"  ambiguous {name}: {' '.join(ambiguous[name])}", file=out)
    for name in sorted(absent):
        print(f"  absent    {name} (x{absent[name]})", file=out)
    for name in sorted(stats.get("identical", {})):
        print(f"  identical {name}: {stats['identical'][name]} copies, one text — resolved per file to the nearest copy", file=out)
    for name, d in stats.get("shadowed", []):
        print(f"  SHADOW    {name} is ambiguous and {d} holds a copy — the directory would resolve it silently; REFUSED", file=out)
    for (includer, d), names in sorted(stats.get("withheld", {}).items()):
        print(f"  withheld  {includer}: per-file dir {d} not applied — it holds the ambiguous {' '.join(sorted(names))} this file includes; the file keeps its refusal", file=out)
    for rel in sorted(stats.get("computed_defines", {})):
        print(f"  computed  {rel}: {' '.join(stats['computed_defines'][rel])} (the driver builds the value at run time; not derived)", file=out)
    for rel in sorted(stats.get("define_ties", {})):
        print(f"  tie       {rel}: {' '.join(stats['define_ties'][rel])} (several variant drivers carry defines; none applied)", file=out)
    for d in sorted(stats.get("define_variants", {})):
        for k, vals in sorted(stats["define_variants"][d].items()):
            shown = " ".join(f"{b}={v}" for b, v in sorted(vals.items()))
            print(f"  variant   {d}: {k} — {shown} (the IP's builds disagree; reported, not applied — SVPP-EXPANSION.4c.3)", file=out)
    for rel in sorted(stats.get("unit_reported", {})):
        print(f"  unit      {rel}: {'; '.join(stats['unit_reported'][rel])}", file=out)
    for rel in sorted(stats.get("unit_ties", {})):
        print(f"  unit-tie  {rel}: {' '.join(stats['unit_ties'][rel])} (several variant drivers name predecessors; none applied)", file=out)
    for rel in sorted(stats.get("fragment_reported", {})):
        print(f"  fragment  {rel}: {'; '.join(stats['fragment_reported'][rel])}", file=out)


def render(all_rows) -> str:
    lines = ["suite\tkind\tvalue\tneeded_by\tresolves"]
    for r in all_rows:
        lines.append("\t".join(r))
    return "\n".join(lines) + "\n"


def render_files(file_rows) -> str:
    lines = ["suite\trelpath\tinclude_dir\tname"]
    for r in file_rows:
        lines.append("\t".join(r))
    return "\n".join(lines) + "\n"


def render_defines(define_rows) -> str:
    lines = ["suite\trelpath\tdefine\tsource"]
    for r in define_rows:
        lines.append("\t".join(r))
    return "\n".join(lines) + "\n"


def render_units(unit_rows) -> str:
    lines = ["suite\trelpath\torder\tunit_file\tsource"]
    for r in unit_rows:
        lines.append("\t".join(str(x) for x in r))
    return "\n".join(lines) + "\n"


def render_fragments(fragment_rows) -> str:
    lines = ["suite\trelpath\torder\tincluder\tline\tclose"]
    for r in fragment_rows:
        lines.append("\t".join(str(x) for x in r))
    return "\n".join(lines) + "\n"


def refuse_if_shadowed(suite: str, stats) -> None:
    """`.4d`: a context table with a back-door guess in it is not written and does not pass --check."""
    if stats.get("shadowed"):
        print(f"PIPELINE-CONTEXT: REFUSED — suite={suite} emits a directory holding an ambiguous name (a guess by the back door; SVPP-EXPANSION.4d)", file=sys.stderr)
        raise SystemExit(4)


NOT_EVALUATED: list[str] = []  # CI-PARITY-GATE-ROT.121 — registered suites absent or EMPTY on this checkout


def derive_all(suites=None):
    NOT_EVALUATED.clear()
    all_rows = []
    all_file_rows = []
    all_define_rows = []
    all_unit_rows = []
    all_fragment_rows = []
    for suite, rel in SUITES.items():
        if suites and suite not in suites:
            continue
        base = ROOT / rel
        if not base.exists() or not any(base.iterdir()):
            # CI-PARITY-GATE-ROT.121 — an uninitialised submodule (every hosted checkout: actions/checkout fetches none)
            # leaves an EMPTY directory, which `exists()`; deriving from it read `files=0` for 13 suites and scored DRIFT.
            why = "missing" if not base.exists() else "EMPTY, an uninitialised submodule"
            print(f"PIPELINE-CONTEXT: suite={suite} NOT EVALUATED ({why}: {rel})", file=sys.stderr)
            NOT_EVALUATED.append(suite)
            continue
        rows, ambiguous, absent, stats, file_rows, define_rows, unit_rows, fragment_rows = rows_for(suite, base)
        report(suite, rows, ambiguous, absent, stats)
        refuse_if_shadowed(suite, stats)
        all_rows.extend(rows)
        all_file_rows.extend(file_rows)
        all_define_rows.extend(define_rows)
        all_unit_rows.extend(unit_rows)
        all_fragment_rows.extend(fragment_rows)
    return all_rows, all_file_rows, all_define_rows, all_unit_rows, all_fragment_rows


def self_test() -> int:
    """A synthetic suite: one local include, one shared header, one ambiguous name, one absent."""
    failures = 0
    with tempfile.TemporaryDirectory(dir=ROOT / "rust" / "target") as tmp:
        base = pathlib.Path(tmp)
        (base / "a").mkdir(); (base / "b").mkdir(); (base / "shared").mkdir(); (base / "x").mkdir(); (base / "y").mkdir()
        (base / "a" / "local.svh").write_text("// local\n")
        (base / "shared" / "hdr.svh").write_text("`define H 1\n")
        (base / "x" / "dup.svh").write_text("// x\n"); (base / "y" / "dup.svh").write_text("// y\n")
        (base / "a" / "top.sv").write_text('`include "local.svh"\n`include "hdr.svh"\n`include "dup.svh"\n`include "gone.svh"\n')
        (base / "b" / "other.sv").write_text('`include "hdr.svh"\n')
        # a per-file resolvable name: two candidates, one of them under the includer's own top
        (base / "top_a" / "env" / "seq_lib").mkdir(parents=True); (base / "top_b" / "env" / "seq_lib").mkdir(parents=True)
        (base / "top_a" / "env" / "seq_lib" / "list.sv").write_text("// a\n"); (base / "top_b" / "env" / "seq_lib" / "list.sv").write_text("// b\n")
        (base / "top_a" / "env" / "pkg.sv").write_text('`include "list.sv"\n')
        # `.4d`: a header with a non-source suffix, held once; a tie of byte-identical copies; a tie of differing texts
        (base / "consts").mkdir(); (base / "consts" / "k.vi").write_text("`define K 1\n")
        (base / "same1").mkdir(); (base / "same2").mkdir()
        (base / "same1" / "same.svh").write_text("// one text\n"); (base / "same2" / "same.svh").write_text("// one text\n")
        (base / "a" / "uses.sv").write_text('`include "k.vi"\n`include "same.svh"\n')
        dirs, ambiguous, absent, stats, per_file = derive(base)
        shared = (base.relative_to(ROOT) / "shared").as_posix()
        consts = (base.relative_to(ROOT) / "consts").as_posix()
        same1 = (base.relative_to(ROOT) / "same1").as_posix()
        top_a_seq = (base.relative_to(ROOT) / "top_a" / "env" / "seq_lib").as_posix()
        checks = [
            ("local include needs no dir", stats["local"] == 1),
            ("the shared header dir is emitted", shared in dirs),
            ("it is needed by both includers", dirs.get(shared, (0, set()))[0] == 2),
            ("a tied ambiguous name is reported with two candidates", len(ambiguous.get("dup.svh", [])) == 2),
            ("the tied ambiguous name emits NO dir", not any(d.endswith("/x") or d.endswith("/y") for d in dirs)),
            ("the absent name is reported", absent.get("gone.svh") == 1),
            ("a name with a NEAREST candidate is resolved per file, to the includer's own top",
             any(d == top_a_seq and n == "list.sv" for _, d, n in per_file)),
            ("and it is not reported as ambiguous", "list.sv" not in ambiguous),
            # SVPP-EXPANSION.4d
            ("a header with a non-source suffix (.vi) held once is a candidate: its dir is emitted, it is not absent",
             consts in dirs and "k.vi" not in absent),
            ("a tie of byte-identical copies is resolved PER FILE to the first copy and reported as identical — no suite dir",
             same1 not in dirs and any(d == same1 and n == "same.svh" for _, d, n in per_file)
             and "same.svh" not in ambiguous and stats["identical"].get("same.svh") == 2),
            ("a tie of differing texts is still ambiguous", len(ambiguous.get("dup.svh", [])) == 2 and "dup.svh" not in stats["identical"]),
            ("no emitted directory holds an ambiguous name here", stats["shadowed"] == []),
        ]
        for label, ok in checks:
            print(f"  {'PASS' if ok else 'FAIL'} {label}")
            failures += 0 if ok else 1
        # `.4d` the SHADOW arm: a dir emitted for one name holds an ambiguous one → reported, and derive_all REFUSES
        (base / "x" / "only_x.svh").write_text("// x only\n")
        (base / "a" / "shadow.sv").write_text('`include "only_x.svh"\n')   # x is now emitted for only_x.svh, and x holds dup.svh
        _, _, _, stats_sh, _ = derive(base)
        ok = ("dup.svh", (base.relative_to(ROOT) / "x").as_posix()) in stats_sh["shadowed"]
        print(f"  {'PASS' if ok else 'FAIL'} a directory emitted for one name that holds an ambiguous name is reported as a SHADOW")
        failures += 0 if ok else 1
        checks.append(("shadow", ok))
        try:
            refuse_if_shadowed("synthetic", stats_sh)
            ok = False
        except SystemExit as e:
            ok = e.code == 4
        print(f"  {'PASS' if ok else 'FAIL'} and the derivation REFUSES it (exit 4) — the red control of the shadow guard")
        failures += 0 if ok else 1
        checks.append(("shadow-refusal", ok))
        (base / "x" / "only_x.svh").unlink(); (base / "a" / "shadow.sv").unlink()
        # `.4d` the per-file form: top_a's pkg gets seq_lib for list.sv; make seq_lib also hold an ambiguous name pkg includes
        # two differing copies EQUIDISTANT from the includer, so amb.svh is a tie (ambiguous) for pkg.sv
        (base / "top_a" / "env" / "other").mkdir()
        (base / "top_a" / "env" / "seq_lib" / "amb.svh").write_text("// a\n"); (base / "top_a" / "env" / "other" / "amb.svh").write_text("// o\n")
        (base / "top_a" / "env" / "pkg.sv").write_text('`include "list.sv"\n`include "amb.svh"\n')
        _, amb_w, _, stats_w, per_file_w = derive(base)
        pkg = (base.relative_to(ROOT) / "top_a" / "env" / "pkg.sv").as_posix()
        ok = ("amb.svh" in amb_w and not any(i == pkg for i, _, _ in per_file_w)
              and (pkg, top_a_seq) in stats_w["withheld"] and stats_w["shadowed"] == [])
        print(f"  {'PASS' if ok else 'FAIL'} a per-file dir that holds an ambiguous name the includer also includes is WITHHELD for that includer, not refused")
        failures += 0 if ok else 1
        checks.append(("withheld", ok))
        (base / "top_a" / "env" / "seq_lib" / "amb.svh").unlink(); (base / "top_a" / "env" / "other" / "amb.svh").unlink(); (base / "top_a" / "env" / "other").rmdir()
        (base / "top_a" / "env" / "pkg.sv").write_text('`include "list.sv"\n')
        # SVPP-EXPANSION.4c.1 — a synthetic verilator tree: a same-stem driver with a literal and a
        # computed define, a single variant driver, and two variant drivers (a tie)
        t = base / "test_regress" / "t"; t.mkdir(parents=True)
        for stem in ("t_a", "t_b", "t_c"):
            (t / f"{stem}.v").write_text("module m; endmodule\n")
        (t / "t_a.py").write_text('test.compile(v_flags2=["+define+LIT=1 -DFLAG", "+define+CYC=" + str(test.cycles)],\n'
                                  '    verilator_flags2=["-CFLAGS", "-DCXX2"], make_flags=["CPPFLAGS_ADD=-DCXX"])\n')
        (t / "t_b_only.py").write_text('test.top_filename = "t/t_b.v"\ntest.compile(v_flags2=["+define+ONE"])\n')
        (t / "b_common.py").write_text('test.top_filename = "t/t_b.v"\ntest.compile(v_flags2=["+define+HELPER"])\n')
        (t / "t_c_x.py").write_text('test.top_filename = "t/t_c.v"\ntest.compile(v_flags2=["+define+X"])\n')
        (t / "t_c_y.py").write_text('test.top_filename = "t/t_c.v"\ntest.compile(v_flags2=["+define+Y"])\n')
        rows, computed, ties, _variants = derive_defines("verilator", base)
        defs = {(r[1].rsplit("/", 1)[1], r[2]) for r in rows}
        checks2 = [
            ("a same-stem driver's literal defines are emitted", ("t_a.v", "LIT=1") in defs and ("t_a.v", "FLAG") in defs),
            ("its computed define is reported, not emitted", not any(d == "CYC" or d.startswith("CYC=") for _, d in defs) and any(k.endswith("t_a.v") for k in computed)),
            ("a single variant driver's define is emitted", ("t_b.v", "ONE") in defs),
            ("a C++ define (-CFLAGS, CPPFLAGS_ADD) is neither emitted nor reported", not any(d.startswith("CXX") for _, d in defs) and not any("CXX" in n for ns in computed.values() for n in ns)),
            ("a *_common.py helper is not a driver", ("t_b.v", "HELPER") not in defs and not any(k.endswith("t_b.v") for k in ties)),
            ("two variant drivers are a tie: nothing emitted, the tie reported", not any(f == "t_c.v" for f, _ in defs) and any(k.endswith("t_c.v") for k in ties)),
        ]
        for label, ok in checks2:
            print(f"  {'PASS' if ok else 'FAIL'} {label}")
            failures += 0 if ok else 1
        checks += checks2
        # SVPP-EXPANSION.4c.2 — unit predecessors: a verilator driver's source-file arguments (top excluded,
        # a -f list reported, $TEST_ROOT resolved), a -U honoured; slang .f files (union, -F expanded, -v
        # skipped, a conflict reported); sv2v run.sh explicit lists (a glob and --siloed reported)
        (t / "t_u.v").write_text("module u; endmodule\n"); (t / "t_u_pre.vh").write_text("`define P 1\n")
        (t / "t_u").mkdir(); (t / "t_u" / "t_u_def.vh").write_text("`define Q 1\n")
        (t / "t_u.py").write_text('test.compile(v_flags2=["-DKEEP -DGONE -UGONE"], verilator_flags2=["t/t_u_pre.vh", "$TEST_ROOT/t_u_def.vh", "-f t/t_u.vc", "t/t_u.v"])\n')
        # .4c.5: the option file — three comment forms, nested -F relative to its own directory, a -v library file,
        # ${TEST_REGRESS} with a doubled slash, +define+ plus-args, an absent nested file; and an unreadable one
        (t / "t_u.vc").write_text("// a line comment t/t_u_no.v\n# a hash comment, as the suite's args.vc writes them\n/* +define+NON_DEF\n   t/t_u_no2.v */\n"
                                  "--stats +define+GOT1 +define+GOT2=1+GOT3 +define+QUOTED=\"\\\"x y\\\"\"\n${TEST_REGRESS}/t//t_u_f1.v\n-F $TEST_REGRESS/t/sub/t_u_sub.vc\n-v t/t_u_lib.v\n-f t/t_u_absent.vc\n")
        (t / "sub").mkdir(); (t / "sub" / "t_u_sub.vc").write_text("t_u_sub.v\n+incdir+.\n")
        for n in ("t_u_f1.v", "sub/t_u_sub.v", "t_u_lib.v", "t_w.v"):
            (t / n).write_text("module w; endmodule\n")
        (t / "t_w.py").write_text('test.lint(v_flags2=["-f t/t_w.vc"], fails=True)\n'); (t / "t_w.vc").write_text("/* an unterminated comment\nt/t_u_f1.v\n")
        urows, ureported, uties = derive_units("verilator", base)
        ulist = [(r[1].rsplit("/", 1)[1], r[2], r[3].rsplit("/", 1)[1]) for r in urows if r[1].endswith("t_u.v")]
        upreds = {}
        for r in urows:
            upreds.setdefault(r[1].split("/t/", 1)[1], []).append((r[2], r[3].rsplit("/", 1)[1]))
        lit, _ = driver_defines((t / "t_u.py").read_text())
        _m, vcdefs, _r = driver_unit_members((t / "t_u.py").read_text(), "t_u.v", "t", base / "test_regress")
        drows, _c, _t, _v = derive_defines("verilator", base)
        udefs = sorted(d for s_, f, d, src in drows if f.endswith("t_u.v"))
        sdata = base / "tests" / "unittests" / "data"; sdata.mkdir(parents=True)
        for n in ("a.sv", "b.sv", "c.sv", "lib.sv"):
            (sdata / n).write_text("module x; endmodule\n")
        (sdata / "u1.f").write_text("--library l a.sv b.sv -DX\n"); (sdata / "u2.f").write_text("b.sv -F u3.f -v lib.sv\n"); (sdata / "u3.f").write_text("c.sv\n")
        (sdata / "u4.f").write_text("c.sv b.sv\n")   # b after c here, c after b in u2 → a conflict
        # SVPP-EXPANSION.4c.6 — include fragments: a two-level chain (root → mid → leaf), a tie, a conditional include,
        # a package target (never a fragment), a local include resolved beside the includer
        fr = base / "frag"; fr.mkdir()
        (fr / "root.sv").write_text('`define X 1\n`include "mid.svh"\n`ifdef Q\n`include "cond.svh"\n`endif\n`include "tie.svh"\n`include "p.sv"\nmodule r; endmodule\n')
        (fr / "mid.svh").write_text('`define Y 2\n`include "leaf.svh"\n')
        (fr / "leaf.svh").write_text('localparam int L = `X + `Y;\n')
        (fr / "cond.svh").write_text('// cond\n'); (fr / "tie.svh").write_text('// tie\n'); (fr / "other.sv").write_text('`include "tie.svh"\n')
        (fr / "p.sv").write_text('package p; endpackage\n')
        (fr / "own.sv").write_text('module own; endmodule\n'); (fr / "incl.sv").write_text('`include "own.sv"\n')
        fdirs, _a, _b, _st, fpf = derive(base)
        frows, freported = derive_fragments("x", base, fdirs, fpf, frozenset({(fr / "own.sv").relative_to(ROOT).as_posix()}))
        fchain = {r[1].rsplit("/", 1)[1]: [(r[2], r[3].rsplit("/", 1)[1], r[4]) for r in frows if r[1] == r[1]] for r in frows}
        fby = {}
        fclose = {}
        for r in frows:
            fby.setdefault(r[1].rsplit("/", 1)[1], []).append((r[2], r[3].rsplit("/", 1)[1], r[4]))
            fclose.setdefault(r[1].rsplit("/", 1)[1], []).append(r[5])
        srows, sreported, _ = derive_units("slang", base)
        srel = [(r[1].rsplit("/", 1)[1], r[2], r[3].rsplit("/", 1)[1]) for r in srows]
        vd = base / "test" / "x"; vd.mkdir(parents=True)
        for n in ("p.svh", "m.sv", "z.sv"):
            (vd / n).write_text("module y; endmodule\n")
        (vd / "run.sh").write_text('convert "std" $cs p.svh m.sv\nrunAndCapture --siloed p.svh m.sv\nrunAndCapture *.sv\n')
        vrows, vreported, _ = derive_units("sv2v", base)
        vrel = [(r[1].rsplit("/", 1)[1], r[2], r[3].rsplit("/", 1)[1]) for r in vrows]
        checks3 = [
            ("verilator: the driver's source arguments precede the top, in order, the top excluded, $TEST_ROOT resolved, the -f file's sources among them (.4c.5)",
             ulist == [("t_u.v", 1, "t_u_pre.vh"), ("t_u.v", 2, "t_u_def.vh"), ("t_u.v", 3, "t_u_f1.v"), ("t_u.v", 4, "t_u_sub.v")]),
            ("verilator: a -f option file is READ — its comments (//, #, /* */) skipped, ${TEST_REGRESS}// resolved, -F relative to its own directory (.4c.5)",
             upreds.get("t_u_f1.v") == [(1, "t_u_pre.vh"), (2, "t_u_def.vh")] and upreds.get("sub/t_u_sub.v") == [(1, "t_u_pre.vh"), (2, "t_u_def.vh"), (3, "t_u_f1.v")]
             and not any("t_u_no" in r[3] for r in urows)),
            ("verilator: every listed member reads after the files before it; the top after all of them (.4c.5)",
             upreds.get("t_u_pre.vh") is None and upreds.get("t_u/t_u_def.vh") == [(1, "t_u_pre.vh")]),
            ("verilator: a -v library file reads after every listed file, the top included, and precedes nothing (.4c.5)",
             upreds.get("t_u_lib.v") == [(1, "t_u_pre.vh"), (2, "t_u_def.vh"), (3, "t_u_f1.v"), (4, "t_u_sub.v"), (5, "t_u.v")] and not any(r[3].endswith("t_u_lib.v") for r in urows)),
            ("verilator: the -f file's +define+A+B=V plus-args reach the top's defines, a commented one does not (.4c.5)",
             vcdefs == ["GOT1", "GOT2=1", "GOT3"] and {"GOT1", "GOT2=1", "GOT3", "KEEP"} <= set(udefs) and "NON_DEF" not in udefs
             and not any(d.startswith("QUOTED") for d in udefs) and any("+define+QUOTED=… in t/t_u.vc (a quoted value" in n for v in ureported.values() for n in v)),
            ("verilator: an absent nested option file and an unterminated comment are REPORTED, not read (.4c.5)",
             any("t/t_u_absent.vc (an option file absent" in n for k, v in ureported.items() if k.endswith("t_u.v") for n in v)
             and any("unterminated comment" in n for k, v in ureported.items() if k.endswith("t_w.v") for n in v) and not any(r[1].endswith("t_w.v") for r in urows)),
            ("verilator: a define the driver -U's again is not emitted", lit == ["KEEP"]),
            ("slang: .f units are unioned in order, -F expanded, -v skipped", ("b.sv", 1, "a.sv") in srel and ("lib.sv", 1, "b.sv") not in srel and not any(r[0] == "lib.sv" for r in srel)),
            ("slang: a pair two .f files order differently is a conflict, reported", not any(r[0] == "c.sv" for r in srel) and any(k.endswith("c.sv") for k in sreported)),
            ("fragments: a leaf reads after its includer's prefix, the root's first (.4c.6)", fby.get("leaf.svh") == [(1, "root.sv", 2), (2, "mid.svh", 2)] and fby.get("mid.svh") == [(1, "root.sv", 2)]),
            ("fragments: a file two includers reach is a tie, reported, not applied (.4c.6)", "tie.svh" not in fby and any(k.endswith("tie.svh") and "tie" in v[0] for k, v in freported.items())),
            ("fragments: an include inside a conditional region is applied with the open region counted, for the lane to close (.4c.6)",
             fby.get("cond.svh") == [(1, "root.sv", 4)] and fclose.get("cond.svh") == [1] and fclose.get("leaf.svh") == [0, 0]),
            ("fragments: a package is a unit of its own, never a fragment (.4c.6)", "p.sv" not in fby and not any(k.endswith("p.sv") for k in freported)),
            ("fragments: a file its own harness compiles keeps its own reading — reported, not a fragment (.4c.6)", "own.sv" not in fby and any(k.endswith("own.sv") and "own context wins" in v[0] for k, v in freported.items())),
            ("sv2v: an explicit multi-file convert names predecessors; --siloed and a glob are reported", vrel == [("m.sv", 1, "p.svh")] and any("siloed" in n for v in vreported.values() for n in v) and any("glob" in n for v in vreported.values() for n in v)),
        ]
        for label, ok in checks3:
            print(f"  {'PASS' if ok else 'FAIL'} {label}")
            failures += 0 if ok else 1
        checks += checks3
        # SVPP-EXPANSION.4c.3 — a synthetic opentitan tree: a base cfg (top-level build_opts, two build modes — one
        # templated value — and a test picking a mode), two run cfgs importing it and enabling one mode each, a cfg
        # with top-level build_opts only, a cfg with no defines, and a nested cfg directory
        ot = base / "ot"
        for d in ("hw/ip/x/dv/env", "hw/ip/x/dv/tb", "hw/ip/y/dv", "hw/ip/z/dv", "hw/ip/p/dv/p_sub", "hw/dv/tools/dvsim"):
            (ot / d).mkdir(parents=True)
        (ot / "hw/dv/tools/dvsim/common_sim_cfg.hjson").write_text('{\n  build_opts: ["+define+UVM", "+define+W={tl_aw}"]\n}\n')
        (ot / "hw/ip/x/dv/x_base_sim_cfg.hjson").write_text(
            '{\n  name: x\n  import_cfgs: ["{proj_root}/hw/dv/tools/dvsim/common_sim_cfg.hjson"]\n'
            '  build_opts: ["+define+XTOP=3", "+define+C={tl_aw}"]\n'
            '  build_modes: [\n    {\n      name: amode\n      build_opts: ["+define+A=1", "+define+T={tl_aw}"]\n    }\n'
            '    {\n      name: bmode\n      build_opts: ["+define+B"]\n    }\n  ]\n'
            '  tests: [\n    {\n      name: t1\n      build_mode: bmode\n    }\n  ]\n}\n')
        (ot / "hw/ip/x/dv/x_a_sim_cfg.hjson").write_text(
            '{\n  variant: a\n  import_cfgs: ["{proj_root}/hw/ip/x/dv/x_base_sim_cfg.hjson"]\n  en_build_modes: ["amode"]\n}\n')
        (ot / "hw/ip/x/dv/x_b_sim_cfg.hjson").write_text(
            '{\n  variant: b\n  import_cfgs: ["{proj_root}/hw/ip/x/dv/x_base_sim_cfg.hjson"]\n  en_build_modes: ["bmode"]\n}\n')
        (ot / "hw/ip/y/dv/y_sim_cfg.hjson").write_text('{\n  name: y\n  build_opts: ["+define+Y=2", "-lutil"]\n}\n')
        (ot / "hw/ip/z/dv/z_sim_cfg.hjson").write_text('{\n  name: z\n  build_modes: []\n}\n')
        (ot / "hw/ip/p/dv/p_sim_cfg.hjson").write_text('{\n  name: p\n  build_opts: ["+define+TOP"]\n}\n')
        (ot / "hw/ip/p/dv/p_sub/p_sub_sim_cfg.hjson").write_text('{\n  name: psub\n  build_opts: ["+define+SUB"]\n}\n')
        for f in ("hw/ip/x/dv/env/x_env.sv", "hw/ip/x/dv/tb/tb.sv", "hw/ip/y/dv/y.sv", "hw/ip/z/dv/z.sv", "hw/ip/p/dv/t.sv", "hw/ip/p/dv/p_sub/s.sv"):
            (ot / f).write_text("module m; endmodule\n")
        rows4, computed4, _ties4, variants4 = derive_defines("opentitan", ot)
        defs4 = {(r[1].rsplit("/", 1)[1], r[2]) for r in rows4}
        xvar = next((v for d, v in variants4.items() if d.endswith("hw/ip/x/dv")), {})
        checks4 = [
            ("a define every build of the directory carries with one value is applied to the files the directory owns, the base's top-level build_opts included (XTOP=3 on env/ and tb/)",
             ("x_env.sv", "XTOP=3") in defs4 and ("tb.sv", "XTOP=3") in defs4),
            ("a name the builds disagree on is reported per build, never applied (A=1 under x_a's builds, absent under x_b's; a base cfg is not a build of its own)",
             not any(f.startswith(("x_env", "tb")) and d in ("A=1", "B") for f, d in defs4)
             and xvar.get("A") == {"x_a_sim_cfg+amode": "1", "x_a_sim_cfg+amode+bmode": "1", "x_b_sim_cfg+bmode": "(absent)"}),
            ("a test's build_mode pick is one more build of the run cfg importing it, on top of its enabled modes",
             xvar.get("B", {}).get("x_a_sim_cfg+amode+bmode") == "(defined)" and xvar.get("B", {}).get("x_a_sim_cfg+amode") == "(absent)"),
            ("a templated value every build shares is computed and reported, never emitted; one a single build carries is a disagreement",
             not any(d.startswith(("C=", "T=")) for _, d in defs4) and any(k.endswith("x_env.sv") and v == ["C"] for k, v in computed4.items()) and "T" in xvar),
            ("a cfg's own top-level build_opts define is emitted; a non-define option is not",
             ("y.sv", "Y=2") in defs4 and not any(f == "y.sv" and d != "Y=2" for f, d in defs4)),
            ("a cfg with no defines emits nothing", not any(f == "z.sv" for f, _ in defs4)),
            ("the nearest cfg directory owns a file (nested cfg)", ("s.sv", "SUB") in defs4 and ("s.sv", "TOP") not in defs4 and ("t.sv", "TOP") in defs4),
            ("the project-wide common cfg's defines are not emitted per file (the suite rows' business)",
             not any(d == "UVM" for _, d in defs4)),
        ]
        for label, ok in checks4:
            print(f"  {'PASS' if ok else 'FAIL'} {label}")
            failures += 0 if ok else 1
        checks += checks4
    print(f"PIPELINE-CONTEXT-SELF-TEST: checks={len(checks)} failed={failures}")
    return 7 if failures else 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--write", action="store_true")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--suite", action="append")
    ap.add_argument("--list-suites", action="store_true")
    args = ap.parse_args()
    if args.list_suites:
        for suite in SUITES:
            print(suite)
        return 0
    if args.self_test:
        return self_test()
    all_rows, all_file_rows, all_define_rows, all_unit_rows, all_fragment_rows = derive_all(args.suite)
    if NOT_EVALUATED:
        # A partial corpus can neither re-derive the tracked context nor rewrite it: --check is NOT EVALUATED (said,
        # exit 0, the sibling arms' convention) and --write REFUSES (exit 2) rather than publishing a shrunken table.
        print(f"PIPELINE-CONTEXT: NOT EVALUATED — {len(NOT_EVALUATED)} registered suite(s) absent or empty here "
              f"({', '.join(NOT_EVALUATED)}); the tracked context cannot be re-derived from a partial corpus "
              f"(git submodule update --init --depth 1 -- stimuli/sv/subs/<suite>)")
        return 2 if args.write else 0
    if args.write:
        TSV.write_text(render(all_rows))
        FILES_TSV.write_text(render_files(all_file_rows))
        DEFINES_TSV.write_text(render_defines(all_define_rows))
        UNITS_TSV.write_text(render_units(all_unit_rows))
        FRAGMENTS_TSV.write_text(render_fragments(all_fragment_rows))
        print(f"PIPELINE-CONTEXT: wrote {TSV.relative_to(ROOT)} rows={len(all_rows)}, {FILES_TSV.relative_to(ROOT)} rows={len(all_file_rows)}, {DEFINES_TSV.relative_to(ROOT)} rows={len(all_define_rows)} and {UNITS_TSV.relative_to(ROOT)} rows={len(all_unit_rows)}, {FRAGMENTS_TSV.relative_to(ROOT)} rows={len(all_fragment_rows)}")
    if args.check:
        current = TSV.read_text() if TSV.exists() else ""
        current_files = FILES_TSV.read_text() if FILES_TSV.exists() else ""
        current_defines = DEFINES_TSV.read_text() if DEFINES_TSV.exists() else ""
        current_units = UNITS_TSV.read_text() if UNITS_TSV.exists() else ""
        current_fragments = FRAGMENTS_TSV.read_text() if FRAGMENTS_TSV.exists() else ""
        if current != render(all_rows) or current_files != render_files(all_file_rows) or current_defines != render_defines(all_define_rows) or current_units != render_units(all_unit_rows) or current_fragments != render_fragments(all_fragment_rows):
            print(f"PIPELINE-CONTEXT: DRIFT — the tracked context differs from a fresh derivation; run --write", file=sys.stderr)
            return 1
        print("PIPELINE-CONTEXT: OK — the tracked context equals a fresh derivation")
    return 0


if __name__ == "__main__":
    sys.exit(main())
