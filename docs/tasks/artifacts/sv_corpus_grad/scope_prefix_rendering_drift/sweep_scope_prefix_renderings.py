#!/usr/bin/env python3
"""`SV-CORPUS-GRAD.13c.2j` / `.40` — the SCOPE-PREFIX RENDERING-DRIFT sweep.

WHAT IT ASKS
------------
IEEE 1800-2017/2023 A.8.4 writes ONE production for the prefix of a hierarchical name:

    primary ::= [ class_qualifier | package_scope ] hierarchical_identifier select
    class_qualifier ::= [ local :: ] [ implicit_class_handle . | class_scope ]

PGEN renders that prefix in FOUR places. One is the canonical named rule
(`primary_hier_scope_prefix`); one is the named helper a CALLABLE receiver reaches
(`split_callable_receiver_scope`, lifted out of an inline group by `.3.42`); and two are the
receiver copies `method_call_receiver_sv_2017` / `method_call_receiver_sv_2023`, whose
`hierarchical_identifier` arm renders NO prefix at all — they exist to cut the cycle
`primary -> call_primary -> method_call -> primary` and cannot simply say `primary`.

⛔ A copy of a normative production drifts, and nothing in this repository compared the four.
It did drift: `SV-EXH-PROOF.3.3.4.b.6.2.37.3` added the `class_scope` branch to the named
rendering for a measured defect (`super_t::m_typename = tname`), and none of the copies received
it — so `p::base::m.g()` (A.8.2's `method_call_root ::= primary`) had no derivation for two months
while `p::base::m` alone parsed. And the sweep itself drifted: `.3.42` lifted the inline groups it
extracted into named rules, `.13c.2m` deleted the `class_qualifier` literal in its vocabulary and
`.3.37` / `.39` added a `local ::` arm it did not know, so from 2026-09-06 to 2026-10-03 it read
`CONTROL FAILED` on every grammar and nothing invoked it (`.40`).

HOW IT DECIDES
--------------
The compared vocabulary is the `[ local :: ] class_scope | package_scope` half of the production —
three SPELLINGS, each a whole alternative as the grammar writes it:

    package_scope : non_typedef_package_scope
    class_scope   : class_scope
    local         : kw_local_939bb46a scope_resolution ( instance_or_class_scope )?

A rendering is one of three forms, and each is read from the grammar on every run:
  * NAMED  — a `NAME?` immediately followed by the name it prefixes (`hierarchical_identifier`, or
    the callable receiver's `( non_keyword_identifier hierarchical_bit_select dot …` loop) whose
    rule body is an alternation drawn entirely from the vocabulary;
  * INLINE — a `( a | b )?` group in the same position whose alternatives are vocabulary spellings
    (the pre-`.3.42` form, kept so a re-inlined copy is still seen);
  * EMPTY  — a receiver copy the spec names, whose `hierarchical_identifier` arm carries no prefix.
The canonical rule's alternative SET (vocabulary members only) is the expectation every other
rendering is compared to. A `NAME?` whose body steps outside the vocabulary is NOT this production
— `implicit_handle_or_class_scope` renders A.1.10's `constraint_primary` prefix, `nonrange_lvalue_scope`
A.8.5's `variable_lvalue` — and is printed as excluded, with the alternative that excluded it.

⭐⭐ A DIVERGENCE IS NOT AUTOMATICALLY A DEFECT, AND THE DIFFERENCE IS MEASURED, NOT ASSERTED.
`.13c.2j` restored `class_scope` at all three copies and the parse-cost ratchet REFUSED it:
`entries` +0.47 %, `memo_hits` +1.05 %, while a per-site attribution showed the callable receiver's
rendering alone carried 100 % of the accept-set gain and neither `method_call_receiver_*` copy moved
a single row on either profile. Costs are rejected here, not traded, so the two copies keep NO
prefix — as a PINNED divergence below, never as silence. And the pin is not prose: by default this
sweep RE-DERIVES the whole attribution, per member. It strips the member from EVERY rendering in a
scratch copy (the pre-fix state, derived from this grammar rather than from git), proves every
reproducer REJECTS there, then restores it at ONE site at a time — for an EMPTY site by widening
its `hierarchical_identifier` arm to `( member )? hierarchical_identifier` — and re-parses through
the interpreter (TOOLBOX 1.5b). Every entry is falsifiable in its own direction — the
load-bearing site must flip the rows, each pinned site must not:

    ATTRIBUTION BROKEN: `class_scope` at method_call_receiver_sv_2017 ALONE accepts …
    ATTRIBUTION BROKEN: `local` at split_callable_receiver_scope ALONE rejects …

⛔ CONTROLS RUN BEFORE ANY VERDICT PRINTS, so a green sweep cannot be a broken probe:
  * the canonical rule MUST resolve and MUST carry at least two vocabulary alternatives —
    otherwise every site trivially "matches" an empty expectation;
  * at least `MIN_SITES` renderings MUST be found — an extractor that finds nothing reads as a
    clean sweep (this is the control that caught `.40`);
  * every alternative named at every site MUST resolve to a defined rule;
  * every EMPTY site the spec names MUST exist and MUST still carry a bare `hierarchical_identifier`
    arm — a copy that grew a prefix is DRIFT against its pin, not a new rendering;
  * the MECHANISM must be demonstrated live: the two synthetic grammars beside this file are run
    through the interpreter and must DISAGREE — `inline_alt_giveback.ebnf` ACCEPTS `p::b::m`, and
    `inline_alt_noLong.ebnf`, the same grammar with one alternative removed from the inline group,
    REJECTS it. That pair is `.13c.2j` in five rules;
  * the attribution spec MUST still describe the grammar: every site it names must be a found
    rendering, every member must be in the canonical set, every reproducer must exist;
  * with a member removed from EVERY rendering, all of its reproducers MUST reject. If one still
    accepts, the attribution is measuring something other than that member.

HONEST BOUNDS, stated rather than hidden
----------------------------------------
1. This decides the alternative SET and the reachability of each pinned omission. It does NOT
   decide what FOLLOWS the prefix: `primary_sv_2017` writes `hierarchical_identifier select`
   where the copies write `hierarchical_identifier` alone. That difference is deliberate and
   measured — `.13c.2c` restored `select` to both receivers and moved ZERO rows in either
   direction, because `hierarchical_identifier`'s own possessive `( identifier
   constant_bit_select dot )*` has already consumed the selects (`LANG-CAPABILITY-AUDIT.10.18`).
   Each site's trailing context is printed so the divergence stays visible.
2. The `implicit_class_handle .` half of `class_qualifier` is OUTSIDE the compared vocabulary: the
   grammar renders it as rule ARMS, not as members of these alternations — the canonical rule's
   `implicit_class_handle_prefix_sv_only` arm (`.29`), the callable host's own
   `implicit_class_handle dot callable_method_call_body` alternative — and removing it from the
   canonical rule alone cannot make a `this.`-rooted call reject, so the attribution control has
   no purchase on it. The canonical rule's arms outside the vocabulary are printed, not compared.
3. Re-derivation runs the INTERPRETER, whose authority is by verification (1.6/1.7) rather than
   by construction, over the reproducers each member's `inputs` names. It answers "does this
   site's omission change the accept set FOR THESE ROWS", which is a structural question on the
   certified surface; it is not a substitute for the corpus, and a construct not represented in
   those inputs is outside what it can see.

    python3 docs/tasks/artifacts/sv_corpus_grad/scope_prefix_rendering_drift/sweep_scope_prefix_renderings.py
    …/sweep_scope_prefix_renderings.py --fast          # textual pass only, prints NOT RE-DERIVED
    …/sweep_scope_prefix_renderings.py --red-control   # proves the sweep can FAIL: a scratch copy
                                                      # with `class_scope` stripped from the callable
                                                      # receiver's rendering must read DRIFT (rc 1)

⭐ `--grammar <file>` points the sweep at any grammar, which is how it was first PROVEN ABLE TO
FAIL ([[a-check-whose-inputs-all-pass-has-not-been-tested]]): run it against the pre-`.13c.2j`
grammar and the un-pinned site reports DRIFT at exit 1. Since `.40` the red control is built in
and the SV syntax-closure gate runs both arms, so a control that cannot run is never again
mistaken for a passing one.
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
GRAMMAR = ROOT / "grammars/systemverilog.ebnf"
PIPELINE = ROOT / "rust/target/debug/ast_pipeline"
HERE = Path(__file__).resolve().parent
REPROS = ROOT / "stimuli/sv/adjudication_repros"

CANONICAL_RULE = "primary_hier_scope_prefix"
MIN_SITES = 4  # the canonical rendering + the callable receiver's rendering + the two receiver copies
TIMEOUT_S = 600

# The compared vocabulary of A.8.4's `[ class_qualifier | package_scope ]`: key -> the whole
# alternative as the grammar spells it (whitespace-normalised). Bound 2 says why the
# `implicit_class_handle .` half is not here.
VOCAB = {
    "package_scope": "non_typedef_package_scope",
    "class_scope": "class_scope",
    "local": "kw_local_939bb46a scope_resolution ( instance_or_class_scope )?",
}
SPELLING_TO_KEY = {v: k for k, v in VOCAB.items()}

# What a rendering of this production is allowed to be followed by.
FOLLOWERS = ("hierarchical_identifier", "( non_keyword_identifier hierarchical_bit_select dot")

# The receiver copies: rule -> the arm that renders the name with NO prefix. `.3.42` lifted their
# inline groups and `.13c.2m` deleted the literal those groups had held; what is left is the
# bare arm, and the pin says it stays bare.
EMPTY_SITES = {
    "method_call_receiver_sv_2017": "hierarchical_identifier",
    "method_call_receiver_sv_2023": "hierarchical_identifier",
}
EMPTY_ARM_TEMPLATE = '{kind: "hierarchical", qualifier: null, name: $1}'
WIDENED_ARM_TEMPLATE = '{kind: "hierarchical", qualifier: $1, name: $2}'

# ⭐ THE MEASURED RULINGS, RE-DERIVED ON EVERY RUN — one entry per vocabulary member.
#   member  : the vocabulary key whose placement is under test
#   inputs  : the reproducers the ruling is about (stimuli/sv/adjudication_repros/)
#   flips   : per site — does adding `member` at THAT SITE ALONE flip the inputs to ACCEPT?
#             True  = load-bearing, the member belongs there
#             False = inert, a pinned divergence (falsifiable: the day the site becomes reachable
#                     this entry goes RED)
ATTRIBUTION = [
    {
        "member": "class_scope",
        "inputs": ["defect_class_scope_method_receiver.sv",
                   "defect_class_scope_indexed_member_call.sv"],
        "profiles": ["sv_2017", "sv_2023"],
        "flips": {
            "primary_hier_scope_prefix": False,
            "split_callable_receiver_scope": True,
            "method_call_receiver_sv_2017": False,
            "method_call_receiver_sv_2023": False,
        },
        "why": {
            "primary_hier_scope_prefix":
                "inert for a CALL: A.8.2 method_call_root over a class-scoped receiver never reaches "
                "the primary's own prefix — the defect state of .13c.2j had class_scope here alone",
            "split_callable_receiver_scope":
                "load-bearing: A.8.2 method_call_root over an indexed-member receiver reaches "
                "split_hierarchical_callable_receiver, and this rendering alone carries the whole flip",
            "method_call_receiver_sv_2017":
                "inert: longest_match hands the class-scoped receiver to the split rule, so widening "
                "here buys nothing and the 3-site variant cost entries +0.47 % / memo_hits +1.05 % "
                "(.13c.2f: no widening without demonstrated gain)",
            "method_call_receiver_sv_2023":
                "inert: the sv_2023 twin of the row above, same measurement",
        },
    },
    {
        "member": "local",
        "inputs": ["defect_local_qualified_method_call.sv"],
        "profiles": ["sv_2017", "sv_2023"],
        "flips": {
            "primary_hier_scope_prefix": False,
            "split_callable_receiver_scope": True,
            "method_call_receiver_sv_2017": False,
            "method_call_receiver_sv_2023": False,
        },
        "why": {
            "primary_hier_scope_prefix":
                "inert for a CALL: `local::q.size` (no call) parsed through this arm since .3.37 "
                "while `local::q.size()` had no derivation (.39's defect)",
            "split_callable_receiver_scope":
                "load-bearing: .39 (release 1.0.248) added the `local ::` arm HERE and the OpenTitan "
                "sequences parsed; a prefixed callable receiver wins through this rendering",
            "method_call_receiver_sv_2017":
                "inert: the copy renders no prefix and a prefixed receiver never wins through it "
                "(the .3.42 liveness census: committed = 0 on every corpus file)",
            "method_call_receiver_sv_2023":
                "inert: the sv_2023 twin of the row above, same measurement",
        },
    },
]

RULE_HEAD_RE = re.compile(r"^([a-zA-Z_][a-zA-Z0-9_]*)\s*:=", re.MULTILINE)
GROUP_RE = re.compile(r"\(\s*([a-zA-Z_][a-zA-Z0-9_ |()?]*?)\s*\)\s*\?")
NAMED_RE = re.compile(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\?\s+")
ANNOTATION_RE = re.compile(r'@[a-zA-Z_]+:\s*"(?:[^"\\]|\\.)*"')


def uncommented(text: str) -> str:
    """Comment lines replaced by SPACES — same length, same line count.

    A rule name quoted inside prose is not a rendering, so comments must not be scanned. But this
    function's output is also where byte spans are computed, and those spans are then applied to
    the ORIGINAL text to build scratch grammars. Dropping or emptying comment lines preserves line
    numbers while silently shifting every offset after the first comment; the first version of this
    sweep did exactly that, and its own attribution control caught it. Length-preserving blanking
    keeps line numbers AND offsets honest."""
    out = "\n".join(
        " " * len(line) if line.lstrip().startswith("#") else line
        for line in text.splitlines()
    )
    return out + text[len(out):] if len(out) < len(text) else out


def rule_bodies(text: str) -> dict[str, tuple[int, int]]:
    """rule name -> (start, end) byte span of its BODY (after `:=`, up to the next rule head or
    directive line at column 0), measured on the uncommented text — the same offsets hold in the
    original, by `uncommented`'s construction."""
    heads = list(RULE_HEAD_RE.finditer(text))
    bodies = {}
    for i, h in enumerate(heads):
        start = h.end()
        limit = heads[i + 1].start() if i + 1 < len(heads) else len(text)
        # a directive line (`@profiles: […]`) between two rules belongs to the NEXT rule
        directive = re.compile(r"^@", re.MULTILINE).search(text, start, limit)
        end = directive.start() if directive else limit
        bodies[h.group(1)] = (start, end)
    return bodies


