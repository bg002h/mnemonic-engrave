//! bh-me-C1 (FIX_me_recovery_checklist_partial_keys.md): `me bundle` names the
//! key slots a partly keyed md1 does not carry. Rows T1-T8 plus the taproot
//! shape row (review N3). Fixtures are signet, BIP-39 published test-vector
//! seeds, byte-identical to `design/evidence/fix-me-checklist-partial-keys/`
//! (gate step C checks that); the expected NOTE lines and `missing_keys` are
//! read from the same directory.

use assert_cmd::Command;

const K0: &[&str] = &["md1yzfds3cf2qqvzrs3pstucw0za5znwrg3hcc5xg5qx8qrjy0gwq7w8d"];
const K1: &[&str] = &[
    "md1ft8g4pqpqjtvywz2sqrqsuygvzlxrnchdq5ms6yd7x9pj9qps4g9qw5l04ulf9q3xd9lsj4j",
    "md1ft8g4pqtkt8uxhhznqfjlfclnm8pdwwn5cpcpzqh0xrk63qadmjrsp4dr4q6dtedxsc7ejcs",
    "md1ft8g4pqnumq2xgka5dkjrpeuy4guurq8v5qh5cvuhegru533f2nvw4qqdukhk6pxcxaf3",
];
const K1ONLY: &[&str] = &[
    "md1fjvucpqpqjtvywz2sqrqsuygvzlxrnchdq5ms6yd7x9pj9qps4g9xnu9qjscydwj3q9chsdv",
    "md1fjvucpqd2n7dlgre6mmlfx5awrafpg6f75ftm44a84khu7xnjjsj5pezm8ut8htu8qhc9hme",
    "md1fjvucpq528yw08rtt6jhlhehe457pqkg3rl3tv6xapgs03jevwfah2qqwu5rdsg3reu76",
];
const K02: &[&str] = &[
    "md1f74f2pspqjtvywz2sqrqsuygvzlxrnchdq5ms6yd7x9pj9qps4c9q820h67ajelp4ac5czvs6ngmlg28fdfd5",
    "md1f74f2ps06w8u7ecttn5axqwqgs9mesak5g8twusuqdtgag0nvpgezmk3k6gv88sj4rnsvqasvd3lqa07yvmya",
    "md1f74f2psjsz7npnjl9q0jjx992d364djar4ss8mrm09h0gg24mm2dgwch7u5lc749eml7y2dgvk7vw38xxekyf",
    "md1f74f2psupw0x30h7qrsf2swfmnldklgrnhpnsy77rf4cstchc24lwp4h27u36wxu79u2lspgssptqw072wq",
];
const K3: &[&str] = &[
    "md1fwydpzspqjtvywz2sqrqsuygvzlxrnchdq5ms6yd7x9pj9qps4u8s820h67ajelp4ac5sh4l0v03fmt0cq",
    "md1fwydpzsvpxta8r70vu94e6wnq8qygzaucwm2yr4hwgwqx45w58ekq5v3dmgmdyxrncf2s7txh90qflahs8",
    "md1fwydpzssuurq8v5qh5cvuhegru533f2nvw4xnu9qjj4flxl5puadal5n2whp75s5dyl2qjnrkah7a99mzv",
    "md1fwydpzs627ad0fad4l835u55y4qwgkel3gu3euudd022l7lxlxkncyzezy079dngm59zq3glnkdq5fn4pe",
    "md1fwydpz3qlr9jcunmw5kew36cgra3ahjmh5y92aa4x58vtlw20u02juallz9xkqh8nghms7ltlxevtj3ug4",
    "md1fwydpz30squz25rjwulmdh6quacvup8hs6dwyz797z40msdd6hhywn3h830zhu423s7l6cecqk8",
];
const K1NOFP: &[&str] = &[
    "md1fuwkmpqpqjtvywz2sqrqsuygvz7yrnchdq4zsezsqc25zs820h67ajec38r06cmjvlegt",
    "md1fuwkmpq0p4ac5czvh6w8u7ecttn5axqwqgs9mesak5g8twusuqdtgagg8207hmhd03nqz",
    "md1fuwkmpqhnvpgezmk3k6gv88sj4rnsvqajsz7npnjl9q0jjx992d365a2wvfpza9wajl",
];
const TR1: &[&str] = &[
    "md1fmhkapqpq2tvywz2sqrqf9gtcgu79mg9dcdzxlz92p8f7zsff25ln06qn0dpeqwwfgd9d",
    "md1fmhkapqg7wk7l6f48tsl2g2xj04z27ad0fad4l835u55y4qwgkel3gusa4c5pml9qd02c",
    "md1fmhkapq3euudd022l7lxlxkncyzezy079dngm59zp7xt93e8kagqqppen33rh8es3p",
];
const MK1_SLOT1: &[&str] = &[
    "mk1qp6sprpqqsq6j95aw6ux3r03z5zrtp70vlv9wmd8c2p992n7dlgre6mmlfx5awrafpg6f75ftm44a84khu7xnjjsj5pezm8y99wkx54fvt5p",
    "mk1qp6sprppdnlz3ernec66754la7d7dd8sg9jyglu2mx3hg2yruvktrj0d6n2qy056ftrgp4z9pw7aw",
];

