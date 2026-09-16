# hashlock kinds — phase 4 whole-diff adversarial execution review

Scope: `seedhammer` `git diff 0562e81..b361b22`, 18 commits, 72 files, +3784/−654,
reviewed as ONE change. Reviewed at commit `b361b22`, not at the working tree —
the tree had already moved on while this review ran (first
`gui/walk_copy_anchors_test.go`, later `gui/composer_consent.go`,
`gui/composer_gates_test.go`, `md/testdata/compose_vectors.provenance.json`,
`scripts/vendor-compose-vectors.sh`), so all builds and probes below were run
against a pristine `git archive b361b22` extracted to `/tmp/hk-review`. Nothing
in the repository was modified.

## VERDICT: NOT GREEN — 0 Critical / 1 Important / 6 Minor / 4 Nit

The one Important is **not** a funds defect and **not** a wrong result. The
codec, record grammar, script lowering, digest dispatch, identity and screen
plumbing are correct at every kind, and I confirmed that by construction rather
than by reading (see "What I proved correct"). The Important is that this diff
**broke the second emulator walk and never ran it** — the acceptance apparatus,
not the firmware.

---

## Critical

None.

---

## Important

### I-1 `cmd/emu/shots_composer.js:665` and `:785` — the composer screenshot walk aborts at step (12). Two anchors this diff itself invalidated, in the file this diff itself edited twice, never re-run.

Both are `must()` assertions on production screen text. `must()` squashes
whitespace on both sides (`shots_composer.js:43,138`), so the needle must appear
as a substring of the squashed frame.

**:665 — `must(hashRows, "hash 1", "the payload's hash row")`.**
Commit `09a9a7b` changed `composerHashRow` so the kind token *replaces* the word
`hash` (`gui/composer_hash.go:66`: `fmt.Sprintf("%s %d  %s", digest.Kind().Token(), i, …)`).
Reproduced by rendering the real screen through the package's own touch harness:

```
payload: "hash:" + strings.Repeat("ab", 32)      (exactly what the emulator walk loads)
row 0                     = "sha256 1  abababab..abababab"
frame (harness-squashed)  = "Path1hashWhichhash?sha2561abababab..ababababTypeahashlockphraseTypeadigestNohashlock"
needle "hash 1" squashed  = "hash1"   -> NOT PRESENT
```
The substring `hash1` cannot appear under *any* op ordering: the only op ending
in `hash` is the title `Path 1 hash`, and no op on this screen begins with `1`.
So `must()` throws and the walk stops at step (12) of ~20 — before the §8i rule
shot, before the stub screens, before consent, before the census. The two
`must()` lines directly under it (`:666` `abababab..abababab`, `:667`
`Type a digest`, `:668` `No hash lock`) all still pass, which is why reading the
block does not reveal it.

**:785 — `must(consent.joined, "hash abababab..abababab", "the consent's hash")`.**
Commit `9e0c63c` changed `composerBranchLines` to
`"  hash "+d.Kind().Token()+" "+composerDigestShort(d)` (`gui/composer_consent.go:103`).
Reproduced by composing the walk's own two-path policy and calling
`composerConsentLinesFor` directly:

```
  12960 blocks (about 90.0 days)
  hash sha256 abababab..abababab
```
needle squashed `hashabababab..abababab`; frame carries `hashsha256abababab..abababab`. Not a substring. (Unreachable today only because :665 throws first.)

