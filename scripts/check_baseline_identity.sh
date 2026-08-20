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
#   bash scripts/check_baseline_identity.sh --digest <kind> FILE            # one digest, for a caller
#   bash scripts/check_baseline_identity.sh --raw-ast FILE.ebnf # the frontend envelope the semantic
#                                                               # digest is taken over, for a caller
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
AST_PIPELINE = "rust/target/debug/ast_pipeline"

# ⭐ DIGEST KINDS. `bytes` is the literal file. A SEMANTIC kind digests what the artifact's
# CONSUMER actually sees, so an edit the consumer provably cannot observe does not stale anything.
#
# ⛔ THIS IS NOT A NEW IDEA IN THIS REPOSITORY AND MUST NOT BECOME A SECOND ONE.
# `scripts/check_sv_contract_currency.sh::sv_semantic_digest` already keys that whole doctrine on
# the EBNF frontend's `raw_ast` envelope — "what the code generator consumes; comments never reach
# it" — and `ebnf_raw_ast` below is byte-for-byte the same definition, deliberately.
#
# ⛔⛔ WHY IT IS LOAD-BEARING RATHER THAN A NICETY (measured 2026-08-20, SV-CORPUS-GRAD.13c.2x.4):
# ONE COMMENT LINE appended to grammars/systemverilog.ebnf made this enforcer exit 1, which made
# check_doctrines.sh exit 1, which BLOCKS EVERY COMMIT — for a change that leaves the generated
# parser byte-identical. False staleness is not a small tax; it is the whole adoption cost.
DIGEST_KINDS = ("bytes", "ebnf_raw_ast")
COMMIT_RE = re.compile(r"^[0-9a-f]{40}$")
VERIFIER_HINT = "scripts/check_baseline_identity.sh"

