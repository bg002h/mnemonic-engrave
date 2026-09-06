# H6 plan — R0 round 0, the JOURNEY WALK

**Artifact:** `design/IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md` at engrave
master `e6d84d9c`, against `design/SPEC_hashlock_H6_preimage_plates.md` at `a2a031fa`.
Verified with `git diff --stat e6d84d9c..HEAD --` on both files: **no change since**.

**One question:** walk the operator through the device and host this plan builds,
asking at each step what they hold, what the thing does, and what ELSE they might
do. A divergence is a finding only when the wrong outcome is worse than telling
the operator nothing.

**What was driven.** Private `cp -a` copies of the plan author's three gated trees
(`/scratch/code/shibboleth/.tmp/h6-journey-{fork,me,ms}`; the originals under
`.tmp/h6-{gate,me,ms}` were never written to and are byte-identical at the end of
this run). Host binaries `me 0.8.1` and `ms 0.18.0` from the gated targets. Fork
`go build ./...` exit 0 on Go 1.26.7. `scripts/gui-shard-test.sh ./gui/ 24` run
four times over the walk: **1285 tests, partition verified exhaustive, 24 of 24
shards ok** — the plan's headline device gate reproduces. `go test ./cmd/emu/` ok;
`GOOS=js GOARCH=wasm go vet ./cmd/emu/` exit 0. `sha256sum
cmd/emu/walk_hashlock_phrase.js` = `990e67812a253061a18fe75df7ed4e83f982c4bcaf6b11b74878e40c8d6629c9`,
which is the literal the plan's Task 12 records. Everything below is a command and
its quoted output.

Findings are anchored on the PLAN. Where the spec says the same thing, both are
named, because the plan is what an implementer executes.

---

## Journey 1 — host-derived: `ms hashlock` → `me sysw pack --pack-preimage` (§12 item 3)

| # | what the operator has | what the tool does | what ELSE they might do | outcome |
| --- | --- | --- | --- | --- |
| 1 | the phrase `correct horse battery staple` | `ms hashlock --hashlock-phrase-stdin --out X.txt` prints `hash:3cf5d421…b70a4c12` on stdout, the 75-char ms1 to `X.txt`, and the engraving card on stderr | — | ok |
| 2 | `X.txt` + the `hash:` line | `me sysw pack --pack-preimage --no-passphrase --in records.txt` → §8.2.1 transit warning, `NOT SEALED`, exit 0 | pack **without** the flag → §8.1.1 by index, exit 4, correct text | ok |
| 3 | | | put the ms1 **on argv** → `argv_secret_guard` refuses before the parser runs with the H6 hashlock wording and offers `me sysw pack --in records.txt`; exit 3. §3.6 holds | ok |
| 4 | | | pack **only** `X.txt` (the minimal journey) → §8.2.3's **note**, not a WARNING. §12 item 3's own reasoning holds | ok |
| 5 | | | omit `--no-passphrase` → `sealing: SEALED`, then §8.2.4, then the generated passphrase | **M-1** — §8.2.4 says "the passphrase **above**"; it is printed *below*, and with `--passphrase-ask` it has not been asked for yet |
| 6 | | | `--pack-preimage` over a payload with no carrier → §8.2.2, exit 0 | ok |
| 7 | | | script it and guard with `--expect` → `unknown --expect kind "preimage"; want one or more of descriptor, cosigner, transaction, mnemonic, secret`; `--expect secret` is *not* satisfied by a preimage plate (`--expect secret was not met: NO record of that kind is in the stream. / Looking for an ms1 codex32 secret.`) | **M-2** |
| 8 | the cut plate's 75 characters | `ms hashlock --in X.txt` reprints `hash:3cf5d421…b70a4c12`, equal to step 1's | — | §12 item 3's read-back holds |
| 9 | the packed payload | `me sysw show` → `secret record 2: hashlock phrase (phrase:) — not shown` | — | ok |

## Journey 2 — the `phrase:` record, the carrier with no producer (§3.1, §13)

| # | what the operator has | what the tool does | what ELSE they might do | outcome |
| --- | --- | --- | --- | --- |
| 1 | a phrase and a method name | **nothing** — §13: "no verb emits one, so the operator hand-builds hex every time" | look for the wire form in `me sysw pack --help` | **I-1** — the help block documents `text:`, `pass:`, `tx:`, `key:`, `hash:`, `now:` bodies and **not** `phrase:` |
| 2 | | | follow the help's own recipe verbatim — `printf '%s' 'correct horse battery staple' \| xxd -p -c 256` — which is the anchor phrase and produces a body with no method and no comma | **I-1** — refused, and the refusal calls it "a `key:`/`hash:`/`now:` record" and offers build recipes for those three |
| 3 | `phrase:` + hex of `hardened,<phrase>` + the matching `hash:` | packs, seals-by-default, no orphan warning | — | ok |
| 4 | | | a **space after the comma** → §8.2.3's phrase WARNING with the derived digest and the space sentence | ok — the hand-build error is caught |
| 5 | | | the **wrong method selector** (`sha256,…` for a hardened digest) → the same WARNING | ok |
| 6 | | | any of the above with `--pack-preimage` and a *malformed* body → §8.2.2's "this payload holds no … `phrase:` record" printed **immediately above** the line that names their `phrase:` record's failing rule | **I-3** |

