#!/usr/bin/env python3
"""K1c cross-build A/B: old (pre-K1c) vs new probe — verdict + AST byte-compare.

For every corpus cell and probe-set pattern: parse with both binaries using
--parse-dump-ast (compact), compare exit codes and AST bytes. Any mismatch is a
stop-and-revert signal per the -0138 acceptance instrument.
"""
import json, subprocess, sys, hashlib, os

# Run from the repo root (or set PGEN_ROOT). OLD = a preserved pre-change probe
# binary; NEW = the freshly rebuilt probe. Both paths are repo-root-relative.
ROOT = os.environ.get("PGEN_ROOT", ".")
SCRATCH = os.environ.get("AB_SCRATCH", ".")
OLD = os.path.join(SCRATCH, "probe_pre_k1c")
NEW = os.path.join(ROOT, "rust/target/debug/parseability_probe")
CORPUS = os.path.join(ROOT, "regex_corpus_bundle/corpus/pcre2/canonical/pcre2_compile_oracle_cases.jsonl")

PROBE_SET = [
    ("probe:a", "a"), ("probe:a_star", "a*"), ("probe:a_lazy", "a+?"),
    ("probe:group", "(x)"), ("probe:group_counted", "(x){2,3}"),
    ("probe:lookahead_star", "a(?=b)*"), ("probe:absorb_E", "x\\E*"),
    ("probe:absorb_QE", "a\\Q\\E*"), ("probe:standalone_QE", "\\Q\\E"),
    ("probe:stray_E", "a\\E"), ("probe:nest3", "(((x)))"),
    ("probe:alt", "a|b"), ("probe:class", "[a-z]+"),
    ("probe:backref", "(a)\\1"), ("probe:named", "(?<n>a)\\k<n>"),
    ("probe:absorb_multi", "x\\E\\E+"), ("probe:reject_E_star", "\\E*"),
    ("probe:reject_QE_star", "\\Q\\E*"),
    ("probe:nest_deep", "(" * 12 + "x" + ")" * 12),
]

def cell_iter():
    for cid, pat in PROBE_SET:
        yield cid, pat
    with open(CORPUS) as f:
        for line in f:
            d = json.loads(line)
            yield d["id"], d["pattern"]

def run(binary, pat_file, out_file):
    try:
        r = subprocess.run([binary, "--parse-dump-ast", "regex", pat_file, out_file],
                           capture_output=True, timeout=30)
        rc = r.returncode
    except subprocess.TimeoutExpired:
        return ("TIMEOUT", b"")
    ast = b""
    if rc == 0 and os.path.exists(out_file):
        with open(out_file, "rb") as f:
            ast = f.read()
    return (rc, ast)

def main():
    pat_file = os.path.join(SCRATCH, "ab_pat.txt")
    out_old = os.path.join(SCRATCH, "ab_old.json")
    out_new = os.path.join(SCRATCH, "ab_new.json")
    n = accept = reject = mismatch = 0
    mismatches = []
    for cid, pat in cell_iter():
        n += 1
        with open(pat_file, "wb") as f:
            f.write(pat.encode("utf-8"))
        for p in (out_old, out_new):
            if os.path.exists(p):
                os.remove(p)
        rc_o, ast_o = run(OLD, pat_file, out_old)
        rc_n, ast_n = run(NEW, pat_file, out_new)
        ok_o, ok_n = rc_o == 0, rc_n == 0
        if ok_o != ok_n or (ok_o and ast_o != ast_n) or rc_o == "TIMEOUT" or rc_n == "TIMEOUT":
            mismatch += 1
            mismatches.append({"id": cid, "rc_old": str(rc_o), "rc_new": str(rc_n),
                               "ast_old_sha": hashlib.sha256(ast_o).hexdigest()[:12],
                               "ast_new_sha": hashlib.sha256(ast_n).hexdigest()[:12]})
        if ok_n:
            accept += 1
        else:
            reject += 1
        if n % 250 == 0:
            print(f"progress {n} accept={accept} reject={reject} mismatch={mismatch}", flush=True)
    print(f"DONE total={n} accept={accept} reject={reject} MISMATCH={mismatch}")
    with open(os.path.join(SCRATCH, "ab_result.json"), "w") as f:
        json.dump({"total": n, "accept": accept, "reject": reject,
                   "mismatch": mismatch, "mismatches": mismatches}, f, indent=1)

if __name__ == "__main__":
    main()
