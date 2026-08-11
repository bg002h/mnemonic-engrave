# SPEC — systemwide payloads: NFC and flash delivery for every program

Status: **DRAFT, pre-R0.** Written 2026-08-11 from a brainstorm with the
operator. No code has been written and none may be until this passes the R0
gate at 0 Critical / 0 Important (project rule: risk-set work, and this is
squarely in it — secrets, admission behaviour, eight programs, two repos).

**Reference convention:** `EPD§` means a section of `SPEC_encrypted_payload_delivery.md`; a bare `§` means a section of THIS
document. The two specs both have a §2.2, a §5.5 and an §8.1, and an
unqualified number would send a reader to the wrong one.

## 0. What this is, in one paragraph

Today a secret reaches a SeedHammer II program in exactly one way: the operator
types it. The one exception is **Sealed Payload**, which reads an encrypted blob
from flash and can do exactly one thing with what it finds — engrave it. This
spec gives every *other* program two more ways to be fed — **NFC** and a **new
flash region** — and deliberately does **not** give them Sealed Payload's wiping
machinery. Sealed Payload itself does not change.

## 1. The operator's decisions, recorded

These were settled in the 2026-08-11 brainstorm. Each is the operator's call;
several overrule a documented prior decision and are marked where they do.

1. **Sealed Payload is frozen.** It keeps its wipes, idle timer and KDF exactly
   as shipped. Its security failures are acknowledged and will be remedied at a
   future date — see the two follow-ups filed with this spec.
2. **The other programs abandon that security model.** No secret-residency
   wipes, no idle wipe, no EPD§10.2.4 clock. This does not overrule EPD§2.2 item 12;
   it *extends* it. That item already says "the machine offers two classes of
   program, and only one of them wipes."
3. **Scope is all eight non-Sealed-Payload programs**: Backup Wallet, BIP-39
   Password, Engrave Text, Account Xpub, Engrave Bundle, Engrave Single-Sig,
   Engrave Multisig, BIP-85 Child Seed.
4. **The systemwide payload lives in its own flash region**, not the Sealed
   Payload region at `0x10E00000`.
5. **Two container variants**: sealed (passphrase + KDF) and plaintext
   (operator-compared hash). The sealed variant unlocks **once per session**.
6. **The plaintext variant may carry secret record classes**, flagged on screen
   at load, paired with an operator-initiated erase.
7. **Verification depth is the operator's choice**, with the full menu, and the
   detection probabilities live in the documentation rather than on the panel.
8. **`me` offers three passphrase modes** — none, user-supplied ASCII, and
   generated N BIP-39 words (`2 ≤ N ≤ 24`, default 12). Not `[cliff]`-above over
   secret content requires an explicit command-line flag.

   **This decision was made, reversed, and remade, and the round trip is worth
   keeping.** R0-C4 found that a user-supplied passphrase had no device entry
   path, so the mode was dropped. The operator then observed that the blocker is
   *which keyboard the unlock flow happens to use*, not a missing capability —
   `NewPassphraseKeyboard` (`gui/passphrase_keyboard.go:76`) is free-text and
   already exists. So the mode is restored, behind a **keyboard choice**.

   **This overrules a normative MUST NOT, and §1's preamble requires that be
   marked (R2-I4).** EPD§2.2 item 1 and EPD§8 forbid a user-supplied
   passphrase, and `crates/me-cli/src/seal/passphrase.rs`'s module doc gives the
   reason: "GENERATED, never user-supplied… a human-chosen passphrase is worth
   25–35 bits — one rented GPU, minutes. `age` reached the same conclusion." The
   operator overrules it deliberately, with §6.1's numbers in view, and §6.2.1
   prices the mode at **0 bits** so nothing downstream mistakes it for
   protection. An earlier fold deleted this acknowledgement; it is restored
   because §1 promises overruled decisions are marked where they occur.

8a. **The operator picks the keyboard at unlock: BIP-39 word, or free-text
    ASCII.** The word keyboard is the default landing, since the generated mode
    is the common case and twelve words are far easier to type with wordlist
    completion. Both normalise through the **same rule** — EPD§8.1's lowercase,
    single-space form — so a word passphrase entered on either keyboard produces
    **byte-identical KDF input**, which is why no header field declares the
    type. (The *rule* is shared; the `seal.NormalisePassphrase` **function** is
    not the required carrier, and R3-I4 is why that distinction is drawn: its
    signature takes and returns a Go `string`, which is exactly the allocation
    §6.2.2's caller-owned buffer and test 21 exist to avoid. Normalise into the
    buffer.) A declared type
    would be an attacker-flippable byte whose only effect is presenting the
    wrong keyboard.

8b. **The checksum gate is PER-INVOCATION, not a property of the keyboard.**
    Operator ruling 2026-08-11, and the root fix for R1-C2. Seed entry keeps the
    gate; **passphrase entry drops it at every length, 2 through 24.** A
    passphrase that happens to satisfy the BIP-39 checksum is fine and nothing
    requires it.

8c. **Variable-length word entry terminates on a `done` key, and the count is
    confirmed before the KDF.** Operator ruling 2026-08-11.

    Seed entry never needed a terminator because N was known; passphrase entry
    has no natural end. The `done` key is a **per-instance opt-in on the
    passphrase instance only**, following the pattern
    `gui/passphrase_keyboard.go:80` already documents for `NewTextKeyboard`'s
    newline key — "a PER-INSTANCE opt-in, not a fourth shared page entry,
    because `PassphraseKeyboard` is also `NewAddressKeyboard` and BIP-85 index
    entry." An unconditional key would appear where it means nothing.

    **The confirmation is the safety, not the key.** Both a misplaced `done`
    press and an accept-on-empty-field would silently truncate the passphrase,
    and the operator would then wait ~31 s for a KDF whose failure is
    indistinguishable from having typed the wrong words. A `N words — unlock?`
    screen makes the truncation visible before it costs anything. Variable
    length introduces a second way to be wrong that looks exactly like the
    first, and this is what separates them.

    Without this the feature is broken at its default: `gui/gui.go:758`'s
    `refreshCands` masks the keyboard to `bip39.LastWordCandidates` the moment
    the cursor reaches the final slot — `gui/unlock_kdf.go:350` calls it "the
    checksum gate" — so **15 of every 16 uniformly generated 12-word passphrases
    would be permanently unopenable** (128 valid last words of 2048). Any N
    divisible by 3 is affected.
9. **Verification is never forced.** The operator may bypass it, or assert a
   by-eye check the device cannot confirm. **Added 2026-08-11 (R0-C3).**

## 2. What already exists, and what genuinely has to be built

The single most important finding of the brainstorm is how much of this is
already specified. Building the wrong thing here would mean re-inventing
controls that exist and are vectored.

### 2.1 Already exists — do not rebuild

| Thing | Where | Note |
| --- | --- | --- |
| An **unsealed** container variant | `seal/wire.go:95`, `Header.Sealed()` | `ct_len == 0` means no encryption, no tag |
| The **operator-compared hash** for that case | EPD§6.6, NORMATIVE | "an out-of-band check the attacker does not control" |
| A **legible display format** for it | EPD§6.6 | first 16 bytes, 8 groups of 4: `a26e d22b b747 dfd0 2367 06ad 14c1 9679` |
| **Downgrade detection** | EPD§6.6's `sealed` byte | stripping a sealed payload to plaintext changes the digest; vectors D and E pin it |
| **Record classification** | `seal/record.go` | `ClassMnemonic`, `ClassCodex32Secret`, `ClassDescriptor`, `ClassMDMK`, `ClassAddress` |
| The **secrecy predicate** | `seal/session.go:17` | `ClassCodex32Secret \|\| ClassMnemonic` |
| **Passphrase generation from a CSPRNG** | `crates/me-cli/src/seal/passphrase.rs` | 12 words / 128 bits, `rand::rng()`, `Zeroizing` |
| **Secrets arriving over NFC** | `gui/scan.go` | already parses `bip39.Parse` and `codex32.New` off a tag |

