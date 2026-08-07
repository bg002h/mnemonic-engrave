# Whole-diff adversarial execution review — Plan A (encrypted payload, host `me seal`)

- **Date:** 2026-08-07
- **Reviewer:** independent opus agent, read-only, no authorship of the code under review
- **Under review:** `git diff master..HEAD` on `feat/encrypted-payload-hostA`, 11 commits, HEAD `4938bf0`
- **Gate:** post-implementation review required by `CLAUDE.md` phase (4) — R0 covered plan
  correctness; this pass covers implementation-introduced defects TDD misses.

Stated to the reviewer as **already machine-verified**, so budget went elsewhere: 169 tests
green across 9 binaries, clippy `-D warnings` clean, `cargo +1.85.0 test --locked` green on
MSRV, rustfmt clean, all pinned vectors matched first run, and the `4938bf0` UF2
self-referential-constant Critical already found and fixed.

Stated as **settled by prior review, not to be re-opened**: the AAD construction, no
cross-payload splicing, §7.2 nonce uniqueness, PBKDF2 over scrypt/Argon2, truncated
SHA-256 at 128 bits, and the known `--seal-secret` spec divergence.

---

## VERBATIM REPORT

Tree restored clean (`git status --short` empty, HEAD `4938bf0`, 169 tests green on a real recompile).

---

## Findings

```
[Important] `me seal`'s §6.6 `sealed` byte has ZERO test coverage — no CLI test
            exercises a mixed payload, and forcing the flag to `false` survives
            the whole suite
File:       crates/me-cli/src/main.rs:373
Claim:      The shipped line `pubhash::public_data_hash(&refs, sealed.passphrase.is_some())`
            is correct. But `me seal` invoked with BOTH `--plaintext` records and an
            encrypted payload — the only shape where this argument can be wrong — is
            not exercised by any test in the diff. A one-token mutation prints the
            UNSEALED digest under a "SEALED" banner and nothing notices.
Proof:      Mechanical scan of tests/seal_cli.rs for tests passing both a positional
            payload and --plaintext:
              seals_and_prints_the_passphrase_to_stderr_only   MIXED=False
              public_only_payload_prints_no_passphrase         MIXED=False
              refuses_a_secret_in_the_public_section           MIXED=False
              printed_hash_matches_me_hash_...whitespace       MIXED=False
              refuses_a_record_carrying_a_cr                   MIXED=False
              (11/11 tests: MIXED=False)

            Mutant applied (file copy, restored, touched, `Compiling mnemonic-engrave`
            confirmed on both builds):
              -    let h = pubhash::public_data_hash(&refs, sealed.passphrase.is_some());
              +    let h = pubhash::public_data_hash(&refs, false);
            cargo test -p mnemonic-engrave:
              116 / 1 / 30 / 1 / 3 / 1 / 6 / 11 / 0 — all "ok. 0 failed"  → 169 passed
            SURVIVED.

            Behaviour of the mutant on vector D's exact records
            (`me seal <ms1> --seal-secret --plaintext mk1 mk2 md1 md1b md1c`):
              public data hash (5 records, SEALED):
                  70f3 e35a acf7 47db c40f 8376 91aa 61e0     <-- spec vector E (UNSEALED)
            After restore, same invocation:
              public data hash (5 records, SEALED):
                  a26e d22b b747 dfd0 2367 06ad 14c1 9679     <-- spec vector D, correct
Impact:     §6.6 is explicit that the `sealed` byte is "what makes the downgrade
            visible" and that this hash is the ONLY integrity control an unsealed
            payload has. Under the mutant the operator writes down `70f3 e35a …`
            for a sealed blob. Two consequences, both silent: (1) every honest
            comparison mismatches, which §6.6 says "teaches the operator that
            mismatches are normal" — disarming the control; (2) an attacker who
            strips the ciphertext produces a payload whose device-displayed hash is
            exactly the value the operator recorded, so the strip becomes invisible.
            The next step after this gate is hardware, where the recorded value
            becomes a permanent artefact. Fix is a mixed-payload CLI test asserting
            the printed line equals `me hash --sealed <the same records>`.
```

```
[Minor]     `seals_and_prints_the_passphrase_to_stderr_only` never asserts stdout
            is empty; duplicating the passphrase to stdout survives the suite
File:       crates/me-cli/tests/seal_cli.rs:31-51 (assertion gap); crates/me-cli/src/main.rs:392
Claim:      The test's name asserts "stderr only" but it only proves the words are ON
            stderr. §9 requires "stderr only, never to a file". Moving the print is
            caught; ADDING a stdout copy is not.
Proof:      Mutant: `eprintln!("    {}", &**p);` + `println!("    {}", &**p);`
            cargo test -p mnemonic-engrave → 169 passed, 0 failed. SURVIVED.
Impact:     Code is correct today (no `println!` in `run_seal_cli`). A regression
            putting the 12 words on stdout lands them in any `me seal … > log`,
            pipeline capture or CI artefact. One-line fix:
            `.stdout(predicate::str::is_empty())`.
```

