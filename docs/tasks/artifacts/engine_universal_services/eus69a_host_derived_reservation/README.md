# `ENGINE-UNIVERSAL-SERVICES.69`(a) SLICE 2 — the host-derived reservation default

The last literal `.65` left standing was `DEDICATED_PARSE_STACK_BYTES = 256 MiB` — the number the
recursion ceiling is derived FROM, so while it was compiled in the ceiling could not be a function
of the host. This directory holds the producers and the recorded runs for making it one.

## The law

```text
reservation = min( 256 MiB , max( 96 MiB , physical_memory / 64 ) )
```

⭐ **The share is chosen so `256 MiB × 64 = 16 GiB`** — the law returns the historic literal
EXACTLY on a 16 GiB host, the developer-machine class it was typed for. It *explains* the constant
rather than replacing it, which is also the strongest available no-regression argument for a change
to a default: every host at or above the design point is provably accept-set-neutral.

⛔ **The share is a POLICY, not a proof.** Deriving it needs the total process footprint at full
stack spend, and `.84` records that quantity as NOT ATTRIBUTED. A bound with an unmeasured term is
not a bound, so none is claimed. `.84` is the leaf that would let it be derived.

## Files

| file | what it is |
|---|---|
| `preregistration.md` | written BEFORE any deciding measurement and before any source edit. Names four frames and seven falsifiable predictions. **P1–P7 all confirmed.** |
| `derivation_check.sh` | producer. Five arms: the resolved reservation + provenance from the shipped binary; the same host quantity from `sysctl`/`/proc/meminfo` (a SECOND instrument); and the three override arms. |
| `derivation_check_debug.txt` | its recorded run. |
| `red_controls.txt` | the mutation battery: one targeted mutation per new lock, each required to make that lock FAIL. **6 of 7 RED.** |
| `generated_digests_after_regeneration.txt` | the zero-generated-bytes proof — 11/11 byte-identical across a full regeneration, and why the gate could not have supplied it. |

## Why every number here is load-independent

The host carried an unrelated `rustc` at ~27 % CPU and a load average of 4.28 throughout this
slice. **Nothing measured here is a wall clock or a peak RSS** — every column is an integer the
process holds at startup, or a `sysctl` reading. Layer A's rule (*never measure peak RSS on a busy
machine*) is why the two cost figures this slice needed — the `.78` ladder's crossing rung and its
runtime — were left as ARITHMETIC over already-measured per-frame costs and routed to `.87`(a) to
be timed on an idle host, rather than estimated here and published as measurements.

## ⛔⛔⛔ The finding that matters most is about the CONTROLS — and my first diagnosis of it was WRONG

The battery required each new lock to fail under a targeted mutation. Six did. The seventh — for
**the most important lock in the slice**, *the first-sample anchor must not track the derived
default*, whose failure would restore `.79`'s measured `rc 134` — came back **GREEN**.

**What I published (`-0157`)**: that the control *"cannot fail on this host"* because the cap makes
the two candidate anchors numerically identical at ≥ 16 GiB, that the defect is *"only observable
in the 6–16 GiB band"*, and — the sentence I led with to the director — *"a control that cannot
fail on your host passes for that reason."*

**What is true (`-0158`, measured under a director challenge, in one command):**

```
# wrapper mutated to anchor on dedicated_parse_stack_bytes()
PGEN_PARSE_STACK_BYTES=536870912 cargo test --lib the_first_sample_reference_does_not_track
  -> FAILED   left: 768   right: 1536
PGEN_PARSE_STACK_BYTES=134217728 ...
  -> FAILED   left: 3072  right: 1536
# unmutated, SAME env, both values -> ok  (so the env alone is not what fails it)
```

⇒ **the lock was always adequate. My HARNESS ran exactly one process configuration.** And in that
configuration the mutated program is *semantically identical* to the correct one — on this host the
derived default IS the reference — so **there was no defect present to detect, and the green was
correct rather than blind.**

⭐ The real lesson, which is not the one I first published: **a mutation is only a control where
the defect it names can exist**, so a green mutation is a question about the CONFIGURATION before
it is a verdict about the lock. And: *when a control comes back green, falsify the EXPLANATION, not
just the control* — I spent a paragraph diagnosing and none checking, and the check cost one
command.

**Both fixes stand.** The LAW half was a genuine weakness: the anchor is now a parameter
(`scale_sample_depth_against_reference`), the same split `scale_sample_depth_to_reservation` had
already been given for the reservation and for the same recorded reason. The WRAPPER half is now
pinned by a THRESHOLD BISECTION — identity exactly AT the anchor, scaling one byte below — which
pins the anchor's VALUE rather than one of its answers. The battery runs M4b in three
configurations plus two green controls. `.87`(e), which was opened on the false premise, is
withdrawn with it.
