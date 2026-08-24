#!/usr/bin/env python3
"""Census the IEEE-1800-only KEYWORDS that a `verilog_2005` parse can still reach.

WHY THIS EXISTS (`SV-CORPUS-GRAD.13e.7` item (c))
-------------------------------------------------
The `verilog_2005` dialect profile is built to **coherence** — every rule admitted into the
profile is reachable and satisfiable there, and `--lint-grammar`'s `profile_orphans` holds that
at 0 on every commit.  Coherence is not **faithfulness**: it never asks whether an admitted rule
is DERIVABLE from IEEE 1364-2005.  Faithfulness was checked only by the 85 curated cases of
`rust/scripts/verilog_2005_conformance_gate.sh` — a SAMPLE against 1 148 v2005-satisfiable rules,
never a closed population.  That is how nine over-acceptances sat behind one `@profiles`
annotation until `.13e.4` grepped for one of them by hand.

This instrument replaces the sample with a **closed population** for one lens: the reserved
KEYWORD.  IEEE 1364-2005 Annex B is a closed, normative list; `grammars/systemverilog.ebnf`
spells every keyword as its own `kw_*` rule.  So "which keywords can a `verilog_2005` parse
reach that IEEE 1364-2005 does not reserve" is a question with a finite, derivable answer.

⛔ THREE LENSES EXIST AND THIS IS ONE OF THEM — stated because the other two are why a clean
run here is not a clean bill of health:

  L1  KEYWORD ADMISSION      — an IEEE-1800-only keyword is reachable.      ← THIS CENSUS
  L2  POSITION               — a genuine 1364-2005 keyword accepted in a
                               position 1364-2005 never allows.  Measured
                               instance: `automatic integer i;` at module
                               scope (`automatic` is Annex B, but Annex A
                               allows it only on `function`/`task`).
  L3  SHAPE / CARDINALITY    — the right keywords in the wrong arity.
                               Measured instance: `reg [7:0][3:0] r;` —
                               `reg_declaration` has exactly ONE `[ range ]`.

L2 and L3 are structurally invisible here: neither introduces a keyword Annex B lacks.  Of the
nine rows `.13e.7` routed in, seven are L1, one is L2 and one is L3.

⛔ AND A FOURTH LENS EXISTS, FOUND WHILE FIXING THE FIRST THREE.  `{ << { a } }` — an IEEE 1800
streaming concatenation — parsed under `verilog_2005`, and no keyword-level census can ever see
it because `<<` and `>>` are OPERATORS.  Call it **L4, PUNCTUATION-ONLY**: an IEEE-1800-only
construct that introduces no keyword at all.  It was gated by `.13e.7` as a whole rule; nothing
enumerates the rest of its class.

HOW A ROW IS EARNED — three independent legs, because two of them are not enough
--------------------------------------------------------------------------------
A candidate keyword becomes a CONFIRMED over-acceptance only when all three hold:

  1. the shipped release probe ACCEPTS the witness under `verilog_2005`;
  2. it ACCEPTS the same witness under `sv_2017` — the control that proves the witness is
     well-formed SystemVerilog and is isolating a PROFILE difference, not a typo;
  3. substituting the keyword for a fresh identifier makes the witness REJECT under
     `verilog_2005` — the falsifier that proves the keyword was consumed AS A KEYWORD.

⛔⛔ LEG 3 IS THE ONE THAT GETS SKIPPED, AND IT MOVED THE FOUNDING HEADLINE FROM 21 TO 14.
Legs 1+2 alone called `type`, `this`, `super`, `new`, `null`, `randomize` and
`tx_path_delay_expression` over-acceptances.  All seven are wrong: none is reserved in IEEE
1364-2005, so `type(y)` is a perfectly legal call to a function NAMED `type`, and the AST dump
puts it in a `plain_tf` function-call slot.  The parser was right and the witness was lying.
Leg 3 costs one extra parse per row and is the difference between publishing 21 defects and
measuring 14 real ones.

THE CANDIDATE SET IS DERIVED, NOT LISTED
----------------------------------------
Reachability runs over the EBNF frontend's own `raw_ast` envelope — the exact token stream the
code generator consumes — filtered by profile, and it is ALTERNATIVE-AWARE: an alternative whose
mandatory members are not all satisfiable under the profile contributes no edges.

⛔ THE RULE-LEVEL MODEL IS NOT GOOD ENOUGH, MEASURED.  A first cut kept a reference alive when a
MANDATORY SIBLING in the same alternative had been profile-filtered away, and reported `rand`,
`randc`, `packed`, `dist`, `inside`, `local`, `matches`, `std` and `type` reachable through
`data_type`'s struct alternative — whose very first element, `struct_union`, is gated to
`sv_2017`/`sv_2023`.  Alternative-awareness dropped seven of those nine, and the probe legs
refuted the rest.  It is a deliberate OVER-approximation in the remaining direction (a
lookahead-gated or predicate-gated path still counts as reachable), so it can carry false
POSITIVES, which the probe legs remove — never false negatives, which nothing would.

REFUSAL POSTURE
---------------
The run FAILS (rc 1) when:
  - IEEE 1364-2005 Annex B does not extract to exactly `ANNEX_B_COUNT` keywords (the oracle
    moved under us, or the PDF→markdown split changed);
  - a word-shaped candidate has no witness in the tracked manifest — a NEW reachable
    IEEE-1800-only keyword cannot land silently;
  - a witness fails its `sv_2017` control while the model says its keyword IS reachable — an
    unexplained hole, which must be adjudicated rather than counted as "clean";
  - the tracked artifact disagrees with a fresh derivation (gate mode).

Usage:
    python3 stimuli/sv/v2005_keyword_faithfulness_census.py [--write]

Without `--write` the census is recomputed and compared against the tracked copy (gate mode).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]

GRAMMAR = REPO_ROOT / "grammars" / "systemverilog.ebnf"
GRAMMAR_REL = "grammars/systemverilog.ebnf"
# ⛔ 1364-2005's Annex A IS in the file its name promises (386 `::=`); 1800-2017's is NOT — the
# PDF→markdown splitter filed it under `section-41-data-read-api.md`.  Recorded because the
# sibling census `lrm_annex_a_gap_census.py` had to learn it the hard way, and this census reads
# only the 2005 side, where the name happens to be honest.
ANNEX_B = (REPO_ROOT / "docs" / "verilog" / "2005" / "md"
           / "section-Annex_B-normative-list-of-keywords.md")

IDENTITY = REPO_ROOT / "scripts" / "check_baseline_identity.sh"
AST_PIPELINE = REPO_ROOT / "rust" / "target" / "debug" / "ast_pipeline"
PROBE = REPO_ROOT / "rust" / "target" / "release" / "parseability_probe"

ARTIFACT_DIR = (REPO_ROOT / "docs" / "tasks" / "artifacts" / "sv_corpus_grad"
                / "v2005_keyword_faithfulness")
WITNESS_DIR = ARTIFACT_DIR / "witnesses"
MANIFEST = WITNESS_DIR / "MANIFEST.tsv"

PROFILE = "verilog_2005"
CONTROL_PROFILE = "sv_2017"

# The profile's declared entry roots.  `sv_multi_entry_root` unions the three real top-level
# entries, so this list is the closed entry universe and not a guess; it matches the
# `cert_union_configs` the `verilog_2005` conformance contract pins.
ENTRY_ROOTS = ("systemverilog_file", "sv_multi_entry_root", "library_text",
               "systemverilog_parseable_file")

# IEEE 1364-2005 Annex B, as printed.  ⛔ PINNED so a re-extraction that silently loses or gains a
# keyword REFUSES instead of quietly moving every verdict downstream of it.
ANNEX_B_COUNT = 124
# The list ends and a footnote begins at this exact line; the footnote's marker is glued onto the
# last keyword by the PDF conversion (`unsigned1`), and the footnote body ("1unsigned is reserved
# for possible future usage.") contributes three more identifier-shaped lines.  Both are handled
# explicitly rather than by a heuristic, so a changed footnote refuses at ANNEX_B_COUNT.
ANNEX_B_START = "identifier shall not be treated as a keyword."
ANNEX_B_END = "1unsigned"
ANNEX_B_FOOTNOTE_GLUE = {"unsigned1": "unsigned"}

IDENTIFIER_RE = re.compile(r"[A-Za-z_$][A-Za-z0-9_$]*")
KEYWORD_RULE_RE = re.compile(r"^(kw_[A-Za-z0-9_]+)\s*:=\s*trivia\s*/(.+?)/\s*$", re.MULTILINE)
# The fresh identifier leg 3 substitutes in.  Deliberately not a word any LRM reserves and not a
# name any witness declares, so a witness that still parses after substitution parsed the
# keyword as an ORDINARY identifier.
FRESH_IDENTIFIER = "zzq_census_ident"

VERDICT_OVER_ACCEPT = "over_acceptance"
VERDICT_GATED = "correctly_gated"
VERDICT_IDENTIFIER = "identifier_consumed"
VERDICT_UNREACHABLE = "unreachable_no_witness"


class Refused(Exception):
    """The census cannot honestly produce a number."""


# --------------------------------------------------------------------------------------------
# Oracles
# --------------------------------------------------------------------------------------------

def annex_b_keywords() -> list[str]:
    """The IEEE 1364-2005 reserved-keyword list, from the tracked LRM."""
    lines = ANNEX_B.read_text().splitlines()
    try:
        start = next(i for i, l in enumerate(lines) if l.strip() == ANNEX_B_START) + 1
        end = next(i for i, l in enumerate(lines) if l.strip() == ANNEX_B_END)
    except StopIteration as exc:
        raise Refused(f"{ANNEX_B.relative_to(REPO_ROOT)} no longer carries the Annex B list "
                      f"delimiters ({ANNEX_B_START!r} / {ANNEX_B_END!r})") from exc

    words = [l.strip() for l in lines[start:end] if l.strip()]
    unexpected = [w for w in words if not IDENTIFIER_RE.fullmatch(w)]
    if unexpected:
        raise Refused(f"Annex B extraction picked up non-keyword lines: {unexpected[:5]}")
    words = [ANNEX_B_FOOTNOTE_GLUE.get(w, w) for w in words]
    if len(words) != ANNEX_B_COUNT or len(set(words)) != ANNEX_B_COUNT:
        raise Refused(f"Annex B extracted {len(words)} keywords ({len(set(words))} unique), "
                      f"expected exactly {ANNEX_B_COUNT} — the oracle moved; re-adjudicate "
                      f"before trusting any verdict below it")
    return sorted(words)


def raw_ast() -> list:
    """The frontend's `raw_ast` for the SV grammar, via the ONE tool that owns that envelope.

    ⛔ NOT a second frontend invocation: `scripts/check_baseline_identity.sh` already owns the
    `ebnf_raw_ast` definition that `BASELINE-IDENTITY`, `SV-CONTRACT-CURRENCY` and
    `PARSE-COST-RATCHET` all key on.
    """
    proc = subprocess.run([str(IDENTITY), "--raw-ast", GRAMMAR_REL],
                          capture_output=True, text=True, cwd=REPO_ROOT)
    if proc.returncode != 0:
        raise Refused(f"the raw_ast of `{GRAMMAR_REL}` could not be derived "
                      f"(exit {proc.returncode}): {(proc.stderr or '').strip()[:400]}")
    return json.loads(proc.stdout)["raw_ast"]


def rule_satisfiability() -> dict[str, list[str]]:
    """`rule -> profiles it is SATISFIABLE under`, from `--dump-rule-profiles` (TOOLBOX 5.4).

    This is the bottom-up half of liveness — "can this rule derive a string under P".  It is the
    same `derive_rule_profiles` fixpoint the profile-orphan lint gates on, so the two instruments
    cannot disagree about what the profile contains.
    """
    if not AST_PIPELINE.exists():
        raise Refused(f"{AST_PIPELINE.relative_to(REPO_ROOT)} is not built — "
                      f"`cargo build --features \"generated_parsers ebnf_dual_run\"` from rust/")
    out = ARTIFACT_DIR / "rule_profiles.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    proc = subprocess.run([str(AST_PIPELINE), GRAMMAR_REL, "--dump-rule-profiles", str(out)],
                          capture_output=True, text=True, cwd=REPO_ROOT)
    if proc.returncode != 0:
        raise Refused(f"--dump-rule-profiles failed (exit {proc.returncode}): "
                      f"{(proc.stderr or '').strip()[:400]}")
    data = json.loads(out.read_text())
    out.unlink()
    return {r: v["satisfiable_under"] for r, v in data["rules"].items()}


# --------------------------------------------------------------------------------------------
# The alternative-aware, profile-filtered reference graph
# --------------------------------------------------------------------------------------------

def parse_body(tokens: list) -> list:
    """`raw_ast`'s flat token stream for one rule -> a nested `alt`/`opt`/`group`/`look`/`ref` tree.

    The frontend emits alternation as a bare `["operator", "|"]` between items and grouping as
    balanced `group_open`/`group_close`, with `?`/`*`/`+` POSTFIX on the preceding item and
    `!`/`&` PREFIX on the group that follows.  Nothing else in the vocabulary affects liveness.
    """
    def seq(i: int, depth: int):
        branches, items = [], []
        while i < len(tokens):
            kind = tokens[i][0]
            value = tokens[i][1] if len(tokens[i]) > 1 else None
            if kind == "group_open":
                sub, i = seq(i + 1, depth + 1)
                items.append(("group", sub))
                continue
            if kind == "group_close":
                if depth == 0:
                    raise Refused("unbalanced group_close in the raw_ast token stream")
                branches.append(items)
                return ("alt", branches), i + 1
            if kind == "operator":
                if value == "|":
                    branches.append(items)
                    items = []
                elif value in ("?", "*"):
                    if items:
                        items[-1] = ("opt", items[-1])
                elif value in ("!", "&"):
                    sub = None
                    if i + 1 < len(tokens) and tokens[i + 1][0] == "group_open":
                        sub, i = seq(i + 2, depth + 1)
                        items.append(("look", value, sub))
                        continue
                    items.append(("look", value, None))
                # `+` is one-or-more: the item stays mandatory, so it needs no marker.
                i += 1
                continue
            if kind == "rule_reference":
                items.append(("ref", value))
            i += 1
        if depth != 0:
            raise Refused("unterminated group in the raw_ast token stream")
        branches.append(items)
        return ("alt", branches), i

    node, _ = seq(0, 0)
    return node


def _live(node, sat: set[str]) -> bool:
    """Can this node be traversed at all under the profile?"""
    kind = node[0]
    if kind == "ref":
        return node[1] in sat
    if kind == "opt":
        return True                      # it can be skipped, so it never kills its sequence
    if kind == "group":
        return _live(node[1], sat)
    if kind == "look":
        # A NEGATIVE lookahead succeeds precisely when its body does not match, so a filtered
        # body cannot kill the alternative.  A POSITIVE one must match, so it can.
        return True if node[1] == "!" else (_live(node[2], sat) if node[2] else True)
    if kind == "alt":
        return any(all(_live(item, sat) for item in branch) for branch in node[1])
    return True


def _refs(node, sat: set[str], out: set[str]) -> None:
    """Rule references on a LIVE path — dead alternatives contribute nothing, and a negative
    lookahead never accepts anything, so it contributes nothing either."""
    kind = node[0]
    if kind == "ref":
        out.add(node[1])
    elif kind == "opt":
        if _live(node[1], sat):
            _refs(node[1], sat, out)
    elif kind == "group":
        _refs(node[1], sat, out)
    elif kind == "look":
        if node[1] == "&" and node[2] is not None:
            _refs(node[2], sat, out)
    elif kind == "alt":
        for branch in node[1]:
            if all(_live(item, sat) for item in branch):
                for item in branch:
                    _refs(item, sat, out)


def reachable_rules(raw: list, satisfiable: dict[str, list[str]]
                    ) -> tuple[set[str], int, list[str]]:
    """The rules a `PROFILE` parse can descend into, the source-grammar satisfiable population it
    is commensurable with, and the eliminator-synthesised rules this instrument cannot adjudicate."""
    sat = {r for r, profiles in satisfiable.items() if PROFILE in profiles}
    edges: dict[str, set[str]] = {}
    for rule in raw:
        name = rule[0][1]
        if name not in sat:
            edges[name] = set()
            continue
        found: set[str] = set()
        _refs(parse_body(rule[1:]), sat, found)
        edges[name] = found

    # ⛔ THE TWO POPULATIONS ARE NOT THE SAME SET, AND SUBTRACTING THEM NAIVELY IS WRONG.
    # `--dump-rule-profiles` runs on the POST-elimination grammar and reports 1 524 rules;
    # `raw_ast` is the SOURCE grammar and holds 1 493. The 31 extras are all `_lr_*` rules the
    # indirect-left-recursion eliminator SYNTHESISES, which exist in no `.ebnf` file — 19 of them
    # are `verilog_2005`-satisfiable. A reachable-vs-satisfiable gap quoted against the raw 1 148
    # would silently fold those 19 in. Report the intersection, and name the residue.
    in_source = {rule[0][1] for rule in raw}
    synthesised = sorted(sat - in_source)
    sat &= in_source

    roots = [r for r in ENTRY_ROOTS if r in sat and r in edges]
    if not roots:
        raise Refused(f"no declared entry root is satisfiable under `{PROFILE}` — "
                      f"the entry universe {ENTRY_ROOTS} no longer describes this grammar")
    seen, stack = set(roots), list(roots)
    while stack:
        for child in edges.get(stack.pop(), ()):
            if child in sat and child not in seen:
                seen.add(child)
                stack.append(child)
    return seen, len(sat), synthesised


def keyword_literals() -> dict[str, str]:
    """`kw_* rule -> the literal it matches`, read from the grammar's own terminal definitions."""
    literals = {}
    for rule, body in KEYWORD_RULE_RE.findall(GRAMMAR.read_text()):
        head = IDENTIFIER_RE.match(body)
        if head:
            literals[rule] = head.group(0)
    return literals


