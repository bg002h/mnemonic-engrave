# PLAN: exporting "our reasonably complex wallet" to Sparrow Wallet

Date: 2026-08-22. Scope: the single question "what file(s) must the m* constellation
emit so Sparrow imports the four-tier degrading vault, watch-only and hot" — per
wrapper (`tr`, `wsh`). Nothing else.

Evidence base: shallow clones of `sparrowwallet/sparrow` @ `e7ae9cc` (2026-08-21)
and `sparrowwallet/drongo` @ `b385610` (2026-08-21) read directly; GitHub releases
page and issue tracker fetched 2026-08-22. Sparrow was **not executed** — every
behavioral claim below is traced through source, and the two load-bearing regexes
were re-run mechanically against the fixture policies (see "machine checks").

---

## VERDICT

**Sparrow cannot represent this wallet. NO for both wrappings, watch-only and hot
alike. There is no file the constellation can emit — today or with any amount of
constellation-side computation — that makes Sparrow import this vault.**
[verified: source, all citations below]

Per wrapper, and they fail differently:

| | tr wrapping | wsh wrapping |
| --- | --- | --- |
| import outcome | **rejected loudly** — error dialog | **imports the WRONG wallet** after a misleading warning |
| exact behavior | `IllegalArgumentException: "Cannot determine the multisig threshold in a descriptor providing 6 keys"` | parses as plain **sortedmulti 3-of-6 P2WSH**; all miniscript structure (`or_i`, `and_v`, both `sha256` hashlocks, `older`, `after`, tiers 2–4) silently discarded; addresses are wrong |
| hot variant | same rejection | same mis-import, with live keys behind it |

The wsh case is the dangerous one: it is a **funds-relevant near-miss**, not a
refusal. Sparrow shows only a "Legacy multisig wallet detected — Sparrow supports
BIP67 compatible multisig wallets only" warning (about key sorting, not about the
discarded policy) and then constructs a 3-of-6 wallet whose addresses belong to
nobody's vault. A user who pastes our wsh descriptor gets a wallet that shows a
zero balance for the real vault, and whose receive addresses, if funded, are
controlled by a plain 3-of-6 — tier 1's hashlock gone, tiers 2–4 gone.
[verified: source, traced below]

### Why, at the root

Sparrow has **no miniscript engine**. The class named `Miniscript` in its
descriptor library drongo is a 59-line regex shim that only extracts a multisig
threshold from standard descriptors
(`drongo/src/main/java/com/sparrowwallet/drongo/policy/Miniscript.java`, patterns
`pkh?\(`, `tr\(`, `sp\(`, `multi\((\d+)`). [verified: read the whole file]

The entire wallet model admits exactly three policy types:

```java
public enum PolicyType {
    SINGLE_HD(...), MULTI_HD(...), SINGLE_SP(...)   // policy/PolicyType.java
}
```

and address derivation is a closed switch over them — anything else throws
`UnsupportedOperationException("Cannot determine addresses for custom policies")`
(`drongo/.../wallet/Wallet.java:729-740`), and no code path can construct such a
policy in the first place. Multisig is **always** BIP67 lexicographically sorted
(`ScriptType.java:323`, `pubKeyBytes.sort(new Utils.LexicographicByteArrayComparator())`).
[verified: source]

Taproot: `P2TR.getAllowedPolicyTypes()` returns `List.of(SINGLE_HD, SINGLE_SP)`
(`ScriptType.java:1210-1213`) — **no taproot multisig, no script trees, no
tapscript of any kind**. The PSBT signer says it in a comment:
`//For now, only support keypath spends` (`drongo/.../psbt/PSBTInput.java:1123`).
So even as a mere PSBT co-signer coordinated by some other wallet, Sparrow cannot
sign any leaf of the tr vault (the tr form is script-path-only under a NUMS key —
there is no key path for Sparrow to sign). [verified: source]

Hashlocks: drongo *parses* `PSBT_IN_SHA256` preimage fields (`PSBTInput.java:31,
247-253`) but nothing in drongo or sparrow ever consumes `sha256Preimage` for
signing or finalization — grep across both repos finds only the parser, the
combiner, and a getter/setter. Preimage-gated witnesses cannot be built.
[verified: grep across both source trees]

Project status: miniscript descriptor support is **open feature request
sparrowwallet/sparrow#1700** (opened 2025-04-26, still Open, no maintainer
commitment visible). Latest release is **2.5.3, 2026-07-30**; no release from
2.2.3 (2025-06) through 2.5.3 mentions miniscript. [verified: GitHub issue +
releases pages fetched 2026-08-22; the issue page loaded partially — see open
questions]

---

## Exact import behavior, traced (the worked example)

