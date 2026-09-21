# F-449 SPEC r2 — journey walk (four journeys)

**VERDICT: 2 Critical / 6 Important / 4 Minor**

Artifact: `/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_liana_unspendable_internal_key.md` (r2)
Ground: `descriptor-mnemonic 6cbd49d8`, `seedhammer 7b6f2fb`, `mnemonic-engrave 49964db1`.
`md 0.17.0`, `me 0.10.0` installed and RUN. Go probe run against `seedhammer/gui` with
`/scratch/code/shibboleth/.toolchain/go`.

Findings are **moments**, not sections. Everything in the brief's FACTS-ALREADY-SETTLED list
(wire version 8, §2's recipe, the three out-of-scope presets, §7a's device-derivation gap,
§4a's per-surface split) is taken as given and is not re-reported. Where a settled fold fails
to cover a moment, that is said explicitly.

---

## J1 — the happy path: three hardware wallets, 2-of-3 now, 1-of-2 after a year

### Step 1. The operator opens the composer and picks `tr`.

`gui/composer_flow.go:73` → `composerStartStep` → wrapper picker. Walks clean.

### Step 2. "Start from?" → preset.

**Q1, what do they have in hand:** an intent — "2-of-3 now, 1-of-2 after a year".
**Q2, what does the thing do:** offers six archetypes (`gui/composer_presets.go:91-99`).
**Q3, what else:** the brief's operator picks `kofn-recovery`. RUN:

```
$ md compose --wrapper tr --preset kofn-recovery,2of3,1of2,older=52560
md: preset kofn-recovery needs exactly 1 <k>of<n> parameter, got 2
```

`kofn-recovery` gives a **single** recovery key, not 1-of-2; the stated intent is
`tiered-recovery,2of3,1of2` and needs **five** slots, not three hardware wallets.
**Classification: not our concern.** Pre-existing composer sizing, unchanged by this spec,
and the `slots: N / keys available: M` live line (`composer_flow.go:66-69`) already answers it.
Recorded so the walk is honest about where it diverged from the brief; the rest of J1 is
walked as `kofn-recovery,2of3,older=52560` (4 slots).

### Step 3. The §0 choice screen. → **C1, I1, I2, I3**

This is where the walk stops being clean, and it stops four separate ways.

**Q1:** the operator has a chosen preset and a timelock and nothing else.
**Q2:** the spec says a screen fires. That is the whole of what it says.
**Q3:** see C1/I1/I2/I3 below.

### Step 4. Template chunks → the stub screen → seating → the keyed stub screen.

`composerTemplateChunksFor` → `composerStubFlow` → `composerSeatingStep` →
`composerArtifactsFor` → `composerStubFlow` again (`gui/composer_flow.go:97-152`).
The stub screen is the one the operator **copies onto steel** and mints cosigner cards from.
Its Template-ID/`mk1 stub` is a function of the tree — and therefore of the kind. See I1.

### Step 5. Consent.

`composerConsentLinesFor` (`gui/composer_consent.go:157`) prints branch lines, the key-path
line, `FormAwareIdChunks` + `FormAwareStubChunks`, then addresses.
The key-path arm and `policySummaryLines` are §7's fable M-3 — settled, not re-reported.
The **id and stub lines** are not covered by anything in the spec. See **C2**.

### Step 6. Engrave, then verify.

`multisig_verify.go:834` compares read-back md1 to engraved md1 byte-for-byte. Clean.

### Step 7. Get the descriptor into Liana.

RUN, keyed `kofn-recovery` over the evidence's four xpubs:

```
$ md encode --key @0=… --key @1=… --key @2=… --key @3=… … | wc -l
8
$ md descriptor <8 chunk strings>
tr(50929b74…,{multi_a(2,[73c5da0a/48'/0'/0'/3']xpub6DXuQ…,…),and_v(v:pk(…),older(26280))})#xnta28tv
```

Eight plates, and `md descriptor` does emit the checksum §8.2 requires — so that gate is
runnable. The last mile itself is undescribed: see **M4**.

**J1 verdict: does not walk clean.** Steps 1, 2, 4, 6, 7 are fine. Step 3 is unspecified in
four independent ways and step 5 shows two lines the spec never rules on.

---

## J2 — the operator who does not know what an unspendable xpub is

### "What does the screen say?"

Nothing. The spec contains the string "choice screen" **three times** — lines 30, 491, 549 —
and never gives a title, a lead, a row label, a default row, or a gate on any of them.
Compare the treatment every other operator-facing artifact in the same spec gets: §4a's
`md encode` refusal is specified down to *"a message naming the two spellings that do
work"*; §7 schedules F-633's copy fix and says what it must additionally say; §6's rows each
name their outcome. The **one screen that decides which of two wallets is cut into steel** is
the only one with no copy requirement. See **I3**.

### "What happens if they pick wrong?"

Before engraving: `ChoiceScreen.Choose` returns on Back, so the operator can re-walk —
*if* the screen is re-entered, which is I2's question. After engraving: the plates carry one
kind; the other kind is a different wallet with different addresses, a different
`WalletPolicyId` and a different 12-word phrase (§3e). Not recoverable; re-cut.
**The spec does not say the choice is revisitable**, and the composer's documented Back
invariant is *"going back should lose nothing"* (`gui/composer_flow.go:88-91`).

### "Can they find out later which they picked?"

- **Host:** yes. `md decode` prints the template, which at kind 1 is `UNSPENDABLE(liana)`
  (§4 row 2), and `md decode --json` exposes `descriptor.tree.body.data.is_nums` (RUN,
  confirmed present at kind 0) which §4a versions to a third state. Clean.
- **Device:** no. See **M1** — the inspect screen the operator actually reaches does not
  switch on `KeyPath` at all, so §7's two-print-site enumeration does not reach it.
- **Device, indirectly:** the `Policy id:` line (`policyIDHeader`) *would* distinguish them —
  and does not, because of **C2**.

### Which policies does the screen even fire on? — **MEASURED, and inverted**

I ran `composerLianaOutsideModelClass` directly against real decoded shapes
(probe test in `seedhammer/gui`, since deleted; chunked md1 built with
`md encode --force-chunked`):

```
simple-timelocked-inheritance-tr (REAL internal key @0)
    KeyPath=2 (KeyPathSpendable)  class=""              -> §0 choice screen fires: true
kofn-recovery-tr (NUMS internal key)
    KeyPath=1 (KeyPathNUMS)       class="NUMS key path" -> §0 choice screen fires: false
```

and `md compose --wrapper tr --preset … --json` confirms the precondition:

```
simple-timelocked-inheritance,older=52560   internal_key_path: 0      (a real internal key)
kofn-recovery,2of3,older=52560              internal_key_path: null   (NUMS)
tiered-recovery,2of3,1of2,older=52560       internal_key_path: null   (NUMS)
```

See **C1**.

**J2 verdict: does not walk clean.** The host half of "which did I pick" is answered; the
device half is not, and the screen the journey is *about* is unspecified and fires on the
wrong preset.

---

## J3 — the scratched plate, a year later

### Step 1. The operator has eight kind-1 plates. One is scratched.

**Q1:** eight md1 chunk strings, one with damaged characters.
**Q2:** they run `md repair`.
**Q3:** what if their `md` predates this cycle? → **I4**.

### Step 2. `md repair` on the set.

Read: `crates/md-cli/src/cmd/repair.rs:88-96`. The BCH correction loop runs **first** and
fills `corrected_strings`; only then does `decode_with_correction`
(`crates/md-codec/src/chunk.rs:642-660`) call `decode_md1_string` / `reassemble`, which is
where `Header::read` rejects version 8. On that `Err` the CLI prints to stderr and
`return Ok(2)` — **discarding the successful correction**, and exit 2 is the code `md repair
--help` documents as *"at least one chunk had corrections applied"* → no, as
*"atomic-fail … ANY chunk failing BCH capacity fails the whole call"*. See **I4**.

### Step 3. They check an address on the device before spending.

`gui/md1_gather.go:163 gatheredDescriptorFlow` → `expandUnsupported` →
`complexAddressSource` → `md1PolicyFlow(ctx, th, tpl, policyIDHeader(collected), at)`.
The screen shows `policyIDHeader` + `md1Summary(tpl)` + an Addresses button.

- The address itself is §7a.2's third branch — settled, not re-reported.
- The **refusal** path when the device cannot derive a kind: `complexAddressSource`'s probe
  `if _, err := src(0, false); err != nil { return nil, false }` already refuses to show an
  address and does not fall back to NUMS, so §7a.3's first two clauses are satisfied by
  shipped plumbing. Its third clause — *"REFUSES to engrave"* — is not. See **M3**.
- `md1Summary` names no internal key. See **M1**.
- `policyIDHeader` → `md.WalletPolicyIdChunks`. See **C2**.

### Step 4. §3c's note, checked.

The brief flags that the device and `md repair` go through the chunked-flag auto-dispatch
while host `md decode` does not. At version 8 the dispatch routes correctly
(`decode.rs:191-193`, first-symbol `0b01000`, bit 0 = 0) — §3c's table is right and the
device/repair asymmetry is **benign at v8**. A first-symbol scratch that flips bit 0
pre-correction mis-dispatches, but identically at v4 and v8; pre-existing,
**not our concern**.

**J3 verdict: does not walk clean** — but the damage is at the *toolchain-version* moment,
not the repair math. Steps 3 and 4 are sound modulo the settled §7a work.

---

## J4 — the mixed fleet

### Step 1. Compose on the new board, carry the plates to the old board.

**Q1:** eight genuine version-8 md1 chunk plates.
**Q2:** they tap them to the old board's NFC reader.
**Q3:** the old board's `md/md.go:302` is `wfRedesignVersion = 4` and
`md/chunk.go:87` refuses any other version with `errWireVersion`.

Traced: `gui/md1_gather.go:33` → `md.ParseChunkHeader(s)` errors → `return gatherIgnored` →
`gui/md1_gather.go:121` → the operator reads **"Not an md1 descriptor chunk."**

That is a false statement about a plate the constellation cut. See **I5**.

Single-string (keyless template) variant: `md.Decode` → `readHeader` → `errWireVersion` →
`gui/gui.go:2784 default:` → **"Can't decode this descriptor."** Non-specific, but not false.

### Step 2. Verify on the old board.

Never reached — step 1 never completes the set.

### Step 3. A cosigner runs an older `md` binary.

`decode_md1_string` → `Header::read` → `Error::WireVersionMismatch { got: 8 }`, whose
Display is `"wire-format version mismatch: got 8, expected 4"`
(`crates/md-codec/src/error.rs:33`). The version number is named, which is what §3d's
*"loud and correctly named"* claims — **true on the host, false on the device** (I5).
The "expected 4" half becomes wrong for the new binary. See **M2**.

### Step 4. The operator builds a payload for the second board with `me`. → **I6**

`me` is the tool in *this repo* that carries md1 to a SeedHammer II, and the spec names it
nowhere (grep for `me-cli|me bundle`: only the baseline SHA on line 13).

- `crates/me-cli/Cargo.toml:26` — `md-codec = "0.42"`; `Cargo.lock` —
  `version = "0.42.0"`, `source = "registry+…crates.io-index"`.
- `crates/me-cli/src/bundle.rs:397` — `reassemble(&refs).map_err(BundleError::SetIncompleteMd)?`
- `crates/me-cli/src/bundle.rs:371` — `if let Ok(d) = decode_md1_string(s) { … }`
- `crates/me-cli/src/sysw/record.rs:251-252`, `sysw/expect.rs:202`

**J4 verdict: does not walk clean.** Step 1 fails with a false message, and step 4's whole
host toolchain is outside the spec's downstream list *and* outside crates.io's reach.

---

# FINDINGS

## C1 — Critical. The §0 choice screen's firing predicate is inverted as written, and still over-fires once corrected. Ungated.

**Moment:** J1 step 3 / J2. The operator has picked `tr` + `kofn-recovery` + a timelock, and
the screen that decides whether their wallet is Liana-importable either appears or does not.

§0 line 30: *"fired **only when** `composerLianaOutsideModelClass` (class 2 skipped for the
new kind) returns `""`."* That is the entire specification of when the screen appears.

