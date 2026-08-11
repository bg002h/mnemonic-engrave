# B2c program-boundary verification — F-88, F-90, F-104

**Question.** For each of F-88, F-90, F-104: is the subject reachable from the
Sealed Payload program (rooted at `unlockPayload` / `gui/unlock_flow.go`,
`gui/unlock_session.go`, `gui/unlock_kdf.go`, `gui/unlock_platelist.go`,
`gui/unlock_plates.go`; wipeGuard brackets at `gui/unlock_session.go:88-89` and
`gui/unlock_kdf.go:136`), or only from other programs? Per SPEC §2.2 item 12
(operator ruling 2026-08-10): reachable from payload → still binds; not
reachable → closes as ACCEPTED.

**Method.** Traced call graphs in the code (not follow-up-entry text) with
`grep`/`Read` over the whole tree at `/scratch/code/shibboleth/seedhammer-b2b`,
branch `b2b`, excluding only `third_party/` and `.git/`. Every claimed call
site below was opened and read; line numbers are current-tree, verified
2026-08-10.

---

## F-88 — three seed-equivalent copies on the mnemonic engrave path

**VERDICT: REACHABLE FROM PAYLOAD** (all three items)

subject: (1) `sentence []byte` inside `bip39.MnemonicSeed` (`bip39/bip39.go:218-224`);
(2) the `[]byte` behind `string(seedqr.QR(m))` and `qr.Code.Bitmap` in
`engraveSeed` (`gui/gui.go:539-553`); (3) `engraveSeed`'s `words []string`
(same function).

call graph:
```
gui/unlock_session.go:270  unlockEngraveMnemonic(ctx, th, rec)      <- called from
                            unlockSecretPlate (unlock_session.go:166,
                            switch case seal.ClassMnemonic), itself called
                            from unlockSecretSession (unlock_session.go:88-89,
                            INSIDE the wipeGuard bracket)
  -> unlock_session.go:291-292  ss := &SeedScreen{NoEdit:true}; ss.Confirm(ctx, th, m)
       -> gui/gui.go:2380 (SeedScreen).Confirm
            -> gui/gui.go:2426  deriveMasterKey(mnemonic, &chaincfg.MainNetParams, "")
                 -> gui/gui.go:246  seed := bip39.MnemonicSeed(m, password)   [item 1]
  -> unlock_session.go:299  masterFingerprintFor(m, &chaincfg.MainNetParams, "")
       -> gui/gui.go:563-564  deriveMasterKey(m, network, password)
            -> gui/gui.go:246  bip39.MnemonicSeed(m, password)               [item 1]
  -> unlock_session.go:305  plate, err := engraveSeed(params, m, mfp)
       -> gui/gui.go:540  qrc, err := qr.Encode(string(seedqr.QR(m)), qr.M)  [item 2]
       -> gui/gui.go:542-545  words := make([]string, len(m)); ...           [item 3]
```

verdict basis: `unlockEngraveMnemonic` is dispatched only from
`unlockSecretPlate`'s `case seal.ClassMnemonic:` (`unlock_session.go:166`),
which runs inside `unlockSecretSession`'s wipeGuard bracket
(`unlock_session.go:88-89`, `g := &wipeGuard{}; ctx.wipe = g`). It directly
calls `engraveSeed` (item 2, 3) and reaches `bip39.MnemonicSeed` (item 1)
through two independent paths — `SeedScreen.Confirm`'s validity probe and its
own `masterFingerprintFor` call. All three copies are therefore created while
the payload program's own secret session is live, not merely reachable in
principle. (The entry's "two of them unreachable from `gui`" describes
*package*-boundary reachability — whether `gui` code can scrub a `bip39`-local
— not program-boundary reachability; it does not change this verdict.
`MnemonicSeed`/`deriveMasterKey`/`engraveSeed` are also called from other
programs — `gui.go:2197`'s `inputWordsFlow`→`SeedScreen.Confirm` on manual
entry, `bip85.go`, `derive.go`, `preview.go` — but that is additional exposure
on top of the payload-reachable path, not a substitute for it.)

**Does not close.** All three items still bind; no closure line proposed.

---

## F-90 — the `ms1` (codex32) engrave arm, items 1 and 3

