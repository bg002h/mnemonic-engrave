//! **F-266 / §6d — no secret material reaches stderr, on any argv that carries
//! it as a token.**
//!
//! `me` echoed a real `ms1` verbatim to stderr on 15 of 24 sampled
//! surface × shape combinations, because clap names the offending VALUE in its
//! error and nothing ran before `Cli::parse()`. `mt`'s source records the same
//! lesson from the other side: when its check lived inside the `encode`
//! subcommand, clap rejected the unexpected positional first **and echoed the
//! entire bearer transaction**. A guard downstream of the parser has already
//! lost.
//!
//! **The gate is a GENERATED CROSS-PRODUCT, not a hand list, on every axis.**
//! Two earlier drafts of this work enumerated first surfaces and then shapes,
//! and both came up short — the shape list missed `Class::Passphrase`, the
//! `pass:` record, which `me` refuses at rc 3 as *SECRET key material on ARGV*
//! and which leaked bare. A list you write by hand is a list you can be short
//! by one, and a security gate short by one class is not a gate.
//!
//! **The surface axis here is LONGER than the plan's, and deliberately.** The
//! plan enumerates `{bare, bundle, sysw, sysw pack, sysw show, sysw wipe, help,
//! sysw help}` — but `me` has FIVE top-level subcommands, and `seal` and `hash`
//! appear nowhere in that list. Both are covered here. `seal` is the reason it
//! matters: measured against the pre-guard binary,
//! `me seal --in <ms1>` printed
//! `me: cannot read ms10entrs…: No such file or directory` — the leak, on the
//! surface the plan forgot.
#![cfg(unix)]

use std::process::{Command, Output};

fn me() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("me")
}

fn run(args: &[&str]) -> Output {
    Command::new(me())
        .args(args)
        .output()
        .expect("me should run")
}