def split_top_level(body: str, sep: str = "|") -> list[tuple[int, int]]:
    """Spans of the top-level alternatives of a rule body (nothing inside `(…)`, `[…]`, `{…}` or a
    string literal separates)."""
    spans, depth, i, start, n = [], 0, 0, 0, len(body)
    while i < n:
        c = body[i]
        if c == '"':
            j = i + 1
            while j < n and body[j] != '"':
                j += 2 if body[j] == "\\" else 1
            i = j + 1
            continue
        if c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
        elif c == sep and depth == 0:
            spans.append((start, i))
            start = i + 1
        i += 1
    spans.append((start, n))
    return [(a, b) for a, b in spans if body[a:b].strip()]


def normalised(alt: str) -> str:
    """An alternative's text without annotations and without its return template, whitespace
    collapsed — the form the vocabulary is spelled in."""
    alt = ANNOTATION_RE.sub(" ", alt)
    cut = split_top_level(alt, "-")  # `->` opens the template; `-` never otherwise appears top-level here
    head = alt
    for a, b in cut:
        if alt[b:b + 2] == "->":
            head = alt[:b]
            break
    return " ".join(head.split())


def defined_rules(text: str) -> set[str]:
    return set(RULE_HEAD_RE.findall(text))


def alternatives(text: str, span: tuple[int, int]) -> list[dict]:
    start, end = span
    body = text[start:end]
    out = []
    for a, b in split_top_level(body):
        norm = normalised(body[a:b])
        out.append({"span": (start + a, start + b), "norm": norm,
                    "key": SPELLING_TO_KEY.get(norm)})
    return out


