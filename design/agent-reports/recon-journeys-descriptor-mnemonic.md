# RECON — round-trip journeys that EXIST in `descriptor-mnemonic`

**Scope.** Read-only inventory of `/scratch/code/shibboleth/descriptor-mnemonic`
against `design/DRAFT_round_trip_journey_definition.md` (mnemonic-engrave repo),
§§1–8, per the operator's 2026-08-19 rulings in §8. Inventories what EXISTS;
does not propose what should exist. Fixed test seeds accepted as origins.
Decoder-reads-preview question and T4 staffing are out of scope (ruled
elsewhere). This repo implements the `md` (Mnemonic Descriptor) format only —
crates `md-codec` and `md-cli` (binary `md`).

## What I actually ran

`ls`/`find` to map `crates/md-cli/tests/`, `crates/md-codec/tests/`,
`design/`, `docs/`, `.github/workflows/`; `grep -rniE "round.?trip"` across
non-vendor, non-target source to enumerate candidate files; `grep -n` passes
inside `address_derivation.rs`, `wallet_policy.rs`, `bitcoind_differential.rs`,
`per_key_use_site_override.rs`, `template_roundtrip.rs`, `cmd_address.rs`,
`cmd_decode.rs`, `cmd_verify.rs`, `smoke.rs` (both crates), `cli_repair.rs`,
`cli_repair_dead_card_strict.rs` for `#[test]`, `fn `, `assert_eq!`,
`derive_address`, `encode_md1_string`/`decode_md1_string`,
`chunk::split`/`chunk::reassemble`, `compute_wallet_policy_id`. Read full text
of `common/mod.rs` (proptest generators), `wallet_policy.rs` (helpers +
journeys 500–650), `address_derivation.rs` (961–1147),
`bitcoind_differential.rs` (1–260, 700–868), `cmd_address.rs` (full),
`template_roundtrip.rs` (full), `cli_repair.rs` (1–300), `cmd_verify.rs`
(full), `docs/verify-reproducibility.md` (full — ruled out as a journey, it's
build reproducibility not codec round-trip). Ran, locally, to confirm pass and
capture exact commands: `cargo test -p md-codec --test wallet_policy
divergent_paths_wallet_policy_2of2_round_trip -- --exact`; `cargo test -p
md-codec --test address_derivation round_trip_then_derive_address -- --exact`;
`cargo test -p md-cli --test cmd_address
address_phrase_mode_round_trips_through_encode -- --exact`; `cargo test -p
md-cli --test template_roundtrip round_trip_each_manifest_entry -- --exact`;
`cargo test -p md-cli --test cli_repair repair_single_chunk_happy_path --
--exact` — all `ok`. Ran `gh run list --repo bg002h/descriptor-mnemonic
--workflow=bitcoind-differential.yml --limit 10` to check the `#[ignore]`-by-
default `bitcoind_address_differential` gate has actually executed (it has:
10 consecutive daily `success` runs, most recent 2026-08-19). Counted, not
estimated: `grep -c "Vector {" crates/md-codec/src/test_vectors.rs` → 16;
`sed -n '122,650p' bitcoind_differential.rs | grep -c "Shape {"` → 16 (cross-
checked against a `label:`-line count, also 16, and against `grep -c "Shape
{"` over the whole file, 17, resolved as 16-in-`corpus()` + 1 in the `Shape`
struct definition itself). Second-method check for cross-repo invocations:
first a targeted string grep (`mnemonic-engrave|me-cli|seedhammer|mnemonic-
toolkit`), then an exhaustive `Command::new(...)`/`StdCommand::new(...)`
enumeration across every test file — both agree: the only external binaries
any test shells out to are the in-repo `md` bin and the system `diff`/
(separately) a CI-provisioned `bitcoin-cli`, never another constellation
repo's tool.

---

## Journeys found (§7 schema)

