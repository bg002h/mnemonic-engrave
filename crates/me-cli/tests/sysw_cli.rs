//! `me sysw` — the plan's stage-2 green criteria, as tests.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fmt::Write as _;

const MD1: &str = "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3";
const SEED: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const TEXT: &str = "text:48656c6c6f2c20576f726c6421";

fn me() -> Command {
    Command::cargo_bin("me").unwrap()
}

/// The digest line, located by its LABEL rather than by shape.
///
/// `seal_cli.rs` records why that matters: a token-count heuristic there matched
/// a line of prose instead of the value, and the assertion built on it went
/// vacuous without failing.
fn digest_line(err: &str) -> Option<&str> {
    err.lines().find_map(|l| l.strip_prefix("digest:   "))
}

#[test]
fn wipe_emits_exactly_one_region() {
    for fill in ["random", "zeros", "ones"] {
        let out = me()
            .args(["sysw", "wipe", "--fill", fill])
            .assert()
            .success();
        assert_eq!(
            out.get_output().stdout.len(),
            65_536,
            "--fill {fill} must fill the region"
        );
    }
}

#[test]
fn wipe_rejects_an_unknown_fill_rather_than_defaulting() {
    me().args(["sysw", "wipe", "--fill", "sideways"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown --fill"));
}

/// Ones is the erased state, so the operator is told — it is the one fill whose
/// result is indistinguishable from "never written".
#[test]
fn wipe_with_ones_says_what_ones_means() {
    me().args(["sysw", "wipe", "--fill", "ones"])
        .assert()
        .success()
        .stderr(predicate::str::contains("ERASED state"));
}

/// The blob goes to stdout and the digest to stderr, so `me sysw pack > f.bin`
/// still shows the operator the number they must compare on the machine.
#[test]
fn pack_separates_the_blob_from_the_digest() {
    let out = me()
        .args(["sysw", "pack", "--no-passphrase", TEXT])
        .assert()
        .success();
    let o = out.get_output();
    assert!(!o.stdout.is_empty(), "the blob goes to stdout");
    assert_eq!(
        &o.stdout[..8],
        b"MNEMSYSW",
        "and it is a systemwide container"
    );
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        digest_line(&err).is_some(),
        "the digest goes to stderr:\n{err}"
    );
}

/// The comparison loop: what `pack` prints must be what `show` prints, or the
/// operator has nothing to compare against.
#[test]
fn show_prints_the_same_digest_pack_did() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("p.bin");
    let packed = me()
        .args(["sysw", "pack", "--no-passphrase", TEXT, "--out"])
        .arg(&f)
        .assert()
        .success();
    let perr = String::from_utf8_lossy(&packed.get_output().stderr).into_owned();
    let shown = me().args(["sysw", "show"]).arg(&f).assert().success();
    let serr = String::from_utf8_lossy(&shown.get_output().stderr).into_owned();
    let a = digest_line(&perr).expect("pack prints a digest");
    let b = digest_line(&serr).expect("show prints a digest");
    assert_eq!(
        a, b,
        "pack and show must agree, or the comparison is worthless"
    );
    assert_eq!(a.split(' ').count(), 8, "grouped in fours for a human");
}

/// Spec §13 D3: `me` WARNS and proceeds. It once refused; a test asserting a
/// non-zero exit here would be re-imposing the demoted rule.
#[test]
fn a_weak_passphrase_over_a_secret_warns_and_still_succeeds() {
    me().args(["sysw", "pack", "--passphrase-words", "2"])
        .write_stdin(SEED)
        .assert()
        .success()
        .stderr(predicate::str::contains("WARNING"))
        .stderr(predicate::str::contains("BELOW the threshold"));
}

#[test]
fn allow_weak_is_accepted_and_says_it_is_ignored() {
    me().args(["sysw", "pack", "--allow-weak", "--no-passphrase", TEXT])
        .assert()
        .success()
        .stderr(predicate::str::contains("accepted and ignored"));
}

/// The default is to GENERATE, not to leave a payload unprotected by omission.
#[test]
fn omitting_every_passphrase_flag_generates_one() {
    // SEED, not MD1: since G-P3.6 the default is decided by CONTENT (§2.4),
    // so the invocation this test is about -- "no flag, and a passphrase is
    // generated rather than the payload left unprotected by omission" -- is
    // the one carrying secret material. A payload of public cards is now
    // deliberately cleartext, which
    // `a_payload_with_no_secret_record_is_not_sealed_by_default` pins.
    me().args(["sysw", "pack"])
        .write_stdin(SEED)
        .assert()
        .success()
        .stderr(predicate::str::contains("write this down"))
        .stderr(predicate::str::contains("12 words"));
}

#[test]
fn the_passphrase_modes_are_mutually_exclusive() {
    me().args([
        "sysw",
        "pack",
        "--no-passphrase",
        "--passphrase-words",
        "8",
        MD1,
    ])
    .assert()
    .failure();
    me().args(["sysw", "pack", "--passphrase-ask", "--no-passphrase", MD1])
        .assert()
        .failure();
}

/// Spec §5.2: structural failures must NEVER say "payload unreadable" — that
/// phrase teaches the operator to read a wrong file as tampering.
#[test]
fn a_wrong_file_is_not_reported_as_tampering() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("junk.bin");
    std::fs::write(&f, b"MNEMBLOB....................").unwrap();
    me().args(["sysw", "show"])
        .arg(&f)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a systemwide container"))
        .stderr(predicate::str::contains("payload unreadable").not());
}

/// A secrets-only payload has no public section, so there is no digest to
/// compare — and `show` says so rather than printing a constant every such
/// payload shares (R0-C2).
#[test]
fn a_secrets_only_payload_reports_no_digest() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("s.bin");
    me().args(["sysw", "pack", "--passphrase-words", "12", "--out"])
        .write_stdin(SEED)
        .arg(&f)
        .assert()
        .success();
    me().args(["sysw", "show"])
        .arg(&f)
        .assert()
        .success()
        .stdout(predicate::str::contains("pub_len:  0"))
        .stderr(predicate::str::contains("no public section"));
}

#[test]
fn records_can_come_from_a_file_instead_of_argv() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("recs.txt");
    std::fs::write(&f, format!("{MD1}\n{TEXT}\n")).unwrap();
    me().args(["sysw", "pack", "--no-passphrase", "--in"])
        .arg(&f)
        .assert()
        .success();
}

/// The plan's stage-2 green line said `me sysw pack … | wc -c` is 65536. It was
/// not: `pack` emitted the container and only `wipe` emitted a region, so the
/// one artifact you can actually write to `0x10D00000` had no command. `--region`
/// is that command.
#[test]
fn region_pads_the_container_to_a_flashable_image() {
    let out = me()
        .args(["sysw", "pack", "--no-passphrase", "--region", TEXT])
        .assert()
        .success();
    let o = &out.get_output().stdout;
    assert_eq!(o.len(), 65_536, "--region emits exactly one region");
    assert_eq!(&o[..8], b"MNEMSYSW", "the container is at offset 0");
    assert!(
        o[600..].iter().all(|&b| b == 0xFF),
        "the tail is 0xFF — the ERASED state, so the image is what the sector \
         looks like with only the container written. Zeros would be a WRITE, \
         and would wear the flash for nothing."
    );
}

/// The padding must not change what the operator compares on screen, or the
/// region image and the container would be two different payloads.
#[test]
fn region_and_container_have_the_same_digest_and_identity() {
    let dir = tempfile::tempdir().unwrap();
    let (c, r) = (dir.path().join("c.bin"), dir.path().join("r.bin"));
    for (f, extra) in [(&c, None), (&r, Some("--region"))] {
        let mut cmd = me();
        cmd.args(["sysw", "pack", "--no-passphrase"]);
        if let Some(e) = extra {
            cmd.arg(e);
        }
        cmd.arg(TEXT).arg("--out").arg(f).assert().success();
    }
    let show = |f: &std::path::Path| {
        let a = me().args(["sysw", "show"]).arg(f).assert().success();
        let o = a.get_output();
        (
            String::from_utf8_lossy(&o.stdout).into_owned(),
            String::from_utf8_lossy(&o.stderr).into_owned(),
        )
    };
    let (co, ce) = show(&c);
    let (ro, re) = show(&r);
    assert_eq!(
        digest_line(&ce),
        digest_line(&re),
        "padding must not move the digest"
    );
    let id = |s: &str| {
        s.lines()
            .find_map(|l| l.strip_prefix("identity: "))
            .map(str::to_string)
    };
    assert_eq!(id(&co), id(&ro), "nor the identity");
    assert!(id(&co).is_some(), "and one was actually printed");
}

/// `--region` describes where the bytes go, not what is in them; refusing to
/// combine it with a sealed payload would make the flashable form the ONE form
/// a secret cannot take.
#[test]
fn region_works_for_a_sealed_payload_too() {
    let out = me()
        .args(["sysw", "pack", "--region", "--passphrase-words", "12"])
        .write_stdin(SEED)
        .assert()
        .success();
    assert_eq!(out.get_output().stdout.len(), 65_536);
}

