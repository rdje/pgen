# Answer-key contradiction census — CORPUS-KEY-AUDIT.1

> DERIVED. Re-run: `python3 docs/tasks/artifacts/corpus_key_audit/key_contradiction_census.py`
>
> ⛔ **STILL A WORKLIST, NOT A DEFECT COUNT.** `.1`(a) attributes each row to the
> evidence its own basis records — a CLAUSE cite keys nothing from the golden, a quoted
> `PARSE-stage refusal ('…')` keys only the message it quotes — which removes the
> founding over-report at its cause. What survives is a candidate: only adjudication
> against the LRM decides, and the census never asks the parser under test.

- keyed rows scanned: **499** (sv_2017 41, verilog_2005 458)
- distinct normalized upstream messages: **176**
- contradictory classes, NAIVE attribution (every message ← the row's verdict): **2**
- contradictory classes, DECIDING-EVIDENCE attribution: **0**
- ⇒ false positives removed: **2** of 2 (**100%** of the founding report)

## Attribution provenance — where each row's key actually came from

| expected | provenance | rows |
|---|---|---|
| `must_accept` | `whole-golden:accept-claim` | 479 |
| `must_reject` | `clause-cited` | 9 |
| `must_reject` | `quoted-decider` | 11 |

`clause-cited` rows contribute NO message class: their key rests on the clause, which the north star ranks above tool testimony (`pr1704726a`'s basis says so in its own words). `whole-golden:accept-claim` is exact rather than a fallback — a `must_accept` key asserts that EVERY message present is post-parse. `whole-golden:UNDETERMINED` is the conservative fallback for a `must_reject` row whose basis records no quotable decider; it keeps the row in the worklist.

## ⛔ Honest power bound — how much could this census have found?

A contradiction needs a class on BOTH sides, so the census can only ever reach the
intersection. After attribution the reject side is **4 classes** wide against **124** on the accept side — quoting `0` without that ratio would read as a far stronger clean bill than it is.

| reject-side deciding class | `must_reject` rows | `must_accept` rows carrying it |
|---|---|---|
| `Missing task/function port direction.` | 3 | 0 |
| `X X  is not a valid expression. Please use operator X instead.` | 1 | 0 |
| `generate/endgenerate regions cannot nest.` | 1 | 0 |
| `syntax error` | 6 | 0 |

⭐ The sharpest row is `syntax error`: it is the upstream's own bare parse refusal, so a `must_accept` row whose golden carried one would be a near-certain key defect — and the founding extractor could not see the class at all, which made the question unaskable rather than answered. It is now asked, over the whole accept population.

## Key integrity — does a basis's quoted evidence exist in the golden it names?

Every quoted decider resolves against its own golden.

## Contradictory classes under the deciding-evidence attribution

No message class carries contradictory expectations.

## For comparison — the classes the NAIVE attribution reported

Kept so the correction is auditable rather than asserted.

| upstream message (normalized) | `must_accept` rows | `must_reject` rows |
|---|---|---|
| `Unresolved wire X cannot have multiple drivers.` | **4** — `uwire_fail`, `uwire_fail2`, `uwire_fail3`, `uwire_fail4` | **1** — `br_gh1087b` |
| `X has already been declared in this scope.` | **1** — `br_gh1225b` | **3** — `pr1704726a`, `pr1704726c`, `pr1704726d` |