## Journey 3 — device-derived phrase to a cut plate (§5.1, §5.3, §5.4, §6)

Driven through the fork's touch harness (`runComposerEngraveStep`, `composerPickScreen`,
`composerReadScreen`), frames read with `ExtractText`.

| # | what the operator has | what the device does | what ELSE they might do | outcome |
| --- | --- | --- | --- | --- |
| 1 | a typed phrase, at HOLD | the confirm modal: `hash b867db87..edbc96cb / method: hardened chars: 28 / Write down this phrase, the method and this digest now. **The phrase and method are not on this device.** …` — and the next statement executed is `composerHoldHashlockMaterial` | — | **I-2** |
| 2 | | the reconcile screen: "Before you cut plates, run ms hashlock with this phrase and method on the host and check the digest matches." | — | ok |
| 3 | a mixed wallet (one keyed path, one hashed+held) | §8h is **drawn nowhere**: `composerEveryPathHashed = false`. Measured | — | pre-existing, recorded by §10.1; but it means step 1's body is the *only* thing the ordinary operator is told — see **I-2** |
| 4 | Done | pick step (A): `Preimage plate / hash bddf2d8a..bc59e8a9 path 2 / phrase: 8 characters method: sha256` then `preimage string`, `phrase + method`, `phrase + method + QR`, `do not cut this preimage` — all four on page 1 | — | ok, §5.3 F13 holds |
| 5 | | | press **Button1**, which is Back at the form pick, the engrave-mode pick and the census | **I-5** — the plate is silently DECLINED and the next screen is the census |
| 6 | | census page 1 is the shipped supply block only; §8.3's block (or the `declined` row) is on page 2 | press Button3 on page 1 | ok — measured: Button3 does **not** continue from page 1; the paging contract holds and the decline row is on a page the operator must visit |
| 7 | | after a decline-all the run **completes**: order `[policy]`, `composerPreimageMarkTitle = "PREIMAGE REQUIRED"`, no preimage plate, phrase scrubbed at flow exit, **no §8.4a** | — | **M-4** |
| 8 | an accepted `preimage string` plate | cut FIRST: `order == [preimage]` before `bundleEngrave`; abort with nothing cut → §8.4a "NO PREIMAGE PLATE WAS CUT. The phrase dies with this composition." | abort after one cut → §8.4b | ok, both arms reachable |
| 9 | two hashed paths | two pick screens, each naming its own `path 2` / `path 3`; census rows `path 2  ba7d532b..047e019a  phrase, sha256` and `path 3  e1c95035..b567c1fd  phrase, sha256`; §8h fourth arm chosen | one path held, one not | ok — the shipped arm is chosen, which §10.1 declines deliberately |

## Journey 4 — a payload phrase record and its lazy derive (§5.1 band 3, §5.2)

| # | what the operator has | what the device does | what ELSE they might do | outcome |
| --- | --- | --- | --- | --- |
| 1 | a payload with a `hash:`, a preimage and a `phrase:` record | `Which hash?` draws six rows, measured: `hash 1  3cf5d421..b70a4c12  (in payload)` / `preimage 1  3cf5d421..b70a4c12` / `phrase record 1 (derive to see the digest)` / `Type a hashlock phrase` / `Type 64 hex` / `No hash lock`; band starts 1/2/3/4/5 | — | ok |
| 2 | | picking band 3 derives once behind the countdown, no method pick, no reconcile screen | — | ok, §5.1's derive-only sibling |
| 3 | | picking band 2 draws `composerCopyHashlockPreimageConfirm`, never `method: hardened chars: 0` | — | ok, gate fix F3 holds |
| 4 | a digest matching no `hash:` record | the confirm modal's relation line reads `no hash: record in the payload has this digest` | — | ok |
| 5 | §12 item 3's exact payload (ms1 + `hash:`, no md1) | door lead `No keys loaded. This builds a key-less template. 1 preimage or phrase record loaded.`; `composerDoorHasPreimage = true`; the route is a member of `walletPolicyFlow`'s door loop | — | ok, §5.2's M-5 and F15 hold |
| 6 | | the flow lists `preimage 1  3cf5d421..b70a4c12`, locator `["hash  3cf5d421..b70a4c12", "matches hash 1 in the payload"]`, pick lead `preimage held, phrase not: only the string form can be cut`, rows `["preimage string" "do not cut this preimage"]` | — | ok |
| 7 | a phrase-only payload | rows `phrase record 1 (derive to see the digest)` → after derive `phrase 1  3cf5d421..b70a4c12`; locator after derive `["hash  3cf5d421..b70a4c12"]` | build the locator **before** the derive | **C-1** — locator is `["hash  00000000..00000000"]`, and nothing in the suite can see it |
| 8 | a plate just cut | the flow returns to the same list, unmarked | cut the same record again | **N-2** |

