//! **§6g — `me sysw pack --expect <kinds>`: state what the container must hold,
//! and be refused if it does not.**
//!
//! The failure this exists for: `mk encode` refuses, the operator's pipeline
//! carries on, `me sysw pack` builds a container from the `md1` records alone
//! at **exit 0**, and a plate is cut from a wallet nobody can restore. The gap
//! shows up years later, at the only moment it matters.
//!
//! **Every refusal below pins its exit DIGIT.** `--expect` is P0's newest
//! funds-path refusal and its exit code was pinned by nothing; a refusal that
//! can silently change what it means to a script is the F-265 shape in new
//! code.
#![cfg(unix)]

use std::process::Command;

fn me() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("me")
}

const MD1_A: &str = "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3";
const MD1_B: &str = "md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374";
const MD1_C: &str = "md1fv9wjpqsp2026hh65xpvugtfhd9792zxgunymm0a82pdju6442q0jskj9gzfaqmz";
const MK1_A: &str = "mk1qpz63tpqqsq3dg4m5wdx5fvqqvzg3vs7mpf0rz2j43zpzpxk0rtjkqkhwreqp6hm7qnp3a8wdvtz6t2k4uxu6ykwxcp9vqugfjyx733cf59g";
const MK1_B: &str =
    "mk1qpz63tppkeg9pdvqz5744004gvzecsknw6tu25yv3exfhkl6w5zm9e4t24aqdah5585wn3e4xdut8";
const MT_EVEN: [&str; 6] = [
    "mt1p9h8jqq9qqqqgqqqqqqqyqherdfykhhpey6z2cvafak8804qd7g0dl6v8ex9wr2cvky023skwkeud2229sax",
    "mt1p9h8jqq9qqphgdqqqqqqqq0mllllupyqj6vqqqqqqqqzcqpfsw7ph2rt5w54kt768636cls8zxg0najlzunp",
    "mt1p9h8jqq9qqzj8yqpnzw4vl2rwffqyqqqqqkqq282yyhc2vavd20hvk94pz39hts3u5s9a0qd8pwskxfl7ju5",
    "mt1p9h8jqq9qqrqfrnq3qzyp77h37cnxzvwutegzmzy5zrrrfvrpykdfsckvk03dcq6rcjtvlsfcglv7zx43yaz",
    "mt1p9h8jqq9qqylgpzqmhcwhuupdvnrc82rncvzzdahpgjsdwgu52jd7vmxsve9x3w5ujeqyssuvddxvwqze4ve",
    "mt1p9h8jqq9qq9qdcc7h75twfxyf340c4sgqzhfdq6xtgt7zhxngpwa049l0z59l6jqcqqqqqq5k5y2ye5nv8yf",
];
/// A transaction with one input carrying neither a scriptSig nor a witness —
/// what `--allow-unsigned-inputs` exists to admit.
const TX_MIXED_STRIPPED: &str = "020000000211111111111111111111111111111111111111111111111111111111111111110000000048473030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030ffffffff22222222222222222222222222222222222222222222222222222222222222220100000000ffffffff0150c3000000000000160014333333333333333333333333333333333333333300000000";

struct Run {
    code: i32,
    err: String,
    out_exists: bool,
}

/// `me sysw pack` over `records`, with `extra` flags, writing to a fresh path.
fn pack(dir: &std::path::Path, records: &[&str], extra: &[&str]) -> Run {
    let inp = dir.join(format!("in-{}.txt", records.len()));
    std::fs::write(&inp, format!("{}\n", records.join("\n"))).unwrap();
    let out = dir.join("p.bin");
    let _ = std::fs::remove_file(&out);

    let mut args: Vec<String> = vec!["sysw".into(), "pack".into(), "--no-passphrase".into()];
    args.extend(extra.iter().map(|s| s.to_string()));
    args.push("--in".into());
    args.push(inp.display().to_string());
    args.push("--out".into());
    args.push(out.display().to_string());

    let o = Command::new(me()).args(&args).output().unwrap();
    Run {
        code: o.status.code().unwrap(),
        err: String::from_utf8_lossy(&o.stderr).into_owned(),
        out_exists: out.exists(),
    }
}

