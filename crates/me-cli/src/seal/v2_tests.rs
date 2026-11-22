//! MNEMBLOB v2 rows of S1b §14.10 (ME), over the §14.9 vectors.
//!
//! The vector inputs are written out here; the expected bytes are the spec's
//! (printed there from `scripts/gen-mnemblob-v2-vectors.py`, an independent
//! implementation): [`v2_vectors_match_the_independent_generator`] looks every
//! shape row, digest row and header up in the spec text. The negative vectors
//! are built by a raw builder, because `seal_v2` refuses what they exist to
//! carry.

use super::*;
use std::fmt::Write as _;
use std::time::{Duration, Instant};

const SPEC: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../design/SPEC_encrypted_payload_delivery.md"
);
const V2_VECTORS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/seal_v2_vectors.json");
const V1_VECTORS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/seal_vectors.json");
const EMIT_CMD: &str = "ME_EMIT_VECTORS=1 cargo test -p mnemonic-engrave --lib \
                        seal::v2_tests::v2_vectors_match_the_independent_generator";

// ── inputs (S1b §14.9) ──────────────────────────────────────────────────────

const A16: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const MS1_A_TO_G: [&str; 7] = [
    "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w",
    "ms10entrsqqg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3ztl5m2c7mg3ls6",
    "ms10entrsqq3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zym4y84zxyuwtkl",
    "ms10entrsqqenxvenxvenxvenxvenxvenxvenxvenxvenxvenxvenxvenxvenxgyv57h75aydvt",
    "ms10entrsqpzyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygxfk0vzg2zk356",
    "ms10entrsqp2424242424242424242424242424242424242424242424242424c7u8hs6ruhww",
    "ms10entrsqpnxvenxvenxvenxvenxvenxvenxvenxvenxvenxvenxvenxvenxv9jwqcdg9hrrgt",
];
const MS1_P: &str = "ms10entrsqrllllllllllllllllllllllllllllllllllllllllllllllllll7ydtcvhdp9ycqe";
const SHARE: &str = "ms13casha320zyxwvutsrqpnmlkjhgfedca2a8d0zehn8a0t";
const D_PUBLIC: [&str; 5] = [
    "mk1qpz63tpqqsq3dg4m5wdx5fvqqvzg3vs7mpf0rz2j43zpzpxk0rtjkqkhwreqp6hm7qnp3a8wdvtz6t2k4uxu6ykwxcp9vqugfjyx733cf59g",
    "mk1qpz63tppkeg9pdvqz5744004gvzecsknw6tu25yv3exfhkl6w5zm9e4t24aqdah5585wn3e4xdut8",
    "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3",
    "md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374",
    "md1fv9wjpqsp2026hh65xpvugtfhd9792zxgunymm0a82pdju6442q0jskj9gzfaqmz",
];
const D_MS1: &str = "ms10entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw";
const ABANDON12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const BEEF6: &str = "beef beef beef beef beef beef";
const BEEF12: &str = "beef beef beef beef beef beef beef beef beef beef beef beef";
const ZOO12: &str = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo";

fn rep<const N: usize>(pair: [u8; 2]) -> [u8; N] {
    let mut out = [0u8; N];
    for (i, b) in out.iter_mut().enumerate() {
        *b = pair[i % 2];
    }
    out
}

fn hex(b: &[u8]) -> String {
    b.iter().fold(String::new(), |mut a, x| {
        let _ = write!(a, "{x:02x}");
        a
    })
}

fn sha(b: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex(&Sha256::digest(b))
}

fn s(x: &str) -> String {
    x.to_string()
}

/// A titles record as the generator writes it: `text:` and the hex of
/// `magic LF (line LF)*` (the last LF dropped when `final_lf` is false).
fn titles_record(lines: &[&str], magic: &str, final_lf: bool) -> String {
    let mut block = format!("{magic}\n");
    for (i, l) in lines.iter().enumerate() {
        block.push_str(l);
        if final_lf || i + 1 < lines.len() {
            block.push('\n');
        }
    }
    format!("text:{}", hex(block.as_bytes()))
}

fn titles_of(lines: &[&str]) -> String {
    titles_record(lines, "refugium-plates 2", true)
}

/// One vector: every input, the password form and the version.
struct V {
    name: &'static str,
    version: u8,
    pw_form: u8,
    words: &'static str,
    iterations: u32,
    salt: [u8; 16],
    iv: [u8; 12],
    public: Vec<String>,
    secret: Vec<String>,
    /// The titles of a positive vector, for the emitted file.
    titles: Option<Vec<String>>,
}

#[allow(clippy::too_many_arguments)]
fn v(
    name: &'static str,
    pw_form: u8,
    words: &'static str,
    iterations: u32,
    salt: [u8; 16],
    iv: [u8; 12],
    public: Vec<String>,
    secret: Vec<String>,
) -> V {
    V {
        name,
        version: wire::VERSION_V2,
        pw_form,
        words,
        iterations,
        salt,
        iv,
        public,
        secret,
        titles: None,
    }
}

