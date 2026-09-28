#!/usr/bin/env bash
# scripts/check_engine_neutrality.sh — DOCTRINE `ENGINE-NEUTRALITY` (structural; ENGINE-UNIVERSAL-SERVICES.4, 2026-09-28)
#
#   The engine is EBNF-, grammar- and parser-NEUTRAL: nothing in it knows any language it parses.
#
# ⛔ WHY THIS EXISTS — the director, 2026-09-28, on learning the tree was opened by "two leaves that found the
# engine/grammar boundary being violated silently": "There must be guards, checks, monitorings that should from
# now on prevent that from happening ever again … EBNF or parser neutrality and agnosticity is the whole point
# of PGEN, break that and you break the whole project." The boundary (ENGINE-UNIVERSAL-SERVICES, the frame):
# a concern belongs to the ENGINE iff it is objectively shared by all EBNFs; it belongs to the GRAMMAR iff it is
# a fact about one language. Through 1.0.246 no gate read for the second half of that — an engine source could
# name a grammar's profile, family or rule and every doctrine stayed green. Measured on 7ac640e94: three universal
# help / lint strings used `sv_2017` / `verilog_2005` / `systemverilog` as their examples; the embedding API typed
# the families and their profiles by hand (37 tokens); the SystemVerilog preprocessor and its keyword tables lived
# inside the engine crate. The first three are fixed by this doctrine's commit; the other two are ratcheted here
# (their counts may only FALL) and owned by the leaves that remove them.
#
# WHAT IT BINDS, on every run (structural, ~1 s, reads the tree, mutates nothing):
#   1. THE FORBIDDEN VOCABULARY IS DERIVED, never typed: every grammar under grammars/*.ebnf is classed in the
#      register as `language` (a parser PGEN builds for users) or `meta` (the engine's own languages: ebnf,
#      semantic_annotation, return_annotation, and the example grammars). The language grammars' NAMES, their
#      `@profiles` values, their `kw_*_<hash>` tokens and their multi-word rule names (>= 8 chars with `_`) are
#      forbidden; the meta grammars' own rule / token names are engine vocabulary by construction and are subtracted.
#      A grammar with no class row FAILS; a row naming no grammar FAILS (closed, two-sided).
#   2. ENGINE sources — every `.rs` under rust/src not classed otherwise — carry NONE of it in production code
#      (outside `#[cfg(test)]` / `mod tests` regions and outside comments). Any hit is a breach naming file:line
#      and the token. The allow-list is EMPTY by design; a non-empty row needs `reason` and `owner_leaf`.
#   3. The NON-ENGINE classes are a closed population with a RATCHET: `per-family` files (a language-specific
#      component that may name its OWN family only), `registry` (lib.rs, build.rs, parser_registry.rs: the derived
#      enumeration of registered parsers), `api` (embedding_api.rs), `bin` (rust/src/bin/*, main.rs) and `harness`
#      (parse-harness suites and censuses). Each carries a ceiling of production-code hits in the register; a RISE
#      fails, a FALL must be promoted deliberately (`--rebaseline`), and every non-zero ceiling names the leaf that
#      owes its zero. A classed file that does not exist FAILS; a file no class covers is `engine`.
#   4. `--self-test` drives every refusal on synthetic inputs (seven arms) and runs FIRST in the gate — a guard
#      nobody has seen refuse is indistinguishable from one that cannot.
#
# Usage:  bash scripts/check_engine_neutrality.sh            # the gate (self-test, then the tree)
#         bash scripts/check_engine_neutrality.sh --census   # the full census, every hit by class / position / kind
#         bash scripts/check_engine_neutrality.sh --self-test
#         bash scripts/check_engine_neutrality.sh --rebaseline   # promote FALLEN ceilings into the register
# Exit:   0 holds · 1 breach · 2 usage / the register or the tree cannot be read
set -uo pipefail
ROOT="${PGEN_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
cd "$ROOT" || exit 2
exec python3 - "$@" <<'PY'
import glob, json, os, re, sys, tempfile, collections
TAG = "engine-neutrality"
REGISTER = "rust/test_data/grammar_quality/engine_neutrality_register_v0.json"
KINDS = ("family", "profile", "keyword-rule", "rule")

def fail(msg, rc=1):
    print(f"{TAG}: ✗ {msg}", file=sys.stderr); return rc

def load_register(path):
    with open(path, encoding="utf-8") as fh: return json.load(fh)