**Measured**, calling the shipped function on real decoded shapes:

| preset under `tr` | `shape.KeyPath` | `composerLianaOutsideModelClass` | screen fires? |
| --- | --- | --- | --- |
| `simple-timelocked-inheritance` | `KeyPathSpendable` | `""` | **yes** |
| `kofn-recovery` | `KeyPathNUMS` | `"NUMS key path"` | **no** |

Two defects, not one:

1. **Read literally against the function it names, the rule is inverted.** It fires on the
   one tr preset where the choice is meaningless, and not on the two the cycle exists for.
   The five-word parenthetical "(class 2 skipped for the new kind)" is the only thing
   standing between the sentence and that inversion.
2. **Applying the parenthetical does not fix it.** With class 2 skipped,
   `simple-timelocked-inheritance` still returns `""` (unlocked = 1 from `KeyPathSpendable`,
   one `older`-in-blocks lock, no hash/`after`/units) and still fires. But that policy has a
   **real** internal key — `md compose … --json` reports `internal_key_path: 0` — where kind 1
   is unrepresentable in §3f's sum type and §6 row 3 rules it a warn-not-ignore no-op.
   The predicate is missing its second conjunct: **the tr internal key must be NUMS today.**
   §0's own prose ("a `tr` policy fell back to NUMS", "5 of 6 presets while the answer
   matters on 2") carries that precondition and the predicate dropped it.

