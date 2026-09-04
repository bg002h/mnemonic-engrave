# Brief: hashlock-phrase brainstorm, R0 round 2 -- security software engineering lens (opus, single agent)

You are an expert security software engineer: secure CLI and firmware
implementation, input and output channel handling, fail-closed design,
misuse-resistant interfaces, and test suites that can actually fail. You are
reviewing a DESIGN RECORD, not code. READ-ONLY except for the one report you
write at the end.

## The one question

**Would you sign off on building `ms hashlock` and the SH2 phrase screen from
this record as written -- and where you would not, what is the concrete input,
sequence or environment under which the software as designed does the wrong
thing, leaks, fails open, or cannot be shown by its own tests to be right?**
An opinion without a construction is not a finding; put it under "Questions
for the operator".

## Scope

Read, in this order:

1. `/scratch/code/shibboleth/mnemonic-engrave/design/BRAINSTORM_hashlock_phrase.md`
   -- ALL sections. Section 2 = the operator's rulings (final; L19 says the
   spec waits for the operator). Section 3 = measured context. Section 4 =
   the agreed design (4.1-4.5) and the PRESENTED test plan (4.6, not yet
   ruled; the operator asked for your lens before ruling on it). Section 5 =
   defaults the operator may veto. Section 7 = the round-0 cryptography
   review's dispositions.
2. The round-0 report and its verification, so you do not repeat them:
   `design/agent-reports/hashlock-brainstorm-R0-r0-crypto-bitcoin-expert.md`,
   `design/agent-reports/hashlock-brainstorm-R0-r1-fold-verification.md`.
3. The shipped code the design extends, in mnemonic-secret
   (`/scratch/code/shibboleth/mnemonic-secret`, `7fc1e58`):
   `crates/ms-cli/src/argv_guard.rs`, `parse.rs`, `out.rs`, `advisory.rs`,
   `process_hardening.rs`, `mlock.rs`, `cmd/encode.rs`, `cmd/derive.rs`,
   `cmd/decode.rs`, `cmd/payload_lang.rs`; `crates/ms-codec/src/envelope.rs`,
   `payload.rs`, `shares.rs`; `.github/workflows/*.yml`; and the constellation
   rules the CLI surface obeys:
   `/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_constellation_cli_uniformity.md`
   sections 6a-6i.
4. The fork (`/scratch/code/shibboleth/seedhammer`, `70008da5`):
   `gui/composer_hash.go`, `gui/composer_sources.go` (the scrub defer and seed
   entry), `gui/passphrase_flow.go`, `gui/passphrase_keyboard.go`,
   `gui/unlock_kdf.go`, `passphrase/passphrase.go`, `codex32/mspayload.go`,
   `sysw/classify.go`, `sysw/record.go`.

NOT in scope: re-doing the cryptography review (the KDF construction, the
script facts, the guessing rates and the kind byte are settled by round 0 and
its dispositions); the operator's rulings; sections not yet designed (the
specs themselves); code style.

## Settled -- do not re-derive, do not re-litigate

- Rulings L1-L19. In particular: two methods, sha256 warns and never refuses
  (L12); fixed salt, no `--salt` (L13); id `hash` (L14); NO scrub discipline
  on the device for the phrase or X (L15) -- you may still REPORT a
  secret-handling consequence, but it is Minor at most.
- **Secret-handling defects are never Critical or Important** (operator
  ruling 2026-08-27): material on argv, in shell history, on a stream, in RAM
  longer than needed, unwiped strings. Record them as Minor, name the class.
  Still blocking: wrong results, fail-open, a refusal that does not refuse, a
  test that cannot fail, a gate that skips and prints ok, data loss.
- Every number in section 3 is measured; the round-0 report's computations
  were reproduced by the controller.

## Questions to answer -- construct the failing input or write "sound" with the reason

Q1 **Input channels.** `--hashlock-phrase` joins the argv guard's flag list;
   `--hashlock-phrase-stdin`, `--in FILE`, `--hex`/`--hex -`, a positional
   ms1 or `-`, `--random`. Read `argv_guard.rs` and `parse.rs` and construct:
   an invocation where the phrase reaches clap's error text or any stream
   before the guard runs; two channels claiming stdin at once; a file with a
   BOM, an interior CR, a trailing space, or CRLF-then-LF; `--in` pointed at
   a file of hex; a phrase supplied on argv WITH `--allow-argv-secret` and
   what the side channel does with it; an empty stdin; a phrase of 101
   characters via each channel. State the designed outcome for each and
   whether the record specifies it.

Q2 **Output channels.** stdout carries only `hash:<hex>`; the stderr card
   carries the preimage (ms1 + hex) and the method line; `--out FILE` carries
   the preimage ms1 0600 and overwrites; `--json` carries everything on
   stdout with the private-key-material advisory. Construct the pipeline or
   shell habit under which the preimage lands somewhere the operator did not
   intend (a `2>&1 | tee`, a `--json > file` at 0644, an `--out` onto a
   symlink, a non-tty stdout that is a log). Say which of those the
   constellation's existing `write_private`/`fd::mode_of` machinery already
   handles (read `out.rs` and the io-lib crate it calls) and which the record
   must state.

