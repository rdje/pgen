//! Dedicated big-stack execution for generated-parser entry boundaries.
//!
//! `SV-CORPUS-GRAD.8c.3` — the recursion ceiling must bound the REAL stack
//! (the PGEN-RGX-0085 law, generalized parser-agnostically). The generated
//! parsers carry a clean mutual-recursion ceiling
//! (4096 frames when `.8c.3` measured it — since
//! `ENGINE-UNIVERSAL-SERVICES.65` that number is only the depth at which the
//! guard first SAMPLES this module's stack extent, and the ceiling itself is
//! whatever the reservation below can really carry), but a ceiling only
//! protects the process if the thread it runs on has enough stack for the
//! frames it admits.
//! Measured (2026-07-22, deep-parens SV synthetic,
//! `docs/tasks/artifacts/sv_corpus_grad/8c3_stack_ceiling_measurement.md`):
//! ≈2 KB/frame in release and ≈17 KB/frame in debug, so the ceiling needs
//! ≈8 MB (release) / ≈70 MB (debug) of real stack — while the default main
//! stack is 8 MB and libtest worker stacks are 2 MB. Result: a ~400-deep
//! parenthesized SV expression (≈4 KB of text) hard-aborted the RELEASE
//! process (uncatchable SIGABRT, rc 134) before the ceiling could fire.
//!
//! The fix locus is the integration/instrument BOUNDARY, not the parse loop:
//! run generated-parser work on a thread with a 256 MiB stack (a virtual
//! reservation, lazily committed — RSS only grows with real use), so the
//! existing ceiling provably fires before the guard page in BOTH build modes
//! with ≥2× margin (release 8 MB, debug 70 MB ≪ 256 MiB). Nothing inside the
//! parse loop changes — zero hot-path cost for every family (the regex
//! perf-floor law is untouched; the regex embedding path keeps its own
//! RGX-0085 worker + nesting pre-check unchanged).
//!
//! Callers:
//! - the embedding-API grammar-family entries (SystemVerilog, VHDL) route
//!   each parse through [`run_on_dedicated_parse_stack`] (spawn-per-call:
//!   ~50–100 µs of thread setup is noise against ms-scale HDL file parses,
//!   and it preserves host-side parallelism — no cross-thread serialization
//!   at the Nexsim boundary, unlike a shared long-lived worker);
//! - the CLI instruments (`parseability_probe`, `ast_pipeline`) wrap their
//!   whole `main` body via [`run_cli_main_on_dedicated_parse_stack`] (one
//!   spawn per process; every driver — parse, AST dump, cert coverage,
//!   stimuli replay — inherits the guaranteed stack, and thread-local
//!   trace/dump configuration stays coherent because setup and parsing run
//!   on the same thread).
//!
//! Re-entrancy: a caller already on a dedicated parse stack (e.g. an
//! embedding-API call made from inside the wrapped `ast_pipeline` main) runs
//! inline — same stack guarantee, no nested spawn.
//!
//! ⭐ `ENGINE-UNIVERSAL-SERVICES.43` — THE OTHER HALF OF THE SAME LAW. This
//! module guarantees the ceiling can FIRE; it says nothing about what happens
//! next, and for a while what happened next was that the parse kept searching.
//! A guard verdict propagates as an ordinary retryable failure, so once the
//! ceiling had tainted the packrat failure cache the enclosing alternatives
//! re-explored indefinitely: measured on the shipped SV parser, 315 nested
//! parens accepted in 0.16 s and 320 returned no result in 30 s — a hang where
//! `.8c.3` had replaced an abort. ⇒ "the ceiling always fires" is necessary and
//! not sufficient; the ceiling must also BOUND the work that follows it, which
//! is a memoisation property and lives in the generated parsers
//! (`memo_fail_depth_gated` / `thin_fail_depth_gated`), not here.

