# SPEC_hashlock_kinds — RESTORE / DURABILITY lens (round 4)

**Artifact:** `design/SPEC_hashlock_kinds.md` at `2a0da99f` ("fold: R0 round 3 —
the four Minors on a GREEN round"). Measured: **494 lines** (the brief's "473"
is stale; the committed blob is 494). Working tree is identical to `2a0da99f`
for this file (`git diff --stat` empty).

**The lens, and only this lens:** an operator holds a pile of engraved steel five
years from now, with no device and no memory. Does the spec guarantee that
everything needed to *reconstruct and satisfy* a hashlock is on something
durable, and does it say so where an implementer will act on it?

**Not re-reviewed:** anything the three correctness rounds closed. Everything
below is new to this lens.

---

## The headline, first: there is no funds-loss path, and the reason is one fact

**The hash kind is on the steel.** It is in the md1 wire bytes, and it comes back
out by name with one command. Machine-verified, not reasoned:

```
$ md encode 'wsh(and_v(v:ripemd160(ba7816bf8f01cfea414140de5dae2223b00361a3),pk(@0/<0;1>/*)))'
md1yqpqqxpyekpwncz6lc7qw0afq5zsx7tkhzygasqds6x2c70kfkfrram89

$ md decode md1yqpqqxpyekpwncz6lc7qw0afq5zsx7tkhzygasqds6x2c70kfkfrram89
wsh(and_v(v:ripemd160(ba7816bf8f01cfea414140de5dae2223b00361a3),pk(@0/<0;1>/*)))
```

and for the dangerous same-width pair, which differ by one character in the
engraved string and decode distinguishably:

```
sha256  -> md1yqpqqxpye4mwncz6lc7qw0afq5zsx7tkhzygasqds689sh02wtgy8lv8eqq9dd9qds44ultvfgf4g
           wsh(and_v(v:sha256(ba78…15ad),pk(@0/<0;1>/*)))
hash256 -> md1yqpqqxpye4lwncz6lc7qw0afq5zsx7tkhzygasqds689sh02wtgy8lv8eqq9dd9qcr8cx6zu6znsl
           wsh(and_v(v:hash256(ba78…15ad),pk(@0/<0;1>/*)))
```

(binary: `/scratch/code/shibboleth/descriptor-mnemonic/target/release/md`;
encoder `crates/md-codec/src/tag.rs:132`, renderer `src/render.rs:199-202`.)

So the operator's stated fear — *"the difference in number of hashes means it's
easy to have a user fail to record info needed to unlock coins"* — does **not**
convert into a lost wallet, because the kind was never something the user had to
record: the codec records it, on the plate that carries the policy. **0 Critical.**

What the lens *does* find is the other half of that sentence. The kind is
recorded in exactly one place, and it is the one place the device explicitly
tells the operator to store **away** from the artifact that carries the secret:

> `composerCopyPreimageKeepApart()` — *"Keep each preimage plate apart from the
> policy plates and from the others."* (`gui/composer_copy.go:782-784`)

After this spec, the separated half is a plate that states its derivation in
plain language, claims a version number, and is silent about one of the two
steps. That is where the four Important findings live.

---

## 1. What each engraved artifact physically carries

| artifact | carries the digest? | carries the KIND? | where / encoder |
| --- | --- | --- | --- |
| **md1 policy / template plate** | yes — the hash node's body, 32 or 20 bytes | **YES**, as the wire tag `0x1D/0x1E/0x1F/0x20` | `md-codec` `src/tag.rs:132`; recovered by name via `md decode` (run above). Plate title mark `PREIMAGE REQUIRED` when any path is hashed — `gui/composer_preimage_plate.go:490-495` |
| **mk1 key card** | no | no | key + stub only; same `PREIMAGE REQUIRED` mark (`gui/composer_flow.go:478-481`) |
| **ms1 secret share** | no | no | seed words; the mark is explicitly excluded for `cardMS1` (same comment) |
| **H6 preimage plate — string form** | abbreviated, 16 of 64 hex | **no** | body = `codex32.EncodeMS1Preimage(X)`; locator = `hashlockPlateLocator` (`gui/composer_preimage_plate.go:392-408`): `path N` / `hash  <first8>..<last8>` / `mk1 stub (…): xxxxxxxx` |
| **H6 preimage plate — phrase form** | abbreviated, same row | **no** | phrase verbatim + `hashlock.MethodLine(hardened)` (`hashlock/hashlock.go:176-182`) + the same locator; optional QR of `hashlock.QRText` (`:188-190`) |

