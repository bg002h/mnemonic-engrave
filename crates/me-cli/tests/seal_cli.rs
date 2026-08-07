use assert_cmd::Command;
use predicates::prelude::*;

const MD1: &str = "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3";
const MD1B: &str = "md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374";
const MD1C: &str = "md1fv9wjpqsp2026hh65xpvugtfhd9792zxgunymm0a82pdju6442q0jskj9gzfaqmz";
const MS1: &str = "ms10entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw";

fn me() -> Command {
    Command::cargo_bin("me").unwrap()
}

/// Find the generated passphrase on stderr.
///
/// **Do NOT match on "a line with 12 whitespace-separated tokens".** Two lines
/// of `me seal`'s own prose have exactly 12 tokens — the passphrase header
/// (`passphrase — write this down and store it APART from the machine:`) and
/// `RECORD THIS WHOLE LINE. The device shows the same value; if it`. A
/// token-count heuristic returns the header, which made the §2.3 containment
/// assertion below VACUOUS: it degenerated to `!uf2.contains("passphrase")`,
/// and a mutation copying the real twelve words into the UF2's padding left the
/// test GREEN.
fn passphrase_line(err: &str) -> Option<&str> {
    err.lines().find(|l| {
        let w: Vec<&str> = l.split_whitespace().collect();
        w.len() == 12 && w.iter().all(|t| t.chars().all(|c| c.is_ascii_lowercase()))
    })
}

#[test]
fn seals_and_prints_the_passphrase_to_stderr_only() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");
    let a = me()
        .args(["seal", MS1, "--seal-secret", "--out", out.to_str().unwrap()])
        .assert()
        .success()
        // §9 is "stderr only", and the name of this test claims it. Finding the
        // words on stderr does not prove they are ONLY there: adding a
        // `println!` copy alongside the `eprintln!` survived the whole 169-test
        // suite. `me seal` writes nothing to stdout at all, so assert exactly
        // that — a stdout copy would land the twelve words in any
        // `me seal … > log`, pipeline capture or CI artefact.
        .stdout(predicate::str::is_empty());
    let err = String::from_utf8(a.get_output().stderr.clone()).unwrap();
    let words = passphrase_line(&err).expect("the 12-word passphrase must reach stderr");
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(bytes.len() % 512, 0);
    // §2.3: the passphrase must never land beside the ciphertext it opens.
    // Assert on the LONGEST word: a 3-letter BIP-39 word ("act", "air") has a
    // ~1-in-30,000 chance of appearing by chance in ~500 random ciphertext
    // bytes — a flake nobody would diagnose.
    let longest = words.split_whitespace().max_by_key(|w| w.len()).unwrap();
    assert!(
        !String::from_utf8_lossy(&bytes).contains(longest),
        "no passphrase word may appear in the UF2"
    );
}