**Why this is Critical and not Important.** §0's headline measured claim — *"the Liana-importable
`tr` preset set goes from 1 of 6 to 3 of 6"* — is delivered **entirely** by this screen firing
on `kofn-recovery` and `tiered-recovery`. It is an unmet guarantee, and §9 stage 4's gate
(*"§8.3's device leg, §7's constructed shape, and an address test for a kind the device cannot
derive"*) **contains no test of the firing predicate at all**, so nothing would catch it.
A gate that has never been run against the thing it exists to protect is a hypothesis.

**Classification: refusal** — the screen must refuse to offer the choice when the internal key
is not NUMS, and §9 stage 4 must gate the predicate on all six `tr` presets (fires on exactly
`kofn-recovery` and `tiered-recovery`).

**Worse than silence?** Yes. Firing on `simple-timelocked-inheritance` asks an operator to
choose on a wallet Liana already imports, where one answer is a no-op and the other has no
representation; not firing on `kofn-recovery` silently deletes the feature.

---

## C2 — Critical. §3e's version-derived identity ruling is stated for Rust only. The Go port's three identical sites are unscheduled and ungated, so kind 0 and kind 1 collide on the device's `Policy id:` line and on the mk1 KEY card stub.

**Moment:** J1 step 5 (consent) and J3 step 3 (inspect, a year later). The operator reads
`Policy id: …` / `mk1 stub (policy): …` and uses it to decide that these plates and these
cosigner cards are the same wallet.

§3e rules: *"the version is DERIVED FROM THE TREE, never a constant. Add
`Descriptor::wire_version()` … and pass it at all three sites"*, and names them —
`encode.rs:181`, `identity.rs:90`, `identity.rs:200`. It rejects the constant-version
alternative in exactly these words: *"two wallets with **different addresses** sharing one
`WalletPolicyId`, one 12-word phrase and one `WalletDescriptorTemplateId`."*

**The Go port has the same three sites, and the spec never mentions them:**

```
md/encode.go:417        writeNode(&w, dc.tree, width)        ← the wire payload
md/template_id.go:53    writeNode(&w, d.tree, width)         ← WalletDescriptorTemplateId
md/walletpolicyid.go:42 writeNode(&treeW, dc.tree, width)    ← WalletPolicyId
```

`func writeNode(w *bitWriter, n node, keyIndexWidth uint8) error` — **no version parameter**,
a 3-for-3 structural mirror of Rust. The consumers are the ones the operator reads:

- `gui/composer_consent.go:216-229` → `md.FormAwareIdChunks` / `FormAwareStubChunks` →
  `WalletPolicyId` → the consent screen's id and **mk1 stub**.
- `gui/md1_gather.go:…` → `policyIDHeader(collected)` → `md.WalletPolicyIdChunks` → the
  `Policy id:` line on the inspect screen — the only line on that screen that could tell a
  kind-1 wallet from a kind-0 one (M1).
- `md/template_id.go:116` → `WalletPolicyIDStub` — the mk1 KEY card's `policy_id_stub`,
  which is what binds a cosigner's key card to a policy.

**Consequence:** an mk1 KEY card minted for the kind-0 wallet seats and verifies against the
kind-1 plates of the same tree, and vice versa. Same stub, different addresses. That is
§3e's own funds-relevant collision, reintroduced on the surface where the plates physically
are.

**And it is not gated.** §9's "Every §8 item has exactly one owning stage" table assigns
§8.5 (identity distinctness) to stage **1b** — Rust. Stage 3's Go gate is *"§8 vectors in Go,
**including §8.4's `ParseChunkHeader`/`Decode` leg**"*; §8.4 is the dispatch round trip, not
identity. By the spec's own one-owning-stage table, the Go half of §8.5 has no owner.

**Classification: default** — the Go identity hashes must take the derived version, never the
constant, exactly as Rust does; plus a scheduling fix (§9 stage 3 names the three Go sites) and
a gate fix (§8.5 gains a Go leg owned by stage 3).

**Worse than silence?** Yes, decisively. A colliding stub is a positive false match on the
screen whose job is to prove two artifacts belong together.

---

## I1 — Important. The choice screen's **placement** in `composerFlow` is unspecified, and §0's own pointer aims the implementer at the last screen before steel.

**Moment:** J1 step 3 vs step 4. The kind changes the tree, therefore the template chunks,
therefore the Template-ID and `mk1 stub` shown on the stub screen — the screen the operator
**copies onto steel and mints cosigner cards from** (`composerStubFlow`,
`gui/composer_flow.go:97-118`).

§0 line 27 says the predicate *"is called only from the consent screen at
`gui/composer_consent.go:261`"*, and §0 line 41 says *"The narrow predicate already ships and
is already called at `gui/composer_consent.go:261`."* An implementer reusing that call site
puts the choice **after** the stub screen, after seating, after cosigner cards may already
exist — and the operator's paper Template-ID is then wrong.

The composer already knows this hazard and has a whole mechanism for it — `composerStubDelta`
and the changed-id banner, whose doc comment reads *"A false statement here is worse than a
missing one: this is the screen whose whole job is to be copied onto steel, and an operator
who has already minted cosigner cards reads that their cards, and other people's, are now
useless."* The spec neither places the screen before `composerTemplateChunksFor` nor requires
a return through `composerStubFlow` + the banner if it sits later.

**Classification: default** — the spec must fix the screen's position (between
`composerShapeFlow` and the first `composerStubFlow`), or require the changed-id path if it
sits after.

