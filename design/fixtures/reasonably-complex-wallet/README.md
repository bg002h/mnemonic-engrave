# "Our reasonably complex wallet" — the named fixture

Named by the operator 2026-08-21. This is the standing reference wallet for
arbitrary-miniscript work across the constellation: complex enough to exercise
the things that actually break, small enough to read.

A four-tier degrading vault:

| tier | condition |
| --- | --- |
| 1 | `@0` **and** `@1` **and** `@2` **and** `sha256(H1)` — at any time |
| 2 | `@3` **and** `@4` **and** `sha256(H2)` — after 32768 blocks (relative) |
| 3 | `@5` alone — after absolute height 1173520 |
| 4 | `sha256(H3)` alone — after absolute height 1383520 |

What it exercises, in one wallet: two `sha256` hashlocks, **both** timelock
flavours (`older` relative, `after` absolute), thresholds at 3/2/1, a spend path
with **no key at all** (tier 4), and six cosigners at one shared path.

## Two wrappings, and wsh is NOT a wrapper swap

`tr.policy` and `wsh.policy` express the same four tiers. **They are different
wallets** — different scripts, different ids, different addresses, and different
keys. Measured, not typed:

| | tr | wsh |
| --- | --- | --- |
| policy chars | 575 | 519 |
| keyless md1 chunks | 4 | 4 |
| keyless + `--fingerprint` | 5 | 5 |
| keyed md1 chunks | 15 | 15 |
| template-id | `68a1a888385797337ce5debc90fcfb1e` | `daee67be4eacf85e8b832ae64fc06566` |
| policy-id | `a0b128ceaef3155a40af6f8e88765ecb` | `9c74e0d2e96dd80c605b5fea19d551a9` |
| first receive | `bc1puvyd9zxz6uvz0y0ehq7r5qz6h30txl4mgr8fxl3dqjp6xzpsy0qsgpgyny` | `bc1qyd7k9t5y0pxsg558y7mypgekdf0y25awnkw9tlvtec59c4wu5eeqsagcgq` |
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
   `m/270028'/0'/0'/1'` from the *same six seeds* — same masters, same
   fingerprints, different account xpubs. `ms derive --template bg002h-wsh`.

**`wsh-shared-tr-keys.policy`** is the variant that reuses the tr keys at
`…/0'`. It is kept because it is the obvious thing to write and it *works* —
`md verify` passes — so it needs to be here with a reason not to use it: it
reuses one key across two script contexts, which is what the level-4 script
field exists to prevent. Its policy-id is `b422a03491f457d36840e5b974a08855`.
Note the **template-id is the same either way** (`daee67be…`) — that id is
key-stable by design, so it will not distinguish these two.

That second point has a consequence worth stating: in `tr` a spend reveals only
the leaf used, while in `wsh` **every spend reveals all four tiers**, including
the other two hashlocks' digests and both deadlines.

## Both need `--experimental`

Tier 4 needs no key, so `rust-miniscript`'s `sanity_check()` refuses both forms
by default — `requires_sig` is a safety policy, not a language rule. `md encode
--experimental` relaxes **only** that rule; malleability, resource limits,
repeated keys and timelock mixing are still enforced, and it warns on every use.
The warning is the important part here: **whoever learns H3's preimage can spend
tier 4 alone, so if that preimage is engraved, the plate is bearer access.**

## The tr form has no key-path spend

Its internal key is `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0`
— the constellation's NUMS pin, script-path only. Checked against the repo, not
against BIP-341's text: it is `md`'s `--unspendable-key` default and the
`nums_taproot` test vector's internal key
(`md-codec/src/test_vectors.rs:232`, `md-cli/README.md:73`). Anyone wanting the
stronger claim should diff it against the BIP directly.

## Keys, and the seating trap

Six keys, one seed each, all at `m/270028'/0'/0'/0'` — the bg002h path
(`m/270028'/coin'/account'/script'`, `0'` = tr). Seeds and xpubs live in
`../../journeys/inputs-hashvault/`.

**All six share one origin, so a keyless template that declares no fingerprints
is unseatable** — every card matches every slot and the device refuses
(`errSeatSlotContested`). Pass one `--fingerprint @i=HEX` per slot; it costs one
extra chunk and changes nothing else. See **F-227**, and
`SeedHammer-II-hashlock-vault-journey.pdf` for the three-arm device proof.

## Provenance

Every number above was produced by running `md`, not copied from a doc. The tr
form is the hashlock-vault journey's wallet
(`design/journeys/inputs-hashvault/wallet-policy-hashvault.txt`, byte-identical);
the wsh form was derived here and both `md verify` round-trip clean at exit 0.
