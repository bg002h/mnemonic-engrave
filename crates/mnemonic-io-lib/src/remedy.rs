//! Purge and remedy text — **`me`'s alone**.
//!
//! **`mt`'s purge text is NOT a source, and this is a disqualification on
//! evidence.** It advises zsh operators `history -d`, which does not delete on
//! zsh 5.9.2 (`-d` prints timestamps), and it tells fish operators to match on
//! the bearer material — typing the secret into history a second time, which is
//! the very thing a purge is for.
//!
//! ## F-264 — the recipe that reported success and purged nothing
//!
//! `me`'s own text was not clean either. It offered, for both bash and zsh:
//!
//! ```text
//! sed -i '/me sysw pack/d' "$HISTFILE"
//! ```
//!
//! The operator puts a secret on argv, `me` refuses and prints that, they run
//! it **immediately** as the message invites — and the shell is still holding
//! the entry **in memory**. `HISTFILE` does not contain it yet. `sed -i` edits
//! a file the secret is not in, **exits 0, prints nothing**, and at session
//! exit the shell writes its in-memory history, secret included, to disk.
//!
//! **That is the same defect as `history -d`, in the message that exists to
//! warn against `history -d`.**
//!
//! ### Measured, not reasoned — and the first proposed fix failed too
//!
//! Under stock zsh 5.9.2 and bash 5, on a real pty, with a control that plants
//! the entry and purges nothing (the secret must reach disk, or the harness is
//! measuring itself):
//!
//! | recipe | outcome |
//! | --- | --- |
//! | *(control — no purge)* | secret on disk |
//! | `sed -i …` alone — **what shipped** | secret on disk |
//! | `fc -W; sed -i …; fc -R` | **secret on disk** |
//! | `fc -W; sed -i …; HISTSIZE=0; HISTSIZE=$h; fc -R` | purged |
//!
//! **The three-step flush-edit-reload fix does not work on its own**, and it is
//! what F-264 and the plan both proposed. `fc -R` *appends* the file to the
//! in-memory list rather than replacing it, so the entry is still in memory and
//! is written back at exit. Zeroing `HISTSIZE` is what actually empties memory;
//! restoring it and re-reading rebuilds the history from the cleaned file, so
//! nothing else in the session is lost.
//!
//! **bash has the identical defect and needed the identical shape** — flush,
//! edit, **clear memory**, reload — which is why the two shells no longer share
//! one line. The shipped text said `bash/zsh:` and was wrong for both.

/// The purge recipes, one per shell, as `(shell, recipe)`.
///
/// Public and structured so a test can **run the emitted recipe** rather than a
/// copy of it. §6 condition 5 asks for a positive test — run it under a real
/// interactive shell and assert the entry is gone — and a test that runs its
/// own hard-coded string proves only that the string works.
///
/// `command` is matched on rather than the secret: quoting the secret into a
/// `sed` pattern is how an operator types it into history a second time.
///
/// **The pattern is word-bounded (`\b…\b`), and that is not decoration.** The
/// bare surface's command is just `me`, and `sed '/me/d'` deletes `make`,
/// `time`, `some`, `name` and `/home/…` — measured on a six-line sample, where
/// plain `/me/d` left ONE line of six standing. `\bme\b` left four, removing
/// only the invocation and `cd /home/me`. GNU `sed` is already assumed here:
/// `-i` without an argument is GNU-only.
pub fn history_purge_recipes(command: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "zsh",
            // fc -W       flush memory to the file, so the entry is IN it
            // sed -i      remove it from the file
            // HISTSIZE=0  empty the in-memory list -- `fc -R` alone APPENDS,
            //             so without this the entry survives in memory and the
            //             shell writes it back at exit (measured)
            // HISTSIZE=$h restore the operator's own size, not a guess
            // fc -R       rebuild memory from the cleaned file
            format!(
                "fc -W; sed -i '/\\b{command}\\b/d' \"$HISTFILE\"; \
                 h=$HISTSIZE; HISTSIZE=0; HISTSIZE=$h; fc -R"
            ),
        ),
        (
            "bash",
            format!(
                "history -w; sed -i '/\\b{command}\\b/d' \"$HISTFILE\"; \
                 history -c; history -r"
            ),
        ),
    ]
}

/// The purge paragraph as it is printed, indented to sit inside a refusal.
///
/// **`history -d` is NAMED here and never OFFERED**, and the distinction is the
/// gate. The donor's own test file records the trap:
///
/// > *"NOT `!err.contains("history -d")` — the message deliberately NAMES that
/// > command in order to warn against it, so the naive negative fails on the
/// > warning itself. The requirement is that it is never OFFERED."*
///
/// So the recipes are structured data and the warning is prose, and the test
/// asserts `history -d` appears in **no recipe** while still appearing in the
/// text. A gate written as "does not contain the string" goes RED against the
/// correct text and can only be made green by deleting the warning — recreating
/// the exact defect that disqualifies `mt`'s wording.
///
/// **fish is described, not prescribed.** `history delete --prefix` PROMPTS:
/// measured here, it blocked for a full two minutes on a planted history and
/// deleted nothing, and the prompt lists the matching commands — the secret
/// with them. A recipe that re-displays the secret and purges nothing
/// unattended is not offered as one; what it does is stated so an operator at
/// a fish prompt is not left to discover it.
pub fn history_purge_block(command: &str) -> String {
    let mut s = String::new();
    s.push_str(
        "TO PURGE WHAT ALREADY LEAKED -- match on the COMMAND, never on the \
         secret, or you type it into history a second time. Run ALL of the \
         steps: your shell is still holding that entry in MEMORY, so editing \
         the history FILE alone changes nothing and the entry is written back \
         when you exit.\n",
    );
    for (shell, recipe) in history_purge_recipes(command) {
        s.push_str(&format!("      \x20   {:<7} {recipe}\n", format!("{shell}:")));
    }
    s.push_str(&format!(
        "      \x20   {:<7} history delete --prefix '{command}'  -- but it \
         PROMPTS, lists the matches with the secret in them, and purges \
         nothing unattended.\n",
        "fish:"
    ));
    s.push_str("      \x20   and `shred -u` any file you pasted it from.\n");
    s.push_str(
        "      (On zsh, `history -d` does NOT delete -- -d prints timestamps. \
         It would report success and purge nothing. Editing the file on its \
         own is the SAME trap, measured on stock zsh 5.9.2 and bash 5: the \
         entry is only in memory, sed exits 0 having changed a file that never \
         held it, and the shell saves it at exit anyway.)",
    );
    s
}
