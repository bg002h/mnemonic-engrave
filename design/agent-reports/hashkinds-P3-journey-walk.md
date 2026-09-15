# Journey walk — hashlock kinds, the three built halves of the chain

**Lens:** operator journey (not correctness). The question is *what does a real
person have in hand at each step, and what happens when they reasonably do
something else* — findings are **missing things at moments**, not wrong things
in sections.

**Artifacts under walk**

| tool | repo | revision |
| --- | --- | --- |
| `md compose` | descriptor-mnemonic | `origin/main` `65eab419` |
| `ms hashlock --kind` | mnemonic-secret | `origin/master` `9dcd2e0` (ms-cli 0.19.0) |
| `me sysw pack` / `show` | mnemonic-engrave | branch `hashkinds-p3` `2e482d19` |
| the device | seedhammer fork | `main` `0562e81` — **phase 4 NOT BUILT** |

All four worktrees were clean before and after. No tracked file was modified.
Scratch lived in `/scratch/code/shibboleth/.tmp/hk-journey/`.

**Toolchain:** `PATH=/home/bcg/.cargo/bin:$PATH`, `cargo build --locked` in each
Rust repo; Go 1.26.7 from `/scratch/code/shibboleth/.toolchain/go`.

**Correction to the dispatch brief, recorded because phase 4 will need it.** The
brief cites the shipped parser as
`third_party/seedhammer/sysw/composer_records.go` in the mnemonic-engrave
checkout. **That file does not exist there.** The submodule is pinned at
UPSTREAM `713aee2e` (`v1.4.2`), which has no `sysw` package at all — it is
vendored for `me bundle --preview` curve math, exactly as `CLAUDE.md:7` says.
The device source walked here is the fork checkout,
`/scratch/code/shibboleth/seedhammer`, at `main` `0562e81`. No design document
in this repo makes the wrong citation; only the brief did.

---

## Journey A — the full chain for `ripemd160`

### A.1 The operator derives the preimage first

They have: a phrase in `phrase.txt` (`correct horse battery staple`), and an
intention to build a wsh wallet with a key-less `ripemd160` escape path.

```
$ ms hashlock --hashlock-phrase-stdin --kind ripemd160 < phrase.txt
```

stdout (the record):

```
hash:ripemd160:09e7bb5051d89788fb4e4b374126721dbcc2946b
```

stderr (the engraving card), verbatim:

```
THIS CARD CARRIES THE PREIMAGE -- the secret. stdout carries only the public digest.
digest:          09e7bb5051d89788fb4e4b374126721dbcc2946b
for md compose:  --path ... ripemd160=09e7bb5051d89788fb4e4b374126721dbcc2946b
                 requires `ripemd160=` support in `md compose`. If it answers "unknown option `ripemd160`", that support has not shipped in your `md` yet -- the preimage and digest above are still correct, and `md` is what has to catch up.
preimage (ms1):  ms10h ashsq 0p7ja f9gsj jpkjv ll2l2 74w8a 388xg qzlew p73sc ptwxg tjugs pvs8t klufg 89hqj
preimage (hex):  c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016
method:          preimage = PBKDF2-HMAC-SHA256(password = phrase, salt = "ms-hashlock-v1", iterations = 100000, dkLen = 32)
phrase:          28 characters -- write the method line AND the hash line (ripemd160) next to your phrase unless the phrase is cut on a HASHLOCK PHRASE plate, which carries both; if the method line is lost, try each method that shipped with the version named on this card (ms-cli 0.19.0), and if the hash line is lost, re-run with no --kind and match the digest against your descriptor
The preimage must be exactly 32 bytes (64 hex characters) for every kind: the script checks OP_SIZE 32 before OP_RIPEMD160 (composer spec §8i, F-132).
One phrase per policy. Spending any path of a wsh wallet publishes this digest. Never use this phrase as a passphrase or a password anywhere else -- a spend publishes the preimage, and anyone can then test guesses at the phrase itself.
source:          phrase (stdin)
the record on stdout needs `ripemd160` support in `me sysw pack`. If it asks for "exactly 64 hex characters", that support has not shipped in your `me` yet -- the record is correct and `me` is what has to catch up. Do not reshape the record to satisfy it.
```