def sites(text: str, bodies: dict[str, tuple[int, int]]) -> tuple[dict, list[dict], list[str]]:
    """The canonical rendering, every other rendering, and the candidates excluded (with why)."""
    canon_alts = alternatives(text, bodies[CANONICAL_RULE]) if CANONICAL_RULE in bodies else []
    canonical = {
        "owner": CANONICAL_RULE, "kind": "named",
        "line": text.count("\n", 0, bodies[CANONICAL_RULE][0]) + 1 if CANONICAL_RULE in bodies else 0,
        "alts": {a["key"] for a in canon_alts if a["key"]},
        "outside": sorted(a["norm"] for a in canon_alts if not a["key"]),
        "span": bodies.get(CANONICAL_RULE, (0, 0)), "parts": canon_alts, "tail": "",
    }
    found, excluded, seen = [], [], set()
    for m in NAMED_RE.finditer(text):
        name = m.group(1)
        tail = text[m.end(): m.end() + 80].lstrip()
        if name == CANONICAL_RULE or name not in bodies or not tail.startswith(FOLLOWERS):
            continue
        if name in seen:
            continue
        seen.add(name)
        parts = alternatives(text, bodies[name])
        strangers = [p["norm"] for p in parts if not p["key"]]
        if strangers:
            excluded.append(f"{name} (alternative `{strangers[0]}` is outside the vocabulary — not "
                            f"a rendering of A.8.4's prefix)")
            continue
        host = None
        for hm in RULE_HEAD_RE.finditer(text, 0, m.start()):
            host = hm.group(1)
        found.append({"owner": name, "kind": "named", "host": host or "<unknown>",
                      "line": text.count("\n", 0, bodies[name][0]) + 1,
                      "alts": {p["key"] for p in parts}, "span": bodies[name], "parts": parts,
                      "tail": tail.split("\n")[0][:48].rstrip()})
    for m in GROUP_RE.finditer(text):
        alts = [" ".join(a.split()) for a in m.group(1).split("|") if a.strip()]
        if not alts or any(a not in SPELLING_TO_KEY for a in alts):
            continue
        tail = text[m.end(): m.end() + 80].lstrip()
        if not tail.startswith(FOLLOWERS):
            continue
        owner = None
        for hm in RULE_HEAD_RE.finditer(text, 0, m.start()):
            owner = hm.group(1)
        found.append({"owner": owner or "<unknown>", "kind": "inline", "host": owner or "<unknown>",
                      "line": text.count("\n", 0, m.start()) + 1,
                      "alts": {SPELLING_TO_KEY[a] for a in alts}, "span": (m.start(1), m.end(1)),
                      "parts": [], "tail": tail.split("\n")[0][:48].rstrip()})
    return canonical, found, excluded