The mk1 **stub** printed on the preimage plate's locator commits to the kind
(it hashes the template chunks), so it can *confirm* a guessed kind — but only
for someone who already holds the rest of the template, i.e. the md1. It is not
an independent record of the kind.

## 2. The restore document

Three measured facts, none of which the spec mentions.

1. **The composer emits no restore document at all.** `multisigRestoreDocFlow`
   has exactly two production callers — `gui/multisig.go:377` (supply) and
   `gui/multisig_build.go:592` (build). `grep -c 'restoreDoc' gui/composer*.go`
   → **0** across every file. `composerEngraveStep`
   (`gui/composer_flow.go:372-487`) ends at `bundleEngrave` and returns. So the
   flow that *authors* hashlocks produces a census **before** the cut and nothing
   after it.
2. **On the supply path a hashlocked md1 reaches the restore doc as "complex".**
   `classifyPolicy` (`md/md.go:1266`) matches only `multi`/`sortedmulti` under a
   wrapper; `wsh(and_v(v:<hash>,…))` falls through to `PolicyComplex`
   (`md/md.go:1327`) → `renderable=false` (`md/md.go:1368`) → `expandUnsupported`
   (`gui/md1_expand.go:177`). The document therefore reads, in full:
   `Wallet policy (read-only):` / `wsh complex` /
   `Addresses unavailable for this policy shape.` No hashlock, no digest, no
   kind, no statement that a preimage is required.
3. **`gui/multisig_restore.go` contains zero occurrences of `hashlock` or
   `preimage`**; `gui/multisig_build_census.go` contains one, and it is a comment
   citing F-132 as the *precedent for the passphrase lines*
   (`multisig_build_census.go:299`).

And the spec's word **"restore" appears once in 494 lines** — line 17, a
background clause ("can be encoded, engraved, restored and derived today").
"engraved" appears once, on the same line.

**The analogous case the brief asks for is already built, and its design rule is
written down.** `buildPassphraseInventoryLines`
(`gui/multisig_build_census.go:288-395`) exists because a BIP-39 passphrase is a
required spending factor absent from every plate. Its header says so, and names
this exact class:

> *"F-132's device sibling exactly: that finding was a hashlock preimage required
> to spend, absent from the backup and unmentioned by it."*

> *"This document is read years later, alone, often by someone who was not the
> operator, holding a pile of steel and asking one question: is this everything?
> … Silence leaves the reader unable to distinguish a complete backup from one
> whose operator forgot to write the passphrase down, and that is the state in
> which people give up on a recovery that would have worked."*

Both arms speak — including the negative one ("No BIP-39 passphrase was used").
The hashlock has neither arm.

## 3. Single-artifact-loss matrix

Composer run, template form, Full mode, one hashed path, phrase-form preimage
plate. Each row = the operator holds **everything except** that artifact.

| lost artifact | can they reconstruct the script? | can they spend? | is the KIND the reason? |
| --- | --- | --- | --- |
| **md1 policy / template** | **No.** The template, wrapper, threshold and key seating are gone. mk1 cards carry keys, not a policy; the preimage plate carries a path *number* and 16 hex of a digest. | No | **No.** The policy is gone for reasons that have nothing to do with hash kinds. The kind is collateral, not cause. |
| **mk1 key cards** | Yes if form A (keyed md1 carries the keys). No in the template form. | form-dependent | No |
| **ms1 secret share** | Yes — policy and kind both intact | No (no signing key) | No |
| **H6 preimage plate** | Yes — and the kind is right there in the md1 | No, unless the phrase is remembered | No |
| **nothing lost** | Yes | Yes | — |

**The interesting cell the brief names — preimage plate survives, md1 does not —
is already dead before the kind is reached.** There is no cell in which the kind
is the load-bearing loss. That is the whole reason this round reports no
Critical, and it is a property of the md1 codec, not of this spec.

