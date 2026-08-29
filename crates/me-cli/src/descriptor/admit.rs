//! **§4.7 — the admitted grammar, as an explicit CONJUNCTION.**
//!
//! R0 found that every safety property that is not a script form had fallen out
//! of the section — key version bytes, key-count bounds, network consistency,
//! origin-path presence — so the predicate is stated as eight conjuncts and
//! §7's row list is derivable from it. **Script shape is ONE conjunct, not the
//! whole rule.**
//!
//! | # | conjunct | where it is enforced |
//! | --- | --- | --- |
//! | 1 | shape — the seven forms, plus the three `multi` twins on the md1 path | here, and it is the one PATH-DEPENDENT conjunct |
//! | 2 | threshold `1 ≤ k ≤ n` | here |
//! | 3 | key count — 15 under a direct `sh`, 20 under `wsh`/`sh(wsh)` | here |
//! | 4 | version bytes in §4.3's five | here **and, first, in the cascade** |
//! | 5 | one network across all keys | here |
//! | 6 | a key with a fingerprint carries a non-empty origin path | here **and, first, in the cascade's branch 1** |
//! | 7 | use-site path in the closed five-member set | here |
//! | 8 | key identity — no origin collision, no duplicated slot | here |
//!
//! **Conjuncts 4 and 6 cannot fail here, and that is a measured fact rather
//! than an oversight.** Every key reaches this predicate through
//! [`super::cascade`], which refuses a non-admitted version inside
//! `parse_extended_key` (which is why `neither/full-origin-ypub` carries
//! `format: "none"` and not `format: "bip380"`), and refuses an origin-less
//! BlueWallet key inside branch 1 (which is why all five `narrowed-4.2` rows
//! carry `format: "none"`). They are stated here anyway because §4.7 is the
//! NORMATIVE predicate and P2's in-process builder will call it over
//! descriptors this cascade did not produce. The IMPL-P1 report's mutation
//! table records which vector row reds the upstream site instead.

use super::cascade::{Derivation, Key, Multi, Parsed, Script};
use super::refusal::{self, Refusal};

/// Which `--as` value's accept set is being asked about. §4.7 conjunct 1 is the
/// only conjunct that differs between them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Path {
    /// §5.2's device-facing set: the seven forms. A `Descriptor` record can
    /// never carry `multi`, so §7's invariant is untouched.
    Descriptor,
    /// §5.3's set: the seven forms plus the three `multi` twins, which md1
    /// carries natively and the device's descriptor parser refuses.
    Md1,
}

/// §4.7's predicate. `Ok(())` means every conjunct holds for `path`.
pub fn admit(d: &Parsed, path: Path) -> Result<(), Refusal> {
    conjunct_1_shape(d, path)?;
    conjunct_2_threshold(d)?;
    conjunct_3_key_count(d)?;
    conjunct_4_versions(d)?;
    conjunct_5_network(d)?;
    conjunct_6_origins(d)?;
    conjunct_7_use_site(d)?;
    conjunct_8_key_identity(d)?;
    Ok(())
}

/// Conjunct 1. The seven forms, and on the md1 path the three `multi` twins.
fn conjunct_1_shape(d: &Parsed, path: Path) -> Result<(), Refusal> {
    use Script::*;
    let single = matches!(d.script, P2PKH | P2WPKH | P2SH_P2WPKH | P2TR);
    let multisig_slot = matches!(d.script, P2WSH | P2SH_P2WSH | P2SH);
    match (d.multi, single, multisig_slot) {
        // pkh(KEY) · wpkh(KEY) · sh(wpkh(KEY)) · tr(KEY)
        (None, true, _) => Ok(()),
        // wsh(sortedmulti) · sh(wsh(sortedmulti)) · sh(sortedmulti)
        (Some(Multi::Sorted), _, true) => Ok(()),
        // The md1-path widening — and under `--as descriptor` the PERMANENT
        // refusal, in every build.
        (Some(Multi::Unsorted), _, true) => match path {
            Path::Md1 => Ok(()),
            Path::Descriptor => Err(refusal::multi_under_descriptor()),
        },
        // wsh(KEY) / sh(KEY) — `Parse` builds these as Singlesig, and they are
        // not descriptors: measured `Supported=false`, no derivable address.
        (None, _, true) => Err(refusal::key_in_script_slot()),
        // tr(sortedmulti(…)) — taproot multisig is `multi_a`/`sortedmulti_a`.
        (Some(m), _, _) if d.script == P2TR => Err(refusal::taproot_multisig(m)),
        // wpkh(sortedmulti) / pkh(sortedmulti) / sh(wpkh(sortedmulti))
        (Some(m), true, _) => Err(refusal::multi_in_single_key_script(m)),
        // Unreachable: `single` and `multisig_slot` partition the seven
        // scripts, so every (multi, script) pair is matched above. Kept total
        // rather than `unreachable!()`, because a panic on an operator path is
        // never the right answer to a future enum arm.
        (Some(m), _, _) => Err(refusal::multi_in_single_key_script(m)),
        (None, false, false) => Err(refusal::key_in_script_slot()),
    }
}