/// **THE FUNDS CASE.** `--expect descriptor,cosigner` on a payload holding only
/// the descriptor cards.
///
/// **This is why `descriptor` and `cosigner` do NOT resolve through `Class`.**
/// `me`'s `Class` has a single `MdMk` variant covering both, so a `Class`-keyed
/// test is satisfied by the `md1` records alone — a refusing `mk encode` still
/// yields exit 0 with the cosigner card missing, and the operator believes a
/// backup is complete when it is not. The HRP discriminant is what separates
/// them: `'d'` reassembles through `md_codec`, `'k'` through `mk_codec`.
#[test]
fn expect_descriptor_cosigner_refuses_a_payload_with_no_cosigner_card() {
    let dir = tempfile::tempdir().unwrap();

    let r = pack(
        dir.path(),
        &[MD1_A, MD1_B, MD1_C],
        &["--expect", "descriptor,cosigner"],
    );
    assert_eq!(r.code, 4, "the input is not what it must be: {}", r.err);
    assert!(
        r.err.contains("--expect cosigner") && r.err.contains("NO record of that kind"),
        "the refusal must name the kind that is MISSING, not the one present: {}",
        r.err
    );
    assert!(!r.out_exists, "nothing may be written: {}", r.err);

    // THE CONTROL, and it is what proves the test is keyed on the HRP: add the
    // cosigner cards and the same expectation is met.
    let r = pack(
        dir.path(),
        &[MD1_A, MD1_B, MD1_C, MK1_A, MK1_B],
        &["--expect", "descriptor,cosigner"],
    );
    assert_eq!(r.code, 0, "a complete backup must pack: {}", r.err);
}

/// A stream with no transaction in it, asked for one.
#[test]
fn expect_transaction_refuses_a_stream_that_holds_none() {
    let dir = tempfile::tempdir().unwrap();
    let r = pack(
        dir.path(),
        &[MD1_A, MD1_B, MD1_C],
        &["--expect", "descriptor,transaction"],
    );
    assert_eq!(r.code, 4, "{}", r.err);
    assert!(
        r.err.contains("--expect transaction") && r.err.contains("NO record of that kind"),
        "{}",
        r.err
    );
    assert!(!r.out_exists);
}

/// **Presence is not completeness — walk 2.** One chunk of a three-chunk `md1`
/// card set is present and useless: it passes every checksum it carries and
/// still cannot be restored from.
#[test]
fn expect_descriptor_refuses_an_incomplete_md1_set() {
    let dir = tempfile::tempdir().unwrap();
    let r = pack(dir.path(), &[MD1_A], &["--expect", "descriptor"]);
    assert_eq!(r.code, 4, "{}", r.err);
    assert!(
        r.err.contains("does not reassemble"),
        "the refusal must distinguish INCOMPLETE from ABSENT -- they need \
         different fixes: {}",
        r.err
    );
    assert!(!r.out_exists);

    // Control: the whole set is accepted.
    let r = pack(
        dir.path(),
        &[MD1_A, MD1_B, MD1_C],
        &["--expect", "descriptor"],
    );
    assert_eq!(r.code, 0, "{}", r.err);
}