# The disposition vocabulary. A row may hold nothing else; the register's own copy of these
# strings is held equal to this dict, so the two cannot drift.
DISPOSITIONS = {
    "adopted":
        "carries an `identity` block whose expectations are CONFIRMED against the digests it "
        "records, and this enforcer re-hashes every input it declares on every run",
    "adopted-unconfirmed":
        "carries an `identity` block that says, in the artifact itself, that its expectations do "
        "NOT describe the recorded tree. The digests still track movement so drift is still "
        "caught, `owner_leaf` owns the re-derivation, and every gate calling `--verify` REFUSES — "
        "so this is declared, owned debt, never a green light",
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
    # ⛔⛔ THE FIELD THAT SLICE 1 SHIPPED WITHOUT, AND THE OMISSION INVERTED THE DOCTRINE.
    # `verified_at_commit` + `inputs` answer *"have the inputs moved since this checkpoint?"*.
    # They do NOT answer *"were these expectations ever right about that tree?"* — and stamping a
    # baseline that is measurably RED made the block assert a derivation that never happened, so
    # the gate reported `identity fresh` and then an UNATTRIBUTABLE drift. That is worse than no
    # block: it is a confident wrong answer. `expectations` is the missing half, and it is
    # REQUIRED, because a default would silently re-create exactly this.
    state = ident.get("expectations")
    if state not in ("confirmed", "unconfirmed"):
        return ("malformed",
                "`identity.expectations` must be \"confirmed\" or \"unconfirmed\". A stamp is an "
                "ASSERTION about whether this file's numbers describe the recorded tree; there is "
                "no honest default")
    if state == "confirmed":
        by = ident.get("confirmed_by")
        if not isinstance(by, str) or len(by) < 24:
            return ("malformed",
                    "`identity.expectations` is \"confirmed\" but `confirmed_by` does not name the "
                    "run that confirmed it. A confirmation nobody can re-check is a claim")
    else:
        why = ident.get("unconfirmed_reason")
        owner = ident.get("owner_leaf")
        if not isinstance(why, str) or len(why) < 24:
            return ("malformed",
                    "`identity.expectations` is \"unconfirmed\" but `unconfirmed_reason` does not "
                    "say why")
        if not isinstance(owner, str) or not owner:
            return ("malformed",
                    "`identity.expectations` is \"unconfirmed\" but no `owner_leaf` owns the "
                    "re-derivation. Unowned debt is buried debt")
    inputs = ident.get("inputs")
    if not isinstance(inputs, dict) or not inputs:
        return ("malformed", "`identity.inputs` must be a non-empty object of path -> sha256")
    normalised = {}
    for path, spec in inputs.items():
        if not isinstance(path, str) or not path or path.startswith("/") or ".." in path.split("/"):
            return ("malformed",
                    "input path %r is not repo-root-relative (CLAUDE.md §12)" % (path,))
        # A bare string is the `bytes` kind — the original shape, still valid, still the default.
        if isinstance(spec, str):
            kind, digest = "bytes", spec
        elif isinstance(spec, dict):
            kind, digest = spec.get("kind"), spec.get("digest")
            if kind not in DIGEST_KINDS:
                return ("malformed",
                        "input `%s` declares digest kind %r; known kinds are %s"
                        % (path, kind, ", ".join(DIGEST_KINDS)))
        else:
            return ("malformed",
                    "input `%s` must be a sha256 string or a {kind, digest} object" % (path,))
        if not isinstance(digest, str) or not SHA256_RE.match(digest):
            return ("malformed", "input `%s` does not carry a 64-hex sha256" % (path,))
        normalised[path] = (kind, digest)
    return ("ok", (commit, normalised, state, ident))


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
    CB = "re-derived by `make -C rust <gate>` at seeds 0/7/42, 2026-08-20"
    UR = "measured RED at HEAD by 71 rules; re-derivation is blocked on SV-CORPUS-GRAD.13c.2x.1"

    def blk(**kw):
        b = {"_verifier": hint, "verified_at_commit": c, "expectations": "confirmed",
             "confirmed_by": CB, "inputs": {"grammars/systemverilog.ebnf": h}}
        b.update(kw)
        return {"identity": b}

    cases = [
        # the two shapes that must be READ
        (blk(), "ok"),
        (blk(expectations="unconfirmed", confirmed_by=None, unconfirmed_reason=UR,
             owner_leaf="SV-CORPUS-GRAD.13c.2x.2"), "ok"),
        # not adopted — a different fact from a broken block
        ({"expected_total": 1362}, "absent"),
        # ⭐⭐ THE STATE CONTROLS. `expectations` is the field whose ABSENCE inverted this doctrine
        # in slice 1: a block with no state asserted a derivation that had not happened, so the
        # consuming gate reported `identity fresh` and then an UNATTRIBUTABLE drift — a confident
        # wrong answer, which is worse than no block at all. There is no default, by construction.
        (blk(expectations=None), "malformed"),
        (blk(expectations="maybe"), "malformed"),
        (blk(confirmed_by=None), "malformed"),
        (blk(confirmed_by="ok"), "malformed"),                       # too short to be re-checkable
        (blk(expectations="unconfirmed", confirmed_by=None), "malformed"),        # no reason
        (blk(expectations="unconfirmed", confirmed_by=None, unconfirmed_reason=UR),
         "malformed"),                                                            # no owner
        # ⭐ THE DIGEST-KIND CONTROLS. A bare string stays the `bytes` kind — the original shape
        # must keep working — and a {kind, digest} object is the semantic form. Everything else is
        # malformed, because a mis-typed kind that fell back to `bytes` would silently restore the
        # false staleness this shape exists to remove.
        (blk(inputs={"grammars/x.ebnf": {"kind": "ebnf_raw_ast", "digest": h}}), "ok"),
        (blk(inputs={"grammars/x.ebnf": {"kind": "bytes", "digest": h}}), "ok"),
        (blk(inputs={"grammars/x.ebnf": {"kind": "raw_ast", "digest": h}}), "malformed"),
        (blk(inputs={"grammars/x.ebnf": {"kind": "ebnf_raw_ast"}}), "malformed"),
        (blk(inputs={"grammars/x.ebnf": {"digest": h}}), "malformed"),
        (blk(inputs={"grammars/x.ebnf": 42}), "malformed"),
        (blk(inputs={"grammars/x.ebnf": {"kind": "ebnf_raw_ast", "digest": "deadbeef"}}),
         "malformed"),
        # ⛔ every one of these once passed a reader that only asked `if obj.get("identity")`
        ({"identity": "yes"}, "malformed"),
        ({"identity": {}}, "malformed"),
        (blk(inputs={}), "malformed"),
        (blk(inputs={"g.ebnf": "deadbeef"}), "malformed"),
        (blk(verified_at_commit="abc"), "malformed"),
        # an ABSOLUTE path would hash something outside the repo (CLAUDE.md §12)
        (blk(inputs={"/etc/hosts": h}), "malformed"),
        (blk(inputs={"../x": h}), "malformed"),
        # ⭐ the pointer at the reader is load-bearing: without it the artifact cannot tell a human
        # what re-derives it, which is half of what this doctrine buys
        (blk(_verifier=None), "malformed"),
        (blk(_verifier="see the docs"), "malformed"),
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


def live_digest(path, kind):
    """The digest of `path` under `kind` today, or None when it cannot be computed here.

    ⛔ None is NOT a mismatch. An unbuildable semantic digest is NOT EVALUATED and says so; scoring
    it as a difference would fail every clean checkout, which is the failure mode this whole
    doctrine exists to remove one level up."""
    full = os.path.join(ROOT, path)
    if not os.path.isfile(full):
        return None
    if kind == "bytes":
        return sha256_of(full)
    if kind == "ebnf_raw_ast":
        raw = raw_ast_envelope(path)
        if raw is None:
            return None
        return hashlib.sha256(canonical_raw_ast(raw).encode()).hexdigest()
    return None


# ⭐ THE ENVELOPE ITSELF, EXPOSED (SV-CORPUS-GRAD.13c.2w). The `ebnf_raw_ast` digest and the
# grammar's rule-REFERENCE GRAPH are two products of ONE derivation, and the graph is what
# `PARSE-COST-RATCHET`'s containment invariant reads. Splitting the frontend invocation across two
# scripts would put the envelope's definition in two places — which is the shape
# `ENGINE-UNIVERSAL-SERVICES.38` measured, where one grammar was keyed by BYTES in two files
# because the same line had been copied. ⇒ one derivation here, two consumers (`--digest` and
# `--raw-ast`).
def raw_ast_envelope(rel):
    """The EBNF frontend's `raw_ast` for one grammar, or None when it cannot be derived here."""
    full = os.path.join(ROOT, rel)
    if not os.path.isfile(full):
        return None
    binary = os.path.join(ROOT, AST_PIPELINE)
    if not os.path.isfile(binary) or not os.access(binary, os.X_OK):
        return None
    # ⛔ ON-VOLUME BY POLICY (CLAUDE.md §13): scratch is derived from the repo root, never
    # $TMPDIR, which can sit on a different filesystem.
    scratch = os.path.join(ROOT, "rust", "target", "baseline_identity_digest")
    try:
        os.makedirs(scratch, exist_ok=True)
    except OSError:
        return None
    out = os.path.join(scratch, "raw_ast.%d.json" % os.getpid())
    try:
        rc = subprocess.run([binary, full, "--emit-raw-ast-json", out],
                            capture_output=True).returncode
        if rc != 0 or not os.path.isfile(out):
            return None
        with open(out, encoding="utf-8") as fh:
            return json.load(fh)["raw_ast"]
    except (OSError, ValueError, KeyError):
        return None
    finally:
        try:
            os.remove(out)
        except OSError:
            pass


def canonical_raw_ast(raw):
    """The ONE serialisation the `ebnf_raw_ast` digest is taken over."""
    return json.dumps(raw, sort_keys=True, separators=(",", ":"))


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
def verify(rel, quiet=False, want_state="confirmed", stale_sink=None):
    """0 fresh+confirmed · 1 STALE · 2 REFUSE · 3 UNCONFIRMED.

    ⛔⛔ 1 AND 3 ARE SPLIT BECAUSE THEY CALL FOR OPPOSITE ACTIONS, and collapsing them is what made
    the first cut of this doctrine block commits (SV-CORPUS-GRAD.13c.2x.4):
      1 STALE       — an input moved and NOBODY has adjudicated the result. RUNNING RESOLVES IT:
                      the constraints either hold (the baseline was stale and still correct) or
                      they do not (stale AND diverging). A gate should measure, then decide.
      3 UNCONFIRMED — a human has already recorded that these numbers are wrong. Running teaches
                      nothing, so a gate should refuse BEFORE spending the measurement.
    The wording for both is produced HERE and nowhere else, so every gate refuses identically.

    `want_state` is what the CALLER requires. A gate about to spend minutes measuring requires
    `confirmed`; the register's `adopted-unconfirmed` rows require `unconfirmed`, so a baseline
    that has since been confirmed cannot sit in the debt class unnoticed."""
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

    commit, declared, state, ident = payload
    if state != want_state:
        if want_state == "confirmed":
            rc_for_state = 3
            fail("%s: THE EXPECTATIONS IN %s ARE UNCONFIRMED — the artifact says so itself.\n"
                 "        reason: %s\n"
                 "        owner : %s\n"
                 "      The recorded digests still track whether the inputs MOVED, but nobody has\n"
                 "      re-derived these numbers against them. Any drift a consumer measures here\n"
                 "      is UNATTRIBUTABLE: it could be a real regression, or it could be this\n"
                 "      baseline. Re-derive, then:\n"
                 "        bash scripts/check_baseline_identity.sh --stamp %s --confirmed-by \"…\""
                 % (TAG, rel, ident.get("unconfirmed_reason", "?"),
                    ident.get("owner_leaf", "?"), rel))
        else:
            rc_for_state = 1
            fail("%s: %s is registered as UNCONFIRMED debt but its block now says `confirmed`. "
                 "Promote the register row to `adopted`, or the debt class hides a finished "
                 "baseline." % (TAG, rel))
        return rc_for_state
    commit_leg(commit, rel)

    stale, checked = [], []
    for dep, (kind, want) in sorted(declared.items()):
        got = live_digest(dep, kind)
        if got is None:
            unevaluated.append(
                "%s: declared input `%s` (kind %s) could not be digested here, so it was NOT "
                "checked%s" % (rel, dep, kind,
                               " — generated/ is not tracked; regenerate with `make -C rust "
                               "SHELL=/bin/bash regenerate_generated_parsers`"
                               if dep.startswith("generated/") else
                               " — build it with `make -C rust ast_pipeline`"
                               if kind != "bytes" else ""))
            continue
        checked.append("%s[%s]" % (dep, kind))
        if got != want:
            stale.append((dep, kind, want, got))

    if stale:
        (stale_sink.append if stale_sink is not None else fail)(
             "%s: THE BASELINE IS STALE — %s no longer describes this tree.\n" % (TAG, rel)
             + "".join("        %s [%s]: baseline `%s…` vs live `%s…`\n"
                       % (d, k, w[:16], g[:16]) for d, k, w, g in stale)
             + "      ⭐ STALE IS NOT A VERDICT ON THE TREE. Running the owning gate resolves it:\n"
               "        constraints GREEN -> the baseline was stale and still correct, and the\n"
               "                             gate RE-STAMPS itself from that run;\n"
               "        constraints RED   -> stale AND diverging, which is the one case that\n"
               "                             genuinely needs a person.\n"
               "      Run the gate. Do not hand-edit the numbers.")
        return 1

    if not checked:
        refuse("%s: REFUSING — %s declares %d input(s) and NONE of them could be hashed here, so "
               "this check evaluated nothing. A check that cannot see must say so, not pass."
               % (TAG, rel, len(declared)))
        return 2
    if state == "unconfirmed":
        # Reached only when the CALLER asked for the debt class (the register). The inputs are
        # fresh, which is all this row promises; the expectations are still owed.
        if not quiet:
            print("%s: %s — inputs unmoved, expectations UNCONFIRMED (owed by %s)"
                  % (TAG, rel, ident.get("owner_leaf", "?")))
        return 0
    if not quiet:
        print("%s: OK — %s identity fresh and expectations CONFIRMED for: %s"
              % (TAG, rel, ", ".join(checked)))
    return 0


# ── --stamp: the only supported way to (re)derive a block ───────────────────────────────────────
def stamp(rel, extra_inputs, confirmed_by=None, unconfirmed=None, owner_leaf=None):
    """`extra_inputs` is a list of (path, kind)."""
    path = os.path.join(ROOT, rel)
    if not os.path.isfile(path) or not rel.endswith(".json"):
        print("%s: --stamp needs an existing tracked .json baseline: %s" % (TAG, rel),
              file=sys.stderr)
        return 2
    with open(path, encoding="utf-8") as fh:
        text = fh.read()
    obj = json.loads(text)
    status, payload = read_identity(obj)
    declared = ([(k, v[0]) for k, v in payload[1].items()] if status == "ok" else [])
    prior = payload[3] if status == "ok" else {}
    # ⛔ A STAMP IS AN ASSERTION, NOT A REFRESH. Carrying the prior state forward silently is how
    # a re-stamp after a grammar change would launder `unconfirmed` into `confirmed` — or the
    # reverse — without anybody deciding. State is inherited ONLY when the caller names none.
    if confirmed_by is None and unconfirmed is None:
        confirmed_by = prior.get("confirmed_by") if prior.get("expectations") == "confirmed" else None
        if prior.get("expectations") == "unconfirmed":
            unconfirmed = prior.get("unconfirmed_reason")
            owner_leaf = owner_leaf or prior.get("owner_leaf")
    if confirmed_by is None and unconfirmed is None:
        print("%s: --stamp REFUSING — a stamp asserts whether this file's numbers describe the\n"
              "  recorded tree, and there is no honest default. Say which:\n"
              "    --confirmed-by \"<the run that re-derived them>\"\n"
              "    --unconfirmed \"<why they do not>\" --owner-leaf <TREE.leaf>" % TAG,
              file=sys.stderr)
        return 2
    if confirmed_by is not None and unconfirmed is not None:
        print("%s: --stamp REFUSING — --confirmed-by and --unconfirmed are contradictory" % TAG,
              file=sys.stderr)
        return 2
    # A path named again on the command line overrides its previously declared KIND, which is the
    # only supported way to migrate an input from `bytes` to a semantic digest.
    merged = dict(declared)
    merged.update(dict(extra_inputs))
    if not merged:
        print("%s: --stamp on an unadopted baseline must name its dependency set:\n"
              "    --input <path>            (digest the file's BYTES)\n"
              "    --input-raw-ast <path>    (digest the EBNF frontend's raw_ast envelope —\n"
              "                               comment- and layout-insensitive by construction)"
              % TAG, file=sys.stderr)
        return 2
    digests = {}
    for dep in sorted(merged):
        kind = merged[dep]
        got = live_digest(dep, kind)
        if got is None:
            # ⛔ A STAMP IS THE ACT THAT ASSERTS "this baseline describes THIS tree". It cannot
            # assert that about an input it never digested, so this refuses here even though
            # --verify reports the same input NOT EVALUATED: verifying with a partial view is
            # honest, stamping with one writes a lie into a tracked artifact.
            print("%s: --stamp REFUSING — declared input `%s` (kind %s) could not be digested."
                  % (TAG, dep, kind), file=sys.stderr)
            return 2
        digests[dep] = (kind, got)
    head = git("rev-parse", "HEAD")
    if head is None or not COMMIT_RE.match(head or ""):
        print("%s: --stamp REFUSING — cannot resolve HEAD, so `verified_at_commit` would be a "
              "guess." % TAG, file=sys.stderr)
        return 2

    if confirmed_by is not None:
        state_rows = ('    "expectations": "confirmed",\n'
                      '    "confirmed_by": %s,\n' % json.dumps(confirmed_by))
        proves = ("the expectations in this file were RE-DERIVED against the inputs below, at the "
                  "commit below, by the run named in `confirmed_by`. The enforcer re-hashes those "
                  "inputs on every run: if one differs, this baseline no longer describes your "
                  "tree and the honest act is to re-derive, not to quote.")
    else:
        if not owner_leaf:
            print("%s: --stamp REFUSING — --unconfirmed needs --owner-leaf; unowned debt is "
                  "buried debt" % TAG, file=sys.stderr)
            return 2
        state_rows = ('    "expectations": "unconfirmed",\n'
                      '    "unconfirmed_reason": %s,\n'
                      '    "owner_leaf": %s,\n'
                      % (json.dumps(unconfirmed), json.dumps(owner_leaf)))
        proves = ("the expectations in this file are KNOWN NOT to describe the tree recorded "
                  "below — see `unconfirmed_reason`. The digests are still re-hashed every run, "
                  "so input drift is still caught, but every gate calling --verify REFUSES to "
                  "measure: a drift measured against an unconfirmed baseline is unattributable.")
    block = ('  "identity": {\n'
             '    "_verifier": "bash %s --verify %s (doctrine BASELINE-IDENTITY, '
             'SV-CORPUS-GRAD.13c.2x.2). DERIVED — re-stamp with --stamp, never hand-edit.",\n'
             '    "_what_this_proves": "%s",\n'
             '%s'
             '    "verified_at_commit": "%s",\n'
             '    "inputs": {\n%s\n    }\n'
             '  }' % (VERIFIER_HINT, rel, proves, state_rows, head,
                      ",\n".join(
                          ('      %s: "%s"' % (json.dumps(d), digests[d][1]))
                          if digests[d][0] == "bytes" else
                          ('      %s: { "kind": "%s", "digest": "%s" }'
                           % (json.dumps(d), digests[d][0], digests[d][1]))
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
    print("%s: stamped %s at %s as %s over %d input(s): %s"
          % (TAG, rel, head[:12], "CONFIRMED" if confirmed_by is not None else "UNCONFIRMED",
             len(digests),
             ", ".join("%s[%s]" % (d, digests[d][0]) for d in sorted(digests))))
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


# ⭐⭐ THE BUDGET IS DERIVED, NEVER STORED (`docs/DERIVED_STATE_CONTAINMENT.md` R1/R3). A written
# "commits since" counter would need a commit to update, which increments it — the exact
# by-construction wrongness the push counter had.
DEFAULT_STALE_BUDGET = 20


def input_commits_since(commit, paths):
    """How many commits since `commit` touched any declared input. None when underivable."""
    if git("rev-parse", "--git-dir") is None:
        return None
    if git("rev-parse", "--is-shallow-repository") == "true":
        return None
    out = git("rev-list", "--count", "%s..HEAD" % commit, "--", *paths)
    if out is None or not out.isdigit():
        return None
    return int(out)


MAKEFILE_TEXT = ""
try:
    with open(os.path.join(ROOT, "rust", "Makefile"), encoding="utf-8", errors="replace") as _fh:
        MAKEFILE_TEXT = _fh.read()
except OSError:
    pass


def stale_rows():
    """[(entry, resolved_by, cost_seconds)] for every ADOPTED baseline that is stale right now."""
    reg = load_register()
    out = []
    if reg is None:
        return out
    for row in reg["entries"]:
        if row.get("disposition") != "adopted":
            continue
        rel = os.path.join(BASE_DIR, row["entry"])
        before = len(failures)
        sink = []
        rc_row = verify(rel, quiet=True, stale_sink=sink)
        del failures[before:]          # this is a REPORT, not a verdict
        if rc_row == 1 and sink:
            out.append((row["entry"], row.get("resolved_by"), row.get("cost_seconds") or 0))
    # ⭐ ASCENDING COST: the cheapest gate that can clear a row runs first, so an operator who
    # interrupts the sweep has still cleared the cheap ones.
    out.sort(key=lambda r: (r[2], r[0]))
    return out


# ── THE STRUCTURAL INVARIANT: NO TRACKED PROVENANCE MAY KEY A GRAMMAR BY BYTES ─────────────────
#
# ⛔⛔ THIS IS THE "ONCE AND FOR GOOD" HALF. Re-keying the two rows that were wrong fixes today;
# it does nothing about the next provenance artifact somebody writes. The defect class is exact
# and mechanically checkable: **a `.ebnf` path paired with a digest, where the digest is of the
# FILE'S BYTES.** The EBNF frontend strips comments, so such a row moves on edits that provably
# cannot change anything derived from the grammar — and this doctrine's own history is what a
# missing guard costs: two such rows in one doctrine, one of them present since its founding
# commit, together blocking EVERY COMMIT on a comment.
#
# ⭐ THE POPULATION IS DERIVED FROM `git ls-files`, never hand-listed — a new artifact is covered
# by construction. Two carriers, both STRUCTURAL so prose cannot false-positive:
#   JSON  — an object carrying a `path` ending `.ebnf` beside a digest, or an `identity.inputs`
#           key ending `.ebnf`;
#   MD    — an identity-table row `| label | \`path.ebnf\` | \`<64 hex>\` |` under a tracked
#           artifacts directory. Truncated digests (`b0395cc85948…`) are not 64 hex and are
#           therefore invisible, which is what keeps changelog prose out of this.
SEMANTIC_KINDS_FOR_EBNF = ("ebnf_raw_ast",)
MD_IDENT_ROW = re.compile(
    r"^\|\s*([^|]+?)\s*\|\s*`([^`]+\.ebnf)`\s*\|\s*`[0-9a-f]{64}`\s*\|")


def grammar_byte_keying_offences():
    """Every tracked provenance row that keys a `.ebnf` input by BYTES. Derived, not listed."""
    out = []
    # ⛔ MARKDOWN IS SCOPED TO `cost.md`, AND THAT IS A CORRECTNESS BOUND RATHER THAN A SHORTCUT.
    # The first cut scanned every artifact `.md` and flagged dated CHARACTERIZATION RECORDS — e.g.
    # `…/ch22_directive_fix/after/characterization.md`. Those are EVIDENCE: they record the byte sha
    # that was true on the day they were written, and rewriting them would be falsifying history.
    # A guard must fire on LIVE provenance — a row some enforcer re-hashes — not on an archive.
    # ⚠️ Honest bound, stated: `cost.md` is the only markdown identity-table carrier in the tree
    # today, so a future one is invisible until it is named here. JSON needs no such bound — it is
    # scanned tree-wide, and every structured provenance artifact this project writes is JSON.
    listed = git("ls-files", "--", "*.json", "*/cost.md", "cost.md") or ""
    for rel in listed.splitlines():
        full = os.path.join(ROOT, rel)
        if not os.path.isfile(full):
            continue
        if rel.endswith(".md"):
            with open(full, encoding="utf-8", errors="replace") as fh:
                for n, line in enumerate(fh, 1):
                    m = MD_IDENT_ROW.match(line)
                    if m and "raw ast" not in m.group(1).lower():
                        out.append("%s:%d — identity row `%s` keys `%s` by BYTES"
                                   % (rel, n, m.group(1).strip(), m.group(2)))
            continue
        try:
            with open(full, encoding="utf-8") as fh:
                doc = json.load(fh)
        except (ValueError, OSError):
            continue

        def walk(node, where):
            if isinstance(node, dict):
                # shape A — {"path": "...ebnf", "sha256": ..., "kind": ...}
                pth = node.get("path")
                if (isinstance(pth, str) and pth.endswith(".ebnf")
                        and any(k in node for k in ("sha256", "digest"))
                        and node.get("kind") not in SEMANTIC_KINDS_FOR_EBNF):
                    out.append("%s — %s keys `%s` by BYTES (kind=%r)"
                               % (rel, where or "identity", pth, node.get("kind")))
                # shape B — an `inputs` map whose KEY is the path
                if where.endswith("identity") and isinstance(node.get("inputs"), dict):
                    for k, v in node["inputs"].items():
                        if not (isinstance(k, str) and k.endswith(".ebnf")):
                            continue
                        kind = v.get("kind") if isinstance(v, dict) else "bytes"
                        if kind not in SEMANTIC_KINDS_FOR_EBNF:
                            out.append("%s — identity.inputs keys `%s` by BYTES (kind=%r)"
                                       % (rel, k, kind))
                for k, v in node.items():
                    walk(v, where + "." + k if where else k)
            elif isinstance(node, list):
                for v in node:
                    walk(v, where)

        walk(doc, "")
    return sorted(set(out))


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

        if disp in ("adopted", "adopted-unconfirmed"):
            # ⛔⛔ STALENESS IS A STATE, NOT A FAILURE — SV-CORPUS-GRAD.13c.2x.4. This enforcer runs
            # from .githooks/pre-commit, so treating a moved input as a hard failure BLOCKED EVERY
            # COMMIT: measured 2026-08-20, one comment line in grammars/systemverilog.ebnf did it,
            # for a change that leaves the generated parser byte-identical. An identity block
            # exists to DISAMBIGUATE a gate's verdict, not to stop the work that would produce it.
            # ⭐ What must stay impossible is ROT — the founding defect was 69 input-touching
            # revisions of silence — so staleness is a NOTE only while it is within a DERIVED
            # budget, and a hard failure the moment it exceeds it.
            stale_msgs = []
            rc_row = verify(rel, quiet=not report, stale_sink=stale_msgs,
                            want_state="confirmed" if disp == "adopted" else "unconfirmed")
            if stale_msgs:
                budget = row.get("stale_budget_commits", DEFAULT_STALE_BUDGET)
                if not isinstance(budget, int) or budget < 1:
                    fail("%s: `%s` declares stale_budget_commits=%r; it must be an integer >= 1"
                         % (TAG, name, budget))
                    budget = DEFAULT_STALE_BUDGET
                paths = []
                try:
                    with open(os.path.join(ROOT, rel), encoding="utf-8") as fh:
                        paths = sorted(read_identity(json.load(fh))[1][1].keys())
                except Exception:
                    pass
                since = input_commits_since(
                    read_identity(json.load(open(os.path.join(ROOT, rel), encoding="utf-8")))[1][0],
                    paths) if paths else None
                if since is None:
                    notes.append("%s: STALE, and the drift budget could not be derived here "
                                 "(no git history). Run its gate to resolve." % name)
                elif since > budget:
                    for m in stale_msgs:
                        failures.append(m)
                    fail("%s: `%s` has been STALE across %d commits touching its declared inputs, "
                         "over its budget of %d. That is ROT, and it is the defect this doctrine "
                         "was founded on (the same artifact class rotted 69 revisions). Run its "
                         "gate — a green run re-stamps it automatically."
                         % (TAG, name, since, budget))
                else:
                    notes.append("%s: STALE but within budget (%d of %d commits touching its "
                                 "inputs). Its gate will re-stamp it on the next green run; no "
                                 "action is required to commit." % (name, since, budget))
            # ⭐⭐ "WHO RE-DERIVES THIS?" IS THE QUESTION A STALE BASELINE RAISES, so the register
            # answers it as DATA. Without it, `--stale` can report the problem and not the remedy,
            # and an operator is back to remembering which of ~120 make targets owns which file.
            # The target must EXIST in rust/Makefile — an unrunnable remedy is not a remedy.
            resolver = row.get("resolved_by")
            if not isinstance(resolver, str) or not resolver:
                fail("%s: `%s` is `%s` with no `resolved_by`. Name the make target that re-derives "
                     "it, or a stale baseline has no route back to green." % (TAG, name, disp))
            elif not re.search(r"(?m)^%s:" % re.escape(resolver), MAKEFILE_TEXT):
                fail("%s: `%s` names resolved_by `%s`, which is not a target in rust/Makefile"
                     % (TAG, name, resolver))
            if disp == "adopted-unconfirmed":
                owner = row.get("owner_leaf")
                if not isinstance(owner, str) or not owner:
                    fail("%s: `%s` is `adopted-unconfirmed` with no `owner_leaf` on its register "
                         "row. Unowned debt is buried debt." % (TAG, name))
                elif not leaf_exists(owner):
                    fail("%s: `%s` names owner_leaf `%s`, which no docs/tasks/ tree mentions"
                         % (TAG, name, owner))
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

    # ⛔ Run on EVERY invocation of the doctrine, not on demand: the whole point is that the next
    # byte-keyed grammar row cannot land, and a guard nobody runs is the defect one level up.
    for offence in grammar_byte_keying_offences():
        fail("%s: a `.ebnf` input is keyed by BYTES — %s\n"
             "      The EBNF frontend strips comments, so this row moves on edits that provably\n"
             "      cannot change anything derived from the grammar. Key it semantically:\n"
             "        \"kind\": \"ebnf_raw_ast\"   (JSON)  ·  label it `… raw ast`  (identity table)\n"
             "      The digest comes from ONE definition:\n"
             "        bash scripts/check_baseline_identity.sh --digest ebnf_raw_ast <path>"
             % (TAG, offence))

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
    confirmed_by = unconfirmed = owner_leaf = None
    while i < len(rest):
        if rest[i] == "--input" and i + 1 < len(rest):
            extra.append((rest[i + 1], "bytes")); i += 2
        elif rest[i] == "--input-raw-ast" and i + 1 < len(rest):
            extra.append((rest[i + 1], "ebnf_raw_ast")); i += 2
        elif rest[i] == "--confirmed-by" and i + 1 < len(rest):
            confirmed_by = rest[i + 1]; i += 2
        elif rest[i] == "--unconfirmed" and i + 1 < len(rest):
            unconfirmed = rest[i + 1]; i += 2
        elif rest[i] == "--owner-leaf" and i + 1 < len(rest):
            owner_leaf = rest[i + 1]; i += 2
        else:
            print("%s: unexpected --stamp argument: %s" % (TAG, rest[i]), file=sys.stderr)
            sys.exit(2)
    sys.exit(stamp(target, extra, confirmed_by, unconfirmed, owner_leaf))
elif argv and argv[0] == "--digest":
    # ⭐ ONE DEFINITION, CALLABLE. `PARSE-COST-RATCHET` needs the same semantic digest, and a third
    # in-repo copy of it would be a third thing to drift. Consumers shell out to this instead.
    if len(argv) != 3 or argv[1] not in DIGEST_KINDS:
        print("usage: --digest <%s> <repo-root-relative path>" % "|".join(DIGEST_KINDS),
              file=sys.stderr)
        sys.exit(2)
    d = live_digest(argv[2], argv[1])
    if d is None:
        print("%s: could not compute the %s digest of %s" % (TAG, argv[1], argv[2]),
              file=sys.stderr)
        sys.exit(2)
    print(d)
elif argv and argv[0] == "--raw-ast":
    # ⭐ THE SAME ENVELOPE THE DIGEST IS TAKEN OVER, EMITTED RATHER THAN HASHED. Its consumer is
    # `scripts/parse_cost_containment.py`, which derives the grammar's rule-reference graph — the
    # population `PARSE-COST-RATCHET`'s containment invariant measures reachability in. A grammar
    # is digested SEMANTICALLY here, and the graph must be derived from the same bytes the digest
    # describes, or a "fresh" stamp would sit beside a graph of a different grammar.
    if len(argv) != 2:
        print("usage: --raw-ast <repo-root-relative .ebnf path>", file=sys.stderr)
        sys.exit(2)
    raw = raw_ast_envelope(argv[1])
    if raw is None:
        print("%s: could not derive the raw_ast of %s (is %s built? "
              "make -C rust ast_pipeline)" % (TAG, argv[1], AST_PIPELINE), file=sys.stderr)
        sys.exit(2)
    print(json.dumps({"grammar": argv[1], "raw_ast": raw}, separators=(",", ":")))
elif argv and argv[0] == "--stale":
    rows = stale_rows()
    del failures[:]
    if not rows:
        print("%s: no adopted baseline is stale." % TAG)
    else:
        print("%s: %d adopted baseline(s) STALE. Each is resolved by ONE green gate run, which "
              "re-stamps it automatically:" % (TAG, len(rows)))
        for entry, resolver, cost in rows:
            print("    %-58s make -C rust SHELL=/bin/bash %s%s"
                  % (entry, resolver or "<no resolved_by>",
                     "   (~%ds)" % cost if cost else ""))
elif argv and argv[0] == "--resolve-stale":
    rows = stale_rows()
    del failures[:]
    if not rows:
        print("%s: no adopted baseline is stale; nothing to run." % TAG)
    else:
        print("%s: resolving %d stale baseline(s), cheapest first." % (TAG, len(rows)))
        for entry, resolver, cost in rows:
            if not resolver:
                print("    SKIP %s — no resolved_by" % entry, file=sys.stderr)
                rc = 1
                continue
            print("    ==> %s   (%s)" % (resolver, entry))
            # ⛔ The gate is what re-stamps, not this driver. A sweep that stamped on its own
            # behalf would be asserting a confirmation it never measured.
            if subprocess.run(["make", "-C", os.path.join(ROOT, "rust"), "SHELL=/bin/bash",
                               resolver]).returncode != 0:
                print("    ⛔ %s did NOT go green — %s stays stale, and that is the one cell that "
                      "needs a person." % (resolver, entry), file=sys.stderr)
                rc = 1
elif argv and argv[0] == "--report":
    register_mode(report=True)
elif not argv:
    register_mode(report=False)
else:
    print("%s: unknown argument: %s" % (TAG, argv[0]), file=sys.stderr)
    sys.exit(2)

if notes:
    # ⛔ A NOTE THAT IS NOT PRINTED IS A SILENT PASS, which is exactly the shape this doctrine
    # exists to remove. Downgrading a failure to a note only stays honest while the note is loud.
    print("%s: %d baseline(s) STALE but within budget — reported, not failed:" % (TAG, len(notes)),
          file=sys.stderr)
    for m in notes:
        print("    %s" % m, file=sys.stderr)
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
# ⛔ `--verify`'s STATUS WINS OVER THE FAILURE LIST, and getting this wrong silently broke the whole
# matrix: `verify()` records its reason via fail(), so an UNCONFIRMED baseline (rc 3) was reported
# as 1 = STALE, and a gate reading that started a two-minute measurement it was meant to skip.
# A four-way contract is only a contract if the exit code carries it.
if rc:
    sys.exit(rc)
if failures:
    sys.exit(1)
sys.exit(0)
PYEOF
