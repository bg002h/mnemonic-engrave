# Continuity — 2026-08-07

Supersedes `CONTINUITY_2026-08-06b.md` for everything about the **encrypted
payload delivery** feature. That doc's other content (the Y-axis-play
resolution, the engraving-settings feature, the `z` glyph, the 8-vs-4 mm/s
re-test) is still current and untouched — this feature was worked in parallel
and did not disturb any of it.

## 1. THE HEADLINE — spec AND plan both GREEN. Implementation is the next act.

Deliver constellation payloads to the SeedHammer II **over a wire, encrypted**,
decrypted on-device with a typed passphrase. **Both artifacts have now passed R0
and the gate is open. Nothing is implemented yet — that is the handoff.**

```
86c0445  spec v1  (5 R0 rounds)
7b76388  PBKDF2 rate measured on real silicon
86c0445  spec v2  (mixed public/encrypted, ms1-first, fixed hash)
00da6a8  spec v2 R0 GREEN
f0ab467  spec CLOSED — confirming pass, 9 rounds total
2ed2695  scripts/plan-build-gate.sh added
b946399  fold plan round 4
36227ad  round 5 report (1 Critical — the fold's own test was dead)
a3f49cf  fold round 5
519695b  PLAN A R0 GREEN — 6 rounds  ← BUILD FROM THIS
```

