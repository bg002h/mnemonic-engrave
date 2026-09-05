# hashlock H2 — post-implementation lens: **can the emulator walk fail?**

**Lens question (the only one answered here):** does
`cmd/emu/walk_hashlock_phrase.js` FAIL when the device is wrong — or is it a
control testing the wrong layer?

**Tree under test:** fork `hashlock-h2` @ `17b3979`, in a throwaway detached
worktree `/scratch/code/shibboleth/.tmp/h2-wf-lens-walk-control` (created from
`17b3979`, every mutation reverted, `git status --short` empty at the end, then
removed). Nothing committed, nothing pushed, no sub-agents.

**Method:** four emulator builds — one unmutated and three with a single
deliberate defect each — every one served on its own fresh port and driven
through playwright with

```js
async () => { const w = await import('./walk_hashlock_phrase.js');
              try { return JSON.stringify(await w.run()); }
              catch (e) { return 'THREW: ' + e.message; } }
```

**Answer, in one line: the walk is a real control for the property it claims.**
Both prescribed mutations make it throw, on the exact assertion they should, and
the unmutated build passes. Its blind spots are real but each one is RED under a
named CI test that I ran and watched fail. **0 Critical, 0 Important.**

---

## The four runs

| # | mutation | walk result |
| --- | --- | --- |
| baseline | none | **PASS** (`ok: true`) |
| **M1** | `hashlockFirst8Last8` shows the wrong last-8 window | **FAIL** on the typed assertion |
| **M2** | `hashlock.Iterations` 100000 → 99999 | **FAIL** on the hardened assertion |
| M3 | hash assigned BEFORE the hold | PASS (blind — see M-1) |
| M4 | correct digest displayed, corrupted digest STORED | PASS (blind — see M-2) |

### Baseline — unmutated `17b3979`, port 8811

```
{"typed":"hashb867db87..edbc96cbmethod:sha256chars:28Writedownthisphrase…",
 "control":"hashc8043156..253e7389method:sha256chars:27…",
 "mixed":"hash95d44470..2297a7ffmethod:sha256chars:28…",
 "hardened":"hash3cf5d421..b70a4c12method:hardenedchars:28…",
 "ok":true,"hardenedFirstFrame":"Write down this phrase",
 "reconcile":"Beforeyoufundthiswallet,runmshashlockwiththisphraseandmethodonthehostandcheckthedigestmatches.Hashlock",
 "pathRow":"Spendpathsslots:0Path1:hashonlyAddaspendpathChangethescriptDone"}
```

All eight fields are **byte-identical** to the implementation report's recorded
run at `e1bf137` (`design/agent-reports/hashlock-H2-implementation-report.md`
lines 158-168). The record reproduces.

### M1 — the digest display (walk must FAIL)

```diff
 func hashlockFirst8Last8(h [32]byte) string {
 	s := hex.EncodeToString(h[:])
-	return s[:8] + ".." + s[len(s)-8:]
+	return s[:8] + ".." + s[len(s)-9:len(s)-1] // MUTATION M1: wrong last-8 window
 }
```

`gui/composer_hashlock.go:131-134`. Rebuilt, served on a fresh port 8812.
Verbatim result:

```
THREW: the anchor phrase's sha256 digest (corpus derivation[0].sha256_h): the screen does not
carry "b867db87..edbc96cb".
Screen: "hashb867db87..6edbc96cmethod:sha256chars:28Writedownthisphraseandthemethodnow.…"
```

The walk threw on trial 1's `must(typed, ANCHOR_SHA_H, …)`
(`walk_hashlock_phrase.js:290`) — the shifted window `6edbc96c` is exactly
characters 55..62 of the true digest, so the mutation was live and the walk read
the mutated device. Spec line: SPEC_hashlock_H2_device §4.5 (the confirm modal
carries the digest first8..last8).

### M2 — the hardened derivation (walk must FAIL)

```diff
-const Iterations = 100000
+const Iterations = 99999 // MUTATION M2
```

`hashlock/hashlock.go:24`. Fresh port 8813. Verbatim result:

```
THREW: the anchor phrase's hardened digest (corpus derivation[0].hardened_h): the screen does
not carry "3cf5d421..b70a4c12".
Screen: "hashec6b5f29..bafb90c7method:hardenedchars:28Writedownthisphraseandthemethodnow.…"
```

Threw on `must(hardened, ANCHOR_HARD_H, …)` (`walk_hashlock_phrase.js:313`).
The three SHA-256 trials passed first (they do not touch `Iterations`), so the
failure is localised to the hardened arm. **The displayed value is the right
answer for the wrong constant** — recomputed independently:

```
$ python3 -c "import hashlib;x=hashlib.pbkdf2_hmac('sha256',b'correct horse battery staple',
    b'ms-hashlock-v1',99999,32);h=hashlib.sha256(x).hexdigest();print(h[:8]+'..'+h[-8:])"
ec6b5f29..bafb90c7
```

so a **one-iteration** deviation in PBKDF2 is caught. Spec line:
SPEC_hashlock_H2_device §3 (`HASHLOCK_ITERATIONS = 100_000`, salt
`ms-hashlock-v1` as a 14-byte slice).

---

## The oracle is genuinely the corpus (machine-checked)

The walk's four hard-coded constants are the only thing standing between it and
self-agreement, so I resolved every one against the vendored corpus and then
against an independent recomputation.

```
$ sha256sum hashlock/testdata/hashlock-v0.8.json
a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30
$ sha256sum /scratch/code/shibboleth/mnemonic-secret/crates/ms-codec/tests/vectors/hashlock-v0.8.json
a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30
```

— equal to each other, equal to the `sha256` field of
`hashlock/testdata/hashlock-v0.8.provenance.json`, and the parsed JSON is
identical (`diff` of both sorted dumps: no output). 11 derivation rows, as the
pin claims.

| walk constant | corpus field | corpus value | independent recompute |
| --- | --- | --- | --- |
| `ANCHOR_SHA_H = "b867db87..edbc96cb"` | `derivation[0].sha256_h` | `b867db87…edbc96cb` ✓ | `sha256(sha256(phrase))` ✓ |
| `ANCHOR_HARD_H = "3cf5d421..b70a4c12"` | `derivation[0].hardened_h` | `3cf5d421…b70a4c12` ✓ | `sha256(pbkdf2(…,100000,32))` ✓ |
| `MIXED_SHA_H = "95d44470..2297a7ff"` | mixed-case row `sha256_h` | `95d44470…2297a7ff` ✓ | ✓ |
| `CONTROL` phrase | *not* a corpus row ✓ (11 rows enumerated) | — | observed `c8043156..253e7389` = `sha256(sha256("correct horse battery stapl"))` ✓ |

So no constant is a transcription of the device's own output, and the negative
control's observed digest is a real derivation of what was typed rather than a
blank — the implementation report's claim on that point holds.

---

## Every assertion the walk makes

Line numbers are `cmd/emu/walk_hashlock_phrase.js` at `17b3979` (331 lines).

**Preconditions (L262-266):** `shScreen`, `shTargets`, `shTap`, `shPress`,
`shRelease`, `shSysw` all exist (the stale-wasm guard). `chooseRow` re-checks
`shTargets` and refuses a row index the frame does not draw (L166-174).

**Route in (L271-287):** `shSysw("none")` → `Load it?` → Back → `SeedHammer` →
`goTo("Wallet Policy")` within 14 carousel taps → `Which script?` →
`Start from?` → `Add a spend path` → `What can spend on this path?` →
`EXPERIMENTAL` → hold (§8a key-less consent) → `Type a hashlock phrase`, then
two `must`s on the §4.1 no-payload lead: `No hash record in the payload` and
`ms hashlock on the host`.

**Per trial (L230-249):** the §8i rule modal (`32-byte value`); the phrase
screen (`Hashlock phrase`); **the counter reads exactly `<len>/100` after
typing** (L212 — this is what proves no residue survived the previous trial's
Back); `Which method?`; for SHA-256 the §4.3b brainwallet modal always appears;
`Write down this phrase`; `method: <sha256|hardened>`; `chars: <n>`.

**Per trial, the digest (L289-315):** trial 1 `must` the corpus SHA-256
constant + `One phrase per policy`; trial 2 `mustNot` `b867db87` (the negative
control); trial 3 `must` the mixed-case constant and `mustNot` the lowercase
one; trial 4 `must` the hardened constant and `mustNot` the SHA-256 one.

**Back contract (L252-259):** the three screens `Which method?` →
`Hashlock phrase` → `Type a hashlock phrase` each appear.

**After HOLD (L318-324):** the §4.5 reconciliation screen
(`run ms hashlock with this phrase`, `check the digest matches`), then
`Spend paths` and `hash` on the path row.

---

## What the walk never inspects

Every item below was checked against the tree; where a CI test covers it I name
the test, and for the two marked **measured** I built the defect and watched the
walk pass anyway.

1. **Nothing assigned before HOLD — measured blind (M-1 below).**
2. **The digest STORED equals the digest SHOWN — measured blind (M-2 below).**
3. **The countdown screen.** `raceFor(["Deriving","Write down this phrase"])`
   (L244) records `hardenedFirstFrame` and asserts nothing; the percentage, its
   advance, the zero-state lead `Deriving. This takes about 10 seconds.` and
   `About N seconds left` are never read. Deliberate and documented at L215-229
   (a timing assertion dressed as a behavioural one). CI:
   `TestHashlockDerivingLead`, `TestHashlockDeriveKeepsAwakeUnderTheScreensaver`.
