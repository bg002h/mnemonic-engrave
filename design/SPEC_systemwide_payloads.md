# SPEC — systemwide payloads: NFC and flash delivery for every program

Status: **DRAFT, pre-R0.** Written 2026-08-11 from a brainstorm with the
operator. No code has been written and none may be until this passes the R0
gate at 0 Critical / 0 Important (project rule: risk-set work, and this is
squarely in it — secrets, admission behaviour, eight programs, two repos).

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
   wipes, no idle wipe, no §10.2.4 clock. This does not overrule §2.2 item 12;
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
8. **`me` offers three passphrase modes** — none, user-supplied, and generated
   N words (default 12, min 2, max 24). Below the cliff over secret content
   requires an explicit command-line flag. *This overrules
   `crates/me-cli/src/seal/passphrase.rs`'s "GENERATED, never user-supplied",
   deliberately and with the numbers in §6 in view.*

## 2. What already exists, and what genuinely has to be built

The single most important finding of the brainstorm is how much of this is
already specified. Building the wrong thing here would mean re-inventing
controls that exist and are vectored.

### 2.1 Already exists — do not rebuild

| Thing | Where | Note |
| --- | --- | --- |
| An **unsealed** container variant | `seal/wire.go:95`, `Header.Sealed()` | `ct_len == 0` means no encryption, no tag |
| The **operator-compared hash** for that case | SPEC_encrypted_payload_delivery §6.6, NORMATIVE | "an out-of-band check the attacker does not control" |
| A **legible display format** for it | §6.6 | first 16 bytes, 8 groups of 4: `a26e d22b b747 dfd0 2367 06ad 14c1 9679` |
| **Downgrade detection** | §6.6's `sealed` byte | stripping a sealed payload to plaintext changes the digest; vectors D and E pin it |
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

## 3. Architecture

### 3.1 One seam, not eight

`seedEntryFlow` (`gui/derive_xpub.go:82`) is called from six sites across five
programs: `bip85.go:271`, `derive_xpub.go:107`, `singlesig.go:33`,
`multisig.go:91`, `multisig_build.go:58`, and the two verify re-entries at
`singlesig_verify.go:67` and `multisig_verify.go:50`.

Teaching that one function to offer three sources — **Typed / Scanned /
Payload** — gives five programs the feature at one stroke, with one set of
tests. `backupWallet`'s `newInputFlow`, plus BIP-39 Password, Engrave Text and
Engrave Bundle, call the same helper so there is **one admission path, not
eight**.

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

### 3.3 Admission is one function

Given `(record class, container variant, requesting program)` it returns admit
or refuse, with a named reason. Today this logic is spread across
`engraveObjectFlow`'s type switch and each flow's private assumptions;
consolidating it is what makes the rest of this reviewable. It is also where a
future rule goes, and where the tests point.

## 4. The flash region

**OPEN — the address is a proposal, not a decision.**

Proposed: `0x10D00000`–`0x10D10000`, 64 KiB, 16 × 4 KiB sectors.

Measured constraints, from SPEC_encrypted_payload_delivery §3 and §5:

| Fact | Value |
| --- | --- |
| Flash | 16 MB, `0x10000000`–`0x11000000` |
| Firmware image ends | `0x10135300` |
| Sector `picotool load` touches | `0x10136000` |
| Sealed Payload region | `0x10E00000`–`0x10E10000` |
| Must stay clear | top sector `0x10FFF000` (`--abs-block` defaults to `0x10ffff00`) |
| Hard limit | past `0x11000000` a write wraps to `0x10000000` and destroys the firmware (RP2350 datasheet §5.5.2) |

`0x10D00000` sits a full megabyte below the Sealed Payload region. **The
separation is the point**: adjacent regions mean a length bug in either runs
into the other's data, while a megabyte of unprogrammed flash between them keeps
an overrun local. ~12.8 MB is unallocated, so the clearance costs nothing.

**A distinct container magic**, not `MNEMBLOB`. Rationale: the operator froze
Sealed Payload, and widening the format it depends on would unfreeze it through
the back door. A distinct magic also means a blob written to the wrong address
is *rejected* rather than half-understood.

## 5. The container

Reuses §6.6's hash construction and `seal/record.go`'s classification. Its own
admission rules.

