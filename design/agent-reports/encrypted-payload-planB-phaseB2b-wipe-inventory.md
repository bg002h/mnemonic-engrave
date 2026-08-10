# B2b wipe inventory — what §10.2.4's wipe actually zeroes, and what survives it

- **Auditor:** independent inventory agent (Fable 5), 2026-08-09
- **Repo:** `/scratch/code/shibboleth/seedhammer-b2b`, branch `b2b` at `484ceb9` (clean; only the test file below added, uncommitted)
- **Question:** after the §10.2.4 idle wipe completes and the UI has restarted, what seed-equivalent material is still resident in RAM — and does every buffer the code claims to zero actually get zeroed?
- **Method:** measurement first. A new Run-level test drives the **real `uiFlow`** through the **real timer wipe** (3:00 idle + 30 s warning, both observed as script steps) on vectors A and F, holding the live backing arrays via the existing hooks, and asserts zero **on the arrays** after the session restarts. Four mutations were applied to non-test code (each reverted; `git status` clean) to prove the assertions can fail. Everything not machine-measurable is established by code reading with line cites and marked as such.

## Verdict in one paragraph

**Fidelity: every wipe the code performs zeroes what it claims, measured end-to-end at Run level — no second `Reset()`-class bug was found.** The `op.Buffer.Scrub()` fix is real and load-bearing: deleting it makes the new test fail with **2,031 non-zero args** recoverable from the abandoned frame buffer after a wipe fired over a rendered SeedScreen. The wipe-as-unwind design (F-89) is also real and now pinned: deleting `unlockEngraveMnemonic`'s `defer clear(m)` makes the new test fail — a live `[]Word` copy of the seed surviving the wipe — where before B2b's own tests it survived deletion silently. **Coverage: everything reachable is wiped.** What survives the wipe is exclusively (a) **unreachable heap garbage** that nothing can zero — Go strings, `math/big` internals, HMAC/AES states, QR bitmaps, and the two engrave closures — which is the class F-83/F-88 already accept, though this audit found **four unrecorded members** of it; and (b) one **walk-away state the timer never covers by design**: a typed passphrase parked on the entry keyboard *before* any unlock, which no bracket arms and nothing ever wipes.

---

## 1. Inventory table

"Verified" means a test exists that fails when the wipe is removed. RL = the new Run-level test (`TestWipeZeroesEveryPinnedBufferAtRunLevel`, real §10.2.4 trigger); FL = flow-level (bypasses `Run`). All paths are in the b2b repo unless noted.

### 1a. Seal-owned buffers (the wipe's primary subject)

