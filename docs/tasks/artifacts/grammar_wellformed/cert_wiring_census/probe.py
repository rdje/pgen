#!/usr/bin/env python3
"""GRAMMAR-WELLFORMED.H.15 / H.18 — the two registry censuses, as ONE re-runnable instrument.

Both answer a question about the CLOSED population of grammar families, and both were originally
computed by hand — which is how three families spent two months outside the certificate-coverage
instrument (H.15) and nine registry rows printed a false "no detail-capable parser registered"
label (H.18). A count that is re-derived by eye is a count that rots.

  CERT-WIRED-CENSUS   is every family in `done_bar_family_register_v0.json` cert-coverage WIRED?
                      (i.e. does its `GENERATED_PARSER_REGISTRY` row carry `parse_and_cover`?)
  LABEL-BLIND-CENSUS  does any registry row lack a detail hook while a WORKING detail arm exists
                      in the other dispatch? (the H.18 defect, kept as a tripwire after the fix)

Both read only TRACKED TEXT — the register JSON and the registry source — so this needs no cargo
build, no generated parser and no parse run. It is deliberately a *reader*, never an authority: the
authority is `ast_pipeline --report-certificate-coverage`, which refuses by name on an unwired
grammar. This instrument exists so the population question can be asked cheaply and repeatedly.

⛔ It is NOT registered in `DIAGNOSIS_SIG` (`scripts/check_diagnosis_evidence.sh`): a token is added
only for an instrument that PRODUCES a root-cause diagnosis, and this one sizes a population. The
diagnosing tool for both leaves is `--report-certificate-coverage`.

Usage:   python3 docs/tasks/artifacts/grammar_wellformed/cert_wiring_census/probe.py
Exit:    0 = both censuses clean · 1 = a family is unwired or a row is label-blind · 2 = harness
         refused (an input could not be read, so NOTHING was scored — never conflate with a pass)
"""

import json
import os
import re
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "..", "..", ".."))
REGISTER = os.path.join(ROOT, "rust/test_data/grammar_quality/done_bar_family_register_v0.json")
REGISTRY = os.path.join(ROOT, "rust/src/parser_registry.rs")


def refuse(msg):
    print("CERT-WIRING-CENSUS: REFUSED — %s (nothing was scored)" % msg, file=sys.stderr)
    sys.exit(2)


def main():
    for path in (REGISTER, REGISTRY):
        if not os.path.isfile(path):
            refuse("missing input %s" % os.path.relpath(path, ROOT))

    with open(REGISTER, encoding="utf-8") as fh:
        families = sorted(json.load(fh).get("families", {}))
    if not families:
        refuse("the register declares no families")

    with open(REGISTRY, encoding="utf-8") as fh:
        src = fh.read()

    try:
        table = src[src.index("static GENERATED_PARSER_REGISTRY"):]
        table = table[:table.index("\n];")]
    except ValueError:
        refuse("could not locate GENERATED_PARSER_REGISTRY in %s" % os.path.relpath(REGISTRY, ROOT))

    # A row is cert-WIRED iff its entry carries a `parse_and_cover: Some(...)`. Comment lines may
    # sit between the fields, so they are skipped rather than assumed absent.
    rows = re.findall(
        r'grammar_name:\s*"([a-z_0-9]+)",\s*\n'
        r'(?:\s*(?://[^\n]*)?\n)*?'
        r'\s*parse_sample:[^\n]*\n'
        r'(?:\s*//[^\n]*\n)*'
        r'\s*parse_and_cover:\s*(Some|None)',
        table,
    )
    if not rows:
        refuse("parsed 0 registry rows — the table's shape changed and this scan is stale")
    wired = {name for name, kind in rows if kind == "Some"}
    known = {name for name, _ in rows}

    unwired = [f for f in families if f not in wired]
    print(
        "CERT-WIRED-CENSUS: register_families=%d wired=%d unwired=%d missing=%s"
        % (len(families), len(families) - len(unwired), len(unwired), unwired)
    )
    for f in families:
        print(
            "  %-30s in_registry=%-4s cert_wired=%s"
            % (f, "yes" if f in known else "NO", "yes" if f in wired else "NO")
        )

    # H.18 tripwire: `parse_error()` must not re-acquire a private detail table that can drift from
    # `parse_sample_detail_with_profile`. A `parse_detail` field re-appearing IS the regression.
    blind = sorted(re.findall(r"parse_detail:\s*None", table))
    dispatch = re.search(
        r"pub fn parse_sample_detail_with_profile.*?\n\}\n", src, re.S
    )
    arms = set(re.findall(r'"([a-z_0-9]+)" =>', dispatch.group(0))) if dispatch else set()
    if dispatch is None:
        refuse("could not locate parse_sample_detail_with_profile — this scan is stale")
    print(
        "LABEL-BLIND-CENSUS: registry_rows=%d parse_detail_fields=%d detail_dispatch_arms=%d"
        % (len(rows), len(blind), len(arms))
    )
    if blind:
        print(
            "  ⛔ the deleted `parse_detail` table has re-appeared — H.18's divergence is back",
            file=sys.stderr,
        )

    rc = 1 if (unwired or blind) else 0
    print("CERT-WIRING-CENSUS: %s" % ("CLEAN" if rc == 0 else "BREACH"))
    return rc


if __name__ == "__main__":
    sys.exit(main())