## Journey 5 — the mistakes

| mistake | what happens | outcome |
| --- | --- | --- |
| a kind-`0x03` string under id `entr` (the mistag) | host: the SHIPPED `TagKindMismatch` text, unchanged, with and without the flag; device: `sysw.Classify` = `ClassUnknown` | ok, §4.3's table holds |
| a kind-`0x03` string under id `seed` (the **1-in-256 collision**) | host: §8.1.2 with its last sentence — *"If this string is a 33-byte seed backup that happens to begin 0x03, it is not a preimage: roughly 1 in 256 of them look like this."*; device: `ClassUnknown` | ok — the collision sentence is on the arm a BIP-93 secret with a user-chosen id actually reaches |
| the malformed plate (X = 16 bytes) and the UPPERCASE plate | host: refused with §8.1.2 **with** the flag; device: `ClassUnknown` | ok, all three conjuncts hold |
| **with `--pack-preimage`, every one of the above ALSO prints §8.2.2 first** | `me: --pack-preimage was passed and this payload holds no preimage plate…` directly above `me: record 0 … is a kind-0x03 preimage payload whose 4-character id is not \`hash\`` | **I-3** |
| an ms1 string typed at Engrave Text / BIP-39 Password | §9 fires. `hashlock.IsMS1Shaped` measured true for the ungrouped plate, the card's group-5 spelling, the hyphenated spelling, UPPERCASE and an `entr` seed plate; false for a sentence, `ms1`, an md1 chunk and a phrase beginning "ms1 is my favourite…" | ok |
| a `phrase:` record at the Password program | `syswNoticeHashlockPhrase` is wired at `gui/passphrase_flow.go:684`, in the `else` of the `ClassPassphrase` offer, and guards on `has(ClassPhrase) && !has(ClassPassphrase)` | ok, §8.8 holds |

## Journey 6 — the controller's own runs: the emulator walk and the QR scan gate

| # | what the controller has | what the plan says | measured | outcome |
| --- | --- | --- | --- | --- |
| 1 | `cmd/emu/walk_hashlock_phrase.js` | four runs at sha `990e678…6629c9` | sha matches; `go test ./cmd/emu/` ok; `GOOS=js GOARCH=wasm go vet ./cmd/emu/` exit 0 | ok |
| 2 | the walk's H6 arm | §11.7: type the phrase, HOLD, reach the census, accept a plate, assert the census token equals the confirm modal's | the arm asserts `path 1` on the pick screen, `Plus 1 preimage plate`, `cut first and NOT part of this backup`, the token, `preimage string`, `Keep each preimage plate apart` | ok |
| 3 | | the walk takes the `preimage string` row | so §8.5's QR warning and the phrase forms are never walked; §12 leaves them to the operator | ok, stated |
| 4 | §12 item 8, the QR scan gate | "Cut it with the single-character test-plate pattern (**~2 s a try**) rather than a full plate (**~21 min**)" | worst-case phrase+QR plate = **43m32s**; the same plate without the QR = **14m40s**; the QR alone at scale 2 = **32m12s**; one 6 mm character = **4 s** | **I-4** |

---

## Findings

### C-1 — the Hashlock plates flow's phrase-form locator has no test that can fail, and the mutation the spec and plan name leaves 1285/1285 green

§5.2 item 4 is the strongest sentence in the stage:

> **The locator's `hash` row is printed from that result, so it is never blank.** A
> phrase-form plate with no locator at all -- a bearer phrase in plain text with no
> digest, no path, and no payload position -- is **the worst artifact this stage can
> cut**, and it is what a literal reading of the first draft produced.

Spec §11.5 states the guard: *"the plate it cuts carries a NON-EMPTY `hash` locator
row **whose digest equals the host's**. … MUTATION: omit the locator's `hash` row
**when the record is a phrase** → the non-empty assertion fails, which is the row
standing between the operator and a bearer plate with no locator at all."*
Plan Task 10 Step 6: *"**MUTATIONS, all five executed** (tails in `## Build gate`):
… **omit the locator's `hash` row** → the non-empty assertion fails, **which is the
row standing between the operator and a bearer plate with no locator at all**."*

**MEASURED.** `TestHashlockPlatesLocatorAlwaysCarriesTheHashRow`
(`gui/composer_hashlock_plates_test.go:202-257`) is the only test in the tree that
calls `hashlockPlatesLocator` — three call sites, at `:209`, `:218` and `:234` —
and **all three build the session from `composerTestPreimageRecord`**. A preimage
record arrives `derived: true` (`hashlockPlatesRecords` decodes X at list time), so
the carrier the rule is written for is never exercised:

```
$ grep -rn 'hashlockPlatesLocator\|hashlockPlateLocator' gui/*_test.go
gui/composer_hashlock_plates_test.go:209:  loc := hashlockPlatesLocator(s, hashlockPlatesRecords(s)[0])
gui/composer_hashlock_plates_test.go:218:  loc := hashlockPlatesLocator(s, hashlockPlatesRecords(s)[0])
gui/composer_hashlock_plates_test.go:234:  loc := hashlockPlatesLocator(s, hashlockPlatesRecords(s)[0])
$ sed -n '202,258p' gui/composer_hashlock_plates_test.go | grep -c composerTestPhraseRecord
0
```

