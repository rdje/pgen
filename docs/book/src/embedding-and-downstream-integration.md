# Embedding and Downstream Integration

PGEN is built to be embedded into other projects, not only run as a standalone repo-local tool.

## Main Downstream Shapes

Downstream consumers interact with PGEN through a few stable kinds of surfaces:

- generated parser artifacts,
- parser registry integration,
- embedding API functions,
- parser-family integration contracts,
- issue reporting and release-support workflow.

## Embedding API

The embedding API exists to provide a versioned surface for selected parser families and annotation parsing without forcing downstream projects to understand the entire repository internals.

This matters especially for:

- parser availability,
- profile names,
- diagnostics shape,
- AST dump availability,
- release-version metadata.

### Current API surface

The embedding API version is **`1.3.2`**. The supported grammar families and their selectable
profile strings are:

- **SystemVerilog** — `sv_2017`, `sv_2023`, and the strict **`verilog_2005`** (IEEE 1364-2005 Verilog
  subset) profile;
- **VHDL** — `vhdl_1076_2019`;
- **regex** — `regex_default` (the regex grammar also has a CLI-only `relaxed` opt-out profile, which
  the embedding API deliberately does not expose).

The authoritative, versioned list — accepted aliases (e.g. `2017` / `ieee1800-2017`, `1364-2005` /
`ieee1364-2005`), the full profile matrix, the diagnostics shape, and the per-family release/schema
metadata — is `rust/docs/EMBEDDING_API_CONTRACT.md`. Treat that contract (not this prose) as the
source of truth for exact version and profile values.

Since `PROFILE-ALIAS.2`, the SystemVerilog alias spellings are **grammar-declared** (`@profile_alias`
in `grammars/systemverilog.ebnf`) and burned into the generated parser (`GRAMMAR_PROFILE_ALIASES` +
case-insensitive resolution inside `set_grammar_profile`), so the artifact itself accepts them at
every entry point. The embedding API's typed `GrammarProfile` enum keeps its own spellings as part
of the typed contract, and a drift-gate test asserts they resolve every grammar-declared alias to
the declared canonical profile — the two surfaces cannot silently diverge.

### Reading source files — `pgen::source_text`

The embedding API parses a `&str`; **turning a file into that `&str` is the host's job**, and it is
a job with a trap in it. `std::fs::read_to_string` refuses any byte stream that is not valid UTF-8,
so a host that uses it rejects the *file* rather than rejecting a *construct* — and vendor RTL
routinely carries a `©`, an accented author name or a `µ` in a header comment, which IEEE
1800-2017 §5.4 explicitly permits (it constrains only where a comment starts and ends, never its
content).

Since `SV-CORPUS-GRAD.12c.1`, PGEN ships the reader it uses itself:

```rust
let decoded = pgen::source_text::read_source_file("design/top.sv")?;
let ast = /* hand `&decoded.text` to the embedding API */;

if !decoded.encoding.preserves_disk_byte_offsets() {
    // Positions PGEN reports are offsets into `decoded.text`, not into the file.
    eprintln!("{}", decoded.non_utf8_notice("design/top.sv").unwrap());
}
```

- **The ladder** is BOM sniff → UTF-8 / UTF-16LE / UTF-16BE → **ISO-8859-1 fallback**. Latin-1 is a
  *total* decoding — all 256 byte values map and round-trip exactly — so the fallback cannot fail.
- **The encoding is reported, never silently applied.** `DecodedSource::encoding` names it, and
  `preserves_disk_byte_offsets()` answers the question a host actually needs: *is a reported byte
  offset still an offset into my file?* (Only plain UTF-8 is; transcoding shifts positions.)
- **A file that declares an encoding through a BOM and then contradicts it is refused**, with the
  offending byte offset named, rather than re-read under a guess.
- ⛔ **Only for user source.** Do not route JSON manifests, generated Rust or reports through it —
  those are UTF-8 by construction, and a decode failure there is corruption that must stay loud.

### Stack robustness at the embedding boundary (`1.3.1`, `1.3.2`, `1.3.4`, `1.3.5`)

A host process embedding a recursive-descent parser must never be aborted by that parser's
recursion. The generated parsers carry a recursion ceiling, but a ceiling only protects the host
when the calling thread has enough real stack for the frames it admits — measured for
SystemVerilog a frame costs ≈2 KB in release and ≈17 KB in debug, so the 4096 frames the ceiling
admitted when this was written needed ≈8 MB / ≈70 MB, more than the 8 MB default main stack (a
~400-deep parenthesized expression, ≈4 KB of text, was enough to SIGABRT a release embedder).

