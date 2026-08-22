# "Our reasonably complex wallet" — the named fixture

> ## The hashlocks are DOUBLE-hashed, and that is load-bearing
>
> Miniscript's `sha256(H)` fragment compiles to
> `OP_SIZE <32> OP_EQUALVERIFY OP_SHA256 <H> OP_EQUAL` — read off the **compiled
> leaf script**, not inferred: `OP_SIZE OP_PUSHBYTES_1 20 OP_EQUALVERIFY
> OP_SHA256 …`, where `0x20` is 32. **The witness preimage must be exactly 32
> bytes**, and that is in the script, so it is consensus-enforced.
>
> The three passphrases are 40, 38 and 34 bytes. So the wallet commits to the
> passphrase hashed **twice**:
>
>     witness preimage = sha256(phrase)          -- 32 bytes, in preimage-N.hex
>     policy literal   = sha256(sha256(phrase))  -- what the descriptor commits to
>
> A recoverer remembers the phrase and hashes it **once** to get the preimage.
>
> **This is a fix, applied 2026-08-22.** Until then the policies committed to
> `sha256(phrase)` directly, so tiers 1, 2 and 4 could never satisfy `OP_SIZE`
> and were unspendable by anyone — three of four tiers, silently. Verified both
> ways: the probe at `design/measurements/` now finalizes a real transaction on
> every tier using these exact preimage files, and asserts each
> `preimage-N.hex` really is `sha256(preimage-N.txt)`.

> ## ⚠ THE PREIMAGES ARE STILL PUBLIC — DO NOT REUSE THEM
>
> All three preimages are committed in plaintext in this repository. Every tier
> now also requires a signature (see below), so the wallet is **no longer bearer
> access** — but a fixture is the thing people copy. If you build a real wallet
> from this shape: **generate your own preimages, never reuse these**, and treat
> them as exactly as sensitive as the seeds.

Named by the operator 2026-08-21. This is the standing reference wallet for
arbitrary-miniscript work across the constellation: complex enough to exercise
the things that actually break, small enough to read.

A four-tier degrading vault:

| tier | condition |
| --- | --- |
| 1 | `@0` **and** `@1` **and** `@2` **and** `sha256(H1)` — at any time |
| 2 | `@3` **and** `@4` **and** `sha256(H2)` — after 32768 blocks (relative) |
| 3 | `@5` alone — after absolute height 1173520 |
| 4 | `@6` **and** `sha256(H3)` — after absolute height 1383520 |

…where each `H` is `sha256(sha256(passphrase))`, per the banner above.

What it exercises, in one wallet: two `sha256` hashlocks, **both** timelock
flavours (`older` relative, `after` absolute), thresholds at 3/2/1/1, and seven
cosigners at one shared path.

## Tier 4 is keyed, and that is a change (2026-08-22)

Tier 4 was `after(1383520) AND sha256(H3)` with **no key at all**. The operator
ruled it out: *"keyless path is not reasonable."* It is now
`after(1383520) AND sha256(H3) AND pk(@6)`, which added a seventh seed.

Two things follow, both measured:

1. **Stock `rust-miniscript` now accepts both forms.** Before the change, the
   `tr` form was refused outright — *"All spend paths must require a
   signature"*, from `Miniscript::sanity_check()`'s `requires_sig` test. That is
   the same `NeedsSignature()` check that closes Bitcoin Core and Nunchuk. The
   `wsh` form was accepted even when keyless, because `Descriptor::from_str`
   only runs `sanity_check` for `Tr` (there is a `FIXME` in
   `miniscript-13.1.0/src/descriptor/mod.rs:1053` calling this preserved 12.x
   behaviour) — so the two forms disagreed about their own validity.
2. **`--experimental` is no longer required.** The previous version of this file
   said both forms need it. They do not: `md encode` succeeds on all three
   policies with no such flag. Verified by running it.

## Two wrappings, and wsh is NOT a wrapper swap

`tr.policy` and `wsh.policy` express the same four tiers. **They are different
wallets** — different scripts, different ids, different addresses, and different
keys. Measured with `md 0.13.0`, not typed:

| | tr | wsh |
| --- | --- | --- |
| policy chars | 616 | 560 |
| keyless md1 chunks | 4 | 4 |
| keyless + `--fingerprint` | 5 | 5 |
| keyed md1 chunks | 16 | 16 |
| template-id | `a00772edbdbb41fb4acb450672c5e5cb` | `6c635eac0f5a772d80c2eb7a43872bc8` |
| policy-id | `fa568be08b48847595bf536db6a1f74d` | `f095e31101e2c77139d77c98c5d6d9f6` |
| first receive | `bc1p8rrz3ts8u4dm2fu7ax3hlwywy3esads3dz2ykrwrwvcjrcqz5q3s6n0vcl` | `bc1qvm27h3dgyr0zr3htd0y8pqer7kvzzyeg60zn674dlxvvausg5vjqa0hjzp` |
| `md verify` keyless | OK (exit 0) | OK (exit 0) |

**Three changes are required, none cosmetic:**

1. **`multi_a` → `multi`.** `multi_a` is BIP-342 tapscript only —
   `wsh(…multi_a…)` is refused outright: *"Multi a(CHECKSIGADD) only allowed
   post tapscript"*. Verified, not assumed.