/// **Presence is not completeness — walk 3, the one `mdmk_unconfirmed` is blind
/// to.**
///
/// That walk filters on `Class::MdMk`, and an `mt1` chunk is `Class::Mt`, so it
/// returns `[]` for a half-transmitted transaction — *"nothing wrong here"* —
/// while `mt_unconfirmed` returns `[0, 1, 2]`. An implementation with only two
/// walks ships an `--expect transaction` that passes a half-transmitted
/// transaction as complete: §6g's own failure mode surviving inside §6g's own
/// remedy.
#[test]
fn expect_transaction_refuses_an_incomplete_mt1_set() {
    let dir = tempfile::tempdir().unwrap();
    let half: Vec<&str> = MT_EVEN[..3].to_vec();

    let r = pack(dir.path(), &half, &["--expect", "transaction"]);
    assert_eq!(r.code, 4, "{}", r.err);
    assert!(
        r.err.contains("does not reassemble"),
        "half an mt1 set is present and unrestorable: {}",
        r.err
    );
    assert!(
        r.err.contains("record 0, 1, 2"),
        "and it must name which -- these are the indices mt_unconfirmed returns \
         and mdmk_unconfirmed cannot see: {}",
        r.err
    );
    assert!(!r.out_exists);

    // Control: the complete six-chunk set satisfies it.
    let r = pack(dir.path(), &MT_EVEN, &["--expect", "transaction"]);
    assert_eq!(r.code, 0, "a complete mt1 set must pack: {}", r.err);
}

/// **`--expect` MUST CONSULT `Admission`, or it invents a false refusal on the
/// funds path.**
///
/// Built without it, `--allow-unsigned-inputs --expect transaction` refuses at
/// exit 4 saying *NO record of that kind is in the stream* — for a record the
/// **same invocation packs at exit 0 without `--expect`**. A false refusal
/// carrying a false message, inside the feature added to prevent exactly that.
#[test]
fn allow_unsigned_inputs_and_expect_transaction_do_not_falsely_refuse() {
    let dir = tempfile::tempdir().unwrap();
    let rec = format!("tx:{TX_MIXED_STRIPPED}");

    // THE CONTROL FIRST: the same invocation WITHOUT --expect. If this is not
    // 0, the test below proves nothing about --expect.
    let r = pack(dir.path(), &[&rec], &["--allow-unsigned-inputs"]);
    assert_eq!(
        r.code, 0,
        "control: this record packs without --expect, so --expect must not \
         change that: {}",
        r.err
    );

    let r = pack(
        dir.path(),
        &[&rec],
        &["--allow-unsigned-inputs", "--expect", "transaction"],
    );
    assert_eq!(
        r.code, 0,
        "adding --expect transaction must not refuse a transaction the same \
         command just packed: {}",
        r.err
    );

    // And the negative half: WITHOUT the admission flag the record is not a
    // transaction, so --expect transaction genuinely is unmet. Same message,
    // but now it is TRUE.
    let r = pack(dir.path(), &[&rec], &["--expect", "transaction"]);
    assert_ne!(
        r.code, 0,
        "without --allow-unsigned-inputs this record is not admitted at all: {}",
        r.err
    );
}

/// The vocabulary refuses what it cannot satisfy, **as a USAGE error**, and
/// says why.
///
/// A flag value out of range is USAGE (2), not invalid (4): no input has been
/// read at the point it is caught, so there is nothing yet for "invalid" to be
/// about. `address` and `passphrase` are the two likely wrong guesses and each
/// gets its own reason rather than a bare "unknown".
#[test]
fn the_vocabulary_excludes_what_can_never_be_satisfied() {
    let dir = tempfile::tempdir().unwrap();
    for (word, because) in [
        ("address", "cannot classify an address record"),
        ("passphrase", "cannot be satisfied on the flag path"),
    ] {
        let r = pack(dir.path(), &[MD1_A, MD1_B, MD1_C], &["--expect", word]);
        assert_eq!(
            r.code, 2,
            "an unknown --expect kind is a USAGE error, not an invalid input: {}",
            r.err
        );
        assert!(
            r.err.contains(because),
            "{word} must say why it is absent, not just that it is: {}",
            r.err
        );
        assert!(
            r.err
                .contains("descriptor, cosigner, transaction, mnemonic, secret"),
            "and it must list what IS available: {}",
            r.err
        );
    }
}

