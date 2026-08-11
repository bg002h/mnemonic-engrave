# R0 review — §10.2.1a `ms1` records MUST be engraveable to be admitted

- **Artifact:** `design/SPEC_encrypted_payload_delivery.md` @ `f03a2bb` (new `#### 10.2.1a`, `§2.2 item 15`, amended `§10.2.1` encrypted-section table row)
- **Reviewer:** independent architect, round 0. Author ≠ reviewer.
- **Scope:** the amendment only. Not a fresh audit of the spec, codec, or crypto.
- **Trees read:** `/scratch/code/shibboleth/seedhammer-b2b` @ `75233b8` (branch `b2b`), `crates/me-cli` in `mnemonic-engrave`.

## VERDICT

**NOT GREEN — 0 Critical, 5 Important, 4 Minor, 1 Nit.**

The **boundary itself is exactly right and I re-derived it independently.** What
blocks is not the number: it is (a) the rule's stated security rationale, which
is false against the code, (b) the operator-visible failure it produces, which
the spec's own §6.4 forbids, (c) an unpinned placement that has a seed-leaking
trap in it, (d) the literal `90` having no mechanical tie to the thing it was
derived from, and (e) two "nothing generates this" claims refuted by running the
fork's own tool.

---

## WHAT I MEASURED (independent re-derivation, not a re-read of the brief)

All numbers below are from code I executed in the `b2b` tree under the project
Go toolchain, not from the commit message.

**1. The QR boundary is exactly 90/91.** `qr.Encode(strings.ToUpper(s), qr.M)`,
synthetic uppercase-bech32 strings, len 20→200:

| len | 38 | 39–61 | 62–90 | 91–122 | 123+ |
| --- | --- | --- | --- | --- | --- |
| QR size | 25 | 29 | **33** | **37** | **41** |

`backup.EngraveSeedString` refuses `qrc.Size > 33` (`backup/backup.go:126-131`).
So 90 cuts, 91 does not. **Confirmed.**

**2. The boundary is content-independent for real records.** QR size is not a
pure function of length in general — a *pure-digit* 91-char string encodes to
size 29 (numeric mode). But every codex32 string carries the `MS1` prefix, and
the encoder does not do mixed-mode segmentation: `"MS10TESTS" + 82×"0"` at len 91
still yields **37**. Uppercased bech32 is entirely inside the QR alphanumeric
charset, so byte mode is unreachable and alphanumeric is the density *ceiling*.
**Therefore no ≤90 record can exceed size 33, and no ≥91 record can come under
it.** The rule is tight in both directions.

**3. End-to-end through the real engrave path.** Built a valid codex32 via
`codex32.NewSeed("ms", 0, "test", 'S', data)` for every payload size 1..80 B,
re-validated each through `codex32.New` (the admission gate), and called
`backup.EngraveSeedString` + `engrave.PlanEngraving`:

- Constructible lengths: 48, 50, 51, … 88, **90, 91, 93**, then **125, 127**.
  (**92 is not constructible** — 70 payload chars × 5 = 350 bits, `350 % 8 = 6 > 4`
  → `errIncompleteGroup`.)
- **Every length ≤ 90 planned successfully. 91, 93, 125, 127 all returned
  `seed too long to engrave QR`.** No panics. No text-layout failure anywhere:
  column 1 alone holds 16 groups of 10 = 160 chars, so the QR is the sole binding
  constraint at 90.

**4. The BIP-39 gap is correct.** `bip39.Parse` hard-caps at 24 words
(`bip39/bip39.go:290`) and requires a valid checksum, so 12/15/18/21/24 only.
Measured SeedQR sizes through `engraveSeed` → `backup.EngraveSeed`:

| words | 12 | 15 | 18 | 21 | 24 |
| --- | --- | --- | --- | --- | --- |
| QR size | 25 | 25 | 29 | 29 | **29** |

All ≤ 33, all ≤ `engrave.ConstantQR`'s own 37 cap, `engraveErr = nil` throughout.
**The mnemonic plate has no analogous overhang.** The gap is right.

**5. The `mdmkText` gap is also correct, for a different reason.** Encrypted
md1/mk1 go through `unlockEngraveFlow` → `validateMdmk`, which has its own
`"This record does not fit any plate size."` path — but they are *not seed*, and
under §10.2.2 they are offered only after every secret is already wiped. A
per-record engrave failure there exposes nothing. **The spec does not say this,
which is Minor-4 below.**

---

## IMPORTANT

### I1 — The rule's central justification is false. Admission of an encrypted record happens *after* the KDF and *after* the seed is in SRAM.

**Exact spec text (§10.2.1a):**