**Counterexample 1 — the spec's own mutation, run.** Omit the `hash` row from
`hashlockPlatesLocator` *when the record is a phrase*:

```
$ go test -run 'TestHashlockPlates' ./gui/
ok      seedhammer.com/gui      0.064s
$ scripts/gui-shard-test.sh ./gui/ 24
RESULT: ok -- all 1285 tests ran across 24 shards
```

**Counterexample 2 — the regression the property actually guards against.** §5.2
item 4's clause is "printed from **that result**", i.e. after the derive. Move the
locator build one statement earlier in `composerHashlockPlatesFlow`, before
`hashlockPlatesDerive`:

```
$ scripts/gui-shard-test.sh ./gui/ 24
RESULT: ok -- all 1285 tests ran across 24 shards
```

That tree cuts a phrase-form plate whose locator reads
`hash  00000000..00000000` and which loses the `matches hash <i> in the payload`
row entirely (the zero digest matches nothing) — measured directly:

```
PROBE H: locator BEFORE derive = ["hash  00000000..00000000"]
PROBE H: locator AFTER derive  = ["hash  3cf5d421..b70a4c12"]
```

So "NON-EMPTY" is true by construction (`hashlockPlateLocator` always appends the
row) and carries none of the meaning §5.2 item 4 asks of it, and the half that does
carry the meaning — *"whose digest equals the host's"* — has no assertion anywhere.
The unconditional mutation *does* red the test, which is presumably what the gate
ran; the mutation both documents actually name does not.

This is the class the project keeps blocking: a gate that cannot fail, on the
funds path, guarding the artifact the plan itself calls the worst one this stage
can cut.

**SUGGESTION.** Add a phrase-record row to
`TestHashlockPlatesLocatorAlwaysCarriesTheHashRow` and make it assert the two
properties separately: (a) the row is present, and (b) **its digest equals
`hashlock.Digest(&hashlock.PreimageHardened(phrase))`** — the host's answer, which
is what §5.2 item 4 says. Then drive it through `composerHashlockPlatesFlow` rather
than by calling `hashlockPlatesLocator` directly, so the derive-then-locate ORDER
is what is under test; a unit call on an already-derived record cannot see the
ordering. Re-run both mutations above and quote the tails. Fix the spec's §11.5
mutation and the plan's Task 10 Step 6 line to name a mutation that fires.

### I-1 — H6 adds a fifth RESERVED prefix and updates no operator-facing description of it, on the one carrier that has no producer

§13 and the plan's follow-up table both record that nothing emits a `phrase:`
record: *"no verb emits one, so the operator hand-builds hex every time, and §3.3's
orphan warning is a mitigation rather than a fix."* §3.3 item 3 names the route the
operator is expected to follow: *"the operator hand-builds it as hex, **following
the `text:`/`pass:` precedent the `pack` help already sets**
(`crates/me-cli/src/main.rs:219`)."*

**MEASURED.** `main.rs:219` is `/// \`text:\`/\`pass:\` bodies are lowercase hex —
see the command help`, and the command help it points at documents the body of
every reserved prefix except the new one:

```
$ me sysw pack --help | grep -oE '`[a-z]+:' | sort -u
`hash:   `key:   `now:   `pass:   `phrase:   `text:   `tx:
$ me sysw pack --help | grep -c 'phrase:'
1
```

The single hit is inside `--pack-preimage`'s own flag text, which names `phrase:`
and never says what its body is. The block that carries `text:<hex of the UTF-8
bytes>`, `key:<hex of "[fingerprint/path]xpub">`, `hash:<64 lowercase hex>`,
`now:<hex of "<seconds>[,<height>]">` **and** the recipe
`printf '%s' 'correct horse battery staple' | xxd -p -c 256` gained no `phrase:`
line. That recipe's example string is the anchor phrase itself, so an operator
following the documented precedent literally produces the hex of the phrase alone —
the body without a method and without a comma.

**What they then see, measured:**

```
$ printf 'phrase:%s\n' "$(printf '%s' 'correct horse battery staple' | xxd -p -c 256)" > ph.txt
$ me sysw pack --pack-preimage --no-passphrase --in ph.txt --out /dev/null
me: --pack-preimage was passed and this payload holds no preimage plate and no `phrase:` record. Nothing was admitted that would otherwise have been refused.
me: record 0 (records count from 0) is a `key:`/`hash:`/`now:` record whose body fails its rule (not <method>,<phrase> with a known method and an admissible phrase).
      record 0: phrase: must be <method>,<phrase> as lowercase hex, with method hardened or sha256
      Build the record with `me sysw pack`'s helpers: a key record is `key:` + the hex of `[fingerprint/path]xpub` exactly as `md decompose` prints it; a hash record is `hash:` + the 32-byte digest as 64 lowercase hex; a now record is `now:` + the hex of `<seconds>[,<height>]`.
```