/// Conjunct 2. `1 ≤ k ≤ n`. The device makes NO threshold check at all: it
/// accepts `sortedmulti(0, …)` and `sortedmulti(-1, …)` and derives real
/// addresses for both.
fn conjunct_2_threshold(d: &Parsed) -> Result<(), Refusal> {
    if d.multi.is_none() {
        return Ok(());
    }
    let n = d.keys.len();
    if d.threshold < 1 {
        return Err(refusal::threshold_below_one(d.threshold));
    }
    if d.threshold > n as i64 {
        return Err(refusal::threshold_exceeds_keys(d.threshold, n));
    }
    Ok(())
}

/// Conjunct 3 (BIP-383). **The bound is the redeemScript's, not the
/// ordering's**, so it reads over `multi` identically.
///
/// Under a DIRECT `sh(…)` the multi's own output script IS the redeemScript —
/// one script element capped at 520 bytes, and 16 compressed keys need 547. In
/// `sh(wsh(…))` the redeemScript is the 34-byte `OP_0 <sha256>`, so only
/// `OP_CHECKMULTISIG`'s 20-key consensus limit binds — which is why a 16-key
/// `sh(wsh(sortedmulti(…)))` is a SPENDABLE wallet and is an accepted row.
fn conjunct_3_key_count(d: &Parsed) -> Result<(), Refusal> {
    if d.multi.is_none() {
        return Ok(());
    }
    let n = d.keys.len();
    let (max, form) = match d.script {
        Script::P2SH => (15usize, "sh(…)"),
        Script::P2WSH => (20, "wsh(…)"),
        Script::P2SH_P2WSH => (20, "sh(wsh(…))"),
        // Unreachable after conjunct 1; the 20-key consensus limit is the
        // weaker of the two, so a future arm cannot be admitted by accident.
        _ => (20, "this script"),
    };
    if n > max {
        return Err(refusal::key_count_exceeded(n, form));
    }
    Ok(())
}

/// Conjunct 4. See the module note: the cascade refuses these first, so this
/// cannot fail from a cascade-produced descriptor.
fn conjunct_4_versions(d: &Parsed) -> Result<(), Refusal> {
    for k in &d.keys {
        if !k.version.admitted() {
            let origin = refusal::origin_prefix(k);
            return Err(refusal::unsupported_key_version(
                k.version,
                &k.canonical_string(),
                Some(origin.as_str()).filter(|s| !s.is_empty()),
            ));
        }
    }
    Ok(())
}

/// Conjunct 5. A mixed `xpub`/`tpub` `sortedmulti` is ACCEPTED by the device's
/// parser and re-parses clean, and `address.Receive` then refuses it — so the
/// record would reach programs whose whole job is deriving addresses they
/// cannot derive.
fn conjunct_5_network(d: &Parsed) -> Result<(), Refusal> {
    let Some(first) = d.keys.first() else {
        return Ok(());
    };
    for (i, k) in d.keys.iter().enumerate().skip(1) {
        if k.network != first.network {
            return Err(refusal::mixed_network(i, k, first));
        }
    }
    Ok(())
}

/// Conjunct 6. `MasterFingerprint != 0 ⇒ len(DerivationPath) > 0` — the
/// predicate §4.2 states over the CANONICAL string, because `Descriptor.encode`
/// emits `[…]` iff `mfp != 0` and `ParseKey` then requires a `/` at offset 8.
///
/// An ALL-ZERO fingerprint is the one case where a key legitimately carries no
/// origin block, so this conjunct does not bind it — §4.2's `--as descriptor`
/// WARNING covers that loss instead, and refusing would reject files several
/// coordinators legitimately emit.
fn conjunct_6_origins(d: &Parsed) -> Result<(), Refusal> {
    for k in &d.keys {
        if k.fingerprint != 0 && k.origin.is_empty() {
            return Err(refusal::bluewallet_no_origin(k.fingerprint, false));
        }
    }
    Ok(())
}