**Why no gate caught it.** `gui/walk_copy_anchors_test.go` was added in commit
`8e582dd` *for exactly this class* ("A WALK MAY NAME PRODUCTION COPY ONLY THROUGH
THIS TABLE") and lists `shots_composer.js` twice — but its own scope paragraph
says "screen TITLES and fixed labels are not composer copy… What it gates is
every fragment that comes from a `composerCopy*` body". Both broken anchors come
from row/line **builders** (`composerHashRow`, `composerBranchLines`), not from
`composerCopy*` bodies, so they sit precisely in the gate's stated blind spot.
The final commit is titled "walk: RUN IT" and the brief records that `run()` and
`runKindTrial()` of `walk_hashlock_phrase.js` were executed — `shots_composer.js`
was edited (in `285f7a9` and `0d25c5a`) and not run.

Repro (any tree at `b361b22`, package `gui`):
```go
sess := composerSessionWith([]string{"hash:" + strings.Repeat("ab", 32)}, nil)
st   := composerStateWithPaths(t, 1)
h    := runComposerHashEdit(t, st, sess, 0, new(bool))
body := h.mustReach("Which hash?")   // -> no "hash1" in squash(body)
```

---

## Minor

### M-1 `gui/composer_hash.go:594` — the §8 20-byte warning on the PAYLOAD route is wired but has no test. Proven by mutation.

`composerWarnTwentyByteUnseen` guards two assignment sites: band 1 (a payload
`hash:` record, `:594`) and the hex pad (`:671`). The hex-pad wiring is covered
by `TestComposerCanBuildARipemd160HashlockOnTheDevice`. The band-1 wiring is
covered by nothing:

```
MUTATION: delete the three-line guard at gui/composer_hash.go:594-596
RESULT:   go test ./gui/ -count=1  ->  155.98s, ZERO committed test failures
          (the only failure was my own probe, which is the control proving the
           mutation was live)
```

Band 1 is §8's *motivating* case — a digest the device did not derive, arriving
from a payload — and it is the only route by which a 20-byte digest reaches a
policy without being typed. No committed test anywhere constructs a
`hash:ripemd160:` or `hash:hash160:` payload record (`grep` over `gui/` and
`sysw/` `*_test.go`: zero hits outside `sysw/testdata/record_class_vectors.json`).

Not blocking: I drove the route by touch and the production behaviour is
**correct** — the row draws `ripemd160 1  09e7bb50..bcc2946b`, the modal fires
("20-BYTE HASH, NOT DERIVED HERE… This is a ripemd160 digest…"), holding assigns
a `KindRipemd160` lock with the right 20 bytes, and it composes and chunks. But
the guard cannot regress noisily, and this cycle's own record is that the warning
was absent until two lenses found it independently.

### M-2 `gui/composer_preimage_plate.go:425` and `:427` — the census names the kind on accepted rows and not on the other two.

`72196d8` added the kind to `composerCopyPreimagePlateRow` with this reasoning:
"The census is the operator's only inventory of what they must store apart, and
it described all four constructions identically." The same function's other two
row producers, eight lines away, were left alone. Reproduced:

```
two paths, ripemd160 and hash160 over the same 20 bytes, both held, both declined
census: "preimage 11111111..11111111: declined, will not be cut"
        "preimage 11111111..11111111: declined, will not be cut"
```
Two different locks, two byte-identical inventory rows. (§6's both-or-neither
rule is arguably satisfied — these rows name no method either — which is why this
is Minor rather than Important, but the §13.3 argument the fold made applies to
them verbatim.)

### M-3 `gui/composer_preimage_plate.go:78-118` — one preimage can now yield TWO bearer plates, because the plate set's dedupe key gained the kind.

`seen` and `hashlockHeld` moved from the bare 32-byte digest to
`HashLock.MapKey()` = `<kind>:<digest>`. Correct for identity, but it means a
composition that uses ONE phrase on two paths at two kinds holds two entries with
the *same* preimage X and offers two plates. `codex32.EncodeMS1Preimage(X)` does
not carry the kind, so the two steel plates are byte-identical apart from the
locator's `hash <kind> …` row. Reproduced: `composerPreimagePlates` returns 2
plates for one X. Before this cycle the case could not arise (one kind). Nothing
tells the operator the two plates are the same secret; §8.3's "keep each apart
from the others" is the closest the copy gets.

### M-4 `sysw/composer_records.go:44-50` — `ErrHashRecord`'s doc describes an error message that does not exist and a §8n line that is never drawn.

The comment: "The §8n line names that kind's own width -- told 'must be exactly
64 hex characters', a ripemd160 record given 64 hex learns nothing… It carries
the kind, unlike `ErrPhraseRecord`."
The value: `errors.New("sysw: hash: body is not that kind's width in lowercase hex")`
— names neither the kind nor a width. And `ParseHashRecord`'s only device-side
caller is `classifyComposer` (`:121`), which discards the error and returns
`ClassUnknown`; `grep` confirms no other consumer, so this error reaches no
screen at all. The vendored host lines it is meant to mirror
(`sysw/testdata/record_class_vectors.json`, e.g. "record 0: hash: ripemd160 needs
exactly 40 lowercase hex characters") are only checked for *presence*, never
content (`sysw/composer_records_test.go:91`).

### M-5 `engrave/h6_qr_test.go:49-55` — "THE FUZZ SAMPLES EVERY KIND" is false; it samples one.

`h6QRTextKind` has exactly one call site, `h6QRText` at `:80-81`, which always
passes `md.KindRipemd160`. The budget fuzz therefore samples the widest kind
only. That is the *right* design for a budget gate (and the very next doc
paragraph says so correctly — "at the widest kind, so every budget they compute
is the worst case"), but the two comments contradict each other and the first is
the one a reader will quote. I verified the underlying measurement is sound:
worst-case payload is sha256 207 / hash256 208 / hash160 208 / **ripemd160 210**
bytes (was 194), and `TestConstantQREngraveAtScale2Completes` asserts `c.Size == 53`.

### M-6 `md/testdata/compose_vectors.provenance.json:6` and `backup/hashlock_test.go:464-465` — two pin messages that no longer describe their own checks.

- The provenance `_comment` still says the test fails "if the file count is not
  156". `md/compose_vectors_pin_test.go:96` now requires **176** and the file
  lists 176 (measured: 36 each of `bytes.hex`/`descriptor.json`/`phrase.txt`/
  `template`, 32 `conformance.json`).
- `backup/hashlock_test.go:464` was loosened from `!= 7` to `< 7` while its
  message still reads "H6 §11.2 pins seven"; the corpus now carries 10 rows and
  the provenance records `qr_text_rows: 10`. The exact-count pin is gone; only
  the file sha256 still constrains it.

---

## Nit

### N-1 `md/policy_shape.go:60-61` — a retracted sentence left two lines above its replacement.
"Only sha256 digests are carried (the composer emits no other hash); the other
hash tags still set Hashlock." directly precedes the new `Hashlocks` field whose
own doc says it "carries EVERY hashlock, whatever its kind". Grep for
`Sha256Digests` in the tree returns exactly one hit, inside that new doc block —
this sentence is the last live statement of the retracted behaviour.

### N-2 `gui/composer_preimage_plate.go:251` — "Measured at 35 characters"; measured 34.
`"hash  " + Token() + " " + first8..last8` = 6 + 9 + 1 + 18 = 34 for `ripemd160`,
the longest token (sha256 31, hash256/hash160 32). Conservative direction, so the
3.0 mm / 10-row / 1.20 mm conclusion is unaffected.

### N-3 `gui/composer_paged.go:347-355` — the re-home branch is unreachable from its only production caller.
`composerPickScreenFrom`'s `homed` block fires only when `sel >= start+shown`.
Its sole non-default caller is the kind screen, which
`TestComposerHashKindScreenDrawsAllFourRows` pins at 4 tappable rows on the first
page with `initial ∈ {0..3}`, so `sel >= 0+4` is never true. The fix for
`start := sel` is correct and `composerPickScreen` is byte-for-byte unchanged in
behaviour (verified: `initial=0` short-circuits the block), but the defensive arm
is dead and untested.

### N-4 `gui/walk_copy_anchors_test.go:53-56` — one row of the anchor table is a tautology.
`{"walk_hashlock_phrase.js", "ripemd160", composerCopyHashlockConfirm(…, md.KindRipemd160), …}`
asserts that a body rendered *at* `KindRipemd160` contains `"ripemd160"`, and
that the walk file contains the string `"ripemd160"` somewhere — it appears 20+
times in that file. The row cannot fail on any rewording of the confirm modal.
(The other five rows are genuine.)

---

## What I proved correct (so the next reviewer does not re-derive it)

Run against `b361b22` in the isolated tree, go1.26.7.

**Mutation-tested — every one of these is caught, i.e. the funds-critical gates
are real gates:**

| mutation | caught by |
| --- | --- |
| `hashlock.DigestOf`: swap the `ripemd160` and `hash160` arms | `hashlock.TestDigestKAT` |
| `md.HashKind.tag()`: `KindRipemd160 -> tagHash160` | `md.TestComposeRoundTripsEveryHashKind`, `md.TestEveryHashKindTakesItsOwnWireTag` |
| `md.pathBody`: 20-byte kinds emit a `hash256Body` | `md.TestComposeRoundTripsEveryHashKind` |
| `md.HashKind.DigestLen()`: ripemd160/hash160 return 32 | `hashlock.TestDigestKAT`, `hashlock.TestHashLockIdentityIgnoresPadding`, `md.TestComposeRoundTripsEveryHashKind`, `sysw.TestComposerRecordsClassifyExactlyAsTheHost` |
| `sysw.ParseHashRecord`: unknown kind token falls back to sha256 | `sysw.TestComposerRecordsClassifyExactlyAsTheHost` |

**Independently recomputed, not read:**

- Every `derivation[0]` row of the re-vendored corpus
  (`hashlock/testdata/hashlock-v0.8.json`) against a python3 `hashlib` oracle —
  `hardened_x/_h/_h_hash256/_h_ripemd160/_h_hash160` and the four `sha256_*`
  twins all match exactly, including the walk's `ANCHOR_HARD_RIPEMD160 =
  09e7bb5051d89788fb4e4b374126721dbcc2946b`. The corpus file's sha256 matches
  its provenance pin (`0a911f78…d8ce`).
- `md/testdata/vectors/keyed_compose_preset_hashlock_gated_ripemd160.conformance.json`
  carries the real corpus digest in its template
  (`ripemd160(09e7bb50…2946b)`) and all three new per-kind vectors execute under
  `TestKeyedConformanceAgreesWithRust` and `TestComposeVectorsMatchTheirProvenancePin`
  (confirmed with `-v` that the subtests ran, not that the package printed `ok`).
- QR budget: 210 bytes worst case (ripemd160 + hardened), up from 194; dim 53
  asserted and green.

**Seams walked and found clean:**

- **Padding.** Every read of the digest goes through `HashLock.Digest()`; `grep`
  for `.Digest()` returns 10 non-test call sites and all of them are
  width-correct. `==` and map-key use are compile errors via `_ [0]func()`.
  `MapKey()` cannot collide across kinds (no token is a prefix of another at the
  `:` boundary). `reflect.DeepEqual` in `md/compose_shape_test.go` does compare
  the padded array, but `NewHashLock` is the only constructor and always writes
  into a zeroed array, so the padding is structurally always zero.
- **Assignment sites.** `grep` for writes to `SpendPath.Hash` gives seven:
  two guarded by §8 (band 1, hex pad), three that are sha256-by-grammar (payload
  phrase record, payload preimage-plate record, preset), one `nil` clear, and one
  internal probe (`composerEditCanRenumber`). No route added by this diff
  bypasses the guard; the guard was added in commit 12 of 18, *after* every route.
- **`composerPickScreenFrom`.** Seven other `composerPickScreen` call sites are
  unaffected: the wrapper passes `initial = 0`, which short-circuits both the
  clamp and the re-home.
- **Decode side.** `md.go:464/474` binds tag→body, so `NewHashLock` in
  `policy_shape.collect` cannot fail and no digest can be silently dropped;
  `composerSelfCheck` now compares through `Equal` (kind + bytes). A mixed-kind
  two-path wallet composes, self-checks and consents correctly (verified by
  probe): `hash ripemd160 …` / `hash hash256 …` and "This wallet's hashes are
  ripemd160 and hash256."
- **Provenance.** `hashlockHeld`/`phraseDigests` keyed by `MapKey()` throughout;
  `p.Hash.Equal(h)` is nil-safe on a nil receiver, so the `composerPreimagePlatePick`
  rewrite of `p.Hash != nil && *p.Hash == h` is faithful. `hashlockPlateChoice`'s
  zero value is `hashlockPlateDecline`, so a held-but-unassigned preimage still
  defaults to not-cut.
- **`policySummaryLines` (`gui/template_engrave.go:188`)** still prints a bare
  `+hashlock`, which is correct under §6's both-or-neither rule: it shows no
  digest, so there is no kind axis to name.
- **Sweep for retired strings** across `cmd/emu/` and `scripts/`
  (`Type 64 hex`, `must be SHA-256`, `Write down this phrase`,
  `run ms hashlock with`, `hash <8hex>..<8hex>`): only the two `shots_composer.js`
  lines in I-1 remain.