**Worse than silence?** Yes — a stale Template-ID copied onto steel, and cosigner cards
minted against it.

---

## I2 — Important. No reset rule. The chosen kind survives a Back-edit into shapes where it is a silent downgrade or a hard refusal, with no screen to unset it.

**Moment:** J1/J2. The operator picks kind 1, presses Back, and edits the policy — the
composer's documented invariant is that Back **loses nothing** (`composer_flow.go:88-91`,
`composerStartStep`'s `fromPaths` leg, `composerSizeAssignments` preserving `st.assigned`).

Three reachable edits, none covered:

| edit | what `composerLianaOutsideModelClass` returns | what kind 1 then means |
| --- | --- | --- |
| lock → time units (`older=Nu`) | `"a lock in time units"` | screen never re-fires; kind 1 sticks. §0's own words: *"different addresses, different `WalletPolicyId` and 12-word phrase, Liana still refusing, Nunchuk acceptance dropping to 1/24"* — a strict downgrade |
| add a hashlock | `"a hash lock"` | same |
| wrapper `tr` → `wsh` | n/a | §6 row 4 **REFUSES** kind 1 nested under `sh`/`wsh`. `composerArtifactsFor` errors → `composerShowRefusal(ctx, th, "Template", err)` — and the choice screen does not fire under `wsh`, so there is **no screen that can clear the flag**. Dead end |