2. **The taptree flattens into one script.** `tr` carries four *separate* leaves
   `{A,{B,{C,D}}}`; `wsh` has exactly one script, so the tiers become an
   explicit disjunction — `or_i(A,or_i(B,or_i(C,D)))`.
3. **The keys change, because the path's script type changes.** bg002h is
   `m/270028'/coin'/account'/script'` with **`0'` = tr and `1'` = wsh**
   (`ms-cli/src/cmd/derive.rs:149`). So the wsh form derives at
   `m/270028'/0'/0'/1'` from the *same seven seeds* — same masters, same
   fingerprints, different account xpubs. `ms derive --template bg002h-wsh`.

**`wsh-shared-tr-keys.policy`** is the variant that reuses the tr keys at
`…/0'`. It is kept because it is the obvious thing to write and it *works* —
`md verify` passes at exit 0 — so it needs to be here with a reason not to use
it: it reuses one key across two script contexts, which is what the level-4
script field exists to prevent. Its policy-id is
`e5546bae3aaf2f88dda91f362b4b7a8d` and its first receive is
`bc1qyx7xwkk47aqdmvwqtxyf6acl489llsl5pdxlspx9f3y7cau59yjqca0eg8`. Note the
**template-id is the same either way** (`6c635eac…`) — that id is key-stable by
design, so it will not distinguish these two.

That second point has a consequence worth stating: in `tr` a spend reveals only
the leaf used, while in `wsh` **every spend reveals all four tiers**, including
the other two hashlocks' digests and both deadlines.

### Which form is "bigger" depends on what you are storing

Measured on the pathological sibling (`design/measurements/`), and the inversion
holds here too: the `tr` **PSBT** is larger than the `wsh` one, because it
carries the whole taptree as metadata; the `tr` **signed transaction** is
smaller, because a spend reveals one leaf and a control block instead of the
entire `or_i` script.

## The tr form has no key-path spend

Its internal key is `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0`
— the constellation's NUMS pin, script-path only. Checked against the repo, not
against BIP-341's text: it is `md`'s `--unspendable-key` default and the
`nums_taproot` test vector's internal key
(`md-codec/src/test_vectors.rs:232`, `md-cli/README.md:73`). Anyone wanting the
stronger claim should diff it against the BIP directly.

## Keys, and the seating trap

Seven keys, one seed each (entropy `0x…01` through `0x…07`), all at one shared
path — `m/270028'/0'/0'/0'` for tr, `…/1'` for wsh.

**All seven share one origin, so a keyless template that declares no
fingerprints is unseatable** — every card matches every slot and the device
refuses (`errSeatSlotContested`). Pass one `--fingerprint @i=HEX` per slot; it
costs one extra chunk and changes nothing else. See **F-227**, and
`SeedHammer-II-hashlock-vault-journey.pdf` for the three-arm device proof.

Seeds live in `../../journeys/inputs-rcw/seeds/`. The journey's committed xpubs
are at **accounts 8 (tr) and 9 (wsh)** — see `derive-rcw-keys.sh`, which is the
only thing that writes them. The **account-0** xpubs this file's table uses are
derivable from the same seeds with `ms derive --account 0`; keys `@0`–`@5` are
byte-identical to `../../journeys/inputs-hashvault/keys/`, and `@6` is new:

| | fingerprint | account-0 xpub |
| --- | --- | --- |
| `@6` tr | `26bd1e33` | `xpub6FFLRoki7AXPyHrBCHVkdrukPsmaKPCLuG78mAfw5He6XMQa6DFX818PF5op3psGenAyUdL6V6RxbAv8Kj5fJaPM68uJvzPCk6df5YW3VY7` |
| `@6` wsh | `26bd1e33` | `xpub6FFLRoki7AXPzGPhhsdcQuGpVURQD6oZEEH5CN8k1s7WbQywoANgXzksvXxKUb82sMEdBTr4Nm7TFLyegt18W3VFJVXhF2SGTdPZ7BweUhA` |

## Divergence from the hashlock-vault journey — NOT yet reconciled

The `tr` form used to be byte-identical to
`../../journeys/inputs-hashvault/wallet-policy-hashvault.txt`. **It no longer
is.** That journey's wallet still has the keyless tier 4 and only six keys, so
its transcript, PDF and committed artifacts describe a different wallet from
this fixture. Reconciling it is a separate piece of work: either key its tier 4
too and re-run the journey, or state plainly that the hashlock-vault journey
pins the historical keyless shape on purpose.

## Provenance

Every number above was produced by running `md 0.13.0` and `ms`, not copied from
a doc. The chunk counts come from `md encode --force-chunked --json`; the ids
from `md inspect`; the addresses from `md address --template … --key …`; the
verify results from `md verify`, whose exit status is quoted. The policies
themselves are the input to `../../journeys/derive-rcw-keys.sh`, which
substitutes the account path and asserts it found **seven** slots before writing.

## Changes of 2026-08-22, in one place

Two operator rulings landed together, and between them they moved every id and
address in this file:

1. **Tier 4 is keyed** — *"keyless path is not reasonable."* Added `@6`, a
   seventh seed.
2. **The hashlocks are double-hashed** — the preimage fix in the banner, which
   made tiers 1, 2 and 4 spendable for the first time.

Both are in `../../journeys/derive-rcw-keys.sh`, which asserts seven slots and
refuses to write a preimage that is not 32 bytes or whose double-hash is not a
literal in `tr.policy`. Still open: reconciling the hashlock-vault journey,
above.
