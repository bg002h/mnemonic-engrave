//! Bug-hunt rows (2026-11-25 hunt at 80dfaeb). Each asserts the CORRECT
//! behaviour; a red row is a confirmed bug.
#![cfg(unix)]

use assert_cmd::Command;

fn me() -> Command {
    Command::cargo_bin("me").unwrap()
}

/// The public test-vector ms1 body from `seal/record.rs`, with the separator
/// `1` mistyped as `l`, so `classify` sees no `ms` HRP.
const MANGLED_MS1: &str =
    "msl0entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw";
const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon \
                        abandon abandon abandon about";
const MD1: &str = "md1yqpqqxqq8xtwhw4xwn4qh";

/// Distinctive pieces of the secrets above; none may reach stderr or stdout.
const SECRET_PIECES: &[&str] = &["entrsqqg5y2z9", "q5f042qmrw90mw", "abandon", "msl0"];

fn assert_no_secret(label: &str, out: &std::process::Output) {
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    for p in SECRET_PIECES {
        assert!(
            !all.contains(p),
            "{label}: output echoes secret text {p:?}:\n{all}"
        );
    }
}

/// E1: the "unrecognized HRP" message interpolated everything before the
/// first `1` of the input, so a mistyped ms1 or a BIP-39 mnemonic followed by
/// any `1` was printed to stderr in full. The converter, `me bundle` and
/// `me seal` must refuse without echoing any of it.
#[test]
fn bh1125_e1_unknown_hrp_never_echoes_the_input() {
    let dir = tempfile::tempdir().unwrap();
    let cases: Vec<(&str, String)> = vec![
        ("mangled-ms1", format!("{MANGLED_MS1}\n{MD1}\n")),
        (
            "mangled-ms1-plate-1",
            format!("{MANGLED_MS1} plate 1\n{MD1}\n"),
        ),
        ("mnemonic", format!("{MNEMONIC}\n{MD1}\n")),
        ("seed-note", format!("seed: {MNEMONIC} (12 words)\n{MD1}\n")),
    ];
    for (label, text) in &cases {
        let path = dir.path().join(format!("{label}.txt"));
        std::fs::write(&path, text).unwrap();

        let out = me().arg("--in").arg(&path).arg("--hex").output().unwrap();
        assert!(!out.status.success(), "{label}: converter accepted it");
        assert_no_secret(&format!("me --in ({label})"), &out);

        let out = me().args(["bundle", "--in"]).arg(&path).output().unwrap();
        assert!(!out.status.success(), "{label}: bundle accepted it");
        assert_no_secret(&format!("me bundle ({label})"), &out);
    }

    // `me seal`: a record whose HRP is unknown (a hex private key here, whose
    // first `1` sits after 12 hex digits) is refused without its prefix.
    let out_path = dir.path().join("x.uf2");
    let out = me()
        .args([
            "seal",
            "--plaintext",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "--out",
        ])
        .arg(&out_path)
        .output()
        .unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("unrecognized HRP"), "{err}");
    assert!(!err.contains("e3b0c442"), "me seal echoed the key: {err}");
}

const K1: &str = "[dc567276/48h/0h/0h/2h]xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan/<0;1>/*";
const K2: &str = "[f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge/<0;1>/*";

fn built(desc: &str) -> mnemonic_engrave::descriptor::md1::Built {
    use mnemonic_engrave::descriptor::{cascade, md1};
    md1::build(&cascade::cascade(&cascade::normalise(desc)).unwrap()).unwrap()
}

