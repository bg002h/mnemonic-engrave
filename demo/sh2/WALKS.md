# Verified mission walks — measured in the emulator, not guessed

Every tap path below was DRIVEN against `cmd/emu` served locally and the screen
text read back after each step. Nothing here is inferred from source.

## Harness facts (these cost an hour to learn; write them down)

- **Selecting a row does nothing.** The idiom is *tap the row, then tap CONFIRM*.
  A walk that only taps rows sees the screen never change and looks broken.
- Device coordinates, 480x320: `BACK [453,70]`, `PAGE [453,160]`,
  `CONFIRM [453,249]`, carousel-next `[455,160]`.
- **Read row positions from `shTargets()`; do NOT compute them.** The walk
  scripts' `rowY(i,n) = 160-(n-1)*12+i*24` is for ChoiceScreens (pitch 24). The
  composer's LISTS use pitch 29 (measured: cy 121,150,179,208,237). Computing
  cost me a run: `rowY(4,5)=208` landed on row 3 ("Change the script") and the
  walk carried on happily into the wrong flow. `shTargets()[i].cx/cy` is the
  device's own geometry and always right.
- **Verify where every row tap landed.** A wrong row picks something else and
  does not error. Post-conditions are what make a coordinate safe.
- Hold-to-confirm = `shPress(CONFIRM)`, 1300 ms, `shRelease(CONFIRM)`.
- `shScreen()` returns whitespace-stripped text; compare squashed.

## Carousel order (12 entries, measured) — and it goes BOTH ways

| i | entry | | i | entry |
|---|---|---|---|---|
| 0 | Backup Wallet | | 6 | Engrave Multisig |
| 1 | BIP-39 Password | | 7 | **Wallet Policy** |
| 2 | Engrave Text | | 8 | Engrave Transaction |
| 3 | Account Xpub | | 9 | Load Payload |
| 4 | Engrave Bundle | | 10 | BIP-85 Child Seed |
| 5 | Engrave Single-Sig | | 11 | **Sealed Payload** |

**Forward is `[455,160]`, BACKWARD is `[25,160]`** (measured by probing; the
left-top and left-bottom positions do nothing). The carousel wraps both ways,
so take the short way round:

- **Sealed Payload: 1 tap BACKWARD** from the start screen, not 11 forward.
- **Wallet Policy: 5 taps backward**, versus 7 forward.

Entry 11 is the conditional `unlockPayload`. I first reported it ABSENT — wrong:
my enumeration loop ran exactly 11 times and stopped one short. The negative
inherited the scope of the loop, not the device.

**Wallet Policy is index 7.** `Engrave Multisig` (index 6) is a DIFFERENT flow —
it asks "Supply or build a policy?" and is not the composer. Easy to land on by
counting wrong; match the title instead.

## THE TRAP: "Done" is on page 2 of the spend-paths screen

The spend-paths list shows `Path 1 / Path 2 / Path 3 / Add a spend path /
Change the script` and **nothing that ends the policy**. `CONFIRM` there edits
the selected path; hold does the same. The **middle button pages**, and page 2
holds a single row: `Done`.

A first-timer will not find this. The guide MUST say it. Flagged as a possible
firmware UX follow-up — page 1 gives no hint page 2 exists.

## Mission: decaying multisig (taproot) — 19 taps from a cold load

| # | taps | action |
|---|---|---|
| 1 | 2 | Boot offer "A systemwide payload is present. Load it?" -> select **SKIP**, CONFIRM |
| 2 | 5 | Carousel **backward** (`[25,160]`) to **Wallet Policy** |
| 3 | 1 | CONFIRM to enter |
| 4 | 2 | **Build a new policy** (row 1 of 2; row 0 is "Scan cards") |
| 5 | 2 | **Taproot (tr)** (row 0 of 4: tr / wsh / sh-wsh / sh) |
| 6 | 2 | **decaying-multisig** (row 6 of 7) |
| 7 | 1 | **Page** to reveal "Done" |
| 8 | 2 | **Done** (row 0 of 1) |

The preset arrives PRE-FILLED — `slots: 4`, `Path 1: 2-of-2 + 13140 blocks`,
`Path 2: 1 key + 26280 blocks`, `Path 3: 1 key + block 1000000`. No numbers need
typing, which is why this is 19 taps and not the digit-pad slog it looked like.

Ends on: `Template-ID: 75e45aca4e4d15ad9cd6eb43432b0692`, the mk1 stub, and the
literal `mk encode --xpub <xpub> --origin-fingerprint <fp> --origin-path <path>
--policy-id-stub 75e45aca` each cosigner must run. That command is the payoff.

