# f449 SPEC r3 — new-design review (scope: only what is new in r3)

**Verdict: 0 Critical / 6 Important / 4 Minor**

Artifact: `design/SPEC_liana_unspendable_internal_key.md` at `9d909a30`
(diff base `a1136d97`). Scope: §0b, §3e's new Go paragraph, §6a, §9 stage 4a,
§7a.3's new scope sentence. Everything else was out of scope and not read for
findings.

Repos as measured: fork `seedhammer` at `7b6f2fb`, `descriptor-mnemonic` at
`6cbd49d8`, `mnemonic-engrave` at `9d909a30`, local `md 0.17.0`.

Two of the five items are **sound as written and I found nothing blocking in
them**: §3e's Go paragraph (item 2) and §7a.3's scope sentence (item 5). The
six Importants sit in §0b (2), §6a (1) and §9 stage 4a (3).

---

## I-1 (§0b, DEFAULT ROW) — `Initial` = the NUMS row is right on the first pass and wrong on every later one; the composer closed this exact defect twice

§0b rules: *"**DEFAULT ROW.** `Initial` is the **NUMS row**. The widget's zero
value selects row 0, so a default row of 'Liana xpub' would re-implement the
alternative §0 explicitly rejects."*

That rationale reaches only the **first** pass. §0b's own PLACEMENT ruling puts
the screen between `composerShapeFlow` and the first `composerStubFlow`, which
is **inside `composerFlow`'s `for !ctx.Done` loop** (`gui/composer_flow.go:83`
opens it; `:85` is `composerShapeFlow`, `:129` the first `composerStubFlow`,
`:133` `if !forward { continue }`). The screen therefore re-fires on every
forward pass, and an unconditional row-0 `Initial` **displays NUMS while the
state holds Liana**.

**Reproduction (structural, from the shipped flow):**

1. tr / `kofn-recovery`. Predicate fires (measured, see the sound section).
   Operator picks "Liana xpub". Kind = 1.
2. Stub screen draws the kind-1 Template-ID and mk1 stub.
3. Operator presses **Back** to re-read the path list — the documented,
   encouraged navigation. `forward == false` → `continue` → top of the loop.
4. `composerShapeFlow` → operator presses **Done**, editing nothing.
5. The choice screen fires again. Per §0b, `Initial` = the NUMS row.
6. The forward key — the control that has advanced every other screen — commits
   **kind 0**. Different addresses, different `WalletPolicyId`, different
   `WalletDescriptorTemplateId`, different mk1 stub (§3e).

This is journey C-1's class, and this package has closed it twice already, both
times by **opening on the value in force**:

- `gui/composer_shape.go:207-210`, `composerWrapperPick`: *"It opens on
  `current`, never on row 0 (journey C-1). A picker that always opens on row 0
  is not showing a setting, it is proposing one, and an operator who opened
  'Change the script' to READ the wrapper and left by the forward button
  committed Taproot over their Segwit policy with nothing downstream to report
  it."*
- `gui/composer_shape.go:353-366`, `composerKeyOrderStep` — the *closest*
  analogue there is: a two-row `ChoiceScreen` at the transition out of the
  shape. Its doc comment records the measured walk verbatim: *"the operator
  picked 'Keep my order', held §8b to confirm, reached the Template screen,
  pressed Back to re-read the list, pressed Done, and the key-order question was
  asked again OPENING ON ROW 0. The forward button … then made the wallet
  sortedmulti, §8b did not re-fire, and the only signal was the stub screen's
  'The shape changed, so this id changed', which names no cause."*

Step 6 above is that paragraph with two words changed.

**Note the ruling's goal is met for free by the existing convention.** `Initial`
= the kind in force opens on NUMS on the first pass (the kind starts at 0), so
§0b's stated concern — "a default row of 'Liana xpub'" — never arises, and the
re-entrant case is correct as well. The ruling as written is strictly worse than
the convention already in the file, with no compensating benefit I can find.

**Mitigation that exists, and why I did not grade this Critical.** With the
screen placed before the stub screen, `composerStubDelta` *will* fire on the
second pass, because the template chunks moved. So the operator is not entirely
without signal. That is the same signal `composerKeyOrderStep`'s comment calls
"names no cause" and which was judged insufficient there — so it mitigates
without closing. Graded Important; the outcome it produces (the other wallet
cut into steel, unstated) is Critical-shaped and an argument to raise it would
not be unreasonable.

Not prescribing a fix: the repo has two precedents and they disagree with the
ruling; which way §0b resolves that is the author's call.

