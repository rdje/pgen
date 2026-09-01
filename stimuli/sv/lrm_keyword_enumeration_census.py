#!/usr/bin/env python3
"""Census every LRM production that ENUMERATES reserved keywords, and ask of each member
whether the shipped SystemVerilog grammar still carries it — the `net_type` class.

WHY THIS EXISTS (`SV-CORPUS-GRAD.13n`(e) / `LRM-GRAMMAR-FIDELITY.1e`)
--------------------------------------------------------------------
`net_type` shipped TEN of the LRM's TWELVE alternatives for the whole life of the flattened
grammar: `tri0` and `tri1` were absent.  Both spellings sat in the reserved-keyword regexes, so
they LEXED as keywords and then had no production to land in — the text was neither an
identifier nor a net type and the parse died with NO signal that a production was missing.
Nine corpus rows on three profiles, found by a census of the v2005 rejects-valid rows
(`.13k`(b)), not by any guard.  ⛔ **A missing alternative in a keyword-bearing alternation is
the quietest defect shape in this repository**: `--lint-grammar` is green (every name
resolves), the certificate pass is green (every rule that exists is witnessed), and the corpus
lanes only see it if a vendored file happens to spell the missing member.

An alternation that lost two members with nothing noticing is a claim about every OTHER
enumerated production in the grammar.  This instrument turns that claim into a closed
population and checks it member by member.

THE TWO ORACLES IT JOINS
------------------------
- **LRM side** — the vendored Annex A markdown of each edition (IEEE 1800-2017, IEEE
  1800-2023, IEEE 1364-2005), read with the SAME primitives `tools/
  extract_systemverilog_lrm_profiles.py` scrapes with, so the shipped grammar's ancestor and
  this census cannot disagree about what a syntax line is (a self-test arm pins the two readers
  equal).  The tracked snapshots `grammars/*_lrm_extracted.ebnf` are a SECONDARY source for
  productions the Annex A markdown does not carry (measured: the snapshots are SUBSETS of
  Annex A for 1800 — 424 and 490 rules against 734 and 747 — and a SUPERSET for 1364-2005,
  where they add the clause-body VCD / system-task / directive productions).  When both
  sources carry an enumeration they must agree or the run refuses (measured: 0 disagreements).
  A production is in the population when EVERY alternative is a single bare token and at
  least one token is a reserved keyword of that edition.  "Reserved keyword" is decided by
  IEEE's own Annex B list for the edition — never by guessing from the spelling — so `supply0`
  is a keyword member and `expression` is a nonterminal member of the same shape of production.
  ⛔ A trailing digit is a FOOTNOTE MARKER only when nothing references the digit-bearing
  spelling (`class_item_qualifier10`, `expression28`); `strength0`, `strength1`, `delay2` and
  `delay3` are real names and are kept — the extractor's own canonicalizer merges those pairs.
- **Shipped side** — the EBNF frontend's `raw_ast` envelope of `grammars/systemverilog.ebnf`
  (via `scripts/check_baseline_identity.sh --raw-ast`, the ONE reader that owns it) plus
  `--dump-rule-profiles` (TOOLBOX 5.4) so each edition is compared under its OWN dialect
  profile.  The shipped rule for an LRM production is looked up by name and by the measured
  profile-variant suffixes (`_sv_2017`, `_sv_2023`, `_sv_only`, `_v2005`, `_verilog_2005`),
  and its member set is the closure over BARE single-reference alternatives (a branch that is
  exactly one `kw_*` reference, one rule reference, or one parenthesised group), restricted to
  alternatives LIVE under the profile.

VERDICTS (per keyword member of a matched production)
-----------------------------------------------------
  present               the member is reachable in the shipped rule's closure under the profile
  MISSING               the shipped grammar HAS a `kw_*` terminal for the spelling, but the
                        rule named for this production cannot reach it — a member dropped from
                        one alternation (the `.13n` shape after the terminal existed)
  MISSING_NO_TERMINAL   no `kw_*` terminal spells it at all — the strongest signal: the word is
                        reserved by the LRM and the shipped grammar cannot produce it anywhere

A production with no shipped rule by name or variant is `unmatched` and NOT scored: the
shipped grammar is a hand-annotated flattened synthesis, so a renamed or lifted production is
an adjudication row, never a MISSING verdict.  Members the shipped closure carries that the
LRM enumeration lacks are printed as `extra` and NOT scored either: Annex-A non-derivability is
a KNOWN-BAD classifier for over-acceptance (`LRM-GRAMMAR-FIDELITY.1c`), and the `verilog_2005`
keyword direction is owned by `v2005_keyword_faithfulness_census.py` (TOOLBOX 5.11).

⛔ THIS IS A TEXT-LEVEL TRIAGE, NOT A PARSE VERDICT.  A `MISSING` row says the shipped rule's
closure does not reach the terminal; whether an INPUT using that keyword is rejected is a
parse question, answered per row with the release probe (`.13k`(b)'s two-sided bisect: the
file rejects, one-keyword substitution accepts).  The census names the rows; the leaf that
consumes it adjudicates them.

REFUSAL POSTURE
---------------
The run FAILS (rc 1) when an Annex B list does not extract to its pinned count, when a snapshot
rule has no terminator, when the Annex A markdown and the snapshot disagree about the members of
one enumeration, or when the tracked artifact disagrees with a fresh derivation (gate mode).  A
member that is neither a keyword, a defined production, a `$name` nor a symbol is classified
`literal` (the LRM writes bare words such as `none` in `default_nettype_value`) and REPORTED,
never scored and never silently dropped.

Usage:
    python3 stimuli/sv/lrm_keyword_enumeration_census.py              # gate mode
    python3 stimuli/sv/lrm_keyword_enumeration_census.py --write      # promote a baseline
    python3 stimuli/sv/lrm_keyword_enumeration_census.py --self-test  # refusal + RED/GREEN arms
    python3 stimuli/sv/lrm_keyword_enumeration_census.py --grammar rust/target/x.ebnf
                                                                      # control only; no --write
"""

from __future__ import annotations

import argparse
import hashlib
import re
import shutil
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))