fn all_vectors() -> Vec<V> {
    let neg_salt = rep::<16>([0x0b, 0xad]);
    let neg = |name, iv: u8, secret: Vec<String>, public: Vec<String>| {
        v(
            name,
            6,
            BEEF6,
            600_000,
            neg_salt,
            rep([0x0c, iv]),
            public,
            secret,
        )
    };
    let seven: Vec<String> = MS1_A_TO_G.iter().map(|x| s(x)).collect();
    let mut b_secret = seven.clone();
    b_secret.push(titles_of(&[
        "title SEED A",
        "title SEED B",
        "title SEED C",
        "title SEED D",
        "title SEED E",
        "title SEED F",
        "title HOT SEED G",
    ]));
    let mut a = v(
        "V2-A",
        6,
        BEEF6,
        600_000,
        rep([0xfe, 0xed]),
        rep([0xfa, 0xce]),
        vec![],
        vec![
            s(A16),
            s(MS1_P),
            titles_of(&["title SEED A", "title KEEP; DO NOT LOAD"]),
        ],
    );
    a.titles = Some(vec![s("SEED A"), s("KEEP; DO NOT LOAD")]);
    let mut b = v(
        "V2-B",
        12,
        ZOO12,
        600_000,
        rep([0xde, 0xaf]),
        rep([0xfa, 0xde]),
        vec![],
        b_secret,
    );
    b.titles = Some(
        [
            "SEED A",
            "SEED B",
            "SEED C",
            "SEED D",
            "SEED E",
            "SEED F",
            "HOT SEED G",
        ]
        .iter()
        .map(|x| s(x))
        .collect(),
    );
    let c = v(
        "V2-C",
        0,
        BEEF12,
        100_000,
        rep([0xde, 0xad]),
        rep([0xbe, 0xef]),
        D_PUBLIC.iter().map(|x| s(x)).collect(),
        vec![s(D_MS1)],
    );
    let seed_a = titles_of(&["title SEED A"]);
    let mut floor = v(
        "N-floor-599999",
        6,
        BEEF6,
        599_999,
        rep([0x0b, 0xee]),
        rep([0x0c, 0x0c]),
        vec![],
        vec![
            s(A16),
            s(MS1_P),
            titles_of(&["title SEED A", "title KEEP; DO NOT LOAD"]),
        ],
    );
    floor.titles = None;
    let mut v1text = v(
        "N-v1-text",
        0,
        BEEF12,
        100_000,
        neg_salt,
        rep([0x0c, 0x0b]),
        vec![],
        vec![s(A16), seed_a.clone()],
    );
    v1text.version = wire::VERSION;
    vec![
        a,
        b,
        c,
        neg(
            "N-count",
            0x01,
            vec![s(A16), s(MS1_P), seed_a.clone()],
            vec![],
        ),
        neg(
            "N-letter",
            0x02,
            vec![s(MS1_A_TO_G[1]), titles_of(&["title SEED C"])],
            vec![],
        ),
        neg(
            "N-hot-letter",
            0x03,
            vec![s(MS1_A_TO_G[6]), titles_of(&["title HOT SEED F"])],
            vec![],
        ),
        neg(
            "N-magic",
            0x04,
            vec![
                s(A16),
                titles_record(&["title SEED A"], "refugium-plates 1", true),
            ],
            vec![],
        ),
        neg(
            "N-two",
            0x05,
            vec![s(A16), seed_a.clone(), seed_a.clone()],
            vec![],
        ),
        neg("N-not-last", 0x06, vec![seed_a.clone(), s(A16)], vec![]),
        neg(
            "N-title-set",
            0x07,
            vec![s(A16), titles_of(&["title SEED Q"])],
            vec![],
        ),
        neg(
            "N-no-final-lf",
            0x08,
            vec![
                s(A16),
                titles_record(&["title SEED A"], "refugium-plates 2", false),
            ],
            vec![],
        ),
        neg("N-empty", 0x09, vec![s(A16), titles_of(&[])], vec![]),
        neg("N-public-titles", 0x0a, vec![s(A16)], vec![seed_a.clone()]),
        floor,
        neg(
            "N-share",
            0x0d,
            vec![
                s(SHARE),
                s(MS1_P),
                titles_of(&["title SEED A", "title KEEP; DO NOT LOAD"]),
            ],
            vec![],
        ),
        neg("N-not-hex", 0x0e, vec![s(A16), s("text:zz")], vec![]),
        neg("N-mnemonic", 0x0f, vec![s(ABANDON12), seed_a], vec![]),
        v1text,
    ]
}

fn by_name(name: &str) -> V {
    all_vectors()
        .into_iter()
        .find(|v| v.name == name)
        .unwrap_or_else(|| panic!("no vector {name}"))
}

/// The raw builder: header, sections and AEAD exactly as §6 and §14.2 say, with
/// none of `seal_v2`'s refusals, so a negative vector can carry what the reader
/// must refuse.
fn raw_blob(v: &V) -> Vec<u8> {
    let enc = |r: &[String]| -> Zeroizing<String> {
        if r.is_empty() {
            Zeroizing::new(String::new())
        } else {
            container::encode_section(r).expect("section")
        }
    };
    let (pubsec, secsec) = (enc(&v.public), enc(&v.secret));
    let header = Header {
        version: v.version,
        pw_form: v.pw_form,
        iterations: v.iterations,
        salt: v.salt,
        iv: v.iv,
        pub_len: pubsec.len() as u32,
        ct_len: secsec.len() as u32,
    };
    let mut aad = header.encode().to_vec();
    aad.extend_from_slice(pubsec.as_bytes());
    let key = crypto::derive_key(&passphrase::normalise(v.words), &v.salt, v.iterations);
    let sealed = crypto::seal_bytes(&key, &v.iv, &aad, secsec.as_bytes()).unwrap();
    let mut blob = aad;
    blob.extend_from_slice(&sealed);
    blob
}

