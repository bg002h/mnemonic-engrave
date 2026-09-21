# F-449 spec r0 — adversarial review (fable)

**Artifact:** `design/SPEC_liana_unspendable_internal_key.md` (DRAFT r0).
**Question asked:** construct a scenario in which the design makes a user (a) engrave plates that define a different wallet than the descriptor they import, (b) lose the ability to recover a wallet they created, or (c) believe a wallet is Liana-compatible when it is not.
**Repos read:** descriptor-mnemonic `6cbd49d8`, seedhammer fork `7b6f2fb`, mnemonic-engrave `49964db1`, Liana v15.0 checkout (`.tmp/fable-liana-src-v15`), libnunchuk main `a7cfb49` (`.tmp/fable-nunchuk-lib`). Every `md` command below was RUN against the installed `md 0.17.0` (md-codec 0.45.1, matches the tree).

## Verdict: 0 Critical / 4 Important / 9 Minor

**No Critical.** I could not construct a sequence that ends in a silently wrong wallet or an unrecoverable one. Every wrong-wallet path I tried is closed by a refusal somewhere — the header exact-match, the auto-dispatch, Nunchuk's own re-render check, md's depth check on `--key`, the composer's lock bounds, the fork's §8g notice. That is a real result and the design's core idea (a version bump as the fail-closed seam, the derived key taking no slot) survives.

**Four Important findings block**, and the first one means the spec as written cannot be implemented: the version number it picks is one the wire format's own §2.4 forbids, and a faithful implementation refuses its own plates on the device and in `md repair` while passing the spec's §8.4 round-trip gate.

---

## I-1 — Version 5 collides with the chunked-flag bit; a faithful implementation refuses its own single-string plates on the device

**Blocks.** Unsound assumption in §3c/§3d; the acceptance gate in §8.4 cannot see it.

**The fact the spec missed.** The single-payload header is 5 bits, `[paths][v3][v2][v1][v0]` (`header.rs:29-31`). The chunk header's first symbol is `[v3][v2][v1][v0][chunked]` (`chunk.rs:3-4`). Auto-dispatch reads **bit 0 of the first symbol** and routes `1` to the chunked reassembler (`chunk.rs:650-651` Rust; `md/chunk.go:193` and `md/md.go:1235` Go). For a single payload, bit 0 of the first symbol **is `v0`, the low bit of the version**. `SPEC_v0_30_wire_format.md` §2.4 says so explicitly: *"WF-redesign versions must have v0 = 0 (i.e. even values) … Usable WF-redesign versions: {4, 8, 12}."* `header.rs:4` and `:26` repeat the set in the very file the spec cites at §3c.

**Version 5 = `0101`, `v0 = 1`.** Every single-string v5 payload dispatches as chunked.

**Sequence.**
1. Operator composes a kofn-recovery wallet at kind 1 on a device running firmware built to this spec; the md1 is short enough to be one string (the common case — the measured preset templates above are 40-60 symbols).
2. The device cuts the plate at version 5 (§3d encoder rule).
3. The device reads the plate back (`gui/multisig_verify.go:834` → `ExpandWalletPolicyChunks` → `Reassemble`; the gather path at `gui/md1_gather.go:31` and `sysw/confirm.go:123` go through `ParseChunkHeader` first). `syms[0]&1 == 1` → 37-bit chunk-header parse → the 4-bit version field reads bits 4..1 = `[paths][0][1][0]` = **2** (or 10 with the divergent bit) → `errWireVersion` (`md/chunk.go:87`), even after that check is widened to `{4,5}`.
4. Host: `md repair` on the same string goes through `decode_with_correction` → same dispatch → `WireVersionMismatch { got: 2 }` (`chunk.rs:650-664`, `chunk.rs:70`). The toolkit's `repair` uses the same entry (`mnemonic-toolkit/src/repair.rs:8-9`).
5. Host: `md decode`, `md descriptor`, `md verify`, `md build`, `md inspect` and `me` all call `decode_md1_string` **directly** (`cmd/decode.rs:22`, `cmd/descriptor.rs:277`, `cmd/verify.rs:31`, `cmd/build.rs:92`, `me-cli/src/bundle.rs:371`) and never touch the dispatcher — so the same plate decodes on the host and is refused on the device.

