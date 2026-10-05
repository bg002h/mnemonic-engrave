# Refugium F5: implementation report (ms1 plus Standard SeedQR plate)

- **Implementer:** single implementer subagent (Opus 5.5), 2026-10-05
- **Plan:** `design/IMPLEMENTATION_PLAN_fork_refugium_F5_F7.md` draft 4 (GREEN), §3 only, with §2 D2 and the round folds that touch §3 (R0 M-7, M-8, M-9, I-8b; round 2 M-5, M-6)
- **Base:** `bg002h/seedhammer` main `be00ef8` (`git ls-remote` on 2026-10-05: main is still `be00ef8857ba6f093682e4ed89acd7c1848be59d`)
- **Branch:** `claude/project-thread-59q2a6`, 3 commits, local only (not pushed, no PR)

| Commit | Subject |
|---|---|
| `4af751a` | bip32: move the master-key derivation out of gui (F-702 F5) |
| `77e698c` | codex32: wipe EncodeMS1's payload buffer (F-702 F5) |
| `54e7276` | backup: ms1 plate with a Standard SeedQR (F-702 F5) |

Diff: 20 files, +1629 / -40 (most of the additions are the vector JSON, `Cargo.lock` and the two gzip goldens).

## Where the branch is, and where this report is (deviations from the brief)

The brief asked for `git -C /home/claude/seedhammer worktree add -b claude/project-thread-59q2a6 /home/claude/wt-f5 origin/main`. **That could not be run.** This agent ran isolated in the mnemonic-engrave worktree `.claude/worktrees/agent-a14cb1634a49d06c8`, and a harness guard refuses any git command aimed outside it (`git -C /home/claude/seedhammer …` and `cd /home/claude/seedhammer && git …` were both refused, including read-only `git log`). The same guard refused writing this report to the shared-checkout path `/home/claude/mnemonic-engrave/design/agent-reports/`. I did not try to get around it.

What I did instead:

- I cloned the fork from `/home/claude/seedhammer` into my own worktree (`git clone /home/claude/seedhammer fork`, which only reads the source) and created the branch there from `be00ef8`. `/home/claude/seedhammer` was not modified.
- The commits are in the clone at `/home/claude/mnemonic-engrave/.claude/worktrees/agent-a14cb1634a49d06c8/fork` (branch `claude/project-thread-59q2a6`).
- They are also in a verified git bundle at `/tmp/claude-0/-home-claude/8b33999c-fe04-5c95-8609-1e0aa46702d1/scratchpad/f5.bundle` (prerequisite `be00ef8`), in case the agent worktree is cleaned up.
- To bring them into the fork checkout: `git -C /home/claude/seedhammer fetch <bundle-or-clone-path> claude/project-thread-59q2a6:claude/project-thread-59q2a6`.
- This report is at `/home/claude/mnemonic-engrave/.claude/worktrees/agent-a14cb1634a49d06c8/design/agent-reports/refugium-F5-impl.md` (uncommitted). It still needs copying to the shared `design/agent-reports/`.

Two side notes:

- The clone source is shallow: `/home/claude/seedhammer` has a depth-1 history at `be00ef8`.
- The commits use the configured git identity (`Claude <noreply@anthropic.com>`) with no DCO `Signed-off-by`. The fork's CLAUDE.md asks for "signed + DCO, authored Brian Goss" on **upstream** PRs. If this branch goes upstream, it needs re-authoring and sign-off.

## Re-validation of §3 against the tree (before coding)

I checked every §3 fact at `be00ef8`. **Nothing was falsified.**

- `seedqr.QR` writes `%04d` per word and panics on an invalid mnemonic (`seedqr/seedqr.go:22-33`). True.
- `seedQRLevel = qr.M`, `seedQRMaxSize = 33` (`backup/backup.go:154-156` before the change); `qrScale = 3` in `engraveSeedString`. True.
- `codex32.EncodeMS1` = `NewSeed("ms",0,"entr",'s',[0x00]‖entropy)` (`codex32/msencode.go:17-31`). True. `NewSeed` writes into a `strings.Builder` and keeps no reference to `data`, so wiping the payload afterwards is safe.
- `masterFingerprintFor` at `gui/gui.go:894-913`. True: it calls `deriveMasterKey` (`gui.go:331-356`, which holds the seed wipe and the two F-94 hooks) and runs `defer mk.Zero()`.
- `backup` can import `codex32`, `seedqr`, `bip39` and `bip32` without a cycle. True: none of the four depends on `backup` (`go list -deps`), and `backup`'s own tests already import all four.
- `previewBuilders` is at `gui/preview.go:99-111`. True.
- Widths at `qr.M`: 25 for 48 digits, 29 for 96. True: measured by the Rust `qrcode` crate in the generator and by the Go `qr.Encode` in the test, for all 26 rows.
- `bip39.Mnemonic` is `[]Word` (English wordlist indices) with no language field (`bip39/bip39.go:22-24`). This is why the non-English refusal case cannot be built (see below).