That last row deserves emphasis because the project's own documents imply
otherwise. **"`ms1` never travels over NFC" was never a property of the
scanner.** The scanner has always accepted it; the rule was enforced by which
flows consume `act.scan`, and `engraveObjectFlow` already has
`case codex32.String`. This spec makes the existing capability deliberate and
routed, rather than latent and undocumented.

### 2.2 Genuinely new

1. A **second flash region** and its container magic (§4).
2. **Widened admission** — a record class for free text, and secret classes
   permitted outside an encrypted section (§5). **Normative → Rust first.**
3. **Routing**: records reaching programs other than the engraver (§3).
4. A **session** that survives across programs until power-off (§3.2).
5. The **verification-depth menu** (§7).
6. **`me` passphrase modes** and the cliff flag (§6).
7. An **NFC source for the emulator**, so the new path is testable (§8.2).
8. **A NEW device unlock flow, and a change to a SHARED one.** R2-I1 asked for
   this and R3-I3 corrected it: forbidding one function is not enough, because
   the two obstacles live in two different places.

   | obstacle | where | what it does |
   | --- | --- | --- |
   | `!m.Valid()` | `gui/unlock_kdf.go:168`, `:359` | rejects every `len(m) % 3 != 0` outright |
   | `refreshCands` → `bip39.LastWordCandidates` | **`gui/gui.go:758`, inside `inputWordsFlow`** | masks the final slot to checksum-valid words |

   So: the systemwide unlock **may not reuse `unlockPassphraseFlow`** (the
   first row), **and `inputWordsFlow` must gain a per-invocation switch for the
   second** — it is shared with five call sites that all still want the mask,
   so it cannot simply be removed. §8b's per-invocation ruling is about
   `refreshCands`, not about `unlockPassphraseFlow`, and naming only the latter
   left the actual blocker in place.
9. **A per-instance `done` key** on the passphrase keyboard instance (§8c).

## 3. Architecture

### 3.1 One shared seam, plus four individual wirings

**MEASURED, after R0-I4 caught the first draft asserting a count its own list
contradicted.** `seedEntryFlow` (`gui/derive_xpub.go:82`) has **7 call sites
across 4 programs**:

| program | sites |
| --- | --- |
| BIP-85 Child Seed | `bip85.go:271` |
| Account Xpub | `derive_xpub.go:107` |
| Engrave Single-Sig | `singlesig.go:33`, `singlesig_verify.go:67` |
| Engrave Multisig | `multisig.go:91`, `multisig_build.go:58`, `multisig_verify.go:50` |

**Two of those 7 sites are verify re-entries, and they are the reason the seam
cannot simply be widened in place (R0-C1).** `seedEntryFlow` is the FIRST
statement of both `singleSigVerifyFlow` and `multisigVerifyFlow`, so a widened
seam offers "Payload" at the verify prompt — and §7.4 forbids exactly that.
A rule stated only in prose is not a mechanism.

**NORMATIVE: there are two entry points, not one flag.**

```
seedEntryFlow(ctx, th)            offers Typed / Scanned / Payload
seedEntryFlowTypedOnly(ctx, th)   offers Typed. No parameter, no source menu.
```

The verify re-entries call the second. A boolean parameter on one function was
rejected deliberately: a boolean can be passed wrongly and the wrong value still
compiles, whereas a verify flow that has no way to name the payload source
cannot reach it by any argument. The test is then structural — no verify flow
mentions `seedEntryFlow` — rather than behavioural, which is what R0-C1 showed
test 1 could not achieve at the store layer alone.

**"One seam, not eight" is therefore false as originally written, and the spec
says so rather than keeping the slogan.** The seam covers 4 of the 8 programs.
The other four do not share a helper at all: `backupWallet` uses `newInputFlow`
(exactly one non-test caller, `gui.go:1704`), and BIP-39 Password, Engrave Text
and Engrave Bundle each have their own entry path.

So the work is **one shared seam plus four individual wirings**, and each of the
four gets its own admission test rather than inheriting one. Pretending
otherwise would have under-scoped the implementation by half the programs.

### 3.2 The session

Unlocking the sealed variant runs the KDF **once**; the decrypted records live
in a session store until power-off. No idle timer, no automatic wipe — per
decision 2.

The store holds **classified records, not a blob**, so a program asks for "a
mnemonic" and cannot be handed a descriptor by accident.

**Every screen that consumes a record names its source** — `from payload`,
`from tag`, `typed`. Provenance must never be something established by reading
code.

The Sealed Payload program has its **own** session semantics and does not share
this store. Two features, two regions, no shared state.

#### 3.2.1 The store — NORMATIVE

```
identity   [32]byte      the §5.4.1 identity digest. NOT the EPD§6.6
                         public-data digest, which does not exist when
                         pub_len == 0 (R1-C3)
compared   bool          the operator compared the digest for THIS identity
sealed     bool          which container variant it came from
weak       bool          sealed, and its passphrase is NOT `[cliff]`-above
                         (§12.1). Named `weak` for brevity only: `[cliff]` is a
                         word count, so `weak == false` does not mean strong
records    []{class, body}   classified at load, never re-sniffed at use
```

One entry, not a list: **the store holds at most one payload.** Loading a second
replaces the first outright. A list would raise "which payload did this record
come from" at every consumption, and that question has exactly one safe answer
when there is only ever one.

**Lifetime is the process.** The store is cleared when the firmware restarts and
at no other time — there is no timer and no idle wipe (decision 2). "Until
power-off" means precisely that: the machine losing power, or `Init()` rebooting
it. No flow clears the store on exit, because a flow that cleared it would
silently reintroduce the per-program KDF that "once per session" exists to
avoid.

**`records` are classified once, at load.** Re-sniffing at the point of use
would let one byte string be admitted as one class and consumed as another.

### 3.3 Admission is one function — the table is NORMATIVE

Today this logic is spread across `engraveObjectFlow`'s type switch and each
flow's private assumptions. Consolidating it is what makes the rest of this
reviewable, and the table below is the consolidation. **An implementer
transcribes it; nothing here is left to be derived.**

**First, a simplification that took a wrong turn to find.** The obvious shape is
a three-axis matrix — `(class, container, program)`. It is the wrong shape. The
container variant does **not** gate admission: decision 6 says the plaintext
variant may carry any class the sealed one may. What the container changes is
whether a **flag** is raised. So:

> **Admission is `(class → program)`. The container variant selects flags, never
> admission.** Two rules, each testable alone, instead of one matrix with a
> redundant axis.

#### 3.3.1 Record classes

| class | secret? | source |
| --- | --- | --- |
| `ClassMnemonic` | **yes** | exists, `seal/record.go` |
| `ClassCodex32Secret` | **yes** | exists |
| `ClassPassphrase` | **yes** | **NEW** (§5.3) |
| `ClassFreeText` | no | **NEW** (§5.3) |
| `ClassDescriptor` | no | exists |
| `ClassMDMK` | no | exists |
| `ClassAddress` | no | exists |

The secret column **extends `seal/session.go:17`**, which today reads
`ClassCodex32Secret || ClassMnemonic`. It becomes those two plus
`ClassPassphrase`. **`ClassFreeText` is NOT secret** even though an operator may
put anything in it — the class states what the format guarantees, not what a
human might do, and a class that claimed secrecy it cannot enforce would be the
same over-claim F-123 was filed against.

#### 3.3.2 The admission table — NORMATIVE

`•` = admitted. Blank = refused with a named reason.