| What holds seed material | Where | Who wipes it | Verified? |
|---|---|---|---|
| `p.Secret[i].Record` — every decrypted secret record | `seal/open.go:31-46` | per-record: `defer p.WipeSecretAt(i)` (`gui/unlock_session.go:112-120`); early: `clear(rec)` at plate build (`:198` ms1, `:304` mnemonic); backstop: `defer p.Wipe()` (`gui/unlock_flow.go:85`) | **YES.** RL both vectors (mutation: deleting `WipeSecretAt` *and* `p.Wipe` fails both subtests; deleting `WipeSecretAt` alone survives RL because the backstop covers the unwind — the per-record wipe's own routes are pinned FL: `unlock_session_test.go:273,324,344,374,409` |
| `plaintext` — gcm.Open's whole decrypted container (every secret in one array) | `seal/unlock_key.go:81-91` | `defer clear(plaintext)` (`:88`) | **YES** (FL, `seal/unlock_key_test.go:245-282`, on the buffer via `unlockPlaintextHook`). Runs and completes at unlock time, long before any §10.2.4 window opens |
| stale `p.Secret` from a **previous** unlock, about to be overwritten | `seal/unlock_key.go:109-112` | explicit `clear` loop before `p.Secret = admitted` | YES (FL, `seal/unlock_key_test.go:307-`) |
| partial `out` on an admission failure | `seal/record.go:465-469` | `wipe(out)` | function behaviour tested; **call sites not regression-testable** (stated in the code itself, `record.go:458-463`) |
| `blob` — region bytes (header ‖ public ‖ ciphertext; **not** plaintext seed) | `gui/unlock_flow.go:38-58` | `clear(blob)` + nil before the session (`:110-111`); deferred closure for other exits (`:58`) | FL (F-79 tests). Not seed-equivalent; listed for completeness |

### 1b. gui-held copies on the unlock path

| What | Where | Who wipes it | Verified? |
|---|---|---|---|
| typed passphrase `m` (`[]Word`, one per attempt) | `gui/unlock_kdf.go:124-145,377-384` | `clear(m)` after each attempt (`:384`) and on the partial/back exit (`:134,141`) | **YES.** RL (both vectors, on the buffer via `unlockPassphraseWordsHook`) + FL `unlock_wipe_test.go:163,192` |
| §8.1 normalised passphrase bytes `pass` | `gui/unlock_kdf.go:330-331` | `defer clear(pass)`; fixed cap 128 so append never orphans | FL `unlock_wipe_test.go:50` (on the buffer) |
| derived 32-byte key | `gui/unlock_kdf.go:332-337` | `defer clear(key)` | **YES.** RL (via `unlockKeyHook`) + FL `unlock_wipe_test.go:90` |
| `Deriver.u`, `Deriver.acc` (key-equivalent accumulators) | `seal/pbkdf2.go:37-53` | `defer d.Wipe()` (`gui/unlock_kdf.go:214`); Wipe is one-way (`dead`), post-Wipe `Key()` = nil | FL `unlock_wipe_test.go:132` + seal's own Deriver tests |
| `bip39.Parse`'s `[]Word` copy `m` in `unlockEngraveMnemonic` | `gui/unlock_session.go:250-315` | `clear(m)` at plate build (`:305`); `defer clear(m)` for the three early returns **and the wipe-unwind** (`:258`) | **YES — newly pinned on the wipe path.** RL vector A parks on `SeedScreen.Confirm` with `m` live; mutation "delete the defer" fails the test. Early returns: FL `unlock_session_test.go:795,845` (F-87 substantially closed) |
| the 64-byte BIP-39 seed | `gui/gui.go:245-259` (`deriveMasterKey`), `gui/derive.go:20-21` | `defer wipeBytes(seed)` | **NO — wipe exists, unpinned (F-94).** Deletable with the suite green; not reached by RL (the park is before Confirm) |
| the BIP-32 master private key | `gui/gui.go:563-580` (`masterFingerprintFor`), SeedScreen probe `gui/gui.go:2423-2436` | `defer mk.Zero()` / explicit `mk.Zero()` | **NO — unpinned (F-94)**, same as above |
| the abandoned session's frame buffer `ctx.B` (rendered seed words as `op.Glyph` args) | `gui/gui.go:69`, scrubbed at `gui/run_flow.go:245` | `ctx.B.Scrub()` — zeroes args+refs **to capacity** (`gui/op/buffer_len.go:23-28`); `Buffer` has exactly those two fields, so Scrub covers the whole struct | **YES.** RL: real SeedScreen rendered, real wipe, `Residue()==(0,0)`; mutation "delete Scrub" fails with 2,031 non-zero args. Synthetic-flow version: `run_flow_scrub_test.go` |
| Run's warning buffer `a.warnBuf` | `gui/run_flow.go:29` | never scrubbed — only ever holds `wipeWarningOp` output (verified: `wipe_warning.go:43-61` is its sole writer) | content is the public warning text; nothing to wipe |

### 1c. Unwipeable, unreachable heap garbage — survives the wipe until GC reuse / power-off (the F-83/F-88 acceptance class)

Nothing in this section is reachable after the wipe; all of it is recoverable from SRAM by the §2.2-item-9 attacker (SWD probe, physical access) until overwritten. Items marked **NEW** appear in no follow-up today.

| What | Where it is made | Recorded? |
|---|---|---|
| `plate.Spline` — a **closure over the plaintext**, re-reads it per knot | `engrave/engrave.go:1016-1026` capturing `frontSideSeed`/`engraveSeedString` closures (`backup/backup.go:214,148`) | F-83, accepted; corrected mechanism 2026-08-09 |
| `sentence []byte` — the plaintext mnemonic inside `MnemonicSeed`, **plus** its append-growth orphans (grows from nil: 6-7 orphaned partials) | `bip39/bip39.go:217-226` | F-88 row 1. Made **twice per mnemonic engrave** (SeedScreen probe + `masterFingerprintFor`) |
| **NEW:** the HMAC-SHA512 ipad/opad inside `x/crypto/pbkdf2.Key(sentence, …)` — sentence ≤107 bytes < the 128-byte SHA-512 block, so the **plaintext mnemonic is recoverable by XOR** from that allocation | `bip39/bip39.go:225` → `golang.org/x/crypto/pbkdf2` | **unrecorded** — belongs on F-88 row 1 (same fix site, `MnemonicSeed`) |
| **NEW:** `math/big` residue from `splitMnemonic` — the **full seed entropy** as big.Int nat arrays plus `entBytes []byte` (`ent.Bytes()` + padding append), never zeroed | `bip39/bip39.go:177-197`, reached via `Valid()` (`:107-115`) | **unrecorded.** Reached with real seed entropy from: `seal.Classify` → `Parse` → `Valid` on **every** bare-mnemonic record at unlock (`seal/record.go:158`; the `clear(m)` there zeroes the `[]Word`, not this); `unlockEngraveMnemonic` → `Parse`; `seedqr.QR(m)` → `Valid`; `SeedScreen.Confirm` → `Valid`. Reached with **passphrase** content from `unlockPassphraseFlow`/`unlockAttemptOnce` `Valid()` and — heavily — `LastWordCandidates` (`bip39/bip39.go:135-154`), which runs `Valid()` up to 2,048 times over the real 11-word prefix during last-word entry. `entBytes` is a plain `[]byte` a one-line `clear` in `bip39` could fix; the big.Int internals cannot be fixed at all |
| the SeedQR: `seedqr.QR(m)` digit string + `bytes.Buffer`, `qr.Code.Bitmap`, and the `ConstantQRCmd`'s `modules` motion list (content-encoding) | `seedqr/seedqr.go:24-33`, `gui/gui.go:540`, `engrave/engrave.go:418-` | F-88 row 2 (bitmap); the digit-`bytes.Buffer` and `modules` are the same class |
| `engraveSeed`'s `words []string` — selection+order is the seed; **`clear(words)` is destructive** (captured by `frontSideSeed`, read during the cut) | `gui/gui.go:544-547` | F-88 row 3, remedy retracted — **not re-proposed here** |
| ms1 arm: `string(rec)` and its aliases (`codex32.String.s`, `id`, `s.String()`, `SeedString.Seed` all share that one allocation — verified `codex32/codex32.go:16-18,98-124` wraps without copying) | `gui/unlock_session.go:166-178` | F-90 item 1 |
| **NEW:** ms1 arm `strings.ToUpper(plate.Seed)` — one fresh uppercased copy of the share in `EngraveSeedString` (`backup/backup.go:126`) **plus one more per ranging of the spline closure** (`backup/backup.go:163` is *inside* the returned func; the curve is ranged at least twice — `bspline.Measure` in `toPlate` and the cut itself) | `backup/backup.go:126,163` | **unrecorded** — F-90 item 1's six-copy enumeration missed it |
| **NEW:** ms1 arm QR — `qr.Encode(ToUpper(share))` bitmap + `ConstantQR`'s derived bitmap and `modules` | `backup/backup.go:127-137`, `engrave/engrave.go:418-` | **unrecorded** — F-90 item 1 has no QR row (the mnemonic arm's F-88 does) |
| `Classify`'s `s := string(b)` for every ms1 record of the encrypted section, at unlock time | `seal/record.go:171` | recorded in `Classify`'s own comment (charged to F-88) |
| **NEW:** keyboard residue during passphrase entry — `Keyboard.Fragment` (`gui/gui.go:993`) and `wordLabel` are Go strings rebuilt per keystroke; the orphaned per-letter concatenations spell out each typed word prefix in typing order. `Parse` similarly orphans a `bytes.ToUpper` copy of every word of every record it accepts (`bip39/bip39.go:294`) | `gui/gui.go:671-744` | **unrecorded** |
| KDF internals: `Deriver.mac`'s ipad/opad — passphrase-recoverable-by-XOR between `NewDeriver` and the first `Step`, key-equivalent (FIPS 198-1 §6) for the Deriver's whole life; `Wipe` cannot reach them | `seal/pbkdf2.go:60-84` (precisely documented) | recorded in the code; unfixable short of hand-rolled HMAC |
| AES-256 round keys + GCM state derived from the payload key in `seal.Open` | `seal/crypto.go:84-92` | same class; **unrecorded but key-, not seed-, equivalent and gone from reach when `Open` returns** |

