//! `DATA-LOCALITY.2` — scratch paths that resolve inside the repository, derived at run time.
//!
//! The standing policy (project instructions §13, [[project_north_star]]): *"All data owned by the
//! project — including generated outputs, build artifacts, caches, logs, runtime-created test
//! fixtures, and temporary workspaces — must reside on the same filesystem volume as the repository
//! … tools must derive absolute paths at runtime from the current root and must never default to
//! /private/tmp, /tmp, user-home caches, or another off-volume location."*
//!
//! `std::env::temp_dir()` and `tempfile::tempdir()` both default to `$TMPDIR`, which on the
//! director's host is a different volume from the repository. This module is the one place that
//! answers *"where does project-owned scratch go"*, so the answer cannot drift one convenience call
//! at a time; the `DATA-LOCALITY` doctrine holds the live surface at zero calls to the defaults.
//!
//! ⚠️ **The root is the CRATE's manifest directory, resolved at compile time**, which is the same
//! definition the repository's other scratch helpers already use. That is correct for every caller
//! here — the parse harness compiles grammars with `cargo` and so already needs the checkout, and
//! the rest are tests — and it is deliberately NOT offered to a downstream consumer embedding the
//! library, whose scratch is its own business.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// `<repo>/rust/target` — the build tree, which is already `.gitignore`d and already on the
/// repository volume.
pub fn scratch_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("target")
}

/// A fresh, process-unique directory under [`scratch_root`], created.
///
/// Uniqueness without a clock or randomness (neither is deterministic here): the process id plus a
/// monotonic per-process counter — the same scheme the parse harness used against `$TMPDIR`, moved
/// onto the repository volume. ⛔ The counter matters: a dozen parallel `cargo test` threads calling
/// this inside one clock tick would otherwise share a directory and overwrite each other's fixtures
/// (observed once in `sv_preprocessor`'s own tests before its helper grew one).
pub fn repo_scratch_dir(prefix: &str) -> io::Result<PathBuf> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = scratch_root().join(format!("{}_{}_{}", prefix, std::process::id(), n));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scratch_is_inside_the_repository_and_never_the_system_temp_root() {
        let dir = repo_scratch_dir("repo_scratch_selftest").expect("create");
        assert!(dir.starts_with(scratch_root()), "{dir:?} is not under {:?}", scratch_root());
        assert!(dir.is_dir());
        // The property the policy is about: NOT the platform temp root.
        // ⛔ Read `TMPDIR` rather than calling `std::env::temp_dir()`. The `DATA-LOCALITY` doctrine
        // counts that call as a breach wherever it appears, and it was right to count this one: an
        // allowlist for "the module that defines the policy" is exactly the spelling-list mistake
        // the census's own three wrong cuts are pinned against. Reading the variable states the same
        // property — we are not under the platform temp root — without creating anything there.
        if let Some(system_temp) = std::env::var_os("TMPDIR") {
            let system_temp = PathBuf::from(system_temp);
            assert!(
                !dir.starts_with(&system_temp),
                "{dir:?} resolved inside the system temp root {system_temp:?}"
            );
        }
        // and distinct on every call, so parallel tests cannot collide
        let second = repo_scratch_dir("repo_scratch_selftest").expect("create");
        assert_ne!(dir, second);
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&second);
    }
}