/// Stack size for dedicated parse threads: 256 MiB.
///
/// Sizing law (`SV-CORPUS-GRAD.8c.3` measurement): the 4096-frame recursion
/// ceiling needs ≈8 MB of real stack in release and ≈70 MB in debug for the
/// most stack-hungry measured family (SystemVerilog deep-parens, ≈2 KB /
/// ≈17 KB per logical frame). 256 MiB gives the debug worst case a >3×
/// margin, so the ceiling always fires (clean `RecursionDepthExceeded`
/// diagnostic) before the guard page (process SIGABRT). Virtual reservation
/// only — the pages are committed lazily as the parse actually deepens.
pub const DEDICATED_PARSE_STACK_BYTES: usize = 256 * 1024 * 1024;

/// `ENGINE-UNIVERSAL-SERVICES.65` — the smallest reservation this module will
/// honour, so an operator cannot shrink the stack below the depth at which the
/// guard first samples it.
///
/// 4096 frames is where a generated parser takes its first stack measurement
/// (`GENERATED_RECURSION_GUARD_STACK_PROBE_FLOOR`), and `SV-CORPUS-GRAD.8c.3`
/// measured ≈17 KB/frame in debug — the expensive build mode. A floor below
/// ≈70 MB could therefore let the first sample happen after the stack was
/// already gone, which is the abort this whole module exists to prevent.
pub const MIN_DEDICATED_PARSE_STACK_BYTES: usize = 96 * 1024 * 1024;

/// `ENGINE-UNIVERSAL-SERVICES.65` — the reservation actually used for dedicated
/// parse threads: [`DEDICATED_PARSE_STACK_BYTES`] unless the host operator
/// names another size in `PGEN_PARSE_STACK_BYTES`.
///
/// ⛔ **WHY THIS IS A KNOB AND NOT A CONSTANT.** Since `.65` the recursion
/// ceiling is derived from the stack a parse really owns, so THIS number — not
/// a frame count — is what a host spends to accept deeper input. The director's
/// capability mandate makes the host's own RAM the legitimate bound
/// (*"the only obvious limit shall be the host RAM and disk allowance"*), and a
/// host cannot express its allowance to a compiled-in literal. It is also what
/// makes the ceiling FALSIFIABLE: halve the reservation and the measured
/// accept boundary must halve with it, which is the control that separates
/// "derived from the stack" from "a bigger number than before".
///
/// Resolved once per process. An unparseable or too-small value falls back to
/// the floor rather than failing the parse — a stack reservation is not a place
/// to turn a typo into an outage — and a value below
/// [`MIN_DEDICATED_PARSE_STACK_BYTES`] is raised to it.
pub fn dedicated_parse_stack_bytes() -> usize {
    static RESOLVED: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *RESOLVED.get_or_init(|| match std::env::var("PGEN_PARSE_STACK_BYTES") {
        Ok(raw) => raw
            .trim()
            .parse::<usize>()
            .map(|bytes| bytes.max(MIN_DEDICATED_PARSE_STACK_BYTES))
            .unwrap_or(DEDICATED_PARSE_STACK_BYTES),
        Err(_) => DEDICATED_PARSE_STACK_BYTES,
    })
}

thread_local! {
    // True on threads created by this module (and only those), so nested
    // boundary crossings run inline instead of spawning again.
    static ON_DEDICATED_PARSE_STACK: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };

    // `ENGINE-UNIVERSAL-SERVICES.65` — the dedicated parse thread's stack
    // extent as `(base_address, reserved_bytes)`, recorded once at thread
    // entry. `None` on every thread this module did not create, which is the
    // whole point: a caller that cannot prove how much stack it owns must not
    // be told a number.
    static PARSE_STACK_EXTENT: std::cell::Cell<Option<(usize, usize)>> =
        const { std::cell::Cell::new(None) };
}

/// `ENGINE-UNIVERSAL-SERVICES.65` — the calling frame's approximate stack
/// address.
///
/// The address of a local is inside the frame the call is inlined into, so the
/// difference between this and the thread's recorded base is the stack the
/// thread has actually consumed. `black_box` keeps the compiler from reasoning
/// the anchor away; the value is an approximation by construction (the
/// optimizer places frame slots as it likes), which is why every consumer
/// spends it against a reserve rather than to the last byte.
///
/// ⛔ Callers must treat a value that does not decrease with depth as
/// "unknown", not as zero usage: stack growth direction is a target property,
/// and every currently supported target grows down.
#[inline(always)]
pub fn approximate_stack_pointer() -> usize {
    let anchor = 0u8;
    std::hint::black_box(&anchor) as *const u8 as usize
}

