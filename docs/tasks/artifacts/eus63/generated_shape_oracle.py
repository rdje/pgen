#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.63(a) — the nesting axes `.61`(b)'s bracket oracle CANNOT see,
and the design-scale parameter that decides how deep a lowering nests.

`.61`(b) measured max nesting over 30 056 real files with a lexical `()[]{}` depth scan and
read 22. That instrument is sound for what it measures and BLIND to two things that matter
for the target population:

  1. A right-associative ternary chain -- `c0 ? v0 : c1 ? v1 : ... : d`, exactly what a
     Chisel/FIRRTL-class emitter produces for a priority mux -- carries ZERO brackets. The
     bracket oracle reads depth 0 on a file nested 1 600 levels deep.
  2. An `else if` ladder -- a priority encoder in procedural form -- nests one level per
     branch and carries no brackets either.

And it answers the wrong question for a GENERATED population. The depth of hand-written
source is a property of what a human will type. The depth of generated source is a
property of the DESIGN PARAMETER the generator was run on: a lowering that emits one
nesting level per way turns an N-way structure into N levels. So this oracle also measures
N -- the widest N-way structures real designs actually contain -- because that is the
number that decides whether a generator's output clears the cliff.

All four axes are LEXICAL: no parser, so no inherited parser blind spot. Strings, line
comments, block comments and based literals (`4'b10??`) are skipped.
⚠️ The `vec_width` column is computed but DELIBERATELY NOT PUBLISHED: its number scan cannot
tell a width literal from any other integer (it read 2 000 000 001 on a constraint test), so it
is an unadjudicated column, not a measurement.
"""
import sys, os, re, json

IDENT = re.compile(r"[A-Za-z_$][A-Za-z0-9_$]*")


def tokens(src, vhdl=False):
    """Yield (kind, text, bracket_depth). kind in {'op','word','num'}."""
    i, n = 0, len(src)
    depth = 0
    stack = []
    PAIR = {")": "(", "]": "[", "}": "{"}
    while i < n:
        c = src[i]
        # --- comments -----------------------------------------------------------
        if not vhdl and c == "/" and i + 1 < n and src[i + 1] == "/":
            j = src.find("\n", i); i = n if j < 0 else j + 1; continue
        if vhdl and c == "-" and i + 1 < n and src[i + 1] == "-":
            j = src.find("\n", i); i = n if j < 0 else j + 1; continue
        if not vhdl and c == "/" and i + 1 < n and src[i + 1] == "*":
            j = src.find("*/", i + 2); i = n if j < 0 else j + 2; continue
        # --- strings ------------------------------------------------------------
        if c == '"':
            i += 1
            while i < n and src[i] != '"':
                i += 2 if (not vhdl and src[i] == "\\") else 1
            i += 1
            continue
        # --- VHDL character literal / SV based literal --------------------------
        if vhdl and c == "'" and i + 2 < n and src[i + 2] == "'":
            i += 3; continue
        if not vhdl and c == "'" and i + 1 < n and src[i + 1] in "bBoOdDhH":
            i += 2
            while i < n and (src[i].isalnum() or src[i] in "_?xXzZ"):
                i += 1
            continue
        # --- brackets -----------------------------------------------------------
        if c in "([{":
            stack.append(c); depth += 1
            yield ("op", c, depth)
            i += 1; continue
        if c in ")]}":
            yield ("op", c, depth)
            if stack and stack[-1] == PAIR[c]:
                stack.pop(); depth -= 1
            i += 1; continue
        # --- words --------------------------------------------------------------
        m = IDENT.match(src, i)
        if m:
            yield ("word", m.group(0), depth)
            i = m.end(); continue
        if c.isdigit():
            j = i
            while j < n and (src[j].isalnum() or src[j] == "_"):
                j += 1
            yield ("num", src[i:j], depth)
            i = j; continue
        if c.isspace():
            i += 1; continue
        # --- VHDL matching operators `?=` `?<` `?/=` are not conditionals --------
        if vhdl and c == "?":
            i += 1
            while i < n and src[i] in "=<>/":
                i += 1
            continue
        yield ("op", c, depth)
        i += 1


BREAKERS = {"module", "endmodule", "function", "endfunction", "task", "endtask",
            "always", "always_comb", "always_ff", "always_latch", "initial",
            "endcase", "class", "endclass", "package", "endpackage"}
VHDL_BREAKERS = {"entity", "architecture", "process", "begin", "end", "package"}


def scan_sv(src):
    max_bracket = 0
    # axis 1: same-bracket-depth `?` run inside one statement == ternary chain length
    tern_run, max_tern = {}, 0
    # axis 2: `else if` ladder run
    else_pending, elif_run, max_elif = False, 0, 0
    # axis 3: case-item count in the innermost open case
    case_stack, max_case = [], 0   # entries are [depth_at_open, item_count]
    max_width, last_num, prev_op = 0, None, None   # retained, NOT published: see module docstring
    for kind, text, depth in tokens(src):
        if depth > max_bracket:
            max_bracket = depth
        if kind == "op":
            if text == "?":
                tern_run[depth] = tern_run.get(depth, 0) + 1
                max_tern = max(max_tern, tern_run[depth])
            elif text == ";":
                tern_run.clear()
            elif text == ":" and case_stack and depth == case_stack[-1][0]:
                case_stack[-1][1] += 1
                max_case = max(max_case, case_stack[-1][1])
            elif text in ")]}":
                tern_run.pop(depth + 1, None)
            if text == "]" and last_num is not None:
                max_width = max(max_width, last_num + 1)
            last_num = None if text != "]" else last_num
            prev_op = text
            continue
        if kind == "num":
            try:
                v = int(text.split("'")[-1].lstrip("bBoOdDhH")) if "'" in text else int(text)
                last_num = v
            except ValueError:
                last_num = None
            continue
        # words
        w = text
        if w == "else":
            else_pending = True
            continue
        if w == "if" and else_pending:
            elif_run += 1
            max_elif = max(max_elif, elif_run)
            else_pending = False
            continue
        if w in ("case", "casez", "casex"):
            case_stack.append([depth, 0])
        elif w == "endcase":
            if case_stack:
                case_stack.pop()
        if w in BREAKERS:
            elif_run = 0
        else_pending = False
    return max_bracket, max_tern, max_elif, max_case, max_width


def scan_vhdl(src):
    max_bracket = 0
    # `when ... else` chain in a conditional signal assignment nests one level per arm
    when_run, max_when = 0, 0
    elsif_run, max_elsif = 0, 0
    max_width, last_num = 0, None
    for kind, text, depth in tokens(src, vhdl=True):
        if depth > max_bracket:
            max_bracket = depth
        if kind == "op":
            if text == ";":
                when_run = 0
            if text == ")" and last_num is not None:
                max_width = max(max_width, last_num + 1)
            last_num = None
            continue
        if kind == "num":
            try:
                last_num = int(text)
            except ValueError:
                last_num = None
            continue
        w = text.lower()
        if w == "else":
            when_run += 1
            max_when = max(max_when, when_run)
        elif w == "elsif":
            elsif_run += 1
            max_elsif = max(max_elsif, elsif_run)
        elif w in VHDL_BREAKERS:
            elsif_run = 0
    return max_bracket, max_when, max_elsif, 0, max_width


def main():
    listing, out_tsv, fam = sys.argv[1], sys.argv[2], sys.argv[3]
    scan = scan_vhdl if fam == "vhdl" else scan_sv
    rows = []
    for path in (l.strip() for l in open(listing) if l.strip()):
        try:
            src = open(path, encoding="utf-8", errors="replace").read()
        except OSError:
            continue
        rows.append(scan(src) + (path,))
    cols = ["bracket", "tern_chain", "else_if", "case_items", "vec_width"]
    with open(out_tsv, "w") as fh:
        fh.write("\t".join(cols) + "\tpath\n")
        for r in sorted(rows, reverse=True):
            fh.write("\t".join(str(x) for x in r) + "\n")
    summary = {"family": fam, "files": len(rows)}
    for k, col in enumerate(cols):
        vals = sorted(r[k] for r in rows)
        summary[col] = {
            "max": vals[-1],
            "p999": vals[min(len(vals) - 1, int(0.999 * len(vals)))],
            "p99": vals[min(len(vals) - 1, int(0.99 * len(vals)))],
            "median": vals[len(vals) // 2],
        }
        top = sorted(rows, key=lambda r: -r[k])[:5]
        summary[col]["top5"] = [[t[k], t[-1]] for t in top]
    print(json.dumps(summary, indent=1))


if __name__ == "__main__":
    main()