**What the operator now believes.** Two named consumers might not be ready:
`md compose` and `me sysw pack`. Both caveats are conditional and tell them
exactly what the refusal would look like. **The device is not mentioned.**

Both named consumers *are* ready at these revisions — verified below. So the
operator runs the chain, meets neither refusal, and has been handed a mental
model in which the readiness question is **closed**.

### A.2 Compose the wallet

```
$ md compose --wrapper wsh --path '2of3' \
      --path 'keyless,ripemd160=09e7bb5051d89788fb4e4b374126721dbcc2946b'
md: this policy needs --experimental:
  path 2 has no key (bearer access to whoever holds the preimage)
$ echo $?          # 1
```

Reasonable divergence, and a good refusal. With the flag:

```
$ md compose --wrapper wsh --experimental --path '2of3' \
      --path 'keyless,ripemd160=09e7bb5051d89788fb4e4b374126721dbcc2946b'
warning: EXPERIMENTAL: path 2 has no key (bearer access to whoever holds the preimage)
wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*),ripemd160(09e7bb5051d89788fb4e4b374126721dbcc2946b)))
note: stdout is a keyless descriptor template (no keys)
exit 0
```

`ripemd160=` is supported, as the ms card allowed for. Nothing here mentions
which machines can engrave the result — correctly so; see *Not acting on*, D-2.

### A.3 Pack the payload

The realistic composer payload — one cosigner key plus the hash record:

```
$ me sysw pack --in A_records.txt --no-passphrase --out A_full.bin
sealing:  NOT SEALED — no record in this payload is secret material, so there
      is nothing to encrypt. The container is cleartext: anyone holding the file
      can read it.
me: appended now:1789497639 as the last record (the pack time, a lower bound the device echoes next to a time lock; it is never a locktime). Pass --no-now to omit it, or supply your own now: record to pin a different bound. Payloads without a key:/hash: record get none unless --now is passed.
strength: no passphrase — BELOW the threshold
digest:   a70a d948 6433 9e67 5290 2c37 0562 41ff
          re-print it with: me sysw show A_full.bin
exit 0
```

**This is the last host moment, and it is silent about the kind.** Four lines
print; none is about the `ripemd160` record. `pack` is demonstrably willing to
speak on this run path — it printed an unprompted `now:` note in the same
output, and it carries a family of `me: note —` and `me: WARNING —` lines for
other conditions.

### A.4 Inspect it

```
$ me sysw show A_full.bin
sealed:   false
pub_len:  353
ct_len:   0
identity: ffb1766a4c84feb5219c9c05398e02c4bbf3a7fedbfab1eba355eb088c68c019
digest:   a70a d948 6433 9e67 5290 2c37 0562 41ff
public record 0: cosigner key (key:) — [73c5da0a/48'/0'/0'/2']xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf
public record 1: ripemd160 hashlock (hash:) — 09e7bb50..bcc2946b
exit 0
```

`show` **names the kind**. This is the P1-journey F-C1 fix working: it no longer
says "sha256 hashlock" under every kind. It does not say whether a device can
read it.

### A.5 The bytes on the wire

```
$ xxd A_payload.bin | tail -4
00000030: 0000 0000 6861 7368 3a72 6970 656d 6431  ....hash:ripemd1
00000040: 3630 3a30 3965 3762 6235 3035 3164 3839  60:09e7bb5051d89
00000050: 3738 3866 6234 6534 6233 3734 3132 3637  788fb4e4b3741267
00000060: 3231 6462 6363 3239 3436 62              21dbcc2946b
```

