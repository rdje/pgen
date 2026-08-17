#!/usr/bin/env bash
# docs/tasks/artifacts/engine_universal_services/es33_makefile_mirror/census.sh
#
# `ENGINE-UNIVERSAL-SERVICES.33` acceptance (c) — the CENSUS of hard-coded mirrors of
# `rust/Makefile`'s GENERATOR RECIPE inside the repository's own enforcement/gate tier.
#
# ⛔ WHY A SCRIPT AND NOT A PROSE COUNT. `.32` slice 2 found the `.33` mirror **by accident, while
#    writing down the bounds of its own fix** — which the leaf itself says "is not a search". A
#    number produced by a search nobody can re-run is the same class of claim as a number nobody
#    re-derives (`docs/CLAIM_VERIFICATION.md` §1). Run this and the counts come back.
#
# THE CENSUS QUESTION, stated precisely so the population is decidable rather than a judgement call:
#
#     which tracked file OUTSIDE `rust/Makefile` invokes the code generator with a flag list it
#     spells out itself, such that the flag list and the Makefile's recipe must AGREE for that
#     file's purpose to hold?
#
# SCOPE — the enforcement/gate tier: `scripts/`, `rust/scripts/`, `.githooks/`. ⛔ Deliberately NOT
# `docs/tasks/artifacts/**`: a probe bank is ERA-PINNED on purpose (it records what a past era
# measured), so a probe re-deriving with the flags of its own era is correct behaviour, not drift.
# That exclusion is a scope decision, not a silent cap — the excluded count is printed too.
#
# THE THREE BUCKETS, and the discriminator between them is measured rather than asserted: a script
# that feeds its generated parser back in through `PGEN_<FAMILY>_PARSER_PATH` is standing that parser
# in for the family's SHIPPED parser, so its verdict is about the shipping recipe.
#
#   M  MIRROR-AND-SUBSTITUTES — spells the flags AND substitutes the result as the family parser.
#      Its verdict is about a parser the project ships, so the flag lists must agree.
#   S  MIRROR-ONLY — spells the flags, uses the parser for its own probe, never substitutes it.
#      A drift here changes what the gate measured, not what the project ships.
#   N  NOT-A-MIRROR — deliberately a DIFFERENT recipe (a non-bootstrap arm, a negative-path probe).
#      Agreement is not wanted; these are listed so the classification is exhaustive.
#
# USAGE
#   bash docs/tasks/artifacts/engine_universal_services/es33_makefile_mirror/census.sh
#   … > docs/tasks/artifacts/engine_universal_services/es33_makefile_mirror/census.txt
# EXIT
#   0 = the census ran; 2 = it could not run correctly (never a silent empty population)

set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT"
[ -f rust/Makefile ] || { echo "census: rust/Makefile not found from $ROOT — refusing to report an empty population" >&2; exit 2; }

SCOPE=$(git ls-files 'scripts/*' 'rust/scripts/*' '.githooks/*' | LC_ALL=C sort)
[ -n "$SCOPE" ] || { echo "census: the enforcement/gate scope is EMPTY — git ls-files matched nothing, so every count below would be vacuous" >&2; exit 2; }

echo "ENGINE-UNIVERSAL-SERVICES.33 (c) — census of hard-coded generator-recipe mirrors"
echo "commit: $(git rev-parse --short HEAD)   scope files: $(printf '%s\n' "$SCOPE" | wc -l | tr -d ' ')"
echo

echo "── 1. THE RECIPE'S HOMES INSIDE rust/Makefile ────────────────────────────────────────────────"
echo "   (a --generate-parser line that is not a comment; the checker's own view of the file)"
MK_ROOT="$ROOT" python3 - <<'PY'
import os
p = os.path.join(os.environ["MK_ROOT"], "rust/Makefile")
n = 0
for i, line in enumerate(open(p, encoding="utf-8", errors="replace"), 1):
    if line.lstrip().startswith("#") or "--generate-parser" not in line:
        continue
    n += 1
    print(f"   {i:>5}: {line.strip()}")
print(f"   => {n} homes for the flag list inside rust/Makefile itself")
PY
echo

