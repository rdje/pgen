#!/usr/bin/env python3
"""`SV-CORPUS-GRAD.13c.2j` — the SCOPE-PREFIX RENDERING-DRIFT sweep.

WHAT IT ASKS
------------
IEEE 1800-2017/2023 A.8.4 writes ONE production for the prefix of a hierarchical name:

    primary ::= [ class_qualifier | package_scope ] hierarchical_identifier select
    class_qualifier ::= [ local :: ] [ implicit_class_handle . | class_scope ]

PGEN renders that prefix in FOUR places. One is a named rule (`primary_hier_scope_prefix`);
the other three are hand-spelled inline groups, because `method_call_receiver_sv_2017`,
`method_call_receiver_sv_2023` and `split_hierarchical_callable_receiver` exist to CUT the
cycle `primary -> call_primary -> method_call -> primary` and so cannot simply say `primary`.

⛔ A copy of a normative production drifts, and nothing in this repository compared the four.
It did drift: `SV-EXH-PROOF.3.3.4.b.6.2.37.3` added the `class_scope` branch to the named
rendering for a measured defect (`super_t::m_typename = tname`), and none of the three copies
received it — so `p::base::m.g()` (A.8.2's `method_call_root ::= primary`) had no derivation
for two months while `p::base::m` alone parsed.

HOW IT DECIDES
--------------
For every rendering it extracts the SET of alternatives and compares it to the canonical set
carried by `primary_hier_scope_prefix`. A rendering is in scope when its alternatives are drawn
from the scope-prefix vocabulary AND it is immediately followed by the name it prefixes
(`hierarchical_identifier`, or the callable receiver's `( identifier constant_bit_select dot … )*`
loop) — that positional test keeps legitimately-different productions out of the comparison
(`nettype_scope_prefix`, `scoped_type_scope_prefix` render A.2.2.1 / A.2.1.3 prefixes and have
their own sets).

⭐⭐ A DIVERGENCE IS NOT AUTOMATICALLY A DEFECT, AND THE DIFFERENCE IS MEASURED, NOT ASSERTED.
`.13c.2j` restored `class_scope` at all three copies and the parse-cost ratchet REFUSED it:
`entries` +0.47 %, `memo_hits` +1.05 %, while a per-site attribution showed
`split_hierarchical_callable_receiver` alone carried 100 % of the accept-set gain and neither
`method_call_receiver_*` copy moved a single row on either profile. Costs are rejected here, not
traded, so the two inert copies keep the shorter prefix — as a PINNED divergence below, never as
silence. And the pin is not prose: by default this sweep RE-DERIVES the whole attribution. It
strips `class_scope` from ALL FOUR renderings in a scratch copy (the pre-fix state, derived from
this grammar rather than from git), proves every reproducer REJECTS there, then restores it at ONE
site at a time and re-parses through the interpreter (TOOLBOX 1.5b). Every entry is falsifiable in
its own direction — the load-bearing site must flip the rows, each pinned site must not:

    ATTRIBUTION BROKEN: `class_scope` at method_call_receiver_sv_2017 ALONE accepts …
    ATTRIBUTION BROKEN: `class_scope` at split_hierarchical_callable_receiver ALONE rejects …

⛔ CONTROLS RUN BEFORE ANY VERDICT PRINTS, so a green sweep cannot be a broken probe:
  * the canonical rule MUST resolve and MUST carry at least two alternatives — otherwise every
    site trivially "matches" an empty expectation;
  * at least `MIN_SITES` renderings MUST be found — an extractor that finds nothing reads as a
    clean sweep;
  * every alternative named at every site MUST resolve to a defined rule — a typo would
    otherwise be silently compared as a distinct-but-equal token;
  * the MECHANISM must be demonstrated live: the two synthetic grammars beside this file are run
    through the interpreter and must DISAGREE — `inline_alt_giveback.ebnf` ACCEPTS `p::b::m`, and
    `inline_alt_noLong.ebnf`, the same grammar with one alternative removed from the inline group,
    REJECTS it. That pair is `.13c.2j` in five rules, and it is what proves an inline alternation
    gives back the way a rule's branches do (`branch_policy=longest_match`) — which is why the fix
    did not need the copies lifted into a shared rule;
  * the attribution spec MUST still describe the grammar: every site it names must exist, and the
    alternative it is about must still be in the canonical set — a stale spec fails rather than
    silently measuring nothing;
  * with the alternative removed from EVERY rendering, all reproducers MUST reject. If one still
    accepts, the attribution below is measuring something other than this alternative.

HONEST BOUNDS, stated rather than hidden
----------------------------------------
1. This decides the alternative SET and the reachability of each pinned omission. It does NOT
   decide what FOLLOWS the prefix: `primary_sv_2017` writes `hierarchical_identifier select`
   where the copies write `hierarchical_identifier` alone. That difference is deliberate and
   measured — `.13c.2c` restored `select` to both receivers and moved ZERO rows in either
   direction, because `hierarchical_identifier`'s own possessive `( identifier
   constant_bit_select dot )*` has already consumed the selects (`LANG-CAPABILITY-AUDIT.10.18`).
   Each site's trailing context is printed so the divergence stays visible.
2. Re-derivation runs the INTERPRETER, whose authority is by verification (1.6/1.7) rather than
   by construction, over the reproducers `ATTRIBUTION["inputs"]` names. It answers "does this
   site's omission change the accept set FOR THESE ROWS", which is a structural question on the
   certified surface; it is not a substitute for the corpus, and a construct not represented in
   those inputs is outside what it can see.
3. One member of the canonical set is itself a defect: `kw_class_qualifier_fa08937d` is
   `trivia /class_qualifier\\b/` — a literal keyword standing in for an LRM NONTERMINAL, owned by
   `.13c.2m`. This sweep asserts that the renderings AGREE; it does not assert that what they
   agree on is right.

    python3 docs/tasks/artifacts/sv_corpus_grad/scope_prefix_rendering_drift/sweep_scope_prefix_renderings.py
    …/sweep_scope_prefix_renderings.py --fast   # textual pass only, prints NOT RE-DERIVED

⭐ `--grammar <file>` points the sweep at any grammar, which is how it was PROVEN ABLE TO FAIL
([[a-check-whose-inputs-all-pass-has-not-been-tested]]): run it against the pre-`.13c.2j` grammar
and the un-pinned site reports DRIFT at exit 1.
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
MIN_SITES = 4  # the canonical rendering + the three cycle-cutting copies
TIMEOUT_S = 600

# The vocabulary of A.8.4's `[ class_qualifier | package_scope ]`.
SCOPE_VOCAB = {
    "kw_class_qualifier_fa08937d",
    "non_typedef_package_scope",
    "class_scope",
    "implicit_class_handle",
}

# What a rendering of this production is allowed to be followed by.
FOLLOWERS = ("hierarchical_identifier", "( identifier constant_bit_select dot")

# ⭐ THE MEASURED RULING, RE-DERIVED ON EVERY RUN.
# `.13c.2j` restored `class_scope` at all three copies and the parse-cost ratchet REFUSED it:
# `entries` +0.47 %, `memo_hits` +1.05 %. A per-site attribution then showed ONE site carried
# 100 % of the accept-set gain. Costs are rejected here, not traded, so only that site keeps the
# alternative — and the other two are PINNED divergences whose claim this sweep re-derives.
#
#   member  : the canonical alternative whose placement is under test
#   inputs  : the reproducers the ruling is about
#   flips   : per site — does adding `member` at THAT SITE ALONE flip the inputs to ACCEPT?
#             True  = load-bearing, the alternative belongs there
#             False = inert, a pinned divergence (this is the pin, and it is falsifiable: the day
#                     the site becomes reachable this entry goes RED)
ATTRIBUTION = {
    "member": "class_scope",
    "inputs": ["defect_class_scope_method_receiver.sv",
               "defect_class_scope_indexed_member_call.sv"],
    "profiles": ["sv_2017", "sv_2023"],
    "flips": {
        "split_hierarchical_callable_receiver": True,
        "method_call_receiver_sv_2017": False,
        "method_call_receiver_sv_2023": False,
    },
    "why": {
        "split_hierarchical_callable_receiver":
            "load-bearing: A.8.2 method_call_root over an indexed-member receiver reaches this "
            "rule, and it alone carries the whole flip",
        "method_call_receiver_sv_2017":
            "inert: longest_match hands the class-scoped receiver to the split rule, so widening "
            "here buys nothing and the 3-site variant cost entries +0.47 % / memo_hits +1.05 % "
            "(.13c.2f: no widening without demonstrated gain)",
        "method_call_receiver_sv_2023":
            "inert: the sv_2023 twin of the row above, same measurement",
    },
}

RULE_HEAD_RE = re.compile(r"^([a-zA-Z_][a-zA-Z0-9_]*)\s*:=", re.MULTILINE)
GROUP_RE = re.compile(r"\(\s*([a-zA-Z_][a-zA-Z0-9_ |]*?)\s*\)\s*\?")


def uncommented(text: str) -> str:
    """Comment lines replaced by SPACES — same length, same line count.

    A rule name quoted inside prose is not a rendering, so comments must not be scanned. But this
    function's output is also where byte spans are computed, and those spans are then applied to
    the ORIGINAL text to build scratch grammars. Dropping or emptying comment lines preserves line
    numbers while silently shifting every offset after the first comment; the first version of this
    sweep did exactly that, and its own attribution control caught it (a scratch grammar built at
    the shifted offsets still accepted the reproducer, which the control refused as
    "this alternative is not what decides these rows"). Length-preserving blanking keeps line
    numbers AND offsets honest."""
    out = "\n".join(
        " " * len(line) if line.lstrip().startswith("#") else line
        for line in text.splitlines()
    )
    # `splitlines()` + `join` drops a trailing newline; restore it so len() matches exactly and
    # every offset — including the last rule's — is usable against the original.
    return out + text[len(out):] if len(out) < len(text) else out


def defined_rules(text: str) -> set[str]:
    return set(RULE_HEAD_RE.findall(text))


def canonical_set(text: str) -> set[str]:
    """The alternative set of the NAMED rendering, read from its rule body."""
    match = re.search(
        rf"^{CANONICAL_RULE}\s*:=(.*?)(?=^[a-zA-Z_][a-zA-Z0-9_]*\s*:=|\Z)",
        text,
        re.MULTILINE | re.DOTALL,
    )
    if not match:
        return set()
    return {n for n in re.findall(r"[a-zA-Z_][a-zA-Z0-9_]*", match.group(1)) if n in SCOPE_VOCAB}


def sites(text: str) -> list[dict]:
    """Every INLINE rendering of the prefix production, with its position and trailing context."""
    found = []
    for m in GROUP_RE.finditer(text):
        alts = {a.strip() for a in m.group(1).split("|") if a.strip()}
        if not alts or not alts <= SCOPE_VOCAB:
            continue
        tail = text[m.end(): m.end() + 60].lstrip()
        if not tail.startswith(FOLLOWERS):
            continue
        owner = None
        for hm in RULE_HEAD_RE.finditer(text, 0, m.start()):
            owner = hm.group(1)
        found.append(
            {
                "line": text.count("\n", 0, m.start()) + 1,
                "owner": owner or "<unknown>",
                "alts": alts,
                "span": (m.start(1), m.end(1)),
                "tail": tail.split("\n")[0][:48].rstrip(),
            }
        )
    return found


def interpret(grammar: Path, input_file: Path, profile: str | None = None) -> bool:
    """TOOLBOX 1.5b — accept/reject for one input against one grammar, no codegen."""
    cmd = [str(PIPELINE), str(grammar), "--interpret-parse", str(input_file)]
    if profile:
        cmd += ["--grammar-profile", profile]
    proc = subprocess.run(cmd, capture_output=True, text=True, timeout=TIMEOUT_S, cwd=ROOT)
    return "accepted=true" in proc.stdout


def rewrite_variant(raw: str, inline: list[dict], member: str, present: set[str]) -> str:
    """A scratch copy of the grammar in which `member` is present at exactly the named sites.

    ⛔ ONE pass, later spans first. Every span was measured against the ORIGINAL text, so a
    sequence of independent rewrites compounds its own offset error — the second version of this
    sweep did exactly that (strip-all, then re-add at one site using the pre-strip offsets) and
    reported the load-bearing site as inert, which its own `flips` expectation refused.
    """
    out = raw
    for site in sorted(inline, key=lambda s: s["span"][0], reverse=True):
        start, end = site["span"]
        alts = [a.strip() for a in out[start:end].split("|") if a.strip()]
        alts = [a for a in alts if a != member]
        if site["owner"] in present:
            alts.append(member)
        out = out[:start] + " " + " | ".join(alts) + " " + out[end:]
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--grammar", type=Path, default=GRAMMAR,
                    help="grammar to sweep (default: the tracked SV grammar)")
    ap.add_argument("--fast", action="store_true",
                    help="textual pass only — skip pin re-derivation and say so loudly")
    args = ap.parse_args()

    raw = args.grammar.read_text(encoding="utf-8")
    text = uncommented(raw)
    rules = defined_rules(text)
    shown = args.grammar.relative_to(ROOT) if args.grammar.is_absolute() else args.grammar
    print(f"SCOPE-PREFIX-RENDERING-DRIFT: grammar={shown}")

    # ---- control 1: the canonical rendering resolves and is non-trivial -------------------
    canon = canonical_set(text)
    if len(canon) < 2:
        print(f"CONTROL FAILED: {CANONICAL_RULE} yielded {len(canon)} alternative(s) "
              f"{sorted(canon)} — the comparison would be vacuous", file=sys.stderr)
        return 2
    print(f"  canonical rendering: {CANONICAL_RULE} = {{{', '.join(sorted(canon))}}}")

    # ---- control 2: the extractor finds the renderings ------------------------------------
    inline = sites(text)
    total = len(inline) + 1  # + the canonical named rendering
    if total < MIN_SITES:
        print(f"CONTROL FAILED: found {total} rendering(s), expected at least {MIN_SITES} — "
              f"the extractor is broken, not the grammar", file=sys.stderr)
        return 2

    # ---- control 3: every alternative named is a real rule --------------------------------
    for site in inline + [{"line": 0, "owner": CANONICAL_RULE, "alts": canon}]:
        unknown = sorted(a for a in site["alts"] if a not in rules)
        if unknown:
            print(f"CONTROL FAILED: {site['owner']}:{site['line']} names undefined rule(s) "
                  f"{unknown}", file=sys.stderr)
            return 2

    if not PIPELINE.exists():
        print(f"CONTROL FAILED: {PIPELINE.relative_to(ROOT)} is missing — build it with "
              f"`cargo build --bin ast_pipeline`", file=sys.stderr)
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

    by_owner = {s["owner"]: s for s in inline}

    # ---- control 5: the attribution spec still describes this grammar ----------------------
    member = ATTRIBUTION["member"]
    if member not in canon:
        print(f"CONTROL FAILED: the attribution is about `{member}`, which is not in the "
              f"canonical set {sorted(canon)} — the spec is stale", file=sys.stderr)
        return 2
    for owner in ATTRIBUTION["flips"]:
        if owner not in by_owner:
            print(f"CONTROL FAILED: the attribution names `{owner}`, which is not a rendering of "
                  f"this production in {shown} — the spec is stale", file=sys.stderr)
            return 2
    targets = []
    for name in ATTRIBUTION["inputs"]:
        target = REPROS / name
        if not target.exists():
            print(f"CONTROL FAILED: the attribution names a missing reproducer {name}",
                  file=sys.stderr)
            return 2
        targets.append(target)

    # ---- the textual verdict ----------------------------------------------------------------
    print(f"  renderings found: {total} (1 named + {len(inline)} inline)")
    drift = []
    for site in sorted(inline, key=lambda s: s["line"]):
        missing = sorted(canon - site["alts"])
        extra = sorted(site["alts"] - canon)
        expected_missing = [] if ATTRIBUTION["flips"].get(site["owner"], True) else [member]
        if missing == expected_missing and not extra:
            mark = "OK   " if not missing else "PIN  "
        else:
            mark = "DRIFT"
            drift.append(site)
        print(f"  {mark} {site['owner']}:{site['line']} "
              f"{{{', '.join(sorted(site['alts']))}}} followed by `{site['tail']}`"
              + (f"  missing={missing} extra={extra}" if mark != "OK   " else ""))
        if mark == "PIN  ":
            print(f"        pin: {ATTRIBUTION['why'][site['owner']]}")

    # ---- the ATTRIBUTION, re-derived: which site alone carries the flip? --------------------
    # Strip `member` from EVERY rendering (the pre-fix state, derived from this grammar rather
    # than from git), prove the defect reproduces there, then restore it at one site at a time.
    # A pin is the entry that must NOT flip; the load-bearing site is the entry that must.
    broken = []
    if args.fast:
        print(f"  ⛔ ATTRIBUTION NOT RE-DERIVED (--fast): the textual sets match the spec, but no "
              f"site's `flips` claim was measured this run")
    else:
        stripped = HERE / ".attr_stripped.ebnf"
        stripped.write_text(rewrite_variant(raw, inline, member, set()), encoding="utf-8")
        try:
            # control: with `member` nowhere, every reproducer must REJECT — otherwise the
            # attribution below is measuring something other than this alternative.
            for target in targets:
                for profile in ATTRIBUTION["profiles"]:
                    if interpret(stripped, target, profile):
                        print(f"CONTROL FAILED: with `{member}` removed from all "
                              f"{len(inline)} renderings, {target.name} [{profile}] still "
                              f"ACCEPTS — this alternative is not what decides these rows",
                              file=sys.stderr)
                        return 2
            print(f"  attribution control: with `{member}` at NO site, all "
                  f"{len(targets)} reproducers REJECT on every profile")

            for owner, should_flip in ATTRIBUTION["flips"].items():
                site = by_owner[owner]
                variant = HERE / f".attr_{owner}.ebnf"
                variant.write_text(
                    rewrite_variant(raw, inline, member, {owner}), encoding="utf-8")
                try:
                    for target in targets:
                        for profile in ATTRIBUTION["profiles"]:
                            got = interpret(variant, target, profile)
                            verdict = "accept" if got else "reject"
                            if got != should_flip:
                                broken.append((owner, target.name, profile))
                                print(f"  ATTRIBUTION BROKEN: `{member}` at {owner} ALONE "
                                      f"{verdict}s {target.name} [{profile}]; the spec says it "
                                      f"must {'accept' if should_flip else 'reject'}",
                                      file=sys.stderr)
                            else:
                                print(f"        {owner} alone -> {target.name} [{profile}] "
                                      f"{verdict}  (as specified)")
                finally:
                    variant.unlink(missing_ok=True)
        finally:
            stripped.unlink(missing_ok=True)

    print(f"SCOPE-PREFIX-RENDERING-DRIFT: sites={total} drift={len(drift)} "
          f"pinned={sum(1 for v in ATTRIBUTION['flips'].values() if not v)} "
          f"attribution_broken={len(broken)} "
          f"attribution_rederived={'no' if args.fast else 'yes'}")
    if drift:
        for site in drift:
            print(f"drift -> {site['owner']}:{site['line']} "
                  f"missing={sorted(canon - site['alts'])} — the attribution spec does not "
                  f"account for this divergence", file=sys.stderr)
    return 1 if (drift or broken) else 0


if __name__ == "__main__":
    sys.exit(main())