/// Pre-flash fable review, C1. `pack` enforced neither bound its own parser
/// enforces, so it emitted containers `show` refuses — exit 0 on write, exit 4
/// on read. With `--region` that becomes a flash-ready image of an unreadable
/// payload, and for a SEALED one that is a seed backup nobody can ever open.
///
/// The rule these pin: **the writer must refuse everything the reader refuses.**
/// A writer looser than its reader is how permanent media gets a payload that
/// was never openable.
#[test]
fn pack_refuses_iterations_its_own_parser_would_reject() {
    for n in ["5", "99999", "2000001"] {
        me().args([
            "sysw",
            "pack",
            "--iterations",
            n,
            "--passphrase-words",
            "12",
            "--allow-argv-secret",
            SEED,
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("iterations"));
    }
}

#[test]
fn pack_accepts_the_iteration_bounds_themselves() {
    for n in ["100000", "2000000"] {
        me().args([
            "sysw",
            "pack",
            "--iterations",
            n,
            "--passphrase-words",
            "12",
            "--allow-argv-secret",
            SEED,
        ])
        .assert()
        .success();
    }
}

/// The NEAREST HOSTILE INPUT to the raised cap: a section one record past it is
/// still refused, on the write path, in the writer's words rather than the
/// reader's.
///
/// THE COUNT IS DERIVED FROM THE CONSTANT. It was a literal 30 records, which
/// was comfortably past 8191 and went silently vacuous the moment the cap was
/// raised to 32,734 -- the section it built became legal and `pack` was right
/// to accept it. That is the shape a raise is supposed to expose, and a
/// hard-coded fixture hides it.
#[test]
fn pack_refuses_a_section_too_long_for_its_own_parser() {
    use mnemonic_engrave::sysw::wire::MAX_SECTION_LEN;
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("big.txt");
    const REC_LEN: usize = "text:".len() + 800; // 400 hex pairs
    let n = MAX_SECTION_LEN / (REC_LEN + 1) + 2; // + the LF between records
    let section_len = n * REC_LEN + (n - 1);
    assert!(
        section_len > MAX_SECTION_LEN,
        "the fixture must exceed the cap to test anything: {section_len} vs {MAX_SECTION_LEN}"
    );
    // ...and it must stay INSIDE the region, or `bound` refuses it for the
    // other reason and this stops being a section-cap test at all.
    assert!(
        52 + section_len < 65_536,
        "still a section refusal, not a region one"
    );
    let recs: Vec<String> = (0..n)
        .map(|_| format!("text:{}", "61".repeat(400)))
        .collect();
    std::fs::write(&f, recs.join("\n")).unwrap();
    me().args(["sysw", "pack", "--no-passphrase", "--in"])
        .arg(&f)
        .assert()
        .code(4)
        .stderr(predicate::str::contains(format!(
            "a section caps at {MAX_SECTION_LEN} bytes"
        )));
}

/// The property behind both, stated once: anything `pack` writes, `show` reads.
#[test]
fn everything_pack_emits_is_readable_by_show() {
    let dir = tempfile::tempdir().unwrap();
    for (i, args) in [
        vec!["--no-passphrase", TEXT],
        vec!["--no-passphrase", MD1],
        vec!["--passphrase-words", "12", "--allow-argv-secret", SEED],
        vec![
            "--iterations",
            "100000",
            "--passphrase-words",
            "2",
            "--allow-argv-secret",
            SEED,
        ],
    ]
    .iter()
    .enumerate()
    {
        let f = dir.path().join(format!("p{i}.bin"));
        let mut c = me();
        c.args(["sysw", "pack"]);
        c.args(args.iter());
        c.arg("--out").arg(&f).assert().success();
        me().args(["sysw", "show"])
            .arg(&f)
            .assert()
            .success()
            .stdout(predicate::str::contains("identity:"));
    }
}

/// Pre-flash fable review, I2. `show` printed a plausible identity and THEN
/// panicked (exit 101) on a container whose header declares more than the file
/// holds. A panic after a plausible-looking line is the worst shape: the
/// operator has already read a number that means nothing.
#[test]
fn show_on_a_truncated_container_fails_cleanly_without_printing_a_digest() {
    let dir = tempfile::tempdir().unwrap();
    let (full, trunc) = (dir.path().join("f.bin"), dir.path().join("t.bin"));
    me().args(["sysw", "pack", "--no-passphrase", TEXT, "--out"])
        .arg(&full)
        .assert()
        .success();
    let all = std::fs::read(&full).unwrap();
    std::fs::write(&trunc, &all[..60.min(all.len())]).unwrap();
    let out = me().args(["sysw", "show"]).arg(&trunc).assert().failure();
    let o = out.get_output();
    assert_ne!(o.status.code(), Some(101), "must not panic");
    assert!(
        digest_line(&String::from_utf8_lossy(&o.stderr)).is_none(),
        "and must not print a digest it cannot compute"
    );
}

#[test]
fn an_unplaceable_record_is_named_with_its_index() {
    me().args(["sysw", "pack", "--no-passphrase", MD1, "not a record"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("record 1"));
}

/// The refusal an operator actually hits: a reserved prefix carrying its body
/// as plain text. It must say THAT, and it must not print the body.
///
/// The body here is a passphrase, which is the whole point — stderr is
/// scrolled back, logged, and pasted into bug reports. The first version of
/// this message named neither the prefix nor the body, and instead explained
/// the descriptor/address gap, which is a different failure with a different
/// remedy.
#[test]
fn a_plain_text_pass_body_is_refused_by_body_and_never_echoed() {
    let out = me()
        .args([
            "sysw",
            "pack",
            "--no-passphrase",
            "pass:correct horse battery staple",
        ])
        .assert()
        .failure();
    let err = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(
        err.contains("not lowercase hex"),
        "the refusal must name the real cause: {err}"
    );
    assert!(
        err.contains("xxd -p"),
        "and hand over the one-liner that fixes it: {err}"
    );
    assert!(
        !err.contains("correct horse"),
        "THE PASSPHRASE MUST NOT REACH STDERR: {err}"
    );
    assert!(
        !err.contains("Descriptors and addresses"),
        "and must not explain the gap that did not apply here: {err}"
    );
}

/// Pre-flash conformance review, I3. Rust models "no passphrase" as `None`, so
/// `Some("")` is a real passphrase here — but the device reads an empty
/// passphrase as *absence*, so a payload sealed with one can never be opened on
/// the machine it was made for. Not a strength refusal (those warn and
/// proceed); an unopenable-artifact refusal.
#[test]
fn an_empty_passphrase_is_refused_because_the_device_could_never_open_it() {
    // `--passphrase-ask` reads the tty, so drive the library-level path the CLI
    // reaches: whitespace-only normalises to empty too, and must refuse alike.
    use mnemonic_engrave::sysw;
    for p in ["", " ", "   \t  "] {
        assert_eq!(
            sysw::pack(vec![TEXT.into()], Some(p), 100_000).unwrap_err(),
            sysw::SyswError::EmptyPassphrase,
            "passphrase {p:?} normalises to nothing and must be refused"
        );
    }
    // A real one still works, so the check cannot pass by refusing everything.
    assert!(sysw::pack(vec![TEXT.into()], Some("abandon about"), 100_000).is_ok());
}

/// Operator ruling 2026-08-12, the cheap-and-narrowing choice: `me` refuses a
/// passphrase the DEVICE cannot type. Decision 8 allowed ASCII; the device only
/// ever grew a word keyboard, so an ASCII passphrase seals a payload that can
/// never be opened on the machine it is for.
///
/// Not a strength rule. Two BIP-39 words still pass, still sit below `[cliff]`,
/// and still only warn — that is decision 8's restored mode and F2's job.
#[test]
fn pack_refuses_a_passphrase_the_device_could_not_type() {
    use mnemonic_engrave::sysw;
    for p in [
        "hunter2",
        "correct horse battery staple",
        "abandon 1",
        "abandon abou",
    ] {
        match sysw::pack(vec![TEXT.into()], Some(p), 100_000) {
            Err(sysw::SyswError::NotEnterableOnDevice(_)) => {}
            other => panic!("{p:?} should be refused as un-typeable, got {other:?}"),
        }
    }
    // The wordlist half only — NOT the [cliff] count. Two words remain legal.
    assert!(sysw::pack(vec![TEXT.into()], Some("abandon about"), 100_000).is_ok());
    // And the check runs AFTER normalisation, which lowercases: "ABOUT" is the
    // same word, and the device's own keyboard is uppercase. Asserted because my
    // first version of this test expected the opposite and the test caught me,
    // not the code.
    assert!(sysw::pack(vec![TEXT.into()], Some("abandon ABOUT"), 100_000).is_ok());
}

/// `[passphrase-bounds]` (§12.5) was declared on both sides and enforced on
/// neither: the constant, a const assertion and an arithmetic test were its only
/// references.
#[test]
fn pack_enforces_the_passphrase_length_bound() {
    use mnemonic_engrave::sysw;
    let long = std::iter::repeat_n("abandon", 40)
        .collect::<Vec<_>>()
        .join(" ");
    assert!(long.len() > sysw::wire::PASSPHRASE_MAX);
    match sysw::pack(vec![TEXT.into()], Some(&long), 100_000) {
        Err(sysw::SyswError::PassphraseTooLong(n)) => {
            assert!(n > sysw::wire::PASSPHRASE_MAX)
        }
        other => panic!("an over-long passphrase must be refused, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// `[mdmk-decode]` (§12.6; §13 D6) at the CLI. It WARNS and proceeds — a test
// here asserting a non-zero exit would be re-imposing the refusal D6 demoted,
// which is the direction test expectations drift when nobody asserts the
// demotion.
// ---------------------------------------------------------------------------

/// The other two chunks of `MD1`'s set (398802), so a COMPLETE card can be
/// packed. Measured, not assumed.
const MD1_B: &str = "md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374";
const MD1_C: &str = "md1fv9wjpqsp2026hh65xpvugtfhd9792zxgunymm0a82pdju6442q0jskj9gzfaqmz";

#[test]
fn pack_warns_once_per_unconfirmed_record_and_still_succeeds() {
    // MD1 is chunk 0 of 3, so alone it cannot reassemble and cannot decode.
    let out = me()
        .args(["sysw", "pack", "--no-passphrase", MD1])
        .assert()
        .success();
    let err = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(
        err.contains(
            "record 0, as given (records count from 0): an md1/mk1 this tool could not decode"
        ),
        "the warning must name the record by the index the operator gave it, AND say \
             that is the basis — `me sysw show` numbers the public section instead, and \
             on a sealed payload the two diverge: {err}"
    );
    assert!(
        err.contains("SECRET"),
        "and say what the device will do with it: {err}"
    );
    // The base is stated because it is 0 and every text editor counts from 1.
    // Writing the Load Payload journey, `record 1` on a three-line file sent me
    // to the wrong line — the index is real, the reader's assumption is not.
    assert!(
        err.contains("records count from 0"),
        "an unlabelled index is read as 1-based: {err}"
    );
}

/// The index is into the OPERATOR'S argv, not into a list filtered to the
/// md1/mk1 records — R1-I2. With a seed first, the same card is record 1.
#[test]
fn the_warning_names_the_record_the_operator_passed() {
    // BOTH records through the SAME channel, in order. This test is about the
    // INDEX the warning names -- record 1, not record 0 -- so splitting them
    // across argv and stdin would silently reduce it to a one-record run
    // (argv wins over stdin) and the assertion would be vacuous.
    let out = me()
        .args(["sysw", "pack", "--no-passphrase"])
        .write_stdin(format!("{SEED}\n{MD1}"))
        .assert()
        .success();
    let err = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(
        err.contains("record 1, as given (records count from 0): an md1/mk1"),
        "{err}"
    );
    assert!(
        !err.contains("record 0, as given (records count from 0): an md1/mk1"),
        "{err}"
    );
}

/// The other direction, which is what makes the test above able to fail: a
/// complete card set produces no warning at all.
#[test]
fn a_complete_card_set_draws_no_warning() {
    let out = me()
        .args(["sysw", "pack", "--no-passphrase", MD1, MD1_B, MD1_C])
        .assert()
        .success();
    let err = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(
        !err.contains("could not decode"),
        "a set that reassembles and decodes is confirmed: {err}"
    );
}

/// `me sysw show` states the answer per record, so the operator can see which
/// card the machine will treat as a secret before they flash it.
#[test]
fn show_states_confirmed_or_unconfirmed_beside_each_mdmk_record() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("p.bin");
    me().args(["sysw", "pack", "--no-passphrase", MD1, MD1_B, MD1_C, TEXT])
        .arg("--out")
        .arg(&f)
        .assert()
        .success();
    let shown = me().args(["sysw", "show"]).arg(&f).assert().success();
    let out = String::from_utf8_lossy(&shown.get_output().stdout).into_owned();
    for i in 0..3 {
        assert!(
            out.contains(&format!("record {i}: md1/mk1 — confirmed")),
            "record {i} of a complete set must read confirmed:\n{out}"
        );
    }
    assert!(
        !out.contains("record 3:"),
        "the text: record is not ClassMDMK and this rule is not about it:\n{out}"
    );

    // And the same command over a lone chunk says the opposite.
    let g = dir.path().join("q.bin");
    me().args(["sysw", "pack", "--no-passphrase", TEXT, MD1, "--out"])
        .arg(&g)
        .assert()
        .success();
    let shown = me().args(["sysw", "show"]).arg(&g).assert().success();
    let out = String::from_utf8_lossy(&shown.get_output().stdout).into_owned();
    assert!(
        out.contains("record 1: md1/mk1 — unconfirmed"),
        "a lone chunk of a declared set must read unconfirmed:\n{out}"
    );
}

// ─── mt1 + tx: records (the transaction-engraving payload path) ──────────────

/// The "even" vector from mt-codec's pinned corpus: a real signed 222-byte
/// transaction, 6 chunks, chunk_set_id 0x2dcf2.
const MT_EVEN: [&str; 6] = [
    "mt1p9h8jqq9qqqqgqqqqqqqyqherdfykhhpey6z2cvafak8804qd7g0dl6v8ex9wr2cvky023skwkeud2229sax",
    "mt1p9h8jqq9qqphgdqqqqqqqq0mllllupyqj6vqqqqqqqqzcqpfsw7ph2rt5w54kt768636cls8zxg0najlzunp",
    "mt1p9h8jqq9qqzj8yqpnzw4vl2rwffqyqqqqqkqq282yyhc2vavd20hvk94pz39hts3u5s9a0qd8pwskxfl7ju5",
    "mt1p9h8jqq9qqrqfrnq3qzyp77h37cnxzvwutegzmzy5zrrrfvrpykdfsckvk03dcq6rcjtvlsfcglv7zx43yaz",
    "mt1p9h8jqq9qqylgpzqmhcwhuupdvnrc82rncvzzdahpgjsdwgu52jd7vmxsve9x3w5ujeqyssuvddxvwqze4ve",
    "mt1p9h8jqq9qq9qdcc7h75twfxyf340c4sgqzhfdq6xtgt7zhxngpwa049l0z59l6jqcqqqqqq5k5y2ye5nv8yf",
];
const MT_EVEN_RAW_HEX: &str = "020000000001017c8da925af70e49a12b0cea7b639df5037c87b7fa61f262b86ac32c47aa3ba1a0000000000fdffffff02404b4c0000000000160014c1de0dd435d1d4ad97ed1f51d63f91c800cc4eab3ea1b92901000000160014751097c299d6354fbb2c5a84512dd708f2902f5e0247304402207debc7d89984c7717940b622504318d2c184966a618b32cf8b700d0f125b3ffa02206ef875f9c0b5931e0ea1cf0c109bdb8512835c8e51526f99b3419929a2ea7259012103718f5fd45b926226357e2b0400574b41a32d0bf0ae69a02eebea5fbc542ff52060000000";
const MT_EVEN_TXID: &str = "2dcf2b973d52044b1e58c988a5a59d388073ff05598b0a1e93eeb04c72ebf630";

/// A complete mt set packs plaintext with NO unconfirmed warnings, and `show`
/// names the transaction it carries.
#[test]
fn pack_accepts_a_complete_mt_set_and_show_confirms_it() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    // Via stdin, not argv: mt1 strings are BEARER and argv is public (P5 I-1).
    // What this test is about is what `pack` does with a complete set.
    let a = me()
        .args(["sysw", "pack", "--no-passphrase", "--out"])
        .arg(&out)
        .write_stdin(MT_EVEN.join("\n"))
        .assert()
        .success();
    let err = String::from_utf8_lossy(&a.get_output().stderr).to_string();
    assert!(
        !err.contains("could not confirm"),
        "a complete set must not warn: {err}"
    );

    let show = me()
        .args(["sysw", "show", out.to_str().unwrap()])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&show.get_output().stdout).to_string();
    assert_eq!(
        stdout.matches("mt1 chunk — confirmed").count(),
        6,
        "{stdout}"
    );
    assert!(stdout.contains(MT_EVEN_TXID), "{stdout}");
    assert!(stdout.contains("222 bytes"), "{stdout}");
}

/// One chunk alone packs — nothing refuses (§13 D6's demotion applies to mt as
/// to mdmk) — but pack WARNS and show reports it unconfirmed.
#[test]
fn pack_warns_on_an_incomplete_mt_set() {
    // Via stdin, not argv: mt1 strings are BEARER and argv is public (P5 I-1).
    let a = me()
        .args(["sysw", "pack", "--no-passphrase"])
        .write_stdin(MT_EVEN[0])
        .assert()
        .success();
    let err = String::from_utf8_lossy(&a.get_output().stderr).to_string();
    // The report is PER SET since G-P3.7, and it names the set, the record
    // indices in it, and every missing string against the header's own count.
    assert!(err.contains("record"), "{err}");
    assert!(err.contains("2dcf2"), "must name the set: {err}");
    assert!(err.contains("did NOT"), "{err}");
    assert!(err.contains("of 6"), "must name the declared count: {err}");
}

/// THE RULED PIPELINE'S `me` HALF: the record `mt encode --qr`
/// emits packs, on stdin, and reads back.
///
/// **The record is CONSTRUCTED here rather than produced.** The producer moved
/// to `mt` with P3b, and this repo cannot invoke it, so what is pinned is the
/// FORMAT: `tx:` + the transaction's canonical serialization in lowercase hex,
/// nothing else. `mt`'s own
/// `tests/tx_record.rs::the_raw_form_is_the_prefix_and_the_transaction_hex_and_nothing_else`
/// asserts its stdout equals that same string over the same `even` vector, so
/// the two repos pin one shape from opposite sides with no shared code between
/// them. If either drifts, one of the two goes red.
///
/// argv is refused for this class (G-P3.5), so stdin is the whole join — and
/// it is the invocation §1.1 said must work.
#[test]
fn the_record_mt_emits_packs_on_stdin() {
    let record = format!("tx:{MT_EVEN_RAW_HEX}");
    me().args(["sysw", "pack", "--no-passphrase"])
        .write_stdin(format!("{record}\n"))
        .assert()
        .success();
}

/// The tx: prefix is RESERVED: hex that is not a transaction is refused with
/// the structural reason, and non-hex with the hex reason.
#[test]
fn pack_refuses_a_tx_record_that_is_not_a_transaction() {
    // Delivered on STDIN. The argv channel is refused for the whole class
    // (G-P3.5) BEFORE anything looks at the body, so these two messages are
    // only reachable through a private channel -- which is also where a real
    // operator meets them.
    me().args(["sysw", "pack", "--no-passphrase"])
        .write_stdin("tx:abab\n")
        .assert()
        .code(4)
        .stderr(predicate::str::contains(
            "not one serialized Bitcoin transaction",
        ));
    me().args(["sysw", "pack", "--no-passphrase"])
        .write_stdin("tx:zz\n")
        .assert()
        .code(4)
        .stderr(predicate::str::contains("not lowercase hex"));
}

/// A flipped character in an mt1 string is REFUSED at pack (exact validity,
/// never correction), with the record's index named.
#[test]
fn pack_refuses_a_damaged_mt1_string() {
    let mut bad = MT_EVEN[0].to_string();
    bad.pop();
    bad.push(if MT_EVEN[0].ends_with('x') { 'y' } else { 'x' });
    me().args(["sysw", "pack", "--no-passphrase", &bad])
        .assert()
        .failure()
        .stderr(predicate::str::contains("record 0"));
}

/// `show` names a tx: record as what it is, with its txid — the value the
/// operator compares against the report `mt encode` printed when it built the
/// record.
#[test]
fn show_names_a_tx_record() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    let rec = format!("tx:{MT_EVEN_RAW_HEX}");
    me().args([
        "sysw",
        "pack",
        "--no-passphrase",
        "--out",
        out.to_str().unwrap(),
    ])
    .write_stdin(format!("{rec}\n"))
    .assert()
    .success();
    me().args(["sysw", "show", out.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("raw signed transaction"))
        .stdout(predicate::str::contains(MT_EVEN_TXID))
        .stdout(predicate::str::contains("222 bytes"));
}

/// GRAFT 1 — THE DELIVERY CEILING, end to end.
///
/// The cap that governs how much one payload can carry is
/// `sysw::wire::MAX_SECTION_LEN`, raised here from 8191 to 32,734. This is the
/// case the raise exists for and the case that failed before it: a public
/// section of ~20 KB packs, and `show` reads back exactly what went in.
///
/// `text:` records are used because the cap is a CONTAINER fact — it counts
/// bytes of section, not transactions — and a `text:` record is the cheapest
/// honest way to occupy them.
#[test]
fn a_payload_past_the_old_8191_cap_packs_and_reads_back() {
    let dir = tempfile::tempdir().unwrap();
    let recs_path = dir.path().join("recs.txt");
    let out = dir.path().join("p.bin");

    // 20 records of 1,005 bytes each, joined by LF: 20*1005 + 19 = 20,119.
    let one = format!("text:{}", "6162636465".repeat(100)); // 5 + 1000 = 1005
    assert_eq!(one.len(), 1005);
    let n = 20;
    let section_len = n * one.len() + (n - 1);
    assert_eq!(section_len, 20_119);
    assert!(
        section_len > 8191,
        "the test is vacuous unless the section is past the OLD cap"
    );
    let body = std::iter::repeat_n(one.as_str(), n)
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&recs_path, &body).unwrap();

    me().args([
        "sysw",
        "pack",
        "--no-passphrase",
        "--in",
        recs_path.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ])
    .assert()
    .success();

    me().args(["sysw", "show", out.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("pub_len:  {section_len}")));
}

// ─── G-P3.4 — the ruled pipeline `mt encode | me sysw pack` ──────────────────

/// SPEC §1.1 ruled that `me sysw pack` gains a stdin path, because the join
/// between the two tools IS a pipe and the first draft was wrong to say they
/// already composed over one. Measured before this gate:
///
/// ```text
/// $ printf 'text:6869\n' | me sysw pack --no-passphrase
/// me: no records: pass them on argv or with --in     (exit 2, stdout empty)
/// ```
#[test]
fn pack_reads_records_from_stdin_when_neither_argv_nor_in_is_given() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    me().args([
        "sysw",
        "pack",
        "--no-passphrase",
        "--out",
        out.to_str().unwrap(),
    ])
    .write_stdin(format!("{TEXT}\n{MD1}\n"))
    .assert()
    .success();
    // Not merely "exit 0": both records must be IN the container.
    me().args(["sysw", "show", out.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("public record 1: md1/mk1"));
}

/// Blank lines are skipped on stdin exactly as they are with `--in`, so a
/// record's index is its position among the NON-blank lines. `mt encode`
/// separates nothing with blanks today, but a shell heredoc does.
#[test]
fn stdin_skips_blank_lines_like_in_does() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    me().args([
        "sysw",
        "pack",
        "--no-passphrase",
        "--out",
        out.to_str().unwrap(),
    ])
    .write_stdin(format!("\n{TEXT}\n\n{MD1}\n\n"))
    .assert()
    .success();
    me().args(["sysw", "show", out.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("public record 1: md1/mk1"));
}

/// R7. EMPTY stdin joins the existing exit-2 path rather than packing an empty
/// container — `fish` reports a pipeline's status as the LAST command's, so an
/// upstream `mt encode` failure arrives here as nothing at all, and a container
/// built from it would be a silent success.
#[test]
fn empty_stdin_is_the_exit_2_path_not_an_empty_container() {
    me().args(["sysw", "pack", "--no-passphrase"])
        .write_stdin("")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("no records"));
    // Whitespace-only is the same thing: `mt encode` writing a bare newline on
    // a failure path must not become a container either.
    me().args(["sysw", "pack", "--no-passphrase"])
        .write_stdin("\n\n\n")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("no records"));
}