Both possible implementations are bad and the spec picks neither: silently dropping the kind
tells the operator they have a Liana wallet when they do not; keeping it strands them.

**Classification: default** — the spec must state that the kind resets to 0 on any shape or
wrapper edit that moves the policy out of the firing set, and that §8j's edit path carries it.

**Worse than silence?** Yes — a wallet cut at kind 1 that Liana still refuses, for no reason
the operator can see, or a refusal with no exit.

---

## I3 — Important. The screen's copy and its default row are unspecified, and the widget's zero value re-implements the alternative §0 explicitly rejects.

**Moment:** J2's whole question. The operator does not know what an unspendable xpub is and
is being asked to choose one.

`gui/gui.go:1912` — *"`Initial` is the row the highlight starts on. **The ZERO VALUE is row 0**"*,
applied once per screen. So whichever option is listed first is the standing proposal, and the
project's own recorded lesson is that a picker opening on row zero **proposes a setting**.

§0's "Rejected alternative" paragraph rejects *"defaulting the device to kind 1 for every NUMS
`tr`"* on the grounds that it *"would silently change the wallet form for every existing
operator"*. An implementer who lists the new capability first — the natural ordering, since
that is the feature — arrives at the rejected alternative through a widget default. The
composer already has the counter-pattern in tree (`gui/passphrase_flow.go:406`,
`cs.Initial = 1 // preserve a deliberate opt-in across Back`); the spec does not invoke it.

Separately: no title, lead or row labels are specified, and §9 stage 4 gates none of them —
in a spec that specifies §4a's refusal message, §6's six refusal outcomes and §7's F-633 copy
amendment in detail.

**Classification: default (the row) + documentation (the copy)** — `Initial` must be the
kind-0 row, kind 0 must be listed first, and the body must say what the operator is choosing
between in terms of consequence (which coordinators import it, that the addresses and the
12-word phrase differ), not in terms of "unspendable xpub".