---

## I-2 (§0b, RESET) — the rule is stated at a granularity the composer cannot observe, and the one existing detector is measurably blind to the edit that flips the predicate

§0b rules: *"**RESET.** The chosen kind resets to 0 on any shape, wrapper or
path-list edit that re-enters `composerShapeFlow`."* and then, in the same
sentence, cites the tension without resolving it: *"— and the composer's
documented Back invariant is 'going back should lose nothing'."*

Two readings, and the spec picks neither:

- **(a) any re-entry resets.** Trivially implementable (one assignment at the
  top of the loop), and it closes the unreachable state completely — but the
  kind is lost on a pure navigational Back, which is the invariant §0b itself
  quotes (`gui/composer_flow.go:87-89`, §7b's *"going back should lose
  nothing"*).
- **(b) only an actual edit resets.** Honours the invariant, but needs a
  detector for "an edit happened", and the composer has exactly one:
  `composerShapeSignature` (`gui/composer_discard.go:45-65`), which
  `composerApplyShapeEdit` compares at `:156-160`. That signature is
  **deliberately** wrapper + per-path key counts + the codec's slot mapping —
  it is documented as being about *slot numbering*, not about policy content.

**Measured (probe run in the `gui` package against fork `7b6f2fb`, then
removed).** Editing `kofn-recovery`'s recovery path from `older(26280)` to
`after(1000000)` under `tr` — an ordinary lock edit, which §7g classifies
DEFAULT:

```
sig older="w0/3,1,|0.0/0.1/0.2/1.0/"  after="w0/3,1,|0.0/0.1/0.2/1.0/"  equal=true
older  KeyPath=NUMS full="NUMS key path" class2skipped=""                FIRE=true
after  KeyPath=NUMS full="NUMS key path" class2skipped="an absolute lock" FIRE=false
```

The signature is **byte-identical** across the edit, and the firing predicate
**flips from true to false** across the same edit. So under reading (b)
implemented on the obvious hook, the operator can:

1. pick "Liana xpub" on tr/`kofn-recovery` (kind = 1),
2. Back, change that path's lock to a date, Done,
3. the screen never fires again (conjunct 2 now returns "an absolute lock"),
4. the kind stays 1, with **no screen that can unset it** — the exact state
   §0b's RESET paragraph exists to prevent, and a shape §0b itself calls a
   strict downgrade.

A secondary obstacle for either reading: §6a's sibling requirement aside, the
reset condition "a path-list edit" has no callable predicate today —
`composerPathEdit` is invoked bare at `gui/composer_shape.go:667`, outside
`composerApplyShapeEdit`.

I-1 and I-2 share a root: the kind's lifecycle across a re-entrant screen is
under-ruled. Both dissolve if the screen opens on the kind in force; I am
reporting them separately because they have distinct reproductions and would
need distinct answers if the author rejects that framing.

---

## I-3 (§6a) — three changes are declared "this cycle's to fix"; §9 owns one of them and gates none

§6a: *"All three are **this cycle's** to fix, because this cycle is what makes a
version-8 plate exist."*

Grep over the whole spec (`repair`, `error.rs`, `WireVersionMismatch`,
`gatherIgnored`):

| §6a row | surface | owning stage in §9 | gate that can see it |
| --- | --- | --- | --- |
| 1 `gatherIgnored` split | fork `gui/` | stage 3 | none — stage 3's gate is "§8 vectors in Go, including §8.4's `ParseChunkHeader`/`Decode` leg and §8.5's Go identity leg", all `md/`-level |
| 2 `md repair` discards a correction | `descriptor-mnemonic` `crates/md-cli` | **none** | none |
| 3 `WireVersionMismatch` Display | `md-codec/src/error.rs:33` | **none** | none |

`md repair` appears in the spec only at lines 244 and 533; `error.rs` only at
534. Neither is named in any stage row.

Stage 1b's "§6 refusals" cannot be stretched to cover them: row 3 changes an
error's `Display` string (not a refusal) and row 2 is in `md-cli`, which stage
1b (md-codec) and stage 2 (`md compose` / `md descriptor`) do not name either.
And stage 3 naming row 1 explicitly is itself evidence that §6a items are meant
to be scheduled individually.

This is the shape opus I3 already found once in this cycle (§8.8 declared
REQUIRED, scheduled nowhere), on a different section.

Row 1's placement is also mis-filed: stage 3's prose is "Go port in the fork's
`md/`", and `gatherIgnored` is `gui/` (`gui/mk1_inspect.go:36` declares it,
`gui/md1_gather.go:33,39` returns it, `:121` prints it). The split additionally
needs a **new exported surface from package `md`** — the sentinel is
`errWireVersion`, unexported at `md/md.go:22` and returned from
`md/chunk.go:88` — so `gui` cannot currently distinguish a version rejection
from any other `ParseChunkHeader` error. §6a does not name that dependency.