The middle line is correct and complete. The two lines around it are not: the
header calls a `phrase:` record "a `key:`/`hash:`/`now:` record", and the closing
advice lists build recipes for those three prefixes and not for the one the
operator is holding. That is exactly the class the plan itself treats as blocking
one file away — Task 3 Step 10 rewrites `argv_secret_guard`'s bearer arm because
*"a refusal that names the wrong material is a defect in what the tool claims to
have found, not a nicety."* The same edit was not made to `U::Composer`
(`crates/me-cli/src/main.rs:3071-3079`), and the plan's Task 3 Step 8 (*"the two
operator-facing surfaces the new classes reach"*) enumerates `class_name` and
`print_composer_confirmation` only. Neither plan nor spec mentions `U::Composer`
or the `pack` doc comment: `grep -n 'U::Composer\|Build the record with' ` over both
documents returns nothing.

**SUGGESTION.** Three edits in Task 3, each one line: (1) add
`phrase:<hex of "<method>,<phrase>">` to the `Pack` doc comment beside the other
prefixes, naming `hardened|sha256` and the cut-on-the-first-comma rule, since with
no producer this help IS the producer; (2) extend `U::Composer`'s header and its
"Build the record with…" list to cover `phrase:`; (3) add the same one word to
`U::Unrecognised`'s enumeration (see M-3). Task 13 Step 3's toolkit-manual entry
should carry the wire form too, not only `--pack-preimage` and the plate forms.

### I-2 — the confirm modal that gates funds still says the phrase is not on the device, and it is not among §0's four falsified records

Spec §0 enumerates the shipped records this stage falsifies, *"named so nothing is
folded silently"*, and Task 13 Step 1 rewrites those four. A fifth is missing.

**MEASURED**, byte-identical at fork baseline `fb0dd04` and in the gated tree
(`git show fb0dd04:gui/composer_copy.go` vs the gated file):

```
$ # composerCopyHashlockConfirm, gui/composer_copy.go:437
hash  b867db87..edbc96cb
method: hardened   chars: 28
Write down this phrase, the method and this digest now. The phrase and method are not on this device. Without both, this path can never be spent.
One phrase per policy. Never use this phrase as a passphrase or a password anywhere else.
```

H6 makes "The phrase and method are not on this device" false: §2.2 stores both in
`hashlockHeld` for the composition's lifetime, and §6 engraves both onto a plate.
In `hashlockPhraseRoute` the falsification is one statement wide — the modal is
drawn, and on acceptance the *next* production statement is
`composerHoldHashlockMaterial(st, d, hashlockMaterial{phrase: phrase, method: m, …})`.

The stage already identified this exact sentence and this exact direction of error,
and fixed it in the other place it appears. `gui/composer_copy.go:556-559`, wired by
this plan's Task 9 Step 6:

> THE HELD ARMS COME FIRST because they are the true statement when they apply: the
> shipped two say the preimage *"is not on this device"*, and H6 §2.2 makes that
> false for a composition that holds it. **Saying a backup does not exist when it is
> about to be cut is the direction that costs the operator a plate.**

But §8h — the family that reasoning fixed — is guarded by `composerEveryPathHashed`,
which §10.1 itself records is *"false the moment ONE path is keyed -- i.e. on the
ordinary mixed hashlock wallet"*. Measured on the smallest composition §5.3 applies
to (one keyed path, one hashed path whose phrase is held):

```
PROBE E: composerEveryPathHashed  = false   (the GUARD on §8h's call site)
PROBE E: composerEveryHashedPathHeld = true
```

So on the ordinary wallet the stage fixed the sentence on the banner that is drawn
**nowhere**, and left it false on the modal that is drawn on **every** phrase route
— which is then the only thing the operator is told about where the phrase lives
before the Done review offers to engrave it.

The error errs safe for the phrase (it asks for a write-down the operator should
make anyway); it errs the other way for the plate, which is the direction the
stage's own comment names.

**SUGGESTION.** Add `gui/composer_copy.go:437` as a fifth entry to spec §0's list
and to Task 13 Step 1, and rewrite the middle sentence in Task 9 so it is true of
what the composition now holds — e.g. *"This composition holds the phrase and
method until it ends, and can cut a plate for them at Done. Write them down anyway:
without both, this path can never be spent."* Re-measure it through
`assertModalBodyFits` (the current body is well inside the margin, so there is
room) and add its row to §8.9's table.

### I-3 — §8.2.2's no-op warning fires for exactly the carriers §4.3 narrowed out, so `me` contradicts itself in two consecutive lines

§3.3 item 2 conditions the warning on *"the flag is given and no preimage or
`phrase:` record is present"*, and `report_preimage_admission` implements "present"
as `classify(r) ∈ {Preimage, Phrase}`. Every carrier §4.3 deliberately excludes —
the mistagged `entr`, the wrong-id plate, the malformed 16-byte X, the UPPERCASE
spelling, and every malformed `phrase:` record — classifies `Unknown`, so
`carriers.is_empty()` is true and the warning fires. The refusal then names the
same record as the thing the warning just said was absent.