# --------------------------------------------------------------------------------------------
# The probe legs
# --------------------------------------------------------------------------------------------

def read_manifest() -> dict[str, str]:
    """`keyword -> witness file name`, from the tracked manifest."""
    if not MANIFEST.exists():
        raise Refused(f"{MANIFEST.relative_to(REPO_ROOT)} is missing")
    witnesses = {}
    for line in MANIFEST.read_text().splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) < 2:
            raise Refused(f"malformed manifest row: {line!r}")
        witnesses[fields[0]] = fields[1]
    return witnesses


def parses(path: Path, profile: str) -> bool:
    proc = subprocess.run([str(PROBE), "--parse", "systemverilog", str(path),
                           "--profile", profile], capture_output=True, cwd=REPO_ROOT)
    return proc.returncode == 0


def adjudicate(keyword: str, witness: Path, scratch: Path) -> tuple[str, bool, bool, bool]:
    """The three legs.  Returns `(verdict, accepts_v2005, accepts_control, accepts_substituted)`."""
    source = witness.read_text()
    substituted = re.sub(rf"\b{re.escape(keyword)}\b", FRESH_IDENTIFIER, source)
    if substituted == source:
        raise Refused(f"witness {witness.name} does not contain the bare keyword `{keyword}` — "
                      f"leg 3 cannot be run, so the row cannot be earned")
    scratch.write_text(substituted)

    accepts = parses(witness, PROFILE)
    control = parses(witness, CONTROL_PROFILE)
    if not control:
        return "witness_invalid", accepts, control, False
    if not accepts:
        return VERDICT_GATED, accepts, control, False
    identifier = parses(scratch, PROFILE)
    return (VERDICT_IDENTIFIER if identifier else VERDICT_OVER_ACCEPT,
            accepts, control, identifier)