### 5.1 Sealed variant

Passphrase + PBKDF2-SHA256 + AES-256-GCM, as `MNEMBLOB`. Unlocked once per
session (§3.2).

### 5.2 Plaintext variant

No encryption, therefore no key, therefore no tag, therefore **no
authentication**. What stands in its place is §6.6's hash: `me` prints it at
creation, the device displays it at load, and **the operator compares**.

**The device never detects a hash mismatch.** It has no idea what the operator
wrote down. The machine's own failures are structural only — wrong magic, bad
lengths, records that do not decode — and they get **named reasons, never the
words "payload unreadable"**, because §2.2 item 4 trains operators to read that
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

**A plaintext container carrying a secret class is flagged on screen at load**,
paired with an operator-initiated **erase this region**. The erase is a menu
item the operator chooses — not the automatic wiping machinery decision 2
rejects — so a warning has something to do besides be dismissed.

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

**The cliff is between 4 and 5 words.** A 2-word passphrase is not weak
protection; it is none — 42 seconds is less time than it takes to type it.

### 6.2 The rule

**Below 5 words (55 bits) over secret content, `me` requires an explicit
command-line flag.** Public-only content is unrestricted. `me` always prints
what the choice bought; the device flags at load when secret material is
protected by less than the cliff.

The warning attaches to the **outcome**, not the mode — so no-password, weak
password and plaintext are one control rather than three special cases. This is
consistent: the operator has already permitted unencrypted secrets in flash with
a flag, and a 2-word passphrase is strictly stronger than that.

### 6.3 Consequence: `is_valid` changes

`is_valid` is `Mnemonic::parse_in(...).is_ok()` — it accepts only 12/15/18/21/24
words **with a valid checksum**; a random 12-word draw passes about one time in
sixteen. Arbitrary N requires drawing words directly from the wordlist rather
than via `Mnemonic::from_entropy_in`, and a changed validity rule.

§8.1 requires host and device to produce **byte-identical KDF input**, so both
sides move together.

The passphrase remains "used ONLY as a passphrase: never seed entropy, never
derives a wallet", so a non-mnemonic word sequence is legitimate here.

## 7. Verification

**The hash and the plate verify defend opposite ends of the pipeline.** The hash
proves the right bits went *in*. Verification proves the right words came *out,
on steel*. No amount of input integrity says anything about the second, and this
machine's output end demonstrably fails: the tilde-plateau artefact was a loose
Y-axis screw, `EngraverStats` counts stalls because steppers stall, F-83 accepts
that a plate under the needle cannot be protected, and the font work exists
because a glyph can lose its identity to one scratch.

### 7.1 When the menu is offered — resolved ambiguity

The menu appears when the secret's source is **independently verified**: a
plaintext container whose hash the operator compared, or a sealed container the
operator unlocked. **A typed seed keeps today's full re-entry.**

This is narrower than it strictly needs to be, and deliberately so. The
read-back does the same job for a typed seed — it catches a mis-cut plate either
way — so the menu *could* apply uniformly. It is scoped to verified sources
because that is what the operator asked for, and widening a verification
weakening beyond the case it was requested for is not a decision to make by
implication. Extending it to typed seeds is **out of scope for this spec** and
would be its own operator ruling.

### 7.2 The menu

The operator chooses the depth. Labels carry the effort; the panel does not
carry the statistics.

| menu label | words typed |
| --- | --- |
| every word | all |
| even words / odd words | half |
| 6 words | 6, chosen at random |
| 3 words | 3, chosen at random |
| read only | none — the operator compares by eye |

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

## 9. Open items

| # | Item | Owner |
| --- | --- | --- |
| O1 | **Flash address** — `0x10D00000` is a proposal | operator |
| O2 | Which keyboard the Sealed Payload unlock screen uses — **not verified**; `PassphraseKeyboard` is free-text (`gui.go:640`) but the unlock path was not traced | implementation |
| O3 | Record class name and encoding for free text | R0 / Rust |
| O4 | `me` subcommand surface for creating a systemwide payload | R0 |

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
- It does not claim the operator-compared hash detects tampering unless the
  operator actually compares it.
- It does not change what protects the operator, which remains **physical
  custody**.
