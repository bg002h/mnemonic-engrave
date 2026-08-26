//! `me sysw` — the plan's stage-2 green criteria, as tests.

use assert_cmd::Command;
use predicates::prelude::*;

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
    me().args(["sysw", "pack", "--passphrase-words", "2", SEED])
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
    me().args(["sysw", "pack", MD1])
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
    me().args(["sysw", "pack", "--passphrase-words", "12", SEED, "--out"])
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
        .args(["sysw", "pack", "--region", "--passphrase-words", "12", SEED])
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
    assert!(52 + section_len < 65_536, "still a section refusal, not a region one");
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
        vec!["--passphrase-words", "12", SEED],
        vec!["--iterations", "100000", "--passphrase-words", "2", SEED],
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
    let out = me()
        .args(["sysw", "pack", "--no-passphrase", SEED, MD1])
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
    let mut args = vec!["sysw", "pack", "--no-passphrase", "--out"];
    args.push(out.to_str().unwrap());
    args.extend(MT_EVEN);
    let a = me().args(&args).assert().success();
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
    assert_eq!(stdout.matches("mt1 chunk — confirmed").count(), 6, "{stdout}");
    assert!(stdout.contains(MT_EVEN_TXID), "{stdout}");
    assert!(stdout.contains("222 bytes"), "{stdout}");
}

/// One chunk alone packs — nothing refuses (§13 D6's demotion applies to mt as
/// to mdmk) — but pack WARNS and show reports it unconfirmed.
#[test]
fn pack_warns_on_an_incomplete_mt_set() {
    let a = me()
        .args(["sysw", "pack", "--no-passphrase", MT_EVEN[0]])
        .assert()
        .success();
    let err = String::from_utf8_lossy(&a.get_output().stderr).to_string();
    assert!(
        err.contains("record 0") && err.contains("could not confirm"),
        "{err}"
    );
}

/// `me tx` builds the record, and the record round-trips through pack + show.
#[test]
fn me_tx_builds_a_record_that_packs() {
    let a = me()
        .arg("tx")
        .write_stdin(MT_EVEN_RAW_HEX)
        .assert()
        .success();
    let record = String::from_utf8_lossy(&a.get_output().stdout)
        .trim()
        .to_string();
    assert_eq!(record, format!("tx:{MT_EVEN_RAW_HEX}"));
    let err = String::from_utf8_lossy(&a.get_output().stderr).to_string();
    assert!(err.contains(MT_EVEN_TXID), "summary must name the txid: {err}");

    me()
        .args(["sysw", "pack", "--no-passphrase", &record])
        .assert()
        .success();
}

/// The tx: prefix is RESERVED: hex that is not a transaction is refused with
/// the structural reason, and non-hex with the hex reason.
#[test]
fn pack_refuses_a_tx_record_that_is_not_a_transaction() {
    me()
        .args(["sysw", "pack", "--no-passphrase", "tx:abab"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not one serialized Bitcoin transaction"));
    me()
        .args(["sysw", "pack", "--no-passphrase", "tx:zz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not lowercase hex"));
}

/// `me tx` refuses bytes that do not parse, at exit 4, naming the shape.
#[test]
fn me_tx_refuses_non_transactions() {
    me()
        .arg("tx")
        .write_stdin("abababab")
        .assert()
        .code(4)
        .stderr(predicate::str::contains("not one serialized Bitcoin transaction"));
    me().arg("tx").write_stdin("zz").assert().code(4);
    me().arg("tx").write_stdin("").assert().code(2);
}

/// A flipped character in an mt1 string is REFUSED at pack (exact validity,
/// never correction), with the record's index named.
#[test]
fn pack_refuses_a_damaged_mt1_string() {
    let mut bad = MT_EVEN[0].to_string();
    bad.pop();
    bad.push(if MT_EVEN[0].ends_with('x') { 'y' } else { 'x' });
    me()
        .args(["sysw", "pack", "--no-passphrase", &bad])
        .assert()
        .failure()
        .stderr(predicate::str::contains("record 0"));
}

/// `show` names a tx: record as what it is, with its txid — the value the
/// operator compares against `me tx`'s own report.
#[test]
fn show_names_a_tx_record() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    let rec = format!("tx:{MT_EVEN_RAW_HEX}");
    me().args(["sysw", "pack", "--no-passphrase", "--out", out.to_str().unwrap(), &rec])
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
    let body = std::iter::repeat_n(one.as_str(), n).collect::<Vec<_>>().join("\n");
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
