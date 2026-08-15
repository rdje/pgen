#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.22 (c) — RED probes for the no-dump REFUSAL.

The defect `.22` records is SILENCE: a corpus file the probe cannot dump used to be folded into a
count and the run continued. The fix is a declared roster plus a refusal — and a refusal nobody has
watched fire is not known to work (`docs/CLAIM_VERIFICATION.md` §6).

⛔ THIS PROBE DELIBERATELY DOES NOT RUN THE PATHOLOGICAL CORPUS FILE, and that is a measurement,
not a shortcut: `…/ExponTimeIfElseGen/dut.sv` peaks at **13.7 GB RSS within ONE SECOND** on a 24 GB
machine (sampled at 1 s intervals: 13735 / 8327 / 5367 MB — the profile SPIKES then falls, because
the growth is `Vec` reallocation, so no timeout makes it affordable). Making every reader of this
probe allocate 57 % of their RAM to re-derive a `dict` lookup would be a poor trade. ⇒ the roster
LOGIC is exercised here with a no-dump that costs nothing (a path that produces no dump instantly),
and the pathological file's own behaviour is measured by its neighbours in this directory —
`probe.sh` (the growth law) and the tracked roster row. One instrument, one job.

The real `parseability_probe`, the real `measure_one_entries` failure classification and the real
`adjudicate_nodump` are all exercised; only the *cause* of the no-dump is made cheap.

Usage:  python3 docs/tasks/artifacts/engine_universal_services/coverage_stack_blowup/nodump_probe.py
Output: .../coverage_stack_blowup/nodump_probe.txt
"""
import io
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", "..", "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "stimuli", "sv"))
os.chdir(ROOT)

import corpus_parse_cost as cpc  # noqa: E402

PATHOLOGICAL = "stimuli/sv/subs/Surelog/tests/ExponTimeIfElseGen/dut.sv"
# A path the probe cannot dump, instantly and for free. It is a no-dump for a DIFFERENT reason
# than the pathological file, which is useful: it also proves the classifier reports the reason it
# actually saw rather than a single hard-coded string.
CHEAP_NODUMP = "rust/target/e22/nodump_probe_absent.sv"

out = io.StringIO()


def emit(line=""):
    print(line)
    out.write(line + "\n")


def pick_control() -> str:
    """An ordinary corpus file — the GREEN control that keeps the REDs non-vacuous."""
    files, _listed = cpc.corpus_files()
    for rel in files:
        if rel != PATHOLOGICAL and 500 < os.path.getsize(os.path.join(ROOT, rel)) < 4000:
            return rel
    raise SystemExit("no control file available")


def arm(name, want_exit, roster_override, files, proves):
    """Run measure_entries + adjudicate_nodump over `files` and assert how it exits."""
    real_reader = cpc.read_nodump_roster
    if roster_override is not None:
        cpc.read_nodump_roster = lambda: roster_override
    code, rows, nodump = 0, [], []
    try:
        probe = cpc.resolve_probe(None)
        rows, nodump = cpc.measure_entries(probe, files, 2)
        cpc.adjudicate_nodump(nodump, "probe")
    except SystemExit as exc:
        code = exc.code if isinstance(exc.code, int) else 2
    finally:
        cpc.read_nodump_roster = real_reader
    ok = code == want_exit
    emit(f"  {'PASS' if ok else 'FAIL'} {name:<38} exit {code} (want {want_exit}) — {proves}")
    emit(f"       rows={len(rows)} nodump={[(n.path, n.reason) for n in nodump]}")
    return ok


def main() -> int:
    control = pick_control()
    os.makedirs(os.path.dirname(os.path.join(ROOT, CHEAP_NODUMP)), exist_ok=True)
    if os.path.exists(os.path.join(ROOT, CHEAP_NODUMP)):
        os.unlink(os.path.join(ROOT, CHEAP_NODUMP))

    emit("=" * 92)
    emit("ENGINE-UNIVERSAL-SERVICES.22 (c) — no-dump REFUSAL, RED probes")
    emit("=" * 92)
    emit(f"control:       {control}")
    emit(f"cheap no-dump: {CHEAP_NODUMP} (absent by construction)")
    emit()

    results = [
        arm("GREEN control alone", 0, None, [control],
            "an ordinary file dumps; the REDs below are not vacuous"),
        arm("RED undeclared drop", 2, {}, [control, CHEAP_NODUMP],
            "an undeclared drop REFUSES instead of shrinking the denominator"),
        arm("GREEN declared drop", 0, {CHEAP_NODUMP: "empty-dump"}, [control, CHEAP_NODUMP],
            "a DECLARED drop proceeds and says so — the roster is not a blanket refusal"),
        arm("RED roster names another file", 2, {"stimuli/sv/nowhere.sv": "timeout"},
            [control, CHEAP_NODUMP],
            "a roster row cannot cover a file it does not name"),
    ]

    # The tracked roster must actually carry the pathological file, since this probe does not run
    # it. Asserted here rather than assumed, so the two halves of the evidence stay joined.
    roster = cpc.read_nodump_roster()
    ok = roster.get(PATHOLOGICAL) == "timeout"
    results.append(ok)
    emit(f"  {'PASS' if ok else 'FAIL'} {'TRACKED roster declares the real file':<38} "
         f"{PATHOLOGICAL} -> {roster.get(PATHOLOGICAL)!r} (want 'timeout')")

    emit()
    emit(f"{sum(results)} passed, {len(results) - sum(results)} failed.")
    if all(results):
        emit("* the refusal has been OBSERVED firing, and observed NOT firing on a declared drop.")
    else:
        emit("X an arm did not behave as declared — the refusal is not proven.")
    with open(os.path.join(HERE, "nodump_probe.txt"), "w", encoding="utf-8") as fh:
        fh.write(out.getvalue())
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