/// R7's OTHER CHANNEL, found by running the pipeline rather than by reading it.
///
/// R7 was implemented on stdin only, and `--in` returned whatever the file
/// held — so an EMPTY file packed an empty container at **exit 0** and wrote
/// 52 bytes of header holding nothing, while the byte-identical situation on
/// stdin exited 2. R7's own stated reason applies verbatim here: a failed
/// upstream leaves a 0-byte file, and `mt encode --qr > rec.txt`
/// fails that way for a reason an operator meets on their first try — §8.2h
/// refuses a world-readable stdout, and `>` under the usual umask creates 0644.
///
/// **The empty container is the worse outcome, which is what earns the
/// refusal.** It flashes, it boots, and the device offers nothing — the same
/// silent-nothing shape as P3's F1, reached from the host side instead.
#[test]
fn an_empty_in_file_is_the_exit_2_path_too() {
    let dir = tempfile::tempdir().unwrap();
    let empty = dir.path().join("rec.txt");
    let out = dir.path().join("p.bin");
    std::fs::write(&empty, "").unwrap();
    me().args([
        "sysw",
        "pack",
        "--no-passphrase",
        "--out",
        out.to_str().unwrap(),
    ])
    .args(["--in", empty.to_str().unwrap()])
    .assert()
    .code(2)
    .stderr(predicate::str::contains("no records"));
    assert!(
        !out.exists(),
        "a refusal must leave no artifact — an empty container that exists is          one an operator can flash"
    );
    // Whitespace-only, the same: `split_record_stream` drops blank lines, so a
    // file of newlines is empty by the same definition stdin uses.
    let blanks = dir.path().join("blanks.txt");
    std::fs::write(&blanks, "\n\n\n").unwrap();
    me().args(["sysw", "pack", "--no-passphrase"])
        .args(["--in", blanks.to_str().unwrap()])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("no records"));
    // THE CONTROL: the same channel with one real record still packs, so the
    // guard is about emptiness and not about `--in`.
    let good = dir.path().join("good.txt");
    std::fs::write(&good, format!("{TEXT}\n")).unwrap();
    me().args([
        "sysw",
        "pack",
        "--no-passphrase",
        "--out",
        out.to_str().unwrap(),
    ])
    .args(["--in", good.to_str().unwrap()])
    .assert()
    .success();
}