### 1d. Examined and found not to matter

- **Run's persistent `op.Drawer` `d`** (`gui/run_flow.go:47`): truncation-only resets, but its working stacks are restored per-op (`op/op.go:369`) so backing capacity is a handful of slots, and the ~30 warning frames redrawn through `d.Draw` before the wipe overwrite them with warning glyphs. Worst case: a few stale interface refs pinning single glyphs. Closure-local, not measurable from a test; established by reading `op/op.go:245-372`.
- **`EventRouter.events` / Run's `evts`** (`gui/event.go:281-331`): truncation-reused; backing arrays hold at most the final tick's tap coordinates (one nav tap by wipe time), not the typing sequence.
- **`labelEncryptedCards`** (`seal/label_encrypted.go:28-41`): stringifies the ClassMDMK subset only; ms1/mnemonic records are never converted. Verified.
- **`unlockPlates`** (`gui/unlock_plates.go:67-91`): includes only `ClassMDMK` from the encrypted section; secrets absent by construction. Record slices alias `p.Secret`, so `p.Wipe` reaches them.
- **The display framebuffer**: the parked screen's pixels (possibly the seed words) persist on the panel until the warning frame repaints it at 3:00 — which is also the §10.2.4 privacy blanking working as designed.
- **`warnBuf` growth**: bounded per frame by `Reset()`; content is public warning text (sole writer verified).