```
[Minor]     The `--seal-secret` opt-in guard covers `ms1` only, not a raw BIP-39
            mnemonic — which is the same seed material
File:       crates/me-cli/src/main.rs:330
Claim:      `if !seal_secret && secret.iter().any(|r| matches!(classify(r), Ok(Format::Ms)))`.
            `classify` on a 24-word mnemonic returns `Err(NoSeparator)` (no `1`
            separator), so the guard never fires, while `record_or_mnemonic`
            (seal/mod.rs:234) explicitly ADMITS the mnemonic. The flag's own doc says
            "Required to encrypt an ms1 (a seed). Sealing a seed must never be
            accidental." NOTE: this is the guard's coverage, not the flag's
            existence — the flag itself is the already-filed plan divergence and I am
            not re-opening it. The plan (line 2288) specifies the ms1-only form, so
            the code is plan-conformant.
Proof:      me seal "bacon bacon … bacon" (×24) --out /tmp/…/a.uf2   [no --seal-secret]
              me: wrote 512 bytes to /tmp/tmp.eQ4K1wXnZt/a.uf2
              passphrase — write this down …
                  harvest hedgehog inquiry pudding check mail snow select boil soda enrich boil
              exit=0
            versus `me seal <ms1> --out …` which exits 3 demanding the flag.
Impact:     An operator who pastes a BIP-39 seed on argv gets an offline-attackable
            ciphertext of it in flash without the deliberate confirmation the flag
            exists to force. Inconsistent, not unsound — the resulting blob is
            correct and §10.2.1's allow-list admits a mnemonic in the encrypted
            section.
```

```
[Minor]     md-codec 0.40 → 0.42 (929 changed source lines across bch/decode/
            validate/canonicalize) is bundled into the Task-1 commit and is not
            required by Plan A
File:       crates/me-cli/Cargo.toml:22, commit 84c4591
Claim:      Every md-codec API `seal` uses already exists in 0.40 — verified:
            `reassemble`, `decode_md1_string`, `pub struct ChunkHeader` (chunk.rs:22),
            `pub chunk_set_id` (chunk.rs:26), `ChunkHeader::read` (chunk.rs:68, same
            signature), `pub mod bitstream` (lib.rs:18). So the bump is an unrelated
            dependency upgrade folded into a feature commit whose message mentions
            "md-codec 0.42" only in passing and never states that it performs the bump.
Proof:      `diff -r md-codec-0.40.0/src md-codec-0.42.0/src | grep -c "^[<>]"` → 929,
            across 11 files including bch.rs, decode.rs, validate.rs, canonicalize.rs.
            The visible behavioural change is a TIGHTENING (new
            `Error::EmptyOriginOverride`, `validate_no_empty_origin_overrides`), i.e.
            the safe direction, and golden/cross_lang are green.
Impact:     No demonstrated defect. It moves the acceptance surface under the
            pre-existing converter/bundle paths inside a commit labelled "52-byte wire
            header", which is the bundling the standard workflow forbids ("unrelated
            process or tooling changes go in a third [commit]") and costs a future
            reviewer the diff that matters.
```

```
[Minor]     Vector G's normative §6.6 hash, and the derived keys/tags for C, D, F, G,
            are spec-normative cross-implementation values pinned by no test
File:       crates/me-cli/src/seal/pubhash.rs:69 (only D and E are pinned);
            crates/me-cli/src/seal/mod.rs (vectors pin blob sha256 only)
Claim:      SPEC §11.4 publishes a §6.6 hash for G (`be11 7b56 9cc4 cd6e b47d 32b6
            fd32 ccb8`) and per-vector derived keys and tags for C/D/F/G. The Go port
            (Plan B) must reproduce them. Nothing in the Rust suite asserts any of them.
Proof:      I resolved all of them against the real implementation and against an
            INDEPENDENT Python reimplementation (hashlib PBKDF2 + cryptography AESGCM):
              me hash --sealed <G's 12 public records>
                → be11 7b56 9cc4 cd6e b47d 32b6 fd32 ccb8   (matches spec)
              Python cross-check of A,B,C,D,E,F,G — len, sha256, derived key, tag,
              52-byte header hex, §6.6 hash:  MISMATCHES: 0
Impact:     None today — every value is correct, and I have now machine-verified the
            whole §11.4 table end to end. But those values are the Go port's contract
            and a Rust-side regression in the key/tag would only surface as a
            cross-language failure in Plan B. Adding G's hash literal to
            `matches_the_pinned_literals` is one line.
```