/// X7: the md-codec pin was 0.48.5, whose `split` minted a card the decoder
/// refuses (0.48.6/0.48.7 CHANGELOG: an `@0` or redundant use-site override, a
/// template numbered out of first-occurrence order). 0.48.7's `encode_payload`
/// decodes its own output, so `--as md1`'s `md1::strings` can no longer return
/// strings no reader decodes. This pins that `me` links a codec that refuses.
#[test]
fn bh1125_x7_md1_strings_never_returns_a_card_the_decoder_refuses() {
    use mnemonic_engrave::descriptor::md1;
    let desc = format!("wsh(sortedmulti(2,{K1},{K2}))");

    // Control: the admitted wallet encodes, and every string decodes back.
    let b = built(&desc);
    let strings = md1::strings(&b).expect("an admitted wallet encodes");
    let refs: Vec<&str> = strings.iter().map(String::as_str).collect();
    md_codec::reassemble(&refs).expect("me's own md1 cards decode");

    // A use-site override keyed on the baseline `@0`: 0.48.5 minted it, and
    // the decoder refuses the card ("use-site override keyed on baseline @0").
    let mut bad = built(&desc);
    bad.descriptor.tlv.use_site_path_overrides = Some(vec![(
        0,
        md_codec::use_site_path::UseSitePath::standard_multipath(),
    )]);
    assert!(
        md1::strings(&bad).is_err(),
        "md-codec minted a card its own decoder refuses"
    );
}

/// The `even` mt1 corpus vector (mnemonic-transaction
/// design/vectors/mt1_v1_vectors.md): a 222-byte signed transaction in 6
/// chunks of 37 bytes.
const EVEN_MT1: [&str; 6] = [
    "mt1p9h8jqq9qqqqgqqqqqqqyqherdfykhhpey6z2cvafak8804qd7g0dl6v8ex9wr2cvky023skwkeud2229sax",
    "mt1p9h8jqq9qqphgdqqqqqqqq0mllllupyqj6vqqqqqqqqzcqpfsw7ph2rt5w54kt768636cls8zxg0najlzunp",
    "mt1p9h8jqq9qqzj8yqpnzw4vl2rwffqyqqqqqkqq282yyhc2vavd20hvk94pz39hts3u5s9a0qd8pwskxfl7ju5",
    "mt1p9h8jqq9qqrqfrnq3qzyp77h37cnxzvwutegzmzy5zrrrfvrpykdfsckvk03dcq6rcjtvlsfcglv7zx43yaz",
    "mt1p9h8jqq9qqylgpzqmhcwhuupdvnrc82rncvzzdahpgjsdwgu52jd7vmxsve9x3w5ujeqyssuvddxvwqze4ve",
    "mt1p9h8jqq9qq9qdcc7h75twfxyf340c4sgqzhfdq6xtgt7zhxngpwa049l0z59l6jqcqqqqqq5k5y2ye5nv8yf",
];

/// E5: `me`'s mt-codec pin predated mnemonic-transaction 95c39a3 (bh-mt-I18),
/// whose decode refuses a set that is not §3b's balanced split. The `even`
/// vector re-cut 38/36/37/37/37/37 under its own set id decodes, in index
/// order, to the true transaction on the old codec -- so `me` confirmed a set
/// no conforming encoder writes. It must not confirm it.
#[test]
fn bh1125_e5_a_set_that_is_not_the_balanced_split_is_not_confirmed() {
    use mnemonic_engrave::sysw::mt::decode_confirmed;
    use mt_codec::string_layer::bch::{bytes_to_5bit, encode_5bit_to_string};

    let even: Vec<String> = EVEN_MT1.iter().map(|s| s.to_string()).collect();
    let (bytes, _) = decode_confirmed(&even).expect("the corpus set confirms");
    let set = mt_codec::decode(&even).unwrap().chunks[0]
        .header
        .chunk_set_id;

    let sizes = [38usize, 36, 37, 37, 37, 37];
    let mut at = 0;
    let recut: Vec<String> = sizes
        .iter()
        .enumerate()
        .map(|(i, &n)| {
            let mut syms = mt_codec::ChunkHeader::new(set, sizes.len(), i)
                .unwrap()
                .to_symbols()
                .to_vec();
            syms.extend(bytes_to_5bit(&bytes[at..at + n]));
            at += n;
            encode_5bit_to_string(&syms).unwrap()
        })
        .collect();
    assert!(
        decode_confirmed(&recut).is_none(),
        "me confirmed an mt1 set that is not §3b's balanced split"
    );
}