**Wrong outcome.** The device cannot read the plate it just cut; a scratched kind-1 plate cannot be repaired by any tool; and the error names version 2, which no encoder ever emitted. Fail-closed, so no wrong wallet — but the spec's §3c claim ("the seam that works … already exists") is false for the value chosen, and a plan built on it either ships this or quietly rewrites SPEC v0.30 §2.3's dispatch rule in both ports.

**Why §8 does not catch it.** §8.4's round trip is "encode → decode → render". Written against `decode_payload`/`decode_md1_string` it passes. Only a round trip through `decode_with_correction` (or the Go `ParseChunkHeader`/`Decode` pair) sees the collision. The Go port's tests would fail immediately — but that is stage 3, after stage 1 has been declared green.

**Also affected by the same fact:** §3d's "a future third recipe would require version 6" — 6 is even and would dispatch correctly, but SPEC §2.4 reserves 2 and lists the usable set as {4, 8, 12}; the spec should say which set it is drawing from rather than counting upward from 5.

---

## I-2 — §8 has no device-address gate, and the device's derivation is exactly where a kind-1 divergence would hide

**Blocks.** Missing case / unsound acceptance. F-449's own acceptance clause (FOLLOWUPS.md:15726-15729) is *"derives byte-identical addresses **on the device**, in md, and in Liana"*. §8 keeps md and Liana and drops the device.

**Why "same vectors" in §9.3 does not cover it.** The Go `md/` port never derives an address. The device derives in `gui/policy_address.go:132-160` (calls `md.EmitTapLeavesChunks` for `ikIndex, isNUMS`, then `address.NUMSInternalKey()` or `address.DeriveChild(internal, index, change)`) and `address/taproot_script_path.go`. Neither is reached by a `md/` vector. A kind-1 device would have to (i) collect leaf pubkeys from `byIndex` (slot-keyed) into **descriptor order, with repeats**, (ii) build an xpub with pubkey H and that chain code, (iii) derive its child at `0/i` or `1/i`, (iv) tweak. Three of those four steps have no gate.

**Two concrete divergence vectors that every §8 vector is blind to.**

*(a) `sortedmulti_a`.* `address/taproot_script_path.go` (const `TapLeafSortedMultiA`, "keys sorted by DERIVED key") sorts the leaf's keys **per index** before emitting the script. Liana's recipe iterates `ms.iter_pk()` in **written** order (`analysis.rs:426-431`), which for md is wire/slot order (`render.rs:246-249` renders `sortedmulti_a` indices as stored). A device implementer who feeds the already-sorted list into the chain-code hash gets a chain code that changes with the address index and never matches md's. None of the eight evidence shapes has a `sortedmulti_a` leaf (`fable-liana-parse-in.jsonl:71,77,83,89,120,151,252,258` — all `multi_a`/`pk`), so §8.1/§8.3 cannot fail on this. Reachable: `md compose --wrapper tr --preset plain-multisig,2of3` emits `sortedmulti_a` (spec §1 row 3) and nothing in §6 refuses kind 1 on it.

*(b) a non-`<0;1>` use-site.* `md encode` accepts `@i/<2;3>/*`, `@i/<0;1;2>/*` and `@i/<0;1>/*h` under `tr(H, …)` today (RUN, all three produced md1 strings). §2.5 and §4 pin the internal key's **rendering** to `/<0;1>/*` but say nothing about **derivation**. Liana accepts an internal key at `<0;1>` beside leaves at `<2;3>` (its `DescKeyChecker` only requires two unhardened alternatives, `analysis.rs:113-129`; `from_str` never compares alternatives across keys, `mod.rs:116-141`) and pairs them positionally through `into_single_descriptors`. A device that applies the wallet's use-site to the internal key derives at `2/i` and shows an address Liana will never derive.