## 2. Findings, ranked

**No Critical.** Nothing reachable survives the wipe; no wipe was found to zero less than it claims.

1. **IMPORTANT (design-boundary coverage): a typed passphrase parked on the entry keyboard is never wiped by anything.** §10.2.4's timer keys on the secret-session bracket, which opens only *after* a successful unlock. An operator who types 11-12 words and walks away leaves `m` (the typed `[]Word`), `Keyboard.Fragment`, and the candidate machinery live behind the screensaver **indefinitely** — and the blob is in flash beside them, so the passphrase is seed-equivalent by composition. The partial-exit `clear(m)` (`unlock_kdf.go:134`) runs only when the operator taps Back; a parked flow never returns. This is not a defect in the wipe — the spec's bracket was chosen deliberately — but the walk-away-state reasoning that justifies arming the hold-to-start screen ("walk-away states with secrets still held", `wipe_guard.go:37-40`) applies verbatim here and the state is uncovered. Belongs with B2c or a spec amendment; one candidate shape is arming a bracket at `unlockSealedFlow` entry.
2. **IMPORTANT (inventory debt): four unrecorded members of the unwipeable class**, §1c NEW rows: (a) the `x/crypto/pbkdf2` HMAC state holding the plaintext mnemonic XOR-recoverable; (b) `splitMnemonic`'s big.Int/`entBytes` residue — notable because it is created by the **classifier itself** on every unlock (before any operator choice) and 2,048× over the passphrase prefix during last-word entry, and because `entBytes` alone is a zeroable `[]byte` fixable with one `clear` in `bip39`; (c) the ms1 arm's per-ranging `ToUpper` copies and QR; (d) keyboard fragment strings. None changes the accepted risk posture (same attacker, same window as F-83); all four belong in B2c's inventory so it does not repeat F-88's "complete inventory that wasn't".
3. **MINOR (was F-87, now largely closed): the wipe-unwind path of `defer clear(m)` was unpinned; it is now pinned by the new Run-level test** — mutation-verified (deleting the defer fails `vectorA-parked-on-seed-screen` with "`[]Word copy SURVIVED the wipe`"). The two showError early returns were already pinned FL (`unlock_session_test.go:795,845`); the Confirm-cancel return remains covered only indirectly.
4. **MINOR (observability, expected): at Run level, the per-record `WipeSecretAt` and the `p.Wipe` backstop are indistinguishable** — deleting `WipeSecretAt` alone leaves the new test green because the unwind's backstop also zeroes the records. The per-record wipe's distinct value (Skip/Back/cancelled-engrave without a session exit) is pinned at flow level only. Acceptable; recorded so nobody reads the RL test as pinning the finer-grained wipe.
5. **NOTE: vector F's 12 encrypted md1/mk1 card records** are wiped by `p.Wipe` on the unwind but are observable at Run level by no seam (`unlockEngraveHook` fires only on plate-list selection). §6.3 classes them privacy-, not seed-, relevant; flow-level tests cover `p.Wipe`. No action proposed.

