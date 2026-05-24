//! SV-EXH-PROOF.3.3.4.b.6.2.22 — live per-rule call-counter dashboard.
//!
//! User-proposed (2026-05-24) to identify rules that dominate a stuck or
//! slow parse without the volume penalty of `--trace`. Each rule's entry
//! increments a per-cell `AtomicU64` (cost: ~1ns, lock-free); this module
//! reads the snapshot every ~250ms and rewrites the same N+1 lines in
//! place using ANSI cursor-up so the display "sticks" — only the counts
//! refresh.
//!
//! Parser-agnostic: every generated parser exposes a
//! `rule_call_counts()` accessor and a `rule_names()` static; this
//! module is grammar-unaware.

use std::collections::HashSet;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

/// One running dashboard. Drop it (or call `shutdown()`) to stop the
/// background thread and reclaim the terminal. Drop is safe on success
/// AND error paths; the thread's tear-down is idempotent.
pub struct CallCountDashboard {
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    top_n: usize,
}

impl CallCountDashboard {
    /// Spawn the dashboard. `counts` is the parser's Arc<Vec<AtomicU64>>
    /// from `<Parser>::rule_call_counts()`; `names` is `<Parser>::rule_names()`.
    /// Both must have the same length (validated by debug_assert).
    ///
    /// `exclude` is a set of rule names to filter OUT of the displayed
    /// top-N — user-set via `--dump-rule-call-counts-exclude r1,r2,...`.
    /// Useful for hiding always-dominant rules like `trivia` (whitespace
    /// handling) that overshadow the diagnostically interesting rules.
    /// Pass `HashSet::new()` for no filtering.
    pub fn spawn(
        counts: Arc<Vec<AtomicU64>>,
        names: &'static [&'static str],
        exclude: HashSet<String>,
        top_n: usize,
        refresh_ms: u64,
    ) -> Self {
        debug_assert_eq!(
            counts.len(),
            names.len(),
            "rule_call_counts() and rule_names() must align by rule_id",
        );

        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();

        // Print the initial header + N blank rule lines so the redraw
        // loop has known content to overwrite. `\x1b[?25l` hides the
        // cursor for the duration; we restore it on shutdown.
        let mut err = std::io::stderr().lock();
        let _ = writeln!(err, "\x1b[?25l=== Rule call counts (live, top {}) ===", top_n);
        for _ in 0..top_n {
            let _ = writeln!(err, "");
        }
        let _ = err.flush();
        drop(err);

        let handle = std::thread::Builder::new()
            .name("pgen-call-count-dashboard".to_string())
            .spawn(move || run_dashboard_loop(counts, names, exclude, shutdown_clone, top_n, refresh_ms))
            .expect("dashboard thread spawn failed");

        Self {
            shutdown,
            handle: Some(handle),
            top_n,
        }
    }

    /// Stop the dashboard thread and restore cursor visibility. Called
    /// automatically on Drop; explicit call lets the caller observe any
    /// thread panic (currently unused — the loop has no panic paths).
    pub fn shutdown(mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        restore_cursor();
    }
}

impl Drop for CallCountDashboard {
    fn drop(&mut self) {
        // Idempotent: shutdown() takes self by value so this only fires
        // on the implicit-drop path (panic, early return without
        // explicit shutdown).
        if self.handle.is_some() {
            self.shutdown.store(true, Ordering::Relaxed);
            if let Some(h) = self.handle.take() {
                let _ = h.join();
            }
            restore_cursor();
        }
    }
}

fn restore_cursor() {
    let mut err = std::io::stderr().lock();
    // Show cursor again (`?25h`) and move to a clean line below the
    // dashboard so subsequent stderr output doesn't overlap the table.
    let _ = writeln!(err, "\x1b[?25h");
    let _ = err.flush();
}

fn run_dashboard_loop(
    counts: Arc<Vec<AtomicU64>>,
    names: &'static [&'static str],
    exclude: HashSet<String>,
    shutdown: Arc<AtomicBool>,
    top_n: usize,
    refresh_ms: u64,
) {
    let interval = Duration::from_millis(refresh_ms);
    let len = counts.len().min(names.len());

    loop {
        // Snapshot all non-zero counters EXCLUDING user-suppressed
        // rules; cheap Relaxed loads. A torn read is acceptable for
        // diagnostic display (off by at most a handful — they'll be
        // correct on the next 250ms tick). Exclusion filter applied
        // BEFORE sort so the top-N reflects post-filter ranking
        // (otherwise excluded `trivia` would steal slot 1 and the
        // user-visible top-N would only have N-1 useful rows).
        let mut snapshot: Vec<(usize, u64)> = (0..len)
            .filter(|i| !exclude.contains(names[*i]))
            .map(|i| (i, counts[i].load(Ordering::Relaxed)))
            .filter(|(_, c)| *c > 0)
            .collect();
        snapshot.sort_by_key(|(_, c)| std::cmp::Reverse(*c));

        // Move cursor up (top_n + 1) lines to overwrite the header +
        // every rule line. `\x1b[2K` clears the line before each
        // rewrite so a shorter line doesn't leave trailing chars.
        let mut err = std::io::stderr().lock();
        let _ = write!(err, "\x1b[{}A", top_n + 1);
        let _ = writeln!(
            err,
            "\x1b[2K=== Rule call counts (live, top {}) ===",
            top_n
        );
        for slot in 0..top_n {
            if let Some((rule_id, count)) = snapshot.get(slot) {
                let _ = writeln!(
                    err,
                    "\x1b[2K  {:50} {:>16}",
                    names[*rule_id],
                    format_thousands(*count)
                );
            } else {
                // Empty slot: clear the line so old content doesn't
                // linger. This is NOT rare — counters are monotone but
                // their RANK is dynamic. Early in a parse, structural
                // rules like white_space/identifier dominate; mid-parse
                // may shift to expression/statement; late may shift to
                // type-resolution rules. A previously-top rule can drop
                // out of the top-N as the parse moves through different
                // grammar regions. Padding keeps the table layout stable.
                let _ = writeln!(err, "\x1b[2K");
            }
        }
        let _ = err.flush();
        drop(err);

        if shutdown.load(Ordering::Relaxed) {
            break;
        }
        std::thread::sleep(interval);
    }
}

/// Format a u64 with thousands separators ("42,891,234"). Simple loop;
/// avoids the formatter crate dependency.
fn format_thousands(n: u64) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() + bytes.len() / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            out.push(b',');
        }
        out.push(*b);
    }
    String::from_utf8(out).expect("ascii in, ascii out")
}

#[cfg(test)]
mod tests {
    use super::format_thousands;

    #[test]
    fn format_thousands_examples() {
        assert_eq!(format_thousands(0), "0");
        assert_eq!(format_thousands(42), "42");
        assert_eq!(format_thousands(999), "999");
        assert_eq!(format_thousands(1_000), "1,000");
        assert_eq!(format_thousands(42_891_234), "42,891,234");
        assert_eq!(format_thousands(u64::MAX), "18,446,744,073,709,551,615");
    }
}