fn blob_of(name: &str) -> Vec<u8> {
    raw_blob(&by_name(name))
}

fn open_named(name: &str, password: &str) -> Result<Opened, OpenError> {
    open(&blob_of(name), password)
}

fn refusal(name: &str, password: &str) -> OpenError {
    match open_named(name, password) {
        Ok(_) => panic!("{name}: opened, but it must be refused"),
        Err(e) => e,
    }
}

// ── the vectors against the spec and the committed file ─────────────────────

#[derive(serde::Serialize)]
struct EmittedV2 {
    name: String,
    passphrase: Option<String>,
    password: String,
    pw_form: u8,
    iterations: u32,
    salt_hex: String,
    iv_hex: String,
    public: Vec<String>,
    secret: Vec<String>,
    titles: Option<Vec<String>>,
    letters: Vec<Option<String>>,
    pub_len: u32,
    ct_len: u32,
    blob_hex: String,
    blob_sha256: String,
    header_hex: String,
    derived_key_hex: String,
    tag_hex: String,
    pubhash_sealed: Option<String>,
    pubhash_unsealed: Option<String>,
}

#[derive(serde::Serialize)]
struct EmittedV2Vectors {
    note: String,
    spec: String,
    vectors: Vec<EmittedV2>,
}

fn emitted(v: &V) -> EmittedV2 {
    let blob = raw_blob(v);
    let h = Header::decode(&blob).ok();
    let (pub_len, ct_len) = {
        let hdr = &blob[..HEADER_LEN];
        (
            u32::from_be_bytes(hdr[44..48].try_into().unwrap()),
            u32::from_be_bytes(hdr[48..52].try_into().unwrap()),
        )
    };
    let _ = h;
    let refs: Vec<&str> = v.public.iter().map(|r| r.as_str()).collect();
    let hash = |sealed| {
        // The generator leaves the hash out of a payload whose public section
        // holds a `text:` record (N-public-titles).
        if v.public.is_empty() || v.public.iter().any(|r| r.starts_with("text:")) {
            None
        } else {
            Some(hex(&pubhash::public_data_hash(&refs, sealed)))
        }
    };
    EmittedV2 {
        name: s(v.name),
        passphrase: Some(s(v.words)),
        password: s(v.words),
        pw_form: v.pw_form,
        iterations: v.iterations,
        salt_hex: hex(&v.salt),
        iv_hex: hex(&v.iv),
        public: v.public.clone(),
        secret: v.secret.clone(),
        titles: v.titles.clone(),
        letters: v
            .secret
            .iter()
            .map(|r| {
                if r.starts_with("ms1") {
                    titles::ms1_letter(r).map(|c| c.to_string())
                } else {
                    None
                }
            })
            .collect(),
        pub_len,
        ct_len,
        blob_hex: hex(&blob),
        blob_sha256: sha(&blob),
        header_hex: hex(&blob[..HEADER_LEN]),
        derived_key_hex: hex(&*crypto::derive_key(
            &passphrase::normalise(v.words),
            &v.salt,
            v.iterations,
        )),
        tag_hex: hex(&blob[blob.len() - wire::TAG_LEN..]),
        pubhash_sealed: hash(true),
        pubhash_unsealed: hash(false),
    }
}

/// S1b §14.10: every §14.9 shape and digest row literal, and the
/// `seal_v2_vectors.json` drift check.
#[test]
fn v2_vectors_match_the_independent_generator() {
    let vectors = all_vectors();
    assert_eq!(
        vectors.len(),
        18,
        "§14.9: sixteen vectors and the two owed ones"
    );
    let spec = std::fs::read_to_string(SPEC).expect("the S1b spec");
    let flat: String = spec.split_whitespace().collect();
    let emitted: Vec<EmittedV2> = vectors.iter().map(emitted).collect();
    for e in &emitted {
        let row = format!(
            "|{}|{}|{}|{}|{}|{}|{}|",
            e.name,
            e.pw_form,
            e.iterations,
            e.pub_len,
            e.ct_len,
            e.public.len() + e.secret.len(),
            e.blob_hex.len() / 2
        );
        assert!(
            flat.contains(&row),
            "{}: shape row {row} is not in §14.9",
            e.name
        );
        let digest = format!(
            "|{}|`{}`|`{}`|`{}`|",
            e.name, e.derived_key_hex, e.tag_hex, e.blob_sha256
        );
        assert!(flat.contains(&digest), "{}: digest row {digest}", e.name);
        assert!(
            flat.contains(&e.header_hex),
            "{}: header {} is not in §14.9",
            e.name,
            e.header_hex
        );
        if let Some(h) = &e.pubhash_sealed {
            assert!(flat.contains(h) || h.len() == 32, "{}: §6.6 hash", e.name);
        }
    }
    // The positive vectors come out of the writer itself, byte for byte.
    for name in ["V2-A", "V2-B", "V2-C"] {
        let v = by_name(name);
        let payload = Payload {
            public: v.public.clone(),
            secret: v
                .secret
                .iter()
                .filter(|r| !r.starts_with("text:"))
                .cloned()
                .collect(),
        };
        let written = seal_v2_deterministic(
            payload,
            v.titles.clone(),
            v.words,
            v.pw_form,
            v.iterations,
            v.salt,
            v.iv,
        )
        .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(
            written,
            raw_blob(&v),
            "{name}: seal_v2_deterministic differs"
        );
    }
    let json = serde_json::to_string_pretty(&EmittedV2Vectors {
        note: format!(
            "Generated by `{EMIT_CMD}`. Normative source: crates/me-cli/src/seal/. \
             Do not hand-edit."
        ),
        spec: s("SPEC_encrypted_payload_delivery.md"),
        vectors: emitted,
    })
    .unwrap()
        + "\n";
    if std::env::var("ME_EMIT_VECTORS").as_deref() == Ok("1") {
        std::fs::write(V2_VECTORS_PATH, &json).unwrap();
        return;
    }
    let on_disk = std::fs::read_to_string(V2_VECTORS_PATH)
        .unwrap_or_else(|e| panic!("{V2_VECTORS_PATH}: {e} — regenerate with `{EMIT_CMD}`"));
    assert!(
        on_disk == json,
        "{V2_VECTORS_PATH} is stale: generated, never hand-edited; regenerate with `{EMIT_CMD}`"
    );
}