/// `ENGINE-UNIVERSAL-SERVICES.65` — how far ABOVE the recorded anchor the
/// thread's true stack top may sit.
///
/// ⛔ **MEASURED, not assumed — and the assumption it replaces was wrong.**
/// [`approximate_stack_pointer`] returns the address of a local, and the
/// compiler places frame slots in whatever order it likes, so the anchor taken
/// at thread entry is a slot INSIDE that frame rather than the frame's top. A
/// probe taken from a shallower position in the same frame can therefore sit
/// ABOVE the recorded base: the first version of this module's own test read
/// `here = base + 176` and failed, which is how the error was found rather than
/// shipped.
///
/// Padding the recorded base upward makes the derived usage an OVER-estimate by
/// at most this much, which is the safe direction (the guard fires marginally
/// early rather than marginally late) and removes the only case in which a
/// shallow probe could read as "direction unknown". One thread-entry frame is
/// orders of magnitude smaller than this pad, and the pad itself is negligible
/// against the reservation it is subtracted from.
const STACK_BASE_ANCHOR_PAD_BYTES: usize = 64 * 1024;

/// `ENGINE-UNIVERSAL-SERVICES.65` — the current thread's parse-stack extent as
/// `(base_address, reserved_bytes)`, or `None` when this thread's stack size is
/// not knowable.
///
/// This is the HOST-derived quantity that replaced the compiled-in recursion
/// ceiling. A generated parser's guard asks how much stack is really left
/// rather than how many frames a developer once typed
/// (`ast_pipeline::RecursionGuard::extend_depth_probe`); off a dedicated parse
/// thread the answer is `None` and the guard keeps its conservative compiled
/// floor, because guessing an unknown thread's stack size is exactly the
/// literal this leaf exists to delete.
pub fn current_parse_stack_extent() -> Option<(usize, usize)> {
    PARSE_STACK_EXTENT.with(|cell| cell.get())
}

/// `ENGINE-UNIVERSAL-SERVICES.69`(d) — WHY the depth probe refused to move the
/// recursion ceiling out.
///
/// The probe answers every unknown by REFUSING, which is the right safety
/// posture (a stack overflow aborts the embedder; a refusal only rejects an
/// input) — but it makes five very different situations indistinguishable from
/// outside. Four are statements about what this process could not LEARN; only
/// [`StackProbeRefusal::BudgetExhausted`] is the host stack actually being
/// spent. The frame count a rejection already carries
/// (`RecursionGuard::last_ceiling_depth`) is the same number in all five, so
/// before this enum an operator could not tell *"this input is too deep for
/// this host"* from *"this parse ran on a thread whose stack size we do not
/// know"*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackProbeRefusal {
    /// No recorded extent — a thread this module did not create, so the process
    /// cannot know how much stack it owns and the compiled sample floor stands.
    /// ⛔ Nothing about bytes is knowable here: every byte field of the
    /// observation is `0` meaning NOT MEASURED, never "measured as zero".
    ExtentUnknown,
    /// The probe address was not below the recorded base, so either the stack
    /// does not grow downward on this target or the guard travelled to another
    /// thread. The reservation is known; the spend is not.
    DirectionUnknown,
    /// Under one byte spent per live frame, so the per-frame cost is not
    /// honestly divisible and extending the ceiling would be extending it off a
    /// rounding error. ⚠️ The SPEND is real here — it is the per-frame QUOTIENT
    /// that is not, which is why [`StackProbeObservation::spend_is_measured`]
    /// still answers `true`.
    FrameCostUnmeasurable,
    /// ⭐ The ONLY refusal that is a fact about the STACK rather than about this
    /// process's knowledge: the spendable budget is used up. This is the
    /// measurement `.69` was opened for — [`StackProbeObservation::used_bytes`]
    /// against [`StackProbeObservation::budget_bytes`] is how tightly the
    /// derived ceiling really tracks the reservation, and it replaces the
    /// `reservation ÷ frames` arithmetic `.71` could only publish as an UPPER
    /// BOUND.
    BudgetExhausted,
    /// Headroom is left but will not pay for one whole frame at the measured
    /// per-frame cost.
    NoFrameAffordable,
}

