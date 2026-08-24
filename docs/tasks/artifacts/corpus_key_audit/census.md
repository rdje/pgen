# Answer-key contradiction census — CORPUS-KEY-AUDIT.1

> DERIVED. Re-run: `python3 docs/tasks/artifacts/corpus_key_audit/key_contradiction_census.py`
>
> ⛔ **A CANDIDATE WORKLIST, NOT A DEFECT COUNT.** A row carries ONE verdict and its
> golden may carry SEVERAL messages, so attributing every message to the row's verdict
> manufactures a disagreement whenever a file was pinned for a reason unrelated to most
> of what its golden says. Each row below is adjudicated BY HAND until `.1` lands the
> deciding-message attribution.

- keyed rows scanned: **499** (sv_2017 41, verilog_2005 458)
- distinct normalized upstream messages: **168**
- message classes carrying CONTRADICTORY expectations: **2**

| upstream message (normalized) | `must_accept` rows | `must_reject` rows |
|---|---|---|
| `Unresolved wire X cannot have multiple drivers.` | **4** — `uwire_fail`, `uwire_fail2`, `uwire_fail3`, `uwire_fail4` | **1** — `br_gh1087b` |
| `X has already been declared in this scope.` | **1** — `br_gh1225b` | **3** — `pr1704726a`, `pr1704726c`, `pr1704726d` |