/// The refusal NAMES THE FILE when the records came from `--in`. "no records:
/// pass them on argv, with --in, or on stdin" is advice to do the thing the
/// operator just did, and it is the message they meet when an upstream tool
/// failed — so it has to say which file was empty.
#[test]
fn the_empty_in_refusal_names_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let empty = dir.path().join("rec.txt");
    std::fs::write(&empty, "").unwrap();
    let a = me()
        .args(["sysw", "pack", "--no-passphrase"])
        .args(["--in", empty.to_str().unwrap()])
        .assert()
        .code(2);
    let err = String::from_utf8_lossy(&a.get_output().stderr).to_string();
    assert!(
        err.contains("rec.txt"),
        "name the file that was empty: {err}"
    );
}

/// The sentence §1.1 quoted must have changed: it named two channels and there
/// are now three. A message that still says "argv or with --in" teaches the
/// operator the pipeline does not exist.
#[test]
fn the_no_records_message_names_stdin() {
    let out = me()
        .args(["sysw", "pack", "--no-passphrase"])
        .write_stdin("")
        .assert()
        .code(2);
    let err = String::from_utf8_lossy(&out.get_output().stderr).to_string();
    assert!(
        err.contains("stdin"),
        "the refusal must name stdin now that it is a channel: {err}"
    );
}

/// argv still wins over stdin when both are present — otherwise every existing
/// invocation that also happens to have something on stdin changes meaning.
#[test]
fn argv_records_win_over_stdin() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    me().args([
        "sysw",
        "pack",
        "--no-passphrase",
        "--out",
        out.to_str().unwrap(),
        TEXT,
    ])
    .write_stdin(format!("{MD1}\n"))
    .assert()
    .success();
    me().args(["sysw", "show", out.to_str().unwrap()])
        .assert()
        .success()
        // The md1 arrived on stdin and must NOT be in the container.
        .stdout(predicate::str::contains("md1/mk1").not());
}

// ─── G-P3.5 / R2 — a `tx:` record on argv is REFUSED ─────────────────────────

/// The "even" vector: a real signed 222-byte transaction, as a `tx:` record.
const TX_RECORD: &str = "tx:020000000001017c8da925af70e49a12b0cea7b639df5037c87b7fa61f262b86ac32c47aa3ba1a0000000000fdffffff02404b4c0000000000160014c1de0dd435d1d4ad97ed1f51d63f91c800cc4eab3ea1b92901000000160014751097c299d6354fbb2c5a84512dd708f2902f5e0247304402207debc7d89984c7717940b622504318d2c184966a618b32cf8b700d0f125b3ffa02206ef875f9c0b5931e0ea1cf0c109bdb8512835c8e51526f99b3419929a2ea7259012103718f5fd45b926226357e2b0400574b41a32d0bf0ae69a02eebea5fbc542ff52060000000";

/// R2. argv is world-readable through `/proc/<pid>/cmdline`, `ps` shows it to
/// every user on the box, and the shell writes it to a history file that
/// outlives the machine. A raw signed transaction is a BEARER instrument, so
/// "prefer --in" is not enough — `mt` already refuses one as an argument for
/// exactly this reason and `me` must agree.
#[test]
fn a_tx_record_on_argv_is_refused() {
    me().args(["sysw", "pack", "--no-passphrase", TX_RECORD])
        .assert()
        .code(3)
        .stdout(predicate::str::is_empty());
}

/// THE POINT OF THE REFUSAL IS THAT NOTHING IS EMITTED FIRST. A guard placed
/// downstream of the work it exists to prevent has already lost: this exact
/// shape (output emitted before the validation that would have made it
/// unnecessary) is why `mt`'s §8.2f was bypassed by the invocation it refused.
///
/// So: no container on stdout, no generated passphrase on stderr, and above
/// all NOT ONE BYTE OF THE TRANSACTION anywhere in either stream.
#[test]
fn the_argv_refusal_echoes_neither_the_transaction_nor_a_passphrase() {
    let body = TX_RECORD.strip_prefix("tx:").unwrap();
    // No --no-passphrase: the DEFAULT path generates one and prints it. The
    // refusal must beat that ceremony, or the operator writes down a
    // passphrase for a container that was never built.
    let out = me()
        .args(["sysw", "pack", TX_RECORD])
        .assert()
        .code(3)
        .get_output()
        .clone();
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        !err.contains(body) && !stdout.contains(body),
        "the refusal echoed the transaction:\nstderr: {err}\nstdout: {stdout}"
    );
    // A 32-character prefix would be enough to identify the artifact too.
    assert!(
        !err.contains(&body[..32]),
        "the refusal echoed a prefix: {err}"
    );
    assert!(
        !err.contains("write this down"),
        "a passphrase was generated before the refusal ran: {err}"
    );
    assert!(
        err.contains("--in") || err.contains("stdin"),
        "the refusal must name the private channel that works: {err}"
    );
}

/// The private channels still take it — the refusal is about the CHANNEL, not
/// about the record. Both, because a refusal that also broke `--in` would make
/// the class unpackable rather than safely packable.
#[test]
fn the_same_tx_record_packs_from_in_and_from_stdin() {
    let dir = tempfile::tempdir().unwrap();
    let recs = dir.path().join("recs.txt");
    std::fs::write(&recs, format!("{TX_RECORD}\n")).unwrap();
    me().args([
        "sysw",
        "pack",
        "--no-passphrase",
        "--in",
        recs.to_str().unwrap(),
        "--out",
        dir.path().join("a.bin").to_str().unwrap(),
    ])
    .assert()
    .success();
    me().args([
        "sysw",
        "pack",
        "--no-passphrase",
        "--out",
        dir.path().join("b.bin").to_str().unwrap(),
    ])
    .write_stdin(format!("{TX_RECORD}\n"))
    .assert()
    .success();
    assert_eq!(
        std::fs::read(dir.path().join("a.bin")).unwrap(),
        std::fs::read(dir.path().join("b.bin")).unwrap(),
        "the two private channels must build the same container"
    );
}

/// A `tx:` record hidden among ordinary ones is still refused, and the refusal
/// names WHICH position — the operator has to be able to find it.
///
/// **NOW COVERS BOTH GATES, where it used to reach only one.** P0 added a
/// pre-parser argv guard that decides before `Cli::parse()`, so a
/// classifier-recognisable `tx:` record is refused there and located by its
/// **argv position**. The donor's post-parse gate is still reachable and still
/// locates by **record index**: its `tx:` PREFIX arm catches a body the
/// classifier cannot decode, which is deliberate — a near-miss is refused for
/// the BEARER reason rather than three screens later for a formatting one.
///
/// Asserting both is strictly more coverage than the single `"record 2"` this
/// test carried before, and it is what keeps the post-parse arm from rotting
/// unnoticed now that the guard shadows it for every well-formed record.
#[test]
fn a_tx_record_anywhere_on_argv_is_refused_and_located() {
    // The PRE-PARSER guard: a decodable transaction, located by argv position.
    // `me sysw pack --no-passphrase TEXT MD1 TX_RECORD` puts it at argv 6.
    let out = me()
        .args(["sysw", "pack", "--no-passphrase", TEXT, MD1, TX_RECORD])
        .assert()
        .code(3)
        .get_output()
        .clone();
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        err.contains("argument 6"),
        "the pre-parser refusal must locate it: {err}"
    );
    assert!(
        !err.contains(&TX_RECORD[..40]),
        "and must never echo the body back: {err}"
    );

    // The POST-PARSE gate, still reached: `tx:0100` does not decode, so
    // `classify` calls it Unknown and only the prefix arm sees it. Located by
    // RECORD index, which is what the operator counts when passing records.
    let out = me()
        .args(["sysw", "pack", "--no-passphrase", TEXT, MD1, "tx:0100"])
        .assert()
        .code(3)
        .get_output()
        .clone();
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        err.contains("record 2"),
        "the post-parse refusal must locate it by record index: {err}"
    );
}

/// NOT over-broad: `text:`/`pass:`/md1/mnemonic records on argv keep working,
/// because argv there is a documented convenience and only the transaction
/// class is bearer-by-construction. A guard that swept up everything would be
/// a different (and unruled) change.
#[test]
fn the_argv_refusal_is_scoped_to_the_transaction_class() {
    me().args(["sysw", "pack", "--no-passphrase", TEXT, MD1])
        .assert()
        .success();
}

// ─── G-P3.3 — `--allow-unsigned-inputs` (FORWARD_PLAN §2.1) ──────────────────

/// `EVEN` with every witness stripped: 113 bytes, and its txid is byte-for-byte
/// the honest transaction's, because stripping the witness is precisely the
/// operation the txid is defined to ignore.
const TX_STRIPPED: &str = "02000000017c8da925af70e49a12b0cea7b639df5037c87b7fa61f262b86ac32c47aa3ba1a0000000000fdffffff02404b4c0000000000160014c1de0dd435d1d4ad97ed1f51d63f91c800cc4eab3ea1b92901000000160014751097c299d6354fbb2c5a84512dd708f2902f5e60000000";