/// Two chunks of a three-chunk md1 set: each is a well-formed md1 string, but
/// the set cannot be reassembled, so `me` cannot confirm it and the device
/// treats both records as SECRET (SPEC_systemwide_payloads §5.3.2, D6).
const MD1_PARTIAL_A: &str = "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3";
const MD1_PARTIAL_B: &str = "md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374";

/// E4: `me sysw pack` of an unconfirmed md1/mk1 printed "NOT SEALED -- no
/// record in this payload is secret material" while the next lines said the
/// device will treat those very records as SECRET. The sealing line must name
/// the unconfirmed records and say they are NOT sealed, so the two lines
/// agree. Ruling D6 stands: the pack still succeeds, unsealed, with the same
/// container bytes.
#[test]
fn bh1125_e4_sealing_line_names_unconfirmed_records_as_not_sealed() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    let res = me()
        .args(["sysw", "pack", "--out", out.to_str().unwrap()])
        .args([MD1_PARTIAL_A, MD1_PARTIAL_B])
        .output()
        .unwrap();
    assert!(
        res.status.success(),
        "D6: an unconfirmed md1 warns, never refuses"
    );
    let err = String::from_utf8_lossy(&res.stderr);
    let flat = err.split_whitespace().collect::<Vec<_>>().join(" ");
    let sealing = flat
        .split("sealing:")
        .nth(1)
        .expect("no sealing line")
        .split("me:")
        .next()
        .unwrap();
    assert!(sealing.contains("NOT SEALED"), "{flat}");
    assert!(
        !sealing.contains("no record in this payload is secret material"),
        "the sealing line still says nothing is secret: {flat}"
    );
    assert!(
        sealing.contains("records 0 and 1"),
        "the sealing line does not name the unconfirmed records: {flat}"
    );
    assert!(
        sealing.contains("will treat as SECRET") && sealing.contains("not encrypted"),
        "the sealing line does not say the unconfirmed records stay unsealed: {flat}"
    );
}

/// E4 control: a complete md1 is confirmed, so the plain no-secret sentence
/// still applies and names no record.
#[test]
fn bh1125_e4_a_confirmed_md1_keeps_the_no_secret_line() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    let res = me()
        .args(["sysw", "pack", "--out", out.to_str().unwrap(), MD1])
        .output()
        .unwrap();
    assert!(res.status.success());
    let flat = String::from_utf8_lossy(&res.stderr)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        flat.contains("NOT SEALED — no record in this payload is secret material"),
        "{flat}"
    );
    assert!(!flat.contains("will treat as SECRET"), "{flat}");
}

/// E4, with secret material alongside: the sealed and the `--no-passphrase`
/// lines also say that the unconfirmed md1/mk1 records are not encrypted,
/// because `pack` encrypts only secret-class records.
#[test]
fn bh1125_e4_sealed_and_no_passphrase_lines_name_unconfirmed_records() {
    for (extra, verdict) in [
        (
            vec!["--passphrase-words", "4"],
            "Not sealed: records 1 and 2",
        ),
        (vec!["--no-passphrase"], "Also, records 1 and 2"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("p.bin");
        let res = me()
            .args(["sysw", "pack", "--out", out.to_str().unwrap()])
            .args(&extra)
            .args([
                "--allow-argv-secret",
                MNEMONIC,
                MD1_PARTIAL_A,
                MD1_PARTIAL_B,
            ])
            .output()
            .unwrap();
        assert!(res.status.success(), "{extra:?}");
        let flat = String::from_utf8_lossy(&res.stderr)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(flat.contains(verdict), "{extra:?}: {flat}");
        assert!(
            flat.contains("will treat as SECRET; they are not encrypted"),
            "{flat}"
        );
    }
}