/// Conjunct 7. The closed set `{absent, /*, /i/*, <i;i+1>, <i;i+1>/*}`.
///
/// Everything else in `parsePath`'s grammar is refused as UNMEASURED, per the
/// closed-set rule — including the two classes measured BROKEN: a hardened
/// use-site component (the device silently derives the UNhardened child) and a
/// non-consecutive multipath (`address.Receive` errors while
/// `address.Supported` still returns true).
fn conjunct_7_use_site(d: &Parsed) -> Result<(), Refusal> {
    for k in &d.keys {
        use_site_ok(&k.children)?;
    }
    Ok(())
}

fn use_site_ok(children: &[Derivation]) -> Result<(), Refusal> {
    use Derivation::*;
    let plain_wildcard = Wildcard { hardened: false };
    let ok = match children {
        [] => true,
        [w] if *w == plain_wildcard => true,
        [Child {
            hardened: false, ..
        }, w]
            if *w == plain_wildcard =>
        {
            true
        }
        [Range { start, end }] => *end == start + 1,
        [Range { start, end }, w] if *w == plain_wildcard => *end == start + 1,
        _ => false,
    };
    if ok {
        return Ok(());
    }
    // Which of §6's three use-site rows: the two measured-broken classes get
    // their own text, and everything else is the closed set's own row.
    let hardened = children.iter().any(|d| {
        matches!(
            d,
            Child { hardened: true, .. } | Wildcard { hardened: true }
        )
    });
    if hardened {
        return Err(refusal::use_site_hardened());
    }
    if children
        .iter()
        .any(|d| matches!(d, Range { start, end } if *end != start + 1))
    {
        return Err(refusal::use_site_non_consecutive());
    }
    Err(refusal::use_site_out_of_set(children))
}

/// Conjunct 8 — the two impossible-wallet checks the Rust-primary `md-codec`
/// enforces ON ENCODE (F-217/F-218). The PUBLISHED 0.42.0 crate `me` links
/// predates them, so `me` enforces both HOST-SIDE, on both `--as` paths.
/// Convergence with the primary, not leading.
///
/// **(b) is keyed on the USE SITE, not the origin** (r2's NEW-I1): the same
/// xpub at `<0;1>/*` and `<2;3>/*` is a legal two-chain wallet, measured, and
/// the device derives a distinct address for each.
fn conjunct_8_key_identity(d: &Parsed) -> Result<(), Refusal> {
    // (a) One origin identifies exactly one key.
    for i in 0..d.keys.len() {
        for j in (i + 1)..d.keys.len() {
            let (a, b) = (&d.keys[i], &d.keys[j]);
            if same_origin(a, b) && a.identity() != b.identity() {
                return Err(refusal::key_identity(i, j, &origin_text(a)));
            }
        }
    }
    // (b) No two slots carry the same (xpub, use-site path).
    for i in 0..d.keys.len() {
        for j in (i + 1)..d.keys.len() {
            let (a, b) = (&d.keys[i], &d.keys[j]);
            if a.identity() == b.identity() && a.children == b.children {
                return Err(refusal::key_identity_duplicate(i, j));
            }
        }
    }
    Ok(())
}

/// Two keys "declare the same origin" only when both actually declare one: an
/// absent fingerprint means "master unknown", which is not a claim about
/// identity and cannot contradict another key's.
fn same_origin(a: &Key, b: &Key) -> bool {
    a.fingerprint != 0 && a.fingerprint == b.fingerprint && a.origin == b.origin
}

fn origin_text(k: &Key) -> String {
    format!(
        "{:08x}{}",
        k.fingerprint,
        super::cascade::path_encode(&k.origin)
    )
}

// ───────────────────────────────────────────────────────────────────────────
// §5.3 — md1 representability
// ───────────────────────────────────────────────────────────────────────────

