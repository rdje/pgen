#!/usr/bin/env python3
"""silent_success_sentinel_sweep.py — the measurement half of the
SILENT-SUCCESS SENTINEL gate (DONE-BAR.5c).

A silent success is a parse that returns Ok with ZERO diagnostics while
handing back a PLACEHOLDER node. Every "did it parse?" gate is green on such
a parse, so it needs its own instrument.

Two arms, both driven by
rust/test_data/grammar_quality/silent_success_sentinel_contract_v0.json:

  STATIC   the codegen-placeholder sentinels must be ABSENT from every
           shipped generated parser.
  DYNAMIC  each family's own stimuli proof surface is generated, parsed and
           AST-dumped; ANY sentinel reached is a defect.

Plus CALIBRATION arms that must reproduce pinned ground truth before any
verdict is offered -- a detector with no positive control cannot tell a
clean sweep from a blind one.

Emits one JSON object on stdout. The bash gate owns exit codes and summaries.
"""
import json
import os
import re
import subprocess
import sys

# ---------------------------------------------------------------- carriers

_SIMPLE = {"n": "\n", "r": "\r", "t": "\t", "0": "\0",
           "\\": "\\", "'": "'", '"': '"'}


def decode_rust_literal(lit):
    """Decode a Rust string literal.

    Python's ast.literal_eval is NOT a substitute: Rust spells a unicode
    escape \\u{HEX} and Python spells it \\uXXXX, so literal_eval rejects
    real generator output (measured: 2 of 25 regex samples). Feeding the
    parser mis-decoded bytes would surface as an 'unparseable sample'
    rather than as an extractor bug -- a blindness trap.
    """
    if not (lit.startswith('"') and lit.endswith('"')):
        raise ValueError(f"not a string literal: {lit[:60]}")
    body, out, i = lit[1:-1], [], 0
    while i < len(body):
        c = body[i]
        if c != "\\":
            out.append(c)
            i += 1
            continue
        i += 1
        if i >= len(body):
            raise ValueError("literal ends in a trailing backslash")
        e = body[i]
        if e in _SIMPLE:
            out.append(_SIMPLE[e])
            i += 1
        elif e == "x":
            out.append(chr(int(body[i + 1:i + 3], 16)))
            i += 3
        elif e == "u":
            j = body.index("}", i)
            out.append(chr(int(body[i + 2:j], 16)))
            i = j + 1
        elif e == "\n":                                  # line continuation
            i += 1
            while i < len(body) and body[i] in " \t\n\r":
                i += 1
        else:
            raise ValueError(f"unknown Rust escape \\{e}")
    return "".join(out)


def split_literals(body):
    """Split an array body into top-level Rust string literals, tracking
    escapes so a literal CONTAINING `",` cannot terminate the scan early."""
    lits, i, n = [], 0, len(body)
    while i < n:
        if body[i] != '"':
            i += 1
            continue
        j, esc = i + 1, False
        while j < n:
            if esc:
                esc = False
            elif body[j] == "\\":
                esc = True
            elif body[j] == '"':
                break
            j += 1
        lits.append(body[i:j + 1])
        i = j + 1
    return lits


def extract_samples(mod_path):
    src = open(mod_path, encoding="utf-8", errors="replace").read()
    m = re.search(r"pub const STIMULI: \[&str; (\d+)\] = \[(.*?)\n\];", src, re.S)
    if not m:
        return None, None
    return ([decode_rust_literal(l) for l in split_literals(m.group(2))],
            int(m.group(1)))


# ------------------------------------------------------------- measurement