# ⭐ Reused, never copied: the envelope reader, the profile-satisfiability oracle, the
# alternative-aware body model and the kw_* literal map all come from the sibling census.
import v2005_keyword_faithfulness_census as kwf  # noqa: E402
from v2005_keyword_faithfulness_census import Refused  # noqa: E402
import lrm_annex_a_gap_census as gap  # noqa: E402  (its `load_extractor()` — the canonicalizer)

SHIPPED_GRAMMAR_REL = "grammars/systemverilog.ebnf"
ARTIFACT_DIR = (REPO_ROOT / "docs" / "tasks" / "artifacts" / "sv_corpus_grad"
                / "lrm_keyword_enumeration")
IDENTITY = REPO_ROOT / "scripts" / "check_baseline_identity.sh"

# edition -> (LRM snapshot, Annex B markdown, the dialect profile it is judged under)
EDITIONS = {
    "1800-2017": ("grammars/systemverilog_2017_lrm_extracted.ebnf",
                  # ⛔ NOT in a file named for it: the PDF→markdown splitter filed the 2017
                  # Annex A AND Annex B under `section-41-data-read-api.md` (TOOLBOX 5.8).
                  "docs/systemverilog/2017/md/section-41-data-read-api.md",
                  "sv_2017"),
    "1800-2023": ("grammars/systemverilog_2023_lrm_extracted.ebnf",
                  "docs/systemverilog/2023/md/section-Annex_B-normative-keywords.md",
                  "sv_2023"),
    "1364-2005": ("grammars/verilog_2005_lrm_extracted.ebnf",
                  "docs/verilog/2005/md/section-Annex_B-normative-list-of-keywords.md",
                  "verilog_2005"),
}
# edition -> the Annex A markdown the shipped grammar descends from (the PRIMARY LRM source).
ANNEX_A = {
    "1800-2017": "docs/systemverilog/2017/md/section-41-data-read-api.md",
    "1800-2023": "docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md",
    "1364-2005": "docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md",
}
# IEEE's own counts, pinned so the oracle cannot move silently: Table B.1 holds 248 words in
# both 1800 editions (2023 added none), and 1364-2005 Annex B holds 124.
ANNEX_B_COUNTS = {"1800-2017": 248, "1800-2023": 248, "1364-2005": 124}
ANNEX_B_TABLE_HEAD = "Table B.1—Reserved keywords"
ANNEX_B_TABLE_END = "Annex C"

# The profile-variant suffixes the shipped grammar uses to split one LRM production by dialect,
# MEASURED over every shipped rule name that extends an LRM production name (2026-09-01):
# `_sv_2023` 51, `_sv_2017` 46, `_sv_only` 11, `_v2005` 1, `_verilog_2005` 1.  Every other
# observed suffix (`_item`, `_list`, `_expression`, …) names a DIFFERENT production.
VARIANT_SUFFIXES = ("sv_2017", "sv_2023", "sv_only", "v2005", "verilog_2005")

RULE_HEAD_RE = re.compile(r"^([A-Za-z_$][A-Za-z0-9_$]*)\s*::=\s*(.*)$")
TOKEN_RE = re.compile(r"^[A-Za-z_$][A-Za-z0-9_$]*$")
KEYWORD_LINE_RE = re.compile(r"^[a-z][a-z0-9_]*$")
WRAPPER_SUFFIX = "_lrm_extracted_file"      # PGEN's own `@entry` wrapper, not an LRM production

V_PRESENT = "present"
V_MISSING = "MISSING"
V_MISSING_NO_TERMINAL = "MISSING_NO_TERMINAL"
MISSING_VERDICTS = (V_MISSING, V_MISSING_NO_TERMINAL)

HEADLINE = "LRM-KEYWORD-ENUMERATION:"


# --------------------------------------------------------------------------------------------
# The LRM side
# --------------------------------------------------------------------------------------------

def annex_b_1800(path: Path, edition: str) -> set[str]:
    """IEEE 1800 Table B.1, read between its caption and Annex C, page furniture dropped."""
    lines = path.read_text().splitlines()
    starts = [i for i, l in enumerate(lines)
              if l.strip() == ANNEX_B_TABLE_HEAD]
    if len(starts) != 1:
        raise Refused(f"{edition}: expected exactly one `{ANNEX_B_TABLE_HEAD}` caption in "
                      f"{path.relative_to(REPO_ROOT)}, found {len(starts)}")
    words: list[str] = []
    for line in lines[starts[0] + 1:]:
        stripped = line.strip()
        if stripped.startswith(ANNEX_B_TABLE_END) and words:
            break
        if KEYWORD_LINE_RE.match(stripped):
            words.append(stripped)
    return pin_annex_b(words, edition, path)


def pin_annex_b(words: list[str], edition: str, path: Path) -> set[str]:
    expected = ANNEX_B_COUNTS[edition]
    if len(words) != expected or len(set(words)) != expected:
        raise Refused(f"{edition}: Annex B in {path.relative_to(REPO_ROOT)} extracted "
                      f"{len(words)} keywords ({len(set(words))} unique), expected exactly "
                      f"{expected} — the oracle moved; re-adjudicate before trusting a verdict")
    return set(words)


def annex_b(edition: str) -> set[str]:
    _, annex_rel, _ = EDITIONS[edition]
    path = REPO_ROOT / annex_rel
    if edition == "1364-2005":
        # The sibling census already owns this extraction (start/end sentinels + footnote glue).
        return pin_annex_b(kwf.annex_b_keywords(), edition, path)
    return annex_b_1800(path, edition)


