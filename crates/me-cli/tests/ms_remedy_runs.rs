//! **P2 row 11 — `me`'s remedy for a secret-class operator, RUN.**
//!
//! §6h: remedy text names channels that exist and pipelines that RUN, and the
//! rule was earned by shipping the opposite once. It was shipped a second time.
//! The line `me` printed —
//!
//! ```text
//!     ms encode --phrase - < seed.txt | me sysw pack --out p.bin
//! ```
//!
//! — carried a source comment four lines above asserting it *"is verified to
//! pipe into pack"*. **Run verbatim it exited 4** with `me: record 0 (records
//! count from 0) is not a form this container can place`, and wrote no payload,
//! because `ms encode`'s stdout was grouped by default and `me sysw pack` cannot
//! classify a grouped `ms1`. Nothing verified it: before this file,
//! `crates/me-cli/tests/` held **14** `.rs` files with **33** `Command::new`
//! sites and **zero** naming an `ms` binary. F-301.
//!
//! P2 made `ms encode`'s stdout the canonical ungrouped `ms1` and gave it
//! `--in FILE`, so the advice becomes the `--in` form and the pipeline runs.
//! **This file BUILDS the assertion rather than retargeting one** — there was
//! no assertion to retarget.
//!
//! ## The cross-repo precondition is part of the gate, not an assumption
//!
//! The pipeline needs an `ms` carrying `--in` and an ungrouped stdout, which
//! exists on no published `mnemonic-secret`. So the test locates its `ms` by
//! `MS_P2_BIN` and **SKIPS EXPLICITLY, naming the reason, when it is unset** —
//! never silently. P2 does not close until it has been run with it set.
//!
//! ## One deviation from the plan's wording, and why
//!
//! Row 11 says the test *"extracts the advised line from `me`'s own stderr"*.
//! **That branch is currently unreachable**, measured 2026-08-27 over eleven
//! argv shapes: the pre-parser `argv_secret_guard` refuses every input for which
//! `class.is_argv_forbidden()` holds, at exit 3, before `read_records` runs, so
//! the secret-class `else` arm can only be selected by an input the pre-parser
//! already rejected. The reachable half always takes the BEARER example. So the
//! line is taken from the `pub const` the binary itself formats into the
//! message — the same bytes, one hop earlier — and the unreachability is
//! asserted here so the day it changes, this test says so. F-362.

use std::process::Command;

/// The exact bytes `me` puts in the message, trimmed of its display indent.
fn advised_pipeline() -> String {
    mnemonic_engrave_advice().trim().to_string()
}

fn mnemonic_engrave_advice() -> &'static str {
    // The binary's own constant, reached through the LIB target -- which is why
    // it lives there. A test that ran its own copy of the string would prove
    // only that the copy works.
    mnemonic_engrave::sysw::advice::SECRET_PRIVATE_CHANNEL_EXAMPLE
}

/// **THE GATE.** The advised pipeline, run, exits 0 and leaves a payload.
#[test]
fn the_advised_pipeline_runs_and_writes_a_payload() {
    let Some(ms) = std::env::var_os("MS_P2_BIN") else {
        eprintln!(
            "SKIPPED, and naming the reason rather than passing silently: this test \
             runs `{}`, which needs an `ms` carrying P2's `--in` and its ungrouped \
             stdout. No published mnemonic-secret has one. Set MS_P2_BIN to such a \
             binary. P2 does not close until this has been run with it set.",
            advised_pipeline()
        );
        return;
    };
    let ms = std::path::PathBuf::from(ms);
    assert!(
        ms.is_file(),
        "MS_P2_BIN is set to {ms:?}, which is not a file"
    );

    let dir = tempfile::tempdir().unwrap();
    let bin = dir.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    std::os::unix::fs::symlink(std::fs::canonicalize(&ms).unwrap(), bin.join("ms")).unwrap();
    std::os::unix::fs::symlink(assert_cmd::cargo::cargo_bin("me"), bin.join("me")).unwrap();

    let work = dir.path().join("work");
    std::fs::create_dir(&work).unwrap();
    std::fs::write(
        work.join("seed.txt"),
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon \
         abandon abandon about\n",
    )
    .unwrap();

    let line = advised_pipeline();
    // Run it VERBATIM. Reading it is the mistake §6h records was made once
    // already, and made a second time in the text this replaces.
    let out = Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("umask 077; {line}"))
        .current_dir(&work)
        .env(
            "PATH",
            format!(
                "{}:{}",
                bin.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .output()
        .expect("sh");

    assert_eq!(
        out.status.code(),
        Some(0),
        "the line `me` advises must RUN.\ncommand: {line}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let payload = work.join("p.bin");
    assert!(
        payload.is_file(),
        "the advised pipeline names `--out p.bin`; it must exist afterwards. \
         stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        std::fs::metadata(&payload).unwrap().len() > 0,
        "a zero-byte payload is not a payload"
    );
}

/// **The control: `me` must never advise a channel `ms` lacks.** The advised
/// command's own `--help` has to declare every flag the line uses.
#[test]
fn every_flag_the_advice_names_exists_on_ms() {
    let Some(ms) = std::env::var_os("MS_P2_BIN") else {
        eprintln!(
            "SKIPPED, naming the reason: needs MS_P2_BIN. P2 does not close until \
             this has been run with it set."
        );
        return;
    };
    let help = Command::new(&ms)
        .args(["encode", "--help"])
        .output()
        .expect("ms encode --help");
    let help = String::from_utf8_lossy(&help.stdout);
    let line = advised_pipeline();
    for flag in line.split_whitespace().filter(|t| t.starts_with("--")) {
        // Only the flags on the `ms encode` side of the pipe.
        if line.split('|').next().unwrap().contains(flag) {
            assert!(
                help.contains(flag),
                "`me` advises `{flag}`, and `ms encode --help` does not declare it. \
                 Advice for a flag that is not there is worse than no advice."
            );
        }
    }
}

/// **The unreachability, pinned.** Row 11 asks for the advised line to be
/// extracted from `me`'s own stderr. It cannot be, because the branch that
/// prints it is dead behind the pre-parser guard — and this test is what will
/// say so the day that changes.
#[test]
fn the_secret_class_advice_branch_is_still_unreachable() {
    let shapes: Vec<&str> = vec![
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
        " MS10ENTRSQQQQQQQQQQQQQQQQQQQQQQQQQQQQCJ9SXRAQ34V7F ",
        "pass:hunter2 correct horse",
        "text:hello",
        "tx:0200000001",
        " TX:0200000001",
        "tx:zzznothex",
    ];
    let mut sightings = Vec::new();
    for shape in &shapes {
        let out = Command::new(assert_cmd::cargo::cargo_bin("me"))
            .args(["sysw", "pack", shape])
            .output()
            .unwrap();
        let err = String::from_utf8_lossy(&out.stderr);
        if err.contains("ms encode --in") {
            sightings.push(*shape);
        }
    }
    assert!(
        sightings.is_empty(),
        "THE BRANCH IS REACHABLE NOW, via {sightings:?}. That is good news and this \
         test is the notice: switch `advised_pipeline()` to extract the line from \
         `me`'s stderr, which is what P2's row 11 asked for and what could not be \
         done while the pre-parser guard refused every input first. F-362."
    );
}
