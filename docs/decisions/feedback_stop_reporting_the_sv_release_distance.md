# Stop reporting the SV release distance — a wrong distance is worse than reporting nothing (director, 2026-10-10)

**Status:** standing directive, verbatim. Supersedes the 2026-09-17 request for *one number per turn* (`SV-CORPUS-GRAD.25`,
the `items=…` distance in every banner).

## Context

On 2026-10-09/10 the published SV release distance read 10 → 12 → 18 → 20 → 21 → 20 within two days while the parser
got no further from `Done`: `.49` and `.51` made the open-leaf census read the status word at every heading level (the
previous form had counted a decoration and missed nine open leaves), and `.52`–`.54` opened and closed instrument leaves
within hours. `items = legs_unmet + bar_rows + open_leaves` added a done-bar leg, a corpus row and a tracker leaf as if they
were one unit; seven of the twenty were engine leaves parked behind the SV lane lock; the trailing-rate ETA printed a
negative rate and `n/a`. The director, reviewing the book, read the jump as the project moving backwards.

## Decision (the director's words, 2026-10-10)

> I noticed that the SV release distance went from 10 -> 12 -> 20, what is happening. It looks like "items" does not seem
> to be the proper metric to measure the distance 'til completion.
>
> Look, if you can't provide a ETA number, then stop producing the distance, because it is just misleading. stop reporting it.
>
> A wrong distance is worst than reporting nothing.

## Consequences

- The distance, `items=…` and any trailing-rate ETA leave every banner, message, commit message and book page — from the
  commit that carries this record, not after it (`SV-CORPUS-GRAD.55`, landed with release `1.0.253`): the SV parser book's
  distance section and its derived block, `scripts/report_sv_release_distance.sh`, `scripts/check_sv_release_distance.sh`,
  the per-release history (`release_distance/history.tsv`), the ceremony's record / splice / check steps and the
  `SV-RELEASE-DISTANCE-CURRENCY` doctrine (46 → 45 registered) are removed. Git keeps the history.
- The done-bar legs (`Distance to Done: N of the 3 legs`, derived by `report_family_status.sh`) are not this number and
  stay: each leg is a defined, measured condition, not a count.
- If a distance is ever wanted again it is a measured remainder per done-bar leg (today only leg 3 has one: the rows still
  awaiting a verdict under the declared reading) plus the known conformance defects — never a count of leaves, which moves
  with filing granularity, and never with an ETA extrapolated from a trailing rate.
