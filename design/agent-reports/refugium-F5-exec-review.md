# Refugium F5: independent adversarial execution review

- **Reviewer:** independent execution-review subagent (Opus 5.5)
- **Date:** 2026-10-05
- **Diff:** `be00ef8..54e7276` on `claude/project-thread-59q2a6` (3 commits, 20 files, +1629/-40), reviewed as a whole
- **Plan:** `design/IMPLEMENTATION_PLAN_fork_refugium_F5_F7.md` draft 4 (GREEN), §2 D2 and §3, with the R0 folds it cites (R0 M-7/M-8/M-9/I-8b; round 2 M-5/M-6)
- **Counts:** **0 Critical / 1 Important / 4 Minor / 4 Nit**

## What I ran (all on copies; the worktree under review is unmodified, `git status` clean)

| Check | Result |
|---|---|
| `GOTOOLCHAIN=go1.26.7 go test ./backup/ ./seedqr/ ./codex32/ ./bip32/ ./bip39/ ./seal/ ./cmd/plateview/` | all `ok` |
| Targeted gui: `go test ./gui/ -run 'Preview\|MasterKey\|MasterFingerprint\|DeriveSeed\|Residue\|Fingerprint'` | `ok` (full sharded gui suite left to the controller, as instructed) |
| Re-fetched both vector sources at the pinned revs and recomputed sha256 | Trezor `vectors.json` = `fa3b937b…30f8`, SeedSigner README = `44fe1a32…1a3c` (also equals today's `dev` HEAD and the recon scratchpad copy). Both match the provenance. |
| Regenerated the vector file: `cargo run --locked -- vectors.json` in a fresh copy of `backup/testdata/gen`, clean target dir | **byte-identical** to the committed `ms1_seedqr_vectors.json` |
| `Cargo.lock`: ms-codec source | `git+https://github.com/bg002h/mnemonic-secret?rev=e5dff6f…` 0.10.0, equals mnemonic-engrave's `crates/me-cli/Cargo.toml:83` pin; no `~/.cargo/config` patch/override |
| SeedSigner rows vs spec text | all 10 digit streams and the 9 TV word lines occur verbatim in the spec; generator also asserts Rust-computed digits equal them |
| Existing goldens | `git diff --stat be00ef8..54e7276 -- backup/testdata/` shows only new files; `codex32-0/1` and every other `.bin` untouched |
| `gofmt -l .` | exactly the five-file baseline |
| `go.mod` | unchanged by the diff; `go 1.25.10` and `t.ArtifactDir` use (e.g. `bspline_test.go:157`, `engrave_test.go:177`) both predate it; CI pins `go-version: '1.26'`. **Pre-existing, not introduced.** |
| TinyGo 0.41.1 `./cmd/controller -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks -size short`, base vs head | base flash 1,672,860 / RAM 63,448; head flash 1,673,500 / RAM 63,448: **+640 B flash, 0 RAM**. `EngraveSeedStringSeedQR` and its four new error strings are NOT in the head ELF (dead-stripped). `go list -deps` (tinygo,rp2350,baremetal tags) identical, 253 packages, base and head. Growth is `codex32.EncodeMS1` 0x82→0x240 (+446, the defer and hook) and the bip32 move (gui.deriveMasterKey+masterFingerprintFor 1252 B → bip32.MasterKey+MasterFingerprint+gui shim 1248 B) plus inlining shifts. |
| Mutation probes on `EngraveSeedStringSeedQR` (copy) | killed: drop fingerprint check, exact-case compare, QR of the ms1 string, drop length switch, drop ms1 compare, `qr.L` instead of `qr.M`. **Survived: drop the word-range check (I-1).** |

## Important

### I-1. The word-range check is load-bearing for "ms1 and SeedQR describe the same seed", and its refusal test is vacuous: deleting the check leaves the suite green.

`backup/backup.go:226-230` refuses any `Word` outside `[0, NumWords)`. The test that claims to pin it, `word-out-of-range` (`backup/ms1_seedqr_test.go:280-281,294`), sets `outOfRange[3] = bip39.NumWords`, which happens to break the BIP-39 checksum, so `m.Valid()` (`:232`) refuses it whether or not the range check exists. Mutation: replacing the condition with `false && (...)` keeps `TestEngraveSeedStringSeedQRRefusals` and every other backup test green.

Why it matters (funds safety, demonstrated): `bip39.Mnemonic.Valid` does NOT range-check. `splitMnemonic` (`bip39/bip39.go:207-213`) builds the entropy with `ent = ent*2048 | w`, so a word `w + 2048` sets bit 11, i.e. bit 0 of the previous word. If the previous word is odd, that OR is a no-op, the entropy and checksum are unchanged, and `Valid()` returns true. With the range check deleted, I fed `seedsigner-tv1` with `m[1] += 2048` (`m[0]` = 115 is odd), fingerprint 0: the function **accepted** it and engraved the correct ms1 string of tv1 next to a SeedQR whose second index is `3373` (`0115 3373 1154 …` vs `0115 1325 1154 …`). That is exactly the "plate rows describe two different things" the function exists to refuse; a SeedQR reader either rejects the plate or, worse, a lenient one maps the index somewhere else. The check present at head stops it (I confirmed head refuses this input with `errSeedQRInvalidMnemonic`), so this is a test gap, not a live bug, but in the risk set an unpinned funds-safety guard is Important.

Fix: replace (or add beside) the `word-out-of-range` case with the aliased-but-Valid shape, and assert the control so it cannot go vacuous again:

```go
alias := append(bip39.Mnemonic(nil), m...)
i := 1
for m[i-1]%2 == 0 { i++ }          // previous word odd: the OR is a no-op
alias[i] += bip39.NumWords
if !alias.Valid() { t.Fatal("control: the aliased mnemonic must pass Valid, or only Valid is being tested") }
// expect errSeedQRInvalidMnemonic, with plate.MasterFingerprint = 0 so the
// fingerprint path (which would index the wordlist out of range) is not what refuses
```

Add a negative-word case (`alias[i] = -1`) too. Follow-up (outside F5, pre-existing): `bip39.Mnemonic.Valid` and `seedqr.QR`/`CompactQR` accept out-of-range words; consider making `Valid` range-check so every caller is protected, and file it in FOLLOWUPS.

## Minor

### M-1. The ms1 comparison is Unicode case-folding, so a non-ASCII `plate.Seed` passes the check and then panics while engraving.

`backup/backup.go:243` compares `ms1 != strings.ToLower(plate.Seed)`. `strings.ToLower` maps U+212A KELVIN SIGN to ASCII `k`, and `k` is in the codex32 alphabet. I set `plate.Seed` to tv-example's ms1 with its first `k` replaced by U+212A: `EngraveSeedStringSeedQR` returned **no error and an engraving**; iterating the engraving panicked `unsupported rune: K` in `engraveSeedString`'s stringer (`ToUpper(U+212A)` is U+212A). It never yields a plate of another seed, and no current caller can supply such a string (F5 wires no flow; F3 will build `plate.Seed` from `EncodeMS1`), so Minor; but the function's contract says it validates the string and it does not refuse here, and the panic is deferred to engrave time (`len(seed)` also counts bytes, so grouping shifts). Fix: compare byte-exactly against ASCII forms, e.g. `if plate.Seed != ms1 && plate.Seed != strings.ToUpper(ms1) { return nil, errSeedQRDisagree }` (`ms1` is ASCII, so `ToUpper` is exact), and add the U+212A case to the refusal table.

### M-2. The "wiped" inventory overstates the digit-stream wipe for 24 words.

The doc comment (`backup/backup.go:211-213`) lists "the []byte digit stream seedqr.QR returns" as wiped, and `:257` clears it. But `seedqr.QR` builds it in a `bytes.Buffer` (`seedqr/seedqr.go:28-32`), which starts at a 64-byte backing array and reallocates at 128 on the 17th word (measured on go1.26.7: realloc at word 0 cap 64, word 16 cap 128). The abandoned 64-byte array still holds the first 16 word indices of a 24-word seed and is never wiped; `fmt`'s pooled `pp` buffer also keeps the last `%04d`. Pre-existing in `seedqr.QR`, and the plan listed the digit string as live anyway, so this is about the comment's accuracy: move "the first 64 digits of a 24-word stream (bytes.Buffer growth) and fmt's buffer" into the live, not-wipeable list, or build the stream into a preallocated `make([]byte, 0, 4*len(m))` in `seedqr.QR` so there is one array to wipe.

### M-3. `EncodeMS1Preimage` keeps the unwiped payload that `EncodeMS1` just stopped keeping.

`codex32/msencode.go:61-70` builds `payload = [0x03]‖X` (a spend preimage) exactly as `EncodeMS1` did and does not wipe it. Out of F5's scope, but the new comment at `:26-29` makes the asymmetry look deliberate. File a one-line FOLLOWUPS item (same `defer clear(payload)` and hook pattern) rather than fold it into F5.

### M-4. The implementer's report and branch are not where the process expects them.

The report says the harness blocked `git -C /home/claude/seedhammer` and writes to the shared `design/agent-reports/`, so the branch lives in a clone and a bundle, and the report in an agent worktree. The commits reviewed here are the expected ones (`4af751a`, `77e698c`, `54e7276` on top of `be00ef8`). Before merge: copy the implementation report into `mnemonic-engrave/design/agent-reports/refugium-F5-impl.md` verbatim, and re-author/sign off (DCO, Brian Goss) if this goes upstream, as the report itself notes.

## Nit

- **N-1.** `seedsigner_rows.json:6` says the example's words are "copied verbatim"; in the spec they appear only as a numbered index table (`README.md` lines 36-47), not as one line. The digit stream is verbatim and the generator asserts the words reproduce it, so integrity holds; say "assembled from the spec's numbered table" for that row.
- **N-2.** `+640 B` flash for code no firmware path calls, mostly the `defer` and hook in `EncodeMS1` (TinyGo defers are not free). Acceptable; record the measured delta in the PR body since §3 did not ask for the ELF build and the implementer did not measure it.
- **N-3.** `backup/testdata/gen/Cargo.lock` in the fork will be picked up by GitHub dependency scanning (Dependabot alerts for a test-only generator). Harmless; consider a `.github/dependabot.yml` ignore or a note in the provenance.
- **N-4.** `bip32/master_test.go:52` comment says "pins MasterKey's `defer wipe(seed)`"; the code is `defer clear(seed)`.

## Checked and found sound

- **Same seed across rows.** ms1 is recomputed from `m.Entropy()` and compared to `plate.Seed`; the fingerprint, when non-zero, is recomputed from `m` with no passphrase; the QR is `seedqr.QR(m)`. All three derive from `m` (modulo I-1's guard, present at head). `TestEngraveSeedStringSeedQRCarriesTheDigits` decodes the QR back off the cut geometry with an independent segment parser (`qrdecode_test.go`), so the "never the ms1 string" claim is tested on the engraving, not on the input. The single-block restriction holds for V2-M and V3-M.
- **No panic on the refusal inputs:** empty, 3 words, out-of-range, bad checksum, wrong fingerprint, another seed's string, a non-entr codex32 string. Length switch first, then range, then `Valid`, so `Valid`'s `m[len(m)-1]` and `MnemonicSeed`'s wordlist lookup are never reached with bad input.
- **EncodeMS1 wipe.** `defer clear(payload)` runs after `s.String()` is evaluated; `NewSeed` copies bytes into a `strings.Builder` and keeps no reference (`codex32.go:288-350`); `String.String()` returns that Go string, not the payload. No aliasing; the caller's entropy is untouched (the test asserts it). Output unchanged (codex32 suite and the 26 Rust rows pass).
- **bip32 move.** Seed wipe (`clear` = `wipeBytes`), the `deriveSeedHook` firing point (right after the deferred wipe, before `NewMaster`), `deriveMasterKeyHook` (still in gui after the derivation), `masterFingerprintFailHook`, error texts and the `mk.Zero()` after the public-key fingerprint are all preserved. gui's F-94 pins still watch the moved wipe (the report's mutation table shows `TestDeriveMasterKeyZeroesTheBIP39Seed` killing a deleted `defer clear(seed)` in bip32). No new hook in gui; `seedQREntropyHook` and `encodeMS1PayloadHook` are nil-in-production observers that cannot change output. In TinyGo the seed and payload are passed to an indirect call and `NewMaster`/`NewSeed`, so they escape and LLVM cannot drop the `memset`.
- **Vectors are independent of the Go under test:** ms1 from ms-codec at the pin, entropy from rust-bip39 (cross-checked to Trezor's column), digits from rust-bip39's wordlist (cross-checked to SeedSigner's streams), widths from the `qrcode` crate. Regenerated byte-identically. Widths: all 12-word rows 25, all 24-word rows 29, matching the spec's stated 25x25/29x29. 26 rows, row count pinned in the test.
- **Plan conformance.** Signature as planned; general layout, no word-count policy; `EngraveSeedString` and `engraveSeedString` unchanged; no flow wired (D2); goldens carry a non-zero fingerprint (asserted) and a title; preview entry in `previewBuilders`, `preview.go` is `!tinygo`; the non-English case is explained (`bip39.Mnemonic` is `[]Word` over the one English list). Extras (digit-slice wipe, more refusal cases, `decodeQRAt`) change no behaviour.