# --------------------------------------------------------------------------------------------
# Report
# --------------------------------------------------------------------------------------------

def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def render(rows: list[dict], stats: dict) -> tuple[str, str]:
    header = ["keyword", "verdict", "witness", "accepts_verilog_2005",
              "accepts_sv_2017_control", "accepts_after_identifier_substitution"]
    tsv = ["\t".join(header)]
    for row in rows:
        tsv.append("\t".join([
            row["keyword"], row["verdict"], row["witness"],
            str(row["accepts"]).lower(), str(row["control"]).lower(),
            "n/a" if row["verdict"] in (VERDICT_GATED, VERDICT_UNREACHABLE, "witness_invalid")
            else str(row["identifier"]).lower(),
        ]))

    confirmed = [r for r in rows if r["verdict"] == VERDICT_OVER_ACCEPT]
    gated = [r for r in rows if r["verdict"] == VERDICT_GATED]
    ident = [r for r in rows if r["verdict"] == VERDICT_IDENTIFIER]
    unreach = [r for r in rows if r["verdict"] == VERDICT_UNREACHABLE]

    md = [
        "# `verilog_2005` keyword-faithfulness census",
        "",
        "Derived by `stimuli/sv/v2005_keyword_faithfulness_census.py`; do not hand-edit.",
        "",
        "## Instrument identity",
        "",
        "Re-hash these inputs; if any hash differs from the row below, **this report no longer",
        "describes your tree** and the honest act is to re-measure, not to quote.",
        "",
        "| input | sha256 |",
        "| --- | --- |",
        f"| `{GRAMMAR_REL}` (ebnf_raw_ast) | `{stats['grammar_raw_ast_sha256']}` |",
        f"| `{ANNEX_B.relative_to(REPO_ROOT)}` | `{stats['annex_b_sha256']}` |",
        f"| `{MANIFEST.relative_to(REPO_ROOT)}` | `{stats['manifest_sha256']}` |",
        "",
        "## Population",
        "",
        f"- rules in the source grammar: **{stats['rules']}**",
        f"- of those, SATISFIABLE under `{PROFILE}`: **{stats['satisfiable']}** "
        "(the bottom-up half — *can this rule derive a string here*)",
        f"- of those, REACHABLE under `{PROFILE}` from the {len(ENTRY_ROOTS)} declared entry "
        f"roots: **{stats['reachable']}** (the top-down half — *can a parse get here*)",
        f"- **satisfiable but UNREACHABLE: {stats['unreachable']}** — rules the profile admits "
        f"that no `{PROFILE}` input can arrive at. `kw_void_e9cede9b` is the worked example: the "
        "terminal is ungated and trivially satisfiable, while both rules that reference it are "
        "`_sv_only`. ⭐ **This is NOT a gap in the certificate accounting, and an earlier version of "
        "this report said it was.** The engine already crosses the two directions: "
        "`gather_verified_profile_proof_covered_rules` (`VERILOG-2005-PROFILE.6.7`) classifies the "
        "WHOLE active rule set, and every profile-entry-unreachable rule gets an explicitly "
        "RE-VERIFIED `ProfileEntryUnreachable` certificate — a re-verify failure is reported as a "
        "linter bug, never silently covered. Measured: all of these rules appear in the cert pass's "
        "`proof` category, exactly. The number below is useful for understanding the profile's live "
        "surface; it is not a defect.",
        f"- not adjudicated here: **{stats['synthesised']}** `_lr_*` rules the "
        "indirect-left-recursion eliminator synthesises, which exist in no `.ebnf` file and so "
        "have no source-graph edges. The certificate denominator counts them too, which is why "
        f"it reads {stats['satisfiable'] + stats['synthesised']} where this row reads "
        f"{stats['satisfiable']}.",
        f"- keyword literals reachable under `{PROFILE}`: **{stats['reachable_keywords']}**",
        f"- of those, reserved by IEEE 1364-2005 Annex B: **{stats['annex_b_hits']}**",
        f"- of those, NOT reserved by Annex B: **{stats['candidates']}** "
        f"(**{stats['word_candidates']}** word-shaped, **{stats['symbol_candidates']}** "
        "single-letter / symbol terminals used by the UDP tables, edge descriptors and "
        "`PATHPULSE$`, which are 1364-2005 terminals rather than keywords)",
        "",
        "### Profile sensitivity — the census carries its own control",
        "",
        "The same computation under `" + CONTROL_PROFILE + "`, where these keywords legitimately",
        "live, gives the denominator this profile is being judged against:",
        "",
        f"- word-shaped IEEE-1800-only keywords reachable under `{CONTROL_PROFILE}`: "
        f"**{stats['control_word_candidates']}**",
        f"- of those, `{PROFILE}` GATES **{stats['gated_out']}** and LEAKS "
        f"**{stats['word_candidates']}**",
        "",
        "⛔ A census blind to the profile would report the identical population under both, so",
        "this row is what separates *\"measured and mostly clean\"* from *\"the reachability",
        "computation never read the profile\"*.",
        "",
        "## Verdicts",
        "",
        f"| verdict | count |",
        "| --- | --- |",
        f"| **over-acceptance (confirmed, 3 legs)** | **{len(confirmed)}** |",
        f"| correctly gated | {len(gated)} |",
        f"| identifier-consumed (refuted by leg 3) | {len(ident)} |",
        f"| unreachable, no witness required | {len(unreach)} |",
        "",
        "### Confirmed `verilog_2005` over-acceptances",
        "",
        "| keyword | witness |",
        "| --- | --- |",
    ]
    md += [f"| `{r['keyword']}` | `{r['witness']}` |" for r in confirmed]
    md += [
        "",
        "### Correctly gated (the control set — the census is not stuck on ACCEPT)",
        "",
        ", ".join(f"`{r['keyword']}`" for r in gated) or "_(none)_",
        "",
        "### Refuted by leg 3 — the keyword was consumed as an ORDINARY IDENTIFIER",
        "",
        ", ".join(f"`{r['keyword']}`" for r in ident) or "_(none)_",
        "",
        "None of these is reserved by IEEE 1364-2005, so accepting the text is CORRECT: the",
        "parse binds it as a user identifier, not as a keyword. Legs 1+2 alone called every one",
        "of them a defect.",
        "",
        "## Honest bound",
        "",
        "This is lens **L1** (keyword admission) only. Two further classes are structurally",
        "invisible to it because neither introduces a keyword Annex B lacks:",
        "",
        "- **L2 — position.** A genuine 1364-2005 keyword accepted where 1364-2005 never allows",
        "  it. Measured: `automatic integer i;` at module scope (Annex A permits `automatic`",
        "  only on `function`/`task`).",
        "- **L3 — shape / cardinality.** The right keywords in the wrong arity. Measured:",
        "  `reg [7:0][3:0] r;` — `reg_declaration` carries exactly one `[ range ]`.",
        "",
        "Reachability is also a deliberate over-approximation in one remaining direction: a path",
        "gated only by a parse-time `@predicate` or a lookahead still counts as reachable. That",
        "can add false POSITIVES, which the probe legs remove; it cannot hide a true one.",
        "",
    ]
    return "\n".join(tsv) + "\n", "\n".join(md)