const EV: &str = "../../design/evidence/fix-me-checklist-partial-keys";

fn expected(case: &str) -> (Vec<String>, serde_json::Value) {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(EV);
    let notes = std::fs::read_to_string(base.join(format!("{case}.notes"))).unwrap();
    let json = std::fs::read_to_string(base.join(format!("{case}.missing.json"))).unwrap();
    (
        notes.lines().map(str::to_string).collect(),
        serde_json::from_str(&json).unwrap(),
    )
}

/// Run `me bundle` on the fixtures (stdin, one string per line); assert exit 0,
/// the `me: NOTE` stderr lines equal the case's `.notes` exactly, and the
/// manifest's `missing_keys` equals the case's `.missing.json` (`null` =
/// absent). Returns stderr.
fn run_case(case: &str, inputs: &[&[&str]]) -> String {
    let stdin: String = inputs
        .iter()
        .flat_map(|f| f.iter())
        .map(|l| format!("{l}\n"))
        .collect();
    let out = Command::cargo_bin("me")
        .unwrap()
        .arg("bundle")
        .write_stdin(stdin)
        .assert()
        .success();
    let o = out.get_output();
    let stderr = String::from_utf8(o.stderr.clone()).unwrap();
    let stdout = String::from_utf8(o.stdout.clone()).unwrap();
    let notes: Vec<String> = stderr
        .lines()
        .filter(|l| l.starts_with("me: NOTE"))
        .map(str::to_string)
        .collect();
    let (want_notes, want_missing) = expected(case);
    assert_eq!(notes, want_notes, "{case}: NOTE lines differ\n{stderr}");
    let m: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let got = m
        .get("missing_keys")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    assert_eq!(got, want_missing, "{case}: missing_keys differs");
    stderr
}

#[test]
fn partial_keys_one_of_three_names_two_missing_slots() {
    let stderr = run_case("k1", &[K1]);
    assert_eq!(
        stderr.lines().next().unwrap(),
        "me: backup needs 3 public plates, plus one ms1 plate per seed you hold:"
    );
}

#[test]
fn partial_keys_middle_slot_only_names_slots_zero_and_two() {
    run_case("k1only", &[K1ONLY]);
}

#[test]
fn partial_keys_two_of_three_names_the_one_missing_slot() {
    run_case("k02", &[K02]);
}

#[test]
fn partial_keys_missing_fingerprint_renders_bare_slot() {
    run_case("k1nofp", &[K1NOFP]);
}

#[test]
fn fully_keyed_policy_has_no_missing_keys() {
    run_case("k3", &[K3]);
}

#[test]
fn keyless_template_lists_every_slot_and_keeps_the_template_note() {
    run_case("k0", &[K0]);
}

#[test]
fn template_and_partial_policies_are_reported_in_plate_order() {
    run_case("mix", &[K1, K0]);
}

#[test]
fn partial_keys_note_fires_with_the_mk1_card_present() {
    run_case("k1mk", &[K1, MK1_SLOT1]);
}

/// Review N3: D2 is shape-independent. A taproot `tr(@0, pk(@1))` md1 keyed on
/// `@1` only names `@0`.
#[test]
fn partial_keys_taproot_key_on_second_slot_names_slot_zero() {
    run_case("tr1", &[TR1]);
}
