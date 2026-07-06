use mnemonic_engrave::convert;

// Convert-level goldens (B1, F14/F6): byte-pinned NDEF for each valid input
// string, spanning short/max md1 and short/chunk mk1. These bytes are also
// decoded through the INDEPENDENT SeedHammer Go oracle in cross_lang.rs (Step 7)
// — never asserted only via me's own decode_text_tlv.
struct Vector {
    name: &'static str,
    input: &'static str,
    golden: &'static [u8],
}

// Max-valid md1 sits at the codex32 93-symbol cap (96 chars incl. HRP); mk1-short
// / mk1-chunk reuse the existing 80/111-char mk-codec v0.1 fixtures.
const VECTORS: &[Vector] = &[
    Vector {
        name: "md1-short",
        input: "md1yqpqqxqq8xtwhw4xwn4qh",
        golden: include_bytes!("vectors/md1-short.ndef"),
    },
    Vector {
        name: "md1-max",
        input: "md15kj6tfd95kj6tfd95kj6tfd95kj6tfd95kj6tfd95kj6tfd95kj6tfd95kj6tfd95kj6tfd95kj6tfd9uguh8nmgfllzz",
        golden: include_bytes!("vectors/md1-max.ndef"),
    },
    Vector {
        name: "mk1-short",
        input: "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x",
        golden: include_bytes!("vectors/mk1-short.ndef"),
    },
    Vector {
        name: "mk1-chunk",
        input: "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf",
        golden: include_bytes!("vectors/mk1-chunk.ndef"),
    },
];

#[test]
fn md1_short_matches_golden() {
    let golden = include_bytes!("vectors/md1-short.ndef");
    let got = convert("md1yqpqqxqq8xtwhw4xwn4qh").unwrap();
    assert_eq!(&got[..], &golden[..]);
}

#[test]
fn all_vectors_match_golden_ndef() {
    for v in VECTORS {
        let got = convert(v.input).unwrap_or_else(|e| panic!("{} convert failed: {e}", v.name));
        assert_eq!(&got[..], v.golden, "golden mismatch for {}", v.name);
    }
}

#[test]
fn vectors_cover_the_full_bech32_alphabet() {
    // Union coverage (B1): every bech32 charset symbol must appear in the data
    // part of ≥1 vector, so the goldens exercise the whole engravable alphabet.
    const BECH32: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let mut seen = std::collections::HashSet::new();
    for v in VECTORS {
        let data = &v.input[v.input.find('1').unwrap() + 1..];
        seen.extend(data.chars());
    }
    let missing: Vec<char> = BECH32.chars().filter(|c| !seen.contains(c)).collect();
    assert!(
        missing.is_empty(),
        "bech32 symbols not covered by any vector: {missing:?}"
    );
}