What the matrix does **not** capture, and what findings I-1..I-3 are about, is
the *degraded* state: an operator who holds the preimage plate and can read it by
eye, but cannot decode the md1 (no `md`, a generic bech32 reader, or simply
reading the steel). They get a complete-*looking* derivation statement that is
one step short.

## 4. Is the kind brute-forceable when lost? Yes — and nothing says so

Four candidates. Given the preimage X (which F1 fixes at 32 bytes for **all
four** kinds) plus any one of: the full digest, the plate's 16-hex `hash` row,
the mk1 stub, or the funded address — the operator computes four digests and
compares. Microseconds. F1, the very fact §4 uses to close the plate surface, is
what makes the recovery cheap.

**Nothing on the steel, in the CLI help, in `ms hashlock`'s stderr card, or in
the spec mentions that this is possible.** Measured: `crates/ms-cli/src/cmd/hashlock.rs`
(447 lines) has no such text, and the spec's §12 item 5 asks only that
`ms hashlock` *reproduce* a digest — a verb that presumes the kind is already
known. A recovery that exists and is undocumented is one most people will not
find, which is the brief's own definition of Important. See **I-2**.

## 5. `ms hashlock` as the verification path

ACCEPTANCE item 1 (`design/ACCEPTANCE_hashlock_H6_preimage_plates.md`) has the
operator run `ms hashlock --hashlock-phrase-stdin` / `--in` over a plate and
match the digest. Spec §9 phase 2: *"`ms hashlock` learns the kind."* §12 item 5:
*"`ms hashlock` reproduces a non-sha256 plate's digest."*

The operator's inputs at that moment are exactly the plate's contents
(`crates/ms-cli/src/cmd/hashlock.rs:42-98`: `--hashlock-phrase-stdin`, `--hex`,
`--in FILE`, positional `MS1`, `--method`). **The plate carries no kind**
(§1 above). So the answer to "where does the operator get the kind to pass?" is
**the md1 card, or memory**, and the spec says neither. See **I-2**.

## 6. What the operator is TOLD before steel

| surface | says the kind exists? | says to keep it? |
| --- | --- | --- |
| §8i entry modal | no — §7.2 keeps the entry body kind-generic, deliberately | no |
| §8i consent modal | **yes** — §7.2 gives the consent body `<kind> of a 32-byte value` | no. It is a rule about the *preimage*, not an instruction to record anything |
| §8.3 census rows | **no** — `hashlockPlateFormWords` (`gui/composer_preimage_plate.go:205-216`) renders the *preimage method*: `phrase, hardened, QR` / `phrase, sha256` / `preimage string`. `composerCopyPreimagePlateRow` (`composer_copy.go:765-768`) is `path %d  %s  %s` | `composerCopyPreimageKeepApart` tells them to keep the plate **apart from the policy plates** — i.e. apart from the only record of the kind |
| plate inventory / restore doc | **not reached at all** from the composer (§2 above) | — |
| `me bundle` checklist | **no** — `crates/me-cli/src/bundle.rs` (v0.9.0): zero occurrences of `hashlock`/`preimage`. Verified by running it on the ripemd160 md1 above: the checklist prints `plate 1/2 md1 policy` and `plate 2/2 ms1 secret` and nothing else | no |

Against the BIP-39-passphrase comparison: a passphrase gets a dedicated
restore-doc function, both arms, per-seed fingerprints, and a distinct
"was/was not used" sentence. The hash kind, after this spec, gets one modal
sentence at consent and nothing durable.

## 7. Does §4's scope boundary hide this? Yes — see I-4

---

# Findings

## I-1 (Important) — §8.6's method line and QR text become an incomplete definition of the derivation, and the spec never touches either

**Sequence.** Operator composes `hash256` from a phrase, hardened method. Cuts a
phrase-form preimage plate with a QR, plus the md1 set. Five years later, no
device. They read the preimage plate:

```
HASHLOCK PHRASE
correct·horse·battery·staple
method: pbkdf2-hmac-sha256 iterations=100000 salt=ms-hashlock-v1 dklen=32
path 1
hash  3cf5d421..b70a4c12
mk1 stub (policy): a1b2c3d4
NOT A SEED
```

They reproduce X exactly as the plate instructs. They compute `sha256(X)` — the
only thing "hashlock" has ever meant on this device — and get a value that is not
the `hash` row. The plate is now self-contradicting and they have no instruction
that resolves it. (They recover: the md1 has the kind, or four tries. Hence
Important, not Critical.)