def empty_sites(text: str, bodies: dict[str, tuple[int, int]]) -> tuple[list[dict], list[str]]:
    """The receiver copies the spec names: each must still carry a bare `hierarchical_identifier`
    arm; an arm that grew a vocabulary prefix is reported as drift against the pin."""
    found, failures = [], []
    for owner, arm in EMPTY_SITES.items():
        if owner not in bodies:
            failures.append(f"{owner} is not a rule of this grammar — the EMPTY_SITES spec is stale")
            continue
        parts = alternatives(text, bodies[owner])
        bare = [p for p in parts if p["norm"] == arm]
        grown = [p["norm"] for p in parts
                 if any(p["norm"] == f"{spelling} {arm}" or p["norm"].startswith(f"( {spelling}")
                        for spelling in VOCAB.values())]
        if not bare and not grown:
            failures.append(f"{owner} carries no `{arm}` arm — the EMPTY_SITES spec is stale")
            continue
        found.append({"owner": owner, "kind": "empty", "host": owner,
                      "line": text.count("\n", 0, bodies[owner][0]) + 1,
                      "alts": set(), "span": bare[0]["span"] if bare else bodies[owner],
                      "parts": parts, "grown": grown, "tail": arm})
    return found, failures


def interpret(grammar: Path, input_file: Path, profile: str | None = None) -> bool:
    """TOOLBOX 1.5b — accept/reject for one input against one grammar, no codegen."""
    cmd = [str(PIPELINE), str(grammar), "--interpret-parse", str(input_file)]
    if profile:
        cmd += ["--grammar-profile", profile]
    proc = subprocess.run(cmd, capture_output=True, text=True, timeout=TIMEOUT_S, cwd=ROOT)
    return "accepted=true" in proc.stdout