class Sweeper:
    def __init__(self, root, contract, work):
        self.root, self.c, self.work = root, contract, work
        self.pipeline = os.path.join(root, "rust/target/debug/ast_pipeline")
        self.probe = os.path.join(root, "rust/target/debug/parseability_probe")
        roster = self.c["sentinel_roster"]
        self.codegen = roster["codegen_placeholder"]["literals"]
        self.runtime = roster["runtime_fallback"]["literals"]
        self.all_sentinels = self.codegen + self.runtime
        os.makedirs(work, exist_ok=True)

    def count_sentinels(self, text):
        return {s: text.count(s) for s in self.all_sentinels if text.count(s)}

    def parse_dump(self, grammar, text, tag, profile=None, entry_rule=None):
        """Parse one input and return (ok, sentinel_counts, detail)."""
        sf = os.path.join(self.work, f"{tag}.in")
        df = os.path.join(self.work, f"{tag}.json")
        with open(sf, "w", encoding="utf-8") as fh:
            fh.write(text)
        if os.path.exists(df):
            os.remove(df)
        cmd = [self.probe, "--parse-dump-ast", grammar, sf, df]
        if profile:
            cmd += ["--profile", profile]
        if entry_rule:
            cmd += ["--entry-rule", entry_rule]
        try:
            r = subprocess.run(cmd, capture_output=True, text=True, timeout=600)
        except subprocess.TimeoutExpired:
            return False, {}, "parse timed out"
        if r.returncode != 0 or not os.path.exists(df):
            return False, {}, (r.stderr or r.stdout).strip()[-200:]
        dump = open(df, encoding="utf-8", errors="replace").read()
        return True, self.count_sentinels(dump), ""

    # -- arm 1: the shipped artifacts ------------------------------------
    def static_arm(self):
        gen = os.path.join(self.root, "generated")
        # DERIVE the roster from every shipped .rs artifact, NOT from a
        # `*_parser.rs` suffix: generated/ebnf.rs is a shipped parser that does
        # not carry that suffix, and a suffix glob silently skipped its 123
        # <invalid_sequence_access> arms on this gate's first run.
        artifacts = sorted(f for f in os.listdir(gen)) if os.path.isdir(gen) else []
        parsers = [f for f in artifacts if f.endswith(".rs")]
        if not parsers:
            return {"status": "REFUSE_NO_GENERATED_ARTIFACTS",
                    "detail": "generated/ is untracked; a sweep over an empty "
                              "room would exit 0 having linted nothing"}
        found, present = {}, {}
        for f in parsers:
            text = open(os.path.join(gen, f), encoding="utf-8",
                        errors="replace").read()
            for s in self.codegen:
                n = text.count(s)
                if n:
                    found.setdefault(s, {})[f] = n
            for s in self.runtime:
                n = text.count(s)
                if n:
                    present.setdefault(s, 0)
                    present[s] += n
        return {"status": "OK", "parsers_scanned": len(parsers),
                "codegen_placeholder_violations": found,
                "runtime_fallback_arms_present": present}

    # -- arm 2: calibration ----------------------------------------------
    def calibration_arm(self):
        arms = []
        for fact in self.c["calibration"]["facts"]:
            ok, counts, detail = self.parse_dump(
                fact["grammar"], fact["input"], f"cal_{fact['id']}",
                fact.get("profile"), fact.get("entry_rule"))
            total = sum(counts.values())
            arms.append({
                "id": fact["id"], "parsed": ok,
                "observed_sentinels": total,
                "expected_sentinels": fact["expected_sentinels"],
                "match": ok and total == fact["expected_sentinels"],
                "detail": detail})
        return arms

    # -- arm 3: the dynamic sweep ----------------------------------------
    def sweep_family(self, fam):
        g = fam["grammar"]
        mod = os.path.join(self.work, f"{g}_stim.rs")
        # Read the raw-AST JSON, NOT the .ebnf: the .ebnf path needs ast_pipeline
        # built with --features ebnf_dual_run, which the standard build (including
        # the one `regenerate_generated_parsers` leaves behind) does NOT enable --
        # so an .ebnf-fed gate REFUSES right after a regeneration. generated/ is
        # already a hard precondition here, so this removes the ambient-build
        # dependency entirely.
        gen_input = fam.get("gen_input") or fam["ebnf"]
        cmd = [self.pipeline, os.path.join(self.root, gen_input),
               "--generate-stimuli-module",
               "--count", str(self.c["sweep"]["sample_count"]),
               "--seed", str(self.c["sweep"]["seed"]),
               "--output", mod, "--validate-parseability"]
        if fam.get("max_depth"):
            cmd += ["--max-depth", str(fam["max_depth"])]
        if fam.get("profile"):
            cmd += ["--grammar-profile", fam["profile"]]
        try:
            r = subprocess.run(cmd, capture_output=True, text=True, timeout=3600)
        except subprocess.TimeoutExpired:
            return {"grammar": g, "status": "REFUSE_GENERATION_TIMEOUT"}
        if r.returncode != 0:
            return {"grammar": g, "status": "REFUSE_GENERATION_FAILED",
                    "detail": (r.stderr or r.stdout).strip()[-300:]}
        try:
            samples, declared = extract_samples(mod)
        except Exception as exc:                      # noqa: BLE001
            return {"grammar": g, "status": "REFUSE_DECODE_ERROR",
                    "detail": str(exc)}
        if samples is None:
            return {"grammar": g, "status": "REFUSE_NO_STIMULI_ARRAY"}
        if not samples:
            return {"grammar": g, "status": "REFUSE_ZERO_SAMPLES"}
        if len(samples) != declared:
            return {"grammar": g, "status": "REFUSE_COUNT_MISMATCH",
                    "detail": f"extracted {len(samples)} vs declared {declared}"}

        ok_n = fail_n = 0
        totals, hits, failed = {}, [], []
        for i, s in enumerate(samples):
            ok, counts, detail = self.parse_dump(g, s, f"{g}_{i}",
                                                 fam.get("profile"))
            if not ok:
                fail_n += 1
                failed.append({"index": i, "detail": detail})
                continue
            ok_n += 1
            if counts:
                for k, v in counts.items():
                    totals[k] = totals.get(k, 0) + v
                hits.append({"index": i, "found": counts, "sample": s[:160]})

        res = {"grammar": g, "status": "OK", "samples": len(samples),
               "parsed_ok": ok_n, "parse_fail": fail_n,
               "sentinel_totals": totals, "sentinel_samples": len(hits),
               "examples": hits[:3]}
        # FAITHFULNESS ORACLE. --validate-parseability guarantees every emitted
        # sample is parser-accepted, so a shortfall here means THIS extractor
        # corrupted the sample -- not that the family has an unparseable one.
        # (Do NOT use the generator's `sample_successes`: that counts
        # GENERATION attempts. Measured: semantic_annotation reports 27/27
        # generated while validation reports 'accepted 25/25 ... 2 rejected
        # over 27 attempts'. Reading it as a parse count raised a false alarm.)
        if ok_n < len(samples):
            res["status"] = "REFUSE_EXTRACTOR_SUSPECT"
            res["detail"] = (f"--validate-parseability guarantees all "
                             f"{len(samples)} emitted samples parse; only "
                             f"{ok_n} parsed here")
            res["failed"] = failed[:5]
        return res

    def run(self):
        return {
            "contract_version": self.c["version"],
            "sample_count": self.c["sweep"]["sample_count"],
            "seed": self.c["sweep"]["seed"],
            "static": self.static_arm(),
            "calibration": self.calibration_arm(),
            "families": [self.sweep_family(f) for f in self.c["families"]],
            "honest_limit": self.c["honest_limit"],
        }


if __name__ == "__main__":
    root, contract_path, work = sys.argv[1], sys.argv[2], sys.argv[3]
    with open(contract_path, encoding="utf-8") as fh:
        contract = json.load(fh)
    print(json.dumps(Sweeper(root, contract, work).run(), indent=2))