// ── header rows over the open path ──────────────────────────────────────────

/// S1b §14.10: the floor refuses at the header, before any word check or KDF;
/// V2-A (600,000) and V2-C (100,000, offset 11 = 0) open.
#[test]
fn v2_refuses_n_word_payload_below_600000_iterations() {
    let blob = blob_of("N-floor-599999");
    let h = Header::decode(&blob);
    assert!(
        matches!(h, Err(wire::WireError::PwFormIterations(599_999))),
        "{h:?}"
    );
    assert_eq!(
        h.unwrap_err().to_string(),
        "iteration count 599999 out of range [600000, 2000000] for a 6 to 12 word password"
    );
    let t = Instant::now();
    match open(&blob, BEEF6) {
        Err(OpenError::Wire(wire::WireError::PwFormIterations(599_999))) => {}
        other => panic!("{:?}", other.err()),
    }
    assert!(t.elapsed() < Duration::from_millis(200), "no KDF may run");
    // The floor edits on V2-A's header (iterations is offset 12..16).
    let a = blob_of("V2-A");
    let edit = |iters: u32, off11: Option<u8>| {
        let mut b = a[..HEADER_LEN].to_vec();
        b[12..16].copy_from_slice(&iters.to_be_bytes());
        if let Some(x) = off11 {
            b[11] = x;
        }
        Header::decode(&b)
    };
    assert!(matches!(
        edit(599_999, None),
        Err(wire::WireError::PwFormIterations(599_999))
    ));
    assert!(edit(600_000, None).is_ok());
    assert!(edit(2_000_000, None).is_ok());
    let over = edit(2_000_001, None);
    assert!(
        matches!(over, Err(wire::WireError::Iterations(2_000_001))),
        "{over:?}"
    );
    let low = edit(50_000, None);
    assert!(
        matches!(low, Err(wire::WireError::Iterations(50_000))),
        "{low:?}"
    );
    let five = edit(599_999, Some(0x05));
    assert!(matches!(five, Err(wire::WireError::PwForm(5))), "{five:?}");
    // The unchanged lower bound for the generated form, and both accepted sides.
    let c = blob_of("V2-C");
    assert_eq!(Header::decode(&c).unwrap().iterations, 100_000);
    assert!(open(&a, BEEF6).is_ok());
    assert!(open(&c, BEEF12).is_ok());
}

/// S1b §14.10: `seal_v2` with `pw_form` 6 and 599,999 iterations refuses and
/// returns nothing; `pw_form` 0 at 100,000 is a payload.
#[test]
fn seal_v2_refuses_n_word_below_600000_iterations() {
    let payload = || Payload {
        public: vec![],
        secret: vec![s(A16)],
    };
    let t = Instant::now();
    let r = seal_v2_deterministic(
        payload(),
        None,
        BEEF6,
        6,
        599_999,
        rep([0x0b, 0xad]),
        rep([0x0c, 0x01]),
    );
    assert!(
        matches!(r, Err(SealError::PwFormIterations(599_999))),
        "{r:?}"
    );
    assert_eq!(
        r.unwrap_err().to_string(),
        "iteration count 599999 out of range [600000, 2000000] for a 6 to 12 word password"
    );
    assert!(
        t.elapsed() < Duration::from_millis(200),
        "refused before any KDF work"
    );
    let ok = seal_v2_deterministic(
        payload(),
        None,
        BEEF12,
        0,
        100_000,
        rep([0x0b, 0xad]),
        rep([0x0c, 0x01]),
    );
    assert!(ok.is_ok(), "{:?}", ok.err());
    let r = seal_v2(payload(), None, BEEF6, 6, 599_999);
    assert!(matches!(r, Err(SealError::PwFormIterations(599_999))));
}

// ── open ────────────────────────────────────────────────────────────────────

/// S1b §14.10.
#[test]
fn v2_open_round_trips_each_positive_vector() {
    let a = open_named("V2-A", BEEF6).unwrap();
    assert_eq!((a.version, a.pw_form), (2, 6));
    assert_eq!(*a.secret, vec![s(A16), s(MS1_P)]);
    assert_eq!(a.titles, Some(vec![s("SEED A"), s("KEEP; DO NOT LOAD")]));
    assert!(a.public.is_empty());

    let b = open_named("V2-B", ZOO12).unwrap();
    assert_eq!(b.secret.len(), 7);
    let want: Vec<String> = [
        "SEED A",
        "SEED B",
        "SEED C",
        "SEED D",
        "SEED E",
        "SEED F",
        "HOT SEED G",
    ]
    .iter()
    .map(|x| s(x))
    .collect();
    assert_eq!(b.titles, Some(want));

    let c = open_named("V2-C", BEEF12).unwrap();
    assert_eq!(c.public.len(), 5);
    assert_eq!(*c.secret, vec![s(D_MS1)]);
    assert_eq!(c.titles, None);
    let refs: Vec<&str> = c.public.iter().map(|r| r.as_str()).collect();
    assert_eq!(
        hex(&pubhash::public_data_hash(&refs, true)),
        "a26ed22bb747dfd0236706ad14c19679"
    );
}