def snapshot_rules(edition: str) -> dict[str, tuple[int, str]]:
    """`production -> (line, body)` from the tracked extraction snapshot (the SECONDARY source).

    A rule runs from its `::=` head to the first line ending in ` ;`.  ⛔ `;` also occurs as a
    LITERAL inside bodies (`program ; { anonymous_program_item } endprogram ;`), so the
    terminator is the line END, never the first semicolon.  A head reached while a previous
    rule is still open is refused — except PGEN's own `@entry` wrapper, which is not an LRM
    production and carries a return annotation instead of a terminator.
    """
    snap_rel, _, _ = EDITIONS[edition]
    lines = (REPO_ROOT / snap_rel).read_text().splitlines()
    rules: dict[str, tuple[int, str]] = {}
    name: str | None = None
    start = 0
    body: list[str] = []

    def close() -> None:
        nonlocal name, body
        if name is not None:
            if name in rules:
                raise Refused(f"{edition}: `{name}` is defined twice in {snap_rel} "
                              f"(lines {rules[name][0]} and {start})")
            text = " ".join(body).strip()
            if text.endswith(" ;"):
                text = text[:-2]
            rules[name] = (start, text.strip())
        name, body = None, []

    for lineno, raw in enumerate(lines, 1):
        # ⛔ Only a COLUMN-0 `(* … *)` is a snapshot comment (the three header lines).  An
        # INDENTED `(* attr_spec { , attr_spec } *)` is IEEE 1800's attribute literal inside
        # `attribute_instance`'s body, and the first cut of this reader ate it as a comment.
        line = "" if raw.startswith("(*") else raw
        head = RULE_HEAD_RE.match(line)
        if head:
            if name is not None:
                if name.endswith(WRAPPER_SUFFIX):
                    name, body = None, []
                else:
                    raise Refused(f"{edition}: `{name}` (line {start}) has no ` ;` terminator "
                                  f"before `{head.group(1)}` opens at line {lineno}")
            name, start, body = head.group(1), lineno, [head.group(2)]
            if line.rstrip().endswith(" ;"):
                close()
            continue
        if name is None:
            continue
        body.append(line)
        if line.rstrip().endswith(" ;"):
            close()
    if name is not None and not name.endswith(WRAPPER_SUFFIX):
        raise Refused(f"{edition}: `{name}` (line {start}) is unterminated at end of file")
    return rules


def annex_a_rules_raw(edition: str, extractor) -> list[tuple[str, int, str]]:
    """`(raw_name, line, body)` in document order from the Annex A markdown (the PRIMARY
    source), read with the EXTRACTOR'S OWN primitives — `normalize_text`, `RULE_HEAD_RE`,
    `is_noise_line`, `looks_like_grammar_line` — and the same body-assembly loop as its
    `extract_rules`, minus the canonicalisation that merges `strength0`/`strength1` into one
    name.  `--self-test` pins this reader equal to `extract_rules` after canonicalisation, so
    a divergence between the LRM and the shipped grammar cannot be an artifact of a second,
    differently-buggy scraper (the `lrm_annex_a_gap_census.py` principle)."""
    path = REPO_ROOT / ANNEX_A[edition]
    lines = extractor.normalize_text(
        path.read_text(encoding="utf-8", errors="replace")).splitlines()
    out: list[tuple[str, int, str]] = []
    idx = 0
    while idx < len(lines):
        match = extractor.RULE_HEAD_RE.match(lines[idx])
        if not match:
            idx += 1
            continue
        raw_name, head_line = match.group(1), idx + 1
        parts: list[str] = []
        inline = match.group(2).strip()
        if inline:
            parts.append(inline)
        idx += 1
        while idx < len(lines):
            if extractor.RULE_HEAD_RE.match(lines[idx]):
                break
            stripped = lines[idx].strip()
            if stripped.startswith("```") or extractor.is_noise_line(stripped):
                idx += 1
                continue
            if not stripped:
                idx += 1
                continue
            if not extractor.looks_like_grammar_line(lines[idx]):
                break
            parts.append(stripped)
            idx += 1
        body = re.sub(r"\s+", " ", " ".join(parts)).strip()
        out.append((extractor.normalize_text(raw_name).strip(), head_line, body))
    return out


FOOTNOTE_RE = re.compile(r"(\d+(,\d+)*)$")


def make_canonicalizer(heads: set[str], referenced: set[str], keywords: set[str] = frozenset()):
    """Strip a trailing footnote marker; keep a REAL trailing digit.

    IEEE prints footnote numbers on definitions (`class_item_qualifier10 ::=`) AND on uses
    (`assignment_pattern_expression37`), but never spells one production with the same digits
    in both places — the digit is a footnote number.  A real name (`strength0`, `delay2`) is
    defined AND used verbatim.  So: a DEFINED digit-bearing name is a footnote form only when
    nothing references it verbatim and the stripped spelling is known; a USED digit-bearing
    name is a footnote form when the stripped spelling is known.  ⛔ The extractor's own
    `canonicalize_rule_name` strips unconditionally and merges `strength0`/`strength1` and
    `delay2`/`delay3` into one rule each — measured, and why this census does not reuse it."""
    def canon(name: str) -> str:
        # ⛔ A reserved keyword is never a footnote form: `tri0`, `strong0`, `unique0` and
        # `supply1` end in a REAL digit and `tri`/`strong`/`unique` are also names, so the
        # rules below would strip them.  The first cut did, and reported `strength0` missing
        # `strong` and `weak` — members the LRM never wrote.
        if name in keywords:
            return name
        stripped = FOOTNOTE_RE.sub("", name)
        if stripped == name or not stripped:
            return name
        known = stripped in heads or stripped in referenced
        if name in heads:
            return stripped if (name not in referenced and known) else name
        return stripped if known else name
    return canon


