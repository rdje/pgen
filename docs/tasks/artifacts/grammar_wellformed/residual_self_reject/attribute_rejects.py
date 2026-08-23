#!/usr/bin/env python3
"""GRAMMAR-WELLFORMED.H.16.6c — ATTRIBUTE every residual self-rejected stimulus BY TOOL.

The leaf opened with an arrow-vs-other split made BY READING. This instrument replaces
that reading with two independent mechanical attributions per input:

  (1) STRUCTURAL SHRINK — a delta-debug (ddmin) over the value's top-level, bracket- and
      quote-aware comma-separated ELEMENTS, with `ast_pipeline --interpret-parse` as the
      oracle. The prefix `@ name :` and the value's own brackets are held FIXED, so every
      candidate stays a well-formed annotation and the shrink cannot collapse to garbage
      the way a character-level ddmin would. The result is the minimal element set that
      still rejects. ⇒ `arrow_in_minimal_core` is then an OBSERVATION, not a reading.

  (2) RULE-STACK ATTRIBUTION — the deepest `rule_stack` the generated parser reports at
      the input's `furthest_position`, read off a `PGEN_TRACE_VERBOSITY=debug --trace` run
      of the SHIPPED parser. Names the rule that actually ran out of alternatives.

Both oracles are recorded per input and any disagreement between the interpreter and the
shipped parser is a hard error, not a silent choice of oracle.

⛔ Read-only: no codegen, no regeneration, no grammar byte moves.
usage: attribute_rejects.py <grammar.ebnf> <corpus.txt> [--out DIR]
"""
import json, pathlib, re, subprocess, sys

ROOT = pathlib.Path(__file__).resolve().parents[5]
AST = ROOT / "rust/target/debug/ast_pipeline"
PROBE = ROOT / "rust/target/release/parseability_probe"
WORK = ROOT / "rust/target/h1666c/attrib"

args = [a for a in sys.argv[1:] if not a.startswith("--")]
GRAMMAR = pathlib.Path(args[0])
CORPUS = pathlib.Path(args[1])
OUT = None
for a in sys.argv[1:]:
    if a.startswith("--out"):
        OUT = pathlib.Path(a.split("=", 1)[1])

for p in (AST, PROBE, GRAMMAR, CORPUS):
    if not p.exists():
        sys.exit(f"attribute_rejects: REFUSED — missing {p}")
WORK.mkdir(parents=True, exist_ok=True)

_probe_file = WORK / "probe.txt"


def interp(text):
    """(accepted, furthest_position) from the code-disjoint interpreter."""
    _probe_file.write_text(text)
    r = subprocess.run([str(AST), str(GRAMMAR), "--interpret-parse", str(_probe_file)],
                       capture_output=True, text=True)
    if "INTERPRET-PARSE:" not in r.stdout:
        return None, None
    m = re.search(r"furthest_position=(\d+)", r.stdout)
    return ("accepted=true" in r.stdout), (int(m.group(1)) if m else None)


def shipped(text):
    """(accepted, furthest_position) from the SHIPPED generated parser."""
    _probe_file.write_text(text)
    r = subprocess.run([str(PROBE), "--parse", GRAMMAR.stem, str(_probe_file)],
                       capture_output=True, text=True)
    blob = r.stdout + r.stderr
    m = re.search(r"furthest_position=(\d+)", blob)
    return (r.returncode == 0), (int(m.group(1)) if m else None)


def deepest_rule_stack(text):
    """The longest `rule_stack` the shipped parser reports, and every rule named at all."""
    _probe_file.write_text(text)
    r = subprocess.run([str(PROBE), "--parse", GRAMMAR.stem, str(_probe_file), "--trace"],
                       capture_output=True, text=True,
                       env={**__import__("os").environ, "PGEN_TRACE_VERBOSITY": "debug"})
    blob = r.stdout + r.stderr
    best, seen = [], set()
    for m in re.finditer(r'rule_stack:\s*\[([^\]]*)\]', blob):
        stack = [s.strip().strip('"') for s in m.group(1).split(",") if s.strip()]
        seen.update(stack)
        if len(stack) > len(best):
            best = stack
    for m in re.finditer(r"Entering branch \d+/\d+ for rule '([^']+)'", blob):
        seen.add(m.group(1))
    return best, sorted(seen)


# ------------------------------------------------------- structural element splitter
OPEN, CLOSE = "([{", ")]}"


def split_top_level(body):
    """Split on top-level commas, honouring brackets, quotes and backslash escapes."""
    parts, buf, depth, quote, esc = [], [], 0, None, False
    for ch in body:
        if esc:
            buf.append(ch); esc = False; continue
        if ch == "\\":
            buf.append(ch); esc = True; continue
        if quote:
            buf.append(ch)
            if ch == quote:
                quote = None
            continue
        if ch in "\"'`":
            quote = ch; buf.append(ch); continue
        if ch in OPEN:
            depth += 1
        elif ch in CLOSE:
            depth -= 1
        if ch == "," and depth == 0:
            parts.append("".join(buf)); buf = []; continue
        buf.append(ch)
    parts.append("".join(buf))
    return parts