/// §8 / §2.2a: the prohibition is load-bearing. Assert the flag is ABSENT —
/// `.failure()` alone would also pass if someone added `--passphrase` with
/// validation that happened to reject the value.
#[test]
fn there_is_no_passphrase_flag() {
    me().args(["seal", MD1, "--passphrase", "hunter2"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
    me().args(["seal", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--passphrase").not());
}

#[test]
fn there_is_no_addr_flag() {
    me().args(["seal", MD1, "--addr", "0x10000000"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
    me().args(["seal", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--addr").not());
}

#[test]
fn refuses_ms1_without_the_opt_in_flag() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");
    me().args(["seal", MS1, "--out", out.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--seal-secret"));
    assert!(!out.exists(), "nothing may be written on the refusal path");
}

/// A public-only payload prompts for nothing and prints no passphrase (§9).
#[test]
fn public_only_payload_prints_no_passphrase() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");
    let a = me()
        .args([
            "seal",
            "--plaintext",
            MD1,
            "--plaintext",
            MD1B,
            "--plaintext",
            MD1C,
            "--out",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();
    let err = String::from_utf8(a.get_output().stderr.clone()).unwrap();
    assert!(
        passphrase_line(&err).is_none(),
        "no passphrase may be printed when nothing is encrypted"
    );
    let b = std::fs::read(&out).unwrap();
    assert_eq!(&b[48..52], &[0, 0, 0, 0], "ct_len must be zero");
}

#[test]
fn refuses_a_secret_in_the_public_section() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");
    me().args(["seal", "--plaintext", MS1, "--out", out.to_str().unwrap()])
        .assert()
        .failure();
    assert!(!out.exists());
}

#[test]
fn refuses_space_grouped_input_with_an_actionable_message() {
    me().args(["seal", "md1fv9w jpqpqpm6", "--out", "/dev/null"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--group-size 0"));
}

#[test]
fn refuses_out_of_range_iterations() {
    for bad in ["5", "3000000000"] {
        me().args([
            "seal",
            MS1,
            "--seal-secret",
            "--out",
            "/dev/null",
            "--iterations",
            bad,
        ])
        .assert()
        .failure();
    }
}

/// The printed hash MUST describe the blob that was written. Removing the CLI
/// trim (for the CR rule) made `public` raw argv while the blob's public section
/// is trimmed — measured: one leading space gave a byte-identical blob and a
/// different hash, on the only integrity control an unsealed payload has.
#[test]
fn printed_hash_matches_me_hash_regardless_of_surrounding_whitespace() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");
    let padded = me()
        .args([
            "seal",
            "--plaintext",
            &format!("  {MD1}  "),
            "--plaintext",
            MD1B,
            "--plaintext",
            MD1C,
            "--out",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();
    let err = String::from_utf8(padded.get_output().stderr.clone()).unwrap();
    // Anchor on the banner, NOT on a token count. `load:  picotool load
    // --verify <path>   (machine in BOOTSEL)` also has exactly 8 whitespace
    // tokens; `.find()` returns the hash line first only because it happens to
    // come earlier. A token-count heuristic is precisely what made the §2.3
    // passphrase assertion vacuous — see `passphrase_line` above.
    let lines: Vec<&str> = err.lines().collect();
    let i = lines
        .iter()
        .position(|l| l.starts_with("public data hash ("))
        .expect("the hash banner line");
    let printed = lines[i + 1].trim().to_string();
    let derived = me()
        .args(["hash", "--unsealed", MD1, MD1B, MD1C])
        .assert()
        .success();
    let expect = String::from_utf8(derived.get_output().stdout.clone()).unwrap();
    assert_eq!(
        printed,
        expect.trim(),
        "me seal's printed hash must equal me hash's for the same records"
    );
}

/// Kills the mutation that trims at the CLI before `encode_section` ever sees
/// the record. Rewriting the `plaintext` binding in `run_seal_cli` to
/// `.map(|s| s.trim().to_string())` makes the CLI ACCEPT a CR-bearing record —
/// exit 0, UF2 written, blob and hash both correct — and left the entire suite
/// green. §6.4 says CRLF is rejected, not tolerated; without this test that
/// normative refusal can be deleted with nothing to notice. The mutation-table
/// row for it once named a manual `me seal` invocation, which is not a killer —
/// the same defect round 3 graded C-2.
///
/// **Pass the COMPLETE card set.** `MD1`/`MD1B`/`MD1C` are three chunks of one
/// md1 card, and `check_public` → `decode_public_set` runs BEFORE
/// `encode_section`'s CR scan. A first version of this test sent `MD1` alone,
/// so it died on `d-card: chunk set incomplete: got 1 chunks, expected 3`
/// identically whether the CLI trimmed or not — it could not fail for the right
/// reason, and could not distinguish the mutant at all. Round 5 caught it by
/// running the test; the build gate had only COMPILED it.
#[test]
fn refuses_a_record_carrying_a_cr() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");
    me().args([
        "seal",
        "--plaintext",
        MD1,
        "--plaintext",
        MD1B,
        "--plaintext",
        &format!("{MD1C}\r"),
        "--out",
        out.to_str().unwrap(),
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "record 2 contains '\\r', which is the record separator",
    ));
    assert!(!out.exists(), "a refused seal must not leave a file behind");
}

/// THE MIXED SHAPE — `--plaintext` records AND an encrypted payload in one
/// invocation. This is spec vector D, and it is the **only** shape in which
/// `run_seal_cli`'s `sealed` argument to `public_data_hash` can be wrong: with
/// no public section nothing is printed, and with no secret section the flag is
/// `false` either way.
///
/// Before this test, 11 of 11 CLI tests passed either a positional payload or
/// `--plaintext`, never both. Rewriting the call to
/// `public_data_hash(&refs, false)` therefore survived the whole 169-test suite
/// while printing vector E's UNSEALED digest under a "SEALED" banner.
///
/// §6.6: the `sealed` byte "is what makes a downgrade visible", and this hash is
/// the only integrity control an unsealed payload has. A wrong value here is not
/// a cosmetic slip — the operator writes it down, every honest comparison then
/// mismatches (teaching them that mismatches are normal), and a ciphertext-strip
/// produces a payload whose device-displayed hash is exactly what they recorded.
///
/// Assert the LITERAL, not just agreement with `me hash --sealed`: the same
/// self-referential trap that let both destructive UF2 constants mutate freely
/// (fixed in `4938bf0`) applies to any assertion whose two sides move together.
#[test]
fn mixed_payload_prints_the_sealed_hash_not_the_unsealed_one() {
    let mk1 = "mk1qpz63tpqqsq3dg4m5wdx5fvqqvzg3vs7mpf0rz2j43zpzpxk0rtjkqkhwreqp6hm7qnp3a8wdvtz6t2k4uxu6ykwxcp9vqugfjyx733cf59g";
    let mk2 = "mk1qpz63tppkeg9pdvqz5744004gvzecsknw6tu25yv3exfhkl6w5zm9e4t24aqdah5585wn3e4xdut8";
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");

    let a = me()
        .args([
            "seal",
            MS1,
            "--seal-secret",
            "--plaintext",
            mk1,
            "--plaintext",
            mk2,
            "--plaintext",
            MD1,
            "--plaintext",
            MD1B,
            "--plaintext",
            MD1C,
            "--out",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();
    let err = String::from_utf8(a.get_output().stderr.clone()).unwrap();
    let lines: Vec<&str> = err.lines().collect();
    let i = lines
        .iter()
        .position(|l| l.starts_with("public data hash ("))
        .expect("the hash banner line");

    // The banner itself carries the record count and the shape word, which §9
    // requires to match what the device displays.
    assert_eq!(
        lines[i], "public data hash (5 records, SEALED):",
        "the banner must report 5 public records and the SEALED shape"
    );

    // Spec §11.4 vector D. The Go port (Plan B) binds to this exact value.
    assert_eq!(
        lines[i + 1].trim(),
        "a26e d22b b747 dfd0 2367 06ad 14c1 9679",
        "a mixed payload must print the SEALED digest"
    );
    // And explicitly NOT vector E's, which is what the downgrade-blind mutant
    // prints. Pinned separately so the failure names the actual confusion.
    assert_ne!(
        lines[i + 1].trim(),
        "70f3 e35a acf7 47db c40f 8376 91aa 61e0",
        "printing the UNSEALED digest for a sealed payload disarms §6.6"
    );

    // Agreement with the operator's own re-derivation months later (§6.6's
    // whole purpose), on top of the literal.
    let derived = me()
        .args(["hash", "--sealed", mk1, mk2, MD1, MD1B, MD1C])
        .assert()
        .success();
    let expect = String::from_utf8(derived.get_output().stdout.clone()).unwrap();
    assert_eq!(lines[i + 1].trim(), expect.trim());
}

#[test]
fn me_hash_reproduces_both_shapes() {
    let mk1 = "mk1qpz63tpqqsq3dg4m5wdx5fvqqvzg3vs7mpf0rz2j43zpzpxk0rtjkqkhwreqp6hm7qnp3a8wdvtz6t2k4uxu6ykwxcp9vqugfjyx733cf59g";
    let mk2 = "mk1qpz63tppkeg9pdvqz5744004gvzecsknw6tu25yv3exfhkl6w5zm9e4t24aqdah5585wn3e4xdut8";
    me().args(["hash", "--unsealed", mk1, mk2, MD1, MD1B, MD1C])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "70f3 e35a acf7 47db c40f 8376 91aa 61e0",
        ));
    me().args(["hash", "--sealed", mk1, mk2, MD1, MD1B, MD1C])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "a26e d22b b747 dfd0 2367 06ad 14c1 9679",
        ));
}