/// `ENGINE-UNIVERSAL-SERVICES.69`(d) — the depth probe's own reading at the
/// moment it refused.
///
/// ⛔ **A ZERO BYTE FIELD MEANS NOT MEASURED.** Which fields carry a real
/// measurement is a function of [`Self::cause`] and of nothing else: the probe
/// refuses at the first thing it cannot establish, so a later quantity was
/// never computed rather than computed as zero. [`Self::spend_is_measured`]
/// answers the only question most consumers have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackProbeObservation {
    /// Which of the five refusals fired.
    pub cause: StackProbeRefusal,
    /// The live frame depth the probe was asked about.
    pub depth: usize,
    /// Stack bytes spent from the thread's recorded base. NOT MEASURED (`0`)
    /// for [`StackProbeRefusal::ExtentUnknown`] and
    /// [`StackProbeRefusal::DirectionUnknown`].
    pub used_bytes: usize,
    /// The thread's whole reservation. NOT MEASURED (`0`) only for
    /// [`StackProbeRefusal::ExtentUnknown`].
    pub reservation_bytes: usize,
    /// The share of the reservation the probe is willing to spend
    /// (`reservation − reservation / STACK_HEADROOM_RESERVE_DIVISOR`). NOT
    /// MEASURED (`0`) until the spend itself is measurable.
    pub budget_bytes: usize,
    /// `used_bytes / depth` — the per-frame cost MEASURED for this parse rather
    /// than read from a table. NOT MEASURED (`0`) until the spend is
    /// measurable.
    pub bytes_per_frame: usize,
}

impl StackProbeObservation {
    /// True when [`Self::used_bytes`] is a real measurement of this thread's
    /// stack rather than a not-measured `0`.
    pub fn spend_is_measured(&self) -> bool {
        !matches!(
            self.cause,
            StackProbeRefusal::ExtentUnknown | StackProbeRefusal::DirectionUnknown
        )
    }
}

thread_local! {
    // `ENGINE-UNIVERSAL-SERVICES.69`(d) — the most recent refusal observed on
    // THIS thread. Written only from the guard's `#[cold]` probe.
    static LAST_STACK_PROBE_REFUSAL: std::cell::Cell<Option<StackProbeObservation>> =
        const { std::cell::Cell::new(None) };
}

/// `ENGINE-UNIVERSAL-SERVICES.69`(d) — the depth probe's reading at its most
/// recent refusal on this thread, or `None` when it has not refused.
///
/// ⛔ **WHY A THREAD-LOCAL AND NOT A FIELD ON THE GUARD.** The reading is
/// produced by `RecursionGuard`, which is owned by the generated parser, which
/// the parse adapters never hand back (`parse_sample_detail_*` return
/// `Result<()>`). Reaching a guard field would mean changing every family's
/// adapter signature — i.e. CODEGEN, and a regeneration of all 11 parsers for a
/// diagnostic. A thread-local is readable by anything on the parse thread and
/// costs the accept path nothing, so this whole instrument lands as ZERO
/// generated bytes.
///
/// ⚠️ Carried across the [`run_on_dedicated_parse_stack`] join, so an embedder
/// reads its own parse's reading even though the parse ran on a thread that no
/// longer exists.
pub fn last_stack_probe_refusal() -> Option<StackProbeObservation> {
    LAST_STACK_PROBE_REFUSAL.with(|cell| cell.get())
}

/// `ENGINE-UNIVERSAL-SERVICES.69`(d) — forget this thread's last refusal.
///
/// A consumer that wants a PER-PARSE reading rather than a per-thread one calls
/// this before the parse: the record is a lifetime latch otherwise, which is
/// exactly the staleness `.61`(a) had to fix for the ceiling-contact counter by
/// snapshotting it at parse entry.
pub fn clear_last_stack_probe_refusal() {
    set_last_stack_probe_refusal(None);
}