## Mission: zen-hodl (taproot, 1 key, older=32768) — 42 taps

Reached via **"Build my own paths"** (preset row 0), not a preset. Steps 1-5 as
above, then Taproot, then:

| taps | action |
|---|---|
| 2 | **Build my own paths** (row 0 of 7) -> empty list, `slots: 0` |
| 2 | **Add a spend path** |
| 2 | **Keys** (at "What can spend on this path?"; the other row is "A hash, no keys") |
| 2 | **1** (at "Path 1: how many keys?" — 1..5) |
| 2 | **1** (at "Threshold: how many must sign?") |
| 2 | open **Path 1** |
| 2 | **Timelock** (row 1 of 4: Keys / Timelock / Hashlock / Remove path) |
| 2 | **After a wait** (row 1 of 3: None / After a wait / After a date or height) |
| 2 | **Blocks** (row 0 of 2: Blocks / Days) |
| 5 | type **32768** on the digit pad |
| 1 | CONFIRM -> "32768 blocks (about 227.6 days)" |
| 1 | CONFIRM -> back in the path editor, now "Path 1: 1 key + 32768 blocks" |
| 1 | **BACK** -> the spend-paths list, path PRESERVED |
| 2 | **Done** |

The digit pad caps at **"1 to 65535 blocks"** and the device renders the wait in
human terms ("about 227.6 days") — both good things to point at.

### TWO TRAPS on this walk, both cost me a run

1. **Do not tap CONFIRM blindly to get back to the list.** A loop of CONFIRMs
   from the lock-summary screen left the list at `slots: 0` — the path was gone.
   **BACK is the correct and safe way out of the path editor**, and it preserves
   the path (verified).
2. **The lock-summary screen has ZERO tap targets** (`shTargets().length === 0`).
   A driver that waits for a row there hangs.

### Digit pad coordinates (machine-checked in the fork)

Pitch 34. Rows `123` / `456` / `789` at x0=206, y=152/198/244; `0` alone at
(240,290) — shifted right because `gui.NewKeyboard` appends a backspace key.
`gui/composer_digitpad_geometry_test.go` pins these against the walk's own
numbers, so they are not free-floating constants.

## THE BEST BEAT: device and CLI agree on the Template-ID

Zen-hodl built by TAPPING the emulated device (Go firmware) and the same policy
composed by the Rust CLI produce a byte-identical 128-bit identity:

```
device screen:  Template-ID: 73c33a5dea17b45376a6246995df0cbb

$ md compose --wrapper tr --path '1of1,older=32768'
tr(50929b...ac0,and_v(v:pk(@0/48'/0'/0'/3'/<0;1>/*),older(32768)))
$ md encode "<that>" --json --group-size 0     # -> md1yqfdsssjuqqcr5e55uqqqgqqq6qqvf4d7h59r2
$ md inspect md1yqfdsssjuqqcr5e55uqqqgqqq6qqvf4d7h59r2
wallet-descriptor-template-id: 73c33a5dea17b45376a6246995df0cbb   <-- SAME
wallet-policy-id:              891b6c372c99c033c98bc80604deb6f0
```

Two independent implementations, one identity. For a self-custody audience this
is the reassurance that matters, and it takes ~15 seconds live.

## Verified tap costs

| mission | wrapper | taps | ends on |
|---|---|---|---|
| decaying-multisig | tr | **19** | Template-ID 75e45aca… |
| hashlock-gated | wsh | **~20** | Template-ID b309d061… |
| zen-hodl (custom) | tr | **42** | Template-ID 73c33a5d… |

Order the guide cheapest-first; zen-hodl is the "if you have time" one.

## Preset picker rows (n=7)

| row | preset |
|---|---|
| 0 | **Build my own paths** (this is how zen-hodl is reached) |
| 1 | plain-multisig |
| 2 | simple-timelocked-inheritance |
| 3 | kofn-recovery |
| 4 | tiered-recovery |
| 5 | **hashlock-gated** (key + hashlock phrase) |
| 6 | **decaying-multisig** |

Steps 1-5 are shared by every mission; only step 6 differs. Swap the wrapper at
step 5 (row 1 = wsh) for the wsh variants.

---

# Shamir / threshold backup — what exists, measured 2026-09-16

## Seeds (ms1): YES, k-of-n works today, host-side

```
ms split --in seed.txt -k 3 -n 5 --group-size 0   # 5 codex32 shares
ms combine --in any-three.txt                     # exact phrase back
ms combine --in any-two.txt                       # error: not enough shares: have 2, need 3
```