**Build from `IMPLEMENTATION_PLAN_encrypted_payload_hostA.md` at `519695b`.** It
is current with the spec, and every ```rust block in it compiles.

## 2. WHAT THE FEATURE IS

Host (`me seal`) encrypts a payload → emits a `data`-family UF2 → operator loads
it at `0x10E00000` with `picotool load` while the machine is in BOOTSEL → the app
reads it back through XIP, prompts, decrypts to RAM, engraves.

**Validated on the real machine 2026-08-06**: the blob lands byte-exact, the
firmware region's sha256 is unchanged, `picotool erase -r` reverses it. Secure
boot gates *booting*, not *writing* — the bootrom's UF2 abort-reason enumeration
has no signature-related reason.

Construction: **PBKDF2-HMAC-SHA256 + AES-256-GCM**, both already linked in the
firmware (GCM arrives free via `crypto/ecdsa`'s fips140 dependency), so zero
marginal flash. Passphrase is a **host-generated 12-word BIP-39 mnemonic** and
the CLI must never accept a user-supplied one.

A payload carries a **public section** (cleartext, authenticated via the AAD), an
**encrypted section**, or both. Passphrase is prompted only when something is
encrypted.

## 3. OPERATIONAL TRAPS — read before touching hardware

These are not in the spec and cost real time to rediscover. One is destructive.

- **⚠ `tinygo flash` targets whatever RP2350 is in BOOTSEL.** If the SeedHammer
  II is plugged in, it gets the image. **Always identify the board first**:
  ```
  udevadm info --query=property --name=/dev/sdX | grep ID_MODEL
      SHII    → the engraver.   STOP.
      RP2350  → the Pico 2.     safe.
  ```
  SH2 chipid `0x77c483b745abf55c`, Pico 2 chipid `0x66d3d60ff20abf2f`.
- **The Pico 2 is secure-boot enabled and does NOT trust your real key.** Its OTP
  holds `rehearsal-work/factory-key.pem` (slot 0) and `rehearsal-work/my-key.pem`
  (slot 1). `sha256` of the real `~/.sh2/sh2-boot-key.pem` pubkey is `846aa289…`
  and matches neither. **Sign Pico images with `rehearsal-work/my-key.pem`.**
- **This kernel has no `cdc_acm`** — not a module, not built in. No `/dev/ttyACM*`
  ever appears. Read TinyGo serial with
  `/tmp/…/scratchpad/cdcread.py` (libusb via ctypes; asserts DTR, reads bulk EP
  `0x83`). Set `argtypes` on **every** libusb call or ctypes truncates the 64-bit
  handle and segfaults.
- **`picotool` cannot reboot a running TinyGo app** (no reset interface), and
  `tinygo flash` fails because nothing mounts the MSD volume. Every flash cycle
  costs a **physical replug** holding BOOTSEL.
- **A one-shot print is lost.** TinyGo's CDC drops output when no host is
  attached. `cmd/kdfbench` prints in a loop for this reason.

## 4. THE MEASUREMENT — 9,715 iterations/sec, and why it mattered

`cmd/kdfbench` in the fork, run on the Pico 2. **Measured 9,715
PBKDF2-HMAC-SHA256 iterations/sec** (dkLen=32, 150 MHz, built with the firmware's
own flags). The estimate it replaced was 15,000 — **high by 1.54×**.

That estimate came from an `≈` on a *range* in a "responsiveness" section
(`SPEC_seedhammer_slip39_recovery.md:273`, repeated at `FOLLOWUPS.md:59`),
phrased in "SHA-256 blocks" while the derivation read it as iterations. It had
been cited twice until it looked like a fact. Nobody had timed it.

Default is now **300,000 iterations = 30.9 s**. The old 450,000 would have been
46 s. Residual: measured on an RP2350**A**; the SH2 is an RP2350**B**. Same core,
same clock, compute-bound — should transfer, but confirm during Plan B.

## 5. DO NOT RE-OPEN — settled by review, with the reasoning

Fable's verified-sound list, so the next session does not re-derive it:

- **The AAD construction is correct.** `AAD = [0, 52+pub_len)` covers `pub_len`
  (offset 44) *and* `ct_len` (offset 48), so the cleartext/ciphertext boundary is
  authenticated and cannot be moved.
- **No cross-payload splicing.** §8 forbids a user-supplied passphrase and §9
  generates a fresh one per seal, so no two payloads ever share one.
- **§7.2's nonce-uniqueness argument is sound** — fresh salt per encryption means
  one key per message, unbreakable procedurally.
- **PBKDF2 over scrypt/Argon2 is settled** with primary sources: neither fits its
  own standard's recommended memory here, and at ~256 KiB an RTX 4090's 72 MB L2
  holds 288 concurrent working sets, so memory-hardness is paid for and not
  received.
- **Truncated SHA-256 is the right primitive**; the only defect ever found in the
  hash was its *width*.

## 6. THE FOUR MISTAKES WORTH NOT REPEATING

Eighteen persisted rounds (11 spec, 7 plan — counted, not estimated).
**Zero findings ever landed in the cryptographic construction.**
Every Critical was in the reasoning around it:

1. **A test vector taken from a tool's display output.** Vector C used
   `mnemonic bundle`'s default `--group-size 5`, whose spaced records the device
   rejects outright (`codex32`'s `inputChar` has no mapping for `0x20`). The
   "positive" test could never have passed. **Always regenerate with
   `--group-size 0`; canonical lengths are 75/111/80/67/67/67.**
2. **A cost model assumed rather than derived.** The 64-bit hash was defended on
   "one child derivation per candidate". It is one to two SHA-256 compressions —
   the attacker grinds origin paths and record order, not keys. $60k–$250k, not
   infeasible. Now 128 bits.
3. **An invariance designed in and sold as a feature.** The hash was deliberately
   "independent of whether anything is encrypted at all", pinned as D ≡ E. That
   *was* the blindness a downgrade attack needs. Now domain-separated by a
   `sealed` byte, and D ≠ E is the detector.
4. **Claims about someone else's code, never run.** "The classifier prevents
   shipping seed material in the clear at all" is false — `ValidMD`/`ValidMK` are
   pure BCH verifiers that never decode, and the fork ships the checksum
   generator, so arbitrary bytes wrap into a record that classifies as public.

And **five tests that could not fail**, including the TDD RED step itself
reporting green in five of eight plan tasks (an undeclared `.rs` file is not
compiled). When a normative value changes, **grep every section that asserts it**,
not only the ones being rewritten — that is how §11.1 nearly deleted its own fix.

## 7. WHAT TO DO NEXT, IN ORDER

The R0 gate is **passed**. Steps 1 and 2 of the old list are done.

1. **Implement Plan A** — ONE agent, own worktree, TDD, nine tasks in order. The
   controller folds small review fixes inline rather than spawning more agents.
   The plan's Task 9 Step 4 expects **11** `seal_cli` tests; the seal lib is
   **55**. Both numbers are measured, not estimated.
2. **Whole-diff adversarial review** — mandatory, non-deferrable. R0 covered plan
   correctness; this catches implementation-introduced regressions TDD misses.
3. **Then Plan B** (firmware), which binds to the vectors Plan A emits and may
   never lead them (Rust-primary rule). **F-68 is owned by Plan B's plan review**
   — close the build gate's CLI blind spot before that review, not after.

### Process rules added on 2026-08-07 — these are now standing, repo-wide

- **`./scripts/plan-build-gate.sh` runs before any fold is committed.** A fold is
  authorship and re-earns the gate. Round 3 spent a whole opus round on five
  compile errors a *fold* introduced.
- **Persist the review verbatim in its own commit, then fold in a second**, with
  the gate output in the fold commit's message. Ordering is forced: persist acts
  on the reviewer's input and precedes the fold; the gate acts on the fold's
  output and follows it. `b946399` bundled all of it and is the counter-example.
- **The gate compiles `tests/seal_cli.rs` but cannot run it** (its binary has no
  `seal` subcommand). That gap cost a Critical in round 5: a test that compiled
  and could never pass. Until F-68 lands, CLI-test *assertions* need a real run.
- **Watch the stale-binary trap when mutation-testing.** Restoring with `mv` can
  leave an mtime older than the artifact, so cargo skips the rebuild and the
  "restored" run is still the mutant. `touch`, and confirm a `Compiling` line.
  It fooled round 4 and the controller once each.

## 8. OPEN ITEMS

- **§12 item 3** — MSD drag-and-drop untested (`/dev/sdc` is not writable without
  root and udisks will not mount it). `picotool load` is the documented path;
  keep drag-and-drop undocumented until someone tests it.
- **§12 residual** — confirm the PBKDF2 rate on the RP2350B during Plan B.
- **F-65** — SH2 boot-key backup. Answer: it already works as 24 BIP-39 words
  (256-bit scalar = exactly 24 words); the gap is only labelling; **do not** add
  a record kind for it.
- **F-66** — arbitrary plain text over the sealed path. Filed with the hazard:
  a naive raw-text kind reopens the `command: lock-boot` → `LockBoot()` → OTP
  path that R0 round 1's first Critical closed.

## 9. STANDING CONSTRAINTS (unchanged from 2026-08-06b)

- **Always `~/bin/sh/sh2-flash`, never picotool by hand, for the SH2.** Judge a
  boot only on MACHINE power — a laptop port gives a dark screen on firmware the
  bootrom ACCEPTED, because `Init()` wants a 20–28 V USB-PD contract.
- If it does not boot: **do NOT burn another OTP slot.** Slots 2 and 3 are the
  only spares.
- `gh` defaults to UPSTREAM. Every fork operation needs `--repo bg002h/seedhammer`.
- **Never a bare `go test ./... -update`.** Scope with `-run`, then check
  `git status`.
- Insert `FOLLOWUPS.md` entries **before** `## Resolved`. Stage paths explicitly.
- All Go work runs under `nix develop --command`. `nix` is NOT on the shell PATH
  — use `/nix/var/nix/profiles/default/bin/nix`.