def rewrite_variant(raw: str, text: str, renderings: list[dict], member: str,
                    present: set[str]) -> str:
    """A scratch copy of the grammar in which `member` is present at exactly the named sites.

    ⛔ ONE pass, later spans first. Every span was measured against the uncommented text (same
    offsets as the original), so a sequence of independent rewrites would compound its own offset
    error — the second version of this sweep did exactly that and reported the load-bearing site as
    inert, which its own `flips` expectation refused. A NAMED rendering is rebuilt from its
    alternatives (comment lines inside the body become whitespace in the scratch copy); an INLINE
    group from its spellings; an EMPTY arm is widened to `( member )? hierarchical_identifier` with
    its template renumbered, or left bare."""
    spelling = VOCAB[member]
    out = raw
    for site in sorted(renderings, key=lambda s: s["span"][0], reverse=True):
        start, end = site["span"]
        if site["kind"] == "named":
            keep = [text[a:b].strip() for (a, b), p in
                    ((p["span"], p) for p in site["parts"]) if p["key"] != member]
            if site["owner"] in present:
                keep.append(spelling)
            body = " " + "\n    | ".join(keep) + "\n"
        elif site["kind"] == "inline":
            alts = [" ".join(a.split()) for a in out[start:end].split("|") if a.strip()]
            alts = [a for a in alts if a != spelling]
            if site["owner"] in present:
                alts.append(spelling)
            body = " " + " | ".join(alts) + " "
        else:  # empty
            if site["owner"] not in present:
                continue
            body = f" ( {spelling} )? hierarchical_identifier -> {WIDENED_ARM_TEMPLATE}\n"
        out = out[:start] + body + out[end:]
    return out


