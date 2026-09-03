---
id: a-commit-that-quotes-a-bank-must-be-gated-on-the-banks-exit-code
title: A commit that quotes a bank or gate count must be made in the same command as the run and chained on its exit code AND its summary line — never from the number you intended
answers:
  - "how do I avoid committing a bank count that the bank did not actually print"
  - "the bank printed six PASS lines and then crashed — what did I commit"
  - "should a commit be conditional on the bank's exit code"
  - "why did the leaf say 7/7 when the probe script died on an unbound variable"
tags: [banks, commits, evidence, claim-verification, discipline]
date: 2026-09-03
status: current
evidence: "PGEN-EBNF-FRONTEND-SILENT-TRUNCATION-0007 and -0008: the non_utf8_grammar bank printed 6 PASS lines and died on `m: unbound variable` (an edit had folded the assignment into a trailing comment); the leaf and both commit messages said 7/7. The repair in -0008 did not even apply (its anchor assertion failed) and was committed anyway. -0009 repaired the line inside a shell command whose commit step was chained on `rc 0 && grep -q '7/7 as declared'`."
reverify: "bash docs/tasks/artifacts/ebnf_frontend_silent_truncation/non_utf8_grammar/probe.sh   # the last line is the number; nothing else is"
---

**The number you commit is the instrument's last line, read after the run — not the number the plan said.**

## The rules

1. **Same command.** Run the bank and commit in one shell invocation; the commit step is `&&`-chained on the bank's
   exit code and on a grep of its summary line (`N/N as declared`). A bank that crashes under `set -u` exits non-zero
   and prints no summary — the chain refuses.
2. **Read the last line, not the PASS lines.** Six PASS lines followed by a traceback is a failed bank. Scroll to
   the end before writing any number down.
3. **A repair is not done when the script says "patched"; it is done when the instrument prints the new number.**
   An `assert` in the patch script that fails is a patch that did not happen.
4. **Twice is a pattern.** After the first false claim, the second commit made the same mistake with a different
   script. The rule is mechanical for a reason: intent does not protect against it.

## Related
[[a-red-arm-that-returns-rc-1-from-a-crash-is-not-a-red-arm]] — the crash side of the same lesson.
[[feedback-stand-by-published-claims-without-re-audit]] — verify BEFORE publishing; this is how.