```
[Nit]       `WireError::TooLarge` / the REGION_LEN check is unreachable and untested
File:       crates/me-cli/src/seal/wire.rs:161-165
Claim:      With MAX_SECTION_LEN = 8191 checked first, the maximum `total` is
            52 + 8191 + 8191 + 16 = 16450 < 65536, so the branch can never be taken
            and no test constructs a case that reaches it.
Proof:      `rejects_out_of_range_lengths` uses 8192 / 0xFFFF_FFF0 / u32::MAX, all of
            which return PubLen/CtLen at wire.rs:119-124 before line 163.
Impact:     None — the code comment states it is deliberate defense-in-depth against a
            future implementation that drops the section caps. Recording it so nobody
            later "simplifies" it away believing a test covers it.
```

```
[Nit]       The printed record count is unasserted, and §11.4's stability pin is absent
File:       crates/me-cli/src/main.rs:376-377; crates/me-cli/src/seal/pubhash.rs tests
Claim:      (a) §9 requires the printed line to match what the device displays "hash,
            record count and sealed/unsealed"; `printed_hash_matches_me_hash_…` anchors
            on `starts_with("public data hash (")` and reads only line i+1, so
            `public.len()` is unchecked. (b) §11.4 requires "seal D twice with
            different salts and assert the hash is unchanged" — no such test exists.
Proof:      Read of the test at seal_cli.rs:182-187 (position → lines[i+1]).
Impact:     Negligible. (b) is structurally satisfied because `public_data_hash` takes
            no salt parameter, so a salt dependency is unrepresentable.
```

## Two implementer-flagged candidates — verified, both refuted as escapes

Both were named in my brief as unverified; I ran the mutants rather than taking them on faith.

- **`aad = header only`** — the concern that `flipping_a_public_section_byte_fails_the_tag` builds its own AAD is *factually correct*, but the mutant is killed. Applied (drop `aad.extend_from_slice(pb)`, re-add `pb` to the blob so the wire shape is unchanged): `FAILED. 113 passed; 3 failed` — `seal::tests::vector_d_mixed`, `seal::tests::vector_g_multisig_public_section_spans_four_cards`, `seal::tests::every_encrypted_vector_round_trips`. §6.1a is defended by the pinned mixed-shape blob sha256s. This matches the mutation report's own Finding 2; it is a mis-attribution in a table, not a gap.
- **`seal_cli::refuses_a_secret_in_the_public_section` weak** — also correct as written, and also not an escape. Removing `check_public`'s `is_secret()` branch: `FAILED. 115 passed; 1 failed` — `seal::tests::refuses_a_secret_in_the_public_section` (the lib test, which matches the specific `SealError::SecretInPublic(_)` variant) kills it. Matches the report's Finding 3.

## What I checked and found clean

- **All seven canonical vectors reproduced from scratch in Python** (different language, different crypto stack): blob length, sha256, derived key, GCM tag, 52-byte header hex, and §6.6 hash for A–G. **0 mismatches.** The spec's §11.4 tables are internally consistent and the Rust output is byte-exact against them.
- Header field offsets, both shapes, and every §6.2 bound match the spec table line by line, including the `u64` widening at wire.rs:162 and the 8191-not-8192 ceiling.
- §6.6 construction matches spec line 734 exactly (`LABEL ‖ 0x00 ‖ sealed ‖ count(u8) ‖ input`).
- Refusal paths: `--out` to a directory → exit 2, empty stdout, no passphrase; `--out` to a nonexistent dir → exit 2, empty stdout, no passphrase. No path prints the passphrase before the write succeeds; `write_private` is only reached after `seal()` returns Ok, so a refused seal cannot truncate an existing file. Duplicate md1 chunks in the public set are refused (`got 4 chunks, expected 3`). Whitespace/case cannot bypass the ms1 opt-in guard — `classify` trims and lowercases the HRP identically to `validate_record`.
- The Task-8 self-referential-constant fix (`4938bf0`) is real, and I found no other instance of that shape: every other UF2 field, both §6.6 literals, both KDF keys, and all seven blob digests are pinned to hex literals, not to the constant under test.

`VERDICT: 0 Critical, 1 Important, 4 Minor, 2 Nit`

---

## Controller's independent reproduction of the Important

Not taken on the reviewer's word. Re-run on this tree before folding:

```
$ python3 <scan tests/seal_cli.rs for tests passing BOTH a positional payload and --plaintext>
  11/11 tests: MIXED=False

$ sed -i 's/public_data_hash(&refs, sealed.passphrase.is_some())/public_data_hash(&refs, false)/' \
      crates/me-cli/src/main.rs        # substitution asserted by grep
$ cargo test -p mnemonic-engrave
  PASSED=169  FAILED=0                 # SURVIVED — confirmed
```

Disposition is recorded in the fold commit that follows this one.
