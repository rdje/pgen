"""Arm 12b's perturbation: mis-register the UNCONFIRMED union contract as plain `adopted`.

⛔ It lives in its own file rather than in a heredoc inside probe.sh, and that is not style:
an inner `<<'PY'` heredoc inside a generator that itself uses `<<'PY'` silently terminates the
OUTER heredoc, and the shell then executes the remaining lines as commands. That happened while
this probe was being written and it ran `cp -f "" ""` against the tracked register.
"""
import json
import sys

path = sys.argv[1]
with open(path, encoding="utf-8") as fh:
    reg = json.load(fh)
for row in reg["entries"]:
    if row["entry"] == "systemverilog_recognized_cert_union_contract.json":
        row["disposition"] = "adopted"
        break
else:
    raise SystemExit("promote_row.py: the union contract has no register row")
with open(path, "w", encoding="utf-8") as fh:
    json.dump(reg, fh, indent=2, ensure_ascii=False)
    fh.write("\n")