/// **THE POSITIVE CONTROL for the whole feature.** Without `--expect`, nothing
/// changes; with a satisfied `--expect`, nothing changes either. A `--expect`
/// that refused everything would satisfy every assertion above.
#[test]
fn expect_changes_nothing_when_it_is_met_or_absent() {
    let dir = tempfile::tempdir().unwrap();
    let all = [MD1_A, MD1_B, MD1_C, MK1_A, MK1_B];

    let without = pack(dir.path(), &all, &[]);
    assert_eq!(without.code, 0, "{}", without.err);

    let with = pack(dir.path(), &all, &["--expect", "descriptor,cosigner"]);
    assert_eq!(with.code, 0, "{}", with.err);
    assert!(
        !with.err.contains("--expect"),
        "a met expectation says nothing at all: {}",
        with.err
    );
}

// ── S2: `descriptor` names TWO carriers ─────────────────────────────────────

const DESCRIPTOR_VECTORS: &str = "testdata/descriptor_seam_vectors.json";
/// An md1-representable happy row, so BOTH carriers are reachable from it.
const HAPPY_ROW: &str = "formats-happy/bip380-sortedmulti-multipath";
const MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn vector_field(name: &str, field: &str) -> String {
    let raw = std::fs::read(DESCRIPTOR_VECTORS).unwrap();
    let doc: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    doc["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"].as_str().unwrap() == name)
        .and_then(|r| r[field].as_str().map(str::to_string))
        .unwrap_or_else(|| panic!("{DESCRIPTOR_VECTORS}: no {field} on row {name:?}"))
}

/// **The belt-and-braces invocation, which used to refuse the record it had
/// just built.** `Kind::Descriptor` resolved by card HRP alone, so once
/// `--as descriptor` packs §5.2's record, `--expect descriptor` alongside it
/// would report that record absent — 100% reproducibly, on the funds path,
/// inside the feature added to prevent exactly that class of false refusal.
///
/// The exit code here is 2, not 0, and that is §5.1 rather than `--expect`:
/// `--expect` resolves FIRST and is MET, and the run then reaches the choice
/// block because `--as` was omitted. What this pins is that it is not exit 4
/// saying no record of that kind is in the stream.
#[test]
fn expect_descriptor_is_satisfied_by_a_descriptor_record() {
    let dir = tempfile::tempdir().unwrap();
    let canonical = vector_field(HAPPY_ROW, "canonical");
    let r = pack(dir.path(), &[&canonical], &["--expect", "descriptor"]);
    assert!(
        !r.err.contains("--expect descriptor was not met"),
        "the record `--as descriptor` packs must satisfy `--expect descriptor`: {}",
        r.err
    );
    assert_eq!(r.code, 2, "{}", r.err);
    assert!(
        r.err.contains("`--as` decides how it is packed"),
        "{}",
        r.err
    );
}

/// The widening ADDS a carrier and removes none: an md1 descriptor card still
/// satisfies the kind, which is the reading every shipped container depends on.
#[test]
fn expect_descriptor_is_still_satisfied_by_an_md1_card() {
    let dir = tempfile::tempdir().unwrap();
    let input = vector_field(HAPPY_ROW, "input");
    let r = pack(
        dir.path(),
        &[&input],
        &["--as", "md1", "--expect", "descriptor"],
    );
    assert_eq!(r.code, 0, "{}", r.err);
    assert!(r.out_exists);
}

/// And it stays refusable. A kind that everything satisfies is not a gate.
#[test]
fn expect_descriptor_still_refuses_a_mnemonic_only_container() {
    let dir = tempfile::tempdir().unwrap();
    let r = pack(dir.path(), &[MNEMONIC], &["--expect", "descriptor"]);
    assert_eq!(r.code, 4, "{}", r.err);
    assert!(
        r.err
            .contains(mnemonic_engrave::sysw::expect::Kind::Descriptor.describes()),
        "the refusal must say what was looked for: {}",
        r.err
    );
    assert!(!r.out_exists, "nothing is written on an unmet expectation");
}