**MEASURED**, all four shapes; the wrong-id row in full:

```
$ me sysw pack --pack-preimage --no-passphrase --in r.txt --out /tmp/x.bin
me: --pack-preimage was passed and this payload holds no preimage plate and no `phrase:` record. Nothing was admitted that would otherwise have been refused.
me: record 0 (records count from 0) is a kind-0x03 preimage payload whose 4-character id is not `hash`. A preimage plate is kind 0x03 under the id `hash` (SPEC_ms_hashlock rule 2), and --pack-preimage admits only that. …
```

and the `phrase:` hand-build case, which is the one that matters because §13 makes
hand-building the only route:

```
me: --pack-preimage was passed and this payload holds no preimage plate and no `phrase:` record. Nothing was admitted that would otherwise have been refused.
me: record 0 (records count from 0) is a `key:`/`hash:`/`now:` record whose body fails its rule (not <method>,<phrase> …)
```

The first line is printed first (`report_preimage_admission` runs at
`main.rs:1518`, `admit_check` later), so it is what the operator reads first, and
its plain meaning is "drop the flag, it did nothing" — which is the wrong next
move. §12 item 5's own standard is *"refused on the host with §8.1.2, **one refusal
and not three**"*; here the operator gets a false statement and then the refusal.

**SUGGESTION.** In Task 3 Step 6, suppress §8.2.2 when any record is *shaped* like
a carrier but not admissible — the cheapest predicate is the diagnostic that
already exists: `seal::record::preimage_plate(r)` (deliberately kept as the wide
DIAGNOSTIC by §4.3) or `r.starts_with(PHRASE_PREFIX)`. The refusal that follows
already says everything the operator needs. Add a row to
`crates/me-cli/tests/sysw_pack_preimage.rs`:
`the_no_op_warning_is_silent_when_a_carrier_shaped_record_is_present`, with the
mutation "restore the unconditional emptiness test → the wrong-id row prints both
lines".

### I-4 — §12 item 8's QR gate carries a cost model that is wrong by ~900×, and names a technique that cannot be applied to the artifact under test

Spec §12 item 8, transcribed verbatim into the plan's Task 13 Step 4:

> Cut it with the single-character test-plate pattern (**~2 s a try**) rather than a
> full plate (**~21 min**).

**MEASURED** on the plan's own gated tree, at production `internal/sh2.Params()`
(`Millimeter = 6400`, `StrokeWidth = 1920`), by `engrave.TimePlan`, and
independently by the device's own instrument (`toPlate` → `Plate.Duration`, which
is what the engrave screen counts down):

| artifact | `engrave.TimePlan` | `Plate.Duration` |
| --- | --- | --- |
| **the worst case §12 item 8 names** — 100-char hardened phrase, v9, 53 modules, scale 2, plus QR | **43m32s** | 41m11s |
| the same plate with the QR removed | 14m40s | 10m45s |
| **the QR alone at scale 2** (`ConstantQRCmd.Engrave`) | **32m12s** | — |
| the anchor phrase + QR (v6, 41 modules) | 26m51s | 24m38s |
| one 6 mm character — the *"~2 s a try"* pattern | **4 s** | — |

Two things are wrong, and both mis-plan the one gate the spec calls *"a GATE, not
an assumption"*:

1. **"~21 min" is not this plate.** 21 minutes is the md1-plate figure from the
   single-character-test-plate record; the artifact under test is **43m32s**.
2. **The single-character pattern cannot be applied to it.** A 53-module
   constant-time QR is not divisible into characters — `ConstantQRCmd.Engrave` runs
   `for range nmod` (1399 moves at v9) and pads every move to `maxDur`, by
   construction, which is the whole reason the toolpath is content-independent.
   Cutting "just the QR" instead of the whole plate saves 11 minutes out of 43, not
   43 minutes down to 2 seconds.

The gate fails open by design (*"If it does not scan, the QR toggle does not ship"*),
so its cost is what decides whether it is actually run. A gate budgeted at 2 seconds
and costing half an hour per attempt is one that gets deferred, and a deferred gate
here ships an untested 53-module QR carrying a spend secret.

**SUGGESTION.** Replace the last sentence of §12 item 8, and the same sentence in
Task 13 Step 4, with the measurement: *"the worst-case plate is a 43m32s cut and its
QR alone is 32m12s at production params (`engrave.TimePlan`, `sh2.Params()`), so
budget one attempt per session; the QR is indivisible — `ConstantQRCmd.Engrave` runs
`for range nmod` — so the single-character test-plate pattern does not apply."*
If a cheaper gate is wanted, say so explicitly and give it its own number: cutting
the QR alone onto a blank at 32m12s (skip the text block) is the only reduction
available, and it is a 25% saving, not a 900× one. Add the two durations to §11.4
as a logged measurement so a later layout change that doubles them is visible.

### I-5 — the Back contract §5.3 step (A) and Task 9 Step 1 state is not what the gated tree does, and its own file says so

Plan Task 9 Step 1 (`design/IMPLEMENTATION_PLAN…:2777-2780`):

