//! F-264 — **the purge recipe must actually purge.**
//!
//! `me` refuses a secret on argv and prints a recipe for removing it from shell
//! history. The recipe it shipped was `sed -i '/me sysw pack/d' "$HISTFILE"`,
//! and under stock zsh it removes **nothing**: the shell is still holding the
//! entry in memory, `HISTFILE` does not contain it yet, `sed` exits 0 having
//! changed a file the secret was never in, and at session exit the shell writes
//! its in-memory history — secret included — to disk.
//!
//! **The operator is told to purge, does exactly as told, sees success, and the
//! secret lands on disk anyway.** That is the same class of defect as
//! `history -d`, which this very message exists to warn against.
//!
//! §6 condition 5 asks for a POSITIVE test: *run the emitted recipe under an
//! interactive shell and assert the entry is gone*, not that a command was
//! printed. So this file:
//!
//! 1. takes the recipe from `io::remedy`, **the same call the binary makes**,
//!    and asserts the binary's stderr really carries that byte string — a test
//!    that runs its own hard-coded copy proves only that the copy works;
//! 2. runs it inside a real interactive zsh on a pty;
//! 3. carries a **control** that plants the entry and purges nothing, because
//!    a harness that records no history at all reports "purged" for everything.
//!    That control caught a broken first draft of this very file.
#![cfg(unix)]

use mnemonic_engrave::io::remedy;
use std::process::Command;

const SECRET: &str = "ms1SECRETSECRETPLANTED";
const COMMAND: &str = "me sysw pack";

fn zsh_bin() -> String {
    let p = "/usr/bin/zsh";
    assert!(
        std::path::Path::new(p).exists(),
        "{p} is required: F-264's gate is 'the emitted recipe, RUN under a real interactive \
         zsh, actually removes the entry', and there is no way to run it without zsh. This \
         is deliberately a FAILURE and not a skip -- a skipped gate prints ok and exit 0. \
         If CI lacks zsh, install it there rather than weakening this."
    );
    p.to_string()
}

/// Plant `COMMAND SECRET` in an interactive zsh's history, then run `recipe`,
/// then let the shell exit. Returns the contents of `HISTFILE` afterwards.
///
/// The shell EXITS before we look, because that is when the defect lands: the
/// in-memory history is written to disk at exit, so a check made while the
/// shell is still alive would see a clean file and call the bug fixed.
fn zsh_history_after(recipe: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    std::fs::write(
        d.join(".zshrc"),
        "HISTFILE=$ZDOTDIR/histfile\nHISTSIZE=1000\nSAVEHIST=1000\n",
    )
    .unwrap();
    std::fs::write(d.join("histfile"), "").unwrap();
    let input = d.join("in.zsh");
    std::fs::write(&input, format!("{COMMAND} {SECRET}\n{recipe}\n")).unwrap();

    let st = Command::new("script")
        .arg("-qec")
        .arg(format!("{} -i -s < '{}'", zsh_bin(), input.display()))
        .arg("/dev/null")
        .env("ZDOTDIR", d)
        .env("HOME", d)
        .output()
        .expect("`script` (util-linux) is required to give zsh a pty");
    assert!(
        st.status.code().is_some(),
        "the zsh session was killed rather than exiting; nothing can be concluded"
    );
    std::fs::read_to_string(d.join("histfile")).unwrap()
}

/// **THE CONTROL, and it runs first for a reason.** A harness that fails to
/// record history at all reports "purged" for every recipe including the broken
/// one. The first draft of this file did exactly that — `.zshrc` was misnamed,
/// zsh recorded nothing, and both the shipped recipe and the fix "passed".
#[test]
fn the_harness_records_history_at_all() {
    let h = zsh_history_after("true nothing-was-purged-here");
    assert!(
        h.contains(SECRET),
        "with NO purge attempt the planted secret must reach disk, or this file is \
         measuring itself rather than the recipe. HISTFILE was:\n{h}"
    );
}

/// **F-264's own reproduction, kept as a test.** Editing the history FILE while
/// the entry is still in MEMORY changes nothing — and reports success.
#[test]
fn editing_the_file_alone_is_the_trap_the_message_warns_about() {
    let h = zsh_history_after(&format!("sed -i '/{COMMAND}/d' \"$HISTFILE\""));
    assert!(
        h.contains(SECRET),
        "if this ever stops holding, zsh's save semantics changed and the recipe's \
         extra steps may no longer be needed -- re-measure before simplifying. \
         HISTFILE was:\n{h}"
    );
}

/// **THE GATE.** The recipe `me` actually emits, run under a real interactive
/// zsh, removes the entry.
#[test]
fn the_emitted_zsh_recipe_actually_purges_the_entry() {
    let recipes = remedy::history_purge_recipes(COMMAND);
    let (_, zsh) = recipes
        .iter()
        .find(|(s, _)| *s == "zsh")
        .expect("a zsh recipe must exist");

    // It is the EMITTED one, not a copy: the binary's refusal carries this
    // exact byte string.
    // The repo's own ms1 fixture, as `tests/cli.rs` uses it.
    const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
    let out = Command::new(assert_cmd::cargo::cargo_bin("me"))
        .args(["sysw", "pack", MS1])
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains(zsh.as_str()),
        "the test must run the recipe the operator is given, byte for byte.\n\
         expected to find:\n{zsh}\nin:\n{err}"
    );

    let h = zsh_history_after(zsh);
    assert!(
        !h.contains(SECRET),
        "the emitted recipe reported success and purged nothing -- F-264. \
         HISTFILE after the session exited was:\n{h}"
    );
}

/// **`history -d` is NAMED, and never OFFERED.**
///
/// The donor's own test file records why the obvious assertion is wrong:
///
/// > *"NOT `!err.contains("history -d")` — the message deliberately NAMES that
/// > command in order to warn against it, so the naive negative fails on the
/// > warning itself. The requirement is that it is never OFFERED."*
///
/// A gate written as "does not contain the string" goes RED against the CORRECT
/// text, and the only way to make it green is to delete the warning — recreating
/// the exact defect that disqualifies `mt`'s wording. So the two halves are
/// asserted separately, against structure rather than against prose.
#[test]
fn history_d_is_named_as_a_warning_and_offered_as_no_recipe() {
    let block = remedy::history_purge_block(COMMAND);
    assert!(
        block.contains("history -d"),
        "it must still be NAMED -- an operator who knows the command needs to be told \
         it does not work: {block}"
    );
    for (shell, recipe) in remedy::history_purge_recipes(COMMAND) {
        assert!(
            !recipe.contains("history -d"),
            "{shell}'s recipe OFFERS `history -d`, which on zsh prints timestamps and \
             deletes nothing: {recipe}"
        );
    }
}