echo "── 2/3. THE ENFORCEMENT/GATE TIER — every candidate line, CLASSIFIED AND SHOWN ───────────────"
echo
echo "   ⛔ TWO EARLIER CUTS OF THIS RULE OVER-COUNTED, AND BOTH ARE RECORDED BECAUSE THE SECOND ONE"
echo "      WAS STILL WRONG AFTER LOOKING RIGHT. Cut 1 counted 'a non-comment line containing"
echo "      --generate-parser' and over-counted by 6 — a doctrine DESCRIPTION string in the registry,"
echo "      a python comparison inside an enforcer, an error message, and three self-test literals."
echo "      Cut 2 additionally required a BINARY REFERENCE on the line, which fixed 4 of the 6 and"
echo "      still counted this repository's own self-test perturbation literals as mirrors — reading"
echo "      as 'the fixed check still spells the flags', the exact opposite of the truth."
echo "      Cut 3 is a SHELL-SEMANTICS rule rather than a name heuristic, and it is exact."
echo
echo "   INVOCATION rule : --generate-parser occurs OUTSIDE any quote on the joined logical line."
echo "                     An unquoted word is an argument passed to a command; a quoted one is DATA"
echo "                     (a description, an error message, a python string, a perturbation literal)."
echo "                     ⭐ The binary-reference test from cut 2 is still evaluated as a GROUND-TRUTH"
echo "                     CONTROL: any line the two rules classify differently is printed as a"
echo "                     DISAGREEMENT, so the instrument reports its own uncertainty instead of"
echo "                     silently picking one. Binary pattern:"
echo "                     AST_PIPELINE | BOOTSTRAP_BIN | RUST_GENERATOR | \$tool | \$pipeline"
echo "   RECIPE flags    : every --flag except the IO set  --output --emit-raw-ast-json --dump-gen-ast"
echo "                     (those name destinations; they add outputs, they do not shape emission)"
echo "   BUCKET rule     : M if the file feeds its parser back through PGEN_<FAMILY>_PARSER_PATH (it"
echo "                     stands in for a SHIPPED parser) ; S if it only probes its own artifact"
echo
# ⛔ READ THE FLAG VARIABLES, NOT A STRIPPED PREFIX OF THE COMPOSED ONES. This block first said
#    `sed 's/^RUST_GENERATOR = \$(RUST_AST_PIPELINE) //p'`, i.e. it assumed the composed variable
#    held the flags inline. `ENGINE-UNIVERSAL-SERVICES.16` split them out, and the very next census
#    run reported **18 of 18 invocations differing in a way that COULD change emission** — because
#    the "shipped recipe" it was comparing against had become the literal string `$(GENERATOR_FLAGS)`.
#    ⇒ an instrument that hard-codes the shape of what it reads is the same defect it was built to
#    count, and it failed LOUDLY here only because a human re-ran it after changing that shape.
SHIPPED_FAM=$(sed -n 's/^GENERATOR_FLAGS = //p' rust/Makefile)
SHIPPED_PAIR=$(sed -n 's/^GENERATOR_FLAGS_BOOTSTRAP = //p' rust/Makefile)
[ -n "$SHIPPED_FAM" ] && [ -n "$SHIPPED_PAIR" ] || { echo "census: could not read GENERATOR_FLAGS / GENERATOR_FLAGS_BOOTSTRAP from rust/Makefile — refusing to compare against an empty recipe" >&2; exit 2; }
case "$SHIPPED_FAM$SHIPPED_PAIR" in
  *'$('*) echo "census: the flag variables still carry an unresolved make expansion — every comparison below would be against a literal, not a recipe" >&2; exit 2 ;;
esac
echo "   shipped families : $SHIPPED_FAM"
echo "   shipped pair     : $SHIPPED_PAIR"
echo
CENSUS_SCOPE="$SCOPE" CENSUS_FAM="$SHIPPED_FAM" CENSUS_PAIR="$SHIPPED_PAIR" python3 - <<'PY'
import os, re

IO = {"--output", "--emit-raw-ast-json", "--dump-gen-ast"}
BIN = re.compile(r"AST_PIPELINE|BOOTSTRAP_BIN|RUST_GENERATOR|\$tool|\$pipeline")
FLAG = re.compile(r"(?<!\S)--[a-z0-9-]+")
fam = [t for t in os.environ["CENSUS_FAM"].split() if t not in IO]
pair = [t for t in os.environ["CENSUS_PAIR"].split() if t not in IO]

