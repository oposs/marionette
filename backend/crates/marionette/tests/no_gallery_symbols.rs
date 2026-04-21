//! FRAME-03 — Default `cargo build -p marionette` compiles zero demo symbols.
//!
//! Each subtest shells out to `cargo build` with its own `--target-dir` so the
//! two builds never clobber each other's rlib. Then inspects the resulting
//! `libmarionette.rlib` with `nm` and asserts presence/absence of the `DEMOS`
//! distributed slice symbols emitted by `marionette::gallery`.
//!
//! Under default build the slice is `#[cfg(feature = "gallery")]`-gated and
//! MUST NOT appear in the rlib; under `--features gallery` the slice symbols
//! + linkme section markers MUST appear.
//!
//! Per RESEARCH §4 / RESOLVED Q4 in `.planning/phases/16-framework-hooks/16-RESEARCH.md`.

#![allow(clippy::needless_return)]

use std::path::PathBuf;
use std::process::Command;

/// Runs `cargo build -p marionette [extra_args...]` into an isolated `--target-dir`,
/// then returns the `nm --demangle` output of the resulting `libmarionette.rlib`.
fn build_and_nm(target_subdir: &str, extra_args: &[&str]) -> String {
    // CARGO_MANIFEST_DIR == backend/crates/marionette; workspace root is two levels up.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_backend = manifest_dir
        .parent()
        .expect("crates/")
        .parent()
        .expect("backend/")
        .to_path_buf();
    let target_dir = workspace_backend.join("target").join(target_subdir);

    let mut build = Command::new("cargo");
    build
        .current_dir(&workspace_backend)
        .args(["build", "-p", "marionette", "--target-dir"])
        .arg(&target_dir);
    build.args(extra_args);
    let out = build.output().expect("cargo build spawn");
    assert!(
        out.status.success(),
        "cargo build failed (args: {extra_args:?}):\n--- stdout ---\n{}\n--- stderr ---\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    // The rlib lives at <target_dir>/debug/libmarionette.rlib
    let rlib = target_dir.join("debug").join("libmarionette.rlib");
    assert!(
        rlib.exists(),
        "expected rlib not found at {}; target_dir contents: {:?}",
        rlib.display(),
        std::fs::read_dir(target_dir.join("debug"))
            .ok()
            .map(|it| it.filter_map(Result::ok).map(|e| e.file_name()).collect::<Vec<_>>())
    );

    let nm = Command::new("nm")
        .args(["--demangle"])
        .arg(&rlib)
        .output()
        .expect("nm spawn");
    assert!(
        nm.status.success(),
        "nm failed:\n{}",
        String::from_utf8_lossy(&nm.stderr)
    );
    String::from_utf8_lossy(&nm.stdout).into_owned()
}

/// Returns lines that contain evidence of registered demo symbols.
///
/// FRAME-03 requires zero *registered demo entries* and zero *slice storage*
/// under default build — not zero public-API surface. Per D-B4, the
/// `registered_demos()` iteration fn is always compiled (returns an empty
/// iterator under default) so downstream consumers never need `#[cfg]` guards
/// of their own; it is therefore NOT a demo-code symbol and is excluded here.
///
/// The true markers of demo registration:
/// - `gallery::DEMOS` — the `#[linkme::distributed_slice] pub static DEMOS`,
///   which only exists under `--features gallery`
/// - `__GALLERY_DEMO_` — the per-entry `static` ident the proc macro assigns
///   to every `#[gallery_demo]`-annotated call site
fn demo_symbol_matches(nm_output: &str) -> Vec<&str> {
    nm_output
        .lines()
        .filter(|l| l.contains("gallery::DEMOS") || l.contains("__GALLERY_DEMO_"))
        .collect()
}

#[test]
fn default_build_has_zero_demo_symbols() {
    let nm = build_and_nm("no-gallery-symbols-test-default", &[]);
    let matches = demo_symbol_matches(&nm);
    assert!(
        matches.is_empty(),
        "FRAME-03 violation: default `cargo build -p marionette` leaked demo symbols into \
         libmarionette.rlib. This means the `gallery` feature gate is incomplete — either \
         `DEMOS` is not `#[cfg(feature = \"gallery\")]`-gated, or `#[gallery_demo]` call \
         sites are emitting ungated statics.\n\nLeaked symbols:\n{}",
        matches.join("\n")
    );
}

#[test]
fn gallery_feature_build_has_demo_symbols() {
    let nm = build_and_nm(
        "no-gallery-symbols-test-gallery",
        &["--features", "gallery"],
    );
    let matches = demo_symbol_matches(&nm);
    assert!(
        !matches.is_empty(),
        "expected `DEMOS`/linkme symbols under `--features gallery` but found none. \
         The feature gate may be mis-wired or the rlib inspection path is stale.\n\n\
         First 20 lines of nm output:\n{}",
        nm.lines().take(20).collect::<Vec<_>>().join("\n")
    );
}