def derive_vocabulary(grammar_dir, reg):
    """The forbidden vocabulary from the grammars on disk, classed by the register (closed, two-sided)."""
    on_disk = {os.path.basename(g)[:-5]: g for g in glob.glob(os.path.join(grammar_dir, "*.ebnf"))}
    classes = reg["grammar_classes"]
    problems = []
    for n in sorted(on_disk):
        if n not in classes: problems.append(f"grammar `{n}` has NO class row in {REGISTER} (language or meta) — an unclassed grammar cannot be guarded")
    for n in sorted(classes):
        if n not in on_disk: problems.append(f"{REGISTER} classes grammar `{n}`, which does not exist under {grammar_dir}/ — a dead row")
    def names(paths):
        out = set()
        for g in paths:
            t = open(g, encoding="utf-8", errors="replace").read()
            out |= set(re.findall(r"^([A-Za-z_][A-Za-z0-9_]*)\s*:=", t, re.M))
        return out
    language = {n: g for n, g in on_disk.items() if classes.get(n) == "language"}
    meta = {n: g for n, g in on_disk.items() if classes.get(n) == "meta"}
    meta_names = names(meta.values()) | set(reg.get("meta_token_types", []))
    profiles = set()
    for g in language.values():
        t = open(g, encoding="utf-8", errors="replace").read()
        for m in re.findall(r"@profiles:\s*\[([^\]]*)\]", t):
            profiles |= {x.strip().strip('"') for x in m.split(",") if x.strip()}
    rules = {r for r in names(language.values()) if not r.startswith("kw_") and len(r) >= 8 and "_" in r} - meta_names
    return dict(families=sorted(language), profiles=sorted(profiles), rules=rules, problems=problems)

def scan(files, vocab, classify):
    fam_rx = re.compile(r"\b(" + "|".join(map(re.escape, sorted(vocab["families"], key=len, reverse=True))) + r")\b") if vocab["families"] else None
    prof_rx = re.compile(r"\b(" + "|".join(map(re.escape, sorted(vocab["profiles"], key=len, reverse=True))) + r")\b") if vocab["profiles"] else None
    kw_rx = re.compile(r"\bkw_[a-z0-9_]+_[0-9a-f]{8}\b")
    rows = []
    for f in files:
        lines = open(f, encoding="utf-8", errors="replace").read().split("\n")
        test_start = next((i for i, l in enumerate(lines) if l.startswith("#[cfg(test)]") or l.startswith("mod tests")), len(lines))
        cls, own = classify(f)
        f = classify.rel(f)   # rows carry the repo-relative path: the register, the ceilings and the messages all speak it
        for i, l in enumerate(lines):
            pos = "test" if i >= test_start else ("comment" if l.lstrip().startswith("//") else "code")
            hits = [("family", m.group(1)) for m in fam_rx.finditer(l)] if fam_rx else []
            if prof_rx: hits += [("profile", m.group(1)) for m in prof_rx.finditer(l)]
            hits += [("keyword-rule", m.group(0)) for m in kw_rx.finditer(l)]
            hits += [("rule", m.group(1)) for m in re.finditer(r'"([a-z][a-z0-9_]{7,})"', l) if m.group(1) in vocab["rules"]]
            for kind, tok in hits:
                if cls == "per-family" and kind == "family" and tok == own: continue   # a component may name its OWN family
                rows.append(dict(file=f, cls=cls, pos=pos, line=i + 1, kind=kind, token=tok, text=l.strip()[:110]))
    return rows

def make_classifier(reg, root):
    classed = {}
    for cls, entries in reg["file_classes"].items():
        for e in entries:
            path = e["path"] if isinstance(e, dict) else e
            classed[path] = (cls, e.get("family") if isinstance(e, dict) else None)
    def rel(f): return os.path.relpath(f, root) if os.path.isabs(f) else f
    def classify(f): return classed.get(rel(f), ("engine", None))
    classify.rel = rel
    return classify, classed

