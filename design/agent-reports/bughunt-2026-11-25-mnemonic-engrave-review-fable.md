# Final review (Fable): mnemonic-engrave PR 68, bug hunt fixes E1-E5 and X7

Reviewed: branch `claude/project-thread-utm1aa` at `86c0ab53` against merge-base `80dfaeb`.
Read-only: nothing committed, pushed or commented. (One `git fetch` was run in
`/home/user/mnemonic-refugium` to read its `origin/master` pin.)

## Verdict

**0 Critical / 0 Important / 4 Minor / 4 Nit.** The code is ready. Two things should be done
before the PR is marked ready: correct the PR body (M-1), and settle the refugium pin merge
order (M-2). Three of the four Minors are already filed as follow-ups; M-1 is new.

## Per-finding check

| ID | Fixed? | How it was checked |
|---|---|---|
| E1 | Yes | `ClassifyError::UnknownHrp` holds `Option<&'static str>`, so neither `Display` nor `Debug` can carry input. Ran the built `me` on 12 shapes (converter, `bundle`, `bundle --preview`, `seal --in`, `seal --plaintext`, `sysw pack`; mangled ms1, mnemonic, seed note, hex key, WIF, `md1 <mnemonic>`, `mk1<body>`): no secret text in any output. `mr` is still named, from the static list only. |
| E2 | Yes (spec only) | The amended §8.4 rule matches BIP-68 and Core's `CalculateSequenceLocks` as I know them: unsigned version >= 2, bit 31 clear, bit 22 selects 512 s units, 16-bit mask, zero value is no lock, coinbase excluded. No leftover text claims a relative lock is a script read. No wire, vector or generator change. Code is in mnemonic-transaction (#32). |
| E3 | Yes, for what the xpub can show | `origin_agrees` (refusal.rs:1072) refuses under `--as md1` only. Ran: child mismatch, depth too long, depth too short, unsorted `multi` (own remedy), mismatch plus fixed index (both rows print); all exit 3. `--as descriptor` still packs. It is mint-side only: `md1_refusals` is called from `as_flag::md1_follower` and `gate::carriage`, never from a decode path. See M-4 for the residue. |
| E4 | Yes (wording, as ruled) | The sealing line names the unconfirmed records in all three branches; the sealing decision is untouched. `mdmk_unconfirmed_quiet` keeps the chunk-set-id warning to one print. |
| E5 | Yes | Pin `9b094e0b` is on mnemonic-transaction `origin/main` and contains `95c39a3`. The test re-cuts a real corpus vector 38/36/37/37/37/37 and requires no confirmation. |
| X7 | Yes | Pin `c8cde234` is the tip of descriptor-mnemonic `origin/main`. Miniscript patch `ff4732e5` is equal in engrave, refugium and descriptor-mnemonic at that rev. |

**Existing cards and wire format.** Between the old and new md-codec revs, `decode.rs` changes
only in its test module; `derive.rs` only moves the chain pre-flight into a shared function
that `derive_address` already ran. `me` has no caller of `to_miniscript_descriptor`, so the new
chain refusal there is not reached. mt-codec's change is a decode refusal for sets no
conforming encoder writes. No encoder output changes.

## Tests run

`cargo clean -p mnemonic-engrave`, then `CARGO_INCREMENTAL=0 cargo test --locked -j 2 -p mnemonic-engrave`
(no `--release`). Logs: `/tmp/claude-0/-home-user/efb9fedd-ab0a-5019-871b-00d31aeaaeb6/scratchpad/fable-me-tests.log`
(lib) and `/tmp/claude-0/-home-user/efb9fedd-ab0a-5019-871b-00d31aeaaeb6/scratchpad/fable-me-tests2.log` (integration).

| Target | Result |
|---|---|
| `--lib` | 350 passed, 1 failed, 1 ignored |
| `bh1125` | 6 passed |
| `descriptor_refusals` | 45 passed |
| `descriptor_as` | 30 passed |
| `descriptor_seam` | 19 passed |
| `bughunt` | 14 passed |
| `cli` | 37 passed |
| `record_corpus` | 6 passed |
| `seal_cli` | 18 passed |
| `sysw_cli` | 107 passed, 1 failed |

Both failures are environmental and match the PR body's list:
- `seal::v2_tests::v2_titles_refusals` fails at `v2_tests.rs:857`, the 200 ms "before any KDF"
  timing assert. The message assert this PR changed (lines 851-856) passes before it.
- `pack_does_not_print_a_digest_for_a_write_that_failed` (`sysw_cli.rs:2919`) needs a write to
  `/dev/null` to fail; this session runs as root, where it succeeds.

`cargo fmt --check` is clean. Clippy and the full workspace were not run.

## Findings

### M-1 (Minor, new): the PR body is out of date and has three wrong details