| program | Mnem | Cdx32 | Passph | FreeText | Descr | MDMK | Addr |
| --- | :-: | :-: | :-: | :-: | :-: | :-: | :-: |
| Backup Wallet | • | • | | | | | |
| BIP-39 Password | | | • | | | | |
| Engrave Text | | | | • | | | |
| Account Xpub | • | • | • | | | | |
| Engrave Bundle | | | | | • | • | |
| Engrave Single-Sig | • | • | • | | | • | |
| Engrave Multisig | • | • | • | | • | • | |
| BIP-85 Child Seed | • | • | • | | | | |
| *Sealed Payload* | — | — | — | — | — | — | — |

Rows that need their reason recorded, because a reviewer will otherwise have to
reconstruct it:

- **`ClassPassphrase` is admitted wherever a seed is**, because those programs
  already take an optional BIP-39 passphrase alongside the mnemonic
  (`deriveAccountXpub(m, passphrase, …)` at `gui/derive.go:19`,
  `deriveBip85Child` at `gui/bip85.go:61`, `deriveMultisigLeg` at
  `gui/multisig_derive.go:32`). Admitting it is matching an existing parameter,
  not inventing a capability.
- **Backup Wallet refuses `ClassPassphrase`** because it engraves the mnemonic
  itself, and the passphrase is deliberately never engraved and never in the QR
  — the words and the SeedQR are byte-identical with or without one. A
  passphrase reaching it would have nowhere to go.