/// TWO inputs: input 0 legacy and still carrying its scriptSig, input 1 a
/// segwit input whose witness was removed. A whole-transaction predicate
/// passes this; only the per-INPUT one names input 1.
const TX_MIXED_STRIPPED: &str = "020000000211111111111111111111111111111111111111111111111111111111111111110000000048473030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030ffffffff22222222222222222222222222222222222222222222222222222222222222220100000000ffffffff0150c3000000000000160014333333333333333333333333333333333333333300000000";

/// The refusal must NAME THE FAILING INPUTS, not just assert unsignedness.
/// "an input is unsigned" sends the operator back to a wallet with nothing to
/// look at; "input 1" is a place to look.
#[test]
fn the_unsigned_refusal_names_the_failing_input_indices() {
    let out = me()
        .args(["sysw", "pack", "--no-passphrase"])
        .write_stdin(format!("tx:{TX_MIXED_STRIPPED}\n"))
        .assert()
        .code(4)
        .get_output()
        .clone();
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        err.contains("input 1"),
        "must name the failing input: {err}"
    );
    assert!(
        !err.contains("input 0"),
        "input 0 IS signed and must not be named: {err}"
    );
    assert!(
        err.contains("--allow-unsigned-inputs"),
        "the refusal must name the override that exists for honest exotica: {err}"
    );
}

/// THE OVERRIDE ITSELF. It exists because the predicate has honest
/// false-positives — a P2A anchor-spend input carries neither scriptSig nor
/// witness and is perfectly valid — and a check with no escape hatch becomes a
/// reason to stop using the tool.
#[test]
fn allow_unsigned_inputs_admits_the_record_and_names_what_it_admitted() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    let res = me()
        .args([
            "sysw",
            "pack",
            "--no-passphrase",
            "--allow-unsigned-inputs",
            "--out",
            out.to_str().unwrap(),
        ])
        .write_stdin(format!("tx:{TX_STRIPPED}\n"))
        .assert()
        .success();
    let err = String::from_utf8_lossy(&res.get_output().stderr).to_string();
    // Loud, and specific: which record, which inputs, and what it costs.
    assert!(err.contains("record 0"), "{err}");
    assert!(
        err.contains("input 0"),
        "must name the input it let through: {err}"
    );
    assert!(
        err.contains("--allow-unsigned-inputs"),
        "the warning must name the flag that caused it: {err}"
    );
    assert!(
        err.to_lowercase().contains("broadcast"),
        "the warning must say what the operator loses: {err}"
    );
    // AND THE RECORD IS REALLY IN THERE. `show` is a READER and reads
    // strictly, so it must not describe this one as a signed transaction --
    // but it must not be SILENT about it either. A container whose `show`
    // omits a record it holds is the worst of both.
    me().args(["sysw", "show", out.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("UNSIGNED"))
        .stdout(predicate::str::contains("input 0"))
        .stdout(predicate::str::contains(MT_EVEN_TXID));
}

/// `show` reports an unsigned `tx:` record it finds in a container, whoever
/// wrote it. The strict reader classifies it `Unknown`, so before this it
/// listed nothing at all: the operator saw a 229-byte public section and no
/// account of what was in it.
#[test]
fn show_names_an_unsigned_tx_record_rather_than_omitting_it() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    me().args([
        "sysw",
        "pack",
        "--no-passphrase",
        "--allow-unsigned-inputs",
        "--out",
        out.to_str().unwrap(),
    ])
    .write_stdin(format!("tx:{TX_MIXED_STRIPPED}\n"))
    .assert()
    .success();
    let res = me()
        .args(["sysw", "show", out.to_str().unwrap()])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&res.get_output().stdout).to_string();
    assert!(stdout.contains("public record 0"), "{stdout}");
    assert!(
        stdout.contains("input 1"),
        "must name the failing input: {stdout}"
    );
    assert!(
        !stdout.contains("input 0"),
        "input 0 is signed and must not be named: {stdout}"
    );
    assert!(
        stdout.to_lowercase().contains("broadcast"),
        "must say what it costs: {stdout}"
    );
}

/// The override is SCOPED to the signature predicate. Everything else the
/// `tx:` prefix requires still refuses with it set — otherwise the flag would
/// be a general "admit anything" switch, which is not what it was ruled to be.
#[test]
fn allow_unsigned_inputs_loosens_nothing_else() {
    for (body, want) in [
        ("tx:abab", "not one serialized Bitcoin transaction"),
        ("tx:zz", "not lowercase hex"),
    ] {
        me().args(["sysw", "pack", "--no-passphrase", "--allow-unsigned-inputs"])
            .write_stdin(format!("{body}\n"))
            .assert()
            .code(4)
            .stderr(predicate::str::contains(want));
    }
}

/// It names ONLY the unsigned inputs of a mixed transaction — the mutation
/// that turns the per-input predicate into a whole-transaction one leaves the
/// single-input vector green and reddens this.
#[test]
fn the_override_reports_exactly_the_unsigned_inputs_of_a_mixed_transaction() {
    let res = me()
        .args(["sysw", "pack", "--no-passphrase", "--allow-unsigned-inputs"])
        .write_stdin(format!("tx:{TX_MIXED_STRIPPED}\n"))
        .assert()
        .success();
    let err = String::from_utf8_lossy(&res.get_output().stderr).to_string();
    assert!(err.contains("input 1"), "{err}");
    assert!(!err.contains("input 0"), "input 0 is signed: {err}");
}

/// A SIGNED transaction produces no warning at all with the flag set. A flag
/// that shouts on every payload trains the operator to ignore it.
#[test]
fn the_override_is_silent_when_nothing_needed_it() {
    let res = me()
        .args(["sysw", "pack", "--no-passphrase", "--allow-unsigned-inputs"])
        .write_stdin(format!("tx:{MT_EVEN_RAW_HEX}\n"))
        .assert()
        .success();
    let err = String::from_utf8_lossy(&res.get_output().stderr).to_string();
    assert!(
        !err.contains("--allow-unsigned-inputs"),
        "a fully-signed transaction must not trip the override warning: {err}"
    );
}

// ─── G-P3.6 — sealing is decided by CONTENT, and says so ─────────────────────

/// SPEC §2.4. `me sysw pack` sealed by DEFAULT — right for a mnemonic, wrong
/// for a transaction, and contrary to the operator's 2026-08-23 ruling *"send
/// via payload unencrypted"*. Sealing a transaction payload costs a 12-word
/// passphrase to store, those 12 words typed on the device's on-screen
/// keyboard, ~31 s of on-device KDF, and a new way to lose the backup — all to
/// protect a payload whose whole purpose is to become a steel plate anyone can
/// read.
#[test]
fn a_payload_with_no_secret_record_is_not_sealed_by_default() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    let res = me()
        .args(["sysw", "pack", "--out", out.to_str().unwrap(), MD1, TEXT])
        .assert()
        .success();
    let err = String::from_utf8_lossy(&res.get_output().stderr).to_string();
    assert!(
        !err.contains("write this down"),
        "no passphrase should have been generated: {err}"
    );
    me().args(["sysw", "show", out.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("sealed:   false"))
        .stdout(predicate::str::contains("ct_len:   0"));
}

/// The other half of the SAME rule, and the half that must not regress: a
/// payload holding a BIP-39 mnemonic still seals with no flag at all.
#[test]
fn a_payload_holding_secret_material_is_still_sealed_by_default() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    me().args(["sysw", "pack", "--out", out.to_str().unwrap()])
        .write_stdin(SEED)
        .assert()
        .success()
        .stderr(predicate::str::contains("write this down"));
    me().args(["sysw", "show", out.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("sealed:   true"));
}

/// **"It MUST say which way it went, and why, on stderr, EVERY TIME."** A
/// content-dependent default that is silent is worse than the default it
/// replaces: the operator cannot tell a deliberate cleartext container from a
/// flag they forgot.
#[test]
fn pack_states_which_way_sealing_went_and_why_every_time() {
    // Four invocations, four sentences: content-unsealed, content-sealed,
    // flag-unsealed, flag-sealed.
    let cases: [(Vec<&str>, &str, &str); 5] = [
        (vec![MD1], "NOT SEALED", "no record"),
        (
            vec!["--allow-argv-secret", SEED],
            "SEALED",
            "secret material",
        ),
        (
            vec!["--no-passphrase", "--allow-argv-secret", SEED],
            "NOT SEALED",
            "--no-passphrase",
        ),
        (
            vec!["--passphrase-words", "4", "--allow-argv-secret", SEED],
            "SEALED",
            "--passphrase-words",
        ),
        // The flag CANNOT seal a payload with nothing secret in it, and the
        // line must say so rather than claim a protection that does not exist.
        (
            vec!["--passphrase-words", "4", MD1],
            "NOT SEALED",
            "IGNORED",
        ),
    ];
    for (extra, verdict, why) in cases {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("p.bin");
        let mut args = vec!["sysw", "pack", "--out", out.to_str().unwrap()];
        args.extend(extra.iter().copied());
        let res = me().args(&args).assert().success();
        let err = String::from_utf8_lossy(&res.get_output().stderr).to_string();
        assert!(
            err.contains(verdict),
            "`me {}` never said {verdict}: {err}",
            args.join(" ")
        );
        assert!(
            err.contains(why),
            "`me {}` said {verdict} without saying why ({why}): {err}",
            args.join(" ")
        );
    }
}

/// The two verdicts are DISTINGUISHABLE, not one sentence with a word swapped
/// — "NOT SEALED" contains "SEALED", so a naive contains() check passes on
/// both and would let the two collapse without a test noticing.
#[test]
fn the_sealed_and_unsealed_lines_are_two_sentences() {
    let grab = |args: &[&str]| -> String {
        let res = me().args(args).write_stdin("").assert();
        String::from_utf8_lossy(&res.get_output().stderr)
            .lines()
            .find(|l| l.contains("SEALED"))
            .unwrap_or("")
            .to_string()
    };
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.bin");
    let b = dir.path().join("b.bin");
    let unsealed = grab(&["sysw", "pack", "--out", a.to_str().unwrap(), MD1]);
    let sealed = grab(&[
        "sysw",
        "pack",
        "--out",
        b.to_str().unwrap(),
        "--allow-argv-secret",
        SEED,
    ]);
    assert!(
        !unsealed.is_empty() && !sealed.is_empty(),
        "{unsealed:?} {sealed:?}"
    );
    assert_ne!(unsealed, sealed);
    assert!(unsealed.contains("NOT SEALED"), "{unsealed}");
    assert!(!sealed.contains("NOT SEALED"), "{sealed}");
}

/// The ruled pipeline, whole: a transaction payload packs UNSEALED with no
/// flags at all, which is the invocation §2.4 exists to make correct.
#[test]
fn a_transaction_payload_packs_unsealed_with_no_flags() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    me().args(["sysw", "pack", "--out", out.to_str().unwrap()])
        .write_stdin(format!("tx:{MT_EVEN_RAW_HEX}\n"))
        .assert()
        .success()
        .stderr(predicate::str::contains("NOT SEALED"));
    me().args(["sysw", "show", out.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("sealed:   false"));
}