> ⛔⛔ **DO NOT SIZE A STACK FROM THE ≈17 KB ABOVE — IT IS A 2026-07 READING AND THE COST HAS
> MOVED.** That figure is `SV-CORPUS-GRAD.8c.3`'s and is kept because it is what the `1.3.1`
> argument was built on. Re-measured 2026-08-30 with the depth probe's own instrument
> (`ENGINE-UNIVERSAL-SERVICES.69`(d)), a debug SystemVerilog frame costs **25 335 B** (`sv_2017`,
> at a real refusal) to **25 874 B** — about **1.5×** the number above — and a `verilog_2005` frame
> costs **29 031 B**. Release is **3 071 – 3 083 B**. ⇒ 4 096 frames costs **98.96 MiB**
> (`sv_2017` @1 024 levels), **101.07 MiB** (`sv_2017` @800) or **113.40 MiB** (`verilog_2005`) in
> debug — not ≈70 MB, and **every one of them is more than the 96 MiB floor**
> `PGEN_PARSE_STACK_BYTES` clamps up to (and 9.9–26 % more than that floor's 90 MiB spendable
> budget); a debug embedder that trusted the floor GOT the abort this section promises it will not
> — until the fix immediately below.
> ⚠️ Quote the RANGE, not one end of it: an earlier revision of this note published only the
> ≈101 MiB figure, which is the largest of the three `sv_2017` readings.
>
> ✅ **FIXED 2026-08-30 (`ENGINE-UNIVERSAL-SERVICES.79`) — AND NOT BY RE-DERIVING THE FLOOR.**
> Re-computing 96 MiB from today's per-frame cost would be the same defect wearing a fresher
> literal, and a census can only ever report the worst cost it happened to RUN. The depth at which
> the guard takes its FIRST sample is now DERIVED from the reservation instead: it is the depth the
> generated parser asks for, scaled by how much of the 256 MiB default this process actually has
> (`dedicated_parse_stack::first_stack_sample_depth`). Per-frame cost does not depend on the
> reservation, so the depth the stack can carry and the depth of the first sample are both LINEAR in
> it and their RATIO — the safety margin — is INVARIANT. ⇒ **a first sample that is reachable at the
> default is reachable at every smaller reservation**, for every family and both build modes, with
> no per-frame constant in the derivation.
>
> Measured before → after with the pre-change binary preserved, debug probe, `PGEN_PARSE_STACK_BYTES`
> at the 96 MiB floor: **7 of 9 rungs went from `rc 134` SIGABRT to a clean `rc 1` refusal**, and
> the refusal names its cause and depth (`BudgetExhausted`, 3 647 frames on the 300-paren input —
> which is `budget ÷ 25 874 B/frame` to within one frame, i.e. the ceiling really is the arithmetic
> it claims to be). The 275-paren rung still ACCEPTS, so no capability was traded for the fix. At the
> **default reservation the scale factor is exactly 1**, so the shipped path samples at precisely the
> depth it did before.
>
> ⛔ **AND IT APPLIES ONLY TO THREADS PGEN CREATES FOR PARSING — a same-day correction
> (`ENGINE-UNIVERSAL-SERVICES.79`, `-0144`).** The first cut scaled by the process-global
> reservation, so `PGEN_PARSE_STACK_BYTES` also lowered the hard ceiling of the **regex** family,
> which parses on its own long-lived 64 MiB worker that the reservation does not describe — measured,
> 400 nested groups went from ACCEPT to a refusal at the scaled depth. The law now reads the parse
> thread's OWN recorded extent: a thread PGEN did not create keeps the depth its parser asked for,
> and a dedicated parse thread is scaled against the reservation it was really created with.
>
> ✅ **AND THE MARGIN IS NOW MEASURED, NOT JUST INVARIANT** (`ENGINE-UNIVERSAL-SERVICES.79`(a)
> slice 1, `-0145`). This paragraph used to quote **2.37×** for SystemVerilog and say the other
> families were unmeasured; the census superseded both halves. Debug per-frame cost, 9 of 11 cells:
>
> | family / profile | B/frame | margin at the 256 MiB default |
> |---|---|---|
> | `systemverilog` / `verilog_2005` | 29 046 | **2.12×** |
> | `systemverilog` / `sv_2023` | 24 903 | 2.47× |
> | `systemverilog` / `sv_2017` | 24 559 | 2.50× |
> | `systemverilog_preprocessor` | 5 236 | 11.73× |
> | `return_annotation` | 4 998 | 12.29× |
> | `json` | 2 882 | 21.32× |
> | `semantic_annotation` | 2 714 | 22.64× |
> | `rtl_frontend` | 1 971 | 31.17× |
> | `rtl_const_expr` | 1 961 | 31.33× |
>
> ⇒ the per-frame cost spans **≈15×** across families, so a single byte floor could never have served
> them all; **the tightest margin is 2.12×**; and **SystemVerilog is the only family under 11×**, by a
> factor of ~5.5. The earlier worry that a family might cost more than 60 KiB (61 440 B, exactly
> `budget ÷ 4 096`) per debug frame is answered — the worst measured is 29 046 B, under half that.
> ⚠️ Read these at **±5 %**: four independent `sv_2017` readings span 24 559–25 878 depending on the
> depth the refusal lands at, so the spread is ≈13–16× and the margins move in the last digit.
>
> ✅ **AND THE RELEASE HALF IS NOW MEASURED TOO** (`ENGINE-UNIVERSAL-SERVICES.79`(a) slice 2b,
> `-0151`), at the same 96 MiB floor so the two halves are directly comparable:
>
> | family / profile | release B/frame | release margin | debug B/frame | debug → release |
> |---|---|---|---|---|
> | `systemverilog` / `verilog_2005` | 3 709 | **16.57×** | 29 046 | 7.83× |
> | `systemverilog` / `sv_2023` | 3 051 | 20.14× | 24 903 | 8.16× |
> | `systemverilog` / `sv_2017` | 3 043 | 20.19× | 24 559 | 8.07× |
> | `systemverilog_preprocessor` | 507 | 121.18× | 5 236 | 10.33× |
>
> ⇒ three things a downstream embedder should take from this. **(1) The DEBUG build is the binding
> case, by 7.83×** — 2.12× against 16.57× at the binding cell — so a debug host is the one to size
> for. **(2) SystemVerilog is the binding family in BOTH build modes**, by a 5.55× separation in
> debug and 7.32× in release. **(3) The build-mode ratio is NOT a single constant, so a release
> figure cannot be derived from a debug one**: the three SystemVerilog profiles agree to 1.042×
> (7.83–8.16×) while `systemverilog_preprocessor` reads 10.33×, 27 % away.
>
> ⭐ The release `sv_2017` reading of **3 043 B/frame** lands **0.9–1.3 %** from the **3 071–3 083**
> that `ENGINE-UNIVERSAL-SERVICES.69`(d) measured independently — a different harness, a different
> nesting SHAPE and a different reservation — which is corroboration nobody engineered for.
>
> ⚠️ **Nine of the twenty-two cells are not measured, and each one names the reason.** `vhdl` in both
> modes and the five cheap-frame families in release are bounded by cost, not by choice: a cell is
> reached at `budget ÷ bytes_per_frame` frames and crossing costs Θ(depth²)
> (`ENGINE-UNIVERSAL-SERVICES.68`), while a release cell needs 8–10× that depth — one such run cost
> 1.17 GB of peak RSS on an 8 KB input (`ENGINE-UNIVERSAL-SERVICES.84`). `regex` is bounded by
> `ENGINE-UNIVERSAL-SERVICES.82`, whose refusal reading is stranded on that family's own worker.
> ⭐ Note the direction: the margin is `budget ÷ (4 096 × bytes_per_frame)`, so **cheap frames mean
> wide margins**, while the census's cost is quadratic in `budget ÷ bytes_per_frame`, so **cheap
> frames also mean expensive cells**. The measurement is cheapest exactly where the risk is highest.
>
> ⛔⛔⛔ **AND ONE MORE ABORT SURVIVED ALL OF THAT, ON THE SHIPPED ARTIFACT — FIXED 2026-08-31
> (`ENGINE-UNIVERSAL-SERVICES.85`).** `.79` made the guard's FIRST sample reachable at every
> reservation. It did not make what happens BETWEEN samples safe, and that is a different defect
> wearing the same symptom. The guard used to set its next sample point by PREDICTION: it priced a
> stride of `affordable / 4` frames from the RUNNING AVERAGE per-frame cost and let the parse descend
> that far unchecked. A parse can therefore spend its whole remaining budget inside ONE stride
> whenever the frames it really pushes cost four times the average that priced them — and nested
> assignment patterns do exactly that.
>
> **Measured, and it is not a near miss.** A **15 400-byte** syntactically valid SystemVerilog file —
> `module m; localparam p = '{'{'{ … 1 … }}}; endmodule` — **hard-aborted the RELEASE parser at the
> shipped 256 MiB default** (`rc 134`, reproduced 2 of 2). In a debug build the same shape aborted
> from **2 344 bytes** at the default and **904 bytes** at the 96 MiB floor. The margin degrades
> smoothly and then the process simply dies: 3.31× → 1.92× → 1.35× → 1.05× → **SIGABRT**.
>
> ✅ **THE FIX REPLACES THE PREDICTION WITH AN OBSERVATION.** The guard now learns
> `stack_floor = base − budget` once, on the parse thread, and tests the real stack pointer against
> it — the trigger is `depth >= max_depth || stack_floor_breached()`. Per-frame cost, average or
> marginal, leaves the safety argument entirely, and the overshoot is bounded by the frames between
> two guard-visible entries instead of by one mispriced stride. `max_depth` survives only as the
> fallback for a thread whose extent the process cannot know.
>
> **What an embedder gets, measured before → after:**
>
> | | before | after |
> |---|---|---|
> | the 15 400 B file, release, shipped default | **`rc 134` SIGABRT** | **`rc 0` accepted**, 5 115× margin |
> | 288–448 nesting levels, 96 MiB floor | **`rc 134` SIGABRT** | **`rc 0` accepted** |
> | 512+ nesting levels, 96 MiB floor | **`rc 134` SIGABRT** | **`rc 1` clean rejection** |
> | worst safety margin over 10 shapes | **1.10×** | **103.86×** |
> | worst overshoot past budget | 244.8 frames | **2.49 frames** |
>
> ⭐ **The accept set WIDENED rather than narrowed.** Inputs that used to kill the process now parse:
> the guard stops the parse AT the budget, so there is stack left to back off and still find an
> accepting derivation. A guard that fires on a real measurement can fire EARLIER than one that
> guesses, so that risk was checked explicitly — every input that accepted before still accepts.
> ⚠️ Honest costs: one shape's margin FELL (`concat` 716× → 107×) because the distribution compressed,
> every shape now paying the same bounded couple-of-frames residual; and a deeply nested input that
> previously aborted is now ≈1.8× slower, because **a crash is a fast way to not answer** and the
> parse must now actually finish. The binding parse-cost metric over the pinned 192-file corpus is
> unchanged.
Since `1.3.1` (`SV-CORPUS-GRAD.8c.3`), every SystemVerilog and VHDL embedding parse runs its
parser work on a dedicated **256 MiB-stack** thread (virtual reservation, lazily committed; one
short-lived thread per call, so concurrent host threads never serialize), which guarantees the
ceiling fires as a clean `E_PARSE_FAILURE` diagnostic before the OS guard page can kill the
process — in both build modes.

