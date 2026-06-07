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
# Scope: this gate concerns ONLY the regex parser. By design (director 2026-06-07) EVERY OTHER grammar
# (sv, vhdl, rtl_*, the annotation grammars, json, …) is ALLOWED to use `/.../` and thus Rust's `regex`
# engine — the `match_regex` helper + `use regex::Regex` import are emitted per-grammar (gated on whether
# that grammar uses any `/.../`). This gate never inspects the other parsers.
#
# REGEX-SELF-HOSTING.6a made the regex parser fully regex-crate-free (the dead helper + import are elided
# for a `/.../`-free grammar), so this gate also asserts ZERO `regex::Regex` references in regex_parser.rs.
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

# (2) The generated regex parser (only when it has been generated locally; generated/ is gitignored, so
# CI regenerates before this runs) must be fully regex-crate-free: zero `match_regex` calls AND zero
# `regex::Regex` references (the helper + `use regex::Regex` import are elided for a /.../-free grammar).
if [ -f "$GEN" ]; then
  refs="$(grep -nE '\.match_regex *\(|regex::Regex|Regex::new|^use regex' "$GEN" || true)"
  if [ -n "$refs" ]; then
    echo "regex-self-hosting: FAIL — $GEN references Rust's regex engine; the regex parser must be regex-crate-free:" >&2
    printf '%s\n' "$refs" | head >&2
    exit 1
  fi
fi

echo "regex-self-hosting: OK"