Toolchain fact the plan does not mention: `go.mod` says `go 1.25.10`, but the backup and gui test files call `testing.T.ArtifactDir` (Go 1.26). So every test here ran with `GOTOOLCHAIN=go1.26.7`, the same toolchain CI uses (`test.yml` `go-version: '1.26'`). This box's default is go1.24.7, which auto-switches to 1.25.10, and at 1.25.10 the backup test package does not compile. That is a pre-existing condition, not caused by F5.

## What was built

### 1. `backup.EngraveSeedStringSeedQR` (`backup/backup.go:220-271`)

Signature exactly as planned: `(params engrave.Params, plate SeedString, m bip39.Mnemonic) (engrave.Engraving, error)`. It is a general layout and applies no word-count policy (R0 M-7). It works in this order:

1. It refuses `errSeedQRInvalidMnemonic` (`:181`) unless `len(m)` is 12, 15, 18, 21 or 24, every word is in `[0, bip39.NumWords)`, and `m.Valid()`. The range check comes first because `Valid`/`LabelFor` on an out-of-range `Word` or an empty slice would otherwise misbehave or panic; a test pins "no panic".
2. It takes `entropy := m.Entropy()`, runs `defer clear(entropy)` (`:235`), and fires the nil-in-production hook `seedQREntropyHook` (`:190`).
3. It computes `codex32.EncodeMS1(entropy)` and refuses `errSeedQRDisagree` unless the result equals `strings.ToLower(plate.Seed)`.
4. If `plate.MasterFingerprint != 0`: it computes `bip32.MnemonicFingerprint(m, MainNet, "")` and refuses `errSeedQRFingerprint` on a mismatch.
5. It computes `digits := seedqr.QR(m)`, encodes it with `qr.Encode(string(digits), seedQRLevel)` (that is `qr.M`), then runs `clear(digits)` (`:257`). It refuses above `seedQRMaxSize`, builds `engrave.ConstantQR`, and returns `engraveSeedString(params, plate, qrCmd)`. `engraveSeedString` is unchanged.

The doc comment lists the secret copies. Wiped: the entropy buffer, the `[]byte` digit stream, and inside the callees `EncodeMS1`'s payload, the 64-byte BIP-39 seed and the master private key. Live and not wipeable: the digit **string**, the `qr.Code` bitmap, the recomputed ms1 string, and the caller's `plate.Seed` and `m` (`m` is also the fingerprint input). This is R0 M-8 and round 2 M-6.

Wiping `digits` (the `[]byte` that `seedqr.QR` returns) goes beyond the plan's list: the plan named the digit string as live, and it still is. The `[]byte` copy behind it is now wiped as well.

### 2. Fingerprint helper moved out of gui (`bip32/master.go`)

I put it in the existing non-gui package `seedhammer.com/bip32` rather than a new package. That package already holds `Fingerprint`, imports `hdkeychain` and `chaincfg`, and is already linked into the firmware.

- `MasterKey(m, net, password, seedHook)` (`:26`) is the derivation from `deriveMasterKey`, with `defer clear(seed)` on the 64-byte BIP-39 seed. `seedHook` is a parameter and not a package variable, so gui can hand its own `deriveSeedHook` through and the hook still fires right after the wipe is deferred, as before.
- `MasterFingerprint(mk)` (`:42`) computes the fingerprint from the public key and runs `defer mk.Zero()`.
- `MnemonicFingerprint` (`:53`) composes the two. `backup` calls it.
- gui: `deriveMasterKey` (`gui/gui.go:333`) is now `bip32.MasterKey(m, net, password, deriveSeedHook)` followed by `deriveMasterKeyHook`. `masterFingerprintFor` (`gui/gui.go:887-903`) keeps its fail hook and error text and returns `bip32.MasterFingerprint(mk)`. The SeedScreen probe still calls `deriveMasterKey` and zeroes the key itself. gui behaviour is unchanged. The F-94 pins in `gui/master_key_residue_test.go` run unchanged; only their comments and one failure message were updated to name the new location.
- The seed wipe went from gui's `wipeBytes` loop to the `clear` builtin, which has the same effect. `clear` is already used in the firmware (`bip39.go:136`).