/// `ENGINE-UNIVERSAL-SERVICES.69`(d) — record a refusal. Called only from the
/// guard's `#[cold]` depth probe, so the accept path never reaches it.
pub(crate) fn record_stack_probe_refusal(observation: StackProbeObservation) {
    set_last_stack_probe_refusal(Some(observation));
}

fn set_last_stack_probe_refusal(observation: Option<StackProbeObservation>) {
    LAST_STACK_PROBE_REFUSAL.with(|cell| cell.set(observation));
}

/// Why a dedicated-stack run did not return a value.
pub enum DedicatedParseStackError {
    /// The OS refused the thread spawn (the 256 MiB reservation is virtual,
    /// so this is effectively thread-exhaustion only).
    Spawn(std::io::Error),
    /// The job panicked. The payload is the original panic value so callers
    /// can either `resume_unwind` it (CLI semantics — identical crash
    /// surface to running inline) or convert it to a structured diagnostic
    /// (embedding semantics — a host process must never be aborted by a
    /// parser bug).
    Panic(Box<dyn std::any::Any + Send + 'static>),
}

/// Runs `f` with [`dedicated_parse_stack_bytes`] of stack guaranteed.
///
/// Already-on-a-dedicated-stack callers run inline (no nested spawn); all
/// others pay one thread spawn+join (~50–100 µs). Panics inside `f` are
/// captured as [`DedicatedParseStackError::Panic`] in both paths, so the
/// two routes are behaviorally identical.
pub fn run_on_dedicated_parse_stack<T, F>(
    thread_name: &str,
    f: F,
) -> Result<T, DedicatedParseStackError>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    if ON_DEDICATED_PARSE_STACK.with(|c| c.get()) {
        // AssertUnwindSafe: `f` is moved in and its result is discarded on
        // panic, so no broken invariant can be observed afterwards — the
        // same justification as the thread-boundary catch below (a thread
        // join IS a catch_unwind).
        return std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
            .map_err(DedicatedParseStackError::Panic);
    }
    let handle = std::thread::Builder::new()
        .name(thread_name.to_string())
        .stack_size(dedicated_parse_stack_bytes())
        .spawn(move || {
            ON_DEDICATED_PARSE_STACK.with(|c| c.set(true));
            // `ENGINE-UNIVERSAL-SERVICES.65` — record the extent at the
            // shallowest point of the thread, so every later depth probe
            // measures consumption from the true top of the reservation.
            PARSE_STACK_EXTENT.with(|c| {
                c.set(Some((
                    approximate_stack_pointer() + STACK_BASE_ANCHOR_PAD_BYTES,
                    dedicated_parse_stack_bytes(),
                )))
            });
            let value = f();
            // `ENGINE-UNIVERSAL-SERVICES.69`(d) — carry the depth probe's
            // reading back across the join. The parse thread is the ONLY one
            // that can observe a refusal and its thread-local dies with it, so
            // without this the embedding path could never see the byte quantity
            // the CLI path reports. The child's reading REPLACES the caller's
            // (including with `None`), so the record always describes the parse
            // that just ran rather than latching an older one.
            (value, last_stack_probe_refusal())
        })
        .map_err(DedicatedParseStackError::Spawn)?;
    let (value, observed) = handle.join().map_err(DedicatedParseStackError::Panic)?;
    set_last_stack_probe_refusal(observed);
    Ok(value)
}

