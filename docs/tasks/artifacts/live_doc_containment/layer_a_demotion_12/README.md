# `LIVE-DOC-CONTAINMENT.12` — layer-A demotion (PGEN-LIVE-DOC-CONTAINMENT-0019)

Layer A (`MEMORY.md`) reached **98 % of its 32 768-byte cap** (32 103 B), leaving 665 bytes —
not enough for the next slice to record its result. The doctrine's remedy is explicit: **a cap is
never raised to land content; the content is demoted.**

## What was demoted, and why those two

Two FIELD bullets held **8 895 B — 28 % of layer A** — while the fields themselves are one word:

| bullet | before | after | what it actually held |
|---|---|---|---|
| `in_flight_uncommitted` | 4 528 B | 626 B | closed-slice batteries, a closed leaf's full derivation, two routed-leaf write-ups |
| `blockers` | 4 367 B | 588 B | a 2026-08-25 retraction, five routed leaves, four more routed findings |

Result: **32 103 B → 24 833 B (98 % → 75 %)**, 7 270 B freed, no cap touched.

Both were carrying a **copy of the routed/parked queue**, which this file's own header already
declares DERIVED (`docs/TASK_TREE.md` is authoritative). That copy is precisely the drift
`docs/DERIVED_STATE_CONTAINMENT.md` R1/R3 exists to stop.

## Proving it lossless

`verify_demotion.sh` re-derives, fact by fact, that **all 20** facts removed from layer A still
exist in a durable layer, searching layers B/C/D with layer A itself excluded by construction —
"still in the file you are demoting from" is not durability.

⛔ **Two probes were FALSE NEGATIVES when the list was built**, both because the token assumed a
spelling layer B does not use: `484 cells` (layer B writes `484 (input, column) cells`) and
`SV-CORPUS-GRAD.13c.2x.8` (a tree refers to its own leaves BARE, as `.13c.2x.8` — the undercount
`DOCTRINE-GAP-OWNERSHIP.8` already records). Both failed in the SAFE direction: they would have
blocked a sound demotion, never hidden a loss.

⛔⛔ **The RED control caught a real defect in the checker on its first run.** The script lives
under `docs/` and QUOTES every token it searches for, so it was matching ITSELF — every probe
passed vacuously, and several rows named the verifier as the fact's durable home. The control
reported FOUND where it must report LOST, which is the only reason this was seen. The script now
excludes its own path. **A census that cannot fail proves nothing.**

## Re-verify

```bash
bash docs/tasks/artifacts/live_doc_containment/layer_a_demotion_12/verify_demotion.sh
# -> 20 ok, RED control LOST, exit 0
bash scripts/check_memory_architecture.sh
# -> layer A 24833/32768 bytes = 75 % of cap
```