### 3. `EncodeMS1` payload wipe (`codex32/msencode.go:30,44`)

`defer clear(payload)` plus a nil-in-production `encodeMS1PayloadHook`. The output is byte-identical: all existing codex32 tests pass, and the new test re-checks ms-codec's 16×`0x7f` string.

### 4. Vectors and provenance

- `backup/testdata/ms1_seedqr_vectors.json`: 26 rows, 15 with 12 words and 11 with 24. Columns: `name`, `words`, `entropy`, `ms1`, `seedqr`, `qr_width_m`.
- `backup/testdata/ms1_seedqr_vectors.provenance.json`: every source with URL, rev and sha256, plus the generator path, lockfile, command, toolchain and the checks the generator makes.
- Generator `backup/testdata/gen/`: a standalone Cargo crate with an empty `[workspace]`, `Cargo.lock` committed and `/target` gitignored. It is under `testdata/`, so the Go tooling ignores it. Dependencies: `ms-codec` git rev `e5dff6f9c2b7ae7dc02a4a137e7322abab32211e` (0.10.0, locked), `bip39` 2.2.2, `qrcode` 0.14.1 (default features off), serde/serde_json/sha2/hex.
  - **ms1 column:** `ms_codec::encode(Tag::ENTR, &Payload::Entr(entropy))`. This is the crate's real API at that rev: `encode(tag: Tag, payload: &Payload) -> Result<String>`, with `Payload::Entr(Vec<u8>)`.
  - **entropy column:** rust-bip39 `to_entropy`, with the checksum enforced. For Trezor rows it is asserted equal to Trezor's published entropy.
  - **seedqr column:** each word's index in rust-bip39's English wordlist, formatted `%04d`. For SeedSigner rows it is asserted equal to the verbatim digit stream. Go `seedqr.QR` is never used.
  - **qr_width_m column:** `qrcode` at `EcLevel::M`.
  - The generator refuses a Trezor file whose sha256 differs from the recorded one. I re-ran it with `--locked`, and the output reproduced byte for byte.
- **Sources:**
  - SeedSigner `docs/seed_qr/README.md` at `b2199c68b475eab0035b863b156f149757a1b4a6` (dev), sha256 `44fe1a32…1a3c`. The scratchpad copy from the recon is byte-identical to that rev. I used the worked example (vacuum … nuclear) and Test Vectors 1-9: 4 rows of 24 words (example excluded) and 6 of 12. Words and digit streams are copied verbatim into `gen/seedsigner_rows.json`, and a script confirmed that every stream and every TV word line occurs verbatim in the spec.
  - Trezor `python-mnemonic` `vectors.json` at `b57a5ad77a981e743f4167ab2f7927a55c1e82a8` (master), sha256 `fa3b937b…30f8`. Fetched by commit URL. The master-branch fetch has the same sha.
  - All 16 English rows with 12 or 24 words are used: indices 0-3, 12, 15, 18 and 21 (12 words); 8-11, 14, 17, 20 and 23 (24 words). There are no M15 rows (R0 M-9). The file itself is not committed (the generator verifies its sha), which keeps a third-party JSON out of the fork.
- **Spot checks:**
  - Trezor row 00 (all-zero entropy) gives `ms10entrsqqqq…cj9sxraq34v7f`, the vector R0 round 2 cross-checked against Go.
  - SeedSigner TV1's entropy equals its published CompactSeedQR bytestream (`0e74b641…`).

### 5. Goldens

`backup/testdata/ms1-seedqr-24.bin` (row `seedsigner-tv1`) and `ms1-seedqr-12.bin` (row `seedsigner-tv4`), written through `compareGolden`. To regenerate: `go test ./backup -run TestEngraveSeedStringSeedQRGolden -update`.

Each golden carries the row's true non-zero fingerprint, which the test derives straight from hdkeychain and not through the helper under test, and a title. The title is the ms1 id `entr` (from `codex32.Split`, the `TestCodex32` pattern).