def main() -> int:
    global PROFILE, WITNESS_DIR, MANIFEST

    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--write", action="store_true",
                    help="write the artifact instead of comparing against the tracked copy")
    # ⭐ CONTROL-ONLY KNOBS, and they are knobs so that the refusals can be PROVEN without
    # mutating a tracked input.  An instrument that can only ever return one reading is not a
    # measurement; `probe.sh` beside the artifact drives these to make each refusal fire and to
    # show the verdict column MOVING when the profile changes.  Either knob forbids `--write`,
    # so a control run can never become the tracked baseline.
    ap.add_argument("--profile", default=None,
                    help=f"control only: the dialect profile to census (default {PROFILE})")
    ap.add_argument("--witness-dir", default=None,
                    help="control only: an alternate witness directory + MANIFEST.tsv")
    args = ap.parse_args()

    controlled = (args.profile is not None and args.profile != PROFILE) \
        or args.witness_dir is not None
    if controlled and args.write:
        print("V2005-KEYWORD-FAITHFULNESS: REFUSED — --write is not available under a control "
              "knob (--profile / --witness-dir); a control run must never become the baseline",
              file=sys.stderr)
        return 2
    if args.profile is not None:
        PROFILE = args.profile
    if args.witness_dir is not None:
        WITNESS_DIR = Path(args.witness_dir)
        if not WITNESS_DIR.is_absolute():
            WITNESS_DIR = REPO_ROOT / WITNESS_DIR
        MANIFEST = WITNESS_DIR / "MANIFEST.tsv"
    if controlled:
        print(f"V2005-KEYWORD-FAITHFULNESS: CONTROL RUN — profile={PROFILE} "
              f"witnesses={WITNESS_DIR.relative_to(REPO_ROOT)}; this reading is NOT a baseline")

    try:
        annex_b = set(annex_b_keywords())
        raw = raw_ast()
        satisfiable = rule_satisfiability()
        reachable, satisfiable_count, synthesised = reachable_rules(raw, satisfiable)
        literals = keyword_literals()

        reachable_keywords = sorted({literals[r] for r in reachable if r in literals})
        candidates = sorted(k for k in reachable_keywords if k not in annex_b)
        words = [k for k in candidates if len(k) > 1 and k == k.lower()]

        # The census's own control: re-run the reachability half under the profile where these
        # keywords BELONG.  It shares every input with the measurement above except the profile,
        # so a computation that ignored the profile would return the identical population.
        control_profile, control_words = PROFILE, None
        try:
            PROFILE = CONTROL_PROFILE
            control_reachable, _, _ = reachable_rules(raw, satisfiable)
            control_keywords = sorted({literals[r] for r in control_reachable if r in literals})
            control_words = [k for k in control_keywords
                             if k not in annex_b and len(k) > 1 and k == k.lower()]
        finally:
            PROFILE = control_profile
        if len(control_words) <= len(words):
            raise Refused(
                f"the control profile `{CONTROL_PROFILE}` reaches {len(control_words)} "
                f"IEEE-1800-only keywords and `{PROFILE}` reaches {len(words)} — the censused "
                f"profile cannot reach at least as many as the profile those keywords belong to, "
                f"so the reachability computation is not reading the profile")

        witnesses = read_manifest()
        missing = [k for k in words if k not in witnesses]
        if missing:
            raise Refused(
                f"{len(missing)} reachable IEEE-1800-only keyword(s) have no witness in "
                f"{MANIFEST.relative_to(REPO_ROOT)}: {missing}. A new one cannot land silently "
                f"— add a witness under {WITNESS_DIR.relative_to(REPO_ROOT)} and adjudicate it.")

        if not PROBE.exists():
            raise Refused(f"{PROBE.relative_to(REPO_ROOT)} is not built — "
                          f"`cargo build --release --features generated_parsers` from rust/")

        ARTIFACT_DIR.mkdir(parents=True, exist_ok=True)
        scratch = ARTIFACT_DIR / ".substituted.sv"
        rows, holes = [], []
        for keyword in sorted(witnesses):
            witness = WITNESS_DIR / witnesses[keyword]
            if not witness.exists():
                raise Refused(f"witness {witness.relative_to(REPO_ROOT)} named by the manifest "
                              f"for `{keyword}` does not exist")
            verdict, accepts, control, identifier = adjudicate(keyword, witness, scratch)
            if verdict == "witness_invalid":
                # Honest only when the model agrees the keyword cannot be reached anyway.
                if keyword in words:
                    holes.append(keyword)
                verdict = VERDICT_UNREACHABLE
            rows.append({"keyword": keyword, "verdict": verdict, "witness": witnesses[keyword],
                         "accepts": accepts, "control": control, "identifier": identifier})
        scratch.unlink(missing_ok=True)

        if holes:
            raise Refused(
                f"{len(holes)} keyword(s) are reachable under `{PROFILE}` but their witness "
                f"fails its `{CONTROL_PROFILE}` control, so nothing was measured: {holes}. "
                f"Fix the witness — an unmeasured keyword must not be counted as clean.")

        stats = {
            "rules": len(raw),
            "satisfiable": satisfiable_count,
            "synthesised": len(synthesised),
            "unreachable": satisfiable_count - len(reachable),
            "reachable": len(reachable),
            "reachable_keywords": len(reachable_keywords),
            "annex_b_hits": len(reachable_keywords) - len(candidates),
            "candidates": len(candidates),
            "word_candidates": len(words),
            "symbol_candidates": len(candidates) - len(words),
            "control_word_candidates": len(control_words),
            "gated_out": len(control_words) - len(words),
            "grammar_raw_ast_sha256": subprocess.run(
                [str(IDENTITY), "--digest", "ebnf_raw_ast", GRAMMAR_REL],
                capture_output=True, text=True, cwd=REPO_ROOT).stdout.strip(),
            "annex_b_sha256": sha256(ANNEX_B),
            "manifest_sha256": sha256(MANIFEST),
        }
        tsv, md = render(rows, stats)
    except Refused as exc:
        print(f"V2005-KEYWORD-FAITHFULNESS: REFUSED — {exc}", file=sys.stderr)
        return 1

    confirmed = sum(1 for r in rows if r["verdict"] == VERDICT_OVER_ACCEPT)
    headline = (f"V2005-KEYWORD-FAITHFULNESS: reachable_rules={stats['reachable']} "
                f"reachable_keywords={stats['reachable_keywords']} "
                f"candidates={stats['word_candidates']} over_acceptances={confirmed} "
                f"gated={sum(1 for r in rows if r['verdict'] == VERDICT_GATED)} "
                f"identifier_consumed={sum(1 for r in rows if r['verdict'] == VERDICT_IDENTIFIER)}")

    tsv_path, md_path = ARTIFACT_DIR / "census.tsv", ARTIFACT_DIR / "census.md"
    if args.write:
        tsv_path.write_text(tsv)
        md_path.write_text(md)
        print(headline + " (written)")
        return 0

    if controlled:
        # ⛔ A CONTROL RUN MUST NOT BE COMPARED AGAINST THE TRACKED BASELINE. Its whole purpose is
        # to produce a DIFFERENT reading (another profile, a mutated witness set), so the drift
        # check would fire on every arm and mask the refusal the arm is actually testing —
        # measured: two `probe.sh` arms reported DRIFT instead of the missing-witness and
        # broken-control refusals they exist to prove.
        print("V2005-KEYWORD-FAITHFULNESS: control run complete — no baseline comparison")
        return 0

    drift = []
    for path, content in ((tsv_path, tsv), (md_path, md)):
        if not path.exists():
            drift.append(f"{path.relative_to(REPO_ROOT)} is missing")
        elif path.read_text() != content:
            drift.append(f"{path.relative_to(REPO_ROOT)} is stale")
    print(headline)
    if drift:
        print("V2005-KEYWORD-FAITHFULNESS: DRIFT — " + "; ".join(drift), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