> **Back contract:** Button1 is `do not cut` for the highlighted plate, matching
> `composerPickScreen`'s shipped decline arm; **backing out of the STEP returns to
> the engrave step's entry** and thence round `composerFlow`'s loop with the state
> intact, which is §2.2 item 4 unchanged.

Spec §5.3 makes it unambiguous — *"backing out of the STEP **(the first screen's
Back)** returns to the engrave step's entry"*. One button cannot both decline the
highlighted plate and back out of the step, and the gated tree implements the first.
`gui/composer_preimage_plate.go`'s own header says the opposite of both documents:

> BACK IS `do not cut` FOR THE HIGHLIGHTED PLATE … **Backing out of the STEP is not
> offered here**: §2.2 item 4 keeps the composition intact through composerFlow's
> own loop, and a Back that unwound the whole step would leave the operator no way
> to answer the question for the remaining digests.

`composerPreimagePlateStep` has no `ok` return; `composerPickScreen`'s `backBtn`
returns `(0, false)` and `composerPreimagePlatePick` maps that to
`hashlockPlateDecline`.

**MEASURED, driven through the harness.** At the first pick screen, press Button1 —
the button that means Back at the form pick, at the engrave-mode pick and at the
census:

```
PROBE A: Back at the pick step -> next screen is the CENSUS, page 1:
   PlatesToCut This engraves 1 plate. md1 template: 1 plate (key-less wallet policy) …
PROBE A: census page carrying the outcome:
   PlatesToCut preimage bddf2d8a..bc59e8a9: declined, will not be cut
PROBE A: after the census, next screen: "Choose engraving TEXT ONLY Card 1 of 1 | Plate 1 of 1"
PROBE A: plates handed to the engraver: [policy]
PROBE A: composerPreimageMarkTitle = "PREIMAGE REQUIRED"
```

So a Back press yields a completed run with md1 and mk1 plates stamped
`PREIMAGE REQUIRED`, no preimage plate, and a device-typed phrase scrubbed at
`composerFlowExit`. The mitigation is real and worth recording: `composerReadScreen`
withholds Button3 until the last page has been laid out, measured —

```
PROBE C: Button3 on census page 1 -> reached the engraver? false
```

— so the operator must page, and the `declined, will not be cut` row is on the page
they must visit. They still need to read it.

**SUGGESTION.** The behaviour is defensible; the documents are not. Delete the
second clause from Task 9 Step 1 and from spec §5.3 step (A), and replace it with
the file's own sentence: *"Backing out of the STEP is not offered; Button1 declines
the highlighted plate and the step advances. The only Back out of the review is the
census's, which returns false from `composerEngraveStep` and sends `composerFlow`
round its loop with the state intact (§2.2 item 4)."* Then take M-4 with it, since
the two compose.

### M-1 — §8.2.4 says "the passphrase above" and the passphrase is below it

`report_sealed_preimage` is called immediately after `decide_sealing`
(`crates/me-cli/src/main.rs:1739-1741`); the passphrase ceremony runs after
(`:1746-1770`). Measured:

```
sealing:  SEALED — this payload holds secret material (record 0 (hashlock preimage plate)), so it is encrypted
      and opens only with the passphrase you chose with --passphrase-words. …
me: this payload is SEALED and holds a hashlock preimage, so the device needs the passphrase above before it can reach it. …
passphrase — write this down and store it APART from the machine:

    develop token public opera
```

With `--passphrase-ask` it is worse: the note prints before the prompt, so there is
nothing above it at all —

```
me: this payload is SEALED … the device needs the passphrase above before it can reach it. …
me: reading the passphrase: …            <- the prompt comes after
```

§3.3's justification (*"AFTER the sealing line, because its own wording … refers to
it"*) is satisfiable by reading "above" as the sealing line's *mention* of a
passphrase, but that is not how the sentence reads on a terminal.

**SUGGESTION.** In Task 3 Step 7, change "the passphrase above" to "this payload's
passphrase". No re-ordering, no re-measurement.

### M-2 — `--expect` has no vocabulary for the two new classes

`--expect`'s whole argument (§6g) is that *"a backup can be silently incomplete …
`me sysw pack` builds a container from the `md1` records alone at exit 0"*. A
scripted H6 pack whose `X.txt` came back empty packs a payload with no preimage at
**exit 0**, with only §8.2.2 on stderr. Measured:

```
$ me sysw pack --pack-preimage … --expect preimage …
me: unknown --expect kind "preimage"; want one or more of descriptor, cosigner, transaction, mnemonic, secret
$ me sysw pack --pack-preimage … --expect secret …
me: --expect secret was not met: NO record of that kind is in the stream.
      Looking for an ms1 codex32 secret.
```

so `secret` does not substitute. Neither `--pack-preimage` (a loosening flag whose
no-op case is a warning by §3.3 item 2) nor `--expect` can make a preimage's
presence a requirement.

**SUGGESTION.** File it as a follow-up owned by a later `me` cycle, with the
one-line remedy: add `preimage` to `sysw::expect`'s vocabulary, satisfied by
`Class::Preimage | Class::Phrase`. It is satisfiable, so it does not fall under the
`address`/`passphrase` exclusion the flag's own help states.