**VERDICT: REACHABLE FROM PAYLOAD**

subject: item 1, an F-88-equivalent inventory for `unlockEngraveCodex32`'s six
copies (`string(rec)`; `codex32.String`'s retained data; `id` from `Split()`;
`s.String()`; `plan`; `plate.Spline`); item 3, an `unlockCodex32Hook` mirroring
`unlockMnemonicHook`.

call graph:
```
gui/unlock_session.go:186  func unlockEngraveCodex32(ctx *Context, th *Colors, rec []byte)
  called ONLY from unlock_session.go:166 (unlockSecretPlate's
  case seal.ClassCodex32Secret:), itself called from unlockSecretSession
  (unlock_session.go:88-89, INSIDE the wipeGuard bracket) -- confirmed by
  whole-tree grep: unlockEngraveCodex32( has exactly one call site.

  unlock_session.go:187  s, err := codex32.New(string(rec))          [string(rec), codex32.String]
  unlock_session.go:193  id, _, _ := s.Split()                       [id]
  unlock_session.go:195-199  backup.EngraveSeedString(params,
                              backup.SeedString{Title: id, Seed: s.String(), ...}) [s.String()]
  unlock_session.go:205  plate, err := toPlate(plan, params)         [plate -> plate.Spline]
```

verdict basis: unlike F-88's subjects, `unlockEngraveCodex32` has **no other
caller anywhere in the tree** (grep confirmed) — every one of item 1's six
copies, and the function item 3 wants a hook in, exists exclusively inside the
payload program's own bracketed secret session. This is not a mixed case.

**Does not close.** Both items still bind; no closure line proposed.

---

## F-104 — four more residues, two unenumerated

**VERDICT: REACHABLE FROM PAYLOAD** (all four items)

### Item 1 — `x/crypto/pbkdf2`'s HMAC state (plaintext mnemonic, XOR-recoverable)

call graph: same as F-88 item 1 above — `pbkdf2.Key(sentence, ...)` is
`bip39/bip39.go:225`, the last line of `MnemonicSeed`, reached from
`unlockEngraveMnemonic` via `masterFingerprintFor`/`SeedScreen.Confirm` inside
the `unlock_session.go:88-89` bracket.

verdict basis: identical chain to F-88 item 1; the pbkdf2 internals are one
frame deeper than `sentence []byte` but on the same call path.

### Item 2 — `splitMnemonic`'s `math/big`/`entBytes` residue (classifier, every unlock; ~2,048x over the passphrase prefix)

call graph (three independent payload paths):
```
(a) classifier, on the encrypted section, every unlock:
gui/unlock_kdf.go:374     return o.UnlockWithKey(blob, p, key)     <- inside
                           unlockAttemptOnce (unlock_kdf.go:354), called from
                           unlockSealedFlow (unlock_kdf.go:412), called from
                           unlockPayloadFlow's KDF path (unlock_flow.go)
  -> seal/unlock_key.go:102  admitted, err := AdmitSection(recs, SectionEncrypted)
       -> seal/record.go:219  c := Classify(r)
            -> seal/record.go:158  bip39.Parse(b)
                 -> bip39/bip39.go:302-303  if !m.Valid() { ... }
                      -> bip39/bip39.go:112  ent, _ := splitMnemonic(m)

(b) classifier, on the public section, every unlock:
gui/unlock_flow.go:64      p, err := o.Inspect(blob)     <- unlockPayloadFlow, root
  -> seal/open.go:149        admitted, err := AdmitSection(recs, SectionPublic)
       -> seal/record.go:219/158  Classify -> bip39.Parse -> m.Valid() -> splitMnemonic

(c) ~2,048x over the passphrase prefix during last-word entry:
gui/unlock_kdf.go:109      func unlockPassphraseFlow(ctx, th)     <- bracketed at
                           unlock_kdf.go:136
  -> unlock_kdf.go:161      inputWordsFlow(ctx, th, m, 0, "")
       -> gui/gui.go:704     cands = bip39.LastWordCandidates(mnemonic)
            -> bip39/bip39.go:135-151  loops all NumWords=2048 words, calling
                                       m.Valid() -> splitMnemonic each time
```