**Wrong outcome if either lands.** The operator verifies a receive address on the device (the address-routes cycle's whole purpose), it disagrees with Liana's, and whichever one they trust, the other party cannot see the funds. Recoverable by someone who knows which internal key was actually used, which is the definition of the plates not defining the wallet.

**Why existing guards do not catch it.** §8.3 is measured in md only; §9.3's vectors stop at `md/`; the two shapes that expose (a) and (b) are not in the evidence.

---

## I-3 — The input side is unspecified: md will emit Liana's key but never recognise it, and today that already yields a phantom slot

**Blocks.** Missing case. The spec covers composer → Liana. The natural opposite journey — a wallet **created in Liana** (which always uses this recipe for taproot, `mod.rs:179` → `analysis.rs:446`) carried onto plates — is not mentioned, and neither is the gate that depends on it.

**Measured today** on the evidence's `preset-kofn-recovery-tr` Liana-form descriptor (`fable-liana-parse-in.jsonl:71`):

```
$ md decompose "$D" --emit template
tr(@0/<0;1>/*,{multi_a(2,@1/48'/0'/0'/3'/<0;1>/*,@2/…,@3/…),and_v(v:pk(@4/48'/0'/1'/3'/<0;1>/*),older(26280))})
note: 1 key(s) state NO origin in this descriptor — @0 (xpub661MyMwAqRbc…) … EXCLUDED from the mk-mintable set
$ md decompose "$D" --emit commands
md: decompose: --emit commands cannot be produced: 1 key(s) state no origin …
```

The unspendable xpub becomes **slot `@0`** — a five-slot template for a four-key wallet, with a slot nobody can mint a card for. (`md encode --key @0=<that xpub>` is refused: *"expected an account-level xpub at depth 3 or 4 … got 0"* — RUN — so the phantom cannot be seated on the host either; the template-only card still encodes, with the partial-decode warning.)

**Sequence.** Liana user → `md decompose … --emit template` → engraves the template card → hand-mints four mk1 cards for @1..@4 → years later `md descriptor --from-mk1 …` → `@0` unseated → refusal. The plates carry a wallet with one slot the wallet does not have. Fail-closed at restore, but only for a user who never learns that md's own kind 1 was the honest spelling.

**Three consequences the spec has to own.**
1. `md decompose` (and the fork's `md.` parser for loaded descriptors, if any) needs a recogniser: recompute §2 over the leaves, compare, encode kind 1. Otherwise two md1 spellings of one wallet exist — the composer's (4 slots, kind 1, v5) and decompose's (5 slots, `Slot(0)`, v4) — with different `WalletDescriptorTemplateId`s and different `SkeletonKey`s.
2. The coordinator-compat conformance gate is `chunks -> key == descriptor -> key` **for every evidence row** (`DESIGN_coordinator_compatibility.md:620-621`). For the `liana-unspendable-xpub` rows `descriptor -> key` goes through the phantom-slot parse and then fails `expand_per_at_n` (no origin on @0 → `SkeletonError::KeysDoNotExpand`, `skeleton.rs:186-193`), so those rows cannot be keyed at all — the ACCEPT evidence this spec exists to exploit is unreachable by the table build.
3. The design document contradicts this spec: `DESIGN_coordinator_compatibility.md:155-158` — *"An unspendable xpub is NOT a variant here: it is an ordinary key_index on the md1 wire, and recognising one means re-deriving a specific coordinator's own function — so that distinction lives in a rule, not in the key."* §7 adds the fourth `KeyPathKind` value but never reconciles that paragraph.

---

## I-4 — Nothing in the spec's own stages lets a device operator get a kind-1 wallet

**Blocks.** Unsound deliverable. §0 says *"An SH2 operator can compose a wallet … whose internal key is an unspendable xpub built by Liana's own recipe"* and claims 1-of-6 → 3-of-6 presets. §0 also rules the target-selection mode out of scope. §9's device stage (4) is *"§7's KeyPathKind, class 2, and the F-633 copy fix"* — verdict plumbing only. `md compose --unspendable` (§9.2) defaults to `nums`.

**Measured:** `composerLianaOutsideModelClass` is called from `gui/composer_consent.go:261` only — the composer consent screen. Loaded payloads (`gui/wallet_policy.go`) never run it. So after all five stages ship: the device composer still emits kind 0 for every NUMS tr wallet (`md/compose.go:961` `isNums: ik < 0`), the new class-2 skip fires on wallets the composer cannot produce, and the only way a device ever holds a kind-1 payload is `md compose --unspendable liana | md encode` on a host and an NFC load — which shows no Liana verdict at all.

Either the device gets a choice (which the spec says is a later cycle) or the device defaults to kind 1 for NUMS tr (which changes every existing NUMS operator's wallet form silently and makes §8f's Nunchuk sentence wrong in a new way — see M-6). The spec must pick one; both have consequences it does not weigh.

---

## Minor

**M-1 — The marker has no parser.** `md compose` emits a template, not an md1 (`md compose --help`, RUN); the operator feeds it to `md encode`, whose `walk_tr` (`md-cli/src/parse/template.rs:1589-1632`) accepts exactly `H` or an `@N`. `UNSPENDABLE(liana)` will be handed to rust-miniscript as a key and refused. §4 defines the marker; nothing says the template grammar admits it. Also `md compose --json`'s "taproot internal-key path" field and `md decode --json`'s `is_nums: bool` (`format/json.rs:326`, `docs/json-schema-v1.md`) need a third state — the JSON schema is a published v1 contract.

**M-2 — §8.1 overclaims for the refused shapes.** The nested tree `{A,{B,C}}` appears only in `preset-decaying-multisig-tr` (`parse-in.jsonl:89`), which Liana refused on policy shape, so no ACCEPT backs the recipe's order for a tree deeper than one level. It is right by reading rust-miniscript's `TapTreeIter` (left pushed last, popped first = left-first DFS), but "measured, not transcribed" is not true of that row.

**M-3 — Fork consumers switch on `KeyPath` with no default.** `gui/template_engrave.go:159-164` (*"THE KEY-PATH LINE COMES FIRST AND IS NEVER OMITTED"*) and `gui/composer_consent.go:205-214` print nothing for a fourth value. Harmless for an unspendable key, but the engrave summary's own invariant is broken silently.

**M-4 — §6's "ACCEPT at decode" for v5-all-kind-0 creates a second encoding of every kind-0 wallet.** `WalletDescriptorTemplateId` hashes the tree bits (`identity.rs:53-58`), which at v5 include the kind bit, so the same wallet has two template ids and two `Md1EncodingId`s depending on version. Hand-crafted only (the encoder refuses), but identity consumers that assume one wallet ↔ one id should be told.

**M-5 — §8f's Nunchuk sentence becomes probabilistically wrong at kind 1.** libnunchuk (`a7cfb49`) accepts any H-xpub at parse (`descriptor.cpp:507-509`, `IsUnspendableXpub` at `:714-719` ignores the chain code), re-renders the key path with the PR-1746 recipe — sorted, deduplicated — (`dto/wallet.cpp:213-214` → `GetUnspendableXpub`, `descriptor.cpp:689-712`), then **requires the re-render to equal the input** (`descriptor.cpp:640-648`, "Failed to verify wallet descriptor"). So Nunchuk refuses a kind-1 descriptor unless the leaf pubkeys already sit in sorted, unique order — 1/2 for two keys, 1/6 for three, 1/24 for four — in which case the two recipes coincide byte-for-byte and Nunchuk imports **the same wallet**. "Nunchuk cannot import a NUMS policy at all" (`composer_copy.go:199-202`) is therefore false for kind 1 some of the time, and the F-633 copy fix that §7 makes gating must say so.

**M-6 — `--unspendable liana` with a bare single path.** `internal_key_path` extracts the first bare single (`compose/tr.rs:11-13`); §6 says kind 1 with a real internal key is "unrepresentable by construction". The CLI flag then has no effect and no message. The user asked for no key-path spend and got one. Not a funds issue (Liana accepts either); it is a silent default where a refusal or a note was owed.

**M-7 — The old-decoder error names version 2.** A v4-only reader — every shipped device and every installed `md` — will report `WireVersionMismatch { got: 2 }` for a single-string v5 plate, because of I-1's dispatch. An operator reading that will look for a version-2 tool. Documentation at minimum; moot if I-1 changes the version.

**M-8 — §2's order rule needs one more sentence.** "Descriptor left-to-right order" is right, but the port note should say *wire order, never the derived-key-sorted order the address builder uses for `sortedmulti_a`* — that is the exact line a device implementer would otherwise get wrong (I-2a).

**M-9 — Pin the internal key's derivation path, or refuse kind 1 off `<0;1>`.** §2.5 fixes the rendering; state in §4 or §6 that the internal key derives at `0/i` (receive) and `1/i` (change) regardless of the wallet's use-site path, in every port — or refuse kind 1 when the use-site is not `<0;1>`, which is what keeps §8.3 honest (I-2b).

---

## Attacks that FAILED — what is actually defended

- **Re-seat / replace a leaf key after composition.** The internal key is never stored; it is recomputed from whatever is seated, so the wallet stays self-consistent. Swapping two same-path cards between slots would change the chain code (and, for `multi_a`, already changes the script today), but the seating engine refuses same-path ambiguity without fingerprints (`seat/partition.rs:214-224` `Ambiguous`; memory "same-path keys can't be seated") and binds exactly with them. No new order-sensitivity is reachable that kind 0 + `multi_a` does not already have.
- **Structure-independence of the chain code (§2).** Harmless as claimed: the output key commits to the tree through the tweak, and `SkeletonKey` carries the template, so two trees over the same keys neither collide on address nor on evidence key.
- **A v5 plate reaching a v4 decoder.** Refused everywhere: single-string via the dispatch (I-1's mechanism, `got: 2`/`10`), chunked via the exact-match at `chunk.rs:70` / `md/chunk.go:87` (`got: 5`). Mixed 4/5 chunk sets are refused by `chunk.rs:373`. No misparse path exists. (`WireVersionMismatch` is the right property; only the value is wrong.)
- **Slot counting (§5).** `key_index_width` is `ceil(log2(n))` over `path_decl.n` (`decode.rs:88`); the derived key adds no slot, so widths, seat counts, mk1 counts and `@i` numbering (`compose/lowering.rs:148-180`) are untouched. `duplicate_keys.go:185-199` already skips the NUMS internal key and will skip kind 1 the same way.
- **Kind 1 unmasking a Liana class the fork cannot name.** Walked every refusal in `analysis.rs` against `composerLianaOutsideModelClass` (`composer_consent.go:381-470`): `InsaneTimelock` for `older` > 65535 blocks — composer refuses `older=65536` and `older=70000` (RUN: *"older in blocks needs 1..=65535"*); `older(0)` — refused (`compose/mod.rs:117`); `DuplicateOriginSamePath` — §8g names Liana (`composer_copy.go:263-275`); `DuplicateKey` — md refuses duplicate slots (F-531/F-533); `InvalidKey` for a non-`<0;1>` or hardened use-site — the composer hard-codes `<0;1>` (`md/compose.go:894-896`) and the classifier only runs on composer output; `IncompatibleDesc` for `tr` without a tree — class 3. No class goes missing.
- **Nunchuk as a second coordinator deriving a different wallet from the same kind-1 string.** Closed by Nunchuk's own re-render-must-match check (M-5). When it accepts, the xpubs are identical and so are the addresses.
- **`tr(H)` with no tree at kind 1.** §6 refuses (no leaf keys); today's `md encode "tr(H)"` is refused for a different reason (*"template contains no @i placeholders"*, RUN).
- **Liana changing its recipe later.** Its comparison is by xpub equality (`analysis.rs:596-601`); a changed recipe would demote the old key to a spendable key without origin → `InvalidKey`, loud. Not our concern, and it would break Liana's own wallets first.
- **F-611 (zero parent fingerprint on md's emitted xpubs).** Irrelevant to the recipe — only bytes `[45..78]` (the point) enter the hash, and Liana's equality on the *internal* key is against its own depth-0/parent-0 construction, which §2.4 reproduces.

## Facts relied on and not re-derived (per brief)

Liana's recipe reproduction for the four accepted shapes; the 14 file:line citations; `header.rs:42` exact-match; the three out-of-model presets.
