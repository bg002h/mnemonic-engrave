//! Unit tests for §4's cascade.
//!
//! The shared vector file is the gate for the COLUMNS — `host_admits`, the
//! device column, the 37 gate rows. What it does not reach is the parser's
//! internal behaviour: which branch claims an adversarial document, whether a
//! wrapper leaves a descriptor single-sig, whether the canonical re-encoding is
//! a fixed point. Those are here, and every measured value carries where it was
//! measured.

use super::*;

/// The fork's own `nonstandard/parse_test.go` fixture — three cosigners, and
/// the checksum the device itself computes.
const FIXTURE: &str = "wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan/0/*,[f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge/0/*,[c5d87297/48h/0h/0h/2h]xpub6DjrnfAyuonMaboEb3ZQZzhQ2ZEgaKV2r64BFmqymZqJqviLTe1JzMr2X2RfQF892RH7MyYUbcy77R7pPu1P71xoj8cDUMNhAMGYzKR4noZ/0/*))#hfwurrvt";

const XPUB: &str = "xpub6C9j4wAxxkWN4cq8G4N2mkV6NrGGhnLFCGdh8GsYY1xreEveW5YEXJMjDZWLAcnZ26xqVft5FmgBxPixdMGoVQZMdtEJRRADxrn4facoGnx";

/// The vector file's own `promotion/15-bare-tpub-host-refused` key — four of
/// §6's five SLIP-132 remedies name a TESTNET target, and an `xpub` standing in
/// for them would test a mainnet wallet the operator does not hold.
const TPUB: &str = "tpubDCXMbAzeg2TpLR1yiFM7yfpThyMvhAqJjuDzUpvgsvikPXbMaJPKfk2ZTbb7h7jnp1Vk7FPwnsWEeaDa2D83Nr1ehUyc6wpTYpNURb6Qt26";

fn ok(input: &str) -> Parsed {
    cascade(&normalise(input)).unwrap_or_else(|e| panic!("{input}: {e:?}"))
}

// ── §4.6 ───────────────────────────────────────────────────────────────────

#[test]
fn whitespace_is_absorbed_before_the_cascade_runs() {
    let base = "wpkh([4bbaa801/84h/0h/0h]xpub6C9j4wAxxkWN4cq8G4N2mkV6NrGGhnLFCGdh8GsYY1xreEveW5YEXJMjDZWLAcnZ26xqVft5FmgBxPixdMGoVQZMdtEJRRADxrn4facoGnx/<0;1>/*)";
    let want = ok(base).encode();
    for spelling in [
        format!("{base}\n"),
        format!(" {base}"),
        format!("{base}\r\n"),
        format!("\n\n  {base}  \n"),
    ] {
        assert_eq!(ok(&spelling).encode(), want, "{spelling:?}");
    }
}

#[test]
fn crlf_inside_a_bluewallet_file_normalises() {
    let bw = "Name: x\nPolicy: 1 of 1\nDerivation: m/48'/0'/0'/2'\nFormat: P2WSH\ndc567276: xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan\n";
    let lf = ok(bw);
    let crlf = ok(&bw.replace('\n', "\r\n"));
    assert_eq!(lf.encode(), crlf.encode());
}

// ── §4.1 precedence ────────────────────────────────────────────────────────

/// The adversarial case §4.1 records: a JSON document whose LABEL is spelled
/// like a BlueWallet header. Branch 1 splits the whole single line on `": "`
/// and gets the key `{"label":"Name`, which is not a known header and not a
/// valid xpub — so branch 3 claims it.
#[test]
fn a_json_label_spelled_like_a_bluewallet_header_is_claimed_by_the_json_branch() {
    let doc = format!(
        "{{\"label\":\"Name: x\",\"descriptor\":\"wpkh([4bbaa801/84h/0h/0h]{XPUB}/<0;1>/*)\"}}"
    );
    let d = ok(&doc);
    assert_eq!(d.branch, Branch::Json);
    assert_eq!(d.title.as_deref(), Some("Name: x"));
}

#[test]
fn a_bare_key_is_never_claimed_by_the_json_branch() {
    // Measured: `{"label":"x","descriptor":"xpub…"}` is REFUSED while the same
    // key alone is promoted. Promotion is only for a bare key arriving bare.
    let wrapped = format!("{{\"label\":\"x\",\"descriptor\":\"{XPUB}\"}}");
    assert!(cascade(&normalise(&wrapped)).is_err());
    assert_eq!(ok(XPUB).branch, Branch::PromotedKey);
}