Sparrow's import routes for a descriptor-shaped wallet are: (1) paste into
Settings → "Edit" output descriptor (`SettingsController.editDescriptor`, →
`setDescriptorText` → `OutputDescriptor.getOutputDescriptor(text).toWallet()`);
(2) File → Import Wallet → "Output Descriptor" file (`io/Descriptor.java`, same
parser); (3) BSMS / BIP-129 (`io/Bip129.java`, same parser + first-address
check); (4) Sparrow's own wallet file (`io/Sparrow.java` / `JsonPersistence` —
deserializes into the **same** `Wallet`/`PolicyType` model, so hand-crafting one
buys nothing). All four bottom out in the same parser and the same three-policy
model. There is also **no address-only watch route** — the only `addr(` script
type is P2A (ephemeral anchor) and its `getAllowedPolicyTypes()` is
`Collections.emptyList()` (`ScriptType.java:1333-1335`). [verified: source]

The parser (`drongo/.../OutputDescriptor.java:507-631`) does **not** parse a
script tree. It: (a) picks the script type from the descriptor **prefix only**
(`ScriptType.fromDescriptor`, longest-prefix match — `wsh(` → P2WSH, `tr(` →
P2TR); (b) takes the threshold from the **first** regex match of
`multi\(\s*(\d+)` anywhere in the string; (c) scoops **every** extended key out
with `XPUB_PATTERN`; (d) throws away everything else. [verified: source]

### What each of our descriptors does to it

The keyed exports (what `md descriptor` renders — six real
`[fp/270028'/0'/…']xpub…/<0;1>/*` expressions in place of `@0…@5`):

**tr**: `tr(50929b74…ac0,{and_v(v:sha256(…),multi_a(3,…)),…})`
- prefix `tr(` → P2TR. `multi_a(` does **not** match `multi\(` (the `_a` breaks
  the literal), so threshold = ABSENT. Six xpubs are found. Then
  `OutputDescriptor.java:619-621` throws:
  `"Cannot determine the multisig threshold in a descriptor providing 6 keys"`.
  The UI wraps it in an "Invalid output descriptor" error dialog
  (`SettingsController.setDescriptorText`). **Clean, loud rejection.**
  [verified: source + regex machine-check below]

**wsh**: `wsh(or_i(and_v(v:sha256(…),multi(3,…)),or_i(…multi(2,…)…)))`
- prefix `wsh(` → P2WSH. First `multi(3` match → threshold **3**. Six xpubs.
  `1 <= 3 <= 6` passes. Result: `OutputDescriptor(P2WSH, 3, {6 keys})` →
  `toWallet()` → `PolicyType.MULTI_HD` wallet whose addresses are
  `MULTISIG.getOutputScript(3, sortedPubkeys)` — a **sortedmulti 3-of-6**.
- On the paste route the only friction is
  `LEGACY_MULTI_PATTERN` (`(?<!sorted)multi\(`) matching, which pops the warning
  `"Legacy multisig wallet detected / Sparrow supports BIP67 compatible multisig
  wallets only. The public keys will be lexicographically sorted…"`
  (`SettingsController.java:492-494`) — a warning about *ordering*, saying
  nothing about four discarded spend tiers — and then imports.
  **Mis-import with wrong addresses.** [verified: source + regex machine-check]
- If a `#checksum` is present it is validated over the pasted string and
  **passes** (it is our own checksum of our own string) — the checksum is no
  guard here. [verified: OutputDescriptor.java:511-520]
- The **BSMS route fails loudly, but only by luck of BIP-129's design**: after
  the same mis-parse, `Bip129.java:315-333` derives the first address from the
  sorted 3-of-6 and compares it to the record's stated first address (which we
  would compute from the real miniscript); they differ, so it throws
  `"The first address in this BSMS record (…) does not match the first address
  of … derived by sorting the provided keys"`. [verified: source; not executed]

### Machine checks run

Sparrow's two live regexes re-run (Python `re`, same PCRE semantics) against the
fixture files `design/fixtures/reasonably-complex-wallet/{tr,wsh}.policy`:

```
tr  MULTI_PATTERN: NO MATCH | LEGACY_MULTI: False | prefix: tr(
wsh MULTI_PATTERN: 3        | LEGACY_MULTI: True  | prefix: wsh(
```

[verified: executed 2026-08-22. Note the fixtures carry `@i` placeholders; the
keyed export adds six xpubs, which only feeds the same code paths — the 6-key
count comes from the fixture README ("six keys, one seed each"), verified
separately by the constellation, not re-derived here.]

---

## The hot-wallet case

Moot for this wallet — the policy cannot be represented, so there is no hot
variant to build. For the record, the mechanism Sparrow *does* have:

- A hot Sparrow wallet is one whose keystores are software keystores, created
  in-app from **BIP39 mnemonic (+ optional passphrase), master xprv, Codex32
  (BIP93), or SLIP39 shares** (`io/Bip39.java`, `Bip32.java` "Master Private
  Key", `Bip93.java`, `Slip39.java`; keystore import framework
  `KeystoreMnemonicImport`/`KeystoreXprvImport`). [verified: source listing +
  class names]