def verdict(root, reg, files, print_census=False, rebaseline=False):
    vocab = derive_vocabulary(os.path.join(root, "grammars"), reg)
    problems = list(vocab["problems"])
    classify, classed = make_classifier(reg, root)
    for path in classed:
        if not os.path.exists(os.path.join(root, path)): problems.append(f"{REGISTER} classes `{path}`, which does not exist — a dead row")
    rows = scan(files, vocab, classify)
    if print_census:
        c = collections.Counter((r["cls"], r["pos"], r["kind"]) for r in rows)
        print(f"{TAG} census: families={len(vocab['families'])} profiles={vocab['profiles']} rules={len(vocab['rules'])} files={len(files)}")
        for k, n in sorted(c.items()): print("  ", k, n)
    # 2. the engine carries nothing
    engine_hits = [r for r in rows if r["cls"] == "engine" and r["pos"] == "code"]
    allow = {(a["file"], a["token"]) for a in reg.get("engine_allow", [])}
    for a in reg.get("engine_allow", []):
        if not a.get("reason") or not a.get("owner_leaf"): problems.append(f"engine_allow row {a.get('file')}:{a.get('token')} carries no reason / owner_leaf — an allow-list row is a debt with an owner or it is nothing")
    live = [r for r in engine_hits if (r["file"], r["token"]) not in allow]
    for r in live:
        problems.append(f"ENGINE source {r['file']}:{r['line']} names the {r['kind']} `{r['token']}` of a language grammar in production code — the engine is neutral; move the fact to the grammar (or the per-family component) | {r['text']}")
    # 3. the non-engine ratchet
    counts = collections.Counter((r["file"], ) for r in rows if r["cls"] != "engine" and r["pos"] == "code")
    ceilings = reg["ceilings"]
    fell = []
    for path, (cls, _) in sorted(classed.items()):
        n = counts.get((path,), 0); c = ceilings.get(path)
        if c is None:
            if n: problems.append(f"`{path}` ({cls}) carries {n} language token(s) in production code and NO ceiling row — name it, with the leaf that owes its zero")
            continue
        if n > c["ceiling"]: problems.append(f"`{path}` ({cls}) ROSE {c['ceiling']} -> {n} language tokens in production code — the class only falls; the leaf that owes its zero is {c.get('owner_leaf', '?')}")
        elif n < c["ceiling"]: fell.append((path, c["ceiling"], n))
    for path in ceilings:
        if path not in classed: problems.append(f"ceiling row `{path}` names a file no class covers — a dead row")
    if fell and not rebaseline:
        for path, old, new in fell: problems.append(f"`{path}` FELL {old} -> {new} — an improvement; promote it deliberately: bash scripts/check_engine_neutrality.sh --rebaseline")
    if rebaseline and fell:
        for path, old, new in fell: ceilings[path]["ceiling"] = new
        with open(os.path.join(root, REGISTER), "w", encoding="utf-8") as fh: json.dump(reg, fh, indent=2, ensure_ascii=False); fh.write("\n")
        print(f"{TAG}: rebaselined {len(fell)} ceiling(s): " + ", ".join(f"{p} {o}->{n}" for p, o, n in fell))
    if problems:
        for p in problems: fail(p)
        print(f"{TAG}: {len(problems)} breach(es)", file=sys.stderr); return 1
    nz = sum(1 for c in ceilings.values() if c["ceiling"] > 0)
    print(f"{TAG}: OK (engine: 0 language tokens in production code over {sum(1 for f in files if classify(f)[0] == 'engine')} files; "
          f"{len(classed)} classed non-engine files at or under their ceilings, {nz} still non-zero and owned; vocabulary derived: "
          f"{len(vocab['families'])} language grammars, {len(vocab['profiles'])} profiles, {len(vocab['rules'])} rules)")
    return 0

def tree_files(root):
    return sorted(glob.glob(os.path.join(root, "rust/src/**/*.rs"), recursive=True)) + [os.path.join(root, "rust/build.rs")]