> **Why at admission and not at the engraver.** Refusing at the engraver means
> the operator has already typed the passphrase, run a ~31-second KDF, and had
> the seed decrypted into SRAM — where §2.2 item 9 says it is readable over SWD
> — before learning the machine cannot cut it. Refusing at admission means the
> payload fails **before any key is derived and before any secret is resident.**
> The failure is identical; only the exposure differs.

**And §2.2 item 15, operator-facing** (§2.2's preamble at line 48: *"This list is
normative and belongs in operator documentation, not only here"*):

> §10.2.1a rejects both at admission, so the operator is told **before the KDF
> runs** rather than after the seed is decrypted into SRAM.

**Both emphasised clauses are false.** An `ms1` only ever legitimately lives in
the **encrypted** section (§10.2.1's table permits `mdmkText` only in the public
one). The device cannot see an encrypted record until it has decrypted it. Traced:

```
seal/unlock_key.go:81   plaintext, err := Open(key, h.IV[:], blob[:split], blob[split:end])
seal/unlock_key.go:87   defer clear(plaintext)
seal/unlock_key.go:102  admitted, err := AdmitSection(recs, SectionEncrypted)
```

`key` is the ~31 s KDF's output. `seal/record.go:139` says it in the code's own
words: *"Classify runs from AdmitSection(recs, SectionEncrypted), i.e. from
inside UnlockWithKey."* So at the instant the length check fires: passphrase
typed ✓, KDF run ✓, seed decrypted into SRAM ✓ — and `AdmitSection` has already
`append`-copied every preceding record out of `plaintext` into `out`.

**Concrete scenario.** Operator seals a 2-of-3 with a 127-char master-seed `ms1`.
At the machine they tap Unlock, type twelve words, watch a 31-second progress
bar, the GCM tag verifies, the seed is decrypted — *and only then* does the
length check reject it. They read §2.2 item 15 beforehand and believed the
machine would screen this out before asking for a passphrase. It cannot.

**What the rule actually buys** (and it is worth having): the residency window
collapses from *"the whole session until the operator taps that plate"* to
*"the duration of `AdmitSection`"*, it is covered by `wipe(out)` +
`defer clear(plaintext)`, and no plate list is ever built. That is a real gain —
roughly minutes-to-hours of SRAM residency removed — just not the gain claimed.

**Not Critical** because the device's behaviour is safe and the residency is
already covered by §2.2 item 9 and §10.2.4's timer; the defect is in the record,
not the machine. But it is an unsound assumption load-bearing for the design
decision, and a false statement in a list that ships to operators.

**Smallest fix.** Replace the second sentence of "Why at admission" with what is
true: *"Refusing at admission does not avoid the KDF — an encrypted record cannot
be classified before it is decrypted — but it collapses the seed's residency from
the whole session to the duration of `AdmitSection`, which then wipes every copy
it made, and no plate list is ever built."* And in §2.2 item 15 replace *"the
operator is told before the KDF runs"* with *"`me seal` refuses to seal one, so
the operator is told on their workstation; on the device the payload is refused
at admission, immediately after the KDF, rather than at the plate."*

---

### I2 — The rejection surfaces as "Payload unreadable.", which §6.4 forbids for exactly this class of failure.

**Exact spec text (§10.2.1a):**

> A `codex32` secret record longer than **90 characters** MUST be rejected at
> admission, **with the same fail-closed treatment as any other disallowed
> classification.**

§10.2.1 defines that treatment: *"Every other classification … MUST be treated as
'payload unreadable'."* Traced to the device: `AdmitSection` returns
`ErrRecordNotPermitted` (`seal/record.go:32`), which reaches
`gui/unlock_kdf.go:434` and lands in `default:` → `showError(…, "Payload
unreadable.")` → `return false`, dropping out of the retry loop.

**Concrete scenario.** The passphrase was *correct* — the GCM tag verified, which
is the one thing the operator can positively conclude. One record is 127 chars.
The machine says **"Payload unreadable."** §2.2 item 4 has trained this operator
that the blob is attacker-writable and that "unreadable" means tampering. They
will retype twelve words, re-run the 31-second KDF, retype again, then conclude
their sealed backup was altered and go hunting a compromise that did not happen.

**This is strictly worse than the status quo the amendment replaces.** Today the
same payload is admitted and fails at the plate with `"This record does not fit
any plate size."` — precise and actionable. The amendment converts a good
diagnostic into the worst one available.

It is also **directly inconsistent with a requirement already in this spec.** The
`ErrTooManyRecords` branch sits four lines above the `default:` in the same
switch, and carries this rationale verbatim (`gui/unlock_kdf.go:429-432`):

> §6.4 requires this be distinguishable from "unreadable": the count is
> authenticated plaintext, so naming it leaks nothing, and conflating a too-large
> wallet with an attack would send the operator chasing a compromise that did not
> happen.

Every clause of that applies unchanged to the `ms1` length: it is authenticated
plaintext, naming it leaks nothing (the operator already knows what they sealed),
and conflating it with an attack does precisely the harm named.

**Smallest fix.** Strike *"with the same fail-closed treatment as any other
disallowed classification"* and replace with: *"The payload is refused whole and
every record wiped, but the rejection MUST be distinguishable from 'payload
unreadable' per §6.4 — e.g. 'This payload holds a codex32 secret longer than 90
characters, which this machine cannot engrave. Nothing was opened.' The length
and the classification are authenticated plaintext; naming them leaks nothing."*

---

### I3 — The spec does not pin *where* in `AdmitSection` the check runs, and the natural slot is the one that does not wipe.

**Exact spec text:** *"with the same fail-closed treatment as any other
disallowed classification"* — which implies routing through `permitted`.

**`permitted` structurally cannot host this check.** Its signature is
`permitted(section Section, c Classification) bool` (`seal/record.go:195`) — it
never sees the record bytes, so it cannot see a length. An implementer must
invent a slot, and `AdmitSection` offers exactly two:

- **inside the per-record loop**, beside pass 2 — both existing failure paths
  there call `wipe(out)` before returning (`seal/record.go:215, 221`); or
- **the encrypted-section post-loop block**, `if section == SectionEncrypted {
  labelEncryptedCards(out) }` (`seal/record.go:264-267`) — which is *the* natural
  home for "the extra rule that applies only to the encrypted section", and which
  **does not wipe**.

That second slot is not hypothetical: the parallel public-section block
immediately above it already has two `return nil, err` paths (`groupRecords`,
`decodePublicSet`) that skip `wipe(out)`. That is currently harmless only because
public records are cleartext — the code says so. Put an `ms1` rejection there and
the same shape leaks.

**Concrete scenario.** A 2-of-3 payload: `ms1`×3, `mk1`×6, `md1`×6. Records 0 and
1 are ordinary 74-char secrets; record 2 is 127 chars. Implemented in the
post-loop block, `out` already holds heap copies of secrets 0 and 1 when the
check fires. `return nil, err` drops them un-zeroed. They are reachable from
neither `Payload.Wipe` (it walks `p.Secret`, never assigned) nor
`RecordsResident()` — so §10.2.4's idle timer reads **false** while two full
seeds sit live on the heap for the rest of the power cycle. This is the exact
defect class the B2a-ii lens-1 review already found once in
`unlockEngraveMnemonic`.

**Smallest fix.** One sentence in §10.2.1a: *"The check runs in `AdmitSection`'s
**per-record pass**, beside the §10.2.1 allow-list — never in the post-loop
section block — and MUST `wipe()` the records already copied before returning,
as the allow-list failure paths do."*

---

### I4 — `90` is a literal with no mechanical tie to the three things it was derived from, and one of them is under active change.

**Exact spec text:** *"the boundary is exact and sits between 90 and 91"*, with
`90` written as a bare literal in the normative Rule sentence.

`90` is the joint output of three independent values in a package the admission
code does not even import (`seal/record.go` does not import `backup`): the ECC
level `qr.M`, the cap `qrc.Size > 33`, and the plate layout. **Measured
sensitivity of the boundary to each:**

| ECC | max len at cap 33 | max len at cap 37 |
| --- | --- | --- |
| `qr.L` | 114 | 154 |
| **`qr.M`** (today) | **90** | 122 |
| `qr.Q` | **67** | 87 |
| `qr.H` | **50** | 64 |

Two concrete, *cheap*, and live changes break it:

**(a) Unsafe direction — re-opens F-113 silently.** O1 hardware legibility is an
open thread on this project. Someone raises the plate QR from `qr.M` to `qr.Q` to
survive brushed steel. The engraver's limit drops to **67 characters** — below
the ordinary 75-char output of `EncodeMS1` on 32 bytes of entropy, i.e. below
*every full-strength seed this constellation produces*. §10.2.1a still says 90,
still admits, and the machine refuses at the plate after the KDF. F-113 is back,
now with a spec that says it cannot happen.

**(b) Over-rejection direction.** `engrave.ConstantQR` already accepts up to dim
**37** (`engrave/engrave.go:420-427`, "the bound is v5 (dim 37)"), and
`bitmapForQRStatic` tabulates 21/25/29/33/**37**. Raising
`EngraveSeedString`'s cap from 33 to 37 is a one-character change with the
machinery already in place; the engraver would then cut to 122 chars and §10.2.1a
would reject the entire 91–93 short band it can now handle.

**Smallest fix — and I do *not* recommend a full derived predicate.** Making the
rule a runtime predicate would force a QR encoder into `me` (the Rust CLI has
none), which is a large cost for a boundary that moves once a decade. Instead,
**keep the literal and pin it with a test**, which costs one function:

> `90` is the value `backup.EngraveSeedString` yields **today**, under `qr.M` and
> the `Size > 33` cap. A test in the fork MUST assert the equivalence directly —
> that a 90-char `codex32` string plans and a 91-char one returns an error
> *through `EngraveSeedString` itself* — so that any change to the ECC level, the
> size cap, or the plate layout **fails the build** rather than silently
> desynchronising this section. Changing the boundary is a spec amendment, not a
> constant edit.

`TestEngraveSeedStringTooLong` / `TestEngraveSeedStringHappy`
(`backup/backup_test.go:417,437`) already cover 93/127/74 — they need the 90 and
91 cases and a comment naming §10.2.1a as what they pin.

---

### I5 — "Nothing in this constellation generates one" and "`NewSeed`'s only non-test caller" are both false. The fork ships a tool that produces the refused string.

**Exact spec text (§10.2.1a):** *"Nothing in this constellation *generates* one:
`codex32.EncodeMS1` caps entropy at BIP-39 lengths (16/20/24/28/32 bytes) and so
tops out at 74 characters, and it is `NewSeed`'s only non-test caller. A long
code can only arrive from third-party BIP-93 tooling."* Repeated in §2.2 item 15:
*"Nothing in this constellation generates such a string; it can only arrive from
third-party BIP-93 tooling."*

**`EncodeMS1` is not `NewSeed`'s only non-test caller.** `cmd/biptool/main.go:312`
(`genSeed`) is the second, and it accepts `-seedlen` anywhere in **[16, 64]**
(`cmd/biptool/main.go:274`), passing the seed straight through with no BIP-39
length restriction. Executed in the `b2b` tree:

```
$ head -c 64 /dev/zero | go run ./cmd/biptool seed -seedlen 64 -id test
ms10testsqqqqqq…qqqmhts8vf8gzxmuxj      # 127 characters
```

That is exactly the 127-char master-seed long code §10.2.1a exists to refuse,
generated by the fork's own tool, with a documented flag, at its default HRP.

**Why this matters beyond the factual error.** The "what this costs" paragraph is
the amendment's whole cost argument, and it rests on the refused string being
producible only by foreign tooling. It is not. An operator following the fork's
own `biptool` can build a payload their own machine refuses — and after I2, all
they are told is "Payload unreadable."

**Smallest fix.** Replace both sentences with: *"The device-facing encoder
(`codex32.EncodeMS1`) cannot produce one — it caps entropy at BIP-39 lengths and
tops out at 75 characters. The repo's host tool `biptool seed` **can**:
`-seedlen` admits 16–64 bytes, and 64 bytes yields a 127-character long code.
That tool, and third-party BIP-93 tooling, are the only sources."* Consider
filing a follow-up to warn in `biptool` when `-seedlen` yields an unengraveable
string.

---

## MINOR

### M1 — `EncodeMS1` tops out at **75**, not 74.

§10.2.1a: *"caps entropy at BIP-39 lengths (16/20/24/28/32 bytes) and so tops out
at 74 characters."* Measured by calling it:

| entropy (B) | 16 | 20 | 24 | 28 | 32 |
| --- | --- | --- | --- | --- | --- |
| `len(EncodeMS1(e))` | 50 | 56 | 62 | 69 | **75** |

`74` is the length of a bare 32-**byte** `NewSeed` payload; `EncodeMS1` prepends
`msPrefixEntr` (`codex32/msencode.go:23`), making the payload 33 bytes → 75
chars. Harmless to the rule (both are under 90) but it is a machine-checkable
number stated wrong in a normative section. **Fix:** `74` → `75`.

### M2 — "version 41 against a hard limit of 33" conflates QR *module count* with QR *version*.

§2.2 item 15. QR versions run 1–40; version *41* does not exist. 41 and 33 are
module dimensions — version 6 and version 4 respectively (`4V + 17`). The fork's
own tests get this right: *"QR dim 41 (V6, unsupported)"*
(`backup/backup_test.go:421`). In operator-facing text an impossible version
number invites the reader to conclude the whole paragraph is unchecked. **Fix:**
*"the plate's QR would be 41 modules across (version 6) against a hard limit of
33 (version 4)."*

### M3 — The prescribed test vectors include one that cannot exercise the rule, and omit the one that can.

§10.2.1a: *"with test vectors at 90/91 and at 124/125/127."*

- **124 cannot reach this rule.** `codex32.New` rejects it as `errInvalidLength`
  before any engraveability question arises (measured: the whole 94–124 band is
  unconstructible). It is a fine test of the length bands; as a §10.2.1a vector
  it is a false-PASS shape — green whether or not the rule exists.
- **93 is missing**, and it is the only other constructible over-90 short-band
  length. (**92 is not constructible** — verified: `350 % 8 = 6 > 4`.) 93 is the
  vector that actually distinguishes this rule from "reject long codes", which is
  the amendment's own stated reason for phrasing it as one length.

**Fix:** vectors at **90 (admit), 91 (reject), 93 (reject), 125 (reject), 127
(reject)**; keep 124 if wanted, but label it as a band test, not a §10.2.1a test.

### M4 — The rule's `ms1`-only scope is correct but unstated, so a later reader will read it as an oversight.

`mdmkText` records have their own engrave-time failure (`unlockEngraveFlow` →
*"This record does not fit any plate size."*), and BIP-39 records are hard-capped
at 24 words. Neither needs an admission rule — the first because md1/mk1 are not
seed and are offered only after every secret is wiped (§10.2.2), the second
because the overhang cannot occur (measured: 24 words → QR 29). **Fix:** add one
sentence saying so, with the 24-word cap cited to `bip39/bip39.go:290`.

### M5 — The Rust home should be named as `record::validate_record`, not `me seal`.

§10.2.1a: *"`me seal` refuses to seal what the device will refuse to admit."*
`me seal` is a *producer*; device admission is a *consumer* check, so as written
the two halves are not the same predicate on the same object and the Rust-primary
binding is loose. The right Rust home is
`crates/me-cli/src/seal/record.rs::validate_record`, whose `Format::Ms` arm
already runs `ms_codec::decode` — a per-record predicate on a record, i.e. the
genuine counterpart of `seal.AdmitSection`'s pass 2. Putting it there also makes
`me hash` inherit it for free (it calls `validate_record` at
`crates/me-cli/src/main.rs:479`); putting it in `run_seal_cli`'s `is_seed`
pre-check would leave `me hash` blessing a record list the device refuses.
**Fix:** name `validate_record` and a new `RecordError` variant explicitly.

---

## NIT

### N1 — The table's first row is not a boundary.

`| 40–62 | 29–33 | cuts |`. Measured, the transitions are at **39** (25→29) and
**62** (29→33); `40` is not a boundary and nothing below 48 is constructible
anyway. Consider `| 48–61 | 29 |` / `| 62–90 | 33 |` so every row edge is a real
transition. (Row 4's `123–127` is fine but only 125 and 127 exist there.)

---

## WHAT I CHECKED AND FOUND SOUND

- **90 is the right boundary**, exactly, in both directions, and is
  content-independent for any real codex32 string. Re-derived end-to-end.
- **Stating it as one length rather than "reject long codes" is correct** — the
  91–93 short-band overhang is real (91 and 93 are constructible; 92 is not).
- **Whole-payload refusal with no partial unlock is right.** Admit-and-skip would
  require holding a decrypted seed the machine can never engrave, resident
  through the plate-list session, for zero benefit — against §10.2.2's
  secrets-first lifecycle. It is also consistent with §6.4's existing fail-closed
  posture, and unlike a silently-dropped record it cannot produce a plate set that
  looks complete (§6.4's "incomplete backup believed complete").
- **No interaction with §10.2.1's group-reassembly requirement** — that is
  public-section-only (`AdmitSection`'s pass 3 is gated on `SectionPublic`), and
  `ms1` is encrypted-section-only. Disjoint.
- **The BIP-39 and `mdmkText` exclusions are correct** (measured; see M4 for the
  documentation gap).
- **The Rust-primary ordering claim is right in substance** — admission is
  normative record-level behaviour and belongs in `me` first (see M5 for the
  placement).

## RECOMMENDATION

Fold I1–I5, fold M1–M5, and re-dispatch a **scoped** re-review: *"did the fold
fix each finding, and did it introduce a new defect"* — not a fresh audit. The
measured facts in this report (boundary 90/91, constructible lengths, ECC table,
`EncodeMS1` → 75, `biptool` → 127) are settled; state them in the brief so they
are not re-derived.

No files were modified other than this report. Scratch test packages created in
`/scratch/code/shibboleth/seedhammer-b2b` during measurement were removed;
`git status` in that tree is clean.