**Why this is a defect of *this* spec and not a pre-existing one.** The plate's
design rule is written into the function that builds the line, verbatim
(`hashlock/hashlock.go:162-175`):

> *"A WIRE RECORD CARRIES A SELECTOR AND A PLATE CARRIES THE DEFINITION, and the
> difference is the whole reason this string is long. `phrase:` carries
> `hardened` because it is read by a tool that already knows the parameter set;
> **a plate is read years later by a person who may have neither the tool nor this
> firmware, so it spells the parameters out.**"*

Today that promise is kept, because `MethodLine` + the digest row *do* determine
H: `H = sha256(X)` is the only possibility. This spec makes it four, and the
plate keeps the same sentence. That is the spec's own framing of the risk in its
brief — *"unloseable information" becoming "metadata that can be lost"* —
landing on the one surface designed to be self-describing.

**The QR is worse than the text, because it claims a version.**
`QRText(hardened, phrase)` = `"hashlock v1\n" + MethodLine + "\nphrase: " + phrase`
(`hashlock/hashlock.go:188-190`), ported byte-for-byte from
`ms_codec::hashlock::qr_text` and pinned cross-language in
`hashlock/testdata/hashlock-v0.8.json` under a `qr_text` array (verified: 4+ rows,
e.g. `"hashlock v1\nmethod: sha256\nphrase: correct horse battery staple"`, with
the note *"the middle line IS the selector"*). After this spec, **one `hashlock
v1` string names four different constructions**, and the corpus row's own note is
false: the middle line is no longer the selector.

**MEASURED: the remedy is free.** Run against the fork's own encoder
(`github.com/seedhammer/kortschak-qr v0.3.2`, ECC-L, the 100-character
worst-case phrase under the hardened method):

| QR text | bytes | modules |
| --- | --- | --- |
| shipped worst case | 194 | **53** |
| `+ "hash: ripemd160\n"` | 210 | **53** |
| `+ "hash: hash256\n"` | 208 | **53** |
| `+ "kind: ripemd160\n"` | 210 | **53** |

53 is exactly `hashlockQREnvelope` (`backup/hashlock.go:70-73`), the size the
layout already reserves. So adding the kind costs **no** module growth, **no**
layout change, and **no** change to ACCEPTANCE item 8's measured 32m12s
constant-time QR cut. The optical gate that item 8 exists to run is unaffected.

**What the spec should do:** bring `hashlock.MethodLine` / `ms_codec::hashlock::qr_text`
into scope, state the plate's new line, and add the corresponding `qr_text` rows
to `hashlock-v0.8.json` (which §11 already lists for the digest KAT, so the pin
move is already in the cycle). Whether the QR format string stays `hashlock v1`
or becomes `v2` is a decision the spec must *make*, not inherit.

## I-2 (Important) — the plate-verification path needs a kind the plate does not carry, and the free recovery is unspecified

**Sequence.** The operator follows ACCEPTANCE item 1 to the letter: plate in one
hand, `ms hashlock` on a laptop. They pipe the phrase in. Under this spec the
command wants a kind; the plate has none; the digest they get back is one of four
and they cannot tell which is right from the plate alone. They must fetch the md1
plate — the plate the device told them to store somewhere else — and decode it,
to run a verification whose whole purpose was to check the plate.

**What I verified.** `crates/ms-cli/src/cmd/hashlock.rs:42-98` — every input is
plate-shaped (`--hashlock-phrase-stdin`, `--in FILE`, positional `MS1`, `--hex`,
`--method`); `--method` is the *preimage* axis (`Method::{Hardened, Sha256}`,
`:34-40`), not the hashlock kind. `crates/ms-codec/src/hashlock.rs:60-64` —
`digest()` is unconditional SHA-256 today. Output is
`format!("hash:{}", hex(&h))` (`cmd/hashlock.rs:326`).

**The fix is four hash calls.** F1 fixes the preimage at 32 bytes for every kind,
so with no kind given `ms hashlock` can print all four digests, and the operator
matches one against the plate's own `hash  first8..last8` row. That turns the
brief's question from a guess into a lookup, on the exact artifact in their hand,
with no md1 required. The spec neither requires nor forbids this; §9 phase 2 says
only *"`ms hashlock` learns the kind"*, which reads as "gains a `--kind` flag" and
leaves the operator supplying information the plate does not have.