fn stderr_of(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// The refusal is recognisable by its own words, never by a bare exit code.
fn is_guard_refusal(o: &Output) -> bool {
    o.status.code() == Some(3)
        && stderr_of(o).contains("Refused BEFORE the command line was parsed")
}

const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon \
                        abandon abandon about";
const PASS: &str = "pass:6869";
const MT1: &str =
    "mt1p9h8jqq9qqqqgqqqqqqqyqherdfykhhpey6z2cvafak8804qd7g0dl6v8ex9wr2cvky023skwkeud2229sax";
/// The repo's own signed-transaction fixture, as `tests/sysw_cli.rs` carries it.
const TX: &str = "tx:020000000001017c8da925af70e49a12b0cea7b639df5037c87b7fa61f262b86ac32c47aa3ba1a0000000000fdffffff02404b4c0000000000160014c1de0dd435d1d4ad97ed1f51d63f91c800cc4eab3ea1b92901000000160014751097c299d6354fbb2c5a84512dd708f2902f5e0247304402207debc7d89984c7717940b622504318d2c184966a618b32cf8b700d0f125b3ffa02206ef875f9c0b5931e0ea1cf0c109bdb8512835c8e51526f99b3419929a2ea7259012103718f5fd45b926226357e2b0400574b41a32d0bf0ae69a02eebea5fbc542ff52060000000";

/// **THE GATE.** Every surface × every argv shape × every argv-forbidden class
/// × every near-miss spelling: the material never reaches stderr.
///
/// The near-miss spellings are the leading-space and UPPERCASE forms.
/// `classify()` neither trims nor case-folds, so ` TX:<hex>` and `MS1…` come
/// back `Unknown` and leaked; a guard that skips trim+lowercase goes RED on
/// those rows while every canonical row stays green. That is why they are an
/// axis and not an afterthought.
#[test]
fn no_argv_forbidden_class_reaches_stderr_on_any_surface() {
    let surfaces: [&[&str]; 10] = [
        &[],
        &["bundle"],
        &["sysw"],
        &["sysw", "pack"],
        &["sysw", "show"],
        &["sysw", "wipe"],
        &["help"],
        &["sysw", "help"],
        // NOT in the plan's list. `seal` is where the omission bites.
        &["seal"],
        &["hash"],
    ];
    let classes = [
        ("Codex32Secret", MS1),
        ("Mnemonic", MNEMONIC),
        ("Passphrase", PASS),
        ("Mt", MT1),
        ("Tx", TX),
    ];

    let mut rows = 0usize;
    let mut leaks = Vec::new();
    for surface in surfaces {
        for (class, canonical) in classes {
            for (spelling, carrier) in [
                ("canonical", canonical.to_string()),
                ("leading space", format!(" {canonical}")),
                ("UPPERCASE", canonical.to_uppercase()),
            ] {
                for (shape, argv) in [
                    ("positional", vec![carrier.clone()]),
                    ("--in X", vec!["--in".to_string(), carrier.clone()]),
                    ("--in=X", vec![format!("--in={carrier}")]),
                ] {
                    let mut args: Vec<&str> = surface.to_vec();
                    args.extend(argv.iter().map(String::as_str));
                    let err = stderr_of(&run(&args));
                    rows += 1;
                    let body = carrier.trim();
                    if !body.is_empty() && err.contains(body) {
                        leaks.push(format!(
                            "me {} / {shape} / {class} / {spelling}",
                            surface.join(" ")
                        ));
                    }
                }
            }
        }
    }

    assert_eq!(
        rows, 450,
        "the cross-product must be generated, not sampled"
    );
    assert!(
        leaks.is_empty(),
        "{} of {rows} rows leaked the material to stderr:\n{}",
        leaks.len(),
        leaks.join("\n")
    );
}

/// **THE ORDERING TEST**, and it is what distinguishes pre- from post-parse
/// without modifying any non-test code.
///
/// `me --nosuchflag <ms1>` reaches the guard (exit **3**, the guard's wording)
/// and not clap (exit **2**, naming the flag). Measured on the pre-guard
/// binary: exit 2, `error: unexpected argument '--nosuchflag' found`.
#[test]
fn the_guard_decides_before_clap_does() {
    let o = run(&["--nosuchflag", MS1]);
    assert!(
        is_guard_refusal(&o),
        "the guard must decide first; got {:?}:\n{}",
        o.status.code(),
        stderr_of(&o)
    );
    assert!(
        !stderr_of(&o).contains("unexpected argument"),
        "clap must not have run: {}",
        stderr_of(&o)
    );
}

/// **The override, and its SCOPE.** It is honoured only where the flag is
/// DECLARED — `sysw pack` and `seal`. Anywhere else the guard refuses even
/// though argv carries the flag, and it refuses with ITS wording rather than
/// letting clap reject an unexpected argument.
#[test]
fn allow_argv_secret_binds_only_where_it_is_declared() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    let uf2 = dir.path().join("p.uf2");

    let o = run(&[
        "sysw",
        "pack",
        "--allow-argv-secret",
        "--no-passphrase",
        "--out",
        out.to_str().unwrap(),
        MS1,
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", stderr_of(&o));

    let o = run(&[
        "seal",
        "--allow-argv-secret",
        MS1,
        "--seal-secret",
        "--out",
        uf2.to_str().unwrap(),
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "seal declares the flag too -- its positional is a deliberately retained \
         channel, so guarding the surface without offering the override would \
         DELETE a documented path rather than gate it: {}",
        stderr_of(&o)
    );

    for surface in [vec!["bundle"], vec!["sysw", "wipe"], vec!["hash"]] {
        let mut args = surface.clone();
        args.push("--allow-argv-secret");
        args.push(MS1);
        let o = run(&args);
        assert!(
            is_guard_refusal(&o),
            "me {} does not declare the flag, so it must still be refused BY THE \
             GUARD -- not by clap calling it an unexpected argument: {}",
            surface.join(" "),
            stderr_of(&o)
        );
    }
}

/// **THE POSITIVE CONTROLS, without which a refuse-everything guard passes
/// every row above.** `fn guard(_) -> ! { exit(3) }` satisfies both the
/// absence-of-secret gate and the ordering test.
#[test]
fn the_guard_refuses_nothing_it_should_not() {
    let dir = tempfile::tempdir().unwrap();

    // Exit codes unchanged from today's MEASURED values -- not bare
    // `.success()`, which `bundle`'s 2 fails for an unrelated reason.
    for (surface, code) in [
        (vec!["bundle"], 2),
        (vec!["help"], 0),
        (vec!["sysw", "help"], 0),
    ] {
        let o = run(&surface);
        assert_eq!(
            o.status.code(),
            Some(code),
            "me {} must be unchanged",
            surface.join(" ")
        );
        assert!(
            !stderr_of(&o).contains("Refused BEFORE"),
            "me {} must not meet the guard at all",
            surface.join(" ")
        );
    }

    // THREE of the eight subcommand shapes are BIP-39 words -- `bundle`, and
    // `help` twice, once inside `sysw help`. A per-token wordlist match would
    // refuse them. Granularity is the classifier's: a single word is not a
    // mnemonic, because a classifier DECODES rather than prefix-matches.
    for word in ["bundle", "help"] {
        assert!(
            bip39::Mnemonic::parse_normalized(word).is_err(),
            "{word} must not classify as a mnemonic on its own"
        );
    }

    // A legitimate record still packs.
    let out = dir.path().join("ok.bin");
    let o = run(&[
        "sysw",
        "pack",
        "--no-passphrase",
        "--out",
        out.to_str().unwrap(),
        "text:6869",
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", stderr_of(&o));

    // A FILENAME containing an HRP is packed, not refused -- and so is one
    // named after an mt1 set. `mt1-2026-08-23-transfer.txt` is a filename, and
    // a classifier that decoded it would have to be a prefix matcher instead.
    for name in [
        format!("{MS1}.txt"),
        "mt1-2026-08-23-transfer.txt".to_string(),
    ] {
        let src = dir.path().join(&name);
        std::fs::write(&src, "text:6869\n").unwrap();
        let out = dir.path().join("fromfile.bin");
        let o = run(&[
            "sysw",
            "pack",
            "--no-passphrase",
            "--in",
            src.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ]);
        assert_eq!(
            o.status.code(),
            Some(0),
            "a file named {name} must be readable: {}",
            stderr_of(&o)
        );
    }
}

/// **WHAT THE GUARD CANNOT REACH, asserted so the residue is a fact and not a
/// hope.** Both are consequences of asking a classifier rather than
/// prefix-matching, and the alternative is worse in both cases.
#[test]
fn the_two_shapes_out_of_reach_are_pinned() {
    let dir = tempfile::tempdir().unwrap();

    // An UNQUOTED twelve-word mnemonic is twelve tokens, each `Unknown`. The
    // guard does not refuse it; only the quoted, single-token phrase is in its
    // reach. Refusing per-word would refuse `me bundle`.
    let mut args = vec!["sysw", "pack", "--no-passphrase"];
    args.extend(MNEMONIC.split(' '));
    let o = run(&args);
    assert!(
        !is_guard_refusal(&o),
        "the guard is not expected to reach an unquoted phrase; if it now does, \
         check what else a per-word rule started refusing"
    );

    // F-267: a secret embedded in a PATH is out of reach, because it IS a
    // filename -- and refusing it would refuse every legitimate path.
    let src = dir.path().join(format!("{MS1}.txt"));
    std::fs::write(&src, "text:6869\n").unwrap();
    let out = dir.path().join("f267.bin");
    let o = run(&[
        "sysw",
        "pack",
        "--no-passphrase",
        "--in",
        src.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "F-267 residue: a path carrying a secret is still read: {}",
        stderr_of(&o)
    );
}