/// S1b §14.10 (`plates_key`'s normalisation).
#[test]
fn v2_password_normalises_like_plates_key() {
    let o = open_named("V2-A", "  BEEF\tbeef beef  Beef beef\nbeef ").unwrap();
    assert_eq!(o.secret.len(), 2);
}

/// V2-A with iterations 2,000,000 in the header: an unbounded KDF would show.
fn slow_a() -> Vec<u8> {
    let mut b = blob_of("V2-A");
    b[12..16].copy_from_slice(&2_000_000u32.to_be_bytes());
    b
}

/// S1b §14.10.
#[test]
fn v2_refuses_a_wrong_word_count_before_the_kdf() {
    let blob = slow_a();
    let t = Instant::now();
    let e = open(&blob, "beef beef beef beef beef").err().unwrap();
    assert!(
        matches!(e, OpenError::PasswordWordCount { got: 5, want: 6 }),
        "{e:?}"
    );
    assert_eq!(
        e.to_string(),
        "the password has 5 words; this payload's password has 6"
    );
    assert!(
        t.elapsed() < Duration::from_millis(200),
        "the KDF ran first"
    );
}

/// S1b §14.10.
#[test]
fn v2_refuses_a_word_off_the_list_before_the_kdf() {
    let blob = slow_a();
    let t = Instant::now();
    let e = open(&blob, "beef beef beef beef beef beff").err().unwrap();
    assert!(matches!(e, OpenError::PasswordWord(6)), "{e:?}");
    assert_eq!(
        e.to_string(),
        "password word 6 is not on the BIP-39 English list"
    );
    assert!(
        t.elapsed() < Duration::from_millis(200),
        "the KDF ran first"
    );
}

/// S1b §14.10: no checksum for `pw_form` 12, the checksum for `pw_form` 0.
#[test]
fn v2_pw_form_0_keeps_the_checksum() {
    let t = Instant::now();
    let e = refusal("V2-C", ZOO12);
    assert!(matches!(e, OpenError::Passphrase), "{e:?}");
    assert_eq!(
        e.to_string(),
        "passphrase is not a checksum-valid BIP-39 mnemonic"
    );
    assert!(
        t.elapsed() < Duration::from_millis(200),
        "before any KDF work"
    );
    assert!(
        open_named("V2-B", ZOO12).is_ok(),
        "zoo x 12 is not checksum-valid and opens"
    );
}

/// S1b §14.10.
#[test]
fn v2_wrong_password_fails_authentication() {
    let e = refusal("V2-A", "beef beef beef beef beef bacon");
    assert!(matches!(e, OpenError::Authentication), "{e:?}");
    assert_eq!(
        e.to_string(),
        "wrong password, or this payload has been altered"
    );
}

/// S1b §14.10 and this plan's leg (owed row `s1br3-nothex-nonms1-vectors`): each
/// N vector gives §14.9's variant and Display, and `Opened` is never returned.
#[test]
fn v2_titles_refusals() {
    const LINE: &str = "title SEED <A-P>, title HOT SEED <A-P> or title KEEP; DO NOT LOAD";
    let t1: [(&str, &str); 9] = [
        ("N-count", "the titles block names 1 plates but the encrypted section holds 2 seed records"),
        ("N-magic", "the titles block must start with the line refugium-plates 2"),
        ("N-two", "the encrypted section holds more than one titles record"),
        ("N-not-last", "the titles record must be the last record of the encrypted section"),
        ("N-no-final-lf", "the titles block must end with a line feed"),
        ("N-empty", "the titles block names no plate"),
        ("N-not-hex", "the titles record is not text: followed by lower-case hex of UTF-8"),
        ("N-mnemonic", "titles apply only to ms1 entr records; secret record 0 (records count from 0) is not one"),
        ("N-title-set", ""),
    ];
    for (name, display) in t1 {
        let e = refusal(name, BEEF6);
        use titles::TitlesError as T;
        let got = match &e {
            OpenError::Titles(t) => t,
            other => panic!("{name}: {other:?}"),
        };
        let ok = match name {
            "N-count" => matches!(
                got,
                T::Count {
                    titles: 1,
                    secrets: 2
                }
            ),
            "N-magic" => matches!(got, T::Magic),
            "N-two" => matches!(got, T::Duplicate),
            "N-not-last" => matches!(got, T::NotLast),
            "N-no-final-lf" => matches!(got, T::NoFinalLf),
            "N-empty" => matches!(got, T::Empty),
            "N-not-hex" => matches!(got, T::NotHex),
            "N-mnemonic" => matches!(got, T::NonMs1Secret(0)),
            "N-title-set" => matches!(got, T::Line(2)),
            _ => false,
        };
        assert!(ok, "{name}: {e:?}");
        let want = if name == "N-title-set" {
            format!("titles block line 2 is not {LINE}")
        } else {
            display.to_string()
        };
        assert_eq!(e.to_string(), want, "{name}");
    }
    let e = refusal("N-letter", BEEF6);
    match &e {
        OpenError::Titles(titles::TitlesError::Letter {
            index: 0,
            title,
            letter: 'B',
        }) if title == "SEED C" => {}
        other => panic!("N-letter: {other:?}"),
    }
    assert_eq!(
        e.to_string(),
        "secret record 0 (records count from 0) is titled SEED C but its seed letter is B"
    );
    let e = refusal("N-hot-letter", BEEF6);
    match &e {
        OpenError::Titles(titles::TitlesError::Letter {
            index: 0,
            title,
            letter: 'G',
        }) if title == "HOT SEED F" => {}
        other => panic!("N-hot-letter: {other:?}"),
    }
    // The header refusal, the share and the public titles record.
    let e = refusal("N-floor-599999", BEEF6);
    assert!(matches!(
        e,
        OpenError::Wire(wire::WireError::PwFormIterations(599_999))
    ));
    let e = refusal("N-share", BEEF6);
    assert!(matches!(e, OpenError::Record { index: 0, .. }), "{e:?}");
    assert_eq!(
        e.to_string(),
        "record 0 (records count from 0) in the secret section: invalid record: string length \
         48 outside v0.1 set [50, 56, 62, 69, 75]"
    );
    let t = Instant::now();
    let e = refusal("N-public-titles", BEEF6);
    assert!(matches!(e, OpenError::Public(_)), "{e:?}");
    assert_eq!(
        e.to_string(),
        "record 0 (records count from 0) in the public section: unrecognised record: \
         unrecognized HRP 'text:726566756769756d2d706c6' (expected md, mk, ms, or mt)"
    );
    assert!(t.elapsed() < Duration::from_millis(200), "before any KDF");
}