Verified on a throwaway 12-word phrase: shares 1+3+5 recombined byte-for-byte;
2 shares refused. codex32 (BIP-93) IS Shamir over GF(32). The fork also carries
a full SLIP-39 implementation (`slip39/{share,combine,gf256,feistel}.go`).

**The DEVICE does not recombine.** `codex32.Interpolate` and
`codex32.ConsistentShares` are called only by `codex32/polish_test.go` and
`cmd/biptool` — no GUI call site. Splitting and combining are CLI operations.
Do not promise share recovery on the device.

## Policies (md1): NO Shamir, BY DESIGN

`descriptor-mnemonic/design/POLICY_BACKUP.md`, decision 2026-04-26 — md1 reuses
codex32's alphabet, BCH polynomials and ECC but NOT the format, because
codex32's "header semantics are hard-coded for BIP 32 seeds (Shamir threshold,
share index)". md1's header has no threshold and no share index. A policy is
PUBLIC; there is no secret for a threshold to protect.

md1 chunking is **n-of-n** (every chunk required), for SIZE, not secrecy.

## What md1 has instead: BCH repair — the better demo

Damaged 4 characters in chunk 0 of a real keyed `wsh(sortedmulti(2,...))`:

```
md decode --in damaged.md1
  -> md: codec error: codex32 decode error: BCH checksum verification failed
md repair --in damaged.md1          # exit 5 = REPAIR_APPLIED
  -> # md1 chunk 0: 4 corrections at position 17: 'q' -> 'y', position 32: 'p' -> '8',
  ->   position 47: 'z' -> '2', position 62: 'r' -> 'd'
```

Corrected chunk was byte-identical to the original and the set decoded back to
the real descriptor. Framing for the demo: **md1 spends its redundancy healing
the plate you HAVE, rather than letting you lose one entirely.**

## Open feature idea (NOT for this weekend)

k-of-n across md1 policy plates — lose a whole plate, still recover. Legitimate
and useful, but it is new header semantics + wire format + Rust/Go parity +
test vectors: risk-set work needing a full spec->plan->gate cycle. File it, do
not improvise it.

## Mission: sealed-payload unlock — 60 taps, WALKED

| taps | action |
|---|---|
| 2 | boot offer -> **SKIP**, confirm |
| 1 | <b>previous</b> `[25,160]` once -> **Sealed Payload** (it is the last entry) |
| 1 | confirm -> the passphrase notice |
| 1 | confirm -> the BIP-39 keyboard, "Word 1 of 12" |
| ~47 | 12 words: **3-4 letters each** plus one confirm per word |

**The keyboard autocompletes and shows a live match count**, which is what makes
this tractable: `m` -> "105 matches", `mo` -> "21 matches", `mos` -> **MOSQUITO,
1 match**. Measured letters per word for this passphrase: 3,3,3,4,4,4,3,4,4,4,3,3
= 42 letters + 12 confirms.

BIP-39 word keyboard coordinates (machine-checked in the fork, and a DIFFERENT
keyboard from the passphrase one in `walk_hashlock_phrase.js`): pitch 34, rows
`qwertyuiop` x0=87 y=198, `asdfghjkl` x0=104 y=244, `zxcvbnm` x0=138 y=290.

Opens straight onto the first plate: **"SECRET seed material / Cut this plate /
Skip / ms1 1/3"**.

The passphrase screen's own copy is worth quoting on the day: *"These words are
the payload's passphrase. They are NOT a seed and no wallet is derived from
them."*

### A claim I could NOT verify, and corrected

`cmd/emu/sealed_test_payload.go`'s provenance comment says Vector F is "3
codex32 shares, 6 mk1 xpub cards, 6 md1 wallet-policy cards". Stepping the
plates shows `ms1 1/3, 2/3, 3/3` then `mk1 1/3` — so the **3 shares check out**,
but the counters are per-plate and do not obviously support "6 and 6". The
demo page now says "three codex32 seed shares, then cosigner key cards" and
states no number it has not measured. Resolve properly before quoting 6/6.
- **plain-multisig**: same shape as decaying-multisig, preset row 1. Expected
  ~19 taps but not separately measured.

## Other beats found while walking

- **Payload Digest screen** (boot -> LOAD): "Compare this against
  `me sysw show <file>` on the host: 55adb8006ec6a06694f36a0e900ac8d5". A second
  independent cross-check, same shape as the Template-ID one.
- The load flow then shows **"A SECRET is stored unencrypted in flash"**, a
  Keep/Unload choice, and a census: "Loaded. It holds: 1 BIP-39 mnemonic,
  1 free text, 1 passphrase."