/// Whether `--as md1` can carry this descriptor AS WRITTEN (§5.3).
///
/// Two members of conjunct 7's set are `--as descriptor`-only: `/i/*` (a single
/// fixed chain index — §5.3(a)) and `<i;i+1>` with no trailing wildcard
/// (§5.3(a″)). A CHILDLESS key is representable: §5.3(a′) materialises it to
/// the device default, and the vector file's `md1-split/childless` row pins the
/// address that materialisation derives.
///
/// `remedy` is the sentence that follows the verdict, so the caller can apply
/// §5.3's window substitution — a build with no `--as descriptor` path must not
/// point the operator at a flag that refuses.
///
/// **The single-refusal form.** §6 says a descriptor mixing an (a)-shaped and
/// an (a″)-shaped key matches BOTH rows and both fire; this returns the FIRST
/// offender, because a §7 gate row names exactly one `refusal_row`. The
/// `--as md1` path, where both are observable, uses [`md1_refusals`].
pub fn md1_representable(d: &Parsed, remedy_a: &str, remedy_a2: &str) -> Result<(), Refusal> {
    match md1_refusals(d, remedy_a, remedy_a2).into_iter().next() {
        Some(r) => Err(r),
        None => Ok(()),
    }
}

/// EVERY key `--as md1` cannot carry, as `(slot, the path it uses)`.
///
/// §5.1's window refusal needs them all -- *"a mixed input repeats the key
/// clause per offender"* -- and so does §6's both-rows-fire rule.
pub fn md1_offenders(d: &Parsed) -> Vec<(usize, String)> {
    use Derivation::*;
    d.keys
        .iter()
        .enumerate()
        .filter(|(_, k)| {
            matches!(
                k.children.as_slice(),
                [Child { .. }, Wildcard { .. }] | [Range { .. }]
            )
        })
        .map(|(i, k)| (i, k.children.iter().map(|c| c.encode()).collect::<String>()))
        .collect()
}

/// EVERY §5.3 refusal this descriptor earns, in §6's own row order: the (a) row
/// then the (a″) row, each naming ALL of its offending keys.
///
/// §6 states the mixed case explicitly -- *"both fire, both are true, and both
/// name the same remedy -- no precedence is needed"* -- and a one-refusal API
/// cannot express it.
pub fn md1_refusals(d: &Parsed, remedy_a: &str, remedy_a2: &str) -> Vec<Refusal> {
    use Derivation::*;
    let slots = |f: fn(&[Derivation]) -> bool| -> Vec<usize> {
        (0..d.keys.len())
            .filter(|i| f(d.keys[*i].children.as_slice()))
            .collect()
    };
    let fixed = slots(|c| matches!(c, [Child { .. }, Wildcard { .. }]));
    let no_wildcard = slots(|c| matches!(c, [Range { .. }]));
    // Everything outside conjunct 7's closed set: unreachable after admission,
    // and md1 carries no shape conjunct 7 refuses, so the conservative answer is
    // the closed set's own row.
    let outside = slots(|c| {
        !matches!(
            c,
            [] | [Wildcard { hardened: false }]
                | [Range { .. }, Wildcard { hardened: false }]
                | [Child { .. }, Wildcard { .. }]
                | [Range { .. }]
        )
    });
    if let Some(i) = outside.first() {
        return vec![refusal::use_site_out_of_set(&d.keys[*i].children)];
    }
    let mut out = Vec::new();
    if !fixed.is_empty() {
        out.push(refusal::md1_fixed_index(&fixed, &d.keys, remedy_a));
    }
    if !no_wildcard.is_empty() {
        out.push(refusal::md1_no_wildcard(&no_wildcard, &d.keys, remedy_a2));
    }
    out
}

// ───────────────────────────────────────────────────────────────────────────
// §5.2 — the classification predicate
// ───────────────────────────────────────────────────────────────────────────

/// §5.2's classification predicate, stated once and implemented by both sides:
///
/// > A record is `ClassDescriptor` iff it parses under §4's cascade **and**
/// > matches §4.7's grammar — the seven forms; conjunct 1's md1-path widening
/// > does not apply here.
///
/// This is exactly what §7's `host_admits` column means. It is NOT "`me`'s
/// cascade parses it" (`me` parses `multi`, and `multi` is `host_admits=false`)
/// and NOT "some `--as` succeeds" (`--as md1` succeeds on `multi`).
pub fn host_admits(input: &str) -> bool {
    match super::cascade::cascade(&super::cascade::normalise(input)) {
        Ok(d) => admit(&d, Path::Descriptor).is_ok(),
        Err(_) => false,
    }
}

