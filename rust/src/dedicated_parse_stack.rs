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
/// Sizing law: a generated parser takes its first stack measurement at
/// `GENERATED_RECURSION_GUARD_STACK_PROBE_FLOOR` frames, and reaching that depth
/// has to fit inside the spendable budget or the process dies before the guard
/// can refuse. This reservation satisfies that on every family measured — which
/// is SystemVerilog, and only SystemVerilog.
///
/// ⛔ **THE MARGIN IS SMALLER THAN THIS COMMENT USED TO CLAIM, AND THE NUMBERS
/// ARE NOW DATED** (`ENGINE-UNIVERSAL-SERVICES.79`, 2026-08-30). It read *"≈70 MB
/// in debug … a >3× margin"* from `SV-CORPUS-GRAD.8c.3`'s 2026-07 measurement of
/// ≈17 KB per debug frame. Re-measured with `.69`(d)'s own instrument, the debug
/// per-frame cost is **25 335 B** (`sv_2017`), **29 031 B** (`verilog_2005`) and
/// **3 071–3 083 B** in release — so 4 096 frames cost **99–113 MiB** in debug,
/// against the **240 MiB** this reservation makes spendable. The real margin is
/// **2.1–2.4×**, not >3×, and it is measured on SystemVerilog alone
/// (`.79`(a) owes the per-family, per-profile, per-build-mode census).
///
/// ⭐ Every SMALLER reservation keeps that same margin without re-measuring
/// anything, because [`first_stack_sample_depth`] scales the first sample with
/// the reservation — see its doc comment for why that is a property rather than
/// a number. Virtual reservation only — the pages are committed lazily as the
/// parse actually deepens.
pub const DEDICATED_PARSE_STACK_BYTES: usize = 256 * 1024 * 1024;

/// `ENGINE-UNIVERSAL-SERVICES.65` — the smallest reservation this module will
/// honour.
///
/// ⛔⛔ **THIS CONSTANT NO LONGER CARRIES A SAFETY ARGUMENT, AND
/// `ENGINE-UNIVERSAL-SERVICES.79` IS WHY IT MUST NOT.** It was documented as
/// *"a floor below ≈70 MB could let the first sample happen after the stack was
/// already gone"* — i.e. 4 096 frames × the ≈17 KB per debug frame
/// `SV-CORPUS-GRAD.8c.3` measured in 2026-07. That per-frame cost has since been
/// re-measured at **25 335–29 031 B** (`.78`, and it differs by 14.6 % between two
/// PROFILES of one family), so reaching the first sample costs **99–113 MiB**
/// against the **90 MiB** this floor makes spendable — and the floor produced
/// exactly the abort its comment promised to prevent. Measured: a **642-byte**
/// SystemVerilog file (300 nested parens) aborts at `rc 134` in 0.5 s with
/// `PGEN_PARSE_STACK_BYTES` set to this value, and `dedicated_parse_stack_bytes`
/// CLAMPS every smaller request up onto it, so `PGEN_PARSE_STACK_BYTES=1` landed
/// on the unsafe number by design.
///
/// ⛔ **A BYTE COUNT CANNOT HOLD THAT ARGUMENT.** To do so it has to encode a
/// per-frame cost, and a per-frame cost is a per-family, per-profile,
/// per-build-mode measurement that will move again — re-deriving this literal
/// from today's numbers would be the same defect wearing a fresher one. The
/// argument now lives where it can be ENFORCED instead of asserted:
/// [`first_stack_sample_depth`] derives the first sample FROM the reservation, so
/// reachability holds at every reservation by construction.
///
/// What remains here is an ordinary usability floor — below it the derived ceiling
/// is too shallow to be worth offering, and a host that names a smaller number
/// gets this one instead. It is a policy, not a proof.
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