/// THE HONESTY INVARIANT, and it is the strongest form this can take: whatever
/// `pack` says on stderr must be what `show` reads back out of the file. A
/// message cannot lie if a second program has to agree with it.
///
/// This is the test that caught the defect G-P3.6 turned up.
/// `me sysw pack --passphrase-words 4 <md1>` printed a passphrase, told the
/// operator to store it APART FROM THE MACHINE -- and wrote a container with
/// `sealed: false, ct_len: 0`. `pack` moves only SECRET records into the
/// ciphertext, so with none there the plaintext was empty, `sealed()` is
/// `ct_len > 0`, and the 16-byte AEAD tag landed past `total_len()` where
/// nothing authenticates it. The passphrase protected nothing and opened
/// nothing, and the operator was told to keep it forever.
#[test]
fn what_pack_says_about_sealing_is_what_show_reads_back() {
    let cases: [Vec<&str>; 6] = [
        vec![MD1],
        vec!["--allow-argv-secret", SEED],
        vec![TEXT, MD1],
        vec!["--no-passphrase", "--allow-argv-secret", SEED],
        vec!["--passphrase-words", "4", "--allow-argv-secret", SEED],
        vec!["--passphrase-words", "4", MD1],
    ];
    for extra in cases {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("p.bin");
        let mut args = vec!["sysw", "pack", "--out", out.to_str().unwrap()];
        args.extend(extra.iter().copied());
        let res = me().args(&args).assert().success();
        let err = String::from_utf8_lossy(&res.get_output().stderr).to_string();
        let claimed_sealed = err.contains("sealing:  SEALED");
        let shown = me()
            .args(["sysw", "show", out.to_str().unwrap()])
            .assert()
            .success();
        let stdout = String::from_utf8_lossy(&shown.get_output().stdout).to_string();
        let really_sealed = stdout.contains("sealed:   true");
        assert_eq!(
            claimed_sealed,
            really_sealed,
            "`me {}` claimed sealed={claimed_sealed} and the file says {really_sealed}\n\
             stderr: {err}\nshow: {stdout}",
            args.join(" ")
        );
        // AND: a passphrase is minted only when it opens something.
        assert_eq!(
            err.contains("write this down"),
            really_sealed,
            "`me {}` minted a passphrase for a container it does not open",
            args.join(" ")
        );
    }
}

// ─── G-P3.7 — "loudly" is NORMATIVE (ruling 2026-08-25) ──────────────────────

/// The ruling says the report MUST name the set and **every** missing index,
/// not the first — r7-M1 — and `me sysw show` must do the same. Measured
/// before this gate: one line per record saying only *"an mt1 chunk whose set
/// this tool could not confirm"*, with no set id and no indices, so an
/// operator holding 201 of 202 strings was told nothing about which one to go
/// and find.
///
/// Two gone and NOT adjacent, so "the first missing one" is visibly not the
/// answer. Chunk numbers are 1-based on every operator-facing surface, which
/// is `mt`'s own convention (SPEC_mt §1.1) — the wire index is 0-based and
/// appears in no message.
#[test]
fn the_incomplete_report_names_the_set_and_every_missing_string() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    let res = me()
        .args([
            "sysw",
            "pack",
            "--no-passphrase",
            "--out",
            out.to_str().unwrap(),
        ])
        .write_stdin(format!("{}\n{}\n{}\n", MT_EVEN[0], MT_EVEN[2], MT_EVEN[5]))
        .assert()
        .success();
    let err = String::from_utf8_lossy(&res.get_output().stderr).to_string();
    assert!(
        err.contains("mt1 set 2dcf2"),
        "must name the chunk_set_id: {err}"
    );
    // 0-based 1, 3, 4 are absent -> strings 2, 4 and 5 of 6, 1-BASED, which is
    // `mt`'s own operator-facing convention (the wire index appears in no
    // message). Asserted as ONE PHRASE rather than as four digits: "2", "4",
    // "5" and "6" all occur in "set 2dcf2 (records 0, 1, 2" too, so a
    // digit-at-a-time check passes a report that names only the first missing
    // string -- which is exactly the r7-M1 defect the ruling calls out.
    assert!(
        err.contains("MISSING strings 2, 4 and 5 of 6"),
        "the report must name EVERY missing string against the header's count: {err}"
    );

    // AND `me sysw show` says the same thing, because a stderr line is gone in
    // a week and `show` is the one that can be re-run.
    let shown = me()
        .args(["sysw", "show", out.to_str().unwrap()])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&shown.get_output().stdout).to_string();
    assert!(
        stdout.contains("mt set 2dcf2: INCOMPLETE"),
        "the ruling requires show to mark the SET incomplete: {stdout}"
    );
    assert!(
        stdout.contains("MISSING strings 2, 4 and 5 of 6"),
        "show must carry the same indices pack printed: {stdout}"
    );
}

/// The five failures are five different sentences, because the remedies are
/// not close: find more strings / re-encode from the transaction / re-export
/// from the signer / throw the payload away.
#[test]
fn the_report_distinguishes_why_a_set_did_not_confirm() {
    let dir = tempfile::tempdir().unwrap();
    let n = std::cell::Cell::new(0);
    let say = |records: &str| -> String {
        n.set(n.get() + 1);
        let out = dir.path().join(format!("p{}.bin", n.get()));
        let res = me()
            .args([
                "sysw",
                "pack",
                "--no-passphrase",
                "--out",
                out.to_str().unwrap(),
            ])
            .write_stdin(records.to_string())
            .assert()
            .success();
        String::from_utf8_lossy(&res.get_output().stderr).to_string()
    };

    // (1) MISSING material. Cheap remedy: go and find the other five strings.
    let incomplete = say(&format!("{}\n", MT_EVEN[0]));
    assert!(
        incomplete.to_lowercase().contains("missing"),
        "{incomplete}"
    );

    // (2) WRONG material -- 32 bytes of entropy wrapped as a complete 1-chunk
    // set. Reassembles; is not a transaction. The C3 smuggling channel.
    let smuggled = say(&format!("{MT_SMUGGLED}\n"));
    assert!(
        smuggled.contains("NOT") && smuggled.contains("transaction"),
        "the smuggling case must say the bytes are not a transaction: {smuggled}"
    );

    // (3) A REAL signed transaction under a FOREIGN set id: complete, parses,
    // and the txid does not carry the set id every string declares.
    let forged = say(&(MT_FORGED.join("\n") + "\n"));
    assert!(
        forged.contains(MT_EVEN_TXID),
        "must name the txid it derived: {forged}"
    );
    assert!(
        forged.contains("00000"),
        "must name the declared set id: {forged}"
    );

    // (4) A set carrying an UNSIGNED transaction: complete, parses, and BINDS
    // -- stripping the witnesses leaves the txid alone. Nothing else can see it.
    let stripped = say(&(MT_STRIPPED.join("\n") + "\n"));
    assert!(
        stripped.contains("scriptSig") && stripped.contains("input 0"),
        "must name the unsigned input: {stripped}"
    );

    // FOUR FAILURES, FOUR SENTENCES. The collapse this exists to prevent is
    // one message serving all of them.
    let all = [&incomplete, &smuggled, &forged, &stripped];
    for i in 0..all.len() {
        for j in i + 1..all.len() {
            assert_ne!(all[i], all[j], "reports {i} and {j} are the same sentence");
        }
    }
}

/// 32 bytes of entropy as a complete, BCH-valid 1-chunk mt1 set. Reassembles;
/// is not a transaction.
const MT_SMUGGLED: &str =
    "mt1pm6kmqqqqqq4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w4sqxxtg7uwrnug7";

/// The EVEN transaction re-encoded under set id 0x00000 — every string valid,
/// the set complete, and the txid binds to 0x2dcf2 instead.
const MT_FORGED: [&str; 6] = [
    "mt1pqqqqqq9qqqqgqqqqqqqyqherdfykhhpey6z2cvafak8804qd7g0dl6v8ex9wr2cvky023shuayazhuzld98",
    "mt1pqqqqqq9qqphgdqqqqqqqq0mllllupyqj6vqqqqqqqqzcqpfsw7ph2rt5w54kt768636clsxsd4wuqyhcptq",
    "mt1pqqqqqq9qqzj8yqpnzw4vl2rwffqyqqqqqkqq282yyhc2vavd20hvk94pz39hts3u5s9a0qv42n3emlhy0y4",
    "mt1pqqqqqq9qqrqfrnq3qzyp77h37cnxzvwutegzmzy5zrrrfvrpykdfsckvk03dcq6rcjtvlsg2rzd3lsate9r",
    "mt1pqqqqqq9qqylgpzqmhcwhuupdvnrc82rncvzzdahpgjsdwgu52jd7vmxsve9x3w5ujeqyssa7xs8rnk2rg5c",
    "mt1pqqqqqq9qq9qdcc7h75twfxyf340c4sgqzhfdq6xtgt7zhxngpwa049l0z59l6jqcqqqqqq4ylettyzmk6ug",
];

/// The 113-byte SIGNATURE-STRIPPED EVEN transaction as a complete 3-chunk set.
/// It parses, and it BINDS -- its txid is the honest transaction's, because
/// stripping the witness is exactly what the txid ignores.
const MT_STRIPPED: [&str; 3] = [
    "mt1p9h8jqqzqqqqgqqqqqp0jx6jfd0wrjf5y4se6nmvwwl2qmus7ml5c0jv2ux4sevg74rhgdqqhgq73ru3s5kep",
    "mt1p9h8jqqzqqpqqqqqq8allll7qjqfdxqqqqqqqqpvqq5c80qm4p46822m9ldragav0u3eqqvcwzfhcyyza74xq",
    "mt1p9h8jqqzqqzf64nagde9yqsqqqqzcqpgagsjlpfn434f7ajck5y2ykawz8jjqh4ucqqqqqq774jy98z3xll2",
];

/// The ruling's THIRD requirement, and the one that makes the other two worth
/// having: it still PACKS. Nothing in the chunk path refuses.
#[test]
fn an_incomplete_set_still_packs_and_is_readable() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    me().args([
        "sysw",
        "pack",
        "--no-passphrase",
        "--out",
        out.to_str().unwrap(),
    ])
    .write_stdin(format!("{}\n{}\n", MT_EVEN[0], MT_EVEN[2]))
    .assert()
    .success();
    me().args(["sysw", "show", out.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("pub_len:  175"));
}

// ─── G-P3.16 — one command, named on both sides of the air gap ──────────────

/// SPEC §3.2. The DEVICE's compare screen used to say *"Compare this against
/// what `me sysw pack` printed"* — the WRITE path. Re-running `pack` means
/// re-supplying every record and re-running the ceremony, and on the sealed
/// path it mints a fresh passphrase. The operator standing at the machine has
/// the FILE. `me sysw show <file>` reads what they have.
///
/// So `pack` names the same command the device names. A pointer that exists on
/// only one side of an air gap is a pointer the operator has to invent.
#[test]
fn pack_points_at_the_command_the_device_names() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    let res = me()
        .args([
            "sysw",
            "pack",
            "--no-passphrase",
            "--out",
            out.to_str().unwrap(),
            MD1,
        ])
        .assert()
        .success();
    let err = String::from_utf8_lossy(&res.get_output().stderr).to_string();
    assert!(
        err.contains("me sysw show"),
        "pack must name the command that re-prints the digest: {err}"
    );
    assert!(
        err.contains(out.to_str().unwrap()),
        "and the FILE, so it can be pasted: {err}"
    );
}

