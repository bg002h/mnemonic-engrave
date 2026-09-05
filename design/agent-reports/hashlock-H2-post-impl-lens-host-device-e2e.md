# H2 post-implementation review — lens: can a user actually do the thing, end to end

**Scope.** One question only: for the operator's own wallet — the constellation
fixture *"our reasonably complex wallet"* in both `tr` and `wsh` forms — can the
hashlock **phrase route** be reached and completed on the harness, and does the
digest the device shows equal what the host prints? Plus the F-474 path: can a
payload carrying a preimage plate exist at all, and what does the device say
about one.

Not a general review. Fork tree under test: detached worktree at
`17b3979` (`/scratch/code/shibboleth/.tmp/h2-wf-lens-host-device-e2e`,
removed after this report). Host binaries: `ms 0.18.0`
(`/scratch/code/shibboleth/mnemonic-secret/target/debug/ms`), `md 0.14.0`
(`~/.cargo/bin/md`), and the **released** `me 0.8.1`
(`gh release download v0.8.1`, `sha256sum -c SHA256SUMS` → all OK).

**Counts: 0 Critical / 1 Important / 3 Minor / 1 Nit.**

---

## What was measured (the answer to the lens's question)

**Yes, in both forms, and the digests agree with the host everywhere I could
make them disagree.** Eight independent host/device comparisons, all equal:

| phrase | method | host `ms hashlock` | device (assigned `Paths[i].Hash`) |
| --- | --- | --- | --- |
| `correct horse battery staple` | sha256 | `b867db87…edbc96cb` | equal (fork's own test) |
| `correct horse battery staple` | hardened | `3cf5d421…b70a4c12` | equal (fork's own test) |
| RCW phrase 1 (40 chars) | sha256 | `a7ef0ba4…725367f1` | equal, `tr` **and** `wsh` |
| RCW phrase 2 (38 chars) | sha256 | `e9955f6f…1d4f09b4` | equal, `tr` **and** `wsh` |
| RCW phrase 3 (34 chars) | sha256 | `7950085d…071d6af7` | equal, `tr` **and** `wsh` |
| all 95 bytes `0x20..0x7E` | sha256 | `f948515e…a991c233` | equal |
| all 95 bytes `0x20..0x7E` | hardened | `b7864c7e…d4d0a094` | equal |

The three RCW digests are not arbitrary: they are the `sha256()` literals in
**both** fixture policies, and `ms hashlock --method sha256` reproduces each one
from the fixture's own phrase file.

```
$ for i in 0 1 2; do f=design/journeys/inputs-rcw/preimages/preimage-$i.txt
    ms hashlock --hashlock-phrase-stdin --method sha256 --no-engraving-card < $f; done
hash:a7ef0ba42dada5629bbb95e386c572006d4bea43d483e5c44f4c3858725367f1
hash:e9955f6f5b49ff288c3f8360e6a7dde1d54aa590eb6a20f28b23db361d4f09b4
hash:7950085dca9f90b67bbcfeb8141499a98df93e32709807420c86f2ff071d6af7

$ grep -o 'sha256([0-9a-f]\{64\})' design/fixtures/reasonably-complex-wallet/tr.policy
sha256(a7ef0ba42dada5629bbb95e386c572006d4bea43d483e5c44f4c3858725367f1)
sha256(e9955f6f5b49ff288c3f8360e6a7dde1d54aa590eb6a20f28b23db361d4f09b4)
sha256(7950085dca9f90b67bbcfeb8141499a98df93e32709807420c86f2ff071d6af7)
```

(The fixture's construction — witness preimage `sha256(phrase)`, policy literal
`sha256(sha256(phrase))`, `design/fixtures/reasonably-complex-wallet/README.md`
lines 11–17 — **is** `ms hashlock --method sha256`. The device's SHA-256 row and
the operator's real wallet are the same construction.)

### The walk, on the harness, in both wrappings

New test `gui/zzlens_rcw_e2e_test.go` (mine, in my worktree only, never
committed) builds the four-tier vault as the device's own model expresses it —
3-of-3 now; 2-of-2 after `older(32768)`; 1 key after height 1173520; 1 key after
height 1383520 — and walks the whole phrase route at paths 1, 2 and 4 through
`composerHashEdit`, which is the arm `composerPathEdit`'s `Hash lock` row
reaches: `Which hash?` → phrase row → §8i rule modal → the four-page keyboard →
`Which method?` → SHA-256 warning → HOLD → the reconcile screen.

```
$ go test ./gui/ -run TestLensRCWPhraseRouteBothForms -v
=== RUN   TestLensRCWPhraseRouteBothForms/tr
    path 1: confirm body = "hasha7ef0ba4..725367f1method:sha256chars:40Writedownthisphrase…"
    path 2: confirm body = "hashe9955f6f..1d4f09b4method:sha256chars:38anotherpathhasadifferenthash:twophrasestobackup…"
    path 4: confirm body = "hash7950085d..071d6af7method:sha256chars:34anotherpathhasadifferenthash:twophrasestobackup…"
    tr: 5 chunks -> /scratch/code/shibboleth/.tmp/h2-lens-rcw-tr.chunks
=== RUN   TestLensRCWPhraseRouteBothForms/wsh
    …the same three digests…
--- PASS: TestLensRCWPhraseRouteBothForms (12.21s)
```

The digests survive to the artifact. The chunks the device emits, decoded by the
host codec:

```
$ md decode $(cat /scratch/code/shibboleth/.tmp/h2-lens-rcw-tr.chunks | tr '\n' ' ')
tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{and_v(v:multi_a(3,…),
sha256(a7ef0ba42dada5629bbb95e386c572006d4bea43d483e5c44f4c3858725367f1)),{…
sha256(e9955f6f5b49ff288c3f8360e6a7dde1d54aa590eb6a20f28b23db361d4f09b4)…
sha256(7950085dca9f90b67bbcfeb8141499a98df93e32709807420c86f2ff071d6af7)…}}})
```

Same three literals under `wsh`. The `tr` internal key comes out as the same
NUMS point the fixture carries.

### A hash-only path, per form

```
$ go test ./gui/ -run TestLensRCWHashOnlyPathPerForm -v
    tr:  after choosing 'A hash, no keys' the screen reads
         "This build will not put a key-less path in taproot. Use wsh, or add a key. Path 1"
    wsh: "KEY-LESS PATH (EXPERIMENTAL) … Hold button to confirm."
```

So a **hash-only** path is a `wsh`-only shape, refused under `tr` by name before
the phrase route is reached — correct, and the refusal names the remedy. Under
`wsh` the route runs to completion (the fork's own
`TestHashlockPhraseRouteSetsTheCorpusDigest` drives exactly that shape). For the
RCW the hashlocked paths are all keyed, and the route is reached from
`composerPathEdit`'s `Hash lock` row instead; both entry points work.

### Lockstep of the phrase rule itself, host vs device

Every refusal, in the same order, measured on `ms 0.18.0`:

```
$ ms hashlock --hashlock-phrase-stdin --no-engraving-card < empty
error: the hashlock phrase is empty
$ … < non-ascii            error: the hashlock phrase must be printable ASCII (bytes 0x20..0x7E); byte 0xc3 at position 3 is not
$ … < ms1-shaped           error: that is an ms1 string, not a hashlock phrase; …
$ … < grouped-ms1-112ch    error: that is an ms1 string, not a hashlock phrase; …   ← shape BEFORE the cap
$ … < 100 a's              hash:c5c747d5ea6ec6f51f7a68add1d9cf58a4e34427de348a87313e36059d7dabef
$ … < 101 a's              error: the hashlock phrase is 101 characters; at most 100 are allowed
$ … < 64 hex               error: that is 64 hex characters -- a preimage, … not a phrase; pass it with --hex
```

That is `hashlock.ValidatePhrase`'s order and verdicts exactly
(`hashlock/hashlock.go:83-102`), including the spec §2 requirement that the
shape test precede the cap.

**And every byte the host accepts is typeable on the device.** This is the half
that a corpus cannot check, so I typed all 95:

```
$ go test ./gui/ -run TestLensKeyboardCoversEveryPhraseByteTheHostAccepts -v
    confirm body: "hashf948515e..a991c233method:sha256chars:95…"
--- PASS (2.09s)
```

`typeOnPassphraseKeyboard` fails loudly on an untypeable rune, and the digest is
the host's for the same 95 bytes — so the keyboard covers `0x20..=0x7E` with no
gap, and a phrase made on the host can always be re-typed on the device.

The hardened warning threshold is in lockstep too — warn at 19, silent at 20, on
**both** sides:

```
$ ms hashlock --hashlock-phrase-stdin --method hardened < 19a   WARNING: a 20-character phrase falls in about 72 days on one GPU; …
$ …                                                    < 20a   (no warning on stderr)
$ go test ./gui/ -run TestLensHardenedTwentyCharsGetsNoWarning -v
    19 characters -> "Evena20-characterphrasefallsinabout72days…"
    20 characters -> "0%Deriving.Thistakesabout10seconds.Deriving"
```

### The reconciliation the device can do for itself

When the operator made the digest on the host first and loaded it, the confirm
modal proves the two legs agree, on the device, before HOLD:

```
$ go test ./gui/ -run TestLensRelationLineAgainstAHostMadeRecord -v
  payload "hash:a7ef0ba4…725367f1" + the same phrase typed:
    "hasha7ef0ba4..725367f1method:sha256chars:40matcheshash1inthepayload…"
  payload "hash:e9955f6f…1d4f09b4" + phrase 1 typed:
    "hasha7ef0ba4..725367f1method:sha256chars:40nohash:recordinthepayloadhasthisdigest…"
```

### F-474 — the preimage-plate payload

Hand-built, as the brief asked, and driven to the screen. `gui/zzlens_f474_test.go`
seals **the same payload twice**, with and without the plate, same passphrase,
same harness — the control is what proves *"Payload unreadable."* was a false
diagnosis rather than a clumsy one:

```
$ go test ./gui/ -run TestLensF474PreimageVersusControl -v
  control -- no preimage plate:            "SECRETseedmaterialCutthisplateSkipms1"
  the same payload plus a plate at record 1:
      "Record1isahashlockpreimage,notaseed.Thispayloadcannotbeunlockedhere.Nothingwasopened.SealedPayload"
  a preimage plate at record 0:
      "Record0isahashlockpreimage,notaseed.…"
--- PASS (0.33s)
```

Not a false PASS. Mutating the arm away (`case false && errors.As(err, &notPermitted):`)
turns both my test and the fork's own red, with the old copy on screen:

```
--- FAIL: TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable
    never reached "hashlock preimage"; last frame "Payloadunreadable.SealedPayload"
--- FAIL: TestLensF474PreimageVersusControl/the_same_payload_plus_a_preimage_plate_at_record_1
    never reached "Record 1"; last frame "Payloadunreadable.SealedPayload"
```

`gui/unlock_kdf.go` was restored from a backup afterwards; `git status` in the
worktree shows only my own new `zzlens_*_test.go` files.

**Stated plainly, as asked: the released `me 0.8.1` will not build such a
payload. A hand-built blob is the only way one exists.** Both container verbs
refuse the plate:

```
$ ./me --version
me 0.8.1
$ ./me sysw pack --in plate.txt --no-passphrase --out /tmp/x.bin
me: record 0 (records count from 0) is a hashlock PREIMAGE plate (kind 0x03), not a seed
    record; this container cannot place one yet. …
exit=4
$ ./me sysw pack --in seed-then-plate.txt --passphrase-words 4 --out /tmp/x.bin
me: record 1 (records count from 0) is a hashlock PREIMAGE plate (kind 0x03), …
exit=4
$ ./me seal --in seed-then-plate.txt --seal-secret --out /tmp/x.bin
me: this record is a hashlock PREIMAGE plate (kind 0x03), not a seed record; …
exit=2
```

So the screen this stage adds is for a payload no released host tool can
produce: a hand-built blob, a future `me`, or a different tool. That does not
make it unnecessary — it is the difference between a diagnosis and a false
accusation of tampering — but it does bound how often it will be seen.

### Records spot-check

The controller's reported walk values reproduce against the host, character for
character: control `correct horse battery stapl` →
`c8043156b1d3d7f5cf5ce67bacf266db4d2f96d22a95e46afab24e47253e7389`
(`c8043156..253e7389` ✓); mixed case `Correct Horse Battery Staple` →
`95d4447031cdc4117f797040c1a9e32367af2a8d97554e442c7bfd002297a7ff`
(`95d44470..2297a7ff` ✓). The implementation report's `Digest` mutation row
(`9a2db2e2…`) is likewise the host's value for the seam plate
(`ms hashlock --in plate.txt` → `hash:9a2db2e23f1504cd056606553ac049c5e718e8f9ce9233876df1a7a1821af885`).

Suites run once, captured: `hashlock` ok, `codex32` ok, `seal` ok (11.97s);
`go test ./gui/ -run 'Hashlock|Preimage|TestLens'` ok (67.16s).

---

## Findings

### I-1 -- "two phrases to back up" is a hard-coded count, and it is wrong on the operator's own wallet

`gui/composer_hashlock.go:119-129` returns a fixed string as soon as it finds
**one** other path with a different hash:

```go
func hashlockOtherPathLine(st *composerState, idx int, h [32]byte) string {
	for i, p := range st.list.Paths {
		if i == idx || p.Hash == nil { continue }
		if *p.Hash != h { return composerCopyHashlockOtherPath() }
	}
	return ""
}
```

and `gui/composer_copy.go` (H2, commit `978a9de`):

```go
func composerCopyHashlockOtherPath() string {
	return "another path has a different hash: two phrases to back up"
}
```

The reasonably complex wallet has **three** hashlocks. Measured, on the real
route, at path 4 — with paths 1 and 2 already carrying different digests:

```
$ go test ./gui/ -run TestLensRCWPhraseRouteBothForms -v
    path 4: confirm body = "hash7950085d..071d6af7method:sha256chars:34anotherpathhasadifferenthash:
                            twophrasestobackupWritedownthisphraseandthemethodnow.…"
```

and pinned as a pure function over 1, 2 and 7 other differing hashes:

```
$ go test ./gui/ -run TestLensOtherPathLineCounts -v
    one other path with a different hash                     -> "another path has a different hash: two phrases to back up"
    TWO other paths with different hashes (the RCW's path 4) -> "another path has a different hash: two phrases to back up"
    SEVEN other paths, all different                         -> "another path has a different hash: two phrases to back up"
```

The fork's only test of this line drives **exactly one** other path
(`gui/composer_hashlock_test.go:872-893`, `st.list.Paths[0].Hash = &other` on a
two-path list), so `n > 2` was never exercised. The plan carries the same string
and the same single-case test (`IMPLEMENTATION_PLAN_hashlock_H2_device.md:1271`,
`:2379`); SPEC §4.5 does not specify this line at all — it is the r0-journey I-1
addition, so the copy is free to change.

**Why this is Important, not Minor.** It is a *missing case* in freshly written
H2 code, and the wrong outcome is a **backup undercount on a hashlock wallet**.
The whole premise of the surrounding modal is *"They are not on this device and
not on your plates. Without both, this path can never be spent."* An operator
who has just typed their third phrase and reads "two phrases to back up" is
being given a number, at the moment they are counting, and the number is one
short. Losing the third phrase makes tier 4 of their vault unspendable forever.
It is the only counting statement anywhere in the flow: §8h's banner is guarded
by `composerEveryPathHashed`, which is **false** for the RCW (path 3 is
un-hashed) — measured, `composerEveryPathHashed = false, hashByPhrase = true` —
so on this wallet the operator never sees it.

The fix is arithmetic: count the distinct other digests and say so, or drop the
number ("other paths have different hashes: back up every phrase"). Add an
`n > 2` row to the existing table test.

### M-1 -- the `first8..last8` form is documented nowhere an operator reads, and the one host command the device names prints only the full 64

Every digest the firmware draws is truncated. There is no full-64 rendering
anywhere in the GUI:

```
$ grep -rn "hex.EncodeToString" gui/*.go | grep -v _test
gui/composer_consent.go:62:	h := hex.EncodeToString(d[:])      → h[:8] + ".." + h[56:]
gui/composer_hashlock.go:132:	s := hex.EncodeToString(h[:])      → s[:8] + ".." + s[len(s)-8:]
gui/composer_hash.go:39:	h := hex.EncodeToString(digest[:]) → "hash %d  %s..%s"
```

The reconcile screen the operator sees immediately after HOLD says:

> Before you fund this wallet, run ms hashlock with this phrase and method on
> the host and check the digest matches.

and `ms hashlock` prints `hash:<64 hex>` and a card line `digest: <64 hex>` —
never an abbreviation. Nothing tells the operator that "matches" means *the
first eight and the last eight*.

Searched and not found: fork `README.md`, `docs/custom-firmware.md`,
`mnemonic-engrave/README.md`, `me sysw show --help`, `ms hashlock --help`. The
form is stated only in `design/SPEC_hashlock_H2_device.md` (lines 179, 246, 284),
which no operator reads.

Mitigating, and why this is Minor rather than Important: the same 8..8 form does
appear host-side, in the released binary, for the reverse direction —

```
$ ./me sysw show /tmp/h2hash.bin
public record 0: sha256 hashlock (hash:) — a7ef0ba4..725367f1
```

— so the correspondence is discoverable, and comparing the two ends of a hex
string against a full one is the obvious reading. Owning phase: docs (H4's walk
is the natural place, since the walk is where an operator first meets it).

### M-2 -- the reconciliation is asked for at a moment when its left-hand side has just left the screen, and the write-down instruction omits it

The confirm modal is the only screen that carries `hash`, `method:` and
`chars:` together. After HOLD the digest is gone from the path list —

```
$ go test ./gui/ -run TestLensRCWDigestIsReadableAfterTheModal -v
    path-list row: "Path 1: 3-of-3 + hash"
    Which hash? re-entered: "Path1hashNohashrecordinthepayload.Typeaphrasebelow,…TypeahashlockphraseType64hexNohashlock"
      (the assigned digest is NOT shown there)
```

— and the modal's own instruction is *"Write down this phrase and the method
now"*, which names neither the digest nor `chars:`, the two things the check
compares. `chars:` in particular is called out in SPEC §4.5 as *"the one signal
that shows a stray space when the operator later reconciles against the host
card's `phrase_chars`"*, and it exists on exactly one screen, once.

Nobody is stuck: the digest reappears on the consent surface, in the same
abbreviated form, before engraving —

```
    consent lines:
      Path 1: 3 key(s), custom
        hash a7ef0ba4..725367f1
      Path 2: 2 key(s), custom
        32768 blocks (about 227.6 days)
        hash e9955f6f..1d4f09b4
      …
        hash 7950085d..071d6af7
```

— which is the right moment for *"before you fund this wallet"*. The gap is that
the instruction and the affordance are never connected: the reconcile screen
names no place to read the digest back, and the consent screen carries the
digest but neither the method nor `chars:`. One clause on the reconcile screen
("the consent screen shows it again") or adding the digest to the write-down
list would close it. Owning phase: H3/H4.

### M-3 -- the F-474 refusal names the record but not the next step

```
Record 1 is a hashlock preimage, not a seed. This payload cannot be unlocked
here. Nothing was opened.
```

`gui/unlock_kdf.go`'s own rationale for the change is *"Naming the record and
the kind is what turns a suspected compromise back into a payload to rebuild on
the host"* — and the screen never says to rebuild it. Every other refusal on
this cycle's surfaces names the route that exists: the ms1-shape refusal says
*"On the host, run ms hashlock with it and load the hash: record it prints"*;
the 64-hex refusal says *"Use the Type 64 hex row"*. This one stops at the
diagnosis.

Low impact, because no released `me` can build such a payload (see above), so
whoever meets this screen assembled the blob themselves and knows what a record
is. Minor. A half-sentence — *"Rebuild the payload on the host without it."* —
would finish the sentence the code comment already writes.

### N-1 -- `me seal` does not name the record index where `me sysw pack` does

Same plate, same release, two containers:

```
$ ./me sysw pack --in seed-then-plate.txt … 
me: record 1 (records count from 0) is a hashlock PREIMAGE plate (kind 0x03), …
$ ./me seal --in seed-then-plate.txt --seal-secret …
me: this record is a hashlock PREIMAGE plate (kind 0x03), …
```

Host-side and pre-existing (H0), outside this diff. Recorded only because the
whole point of F-474 is that *which record* is the fact the operator needs, and
one of the two host verbs that refuses the same plate does not supply it.

---

## Not findings (checked, and clean)

- **Seat loss on a hash edit.** Setting a hash on any RCW path moves no slot in
  either wrapping, so §8j never fires and no seat is discarded:
  `composerEditCanRenumber(list, i, composerFieldHash) = false` for all four
  paths, both wrappers, and also with paths 1–2 already hashed
  (`TestLensRCWHashEditCanRenumber`).
- **The 20-character hardened boundary.** The modal's copy says "Even a
  20-character phrase falls…" while the guard is `len(phrase) < 20`, so a
  20-character phrase gets no warning. The **host does exactly the same** (warns
  at 19, silent at 20), so device and host are in lockstep and any change belongs
  to `SPEC_ms_hashlock`, not here.
- **`ms hashlock` with no source does not hang.** `exit=64`, *"no source given;
  exactly one source: …"* — the device's reconcile instruction does not walk the
  operator into a blocking read.
- **The ms1-shape refusal's remedy runs.** An ms1 on argv is refused by the argv
  guard with the fix named; `ms hashlock --in plate.txt` prints
  `hash:9a2db2e2…1af885` and the card. The device's copy points somewhere real.
- **Nothing normalises.** The fork's own `TestHashlockPhraseRouteDoesNotNormalise`
  covers case, whitespace and separators; my 95-byte full-ASCII run adds the
  characters a normaliser would most likely eat.

---

## Reproduction

My tests live only in the detached worktree (never committed, removed after this
report): `gui/zzlens_rcw_e2e_test.go`, `zzlens_f474_test.go`,
`zzlens_keyboard_test.go`, `zzlens_hardened_test.go`, `zzlens_relation_test.go`,
`zzlens_otherpath_test.go`, `zzlens_renumber_test.go`,
`zzlens_warnthreshold_test.go`. To re-create:

```sh
git -C /scratch/code/shibboleth/seedhammer worktree add --detach <path> 17b3979
export PATH=/scratch/code/shibboleth/.toolchain/go/bin:$PATH
go test ./gui/ -run 'TestLens' -v
```

The three RCW phrases are the fixture's own, at
`mnemonic-engrave/design/journeys/inputs-rcw/preimages/preimage-{0,1,2}.txt`;
they are committed plaintext in the repo already and are marked
DO-NOT-REUSE by that fixture's README. No phrase or preimage is written into any
log this report keeps.