**Worse than silence?** Yes for the default row — it re-creates an alternative the spec
rejected by name. Yes for the copy — an operator who cannot read the question cannot answer it,
and the answer is irreversible once cut.

---

## I4 — Important. `md repair` on an older `md` throws away a **successful** BCH correction and exits with the code documented as "the plate is too damaged".

**Moment:** J3 step 2. The operator's sole backup is scratched. This is the moment they find
out whether it is recoverable.

`crates/md-cli/src/cmd/repair.rs:88-96`:

```rust
let (descriptor, details) = match md_codec::decode_with_correction(&str_refs) {
    Ok(t) => t,
    Err(e) => {
        eprintln!("md: repair: {e}");
        return Ok(2);
    }
};
```

and `crates/md-codec/src/chunk.rs:642-660` runs the whole correction loop and populates
`corrected_strings` **before** calling `decode_md1_string` / `reassemble`. So on a version-8
plate read by a pre-cycle `md`:

1. BCH correction **succeeds**.
2. Decode fails with `WireVersionMismatch { got: 8 }`.
3. The corrected string is **discarded** — `md repair --help`: *"NO partial corrected output
   is emitted on stdout"*.
4. Exit **2**, documented as *"atomic-fail per plan §1 D28: ANY chunk failing BCH capacity
   fails the whole call"* — i.e. **"more than 4 errors, unrecoverable"**.

The operator is told their intact plate exceeded error-correction capacity. The true cause
(`wire-format version mismatch: got 8, expected 4`) is on stderr, but the exit code and the
documented meaning of exit 2 say the opposite. The natural response to "this plate cannot be
repaired" for a sole backup is drastic.

This is the same class as the F-557 lesson already in the tree: a refusal that names the wrong
reason. §3c reasons only about whether the *dispatch routes*; §6 row 5 rules only what the
*decoder* returns. Neither asks what `md repair` does, and repair is the verb J3 reaches for.

**Classification: warning** — `md repair` must distinguish "BCH capacity exceeded" from "this
card was written by a newer md"; the latter is not an atomic-fail of the repair and should
emit the corrected strings. §8 gains a vector for it.

**Worse than silence?** Yes — the wrong outcome is the operator believing an intact backup is
dead.

---

## I5 — Important. On a device running the old firmware a genuine version-8 plate is reported as **"Not an md1 descriptor chunk."** §3d's *"loud and correctly named"* is true of Rust only.

**Moment:** J4 step 1. The operator taps a plate the constellation cut onto their second board.

Traced end to end:

```
gui/md1_gather.go:33   h, err := md.ParseChunkHeader(s); if err != nil { return gatherIgnored }
md/chunk.go:87         if version != wfRedesignVersion { return ChunkHeader{}, errWireVersion }
md/md.go:302           const wfRedesignVersion = 4
gui/md1_gather.go:121  case gatherIgnored: msg = "Not an md1 descriptor chunk."
```

§3d says of the v8-to-v4 case: *"A v4 decoder then refuses at `chunk.rs:70` with `got: 8` —
**loud and correctly named**."* §6 row 5 repeats it. That holds for the Rust host, where the
Display carries `got 8`. On the device the version is never surfaced: the chunk is answered
`gatherIgnored`, which the UI renders as a **false statement about a real plate**, identical
to what it says for a random text tag.

The operator's reasonable conclusions from that sentence are all wrong and some are
destructive: the plate was mis-cut, the engraving is bad, the NFC read failed, re-cut it.

(The single-string arm is merely vague — `gui/gui.go:2784` → *"Can't decode this descriptor."*
— not false. Worth fixing in the same change, not a separate finding.)

**Classification: warning** — `gatherIgnored` must be split so a well-formed md1 at an
unknown wire version gets its own message naming the version and the remedy ("this card was
written by a newer SeedHammer; update this device"). §9 stage 3 or 4 owns it, and it must be
gated, because the *old* firmware cannot be changed — the message on the **new** firmware is
what protects the next mixed fleet.

**Worse than silence?** Yes — a false diagnosis pointing at the plate instead of the reader.

---

## I6 — Important. `me` is absent from §9's downstream list, pins `md-codec 0.42` from crates.io, and cannot be bumped there because md-codec is unpublishable.

**Moment:** J4 step 4. The operator uses `me bundle` / `me sysw` — the tools in *this repo* —
to check their plates and build a payload for the second board.

§9's closing paragraphs enumerate downstream consumers deliberately: `md-cli`'s
`= "0.45.1"` pin must move in the same commit; `mnemonic-toolkit` *"is in scope for this cycle
and gets its pin bump and a golden refresh after stage 2"*. `me` appears nowhere (the only
`mnemonic-engrave` mention in the spec is the baseline SHA on line 13).