⭐ **Since `ENGINE-UNIVERSAL-SERVICES.65` that reservation is also what SETS the ceiling**, rather
than merely being large enough for it: the guard measures how much of this thread's stack the parse
has really spent and refuses only when the headroom is gone. `PGEN_PARSE_STACK_BYTES` (bytes,
floored at 96 MiB) therefore buys proportionally more nesting capacity on a host that can afford
it. See *[The Recursion Ceiling Is Now The Host's Stack](#the-recursion-ceiling-is-now-the-hosts-stack)*
below.

⭐⭐ **And since `ENGINE-UNIVERSAL-SERVICES.69`(a) the DEFAULT is derived from the host rather than
typed.** The reservation an embedder gets with no configuration at all is now

```text
reservation = min( 256 MiB , max( 96 MiB , physical_memory / 64 ) )
```

so a host that cannot afford 256 MiB per parse is no longer handed it. Ask any build what it
resolved, and where the number came from:

```bash
parseability_probe --stack-reservation
```

```text
reservation_bytes=268435456
reservation_mib=256
source=host_memory_derived
host_memory_bytes=25769803776
host_memory_mib=24576
unclamped_share_bytes=402653184
share_divisor=64
floor_bytes=100663296
reference_bytes=268435456
```

`source` distinguishes the three provenances a single integer used to hide: `operator_override`
(you set `PGEN_PARSE_STACK_BYTES`), `host_memory_derived`, and `reference_fallback` (the host's
physical memory could not be read, so the fixed 256 MiB reference stands — an unreadable `sysctl`
is never allowed to become an outage).

**What this changes for you, concretely:**

| host physical memory | resolved reservation | what decided it |
|---|---|---|
| 4 GiB | 96 MiB | the usability floor |
| 8 GiB | 128 MiB | the derivation |
| 12 GiB | 192 MiB | the derivation |
| **16 GiB** | **256 MiB** | **the derivation — and it equals the old literal exactly** |
| 24 GiB, 64 GiB, 1 TiB | 256 MiB | the reference cap |

⭐ The `1/64` share is chosen so that `256 MiB × 64 = 16 GiB`: the number that used to be
hard-coded is exactly what the law now produces on the developer-machine class it was written for.
The literal is *explained*, not replaced.

⛔⛔ **What that costs on a smaller host, said plainly.** The accept boundary is LINEAR in the
reservation (measured: 18.82 nesting levels per MiB, flat to 0.14 %), so a smaller reservation
means a proportionally shallower accept set — an 8 GiB host takes **≈2 410** nested casts where a
16 GiB host takes **4 813**. ⇒ **the same file can parse on one host and be refused on a smaller
one**, as a bounded rejection with the ceiling label, never a crash. That is the capability mandate
working as specified — the limit is the host's RAM — and if you need a host-independent accept set
you pin `PGEN_PARSE_STACK_BYTES` to one value across your fleet. ⚠️ Scale context: no file in the
16 336-file tracked corpus approaches the boundary (p99.9 bracket nesting is 8), so this bites
machine-emitted or adversarial input rather than ordinary RTL.

⚠️ **Two honest limits, stated rather than implied.**

1. **The share is a policy, not a proof.** Deriving it would need the total process footprint at
   full stack spend, and that quantity is not yet attributed (`ENGINE-UNIVERSAL-SERVICES.84`: on
   one grammar peak RSS is quadratic in nesting depth while the arena holds 0.03 % of it). A bound
   with an unmeasured term is not a bound, so no bound is claimed.
2. **Only the downward half of "the limit is the host's RAM" has landed.** A host *larger* than
   16 GiB is capped at the reference today rather than accepting deeper input, because raising the
   reservation moves the depth at which the ceiling ladder crosses and crossing cost is quadratic
   in depth — the test suite's runtime would become a function of the CI host's RAM.
   `ENGINE-UNIVERSAL-SERVICES.87` owns lifting the cap and the test-pinning it needs first.

⚠️ **The derivation is per-parse.** The embedding API spawns one dedicated thread per call, so `N`
concurrent maximally-deep parses can commit up to `N/64` of host memory. An embedder running high
parse concurrency should name a smaller `PGEN_PARSE_STACK_BYTES` rather than rely on the default. The regex family keeps its own earlier defense
(the RGX-0085 dedicated worker plus a PCRE2-parity nesting pre-check), unchanged. The same
256 MiB guarantee wraps the `parseability_probe` and `ast_pipeline` instrument drivers, so a
pathologically deep input is a reproducible clean rejection everywhere rather than a crash.

#### A ceiling is only a bound if it also bounds TIME

`1.3.1` made crossing the ceiling safe for the host's *stack*. It did not make it bounded in
*time*, and for a while it was not: crossing the ceiling made the parse run without returning at
all. Measured on the shipped SystemVerilog parser, a `module m; assign x = (…)1(…); endmodule`
input crossed the cliff between **315 nested parens (0.16 s, accepted)** and **320 (no result in
30 s)** — a factor of more than a thousand across five characters. VHDL showed the identical
threshold further out, in a parser whose bytes had not changed, so the defect was engine-level and
not a property of either grammar.

The mechanism was one line of scope. Every cycle-guard verdict names the *frame* that blocked, and
a rejection is kept out of the packrat cache only up to the ancestor that owns that frame — the
outcome depends on stack CONTENT, which the memo key `(rule, position)` does not carry. The
whole-stack depth ceiling names no frame, so it was recorded as frame `0`: a scope that is honest
and unbounded. Frame `0` is outside *every* rule, so a single ceiling trip suppressed failure
memoisation for the entire rest of the parse, and the packrat parse collapsed into plain
exponential backtracking over the expression cascade's alternatives.

The fix keeps both halves true at once, because both are real:

- **Content-scoped blocks** (`Infinite`, `LeftRecursive`) are unchanged. Their failures are still
  never cached, for the reason they never were — replaying one from a different stack refuses a
  parse the guard would have allowed, which was a measured wrong-rejection on real corpus files.
- **The depth ceiling** is routed to its own channel and its failures ARE cached — under a **depth
  stamp**. Re-entering the same rule at the same position from a *deeper* stack prunes everything
  the first attempt pruned and possibly more, so a failure at depth `D` is a failure at every depth
  `≥ D`. That condition the entry can carry, so it is carried and validated on every replay.

The result is the contract `1.3.1` intended: over-ceiling input comes back as a clean
`E_PARSE_FAILURE` in bounded time instead of never. ⚠️ **"Bounded" is not "fast", and an embedder
should size for the plateau rather than for the cliff edge.** Just past the ceiling the reject is
indeed sub-second (SystemVerilog at nesting depth 350/400: **0.31 / 0.77 s**), but the cost rises
to a plateau and then *saturates*: SystemVerilog costs **12.4 s** at depth 800 and **13.0 s** at
depth 2000 on ~1.6 KB and ~4 KB of input respectively, and VHDL sits at **~19 s** across the same
range (18.92 s at 700, 19.09 s at 2000 — **0.9 %** more time for **2.76×** more input, which is what
saturation looks like). Saturation is the proof
the search is bounded — past the ceiling the parse only ever descends 4096 frames however long the
input is — but a bounded 12 s is still a real cost for a host parsing untrusted or machine-generated
source. ⚠️ **Every figure in this paragraph was measured while the ceiling was a fixed 4096
frames.** `ENGINE-UNIVERSAL-SERVICES.65` moved the ceiling to the host's stack, so the plateau
moved with it — the same shape now refuses one level past its (much deeper) boundary in **37.7 s**
where it refused in 0.19 s, because the parse descends much further before the refusal. The
saturation *argument* is unchanged; the numbers it saturates at are not. ⛔ This is tracked as an open defect, not an accepted residual, in
`docs/tasks/ENGINE-UNIVERSAL-SERVICES.md` leaf `.44`, whose (d) ruling is explicit that
*bounded-and-terminating is the difference between a crash and a slowdown, not the difference
between a defect and a non-defect*. Where the time actually goes has been measured rather than
guessed: on SystemVerilog error construction is only ~24 % of it, the parse cascade re-descending is
~72 %, and the driving term is **call volume** — 18.9 M rule calls at depth 800, of which about a
quarter run on rules whose packrat memo is deliberately elided because inlining their one-node body
is cheaper than probing for it.

#### Contacting the ceiling does not mean being rejected by it

Everything above prices a *rejection*. `ENGINE-UNIVERSAL-SERVICES.69`(d) measured the case the
section did not cover, and it is the one an embedder is more likely to meet: **a parse can contact
the ceiling, have the guard refuse, back-track and still ACCEPT** — same verdict, same AST, at a
cost that is not close.

Measured on the shipped SystemVerilog parser (release, `--profile sv_2017`, 3 000 nested parens,
`rc=0` both times):

| reservation | verdict | wall clock | ceiling contacted? | depth at refusal |
|---|---|---|---|---|
| 256 MiB (default) | accept | 9.67 s | no | none |
| 96 MiB (the floor) | accept | 162.96 s | yes, `BudgetExhausted` | 31 568 frames |

**16.9× for one verdict.** The reservation is the only thing that changed. This matters to an
embedder because it inverts the usual reading of a green result: a parse that succeeded is not
evidence that the input was comfortably within capacity, and shrinking `PGEN_PARSE_STACK_BYTES` to
save address space can make *successful* parses dramatically more expensive without changing a
single verdict — so nothing in a pass/fail suite would notice.

⭐ **The ceiling is a property of the stack, not of the input**, and the same measurement shows it
directly: two different inputs of different depths (2 600 and 3 000 nested parens) refuse at the
**same frame depth, 31 568**, under the same reservation.

`PGEN_REPORT_STACK_PROBE=1` reports this in one line — the named cause, the stack bytes actually
spent, the reservation, the spendable budget and the measured per-frame cost. Before it, the only
evidence that any of the above had happened was the wall clock, which does not say why. The
five named causes, and the reason a zero byte field means NOT MEASURED rather than a measured
zero, are in *Diagnostic & Debug Toolbox* in this book and in `TOOLBOX.md` § 3.3d.

**The bar this is measured against, and it has two legs rather than one.** The first is a ratio
rather than a stopwatch: a rejection may cost no more than **2×** the same parser's accepted parse of
the same shape — because a parser must not spend more effort failing than succeeding, and because an
absolute second count is a claim about one laptop. The second leg exists because the first one, on
its own, turned out to be satisfiable by making success worse: **no accepted depth may cost more than
2× the accepted depth at 80 % of the parser's cliff.** Measured across five families, on plain runs,
with the accept cliff found by bisection and both sides timed as the best of five runs:

| family | cliff (deepest accepted) | accept cost | saturated reject | reject cost | leg 1 (ratio) | leg 2 (accept plateau) |
|---|---:|---:|---:|---:|---:|---:|
| json | 1 364 | 0.071 s | 4 000 | 0.071 s | **1.0×** ✅ | ✅ |
| regex | 511 | 0.020 s | 2 000 | 0.010 s | **0.5×** ✅ | ✅ |
| rtl_const_expr | 314 | 0.020 s | 1 000 | 0.019 s | **1.0×** ✅ | ✅ |
| systemverilog | 322 | 0.14–0.18 s | 800 | 12.3–12.4 s | **67–90×** ⛔ | **1.4×** ✅ |
| vhdl | 681 | 18.6 s | 800 | 18.7 s | **1.0×** ✅ | **102×** ⛔ |

⚠️ Read the two red cells before anything else, because they are red for **different reasons**.
SystemVerilog rejects far more expensively than it accepts. VHDL does not — it passes leg 1 at 1.0×
**because its accept path is just as expensive**: valid VHDL at nesting depth 681 parses
*successfully* in **18.6 s**, a 102× rise over depth 575, and that is tracked as its own defect
(`ENGINE-UNIVERSAL-SERVICES.60`). A single-leg bar scored that parser as passing, which is why the
second leg exists.

⚠️ Two more honest notes on this table. The SystemVerilog ratio is quoted as a **range** on purpose:
its accept anchor is a sub-second measurement whose spread (four anchored runs read 67.0, 70.4,
80.6 and 89.5) is the whole range, so nothing finer than *two orders of magnitude* is supported. And for the three fast families both sides are dominated
by fixed process cost, so their ≈1× is close to true by construction — what is load-bearing about
them is not the ratio but the **absence of any plateau at all**: tens of milliseconds against
SystemVerilog's twelve seconds.

**Why those three are cheap turned out to refute the obvious explanation, and that is worth stating
because it redirected the fix.** The natural theory was PGEN's *attempt-elision* machinery: a family
whose choice sites can dispatch on the next input byte never enters the branches that cannot match,
so its tournaments stay small. That machinery reports exactly zero applicable sites on
SystemVerilog, for one reason — SystemVerilog's terminals skip leading whitespace and comments, so a
raw byte peek at the current position is unsound. Making the dispatch *layout-aware* (skip layout
once, then peek) would unlock a large surface: measured on a copy of the grammar differing by one
line, the per-branch admission gate goes from **0 to 1 906 of 1 934** top-level branches, and at the
site that dominates a deep-nesting reject, **12 of its 15 alternatives are refutable on the first
byte**.

But `json` and `rtl_const_expr` have the *same* zero elision surface for the *same* layout reason,
and they show no plateau anyway. So the zero is a symptom, not the cause. Priced against the real
workload, a perfect layout-aware dispatch removes at most 80 % of the rule entries — worth roughly
**1.8–3.5×** — where the bar needs about **41×**. The term that can supply it is *repetition*: on a
1 633-byte input with at most 1 634 distinct memo keys, the dominant expression-spine rule is
entered **601 658 times with zero memo hits**, roughly 368 re-derivations per position, while the
leaves of the same tournament are served from the memo ~100 % of the time.

**That second term has since been priced, with a measurement-only build that was reverted in the same
change.** Making both memos retain every failure — which is *unsound*, and deliberately so — collapses
the depth-800 parse from 18 901 646 rule entries to **149 914**, and the reject from twelve seconds to
**0.132 s**, flat at every depth from 315 to 800. That is an **87–91×** improvement against the ~41×
the bar needs. It is a **ceiling, not a fix**: the same build rejects valid input at depths 315 and
320, which is exactly the correctness the refusal was protecting. The open engineering question is
therefore narrow and well-posed — the sound depth-stamped channel already exists, and the question is
whether a recursion-tainted failure can be keyed by what it actually depends on. Tracked as
`ENGINE-UNIVERSAL-SERVICES.56`, with `.58` holding the layout-aware emission behind it.

> 📮 **Open question for embedders (Nexsim in particular) — tracked as
> `ENGINE-UNIVERSAL-SERVICES.57`.** The ratio bar above is derived from the parser and needs nobody's
> permission. What PGEN cannot derive is your side of it: **how many files a representative
> elaboration parses and whether that is parallel, what total front-end time reads as "hung" to your
> users, whether you would rather supply a per-file budget through the embedding API than have PGEN
> pick one, and whether your real corpora contain machine-generated RTL deep enough to reach the
> ceiling.** An answer can only tighten the bar, never loosen it, so no work waits on it — but it
> would let us set the target from your elaboration budget instead of from the parser's own
> behaviour. Please raise it with the PGEN maintainers when convenient. The ceiling's location did not move — a
sub-ceiling control is accepted at exactly the same depth as before — and this is locked from both
sides by `parser_embedding_*_over_ceiling_nesting_fails_in_bounded_time` (over the cliff: clean
diagnostic **within a wall-clock budget**) and
`parser_embedding_systemverilog_under_ceiling_nesting_still_parses` (under it: still accepted, so
bounded time cannot be bought by lowering the ceiling). ⛔ The reason that pair had to be written
is instructive: the pre-existing regression locks aim at depth **2000**, far past the cliff, so
while the defect was live *the only instrument aimed at it could not terminate*. A regression lock
that cannot finish is not a lock.

`PGEN_REPORT_MEMO_STATS` now reports `depth-ceiling rejections` and the `depth-gated` failure
count on its headline, so "this input is at the ceiling" is one command away rather than inferred
from a hang.

### The Rejection Now Says It Is A Ceiling

⛔ **Before you size for the plateau, understand what the ceiling costs you in *correctness*.**
Everything above treats crossing the ceiling as a performance question. It is an accuracy question
first, and for the whole life of the `1.3.1`/`1.3.2` guarantees the accuracy half had no owner.

Neither IEEE 1800 nor IEEE 1076 bounds expression nesting depth, so an over-ceiling input is
**syntactically well-formed** and the parser **rejects it anyway**. That is under-acceptance — the
same defect class PGEN calls out for over-acceptance, pointed in the direction that breaks a
working file. Measured on the shipped parsers, one nested-parenthesis shape per family:

| family | deepest ACCEPTED | first REJECTED |
|---|---:|---:|
| systemverilog | **322** | **323** |
| vhdl | **681** | **682** |

⚠️ Those are *this shape's* capacity, not the parsers' capacity in general — the ceiling counts
parser frames, and a different construct spends a different number of frames per nesting level.

The second half was worse than the first. A ceiling refusal is backtracked away like any other
failing alternative, so the parse surfaced as an ordinary syntax error at the **start** of the
construct:

```text
systemverilog: Parser did not consume full input at position 0   [furthest_position=668, …]
vhdl:          Parser did not consume full input at position 24  [furthest_position=741, …]
```

Grepping each shipped message for the five terms a consumer would search — `depth`, `ceiling`,
`limit`, `nest`, `recursion` — returned **0 of 5 on both families**. An embedder was told their
syntactically correct file had a syntax error near its start, and pointed at the wrong place. The
one true statement lived only behind `PGEN_REPORT_MEMO_STATS`: a debugging surface, not the error
they receive.

⭐ **`ENGINE-UNIVERSAL-SERVICES.61`(a) closes that half.** A rejection from a parse that contacted
the ceiling now says so, on every family, in the error an embedder actually gets:

```text
Parser did not consume full input at position 0 [recursion ceiling CONTACTED: this host's parse
stack ran out at 81675 parser frames, refusing 513 rule entries during this parse. The limit
counts PARSER FRAMES, not source nesting levels — one level of a nested construct costs several
frames — and it is derived from the parse thread's stack rather than compiled in, so raising
PGEN_PARSE_STACK_BYTES buys proportionally more depth on a host that can afford it. If the input
is syntactically valid it exceeds this parser's nesting capacity ON THIS HOST — not a syntax error
at position 0] [furthest_position=28906, +28906 bytes deeper than surface position]
```

Same census, same five terms, same two families: **5 of 5**. (That is verbatim output, captured
from the shipped release probe on 4 814 nested casts — one level past this host's boundary. The
**81 675** frames is what the guard actually stopped at on a 256 MiB parse stack, which is why the
number differs from host to host and from build mode to build mode. The wording above is the
post-`ENGINE-UNIVERSAL-SERVICES.65` form: the frame count is now the depth at which *this host's*
stack ran out, read from the guard that stopped there, and the sentence names the knob that moves
it. `.61`(a) shipped the same label with a fixed `4096-frame limit` in place of the first clause.)

Four properties of that label are worth knowing before you build on it:

- **It wraps, it never replaces.** The original diagnostic is rendered first and unchanged, and the
  surface position it reports is byte-identical to the unlabelled one. Anything already parsing
  `at position N` or the `furthest_position` suffix out of these messages keeps working.
- **The published diagnostic code is unchanged.** An over-ceiling rejection is still
  `E_PARSE_FAILURE`. Introducing a new code would have been the more legible design and was
  declined deliberately: it would silently stop matching every embedder arm that branches on
  `E_PARSE_FAILURE` today. Rust embedders who want the distinction *typed* should match
  `ParseError::RecursionCeilingContacted`, which carries the frame `ceiling` and the refusal
  `events` as fields rather than as prose.
- **It is a CONTACT report, not a causal proof, and the wording is hedged for exactly that
  reason.** The guard can refuse a branch that was going to be abandoned anyway, so the label says
  *"if the input is syntactically valid it exceeds this parser's nesting capacity"* rather than
  asserting the ceiling caused the rejection. A label that over-claimed would trade a diagnostic
  naming the wrong cause for one naming a wrong cause more often.
- **It costs the accept path nothing.** The label is composed on the failure path only, from a
  counter the guard already maintains, as a per-parse delta — so a parser reused across parses
  cannot inherit an earlier input's contact.

### The Recursion Ceiling Is Now The Host's Stack

> ⛔⛔ **CORRECTION, 2026-08-28.** An earlier version of this section ruled that *"4096 frames stays
> where it is"* on the strength of the contact census below. **That ruling was withdrawn**
> (`ENGINE-UNIVERSAL-SERVICES.63`), and the standing requirement is the opposite: **a PGEN RTL
> parser must ingest any RTL file — hand-written, generated, optimized or not — with no limitation
> or restriction, the only legitimate bound being the host's RAM and disk.** The census facts below
> survive; the conclusion drawn from them did not. What changed is that the census was measured over
> hand- and tool-written open-source RTL, and the target population is all RTL a real chip project
> contains. **Both limits have since been removed** — the frame ceiling (`.65`, described at the end
> of this section) and a second, silent one (`.66`, the section after it).

⭐ The ceiling rejects conforming source, so the fair question is whether *you* will meet it. The
honest answer for a generated-RTL flow is **yes, and sooner than the corpus suggests**.

**Ceiling contact across both released families, every file, the accept path included** — read from
the guard's own refusal counter rather than from the diagnostic, because the label above is composed
on the failure path and a deep *accepted* parse can touch the ceiling without producing one:

| family | files | accepted | rejected | **contacting the ceiling** |
|---|---:|---:|---:|---:|
| systemverilog (`sv_2017`) | 16 336 | 9 794 | 6 542 | **0** |
| vhdl | 13 720 | 4 369 | 9 351 | **0** |
| **total** | **30 056** | 14 163 | 15 893 | **0** |

Not one refusal, in either family, across fourteen upstream projects — OpenTitan, Verilator,
iverilog, Surelog, slang, verible, sv2v, black-parrot, VeeR, ibex, GHDL, VESTS, UVVM and others.
That is a fact about those files, and it is the whole of what the census establishes.

⛔⛔ **The oracle that turned it into a safety claim was measuring the wrong thing.** A lexical scan
of `()[]{}` depth puts the deepest SystemVerilog file at **22** levels, and an earlier version of
this page reported an **11.3× margin** on that basis. **A right-associative conditional chain
carries no brackets at all** — `c0 ? v0 : c1 ? v1 : … : d`, which is what a priority mux lowers to —
so that oracle scores such a file **0** however deep it nests. Re-measured on the axis that matters,
the same corpus reads:

| axis | max | p99.9 | deepest file |
|---|---:|---:|---|
| bracket depth (the old oracle) | 22 | 8 | `iverilog/…/comp1000.v` |
| **conditional-chain length** | **256** | 8 | `iverilog/…/br_gh330.v` |
| items in one `case` | **569** | 169 | `opentitan/…/pinmux_reg_top.sv` |

`br_gh330.v` is a 4-bit truth table lowered into two 256-arm mux chains; the parser's own AST depth
on it is **561**. `pinmux_reg_top.sv` is OpenTitan register-generator output.

⚠️ **And the ceiling is not a single depth.** It counts parser frames, and constructs spend frames at
different rates — a **13.4× range** across the shapes measured, so the number to plan against is the
worst, not the typical:

| shape | deepest accepted | frames per level |
|---|---:|---:|
| `$signed({1'b0, (s ? a : (…))})` — sign-coerced mux | **61** | 67.1 |
| `int'(s ? int'(a) : int'(…))` — width-coerced mux | 113 | 36.2 |
| `(s ? a : (…))` — mux, both arms parenthesised | **146** | 28.1 |
| `$signed(…)` | 163 | 25.1 |
| `int'(int'(…))` | 249 | 16.4 |
| `a ? b : (…)` | 272 | 15.1 |
| `{{{…}}}` | 314 | 13.0 |
| `(((…)))` | 322 | 12.7 |
| `f(f(f(…)))` | 340 | 12.0 |
| `c0 ? v0 : c1 ? v1 : …` — **bracket-free** chain | 2 041 | 2.0 |
| nested `generate`-`if` / `if` / `begin`-`end` | 583 / 815 / 816 | 7.0 / 5.0 / 5.0 |
| `y[0][0][0]…` | no frame limit | iterative — costs no frames |

⛔⛔ **What this means for a generated-RTL flow, demonstrated rather than argued.** Take the 568-arm
`unique case` out of that vendored OpenTitan register file and lower it the way every case→mux
lowering does — one conditional level per item, the file's own conditions and expressions:

| emitted style | levels | verdict |
|---|---:|---|
| bracket-free chain | 568 | accepted |
| **parenthesised** (sv2v / Verilator / yosys style) | 568 | **rejected — ceiling-labelled** |

⇒ whether your generated RTL parses today can depend on **your emitter's bracketing style**, which is
not a property of your design.

⚠️ **Read the margins carefully — they are per emission style, and an earlier draft of this section
compared two that are not commensurable.** `br_gh330.v` carries its 256 arms **bracket-free**, and
that style's own cliff is **2 041**, so the file's real headroom is **≈8×**. The **1.06×** figure
(271 ÷ 256) is what the margin would become **if the same logic were re-emitted parenthesised** — a
consequence of emitter choice, not a measurement of the corpus. The earlier *"11.3× margin"* was
incommensurable in the other direction: it divided one shape's frame capacity (249) by a
shape-agnostic bracket depth (22). **The defensible summary is that headroom ranges from ≈8× down to
≈1× across emission styles for the same logic**, which is why the ceiling is being removed rather
than re-sized.

### ✅ The Frame Ceiling Is Gone — What Replaced It, And What It Costs

`ENGINE-UNIVERSAL-SERVICES.65` removed the constant. **`4096` survives only as the depth at which
the guard first *measures* how much stack the parse has spent**; from there the ceiling is whatever
the parse thread's remaining headroom can still carry. Nothing in the mechanism reads a
frames-per-level table, a build mode, or a family: the per-frame cost is derived from the spend the
parse has already made, which is why release and debug — whose frames differ by ~8× — size
themselves.

**How you can tell it is genuinely host-derived, and not just a bigger number.** A larger boundary
proves nothing on its own. The discriminating prediction is *proportionality*: if the ceiling is the
stack, halving the stack must halve the depth. Measured on nested `int'(…)` casts, varying the
reservation through `PGEN_PARSE_STACK_BYTES`:

| parse-stack reservation | nested `int'(…)` | levels / MiB | mux, both arms parenthesised | levels / MiB |
|---:|---:|---:|---:|---:|
| 96 MiB | 1 807 | 18.823 | 1 142 | 11.896 |
| 128 MiB | 2 410 | 18.828 | 1 524 | 11.906 |
| 192 MiB | 3 611 | 18.807 | 2 286 | 11.906 |
| **256 MiB (the shipped default)** | **4 813** | 18.801 | **3 049** | 11.910 |

**Flat to 0.145 % and 0.120 % across a 2.67× range.** No constant can produce either curve. Every
boundary in the table is a genuine ceiling contact — confirmed by the `.61`(a) label rather than
assumed — and the boundaries are exactly reproducible (three accepts at 4 813, three rejections at
4 814).

⭐ **The second shape was a prediction, not an illustration.** The first version of this table
measured one shape, and that was recorded here as an open qualification rather than smoothed over.
Discharging it (`ENGINE-UNIVERSAL-SERVICES.71`), the mux column's 96 MiB point was used to publish a
falsifiable number *before* the run — **≈ 2 284 levels at 192 MiB** — and the measurement came back
at **2 286**, inside **0.09 %**.

**A second instrument agrees, and it is not the same measurement.** The two columns above come from
bisecting the accept/reject boundary. The guard also reports the frame ceiling the host gave it,
inside its own rejection text, and nothing in the bisection computes that number:

| parse-stack reservation | nested `int'(…)` frames | frames / MiB | mux frames | frames / MiB |
|---:|---:|---:|---:|---:|
| 96 MiB | 30 615 | 318.91 | 32 008 | 333.42 |
| 128 MiB | 40 827 | 318.96 | 42 684 | 333.47 |
| 192 MiB | 61 250 | 319.01 | 64 039 | 333.54 |
| **256 MiB (the shipped default)** | **81 675** | 319.04 | **85 392** | 333.56 |

Flat to **0.041 %** and **0.042 %** — tighter than the levels tables. A straight line fits each with
**residuals under 0.001 %** and an intercept of **−0.03 %** of the largest ceiling. ⭐ **That
near-zero intercept is what separates the two hypotheses**: a fixed ceiling is precisely the
intercept-only model — slope zero, the whole value in the constant — and this is its opposite.

⚠️ **Read the claim at its real width: two shapes, not the shape set.** The frame ceiling itself is
**shape-dependent at the 4.6 % level** (318.9 vs 333.6 frames/MiB), because different rule mixes have
different average frame sizes — so "frames per MiB" is a property of *this parser on this shape*, not
of the host alone. What is established is that both measured shapes are linear in the reservation,
which no constant can be; that every shape is, is an ordinary empirical expectation, not a proof.

**What each shape gained.** The before column is the fixed 4096-frame ceiling, which no reservation
could move; the after column is measured at a deliberately small **96 MiB** reservation — *under a
third of the shipped default* — so it is a floor on what you get, not a best case:

| shape | before | after (at 96 MiB) |
|---|---:|---:|
| `(s ? a : (…))` — mux, both arms parenthesised (the worst measured shape) | **146** | **1 142** |
| `$signed(…)` | 163 | 1 550 |
| `int'(int'(…))` | 249 | 1 807 → **4 813** at the default |
| `(((…)))` | 322 | 2 434 |
| `{{{…}}}` | 314 | 2 487 |
| `f(f(f(…)))` | 340 | 3 055 |
| VHDL `((( … )))` | 681 | **no cliff below 16 384**, the harness's own cap |

⚠️ That table is a subset, and the shapes left out are the ones that would have looked BEST: the
bisection is itself quadratic in depth, so the cheapest-per-level shapes (`begin`/`end` nesting,
nested `if`, the bracket-free ternary chain that already reached 2 041) were dropped for cost. They
spend the fewest frames per source level and therefore gain the most, so the numbers above are a
floor rather than a flattering selection.

⚠️ **What it costs, and this is not a footnote.** Parse cost is **quadratic in depth**. Measured on
the shipped release probe, the nested-cast accept path: 250 levels → **0.13 s**, 500 → 0.40 s,
1 000 → 1.43 s, 2 000 → 5.63 s, 4 000 → **22.24 s** — each doubling costing 3.1×, 3.6×, 3.9×, 3.95×,
converging on 4×. The refusal side moves with it: rejecting input one level past the boundary cost
**0.19 s** before this change and costs **37.7 s** now, still bounded and still ceiling-labelled.
⇒ **the stack is no longer the first thing a deep parse runs out of — time is.** If you feed this
parser thousands of nesting levels, plan for seconds, not milliseconds. The quadratic term is a
whole-stack scan on every rule entry and is tracked as `ENGINE-UNIVERSAL-SERVICES.68`.

⚠️ **One literal remains, named rather than left to be discovered.** The ceiling is derived from the
reservation; the reservation's *default* is still a compiled-in 256 MiB
(`ENGINE-UNIVERSAL-SERVICES.69`). It is host-settable today — `PGEN_PARSE_STACK_BYTES`, in bytes,
floored at 96 MiB — which is what made the proportionality table above possible at all.

⚠️ **Scope.** The derived bound applies to threads PGEN creates for parsing, which is every
embedding-API entry and every CLI instrument. A parse driven on some other thread keeps a HARD
ceiling at the first-sample depth, deliberately: this process cannot know that thread's stack size,
and guessing it is how a diagnosed rejection becomes a `SIGSEGV` in someone else's host. ⭐ Since
`ENGINE-UNIVERSAL-SERVICES.79` that hard ceiling is 4 096 frames at the default reservation and
proportionally LOWER when the operator has asked for a smaller one — never higher, because raising
it is the one direction that could make an unknown host less safe than it is today.

**No regression, both released families**: the external corpus shows **0 differences over 16 336
SystemVerilog files and 0 over 13 720 VHDL files**, per file rather than in aggregate, measured on
the fused graph that actually ships with the pre-change binary preserved for the A/B.

⚠️ **Read that zero for what it is.** The new bound only engages once a parse passes the first-sample
depth — 4 096 frames at the shipped default, and proportionally fewer at a smaller reservation since
`ENGINE-UNIVERSAL-SERVICES.79` — and **no corpus file gets there** — 0 of 400 sampled files record any ceiling contact, consistent
with the 0-of-30 056 census above. So the zero proves this change does not disturb the parses that
never approach the ceiling; it is *not* evidence that the derived bound itself is correct. That
evidence is the proportionality table, the boundary's exact reproducibility, and the engine's unit
tests.

### A Second Limit, And This One Was Silent — Now Removed

The shape sweep above turned up a limit nobody had recorded, and for an embedder it mattered more
than the frame ceiling. Every quantifier in every generated parser was capped at **10 000
iterations** (`SAFETY_LIMIT`). It did not refuse — **it stopped iterating**, so the parse failed at
the enclosing level as an ordinary *"did not consume full input"* with **no diagnostic naming the
cause**. A gate-level netlist of 10 001 instances did not parse, and real ASIC netlists run 10⁵–10⁷.

✅ **Removed in parser release `1.0.200` (SystemVerilog) / `1.0.6` (VHDL)**
(`ENGINE-UNIVERSAL-SERVICES.66`). The bound is now derived from the input — a repetition cannot have
more elements than the input has bytes, because every counted iteration must advance the cursor — so
it is unreachable for any real parse while still terminating a pathological one:

| generated shape | 10 001 items | 12 000 items | 20 000 items |
|---|---|---|---|
| flattened-netlist instances | rejected → **accepted** | rejected → **accepted** | rejected → **accepted** |
| wire declarations | rejected → **accepted** | rejected → **accepted** | rejected → **accepted** |
| `case` items | rejected → **accepted** | rejected → **accepted** | rejected → **accepted** |
| concat elements | accepted | rejected → **accepted** | rejected → **accepted** |
| module port list | accepted | rejected → **accepted** | rejected → **accepted** |

VHDL was affected identically — `when`/`else` chains and `elsif` ladders capped at the same 10 000 —
and is fixed by the same engine change.

⭐ **A real file in the corpus was being rejected by this cap, which is the clearest evidence it was
not a theoretical limit.** Against a pre-change per-file census, SystemVerilog shows **0 differences
over 16 336 files** and VHDL shows exactly **one of 13 720**:
`ghdl/testsuite/gna/issue3002/top_concat_16k.vhdl` — 192 KB carrying **17 407** `&` concatenation
operators, 1.7× past the cap — goes from rejected to accepted. It is a GHDL regression file, and
while the cap stood it looked like an ordinary syntax error. If you saw a rejection like that, this
release is the explanation. The frame-ceiling boundaries are byte-for-byte where they were.

⚠️ **What this does not yet buy you, stated plainly.** Removing the cap makes a large netlist *legal*;
it does not yet make one *ingestible*. Parse memory measures **2 500×–8 000× the input size** and
scales linearly — a 523 KB netlist peaks at ≈4.0 GB, a 1.1 MB one at ≈8.1 GB. So the limit is now
your host's RAM rather than a constant in our source, which is the intended shape, but it arrives
sooner than it should. That footprint is tracked as `ENGINE-UNIVERSAL-SERVICES.67`. With the frame
ceiling now removed (`.65`, above) it is — together with the quadratic depth cost (`.68`) — what
actually bounds ingestion today.

## Linter-Oriented Downstream Surfaces

One downstream direction now tracked explicitly is linter enablement. The idea is not that PGEN should become a linter by itself. The idea is that PGEN should be able to hand downstream tools a stronger front-end product than "just a parse tree."

That means the longer-term downstream surface should include:

- grammar-shaped ASTs,
- provenance-carrying semantic seeds,
- scope and declaration facts,
- pragma and waiver capture,
- version and dialect gating,
- and stable machine-readable export for later attribution and rule engines.

The detailed platform plan for that lane lives in:

- `docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md`

## Compiler And Elaborator-Oriented Downstream Surfaces

Another explicit downstream direction is compiler and elaborator enablement.

Here the goal is broader than linting. The goal is to let downstream tools start from a real front-end bundle instead of only:

- parse success,
- plus an opaque tree,
- plus a lot of hand-written glue work.

The longer-term workbench surface should therefore include:

- lossless or source-faithful front-end structure where needed,
- shaped ASTs,
- stable node ids,
- traversal and query helpers,
- semantic-bundle export,
- source-map and diagnostic-anchor fidelity,
- and explicit handoff scaffolding into later binding, typing, lowering, or elaboration passes.

The detailed platform plan for that lane lives in:

- `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`

## Parser Registry

The parser registry is the in-process discovery and routing seam used by several internal and proof paths. It matters because many contract lanes and probes do not call generated parsers directly; they go through registry-backed surfaces that downstream consumers can rely on more easily.

## Contracts As Downstream Truth

For public downstream use, the contracts are the most important authority. They state:

- what is supported,
- what version or profile names exist,
- what proof lane backs the surface,
- what bug-report workflow downstream users should follow.

## Release And Support Model

PGEN treats downstream parser support as a maintained surface:

- released parser bug ledgers exist,
- issue reporting protocol is explicit,
- maintenance releases can widen or harden syntax support without pretending the support boundary is undocumented.

That model is especially visible in the regex parser track because RGX actively consumes it.

For the regex track, PGEN is intentionally transparent about how PCRE2 compatibility evidence is derived. PCRE2 does not provide a formal EBNF/PEG for the whole flavor, so PGEN treats `pcre2syntax(3)` and `pcre2pattern(3)` as the prose intent layer, `src/pcre2_compile.c` as the hand-written parser edge-case authority, and PCRE2 upstream `testdata/testinput*` plus expected outputs as the executable regression oracle. RGX conformance reports are then folded back into the regex integration contract and released-parser bug ledger, rather than fixed as one-off examples.

The 2026-04-14 regex audit makes that split concrete: syntax forms such as string callouts, non-atomic lookarounds, scan-substring/script-run groups, strict VERSION conditionals, and PCRE2 escape spellings live in `regex.ebnf`, while source-derived compile checks such as verb legality, `\K` lookaround restrictions, forbidden class escapes, POSIX class names, and scan-substring reference existence live in the generated-host validation layer. That is intentional transparency, not a hidden second grammar.

The 2026-04-15 regex maintenance release extends that same split for RGX `PGEN-RGX-0058` through `PGEN-RGX-0060`: bounded variable-length lookbehind and Unicode capture names are now modeled as accepted PCRE2 surface, while unbounded lookbehind, malformed named references, names that violate PCRE2's `128`-byte limit, and empty substantive character classes remain generated-host compile-contract rejections. Orphan `\E` inside a character class follows `pcre2_compile.c` rather than folklore: it is a zero-width scanner marker, not a literal `E` atom, so non-empty classes such as `^[\Eabc]` are valid while `[\E]` remains invalid.

The 2026-04-16 regex maintenance release continues that source-derived approach for RGX `PGEN-RGX-0061` and `PGEN-RGX-0062`. PCRE2 `\C` now transports as a dedicated `single_byte_escape` AST node because `pcre2_compile.c` treats it as `ESC_C`, not as a generic escaped `C`. Conditional assertion tests may also be preceded by an explicit callout, so patterns like `^(?(?C25)(?=abc)abcd|xyz)` and the string-callout variant `^(?(?C$abc$)(?=abc)abcd|xyz)` preserve both `condition_callout` and `condition_assertion` structure.

The current regex maintenance release also documents a resource-depth distinction that downstream embedders should understand: legal PCRE2 patterns can be syntactically deep enough to stress a generated parser even when they are small in bytes. PGEN therefore runs generated regex entrypoints on a larger bounded worker stack (`64 MiB`) and keeps the generated recursion guard bounded (`4096`) but high enough for the retained PCRE2 conformance samples.

PNR now also has a PGEN-side downstream contract addendum. It captures PNR's inbound request for future LEF, DEF, Liberty, SDC, structural Verilog netlist, and SPEF parser crates, while explicitly marking those parser-family releases as pending rather than shipped.

For that pending PNR/SDC lane, `docs/tcl/md/tcl.md` records a local Tcl syntax note so future work remembers that SDC is Tcl-shaped at the command/word/substitution layer. That note is reference input only; the authoritative SDC command semantics still need the Synopsys SDC source path recorded in the PNR contract before EBNF work starts.

## Primary Source Docs

- `rust/docs/EMBEDDING_API_CONTRACT.md`
- `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`
- `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
- `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
- `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_PNR_PARSER_INTEGRATION_CONTRACT.md`
- `docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md`
- `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`