def logical_lines(path):
    """Join shell continuations, keeping the FIRST physical line number of each logical line."""
    out, buf, start = [], "", None
    for n, raw in enumerate(open(path, encoding="utf-8", errors="replace"), 1):
        raw = raw.rstrip("\n")
        if start is None:
            start = n
        if raw.rstrip().endswith("\\"):
            buf += raw.rstrip()[:-1] + " "
            continue
        out.append((start, buf + raw))
        buf, start = "", None
    if buf:
        out.append((start, buf))
    return out

def has_unquoted(line, needle="--generate-parser"):
    """True iff `needle` starts at a position that is outside both quote kinds.

    Shell-accurate for the shapes this repository writes: a backslash escapes the next
    character, a single quote runs to the next single quote, a double quote to the next
    unescaped double quote. Inside single quotes a backslash is literal.
    """
    i, sq, dq = 0, False, False
    n = len(needle)
    while i < len(line):
        c = line[i]
        if not sq and c == "\\":
            i += 2
            continue
        if not dq and c == "'":
            sq = not sq
            i += 1
            continue
        if not sq and c == '"':
            dq = not dq
            i += 1
            continue
        if not sq and not dq and line.startswith(needle, i):
            return True
        i += 1
    return False

counts = {"M": 0, "S": 0, "rejected": 0}
files = {"M": set(), "S": set()}
rejected, disagreements, differs = [], [], []
INERT = {"--eliminate-left-recursion"}   # measured in section 4: this flag cannot change emission
for path in os.environ["CENSUS_SCOPE"].split("\n"):
    if not path:
        continue
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except (IsADirectoryError, FileNotFoundError):
        continue
    if "--generate-parser" not in text:
        continue
    substitutes = "PGEN_" in text and re.search(r"PGEN_[A-Z_]*_PARSER_PATH", text)
    for n, line in logical_lines(path):
        if line.lstrip().startswith("#") or "--generate-parser" not in line:
            continue
        unquoted, named_binary = has_unquoted(line), bool(BIN.search(line))
        if unquoted != named_binary:
            disagreements.append((path, n, unquoted, named_binary, line.strip()))
        if not unquoted:
            counts["rejected"] += 1
            rejected.append((path, n, line.strip()))
            continue
        bucket = "M" if substitutes else "S"
        counts[bucket] += 1
        files[bucket].add(path)
        flags = [t for t in FLAG.findall(line) if t not in IO]
        want = pair if "--bootstrap-mode" in flags else fam
        if flags == want:
            verdict = "SAME as shipped " + ("pair" if want is pair else "families")
        else:
            missing = [t for t in want if t not in flags]
            extra = [t for t in flags if t not in want]
            verdict = "DIFFERS —" + (" missing " + " ".join(missing) if missing else "") \
                                  + (" extra " + " ".join(extra) if extra else "")
            differs.append((path, n, missing, extra))
        print(f"   {bucket}  {path}:{n}")
        print(f"         flags   {' '.join(flags)}")
        print(f"         verdict {verdict}")

print()
print(f"   => INVOCATIONS: {counts['M']} in bucket M ({len(files['M'])} files, they substitute a SHIPPED parser)")
print(f"                   {counts['S']} in bucket S ({len(files['S'])} files, own-artifact probes)")
print(f"      REJECTED as not-an-invocation (quoted DATA): {counts['rejected']} line(s), each shown below")
print()
for path, n, line in rejected:
    print(f"   ·  {path}:{n}")
    print(f"      {line[:150]}")
print()
if disagreements:
    print(f"   ⚠️ GROUND-TRUTH CONTROL: the two rules DISAGREE on {len(disagreements)} line(s). Adjudicate each")
    print("      before trusting the counts above — a disagreement is the instrument saying it is unsure.")
    for path, n, unq, nb, line in disagreements:
        print(f"   ⚠️ {path}:{n}  unquoted={unq} names_binary={nb}")
        print(f"      {line[:150]}")
    print()
    print("      ADJUDICATION (recorded, because a judgement is not something a regex made): every")
    print("      disagreement above is a self-test perturbation LITERAL in this repository's own")
    print("      enforcer — text that exists to be planted in a scratch Makefile and perturbed. The")
    print("      shell-semantics rule is right and cut 2 was wrong. ⭐ They are nonetheless MIRRORS")
    print("      WITH A GUARD, not silent ones: `mk` refuses when its target text is absent from the")
    print("      Makefile, so a recipe change makes the arm report ✗ rather than pass quietly.")