/// `ENGINE-UNIVERSAL-SERVICES.79` — the depth at which a parse running on THIS
/// THREAD takes its FIRST stack measurement, given the depth the generated parser
/// was built to ask at.
///
/// ⛔ **WHY THIS EXISTS: A FRAME COUNT AND A BYTE COUNT WERE FREE TO MOVE
/// INDEPENDENTLY, AND THE PROCESS DIED IN THE GAP.** A generated parser asks the
/// host stack a question at a compiled-in depth
/// (`GENERATED_RECURSION_GUARD_STACK_PROBE_FLOOR` — 4 096 frames, baked into all
/// eleven artifacts). Reaching that depth costs real stack, and how much is a
/// property of the family, the profile and the build mode. Nothing connected the
/// two except [`MIN_DEDICATED_PARSE_STACK_BYTES`]'s comment, which encoded ONE
/// per-frame cost measured ONCE, for ONE family, in 2026-07. The cost moved; the
/// constant did not; and at the documented floor a 642-byte SystemVerilog file
/// hard-aborted the process (`rc 134`, 0.5 s) because the stack ran out between
/// 3 550 and 4 034 frames while the guard's first question was at 4 096.
///
/// ⭐⭐ **THE FIX IS A PROPERTY, NOT A BIGGER NUMBER.** Per-frame stack cost does
/// not depend on the reservation, so
///
/// * the depth the reservation can really carry — `budget / bytes_per_frame`,
///   where `budget` is a fixed share of the reservation — is LINEAR in it, and
/// * the depth of the first sample, as computed here, is LINEAR in it too.
///
/// Their RATIO is therefore INVARIANT in the reservation. ⇒ **a first sample that
/// is reachable at the default reservation is reachable at every smaller one**,
/// for every family, every profile and both build modes, with no per-frame
/// constant anywhere in the derivation. That is the guarantee the byte floor was
/// asked for and could not give.
///
/// ⚠️ **WHAT IT DOES NOT CLAIM, stated rather than implied.** It makes the margin
/// INVARIANT; it does not measure the margin. At the default that margin is
/// `240 MiB ÷ (4 096 × 25 874 B)` = **2.37×**, measured on SystemVerilog in debug
/// and on no other family. A family whose debug frames cost more than **60 KiB**
/// (61 440 B, which is exactly `budget ÷ 4 096`) would reach its first sample too
/// late at EVERY reservation equally — which is
/// exactly the per-family, per-profile, per-build-mode census `.79`(a) owes, and
/// which this function deliberately does not pretend to substitute for.
///
/// ⛔ **IT ONLY EVER SCALES DOWN.** A reservation above the default returns the
/// requested depth unchanged: raising the first sample raises the depth a parse
/// descends before anything checks, which is the one direction that can make a
/// host less safe than it is today.
///
/// ⛔⛔ **AND IT APPLIES ONLY TO A THREAD THIS MODULE CREATED — a correction, and
/// the first cut got it wrong** (`-0144`). The law reads THIS THREAD's recorded
/// extent, not the process-global reservation, because
/// `PGEN_PARSE_STACK_BYTES` describes the threads this module creates and nothing
/// else. A parse on any other thread keeps the depth its parser asked for, which
/// is both the pre-`.79` behaviour and the only honest answer: the process cannot
/// know that thread's stack size ([`StackProbeRefusal::ExtentUnknown`]).
/// The first cut scaled by the global, and the regex family — which runs on its
/// own long-lived 64 MiB worker — measurably lost accept-set at a reduced
/// reservation because of it.
pub fn first_stack_sample_depth(requested_depth: usize) -> usize {
    // ⛔⛔ **THE RESERVATION ONLY DESCRIBES A THREAD THIS MODULE CREATED, AND THE
    // FIRST CUT OF THIS FUNCTION FORGOT THAT** (`.79`(b)(c) correction, `-0144`).
    // It read the PROCESS-GLOBAL `dedicated_parse_stack_bytes()`, so lowering
    // `PGEN_PARSE_STACK_BYTES` also lowered the hard ceiling kept by threads the
    // reservation has nothing to do with — measured, the regex family runs on its
    // own long-lived 64 MiB worker (`embedding_api::generated_regex_worker`) and
    // 400 nested groups went from ACCEPT to a 1 536-frame refusal at a 96 MiB
    // reservation, an accept-set narrowing on a shipping family.
    //
    // Reading THIS THREAD's recorded extent instead is both correct and stronger:
    // a thread whose reservation is unknown keeps the depth its parser asked for
    // — exactly the pre-`.79` behaviour — and a dedicated parse thread is scaled
    // against the reservation it was really created with rather than against a
    // process-wide guess.
    match current_parse_stack_extent() {
        Some((_, reservation)) => scale_sample_depth_to_reservation(requested_depth, reservation),
        None => requested_depth,
    }
}