The tag reaches the wire, as phase 3's round-1 fold intended.

### A.6 What the device makes of it — MEASURED, not read

Built an out-of-tree probe (`/scratch/code/shibboleth/.tmp/hk-journey/goprobe`,
a `replace` onto the fork; nothing in the fork was touched) and called the
device's own exported entry point `sysw.Classify`:

```
key:5b37336335646130612f3438272f30272f30272f32...  -> ClassKey
hash:ripemd160:09e7bb5051d89788fb4e4b374126721...  -> ClassUnknown (INERT)
hash:hash160:b5b72c0e6896ff59dfa99e0d1052a9c02...  -> ClassUnknown (INERT)
hash:hash256:98a20fc25dbcdf236fb0307e3f82cad47...  -> ClassUnknown (INERT)
hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba9...  -> ClassHash
now:31373839343937363339                           -> ClassNow

door counts: keys=1 seeds=0 preimages=0 inert=3
```

And directly against `ParseHashRecord` (`sysw/composer_records.go:192`):

```
hash:ripemd160:09e7bb…  parse=sysw: hash: must be exactly 64 lowercase hex characters
hash:hash256:2d711642…  parse=sysw: hash: must be exactly 64 lowercase hex characters
hash:sha256:2d711642…   parse=sysw: hash: must be exactly 64 lowercase hex characters
hash:2d711642…          parse=<nil>  digest=2d711642
```

So on a shipped device, Journey A's payload gives the composer door exactly two
lines (`gui/composer_door.go:45-96`, `gui/composer_copy.go:270,291`):

```
Keys loaded: 1
1 payload record was not understood.
```

It does not name the record, the prefix, or the reason. And the hash step's
only fallback is the 64-hex pad (`gui/composer_hash.go:78-95`), which shows
`N of 64 hex` and reveals the OK button only at exactly 64 — **a 40-character
`ripemd160` digest cannot be entered at all.** The operator is stopped, not
misled. Fail-closed, as `SPEC_hashlock_kinds.md:188` intends.

### A.7 The one place in the whole chain that says anything

`me sysw pack --help`, in the kind paragraph:

> omit it for `sha256` (64 hex, **the form every shipped device reads**), or
> write `hash:hash256:` (64 hex), `hash:ripemd160:` or `hash:hash160:` (40 hex).

True, verified, and the only statement of it anywhere. It is in a `--help`
body an operator reads once, months before; it is phrased as a property of
sha256 rather than as a consequence for the other three; and nothing on any
run path repeats it.

**Journey A verdict.** The operator composes, derives, packs and inspects a
`ripemd160` wallet with four exit-0s and zero mention of the device, then
discovers at the machine — with plates planned — that one record "was not
understood".

---

## Journey B — the same for `sha256`

```
$ ms hashlock --hashlock-phrase-stdin --kind sha256 < phrase.txt
hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
```

Card (stderr), the differing lines only:

```
digest:          3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
for md compose:  --path ... sha256=3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
```

**The `md compose` caveat line and the closing `me sysw pack` notice are both
ABSENT.** Confirmed at source: both are gated on
`args.kind.is_some_and(|k| k != HashKind::Sha256)`
(`crates/ms-cli/src/cmd/hashlock.rs:473, :566`).

`ms` also normalises on output: `--kind sha256` emits the **bare** record, never
`hash:sha256:`. Producer rule, working.

`me sysw pack` normalises input too. Fed `hash:sha256:<64hex>`, the wire carries:

```
00000030: 0000 0000 6861 7368 3a32 6437 3131 3634  ....hash:2d71164
```

— the bare form, which the device parses (`parse=<nil>`). Verified: an operator
who writes the explicit sha256 tag is *not* punished with an inert record.

### Is the difference visible to the operator?

Yes, at exactly one place: **the ms card**, which prints two extra caveat lines
for a non-sha256 kind and none for sha256. Downstream the two journeys are
indistinguishable — `me sysw pack` produces byte-identical output shapes, and
`me sysw show` differs only in the kind token it names.