4. **Back DURING the derivation** (§4.4's abandon path) is never pressed.
   CI: `TestHashlockBackContractKeepsThePath`.
5. **"The phrase survives an inner Back"** — the other half of §4.6.
   `backToWhichHash` waits for `Hashlock phrase` but never reads the counter, so
   a route that dropped the phrase on the *method-pick* Back would still pass.
   CI: `TestHashlockBackContractKeepsThePath` (asserts `28/100` after that Back).
   Note the *drop* half IS covered, indirectly but soundly: a retained phrase
   would make the next trial's `waitFor("27/100")` (L212) time out.
6. **§4.5's relation line.** The walk runs `shSysw("none")`, so
   `composerPayloadDigests` is empty, `hashlockRelationLine` returns `""`
   (`gui/composer_hashlock.go:96-99`) and **neither**
   `matches hash N in the payload` **nor** `no hash: record in the payload has
   this digest` is drawn in any of the four trials — confirmed absent from all
   four observed modals. CI: `TestHashlockConfirmRelationLine`.
7. **The other-path warning** `another path has a different hash: two phrases to
   back up` — one path only, never fires. CI:
   `TestHashlockOtherPathLineIsSilentOnAnEqualHash`.
8. **The §4.3a hardened-under-20-chars modal** — the phrases are 27 and 28
   characters, so it cannot fire, and its *absence* is not asserted either
   (`chooseRow(0, null, …)`, L242). CI: `TestHashlockMethodModalsFireOnCondition`.
9. **Every §2 refusal** (empty, non-printable, ms1-shaped, >100 chars, 64-hex) —
   never typed. CI: `TestHashlockPhraseRefusalsOnScreen` + the `hashlock`
   package's own corpus-driven refusal table.
10. **The other `Which hash?` rows** (§5: payload rows, `Type 64 hex`,
    `No hash lock`) and Back at `Which hash?` deleting the path at creation. CI:
    `TestComposerHashEditDispatchesByRowLabel`, `TestHashlockHexRowBackKeepsThePath`.
11. **The reconcile copy is not verbatim** — two fragments of
    `composerCopyHashlockReconcile()` (`gui/composer_copy.go:443-446`), not the
    whole sentence.

---

## Findings

### M-1 — the walk passes when the device assigns the hash BEFORE the hold; CI is what catches it

**Measured, not argued.** `gui/composer_hashlock.go:67-68`:

```diff
 			body := composerCopyHashlockConfirm(hashlockFirst8Last8(h), m.String(), len(phrase),
 				hashlockRelationLine(payload, h), hashlockOtherPathLine(st, idx, h))
+			dEarly := h // MUTATION M3: assign BEFORE the hold
+			st.list.Paths[idx].Hash = &dEarly
 			if composerConfirmScreen(ctx, th, "Hash lock", composerConfirmBody(body)) {
```

Fresh port 8814, verbatim: `{"typed":…,"control":…,"mixed":…,"hardened":…,`
**`"ok":true`**`,…}` — the walk passed all four trials with the defect live,
because trials 1-3 back out of the confirm modal and the walk never re-reads the
path's state afterwards.

The property is nevertheless gated, in CI, and I ran it against the same
mutation:

```
$ go test ./gui/ -run TestHashlockBackContractKeepsThePath -count=1
--- FAIL: TestHashlockBackContractKeepsThePath (2.06s)
    composer_hashlock_test.go:442: hash assigned before HOLD
FAIL	seedhammer.com/gui	2.068s
```

Spec line: §4.6 — "Back on the confirm → method pick, nothing assigned."
**Not Critical or Important:** the shipped device is correct here
(`gui/composer_hashlock.go:67-70` assigns only inside the `if`), the guarantee
has an executed gate, and the walk never claimed this ground. Recorded so nobody
later treats the walk as the acceptance for §4.6.

### M-2 — the walk passes when the stored digest differs from the displayed one; CI catches that too

`gui/composer_hashlock.go:68-69`, display left correct, storage corrupted:

```diff
 			if composerConfirmScreen(ctx, th, "Hash lock", composerConfirmBody(body)) {
 				d := h
+				d[0] ^= 1 // MUTATION M4: store a digest the screen never showed
 				st.list.Paths[idx].Hash = &d
```

Fresh port 8815, verbatim: all four digest strings correct, **`"ok":true`**,
`"pathRow":"Spendpathsslots:0Path1:hashonlyAddaspendpathChangethescriptDone"`.
The walk reads only the modal text and then `must(list, "hash")` (L323), which
is a label, not a value — so a device that showed the operator one digest and
locked the funds to another would pass the walk. That is the worst shape this
stage can have (the operator writes down a digest the wallet does not use), and
it is why I built it.

Gated in CI, run against the same mutation:

```
$ go test ./gui/ -run 'TestHashlockPhraseRouteSetsTheCorpusDigest|TestHashlockPhraseRouteDoesNotNormalise' -count=1
--- FAIL: TestHashlockPhraseRouteSetsTheCorpusDigest/hardened_anchor (2.06s)
    composer_hashlock_test.go:365: path hash = &[61 245 212 33 …], want 3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
--- FAIL: TestHashlockPhraseRouteSetsTheCorpusDigest/sha256_anchor (3.03s)
    composer_hashlock_test.go:365: path hash = &[185 103 219 135 …], want b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb
--- FAIL: TestHashlockPhraseRouteDoesNotNormalise (3.04s)
    composer_hashlock_test.go:397: "Correct Horse Battery Staple": path hash = &[148 212 68 112 …], want 95d4447031cdc4117f797040c1a9e32367af2a8d97554e442c7bfd002297a7ff
```

Those tests compare the **full 64-hex stored digest** against the corpus, so the
storage layer has a real oracle. Spec line: §4.5. Same reasoning as M-1 for the
severity. If one line were added to the walk it should be this: read the path
row's own digest rather than the word `hash`.

### M-3 — the implementation report says the walk is "297 lines"; it is 331

`design/agent-reports/hashlock-H2-implementation-report.md:150` —
"**Fork commit `e1bf137`** — `cmd/emu/walk_hashlock_phrase.js` (297 lines)."

```
$ git show e1bf137:cmd/emu/walk_hashlock_phrase.js | wc -l
331
$ git show 17b3979:cmd/emu/walk_hashlock_phrase.js | wc -l
331
$ git log --oneline c4a64fc..17b3979 -- cmd/emu/walk_hashlock_phrase.js
e1bf137 emu: hashlock phrase walk -- both methods, the mixed-case row, a negative control
```

The file was touched by exactly one commit and has been 331 lines since it
existed. 297 also matches neither the non-blank count (308) nor the
non-comment count (234), so it is not a differently-defined measure — it is a
stale number, presumably from a draft. Every *load-bearing* claim in that same
section reproduced exactly (the eight `run()` fields, byte for byte), so this is
a records defect and nothing more; the constellation rule it breaks is "never
hand-count what a tool can count."

### N-1 — the walk picks the phrase row by INDEX, in the one production place the code deliberately stopped doing that

`walk_hashlock_phrase.js:232` — `chooseRow(0, "32-byte value", "Type a hashlock
phrase")`. Row 0 is the phrase row *only* because `shSysw("none")` leaves the
payload empty (`composerHashRows`, `gui/composer_hash.go:158-174`), and the
`expect` string does not disambiguate: `composerCopyHashRule()` is shown for
payload rows, the phrase row **and** the hex row alike
(`gui/composer_hash.go:212-214`). This is the shape `composerHashRowSet` exists
to avoid — its own comment cites "r2 review C-4: the shipped default arm cleared
the lock when a row moved."

It degrades safely rather than silently: a moved row lands on a payload digest
or the hex keypad, and the very next `waitFor("Hashlock phrase")` (L234) times
out, so the walk fails loudly. And the configuration is pinned *and asserted*
(`must(which, "No hash record in the payload", …)`, L285). A Nit, not a defect:
worth a label-keyed pick if the walk is ever re-pointed at a payload.

### N-2 — `out.ok` is a redundant restatement of assertions that already threw

L326-329 recomputes four conditions that `must`/`mustNot` have already enforced,
against strings truncated to 220 characters. It cannot currently be `false`
without an earlier throw, and the truncation is safe only because the digest is
the modal's first line. Harmless; noted because a future reader may take `ok`
for an independent check rather than an echo.

---

## Housekeeping

- Servers: ports 8811-8815 stopped by PID (`kill 178319 184079 185804 196135
  207607`); `ss -ltn | grep ':88[0-9][0-9]'` → no listeners remain. No
  `pkill -f` was used.
- Worktree `/scratch/code/shibboleth/.tmp/h2-wf-lens-walk-control`: all four
  mutations reverted (`git status --short` empty at `17b3979`), then removed with
  `git worktree remove --force`.
- No phrase or 32-byte preimage was written to any retained log; the phrases
  used are the corpus's own published rows.

## Counts

**0 Critical / 0 Important / 3 Minor / 2 Nit.**

The lens closes: the walk fails when the device is wrong about the thing it
tests — the digest on the confirm modal, for both methods, against a pinned host
corpus — and one-iteration and one-nibble deviations are both caught. It does
not test the storage layer or the state machine, and it does not claim to; both
are gated by `gui` tests I executed under mutation.