Measured:

- `crates/me-cli/Cargo.toml:26` → `md-codec = "0.42"`; `Cargo.lock` → `version = "0.42.0"`,
  `source = "registry+https://github.com/rust-lang/crates.io-index"`.
- In-tree md-codec is **0.45.1** and its workspace pins `miniscript` to a **git rev**
  (`descriptor-mnemonic/Cargo.toml:44`). That is precisely why `mnemonic-toolkit` pins by git
  and records *"md-codec is unpublishable while it depends on miniscript master"*. So `me`
  sits on the last publishable version and **a version bump is not available** — `me` must
  move to a git pin, which is a different piece of work from the one-line bump §9 schedules
  for the other two.

Two operator-visible arms, and the second is fail-**open**:

| surface | code | kind-1 outcome |
| --- | --- | --- |
| chunked set (the 8-plate `kofn-recovery` wallet) | `bundle.rs:397` `reassemble(&refs).map_err(BundleError::SetIncompleteMd)?` | **`me bundle` fails**, naming the chunk-set-id, telling an operator holding all eight correct plates that **the set is incomplete**. Same wrong-reason class as I4 |
| single-string (a keyless kind-1 template plate) | `bundle.rs:371` `if let Ok(d) = decode_md1_string(s) { … }` | decode silently skipped; `key_slots` stays 0, `keyless_template` stays false, `hashlock_kinds` stays empty — and the checklist's *"backup needs N plates"* **completeness claim** is then computed from a policy `me` never read. F-557's own comment at that site calls this shape *"a completeness claim with a hole in it"* |
| `me sysw` | `sysw/record.rs:251-252` → `sysw/expect.rs:202` | `--expect descriptor` reports the record broken |

The plain converter (`me < md1`) is unaffected — `validate.rs:98` only calls
`codex32::unwrap_string`, which is version-agnostic.