## 3. What could not be measured, and why

- **Unexported third-party internals**: `crypto/hmac` ipad/opad (both in `seal.Deriver` and inside `x/crypto/pbkdf2`), AES round keys, GCM state, `math/big` nat arrays. Measuring requires `unsafe`/reflection into stdlib internals; established instead by reading the cited sources (the `pbkdf2.go:60-84` caveat cites go1.26.3's fips140 hmac and matches it).
- **Dropped Go strings** (codex32 aliases, `ToUpper` copies, keyboard fragments, `Classify`'s `s`): unreachable the moment they are dropped; no test can observe them without heap scanning. Established by code reading with line cites (§1c).
- **Run's closure-locals** (`d`, `a.warnBuf` residue beyond `warnBufHook`'s lengths): no seam reaches them; argued from `op/op.go` code structure only.
- **TinyGo fidelity**: all measurements ran host Go under `synctest`. TinyGo's conservative GC may retain or copy any of §1c longer than host Go would; this is the firmware's standing caveat and this audit inherits it.
- **The post-wipe hang** and §10.2.4 timing: out of scope per the brief; the new test's wipes completed and restarted sessions normally on host.

## 4. Is the F-88 / F-90(1,3) / F-94 deferral to B2c safe?

**Yes, with two conditions.** The measured result underwrites the deferral: after the wipe, everything those follow-ups cover is *unreachable garbage*, exposed only to the SWD-probe attacker who — per F-83, accepted — already gets the whole seed from the spline closure during any cut. Deferring them widens no window that F-83 does not hold open, and nothing in them is reachable through a live reference after the wipe (measured: records, parsed words, passphrase buffers, key, and frame buffer all zero; nothing else in `gui` retains a handle).

Conditions:

1. **F-94's wipes remain silently deletable until B2c.** The seed/master-key scrubs exist but no test fails when they are removed (their functions are not reached by the new Run-level parks). The deferral is a bet that no refactor touches `deriveMasterKey`/`masterFingerprintFor` before B2c. That bet is reasonable (shared funds-path code, rarely edited) but should be named in B2c's entry criteria: land the `deriveSeedHook` seam **first** in that phase.
2. **B2c's scope must absorb §1c's NEW rows** — especially `entBytes` (the one zeroable `[]byte` in the big.Int class, a one-line `bip39` fix that wants the same own-review as `MnemonicSeed`'s scrub) and the pbkdf2-HMAC note on F-88 row 1 — or B2c's inventory will claim completeness while incomplete, which is the exact failure F-88's history warns about. Finding 1 (the un-bracketed passphrase-entry park) should be triaged there too, as it is the only item in this report that is *coverage*, not garbage-hygiene.

F-90 item 2 is confirmed dissolved: `RecordsResident` is renamed, its contract states the narrow reading (`seal/session.go:20-50`), and the timer keys on the bracket (`gui/wipe_guard.go`), not the predicate — verified in code, and the bracket behaviour is what the new test exercised.

## 5. Evidence artifacts

**Test file left behind (uncommitted, test-only):**
- `/scratch/code/shibboleth/seedhammer-b2b/gui/wipe_inventory_audit_test.go` — `TestWipeZeroesEveryPinnedBufferAtRunLevel`, two subtests: `vectorA-parked-on-seed-screen` (mnemonic arm, seed rendered, `m` live at wipe) and `vectorF-parked-on-ms1-cutskip` (default arm, record live at wipe). Both PASS at `484ceb9`; full `./gui ./seal ./bip39` suites remain green with it present.

**Mutation evidence (all reverted; working tree clean):**

| Mutant | Result |
|---|---|
| delete `ctx.B.Scrub()` (`run_flow.go:245`) | FAIL — "2031 non-zero args … recoverable from the backing array" |
| delete `defer clear(m)` (`unlock_session.go:258`) | FAIL — "[]Word copy SURVIVED the wipe (F-89's exact shape)" |
| delete `p.WipeSecretAt(i)` alone | pass (backstop covers the unwind — finding 4) |
| delete `p.WipeSecretAt(i)` **and** `defer p.Wipe()` | FAIL — both vectors, every record non-zero |