def lrm_rules(edition: str, extractor) -> tuple[dict[str, dict], set[str], object]:
    """`production -> {members, sources, cite}` for every ENUMERATION-shaped production the
    edition defines (all alternatives bare tokens), from Annex A first and the snapshot
    second, plus the closed set of names either source defines (for member classification)
    and the canonicalizer both were read with."""
    annex = annex_a_rules_raw(edition, extractor)
    snap = snapshot_rules(edition)
    keywords = annex_b(edition)
    heads = {n for n, _, _ in annex} | set(snap)
    referenced: set[str] = set()
    for _, _, body in annex:
        referenced.update(re.findall(r"[A-Za-z_$][A-Za-z0-9_$]*", body))
    for _, body in snap.values():
        referenced.update(re.findall(r"[A-Za-z_$][A-Za-z0-9_$]*", body))
    canon = make_canonicalizer(heads, referenced, keywords)
    defined = heads | {canon(h) for h in heads}

    def enumeration(body: str) -> list[str] | None:
        alts = [a.strip() for a in body.split("|")]
        if alts and all(TOKEN_RE.match(a) for a in alts):
            return [canon(a) for a in alts]
        return None

    # Every production's BARE single-token alternatives, enumeration-shaped or not, so the
    # LRM-side closure can expand `data_type ::= … | string | chandle | event | …` the same
    # way the shipped-side closure expands `data_type := … | kw_string | …`.  The first cut
    # expanded only enumeration-shaped members and reported `chandle`/`event`/`string` as
    # `extra` on six mixed productions — members the LRM reaches one hop further down.
    bare: dict[str, set[str]] = {}
    for raw_name, _, body in annex:
        bare.setdefault(canon(raw_name), set()).update(
            canon(a.strip()) for a in body.split("|") if TOKEN_RE.match(a.strip()))
    for raw_name, (_, body) in snap.items():
        bare.setdefault(canon(raw_name), set()).update(
            canon(a.strip()) for a in body.split("|") if TOKEN_RE.match(a.strip()))

    rules: dict[str, dict] = {}
    for raw_name, line, body in annex:
        members = enumeration(body)
        if members is None:
            continue
        name = canon(raw_name)
        if name in rules and rules[name]["members"] != members:
            raise Refused(f"{edition}: `{name}` is enumerated twice in {ANNEX_A[edition]} "
                          f"with different members (lines {rules[name]['line']} and {line})")
        rules.setdefault(name, {"members": members, "sources": [], "line": line,
                                "cite": f"`{ANNEX_A[edition]}:{line}`"})
        if "annex_a" not in rules[name]["sources"]:
            rules[name]["sources"].append("annex_a")
    snap_rel = EDITIONS[edition][0]
    for raw_name, (line, body) in snap.items():
        members = enumeration(body)
        if members is None:
            continue
        name = canon(raw_name)
        if name in rules:
            if rules[name]["members"] != members:
                # The two surfaces define a production of the same NAME differently.  Measured
                # today this is `size` in 1800-2023 — A.8.7's `size ::= unsigned_number` versus
                # the clause-21 VCD `size ::= decimal_number` the snapshot took — a clause-body
                # HOMONYM, outside this census's population.  Refuse only when a KEYWORD
                # enumeration is in dispute; otherwise keep Annex A and record the conflict.
                if any(m in keywords for m in rules[name]["members"] + members):
                    raise Refused(f"{edition}: `{name}` differs between {ANNEX_A[edition]} "
                                  f"({rules[name]['members']}) and {snap_rel}:{line} ({members})")
                rules[name]["conflict"] = f"`{snap_rel}:{line}` reads {members}"
                continue
            rules[name]["sources"].append("snapshot")
        else:
            rules[name] = {"members": members, "sources": ["snapshot"], "line": line,
                           "cite": f"`{snap_rel}:{line}`"}
    return rules, defined, canon, bare


def lrm_closure(name: str, bare: dict[str, set[str]], keywords: set[str],
                seen: set[str] | None = None) -> set[str]:
    """The keywords an LRM production reaches TRANSITIVELY through BARE single-token
    alternatives — `casting_type` → `simple_type` → `integer_type` → `bit | logic | reg …`,
    and `data_type_or_void` → `data_type` → `string`, `chandle`, `event`.  The shipped side's
    closure is built by the same rule, so the two are commensurable and a mixed production's
    expansion of its own nonterminals is never reported as `extra`."""
    seen = set() if seen is None else seen
    if name in seen or name not in bare:
        return set()
    seen.add(name)
    out: set[str] = set()
    for member in bare[name]:
        if member in keywords:
            out.add(member)
        else:
            out |= lrm_closure(member, bare, keywords, seen)
    return out


def classify_member(token: str, keywords: set[str], defined: set[str], canon) -> str:
    if token in keywords:
        return "keyword"
    if token in defined or canon(token) in defined:
        return "nonterminal"
    if token.startswith("$"):
        return "system_name"
    if len(token) == 1 or not KEYWORD_LINE_RE.match(token):
        return "symbol"
    return "literal"


def lrm_population(edition: str, extractor) -> list[dict]:
    """Every enumeration-shaped production that names >= 1 reserved keyword of the edition."""
    rules, defined, canon, bare = lrm_rules(edition, extractor)
    keywords = annex_b(edition)
    population = []
    for name, rule in sorted(rules.items()):
        kinds = {m: classify_member(m, keywords, defined, canon) for m in rule["members"]}
        if "keyword" not in kinds.values():
            continue
        population.append({
            "edition": edition,
            "production": name,
            "line": rule["line"],
            "cite": rule["cite"],
            "sources": "+".join(rule["sources"]),
            "conflict": rule.get("conflict", ""),
            "members": rule["members"],
            "kinds": kinds,
            "literals": sorted(m for m, k in kinds.items() if k == "literal"),
            "kind": "pure" if all(k == "keyword" for k in kinds.values()) else "mixed",
            "lrm_closure": lrm_closure(name, bare, keywords),
        })
    return population


# --------------------------------------------------------------------------------------------
# The shipped side
# --------------------------------------------------------------------------------------------

class Shipped:
    def __init__(self, grammar_rel: str):
        self.grammar_rel = grammar_rel
        raw = kwf.raw_ast(grammar_rel)
        self.bodies = {rule[0][1]: rule[1:] for rule in raw}
        self.sat = kwf.rule_satisfiability(grammar_rel)
        self.literals = kwf.keyword_literals(REPO_ROOT / grammar_rel)
        self.spellings = set(self.literals.values())
        self._parsed: dict[str, object] = {}

    def body(self, rule: str):
        if rule not in self._parsed:
            self._parsed[rule] = kwf.parse_body(self.bodies[rule])
        return self._parsed[rule]

    def variants(self, production: str) -> list[str]:
        names = [production] + [f"{production}_{s}" for s in VARIANT_SUFFIXES]
        return [n for n in names if n in self.bodies]

    def members(self, rule: str, profile: str, seen: set[str] | None = None) -> set[str]:
        """Keyword spellings reachable through BARE single-reference alternatives, LIVE under
        the profile.  Strict on purpose: `kw_x kw_y?` is not a bare member `x`, so a rule that
        spells a member that way surfaces as a row to adjudicate rather than vanishing."""
        seen = set() if seen is None else seen
        if rule in seen or rule not in self.bodies:
            return set()
        seen.add(rule)
        if profile not in self.sat.get(rule, ()):
            return set()
        live = {r for r, ps in self.sat.items() if profile in ps}
        return self._alt_members(self.body(rule), profile, live, seen)

    def _alt_members(self, node, profile: str, live: set[str], seen: set[str]) -> set[str]:
        out: set[str] = set()
        for branch in node[1]:
            if not all(kwf._live(item, live) for item in branch):
                continue
            items = [it for it in branch if it[0] != "look"]
            if len(items) != 1:
                continue
            item = items[0]
            if item[0] == "ref":
                if item[1] in self.literals:
                    out.add(self.literals[item[1]])
                else:
                    out |= self.members(item[1], profile, seen)
            elif item[0] == "group":
                out |= self._alt_members(item[1], profile, live, seen)
        return out