**What the spec should do:** say what `ms hashlock` prints when the kind is
*unknown*, and make §12 item 5 assert the plate-only route rather than the
kind-already-known route.

## I-3 (Important) — the census row describes four constructions identically, on the operator's only inventory of what they must store apart

**Sequence.** A policy with two hashed paths sharing one preimage under two
different kinds (legal after §5 re-keys `hashlockHeld` on the whole `HashLock`).
`composerPreimagePlates` dedupes by **digest** (`gui/composer_preimage_plate.go:78`,
`seen := map[[32]byte]bool{}`), and the two kinds give two digests, so the census
lists **two plates**:

```
Plus 2 preimage plate(s), cut first and NOT part of this backup:
path 1  3cf5d421..b70a4c12  phrase, hardened
path 2  9a2db2e2..1821af885 phrase, hardened
Keep each preimage plate apart from the policy plates and from the others.
```

Both plates carry the **same phrase** and the **same method line**. They differ
in 16 hex characters and a path number. The operator stores two apparently
identical bearer plates in two places, with no plain-language statement anywhere
of why there are two or what distinguishes them.

**What I verified.** `hashlockPlateFormWords` (`gui/composer_preimage_plate.go:205-216`)
returns the *preimage method*, never the hashlock kind.
`composerCopyPreimagePlateRow` (`gui/composer_copy.go:765-768`) is
`"path %d  %s  %s"`. `composerCopyPreimagePlateHeading`, `…KeepApart`,
`…CensusScope` (`:736-784`) say nothing about kinds.

**And the spec names none of these surfaces.** Measured over all 494 lines,
occurrences of each identifier in `SPEC_hashlock_kinds.md`:

| identifier | occurrences |
| --- | --- |
| `hashlockPlateFormWords` | 0 |
| `hashlockPlateLocator` | 0 |
| `composerCopyPreimagePlateRow` | 0 |
| `MethodLine` | 0 |
| `QRText` / `qr_text` | 0 / 0 |
| `backup/hashlock` | 0 |
| `composerCopyPreimage…` (any) | 0 |
| `multisig_restore` | 0 |
| `buildPlateInventoryLines` | 0 |
| `hashlockFirst8Last8` | **1** |

The single hit is in §11's gate table and is there for an *arithmetic* reason —
its `[32]byte` parameter — not a recording one. §11 is otherwise complete and
accurate: I independently confirmed the "six hardcoded `[56:]` sites" claim
(4 Go: `gui/composer_hash.go:50,173,188`, `gui/composer_consent.go:63`; 2 Rust:
`crates/me-cli/src/main.rs:2277,2602`) and that `hashlockFirst8Last8` is indeed
invisible to that grep. The gate table is a good list of *arithmetic* that must
move. It is not a list of *statements to the operator* that must move, and this
spec has both kinds.

## I-4 (Important) — §4 uses F1 to close a question F1 does not answer

**The text.** §4: *"**Out:** The H6 preimage plates and the phrase→**preimage**
derivation, **by F1**. The preimage stays `sha256(phrase)`, 32 bytes, for every
kind."*

**F1 is true, and it settles three things:** the plate's *layout* (32 bytes at
every kind, so `backup.Hashlock` needs no geometry change), the `OP_SIZE <32>`
script rule, and §8i's "32-byte value" sentence. §3's own wording confirms that
is the scope of the claim: *"This is what keeps the H6 preimage plates and the
§8i '32-byte value' rule **structurally** intact."*

**It does not settle whether the plate SET records the kind**, which is a
question about *text*, not *structure*. §4 cites F1 once and closes the entire
plate surface, and the measurement in I-3 shows the consequence: not one
plate-text function appears anywhere in the document.

The spec already knows how to handle a boundary that a review found
contradicted — §4's C-2 paragraph is *"stated twice on purpose"* and spells out
precisely which half of the phrase route is in scope and which is out. The same
treatment is owed here: **F1 keeps the plate's layout and the 32-byte rule out of
scope; it says nothing about what the plate says, and after this cycle the plate
says less than it needs to.**

