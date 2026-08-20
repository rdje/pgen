#!/usr/bin/env bash
# scripts/check_baseline_identity.sh
#
# DOCTRINE `BASELINE-IDENTITY` (structural) — SV-CORPUS-GRAD.13c.2x.2, DIRECTOR-ORDERED 2026-08-20:
# *"did you or are you going to add support sha for both the grammar and the parser plus a
# verified_at_commit … please add full support for that if not done already."*
#
#   A baseline that holds a number DERIVED from the tree, and cannot say WHICH tree it was
#   derived from, cannot tell a stale expectation from a real regression.
#
# ⛔ WHY THIS IS A DOCTRINE AND NOT A FIELD. `SV-CORPUS-GRAD.13c.2x` measured
# `systemverilog_recognized_cert_union_contract.json` RED at HEAD by 71 rules and 53 UNKNOWNs
# after ELEVEN revisions of `grammars/systemverilog.ebnf` — every one of its `expected_*` fields an
# exact function of that grammar, and not one byte in the file able to say the baseline was the
# stale half. Its `(d)` sweep then found the shape is repo-wide, not a one-off.
#
# ⛔⛔ THE ONE DESIGN CONSTRAINT: A BLOCK NOBODY READS IS THE DEFECT, NOT THE FIX.
# `SV-CORPUS-GRAD.13i` measured this repository shipping SIX oracles that already carried a
# self-describing identity block; exactly ONE was gate-checked and FOUR were measurably stale. So
# this enforcer does not ask whether a block EXISTS. It RE-HASHES every input the block declares,
# on every run, and it refuses a `deferred` row that carries a block — because a present-but-unread
# block is precisely the `.13i` failure and must be impossible to reach through this register.
#
# ⭐ THE BLOCK IS GENERIC, AND THAT IS THE DESIGN. The population does NOT share inputs: the SV cert
# contracts derive from the SV grammar + generated parser, `generated_clippy_correctness_contract_v0`
# from generated Rust + the installed clippy, `live_document_currency_register_v0` from tracked
# docs, `regex_pcre2_compile_oracle_lightweight_v0.env` from the regex grammar + its case corpus. So
# each baseline DECLARES its own dependency set AS DATA and this verifier re-hashes exactly that:
#
#     "identity": {
#       "_verifier": "bash scripts/check_baseline_identity.sh --verify <file> …",
#       "verified_at_commit": "<40-hex>",
#       "inputs": { "grammars/systemverilog.ebnf": "<sha256>", … }
#     }
#
# This is strictly stronger than `PARSE-COST-RATCHET`'s block, whose four inputs are HARD-CODED in
# its gate: there, adding an input means remembering to edit a script (and `.21` records exactly
# that being forgotten). Here a forgotten input is a missing map key in the artifact itself.
#
# ⛔ THE POPULATION IS CLOSED AND TWO-SIDED, on the `gate_reachability_register_v0.json` model. The
# entry set of `rust/test_data/grammar_quality/` is RE-DERIVED every run and joined against the
# register: an entry with no row FAILS, and a row naming no entry FAILS. A one-shot census is the
# same disease one level up — it gets written down and then nothing re-applies it.
#
# Exit codes (a gate calling `--verify` should distinguish all three):
#   0  fresh — every declared input re-hashed equal (inputs that are absent are reported NOT
#      EVALUATED and never silently counted as fresh)
#   1  STALE — a declared input moved, so the baseline no longer describes this tree
#   2  REFUSE — no block, a malformed block, or a register violation
#
# Usage:
#   bash scripts/check_baseline_identity.sh                     # doctrine mode (register + all adopted)
#   bash scripts/check_baseline_identity.sh --verify FILE       # one baseline, for a gate to call
#   bash scripts/check_baseline_identity.sh --stamp FILE [--input PATH]...   # (re)derive the block
#   bash scripts/check_baseline_identity.sh --report            # the full inventory
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"

exec python3 - "$@" <<'PYEOF'
import hashlib, json, os, re, subprocess, sys

ROOT = os.getcwd()
BASE_DIR = "rust/test_data/grammar_quality"
REGISTER = os.path.join(BASE_DIR, "baseline_identity_register_v0.json")
TAG = "baseline-identity"

SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
COMMIT_RE = re.compile(r"^[0-9a-f]{40}$")
VERIFIER_HINT = "scripts/check_baseline_identity.sh"

# The disposition vocabulary. A row may hold nothing else; the register's own copy of these
# strings is held equal to this dict, so the two cannot drift.
DISPOSITIONS = {
    "adopted":
        "carries an `identity` block, and this enforcer re-hashes every input it declares on "
        "every run",
    "deferred":
        "holds expectations that ARE a function of the tree, adoption is owed, and `owner_leaf` "
        "names the task leaf that owes it. An ACCEPTED RISK, not a clean bill of health — what "
        "it buys is that the debt is VISIBLE, OWNED and cannot grow silently",
    "not-a-derived-baseline":
        "holds no value that is a function of the tree — schema versions, run configuration, "
        "policy budgets, case lists, human dispositions — so an identity block would guard "
        "nothing and would only add a surface to rot",
    "identity-native":
        "the artifact IS an identity record — it stores digests rather than expectations — and a "
        "named doctrine already re-derives it every run. `verified_by` names that enforcer and "
        "this register checks the enforcer exists; it does NOT re-run it",
    "corpus-directory":
        "a corpus tree rather than a baseline file; its contents are inputs that a baseline "
        "DECLARES, not expectations of their own",
}

failures, refusals, notes, unevaluated = [], [], [], []


def fail(msg):
    failures.append(msg)


def refuse(msg):
    refusals.append(msg)