# --------------------------------------------------------------------------------------------
# The join
# --------------------------------------------------------------------------------------------

def census(grammar_rel: str) -> tuple[list[dict], list[dict], dict]:
    extractor = gap.load_extractor()
    shipped = Shipped(grammar_rel)
    rows: list[dict] = []
    productions: list[dict] = []
    for edition in EDITIONS:
        profile = EDITIONS[edition][2]
        for prod in lrm_population(edition, extractor):
            variants = shipped.variants(prod["production"])
            keyword_members = [m for m in prod["members"] if prod["kinds"][m] == "keyword"]
            entry = {
                "edition": edition, "profile": profile, "production": prod["production"],
                "line": prod["line"], "cite": prod["cite"], "sources": prod["sources"],
                "kind": prod["kind"], "members": prod["members"], "literals": prod["literals"],
                "keyword_members": keyword_members, "variants": variants,
                "closure": set(), "extra": [], "status": "unmatched" if not variants else "",
            }
            if variants:
                closure: set[str] = set()
                for v in variants:
                    closure |= shipped.members(v, profile)
                entry["closure"] = closure
                entry["extra"] = sorted(closure - prod["lrm_closure"])
                verdicts = []
                for member in keyword_members:
                    if member in closure:
                        verdict = V_PRESENT
                    elif member in shipped.spellings:
                        verdict = V_MISSING
                    else:
                        verdict = V_MISSING_NO_TERMINAL
                    verdicts.append(verdict)
                    rows.append({
                        "edition": edition, "profile": profile,
                        "production": prod["production"], "line": prod["line"],
                        "cite": prod["cite"], "kind": prod["kind"], "member": member,
                        "verdict": verdict, "shipped_rules": ";".join(variants),
                    })
                entry["status"] = ("complete" if all(v == V_PRESENT for v in verdicts)
                                   else "INCOMPLETE")
            productions.append(entry)

    stats = {
        "grammar_rel": grammar_rel,
        "productions": len(productions),
        "matched": sum(p["status"] != "unmatched" for p in productions),
        "unmatched": sum(p["status"] == "unmatched" for p in productions),
        "incomplete": sum(p["status"] == "INCOMPLETE" for p in productions),
        "keyword_members": len(rows),
        "present": sum(r["verdict"] == V_PRESENT for r in rows),
        "missing": sum(r["verdict"] == V_MISSING for r in rows),
        "missing_no_terminal": sum(r["verdict"] == V_MISSING_NO_TERMINAL for r in rows),
        "extra": sum(len(p["extra"]) for p in productions),
        "literal_members": sum(len(p["literals"]) for p in productions),
        "per_edition": {},
    }
    for edition in EDITIONS:
        ps = [p for p in productions if p["edition"] == edition]
        rs = [r for r in rows if r["edition"] == edition]
        stats["per_edition"][edition] = {
            "productions": len(ps),
            "matched": sum(p["status"] != "unmatched" for p in ps),
            "unmatched": sum(p["status"] == "unmatched" for p in ps),
            "keyword_members": len(rs),
            "present": sum(r["verdict"] == V_PRESENT for r in rs),
            "missing": sum(r["verdict"] in MISSING_VERDICTS for r in rs),
            "extra": sum(len(p["extra"]) for p in ps),
        }
    return rows, productions, stats


# --------------------------------------------------------------------------------------------
# Rendering
# --------------------------------------------------------------------------------------------

def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def raw_ast_digest(grammar_rel: str) -> str:
    proc = subprocess.run([str(IDENTITY), "--digest", "ebnf_raw_ast", grammar_rel],
                          capture_output=True, text=True, cwd=REPO_ROOT)
    if proc.returncode != 0:
        raise Refused(f"ebnf_raw_ast digest of `{grammar_rel}` failed: "
                      f"{(proc.stderr or '').strip()[:300]}")
    return proc.stdout.strip().splitlines()[-1]


def headline(stats: dict) -> str:
    return (f"{HEADLINE} editions={len(EDITIONS)} productions={stats['productions']} "
            f"matched={stats['matched']} unmatched={stats['unmatched']} "
            f"keyword_members={stats['keyword_members']} present={stats['present']} "
            f"missing={stats['missing']} missing_no_terminal={stats['missing_no_terminal']} "
            f"incomplete_productions={stats['incomplete']} extra_members={stats['extra']} "
            f"literal_members={stats['literal_members']}")


def render_tsv(rows: list[dict], productions: list[dict]) -> str:
    out = ["\t".join(["edition", "profile", "production", "lrm_line", "kind", "member",
                      "verdict", "shipped_rules"])]
    for r in rows:
        out.append("\t".join([r["edition"], r["profile"], r["production"], str(r["line"]),
                              r["kind"], r["member"], r["verdict"], r["shipped_rules"]]))
    for p in productions:
        if p["status"] == "unmatched":
            out.append("\t".join([p["edition"], p["profile"], p["production"], str(p["line"]),
                                  p["kind"], "*", "unmatched_production", ""]))
    return "\n".join(out) + "\n"