/// On stdout there is no path to name, so it says so rather than printing a
/// command that cannot be run.
#[test]
fn the_pointer_admits_it_does_not_know_the_path_on_stdout() {
    let res = me()
        .args([
            "sysw",
            "pack",
            "--no-passphrase",
            "--allow-world-readable",
            MD1,
        ])
        .assert()
        .success();
    let err = String::from_utf8_lossy(&res.get_output().stderr).to_string();
    assert!(err.contains("me sysw show"), "{err}");
    assert!(
        err.contains("the file you just wrote"),
        "it must not invent a path it does not have: {err}"
    );
}

/// AND THE COMMAND IT NAMES REALLY PRINTS THE SAME NUMBER. A pointer to a
/// command that prints something else is worse than no pointer: the operator
/// compares two numbers that were never meant to match and concludes the
/// payload is tampered with.
#[test]
fn the_named_command_prints_the_same_digest() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    let packed = me()
        .args([
            "sysw",
            "pack",
            "--no-passphrase",
            "--out",
            out.to_str().unwrap(),
            MD1,
            TEXT,
        ])
        .assert()
        .success();
    let from_pack = digest_line(&String::from_utf8_lossy(&packed.get_output().stderr))
        .expect("pack prints a digest")
        .to_string();
    let shown = me()
        .args(["sysw", "show", out.to_str().unwrap()])
        .assert()
        .success();
    let from_show = digest_line(&String::from_utf8_lossy(&shown.get_output().stderr))
        .expect("show prints a digest")
        .to_string();
    assert_eq!(from_pack, from_show);
}

// ── F-246: no line describing a container may print before the write gate ────

/// **A DIGEST FOR A PAYLOAD THAT WAS NEVER WRITTEN.**
///
/// The walk typed a bare `me sysw pack`, pasted a record, and got `sealing:`,
/// `strength:`, `digest:` and *"re-print it with: me sysw show &lt;the file you
/// just wrote&gt;"* — and only then the §8.2h refusal, exit 2, with a 0-byte
/// file. The digest is the value the operator verifies the PLATE against on the
/// device, so recording it means carrying a checksum for a payload that does
/// not exist, under a line that is false as it prints.
///
/// This is the rule the passphrase ceremony already follows two screens up
/// (*"generating a passphrase, telling the operator to write it down, and THEN
/// refusing the container teaches them that the note they just made is
/// worthless"*), applied to the gate that can abort the write.
#[test]
fn a_refused_write_prints_no_line_describing_the_container() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let sink = dir.path().join("blob.bin");
    let handle = std::fs::File::create(&sink).unwrap();
    std::fs::set_permissions(&sink, std::fs::Permissions::from_mode(0o644)).unwrap();

    let o = std::process::Command::new(assert_cmd::cargo::cargo_bin("me"))
        .args(["sysw", "pack", "--no-passphrase", TEXT])
        .stdout(std::process::Stdio::from(handle))
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();

    assert!(!o.status.success(), "§8.2h must refuse a 0644 stdout");
    assert_eq!(
        std::fs::metadata(&sink).unwrap().len(),
        0,
        "a refusal must leave no artifact"
    );
    let err = String::from_utf8_lossy(&o.stderr).to_string();

    for described in ["digest:", "sealing:", "strength:"] {
        assert!(
            !err.contains(described),
            "{described:?} describes a container that was never written: {err}"
        );
    }
    assert!(
        !err.contains("the file you just wrote"),
        "and nothing may refer to a file that does not exist: {err}"
    );
    // The refusal ITSELF must still be there and still be useful.
    assert!(
        err.contains("world-readable"),
        "the refusal survives: {err}"
    );
    assert!(err.contains("--out"), "with its remedies: {err}");
}

/// THE CONTROL: on a run that actually writes, every one of those lines is
/// still printed. Without this, deleting the report entirely would pass above.
#[test]
fn a_successful_pack_still_reports_everything() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    let a = me()
        .args(["sysw", "pack", "--no-passphrase", "--out"])
        .arg(&out)
        .arg(TEXT)
        .assert()
        .success();
    let err = String::from_utf8_lossy(&a.get_output().stderr).to_string();
    for described in ["digest:", "sealing:", "strength:"] {
        assert!(err.contains(described), "{described:?} missing: {err}");
    }
    assert!(out.exists(), "and the file exists");
}

/// **F-246's ORIGINAL instance: a passphrase generated, printed, and told to
/// the operator to "write down and store APART from the machine" — for a run
/// that then refuses and produces nothing.**
///
/// The passphrase is meant to be shown; that is not the defect. The defect is
/// showing it for a run that produces NOTHING, so the operator is handed
/// material to record off-machine that protects no artifact, immediately above
/// an error saying the run failed.
#[test]
fn an_unpackable_record_is_refused_before_a_passphrase_is_minted() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("t.bin");
    let a = me()
        .args(["sysw", "pack", "--passphrase-words", "12", "--out"])
        .arg(&out)
        .arg("this is not a record of any class")
        .assert()
        .failure();
    let err = String::from_utf8_lossy(&a.get_output().stderr).to_string();

    assert!(
        !err.contains("write this down"),
        "no passphrase ceremony for a run that produces nothing: {err}"
    );
    assert!(
        !err.contains("strength:"),
        "and nothing describing the container either: {err}"
    );
    assert!(
        err.contains("not a form this container can place"),
        "the real refusal must still be the one shown: {err}"
    );
    assert!(!out.exists(), "and no artifact");
}

/// THE CONTROL: a run whose records ARE admissible still mints and prints the
/// passphrase. Without this, never minting one would pass the test above.
#[test]
fn an_admissible_record_still_gets_its_passphrase_ceremony() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("ok.bin");
    let a = me()
        .args(["sysw", "pack", "--passphrase-words", "12", "--out"])
        .arg(&out)
        .write_stdin(SEED)
        .assert()
        .success();
    let err = String::from_utf8_lossy(&a.get_output().stderr).to_string();
    assert!(err.contains("write this down"), "ceremony missing: {err}");
    assert!(err.contains("strength:"), "strength missing: {err}");
}

// ── F-252: name the MODE; claim nothing about reachability ───────────────────

/// **The guard measures a mode. The message asserted a FACT it never checked.**
///
/// `stdout_is_world_readable` is `fstat(1)` + `mode & 0o044`. POSIX requires
/// search permission on every directory in a path, so a 0644 file beneath a
/// 0700 ancestor cannot be opened by anyone else — and `$HOME` is 0700 on the
/// operator's machine, which makes "world-readable" false for the commonest
/// destination there is. Measured 2026-08-25: a `tempdir()` (0700) holding a
/// 0644 file is unreachable, and `me` refused it anyway — as it does here.
///
/// **The guard is unchanged and must stay:** a 0644 file becomes readable the
/// moment it is moved, copied, or its parent relaxed, and this is bearer
/// material. Only the sentence changes.
#[test]
fn the_world_readable_refusal_names_the_mode_and_claims_no_more() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let sink = dir.path().join("blob.bin");
    let handle = std::fs::File::create(&sink).unwrap();
    std::fs::set_permissions(&sink, std::fs::Permissions::from_mode(0o644)).unwrap();
    let o = std::process::Command::new(assert_cmd::cargo::cargo_bin("me"))
        .args(["sysw", "pack", "--no-passphrase", TEXT])
        .stdout(std::process::Stdio::from(handle))
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert!(!o.status.success());
    let err = String::from_utf8_lossy(&o.stderr).to_string();

    assert!(err.contains("0644"), "name the mode it measured: {err}");
    assert!(
        !err.contains("stdout is a world-readable file"),
        "do not assert reachability the guard never established: {err}"
    );
    assert!(
        err.contains("directory above"),
        "say what was NOT checked, so the operator can judge: {err}"
    );
    // The three remedies survive — they are the part `mt` was measured against
    // and found equal, and they are why this refusal is useful at all.
    for r in ["--out", "umask 077", "--allow-world-readable"] {
        assert!(err.contains(r), "remedy {r:?} lost: {err}");
    }
}

/// **THE CONTROL for F-253, and the one that matters most.** The ruled pipeline
/// is `mt encode --qr | me sysw pack`, so a PIPE must still receive the
/// container. If the terminal refusal were written as "refuse whenever there is
/// no `--out`", this goes red — and the feature's whole reason for existing
/// would be gone.
#[test]
fn a_pipe_still_receives_the_container() {
    let a = me()
        .args(["sysw", "pack", "--no-passphrase", TEXT])
        .assert()
        .success();
    let out = &a.get_output().stdout;
    assert!(!out.is_empty(), "a pipe must still get the bytes");
    assert_eq!(&out[..8], b"MNEMSYSW", "and they must be the container");
}

/// A redirect to a 0600 file is a Stream too — not a terminal, and not refused
/// by the mode guard either, so the bytes land.
#[test]
fn a_private_redirect_still_receives_the_container() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let sink = dir.path().join("p.bin");
    let handle = std::fs::File::create(&sink).unwrap();
    std::fs::set_permissions(&sink, std::fs::Permissions::from_mode(0o600)).unwrap();
    let o = std::process::Command::new(assert_cmd::cargo::cargo_bin("me"))
        .args(["sysw", "pack", "--no-passphrase", TEXT])
        .stdout(std::process::Stdio::from(handle))
        .output()
        .unwrap();
    assert!(o.status.success(), "a 0600 redirect is fine");
    assert!(
        std::fs::metadata(&sink).unwrap().len() > 0,
        "and it received the container"
    );
}

// ── F-251: the help tree must name the operator's goal ───────────────────────

/// **The operator held a `tx:` record and typed `me -h` "because I want to
/// start engraving a QR coded tx". Not one word on the screen matched.**
///
/// Measured across the tree at the time: `me -h` and `me sysw help` scored 0
/// for both "transaction" and "QR"; `me sysw pack -h` scored 1 and 0. The one
/// sentence that routes an operator to QR plates sat in paragraph 3 of `pack`'s
/// doc comment, so clap rendered it under `--help` (73 lines) and never under
/// `-h` (19). They typed `-h` twice.
///
/// **This asserts `-h`, deliberately.** Clap's short/long split is a sound
/// convention and is not the defect; putting the load-bearing sentence outside
/// the form everyone types is.
#[test]
fn the_help_tree_names_transactions_at_every_level_an_operator_types() {
    let short = |args: &[&str]| -> String {
        let mut c = me();
        c.args(args);
        let o = c.arg("-h").output().unwrap();
        String::from_utf8_lossy(&o.stdout).to_lowercase()
            + &String::from_utf8_lossy(&o.stderr).to_lowercase()
    };

    let top = short(&[]);
    assert!(
        top.contains("transaction"),
        "`me -h` must name transactions: {top}"
    );

    let sysw = short(&["sysw"]);
    assert!(
        sysw.contains("transaction") || sysw.contains("engrav"),
        "`me sysw -h` must connect to the operator's job: {sysw}"
    );

    let pack = short(&["sysw", "pack"]);
    assert!(
        pack.contains("tx:"),
        "`me sysw pack -h` must name the record: {pack}"
    );
    assert!(
        pack.contains("qr"),
        "and the QR path -- this is the sentence that was only in --help: {pack}"
    );
    assert!(
        pack.contains("mt encode --qr"),
        "and the command that produces it: {pack}"
    );
}

/// `me`'s one-liner said it converts `(md1/mk1)`. It accepts `mt1` too — the
/// walk fed one through the bare converter and got NDEF bytes back — so the
/// description was stale with respect to a capability this cycle added.
#[test]
fn the_one_liner_admits_the_string_kinds_me_actually_accepts() {
    let o = me().arg("-h").output().unwrap();
    let top = String::from_utf8_lossy(&o.stdout).to_lowercase();
    assert!(
        top.contains("mt1"),
        "mt1 is accepted and must be named: {top}"
    );
}