**Classification: refusal (the chunked arm's message) + scheduling** — §9 gains an `me` entry
stating the git-pin route, and the single-string arm must not silently skip a decode that
feeds a completeness claim.

**Worse than silence?** Yes for both arms: one says an intact eight-plate backup is
incomplete; the other emits a completeness verdict it did not compute.

---

## M1 — Minor. The device inspect screen never names the internal-key kind, and §7's print-site enumeration cannot reach it.

**Moment:** J2's "can they find out later", asked on the device in J3 step 3.

§7 names two print sites because both **switch on `KeyPath`**:
`gui/template_engrave.go:159-164` and `gui/composer_consent.go:205-214`. Confirmed complete —
those are the only two `switch shape.KeyPath` in non-test `gui/`, `md/`, `sysw/`.

But the screen J3 actually lands on renders `md1Summary(tpl)` (`gui/md1_gather.go:163` →
`md1PolicyFlow`), which does not switch on `KeyPath` at all: it prints
`"Type: " + scriptName(tpl) + " " + policyLine(tpl)` and the `@N` rows. For a taptree
`policyLine` returns `"complex"`. **Kind 0 and kind 1 of the same tree render identically.**

Before this cycle one tree was one wallet, so the screen was complete; this cycle makes it
insufficient — a diff falsifying text it never touches. Minor rather than Important only
because `policyIDHeader`'s `Policy id:` line sits directly above it and *would* distinguish
them — **conditional on C2 being fixed**.

**Classification: documentation** — the spec's print-site list should name `md1Summary` as a
third surface, or state explicitly that the `Policy id:` line is the intended distinguisher
there (which makes C2 load-bearing for it).

---

## M2 — Minor. `WireVersionMismatch`'s message hardcodes "expected 4" and becomes false the moment §3d admits `{4, 8}`.

`crates/md-codec/src/error.rs:33`:
`#[error("wire-format version mismatch: got {got}, expected 4")]`

§3d rules *"The decoder accepts `{4, 8}` and rejects everything else with
`WireVersionMismatch`."* After that, the new binary meeting a version-6 or version-12 string
prints "expected 4" — and this string is the **only** thing an operator or a cosigner sees at
that moment (J4 step 3; also the stderr line in I4). The spec changes the accepted set and
never touches the sentence that states it.

**Classification: documentation.** Reported despite being thin because it is the one text at
a moment two findings already pass through.

---

## M3 — Minor. §7a.3's *"REFUSES to engrave"* is unscoped against the shipped D3/D4 behaviour.

§7a.3: *"If the device meets an internal-key kind it cannot derive, it REFUSES to show an
address and REFUSES to engrave."*

The first clause is already satisfied by shipped plumbing (`complexAddressSource`'s
`src(0,false)` probe returning `nil, false`). The second is new, and it cuts across a
documented shipped rule: `gui/composer_consent.go` D4 — *"ADDRESSES, or a line saying plainly
why there are none … Never silence"* — under which a policy the device cannot address is
consented to and engraved **today**, with a line saying why. Template-only cards (D3) are the
same. A literal reading of §7a.3 regresses both.

**Classification: refusal, needing a scope sentence** — the engrave refusal applies to an
*unrecognised internal-key kind* specifically, not to every policy without an address source.

---

## M4 — Minor. The last mile of §0's promise — plates to a Liana import — is nowhere described.

**Moment:** J1 step 7. §0 promises *"so Liana imports it"* and §9 stage 2 owns `md descriptor`
kind 1, but no step says the operator carries the wallet off the device by transcribing the
md1 strings into a host `md descriptor`.

Measured: the keyed `kofn-recovery` wallet is **8 chunks** (RUN), and `md descriptor` does
emit the checksum §8.2 gates on (`#xnta28tv`, RUN). So the path works — it is just eight
hand-transcribed strings, and nothing in the spec says so.

**Classification: documentation only.** Reported at Minor because nothing goes wrong and
telling the operator nothing is not worse than the alternative — but §0's headline is an
*importability* claim and the spec never describes the route by which an import happens.

---

# WHAT WALKED CLEAN

Stated positively, because a clean walk is a result:

- **§2's leaf order is safe against the one thing that could break it.** §2 asserts descriptor
  left-to-right order **is** wire/slot order. I tried to falsify it two ways. A slot repeated
  across two leaves is **refused** by `md encode` (RUN: BIP-388 disjointness, *"@0 appears at
  2 use sites … forbidden by BIP 388's disjointness rule"*), so "not deduplicated" can never
  bite. And `md encode` **renumbers** `@i` by first appearance — a template written with the
  recovery leaf first as `@1` decodes back as `@0` (RUN) — so ascending slot index always
  equals leaf order. The equivalence holds.
- **`multi_a` is not sorted.** `address/taproot_script_path.go:262-270` sorts only when
  `sorted` is set, so §6's `sortedmulti_a` refusal is a genuine belt and the `multi_a` path
  the presets actually emit is not exposed to fable M-8.
- **§8.3's "the evidence's recorded Liana addresses" leg has data.** The
  `liana-unspendable-xpub` rows in
  `design/evidence/composer-fable-r0/fable-liana-parse-out-v15.jsonl` carry `change` and
  receive address arrays alongside `liana_desc` and its `#8jc8gq6v` checksum. That gate can run.
- **§7's class-2 / unlocked-path ruling lands correctly on the shipped code.**
  `composer_consent.go:407` is `if shape.KeyPath == md.KeyPathSpendable { unlocked++ }`, so a
  fourth enum value is excluded from `unlocked` by construction and §7's constructed failure
  shape (`tr(<derived xpub>, and_v(v:pk(@0),older(26280)))` → class 7) fires as specified.
- **§3c's dispatch reasoning survives the device/repair asymmetry.** At version 8 the first
  symbol is `0b01000`, bit 0 clear, and both `decode.rs:191-193` and `md/chunk.go:193` route
  to the single-payload reader. The residual first-symbol-scratch hazard is identical at v4.
- **J1 steps 1, 4, 6.** Wrapper pick, the stub/seating/artifact loop, and
  `multisig_verify.go:834`'s exact-chunk-equality read-back all carry kind 1 with no change.

---

# METHOD NOTE

Every divergence below the reporting bar was dropped under the worse-than-silence rule, and
they are listed so the bar is auditable: `kofn-recovery` not accepting a 1-of-2 recovery tier
(pre-existing composer sizing, and the live slot-count line already answers it); the
first-symbol-scratch mis-dispatch (identical at v4, pre-existing); `PolicyShapeChunks`
rejecting a non-chunked md1 (an internal API shape, no operator surface); `me`'s plain
converter being version-agnostic (correct, and a positive); BIP-68's 16-bit ceiling on
`older` (52560 is well inside it).

The probe test used for C1/J2 was written to `seedhammer/gui/zz_journey_probe_test.go`, run,
and **deleted**; `git status` in the fork is clean apart from the pre-existing
`.claude/worktrees/`.