def render_md(rows: list[dict], productions: list[dict], stats: dict) -> str:
    md = [
        "# LRM keyword-enumeration membership census — the `net_type` class",
        "",
        "Derived by `stimuli/sv/lrm_keyword_enumeration_census.py`; do not hand-edit. Re-run it",
        "in gate mode and diff.",
        "",
        "**The question.** For every LRM production whose alternatives are all bare tokens and",
        "that names at least one reserved keyword — `net_type ::= supply0 | supply1 | … | wor`",
        "— does the shipped grammar's rule for that production still reach EVERY keyword",
        "member under the profile that edition is released as? `net_type` shipped ten of",
        "twelve for the whole life of the flattened grammar (`SV-CORPUS-GRAD.13n`, ledger",
        "`SV-0075`) and nothing structural noticed: every name resolved, every existing rule",
        "was witnessed, and the corpus only saw it where a vendored file spelled `tri0`.",
        "",
        "## Instrument identity",
        "",
        "Re-hash these inputs; if any hash differs from the row below, **this report no longer",
        "describes your tree** and the honest act is to re-measure, not to quote.",
        "",
        "| input | sha256 |",
        "| --- | --- |",
        f"| `{stats['grammar_rel']}` (ebnf_raw_ast) | `{stats['grammar_raw_ast_sha256']}` |",
    ]
    for edition, (snap_rel, annex_b_rel, _) in EDITIONS.items():
        if ANNEX_A[edition] == annex_b_rel:
            # 1800-2017: the splitter filed Annex A and Annex B in ONE file (TOOLBOX 5.8).
            md.append(f"| `{ANNEX_A[edition]}` (Annex A + Annex B, {edition}) | "
                      f"`{stats['input_sha256'][ANNEX_A[edition]]}` |")
        else:
            md.append(f"| `{ANNEX_A[edition]}` (Annex A, {edition}) | `{stats['input_sha256'][ANNEX_A[edition]]}` |")
            md.append(f"| `{annex_b_rel}` (Annex B, {edition}) | `{stats['input_sha256'][annex_b_rel]}` |")
        md.append(f"| `{snap_rel}` (snapshot, {edition}) | `{stats['input_sha256'][snap_rel]}` |")
    md += [
        "",
        "## Headline",
        "",
        "```text",
        headline(stats),
        "```",
        "",
        "| edition | judged under | productions in population | matched by name | unmatched | keyword members | present | MISSING | extra (informational) |",
        "|---|---|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for edition, e in stats["per_edition"].items():
        md.append(f"| {edition} | `{EDITIONS[edition][2]}` | {e['productions']} | {e['matched']} | "
                  f"{e['unmatched']} | {e['keyword_members']} | {e['present']} | "
                  f"**{e['missing']}** | {e['extra']} |")

    missing = [r for r in rows if r["verdict"] in MISSING_VERDICTS]
    md += [
        "",
        "## Every MISSING member — the rows to adjudicate",
        "",
        "`MISSING` = a `kw_*` terminal spells the word but the rule named for this production",
        "cannot reach it under the profile. `MISSING_NO_TERMINAL` = no terminal spells it at",
        "all. ⛔ Text-level triage, not a parse verdict: each row is settled by the release",
        "probe with a two-sided bisect (the input rejects; substituting a sibling member",
        "accepts), exactly as `.13k`(b) settled `tri0`.",
        "",
        "| edition | production | LRM citation | member | verdict | shipped rule(s) consulted |",
        "|---|---|---|---|---|---|",
    ]
    if missing:
        for r in missing:
            md.append(f"| {r['edition']} | `{r['production']}` | {r['cite']} | "
                      f"`{r['member']}` | `{r['verdict']}` | `{r['shipped_rules']}` |")
    else:
        md.append("| _(none)_ | | | | | |")

    unmatched = [p for p in productions if p["status"] == "unmatched"]
    md += [
        "",
        "## Unmatched productions — no shipped rule by name or profile variant (NOT scored)",
        "",
        "The shipped grammar is a hand-annotated flattened synthesis, so a production the LRM",
        "names may be lifted, renamed or folded into a neighbour. These rows are an",
        "ADJUDICATION worklist: each needs a reader to name the shipped rule that carries the",
        "enumeration (or to show the members are carried some other way) before a verdict",
        "exists. They are listed so the bound is visible, not to be counted as defects.",
        "",
        "| edition | production | LRM citation | kind | members |",
        "|---|---|---|---|---|",
    ]
    if unmatched:
        for p in unmatched:
            members = " ".join(f"`{m}`" for m in p["members"])
            md.append(f"| {p['edition']} | `{p['production']}` | {p['cite']} | "
                      f"{p['kind']} | {members} |")
    else:
        md.append("| _(none)_ | | | | |")

    extras = [p for p in productions if p["extra"]]
    md += [
        "",
        "## Extra members — the shipped closure reaches a keyword the LRM enumeration does not name (informational, NOT scored)",
        "",
        "⛔ Deliberately not a verdict. *The annex does not derive it* is a KNOWN-BAD classifier",
        "for over-acceptance (`LRM-GRAMMAR-FIDELITY.1c`: three constructs flagged that way were",
        "all legal), the closure follows the shipped grammar's own profile-variant folding, and",
        "the `verilog_2005` keyword-admission direction is owned by",
        "`stimuli/sv/v2005_keyword_faithfulness_census.py` (TOOLBOX 5.11), which adjudicates",
        "each candidate on the release probe with three legs.",
        "",
        "| edition | production | extra members in the shipped closure |",
        "|---|---|---|",
    ]
    if extras:
        for p in extras:
            md.append(f"| {p['edition']} | `{p['production']}` | "
                      + " ".join(f"`{m}`" for m in p["extra"]) + " |")
    else:
        md.append("| _(none)_ | | |")

    md += [
        "",
        "## Every production in the population",
        "",
        "`sources` says which LRM surface carried the enumeration: `annex_a` (the markdown the",
        "shipped grammar descends from), `snapshot` (the tracked extraction snapshot), or both.",
        "`literal` members are bare words the LRM writes that are neither keywords nor",
        "productions (`none` in `default_nettype_value`); they are listed, never scored.",
        "",
        "| edition | production | LRM citation | sources | kind | status | keyword members | literal members | shipped rule(s) |",
        "|---|---|---|---|---|---|---:|---|---|",
    ]
    for p in productions:
        shipped = " ".join(f"`{v}`" for v in p["variants"]) or "_(none)_"
        literals = " ".join(f"`{m}`" for m in p["literals"]) or ""
        md.append(f"| {p['edition']} | `{p['production']}` | {p['cite']} | {p['sources']} | "
                  f"{p['kind']} | {p['status']} | {len(p['keyword_members'])} | {literals} | {shipped} |")

    md += [
        "",
        "## Honest bound",
        "",
        "- **Text-level.** A `present` verdict means the shipped rule's closure REACHES the",
        "  terminal under the profile; it does not prove an input using the keyword parses in",
        "  every context the LRM allows. A `MISSING` verdict is a triage row for the probe.",
        "- **Bare-member closure, strict on purpose.** Only an alternative that is exactly one",
        "  reference (or one parenthesised group) contributes; `kw_x kw_y?` does not count as",
        "  `x`. That errs toward reporting, which is the direction nothing else catches.",
        "- **Name-matched.** A production the shipped grammar renamed or lifted is `unmatched`,",
        "  never `MISSING`. The unmatched list IS the bound on what this census could not see.",
        "- **Source provenance.** The LRM side is the Annex A markdown first and the tracked",
        "  extraction snapshot second, and the two are held equal on every shared enumeration.",
        "  A production that IEEE defines only in a clause body and that neither surface carries",
        "  is the business of `stimuli/sv/lrm_annex_a_gap_census.py` (TOOLBOX 5.8).",
        "- **One direction.** Under-acceptance only. Over-acceptance candidates are printed and",
        "  not scored (see *Extra members*).",
        "",
    ]
    return "\n".join(md)


# --------------------------------------------------------------------------------------------
# Driver
# --------------------------------------------------------------------------------------------

def run(grammar_rel: str) -> tuple[str, str, dict, list[dict], list[dict]]:
    rows, productions, stats = census(grammar_rel)
    stats["grammar_raw_ast_sha256"] = raw_ast_digest(grammar_rel)
    stats["input_sha256"] = {}
    for edition, (snap_rel, annex_b_rel, _) in EDITIONS.items():
        stats["input_sha256"][ANNEX_A[edition]] = sha256(REPO_ROOT / ANNEX_A[edition])
        stats["input_sha256"][snap_rel] = sha256(REPO_ROOT / snap_rel)
        stats["input_sha256"][annex_b_rel] = sha256(REPO_ROOT / annex_b_rel)
    return render_tsv(rows, productions), render_md(rows, productions, stats), stats, rows, productions


def self_test() -> int:
    """Refusal and control arms.  Every RED arm is a real mutation of the real inputs, and the
    GREEN arm is the real tree, so a red arm cannot be red for the copy."""
    scratch = REPO_ROOT / "rust" / "target" / "lrm_keyword_enumeration_selftest"
    if scratch.exists():
        shutil.rmtree(scratch)
    scratch.mkdir(parents=True)
    checks: list[tuple[str, bool, str]] = []

    def arm(name: str, ok: bool, detail: str) -> None:
        checks.append((name, ok, detail))
        print(f"  {'✓' if ok else '✗'} {name} — {detail}")

    # GREEN control: the real tree, and `net_type` is complete on all three editions since
    # `.13n` landed.  Also the determinism arm: two derivations are byte-identical.
    tsv1, md1, stats, rows, productions = run(SHIPPED_GRAMMAR_REL)
    tsv2, md2, _, _, _ = run(SHIPPED_GRAMMAR_REL)
    arm("determinism", tsv1 == tsv2 and md1 == md2, "two derivations are byte-identical")
    net = [p for p in productions if p["production"] == "net_type"]
    arm("green: net_type complete on every edition",
        len(net) == 3 and all(p["status"] == "complete" for p in net),
        f"{[(p['edition'], p['status']) for p in net]}")
    arm("green: tri0/tri1 present on every edition",
        all(r["verdict"] == V_PRESENT for r in rows
            if r["production"] == "net_type" and r["member"] in ("tri0", "tri1")),
        "6 of 6 rows present")

    # RED arm 1 — replay `.13n` as it was before the fix: the two branches removed, the
    # terminals kept.  Must report `MISSING` for tri0/tri1 on all three editions.
    src = (REPO_ROOT / SHIPPED_GRAMMAR_REL).read_text()
    branch_re = re.compile(r"^\s*\| kw_tri[01]\s+-> \{kind: \"tri[01]\"\}\n", re.MULTILINE)
    mutated = branch_re.sub("", src)
    if mutated == src:
        raise Refused("self-test could not find the `kw_tri0`/`kw_tri1` net_type branches to remove")
    red1 = scratch / "net_type_without_tri01.ebnf"
    red1.write_text(mutated)
    _, _, _, rows_r1, prods_r1 = run(str(red1.relative_to(REPO_ROOT)))
    got = sorted((r["edition"], r["member"], r["verdict"]) for r in rows_r1
                 if r["production"] == "net_type" and r["member"] in ("tri0", "tri1"))
    arm("red: branches removed -> MISSING on all three editions",
        len(got) == 6 and all(v == V_MISSING for _, _, v in got),
        f"{got}")
    arm("red: nothing else moved",
        sum(r["verdict"] != V_PRESENT for r in rows_r1) == sum(r["verdict"] != V_PRESENT for r in rows) + 6,
        "exactly six rows changed verdict")

    # RED arm 2 — remove the terminals too: the strongest signal.
    terminal_re = re.compile(r"^kw_tri[01]\s*:=\s*trivia\s*/tri[01]\\b/\s*\n", re.MULTILINE)
    mutated2 = terminal_re.sub("", mutated)
    if mutated2 == mutated:
        raise Refused("self-test could not find the `kw_tri0`/`kw_tri1` terminal definitions")
    red2 = scratch / "net_type_without_tri01_terminals.ebnf"
    red2.write_text(mutated2)
    _, _, _, rows_r2, _ = run(str(red2.relative_to(REPO_ROOT)))
    got2 = sorted((r["edition"], r["member"], r["verdict"]) for r in rows_r2
                  if r["production"] == "net_type" and r["member"] in ("tri0", "tri1"))
    arm("red: terminals removed -> MISSING_NO_TERMINAL",
        len(got2) == 6 and all(v == V_MISSING_NO_TERMINAL for _, _, v in got2),
        f"{got2}")

    # Refusal arms on the LRM side.
    try:
        pin_annex_b(["a", "b"], "1800-2017", REPO_ROOT / EDITIONS["1800-2017"][1])
        arm("refusal: Annex B count", False, "a two-word list was accepted")
    except Refused as exc:
        arm("refusal: Annex B count", "expected exactly 248" in str(exc), str(exc)[:80])

    extractor = gap.load_extractor()
    canon = make_canonicalizer({"expression", "strength0"}, {"expression", "strength0"})
    arm("classify: keyword vs nonterminal vs system vs symbol vs literal",
        [classify_member(t, {"tri0"}, {"expression", "strength0"}, canon)
         for t in ("tri0", "expression", "expression32", "$setup", "x", "none")]
        == ["keyword", "nonterminal", "nonterminal", "system_name", "symbol", "literal"],
        "six tokens, five classes")
    arm("canonicalize: footnote digits stripped, real digits kept",
        [canon(n) for n in ("expression32", "strength0", "strength1", "class_item_qualifier10")]
        == ["expression", "strength0", "strength1", "class_item_qualifier10"],
        "`strength0` survives; an unreferenced unknown keeps its digits")

    # The raw Annex A reader equals the extractor's own `extract_rules` after canonicalisation,
    # for all three editions — names AND bodies.
    for edition in EDITIONS:
        mine: dict[str, list[str]] = {}
        for raw_name, _, body in annex_a_rules_raw(edition, extractor):
            key = extractor.canonicalize_rule_name(raw_name)
            mine.setdefault(key, [])
            if body:
                mine[key].append(body)
        theirs = extractor.extract_rules(REPO_ROOT / ANNEX_A[edition])
        arm(f"reader parity: {edition} Annex A reader == extract_rules",
            dict(theirs) == mine, f"{len(theirs)} canonical rules")

    # Refusal: an unterminated snapshot rule.
    snap_rel = EDITIONS["1364-2005"][0]
    bad = (REPO_ROOT / snap_rel).read_text().replace(
        "net_type ::=\n    supply0 | supply1 | tri | triand | trior | tri0 | tri1 | uwire | wire | wand | wor ;",
        "net_type ::=\n    supply0 | supply1 | tri | triand | trior | tri0 | tri1 | uwire | wire | wand | wor")
    if bad == (REPO_ROOT / snap_rel).read_text():
        raise Refused("self-test could not find the 1364-2005 net_type rule to unterminate")
    bad_path = scratch / "unterminated_2005.ebnf"
    bad_path.write_text(bad)
    saved = EDITIONS["1364-2005"]
    EDITIONS["1364-2005"] = (str(bad_path.relative_to(REPO_ROOT)), saved[1], saved[2])
    try:
        snapshot_rules("1364-2005")
        arm("refusal: unterminated snapshot rule", False, "was accepted")
    except Refused as exc:
        arm("refusal: unterminated snapshot rule", "no ` ;` terminator" in str(exc), str(exc)[:80])
    finally:
        EDITIONS["1364-2005"] = saved

    # Gate mode disagrees with a tampered tracked artifact.
    tracked = ARTIFACT_DIR / "census.tsv"
    if tracked.exists():
        arm("gate: tracked artifact equals a fresh derivation", tracked.read_text() == tsv1,
            "byte-identical" if tracked.read_text() == tsv1 else "DIFFERS — run --write after review")

    shutil.rmtree(scratch)
    failed = [c for c in checks if not c[1]]
    print(f"{HEADLINE} self-test arms={len(checks)} failed={len(failed)}")
    return 1 if failed else 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--write", action="store_true",
                    help="write the artifact instead of comparing against the tracked copy")
    ap.add_argument("--self-test", action="store_true", help="fire the refusal and control arms")
    ap.add_argument("--grammar", default=None,
                    help="control only: a repo-relative grammar path to census instead of the "
                         "shipped grammar (forbids --write)")
    args = ap.parse_args()

    try:
        if args.self_test:
            return self_test()
        grammar_rel = args.grammar or SHIPPED_GRAMMAR_REL
        if args.grammar and args.write:
            print(f"{HEADLINE} REFUSED — --write is not available under --grammar (a control "
                  "run can never become the tracked baseline)", file=sys.stderr)
            return 2
        tsv, md, stats, _, _ = run(grammar_rel)
        print(headline(stats))
        for edition, e in stats["per_edition"].items():
            print(f"  {edition:>9} under {EDITIONS[edition][2]:<12} productions={e['productions']} "
                  f"matched={e['matched']} unmatched={e['unmatched']} members={e['keyword_members']} "
                  f"present={e['present']} missing={e['missing']} extra={e['extra']}")
        if args.grammar:
            return 0
        ARTIFACT_DIR.mkdir(parents=True, exist_ok=True)
        tsv_path, md_path = ARTIFACT_DIR / "census.tsv", ARTIFACT_DIR / "census.md"
        if args.write:
            tsv_path.write_text(tsv)
            md_path.write_text(md)
            print(f"{HEADLINE} wrote {tsv_path.relative_to(REPO_ROOT)} and "
                  f"{md_path.relative_to(REPO_ROOT)}")
            return 0
        if not tsv_path.exists() or not md_path.exists():
            print(f"{HEADLINE} no tracked artifact under {ARTIFACT_DIR.relative_to(REPO_ROOT)}; "
                  "review the headline and run --write", file=sys.stderr)
            return 1
        if tsv_path.read_text() != tsv or md_path.read_text() != md:
            print(f"{HEADLINE} DRIFT — the tracked artifact is not what this tree derives; "
                  "re-run with --write after reviewing the diff", file=sys.stderr)
            return 1
        print(f"{HEADLINE} tracked artifact equals a fresh derivation")
        return 0
    except Refused as exc:
        print(f"{HEADLINE} REFUSED — {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