---

## I-4 (§9 stage 4a) — md-codec 0.45.1 does not compile in `mnemonic-engrave`'s graph; the unpin also requires a `[patch.crates-io] miniscript` override the stage does not name

Stage 4a's content: *"`me` (this repo): unpin `md-codec 0.42` from crates.io
onto the workspace/git source and carry the new version."*

**Measured.** I pointed `crates/me-cli/Cargo.toml:26` at the local md-codec
(0.45.1) and ran `cargo check -p mnemonic-engrave --all-targets`:

```
error[E0599]: no method named `derive_at_index` found for enum `miniscript::Descriptor<Pk>`
   --> descriptor-mnemonic/crates/md-codec/src/derive.rs:147:18
error[E0599]: no method named `into_definite` found for enum `miniscript::Descriptor<Pk>`
   --> descriptor-mnemonic/crates/md-codec/src/derive.rs:149:18
error[E0599]: no variant, associated function, or constant named `SortedMultiA` found for enum `Terminal<Pk, Ctx>`
   --> descriptor-mnemonic/crates/md-codec/src/to_miniscript.rs:481:23
error: could not compile `md-codec` (lib) due to 3 previous errors
```

`mnemonic-engrave`'s `Cargo.lock` resolves `miniscript 13.1.0` from crates.io.
`descriptor-mnemonic`'s root `Cargo.toml:43-44` carries
`[patch.crates-io] miniscript = { git = "https://github.com/rust-bitcoin/rust-miniscript", rev = "ff4732e5f75aa555682343cb180fa72ee3e8e9d5" }`
— an unreleased master rev — and md-codec 0.45.1 needs those three APIs from it.
`mnemonic-engrave`'s root `Cargo.toml` has **no `[patch]` section at all**.

Adding that same patch stanza is **not sufficient on its own**: with the
existing lock it lands in `[[patch.unused]]` (verified in the regenerated lock)
because the lock pins registry 13.1.0. Deleting `Cargo.lock` and re-resolving,
the build is clean:

```
Checking miniscript v13.0.0 (https://github.com/rust-bitcoin/rust-miniscript?rev=ff4732e5…)
Finished `dev` profile … in 8.37s     # cargo check -p mnemonic-engrave --all-targets, exit 0
```

So stage 4a is three things, not one: the md-codec source change, a
`[patch.crates-io] miniscript` git override at the **engrave workspace root**,
and a `Cargo.lock` re-resolve that moves `miniscript` off crates.io onto an
unreleased rev. CI runs `cargo clippy --all-targets --locked` and
`cargo test --locked` (`.github/workflows/release.yml:116,151`), so the lock
must be committed with it, and `rust-build` cross-compiles through `cross`
(`:298-316`), which fetches git deps in-container.

All working files were restored byte-exact (`md5sum` re-verified;
`git status --porcelain` shows no tracked modification).

---

## I-5 (§9 stage 4a) — §3f's type change breaks `me`'s own source, and no stage names it

§3f enumerates the `Body::Tr` blast radius carefully — 88 `is_nums` occurrences
across 14 md-codec files, the fork's `md/`+`gui/`, and **four** non-test md-cli
sites, and it goes to the trouble of clearing a fifth (`seat/compose.rs:148`,
which survives because `..` absorbs the field change). `me` is not on that list,
and stage 4a describes its work as "carry the new version".

`me` constructs `Body::Tr` **by field name**:

```rust
// crates/me-cli/src/descriptor/md1.rs:351-357
Script::P2TR => Node {
    tag: Tag::Tr,
    body: Body::Tr {
        is_nums: false,
        key_index: 0,
        tree: None,
    },
},
```

Replacing `is_nums: bool` + `key_index: u8` with `InternalKey` makes this a hard
compile error, and it is the one site in `me` that must *decide* which
`InternalKey` a single-key P2TR gets — a semantic choice, not a mechanical
rename. (`me`'s other `Body::Tr` use, `crates/me-cli/src/bundle.rs:260`, is
`Body::Tr { tree, .. }` and survives, exactly as §3f says of
`seat/compose.rs:148`.)