def self_test():
    """Seven arms on synthetic inputs: every refusal is driven for real."""
    arms = []
    with tempfile.TemporaryDirectory(dir=os.path.join(ROOT, "rust/target") if os.path.isdir(os.path.join(ROOT, "rust/target")) else None) as d:
        os.makedirs(os.path.join(d, "grammars")); os.makedirs(os.path.join(d, "rust/src/bin")); os.makedirs(os.path.join(d, "rust/test_data/grammar_quality"))
        open(os.path.join(d, "grammars/lang.ebnf"), "w").write('@profiles: ["lp_2017"]\nlang_top_rule := "x"\nkw_thing_deadbeef := "y"\n')
        open(os.path.join(d, "grammars/ebnf.ebnf"), "w").write('quoted_string := "q"\n')
        open(os.path.join(d, "rust/src/sv_thing.rs"), "w").write('fn f() { let a = "lang"; }\n')
        open(os.path.join(d, "rust/src/lib.rs"), "w").write('pub mod lang;\n')
        open(os.path.join(d, "rust/build.rs"), "w").write('fn main() {}\n')
        base_reg = {"grammar_classes": {"lang": "language", "ebnf": "meta"}, "meta_token_types": ["quoted_string"],
                    "file_classes": {"per-family": [{"path": "rust/src/sv_thing.rs", "family": "lang"}], "registry": ["rust/src/lib.rs", "rust/build.rs"]},
                    "ceilings": {"rust/src/lib.rs": {"ceiling": 1, "owner_leaf": "T"}, "rust/build.rs": {"ceiling": 0, "owner_leaf": "T"}, "rust/src/sv_thing.rs": {"ceiling": 0, "owner_leaf": "T"}},
                    "engine_allow": []}
        def run(reg, extra=None, rebaseline=False):
            files = tree_files(d)
            import io, contextlib
            err = io.StringIO()
            with contextlib.redirect_stderr(err), contextlib.redirect_stdout(io.StringIO()):
                rc = verdict(d, json.loads(json.dumps(reg)), files, rebaseline=rebaseline)
            return rc, err.getvalue()
        def arm(label, want, reg, plant=None):
            if plant: open(os.path.join(d, plant[0]), "w").write(plant[1])
            rc, err = run(reg)
            if plant: os.remove(os.path.join(d, plant[0]))
            ok = rc == want; arms.append(ok); print(f"  {'✅' if ok else '❌'} {label} (rc {rc}, wanted {want})" + ("" if ok else "\n     " + err.strip()[:300]))
        arm("GREEN: a neutral engine file", 0, base_reg, ("rust/src/engine.rs", 'fn g() { let s = "nothing"; }\n'))
        arm("RED: an engine file names a language grammar in code", 1, base_reg, ("rust/src/engine.rs", 'fn g() { let s = "lang"; }\n'))
        arm("RED: an engine file names a profile in a help string", 1, base_reg, ("rust/src/engine.rs", 'const U: &str = "for example lp_2017";\n'))
        arm("GREEN: the same token inside #[cfg(test)]", 0, base_reg, ("rust/src/engine.rs", 'fn g() {}\n#[cfg(test)]\nmod tests { const P: &str = "lp_2017"; }\n'))
        arm("GREEN: the same token in a comment", 0, base_reg, ("rust/src/engine.rs", '// lang lp_2017 kw_thing_deadbeef\nfn g() {}\n'))
        reg2 = json.loads(json.dumps(base_reg)); reg2["file_classes"]["per-family"] = [{"path": "rust/src/sv_thing.rs", "family": "other"}]; reg2["grammar_classes"]["other"] = "language"
        open(os.path.join(d, "grammars/other.ebnf"), "w").write('other_top_rule := "z"\n')
        arm("RED: a per-family component names ANOTHER language family", 1, reg2)
        os.remove(os.path.join(d, "grammars/other.ebnf"))
        reg3 = json.loads(json.dumps(base_reg)); reg3["ceilings"]["rust/src/lib.rs"]["ceiling"] = 0
        arm("RED: a non-engine file ROSE above its ceiling", 1, reg3)
        reg4 = json.loads(json.dumps(base_reg)); del reg4["grammar_classes"]["ebnf"]
        arm("RED: a grammar with no class row", 1, reg4)
        reg5 = json.loads(json.dumps(base_reg)); reg5["file_classes"]["registry"].append("rust/src/missing.rs")
        arm("RED: a classed file that does not exist", 1, reg5)
        reg6 = json.loads(json.dumps(base_reg)); reg6["engine_allow"] = [{"file": "rust/src/engine.rs", "token": "lang"}]
        arm("RED: an allow-list row without reason / owner", 1, reg6, ("rust/src/engine.rs", 'fn g() { let s = "lang"; }\n'))
    n = len(arms); ok = sum(arms)
    print(f"{TAG} --self-test: arms={n} failed={n - ok}")
    return 0 if ok == n else 1

mode = sys.argv[1] if len(sys.argv) > 1 else ""
if mode not in ("", "--census", "--self-test", "--rebaseline"):
    print(f"usage: bash scripts/check_engine_neutrality.sh [--census | --self-test | --rebaseline]", file=sys.stderr); sys.exit(2)
ROOT = os.getcwd()
if mode == "--self-test": sys.exit(self_test())
if self_test() != 0: sys.exit(fail("the self-test FAILED — a guard that cannot refuse is not a guard", 1))
try: reg = load_register(os.path.join(ROOT, REGISTER))
except Exception as e: sys.exit(fail(f"cannot read {REGISTER}: {e}", 2))
sys.exit(verdict(ROOT, reg, tree_files(ROOT), print_census=(mode == "--census"), rebaseline=(mode == "--rebaseline")))
PY
