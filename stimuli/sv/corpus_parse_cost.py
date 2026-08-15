#!/usr/bin/env python3
"""stimuli/sv/corpus_parse_cost.py — the SV parser's PARSE COST, on two metrics.

`ENGINE-UNIVERSAL-SERVICES.20` acceptance (d), under the 2026-08-14 ruling C.

⛔ WHY THIS EXISTS. `.17` slice 9 shipped the guarded left-recursion admission and it cost
**+24.3 % parse time** on the SV corpus. Every gate the repository has was GREEN across that
slowdown — the generated lint, the two-sided repro ratchet, the corpus pass/fail count and all
registered doctrines. *A cost nothing measures is a cost that grows.* This instrument is the
thing that measures it.

⭐⭐ TWO METRICS, TWO GRAPHS, AND ONLY ONE OF THEM BINDS — which is the whole design.

PGEN runs a parse through one of two graphs (the "observability twin", TOOLBOX 2.1 / 3.4
ROUTING). A BARE parse runs the fused `cascade_*` functions; a parse with any diagnostic
consumer attached runs the PROTOCOL graph. They are different code, and the +24.3 % was
measured on the fused one.

| metric | graph observed | machine-dependent? | verdict role |
|---|---|---|---|
| total rule ENTRIES | PROTOCOL (`parse_*`) | no — exact integer | **BINDING** |
| bare-parse wall clock | FUSED (`cascade_*`) | yes | ADVISORY, wide band |

- **Entries bind** because they are an exact integer: verified deterministic across repeated
  release runs AND byte-identical between the debug and release probes. A number that cannot
  drift with machine load is the only kind a standing ratchet can key on. This matters
  concretely: the first cost figure this leaf recorded, `~11 %`, was WRONG because it compared
  wall clock across "materially faster machine conditions". A wall-clock-primary ratchet
  inherits exactly that defect.
- **Wall clock advises** because it is the only view of the fused graph the +24.3 % was
  measured on — and it is the half that cannot survive a machine change or a hosted runner.

⛔ THE BLIND SPOT IS DECLARED, NOT DISCOVERED LATER. The entry counters tick only in the
PROTOCOL graph. The fused twins of the very rules this leaf is about — `cascade_match_
casting_type_lr_suffix`, `cascade_build_select_expression_lr_base`, … — tick nothing. So the
binding metric CANNOT describe fused-graph cost, and the advisory one cannot be trusted to a
few percent. Neither is sufficient alone; the report says so on every run rather than
publishing one number that quietly means less than it appears to.

⚠️ THE PROCESS FLOOR IS SUBTRACTED, BECAUSE IT MEASURABLY DOMINATES. A `parseability_probe`
invocation on a one-line module takes ~9.8 ms before it parses anything (fork + exec + the SV
stdlib preload). The median corpus file's whole wall time is ~18 ms. An unadjusted per-file
wall-clock advisory is therefore mostly a measurement of `fork`, and would drift with the
loader rather than with the parser. The floor is re-measured in the SAME run — never assumed —
and subtracted, and the advisory sample is restricted to files where the remaining signal
dominates.

MODES
  (default)  measure the pinned sample -> the tracked baseline artifacts
  --census   full-corpus entry census (~10 min at -j8); the input to --select
  --select   derive the pinned sample manifest FROM a census, deterministically

USAGE
  python3 stimuli/sv/corpus_parse_cost.py --outdir <dir>
  python3 stimuli/sv/corpus_parse_cost.py --census --outdir <dir>
  python3 stimuli/sv/corpus_parse_cost.py --select --census-tsv <f> --out <manifest>

CONTRACT: deterministic on the binding metric; repo-root-relative paths everywhere; refuses
(exit 2) rather than reporting a clean measurement it could not take.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import hashlib
import json
import os
import re
import statistics
import subprocess
import sys
import tempfile
import time

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

GRAMMAR = "systemverilog"
PROFILE = "sv_2017"
GRAMMAR_FILE = "grammars/systemverilog.ebnf"
GENERATED_PARSER = "generated/systemverilog_parser.rs"
DEFAULT_MANIFEST = "stimuli/sv/parse_cost_sample.tsv"
DEFAULT_PROBE = "rust/target/release/parseability_probe"
FALLBACK_PROBE = "rust/target/debug/parseability_probe"
CORPUS_MANIFEST = "stimuli/sv/characterization/durations.tsv"

PER_FILE_TIMEOUT_S = 120

# ── the LR-family classifier ────────────────────────────────────────────────────────────────
#
# The guarded admission rewrites a left-recursive rule `X` into `X_lr_base ( X_lr_suffix )*`,
# emitting `X_lr_base`, `X_lr_suffix` and the per-alternative `X_lr_suffix_r<N>`. Attributing
# entries to that family is what lets `.20` acceptance (b)'s A/B compute its delta directly
# off this artifact instead of re-deriving it.
#
# ⛔ ANCHORED, not a substring test. `_lr_base` is a PREFIX of `_lr_baseline`, so a substring
# match would silently absorb any future rule whose name merely starts that way — a miss in the
# flattering direction, since it would inflate the family and make the admission look like it
# already owns cost it does not. The controls below pin exactly that case.
LR_FAMILY_RE = re.compile(r"_lr_base$|_lr_suffix(_r\d+)?$")


def is_lr_family(rule: str) -> bool:
    """Does this rule name belong to a guarded-admission LR family?"""
    return LR_FAMILY_RE.search(rule) is not None


def _self_check() -> None:
    """GROUND TRUTH, re-run on every invocation (microseconds).

    An instrument with no ground truth is a confident guess
    (docs/decisions/feedback_instrument_needs_ground_truth.md). Both the POSITIVE controls and
    the NEGATIVES are pinned, and a miss REFUSES rather than publishing a measurement.
    """
    cases = [
        # every shape the pass actually emits, taken from generated/systemverilog_parser.rs
        ("casting_type_lr_base", True),
        ("casting_type_lr_suffix", True),
        ("casting_type_lr_suffix_r0", True),
        ("property_expr_lr_suffix_r10", True),
        ("incomplete_class_scoped_type_sv_2023_lr_base", True),
        # the rule the admission REPLACED is not itself in the family
        ("casting_type", False),
        # ⛔ the near-miss this classifier is anchored for
        ("something_lr_baseline", False),
        ("lr_base_helper", False),
        # an unrelated rule that merely contains the token
        ("suffix_rule", False),
        ("module_declaration", False),
    ]
    misses = [(r, want, is_lr_family(r)) for r, want in cases if is_lr_family(r) != want]
    if misses:
        for rule, want, got in misses:
            print(f"parse-cost: CONTROL MISSED: {rule!r} want={want} got={got}", file=sys.stderr)
        print("parse-cost: the LR-family classifier does not discriminate; refusing", file=sys.stderr)
        sys.exit(2)


def die(msg: str, code: int = 2) -> None:
    print(f"parse-cost: {msg}", file=sys.stderr)
    sys.exit(code)


def sha256_of(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def resolve_probe(explicit: str | None) -> str:
    """The parse binary. Either build is legitimate: entry counts are build-mode-independent
    (verified — the debug and release probes emit byte-identical dumps), so a tree with only a
    debug build can still run the BINDING metric. The advisory one says which build it used,
    because that number is not build-independent at all."""
    if explicit:
        p = explicit if os.path.isabs(explicit) else os.path.join(ROOT, explicit)
        if not os.access(p, os.X_OK):
            die(f"--probe {explicit} is not executable")
        return p
    for rel in (DEFAULT_PROBE, FALLBACK_PROBE):
        p = os.path.join(ROOT, rel)
        if os.access(p, os.X_OK):
            return p
    die("no parseability_probe found. Build one:\n"
        "  (cd rust && cargo build --release --features generated_parsers --bin parseability_probe)")
    raise AssertionError("unreachable")


def sub_corpus_of(rel: str) -> str:
    m = re.search(r"/subs/([^/]+)/", rel)
    if m:
        return m.group(1)
    if "/stimuli/sv/uvm/" in f"/{rel}":
        return "uvm-core"
    return "(unknown)"


# ── measurement: the BINDING metric ─────────────────────────────────────────────────────────

def measure_one_entries(args: tuple[str, str]) -> tuple | None:
    """One file -> its exact counter row. None when the probe produced no dump.

    ⭐ USES THE OUTCOME DUMP (TOOLBOX 3.5), NOT THE PLAIN ENTRY DUMP (3.4), AND THAT IS A
    MEASURED CHOICE. The outcome dump is a strict superset — raw entries, COMMITTED entries and
    memo hits — and it was measured to cost the SAME 0.04 s on the same file, so the extra two
    dimensions are free. They matter because `raw − committed` is the parse's FAILED-SPECULATION
    work, which is the mechanism a structural GUARD spends its cost through: measured on
    `ast_reg_pkg.sv`, 95.4 % of all rule entries are rolled back, and the guarded-admission
    family is 100 % rolled back (committed = 0). A ratchet on raw entries alone would be blind
    to a guard that doubles its probing and commits the same amount.
    ⚠️ Verified before being relied on, not assumed: the two dumps report an IDENTICAL
    `total_entries`, so this baseline and the census that selected its sample stay comparable.
    """
    probe, rel = args
    fd, tmp = tempfile.mkstemp(suffix=".json", dir=os.path.join(ROOT, "rust", "target"))
    os.close(fd)
    try:
        subprocess.run(
            [probe, "--parse", GRAMMAR, os.path.join(ROOT, rel), "--profile", PROFILE,
             "--dump-rule-outcome-counts-json", tmp],
            capture_output=True, timeout=PER_FILE_TIMEOUT_S,
        )
        if os.path.getsize(tmp) == 0:
            return None
        with open(tmp, encoding="utf-8") as fh:
            d = json.load(fh)
    except (subprocess.TimeoutExpired, json.JSONDecodeError, OSError):
        return None
    finally:
        try:
            os.unlink(tmp)
        except OSError:
            pass
    entries = d.get("rule_entry_counts", {})
    committed = d.get("rule_committed_counts", {})
    lr_e = sum(v for k, v in entries.items() if is_lr_family(k))
    lr_c = sum(v for k, v in committed.items() if is_lr_family(k))
    return (rel, "yes" if d.get("accepted") else "no",
            int(d.get("total_entries", 0)), int(d.get("total_committed", 0)),
            int(d.get("total_memo_hits", 0)), lr_e, lr_c)


def measure_entries(probe: str, files: list[str], jobs: int) -> tuple[list[tuple], list[str]]:
    """Entries for every file, in parallel.

    ⭐ Parallelism is SOUND here and is not for the advisory metric: an entry count is an exact
    property of (parser, input), so contention cannot move it. The wall-clock pass below is
    serial for exactly the opposite reason.
    """
    rows: list[tuple] = []
    nodump: list[str] = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as ex:
        for rel, res in zip(files, ex.map(measure_one_entries, [(probe, f) for f in files])):
            if res is None:
                nodump.append(rel)
            else:
                rows.append(res)
    rows.sort(key=lambda r: (sub_corpus_of(r[0]), r[0]))
    return rows, sorted(nodump)


# ── measurement: the ADVISORY metric ────────────────────────────────────────────────────────

def _bare_parse_ms(probe: str, path: str) -> float:
    t0 = time.perf_counter()
    subprocess.run([probe, "--parse", GRAMMAR, path, "--profile", PROFILE],
                   capture_output=True, timeout=PER_FILE_TIMEOUT_S)
    return (time.perf_counter() - t0) * 1000.0


def measure_process_floor(probe: str, repeats: int) -> float:
    """The per-invocation cost that is NOT parsing: fork + exec + the SV stdlib preload.

    ⛔ Re-measured every run, never a constant in this file. It is a property of the machine and
    the binary, and baking a measured-once number into the instrument is how a measurement stops
    describing the tree it runs in.
    """
    with tempfile.NamedTemporaryFile("w", suffix=".sv", delete=False,
                                     dir=os.path.join(ROOT, "rust", "target")) as fh:
        fh.write("module pgen_parse_cost_floor_probe; endmodule\n")
        tiny = fh.name
    try:
        return statistics.median([_bare_parse_ms(probe, tiny) for _ in range(repeats)])
    finally:
        try:
            os.unlink(tiny)
        except OSError:
            pass


def measure_wallclock(probe: str, files: list[str], repeats: int) -> dict:
    """Bare-parse wall clock over the advisory sample — SERIAL, median-of-K, floor-subtracted.

    ⛔ Serial on purpose. Under `-P 8` a file competes with seven siblings for the same cores,
    which is how a timing number becomes a statement about the machine instead of the parser —
    the confound that produced this leaf's wrong `~11 %`.
    """
    floor = measure_process_floor(probe, repeats)
    per_file = []
    for rel in files:
        med = statistics.median([_bare_parse_ms(probe, os.path.join(ROOT, rel))
                                 for _ in range(repeats)])
        per_file.append((rel, med, max(0.0, med - floor)))
    total_adj = sum(a for _, _, a in per_file)
    total_raw = sum(m for _, m, _ in per_file)
    return {
        "files": len(files),
        "repeats": repeats,
        "process_floor_ms": round(floor, 3),
        "total_raw_ms": round(total_raw, 1),
        "total_parse_ms": round(total_adj, 1),
        "floor_share_pct": round(100.0 * (total_raw - total_adj) / total_raw, 1) if total_raw else 0.0,
        "per_file": [{"path": r, "median_ms": round(m, 2), "parse_ms": round(a, 2)}
                     for r, m, a in per_file],
    }


# ── identity ────────────────────────────────────────────────────────────────────────────────

def sample_input_digest(files: list[str]) -> str:
    """One digest over the sample's INPUTS — the manifest order plus every file's bytes.

    ⛔ This is the half of the identity that a parser hash cannot cover. Entries are a function
    of (parser, grammar profile, INPUT). The corpora are git submodules, so a submodule bump can
    move the measurement without touching a single byte of PGEN — and the cheap identity leg of
    the gate would have reported a fresh baseline over a corpus that had changed underneath it.
    """
    h = hashlib.sha256()
    for rel in files:
        h.update(rel.encode("utf-8"))
        h.update(b"\0")
        with open(os.path.join(ROOT, rel), "rb") as fh:
            for chunk in iter(lambda: fh.read(1 << 20), b""):
                h.update(chunk)
        h.update(b"\0")
    return h.hexdigest()


def identity(files: list[str]) -> dict:
    ident = {}
    for label, rel in (("grammar", GRAMMAR_FILE), ("generated_parser", GENERATED_PARSER)):
        path = os.path.join(ROOT, rel)
        if not os.path.isfile(path):
            die(f"required input missing: {rel}\n"
                f"  generated/ is not tracked — regenerate it:\n"
                f"    make -C rust SHELL=/bin/bash regenerate_generated_parsers")
        ident[label] = {"path": rel, "sha256": sha256_of(path)}
    ident["sample_inputs"] = {"path": DEFAULT_MANIFEST, "sha256": sample_input_digest(files)}
    return ident


# ── the pinned sample ───────────────────────────────────────────────────────────────────────

def read_manifest(path: str) -> list[tuple[str, str]]:
    """The pinned sample: `tier \t repo-root-relative path`, comments and blanks skipped."""
    if not os.path.isfile(path):
        die(f"sample manifest missing: {os.path.relpath(path, ROOT)}")
    out = []
    with open(path, encoding="utf-8") as fh:
        for n, line in enumerate(fh, 1):
            line = line.rstrip("\n")
            if not line.strip() or line.startswith("#"):
                continue
            parts = line.split("\t")
            if len(parts) != 2:
                die(f"{os.path.relpath(path, ROOT)}:{n}: expected 2 tab-separated fields, got "
                    f"{len(parts)}")
            out.append((parts[0], parts[1]))
    if not out:
        die(f"{os.path.relpath(path, ROOT)} lists no files")
    return out


def select_sample(census_tsv: str, hot: int, lr: int, breadth: int) -> list[tuple[str, str]]:
    """Derive the pinned sample from a full-corpus census, DETERMINISTICALLY.

    ⛔ The derivation lives in this tracked instrument rather than in a shell one-liner, so the
    sample can be re-derived and audited instead of being a list somebody once produced. Three
    tiers, because one selection rule cannot serve both jobs the ratchet has:

      hot      the heaviest files by entries — where parse cost actually concentrates, so a
               regression in the hot path cannot hide behind thousands of trivial files;
      lr       the heaviest files by GUARDED-ADMISSION entries — the mechanism `.20` is about,
               so the ratchet is sensitive to the specific thing that regressed;
      breadth  a deterministic stride across everything else — so the ratchet is not blind to a
               regression that lands somewhere the first two tiers do not look.
    """
    rows = []
    with open(census_tsv, encoding="utf-8") as fh:
        for line in fh:
            parts = line.rstrip("\n").split("\t")
            if len(parts) != 5 or parts[2] not in ("True", "False"):
                continue
            rows.append((parts[1], int(parts[3]), int(parts[4])))
    if not rows:
        die(f"census {census_tsv} yielded no usable rows")
    chosen: dict[str, str] = {}
    for rel, _e, _l in sorted(rows, key=lambda r: (-r[1], r[0]))[:hot]:
        chosen[rel] = "hot"
    for rel, _e, lrv in sorted(rows, key=lambda r: (-r[2], r[0])):
        if len(chosen) >= hot + lr:
            break
        if lrv > 0 and rel not in chosen:
            chosen[rel] = "lr"
    # ⛔ BREADTH IS STRATIFIED PER SUB-CORPUS, NOT A FLAT STRIDE ACROSS THE WHOLE LIST. A flat
    # stride is proportional, so it silently drops the SMALL sub-corpora entirely: the first cut
    # covered 12 of the 14 vendored suites and missed `scr1` and `uvm-core` outright. Coverage
    # that depends on a suite being big enough to survive a stride is coverage by luck. Every
    # sub-corpus present therefore gets at least one row, and the remaining budget is filled by
    # a deterministic stride within each suite.
    rest: dict[str, list[str]] = {}
    for rel in sorted(r[0] for r in rows if r[0] not in chosen):
        rest.setdefault(sub_corpus_of(rel), []).append(rel)
    if breadth > 0 and rest:
        per = max(1, breadth // len(rest))
        for sub in sorted(rest):
            pool = rest[sub]
            stride = max(1, len(pool) // per)
            for rel in pool[::stride][:per]:
                chosen[rel] = "breadth"
    # ⛔ (tier, path) — the manifest's own column order. `chosen` is keyed by path, so returning
    # `.items()` directly would emit (path, tier) and every row would be swapped; the first cut
    # did exactly that and wrote a 200-row manifest whose tier column held file paths.
    order = {"hot": 0, "lr": 1, "breadth": 2}
    return sorted(((tier, rel) for rel, tier in chosen.items()),
                  key=lambda tr: (order[tr[0]], tr[1]))


# ── reporting ───────────────────────────────────────────────────────────────────────────────

def write_entries_tsv(path: str, rows: list[tuple], tiers: dict[str, str]) -> None:
    """⛔ FAILED SPECULATION IS NOT A COLUMN. It is exactly `entries − committed`, and a field a
    reader can derive exactly is looked up, never stored (docs/DERIVED_STATE_CONTAINMENT.md
    R1/R3) — a stored copy is one more thing to drift out of agreement with its own inputs."""
    with open(path, "w", encoding="utf-8") as fh:
        fh.write("sub_corpus\ttier\tpath\taccepted\tentries\tcommitted\tmemo_hits"
                 "\tlr_entries\tlr_committed\n")
        for rel, accepted, ent, com, memo, lr_e, lr_c in rows:
            fh.write(f"{sub_corpus_of(rel)}\t{tiers.get(rel, '?')}\t{rel}\t{accepted}"
                     f"\t{ent}\t{com}\t{memo}\t{lr_e}\t{lr_c}\n")


def write_report(path: str, rows: list[tuple], ident: dict, nodump: list[str],
                 tiers: dict[str, str]) -> None:
    total = sum(r[2] for r in rows)
    committed = sum(r[3] for r in rows)
    memo = sum(r[4] for r in rows)
    lr_total = sum(r[5] for r in rows)
    lr_committed = sum(r[6] for r in rows)
    accepted = sum(1 for r in rows if r[1] == "yes")
    acc_entries = sum(r[2] for r in rows if r[1] == "yes")
    acc_committed = sum(r[3] for r in rows if r[1] == "yes")
    by_tier: dict[str, list[int]] = {}
    for row in rows:
        by_tier.setdefault(tiers.get(row[0], "?"), []).append(row[2])

    L = []
    A = L.append
    A("# SV Parse-Cost Ratchet — the BINDING baseline")
    A("")
    A("`ENGINE-UNIVERSAL-SERVICES.20` acceptance (d). Produced by")
    A("`stimuli/sv/corpus_parse_cost.py`; held by `scripts/check_parse_cost_ratchet.sh`.")
    A("")
    A("> ⛔ **This file is byte-compared by the gate.** It carries only values that are exact")
    A("> functions of the tree — no wall clock, no timestamp, no `HEAD`. That is not tidiness:")
    A("> a baseline whose bytes move on their own cannot be diffed, and a baseline nobody")
    A("> diffs is the staleness defect this leaf's own `.13i` is a record of.")
    A("")
    A("## What binds, and what it cannot see")
    A("")
    A("| metric | graph observed | machine-dependent | role |")
    A("|---|---|---|---|")
    A("| total rule entries | PROTOCOL (`parse_*`) | no | **BINDING** |")
    A("| bare-parse wall clock | FUSED (`cascade_*`) | yes | advisory, wide band |")
    A("")
    A("⛔ **The binding metric's blind spot, declared rather than discovered.** Requesting the")
    A("entry dump routes the parse to the PROTOCOL graph (TOOLBOX 3.4 ROUTING). A production")
    A("parse with no diagnostic consumer runs the FUSED `cascade_*` graph, which ticks no")
    A("per-rule counters — including the fused twins of the very rules this leaf is about")
    A("(`cascade_match_casting_type_lr_suffix`, `cascade_build_select_expression_lr_base`, …).")
    A("The +24.3 % that opened `.20` was measured on the fused graph. So this number guards")
    A("**structural work** and cannot, on its own, describe that cost. The wall-clock advisory")
    A("in `advisory.json` is the only view of the other graph, and it is machine-dependent —")
    A("which is why it advises and does not bind. **Neither metric alone is sufficient.**")
    A("")
    A("## Instrument identity (what produced this number)")
    A("")
    A("> Re-hash these three inputs. If any differs, **this baseline no longer describes your")
    A("> tree** and the honest act is to re-measure, not to quote. The gate re-hashes them on")
    A("> every run — unlike the six oracles `SV-CORPUS-GRAD.13i` found carrying an identity")
    A("> block that nothing read, four of which were measurably stale.")
    A("")
    A("| input | repo-root-relative path | sha256 |")
    A("|---|---|---|")
    for label in ("grammar", "generated_parser", "sample_inputs"):
        e = ident[label]
        A(f"| {label.replace('_', ' ')} | `{e['path']}` | `{e['sha256']}` |")
    A("")
    A("`sample inputs` digests the manifest ORDER plus every sampled file's bytes: the corpora")
    A("are git submodules, so a bump can move this measurement without touching one byte of")
    A("PGEN, and a parser hash alone would not notice.")
    A("")
    A("## The binding numbers")
    A("")
    A("Four exact integers. Each was verified deterministic across repeated release runs AND")
    A("byte-identical between the debug and release probes before being made binding.")
    A("")
    A("| quantity | value | what a rise means |")
    A("|---|---:|---|")
    A(f"| sample files measured | {len(rows)} | — |")
    A(f"| accepted / rejected | {accepted} / {len(rows) - accepted} | a correctness move, not a cost move |")
    A(f"| **rule entries** | **{total:,}** | more rule-method entries: structural work grew |")
    A(f"| **committed entries** | **{committed:,}** | more surviving work |")
    A(f"| **failed speculation** (`entries − committed`) | **{total - committed:,}** | more probing waste — the mechanism a GUARD spends through |")
    A(f"| **memo hits** | **{memo:,}** | memo behaviour moved |")
    A("")
    A(f"Failed speculation is **{100.0 * (total - committed) / total:.1f} %** of all rule entries in")
    A("this sample: the parse is overwhelmingly probing work, so a guard that probes more shows")
    A("up here long before it shows up in the raw entry count.")
    A("")
    A("⚠️ **`committed` is only meaningful for an ACCEPTED parse** (TOOLBOX 3.5) — a rejected parse")
    A("commits nothing durable, so its entries are ALL speculation by construction and it drags")
    A("the whole-sample ratio up. The accepted-only sub-total is published beside it so neither is")
    A(f"mistaken for the other: over the {accepted} accepted files, entries {acc_entries:,} and")
    A(f"committed {acc_committed:,} — {100.0 * (acc_entries - acc_committed) / acc_entries:.1f} %")
    A("failed speculation even where the parse succeeded.")
    A("")
    A("## The guarded admission's own family")
    A("")
    A("| quantity | value |")
    A("|---|---:|")
    A(f"| `*_lr_base` / `*_lr_suffix*` entries | {lr_total:,} |")
    A(f"| of those, committed | {lr_committed:,} |")
    A(f"| family share of all entries | {100.0 * lr_total / total:.3f} % |")
    A("")
    A("⭐ Published so `.20` acceptance (b)'s third A/B arm computes its delta straight off this")
    A("artifact instead of re-deriving it.")
    A("")
    A(f"⭐ The family commits **{lr_committed:,}** of its {lr_total:,} entries. The guarded")
    A("admission is, to three significant figures, **pure speculation**: it is entered, it probes,")
    A("it rolls back. That is the expected shape of a structural guard and it is stated here so a")
    A("later reader does not mistake the family's entry count for productive work.")
    A("")
    A("⛔⛔ **AND IT CARRIES A FINDING THAT BOUNDS THIS WHOLE INSTRUMENT.** Across the full")
    A("16 336-file corpus the family takes **0.681 %** of all rule entries. The flip's entry DELTA")
    A("is smaller still — the rules it replaced were themselves entered — so the entry count moved")
    A("**well under 1 %** while wall clock moved **+24.3 %**. ⇒ the binding metric is **at least")
    A("~35× less sensitive** to *this* regression than the advisory one. That is not a reason to")
    A("discard it: it catches structural growth EXACTLY and cannot be fooled by a busy machine. It")
    A("is a reason to state plainly what it does **not** prove — the +24.3 % is a rise in cost PER")
    A("entry, not in the NUMBER of entries, and no counter can see that. `.20` acceptance (a)'s")
    A("profile is what attributes it; this ratchet stops it growing further unwatched meanwhile.")
    A("")
    A("## Per tier")
    A("")
    A("| tier | files | entries | what it is for |")
    A("|---|---:|---:|---|")
    why = {
        "hot": "the heaviest files — where parse cost concentrates",
        "lr": "the heaviest guarded-admission files — the mechanism `.20` owns",
        "breadth": "a deterministic stride across the rest — so the ratchet is not blind elsewhere",
    }
    for tier in ("hot", "lr", "breadth"):
        if tier in by_tier:
            A(f"| `{tier}` | {len(by_tier[tier])} | {sum(by_tier[tier]):,} | {why[tier]} |")
    A("")
    if nodump:
        A("## ⚠️ Files that produced NO dump")
        A("")
        A("Reported, never dropped: a file the probe could not dump is a file this baseline does")
        A("not cover, and silently shrinking the sample is how a ratchet loosens itself.")
        A("")
        for rel in nodump:
            A(f"- `{rel}`")
        A("")
    A("_Per-file rows: `entries.tsv` — 9 columns (sub-corpus, tier, path, accepted, entries,")
    A("committed, memo_hits, lr_entries, lr_committed), sorted by (sub-corpus, path) so two runs")
    A("diff directly. Failed speculation is deliberately NOT a column: it is exactly")
    A("`entries − committed` and is derived, never stored._")
    with open(path, "w", encoding="utf-8") as fh:
        fh.write("\n".join(L) + "\n")


# ── modes ───────────────────────────────────────────────────────────────────────────────────

def run_census(probe: str, outdir: str, jobs: int) -> int:
    manifest = os.path.join(ROOT, CORPUS_MANIFEST)
    if not os.path.isfile(manifest):
        die(f"corpus manifest missing: {CORPUS_MANIFEST}")
    files = sorted({l.split("\t")[2] for l in open(manifest, encoding="utf-8") if l.strip()})
    files = [f for f in files if os.path.isfile(os.path.join(ROOT, f))]
    if not files:
        die("no corpus files found — the vendored corpora are git submodules; check them out")
    print(f"parse-cost: census over {len(files)} files at -j{jobs} ...", file=sys.stderr)
    rows, nodump = measure_entries(probe, files, jobs)
    os.makedirs(outdir, exist_ok=True)
    out = os.path.join(outdir, "census.tsv")
    with open(out, "w", encoding="utf-8") as fh:
        for rel, accepted, ent, _com, _memo, lr_e, _lr_c in rows:
            fh.write(f"{sub_corpus_of(rel)}\t{rel}\t{'True' if accepted == 'yes' else 'False'}"
                     f"\t{ent}\t{lr_e}\n")
    print(f"parse-cost: census -> {out} ({len(rows)} rows, {len(nodump)} no-dump)", file=sys.stderr)
    return 0


def run_measure(probe: str, outdir: str, manifest_path: str, jobs: int, repeats: int) -> int:
    pinned = read_manifest(manifest_path)
    tiers = {rel: tier for tier, rel in pinned}
    files = [rel for _t, rel in pinned]

    missing = [f for f in files if not os.path.isfile(os.path.join(ROOT, f))]
    if missing:
        die(f"{len(missing)} pinned sample file(s) are absent, e.g. {missing[0]}\n"
            f"  The corpora are git submodules. Check them out, or re-derive the sample:\n"
            f"    git submodule update --init --recursive")

    ident = identity(files)
    rows, nodump = measure_entries(probe, files, jobs)
    if not rows:
        die("every sampled file failed to produce a dump — refusing to publish an empty baseline")

    os.makedirs(outdir, exist_ok=True)
    write_entries_tsv(os.path.join(outdir, "entries.tsv"), rows, tiers)
    write_report(os.path.join(outdir, "cost.md"), rows, ident, nodump, tiers)

    advisory_files = [rel for tier, rel in pinned if tier == "hot"]
    adv = measure_wallclock(probe, advisory_files, repeats)
    adv["probe"] = os.path.relpath(probe, ROOT)
    adv["note"] = ("ADVISORY ONLY — machine-dependent, measured on the FUSED cascade_* graph "
                   "that the BINDING entry counters cannot see. Compared with a wide band; a "
                   "breach warns, never fails.")
    with open(os.path.join(outdir, "advisory.json"), "w", encoding="utf-8") as fh:
        json.dump(adv, fh, indent=2, sort_keys=True)
        fh.write("\n")

    total = sum(r[2] for r in rows)
    com = sum(r[3] for r in rows)
    print(f"parse-cost: {len(rows)} files — BINDING entries={total:,} committed={com:,} "
          f"failed_speculation={total - com:,} memo_hits={sum(r[4] for r in rows):,} "
          f"(lr-family {sum(r[5] for r in rows):,}); ADVISORY parse={adv['total_parse_ms']:.0f} ms "
          f"over {adv['files']} hot files (floor {adv['process_floor_ms']:.1f} ms/invocation)",
          file=sys.stderr)
    return 0


def main() -> int:
    _self_check()
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--outdir", default=None)
    ap.add_argument("--manifest", default=os.path.join(ROOT, DEFAULT_MANIFEST))
    ap.add_argument("--probe", default=None)
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--repeats", type=int, default=3,
                    help="median-of-K for the advisory wall clock (default 3)")
    ap.add_argument("--census", action="store_true", help="full-corpus entry census")
    ap.add_argument("--select", action="store_true", help="derive the sample manifest")
    ap.add_argument("--census-tsv", default=None)
    ap.add_argument("--out", default=None)
    ap.add_argument("--hot", type=int, default=40)
    ap.add_argument("--lr", type=int, default=40)
    ap.add_argument("--breadth", type=int, default=120)
    args = ap.parse_args()

    if args.select:
        if not args.census_tsv or not args.out:
            die("--select needs --census-tsv and --out")
        sample = select_sample(args.census_tsv, args.hot, args.lr, args.breadth)
        with open(args.out, "w", encoding="utf-8") as fh:
            fh.write("# stimuli/sv/parse_cost_sample.tsv — the PINNED parse-cost sample\n")
            fh.write("#\n")
            fh.write("# ENGINE-UNIVERSAL-SERVICES.20 acceptance (d). Two tab-separated fields:\n")
            fh.write("#   tier \\t repo-root-relative path\n")
            fh.write("#\n")
            fh.write("# Derived deterministically from a full-corpus entry census — re-derive with:\n")
            fh.write("#   python3 stimuli/sv/corpus_parse_cost.py --census --outdir <dir>\n")
            fh.write("#   python3 stimuli/sv/corpus_parse_cost.py --select \\\n")
            fh.write("#       --census-tsv <dir>/census.tsv --out stimuli/sv/parse_cost_sample.tsv\n")
            fh.write("#\n")
            fh.write(f"# tiers: hot={args.hot} (heaviest by entries) lr={args.lr} (heaviest by\n")
            fh.write(f"# guarded-admission entries) breadth={args.breadth} (deterministic stride).\n")
            fh.write("#\n")
            fh.write("# ⛔ PINNED, not re-derived per run. A sample that re-selects itself from the\n")
            fh.write("# tree it is measuring can shed the file that regressed and report an\n")
            fh.write("# improvement. Re-derivation is a deliberate, reviewed act.\n")
            for tier, rel in sample:
                fh.write(f"{tier}\t{rel}\n")
        print(f"parse-cost: wrote {len(sample)} rows -> {args.out}", file=sys.stderr)
        return 0

    probe = resolve_probe(args.probe)
    if args.census:
        if not args.outdir:
            die("--census needs --outdir")
        return run_census(probe, args.outdir, args.jobs)
    if not args.outdir:
        die("--outdir is required")
    return run_measure(probe, args.outdir, args.manifest, args.jobs, args.repeats)


if __name__ == "__main__":
    sys.exit(main())