- **Where:** PR 68 body (no file under `scratchpad/prbodies/`; read from GitHub).
- **What:**
  1. It says reviews r1 and r2 are saved. r3 exists
     (`design/agent-reports/bughunt-2026-11-25-mnemonic-engrave-review-r3.md`) and its
     follow-ups (`bh1125-seam-lost-canonical-depth-witness`, `bh1125-seam-readme-sha-label`,
     `bh1125-seam-synthetic-fingerprints`) are not in "Left unfixed".
  2. The findings table grades E3 "Medium (funds)" and E4 "Medium (secret)". The bug hunt
     report grades both Low.
  3. "descriptor_seam 30, descriptor_as 19" is swapped: measured `descriptor_seam` 19,
     `descriptor_as` 30.
- **Failure scenario:** Brian reads the body as the record of what was reviewed and what is
  left; it omits a round and the one follow-up that weakens a cross-language pin.
- **Fix:** add r3 and its three follow-ups, correct the two grades, swap the two counts.

### M-2 (Minor, already filed as `bh1125-x7-refugium-pin-order`): the pin-equality rule does not hold on master yet

- **Where:** `crates/me-cli/Cargo.toml:39-42` ("this rev EQUALS mnemonic-refugium's workspace pin").
- **What:** mnemonic-refugium `origin/master` (`25051a06`) still pins md-codec `c8bb97a5`.
  The re-pin to `c8cde234` exists only as uncommitted changes in the local refugium checkout.
- **Failure scenario:** if PR 68 merges first, the comment is false and anything linking both
  (the Refugium app) holds two md-codec versions until refugium lands.
- **Fix:** merge refugium's re-pin first or in the same sitting. The PR body already says so;
  treat it as a merge condition, not a later follow-up.

### M-3 (Minor, already filed as `bh1125-changelog`): no CHANGELOG entry

- **Where:** `crates/me-cli/CHANGELOG.md`, `[Unreleased]`.
- **What:** the section already lists earlier bug-hunt fixes, but nothing for this PR: a
  changed refusal message (E1), a new refusal that turns a previous exit 0 into exit 3 (E3), a
  reworded sealing line (E4) and two pin moves.
- **Fix:** add the entries in this PR; it is a few lines and E3 is a behaviour change users
  will meet.

### M-4 (Minor, already filed as `bh1125-e3-originless-implied-path`): E3's harm is still reachable through an origin-less key

- **Where:** `crates/me-cli/src/descriptor/cascade.rs` (implied path), `md1.rs:307`.
- **What:** reproduced. `wpkh(<E3_XPUB>/<0;1>/*)` with no `[...]` block packs under `--as md1`,
  exit 0, with the script's implied `84h/0h/0h` written as the origin and no notice. Depth 3
  and child `0h` agree, so E3 cannot object, but nothing says the key was made at purpose 84.
  A BIP-44 or BIP-49 account key inside `wpkh()` gets a false origin on steel.
- **Failure scenario:** the same as E3's: a recoverer derives at the engraved origin, gets a
  different key and sees an empty wallet.
- **Why Minor, not Important:** it predates this PR, the xpub's bytes cannot reveal it, and
  the implied path is the spec's stated behaviour. It is the one residue I would put in front
  of Brian, since he ruled on E3 to close this harm.
- **Fix:** as filed: refuse origin-less keys under `--as md1`, or print the implied origin as
  implied in the identification block.

### N-1 (Nit): the `--as`-omitted choice block still offers `--as md1` for a mismatched wallet

- **Where:** `descriptor/gate.rs:233-241` (`carriage`), `identify.rs:53-68`.
- **What:** with no `--as`, a mismatched key prints a `wallet-id:` and the two-option menu;
  choosing `--as md1` then refuses. Fails closed. Overlaps the filed `bh1125-e3-identify-wallet-id`.

### N-2 (Nit): a depth-0 key's child number is not checked

- **Where:** `descriptor/refusal.rs:1072-1075`.
- **What:** `origin_agrees` accepts depth 0 with an empty origin whatever the child number
  and parent fingerprint bytes are. A malformed depth-0 xpub with a non-zero child number is
  rebuilt by md1 with zero. No conforming wallet exports one.
- **Fix:** add `k.depth != 0 || k.child_number == 0` if wanted.

### N-3 (Nit): stale doc comments beside the new code

- `descriptor/admit.rs:381-386`: the doc on `md1_refusals` still says it returns "the (a) row
  then the (a'') row"; it now returns a third.
- `main.rs:2191`: "`decide_sealing` itself is BYTE-UNCHANGED" (filed as `bh1125-e4-stale-comments`).

### N-4 (Nit): `bh1125_e3_...controls_pack` uses a "good" origin that is not known to be true

- **Where:** `tests/descriptor_refusals.rs:350`.
- **What:** the control asserts `[73c5da0a/84h/0h/0h]<E3_XPUB>` packs. That shows depth and
  child agree, which is all the rule checks. The comment could say so, so nobody reads it as
  proof the key is the purpose-84 account.

## What I did not check

- BIP-68 and BIP-112 wording was checked from memory, not against the BIP text.
- The seedhammer fork's copy of the vectors and the 16-key row's derivation (settled in r3).
- The device's handling of unconfirmed records (E4 is wording only).
- Clippy, the full workspace run and the rustdoc build.