- **`ClassMDMK` is admitted to the two multisig-capable programs** for the
  supplied-md1 path they already have (`deriveMultisigLeg`'s `suppliedMd1`).
- **`ClassAddress` is admitted nowhere.** It is consumed only by the
  verify-address flow, which `engraveObjectFlow` deliberately has no case for
  (R0-M5). This spec does not change that.
- **Sealed Payload is dashes, not blanks** — it is out of scope entirely
  (decision 1), not a program whose every cell happens to be refused.

#### 3.3.2a NFC records go through the SAME function — NORMATIVE (R0-I6)

**"Source is not an admission input" is about the TABLE, not about §5.4.1's
`compared` precondition (R1-M1).** The class table is source-blind. The
`compared` gate is a separate, earlier check on the *payload* — it asks whether
this payload was authenticated at all, before any record of it is classified.
An NFC record has no payload and no `compared` gate; it is admitted by class and
flagged by F4.

**The order is NORMATIVE, and §5.4.1 governs (R2-M1): `compared` is checked at
CONSUMPTION**, not before classification. Classification happens once at load
(§3.2.1) and is independent of authentication; what `compared` gates is whether
a classified record may be *handed to a program*. "Before any record of it is
classified" was loose wording here and is withdrawn.

The table's axis is `(class → program)` and says nothing about where a record
came from, which left the NFC path — the one §5.4 just removed all integrity
checking from — outside the single admission function.

**It is not outside it.** An NFC-delivered record is admitted by the same table,
by class, exactly as a payload-delivered one. Source is not an admission input;
it is a **flag** input (F3, and F4 below). One function, every path, no
exceptions — which was the point of §3.3 and which the first draft's tuple
quietly broke.

#### 3.3.3 The flag rules — NORMATIVE

Evaluated after admission, never as part of it. Each is independent; more than
one can fire.

| # | condition | screen says |
| --- | --- | --- |
| F1 | admitted class is secret **and** container is plaintext | this secret is unencrypted in flash; offers erase (§5.5) |
| F2 | admitted class is secret **and** the passphrase is not `[cliff]`-above (§12.1) | this secret is weakly protected |
| F3 | always, for anything not typed | the source, at the point of use (§3.2) |
| F4 | admitted class is secret **and** source is NFC | this secret arrived with **no integrity check at all** — §5.4 scopes digest verification to flash, so nothing stands behind a tag's contents |

## 4. The flash region

**DECIDED 2026-08-11 (operator): `0x10D00000`–`0x10D10000`**, 64 KiB,
16 × 4 KiB sectors. Fixed and normative, for the same reason `PayloadAddr` is:
any other value produces a blob the device never looks at.

Measured constraints, from EPD§3 and EPD§5:

| Fact | Value |
| --- | --- |
| Flash | 16 MB, `0x10000000`–`0x11000000` |
| Firmware image ends | `0x10135300` |
| Sector `picotool load` touches | `0x10136000` |
| Sealed Payload region | `0x10E00000`–`0x10E10000` |
| Must stay clear | top sector `0x10FFF000` (`--abs-block` defaults to `0x10ffff00`) |
| Hard limit | past `0x11000000` a write wraps to `0x10000000` and destroys the firmware (RP2350 datasheet section 5.5.2) |

`0x10D00000` sits a full megabyte below the Sealed Payload region. **The
separation is the point**: adjacent regions mean a length bug in either runs
into the other's data, while a megabyte of unprogrammed flash between them keeps
an overrun local. ~12.8 MB is unallocated, so the clearance costs nothing.

### 4.1 The magic — NORMATIVE

**8 bytes, ASCII, `MNEMSYSW`.** Eight to match `MNEMBLOB` (`seal/wire.go:25`) so
both containers present a same-width discriminator at offset 0, and `SYSW` for
*systemwide*. A reader that finds `MNEMBLOB` at `0x10D00000`, or `MNEMSYSW` at
`0x10E00000`, **refuses** — it does not attempt to parse a container it
recognises at the wrong address. See §8.3 test 7.

**A distinct container magic**, not `MNEMBLOB`. Rationale: the operator froze
Sealed Payload, and widening the format it depends on would unfreeze it through
the back door. A distinct magic also means a blob written to the wrong address
is *rejected* rather than half-understood.

## 5. The container

Reuses EPD§6.6's hash construction and `seal/record.go`'s classification. Its own
admission rules.

### 5.1 Sealed variant

Passphrase + PBKDF2-SHA256 + AES-256-GCM, as `MNEMBLOB`. Unlocked once per
session (§3.2).

### 5.2 Plaintext variant

No encryption, therefore no key, therefore no tag, therefore **no
authentication**. What stands in its place is EPD§6.6's hash: `me` prints it at
creation, the device displays it at load, and **the operator compares**.

**The device never detects a hash mismatch.** It has no idea what the operator
wrote down. The machine's own failures are structural only — wrong magic, bad
lengths, records that do not decode — and they get **named reasons, never the
words "payload unreadable"**, because EPD§2.2 item 4 trains operators to read that
exact phrase as *tampering*. A wrong file in the region is not an attack and
must not be reported as one.

### 5.3 Widened admission — NORMATIVE, Rust first

Today the public section admits **`ClassMDMK` only** (`seal/record.go`'s own
fail-closed comment says so). Two changes:

1. **A record class for free text**, so Engrave Text can be fed. None exists.
2. **Secret classes permitted in a plaintext container.** An unsealed payload
   has no encrypted section, so a secret has nowhere else to live.

Both are wire/admission behaviour. Per the Rust-primary rule they land in
`mnemonic-engrave`'s Rust **with test vectors first**; the fork's Go is a
behaviour-faithful port and may never lead.

#### 5.3.1 The two new classes collide with EPD§6.4 — and are ENCODED, not exempted

Writing this section specifically is what found the collision, and it is a hard
one. EPD§6.4 is normative and emphatic:

> "Every record MUST be the canonical, unbroken string — **no interior spaces,
> no hyphens, no grouping of any kind**."

and it justifies LF as the separator on the grounds that "no constellation
string contains a newline."

**Both new classes violate both clauses.** `Hello, World!` has spaces.
`correct horse battery staple` has spaces. And Engrave Text's keyboard carries
an **`nl` key**, so free text can contain the exact byte used as the record
separator. EPD§6.6 additionally hashes "canonical **LOWERCASE** records", so any
encoding must also survive lowercasing.

**The exemption is refused.** Relaxing EPD§6.4 for two classes would weaken the
rule for all of them, and that rule is load-bearing for a reason EPD states at
length: `mdmkFlow`/`bundleEngrave` engrave records **verbatim**, so a record
carrying separator characters the BCH checksum never covered turns a scratch on
the operator's only copy into silently-absorbed damage rather than a detected
error.

**So the body is encoded, and the record stays canonical:**

```
text:<lowercase hex of the UTF-8 bytes>
pass:<lowercase hex of the UTF-8 bytes>
```

**The digest label is `"MNEMSYSW/pub/v1"`, not `"MNEMBLOB/pub/v1"` (R0-M1).**
EPD§6.6's label exists to stop cross-context collisions, and §4 insists the two
containers be distinguishable; reusing the label would have made two containers
the spec separates produce identical digests over identical public sections.
The construction is otherwise EPD§6.6's, verbatim.

Lowercase hex, not base64 or base32: it is case-insensitive by construction so
lowercasing cannot destroy it, it contains no space, hyphen or LF, and it is the
easiest form for a human to compare on a screen. The cost is 2×, against
`MaxSectionLen` of 8191 — about 4 KB of text, far past what 24 engraved lines
can hold, so the expansion binds nothing real.

**Classification order is normative:** the two prefixes are matched **before**
the existing sniffers in `Classify`. Free text is the universal fallback — any
string could be free text — so a sniffer that ran first would claim
`text:...` records whose hex body happened to parse as something else. The
prefixes are also **reserved**: a record beginning `text:` or `pass:` that is not
valid lowercase hex is `ClassUnknown` and refused, never silently treated as
free text.

#### 5.3.2 The card-set DECODE requirement APPLIES — NORMATIVE (R0-I1)

EPD§6.3's per-card-set decode requirement carries over to this container
unchanged. Stating it is not a formality: without it the §5.3 flag is **silently
defeated in exactly the case that matters**.

`ValidMD`/`ValidMK` are **pure BCH verifiers**, so 32 bytes of seed entropy wrap
into a record that classifies as `ClassMDMK` — not secret, no flag, no offer to
erase, sitting in cleartext flash where `picotool save` reaches it with no
passphrase, on a device whose BOOTSEL is enabled by design. EPD measured that
bypass and closed it with `decodePublicSet`, whose comment names the threat: *"a
defective or third-party sealer can put seed entropy in the cleartext section."*

**A `ClassMDMK` record that does not REASSEMBLE AND DECODE is refused.** The
widening in §5.3 admits *declared* secrets; it does not admit undeclared ones.

**And pass 3 must be restructured, not merely permitted through.**
`AdmitSection`'s pass 3 sends every public record through `cardKey`, whose
`default` branch fails closed with "record %d is not an md1 or mk1 card".
Widening `permitted` without touching pass 3 would reject every payload the
widening was meant to allow. **Pass 3 runs over the `ClassMDMK` subset only** — and **the subset must carry
its ORIGINAL indices** (R1-I2). `groupRecords`/`cardKey`/`labelCards` are
index-coupled to the full record list (`cardKey` returns `uniq: i + 1`), so
transcribing "run over the subset" literally — by compacting the subset into a
fresh slice and re-indexing from zero — backfills plate identity onto the wrong
records. Filter the *iteration*, never the indices.

The two passes are coupled and a fold that changed one would have shipped a
container that admits nothing.

**A plaintext container carrying a secret class is flagged on screen at load**,
paired with an operator-initiated **erase this region**. The erase is a menu
item the operator chooses — not the automatic wiping machinery decision 2
rejects — so a warning has something to do besides be dismissed.

### 5.4 Flash-delivered input is digest-verified, once per payload — operator ruling 2026-08-11

**"Flash verify user input everywhere. Once per payload."**

Two clauses, and each rules something out.

**"Flash"** scopes this to payloads read from the region. **NFC is NOT covered.**
A tag carries a bare record with no header and no digest, and manufacturing one
would mean `me` printing a digest when it writes a tag plus a new
domain-separation label — `"MNEMBLOB/pub/v1"` cannot be reused over a
differently-shaped input, which is what that label exists to prevent. That is
net-new normative work for a transient delivery path, and it is **out of scope
for this spec**. See §11 for what that means the operator is *not* getting.

**"Everywhere"** means every *program*, not every *variant*: **wherever a
digest exists**, it is shown, and so it is shown to every program that consumes
from the region. The digest is not the plaintext container's consolation prize.

**A digest does not always exist.** EPD§6.6 displays one only when
`pub_len > 0`. A secrets-only sealed payload has none — see the table below and
§5.4.1's `compared` rule for what authenticates it instead.

**CORRECTED after R0-C2 and R0-I2. The first draft was wrong twice here.**

| container | digest shown? | what the AEAD tag covers |
| --- | :-: | --- |
| plaintext (`pub_len > 0`) | yes | no tag exists |
| sealed, with public records (`pub_len > 0`) | yes | the ciphertext, **and it binds the header and public section as AAD** |
| sealed, secrets only (`pub_len == 0`) | **NO** | as above |

**There is no digest for a secrets-only sealed payload, and that is EPD's own
rule, not a gap.** EPD§6.6: *"Displayed whenever `pub_len > 0`, sealed or not…
When `pub_len == 0` **nothing is displayed**: the digest of an empty record set
is a constant."* The first draft said the digest is shown for both variants and
every consuming program — which for the sealed variant's **main case** would
have displayed one number that every fully-encrypted payload shares. An operator
comparing it would match every time, against anything.

**And the tag's job was stated wrongly.** EPD§6.1a is normative: *"AAD = the
header AND the public section."* So the public section is covered **twice**,
deliberately — cryptographically by the tag and out-of-band by the digest — and
the first draft's "neither is redundant" was both false and the premise for a
coverage-is-complete claim that does not hold.

**What the sealed digest is actually for, in one line:** not coverage the tag
lacks, but **downgrade detection** — visible *before any key exists*, which is
the one thing an AEAD structurally cannot do.

**The systemwide container's AAD is stated here rather than by reference:**
`AAD = header ‖ public section`, bytes `[0, HeaderLen + pub_len)`, identical to
EPD§6.1a. Left implicit, an implementer could bind the ciphertext framing alone
and reopen the funds-loss path EPD§6.1a exists to close — an attacker swaps an
`mk1` for one encoding *their* xpub, the tag still verifies, and the operator
engraves a steel backup of a wallet they do not control.

**"Once per payload"** fixes the frequency. The operator compares **at load, one
time**, not once per consuming program. A payload that feeds five programs in a
session is compared once.

That has a design consequence which must not be left implicit: **the session
records that this payload's digest was compared**, so downstream screens state
the fact rather than re-asking. A screen that re-prompts teaches the operator
that dismissing the prompt is normal — the same disarming effect EPD§6.6 warns
about when it explains why `public_record_count` had to be bound in ("teaching
the operator that mismatches are normal, which disarms the single control
EPD§6.6 exists to be").

**The flag is on the payload, not the record.** Re-reading the region produces a
new payload identity and therefore a new comparison; consuming a fifth record
from an already-compared payload does not.

#### 5.4.1 Payload identity — NORMATIVE

**Defined by `[identity]` (§12.3); this section states no version of its own.**

Why it is not the EPD§6.6 digest, recorded here because R0-C2 cost a round: that
digest does not exist for a secrets-only sealed payload (`pub_len == 0`), so
every such payload would have shared one identity and a swapped payload would
have inherited the previous one's `[compared]` — a silent authentication bypass
in exactly the case that carries the seed.

Nothing else is admissible, and the alternatives are worth ruling out by name
because each looks reasonable and each fails:

| candidate | why it fails |
| --- | --- |
| a load counter | two different payloads loaded in sequence both get "compared", and the second inherits the first's flag |
| the region address | constant; every payload ever written shares it |
| a length or record count | two payloads trivially collide |
| the **displayed** 16 bytes | a truncation used as an equality key; the full digest costs nothing and is already computed |

The session stores `(identity, compared: bool)`. A record is admitted for
consumption only when `compared` is true for the identity it came from.
Re-reading the region recomputes the identity; if it differs, the entry is a
different payload and `compared` starts false.

**What sets the flag is defined by `[compared]` (§12.3's neighbour, §12.2).**
This section states no version of it. R1-C4 is why the rule needs a home at all:
it found the sealed variant's main case unusable and tests 9 and 15 mutually
unsatisfiable, because two sections had each answered the question differently.

**The AEAD route is scoped to strong keys, and R2 is why (operator ruling
2026-08-11).** The first fold said any successful open sets `compared`, on the
grounds that a cryptographic tag beats a human reading sixteen hex digits. That
is true only while the key is strong. This spec permits **22 bits** (2 words, 42
seconds on one GPU) and treats user-supplied as **0 bits** — so an attacker
brute-forces the passphrase, produces a payload that opens cleanly, and the flag
that was supposed to mean "authenticated" is set by the attack itself. A tag
proves someone knew the passphrase; that is worth exactly what the passphrase is
worth.

**Consequence, stated rather than discovered:** a *secrets-only* sealed payload
under a sub-cliff passphrase **cannot be consumed at all** — it has no digest to
compare and no qualifying open. That is the honest outcome. The alternative is a
payload the machine calls authenticated because someone guessed a two-word
passphrase in under a minute.

A secrets-only sealed payload (`pub_len == 0`) displays no digest — see
`[digest-shown]` — so there is nothing for the operator to compare. What may
authenticate it instead is defined by **`[compared]` (§12.2)**, and this section
states no version of its own.

A plaintext payload has no tag, so for it the operator comparison is the only
route — which is exactly EPD§6.6's framing of what stands in for a missing tag.

**The consequence this closes:** without a content-derived identity, an attacker
who swaps the region between two consumptions gets their payload treated as
already verified — the operator would be reassured by a comparison they made
against different bytes. That is a silent authentication bypass, and it is the
whole reason identity is specified here rather than left to the implementer.

### 5.5 The overwrite payload — operator ruling 2026-08-11

After an engrave that consumed a payload-sourced record, **the device reminds
the operator to overwrite the region.** A reminder, not an automatic wipe —
decision 2 stands.

**It is NOT a container (R0-I5).** The first draft called it a "payload", which
made it subject to the format's own caps — EPD admits at most 16,450 bytes
against a 65,536-byte region, so a "max-length payload" would have left **74% of
the region untouched** and made "fills the region" false and test 12
unsatisfiable. The overwrite artefact is a **raw region image**: exactly
`RegionLen` bytes of fill, carrying no magic, no header and no records. It is
written to the region and is not parseable as a container by design — a reader
finding it sees no magic and reports "no payload", which is the correct answer.

**It is a FULL-REGION image, not a zero-length payload.** A zero-length payload
rewrites the header and leaves every byte of the old body sitting in flash,
which looks like erasure and is not. The overwrite payload fills the region, so
the previous contents are physically replaced.

`me` emits it with a chosen fill: **all zeros, all ones, or random.**

Notes that belong in the spec rather than in a reviewer's head:

- **All-ones is the erased state** of NOR flash — an erase sets bits to 1 — so a
  region written to `0xFF` is indistinguishable from one that was erased and
  never written. That is a feature if the operator wants deniability about
  whether a payload ever existed, and a bug if they want evidence that they
  overwrote deliberately. The three fills are not interchangeable and the
  documentation must say which does what.
- **Random is the DEFAULT** — decided here rather than left open. Where the goal
  is that nothing be inferable from the residue, random is the only fill that
  achieves it, and a default of all-ones would hand the operator deniability
  they did not ask for while destroying the evidence that they acted. Zeros and
  ones remain available by flag.
- This region is **raw XIP NOR with no flash translation layer**, so an
  erase-and-program rewrites the same physical cells. That is a materially
  stronger guarantee than overwriting a file on an SSD, and weaker than a claim
  that the prior contents are unrecoverable by any means. Claim the former only.

### 5.6 The `me` command surface — NORMATIVE

```
me sysw pack   [--out FILE] [--passphrase-words N | --passphrase-ask | --no-passphrase]
               [--allow-weak] RECORD...
me sysw wipe   [--out FILE] [--fill random|zeros|ones]
me sysw show   FILE
```

| flag | meaning |
| --- | --- |
| `--passphrase-words N` | generate, `2 ≤ N ≤ 24`, **default 12** if no passphrase flag is given |
| `--passphrase-ask` | prompt for a user-supplied passphrase; **never** taken from argv or an env var, where it would land in shell history and `/proc` |
| `--no-passphrase` | plaintext container |
| `--allow-weak` | required by §6.2.1 when secret content meets a sub-cliff passphrase. **Refuses with a non-zero exit otherwise** |
| `--fill` | §5.5; **default `random`** |

`me sysw pack` **prints the digest to stderr** in the EPD§6.6 display form,
because that is the value the operator writes down and compares on the machine.
Stderr, not stdout, for the reason `me seal` already prints its passphrase
there: stdout may be a redirected blob.

**`me sysw show` exists so the operator can re-derive the digest from a file
they still hold** without a machine, which is what makes the comparison a real
control rather than a number they must have transcribed correctly the first
time.

Subcommands sit under `sysw` rather than beside `seal` so that no invocation can
produce a systemwide container while the operator believes they are producing a
Sealed Payload one, or vice versa. The two features are frozen apart
(decision 1); their command surfaces are too.

## 6. Passphrases

Three modes in `me`: **none**, **user-supplied**, **generated N words** (default
12, min 2, max 24).

### 6.1 What each length buys

At `seal`'s `MinIterations` (100,000 PBKDF2-SHA256), 2048-word list, expected
work = half the keyspace. Model: one high-end GPU ≈ 10¹⁰ SHA-256 compressions/s,
and PBKDF2 costs ~2 compressions per iteration, so ~200,000 per guess ⇒ **~50,000
guesses/s per GPU**. The rate is an order-of-magnitude model, not a measurement;
the *shape* — a cliff between 4 and 5 words — survives an order of magnitude in
either direction, which is the only property the rule in §6.2 rests on.

| words | bits | one GPU | 100 rented GPUs |
| --- | --- | --- | --- |
| 2 | 22 | **42 seconds** | **0.4 seconds** |
| 3 | 33 | 24 hours | 14 minutes |
| 4 | 44 | 5.6 years | 20 days |
| **5** | **55** | **11,400 years** | **114 years** |
| 6 | 66 | 2.3×10⁷ y | 2.3×10⁵ y |
| 12 *(default)* | 132 | 1.7×10²⁷ y | 1.7×10²⁵ y |

**Entropy falls off a cliff between 4 and 5 words**, which is where the
threshold in `[cliff]` (§12.1) came from. A 2-word passphrase is not weak
protection; it is none — 42 seconds is less time than it takes to type it.

**But `[cliff]` counts words, not bits.** This table describes what *uniform
generation* buys. It is not the gate and must not be read as one: a five-token
degenerate passphrase sits at zero bits and is `[cliff]`-above.

### 6.2 The rule — and the metric it needs, which the first draft omitted

**Below 5 words (55 bits) over secret content, `me` requires an explicit
command-line flag.** Public-only content is unrestricted. `me` always prints
what the choice bought; the device flags at load when secret material is
protected by less than the cliff.

The warning attaches to the **outcome**, not the mode — so no-password, weak
password and plaintext are one control rather than three special cases. This is
consistent: the operator has already permitted unencrypted secrets in flash with
a flag, and a 2-word passphrase is strictly stronger than that.

#### 6.2.1 How strength is computed — NORMATIVE

The rule above says "below 5 words (55 bits)". That is measurable for a
*generated* passphrase and **meaningless for a user-supplied one** —
`correct horse battery staple` has no word count in the wordlist sense and
`Tr0ub4dor&3` has none at all. The first draft of this spec stated the gate in
units that only apply to one of the three modes, which left the condition
undefined for another. This closes it:

**The gate is `[cliff]` (§12.1) — a word count over the normalised string —
and NOT the mode.** This table is retained for what each mode is worth, which is
a different question:

| mode | entropy | `[cliff]`? |
| --- | --- | --- |
| generated, N words | exactly `11 × N` bits, drawn uniformly | above iff `N ≥ 5` |
| user-supplied ASCII | not estimated; see below | **below**, always — its tokens are not wordlist entries |
| none | 0 bits | below |

**R3-C2 is why this is stated as deference rather than as a second
definition.** An earlier version defined the threshold here by mode and bits
while §12.1 defined it by word count, so `me` would have sealed a user-supplied
five-word payload as above-cliff while the device refused it as below — a
permanently unconsumable payload when secrets-only.

**User-supplied is not estimated, and that is the point.** Every estimator for
human-chosen passphrases is either a dependency with its own failure modes or a
charset-times-length formula that scores `Tr0ub4dor&3` far above
`correct horse battery staple` and is wrong about both. `passphrase.rs` already
records the honest number — human-chosen is "worth 25–35 bits — one rented GPU,
minutes" — and that entire range is far below what generation buys, so any
faithful estimator returns the same answer for every input. Under `[cliff]` the
question does not even arise: a user-supplied ASCII passphrase is below it by
definition.

**And an estimator here would have to model that CASE IS DISCARDED before
hashing.** `seal.NormalisePassphrase` is
`strings.ToLower(strings.Join(strings.Fields(s), " "))` (`seal/open.go:76`), so
`Tr0ub4dor&3` and `tr0ub4dor&3` are the same passphrase and `a  b` is `a b`.
That is the *same* mechanism that lets the two keyboards agree byte-for-byte, so
it cannot be dropped without breaking 8a. It is harmless for generated word
passphrases, which are already lowercase, and silently weakening for a
user-chosen mixed-case one — **so it must be stated where the operator chooses
their own.**

So the deterministic rule an implementer transcribes:

> **Secret content + (user-supplied OR no passphrase OR generated with `N < 5`)
> ⇒ `me` refuses without the explicit flag.**

`me` prints the strength either way — `"12 words, 132 bits"` or
`"user-supplied, unmeasured, below the cliff"` — the latter being a
consequence of `[cliff]` (§12.1), not a separate judgement.

#### 6.2.2 Host and device must agree on what is ENTERABLE — NORMATIVE

Three host/device mismatches of the R0-C4 shape — the host seals what the device
cannot accept — two found by review and one by measurement:

| constraint | value | why |
| --- | --- | --- |
| character range | `0x20`–`0x7E` only | The device's real constraint: `passphrase.ValidatePassphrase` rejects anything else as `ErrNonASCII`, so a UTF-8 passphrase would seal and never open. **The next row rejects the same function's `MaxLen`, and the two are not in tension (R2-N1):** the range is about what entry can represent, while `MaxLen = 100` is by its own comment "a plate-capacity limit chosen for legibility" — a fact about steel, not about typing. Take the constraint that is about entry; reject the one that is about a plate |
| length cap | **exactly 215 bytes**, host and device | An inequality is not a spec (R2-I3): "≥ 215" on the host and "≥ 215" on the device can be two different numbers — the R0-C4 shape inside the section named after it. 215 is the measured maximum (24 words × 8 characters + 23 separators; a 12-word passphrase already reaches 107), and it bounds the **user-supplied** mode too, which otherwise had no derived bound at all. `passphrase.MaxLen` is 100 and does not apply — see the row above |
| checksum | never required (§8b) | else 15 of 16 default draws are unopenable |

`me` enforces the identical range and cap at creation.

**Do not call `passphrase.ValidatePassphrase` on this path.** It bundles the
range with `MaxLen = 100`, so calling it to get the range imports the cap this
spec rejects. Take the range; write the check.

**And the device's buffer must be sized for the maximum, not the default
(R1-I3).** `passphraseBytes`' capacity was chosen for twelve words. A 24-word
passphrase regrows it, and the regrow **orphans an unwipeable copy of a
seed-equivalent secret** — the same defect class as the `secret[:n]` residue
already in `FOLLOWUPS.md`, and one the operator's 2..24 range creates.

> **The buffer is allocated once at exactly `[passphrase-bounds]`' 215 bytes
> (§12.5) and never regrows.** Identical reasoning to `passphrase.rs`'s `normalise`, which
> pre-counts precisely so "a `String` that reallocates mid-build orphans the
> partially-written copy."

#### 6.2.2a Residue on these paths is ACCEPTED, and the no-regrow rule is not a wipe claim (R2-I2)

`seal.NormalisePassphrase` takes and returns a Go `string`
(`seal/open.go:76`), so the free-text path necessarily holds a seed-equivalent
passphrase in allocations nothing can scrub — the exact shape `passphraseBytes`
exists to avoid.

**This is consistent with decision 2, not a violation of it.** These programs
are the non-wiping class: they leave secret material resident with no timer
behind them, by the operator's explicit ruling and by EPD§2.2 item 12 before it.
The spec therefore does **not** claim the passphrase is wiped on these paths,
and the §6.2.2 no-regrow rule is **not** a wipe guarantee. Its narrower job is
to avoid a *gratuitous* second copy — one the code would orphan for no reason —
which is worth having even where the first copy persists.

Anything stronger belongs to F-124, not here. A spec that implied these paths
scrub would be making F-123's mistake about a different control.

#### 6.2.3 What the operator is told

`me` prints, and the device shows before the KDF runs, that a user-supplied
passphrase is **lowercased and whitespace-collapsed** before hashing. An
operator who chose a mixed-case passphrase is otherwise never told that half of
what they chose was discarded.

### 6.3 Consequence: `is_valid` changes, and what replaces the typo screen

`is_valid` is `Mnemonic::parse_in(...).is_ok()` — it accepts only 12/15/18/21/24
words **with a valid checksum**; a random 12-word draw passes about one time in
sixteen. Arbitrary N requires drawing words directly from the wordlist rather
than via `Mnemonic::from_entropy_in`, and a changed validity rule.

EPD§8.1 requires host and device to produce **byte-identical KDF input**, so both
sides move together.

The passphrase remains "used ONLY as a passphrase: never seed entropy, never
derives a wallet", so a non-mnemonic word sequence is legitimate here.

**What is LOST, and what replaces it (R0-N1).** `Mnemonic::parse_in` also
validates the BIP-39 checksum, which today catches a mistyped word *before* the
~31 s KDF and reports a typo rather than "wrong passphrase". Dropping the
checksum drops that screen: with arbitrary N there is no checksum to fail, so
every typo becomes an indistinguishable failed unlock after a half-minute wait.

**Replacement, NORMATIVE:** every entered word is checked against the 2048-word
list at entry — the word keyboard already does this for seed entry
(`bip39.ClosestWord`) — so a *non-word* is rejected at the keystroke, before the
KDF. What is genuinely unrecoverable is a valid word in the wrong place, which no
checksum-free scheme can catch and which the operator documentation must say
plainly rather than leave as a surprise after 31 seconds.

## 7. Verification

**The hash and the plate verify defend opposite ends of the pipeline.** The hash
proves the right bits went *in*. Verification proves the right words came *out,
on steel*. No amount of input integrity says anything about the second, and this
machine's output end demonstrably fails: the tilde-plateau artefact was a loose
Y-axis screw, `EngraverStats` counts stalls because steppers stall, F-83 accepts
that a plate under the needle cannot be protected, and the font work exists
because a glyph can lose its identity to one scratch.

### 7.1 The menu is always offered, and never forced — operator ruling 2026-08-11 (R0-C3)

**REPLACES the first draft, which gated the menu on "the operator compared the
hash".** R0-C3: the device **cannot observe that**. §5.2 says so itself — the
machine has no idea what the operator wrote down; it sees a button press. So the
first draft let a *dismissed* prompt buy a weakened verification, and made one
defence unlock the other after §7 opens by declaring them independent.

**Operator ruling:** *"User shouldn't be prompted to verify something
unnecessarily and should be given option to bypass verification or insist
without proof verification was performed by eye and passed."*

So:

- **The menu is offered regardless of source.** No gate, because there is no
  gate the device can honestly evaluate.
- **Bypass is a menu option**, not a hidden escape.
- **"Read only" is an operator ASSERTION.** The operator declares they checked
  by eye and it passed; the device records the declaration and does not pretend
  to have confirmed it.

#### 7.1.1 Verification provenance — NORMATIVE

Because two of the options produce a result the device did not compute, the
outcome carries its provenance, exactly as a record carries its source (§3.2):

| outcome | meaning |
| --- | --- |
| `device-compared (every word)` | the device compared all words |
| `device-compared (N of M)` | the device compared a subset; §7.3 gives the rate |
| `operator-asserted` | the operator declared a by-eye check passed |
| `not verified` | bypassed |

**Nothing may render any of these as the bare word "verified".** An operator
assertion and a device comparison are different facts, and collapsing them is
the same over-claim F-123 was filed against — this time about the plate rather
than the wipe.

### 7.2 The menu

The operator chooses the depth. Labels carry the effort; the panel does not
carry the statistics.

| menu label | words typed |
| --- | --- |
| every word | all |
| even words / odd words | half |
| 6 words | 6, chosen at random |
| 3 words | 3, chosen at random |
| read only | none — the operator compares by eye, and ASSERTS the result (§7.1.1) |
| skip | none — verification is **bypassed**; outcome is `not verified` |

The `skip` row is normative and R1-I1 is why: §7.1 calls bypass "a menu option,
not a hidden escape", and without a row for it §7.1.1's `not verified` outcome
is unreachable by any path an implementer transcribing this table would build.

#### 7.2.1 Selection — NORMATIVE

- **`even words` / `odd words`** are 1-indexed over the engraved word list:
  *even* is words 2, 4, 6, …; *odd* is 1, 3, 5, …. Stated because "even" is
  ambiguous between the two indexings, and an implementer picking the other one
  produces a verify that silently checks the complement of what the label says.
- **`6 words` / `3 words`** are drawn **without replacement**, uniformly, from
  all positions.
- **The draw is fresh for every verification attempt.** If the operator fails
  and retries, a new set is drawn. A fixed set would let a second attempt pass by
  reciting the same positions, which is a memory test rather than a plate check.
- **The RNG is the device's CSPRNG**, the same source seed entropy comes from —
  never a counter, a tick, or a hash of the mnemonic. A predictable draw lets an
  attacker who controls the plate know which positions go unchecked.
- **On a 12-word seed, `6 words` and `even words` are equal in strength** (both
  50%, per §7.3) and both are offered anyway, because removing an option on some
  seed lengths and not others is a worse surprise than a redundant one.

### 7.3 Detection rates — for the documentation, not the panel

P(catching a mis-cut plate), by how many words the machine got wrong:

| check | 12-word: 1 | 2 | 3 | 24-word: 1 | 2 | 3 |
| --- | --- | --- | --- | --- | --- | --- |
| 3 words | 25.0% | 45.5% | 61.8% | **12.5%** | 23.9% | 34.3% |
| 6 words | 50.0% | 77.3% | 90.9% | 25.0% | 44.6% | 59.7% |
| even/odd | 50.0% | 77.3% | 90.9% | **50.0%** | 76.1% | 89.1% |
| every word | 100% | 100% | 100% | 100% | 100% | 100% |

Two facts the operator documentation must state plainly:

- **A 3-word check on a 24-word seed catches a single wrong word 12.5% of the
  time.** A pass has taught the operator very little.
- **"Read only" is the one option whose rate cannot be stated**, because the
  human is the comparator. The docs must say so rather than assign it a number
  it has not earned.

### 7.4 The rule that is not negotiable

**The session cache must never answer a verification prompt on the operator's
behalf.** "Read only" is a choice made at a menu; it must not become something
that happens because a secret was already in memory. Otherwise verify compares
the engrave source against itself and passes unconditionally — certifying a
*wrong plate* as good, silently. A test asserts a cached secret cannot reach a
verify comparison.

## 8. Testing

### 8.1 Rules this follows

- Widened admission lands in **Rust with vectors first**; Go is a port.
- **Mutation testing**, because this repo has found five false-passing tests by
  breaking code and none by reading it.
- Every machine-checkable claim in this spec is checked before a reviewer sees
  it (build gate).

### 8.2 The emulator gap — must be closed as part of this work

`cmd/emu`'s `NFCReader()` returns nil: "this emulator has no tag source." If NFC
becomes a first-class secret path for eight programs, **the tool used to qualify
screens is blind to the new path.** `platform.go` already sketches the fix — "a
syscall/js read of `location.search` or a JS global set from the host page —
nothing here forecloses that." Build it, so the path is walkable in a browser
rather than only on hardware.

### 8.3 Named tests

1. A cached secret cannot reach a verify comparison (§7.4).
2. A wrong word at any position *included* in a check is caught.
3. Random word selection is uniform and re-drawn per verification.
4. A plaintext container carrying a secret class raises the flag.
5. `me` refuses sub-cliff + secret without the flag, and permits it with.
6. Host and device produce byte-identical KDF input for an arbitrary-N
   passphrase.
7. A blob written to the wrong region is refused on magic, not half-parsed.
8. Structural failures never emit the words "payload unreadable" (§5.2).
   **The assertion is against `gui`, not `seal` (R0-M2)** — the phrase is a UI
   string and lives in the flow layer, so a test scoped to `seal` cannot fail
   and would be a false pass.
9. The digest is displayed **wherever one exists** (`pub_len > 0`), for every
   program that consumes from the region (§5.4). A program that consumes
   payload-sourced input without a satisfied `compared` fails. **This test and
   test 15 must both pass**: 15 asserts no digest for `pub_len == 0`, and a
   version of this test asserting "always displayed" makes the pair
   unsatisfiable (R2-C1).
10. The digest is compared **once per payload**: a second program consuming from
    the same loaded payload does NOT re-prompt, and re-reading the region DOES.
    A test that only checks the first consumption cannot tell these apart.
11. The overwrite payload **fills the region** (§5.5): after writing it, no byte
    of the previous payload remains. A zero-*length* payload must fail this test
    — it is the defect the requirement exists to prevent.
12. Each fill — zeros, ones, random — produces the region it claims to, and
    all-ones is byte-identical to an erased region.
13. The post-engrave overwrite reminder fires after a payload-sourced engrave
    and not after a typed one.
14. **(R0-I1)** A BCH-valid `md1` carrying non-decodable entropy is refused,
    asserted against the real `md.Reassemble` — not a hand-built fixture. Test 4
    cannot catch this: it constructs a record that classifies as secret, and the
    defect lives entirely in records that do not.
15. **(R0-C2)** A secrets-only sealed payload (`pub_len == 0`) displays NO
    digest, and two different such payloads have DIFFERENT identities (§5.4.1).
    A test asserting only the first half passes on the bypass.
16. **(R0-C1)** No verify flow can reach a payload-sourced secret — asserted
    structurally. **The match must be on the identifier, not a substring
    (R1-N1)**: `seedEntryFlowTypedOnly` contains `seedEntryFlow`, so a
    `strings.Contains` assertion fails on a correct implementation. Parse the
    AST and compare `*ast.Ident` names, as `gui/plate_hook_test.go` already
    does.
17. **(R0-C3)** An operator-asserted verification is never rendered as
    "verified"; the four provenances of §7.1.1 are distinguishable in whatever
    the flow records and displays.
18. **(R0-I2)** The systemwide container's AAD is `header ‖ public section`: a
    payload whose public section is altered after sealing fails to open —
    **and the alteration must be one that survives the structural checks
    (R1-M2)**, or the test passes because the payload was refused before the
    AEAD ran and proves nothing about the AAD. Alter a record to another
    *valid* record of the same class and length.
19. **(R1-C2)** A uniformly generated N-word passphrase is enterable for every
    `N` in 2..24, including every `N % 3 == 0`. Draw many; a test using one
    fixed passphrase passes 1 time in 16 by luck.
20. **(R1-C4, corrected R3-C1)** A secrets-only sealed payload is consumable
    **when its passphrase is `[cliff]`-above**, and is NOT consumable when it is
    below — per `[compared]` (§12.2). The first version of this test asserted
    the unconditional rule and would have MANDATED the sub-cliff bypass rather
    than merely permitting it. Test 15 asserts no digest is displayed; neither
    alone is enough.
21. **(R1-I3)** The passphrase buffer never regrows: entering 24 words leaves no
    orphaned copy. Assert on the buffer's identity, not on its contents.
22. **(§8c)** A `done` press mid-passphrase yields a confirmation naming the
    SHORT count, not the intended one — the truncation is visible before the
    KDF.

## 9. Open items

| # | Item | Owner |
| --- | --- | --- |
| ~~O1~~ | ~~Flash address~~ — **RESOLVED 2026-08-11: `0x10D00000`** | — |
| ~~O2~~ | ~~Which keyboard the unlock screen uses~~ — **RESOLVED**: `unlockPassphraseFlow` returns a `bip39.Mnemonic` on the WORD keyboard and enforces `!m.Valid()`. A defect, not a question. It did NOT remove a passphrase mode (R2-M2) — the mode was restored (§8) and the resolution is a NEW unlock flow (§2.2 item 8) | — |
| O3 | Record class name and encoding for free text | R0 / Rust |
| O4 | `me` subcommand surface for creating a systemwide payload, and for the §5.5 overwrite payload | R0 |
| ~~O5~~ | ~~NFC digest domain separation~~ — **DISSOLVED 2026-08-11**: the operator scoped digest verification to FLASH, so no NFC digest is specified (§5.4) | — |
| ~~O6~~ | ~~Default fill~~ — **DECIDED: `random`** (§5.5, §5.6). Round 0 endorsed it; R1-N2 noted this row still described it as open | — |

## 10. Follow-ups filed with this spec

- **F-123** — the documentation implies the wiping class is meaningfully safer
  than it is. Operator ruling 2026-08-11: fix the docs.
- **F-124** — remedy Sealed Payload's security failures. Deferred deliberately;
  Sealed Payload is frozen for this cycle.

## 11. What this spec does NOT claim

- It does not claim the machine becomes safer. It adds convenience to programs
  that were already the non-wiping class, and it adds a **durable** plaintext
  resting place for secrets that did not previously exist. NFC is transient — a
  tag crosses the reader once; flash persists until overwritten, on a device
  whose SWD port is readable and whose BOOTSEL is enabled by design.
- **NFC-delivered secrets get no integrity check whatsoever.** §5.4 scopes
  digest verification to flash, so a tag's contents are admitted on their
  classification alone — nothing stands behind them, and the operator has
  nothing to compare. Flag F4 says so on screen at the point of use. This
  fulfils §5.4's forward reference, which R0-M3 found unfulfilled.
  **The §5.5 overwrite artefact is the mitigation for the flash case, and it is a REMINDER the
  operator must act on** — not something the machine does for them. A spec that
  counted it as protection would be making the same mistake F-123 was filed
  against: describing a control by its intent rather than by what it does when
  nobody runs it.
- It does not claim the operator-compared hash detects tampering unless the
  operator actually compares it.
- It does not change what protects the operator, which remains **physical
  custody**.

## 12. Normative definitions — the single source for each rule

**This section exists because four R0 rounds found the same defect: one rule
stated in several places, corrected in one of them, and left standing in the
others.** Five sites defined "the cliff"; two of those were written by the very
fold that redefined it. Enumerating invariants could not keep up, because it
required remembering the rule that had just been forgotten.

So from here: **every rule below is defined HERE and nowhere else.** Any other
section that needs one references it by name — `[cliff]`, `[compared]` — and
states no version of its own. `scripts/spec-check.py` enforces that: a
definitional phrasing found outside this section is a build failure, not a
review finding.

### 12.1 `[cliff]`

**A normalised passphrase is ABOVE THE CLIFF if and only if it consists of five
or more whitespace-separated tokens, every one of which is in the BIP-39 English
wordlist. Otherwise it is below.** Operator ruling 2026-08-11.

It is a **pure function of the normalised string** (§6.2.1's normalisation), so
host and device compute it identically with no shared state, no header field and
nothing attacker-controlled. That is the whole reason it is a word count: an
entropy threshold would have to be recorded somewhere the device could read, and
any such field is attacker-controlled.

**IT IS A SPEED BUMP, NOT A STRENGTH MEASURE, AND NOTHING MAY DESCRIBE IT AS
ONE.** `abandon abandon abandon abandon abandon` is five wordlist tokens, carries
**zero** entropy, and is above the cliff. This is deliberate: these programs are
the lower-assurance branch (decision 2, EPD§2.2 item 12) and the operator's
ruling was explicit that only modest effort goes into safety here.

Two consequences that follow mechanically and have caught this spec out:

- **Every user-entered non-BIP-39 password is below the cliff**, by operator
  ruling and by the definition — a free-text ASCII passphrase contains tokens
  that are not wordlist entries.
- **`correct horse battery staple` is BELOW the cliff.** Measured against
  `bip39/wordlist.go`: `correct` and `horse` are wordlist entries; **`battery`
  and `staple` are not.** It is 2-of-4, not 4-of-4. This spec used that phrase
  repeatedly as its illustration of a four-word passphrase, which was wrong on
  its own terms.

§6.1's bits table is **information about what generation buys**, not this gate.
The two must never be conflated: bits describe entropy, `[cliff]` counts words.

### 12.2 `[compared]`

**A payload's `compared` flag is set by EITHER:**

- **the operator comparing the displayed EPD§6.6 digest**, which exists only
  when `pub_len > 0`; **or**
- **a successful AEAD open whose passphrase is `[cliff]`-above.**

**A sub-cliff open does NOT set it.** A tag proves someone knew the passphrase,
which is worth exactly what the passphrase is worth — and `[cliff]` admits
zero-entropy passphrases by design, so the open is forgeable in precisely the
cases `[cliff]` waves through.

**Consequence, stated so it is not discovered:** a *secrets-only* sealed payload
(`pub_len == 0`) under a sub-cliff passphrase **cannot be consumed at all.** It
has no digest to compare and no qualifying open. That is the honest outcome; the
alternative is a machine reporting "authenticated" because someone typed
`abandon` five times.

### 12.3 `[identity]`

`SHA-256("MNEMSYSW/id/v1" ‖ 0x00 ‖ the region bytes as read, bounded by the
header's declared total)` — the full 32 bytes.

**Not the EPD§6.6 digest**, which does not exist when `pub_len == 0` and would
therefore give every secrets-only payload one shared identity, letting a swapped
payload inherit a previous one's `[compared]`.

### 12.4 `[digest-shown]`

**The EPD§6.6 digest is displayed wherever one exists — that is, whenever
`pub_len > 0` — and nowhere else.** EPD§6.6's own rule: when `pub_len == 0`
nothing is displayed, because the digest of an empty record set is a constant
that every such payload would share.

### 12.5 `[passphrase-bounds]`

| bound | value |
| --- | --- |
| character range | `0x20`–`0x7E` |
| length | **exactly 215 bytes**, host and device, over the NORMALISED string |
| word count | `2 ≤ N ≤ 24` for the generated mode |
| checksum | never required, at any length (§8b) |

215 is `LongestWord × 24 + 23`, and `bip39.LongestWord` is a declared constant
equal to 8. `passphrase.MaxLen = 100` does **not** apply: its own comment calls
it "a plate-capacity limit chosen for legibility", a fact about steel rather
than about entry.
