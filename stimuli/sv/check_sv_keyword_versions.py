#!/usr/bin/env python3
"""check_sv_keyword_versions.py — the preprocessor's §22.14 version tables ARE the LRMs' tables.

`SVPP-EXPANSION.8` (`PGEN-SVPP-EXPANSION-0022`). IEEE 1800-2017 §22.14 makes
`` `begin_keywords "version" `` select WHICH identifiers are reserved inside a region, from the
standard's own per-version tables (Table 22-1 … Table 22-6 plus the "1364-2001-noconfig"
exclusion list). `rust/src/sv_keyword_versions.rs` carries those tables so the preprocessor can
render a token that is a keyword of the PARSE PROFILE but not of the REGION as an escaped
identifier (§5.6.1/§5.6.2). This instrument DERIVES the tables from the in-repo LRM text and
compares them against that file, both directions — the tables are never typed by hand.

  gate mode (default)   derive + compare; exit 1 naming every word missing from or foreign to a table
  --write               regenerate the derived block of rust/src/sv_keyword_versions.rs
  --self-test           a planted missing word, a planted foreign word and a broken split are refused
  --print               print the derived tables (one `name: words` line each) and exit 0

⛔ THE TABLE BODIES ARE DISPLACED AND MERGED BY THE PDF→md EXTRACTION, so the split points are
DERIVED, never assumed — each by an oracle that is blind to the others:

  Table 22-1                  = the first multi-row body of 1800-2017 §22.14, CHECKED EQUAL to
                                IEEE 1364-2005 §19.11 Table 19-2 (a different standard, a different
                                PDF, a different extraction).
  Table 22-2 ∪ Table 22-3     = the second body (22 words). Its two tables are NOT separable by any
                                property of the 1800 text — the concatenation is strictly
                                alphabetical, so every split point is admissible — and they are
                                split by the 1364-2005 document instead:
                                  22-2 = Table 19-3 (full 1364-2001) − Table 19-2,
                                  22-3 = Table 19-4 (full 1364-2005) − Table 19-3.
                                Their union is then checked equal to the 1800 body.
  Table 22-4 ∪ Table 22-5     = the third body (120 words). No second document carries these, so the
                                split is the UNIQUE one that leaves both tables strictly increasing:
                                the body has EXACTLY ONE descent (`within` → `accept_on`), and every
                                other table in both documents is strictly increasing, so a split
                                anywhere else leaves a table that is not. A body with a descent count
                                other than (tables − 1) is REFUSED rather than guessed.
  Table 22-6                  = the fourth body (4 words).
  noconfig exclusions         = the run that follows "excluded from the reserved list in Table 22-N:",
                                taken from BOTH documents and checked identical, and checked ⊆ 22-2.

  Σ of all six tables         = IEEE 1800-2017 Annex B Table B.1 (248), which is §22.14.9's own
                                sentence: "The full set of reserved identifiers … is listed in
                                Annex B, which reflects the combination of all version tables."
  1800-2017 §22.14 == 1800-2023 §22.14, word for word (two editions, two PDFs).

Sources (never inferred, always these files):
  1800-2017 §22.14   docs/systemverilog/2017/md/section-22-compiler-directives.md
  1800-2023 §22.14   docs/systemverilog/2023/md/section-22-compiler-directives.md
  1364-2005 §19.11   docs/verilog/2005/md/section-19-compiler-directives.md
  1800-2017 Annex B  docs/systemverilog/2017/md/section-41-data-read-api.md
  the profile map    grammars/systemverilog.ebnf (`@profile_alias` + `keywords_version_specifier_*`)
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RUST = ROOT / "rust/src/sv_keyword_versions.rs"
GRAMMAR = ROOT / "grammars/systemverilog.ebnf"

SV2017 = ROOT / "docs/systemverilog/2017/md/section-22-compiler-directives.md"
SV2023 = ROOT / "docs/systemverilog/2023/md/section-22-compiler-directives.md"
V2005 = ROOT / "docs/verilog/2005/md/section-19-compiler-directives.md"
ANNEX_B = ROOT / "docs/systemverilog/2017/md/section-41-data-read-api.md"

TOKEN = re.compile(r"[a-z_][a-z0-9_$]*")
# The PDF's page furniture inside a table: running headers, the licence line, a bare page number.
FURNITURE = re.compile(
    r"^(Authorized licensed use|IEEE Std |IEEE Standard for|IEEE STANDARD|Copyright ©|Std 1364|IEEE$|\d+$)"
)
EXCLUSION_SENTENCE = "excluded from the reserved list"

# The derived block of the Rust file, rewritten verbatim by --write.
BLOCK_OPEN = "// <derived-tables>"
BLOCK_CLOSE = "// </derived-tables>"

TABLE_NAMES = ["TABLE_22_1", "TABLE_22_2", "TABLE_22_3", "TABLE_22_4", "TABLE_22_5", "TABLE_22_6"]


def region(path, start_pat, end_pat):
    """The lines of one clause, from its heading to the next clause at the same level."""
    lines = path.read_text(encoding="utf-8").split("\n")
    start = next(i for i, line in enumerate(lines) if re.match(start_pat, line))
    end = next((i for i, line in enumerate(lines) if i > start and re.match(end_pat, line)), len(lines))
    return lines[start:end]


def scan(lines, caption_pat):
    """(captions in order, multi-row bodies in order, the noconfig exclusion list).

    A BODY is a maximal run of identifier-only lines with page furniture removed. A run of ONE is a
    code example's stray `endmodule`, never a table — every real table body in both documents has
    at least four rows. The run that follows the exclusion SENTENCE is that list, not a body.
    """
    captions, bodies, exclusions = [], [], None
    run, run_is_exclusion, seen_caption = [], False, False

    def flush():
        nonlocal run, run_is_exclusion, exclusions
        if run:
            if run_is_exclusion:
                exclusions = run
            elif seen_caption and len(run) > 1:
                bodies.append(run)
        run, run_is_exclusion = [], False

    for raw in lines:
        text = raw.strip()
        if not text or FURNITURE.match(text):
            continue
        if TOKEN.fullmatch(text):
            run.append(text)
            continue
        flush()
        if re.match(caption_pat, text):
            captions.append(text)
            seen_caption = True
        if EXCLUSION_SENTENCE in text:
            run_is_exclusion = True
    flush()
    return captions, bodies, exclusions


def descents(words):
    return [i for i in range(len(words) - 1) if words[i] >= words[i + 1]]


def split_by_monotonicity(body, parts, label):
    """Split one merged body into `parts` strictly-increasing tables, or raise."""
    cuts = descents(body)
    if len(cuts) != parts - 1:
        raise ValueError(
            f"{label}: {len(body)} words with {len(cuts)} descent(s) cannot be split into {parts} "
            f"strictly-increasing tables — the extraction changed shape; re-read the clause"
        )
    out, prev = [], 0
    for cut in cuts:
        out.append(body[prev:cut + 1])
        prev = cut + 1
    out.append(body[prev:])
    return out


def annex_b():
    """IEEE 1800-2017 Annex B Table B.1 — the union oracle (§22.14.9's own sentence)."""
    lines = ANNEX_B.read_text(encoding="utf-8").split("\n")
    start = next(i for i, line in enumerate(lines) if line.startswith("Table B.1—Reserved keywords"))
    end = next((i for i, line in enumerate(lines) if i > start and line.startswith("Annex C")), len(lines))
    words = []
    for raw in lines[start + 1:end]:
        text = raw.strip()
        if not text or raw.startswith("Table B.1") or FURNITURE.match(text):
            continue
        if not TOKEN.fullmatch(text):
            if words:
                break
            continue
        words.append(text)
    return words


def derive():
    """The six tables + the exclusion list, with every cross-document check applied.

    Returns (tables, problems) — `tables` is a dict name -> sorted word list; `problems` is the
    list of failed checks (empty when every oracle agreed).
    """
    problems = []

    caps17, bodies17, excl17 = scan(region(SV2017, r"^### 22\.14 ", r"^### 22\.15"), r"^Table 22-\d")
    caps23, bodies23, excl23 = scan(region(SV2023, r"^### 22\.14 ", r"^### 22\.15"), r"^Table 22-\d")
    caps05, bodies05, excl05 = scan(region(V2005, r"^### 19\.11 ", r"^### 19\.12|^## "), r"^Table 19-\d")

    if len(caps17) != 6 or len(bodies17) != 4:
        raise ValueError(
            f"1800-2017 §22.14: expected 6 table captions and 4 merged bodies, read "
            f"{len(caps17)} and {len(bodies17)} — the extraction changed shape"
        )
    if len(caps05) != 3 or len(bodies05) != 3:
        raise ValueError(
            f"1364-2005 §19.11: expected 3 table captions and 3 bodies, read "
            f"{len(caps05)} and {len(bodies05)} — the extraction changed shape"
        )

    # two editions, two PDFs, two extractions — word for word
    if bodies17 != bodies23:
        problems.append(
            "1800-2017 and 1800-2023 §22.14 table bodies differ: "
            + "; ".join(
                f"body {i}: only-2017={sorted(set(a) - set(b))} only-2023={sorted(set(b) - set(a))}"
                for i, (a, b) in enumerate(zip(bodies17, bodies23))
                if a != b
            )
        )
    if excl17 != excl23:
        problems.append(f"1800-2017 noconfig exclusions {excl17} != 1800-2023 {excl23}")
    if excl17 != excl05:
        problems.append(f"1800-2017 noconfig exclusions {excl17} != 1364-2005 §19.11 {excl05}")

    kw1995_2005doc, kw2001_2005doc, kw2005_2005doc = (set(b) for b in bodies05)

    # Table 22-1 — checked against the other standard's own full 1364-1995 list
    table_22_1 = bodies17[0]
    if set(table_22_1) != kw1995_2005doc:
        problems.append(
            "Table 22-1 != IEEE 1364-2005 Table 19-2: "
            f"only-1800={sorted(set(table_22_1) - kw1995_2005doc)} "
            f"only-1364={sorted(kw1995_2005doc - set(table_22_1))}"
        )

    # Tables 22-2 / 22-3 — the 1800 body cannot split itself, the 1364-2005 document splits it
    table_22_2 = sorted(kw2001_2005doc - kw1995_2005doc)
    table_22_3 = sorted(kw2005_2005doc - kw2001_2005doc)
    if sorted(bodies17[1]) != sorted(table_22_2 + table_22_3):
        problems.append(
            "Table 22-2 ∪ 22-3 (1800) != (1364-2001 − 1364-1995) ∪ (1364-2005 − 1364-2001): "
            f"only-1800={sorted(set(bodies17[1]) - set(table_22_2 + table_22_3))} "
            f"only-1364={sorted(set(table_22_2 + table_22_3) - set(bodies17[1]))}"
        )

    # Tables 22-4 / 22-5 — no second document; the unique strictly-increasing split
    table_22_4, table_22_5 = split_by_monotonicity(bodies17[2], 2, "Table 22-4 ∪ 22-5")
    table_22_6 = bodies17[3]

    tables = {
        "TABLE_22_1": sorted(table_22_1),
        "TABLE_22_2": sorted(table_22_2),
        "TABLE_22_3": sorted(table_22_3),
        "TABLE_22_4": sorted(table_22_4),
        "TABLE_22_5": sorted(table_22_5),
        "TABLE_22_6": sorted(table_22_6),
        "NOCONFIG_EXCLUSIONS": sorted(excl17 or []),
    }

    # the tables partition — no word may appear in two of them
    seen = {}
    for name in TABLE_NAMES:
        for word in tables[name]:
            if word in seen:
                problems.append(f"'{word}' appears in both {seen[word]} and {name}")
            seen[word] = name

    # §22.14.9: the union of the version tables IS Annex B Table B.1
    union = set(seen)
    b1 = set(annex_b())
    if union != b1:
        problems.append(
            f"Σ(Table 22-1 … 22-6) != Annex B Table B.1: "
            f"only-tables={sorted(union - b1)} only-annexB={sorted(b1 - union)}"
        )

    # the exclusion list is drawn from Table 22-2
    stray = sorted(set(tables["NOCONFIG_EXCLUSIONS"]) - set(tables["TABLE_22_2"]))
    if stray:
        problems.append(f"noconfig exclusions not in Table 22-2: {stray}")

    return tables, problems


# ── the Rust side ────────────────────────────────────────────────────────────────────────────────

def rust_arrays(text):
    """name -> word list, read from the derived block of the Rust module."""
    body = text.split(BLOCK_OPEN, 1)[1].split(BLOCK_CLOSE, 1)[0]
    out = {}
    for match in re.finditer(r"const (\w+): &\[&str\] = &\[(.*?)\];", body, re.S):
        out[match.group(1)] = re.findall(r'"([^"]+)"', match.group(2))
    return out


def render_block(tables, aliases, specifiers):
    lines = [BLOCK_OPEN, "// ⛔ DERIVED — regenerate with:"]
    lines.append("//     python3 stimuli/sv/check_sv_keyword_versions.py --write")
    lines.append("// held equal to the LRM text on every commit by the `SV-KEYWORD-VERSIONS` doctrine.")
    captions = {
        "TABLE_22_1": "IEEE 1800-2017 Table 22-1 — the IEEE 1364-1995 reserved keywords (§22.14.2).",
        "TABLE_22_2": "Table 22-2 — what \"1364-2001\" ADDS to \"1364-1995\" (§22.14.3).",
        "TABLE_22_3": "Table 22-3 — what \"1364-2005\" ADDS to \"1364-2001\" (§22.14.5).",
        "TABLE_22_4": "Table 22-4 — what \"1800-2005\" ADDS to \"1364-2005\" (§22.14.6).",
        "TABLE_22_5": "Table 22-5 — what \"1800-2009\" ADDS to \"1800-2005\" (§22.14.7).",
        "TABLE_22_6": "Table 22-6 — what \"1800-2012\" ADDS to \"1800-2009\" (§22.14.8).",
        "NOCONFIG_EXCLUSIONS": "§22.14.4 — the identifiers \"1364-2001-noconfig\" EXCLUDES from Table 22-2.",
    }
    for name in TABLE_NAMES + ["NOCONFIG_EXCLUSIONS"]:
        words = tables[name]
        lines.append("")
        plural = "word" if len(words) == 1 else "words"
        lines.append(f"/// {captions[name]} {len(words)} {plural}.")
        lines.append(f"const {name}: &[&str] = &[")
        row = "    "
        for word in words:
            piece = f'"{word}", '
            if len(row) + len(piece) > 99:
                lines.append(row.rstrip())
                row = "    "
            row += piece
        if row.strip():
            lines.append(row.rstrip())
        lines.append("];")
    lines.append("")
    lines.append("/// `PROFILE-ALIAS.2` — every `--profile` spelling `grammars/systemverilog.ebnf` declares,")
    lines.append("/// paired with the keyword version that profile's parser reserves.")
    lines.append("const PROFILE_KEYWORD_VERSION: &[(&str, KeywordVersion)] = &[")
    for spelling, variant in aliases:
        lines.append(f'    ("{spelling}", KeywordVersion::{variant}),')
    lines.append("];")
    lines.append("")
    lines.append("/// The version specifiers each profile's own EDITION defines (§22.14 Syntax 22-10), read from")
    lines.append("/// `keywords_version_specifier_<profile>` in `grammars/systemverilog.ebnf` — the parser refuses")
    lines.append("/// any other spelling, so a region naming one is worth a diagnostic here.")
    lines.append("const ADMITTED_SPECIFIERS: &[(&str, &[&str])] = &[")
    for canonical in sorted(specifiers):
        admitted = ", ".join(f'"{s}"' for s in sorted(specifiers[canonical], reverse=True))
        lines.append(f'    ("{canonical}", &[{admitted}]),')
    lines.append("];")
    lines.append(BLOCK_CLOSE)
    return "\n".join(lines)


PROFILE_VARIANT = {"sv_2017": "Sv2017", "sv_2023": "Sv2023", "verilog_2005": "Verilog2005"}


def grammar_profiles():
    """The declared `--profile` spellings, canonical first, from the grammar itself."""
    text = GRAMMAR.read_text(encoding="utf-8")
    pairs = {canonical: [canonical] for canonical in PROFILE_VARIANT}
    for line in text.split("\n"):
        if not line.startswith("@profile_alias:"):
            continue
        for alias, canonical in re.findall(r'"([^"]+)"\s*:\s*(\w+)', line):
            pairs.setdefault(canonical, []).append(alias)
    out = []
    for canonical in sorted(pairs):
        if canonical not in PROFILE_VARIANT:
            raise ValueError(f"grammar declares profile '{canonical}' with no keyword version")
        for spelling in sorted(set(pairs[canonical])):
            out.append((spelling, PROFILE_VARIANT[canonical]))
    return out


def grammar_specifiers():
    """profile -> the version specifiers `keywords_version_specifier_<profile>` admits."""
    text = GRAMMAR.read_text(encoding="utf-8")
    out = {}
    for suffix, canonical in (("sv_2017", "sv_2017"), ("sv_2023", "sv_2023"), ("v2005", "verilog_2005")):
        line = next(
            (l for l in text.split("\n") if l.startswith(f"keywords_version_specifier_{suffix} :=")),
            None,
        )
        if line is None:
            raise ValueError(f"grammars/systemverilog.ebnf has no keywords_version_specifier_{suffix}")
        out[canonical] = set(re.findall(r"\d{4}-\d{4}(?:-noconfig)?", line))
    return out


def rust_specifiers(text):
    """profile -> the version specifiers the Rust module admits for it (from ADMITTED_SPECIFIERS)."""
    out = {}
    block = re.search(r"const ADMITTED_SPECIFIERS:.*?\n\];", text, re.S)
    if not block:
        raise ValueError("rust/src/sv_keyword_versions.rs has no ADMITTED_SPECIFIERS table")
    for canonical, body in re.findall(r'\("(\w+)",\s*&\[(.*?)\]\)', block.group(0), re.S):
        out[canonical] = set(re.findall(r'"([^"]+)"', body))
    return out


def compare(text, tables, aliases):
    problems = []
    have = rust_arrays(text)
    for name in TABLE_NAMES + ["NOCONFIG_EXCLUSIONS"]:
        want = tables[name]
        got = have.get(name)
        if got is None:
            problems.append(f"{name}: absent from the derived block of rust/src/sv_keyword_versions.rs")
            continue
        if got != want:
            missing = sorted(set(want) - set(got))
            foreign = sorted(set(got) - set(want))
            if missing:
                problems.append(f"{name}: {len(missing)} LRM word(s) NOT in the table: {' '.join(missing)}")
            if foreign:
                problems.append(f"{name}: {len(foreign)} table word(s) NOT in the LRM: {' '.join(foreign)}")
            if not missing and not foreign:
                problems.append(f"{name}: same words, different order — regenerate with --write")
    want_aliases = [f'("{s}", KeywordVersion::{v})' for s, v in aliases]
    got_aliases = re.findall(r'\("([^"]+)",\s*KeywordVersion::(\w+)\)', text.split(BLOCK_OPEN, 1)[1])
    got_aliases = [f'("{s}", KeywordVersion::{v})' for s, v in got_aliases]
    if got_aliases != want_aliases:
        problems.append(
            f"PROFILE_KEYWORD_VERSION != the grammar's @profile_alias declarations: "
            f"grammar={want_aliases} rust={got_aliases}"
        )
    grammar_spec = grammar_specifiers()
    rust_spec = rust_specifiers(text)
    for profile in sorted(grammar_spec):
        if grammar_spec[profile] != rust_spec.get(profile, set()):
            problems.append(
                f"admitted version specifiers for {profile} disagree with "
                f"keywords_version_specifier: grammar={sorted(grammar_spec[profile])} "
                f"rust={sorted(rust_spec.get(profile, set()))}"
            )
    problems += admitted_specifiers_are_subsets(tables, grammar_spec)
    return problems


# The reserved-word set each version selects, derived from the tables the same way the Rust module
# builds it (§22.14.3/.5/.6/.7/.8 accumulate; §22.14.4 is the clause's one subtraction).
VERSION_CHAIN = [
    ("1364-1995", ["TABLE_22_1"]),
    ("1364-2001", ["TABLE_22_1", "TABLE_22_2"]),
    ("1364-2005", ["TABLE_22_1", "TABLE_22_2", "TABLE_22_3"]),
    ("1800-2005", ["TABLE_22_1", "TABLE_22_2", "TABLE_22_3", "TABLE_22_4"]),
    ("1800-2009", ["TABLE_22_1", "TABLE_22_2", "TABLE_22_3", "TABLE_22_4", "TABLE_22_5"]),
    ("1800-2012", TABLE_NAMES),
    ("1800-2017", TABLE_NAMES),
    ("1800-2023", TABLE_NAMES),
]
PROFILE_VERSION = {"sv_2017": "1800-2017", "sv_2023": "1800-2023", "verilog_2005": "1364-2005"}


def reserved_words_of(tables, specifier):
    if specifier == "1364-2001-noconfig":
        words = set(tables["TABLE_22_1"]) | set(tables["TABLE_22_2"])
        return words - set(tables["NOCONFIG_EXCLUSIONS"])
    for name, parts in VERSION_CHAIN:
        if name == specifier:
            words = set()
            for part in parts:
                words |= set(tables[part])
            return words
    raise ValueError(f"no reserved-word set for '{specifier}'")


def admitted_specifiers_are_subsets(tables, grammar_spec):
    """`SVPP-EXPANSION.8` — EVERY SPECIFIER A PROFILE ADMITS RESERVES A SUBSET OF THAT PROFILE.

    ⛔ This is the arm that RULES OUT the one direction the preprocessor cannot render. It can
    UNRESERVE a keyword (§5.6.1: an escaped identifier is not a keyword) and it cannot RESERVE a
    non-keyword — no text can — so a region naming a version that reserves MORE than the parse
    profile would be applied only half way, and that is an OVER-ACCEPTANCE the stage cannot fix.
    The reason it can never bite is structural rather than lucky: a specifier is admitted by a
    profile exactly when that profile's own EDITION lists it (§22.14 Syntax 22-10), and an edition
    lists only versions at or below itself — so `admitted ⇒ subset ⇒ never above`, and the
    directive is a PARSE REJECTION in every case where the warning could fire.

    Measured (2026-09-15, 16 336 corpus rows): 14 occurrences of a specifier above `verilog_2005`
    (`1800-2005` x4, `1800-2009` x2, `1800-2012` x3, `1800-2017` x2, `1800-2023` x5) and every one
    of them is refused by `keywords_version_specifier_v2005`, which lists the four 1364 versions
    only. Under `sv_2017` / `sv_2023` no specifier reserves more at all.

    Held here rather than left as a warning in the runtime, so a future edition (or a vendor
    specifier added to a profile's list) that broke the implication would FAIL this check instead
    of silently turning the warning into a live over-acceptance.
    """
    problems = []
    for profile in sorted(grammar_spec):
        profile_words = reserved_words_of(tables, PROFILE_VERSION[profile])
        for specifier in sorted(grammar_spec[profile]):
            region_words = reserved_words_of(tables, specifier)
            extra = sorted(region_words - profile_words)
            if extra:
                problems.append(
                    f"profile {profile} ADMITS version_specifier \"{specifier}\", which reserves "
                    f"{len(extra)} word(s) the profile does not ({' '.join(extra[:6])}"
                    f"{' …' if len(extra) > 6 else ''}) — the preprocessor cannot RESERVE a "
                    f"non-keyword, so this region would be applied only half way and the stage "
                    f"would hand the parser an OVER-ACCEPTANCE it cannot see"
                )
    return problems


def write(text, tables, aliases):
    head, rest = text.split(BLOCK_OPEN, 1)
    _, tail = rest.split(BLOCK_CLOSE, 1)
    return head + render_block(tables, aliases, grammar_specifiers()) + tail


def self_test():
    fails = []
    tables, problems = derive()
    aliases = grammar_profiles()
    text = RUST.read_text(encoding="utf-8")
    if problems:
        fails.append("control: the LRM derivation must be clean before planting (" + "; ".join(problems) + ")")
    if compare(text, tables, aliases):
        fails.append("control: the tracked Rust tables must pass before planting")

    planted = text.replace('"logic",', "", 1)
    if not any("NOT in the table: logic" in p for p in compare(planted, tables, aliases)):
        fails.append("planted MISSING word `logic` was not refused")

    planted = text.replace('"logic",', '"logic", "frobnicate",', 1)
    if not any("NOT in the LRM: frobnicate" in p for p in compare(planted, tables, aliases)):
        fails.append("planted FOREIGN word `frobnicate` was not refused")

    planted = text.replace('("sv_2017", KeywordVersion::Sv2017)', '("sv_2017", KeywordVersion::Sv2023)', 1)
    if not any("PROFILE_KEYWORD_VERSION" in p for p in compare(planted, tables, aliases)):
        fails.append("planted WRONG profile mapping was not refused")

    planted = text.replace('"1364-2001", "1364-1995"]', '"1364-2001"]', 1)
    if not any("admitted version specifiers for" in p for p in compare(planted, tables, aliases)):
        fails.append("planted MISSING admitted specifier was not refused")

    try:
        split_by_monotonicity(["b", "a", "d", "c"], 2, "planted")
        fails.append("a body with 2 descents was split into 2 tables instead of being refused")
    except ValueError:
        pass

    # the subset arm: a planted profile that admits a RICHER version must be refused
    planted_spec = {k: set(v) for k, v in grammar_specifiers().items()}
    planted_spec["verilog_2005"] = planted_spec["verilog_2005"] | {"1800-2005"}
    if not any("cannot RESERVE a non-keyword" in p
               for p in admitted_specifiers_are_subsets(tables, planted_spec)):
        fails.append("a profile admitting a RICHER version_specifier was not refused")
    if admitted_specifiers_are_subsets(tables, grammar_specifiers()):
        fails.append("control: the shipped profile/specifier lists must all be subsets")

    sizes = {name: len(tables[name]) for name in TABLE_NAMES + ["NOCONFIG_EXCLUSIONS"]}
    expected = {
        "TABLE_22_1": 102, "TABLE_22_2": 21, "TABLE_22_3": 1,
        "TABLE_22_4": 97, "TABLE_22_5": 23, "TABLE_22_6": 4,
        "NOCONFIG_EXCLUSIONS": 10,
    }
    if sizes != expected:
        fails.append(f"table sizes moved: {sizes} (expected {expected} — re-read §22.14 before trusting either side)")

    print(f"sv-keyword-versions self-test: {'PASS' if not fails else 'FAIL'} ({len(fails)} failing)")
    for fail in fails:
        print("  - " + fail)
    return 1 if fails else 0


def main(argv):
    if "--self-test" in argv:
        return self_test()
    tables, problems = derive()
    aliases = grammar_profiles()
    if "--print" in argv:
        for name in TABLE_NAMES + ["NOCONFIG_EXCLUSIONS"]:
            print(f"{name}: {' '.join(tables[name])}")
        return 0
    if "--write" in argv:
        RUST.write_text(write(RUST.read_text(encoding="utf-8"), tables, aliases), encoding="utf-8")
    text = RUST.read_text(encoding="utf-8")
    problems += compare(text, tables, aliases)
    tag = " ".join(f"{name.replace('TABLE_22_', '22-').replace('NOCONFIG_EXCLUSIONS', 'noconfig')}="
                   f"{len(tables[name])}" for name in TABLE_NAMES + ["NOCONFIG_EXCLUSIONS"])
    if problems:
        print(f"SV-KEYWORD-VERSIONS: FAIL ({tag})")
        for problem in problems:
            print("  - " + problem)
        return 1
    print(f"SV-KEYWORD-VERSIONS: OK — rust/src/sv_keyword_versions.rs == IEEE 1800-2017 §22.14 "
          f"Tables 22-1…22-6 + §22.14.4, cross-checked against IEEE 1364-2005 §19.11 and "
          f"IEEE 1800-2023 §22.14, Σ == Annex B Table B.1 ({tag})")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