def run(grammar: Path, fast: bool) -> int:
    raw = grammar.read_text(encoding="utf-8")
    text = uncommented(raw)
    rules = defined_rules(text)
    bodies = rule_bodies(text)
    shown = grammar.relative_to(ROOT) if grammar.is_relative_to(ROOT) else grammar
    print(f"SCOPE-PREFIX-RENDERING-DRIFT: grammar={shown}")

    # ---- control 1: the canonical rendering resolves and is non-trivial -------------------
    canonical, named_or_inline, excluded = sites(text, bodies)
    canon = canonical["alts"]
    if len(canon) < 2:
        print(f"CONTROL FAILED: {CANONICAL_RULE} yielded {len(canon)} vocabulary alternative(s) "
              f"{sorted(canon)} — the comparison would be vacuous", file=sys.stderr)
        return 2
    print(f"  canonical rendering: {CANONICAL_RULE} = {{{', '.join(sorted(canon))}}}"
          + (f"  (arms outside the compared vocabulary, bound 2: {canonical['outside']})"
             if canonical["outside"] else ""))
    for why in excluded:
        print(f"  excluded: {why}")

    # ---- control 2: the extractor finds the renderings ------------------------------------
    empties, stale = empty_sites(text, bodies)
    for msg in stale:
        print(f"CONTROL FAILED: {msg}", file=sys.stderr)
    if stale:
        return 2
    renderings = [canonical] + named_or_inline + empties
    if len(renderings) < MIN_SITES:
        print(f"CONTROL FAILED: found {len(renderings)} rendering(s), expected at least "
              f"{MIN_SITES} — the extractor is broken, not the grammar", file=sys.stderr)
        return 2

    # ---- control 3: every alternative named is a real rule --------------------------------
    for site in renderings:
        for p in site.get("parts", []):
            for name in re.findall(r"[a-zA-Z_][a-zA-Z0-9_]*", p["norm"]):
                if name not in rules:
                    print(f"CONTROL FAILED: {site['owner']}:{site['line']} names undefined rule "
                          f"`{name}`", file=sys.stderr)
                    return 2

    if not PIPELINE.exists():
        print(f"CONTROL FAILED: {PIPELINE.relative_to(ROOT)} is missing — build it with "
              f"`cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline`",
              file=sys.stderr)
        return 2

    # ---- control 4: the mechanism is demonstrated live, both ways ---------------------------
    probe = HERE / ".probe_input.txt"
    probe.write_text("p::b::m", encoding="utf-8")
    try:
        with_alt = interpret(HERE / "inline_alt_giveback.ebnf", probe)
        without_alt = interpret(HERE / "inline_alt_noLong.ebnf", probe)
    finally:
        probe.unlink(missing_ok=True)
    if not with_alt or without_alt:
        print(f"CONTROL FAILED: the mechanism synthetics did not behave — "
              f"with-alternative accepted={with_alt} (must be True), "
              f"without-alternative accepted={without_alt} (must be False)", file=sys.stderr)
        return 2
    print("  mechanism control: inline group WITH the alternative accepts `p::b::m`, WITHOUT it "
          "rejects — a missing inline alternative is accept-set-visible")

    by_owner = {s["owner"]: s for s in renderings}

    # ---- control 5: the attribution spec still describes this grammar ----------------------
    for entry in ATTRIBUTION:
        member = entry["member"]
        if member not in canon:
            print(f"CONTROL FAILED: the attribution is about `{member}`, which is not in the "
                  f"canonical set {sorted(canon)} — the spec is stale", file=sys.stderr)
            return 2
        for owner in entry["flips"]:
            if owner not in by_owner:
                print(f"CONTROL FAILED: the attribution names `{owner}`, which is not a rendering "
                      f"of this production in {shown} — the spec is stale", file=sys.stderr)
                return 2
        for name in entry["inputs"]:
            if not (REPROS / name).exists():
                print(f"CONTROL FAILED: the attribution names a missing reproducer {name}",
                      file=sys.stderr)
                return 2

    # ---- the textual verdict ----------------------------------------------------------------
    print(f"  renderings found: {len(renderings)} ({sum(1 for s in renderings if s['kind'] == 'named')} "
          f"named + {sum(1 for s in renderings if s['kind'] == 'inline')} inline + "
          f"{len(empties)} empty)")
    pinned_missing = {}
    for entry in ATTRIBUTION:
        for owner, flips in entry["flips"].items():
            if not flips:
                pinned_missing.setdefault(owner, set()).add(entry["member"])
    drift = []
    for site in sorted(renderings[1:], key=lambda s: s["line"]):
        missing = sorted(canon - site["alts"])
        extra = sorted(site["alts"] - canon)
        expected_missing = sorted(pinned_missing.get(site["owner"], set()))
        if site["kind"] == "empty":
            # the pin says: NO prefix here. Any grown arm is drift; the whole canonical set is the
            # expected omission.
            expected_missing = sorted(canon)
            if site["grown"]:
                extra = site["grown"]
        if missing == expected_missing and not extra:
            mark = "OK   " if not missing else "PIN  "
        else:
            mark = "DRIFT"
            drift.append(site)
        where = f"{site['owner']}:{site['line']}" + (
            f" (used by {site['host']})" if site["kind"] == "named" and site.get("host") else "")
        print(f"  {mark} {where} [{site['kind']}] {{{', '.join(sorted(site['alts']))}}} followed by "
              f"`{site['tail']}`" + (f"  missing={missing} extra={extra}" if mark != "OK   " else ""))
        if mark == "PIN  ":
            for entry in ATTRIBUTION:
                if site["owner"] in entry["flips"] and not entry["flips"][site["owner"]]:
                    print(f"        pin [{entry['member']}]: {entry['why'][site['owner']]}")

    # ---- the ATTRIBUTION, re-derived per member: which site alone carries the flip? ---------
    broken = []
    if fast:
        print("  ⛔ ATTRIBUTION NOT RE-DERIVED (--fast): the textual sets match the spec, but no "
              "site's `flips` claim was measured this run")
    else:
        for entry in ATTRIBUTION:
            member = entry["member"]
            targets = [REPROS / n for n in entry["inputs"]]
            stripped = HERE / f".attr_{member}_stripped.ebnf"
            stripped.write_text(rewrite_variant(raw, text, renderings, member, set()),
                                encoding="utf-8")
            try:
                for target in targets:
                    for profile in entry["profiles"]:
                        if interpret(stripped, target, profile):
                            print(f"CONTROL FAILED: with `{member}` removed from all "
                                  f"{len(renderings)} renderings, {target.name} [{profile}] still "
                                  f"ACCEPTS — this member is not what decides these rows",
                                  file=sys.stderr)
                            return 2
                print(f"  attribution control [{member}]: with `{member}` at NO site, all "
                      f"{len(targets)} reproducer(s) REJECT on every profile")
                for owner, should_flip in entry["flips"].items():
                    variant = HERE / f".attr_{member}_{owner}.ebnf"
                    variant.write_text(rewrite_variant(raw, text, renderings, member, {owner}),
                                       encoding="utf-8")
                    try:
                        for target in targets:
                            for profile in entry["profiles"]:
                                got = interpret(variant, target, profile)
                                verdict = "accept" if got else "reject"
                                if got != should_flip:
                                    broken.append((member, owner, target.name, profile))
                                    print(f"  ATTRIBUTION BROKEN: `{member}` at {owner} ALONE "
                                          f"{verdict}s {target.name} [{profile}]; the spec says it "
                                          f"must {'accept' if should_flip else 'reject'}",
                                          file=sys.stderr)
                                else:
                                    print(f"        [{member}] {owner} alone -> {target.name} "
                                          f"[{profile}] {verdict}  (as specified)")
                    finally:
                        variant.unlink(missing_ok=True)
            finally:
                stripped.unlink(missing_ok=True)

    print(f"SCOPE-PREFIX-RENDERING-DRIFT: sites={len(renderings)} drift={len(drift)} "
          f"pinned={sum(1 for s in renderings[1:] if s['kind'] == 'empty' or pinned_missing.get(s['owner']))} "
          f"members={len(ATTRIBUTION)} attribution_broken={len(broken)} "
          f"attribution_rederived={'no' if fast else 'yes'}")
    if drift:
        for site in drift:
            print(f"drift -> {site['owner']}:{site['line']} missing={sorted(canon - site['alts'])} "
                  f"extra={sorted(site['alts'] - canon) or site.get('grown', [])} — the attribution "
                  f"spec does not account for this divergence", file=sys.stderr)
    return 1 if (drift or broken) else 0