### M-3 — `U::Unrecognised`'s enumeration of the prefixed forms omits `phrase:`

`crates/me-cli/src/main.rs:3065-3070`: *"not a `text:`/`pass:`/`tx:`/`key:`/`hash:`/
`now:` record"*. H6 adds a fifth reserved prefix and the sentence still lists six.
Lower stakes than I-1 (a `phrase:`-prefixed record reaches `U::Composer`, not this
arm), but it is the same omission and the same one-word fix.

**SUGGESTION.** Fold into I-1's Task 3 edit.

### M-4 — a decline-all completed run reaches §8.4a's exact end state without §8.4a

§8.4a exists for *"NO PREIMAGE PLATE WAS CUT. The phrase dies with this composition.
Do not fund this wallet."* It fires only on an engrave FAILURE with `cut == 0`.
Measured (PROBE A above): declining every plate and completing the run reaches the
identical end state — `PREIMAGE REQUIRED` on the md1 and mk1 plates, no preimage
plate, the phrase gone at flow exit — and draws no arm at all. §5.3 item 5 is right
that declining must not abort the run (the operator may already hold the plate;
§13 makes cross-run awareness out of scope), so a refusal would be wrong. What is
missing is a notice at the moment of no return.

**SUGGESTION.** Non-gating on its own; it earns a line because I-5 makes it
reachable by a Back press. Add to Task 9 Step 4 a third, narrow arm on the same
counter: when `cut == 0` **and** every held plate was declined **and** some held
material's provenance is `hashlockFromPhrase` (so the phrase exists nowhere but this
composition), draw a confirm-to-proceed before `bundleEngrave` rather than an abort
— *"No preimage plate will be cut. This composition holds the only copy of the
phrase; it ends with the run."* If that is judged out of scope, record the
asymmetry in §13 beside "cross-run awareness" so the next reader sees a decision.

### N-1 — the plan says the walk was run three times; its gate table records four

Task 12 Step 2: *"**run it three times** — unmutated, and twice against a
mutation."* The boundary-gate table records four: *"(a) ok=true 60.3 s; (b) census
digest dropped → RED; (c) census digest perturbed → RED; (d) ok=true 60.3 s."* The
fourth run (the re-clean) is the better protocol; the step should say so.

### N-2 — the Hashlock plates flow returns to an unmarked list after a successful cut

`composerHashlockPlatesFlow`'s loop `continue`s after a successful engrave, back to
the same list with the same rows. A phrase row does change (`phrase 1 (derive to
see the digest)` → `phrase 1  <digest>`), but a preimage row is identical before and
after. §13's out-of-scope entry is about *cross-run* awareness; within one flow the
information exists and is discarded. A 32-minute cut makes an accidental repeat
expensive, and §8.3's own line tells the operator to store duplicates apart.

**SUGGESTION.** One field on `hashlockPlatesRecord` and one word on the row.
Follow-up, owned by a later device cycle.

---

## Closing counts

**1 Critical / 5 Important / 4 Minor / 2 Nit.**

- **C-1** the Hashlock plates flow's phrase-form locator gate cannot fail; both named mutations verified against the full 24-shard suite
- **I-1** `phrase:` is a reserved prefix no operator-facing text describes, and its refusal names the wrong record kinds
- **I-2** the HOLD confirm modal still says the phrase is not on the device; not among §0's four falsified records
- **I-3** §8.2.2 fires for the carriers §4.3 narrowed out and contradicts the next line
- **I-4** §12 item 8's QR gate cost model is wrong by ~900× and its technique cannot be applied
- **I-5** the pick step's Back contract in the plan and spec is not the one the gated tree implements
- **M-1** §8.2.4's "the passphrase above" points below it
- **M-2** `--expect` cannot require a preimage
- **M-3** `U::Unrecognised` omits `phrase:`
- **M-4** decline-all reaches §8.4a's end state without §8.4a
- **N-1** three runs claimed, four recorded
- **N-2** the plates flow's list does not mark what it just cut

**What reproduced.** Every number the plan carries that this walk touched:
`constantTimeQRModules` at 843/1013/1199/1399 with the shipped five untouched, the
73-character hardened method line, the 122/63/194-byte QR texts, the four pick-row
labels on one page, the `(in payload)` annotation, the two-line masked lead, the
census's forced paging, the deterministic plate order, both §8.4 arms reachable and
neither on a completed run, `PREIMAGE REQUIRED` on md1 and mk1, the ms1 read-back
through `ms hashlock --in`, all four §8.2 warnings on their conditions, the id
partition with all three conjuncts, device inertness of every collision shape, the
walk file's sha256, and `1285 tests / 24 of 24 shards ok` — run four times,
including twice under mutation.

**Read-only.** Nothing was committed. The plan author's trees under
`.tmp/h6-{gate,me,ms}` were never written to; both files this walk mutated in the
private copy were restored and verified byte-identical with `cmp`, and the copies
are removed.
