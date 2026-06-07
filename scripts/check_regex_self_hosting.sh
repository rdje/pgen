#!/usr/bin/env bash
# scripts/check_regex_self_hosting.sh
#
# REGEX-SELF-HOSTING.6 capstone guard.
#
# THE INVARIANT: the regex parser is SELF-HOSTING — `grammars/regex.ebnf` expresses the whole regex
# language with native EBNF terminals (literals, the `builtin_any_char`/`builtin_ascii_char` primitives,
# `$text`/`$0`, `@transform`), and uses NO `/.../` regex literals. Consequently the generated regex parser
# never CALLS Rust's `regex` engine (`match_regex`). The director's directive (2026-06-07): "build a
# from-scratch regex parser and not use Rust's own regex engine." (Tree: docs/tasks/REGEX-SELF-HOSTING.md.)
#
# This gate fails if either invariant regresses:
#   (1) a `/.../` regex literal reappears in grammars/regex.ebnf, or
#   (2) the generated regex parser (if present locally) emits a `match_regex` call.
#
# NOTE (REGEX-SELF-HOSTING.6a, pending): the generated parser still EMITS an (uncalled) `match_regex`
# helper + `use regex::Regex` import — link hygiene, tracked separately. This gate asserts zero CALLS,
# which is what makes the engine never run.
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

EBNF="grammars/regex.ebnf"
GEN="generated/regex_parser.rs"

if [ ! -f "$EBNF" ]; then
  echo "regex-self-hosting: FAIL — $EBNF not found" >&2
  exit 1
fi

# (1) Zero /.../ regex literals in regex.ebnf. Strip quoted literals ('...' / "...") and # comments
# first so a slash inside a char/string terminal or a comment never trips the check.
offending="$(python3 - "$EBNF" <<'PY'
import re, sys
hits = []
for n, line in enumerate(open(sys.argv[1], encoding="utf-8"), 1):
    code = re.sub(r"'(?:\\.|[^'])*'", "", line)       # strip '...'
    code = re.sub(r'"(?:\\.|[^"])*"', "", code)        # strip "..."
    code = code.split("#", 1)[0]                        # strip trailing comment
    if re.search(r"/(?:\\.|[^/])+/", code):            # an EBNF /.../ regex literal
        hits.append(f"{n}: {line.rstrip()}")
print("\n".join(hits))
PY
)"
if [ -n "$offending" ]; then
  echo "regex-self-hosting: FAIL — $EBNF still contains /.../ regex literal(s) (self-hosting requires native terminals):" >&2
  printf '    %s\n' "$offending" >&2
  echo "  Express the construct with literals / builtin_any_char / builtin_ascii_char / \$text. See docs/tasks/REGEX-SELF-HOSTING.md." >&2
  exit 1
fi

# (2) Zero match_regex CALLS in the generated regex parser (only when it has been generated locally;
# generated/ is gitignored, so CI regenerates before this runs).
if [ -f "$GEN" ]; then
  calls="$(grep -cE '\.match_regex *\(' "$GEN" || true)"
  if [ "${calls:-0}" -ne 0 ]; then
    echo "regex-self-hosting: FAIL — $GEN emits $calls match_regex call(s); the regex parser must not invoke Rust's regex engine." >&2
    grep -nE '\.match_regex *\(' "$GEN" | head >&2
    exit 1
  fi
fi

echo "regex-self-hosting: OK"