// ── §4.2 ───────────────────────────────────────────────────────────────────

/// The four narrowings, each with the §6 row it must select. `me`'s branch 1
/// FAILS on all four, which is what makes the vector file's `format` column
/// read `none` on every one.
#[test]
fn the_four_bluewallet_narrowings_each_fail_branch_one_with_their_own_cause() {
    let key = "xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan";
    type Want = fn(&BlueWalletError) -> bool;
    let cases: Vec<(&str, String, Want)> = vec![
        (
            "no Format:",
            format!("Name: x\nPolicy: 1 of 1\nDerivation: m/48'/0'/0'/2'\ndc567276: {key}\n"),
            |e| matches!(e, BlueWalletError::NoFormat),
        ),
        (
            "zero cosigners",
            "Name: only\n".to_string(),
            |e| matches!(e, BlueWalletError::ZeroCosigners),
        ),
        (
            "Derivation after the keys",
            format!("Name: x\nPolicy: 1 of 1\nFormat: P2WSH\ndc567276: {key}\nDerivation: m/48'/0'/0'/2'\n"),
            |e| matches!(e, BlueWalletError::NoOrigin { after_keys: true, .. }),
        ),
        (
            "no Derivation at all",
            format!("Name: x\nPolicy: 1 of 1\nFormat: P2WSH\ndc567276: {key}\n"),
            |e| matches!(e, BlueWalletError::NoOrigin { after_keys: false, .. }),
        ),
        (
            "short fingerprint",
            format!("Name: x\nPolicy: 1 of 1\nDerivation: m/48'/0'/0'/2'\nFormat: P2WSH\nab: {key}\n"),
            |e| matches!(e, BlueWalletError::BadFingerprint(_)),
        ),
    ];
    for (what, input, want) in cases {
        let errs = cascade(&normalise(&input)).expect_err(what);
        let bw = errs.bluewallet.as_ref().expect(what);
        assert!(want(bw), "{what}: got {bw:?}");
    }
}

/// The `Name:` gate runs BEFORE the key-count check, and the reason is
/// measured: `deadbeef: <xpub>` alone carries no `Policy:` header, so the
/// device's own order fails it at `bluewallet: expected 0 keys, but got 1`
/// (measured against `parseBlueWalletDescriptor` at fork `1f09537`). §6's count
/// row would then print a `Policy:` line the operator's file does not contain.
#[test]
fn a_headerless_cosigner_line_is_refused_for_the_missing_name_not_the_count() {
    let errs = cascade(&normalise(&format!("deadbeef: {XPUB}"))).unwrap_err();
    assert!(
        matches!(
            errs.bluewallet,
            Some(BlueWalletError::NoName { cosigners: 1, .. })
        ),
        "{:?}",
        errs.bluewallet
    );
}

// ── §4.3 ───────────────────────────────────────────────────────────────────

#[test]
fn the_checksum_is_validated_and_a_doubled_one_is_refused() {
    assert!(cascade(&normalise(FIXTURE)).is_ok());
    let body = FIXTURE.strip_suffix("#hfwurrvt").unwrap();
    for bad in [
        format!("{body}#00000000"),
        format!("{body}#hfwurrvt#hfwurrvt"),
    ] {
        let errs = cascade(&normalise(&bad)).unwrap_err();
        assert_eq!(errs.bip380, Some(Bip380Error::InvalidChecksum), "{bad}");
    }
}

/// The control-flow subtlety `Parse` turns on: after a `sh(` wrapper, when the
/// SECOND `parseFunc` fails there is no multi form and the descriptor stays
/// single-sig. Getting this wrong turns `sh(wpkh(KEY))` into a parse error.
#[test]
fn a_wrapper_over_a_bare_key_stays_single_sig() {
    let d = ok(&format!("sh(wpkh([4bbaa801/49h/0h/0h]{XPUB}/<0;1>/*))"));
    assert_eq!(d.script, Script::P2SH_P2WPKH);
    assert_eq!(d.multi, None);
    assert_eq!(d.keys.len(), 1);
}

#[test]
fn me_reads_multi_where_the_device_refuses_it() {
    let body = FIXTURE
        .strip_suffix("#hfwurrvt")
        .unwrap()
        .replace("sortedmulti(", "multi(");
    let d = ok(&body);
    assert_eq!(d.multi, Some(Multi::Unsorted));
    // …and the canonical re-encoding keeps it `multi`. `me` never rewrites a
    // policy: `sortedmulti` differs from `multi` at spend time.
    assert!(d.encode().contains("wsh(multi("), "{}", d.encode());
}