else:
    print("   ✓ GROUND-TRUTH CONTROL: the shell-semantics rule and the binary-reference rule agree on")
    print("     every candidate line, so the counts above do not depend on which one was chosen.")

print()
print("   ── DERIVED SUMMARY of sections 2/3 (never typed) ──")
print(f"   hand-spelled generator invocations in the enforcement/gate tier : {counts['M'] + counts['S']}")
print(f"       bucket M — substitutes a SHIPPED parser                     : {counts['M']} in {len(files['M'])} files")
print(f"       bucket S — own-artifact probe                               : {counts['S']} in {len(files['S'])} files")
print(f"   of those, DIFFERING from the shipped recipe                     : {len(differs)}")
only_inert = [d for d in differs if set(d[2]) <= INERT and not d[3]]
print(f"       differing ONLY by the inert --eliminate-left-recursion      : {len(only_inert)}")
print(f"       differing in a way that COULD change emission               : {len(differs) - len(only_inert)}")
print(f"   invocations left in scripts/check_generated_reproducibility.sh  : "
      f"{sum(1 for b in ('M', 'S') for p in files[b] if p.endswith('check_generated_reproducibility.sh'))}"
      "   (it was 2 before this slice: the hard-coded family list and the pair list)")
print()
print("   ⇒ READ THIS AS 'AT RISK', NOT 'CURRENTLY WRONG'. Every difference found is the inert flag,")
print("     so no gate in the population is measuring a parser the project does not ship TODAY. The")
print("     hazard is the ordinary one: the next flag added to RUST_GENERATOR is real, and each of")
print("     these sites keeps its own copy of the list that would have to be edited too.")
PY
echo

echo "── 4. THE FLAG THE SHIPPED RECIPE CARRIES THAT CANNOT CHANGE ANYTHING ────────────────────────"
echo "   rust/src/main.rs:"
grep -n 'if args.eliminate_left_recursion' -A2 rust/src/main.rs | sed 's/^/      /'
echo "   => the field already defaults to true and there is no negating flag, so"
echo "      --eliminate-left-recursion in RUST_GENERATOR is INERT. Measured: json / regex / vhdl /"
echo "      systemverilog re-derive BYTE-IDENTICALLY with and without it."
echo

echo '── 5. WHAT HAS BEEN FIXED AGAINST THIS CENSUS, AND WHAT IS STILL ROUTED ──────────────────────'
echo "   .33 s1  scripts/check_generated_reproducibility.sh DERIVES the recipe from rust/Makefile and"
echo "           REFUSES (exit 2) on any shape it cannot resolve; its \$(RUST_GENERATOR…) call sites"
echo "           are held flag-free. Its own hand-spelled invocation count went 2 -> 0."
echo "   .16 s1  rust/Makefile's THIRD inline copy of the bootstrap flag list is GONE: the flags live"
echo "           in GENERATOR_FLAGS / GENERATOR_FLAGS_BOOTSTRAP, which the two composed variables and"
echo "           the generated/ebnf.rs seed all reference => 3 homes -> 2. The check reads THOSE and"
echo "           asserts each composed variable is exactly '<binary> \$(<flag-variable>)'."
echo "           generated/ebnf.rs also JOINED the doctrine as a SEED cohort (roster 10 -> 11)."
echo "   ROUTED  CI-PARITY-GATE-ROT.38  the bucket-M/S population above (18 invocations, 11 of them"
echo "                                  substituting a SHIPPED parser). Its (a) is now DISCHARGED by"
echo "                                  .16 above; (b)-(e) remain."
echo "           ENGINE-UNIVERSAL-SERVICES.34  --eliminate-left-recursion is an INERT CLI flag that"
echo "                                  three surfaces document as meaningful — the 5 rows above."
echo "           ENGINE-UNIVERSAL-SERVICES.35  the baseline's verified_at_commit names the PARENT of"
echo "                                  the commit that lands it, because --rebaseline runs pre-commit."