/// The value §7's `format` column carries for this input — see `mod.rs` on the
/// F-1 reading this implements.
pub fn format_of(input: &str) -> &'static str {
    match super::cascade::cascade(&super::cascade::normalise(input)) {
        Ok(d) => d.branch.format(),
        Err(_) => "none",
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Conjuncts 4 and 6, reached the only way they CAN be reached
// ───────────────────────────────────────────────────────────────────────────

/// **The close for IMPL-P1's F-1 — the round's only Important-class finding.**
///
/// Conjuncts 4 and 6 have no vector row that can red them, and that is
/// structural rather than an omission: every `Key` P1 could build came through
/// `cascade::parse_extended_key`, which refuses a non-admitted version first,
/// and through branch 1, which refuses an origin-less BlueWallet key first. So
/// deleting either conjunct left the whole 485-test suite green (measured, P1).
///
/// **P2.2 is the phase that changes the argument**, which is why the close
/// lands here and not in P3: `descriptor::md1` builds a `md_codec::Descriptor`
/// from a `Parsed`, so this crate now has a second place a descriptor is
/// assembled, and any future caller that builds a `Parsed` by another route
/// makes these two conjuncts the ONLY enforcement.
///
/// It is a UNIT test and deliberately not a vector row: no cascade-reachable
/// input can produce either state, so a row asserting one would be a lie about
/// what the two parsers do.
#[cfg(test)]
mod conjunct_reachability {
    use super::*;
    use crate::descriptor::cascade::{Branch, KeyVersion, Network};

    /// A `Key` built with NO parser in the way. Every field is public, which is
    /// what makes the bypass possible — and is exactly the risk F-1 names.
    fn key(version: KeyVersion, fingerprint: u32, origin: Vec<u32>) -> Key {
        Key {
            as_supplied: "xpub-under-test".to_string(),
            fingerprint,
            origin,
            origin_explicit: fingerprint != 0,
            children: vec![Derivation::Wildcard { hardened: false }],
            version,
            network: Network::Mainnet,
            parent_fingerprint: 0,
            chain_code: [7u8; 32],
            key_data: [2u8; 33],
        }
    }

    fn single(k: Key) -> Parsed {
        Parsed {
            branch: Branch::Bip380,
            title: None,
            script: Script::P2WPKH,
            multi: None,
            threshold: 0,
            keys: vec![k],
            promoted: false,
        }
    }

    #[test]
    fn conjunct_4_refuses_a_version_outside_the_admitted_five() {
        let h = |n: u32| crate::descriptor::cascade::HARDENED + n;
        // The control FIRST: the identical descriptor with an admitted version
        // is admitted, so a failure below is the version and nothing else.
        let ok = single(key(KeyVersion::Xpub, 0x4bba_a801, vec![h(84), h(0), h(0)]));
        assert!(
            admit(&ok, Path::Md1).is_ok(),
            "the control must be admitted"
        );

        for v in [
            KeyVersion::Ypub,
            KeyVersion::Upub,
            KeyVersion::Vpub,
            KeyVersion::UpubCap,
            KeyVersion::VpubCap,
        ] {
            let d = single(key(v, 0x4bba_a801, vec![h(84), h(0), h(0)]));
            let r = admit(&d, Path::Md1).expect_err("a non-admitted version must refuse");
            assert_eq!(
                r.row.slug(),
                "unsupported-key-version",
                "{v:?} refused, but by the wrong §6 row"
            );
        }
    }

    #[test]
    fn conjunct_6_refuses_a_fingerprint_with_no_origin() {
        // `Descriptor.encode` emits the `[…]` block iff the fingerprint is
        // non-zero, and `ParseKey` then requires a `/` at offset 8 — so this
        // state re-encodes to a string the DEVICE cannot read back.
        let d = single(key(KeyVersion::Xpub, 0x4bba_a801, Vec::new()));
        let r = admit(&d, Path::Md1).expect_err("a fingerprint with no origin must refuse");
        assert_eq!(r.row.slug(), "bluewallet-no-origin");

        // The all-zero fingerprint is the one case where a key legitimately
        // carries no origin block: "master unknown" is not a claim about
        // identity, and refusing it would reject files several coordinators
        // emit. Without this half, a conjunct 6 that simply required a non-empty
        // origin would pass the assertion above.
        let d = single(key(KeyVersion::Xpub, 0, Vec::new()));
        assert!(
            admit(&d, Path::Md1).is_ok(),
            "an all-zero fingerprint with no origin is a legal key"
        );
    }
}