#[test]
fn both_hardening_spellings_parse_to_one_result() {
    let a = ok(&format!("wpkh([4bbaa801/84'/0'/0']{XPUB}/<0;1>/*)"));
    let b = ok(&format!("wpkh([4bbaa801/84h/0h/0h]{XPUB}/<0;1>/*)"));
    assert_eq!(a.encode(), b.encode());
}

#[test]
fn an_uppercase_fingerprint_is_accepted_and_normalises_to_lowercase() {
    let d = ok(&format!("wpkh([4BBAA801/84h/0h/0h]{XPUB}/<0;1>/*)"));
    assert!(d.encode().contains("[4bbaa801/"), "{}", d.encode());
}

/// `parsePath` cuts on the FIRST `;` and checks `start > end`, so a
/// three-element group and a reversed pair are both parse REFUSALS — never
/// admitted shapes conjunct 7 has to exclude.
#[test]
fn three_element_and_reversed_ranges_are_parse_refusals() {
    for tail in ["<0;1;2>/*", "<1;0>/*"] {
        let input = format!("wpkh([4bbaa801/84h/0h/0h]{XPUB}/{tail})");
        let errs = cascade(&normalise(&input)).unwrap_err();
        assert!(
            matches!(
                errs.bip380,
                Some(Bip380Error::Key(KeyError::InvalidChildrenPath(_)))
            ),
            "{tail}: {:?}",
            errs.bip380
        );
    }
}

// ── §4.4 ───────────────────────────────────────────────────────────────────

#[test]
fn json_fields_match_case_insensitively_and_unknown_fields_are_ignored() {
    let inner = format!("wpkh([4bbaa801/84h/0h/0h]{XPUB}/<0;1>/*)");
    let doc = format!(
        "{{\n  \"Label\": \"L\",\n  \"blockheight\": 481824,\n  \"Descriptor\": \"{inner}\",\n  \
         \"devices\": [{{\"type\":\"other\"}}]\n}}\n"
    );
    let d = ok(&doc);
    assert_eq!(d.branch, Branch::Json);
    assert_eq!(d.title.as_deref(), Some("L"));
}

#[test]
fn a_missing_label_is_fine_and_an_empty_object_reports_the_inner_reason() {
    let inner = format!("wpkh([4bbaa801/84h/0h/0h]{XPUB}/<0;1>/*)");
    assert!(cascade(&normalise(&format!("{{\"descriptor\":\"{inner}\"}}"))).is_ok());
    let errs = cascade(&normalise("{}")).unwrap_err();
    assert!(
        matches!(
            errs.json,
            Some(JsonError::Inner { ref inner, .. }) if **inner == Bip380Error::MissingOpenParen
        ),
        "{:?}",
        errs.json
    );
}

// ── §4.5 ───────────────────────────────────────────────────────────────────

/// §4.5, R0's I5: the announcement prints BOTH forms, and the second assertion
/// is why. `XPUB` is a depth-3 key, so its canonical re-serialisation is
/// byte-identical and printing one form would look sufficient. `COSIGNER` is
/// the depth-4 fixture key: promoted to `pkh(…)`, `Key.ExtendedKey()` rebuilds
/// depth from the INVENTED `44h/0h/0h` and the canonical is a base58 string the
/// operator has never seen. The one check the announcement exists for is "is
/// that my key?", and on this input printing only the canonical makes that
/// check fail on a correct result.
#[test]
fn promotion_announces_both_the_supplied_key_and_the_inferred_wallet() {
    const COSIGNER: &str = "xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan";
    let shallow = ok(XPUB);
    assert!(shallow.promoted);
    let a = crate::descriptor::gate::promotion_announcement(&shallow).expect("promoted");
    assert!(a.contains(XPUB), "the key AS SUPPLIED must appear verbatim");
    assert!(
        a.contains(&shallow.encode()),
        "and the inferred wallet in full"
    );

    let deep = ok(&format!("[4bbaa801/44h/0h/0h]{COSIGNER}"));
    let canonical = deep.encode();
    assert!(
        !canonical.contains(COSIGNER),
        "a depth-4 key promoted to a depth-3 origin must RE-SERIALISE: {canonical}"
    );
    let a = crate::descriptor::gate::promotion_announcement(&deep).expect("promoted");
    assert!(
        a.contains(COSIGNER) && a.contains(&canonical),
        "both forms, or the operator cannot answer \"is that my key?\": {a}"
    );
}