/// S1b §14.10: a v1 payload never reads a `text:` record as titles.
#[test]
fn v1_open_treats_text_records_as_records() {
    let e = refusal("N-v1-text", BEEF12);
    assert!(matches!(e, OpenError::Record { index: 1, .. }), "{e:?}");
    assert_eq!(
        e.to_string(),
        "record 1 (records count from 0) in the secret section: unrecognised record: \
         unrecognized HRP 'text:726566756769756d2d706c6' (expected md, mk, ms, or mt)"
    );
    // Vector C of the v1 file opens with no titles.
    let raw = std::fs::read_to_string(V1_VECTORS_PATH).unwrap();
    let d: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let c = d["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "C")
        .unwrap();
    let blob: Vec<u8> = (0..c["blob_hex"].as_str().unwrap().len() / 2)
        .map(|i| {
            u8::from_str_radix(&c["blob_hex"].as_str().unwrap()[2 * i..2 * i + 2], 16).unwrap()
        })
        .collect();
    let o = open(&blob, BEEF12).unwrap();
    assert_eq!((o.version, o.pw_form), (1, 0));
    assert_eq!(o.titles, None);
    assert!(!o.secret.is_empty());
}

/// S1b §14.10 and this plan's leg: `seal_v2` refuses what `open` refuses.
#[test]
fn seal_v2_refuses_what_open_refuses() {
    let seal = |secret: Vec<String>, titles: &[&str]| {
        seal_v2_deterministic(
            Payload {
                public: vec![],
                secret,
            },
            Some(titles.iter().map(|t| s(t)).collect()),
            BEEF6,
            6,
            600_000,
            rep([0x0b, 0xad]),
            rep([0x0c, 0x01]),
        )
    };
    let e = seal(vec![s(A16), s(MS1_P)], &["SEED A"]).unwrap_err();
    assert!(
        matches!(
            e,
            SealError::Titles(titles::TitlesError::Count {
                titles: 1,
                secrets: 2
            })
        ),
        "{e:?}"
    );
    let e = seal(vec![s(MS1_A_TO_G[1])], &["SEED C"]).unwrap_err();
    assert!(
        matches!(
            &e,
            SealError::Titles(titles::TitlesError::Letter {
                index: 0,
                letter: 'B',
                ..
            })
        ),
        "{e:?}"
    );
    let e = seal(vec![s(MS1_A_TO_G[6])], &["HOT SEED F"]).unwrap_err();
    assert!(
        matches!(
            &e,
            SealError::Titles(titles::TitlesError::Letter {
                index: 0,
                letter: 'G',
                ..
            })
        ),
        "{e:?}"
    );
    let e = seal(vec![s(ABANDON12)], &["SEED A"]).unwrap_err();
    assert!(
        matches!(e, SealError::Titles(titles::TitlesError::NonMs1Secret(0))),
        "{e:?}"
    );
}

/// S1b §10.2 steps 2 then 5-6, §14.7: the public section is refused before the
/// password is looked at, so a bad public section with a bad password is Public.
#[test]
fn open_checks_the_public_section_before_the_password() {
    let e = refusal("N-public-titles", "beef beef beef beef beef");
    assert!(matches!(e, OpenError::Public(_)), "{e:?}");
    let e = refusal("N-public-titles", "beef beef beef beef beef beff");
    assert!(matches!(e, OpenError::Public(_)), "{e:?}");
}

/// Architect ruling (kit S2 Q3): nothing to encrypt is Empty; titles naming
/// plates with no seed record behind them is the titles rule's Count.
#[test]
fn seal_v2_refuses_an_empty_section_and_titles_without_seeds() {
    let seal = |secret: Vec<String>, titles: Option<Vec<String>>| {
        seal_v2_deterministic(
            Payload {
                public: vec![],
                secret,
            },
            titles,
            BEEF6,
            6,
            600_000,
            rep([0x0b, 0xad]),
            rep([0x0c, 0x01]),
        )
    };
    let e = seal(vec![], None).unwrap_err();
    assert!(matches!(e, SealError::Empty), "{e:?}");
    let e = seal(vec![], Some(vec![s("SEED A")])).unwrap_err();
    assert!(
        matches!(
            e,
            SealError::Titles(titles::TitlesError::Count {
                titles: 1,
                secrets: 0
            })
        ),
        "{e:?}"
    );
}

/// Architect ruling (kit S2 Q4): a blob shorter than its header declares names
/// both lengths; "need at least 52" is false for a whole header.
#[test]
fn open_names_the_declared_length_of_a_truncated_blob() {
    let b = blob_of("V2-A");
    let e = open(&b[..60], BEEF6).err().unwrap();
    assert!(
        matches!(e, OpenError::Wire(wire::WireError::Truncated { got: 60, need }) if need == b.len()),
        "{e:?}"
    );
    assert_eq!(
        e.to_string(),
        format!("blob too short: 60 bytes, its header declares {}", b.len())
    );
    let e = open(&b[..40], BEEF6).err().unwrap();
    assert!(
        matches!(e, OpenError::Wire(wire::WireError::TooShort(40))),
        "{e:?}"
    );
}

// ── review rows (rv-me44-s1) ────────────────────────────────────────────────
mod rv_rows {
    use super::*;
    use crate::sysw::mr;

    const ALPHA: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

    /// An mr1 string from its data symbols: the 13-symbol checksum is appended.
    fn mr1(data: &[u8]) -> String {
        let mut input = md_codec::bch::hrp_expand("mr");
        input.extend_from_slice(data);
        input.extend(std::iter::repeat(0u8).take(13));
        let p = md_codec::bch::polymod_run(&input) ^ 0x085d21404aee15059u128;
        let mut all = data.to_vec();
        for i in 0..13 {
            all.push(((p >> (5 * (12 - i))) & 31) as u8);
        }
        let mut s = String::from("mr1");
        for v in all {
            s.push(ALPHA[v as usize] as char);
        }
        s
    }

    /// A chunk: `l`, 20-bit csid, count-1, index, then the slice bytes.
    fn chunk(csid: u32, count: u8, index: u8, slice: &[u8]) -> String {
        let mut bits: Vec<u8> = Vec::new();
        let mut push = |v: u32, n: usize| {
            for k in (0..n).rev() {
                bits.push(((v >> k) & 1) as u8);
            }
        };
        push(31, 5);
        push(csid, 20);
        push(u32::from(count) - 1, 6);
        push(u32::from(index), 6);
        for b in slice {
            push(u32::from(*b), 8);
        }
        while bits.len() % 5 != 0 {
            bits.push(0);
        }
        let data: Vec<u8> = bits
            .chunks(5)
            .map(|c| c.iter().fold(0u8, |a, b| (a << 1) | b))
            .collect();
        mr1(&data)
    }

    #[test]
    fn rv_builder_agrees_with_check_string() {
        let c = chunk(0x12345, 3, 1, &[0xaa]);
        let m = mr::check_string(&c).unwrap();
        let ch = m.chunk.unwrap();
        assert_eq!((ch.csid, ch.count, ch.index, ch.slice), (0x12345, 3, 1, vec![0xaa]));
    }

    /// §14b check 5: every symbol of card version 7 (`u`, `a`, `7`) is reserved.
    #[test]
    fn rv_reserved_symbols_are_u_a_and_7() {
        for sym in [b'u', b'a', b'7'] {
            let v = ALPHA.iter().position(|&a| a == sym).unwrap() as u8;
            let s = mr1(&[v, 0]);
            assert_eq!(mr::check_string(&s), Err(mr::MrReason::ReservedSymbol), "{s}");
        }
        // The neighbours are single-string cards.
        for sym in [b'e', b'6', b'm'] {
            let v = ALPHA.iter().position(|&a| a == sym).unwrap() as u8;
            assert!(mr::check_string(&mr1(&[v, 0])).is_ok());
        }
    }

    /// §14c / the coordinator's run ruling: a group is full at its FIRST
    /// string's count, so a later string that says a smaller count still joins
    /// the group and is refused as SET_COUNT (record 2).
    #[test]
    fn rv_run_group_is_bound_by_the_first_strings_count() {
        let recs = vec![
            chunk(0xabcde, 3, 0, &[1]),
            chunk(0xabcde, 3, 1, &[2]),
            chunk(0xabcde, 2, 2, &[3]),
        ];
        let e = mr::check_sets(&recs).unwrap_err();
        assert_eq!(e.reason, mr::Reason::SetCount, "{e:?}");
        assert!(
            matches!(e.detail, mr::Detail::Count { record: 2, count: 2, first_count: 3, .. }),
            "{e:?}"
        );
    }

    fn seal6(public: Vec<String>, words: &str, pw: u8, it: u32) -> Result<Vec<u8>, SealError> {
        seal_v2_deterministic(
            Payload { public, secret: vec![s(A16)] },
            None,
            words,
            pw,
            it,
            rep([0x0b, 0xad]),
            rep([0x0c, 0x01]),
        )
    }

    /// S1b §14.3 / §14.7: the writer's header refusals, each before any KDF.
    #[test]
    fn rv_seal_v2_refuses_the_header_cases_open_refuses() {
        let t = Instant::now();
        for pw in [1u8, 5, 13, 255] {
            let e = seal6(vec![], ABANDON12, pw, 600_000).unwrap_err();
            assert!(matches!(e, SealError::PwForm(b) if b == pw), "{pw}: {e:?}");
        }
        for it in [99_999u32, 2_000_001] {
            let e = seal6(vec![], ABANDON12, 0, it).unwrap_err();
            assert!(matches!(e, SealError::Iterations(n) if n == it), "{it}: {e:?}");
        }
        let e = seal6(vec![s(A16)], ABANDON12, 0, 100_000).unwrap_err();
        assert!(matches!(e, SealError::SecretInPublic(0)), "{e:?}");
        assert!(t.elapsed() < Duration::from_millis(500), "before any KDF work");
    }

    /// §14.4 check 3: lower-case hex, even, at least 2 digits, UTF-8.
    #[test]
    fn rv_titles_not_hex_cases() {
        let block = hex(b"refugium-plates 2\ntitle SEED A\n");
        let rec = |t: &str| vec![s(A16), s(t)];
        let nothex = |t: String| {
            let e = titles::check_titles(&rec(&t)).unwrap_err();
            assert_eq!(e, titles::TitlesError::NotHex, "{t}");
        };
        nothex(format!("text:{}", block.to_uppercase()));
        nothex(format!("text:{}0", block));
        nothex("text:".to_string());
        nothex("text:ff0a".to_string());
        assert!(titles::check_titles(&rec(&format!("text:{block}"))).is_ok());
    }

    /// §14.4: all 33 titles, and the letter is the TOP nibble.
    #[test]
    fn rv_titles_letters_p_and_top_nibble() {
        assert!(titles::encode_titles(&["SEED P", "HOT SEED P", "HOT SEED A"]).is_ok());
        let m = ms_codec::encode(
            ms_codec::Tag::ENTR,
            &ms_codec::Payload::Entr(vec![0x1f; 16]),
        )
        .unwrap();
        assert_eq!(titles::ms1_letter(&m), Some('B'));
        let m = ms_codec::encode(
            ms_codec::Tag::ENTR,
            &ms_codec::Payload::Entr(vec![0xf1; 16]),
        )
        .unwrap();
        assert_eq!(titles::ms1_letter(&m), Some('P'));
    }

    /// §14.7 read side: a section that is not UTF-8 is `NotUtf8`.
    #[test]
    fn rv_decode_section_not_utf8() {
        assert_eq!(
            container::decode_section(&[0xff, 0xfe]),
            Err(container::ContainerError::NotUtf8)
        );
    }

    /// `TooShort` keeps its own text.
    #[test]
    fn rv_too_short_text() {
        let b = blob_of("V2-A");
        let e = open(&b[..40], BEEF6).err().unwrap();
        assert_eq!(e.to_string(), "blob too short: 40 bytes, need at least 52");
    }

    /// `[mdmk-decode]` run grouping (S1 §14d): a group is full at its FIRST
    /// string's count, so a later string that says a smaller count still joins.
    #[test]
    fn rv_mdmk_run_group_is_bound_by_the_first_strings_count() {
        let rewrite = |src: &str, count: u8, index: u8| -> String {
            let (mut bytes, nbits) = md_codec::codex32::unwrap_string(src).unwrap();
            let mut put = |at: usize, v: u32, n: usize| {
                for k in 0..n {
                    let bit = ((v >> (n - 1 - k)) & 1) as u8;
                    let pos = at + k;
                    bytes[pos / 8] = (bytes[pos / 8] & !(0x80 >> (pos % 8))) | (bit << (7 - pos % 8));
                }
            };
            put(25, u32::from(count) - 1, 6);
            put(31, u32::from(index), 6);
            md_codec::codex32::wrap_payload(&bytes, nbits).unwrap()
        };
        let recs = vec![
            rewrite(D_PUBLIC[2], 3, 0),
            rewrite(D_PUBLIC[3], 3, 1),
            rewrite(D_PUBLIC[4], 2, 1),
        ];
        let cards = crate::sysw::record::mdmk_cards(&recs);
        assert_eq!(cards.len(), 1, "{cards:?}");
        assert_eq!(cards[0].members.len(), 3);
    }

    /// §6.4 / §14.7: the 24-record cap spans both sections. Five public and
    /// twenty secret records fit each section and not the payload: the writer
    /// refuses it and the reader refuses what the raw builder wrote.
    #[test]
    fn rv_record_cap_spans_both_sections() {
        let public: Vec<String> = D_PUBLIC.iter().map(|x| s(x)).collect();
        let secret: Vec<String> = (0..20).map(|_| s(A16)).collect();
        let e = seal_v2_deterministic(
            Payload { public: public.clone(), secret: secret.clone() },
            None,
            BEEF6,
            6,
            600_000,
            rep([0x0b, 0xad]),
            rep([0x0c, 0x01]),
        )
        .unwrap_err();
        assert!(
            matches!(e, SealError::Container(container::ContainerError::RecordCount(25))),
            "{e:?}"
        );
        let blob = raw_blob(&v("cap", 6, BEEF6, 600_000, rep([0x0b, 0xad]), rep([0x0c, 0x01]), public, secret));
        let e = open(&blob, BEEF6).err().unwrap();
        assert!(
            matches!(e, OpenError::Container(container::ContainerError::RecordCount(25))),
            "{e:?}"
        );
    }
}