- A pasted descriptor containing a **bare master xprv with no key-origin**
  becomes a hot keystore via `Keystore.fromMasterPrivateExtendedKey`
  (`OutputDescriptor.getOutputDescriptorImpl` → `masterPrivateKeyMap` →
  `toWallet()`, lines 594-604 and 334-346). An xprv **with** an origin or
  non-standard child path is downgraded to its xpub with the warning "Sparrow
  will convert the provided private key to a public key for use in a watch only
  wallet" (`SettingsController.java:505-509`). [verified: source]
- So the hot file, had the wallet been representable, would be the same
  descriptor with the six xpubs replaced by the six **master xprvs** (derived
  from the six seeds in `design/journeys/inputs-hashvault/`) — i.e. all six
  seeds' full signing power in one plaintext file. One-sentence security flag:
  that file is the entire vault minus the three preimages, and unlike the
  engraved plates it is copyable and greppable. [unverified: the exact xprv
  form Sparrow would want was not exercised, since the wallet cannot import
  anyway]

---

## What the constellation must compute / emit

1. **For Sparrow, for this wallet: nothing — and that must be enforced, not just
   documented.** If the export surface ever grows a `--target sparrow` (or any
   "descriptor for import elsewhere" file), it must classify the policy first
   and **refuse** non-representable ones (anything beyond `pk`/`sortedmulti`
   under the standard wrappers), because Sparrow itself only half-refuses: tr
   errors out, but **wsh imports as a different wallet with a warning that
   misidentifies the problem**. Handing a user our wsh descriptor labeled
   "for Sparrow" is handing them a wrong-address wallet. [verified: behavior
   traced above; the recommendation itself is judgment]
2. Nothing else. No file shape, field set, or precomputation (checksums,
   normalized form, BSMS packaging, hand-crafted Sparrow JSON) changes the
   verdict — every route terminates in the same three-policy model.
   [verified: all four routes traced to the same model]
3. Out of scope but worth one line for the parent plan: for wallets the
   constellation *can* express in Sparrow's dialect (single-sig, sortedmulti),
   the right emission is a plain **descriptor text file** (`io/Descriptor.java`
   imports it; multipath `/<0;1>/*` is accepted — `XPUB_PATTERN` and
   `toWallet()` handle it explicitly). No descriptor length limit exists in the
   parser. [verified: source; length claim = absence of any such check in
   OutputDescriptor.java]

---

## Sparrow-side limits that bite (summary against the brief's checklist)

- **Miniscript dialect**: none at all — not a dialect difference from
  rust-miniscript but total absence. [verified: source]
- **Taproot script paths**: not represented, not signable ("only support
  keypath spends"). The tr vault has *only* script paths. [verified: source]
- **Keyless spend path (tier 4)**: unrepresentable — every Sparrow wallet is a
  set of keystores; there is no keystore-free policy and no address-only watch
  wallet. [verified: PolicyType enum + P2A empty policy list]
- **Watch a policy it cannot sign**: yes in general (`SW_WATCH` keystores), but
  only for policies the model can express — so not this one. [verified: source]
- **Preimages to display vs to spend**: moot; it can neither display the
  hashlock condition nor build a preimage witness (field parsed, never
  consumed). [verified: source grep]
- **Descriptor length**: no limit found in the import path. [verified: absence
  in source]

## Open questions (explicitly unresolved)

1. **Roadmap**: whether Sparrow will gain miniscript. Issue #1700 is open with
   no visible maintainer commitment; my fetch of the issue page loaded
   partially (GitHub error banner), so a buried comment from Craig Raw could
   exist that I did not see. No release note through 2.5.3 mentions it.
   [unverified beyond the partial fetch]
2. **Source vs. release skew**: claims are from `master` (2026-08-21), one
   month past the 2.5.3 tag. I did not diff the tag; the relevant classes
   (`PolicyType`, `Miniscript`, `OutputDescriptor`) are structural and stable
   across the 2.x notes I read, but 2.5.3-exact line numbers were not checked.
   [unverified: reasoning]
3. **Not executed**: Sparrow was never run against the real keyed descriptors;
   the tr rejection message and wsh mis-import are source-traced with the two
   deciding regexes machine-checked, not observed in the GUI. If the parent
   wants an executed proof before publishing a user-facing "do not use Sparrow"
   claim, it is a ~10-minute headless-JVM test against drongo alone
   (`OutputDescriptor.getOutputDescriptor(...)` on both keyed strings).
   [unverified: end-to-end; verified: each deciding step]
4. **Six-xpub keyed export**: taken from the fixture README (six keys, one seed
   each) and the settled facts in the dispatch brief; the rendered `md
   descriptor` output itself was not re-run here per the brief's "already
   verified, do not re-check". [verified: fixture README]