Q3 **Fail-closed audit.** For every refusal the record names (empty phrase,
   non-ASCII, over 100, 64-hex, `--hex` length, wrong ms1 kind, wrong-length
   `0x03`, two sources, `derive`/`verify` on the kind), say whether the
   design places it BEFORE any irreversible or observable effect (a partial
   card, a written `--out`, a set path hash on the device) and whether a
   panic path remains (the four `unreachable!` sites; `getrandom`; the
   PBKDF2 call; `try_from`). On the device: Back during the 10-second
   derivation; power loss mid-derivation; choosing the wrong method and
   confirming; the confirm modal's `first8..last8` as the only comparison
   surface -- construct the substitution an attacker who can edit the payload
   file would make and say whether 64 visible bits stop it, and whether that
   attacker is in the threat model at all (the host is trusted; say so if
   that settles it).

Q4 **The test plan (4.6).** Judge it as a security test suite. Name what is
   missing or cannot fail: negative-content assertions (the phrase and the
   preimage never appear in stderr, stdout or any error under any refusal --
   is there a test per refusal?); the argv-guard test proving the guard ran
   BEFORE clap (see the constellation lesson "a guard downstream of the
   parser has lost"); the reproduction test's skip-is-failure requirement and
   whether ms's CI matrix (read the workflows: stable/beta/MSRV, FreeBSD
   compile gate, musl checks) actually provides `python3` and `openssl` on
   every row that runs it; fuzz or property coverage of `dispatch_payload`
   for `0x03`; a test that a `0x03` string on ms-cli 0.17.x refuses rather
   than panics (a downgrade test); mutation adequacy for the device's
   touch-harness test; the negative control in the capture arm. For each
   gap: the exact test, what it asserts, and the mutation that would slip
   through without it.

Q5 **Misuse and the operator's mental model.** Two methods with a
   `hardened` default; "try each method that shipped with the version named
   on this card"; the `hash:` record vs the `sha256=` operand (same hex, two
   spellings); `--hex` vs the phrase slot (I-6's refusal); a preimage plate
   that reads `ms10hash...` beside seed plates; the device's `Type a hashlock
   phrase` row beside `Type 64 hex`. Construct the operator sequence that
   yields a funded policy whose preimage the operator does not hold, or a
   backup the operator believes is complete and is not, that the record's
   copy does not prevent.

Q6 **Supply chain and build.** New dependencies for ms-codec (`pbkdf2`,
   `sha2`, `hmac`): versions already trusted by `me`
   (`/scratch/code/shibboleth/mnemonic-engrave/crates/me-cli/Cargo.toml`),
   MSRV 1.85 compatibility, `no_std`/feature flags, the exact-pin policy the
   ms repo applies to `codex32`; the fork's `golang.org/x/crypto/pbkdf2`
   already vendored. Anything a reproducible-build gate (ms's `man-release.yml`
   musl job, F-324 history in `design/FOLLOWUPS.md` of mnemonic-secret) would
   choke on.

Q7 **The device leg as software.** The phrase screen reuses the passphrase
   keyboard: does anything the BIP-39 passphrase flow does (pass-proof
   trigger `PASSPROOF!`, the settings gear, the 100-cap message "too long for
   one plate") leak into the hashlock screen wrongly? The countdown screen is
   shared with unlock: any state it carries? The payload class for a `0x03`
   ms1: does `sysw.Classify`'s existing `isStrictMs1` path (checksum-valid,
   HRP `ms`, at most 90 characters) classify a preimage as ClassCodex32Secret
   BEFORE the new class can, and is the design's "refuse by name at seed
   entry" reachable given `DecodeMS1`'s current default arm? Trace the
   actual call order.

Q8 **Anything else** a security software engineer would refuse to sign off
   on, within the record's scope.

## Severity

- **Critical**: wrong result, fail-open, a refusal that does not refuse, a
  test that cannot fail, a gate that skips and prints ok, data loss, a
  funded-but-unspendable or anyone-can-spend outcome the copy does not
  prevent.
- **Important**: a real defect, missing case or unsound assumption to resolve
  before the spec is written.
- **Minor / Nit**: recorded, not gating (all secret-handling findings land
  here by ruling).

## Output -- write this file as your FINAL action, then return a 5-line summary plus the path

`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/hashlock-brainstorm-R0-r2-security-software-expert.md`

Structure, exactly:

1. Header: date, model, the record's commit
   (`git -C /scratch/code/shibboleth/mnemonic-engrave log -1 --format=%h -- design/BRAINSTORM_hashlock_phrase.md`),
   every file read with the repo commit.
2. One counts line: `C:<n> I:<n> M:<n> N:<n>`.
3. Findings numbered C-1.., I-1.., M-1.., N-1.., each with: **claim** (one
   sentence); **evidence** (file:line, or the record's section + line);
   **construction** (concrete input/sequence -> concrete wrong outcome);
   **remedy** (non-authoritative; the controller and operator decide).
4. "Confirmed sound" -- one line per Q1-Q8 item cleared, with reason and
   citation.
5. "Test plan additions" -- a table: test | asserts | the mutation it catches
   | stage (H1/H2/H4).
6. "Questions for the operator" -- judgement calls with no construction.
7. "Sources consulted".

Rules: write nothing anywhere else; do not edit any file; never read any
`*.jsonl`; do not implement or run cargo builds (reading and small `python3`
or shell checks are fine -- paste the command and its output when you do).