def red_control(grammar: Path) -> int:
    """The sweep must be able to FAIL: strip `class_scope` from the callable receiver's rendering
    in a scratch copy and require DRIFT (rc 1) from the textual pass."""
    raw = grammar.read_text(encoding="utf-8")
    text = uncommented(raw)
    bodies = rule_bodies(text)
    _, found, _ = sites(text, bodies)
    target = next((s for s in found if s["owner"] == "split_callable_receiver_scope"), None)
    if target is None or "class_scope" not in target["alts"]:
        print("RED CONTROL FAILED: split_callable_receiver_scope is not a found rendering carrying "
              "`class_scope` — nothing to strip", file=sys.stderr)
        return 2
    scratch = HERE / ".red_control.ebnf"
    scratch.write_text(rewrite_variant(raw, text, [target], "class_scope", set()), encoding="utf-8")
    try:
        proc = subprocess.run([sys.executable, str(Path(__file__)), "--grammar", str(scratch),
                               "--fast"], capture_output=True, text=True, cwd=ROOT)
    finally:
        scratch.unlink(missing_ok=True)
    drifted = proc.returncode == 1 and "DRIFT split_callable_receiver_scope" in proc.stdout
    print(f"SCOPE-PREFIX-RENDERING-DRIFT-RED-CONTROL: stripped=class_scope@split_callable_receiver_scope "
          f"rc={proc.returncode} drift_reported={'yes' if drifted else 'no'}")
    if not drifted:
        print(proc.stdout[-2000:], file=sys.stderr)
        print(proc.stderr[-2000:], file=sys.stderr)
        print("RED CONTROL FAILED: the sweep did not report DRIFT on a grammar with the member "
              "stripped — it cannot fail, so its green means nothing", file=sys.stderr)
        return 2
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--grammar", type=Path, default=GRAMMAR,
                    help="grammar to sweep (default: the tracked SV grammar)")
    ap.add_argument("--fast", action="store_true",
                    help="textual pass only — skip pin re-derivation and say so loudly")
    ap.add_argument("--red-control", action="store_true",
                    help="prove the sweep can fail: a scratch copy with class_scope stripped from "
                         "the callable receiver's rendering must read DRIFT")
    args = ap.parse_args()
    if args.red_control:
        return red_control(args.grammar)
    return run(args.grammar, args.fast)


if __name__ == "__main__":
    sys.exit(main())