**So the whole of the operator-visible difference between "works end to end" and
"inert on every device that exists" is two `ms` lines that name the two tools
that DO work and omit the one that does not.** That is finding J-2.

---

## Journey C — the operator who does not know kinds exist

They type what they always typed.

```
$ ms hashlock --hashlock-phrase-stdin < phrase.txt
hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
```

**stdout is unchanged.** The card gained two things:

1. the `phrase:` line now reads `write the method line AND the hash line
   (sha256) next to your phrase … which carries both; … and if the hash line is
   lost, re-run with no --kind and match the digest against your descriptor`
   (was: `write the method line next to your phrase … which carries it`);
2. a new trailing block:

```
no --kind given; stdout carries the sha256 record. This preimage's digest under each kind:
  sha256     3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
  hash256    98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488
  ripemd160  09e7bb5051d89788fb4e4b374126721dbcc2946b
  hash160    b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd
```

Pack and show:

```
$ me sysw show B_tagged.bin
public record 0: sha256 hashlock (hash:) — 2d711642..921a4881
```

**Verified identical to pre-cycle.** `git diff master..hashkinds-p3 --
crates/me-cli/src/main.rs` shows the only change to this print site is
`"sha256 hashlock"` → `"{} hashlock", h.kind().token()`, and
`RecordHashKind::Sha256 => "sha256"`
(`crates/me-cli/src/sysw/composer_records.rs:139`). No print site was added or
removed on the untagged path. The wire bytes are unchanged (§6 producer rule).

**Journey C verdict: clean.** The requirement "it must not change for them" is
met on stdout, on the wire, and in `me`. The only change is a longer stderr
card in `ms`, which is additive and is the mechanism Journey D depends on.

---

## Journey D — coming back later, with a payload file and a plate

They have `A_full.bin` and an ms1 preimage plate. Nothing else.

**Which kind does the payload commit to?** Three independent answers, and they
agree:

```
$ me sysw show A_full.bin
public record 1: ripemd160 hashlock (hash:) — 09e7bb50..bcc2946b

$ ms hashlock --in D_plate.txt
…
no --kind given; stdout carries the sha256 record. This preimage's digest under each kind:
  sha256     3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
  hash256    98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488
  ripemd160  09e7bb5051d89788fb4e4b374126721dbcc2946b
  hash160    b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd
```

plus the descriptor itself, which spells `ripemd160(09e7…)` in plain text.
Matching `show`'s `09e7bb50..bcc2946b` against the `ripemd160` row is a
first-8/last-8 comparison anyone can do. **This journey works well** and the
four-digest lookup is precisely the right affordance for it — it survives the
loss of the kind note, which is the failure the card's own fallback sentence
anticipates.

**Can they tell whether their device can read it?** No. `me sysw show --help` is
two lines and carries no kind vocabulary at all; the only statement of device
readability in the entire chain lives in `me sysw pack --help`, a verb they are
not running. This is J-1 seen from the other end, plus J-5.

---

## Findings

### J-1 — Important — **warning**

**Nothing on any run path tells the operator that a kind-tagged `hash:` record
is inert on a device without kind support. `me sysw pack` is the last moment it
could, and it says nothing.**

Measured: the record reaches the wire tagged (A.5); `sysw.Classify` returns
`ClassUnknown` (A.6); the device's only signal is the door's generic
`1 payload record was not understood.` The host prints four lines at pack and
none is about the kind.

`SPEC_hashlock_kinds.md:188` specifies this behaviour and is right to — the
rejected alternative (an old parser reading a non-sha256 digest *as* sha256)
composes an unspendable wallet silently. The spec never asks whether the host
should **say so**, which is why five correctness rounds had no reason to find it.