This is the structural finding behind I-1 to I-3, and it is why three correctness
rounds could not reach them: every one of those rounds was reviewing a document
whose own scope statement said the plates were settled.

## I-5 (Important) — F-132's open half is widened without being named

F-132 (`design/FOLLOWUPS.md:4298`) is **"PLATE HALF CLOSED 2026-09-06 (hashlock
H6) — the hashlock preimage is required to spend, absent from the backup, and
unmentioned by it"**, and its open half is stated verbatim:

> *"**WHAT STAYS OPEN is the RESTORE-DOC half**, which is where this item was
> filed from and which H6 deliberately does not touch: `buildPlateInventoryLines`
> (`gui/multisig_build_census.go`) still does not state that the policy contains a
> hashlock, name which branches it gates, or say the preimage is not in the set."*

This spec adds a **second** factor to the set that surface is already silent
about, and **`SPEC_hashlock_kinds.md` mentions F-132 zero times** (grep: the only
follow-ups named are F-507 and F-150). The silence also now extends further than
F-132 recorded, in a way worth writing down while it is measured:

- `gui/multisig_restore.go`: 0 occurrences of `hashlock`/`preimage`.
- **The composer emits no restore document at all** (§2.1 above) — so for the
  flow that authors hashlocks, the open half is not "a missing sentence in a
  document", it is "a missing document".
- `crates/me-cli/src/bundle.rs` (v0.9.0): 0 occurrences — the host-side
  checklist is hashlock-blind too, and phase 3 opens that crate anyway.

**I am not re-opening the closed plate half.** The finding is narrow and cheap:
§4 or §13 must record that after this cycle the hash kind joins the preimage as a
required spending factor recorded **only** in the md1 wire bytes, and F-132's
entry must say so, so the operator-journeys phase that owns it inherits both
factors rather than one. Without that sentence this spec is the second cycle in a
row to walk past the same gap without leaving a mark on it.

---

# Minor

- **M-1 — a fifth decode-side display the §7.4 consumer list does not name.**
  `policySummaryLines` (`gui/template_engrave.go:186-188`) appends `" +hashlock"`
  for any branch with `b.Hashlock`, which is all four kinds. It keys off
  `Hashlock` and not `len(Sha256Digests)`, so §7.4's correctness argument holds
  and **no code change is required** — but it is an operator-facing consent
  surface that will describe four constructions identically. §7.4 says "Four
  production consumers, not two"; this is a fifth *reader* of the same field,
  and naming it costs a table row.

- **M-2 — `me bundle`'s checklist is hashlock-blind.** `crates/me-cli/src/bundle.rs`:
  0 occurrences of `hashlock`/`preimage`. Verified by execution against the
  ripemd160 md1 built above: the manifest reports `md1 policy` + `ms1 secret` and
  says nothing about a required preimage or a hash kind. Host-side sibling of
  I-5; phase 3 already opens this crate.

- **M-3 — the locator's layout cost is *not* free the way the QR's is.**
  `backup.EngraveHashlock` auto-fits down `FontSizes` and refuses with
  `ErrHashlockTooLarge` (`backup/hashlock.go:78-86`). The QR measurement in I-1
  shows the QR text is free; a kind token added to the **printed locator** has an
  unmeasured layout cost at the worst rung. If I-1/I-3 are folded, the plan must
  measure the locator the way §7.5 insists the *screen* row be measured — same
  argument, different surface, and §7.5 currently scopes it to the screen only.

# Nit

- **N-1** — the brief's "473 lines" is stale: the committed blob at `2a0da99f` is
  **494** lines. "restore": 1 occurrence, confirmed on the committed blob, not
  the working tree.

---

# Counts

**0 Critical / 5 Important / 3 Minor / 1 Nit.**

# Verdict

**NOT GREEN under this lens** — no funds-loss path exists, because the md1 wire
tag records the hash kind and `md decode` returns it by name (machine-verified);
but the spec closes the entire engraved-plate surface on F1, which is a fact about
the preimage and not about what the plates *say*, and as a result this cycle
makes the preimage plate's own stated derivation incomplete, leaves the
plate-verification command asking for a kind the plate does not carry, and adds a
second lost factor to a restore surface F-132 already has open — for a QR-text
cost measured at exactly **zero** modules.
