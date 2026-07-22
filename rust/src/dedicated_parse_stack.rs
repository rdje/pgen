//! Dedicated big-stack execution for generated-parser entry boundaries.
//!
//! `SV-CORPUS-GRAD.8c.3` — the recursion ceiling must bound the REAL stack
//! (the PGEN-RGX-0085 law, generalized parser-agnostically). The generated
//! parsers carry a clean mutual-recursion ceiling
//! (`GENERATED_RECURSION_GUARD_MAX_DEPTH = 4096`,
//! `ast_pipeline/ast_based_generator.rs`), but a ceiling only protects the
//! process if the thread it runs on has enough stack for 4096 frames.
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

thread_local! {
    // True on threads created by this module (and only those), so nested
    // boundary crossings run inline instead of spawning again.
    static ON_DEDICATED_PARSE_STACK: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
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

/// Runs `f` with [`DEDICATED_PARSE_STACK_BYTES`] of stack guaranteed.
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
        .stack_size(DEDICATED_PARSE_STACK_BYTES)
        .spawn(move || {
            ON_DEDICATED_PARSE_STACK.with(|c| c.set(true));
            f()
        })
        .map_err(DedicatedParseStackError::Spawn)?;
    handle.join().map_err(DedicatedParseStackError::Panic)
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