**Why the wrong outcome is worse than telling them nothing.** Silence at pack
reads as assent, and the operator's next action is to carry the payload to a
machine — the point at which plates get planned and steel gets cut. The
recovery is a full re-pack, and the diagnostic they are given at that point
names neither the record nor the reason.

**This does not expire with phase 4.** Phase 4 makes *upgraded* firmware read
the tag; every device not yet flashed keeps treating it as inert, forever. A
note phrased as a property of the firmware rather than of the calendar — *this
record's kind tag makes it inert on firmware without hashlock-kind support; such
a device counts it as "not understood" and builds no hashlock path from it* —
is durably true and needs no version claim.

Precedent for the shape exists in the same output: `me sysw pack` already prints
`me: note — this payload holds no 'hash:' record, so nothing here says which
policy the preimage unlocks…` (`crates/me-cli/src/main.rs:2606`) for a strictly
less consequential condition.

**Owning phase: P3** (the note belongs where the record is produced), or P4 at
the latest, but it is a host change either way.

### J-2 — Important — **warning**

**`ms hashlock --kind <non-sha256>` names the two consumers that DO support the
kind and omits the one that does not. The asymmetry converts silence into
assent.**

The card prints, for every non-sha256 kind:

- `crates/ms-cli/src/cmd/hashlock.rs:479` — *"requires `ripemd160=` support in
  `md compose`. If it answers 'unknown option `ripemd160`', that support has not
  shipped in your `md` yet"*
- `crates/ms-cli/src/cmd/hashlock.rs:568` — *"the record on stdout needs
  `ripemd160` support in `me sysw pack`. If it asks for 'exactly 64 hex
  characters', that support has not shipped in your `me` yet … Do not reshape
  the record to satisfy it."*

Both were written as conditionals so that phases 1 and 3 shipping would not make
them false — a good decision, and they are not false. But at `md` `65eab419` and
`me` `hashkinds-p3` **both conditions are now unreachable**: the operator meets
neither refusal. The card has established "here is who might not be ready", run
that list to exhaustion, and come up clean — while the actual unready consumer
is not on the list.

This is *output that is correct but that a hurried person misreads*. The fix is
one more conditional line naming the device, in the same shape as the two that
are already there; or, if the two are trimmed once their phases have shipped
everywhere, the device line replaces them.

**Owning phase: P3 or P4** (a `mnemonic-secret` change; Rust-primary rules do
not bind — this is CLI copy, not normative codec behaviour).

**J-1 and J-2 share one remedy** and either alone materially helps. The
strongest single version is a `me sysw pack` note that names the inertness *and*
forbids the obvious workaround (see J-3), because that is the moment the record
becomes a file the operator will carry.

### J-3 — Minor — **documentation only**

**`me sysw pack --help`'s kind paragraph lacks the do-not-strip-the-tag clause
that both the failure message and `ms` carry.**

The device's inertness supplies a live motive to strip: *"the machine ignored my
record; I'll make it look like one it reads."* Stripping is **asymmetric**:

```
$ printf 'hash:ripemd160:09e7…\n' | me sysw pack …    # tag stripped -> 40 hex -> REFUSED (exit 4)
$ printf 'hash:98a20fc2…(64 hex hash256)\n' | me sysw pack …   # tag stripped -> ACCEPTED, exit 0
```

— that is, the *harmless* case is refused loudly and the *harmful* one succeeds
silently. The `DO NOT DELETE IT TO SATISFY THIS RULE` warning
(`crates/me-cli/src/main.rs:3237`) appears **only in the failure text**, so a
`hash256` operator with a correct 64-hex digest never sees it.

Filed **Minor, not Important**, because the primary producer path already covers
it: `ms hashlock --kind hash256` prints *"Do not reshape the record to satisfy
it"* at derivation. And there is a second net — with a preimage co-packed,
`me` catches the stripped tag:

```
$ { echo 'hash:98a20fc2…'; echo "$MS1"; } | me sysw pack --pack-preimage …
me: WARNING — record 1 (records count from 0) is a preimage whose digest matches no `hash:` record in this payload (checked: sha256 3cf5d421..b70a4c12).
```

(with the tag present, no warning fires — the kind-aware comparison at
`main.rs:2619` is correct). The residual gap is help-text placement for an
operator who hand-writes the record from a descriptor rather than from `ms`.

**Owning phase: P3.**

### J-4 — Minor — **documentation only**

**The `ms` card tells the operator a HASHLOCK PHRASE plate carries the hash
line. For three of the four kinds no such plate can exist today, and the plate
that does exist carries no kind token.**

`crates/ms-cli/src/cmd/hashlock.rs:502`:

> `write the method line AND the hash line (ripemd160) next to your phrase
> **unless the phrase is cut on a HASHLOCK PHRASE plate, which carries both**`

Measured on the device: the plate's locator rows come from
`hashlockPlateLocator` (`gui/composer_preimage_plate.go:232`), which emits
`"hash  " + hashlockFirst8Last8(digest)` where `digest` is a `[32]byte` — no
kind token, and structurally incapable of holding a 20-byte digest. And under
`ripemd160` the plate cannot be reached at all: the route
`composerRouteHashlockPlates` is offered only for records the device could cut,
and the record is `ClassUnknown`.

So the `unless` clause is an escape hatch that does not open, printed verbatim
under a kind for which it cannot.