def sha256_of(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


# ── the identity reader, a pure function so it can carry ground truth ───────────────────────────
#
# ⛔ IT RETURNS A THREE-WAY STATUS, NOT A TRUTH VALUE. "absent" and "malformed" are DIFFERENT
# facts about a baseline and a reader that collapses them into a falsy return makes a mangled
# block indistinguishable from an unadopted one — a miss in the passing direction, which is the
# exact failure shape `.13i` is a record of.
def read_identity(obj):
    """('ok', (commit, {path: sha256})) | ('absent', None) | ('malformed', reason)."""
    if not isinstance(obj, dict):
        return ("malformed", "the document's top level is not a JSON object")
    if "identity" not in obj:
        return ("absent", None)
    ident = obj["identity"]
    if not isinstance(ident, dict):
        return ("malformed", "`identity` is present but is not an object")
    hint = ident.get("_verifier")
    if not isinstance(hint, str) or VERIFIER_HINT not in hint:
        return ("malformed",
                "`identity._verifier` must be a string naming " + VERIFIER_HINT +
                ", so a reader of the artifact is told what re-derives it")
    commit = ident.get("verified_at_commit")
    if not isinstance(commit, str) or not COMMIT_RE.match(commit):
        return ("malformed", "`identity.verified_at_commit` must be a 40-hex git commit sha")
    inputs = ident.get("inputs")
    if not isinstance(inputs, dict) or not inputs:
        return ("malformed", "`identity.inputs` must be a non-empty object of path -> sha256")
    for path, digest in inputs.items():
        if not isinstance(path, str) or not path or path.startswith("/") or ".." in path.split("/"):
            return ("malformed",
                    "input path %r is not repo-root-relative (CLAUDE.md §12)" % (path,))
        if not isinstance(digest, str) or not SHA256_RE.match(digest):
            return ("malformed", "input `%s` does not carry a 64-hex sha256" % (path,))
    return ("ok", (commit, dict(inputs)))


# ── a string-aware JSON value scanner, so --stamp can SPLICE rather than reformat ───────────────
#
# ⛔ A `json.dump` round-trip would rewrite every one of these tracked, hand-formatted artifacts —
# re-flowing prose fields, re-escaping non-ASCII — and a stamp that produces a 400-line diff is a
# stamp nobody will run. This walks the raw text instead, so everything outside the block stays
# byte-identical.
def scan_value_end(text, i):
    """Index just past the complete JSON value starting at text[i]."""
    c = text[i]
    if c == '"':
        i += 1
        while i < len(text):
            if text[i] == "\\":
                i += 2
                continue
            if text[i] == '"':
                return i + 1
            i += 1
        raise ValueError("unterminated string")
    if c in "{[":
        close = "}" if c == "{" else "]"
        depth = 0
        while i < len(text):
            ch = text[i]
            if ch == '"':
                i = scan_value_end(text, i)
                continue
            if ch == c:
                depth += 1
            elif ch == close:
                depth -= 1
                if depth == 0:
                    return i + 1
            i += 1
        raise ValueError("unterminated %s" % c)
    j = i
    while j < len(text) and text[j] not in ",}\n":
        j += 1
    return j


def self_check():
    """GROUND TRUTH, re-run on every invocation (microseconds). Positive controls AND negatives;
    a MISS refuses (exit 2) rather than reporting a clean tree
    (docs/decisions/feedback_instrument_needs_ground_truth.md)."""
    h = "0" * 64
    c = "a" * 40
    hint = "bash " + VERIFIER_HINT + " --verify x"
    good = {"identity": {"_verifier": hint, "verified_at_commit": c,
                         "inputs": {"grammars/systemverilog.ebnf": h}}}
    cases = [
        # the shape that must be READ
        (good, "ok"),
        # not adopted — a different fact from a broken block
        ({"expected_total": 1362}, "absent"),
        # ⛔ every one of these once passed a reader that only asked `if obj.get("identity")`
        ({"identity": "yes"}, "malformed"),
        ({"identity": {}}, "malformed"),
        ({"identity": {"_verifier": hint, "verified_at_commit": c, "inputs": {}}}, "malformed"),
        ({"identity": {"_verifier": hint, "verified_at_commit": c,
                       "inputs": {"g.ebnf": "deadbeef"}}}, "malformed"),
        ({"identity": {"_verifier": hint, "verified_at_commit": "abc", "inputs": {"g": h}}},
         "malformed"),
        # an ABSOLUTE path would hash something outside the repo (CLAUDE.md §12)
        ({"identity": {"_verifier": hint, "verified_at_commit": c, "inputs": {"/etc/hosts": h}}},
         "malformed"),
        ({"identity": {"_verifier": hint, "verified_at_commit": c, "inputs": {"../x": h}}},
         "malformed"),
        # ⭐ the pointer at the reader is load-bearing: without it the artifact cannot tell a human
        # what re-derives it, which is half of what this doctrine buys
        ({"identity": {"verified_at_commit": c, "inputs": {"g": h}}}, "malformed"),
        ({"identity": {"_verifier": "see the docs", "verified_at_commit": c, "inputs": {"g": h}}},
         "malformed"),
        ("not an object", "malformed"),
    ]
    misses = 0
    for obj, want in cases:
        got = read_identity(obj)[0]
        if got != want:
            print("%s: CONTROL MISSED: %r want=%s got=%s" % (TAG, obj, want, got), file=sys.stderr)
            misses += 1
    scan_cases = [
        ('{"a": 1}', 6, 7),                       # a bare scalar
        ('{"a": "x,y"}', 6, 11),                  # a comma INSIDE a string is not a terminator
        ('{"a": {"b": {"c": 1}}, "d": 2}', 6, 21),   # nesting
        ('{"a": [1, [2]], "d": 2}', 6, 14),
        ('{"a": {"b": "}"}, "d": 2}', 6, 16),      # ⛔ a brace inside a string must not close it
        ('{"a": "he said \\"}\\"", "d": 2}', 6, 21),  # …nor an ESCAPED quote inside one
    ]
    for text, start, want in scan_cases:
        try:
            got = scan_value_end(text, start)
        except ValueError as exc:
            got = "raised %s" % exc
        if got != want:
            print("%s: CONTROL MISSED: scan_value_end(%r, %d) want=%d got=%s"
                  % (TAG, text, start, want, got), file=sys.stderr)
            misses += 1
    if misses:
        print("%s: the identity reader / value scanner does not discriminate (%d control(s) "
              "missed); refusing" % (TAG, misses), file=sys.stderr)
        sys.exit(2)


# ── git legs ────────────────────────────────────────────────────────────────────────────────────
def git(*args):
    try:
        out = subprocess.run(["git", "-C", ROOT] + list(args), capture_output=True, text=True)
    except OSError:
        return None
    return out.stdout.strip() if out.returncode == 0 else None


def commit_leg(commit, label):
    """Is `verified_at_commit` a commit THIS repository contains, on the way to HEAD?

    ⛔ A SHALLOW CLONE IS A LEGITIMATE STATE, NOT A BREACH. `actions/checkout` fetches depth 1 by
    default, so the parent commit a stamp names is genuinely absent there. Reporting that as a
    refusal would make this doctrine fail on every hosted run — so it is NOT EVALUATED, named, and
    the input-sha legs (which need no history at all) still bind."""
    if git("rev-parse", "--git-dir") is None:
        unevaluated.append("%s: not a git repository, so `verified_at_commit` cannot be checked"
                           % label)
        return
    if git("rev-parse", "--is-shallow-repository") == "true":
        unevaluated.append("%s: shallow clone — `verified_at_commit` %s cannot be resolved here"
                           % (label, commit[:12]))
        return
    if git("cat-file", "-e", commit + "^{commit}") is None:
        fail("%s: `verified_at_commit` %s is not a commit this repository contains. A stamp names "
             "the tree it was verified against; a sha nothing can resolve names nothing."
             % (label, commit[:12]))
        return
    if subprocess.run(["git", "-C", ROOT, "merge-base", "--is-ancestor", commit, "HEAD"],
                      capture_output=True).returncode != 0:
        fail("%s: `verified_at_commit` %s is not an ancestor of HEAD — this baseline was verified "
             "against a tree that is not in your history." % (label, commit[:12]))


# ── the single verifier every gate calls ────────────────────────────────────────────────────────
def verify(rel, quiet=False):
    """Returns 0 fresh / 1 stale / 2 refuse. The refusal wording is produced HERE and nowhere
    else, so every gate in the repository refuses in provably identical words."""
    path = os.path.join(ROOT, rel)
    if not os.path.isfile(path):
        refuse("%s: no such baseline: %s" % (TAG, rel))
        return 2
    if not rel.endswith(".json"):
        refuse("%s: %s is not JSON. The identity block has a JSON carrier only; a carrier for the "
               "`.env` baselines is owed by SV-CORPUS-GRAD.13c.2x.2 and until it lands such a "
               "baseline must be registered `deferred`, never `adopted`." % (TAG, rel))
        return 2
    with open(path, encoding="utf-8") as fh:
        try:
            obj = json.load(fh)
        except ValueError as exc:
            refuse("%s: %s is not parseable JSON: %s" % (TAG, rel, exc))
            return 2
    status, payload = read_identity(obj)
    if status == "absent":
        refuse("%s: REFUSING — %s carries NO identity block. A baseline that cannot say what "
               "produced it cannot be checked for staleness.\n"
               "    Adopt it:  bash scripts/check_baseline_identity.sh --stamp %s --input <path>…"
               % (TAG, rel, rel))
        return 2
    if status == "malformed":
        refuse("%s: REFUSING — %s carries a MALFORMED identity block: %s" % (TAG, rel, payload))
        return 2

    commit, declared = payload
    commit_leg(commit, rel)

    stale, checked = [], []
    for dep, want in sorted(declared.items()):
        dpath = os.path.join(ROOT, dep)
        if not os.path.isfile(dpath):
            unevaluated.append(
                "%s: declared input `%s` is absent, so it was NOT re-hashed%s"
                % (rel, dep,
                   " — generated/ is not tracked; regenerate with `make -C rust "
                   "SHELL=/bin/bash regenerate_generated_parsers`" if dep.startswith("generated/")
                   else ""))
            continue
        got = sha256_of(dpath)
        checked.append(dep)
        if got != want:
            stale.append((dep, want, got))

    if stale:
        fail("%s: THE BASELINE IS STALE — %s no longer describes this tree.\n" % (TAG, rel)
             + "".join("        %s: baseline `%s…` vs live `%s…`\n" % (d, w[:16], g[:16])
                       for d, w, g in stale)
             + "      Every expectation in this baseline is a function of the inputs above, so a\n"
               "      mismatch downstream is UNDIAGNOSED until this is resolved: it may be a real\n"
               "      regression, or it may be this baseline. Re-derive the expectations, then:\n"
               "        bash scripts/check_baseline_identity.sh --stamp %s" % rel)
        return 1

    if not checked:
        refuse("%s: REFUSING — %s declares %d input(s) and NONE of them could be hashed here, so "
               "this check evaluated nothing. A check that cannot see must say so, not pass."
               % (TAG, rel, len(declared)))
        return 2
    if not quiet:
        print("%s: OK — %s identity fresh for: %s" % (TAG, rel, ", ".join(checked)))
    return 0


# ── --stamp: the only supported way to (re)derive a block ───────────────────────────────────────
def stamp(rel, extra_inputs):
    path = os.path.join(ROOT, rel)
    if not os.path.isfile(path) or not rel.endswith(".json"):
        print("%s: --stamp needs an existing tracked .json baseline: %s" % (TAG, rel),
              file=sys.stderr)
        return 2
    with open(path, encoding="utf-8") as fh:
        text = fh.read()
    obj = json.loads(text)
    status, payload = read_identity(obj)
    declared = list(payload[1].keys()) if status == "ok" else []
    inputs = list(dict.fromkeys(declared + list(extra_inputs)))
    if not inputs:
        print("%s: --stamp on an unadopted baseline must name its dependency set: "
              "--input <repo-root-relative path> (repeatable)" % TAG, file=sys.stderr)
        return 2
    digests = {}
    for dep in sorted(inputs):
        dpath = os.path.join(ROOT, dep)
        if not os.path.isfile(dpath):
            # ⛔ A STAMP IS THE ACT THAT ASSERTS "this baseline describes THIS tree". It cannot
            # assert that about a file it never read, so an absent input refuses here even though
            # --verify reports it NOT EVALUATED: verifying with a partial view is honest, stamping
            # with one is a lie written into a tracked artifact.
            print("%s: --stamp REFUSING — declared input `%s` does not exist, so it cannot be "
                  "hashed." % (TAG, dep), file=sys.stderr)
            return 2
        digests[dep] = sha256_of(dpath)
    head = git("rev-parse", "HEAD")
    if head is None or not COMMIT_RE.match(head or ""):
        print("%s: --stamp REFUSING — cannot resolve HEAD, so `verified_at_commit` would be a "
              "guess." % TAG, file=sys.stderr)
        return 2

    block = ('  "identity": {\n'
             '    "_verifier": "bash %s --verify %s (doctrine BASELINE-IDENTITY, '
             'SV-CORPUS-GRAD.13c.2x.2). DERIVED — re-stamp with --stamp, never hand-edit.",\n'
             '    "_what_this_proves": "every expectation in this file was derived from the '
             'inputs below, at the commit below. The enforcer re-hashes them on every run: if one '
             'differs, this baseline no longer describes your tree and the honest act is to '
             're-derive, not to quote.",\n'
             '    "verified_at_commit": "%s",\n'
             '    "inputs": {\n%s\n    }\n'
             '  }' % (VERIFIER_HINT, rel, head,
                      ",\n".join('      %s: "%s"' % (json.dumps(d), digests[d])
                                 for d in sorted(digests))))

    key = '"identity"'
    at = text.find("\n  " + key)
    if at >= 0:
        start = at + 1
        colon = text.index(":", start + len(key) + 2)
        vstart = colon + 1
        while text[vstart] in " \t\n":
            vstart += 1
        end = scan_value_end(text, vstart)
        new = text[:start] + block + text[end:]
    else:
        brace = text.index("{")
        eol = text.index("\n", brace) + 1
        new = text[:eol] + block + ",\n" + text[eol:]
    json.loads(new)          # never write a file this repository cannot parse
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(new)
    print("%s: stamped %s at %s over %d input(s): %s"
          % (TAG, rel, head[:12], len(digests), ", ".join(sorted(digests))))
    return 0


# ── the CLOSED population ───────────────────────────────────────────────────────────────────────
def load_register():
    if not os.path.isfile(REGISTER):
        refuse("%s: REFUSING — the register %s is missing, so the population is not closed and "
               "a new baseline could be added with no verdict." % (TAG, REGISTER))
        return None
    with open(REGISTER, encoding="utf-8") as fh:
        try:
            reg = json.load(fh)
        except ValueError as exc:
            refuse("%s: REFUSING — %s is not parseable JSON: %s" % (TAG, REGISTER, exc))
            return None
    if reg.get("register") != "baseline_identity_register_v0" or not isinstance(
            reg.get("entries"), list):
        refuse("%s: REFUSING — %s is not a baseline_identity_register_v0 with an `entries` array"
               % (TAG, REGISTER))
        return None
    # ⛔ THE VOCABULARY IS DEFINED IN THIS ENFORCER AND MIRRORED IN THE ARTIFACT, AND THE TWO ARE
    # HELD EQUAL. A register that documents dispositions the code does not implement teaches a
    # reader a rule that is not enforced — which is the same defect as an unread identity block.
    if reg.get("dispositions") != DISPOSITIONS:
        refuse("%s: REFUSING — %s's `dispositions` block disagrees with the enforcer's own "
               "vocabulary. Update the artifact to match scripts/check_baseline_identity.sh, or "
               "change both together." % (TAG, REGISTER))
        return None
    return reg


def leaf_exists(leaf):
    """`SV-CORPUS-GRAD.13c.2x.2` -> docs/tasks/SV-CORPUS-GRAD.md must exist AND name the leaf.
    An owner nothing can find is not an owner (the WAIVER-ROUTING shape, one surface over)."""
    tree = leaf.split(".", 1)[0]
    tpath = os.path.join(ROOT, "docs", "tasks", tree + ".md")
    if not os.path.isfile(tpath):
        return False
    with open(tpath, encoding="utf-8", errors="replace") as fh:
        body = fh.read()
    return ("`.%s`" % leaf.split(".", 1)[1]) in body or ("`%s`" % leaf) in body


def population():
    """RE-DERIVED every run. Never hand-listed — a hand-listed population is a census, and a
    census is what this doctrine exists to replace."""
    return sorted(os.listdir(os.path.join(ROOT, BASE_DIR)))


def register_mode(report=False):
    reg = load_register()
    if reg is None:
        return 2
    live = population()
    rows = {}
    for row in reg["entries"]:
        name = row.get("entry")
        if not isinstance(name, str):
            refuse("%s: register row with no string `entry`: %r" % (TAG, row))
            continue
        if name in rows:
            fail("%s: register names `%s` twice, so one of the two verdicts is unreachable"
                 % (TAG, name))
        rows[name] = row

    for name in live:
        if name not in rows:
            fail("%s: `%s/%s` is in the baseline directory with NO register row. Every entry needs "
                 "a verdict — that is the mechanism that stops this population re-growing "
                 "silently. Add a row with one of: %s"
                 % (TAG, BASE_DIR, name, ", ".join(sorted(DISPOSITIONS))))
    for name in sorted(rows):
        if name not in live:
            fail("%s: register row `%s` names nothing in %s/. A disposition that no longer "
                 "describes anything is a dead exemption and must be removed."
                 % (TAG, name, BASE_DIR))

    counts = dict((d, 0) for d in DISPOSITIONS)
    for name in live:
        row = rows.get(name)
        if row is None:
            continue
        disp = row.get("disposition")
        if disp not in DISPOSITIONS:
            fail("%s: `%s` carries disposition %r, which is not in the vocabulary (%s)"
                 % (TAG, name, disp, ", ".join(sorted(DISPOSITIONS))))
            continue
        counts[disp] += 1
        if not isinstance(row.get("reason"), str) or len(row.get("reason", "")) < 24:
            fail("%s: `%s` carries no substantive `reason`. A disposition without one is an "
                 "assertion nobody can review." % (TAG, name))
        rel = os.path.join(BASE_DIR, name)
        is_dir = os.path.isdir(os.path.join(ROOT, rel))

        if disp == "corpus-directory":
            if not is_dir:
                fail("%s: `%s` is registered `corpus-directory` but is not a directory" % (TAG, name))
            continue
        if is_dir:
            fail("%s: `%s` is a directory but is registered `%s`" % (TAG, name, disp))
            continue

        if disp == "adopted":
            verify(rel, quiet=not report)
        else:
            # ⛔⛔ THE ANTI-`.13i` RULE, AND IT IS THE POINT OF THIS WHOLE ENFORCER. `.13i` measured
            # six oracles carrying an identity block that NOTHING read, four of them stale. A block
            # on a row this enforcer does not verify would recreate exactly that — so carrying one
            # while registered anything but `adopted` is a hard failure, not a nudge.
            if rel.endswith(".json"):
                try:
                    with open(os.path.join(ROOT, rel), encoding="utf-8") as fh:
                        st = read_identity(json.load(fh))[0]
                except ValueError:
                    st = "absent"
                if st != "absent":
                    fail("%s: `%s` is registered `%s` yet CARRIES an identity block. Nothing "
                         "re-hashes it, which is the SV-CORPUS-GRAD.13i defect verbatim: a block "
                         "that exists and is not read. Promote the row to `adopted` or remove the "
                         "block." % (TAG, name, disp))
            if disp == "identity-native":
                by = row.get("verified_by")
                if not isinstance(by, str) or not by:
                    fail("%s: `%s` is `identity-native` with no `verified_by`. The class exists "
                         "only because another doctrine already re-derives the artifact; unnamed, "
                         "that is a claim rather than a route." % (TAG, name))
                elif not os.path.isfile(os.path.join(ROOT, by)):
                    fail("%s: `%s` names verified_by `%s`, which does not exist"
                         % (TAG, name, by))
            if disp == "deferred":
                owner = row.get("owner_leaf")
                if not isinstance(owner, str) or not owner:
                    fail("%s: `%s` is `deferred` with no `owner_leaf`. An unowned deferral is a "
                         "buried finding." % (TAG, name))
                elif not leaf_exists(owner):
                    fail("%s: `%s` names owner_leaf `%s`, which no docs/tasks/ tree mentions"
                         % (TAG, name, owner))

    if report:
        print("%s: %d entries in %s/" % (TAG, len(live), BASE_DIR))
        for d in sorted(DISPOSITIONS):
            print("    %-24s %3d" % (d, counts[d]))
        for name in live:
            row = rows.get(name, {})
            print("    %-24s %s" % (row.get("disposition", "<UNREGISTERED>"), name))
    return None


# ── driver ──────────────────────────────────────────────────────────────────────────────────────
self_check()

argv = sys.argv[1:]
rc = 0
if argv and argv[0] == "--verify":
    if len(argv) != 2:
        print("usage: --verify <repo-root-relative baseline path>", file=sys.stderr)
        sys.exit(2)
    rc = verify(argv[1])
elif argv and argv[0] == "--stamp":
    rest, extra = argv[1:], []
    if not rest:
        print("usage: --stamp <file> [--input <path>]...", file=sys.stderr)
        sys.exit(2)
    target, i = rest[0], 1
    while i < len(rest):
        if rest[i] == "--input" and i + 1 < len(rest):
            extra.append(rest[i + 1]); i += 2
        else:
            print("%s: unexpected --stamp argument: %s" % (TAG, rest[i]), file=sys.stderr)
            sys.exit(2)
    sys.exit(stamp(target, extra))
elif argv and argv[0] == "--report":
    register_mode(report=True)
elif not argv:
    register_mode(report=False)
else:
    print("%s: unknown argument: %s" % (TAG, argv[0]), file=sys.stderr)
    sys.exit(2)

for m in refusals:
    print(m, file=sys.stderr)
for m in failures:
    print(m, file=sys.stderr)
if unevaluated:
    print("%s: NOT EVALUATED (%d) — reported rather than counted as fresh:" % (TAG, len(unevaluated)),
          file=sys.stderr)
    for m in unevaluated:
        print("    %s" % m, file=sys.stderr)
if refusals:
    sys.exit(2)
if failures:
    sys.exit(1)
sys.exit(rc)
PYEOF