### J1 — `divergent_paths_wallet_policy_2of2_round_trip`

| field | value |
|---|---|
| name | `divergent_paths_wallet_policy_2of2_round_trip` |
| kind | **ambiguous** — see finding F1 below; does not cleanly fit generative or custodial |
| tier | T1 (codec, in-process; no CLI subprocess) |
| origin artifact | An in-memory `Descriptor` (2-of-2 `wsh(multi)`, divergent per-key origin paths) with **synthetic** 65-byte "xpub" TLV entries built by `make_xpub(seed)` — chain code = the seed byte repeated 32×, pubkey = the secp256k1 generator point G, for every key. Structurally valid, **not derived from any BIP-39 seed or real entropy**. |
| ordered invocations (repo-by-repo, this repo only) | `roundtrip_via_string_or_chunks(&d)` → tries `encode_md1_string`, falls back to `chunk::split`/`chunk::reassemble` on `PayloadTooLongForSingleString` → `compute_wallet_policy_id` on both `d` and the round-tripped `d2` |
| structural assertion | `assert_eq!(d, d2)` — full `Descriptor` struct equality, decoded vs. original |
| functional assertion | `assert_eq!(id_1, id_2)` where `id_1/id_2 = compute_wallet_policy_id(...)` — **explicitly named**: `WalletPolicyId` (satisfies §4's naming requirement) |
| the ONE command | `cargo test -p md-codec --test wallet_policy divergent_paths_wallet_policy_2of2_round_trip -- --exact` (ran locally: `ok`) |
| stated non-coverage | None stated in the test itself — no non-coverage line is printed or documented near this test. **Finding (§6): no coverage statement.** |

**§7 finding F1 (kind).** `make_xpub`'s chain-code-from-a-repeated-byte / fixed-G-pubkey construction is neither "entropy or a seed" (generative) nor "an artifact already in hand" (custodial) — it's fabricated bytes that only need to pass `validate_xpub_bytes`. The assertion under test (WalletPolicyId stability) doesn't need real key material, so this isn't a defect in the test's *purpose*, but it means J1 cannot be read as proof that a **real** wallet's policy ID survives the round trip — only that the ID-computation function is a pure function of the (any) TLV bytes it's given. Contrast J2, which uses a real fixed BIP-39 seed.

### J2 — `round_trip_then_derive_address`

| field | value |
|---|---|
| name | `round_trip_then_derive_address` |
| kind | generative (real fixed BIP-39 test seed) — cleanly fits |
| tier | T1 (codec, in-process) |
| origin artifact | `ABANDON_MNEMONIC` (the public BIP-84/86/49/44 test-vector mnemonic, fixed — ruling §8.2 permits this), account xpub derived at `m/84'/0'/0'` via `rust-bitcoin` bip32 (a real, trusted derivation, not fabricated bytes) |
| ordered invocations | Direct `Descriptor` construction with the real xpub bytes → `encode_md1_string` (asserted to reject: `PayloadTooLongForSingleString`, an intentional negative check of the single-string cap) → `chunk::split` → `chunk::reassemble` → `derive_address(0, 0, Network::Bitcoin)` on both the pre-round-trip and post-round-trip descriptors |
| structural assertion | **MISSING.** No `assert_eq!` compares the decoded `Descriptor` to the original struct, nor the chunk bytes to anything independent. **Finding (§7): "an existing path that lacks any field is a finding" — this journey has no structural leg**, only two functional comparisons (`direct == after`, `after == golden`). A decode bug that altered an unrelated field (e.g. a fingerprint, a use-site path on a key not exercised by chain 0/index 0) would not be caught here. |
| functional assertion | `assert_eq!(direct, after)` then `assert_eq!(after, "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu")` — the published BIP-84 receive-address-0 vector. **Receive-only** — see F2 below. |
| the ONE command | `cargo test -p md-codec --test address_derivation round_trip_then_derive_address -- --exact` (ran locally: `ok`) |
| stated non-coverage | None stated. **Finding (§6).** |

**Finding F2 (change addresses).** J2 only exercises `derive_address(0, 0, ...)` — the receive chain, index 0. It never calls `derive_address(1, ...)` (change) in this test. §4 is explicit: *"Change addresses are not optional. Receive-only is the check that passes while a policy mismatch quietly loses money on the change chain."* No journey found anywhere in this repo combines an actual md1 wire round-trip (encode/decode or split/reassemble) with a change-address (chain 1) functional check in the same test. (`per_key_use_site_override.rs::divergent_suffix_change_chain_independent_golden` does check chain 1 with an independent hand-rolled golden, but constructs the `Descriptor` directly — no encode/decode step — so it isn't a round trip either; see "Related, non-qualifying" below.)

### J3 — `address_phrase_mode_round_trips_through_encode`

| field | value |
|---|---|
| name | `address_phrase_mode_round_trips_through_encode` (`crates/md-cli/tests/cmd_address.rs:138`) |
| kind | generative (real fixed BIP-39 test seed, same ABANDON mnemonic) |
| tier | **T2** (real `md` binary subprocesses via `assert_cmd`/`std::process::Command`, separate processes, real exit/stdout capture) |
| origin artifact | ABANDON mnemonic → account xpub at `m/84'/0'/0'` (real bip32 derivation) → CLI arg `--key @0=<xpub>` |
| ordered invocations | `md encode wpkh(@0/<0;1>/*) --key @0=<xpub> --force-chunked --group-size 0` (subprocess) → `md address <chunk(s)...>` (second subprocess) |
| structural assertion | **MISSING.** The test never runs `md decode` on the phrase(s) to confirm the template text round-trips; it only checks the `address` command's stdout. **Finding.** |
| functional assertion | `stdout.contains("bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu")` — published BIP-84 receive-0 address. Receive-only (same F2 gap, at the CLI tier too — confirmed by grep: `encode_template_with_key`, the only helper that actually produces an md1 phrase before calling `address`, is never used together with `--change` anywhere in `cmd_address.rs`; the two tests that DO pass `--change` — `address_mainnet_wpkh_first_change`, `address_change_and_chain_together_rejected` — go straight through `--template`/`--key` mode and never produce or consume an md1 phrase at all). |
| the ONE command | `cargo test -p md-cli --test cmd_address address_phrase_mode_round_trips_through_encode -- --exact` (ran locally: `ok`) |
| stated non-coverage | None. **Finding (§6).** |

### J4 — `round_trip_each_manifest_entry` (+ `reencode_round_trip_each_manifest_entry`)

| field | value |
|---|---|
| name | `round_trip_each_manifest_entry` (`crates/md-cli/tests/template_roundtrip.rs:36`) |
| kind | Neither — see finding F3. Origin is an abstract template string, not entropy/seed nor an artifact "in hand." |
| tier | T2 (real `md` binary subprocesses) |
| origin artifact | Each of 16 entries in `md_codec::test_vectors::MANIFEST` (`crates/md-codec/src/test_vectors.rs`) — counted via `grep -c "Vector {"` = 16, all with `keys: &[]` (no key material at all) |
| ordered invocations | `md encode <template> [--path <p>]` → `md decode <phrase>` |
| structural assertion | `assert_eq!(back, v.template)` — decoded text equals the original template string, byte-for-byte |
| functional assertion | **STRUCTURALLY IMPOSSIBLE, not merely missing.** Every manifest entry has `keys: &[]`; there is no xpub anywhere in the fixture, so no address/fingerprint/wallet-id could ever be computed from it. **Finding: this whole 16-entry manifest family (also exercised by `cmd_decode.rs::decode_round_trips_to_template`, `cmd_verify.rs`, and the codec-level `wallet_policy.rs` template-only smoke tests) can never carry a functional leg as currently constructed** — it is a pure wire/text-fidelity check, permanently one-legged by its own fixture design. |
| the ONE command | `cargo test -p md-cli --test template_roundtrip round_trip_each_manifest_entry -- --exact` (ran locally: `ok`) |
| stated non-coverage | None printed. **Finding (§6).** |

`reencode_round_trip_each_manifest_entry` (same file, line 207) is the companion: `encode → decode → re-encode`, asserting the re-encoded phrase equals the first — still structural-only, same manifest, same F3/F4.

**Finding F3 (kind).** A template string with `@N` placeholder keys is neither entropy (nothing is generated) nor an artifact a real operator would hold (no card exists with placeholder keys engraved on it — an operator's card always carries real key material once past the design stage). This whole family sits in a gap the two-kind taxonomy in §2 doesn't name.

### J5 — `cli_repair.rs` repair-journeys (custodial)

| field | value |
|---|---|
| name | `repair_single_chunk_happy_path`, `repair_multi_chunk_one_corrupted` (and siblings `repair_multi_chunk_all_valid_passthrough`, `repair_multi_chunk_atomic_failure_per_d28`, `repair_json_multi_chunk_envelope_shape`) |
| kind | custodial — origin is a corrupted card an operator would hold, closest clean fit to §2 of any journey found |
| tier | T2 (real `md` binary subprocesses) |
| origin artifact | `md encode --force-chunked --group-size 0 wpkh(@0/<0;1>/*)` (template-only, `keys: &[]` again) → one codex32 symbol flipped in-test via `corrupt_at` |
| ordered invocations | `md encode ...` (fixture) → hand corruption (in-test, not a CLI step) → `md repair <corrupted chunk(s)>` |
| structural assertion | `stdout.lines().any(\|line\| line == valid.as_str())` — the repaired chunk text equals the **pre-corruption** original, an independent source (produced by the earlier `encode` call, not by `repair` itself) — legitimate, non-circular |
| functional assertion | **MISSING — cannot exist**, same root cause as J4: the fixture template carries no keys, so there is no address/wallet-id to check post-repair. **Finding.** |
| the ONE command | `cargo test -p md-cli --test cli_repair repair_single_chunk_happy_path -- --exact` (ran locally: `ok`) |
| stated non-coverage | None. **Finding (§6).** |

`cli_repair_dead_card_strict.rs` is a **negative-only** control (asserts exit 2 / un-repairable) — not a journey under §1 ("ends in two independent equality assertions"); a rejection isn't an equality. Not counted as a journey, and not a finding — it's simply out of scope for this schema.

---

## Related artifacts that do NOT qualify as journeys

### `bitcoind_address_differential` — the strongest functional oracle in the repo, but never touches the wire format

`crates/md-codec/tests/bitcoind_differential.rs::bitcoind_address_differential`,
`#[ignore]`-by-default, wired by `.github/workflows/bitcoind-differential.yml`
(push/PR on the derive/render/canonicalize/encode source paths + daily
05:17 UTC cron). **Gate-execution check (§5 "every gate must have executed at
least once"): confirmed executed** — `gh run list` shows 10/10 recent runs
`success`, most recently 2026-08-19T05:46:53Z, ~30–44s each.

It cross-checks `Descriptor::derive_address` and the rendered descriptor's
checksum against a pinned, sha256-verified Bitcoin Core v27.0 `bitcoin-cli`
(an independent C++ implementation — the strongest oracle in the repo), for
16 corpus shapes × **both chains (0=receive, 1=change — this DOES cover
change)** × indices 0..=4, plus two independent hand-computed anti-vacuity
goldens (`WPKH_CHAIN0_IDX0_GOLDEN`, `DIVERGENT_WSH_MULTI_CHAIN0_IDX0_GOLDEN`)
that must be hit (`assert!(golden_asserted)` / `assert!(divergent_golden_asserted)`
at the end) so a broken bitcoind connection can't pass vacuously.

**Why it doesn't qualify as a journey.** Every corpus `Shape` is a directly-
constructed Rust `Descriptor` (same pattern as J1/J2's helpers). Confirmed by
grep: this file contains **zero** calls to `encode_md1_string`,
`decode_md1_string`, `chunk::split`, or `chunk::reassemble`. No md1 phrase is
ever produced or consumed. It proves `derive_address`/`to_miniscript_descriptor`
render fidelity against an external oracle, not that the **m-format wire
round trip** (the actual subject of this audit) is faithful. **This is a real
coverage gap worth naming precisely: no test anywhere in this repo combines
an actual md1 encode→decode (or split→reassemble) with the bitcoind external
oracle.** J1/J2 prove the wire round-trip is structurally/functionally sound
using in-codec or fabricated key material; `bitcoind_address_differential`
proves address derivation is oracle-correct using directly-constructed
descriptors. The two never compose in one test.

**Finding F4 (records are wrong — machine-counted, not described).** The
test's own doc comment (line ~711: *"the 10 R0-proven corpus shapes"*) and the
CI workflow's doc comment (`bitcoind-differential.yml` line ~4: *"10 R0-proven
corpus shapes × 2 chains × indices 0..=4 (100 address checks + 20 checksum
round-trips)"*) both say **10**. I counted `Shape {` literals inside `fn
corpus()` (lines 122–650) three independent ways — `grep -c "Shape {"` on the
sliced range (16), a `label:`-line count on the same range (16), and a full-
file `grep -c "Shape {"` (17, the extra one being the `struct Shape` decl
itself, not an instance) — all agreeing on **16, not 10**. At the stated
formula that makes the real totals **16 × 2 × 5 = 160 address checks** and
**16 × 2 = 32 checksum round-trips**, not the documented 100 / 20. The gate
itself is sound and has run green 10/10 times; only its own two written
descriptions of its size are stale.

### `proptest_roundtrip.rs` (P1–P5, P1(W)–P5(W))

T1, in-process, `cargo test -p md-codec --test proptest_roundtrip`. Origin is
a proptest-generated abstract `Descriptor` (via `descriptor_strategy()` /
`wire_descriptor_strategy()` in `common/mod.rs`) — not entropy, not an
artifact in hand; structurally exhaustive fuzzing of the codec bijection.
Structural-only (`prop_assert_eq!` on payload bytes / decoded structs /
chunk-reassembled structs); **no functional assertion anywhere in this file**
— none of the five properties ever calls `derive_address` or computes a
wallet id. Not counted as a journey (no functional leg, and its origin fits
neither §2 kind), but it is real and by far the widest structural coverage in
the repo (full wire-domain fuzzing, not just the 16-entry manifest).

### `smoke.rs` (md-cli) — not even a round trip

`crates/md-cli/tests/smoke.rs::encode_wpkh_default_phrase` pins
`md encode wpkh(@0/<0;1>/*) --group-size 0` stdout to a literal golden string.
There is no decode step at all — it's an encode-only canary, not a round
trip in either direction. Flagged so it isn't mistaken for a journey by name
similarity to the codec-level `smoke.rs`.

### `crates/md-codec/tests/smoke.rs::bip84_single_sig_round_trip`

T1, `encode_payload`/`decode_payload`, `assert_eq!(d, d2)` — structural only,
`TlvSection::new_empty()` (no keys), so no functional leg is possible. Same
F3/F4-shaped gap as J4/J5.

### `docs/verify-reproducibility.md`

Read in full. This is **build reproducibility** (bit-identical musl binary
across two build paths), not a codec round trip — no encode/decode of any
constellation string occurs anywhere in it. Ruled out as in-scope, noted only
because it was the one `docs/` hit for a "verify" search and a future pass
might otherwise mistake its "two-distinct-path" self-test language for a
round-trip journey.

---

## §5 anti-requirement checks (of the journeys/near-journeys above)

- **Reads an intermediate nothing writes:** not observed in any journey
  above. Every intermediate consumed (an md1 phrase, a chunk vector, an xpub)
  is produced earlier in the same test by a call whose output is captured in
  a variable, never read from a file path or fixture that lacks a writer in
  the same process. (I did not find an equivalent of mnemonic-engrave's
  `design/journeys/transcript_pathological.sh:18` pattern anywhere in this
  repo — there are no shell-script journeys here at all, only Rust `#[test]`
  functions; confirmed by the shell-script sweep in "what I actually ran.")
- **Asserts against a self-produced value with no independent source:** not
  found in J1–J5. J1/J2's structural legs compare against the pre-round-trip
  input (independent of the codec's decode output). J2/J3's functional legs
  compare against published BIP-84 vectors (external). J5's structural leg
  compares against the pre-corruption fixture (independent of `repair`'s
  output). `bitcoind_address_differential`'s functional leg is external by
  construction (a second implementation). No circular/self-blessing pattern
  observed in any test read for this recon.
- **A skipped step passes instead of failing:** `bitcoind_address_differential`
  is the one test in this repo built around an explicit skip path (`read_wiring()
  -> None` when the three env vars are unset) — and it is correctly designed
  per §5: unset vars → `eprintln!` + `return` (an actual pass, but this is the
  documented **local** default, not the CI path); set-but-broken → `panic!`
  on the `getblockchaininfo` check, never a silent pass. CI always sets the
  vars and starts the node, so the CI-observed 10/10 green runs are real runs,
  not skip-passes — confirmed by the workflow file (no `if:` guard suppresses
  the differential step) and by the ~30–44s run durations (consistent with
  actually starting bitcoind + running the loop, not an instant skip).
- **A gate that has never executed:** `bitcoind_address_differential` was the
  one candidate at risk of this (`#[ignore]`-by-default) and is **confirmed
  executed** (§ above). I found no other gated/ignored test in the journey
  set — J1–J5 all run under plain `cargo test` with no `#[ignore]`.
- **Empty output is not proof of absence:** applied to my own negative claims
  in this report. The "no cross-repo invocation" claim and the "MANIFEST has
  zero keyed entries" claim were both checked two independent ways (see "what
  I actually ran"), not from a single grep.

## The known blind spot (§8.3, restated as required)

This is a single-repo sweep. It cannot see whether a journey's origin
artifact (e.g., a real `md` card) is ever actually fed into
`mnemonic-engrave`'s NDEF/engraving path, or whether an `md`-encoded phrase
survives a hop into another constellation repo. Confirmed only: **nothing in
this repo's test suite invokes another constellation repo's tooling** — so if
such a round trip exists at all, it is homed elsewhere, not here.

## One-paragraph summary of what exists

Five journey-shaped tests were found (J1–J5), all in Rust `#[test]`
functions run via `cargo test`, split T1 (codec, in-process: J1, J2) / T2
(CLI subprocess: J3, J4, J5). **Not one of the five satisfies §4 in full** —
J1 has structural+functional but a non-seed origin; J2 has functional but no
structural leg and is receive-only; J3 (CLI) has functional but no structural
leg and is also receive-only; J4 and J5 have structural but can never grow a
functional leg because their shared 16-entry manifest fixture carries no key
material at all. The single most rigorous functional check in the repo
(`bitcoind_address_differential`, external-oracle, both chains, gate
confirmed green 10/10 runs) never touches the md1 wire format and so isn't a
journey by this document's definition — and its own doc comment and its
CI workflow's doc comment both understate its corpus size (10 claimed vs. 16
actual, machine-counted three ways), overstating neither risk nor safety but
simply wrong on the numbers. No journey anywhere in the repo combines a real
md1 wire round-trip with a change-address (chain 1) functional check in the
same test.
