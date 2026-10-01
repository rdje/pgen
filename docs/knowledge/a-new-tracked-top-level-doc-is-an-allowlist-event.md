---
id: a-new-tracked-top-level-doc-is-an-allowlist-event
title: A new tracked top-level doc is an allowlist event, and a moved public spelling is a needle event — update the CI mirror's hand-kept rosters in the commit that moves the tree
answers:
  - "why did ci_workflow_local_gate fail with top-level docs allowlist drift detected"
  - "what to do when the local CI mirror reports a docs allowlist drift"
  - "where is the roster of allowed docs/*.md files"
  - "why did the push lane go red before the push"
  - "why did ci_workflow_local_gate fail with file content drift detected in rust/src/embedding_api.rs"
  - "what to do when the CI mirror's embedding API surface audit pins a constant that moved"
tags: [ci, ci_workflow_local_gate, allowlist, docs, push-lane, ops]
date: 2026-09-30
status: current
evidence: "OPS-MEMSAFE.5 correction (PGEN-OPS-MEMSAFE-0010): docs/ARTIFACT_CLEANUP.md, created by 707d276a2 on 2026-09-29, tripped `auditing top-level docs allowlist` on the first push-lane run (rust/target/ci_workflow_local_gate/run.zk13Tg); after the admission the audit's own comparison replays TOP-LEVEL-DOCS-ALLOWLIST: OK (14 docs, expected == actual, verbatim order)"
reverify: "python3 - <<'PY'\nimport re,subprocess\ns=open('rust/scripts/ci_workflow_local_gate.sh').read();i=s.index('expected_top_level_docs=(');j=s.index('\\n  )\\n',i)\nexp=re.findall(r'\"(docs/[^\"]+)\"',s[i:j]);act=[a for a in subprocess.run(\"git ls-files -z | perl -0ne 'for (split /\\\\0/) { print \\\"$_\\\\n\\\" if /\\\\Adocs\\\\/[^\\\\/]+\\\\.md\\\\z/ }' | sort\",shell=True,capture_output=True,text=True).stdout.split('\\n') if a]\nprint('OK' if exp==act else 'DRIFT', len(exp), len(act))\nPY"
---

`rust/scripts/ci_workflow_local_gate.sh` — the full CI mirror the 400-commit push lane runs before
every push — audits the tracked `docs/*.md` set VERBATIM against a hand-kept roster
(`audit_top_level_docs_surface`, `expected_top_level_docs`), and two sibling audits do the same for
the root markdown set and `docs/contracts/`. A tracked top-level doc the roster does not name is a
**drift**, whatever created it: `error: top-level docs allowlist drift detected; prune stale docs
or update the tracked policy deliberately`.

- **Creating a tracked `docs/*.md` is therefore an allowlist event.** Admit it in the SAME
  commit, in the audit's own `sort` order (order is compared verbatim — this locale sorts
  `TASK_TREE_README.md` before `TASK_TREE.md`), with the admission's evidence in the roster
  comment: the file's history (`git log -- <path>`) and what references it. The roster's
  comments carry three such admissions (`CI-PARITY-GATE-ROT.1`, `.46`, `OPS-MEMSAFE.5`'s
  correction); each time the audit was right that no deliberate update had happened.
- **Nothing between pushes runs these audits.** The ordinary-commit tier is the doctrine
  enforcer only; the surface audits live in the mirror, which fires at the 400-commit cadence
  or on an operator's `make -C rust SHELL=/bin/bash ci_workflow_local_gate`. So a drift is
  invisible for up to 400 commits and surfaces exactly when the push is due — read the
  mirror's log for the first refusing audit (it stops at the first), not just its exit code.
- **The same holds for every hand-kept needle roster in the mirror.** `audit_embedding_api_surface` pins ~140 spellings across the API, its contract doc and the user guide with `assert_file_contains` / `assert_file_matches`; `ENGINE-UNIVERSAL-SERVICES.112` moved fifteen of them (`CI-PARITY-GATE-ROT.67`) and the mirror refused at the first. Size the population first — replay every needle with the helpers' own `grep -F` / `grep -E` — then re-point each invariant to where the tree carries it; never delete the invariant.
- **Replay the audit's comparison before re-running the mirror** — the reverify command above
  extracts the roster verbatim and compares it to the same `git ls-files … | sort` the audit
  uses; a green replay costs a second, the mirror costs hours (it replays `sota-exit-gate`).

Related: [[your-build-tools-timestamp-resolution-is-part-of-your-correctness-argument]] (the
mirror's other silent class), the push-pacing rule in `docs/decisions/feedback_push_pacing.md`.