**Minor, not Important**, because the loss is recoverable by design: the card's
own fallback (*"re-run with no --kind and match the digest against your
descriptor"*) plus Journey D's four-digest lookup recovers the kind from the
preimage alone, against any one of the descriptor, the payload, or a funded
address. `SPEC_hashlock_kinds.md:~493` already schedules a per-kind locator row
for phase 4, which makes the sentence true for the sha256 case and reachable for
the others — so this is a timing defect, not a permanent one.

**Owning phase: P4** (the plate change), with the sentence corrected in `ms`
whenever P3 or P4 next touches that file.

### J-5 — Minor — **documentation only**

**`me sysw show` is the coming-back-later verb and carries no kind vocabulary.**

```
$ me sysw show --help
Print what a container holds, and its digest

Usage: me sysw show <FILE>
```

It *prints* `public record 1: ripemd160 hashlock (hash:)`, a token whose meaning
— and whose device consequence — is documented only under `me sysw pack --help`,
a verb this operator is not running. One sentence in `show`'s help pointing at
the kind axis (and at J-1's device consequence) closes it.

**Owning phase: P3.**

---

## Divergences I am explicitly NOT recommending a change for

**D-1 — the device's `N payload records were not understood.` is generic.**
It names neither the record nor the reason. **Not our concern for this cycle.**
It is the shipped §8r contract deliberately covering all four composer classes
with one line (`gui/composer_door.go:35-43`), it is firmware, and phase 4 owns
that surface. Making it specific is a phase-4 design question, not a phase-3
defect.

**D-2 — `md compose` says nothing about SeedHammer firmware support.**
**Not our concern.** `md` composes descriptors for any consumer. A
SeedHammer-firmware caveat in a general descriptor tool would be wrong for every
user who is not engraving, and `md`'s output is exactly as valid for a
`ripemd160` policy as for a `sha256` one — Bitcoin does not care which firmware
the operator owns.

**D-3 — the device's 64-hex pad refuses a 40-hex `ripemd160` digest.**
**Working as designed.** `gui/composer_hash.go:88-95` caps the fragment at 64
and only reveals OK at exactly 64, showing a live `N of 64 hex`. The operator is
stopped, not misled. `SPEC_hashlock_kinds.md` §7.1 schedules a kind pick screen
before the pad in phase 4.

**D-4 — an operator could type a 64-hex `hash256` digest into that pad and get a
`sha256(D)` wallet.** Considered carefully and **not recommending a change.**
The device states the function twice — `composerCopyHashRule`
(`gui/composer_copy.go:190`): *"The hash must be SHA-256 of a 32-byte value"* —
at entry and again at consent (§8i), and `md.SpendPath.Hash` is `*[32]byte` with
`pathBody` emitting `tagSha256` only (`md/compose.go:167, :403`), so the device
cannot silently build anything else. Phase 4's kind pick screen is the designed
close. Acting here would be pre-empting phase 4 with firmware changes.

**D-5 — Journey C's longer stderr card.** **No change.** stdout, the wire bytes
and all of `me` are byte-identical (verified by diff and by
`RecordHashKind::token`). The added block is the mechanism Journey D depends on;
suppressing it to reduce noise would delete the cycle's best recovery
affordance.

**D-6 — `--experimental` required for a key-less hashlock path.**
**Working as designed**, with an accurate warning naming bearer access, and a
second `note: stdout is a keyless descriptor template` line.

**D-7 — `me sysw pack`'s refusals for wrong widths, unknown kinds and wrong
case.** **Working as designed and unusually good.** All four measured:
`hash:ripemd160:<64hex>` → *"hash: ripemd160 needs exactly 40 lowercase hex
characters"*; `hash:RIPEMD160:` and `hash:sha1:` → *"unknown hash kind; expected
`hash:<hex>` (sha256) or `hash:<kind>:<hex>` with kind hash256, ripemd160 or
hash160, lowercase"*; uppercase body → the width message. Exit 4 throughout.
These are the class `SPEC_hashlock_kinds.md:~488` notes Core refuses "most
sharply and least helpfully", and the host is carrying that weight correctly.

**D-8 — `me sysw show` does not list a packed ms1 preimage record.**
Observed (`pub_len: 131` with only `public record 0` printed) and **out of scope
for this lens** — it is consistent secret-handling behaviour across every record
class, not a kinds question, and secret-handling defects are never blocking here
in any case.

---

## Counts

| severity | count |
| --- | --- |
| Critical | 0 |
| Important | 2 (J-1, J-2) |
| Minor | 3 (J-3, J-4, J-5) |
| Nit | 0 |

Divergences examined and deliberately not acted on: 8.

## Verdict

**NOT GREEN** — 0 Critical / 2 Important.

Both Importants are the same missing moment seen from the two ends of the chain,
and both are additive copy: **the chain never tells the operator that a
kind-tagged record is inert on a device that has not been upgraded, and `ms`
actively misdirects by naming the two consumers that are ready.** Neither
touches normative behaviour, a wire format, or the device. Nothing found here
contradicts the correctness rounds — the fail-closed design is correct, the
producer rule reaches the wire, the kind survives `show`, and the recovery story
in Journey D is genuinely good. What is missing is a sentence at the moment the
payload becomes a file.

## Reproduction

```
export PATH=/home/bcg/.cargo/bin:$PATH
cd /scratch/code/shibboleth/descriptor-mnemonic && cargo build -p md-cli --bin md --locked
cd /scratch/code/shibboleth/mnemonic-secret     && cargo build -p ms-cli --locked
cd /scratch/code/shibboleth/mnemonic-engrave    && cargo build --locked   # branch hashkinds-p3

mkdir -p /tmp/hk && cd /tmp/hk
printf 'correct horse battery staple' > phrase.txt
ms hashlock --hashlock-phrase-stdin --kind ripemd160 < phrase.txt > rec.txt
me sysw pack --in rec.txt --no-passphrase --no-now --out p.bin
me sysw show p.bin
xxd p.bin | tail -4
```

The device probe is an out-of-tree Go module with
`replace seedhammer.com => /scratch/code/shibboleth/seedhammer`, calling
`sysw.Classify` and `sysw.ParseHashRecord`. Go 1.26.7 from
`/scratch/code/shibboleth/.toolchain/go`. Nothing in the fork was modified.