/// **A refusal about the INPUT outranks one about the destination.**
///
/// F-246 hoisted the write gate so nothing describes a container before it
/// runs. The first attempt hoisted it above `read_records`, which pre-empted
/// R2 — the refusal for a `tx:` record passed on ARGV, where the transaction is
/// already in the shell's history and in `ps`. That is both more urgent and
/// more specific than "your stdout is 0644", and it exits 3 rather than 2.
///
/// **The regenerated journey caught that swap; no test did.** This is the test.
#[test]
fn a_bearer_record_on_argv_outranks_the_write_gate() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let sink = dir.path().join("out.bin");
    let handle = std::fs::File::create(&sink).unwrap();
    std::fs::set_permissions(&sink, std::fs::Permissions::from_mode(0o644)).unwrap();

    // BOTH faults at once: a tx: record on argv AND a world-readable stdout.
    let o = std::process::Command::new(assert_cmd::cargo::cargo_bin("me"))
        .args(["sysw", "pack", "--no-passphrase", "tx:0100"])
        .stdout(std::process::Stdio::from(handle))
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&o.stderr).to_string();

    assert_eq!(
        o.status.code(),
        Some(3),
        "R2's code, not the write gate's: {err}"
    );
    assert!(err.contains("ARGV"), "R2 must be the refusal shown: {err}");
    assert!(
        !err.contains("its permissions grant read"),
        "the destination complaint must not pre-empt the bearer one: {err}"
    );
}

// ── P5 I-1: argv refuses every SECRET and BEARER class, not just `tx:` ───────

/// **The argv gate covered ONE of the classes it should.** Measured 2026-08-26:
/// `me sysw pack` refused a `tx:` record on argv at exit 3 — and accepted, at
/// exit 0 with no complaint, the SAME transaction carried as `mt1` strings.
///
/// argv is public: `/proc/<pid>/cmdline` without hidepid, `ps` for every user on
/// the box, and a shell history file that outlives the machine.
#[test]
fn argv_refuses_every_bearer_class() {
    let dir = tempfile::tempdir().unwrap();
    let cases: Vec<(&str, String)> = vec![
        ("a tx: record", "tx:0100".to_string()),
        ("an mt1 string", MT_EVEN[0].to_string()),
    ];
    for (what, rec) in cases {
        let out = dir.path().join("nope.bin");
        let a = me()
            .args(["sysw", "pack", "--no-passphrase", "--out"])
            .arg(&out)
            .arg(&rec)
            .write_stdin("")
            .assert()
            .failure();
        let err = String::from_utf8_lossy(&a.get_output().stderr).to_string();
        assert!(
            err.contains("ARGV"),
            "{what} must be refused for the argv reason: {err}"
        );
        assert!(!out.exists(), "{what}: nothing may be written");
        // NEVER echoed: printing it back puts the material in a SECOND public
        // place, which is the defect the refusal exists to name.
        assert!(
            !err.contains(&rec[..rec.len().min(24)]),
            "{what}: the refusal must not echo the body: {err}"
        );
    }
}

/// **RULED 2026-08-26 — secret and bearer classes behave THE SAME on argv.**
///
/// Operator: *"we want uniform behavior with secret bearing between ms1 and
/// passwords and mt1 to the extent we can."* Before this, `me sysw pack`
/// refused a `tx:` record and an `mt1` string on argv while accepting, at exit
/// 0 in silence, a BIP-39 mnemonic, an `ms1` string and a `pass:` record — so
/// it refused a TRANSACTION and accepted a SEED PHRASE.
///
/// `--allow-argv-secret` is the escape hatch, and it is what makes the refusal
/// fair on a single-user air-gapped box or an amnesic Tails session where the
/// argv threat model does not bite. It is greppable in a script, so a reviewer
/// can find it.
#[test]
fn argv_refuses_every_secret_class_too() {
    let dir = tempfile::tempdir().unwrap();
    let passhex: String =
        "correct horse battery staple"
            .bytes()
            .fold(String::new(), |mut acc, b| {
                let _ = write!(acc, "{b:02x}");
                acc
            });
    for (what, rec) in [
        ("a BIP-39 mnemonic", SEED.to_string()),
        ("a pass: record", format!("pass:{passhex}")),
    ] {
        let out = dir.path().join("nope.bin");
        let a = me()
            .args(["sysw", "pack", "--no-passphrase", "--out"])
            .arg(&out)
            .arg(&rec)
            .write_stdin("")
            .assert()
            .failure();
        let err = String::from_utf8_lossy(&a.get_output().stderr).to_string();
        assert!(err.contains("ARGV"), "{what} must be refused: {err}");
        assert!(!out.exists(), "{what}: nothing may be written");
        assert!(
            !err.contains(&rec[..rec.len().min(24)]),
            "{what}: never echo the body: {err}"
        );
    }
}

/// **The override makes it a speed bump, not a wall** — the operator's balance
/// between helpful and annoying. Air-gapped and amnesic machines are real.
#[test]
fn allow_argv_secret_proceeds() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("ok.bin");
    me().args([
        "sysw",
        "pack",
        "--no-passphrase",
        "--allow-argv-secret",
        "--out",
    ])
    .arg(&out)
    .arg(SEED)
    .write_stdin("")
    .assert()
    .success();
    assert!(out.exists(), "the override must actually proceed");
}

/// **The refusal TEACHES the cleanup command, not just the need for one.**
///
/// Operator: telling someone to "remove the line from your shell history" says
/// WHAT and not HOW. Two measured facts shape this:
/// - zsh's `history -d` does NOT delete (on 5.9.2 `-d` is a timestamp display
///   flag), so it must never be suggested;
/// - the pattern must anchor on the COMMAND NAME, because grepping for the
///   secret would type the secret into history a second time.
#[test]
fn the_argv_refusal_names_the_command_that_purges_history() {
    let dir = tempfile::tempdir().unwrap();
    let a = me()
        .args(["sysw", "pack", "--no-passphrase", "--out"])
        .arg(dir.path().join("n.bin"))
        .arg(SEED)
        .write_stdin("")
        .assert()
        .failure();
    let err = String::from_utf8_lossy(&a.get_output().stderr).to_string();
    assert!(err.contains("HISTFILE"), "name the history file: {err}");
    assert!(err.contains("sed -i"), "give the actual command: {err}");
    // NOT `!err.contains("history -d")` -- the message deliberately NAMES that
    // command in order to warn against it, so the naive negative fails on the
    // warning itself. The requirement is that it is never OFFERED, which is
    // what the explicit disclaimer proves.
    assert!(
        err.contains("does NOT delete"),
        "must actively warn that zsh's history -d does not delete: {err}"
    );
}

/// THE CONTROL: watch-only public material stays usable on argv. `md verify
/// <STRINGS>` and `mk verify [MK1]…` take theirs positionally by design — a
/// leak there costs privacy, not funds — so this gate must not swallow them.
#[test]
fn argv_still_accepts_watch_only_and_free_text() {
    let dir = tempfile::tempdir().unwrap();
    for (what, rec) in [("an md1 string", MD1), ("a text: record", TEXT)] {
        let out = dir.path().join(format!("ok{what}.bin").replace(' ', "_"));
        me().args(["sysw", "pack", "--no-passphrase", "--out"])
            .arg(&out)
            .arg(rec)
            .write_stdin("")
            .assert()
            .success();
        assert!(out.exists(), "{what} must still pack from argv");
    }
}

/// **P5 N-1 — the `sealing:` line printed before the `--iterations` gate could
/// abort the run.** F-246's rule is that no line describing a container may
/// print until every gate that can abort the write has run; the iterations
/// range check was the one gate that had not been moved behind it.
#[test]
fn an_out_of_range_iterations_count_aborts_before_sealing_is_described() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("n.bin");
    let a = me()
        .args([
            "sysw",
            "pack",
            "--iterations",
            "5",
            "--passphrase-words",
            "12",
            "--out",
        ])
        .arg(&out)
        .write_stdin(SEED)
        .assert()
        .failure();
    let err = String::from_utf8_lossy(&a.get_output().stderr).to_string();
    assert!(
        !err.contains("sealing:"),
        "nothing may describe the container before a gate that aborts: {err}"
    );
    assert!(
        err.contains("--iterations"),
        "the real refusal survives: {err}"
    );
    assert!(!out.exists(), "and no artifact");
}

// ── The refusal names the PROFILE, not the classifier ────────────────────────

/// BIP-93 test vector 1 — a perfectly good 128-bit codex32 secret, and not a
/// constellation `ms1`. Verbatim from the fork's `codex32/codex32_test.go`.
const BIP93_SECRET: &str = "ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw";

fn refuse_one_record(record: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    let recs = dir.path().join("r.txt");
    std::fs::write(&recs, format!("{record}\n")).unwrap();
    let out = dir.path().join("p.bin");
    let a = me()
        .args(["sysw", "pack", "--no-passphrase", "--in"])
        .arg(&recs)
        .arg("--out")
        .arg(&out)
        .assert()
        .failure();
    assert!(!out.exists(), "a refused run must write nothing");
    String::from_utf8_lossy(&a.get_output().stderr).to_string()
}

/// **The refusal named the wrong cause.**
///
/// A valid BIP-93 codex32 that is not a constellation `ms1` used to be refused
/// with *"not a BIP-39 mnemonic, not an md1/mk1/ms1/mt1 string"* — false on a
/// plain reading, because the string IS an `ms1` string. It is not a
/// *constellation* one, and the cause is the two-gate PROFILE, not the
/// classifier the old sentence pointed at. Measured cost of that misdirection,
/// once: an hour.
///
/// **The record is a SEED, so nothing of it may appear in the message** — the
/// check is the whole string and every 12-character window of it, not a glance.
#[test]
fn a_valid_bip93_string_is_told_it_is_bip93_and_not_a_constellation_ms1() {
    let err = refuse_one_record(BIP93_SECRET);
    for want in [
        "BIP-93",
        "not a constellation `ms1` record",
        "`entr`",
        "This one is 48 characters",
    ] {
        assert!(err.contains(want), "{want:?} missing from: {err}");
    }
    // The sentence that was false about this input, gone.
    assert!(
        !err.contains("not an md1/mk1/ms1/mt1 string"),
        "the classifier sentence is false for a BIP-93 string: {err}"
    );
    assert!(!err.contains(BIP93_SECRET), "the record is echoed: {err}");
    for w in BIP93_SECRET.as_bytes().windows(12) {
        let w = std::str::from_utf8(w).unwrap();
        assert!(
            !err.contains(w),
            "a 12-character window {w:?} is echoed: {err}"
        );
    }
}

/// **THE CONTROL.** Without it, replacing the general refusal with the BIP-93
/// one everywhere would pass the test above — and the descriptor/address gap,
/// which is what that sentence is genuinely about, would stop being named.
#[test]
fn a_record_of_no_class_at_all_still_names_the_classifier() {
    let err = refuse_one_record("this is not a record of any class");
    assert!(
        err.contains("not an md1/mk1/ms1/mt1 string"),
        "the general refusal must survive: {err}"
    );
    assert!(
        !err.contains("BIP-93"),
        "and must not claim this is codex32: {err}"
    );
}