#[test]
fn a_testnet_key_is_never_promoted() {
    const TPUB: &str = "tpubDCXMbAzeg2TpLR1yiFM7yfpThyMvhAqJjuDzUpvgsvikPXbMaJPKfk2ZTbb7h7jnp1Vk7FPwnsWEeaDa2D83Nr1ehUyc6wpTYpNURb6Qt26";
    let errs = cascade(&normalise(TPUB)).unwrap_err();
    assert!(matches!(errs.promotion, Some(PromotionError::TestnetKey)));
}

// ── Key material ───────────────────────────────────────────────────────────

/// **The curve check, cross-checked against the device's own parser.**
///
/// Each key below is a hand-built 78-byte envelope with a chosen `x`, and the
/// verdicts were MEASURED against `bip380.ParseExtendedKey` at fork `1f09537`
/// (probe run at implementation time; the Go column is the second word):
///
/// ```text
/// off-curve x=5          REFUSE hdkey: invalid extended key
/// off-curve x=7          REFUSE hdkey: invalid extended key
/// on-curve x=1           ACCEPT
/// on-curve G             ACCEPT
/// x=p (out of field)     REFUSE hdkey: invalid extended key
/// ```
///
/// This is the one predicate in the cascade with no vector row behind it, and
/// it is in the direction §7 forbids — a host that skipped it would ADMIT a key
/// the device REFUSES.
#[test]
fn the_curve_check_agrees_with_the_devices_own_parser() {
    const CASES: &[(&str, bool, &str)] = &[
        ("off-curve x=5", false, "xpub6DHhW8e7CSp87DVf8cFUseRy45xmuSZoUmj4tmh9yxFkR5LnFyonCKvKVAMmxksBaHDoXmrw64b8E2QUMqrEyhunDCQc8kTZneY9aSczrXT"),
        ("off-curve x=7", false, "xpub6DHhW8e7CSp87DVf8cFUseRy45xmuSZoUmj4tmh9yxFkR5LnFyonCKvKVAMmxksBaHDoXmrw64b8E2QUMqrEyhunDCQc8kTZneY9aiLBhxM"),
        ("on-curve x=1", true, "xpub6DHhW8e7CSp87DVf8cFUseRy45xmuSZoUmj4tmh9yxFkR5LnFyonCKvKVAMmxksBaHDoXmrw64b8E2QUMqrEyhunDCQc8kTZneY9a1Zdkz2"),
        ("on-curve G", true, "xpub6DHhW8e7CSp87DVf8cFUseRy45xmuSZoUmj4tmh9yxFkR5LnFyonCKvKVBHPk3FiCTun3B132MrYYQP5SbiVeN52xWvYxnTB4ZaBVujmcBn"),
        ("x=p (out of field)", false, "xpub6DHhW8e7CSp87DVf8cFUseRy45xmuSZoUmj4tmh9yxFkR5LnFyonCKvKVCJX8zKDRhu2fbSdzeD8BM1FGxTxdQkezqUiNu9T3V2JbeP6sMN"),
    ];
    for (what, device_accepts, key) in CASES {
        let got = cascade(&normalise(&format!(
            "wpkh([4bbaa801/84h/0h/0h]{key}/<0;1>/*)"
        )));
        assert_eq!(got.is_ok(), *device_accepts, "{what}");
        if !device_accepts {
            let errs = got.unwrap_err();
            assert!(
                matches!(
                    errs.bip380,
                    Some(Bip380Error::Key(KeyError::NotAPublicPoint))
                ),
                "{what}: refused for the wrong reason: {:?}",
                errs.bip380
            );
        }
    }
}

// ── The canonical re-encoding ──────────────────────────────────────────────

/// §7 requirement 4's fixed point, on the side that produces it. The fork's own
/// fixture re-encodes to itself, checksum included — so `me`'s encoder agrees
/// with `Descriptor.Encode()` on a string the device wrote.
#[test]
fn the_forks_own_fixture_is_a_fixed_point_of_the_encoder() {
    assert_eq!(ok(FIXTURE).encode(), FIXTURE);
}

