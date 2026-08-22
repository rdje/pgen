#!/usr/bin/env bash
# GRAMMAR-WELLFORMED.H.17 — regex LOOK-AROUND in an EBNF regex terminal never compiles, so the rule
# silently never matches. Re-runnable diagnosis + the live census of affected rules.
#
# Rust's `regex` crate does not support look-around. A terminal such as `/((?:[^*]|\*(?!\/))*)/`
# therefore fails to compile ON EVERY INVOCATION, the rule reports an ordinary rule-exit error, and
# the PEG engine backtracks past it exactly as it would past a branch that simply did not match. The
# grammar defect is unconditional and permanent; its runtime signature is indistinguishable from
# routine speculation. SystemVerilog hit this twice (SV-EXH-PROOF.3.3.4.b.6.2.15, where it was the
# DOMINANT source of catastrophic backtracking behind a >180 s hang) and was fixed by moving the
# assertion to a grammar-level `!rule` lookahead — but the sweep across the other grammars was never
# done, so five live rules in two shipped grammars still carry it.
#
# Usage:  bash docs/tasks/artifacts/grammar_wellformed/regex_lookaround/probe.sh
# Exit:   0 = every arm reproduced as recorded · 1 = an arm diverged (a fix landed, or a regression)
#         2 = harness refused (a required binary or input is missing — NOTHING was scored)
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT" || exit 2

AST="./rust/target/debug/ast_pipeline"          # needs --features "generated_parsers ebnf_dual_run"
[ -x "$AST" ] || { echo "REGEX-LOOKAROUND: REFUSED — $AST is missing (nothing scored)" >&2; exit 2; }

WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT
fails=0

say() { printf '%s\n' "$*"; }

# ---------------------------------------------------------------- arm 1: the minimal mechanism
# A synthetic grammar isolating ONE feature per arm, so "the lookahead is the cause" is measured
# rather than inferred. `(a)` and `(?:a)` and `(a(?:b)?)` establish that plain groups, non-capturing
# groups and optionals all work — only the look-around arms fail.
say "== arm 1: feature bisection over regex terminals (synthetic grammars, interpreter) =="
bisect() {  # $1 = regex body, $2 = input, $3 = expected accepted=<v>
    printf '@entry: true\nprobe := /%s/\n' "$1" > "$WORK/p.ebnf"
    printf '%s' "$2" > "$WORK/in.txt"
    local got
    got="$("$AST" "$WORK/p.ebnf" --interpret-parse "$WORK/in.txt" 2>&1 \
           | grep -o 'accepted=[a-z]*' | head -n1)"
    got="${got:-accepted=<none>}"
    if [ "$got" = "accepted=$3" ]; then
        printf '   ok   /%-28s/ on %-5s -> %s\n' "$1" "'$2'" "$got"
    else
        printf '   DIVERGED /%-28s/ on %-5s -> %s (recorded: accepted=%s)\n' "$1" "'$2'" "$got" "$3"
        fails=$((fails + 1))
    fi
}
# controls — these MUST pass, or the arm proves nothing about look-around specifically
bisect '([^*]*)'              'abc' true
bisect '((?:[^*])*)'          'abc' true
bisect '((?:[^*]|x)*)'        'abc' true
bisect '(a(?:b)?)'            'ab'  true
# the defect — a bare 'a' trivially satisfies `(?!b)` and `*` trivially satisfies `(?!\/)` at EOF
bisect '(a(?!b))'             'a'   false
bisect '((?:\*(?!\/))*)'      '*'   false
bisect '((?:[^*]|\*(?!\/))*)' 'abc' false

# ---------------------------------------------------------------- arm 2: the shipped consequence
say "== arm 2: the ebnf meta-grammar cannot parse ANY block comment =="
blk() {  # $1 = label, $2 = input, $3 = expected accepted=<v>
    printf '%s' "$2" > "$WORK/in.txt"
    local got
    got="$("$AST" grammars/ebnf.ebnf --interpret-parse "$WORK/in.txt" 2>&1 \
           | grep -o 'accepted=[a-z]*' | head -n1)"
    got="${got:-accepted=<none>}"
    if [ "$got" = "accepted=$3" ]; then
        printf '   ok   %-16s %-10s -> %s\n' "$1" "'$2'" "$got"
    else
        printf '   DIVERGED %-16s %-10s -> %s (recorded: accepted=%s)\n' "$1" "'$2'" "$got" "$3"
        fails=$((fails + 1))
    fi
}
blk empty_block  '/**/'    false
blk star_block   '/***/'   false
blk normal_block '/* x */' false   # an ORDINARY block comment — not an edge case
blk doc_block    '/** d */' false
blk line_comment '# line'  true    # the ACCEPTING control: line comments are unaffected

# ---------------------------------------------------------------- arm 3: the live census
# Every regex terminal in a tracked grammar carrying a look-around assertion, comment lines excluded.
# `regex.ebnf`'s "(?=" / "(?!" are double-quoted STRING LITERALS — the regex grammar matching PCRE
# lookaround SYNTAX — and are correctly not uses; the `=` (not `:=`) form excludes them.
say "== arm 3: live census of regex look-around uses =="
census="$(grep -nE ':=.*/[^/]*\(\?<?[!=]' grammars/*.ebnf 2>/dev/null | grep -vE ':[0-9]+:[[:space:]]*#' || true)"
n="$(printf '%s' "$census" | grep -c . || true)"
printf '%s\n' "$census" | sed 's/^/   /'
say "REGEX-LOOKAROUND-CENSUS: live_uses=$n"
if [ "$n" -ne 6 ]; then
    say "   DIVERGED — recorded 6 live uses (3 ebnf, 2 semantic_annotation, 1 sv_lrm_profiled_generated)"
    fails=$((fails + 1))
fi

# ---------------------------------------------------------------- arm 4: the static check
# ⭐ THIS ARM WAS INVERTED BY `H.17.2` (-0159), AND THAT IS THE POINT. It was written to record that
# `--lint-grammar` reported the defect ZERO times — the reason it survived a documented prior fix in
# a sibling family. `H.17.2` added the `uncompilable_regex_terminals` error class, so the recorded
# value is now the COUNT the linter finds, and the arm keeps watching it from the other side: if it
# falls to 0 without `H.17.1` landing, the check has been weakened rather than the grammar fixed.
say "== arm 4: the static check reports it (inverted by H.17.2) =="
for pair in "ebnf:3" "semantic_annotation:2" "systemverilog:0"; do
    g="${pair%%:*}"; want="${pair##*:}"
    got="$("$AST" "grammars/$g.ebnf" --lint-grammar 2>&1 \
           | grep -oE 'uncompilable_regex_terminals=[0-9]+' | head -n1 | cut -d= -f2)"
    got="${got:-<none>}"
    if [ "$got" = "$want" ]; then
        printf '   ok   %-22s uncompilable_regex_terminals=%s\n' "$g" "$got"
    else
        printf '   DIVERGED %-22s uncompilable_regex_terminals=%s (recorded: %s)\n' "$g" "$got" "$want"
        fails=$((fails + 1))
    fi
done
say "   (systemverilog is the ACCEPTING control: it carries the same construct in COMMENTS only,"
say "    having been repaired in SV-EXH-PROOF.3.3.4.b.6.2.15, and must stay at 0)"

say ""
if [ "$fails" -eq 0 ]; then
    say "REGEX-LOOKAROUND: REPRODUCED (all arms as recorded)"; exit 0
fi
say "REGEX-LOOKAROUND: DIVERGED in $fails arm(s) — a fix landed or a regression occurred; re-adjudicate"
exit 1