This is the second time in this cycle that `me` was the forgotten downstream —
the journey found it once for the pin (I6, now §9's new paragraph) and the same
paragraph does not carry it through to the type change.

---

## I-6 (§9 stage 4a) — the stage's gate is already satisfied by the status quo

Stage 4a's gate: *"`me` round-trips a version-8 payload, or refuses it with a
message naming the version."*

**Measured: the first disjunct is true today, on the pinned md-codec 0.42, with
no change at all.** `me`'s headline conversion never reads the wire version:

```rust
// crates/me-cli/src/lib.rs:75-83
pub fn convert(input: &str) -> Result<Vec<u8>, ConvertError> {
    let s = input.trim();
    let fmt = classify::classify(s)…;
    if fmt == Format::Ms { return Err(ConvertError::RefusedSecret); }
    validate::validate(fmt, s)…;
    ndef::encode_text_tlv(s)…
}
```

and `validate`'s `Format::Md` arm is a canonical-form check plus
`md_codec::codex32::unwrap_string(s)` (`crates/me-cli/src/validate.rs:95-100`)
— the codex32/BCH layer, which is version-agnostic. `encode_text_tlv` then
encodes the string verbatim. A well-formed version-8 md1 therefore round-trips
through `me` today.

The surface where the version *does* bite is a different one:
`crates/me-cli/src/sysw/record.rs:251-252` calls the real decoders
(`md_codec::reassemble` / `decode_md1_string`) and reduces them to `.is_ok()`,
so on 0.42 a version-8 record is silently classified **unconfirmed** with no
message naming anything. That is the §6a-shaped defect on `me`, and it is what
the gate should be pointed at. As written, the gate names no surface and its OR
lets the stage close without the change.

---

## M-1 (§6a row 2) — "documented as 'the plate is too damaged'" is not in the source

§6a row 2: *"…and exit 2 is documented as 'the plate is too damaged'"*. The
string `too damaged` does not appear anywhere in `descriptor-mnemonic`
(`grep -rn "too damaged" .` over the repo, excluding `target`/`.git`: no
matches). The actual documentation is `crates/md-cli/src/cmd/repair.rs:15-19`:

```
//!   - 2 — atomic-fail: BCH-uncorrectable / HRP-mismatch / parse-reject;
//!     caller's named-chunk error surfaces on stderr
```

which already distinguishes three causes, one of which ("parse-reject") a
version rejection arguably falls under. The row's *mechanism* is correct and I
verified it end to end (see the sound section), and its required change stands —
but a quoted sentence that is not in the source is the transcription class this
repo tracks, and it weakens the row's stated justification.

---

## M-2 (§0b, PLACEMENT) — the cited window is short by ten lines, and the ordering inside it is load-bearing but unstated

§0b: *"The screen sits **between `composerShapeFlow` and the first
`composerStubFlow`** (`gui/composer_flow.go:97-118`)."*

Measured: `composerShapeFlow` closes at `:95`, the first `composerStubFlow` is
at `:129`. The true window is `96-128`; `97-118` lands inside it but stops
mid-comment, ten lines short.

More substantively, the screen's position **within** that window is not free.
Conjunct 2 needs `composerLianaOutsideModelClass(root, shape)`, i.e. a decoded
`Template.Root` and a `PolicyShape`, and the only source of those in the window
is the chunks produced by `composerTemplateChunksFor(st)` at `:98` (the consent
screen gets both the same way, `md.PolicyShapeChunks` +
`md.ExpandWalletPolicyChunks`, `gui/composer_consent.go:158,171`). So the screen
must sit **after** `:98`, and the template must then be **recomputed** with the
chosen kind before `:129` or the stub screen displays a kind-0 id for a kind-1
wallet. §0b states neither.

---

## M-3 (§6a row 1) — the required split crosses the `md`/`gui` package boundary

Included in I-3 above; recorded separately because it is a distinct required
change: `errWireVersion` is unexported (`md/md.go:22`), so the `gatherIgnored`
split needs a new exported sentinel or typed error from package `md` before
`gui` can say "a well-formed md1 at an unsupported version".

---

## M-4 (§7a.3) — the probe is attributed one function too high

§7a.3: *"`complexAddressSource` probes `src(0, false)` and returns `nil, false`
rather than falling back."* The probe is at `gui/policy_address.go:188-190`, in
`complexAddressDeriver`, which `complexAddressSource` calls at `:88` after the
F-531 duplicate-slot gate. The claim is true of the call chain and
`complexAddressSource` is the right entry point to name (its own doc comment at
`:93-94` says every screen goes through it), so this changes nothing — recorded
only so an implementer greps the right function.

---

# What I checked and found sound

**§0b's firing predicate — CORRECT, and its diagnosis of r1 is CORRECT.**
I ran the predicate over all six `tr` presets in the `gui` package at fork
`7b6f2fb` (temporary test, removed afterwards; `git status` clean):

```
plain-multisig                KeyPath=NUMS      full="NUMS key path" class2skipped="no locked path"   FIRE=false
simple-timelocked-inheritance KeyPath=Spendable full=""              class2skipped=""                 FIRE=false
kofn-recovery                 KeyPath=NUMS      full="NUMS key path" class2skipped=""                 FIRE=true
tiered-recovery               KeyPath=NUMS      full="NUMS key path" class2skipped=""                 FIRE=true
hashlock-gated                KeyPath=NUMS      full="NUMS key path" class2skipped="a hash lock"      FIRE=false
decaying-multisig             KeyPath=NUMS      full="NUMS key path" class2skipped="an absolute lock" FIRE=false
```

Both conjuncts are load-bearing and neither is redundant: conjunct 1 alone would
admit `plain-multisig`/`hashlock-gated`/`decaying-multisig`; conjunct 2 alone
would admit `simple-timelocked-inheritance`. The predicate fires on exactly
`kofn-recovery` and `tiered-recovery`, which is §9 stage 4's stated gate. "Class
2 skipped" is also well-defined: `KeyPathNUMS` contributes nothing to the
`unlocked` count (`gui/composer_consent.go:406-408` increments only on
`KeyPathSpendable`), so skipping the class changes nothing but the early return.

§0b's three-row table is accurate: `simple-timelocked-inheritance` under `tr`
reports `"internal_key_path": 0` and the other tr presets report `null`
(verified with `md compose --wrapper tr --preset … --json`, local `md 0.17.0`),
which is exactly conjunct 1 on the device.

**§3e's new Go paragraph — SOUND, and the collision is reachable. Every
citation is byte-exact.**

- `writeNode` signature at `md/encode.go:159` is literally
  `func writeNode(w *bitWriter, n node, keyIndexWidth uint8) error` — version-less
  as claimed.
- The three non-test callers are exactly `md/encode.go:417`,
  `md/template_id.go:53`, `md/walletpolicyid.go:42` (the other hits are
  recursion at `:166/:184/:212` and two doc comments). 3-for-3 with Rust.
- `md/template_id.go:112` is the `FormAwareStub` dispatcher; `:113`→
  `WalletPolicyIDStub`, `:116`→`WalletDescriptorTemplateIdStub`. Both flavours
  root on `WalletPolicyId` / `WalletDescriptorTemplateId`, i.e. both go through
  the version-less `writeNode`. The r3 correction of the journey's `:116` is
  right and the widened-exposure claim is right.
- `gui/composer_consent.go:216,220` are `md.FormAwareIdChunks` /
  `md.FormAwareStubChunks`, printed at `:228` as the id and "mk1 stub".
- `policyIDHeader` (`gui/md1_gather.go:244-250`) is `md.WalletPolicyIdChunks`,
  printed as `Policy id: …`, reached from `md1PolicyFlow` at `:206`.

**The claimed consequence is real — traced.** §3d puts the kind bit *inside* the
Tr node body, gated on version 8 (`Tag::Tr | is_nums(1) | [kind(1) iff is_nums]
| …`), and at version 4 no kind bit is written. So a `writeNode` handed a
constant version 4 emits **identical bytes** for kind 0 and kind 1 of the same
tree → identical `WalletPolicyId` → identical `WalletPolicyIDStub`. Seating then
matches: `gui/key_card_seating.go:19-21` states the two layers as *"LAYER 1 the
card's policy_id_stub must include this template's stub; LAYER 2 the card's
origin must equal the slot's declared origin, and its fingerprint the slot's"*,
and `seatKeyCards` computes layer 1 with `md.FormAwareStubChunks(templateMd1)`
at `:54`. Layer 2 cannot separate the two either: per §5 the derived internal
key takes no slot, so kind 0 and kind 1 of one tree have the **same** slot set
and the same declared origins. An mk1 KEY card minted for the kind-0 wallet
therefore passes both layers against kind-1 plates. The Critical stands; it is
not downgradeable.

**§6a rows 1 and 3 — accurate, and row 2's mechanism is accurate.**

- Row 1: `md/chunk.go:87-88` returns `errWireVersion` for any version != 4, so
  `ParseChunkHeader` errors; `gui/md1_gather.go:31-33` maps that to
  `gatherIgnored`; `:121-122` prints `"Not an md1 descriptor chunk."` The claim
  that this is a false statement about a well-formed constellation plate is
  right, and so is the follow-on that the host names the version while the
  device does not.
- Row 2 mechanism: `md_codec::decode_with_correction`
  (`crates/md-codec/src/chunk.rs:531-666`) completes the whole BCH loop —
  correcting, re-verifying, and building `corrected_strings` + `all_details` —
  and **only then** calls `reassemble` / `decode_md1_string`, which is where the
  version is enforced. `crates/md-cli/src/cmd/repair.rs:88-96` is exactly the
  match block, and `:94` is `return Ok(2)` in the `Err` arm, with the module doc
  at `:10-12` recording that **no** stdout is emitted in that branch. So a
  successful correction is genuinely discarded and the exit code genuinely
  collapses into the uncorrectable case. Only the quoted doc sentence is wrong
  (M-1).
- Row 3: `crates/md-codec/src/error.rs:33` is literally
  `#[error("wire-format version mismatch: got {got}, expected 4")]`. "expected
  4" does become false once `{4, 8}` is accepted, and the message is what
  `repair.rs:93` prints to stderr, so the fix has a second consumer.

**§9 stage 4a's premise — accurate.** `crates/me-cli/Cargo.toml:26` is
`md-codec = "0.42"`, and `Cargo.lock:547-550` resolves `md-codec 0.42.0` from
`registry+https://github.com/rust-lang/crates.io-index` with a checksum, while
the in-tree md-codec is 0.45.1. The direction of the fix is also right, and the
repo already does it: `ms-codec` and `mt-codec` are both pinned git revs in this
same tree. One wording note — "workspace" is not available here
(`Cargo.toml:3` members are `crates/me-cli` and `crates/mnemonic-io-lib`;
md-codec lives in another repo), so the answer is a pinned **git rev**, matching
the existing pattern. `cargo publish` is not an obstacle: `me` already depends
on `mnemonic-io-lib` by bare path with no version, so it is not publishable to
crates.io today and is shipped as release binaries. `me`'s own source compiles
clean against 0.45.1 as it stands (only §3f will break it — I-5).

**§7a.3's new scope sentence — SOUND.** The refusal-not-fallback structure does
ship: `complexAddressDeriver` probes `if _, err := src(0, false); err != nil {
return nil, false }` (`gui/policy_address.go:188-190`) and every screen reaches
it through `complexAddressSource` (`:88`, doc at `:93-94`). And the carve-out it
protects is real, not hypothetical: D3 is `gui/policy_address.go:97`
(`return nil, false // template-only (D3): nothing to derive from`) and
`gui/wallet_policy.go:198-201` (*"Skipping the gather is still valid and still
reaches consent — without address proof — which is D3's second half and shipped
first"*), and D4 is `gui/wallet_policy.go:226` (*"A keyless template is
engravable on purpose (D4)"*). A blanket "no address, no engrave" really would
retire both. The sentence's scoping — "a policy whose internal-key kind *this
firmware cannot derive*" — is the correct discriminator, and it composes
correctly with §7a's table: the probe catches naive-port row 2 (`byIndex`
misses → error → `nil, false`) but *cannot* catch row 1 (the NUMS fallback
derives successfully, at the wrong wallet), which is precisely why item 3's
"never fall back to the NUMS branch" is stated as a separate rule rather than
left to the probe. Nothing missing.

---

## out of scope, noticed anyway

§9 stage 4 schedules §0b's screen on the device, but the capability it needs —
the fork's Go **compose** side emitting kind 1, as opposed to `EmitTapLeavesChunks`
decoding it — is named in no stage row; stage 3's "Go port in the fork's `md/`"
plausibly covers it, but it is not written down.

---

*Method note: all measurements were run against the repos at the SHAs named at
the top. Two temporary Go test files were added to the fork's `gui` package to
run the predicate probes and were deleted afterwards
(`git status --porcelain` clean). `mnemonic-engrave`'s `Cargo.toml`,
`crates/me-cli/Cargo.toml` and `Cargo.lock` were modified for the build probe
and restored byte-exact, verified by `md5sum` and `git status --porcelain`.*