#[test]
fn the_canonical_normalises_a_slip132_version_to_xpub() {
    const ZPUB: &str = "zpub6qpFgGWoG7bKmDDMvmwHBvg6inZAb2KF2Vg8h4fKJ2ickSZ71PsMmRg1FyRWAS6PqPCSzd5CB6PHixx64k6q5svZNZd9bEoCWJuMSkSRzJx";
    let e = ok(ZPUB).encode();
    assert!(e.starts_with("wpkh(xpub"), "{e}");
}

// ── §6h: the remedy must be EXECUTABLE ─────────────────────────────────────

/// **§6h — every remedy `me` prints is an input `me` admits, on ALL THREE
/// generators.**
///
/// `SPEC_constellation_cli_uniformity` §6h: where a row says "supply the
/// descriptor", it prints the descriptor with the operator's own key and origin
/// substituted in, not a placeholder. An ELIDED key (`xpub6C9j4wAxxk…acoGnx`)
/// is a placeholder wearing the operator's fingerprint, and it is not base58 —
/// so feeding the remedy back through the cascade is what catches it.
///
/// **M3 (IMPL-P1's review):** this covered ONE of the three generators. An
/// elision introduced in either of the other two would have passed the whole
/// 485-test suite. The loop now runs all three and its floor counts them, so a
/// dropped generator reds the suite instead of shrinking the coverage silently.
#[test]
fn every_remedy_me_prints_is_an_input_me_admits() {
    const ZPUB: &str = "zpub6qpFgGWoG7bKmDDMvmwHBvg6inZAb2KF2Vg8h4fKJ2ickSZ71PsMmRg1FyRWAS6PqPCSzd5CB6PHixx64k6q5svZNZd9bEoCWJuMSkSRzJx";
    let mut executable = 0;

    // Generator 1 — `suggested_descriptor_for`, through the two promotion rows.
    for input in [
        format!("[4bbaa801/86'/0'/0']{XPUB}"),
        format!("[4bbaa801/84'/0'/1']{ZPUB}"),
        format!("[4bbaa801/48'/0'/0'/2']{XPUB}"),
        format!("[4bbaa801/84/0/0]{ZPUB}"),
    ] {
        let errs = cascade(&normalise(&input)).expect_err(&input);
        let k = match errs.promotion.as_ref().expect(&input) {
            PromotionError::PathNotInferable(k) | PromotionError::AccountNotZero(k) => k.clone(),
            other => panic!("{input}: {other:?}"),
        };
        let Some(remedy) = crate::descriptor::refusal::suggested_descriptor_for(&k.origin, &k)
        else {
            // The unhardened purpose: `me` names NO script. Asserted for real
            // below, in `an_unrecognised_purpose_names_no_script`.
            continue;
        };
        // The multisig forms carry `<the other cosigners>`, information the
        // operator holds and `me` does not. The exemption is narrow and stated:
        // a multisig cosigner key has no single-key wallet to name.
        if remedy.contains("<the other cosigners>") {
            continue;
        }
        assert!(
            crate::descriptor::host_admits(&remedy),
            "generator 1: the remedy is not an input `me` admits: {remedy}"
        );
        executable += 1;
    }

    // Generator 2 — `promotion_fingerprint_no_path`'s full-origin key.
    for supplied in [format!("[4bbaa801]{XPUB}"), format!("[4bbaa801]{ZPUB}")] {
        let remedy = crate::descriptor::refusal::remedy_full_origin(&supplied);
        assert!(
            crate::descriptor::host_admits(&remedy),
            "generator 2: the remedy is not an input `me` admits: {remedy}"
        );
        // And the row's own text must QUOTE the remedy it built, or the two can
        // drift into an elided copy of an executable string.
        let printed = crate::descriptor::refusal::promotion_fingerprint_no_path(&supplied);
        assert!(
            printed.text.contains(&remedy),
            "generator 2: the row prints a remedy it did not build:\n{}",
            printed.text
        );
        executable += 1;
    }

    // Generator 3 — `unsupported_key_version`'s FIVE per-version remedies.
    for v in [
        KeyVersion::Ypub,
        KeyVersion::Upub,
        KeyVersion::Vpub,
        KeyVersion::UpubCap,
        KeyVersion::VpubCap,
    ] {
        // The converted key is the row's own input re-serialised under the
        // target version, which is what the real caller passes.
        let converted = if matches!(v, KeyVersion::Ypub) {
            XPUB
        } else {
            TPUB
        };
        let remedy = crate::descriptor::refusal::remedy_for_version(v, converted, None);
        let printed = crate::descriptor::refusal::unsupported_key_version(v, converted, None);
        assert!(
            printed.text.contains(&remedy),
            "generator 3: the row prints a remedy it did not build:\n{}",
            printed.text
        );
        assert!(
            !remedy.contains('…'),
            "generator 3: an elided key in a remedy: {remedy}"
        );
        if remedy.contains("<the other cosigners>") {
            // `Upub`/`Vpub` are testnet MULTISIG accounts: no single-key remedy
            // exists, so the descriptor is necessarily incomplete. Skipped from
            // the executable count, not from the elision check above.
            continue;
        }
        assert!(
            crate::descriptor::host_admits(&remedy),
            "generator 3: the remedy is not an input `me` admits: {remedy}"
        );
        // §6's per-version SOUNDNESS, independently tabulated here rather than
        // asked of the code that produced it: the script must be the VERSION's.
        let want = match v {
            KeyVersion::Ypub | KeyVersion::Upub => "sh(wpkh(",
            _ => "wpkh(",
        };
        assert!(
            remedy.starts_with(want),
            "generator 3: {v:?} names {remedy}, which is not the script that version \
             declares"
        );
        executable += 1;
    }

    // M3's floor, raised and re-derived rather than guessed: generator 1 runs 2
    // (of 4 inputs, one is the multisig-cosigner exemption and one is M2's
    // no-script case), generator 2 runs 2, generator 3 runs 3 (of 5 versions,
    // `Upub`/`Vpub` are testnet MULTISIG accounts with no single-key form).
    // P1's loop stood at 3. Without the floor the loop passes by skipping.
    assert_eq!(
        executable, 7,
        "remedies actually run back through the cascade"
    );
}