**SVG (described, not required):** I rendered the `ms1seedqr` preview (all-zero 12-word seed) to PNG through `cmd/plateview` and headless Chromium.

- The fingerprint row `73C5DA0A` is centred above the block.
- The uppercase ms1 string runs in five rows of ten characters in column 1: `MS10ENTRSQ / QQQQQQQQQQ / QQQQQQQQQQ / QQQQQQQCJ9 / SXRAQ34V7F`.
- A 25-module QR sits on the right, centred at x = 60 mm and vertically on the block.
- The title `ENTR` is centred below.

This is the same frame as `codex32-*`, with a smaller QR. In the 24-word case the QR is 29 modules, against 33 for today's ms1-string QR.

### 6. Preview

`"ms1seedqr"` was added to `previewBuilders` (`gui/preview.go:113`). The builder `ms1SeedQRPreview` (`:222`) uses `previewSeed`: it calls `EncodeMS1` (and wipes the entropy), then `masterFingerprintFor`, then `backup.EngraveSeedStringSeedQR` with title `entr`, then `toPlate`. `go run ./cmd/plateview -plate ms1seedqr` prints "fixed layout, 0 rows, ~12m52s to engrave (carries a QR code)". `preview.go` is `!tinygo`, so none of this reaches the firmware.

## Tests

All test files were written first and seen to fail before the implementation: compile failures on the missing `EngraveSeedStringSeedQR`, `MnemonicFingerprint` and `encodeMS1PayloadHook`; then golden "no such file" before `-update`; and `TestPreviewMS1SeedQR` failing on "want an ms1seedqr entry".

New tests:

- `backup/ms1_seedqr_test.go`
  - `TestMS1SeedQRVectors` (`:95`). Per row it checks that `seedqr.QR` equals the digits, that `seedqr.Parse` gives back the words, that `m.Entropy()` equals the entropy column, that `EncodeMS1(entropy)` equals the Rust ms1, and that the `qr.Encode` width at `seedQRLevel` equals the recorded width and is at most 33. It also requires exactly 26 rows and both lengths present.
  - `TestEngraveSeedStringSeedQRGolden` (`:163`).
  - `TestEngraveSeedStringSeedQRCarriesTheDigits` (`:224`) **reads the QR back off the engraving**: the module grid is recovered from the cut geometry and decoded. For four rows (12 and 24 words, SeedSigner and Trezor) the result equals the SeedQR digits and contains no ms1 string. To do this, `decodeQR` in `backup/qrdecode_test.go` was generalised to `decodeQRAt(level)`; `decodeQR` still means ECC-L.
  - `TestEngraveSeedStringSeedQRAcceptsUppercase` (`:246`) checks an uppercase `plate.Seed` and a zero fingerprint.
  - `TestEngraveSeedStringSeedQRRefusals` (`:258`). Each case asserts `errors.Is`, no engraving, and no panic:
    - `word-changed`: word 0 bumped and the checksum repaired, so the words are a valid mnemonic of another seed;
    - `wrong-fingerprint`;
    - `bad-checksum`;
    - `word-out-of-range`;
    - `three-words`;
    - `no-words`;
    - `string-of-another-seed`;
    - `not-an-ms1-string`: a non-entr codex32 string.
  - `TestEngraveSeedStringSeedQRWipesTheEntropy` (`:332`).
- `bip32/master_test.go`: `TestMnemonicFingerprintKnownVector` (abandon…about gives `73c5da0a`, BIP-84's printed fingerprint), `TestMasterKeyWipesTheBIP39Seed` and `TestMasterFingerprintZeroesTheKey`.
- `codex32/msencode_wipe_test.go`: `TestEncodeMS1WipesItsPayload`.
- `gui/preview_ms1seedqr_test.go`: `TestPreviewMS1SeedQR`.

**Non-English mnemonic (plan §3 item 4).** No such case can be built. `bip39.Mnemonic` is `[]Word`, a slice of indices into the single English wordlist the package embeds, with no language field. A non-English mnemonic therefore cannot be represented in the type and cannot reach the function. The doc comment says so. The closest representable inputs (out-of-range indices, wrong lengths) are refusal cases.

**Mutation checks (all killed):**

| Deleted line | Killed by |
|---|---|
| `defer clear(entropy)` | `TestEngraveSeedStringSeedQRWipesTheEntropy` |
| `defer clear(payload)` | `TestEncodeMS1WipesItsPayload` |
| `defer clear(seed)` and `defer mk.Zero()` in `bip32/master.go` | both bip32 wipe tests |
| `defer clear(seed)` alone | gui's own `TestDeriveMasterKeyZeroesTheBIP39Seed` (proves gui's F-94 pin still watches the moved wipe) |