/// CLI-main wrapper: runs `f` on a dedicated parse stack and preserves CLI
/// crash semantics exactly — a panic in the body resumes unwinding on the
/// caller (same abort + backtrace surface as an unwrapped `main`), and a
/// spawn failure is a loud startup panic (there is nothing useful a CLI can
/// do without its guaranteed stack).
pub fn run_cli_main_on_dedicated_parse_stack<T, F>(thread_name: &str, f: F) -> T
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    match run_on_dedicated_parse_stack(thread_name, f) {
        Ok(value) => value,
        Err(DedicatedParseStackError::Panic(payload)) => std::panic::resume_unwind(payload),
        Err(DedicatedParseStackError::Spawn(err)) => panic!(
            "failed to spawn dedicated parse-stack thread '{}': {}",
            thread_name, err
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_value_from_dedicated_thread() {
        let result = run_on_dedicated_parse_stack("pgen-test-stack", || 41 + 1);
        assert!(matches!(result, Ok(42)));
    }

    #[test]
    fn nested_call_runs_inline_and_returns() {
        let result = run_on_dedicated_parse_stack("pgen-test-outer", || {
            // The nested call must not deadlock or spawn-fail; the inline
            // path keeps the same result contract.
            run_on_dedicated_parse_stack("pgen-test-inner", || "nested").unwrap_or("error")
        });
        assert!(matches!(result, Ok("nested")));
    }

    #[test]
    fn panic_is_captured_not_propagated() {
        let result: Result<(), _> =
            run_on_dedicated_parse_stack("pgen-test-panic", || panic!("boom"));
        assert!(matches!(result, Err(DedicatedParseStackError::Panic(_))));
    }

    #[test]
    fn nested_panic_is_captured_by_inline_path() {
        let result = run_on_dedicated_parse_stack("pgen-test-outer-panic", || {
            let inner: Result<(), _> =
                run_on_dedicated_parse_stack("pgen-test-inner-panic", || panic!("inner boom"));
            matches!(inner, Err(DedicatedParseStackError::Panic(_)))
        });
        assert!(matches!(result, Ok(true)));
    }

    #[test]
    fn stack_extent_is_known_only_on_a_dedicated_thread() {
        // `ENGINE-UNIVERSAL-SERVICES.65` — the extent is what replaced the
        // compiled-in recursion ceiling, so "who is allowed to see one" is a
        // correctness property, not a detail: an unknown thread must report
        // `None` so the guard keeps its conservative floor.
        assert_eq!(
            current_parse_stack_extent(),
            None,
            "a libtest worker thread must not claim a parse-stack extent"
        );
        // `DedicatedParseStackError` is deliberately not `Debug` (it carries a
        // panic payload), so unwrap through a match rather than `expect`.
        let seen = match run_on_dedicated_parse_stack("pgen-test-extent", || {
            let extent = current_parse_stack_extent();
            let here = approximate_stack_pointer();
            (extent, here)
        }) {
            Ok(seen) => seen,
            Err(_) => panic!("dedicated parse-stack spawn failed"),
        };
        let (extent, here) = seen;
        let Some((base, reserved)) = extent else {
            panic!("a dedicated parse thread must know its own extent");
        };
        assert_eq!(reserved, dedicated_parse_stack_bytes());
        assert!(
            here < base && base - here < reserved,
            "a shallow probe must read as a small, positive spend: here={here} base={base}"
        );
    }

    /// `ENGINE-UNIVERSAL-SERVICES.65` — THE ASSUMPTION THE DERIVED CEILING RESTS
    /// ON: the stack grows DOWNWARD, and spend measured from the recorded base
    /// increases with call depth.
    ///
    /// The guard converts (base − current) into a per-frame cost and then into a
    /// frame capacity. If that difference did not grow monotonically with depth
    /// the capacity would be meaningless, so it is measured here rather than
    /// asserted in prose — and it is measured at real depth, because the first
    /// version of the test above proved that a difference read WITHIN one frame
    /// says nothing about direction.
    #[test]
    fn stack_spend_grows_with_call_depth() {
        #[inline(never)]
        fn descend(remaining: usize) -> usize {
            let mut ballast = [0u8; 1024];
            ballast[remaining % ballast.len()] = remaining as u8;
            std::hint::black_box(&ballast);
            if remaining == 0 {
                approximate_stack_pointer()
            } else {
                descend(remaining - 1)
            }
        }
        let measured = run_on_dedicated_parse_stack("pgen-test-growth", || {
            let (base, reserved) = current_parse_stack_extent().expect("extent on this thread");
            (base, reserved, descend(16), descend(256), descend(1024))
        });
        let Ok((base, reserved, at16, at256, at1024)) = measured else {
            panic!("dedicated parse-stack spawn failed");
        };
        let (s16, s256, s1024) = (base - at16, base - at256, base - at1024);
        assert!(
            s16 < s256 && s256 < s1024,
            "spend must grow with depth: 16={s16} 256={s256} 1024={s1024}"
        );
        assert!(
            s1024 < reserved,
            "1024 ballasted frames must fit inside the reservation ({s1024} of {reserved})"
        );
        // The per-frame cost the guard divides by is a real quantity: with a
        // 1 KiB ballast per frame it must be at least that.
        assert!(
            (s1024 - s256) / (1024 - 256) >= 1024,
            "per-frame spend under-reads its own ballast"
        );
    }

    /// `ENGINE-UNIVERSAL-SERVICES.69`(d) — A READING TAKEN ON THE PARSE THREAD
    /// SURVIVES THE JOIN, AND A CLEAN PARSE DOES NOT INHERIT A STALE ONE.
    ///
    /// The parse thread is the only one that can observe a refusal, and its
    /// thread-local dies with it. Without the hand-off across the join the
    /// embedding path — which spawns per call — could never read the byte
    /// quantity the CLI path prints, so this is the property that makes the
    /// instrument reach both callers rather than one.
    #[test]
    fn a_refusal_observed_on_the_parse_thread_survives_the_join() {
        clear_last_stack_probe_refusal();
        assert_eq!(last_stack_probe_refusal(), None);
        let sentinel = StackProbeObservation {
            cause: StackProbeRefusal::BudgetExhausted,
            depth: 7,
            used_bytes: 11,
            reservation_bytes: 13,
            budget_bytes: 17,
            bytes_per_frame: 19,
        };
        let ran = run_on_dedicated_parse_stack("pgen-test-probe-join", move || {
            // The child starts clean — a fresh thread owns a fresh record.
            assert_eq!(last_stack_probe_refusal(), None);
            record_stack_probe_refusal(sentinel);
            true
        });
        assert!(matches!(ran, Ok(true)));
        assert_eq!(
            last_stack_probe_refusal(),
            Some(sentinel),
            "a reading taken on the parse thread must survive the join"
        );

        // RED CONTROL: the child's `None` must REPLACE the caller's reading. If
        // the hand-off only propagated `Some`, the record would latch the first
        // refusal of the process forever and every later parse would be
        // reported as having contacted the ceiling.
        let ran = run_on_dedicated_parse_stack("pgen-test-probe-join-clean", || true);
        assert!(matches!(ran, Ok(true)));
        assert_eq!(
            last_stack_probe_refusal(),
            None,
            "a parse that refused nothing must clear the previous reading"
        );
        clear_last_stack_probe_refusal();
    }

    #[test]
    fn reservation_is_never_below_the_first_sample_depth() {
        // The resolved reservation must always leave room for the depth at which
        // a generated parser takes its FIRST stack measurement; a smaller one
        // could let the abort happen before the guard ever looks.
        assert!(dedicated_parse_stack_bytes() >= MIN_DEDICATED_PARSE_STACK_BYTES);
    }

    #[test]
    fn dedicated_thread_has_the_big_stack() {
        // Recurse to a depth that needs far more than the 2 MiB libtest
        // worker stack (but well under 256 MiB) — only survivable if the
        // closure really runs on the dedicated stack. ~12,000 frames ×
        // ~65 KB/frame ≈ 780 MB would overflow; use 48 KB × 3000 ≈ 144 MB
        // in debug terms conservatively: pin actual usage with an explicit
        // per-frame buffer instead of relying on optimizer behavior.
        fn burn(depth: usize) -> usize {
            // 64 KiB per frame, written so it cannot be optimized away.
            let mut buf = [0u8; 64 * 1024];
            buf[depth % buf.len()] = depth as u8;
            if depth == 0 {
                buf[0] as usize
            } else {
                burn(depth - 1).wrapping_add(buf[depth % buf.len()] as usize)
            }
        }
        // 512 frames × 64 KiB = 32 MiB — beyond any default test stack,
        // comfortably inside the 256 MiB reservation.
        let result = run_on_dedicated_parse_stack("pgen-test-depth", || burn(512));
        assert!(result.is_ok());
    }
}