/// **M2 (IMPL-P1's review) — `me` names no script for an unrecognised purpose.**
///
/// The previous `_ => P2PKH` fallback handed the operator a LEGACY P2PKH remedy
/// for a NATIVE-SEGWIT key whenever the purpose step was unrecognised — an
/// unhardened `84/0/0`, which is a real coordinator export bug. The remedy was
/// executable and admitted, so §6h was satisfied and the §6h gate above passed
/// on it: a gate that can only see elision cannot see an unsound guess.
///
/// This is the assertion that CAN fail on one. Restoring the fallback makes
/// `suggested_descriptor_for` return `Some("pkh(…)")` and reds both halves.
#[test]
fn an_unrecognised_purpose_names_no_script() {
    const ZPUB: &str = "zpub6qpFgGWoG7bKmDDMvmwHBvg6inZAb2KF2Vg8h4fKJ2ickSZ71PsMmRg1FyRWAS6PqPCSzd5CB6PHixx64k6q5svZNZd9bEoCWJuMSkSRzJx";
    // Both halves of "unrecognised": the UNHARDENED purpose (a real coordinator
    // export bug, and the one the review measured) and a HARDENED purpose
    // outside the six. Each reaches a different `None` in `purpose_script`, and
    // a mutation that restores the guess in only one place must still red.
    let mut checked = 0;
    for (input, why) in [
        (
            format!("[4bbaa801/84/0/0]{ZPUB}"),
            "an unhardened `84/0/0` is not `m/84h/0h/0h`",
        ),
        (
            format!("[4bbaa801/99'/0'/0']{ZPUB}"),
            "purpose 99h is outside the six `me` maps",
        ),
    ] {
        let errs = cascade(&normalise(&input)).expect_err(&input);
        let k = match errs.promotion.as_ref().expect(&input) {
            PromotionError::PathNotInferable(k) => k.clone(),
            other => panic!("{input}: {other:?}"),
        };
        assert!(
            crate::descriptor::refusal::suggested_descriptor_for(&k.origin, &k).is_none(),
            "{why}, and naming a script for it is a guess"
        );
        let printed = crate::descriptor::refusal::promotion_path_not_inferable(
            &k,
            crate::descriptor::refusal::suggested_descriptor_for(&k.origin, &k).as_deref(),
        );
        for guess in ["pkh(", "wpkh(", "sh(wpkh(", "tr("] {
            assert!(
                !printed.text.contains(&format!(": {guess}")),
                "{why}: the refusal hands back a guessed script:\n{}",
                printed.text
            );
        }
        assert!(
            printed.text.contains("will not guess"),
            "{why}: the refusal must SAY it is not guessing:\n{}",
            printed.text
        );
        checked += 1;
    }
    assert_eq!(
        checked, 2,
        "both `None` arms of the purpose map are exercised"
    );
}