**Commands and counts** (each run once, captured to a file and grepped, all with `GOTOOLCHAIN=go1.26.7`):

`go test -v ./backup/ ./seedqr/ ./codex32/ ./bip32/ ./bip39/ ./cmd/plateview/ ./seal/`: all `ok`, 0 `--- FAIL`. Top-level `--- PASS` counts:

| Package | PASS |
|---|---|
| backup | 147 |
| seedqr | 2 |
| codex32 | 64 |
| bip32 | 3 |
| bip39 | 13 |
| cmd/plateview | 2 |
| seal | 116 |

- `TestCodex32` (goldens `codex32-0/1`) and `TestEngraveSeedStringTooLong`/`Happy` pass untouched. No existing golden `.bin` changed (`git diff --stat` on `backup/testdata/*.bin` is empty). The seal package, which holds the `MaxEngraveableCodex32Len` derivation, is green.
- `mnemonic-engrave/scripts/gui-shard-test.sh ./gui 24`, run from the clone: `partition verified exhaustive: 1404 == 1404`; `RESULT: ok -- all 1404 tests ran across 24 shards`; wall time 89 s.
- After the shard run I made one more change: the `digits` `[]byte` wipe in `backup.go`, plus comment wording. It touches no gui code. After it I re-ran `./backup/` (ok, goldens unchanged), the gui tests matching `Preview|MasterKey|MasterFingerprint|DeriveSeed` (ok) and `./cmd/plateview/` (ok). I did not re-run the full sharded gui suite after that change.
- `go build ./...`: ok.
- `gofmt -l .` equals the five-file baseline exactly: `gui/transaction.go`, `gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go`.

## Deviations and why

1. **Worktree and report location** (above): the harness guard blocked git against `/home/claude/seedhammer` and writes into the shared mnemonic-engrave checkout. The branch was created in a clone inside the agent's worktree, with a bundle for safety, and the report sits in the agent worktree's `design/agent-reports/`.
2. **Helper home is `bip32`, not a new package.** The plan says "a non-gui package both callers use". `bip32` is the natural owner, already firmware-linked, and needs no new import edges beyond `bip39`.
3. **`EncodeMS1` itself changed** (to wipe its payload). The plan says the payload "is wiped with a helper the implementer names". The payload is a local of `EncodeMS1`, so the wipe has to live there. Output is unchanged, so this is not a normative change under the Rust-primary rule. Ported-package provenance pins need no bump: `PROVENANCE.md` has no `EncodeMS1` entry.
4. **Wipe helper:** the `clear` builtin, used directly in `backup`, `bip32` and `codex32`, instead of a local `wipeBytes` copy. It has the same effect and is already used in firmware code.
5. **Extra wipe:** the `[]byte` digit stream (see §1).
6. **Extra refusal cases** beyond the plan's four: out-of-range word, three words, no words, a string of another seed, a non-entr codex32 string. There is also a QR read-back test.

## Not done or not in scope

- **TinyGo/ELF build.** §3 does not ask for one. F5 adds no new package to the firmware's import graph (`backup` now imports `bip32`/`bip39`/`codex32`/`seedqr`/`chaincfg`, all already linked through gui). The new function has no firmware caller, so it should be dead-stripped. **Not measured.**
- **No flow is wired.** That is by design (D2): F3/F4 are the consumer.

## Open questions

1. Who brings the branch into `/home/claude/seedhammer` (fetch from the clone or the bundle), and should it be re-authored and signed off as Brian before a PR?
2. Should the golden's title stay `entr` (the ms1 id, as `TestCodex32` does)? Refugium's letter row ("A", "B", …) is F3's choice, and `Title` is caller-supplied, so either works.
3. Should the plan or CI pin `GOTOOLCHAIN` (or `go.mod` move to `go 1.26`)? `go.mod`'s `go 1.25.10` cannot compile the `backup` and `gui` test packages (`testing.ArtifactDir`). `go vet ./backup/ ./gui/` therefore reports a stdversion error even at 1.26.7. This predates F5.