VALUE_RE = re.compile(r"^(\s*@\s*\w+\s*:\s*)(.*)$", re.S)


def decompose(line):
    """(prefix, open_delim, elements, close_delim) or None when the value is not a collection."""
    m = VALUE_RE.match(line)
    if not m:
        return None
    prefix, value = m.group(1), m.group(2)
    v = value.rstrip()
    trail = value[len(v):]
    for o, c in (("#{", "}"), ("{", "}"), ("[", "]"), ("(", ")")):
        if v.startswith(o) and v.endswith(c):
            return prefix, o, split_top_level(v[len(o):-len(c)]), c + trail
    return None


def ddmin(prefix, o, elements, c):
    """Minimal SUBSET of `elements` that still rejects. Classic ddmin, granularity-doubling."""
    def rejects(sub):
        if not sub:
            return False
        acc, _ = interp(prefix + o + ",".join(sub) + c)
        return acc is False

    cur = list(elements)
    if not rejects(cur):
        return None            # the decomposition itself changed the verdict — refuse to guess
    n = 2
    while len(cur) >= 2:
        chunk = max(1, len(cur) // n)
        blocks = [cur[i:i + chunk] for i in range(0, len(cur), chunk)]
        for b in blocks:                                   # try a single block
            if rejects(b):
                cur, n = b, 2
                break
        else:
            for i in range(len(blocks)):                   # try the complement
                comp = [x for j, b in enumerate(blocks) if j != i for x in b]
                if rejects(comp):
                    cur, n = comp, max(n - 1, 2)
                    break
            else:
                if n >= len(cur):
                    break
                n = min(len(cur), n * 2)
    return cur


lines = [ln for ln in CORPUS.read_text(errors="replace").split("\n") if ln.strip()]
rows, disagreements = [], 0
for idx, ln in enumerate(lines):
    a_acc, a_fp = interp(ln)
    if a_acc is not False:
        continue
    s_acc, s_fp = shipped(ln)
    agree = (s_acc is False) and (a_fp == s_fp)
    if not agree:
        disagreements += 1
    stack, seen = deepest_rule_stack(ln)
    d = decompose(ln)
    core, core_text, shrink = None, None, "not-a-collection"
    if d:
        prefix, o, els, c = d
        core = ddmin(prefix, o, els, c)
        if core is None:
            shrink = "REFUSED-decomposition-changed-verdict"
        else:
            shrink = f"{len(els)} -> {len(core)} elements"
            core_text = prefix + o + ",".join(core) + c
    probe_text = core_text or ln
    _, core_fp = interp(probe_text)
    arrow_in_core = "=>" in (",".join(core) if core else ln)
    rows.append({
        "idx": idx, "text": ln, "bytes": len(ln),
        "interp_furthest": a_fp, "shipped_furthest": s_fp, "oracles_agree": agree,
        "at_furthest": ln[a_fp:a_fp + 12] if a_fp is not None else None,
        "core_furthest": core_fp,
        "at_core_furthest": probe_text[core_fp:core_fp + 12] if core_fp is not None else None,
        "shrink": shrink, "minimal_core": core_text,
        "arrow_text_in_minimal_core": arrow_in_core,  # TEXT containment only — NOT causal
        "deepest_rule_stack": stack,
        "core_rule_stack": deepest_rule_stack(probe_text)[0] if core_text else stack,
    })

arrow = sum(1 for r in rows if r["arrow_text_in_minimal_core"])
print(f"REJECT-ATTRIBUTION: grammar={GRAMMAR.name} corpus={CORPUS.name} rejected={len(rows)} "
      f"arrow_TEXT_in_core={arrow} no_arrow_text={len(rows) - arrow} oracle_disagreements={disagreements}  ⛔ text-containment, NOT causal — see ledger_arms.py")
print()
for r in rows:
    tag = "has-=>" if r["arrow_text_in_minimal_core"] else "no-=>"
    print(f"#{r['idx']:04d} [{tag}] core_furthest={r['core_furthest']} "
          f"at={r['at_core_furthest']!r} shrink={r['shrink']}")
    print(f"        core: {r['minimal_core'] or r['text']}")
    print(f"        stack: {' > '.join(r['core_rule_stack']) or '<none reported>'}")
if OUT:
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "reject_attribution.json").write_text(json.dumps(rows, indent=2))
    print(f"\nfull attribution -> {OUT / 'reject_attribution.json'}")
if disagreements:
    sys.exit(f"attribute_rejects: {disagreements} interpreter/shipped-parser DISAGREEMENTS — hard error")