/// The pure kernel of [`first_stack_sample_depth`].
///
/// Split out because the resolved reservation is a process-wide `OnceLock`: a test
/// that read it could pin the law at exactly one reservation and never at a second
/// one, which is the only interesting case. Taking the reservation as an argument
/// makes the law checkable at every value in one process.
fn scale_sample_depth_to_reservation(requested_depth: usize, reservation: usize) -> usize {
    if reservation >= DEDICATED_PARSE_STACK_BYTES {
        return requested_depth;
    }
    // 128-bit intermediate: `requested_depth` is a caller-supplied frame count and
    // nothing here bounds the product. The branch above makes the quotient
    // ≤ `requested_depth`, so the narrowing cast cannot truncate.
    let scaled =
        (requested_depth as u128 * reservation as u128) / DEDICATED_PARSE_STACK_BYTES as u128;
    // A guard whose sample depth is 0 refuses at depth 0 with
    // `StackProbeRefusal::FrameCostUnmeasurable` and can never advance, so the law
    // floors at one frame — but only for a caller that asked for at least one.
    // A caller asking for 0 is asking for no parsing at all and keeps that answer.
    (scaled as usize).max(requested_depth.min(1))
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

    /// ⛔ `ENGINE-UNIVERSAL-SERVICES.79` — THIS TEST'S OWN RATIONALE WAS THE
    /// REFUTED CLAIM. It read *"the resolved reservation must always leave room
    /// for the depth at which a generated parser takes its FIRST stack
    /// measurement; a smaller one could let the abort happen before the guard ever
    /// looks"* — and the floor it checked did NOT leave that room: at
    /// `MIN_DEDICATED_PARSE_STACK_BYTES` the abort happens between 3 550 and 4 034
    /// frames, below the 4 096-frame first sample. The assertion was true and the
    /// reason for it was false, which is the shape a green test cannot expose.
    ///
    /// What it locks now is the ordinary clamp — an operator's number is honoured
    /// only down to the usability floor. Reachability of the first sample is a
    /// different property, and it is locked by
    /// [`first_stack_sample_depth_is_reachable_at_every_reservation`] instead.
    #[test]
    fn resolved_reservation_is_never_below_the_usability_floor() {
        assert!(dedicated_parse_stack_bytes() >= MIN_DEDICATED_PARSE_STACK_BYTES);
    }

    /// `ENGINE-UNIVERSAL-SERVICES.79` — THE LAW ITSELF: the first sample scales
    /// with the reservation, and only ever DOWN.
    #[test]
    fn first_sample_depth_scales_with_the_reservation_and_never_upward() {
        const REQUESTED: usize = 4096;
        // The default is the anchor: the shipped path must be byte-identical to
        // the behaviour before this leaf, not merely close to it.
        assert_eq!(
            scale_sample_depth_to_reservation(REQUESTED, DEDICATED_PARSE_STACK_BYTES),
            REQUESTED,
            "at the default reservation the law must be the identity"
        );
        // Above the default it stays the identity — raising the first sample would
        // raise the hard ceiling an unknown-extent thread keeps.
        for bigger in [
            DEDICATED_PARSE_STACK_BYTES + 1,
            DEDICATED_PARSE_STACK_BYTES * 4,
            usize::MAX,
        ] {
            assert_eq!(
                scale_sample_depth_to_reservation(REQUESTED, bigger),
                REQUESTED,
                "the law must never scale the first sample UP (reservation {bigger})"
            );
        }
        // Below the default it is proportional. 96 MiB is 3/8 of 256 MiB, so the
        // 4 096-frame request becomes 1 536 — comfortably under the 3 550…4 034
        // frames that reservation was MEASURED to survive, which is the whole
        // point of the leaf.
        assert_eq!(
            scale_sample_depth_to_reservation(REQUESTED, MIN_DEDICATED_PARSE_STACK_BYTES),
            1536
        );
        assert_eq!(
            scale_sample_depth_to_reservation(REQUESTED, DEDICATED_PARSE_STACK_BYTES / 2),
            REQUESTED / 2
        );
        // Degenerate inputs stay sane: a request for one frame never rounds to a
        // guard that refuses at depth 0 and can never advance, and a request for
        // zero frames keeps its answer.
        assert_eq!(scale_sample_depth_to_reservation(1, 1), 1);
        assert_eq!(scale_sample_depth_to_reservation(0, 1), 0);
        assert_eq!(
            scale_sample_depth_to_reservation(usize::MAX, DEDICATED_PARSE_STACK_BYTES),
            usize::MAX,
            "the 128-bit intermediate must not overflow on an unbounded request"
        );
    }

    /// ⛔⛔ `ENGINE-UNIVERSAL-SERVICES.79` (`-0144`) — **A THREAD THIS MODULE DID NOT
    /// CREATE KEEPS THE DEPTH ITS PARSER ASKED FOR.** The first cut of the law scaled
    /// by the PROCESS-GLOBAL reservation, so `PGEN_PARSE_STACK_BYTES` reached across
    /// and lowered the hard ceiling of threads it does not describe. Measured on the
    /// shipping regex family, which parses on its own long-lived 64 MiB worker: 400
    /// nested groups ACCEPT at the 256 MiB default and were refused at **1 536
    /// frames** — the scaled sample depth — once the reservation was set to 96 MiB.
    /// An accept-set narrowing on a released parser, invisible at the default because
    /// the scale factor there is exactly 1, which is why every lock stayed green.
    ///
    /// This test runs on a libtest worker, which is precisely such a thread, so it is
    /// the RED control for that regression as well as the lock against it.
    #[test]
    fn a_thread_without_a_recorded_extent_keeps_the_requested_depth() {
        assert_eq!(
            current_parse_stack_extent(),
            None,
            "a libtest worker must have no recorded extent — the premise of this lock"
        );
        // Whatever the process resolved, an unknown-extent thread is unscaled.
        for requested in [1usize, 4096, 65_536] {
            assert_eq!(
                first_stack_sample_depth(requested),
                requested,
                "the reservation describes threads THIS module creates and nothing else"
            );
        }
        // And on a thread this module DID create, the law engages against that
        // thread's own recorded reservation.
        // `DedicatedParseStackError` carries a panic payload and deliberately does
        // not implement `Debug`, so this file's idiom is a let-else, not `.expect`.
        let Ok((extent, scaled)) =
            run_on_dedicated_parse_stack("pgen-test-sample-depth", || {
                let extent = current_parse_stack_extent();
                (extent, first_stack_sample_depth(4096))
            })
        else {
            panic!("dedicated parse-stack spawn failed");
        };
        let Some((_, reservation)) = extent else {
            panic!("a dedicated parse thread must record its extent");
        };
        assert_eq!(
            scaled,
            scale_sample_depth_to_reservation(4096, reservation),
            "a dedicated parse thread scales against ITS OWN reservation"
        );
    }

    /// ⭐⭐ `ENGINE-UNIVERSAL-SERVICES.79` — THE PROPERTY THE BYTE FLOOR COULD NOT
    /// HOLD: reaching the first sample fits inside the spendable budget at EVERY
    /// reservation, and the margin is the SAME at all of them.
    ///
    /// This is the assertion whose failure is the defect. It is written against a
    /// per-frame cost supplied as a parameter rather than pinned as a constant —
    /// the cost is a per-family, per-profile, per-build-mode measurement
    /// (`.79`(a)), so a test that pinned one would be the very literal this leaf
    /// deletes. What is checked is that the ANSWER does not depend on the
    /// reservation, for any cost at all.
    ///
    /// ⛔ RED CONTROL, run as part of the test: the same arithmetic against a FIXED
    /// first sample — the pre-`.79` behaviour — is asserted to FAIL at the floor
    /// on the measured debug cost. A property test that cannot go red is a census
    /// that cannot fail.
    #[test]
    fn first_stack_sample_depth_is_reachable_at_every_reservation() {
        const REQUESTED: usize = 4096;
        // ⛔ THE ENGINE'S OWN RESERVE DIVISOR, READ RATHER THAN RE-TYPED. `.78`
        // established that asserting `budget == reservation - reservation / 16`
        // puts a second copy of a private engine constant in the tests — the
        // defect this leaf's neighbours keep closing. The constant is
        // `pub(crate)` for exactly this line, so if the engine's reserve changes,
        // this law is re-checked against the new one instead of against a stale
        // copy of the old one.
        let budget = |reservation: usize| {
            reservation - reservation / crate::ast_pipeline::STACK_HEADROOM_RESERVE_DIVISOR
        };

        // Per-frame costs MEASURED by `.78`'s ladder and `.69`(d)'s probe, in
        // bytes: the two SystemVerilog profiles in debug and the release figure.
        for bytes_per_frame in [3_083usize, 25_335, 25_874, 29_031] {
            let mut margins = Vec::new();
            for reservation in [
                MIN_DEDICATED_PARSE_STACK_BYTES,
                MIN_DEDICATED_PARSE_STACK_BYTES * 2,
                DEDICATED_PARSE_STACK_BYTES / 2,
                DEDICATED_PARSE_STACK_BYTES,
            ] {
                let sample = scale_sample_depth_to_reservation(REQUESTED, reservation);
                let cost_to_reach = sample * bytes_per_frame;
                assert!(
                    cost_to_reach < budget(reservation),
                    "reaching the first sample ({sample} frames × {bytes_per_frame} B = \
                     {cost_to_reach} B) must fit inside the spendable budget \
                     ({} B) at a {reservation} B reservation — this is the SIGABRT \
                     ENGINE-UNIVERSAL-SERVICES.79 exists to remove",
                    budget(reservation)
                );
                // Parts-per-thousand so integer truncation in the scaling cannot
                // masquerade as a real drift in the margin.
                margins.push((budget(reservation) as u128 * 1000) / cost_to_reach as u128);
            }
            let first = margins[0];
            for margin in &margins {
                assert!(
                    first.abs_diff(*margin) <= 1,
                    "the safety margin must be INVARIANT in the reservation — that \
                     invariance is what makes one measurement at the default cover \
                     every smaller size. Got {margins:?} (‰) at {bytes_per_frame} B/frame"
                );
            }
        }

        // ⛔ RED CONTROL — the pre-`.79` law (a first sample FIXED at the requested
        // depth) really is unreachable at the floor on the measured debug cost.
        // Without this the test above would pass just as happily on a law that
        // never scales at all.
        assert!(
            REQUESTED * 25_874 > budget(MIN_DEDICATED_PARSE_STACK_BYTES),
            "the control must reproduce the defect: a fixed 4096-frame first \
             sample does NOT fit in the floor's budget at the measured debug cost"
        );
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