verdict basis: the entry's own two clauses map onto two distinct payload call
sites — "created by the classifier, on every unlock" is (a)/(b) inside
`UnlockWithKey`/`Inspect`, both roots of the payload program itself; "roughly
2,048x over the passphrase prefix during last-word entry" is (c), and
`unlockPassphraseFlow` is the named, bracketed payload function
(`gui/unlock_kdf.go:109-136`) that reuses `inputWordsFlow` unmodified (per its
own doc comment) for exactly the passphrase keyboard. `inputWordsFlow` is also
shared with manual seed entry, but the payload-owned call sites above are
sufficient on their own.

### Item 3 — the `ms1` arm's per-column `ToUpper` copies and its QR (missing from F-90's enumeration)

call graph:
```
gui/unlock_session.go:196  plan, err := backup.EngraveSeedString(params,
                            backup.SeedString{..., Seed: s.String(), ...})
                            <- inside unlockEngraveCodex32 (sole caller: see F-90)
  -> backup/backup.go:125-137  func EngraveSeedString(params, plate)
       backup.go:126   seed := strings.ToUpper(plate.Seed)        [ToUpper copy #1]
       backup.go:127   qrc, err := qr.Encode(seed, qr.M)          [the QR]
       backup.go:137   return engraveSeedString(params, plate, qrCmd)
            -> backup.go:163  seed := strings.ToUpper(plate.Seed) [ToUpper copy #2,
                                                                    sliced into
                                                                    column ranges
                                                                    by stringColumn]
```

verdict basis: `backup.EngraveSeedString` is called from exactly two sites in
the tree — `unlock_session.go:196` (payload, inside `unlockEngraveCodex32`,
itself inside the `unlock_session.go:88-89` bracket) and `gui.go:2243`
(`backupSeedStringFlow`, a different, non-payload program). Both `ToUpper`
copies and the QR are produced on the payload call path, matching F-90's own
"six of seven [canonical vectors]" framing — this residue was missing from
F-90's inventory of that same function, not from a different one.

### Item 4 — keyboard fragment strings

call graph:
```
gui/unlock_kdf.go:109  unlockPassphraseFlow  (bracketed, unlock_kdf.go:136)
  -> unlock_kdf.go:161  inputWordsFlow(ctx, th, m, 0, "")
       -> gui/gui.go:1013  kbd := NewKeyboard(ctx, wordKeys)   (type Keyboard, gui.go:992)
       -> gui/gui.go: kbd.Update(ctx) loop
            -> gui/gui.go:1305  k.Fragment = k.Fragment + string(unicode.ToUpper(r))
```

verdict basis: `Keyboard.Fragment` (`gui/gui.go:992`) is grown by
concatenation (`gui.go:1305`), which orphans every prior partial copy on the
heap exactly as the entry describes; `inputWordsFlow`'s keyboard is the one
`unlockPassphraseFlow` reuses unmodified for the 12-word passphrase, inside its
own bracket. (There is a second, distinct `Fragment` field on
`PassphraseKeyboard`, `gui/passphrase_keyboard.go:51`, used by
`gui/passphrase_flow.go`/`freetext_flow.go` — a different, non-payload program
— but that is a separate instance of the same class of bug, not the one this
verdict rests on.)

**Condition on B2c, from the audit entry:** land F-94's seam first — unaffected
by this verification, still applies since F-88 also survives.

**None of F-104's four items close.** All four bind.

---

## Summary

| item | verdict |
| --- | --- |
| F-88 (all 3) | REACHABLE FROM PAYLOAD |
| F-90 (items 1, 3) | REACHABLE FROM PAYLOAD |
| F-104 (all 4) | REACHABLE FROM PAYLOAD |

No item closes under SPEC §2.2 item 12. Every subject named in these three
entries is created during a call reachable from `unlockSecretSession` /
`unlockPassphraseFlow` — i.e. from inside one of the payload program's own two
wipeGuard brackets — so the operator's "reachable from payload still binds"
rule keeps all of them open. Several subjects are *also* reachable from other
programs (`SeedScreen.Confirm` via manual entry, `EngraveSeedString` via
`backupSeedStringFlow`, `inputWordsFlow`'s keyboard via other flows), but that
additional exposure is irrelevant to the verdict once the payload-reachable
path is confirmed — it does not soften or remove the binding.
