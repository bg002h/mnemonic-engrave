# IMPLEMENTATION PLAN — SeedHammer fork: the Refugium kit door and MNEMBLOB v2 (goal 2, plan S2)

> **Status: DRAFT 1, pre-R0 (2026-11-22).** Author: Claude Opus 5.5 (claude-opus-5-5). SLIM plan
> (coordinator `briefs/author-brief-template.md`): step order, rows with expected output, the coverage map
> (`design/IMPLEMENTATION_PLAN_fork_refugium_kit-coverage.tsv`), the tests that change, and the committed gate
> `scripts/plan-gate-fork-refugium-kit.sh`. No code blocks. Risk set (seeds, a wiping container, a wire format):
> R0 to 0 Critical / 0 Important before code; one implementer per phase; an adversarial review of each phase's
> whole diff after. TRACE rows: the Rule cell and its plan legs are binding.

## 0. Baselines

| What | Where | Revision |
|---|---|---|
| Spec S1 (kit door, `ClassMr`, Refugium warning) | this repo `design/SPEC_systemwide_payloads.md` §14 (branch `spec/sysw-class-mr`) | `c4737e8b` (GREEN) |
| Spec S1b (MNEMBLOB v2) | this repo `design/SPEC_encrypted_payload_delivery.md` §14 (branch `spec/mnemblob-v2`) | `844572cd` (GREEN; f7c98cbf plus the I-3 sentence) |
| This repo | `master` | `c9c4c54b` |
| Fork | `bg002h/seedhammer` `origin/main` | `25abe8e` |
| S1 vectors | `design/vectors/sysw_mr_v1_vectors.json` at `c4737e8b` (13 string, 12 set, 10 kit) | `c4737e8b` |
| S1b vectors | `scripts/gen-mnemblob-v2-vectors.py` at `844572cd` (16 vectors, `--check` run 2026-11-22: "checked 16 vectors, 0 missing"; this plan adds 2, row `v2_titles_refusals`) | `844572cd` |
| Go | 1.26.4 (`/home/bcg/.local/go/bin`); the flake's Go for TinyGo and `nix run .#build-firmware` | measured 2026-11-22 |
| Rust | `cargo nextest run --locked`, own `CARGO_TARGET_DIR`, never `--release` | 715 tests listed at `c9c4c54b` |

Both specs are on unmerged branches. **Precondition for every phase: S1 and S1b are merged to `master`**, and
the phase's implementer re-reads the merged §14 against the cites here (a plan's GREEN expires, CLAUDE.md).

## 1. Decisions this plan takes (code choices, not product points)

- **D1. Rust first, in this plan.** Both specs make the Rust the primary and the Go a port pinned to vectors
  (S1 §8.1, S1b §14.3, this repo's Rust-primary rule). No other plan builds the `me` side, and the owed row
  `s1br3-nothex-nonms1-vectors` is an `me` row, so phases ME-1 and ME-2 are here and precede the fork phases
  that vendor their output.
- **D2. "The byte-slice Open API" is the Sealed Payload path, not a new `sysw.Open`.** The synthesis graft (a
  byte-slice `sysw.Open` for secrets, architecture B §2.4) is dropped by the architect report §5 ("Dropped from
  the old P2: B §2.4's wipeable sysw open"): secrets no longer ride sysw. The secret path this plan touches is
  already byte-slice (`seal.Payload` records are `[]byte`, `seal.NewDeriver` and `UnlockWithKey` take bytes,
  `passphraseBytes` builds a fixed-capacity buffer). This plan keeps it so: the n-word password, the titles block,
  the letter check and the F5 SeedQR words are all built in `[]byte` / `bip39.Mnemonic` buffers that are zeroed
  (§5). The non-secret kit path (mr1, mk1, md1) stays on `sysw` strings, which S1 allows (`ClassMr` is not secret).
- **D3. No plate-pack vector generator.** S5 is dropped (architect ruling on S4 r1 I-3): the device's packer is
  the only packer, and test 32's row ties its lead count to its own cuts.
- **D4. The Go-vector capacity contract** (what the Go port must hold to, by vector): (a) the vendored files are
  byte-identical to the `me` files, with their `me` commit and SHA-256 in a provenance file, checked by a test;
  (b) S1b §14.3's record budget (24 records with the titles record, a titles record of at most 512 bytes, so 18
  `SEED <L>` titles fit and 19 do not); (c) S1b §14.8's bound: a v2 blob is read through `boundBlob` and copies
  at most 16,450 bytes; (d) S1 test 32: the lead's M equals the plates the door cuts.
- **D5. F1 (lane F, F-702) is met by this plan's string layer and payload reading only.** F1's "the card's QR"
  is not built: S1 §14e gives the kit door one choice, TEXT ONLY (Q10).
- **D6. Placement.** Fork code lands on `bg002h/seedhammer` `main` through PRs branched off `origin/main` (as F5
  and F7 did, PRs 2 and 3); `me` code through this repo's PR track. Per-phase branches: `build/kit-me-sysw-mr`,
  `build/kit-me-seal-v2` (this repo), `kit-sysw-mr`, `kit-door`, `sealed-v2-port`, `sealed-v2-device`,
  `kit-emu-walks` (fork).
- **D7. Test seams** are §7; each test's fakes are named there.

## 2. Phases and step order

PHASE ME-1 — `me` sysw: `ClassMr`, the chunk-set and kit rules, `me sysw pack` sentences (S1 §14b-§14d, §14g, §14h).
PHASE ME-2 — `me` seal v2 library and vectors (S1b §14.2-§14.5, §14.7, §14.9, §14.10 ME rows), with the two owed vectors.
PHASE SH-1 — fork codec: `codex32.ValidMR`, `sysw.ClassMr`, §14c/§14d rules in Go, the run grouping, admission (S1 tests 24-26, 28).
PHASE SH-2 — fork device: payload menu, Kit Refused, the Engrave Bundle door, the kit list and run, marks, the Refugium warning (S1 tests 29-32).
PHASE SH-3 — fork `seal/` v2 port: header, `pw_form`, the floor, titles, letter, vendored vectors, capacity (S1b §14.2-§14.4, §14.8).
PHASE SH-4 — fork device: n-word unlock, titles on screen and steel, T-1/T-2, the F5 SeedQR plate and its wipes (S1b §14.4-§14.6).
PHASE SH-5 — emulator walks: kit door and v2 unlock, both builds.
PHASE SH-6 — hardware: H-v2 (S1b §14.8) and the size row.
PHASE G-REL — the F-124 / F-123 release gate (not a build; §9).

Order: ME-1 ∥ ME-2. SH-1 after ME-1; SH-2 after SH-1. SH-3 after ME-2; SH-4 after SH-3. SH-5 after SH-2 and
SH-4. SH-6 after SH-5. Each phase: RED commit (its rows written, run, seen failing as the row's RED cell
says), GREEN commit(s), the gate's `counts --built <phases so far, comma-separated>` (e.g. `counts --built
ME-1,SH-1`), then the repo's full CI locally (`cargo nextest run --locked` plus every CI job for `me`; for the
fork, every `run:` step of `.github/workflows/test.yml` at the phase head: `CGO_ENABLED=0 go test -timeout 20m
./...` (the `gui` package may run as `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui
24` from the fork root), `CGO_ENABLED=0 go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/
./cmd/emu/`, `./scripts/test-32bit.sh`, `GOOS=js GOARCH=wasm go vet ./cmd/emu/` and the same with `-tags
refugium`, `./scripts/refugium-tagged-test.sh`, the two `nix develop --command tinygo build ...` lines as written
there, then `./scripts/refugium-elf-check.sh default.elf refugium.elf`; and `gofmt -l .` lists exactly the five
baseline files (gui/transaction.go, gui/transaction_golden_test.go, gui/transaction_txrecord_test.go, mt/mt.go,
mt/mt_test.go). No `go vet ./...` (measured 10 findings at 25abe8e).)

### ME-1 steps
1. RED: write rows `sysw_mr_string_vectors`, `sysw_mr_set_vectors`, `sysw_mr_kit_vectors`, `sysw_pack_prints_mr_sentences`,
   `sysw_mdmk_groups_by_runs`, `sysw_show_names_mr1_string`, `seal_still_refuses_mr1`; change the two `sysw_cli.rs`
   tests and `assert_every_named_test_is_placed` (§8). Run: each new row fails (absent class) except the guard
   `seal_still_refuses_mr1`, which passes.
2. GREEN: `sysw/record.rs` `Class::Mr` (not secret, not bearer); `sysw/mr.rs` (§14b-§14d, loads the vector file);
   `sysw/mod.rs` classify arm after the mt1 arm and the §14b `UnknownReason`s; `main.rs` sentences, pack steps,
   `class_name` "mr1 string", the `show` line; `mdmk_unconfirmed_why` groups by runs; `sysw/coverage.rs` places
   tests 24-32, `HIGHEST` 32 (the entries are in §8's CHANGE line).
3. Toolkit manual lockstep (S1 §14i): `mnemonic-toolkit` `docs/manual/src/30-workflows/3B-payload-unlock-refusals.md`
   "Which records get refused" gains the §14g mr1 rows and the new `Unrecognised` sentence, in a toolkit PR merged
   with this phase; toolkit CI (manual lint and `ci/doc-flag-lint.test.sh`) green on its head.
4. `scripts/check-sysw-mr-vectors.sh` prints no `RED` line (its `RED kit-two-copies-mk1` flips here).

### ME-2 steps
1. Owed vectors first (closes `s1br3-nothex-nonms1-vectors`): `scripts/gen-mnemblob-v2-vectors.py` gains N-not-hex
   and N-mnemonic exactly as §3's leg of `v2_titles_refusals`; spec §14.9 gains their table rows, header hex and
   titles-record lines; `scripts/spec-s1b-gate.sh` passes ("checked 18 vectors, 0 missing").
2. RED: every ME row of §3 (ME-2), and the changed `rejects_bad_magic_version_reserved_kdf_and_aead`. Run: the v2
   rows fail to compile or fail as their RED cell says; the guards `v1_still_refuses_a_nonzero_reserved_byte` and
   `me_seal_still_writes_version_1` pass.
3. GREEN: `seal/wire.rs` (`VERSION_V2`, `PW_WORDS_MIN/MAX`, `MIN_ITERATIONS_NWORD`, `PwForm`, `PwFormIterations`,
   `Header.version`/`pw_form`, check order of §14.2); `seal/titles.rs`; `seal_v2`, `seal_v2_deterministic`, `open`;
   `testdata/seal_v2_vectors.json` emitted and drift-checked as `emit_vectors` does for v1; the F-125
   `passphrase.rs` doc edit (S1b §14.11).

### SH-1 steps
1. Vendor `sysw_mr_v1_vectors.json` into `sysw/testdata/` with a provenance entry (`me` merge commit, SHA-256).
2. RED: rows of §3 (SH-1). Run: absent symbols fail to compile;
   `TestSyswMDMKGroupsByRuns` fails on today's grouping (all four records unconfirmed).
3. GREEN: `codex32.ValidMR` (copy of `ValidMT`: hrp `mr`, target `0x0`/`0x85d21404aee15059`, ≤ 93 data symbols);
   `sysw/record.go` `ClassMr` (not secret); `sysw/classify.go` arm after mt1 with the §14b reasons; a new
   `sysw/mr.go` for §14c/§14d returning a reason and its detail; `sysw/confirm.go` grouping by runs;
   `gui/sysw_admit.go` `progBundle` admits `ClassMr`; `gui/transaction.go` `txClassName` "mr1 string".

### SH-2 steps
1. RED: rows of §3 (SH-2); run each, failing as stated.
2. GREEN: `gui/sysw_unload.go` (`syswPayloadMenu`: the kit lead and ENGRAVE KIT, the Kit Refused screen);
   `gui/sysw_load.go` (Kit Refused after Payload Warnings, not after UNLOAD; the §14f line in a
   `sysw_load_refugium.go` / `sysw_load_default.go` twin per `gui/refugium_build_test.go`); `gui/bundle_flow.go`
   (the door at `bundleFlow`'s entry; `kitCards`, the one function that reads the session's `ClassMr` and
   `ClassMDMK` records; `kitListFlow`, `kitRun`, marks "CARD k/N" / "PLATE p/q" through
   `bundlePlateMark`'s slots, the pause, end and does-not-fit screens; no ms1 reminder).
3. After the fork SH-2 PR merges, an `me` PR (branch `build/kit-me-coverage-device`) flips tests 28-32 in
   `sysw/coverage.rs` to `Where::Device`, each with a comment naming its fork test (TestAdmitMrAtBundleOnly,
   TestKitMenuOffersEngraveKit, TestKitDoorRun, TestRefugiumSecretWarningLine, TestKitLeadPlatesEqualCuts); full `me` CI.

### SH-3 steps
1. Vendor `seal_v2_vectors.json` into `seal/testdata/` with a provenance entry (`me` merge commit, SHA-256).
2. RED: rows of §3 (SH-3) and the changed `TestRejectsBadMagicVersionReservedKDFAndAEAD` (§8).
3. GREEN: `seal/wire.go` (`VersionV2`, `PwWordsMin/Max`, `MinIterationsNWord`, `ErrPwForm`, `Header.Version`,
   `Header.PwForm`, §14.2 order); `seal/titles.go` (titles parse, `TitlesError` with the ten checks and the Rust
   Display texts, `ms1Letter`); `seal/open.go` / `seal/unlock_key.go` (the `text:` record leaves §10.2.1 for the
   titles rule in a v2 encrypted section; `Payload.Titles`; a titles refusal wipes every record first);
   the "that one stays frozen" comment in `gui/gui.go` above the boot load corrected (S1b §14.11).

### SH-4 steps
1. RED: rows of §3 (SH-4) and the changed `TestRefugiumUnlockCodex32PlateIsTextOnly` (§8).
2. GREEN: `gui/unlock_kdf.go` (notice by `pw_form`, n-slot entry with no checksum gate and no terminator, the
   first-frame estimate, the retry body), `gui/unlock_flow.go` (T-1/T-2 under title `Sealed Payload`, then the
   menu), `gui/unlock_session.go` (lead line, title on steel, the Refugium SeedQR plate and its wipes, §5),
   `gui/ms1_qr_gate.go` (the SeedQR selector), the HONEST CAVEAT inventory.

### SH-5 steps
`cmd/emu`: payload choices `kit` (S1 `kit-2of3`, packed by `me sysw pack --no-passphrase` at the ME-1 merge) and a
sealed choice `v2` (vector V2-A's blob, emitted from the vendored JSON); host tests pin both blobs' SHA-256 and
provenance; walks `cmd/emu/walk_kit_door.js` and `cmd/emu/walk_sealed_v2.js`; runner `scripts/emu-kit-walk.sh
kit|v2 default|refugium` (the shape of `scripts/emu-refugium-walk.sh`). Rows: §3 WALK lines.

### SH-6 steps
Hardware rows of §3 (HW lines) on the SeedHammer II, key 846aa289 build, signet test material; results to
`design/HARDWARE_RESULT_<date>_kit_v2.md`.

## 3. Rows

Format (the gate parses these lines): `ROW <name> | <phase> | <repo> | <target> | <tags> | now=<n> | built=<n>`.
`repo` is `me` (target `me-cli`, matched on the last `::` segment of `cargo nextest list`) or `sh` (target a Go
package, matched by `go test -list '^<name>$'`). `now` is the count at the baselines (0: absent). Each ROW's
Rule (expected output) and its RED state and caught mutant follow it.

### ME-1
ROW sysw_mr_string_vectors | ME-1 | me | me-cli | - | now=0 | built=1
- Rule: every `string_vectors` row classifies as its `class`; an `Unknown` row reports its `reason` (S1 test 24).
  RED: absent (no `Class::Mr`; today `single-card` is `Unrecognised`). Mutant: the case check after the BCH check
  (`upper` would then read `MR_CHECKSUM`, not `MR_CASE`).
ROW sysw_mr_set_vectors | ME-1 | me | me-cli | - | now=0 | built=1
- Rule: every `set_vectors` row gets its verdict, reason and detail (S1 test 25); `complete-reversed` and
  `framing-only` pass. RED: absent. Mutant: sets keyed by chunk-set id alone (`kit-two-copies` would refuse
  `SET_DUPLICATE`).
ROW sysw_mr_kit_vectors | ME-1 | me | me-cli | - | now=0 | built=1
- Rule: every `kit_vectors` row gets its verdict, reason, detail and cards (kind, string count, begins, ends) in
  order (S1 test 26); `kit-2of3` is 6 cards. RED: absent. Mutant: `begins` taken from the first record in payload
  order instead of index 0 (`complete-reversed` in a kit would differ).
ROW sysw_pack_prints_mr_sentences | ME-1 | me | me-cli | - | now=0 | built=1
- Rule: `me sysw pack` prints "me: " and each §14g sentence in `me`'s record form and exits 4 for every string
  and set refusal; prints "me: this payload is not a kit the device will engrave: " and the sentence once and
  exits 0 for `KIT_OTHER_RECORD`, `KIT_UNCONFIRMED_MDMK` and an mk1 `CARD_SPLIT`; a payload with no mr1 prints
  neither; a set given around a secret record packs (S1 test 27). RED: today exit 4 with "not an md1/mk1/ms1/mt1
  string". Mutant: the kit warning made a refusal (exit 4).
ROW sysw_mdmk_groups_by_runs | ME-1 | me | me-cli | - | now=0 | built=1
- Rule: `me sysw pack --no-passphrase --in dupmk.txt --out dupmk.bin` then `me sysw show dupmk.bin`, `dupmk.txt`
  being `kit-two-copies-mk1`'s first mk1 card given twice: no record is "md1/mk1 — unconfirmed" and no "SECRET"
  flag line (S1 §14d, §14i). RED (run by S1's author, printed by `scripts/check-sysw-mr-vectors.sh` as `RED
  kit-two-copies-mk1`): all four "md1/mk1 — unconfirmed", "the device will treat it as a SECRET". Mutant:
  grouping by (hrp, chunk-set id).
ROW sysw_show_names_mr1_string | ME-1 | me | me-cli | - | now=0 | built=1
- Rule: `me sysw show` prints "public record 0: mr1 string" for `single-card`; the `Unrecognised` refusal reads
  "not an md1/mk1/mr1/ms1/mt1 string" and `pack --help` "an md1/mk1/mr1/ms1/mt1 string" (§14h). RED: absent.
  Mutant: `class_name` left "unrecognised record".
ROW seal_still_refuses_mr1 | ME-1 | me | me-cli | - | now=0 | built=1
- Rule: `me seal --plaintext` over `single-card` exits non-zero with "unrecognized HRP 'mr' (expected md, mk,
  ms, or mt)" (S1 test 28, sealed half). GUARD: green today (architect report §1 ran it). Mutant: `Class::Mr`
  leaking into the seal allow-list.

### ME-2 (S1b §14.10's ME rows; names as there)
ROW v2_header_round_trips_each_password_form | ME-2 | me | me-cli | - | now=0 | built=1
- Rule and RED as S1b §14.10 (RED run: `Err(UnknownVersion(2))`). Mutant: a decoder pinning `version == 1`.
ROW v2_refuses_pw_form_outside_0_and_6_to_12 | ME-2 | me | me-cli | - | now=0 | built=1
- Rule as §14.10 (each offset-11 edit of §14.9 gives `PwForm(b)`). RED: no variant. Mutant: `<= 12` only.
ROW v2_refuses_n_word_payload_below_600000_iterations | ME-2 | me | me-cli | - | now=0 | built=1
- Rule as §14.10 (N-floor-599999 → `PwFormIterations(599999)` in under 200 ms; every §14.9 floor edit). RED: absent.
  Mutant: floor applied to `pw_form` 0.
ROW seal_v2_refuses_n_word_below_600000_iterations | ME-2 | me | me-cli | - | now=0 | built=1
- Rule as §14.10. RED: absent. Mutant: a writer that emits what `open` refuses.
ROW v2_public_only_refuses_nonzero_pw_form | ME-2 | me | me-cli | - | now=0 | built=1
- Rule as §14.10 (`pw_form must be zero when nothing is encrypted`). RED: absent. Mutant: unsealed branch skips offset 11.
ROW v1_still_refuses_a_nonzero_reserved_byte | ME-2 | me | me-cli | - | now=0 | built=1
- Rule as §14.10 (`reserved byte must be 0, got 6`). GUARD: green today. Mutant: v2 parsing leaking into v1.
ROW ms1_letter_is_the_top_nibble_of_the_entropy | ME-2 | me | me-cli | - | now=0 | built=1
- Rule as §14.10 (`A16`→A … `P`→P; the N-share string, a plain BIP-93 seed, a non-`0x00` prefix → `None`). RED:
  absent. Mutant: reading the prefix byte (every letter A).
ROW v2_vectors_match_the_independent_generator | ME-2 | me | me-cli | - | now=0 | built=1
- Rule as §14.10, over all 18 vectors (§14.9 plus the two below). RED: absent. Mutant: any layout, KDF-input or AAD byte.
ROW v2_open_round_trips_each_positive_vector | ME-2 | me | me-cli | - | now=0 | built=1
- Rule as §14.10 (V2-C sealed hash `a26ed22bb747dfd0236706ad14c19679`). RED: absent. Mutant: titles left in `secret`.
ROW v2_password_normalises_like_plates_key | ME-2 | me | me-cli | - | now=0 | built=1
- Rule as §14.10. RED: absent. Mutant: byte-exact password compare.
ROW v2_refuses_a_wrong_word_count_before_the_kdf | ME-2 | me | me-cli | - | now=0 | built=1
- Rule as §14.10 (`PasswordWordCount { got: 5, want: 6 }` in under 200 ms). RED: absent. Mutant: KDF before the count.
ROW v2_refuses_a_word_off_the_list_before_the_kdf | ME-2 | me | me-cli | - | now=0 | built=1
- Rule as §14.10 (`PasswordWord(6)`). RED: absent. Mutant: no list check.
ROW v2_pw_form_0_keeps_the_checksum | ME-2 | me | me-cli | - | now=0 | built=1
- Rule as §14.10. RED: absent. Mutant: checksum by version, not by `pw_form`.
ROW v2_wrong_password_fails_authentication | ME-2 | me | me-cli | - | now=0 | built=1
- Rule as §14.10. RED: absent. Mutant: partial plaintext on failure.
ROW v2_titles_refusals | ME-2 | me | me-cli | - | now=0 | built=1
- Rule as §14.10: each N vector gives the variant and Display of §14.9's last column, `Opened` never returned.
  **plan leg (owed row `s1br3-nothex-nonms1-vectors`, S1b r3 I-2):** vectors N-not-hex (`pw_form` 6, 600,000,
  salt `0bad`×8, IV `0c0e`×6, encrypted `A16` then `text:zz`) → `NotHex`, `the titles record is not text: followed
  by lower-case hex of UTF-8`, T-1; and N-mnemonic (same, IV `0c0f`×6, encrypted `abandon`×11 `about` then titles
  `SEED A`) → `NonMs1Secret(0)`, `titles apply only to ms1 entr records; secret record 0 (records count from 0) is
  not one`, T-1. Their values, computed by the S1b generator with these two builds added (the gate recomputes them):
  shape rows `| N-not-hex | 6 | 600000 | 0 | 58 | 2 | 126 |` and `| N-mnemonic | 6 | 600000 | 0 | 161 | 2 | 229 |`;
  key `c89a222034c1c198d154757ff218c1fe6faa97174ac575493b121fd64d5c8d75` (both); N-not-hex tag
  `f00d80f68d3a83fbdef6c4f22c0e08b0`, blob sha256
  `d2b31db4bc5be9f72ca44431bbcbb1ae9e0d6bfccd068d77e9fdd902aa6aa472`, header
  `4d4e454d424c4f4202010106000927c00bad0bad0bad0bad0bad0bad0bad0bad0c0e0c0e0c0e0c0e0c0e0c0e000000000000003a`;
  N-mnemonic tag `e9cc62cfdadc2f266d8e4007119e1ead`, blob sha256
  `916a9c96ff03cd6b62688d1297adda8d31998454c54e521fb08619f7d562b54c`, header
  `4d4e454d424c4f4202010106000927c00bad0bad0bad0bad0bad0bad0bad0bad0c0f0c0f0c0f0c0f0c0f0c0f00000000000000a1`.
  State at the RED: the generator lacks both (its `--check` reports "checked 16 vectors"); the leg is appended to the row's
  TRACE Rule cell in S1b §14.10 by the RED commit. Mutants: dropping check 3 (N-not-hex then gives some variant other than
  `NotHex`) and dropping check 8 (N-mnemonic then opens: `Opened` is returned, the mnemonic counted as a seed).
ROW v1_open_treats_text_records_as_records | ME-2 | me | me-cli | - | now=0 | built=1
- Rule as §14.10. RED: absent. Mutant: titles parsing applied to v1.
ROW seal_v2_refuses_what_open_refuses | ME-2 | me | me-cli | - | now=0 | built=1
- Rule as §14.10 (N-count, N-letter, N-hot-letter, and N-mnemonic by this plan's leg; N-not-hex is reader-only:
  `seal_v2` builds the titles record through `encode_titles` and cannot emit non-hex). RED: absent.
  Mutant: a writer that skips checks 8-10.
ROW encode_titles_refuses_empty_and_unknown_titles | ME-2 | me | me-cli | - | now=0 | built=1
- Rule: `encode_titles(&[])` is `Err(Empty)`; `encode_titles(&["SEED A", "SEED Q"])` is `Err(Line(3))`;
  `encode_titles(&["SEED A"])` is `text:726566756769756d2d706c6174657320320a7469746c65205345454420410a` (the
  generator's `titles_record(["title SEED A"])`). RED: absent. Mutant: `Line(i + 1)`.
ROW me_seal_still_writes_version_1 | ME-2 | me | me-cli | - | now=0 | built=1
- Rule as §14.10 (UF2 payload byte 8 = `0x01`, byte 11 = `0x00`). GUARD: green today. Mutant: `me seal` switched to v2.

### SH-1
ROW TestValidMR | SH-1 | sh | ./codex32/ | - | now=0 | built=1
- Rule: `ValidMR` is true for `single-card`, `chunk-0`, `chunk-1` and false for `checksum`; true for a 96-character
  93-symbol string and false at 94 symbols. RED: undefined. Mutant: the mt1 target constant kept.
ROW TestSyswMrStringVectors | SH-1 | sh | ./sysw/ | - | now=0 | built=1
- Rule: S1 test 24 in Go, from the vendored file. RED (probe run 2026-11-22, `scripts/plan-gate-fork-refugium-kit.sh
  probes`): `Classify(single-card)` is `ClassUnknown`. Mutant: case check after BCH.
ROW TestSyswMrSetVectors | SH-1 | sh | ./sysw/ | - | now=0 | built=1
- Rule: S1 test 25 in Go (verdict, reason, detail). RED: absent. Mutant: `SET_MISSING` before `SET_DUPLICATE`.
ROW TestSyswMrKitVectors | SH-1 | sh | ./sysw/ | - | now=0 | built=1
- Rule: S1 test 26 in Go (verdict, reason, detail, cards). RED: absent. Mutant: public section only (a kit's
  secret-section record not checked).
ROW TestSyswMDMKGroupsByRuns | SH-1 | sh | ./sysw/ | - | now=0 | built=1
- Rule: the four records of `dupmk` (as ME-1) confirm as two complete groups; F1, F2 and F4 read the same grouping
  (S1 §14d). RED: all four unconfirmed. Mutant: grouping by (hrp, chunk-set id).
ROW TestSyswMrVectorsProvenance | SH-1 | sh | ./sysw/ | - | now=0 | built=1
- Rule: the vendored file's SHA-256 equals its provenance entry, which names an `me` commit (D4 (a)). RED: absent.
  Mutant: a hand-edited vector.
ROW TestAdmitMrAtBundleOnly | SH-1 | sh | ./gui/ | - | now=0 | built=1
- Rule: `admitted(progBundle, ClassMr)` and no other program admits `ClassMr`; `seal.Classify` of `single-card` is
  not permitted in either section (S1 test 28). RED: absent. Mutant: `ClassMr` added to `progWalletPolicy`.
ROW TestTxClassNameMr | SH-1 | sh | ./gui/ | - | now=0 | built=1
- Rule: `txClassName(sysw.ClassMr)` is "mr1 string". RED: absent. Mutant: default "unrecognised record".

### SH-2 (device texts are S1 §14e/§14g, quoted in the coverage map)
ROW TestKitMenuOffersEngraveKit | SH-2 | sh | ./gui/ | - | now=0 | built=1
- Rule: loaded `kit-2of3`: menu title "Load Payload", lead "Engrave kit now (6 cards, 6 plates)?", choices
  ENGRAVE KIT, LOAD AGAIN, UNLOAD; `kit-single`: "Engrave kit now (1 card, 1 plate)?"; ENGRAVE KIT opens the kit
  list; the same at boot (`syswLoadFlow` atBoot) and on a second opening (S1 test 29, met). RED: lead is "Loaded.
  It holds: …". Mutant: "1 cards".
ROW TestKitMenuRefusedShowsReasonThenMenu | SH-2 | sh | ./gui/ | - | now=0 | built=1
- Rule: each refused kit vector shows the screen "Kit Refused" with its device sentence, a blank line and "Nothing
  from this payload can be engraved as a kit. Write the kit payload again.", then today's menu; e.g. `kit-text`:
  "Record 3 of this payload is not an md1, mk1 or mr1 string (it is: free text), so this payload is not a kit.";
  `kit-upper`: "Record 1 of this payload is an mr1 string with capital letters; the mr1 strings of a kit are all
  lowercase, as mr prints them."; `kit-mk1-split`: "The mk1 card that begins mk1qppc9 is split: record 4 of this
  payload sits between its strings but is not part of it. Keep each card's strings together." (S1 test 29,
  violated). RED: absent. Mutant: `me`'s record form ("record 2, as given") on the device.
ROW TestKitMenuNoMr1Unchanged | SH-2 | sh | ./gui/ | - | now=0 | built=1
- Rule: `no-mr1` shows today's menu (lead "Loaded. It holds: …", no ENGRAVE KIT, no Kit Refused), on first and
  second opening. GUARD: green today. Mutant: the kit offer keyed on `ClassMDMK`.
ROW TestKitRefusedAfterWarnings | SH-2 | sh | ./gui/ | - | now=0 | built=1
- Rule: fixture `kit-with-ms1` (plaintext: `kit-single`'s mr1, then `A16`): at load, Payload Warnings (F1's line),
  KEEP → "Kit Refused" with "Record 2 of this payload is not an md1, mk1 or mr1 string (it is: codex32 secret), so
  this payload is not a kit." → the menu; UNLOAD → no Kit Refused. RED: absent. Mutant: Kit Refused before the
  warnings.
ROW TestKitBundleDoor | SH-2 | sh | ./gui/ | - | now=0 | built=1
- Rule: Engrave Bundle with eligible kit opens "Engrave Kit" at once (no "Cards from where?"); refused shows Kit
  Refused then the carousel; no payload or `no-mr1`: the gather screen as today. RED: today the gather screen in
  all three. Mutant: door opened for `no-mr1`.
ROW TestKitListEntries | SH-2 | sh | ./gui/ | - | now=0 | built=1
- Rule: `kit-2of3` list title "Engrave Kit", first line "6 cards, 6 plates. Start at:", entry 1 "Card 1 of 6: mr1,
  begins mr1lg2c0, ends 06uf", entry 2 "Card 2 of 6: mk1, begins mk1qppc9, ends 0uvz"; " (cut)" after a card's
  plates are all cut, gone after LOAD AGAIN; highlight on card 1 from the menu, from Engrave Bundle and after a
  run's end, on the stopped card after a Back stop; Back returns to the carousel. RED: absent. Mutant: highlight
  reset to 1 after a stop.
ROW TestKitDoorRun | SH-2 | sh | ./gui/ | - | now=0 | built=1
- Rule (S1 test 30): start at card 2 of `kit-2of3`: cards 2-6 cut in order; each choice screen title "Card k of 6
  | Plate 1 of 1", lead "Choose engraving", one choice TEXT ONLY; each plate's title "CARD k/6", footer "PLATE
  1/1"; end screen "Engrave Kit" "Cards 2 to 6 of this kit are cut."; start at card 6: "Card 6 of this kit is
  cut."; an engrave that does not complete shows the same choice screen; Back at p = 1 returns to the list with no
  other screen; with the §7 one-string-per-plate seam on a 2-string card, Back at plate 2 shows "Stopped before
  plate 2 of 2 of card k of 6. To finish this card, choose card k in the list again: it starts again at its plate
  1."; no ms1 reminder. RED: absent. Mutant: marks taken from the gather flow's title.
ROW TestKitDoorPlateDoesNotFit | SH-2 | sh | ./gui/ | - | now=0 | built=1
- Rule: with the §7 layout seam failing, "Engrave Kit" shows "Card 1 of 6, plate 1 of 1 does not fit on a plate,
  so this kit cannot be engraved on this machine." and returns to the list. RED: absent. Mutant: a silent skip.
ROW TestKitLeadPlatesEqualCuts | SH-2 | sh | ./gui/ | - | now=0 | built=1
- Rule (S1 test 32, D4 (d)): for every eligible kit vector, M in the lead equals the plates cut when the run starts
  at card 1 and runs to the end (6, 1, 4, 4 measured with today's packer, probe 2026-11-22). RED: absent. Mutant:
  a planner that ignores the marks.
ROW TestKitSentencesDevice | SH-2 | sh | ./gui/ | - | now=0 | built=1
- Rule: every §14g reason renders with the device placeholders (`Record <n> of this payload`, `record <n> …`,
  `records <n1> and <n2> of this payload`, `<n>` from 1, public section first) over every refused string, set and
  kit vector. RED: absent. Mutant: `<n>` counted from 0.
ROW TestRefugiumSecretWarningLine | SH-2 | sh | ./gui/ | refugium | now=0 | built=1
- Rule (S1 test 31): plaintext payload with a secret: the line "Refugium build: secrets travel only in the Sealed
  Payload, never in this one. Unload this payload unless you mean to engrave its secret now." directly after "A
  SECRET is stored unencrypted in flash."; sealed with a secret (S-D): the last line (alone with no flag); sealed with F2 raised (S-E): F2's line,
  then the §14f line as the screen's last line; KEEP / UNLOAD follow; an unconfirmed md1/mk1 alone: no line. RED:
  absent. Mutant: line keyed on `IsSecret` (an unconfirmed md1/mk1 would raise it); the line placed before F2's.
ROW TestDefaultBuildNoRefugiumWarning | SH-2 | sh | ./gui/ | - | now=0 | built=1
- Rule: the untagged build never shows the §14f line for the same payloads. GUARD: green today. Mutant: line not
  profile-gated.

### SH-3 (Go parity of S1b §14.10; Go names are the Rust names in CamelCase)
ROW TestV2HeaderRoundTripsEachPasswordForm | SH-3 | sh | ./seal/ | - | now=0 | built=1
- Rule: V2-A's header parses to `Version 2, PwForm 6, Iterations 600000, PubLen 0, CtLen 242`; 0, 6, 12 encode and
  parse equal. RED (probe run 2026-11-22): `seal: unsupported format version 2`. Mutant: `buf[8] != Version`.
ROW TestV2RefusesPwFormOutside0And6To12 | SH-3 | sh | ./seal/ | - | now=0 | built=1
- Rule: offset 11 = 1, 5, 13, 128, 255 → `errors.Is(err, ErrPwForm)`, text "seal: password form byte must be 0 or 6
  to 12, got <b>"; offset 8 = 0 or 3 → `ErrUnknownVersion`. RED: absent. Mutant: `<= 12` only.
ROW TestV2RefusesNWordPayloadBelow600000Iterations | SH-3 | sh | ./seal/ | - | now=0 | built=1
- Rule: N-floor-599999 → `errors.Is(err, ErrIterations)`, text "seal: iteration count out of range: 599999 is below
  600000 for a 6 to 12 word password" (the existing sentinel's text, then ": ", then §14.2's Go format); every
  §14.9 floor edit as stated there. RED: absent. Mutant: floor for `pw_form` 0.
ROW TestV2PublicOnlyRefusesNonzeroPwForm | SH-3 | sh | ./seal/ | - | now=0 | built=1
- Rule: `errors.Is(err, ErrUnsealedFieldNotZero)`, wrapped ": pw_form". RED: absent. Mutant: unsealed branch skips byte 11.
ROW TestV1StillRefusesANonzeroReservedByte | SH-3 | sh | ./seal/ | - | now=0 | built=1
- Rule: v1 header with offset 11 = 6 → `ErrReservedNotZero`, ", got 6". GUARD: green today. Mutant: v2 parsing in v1.
ROW TestMS1LetterIsTheTopNibbleOfTheEntropy | SH-3 | sh | ./seal/ | - | now=0 | built=1
- Rule: as the Rust row; the share and a non-`0x00` prefix → not ok. Wipe (§7 `ms1LetterEntropyHook`; secret rows
  `s1b-r2-letter-check-materialise`, `plankits2-r1-s1-wipe-hook-vacuous`): the hook fires exactly once per
  `ms1Letter` call in which `DecodeMS1` returns entropy and never otherwise: once each for `A16`, `A` … `G`, `P` and
  the row's `0x02` payload, never for the N-share string (the threshold digit is tested first). Also a `0x02`
  payload with non-zero entropy, `ms10entrsqgqpzyg3zyg3zyg3zyg3zyg3zyg3zzkvk8nkwsv6rl` (`DecodeMS1`: prefix 2,
  language 0, 16 × `0x11`; not ok): the hook fires once, the copy is 16 × `0x11`, and the kept slice is zero after
  `ms1Letter` returns (the row's Rust-mirrored `0x02` payload is all-zero while live). The hook is called with
  `ms1Letter`'s own entropy slice (not a copy) while that slice still holds the secret; the test keeps the slice
  itself and a copy of its contents taken inside the hook, asserts the copy equals the vector's entropy bytes
  (`B`: 32 × `0x11`, `P`: 32 × `0xff`) and, after `ms1Letter` returns, that every byte of the kept slice is zero.
  The copy has `len` 16 for `A16` and for the `0x02` payload, and 32 for `A` … `G` and `P`. A hook call count of 0
  for any of those vectors fails: a nil capture is not "all zero", and `A16`, `A` and the row's Rust-mirrored
  `0x02` payload are all-zero while live, so `B` … `G`, `P` and the non-zero `0x02` payload carry the wipe proof.
  RED: absent. Mutants: no `clear`; hook not called; hook called with a copy (`bytes.Clone(entropy)`); return on a
  non-`0x00` prefix before `clear`.
ROW TestV2VectorsMatchTheIndependentGenerator | SH-3 | sh | ./seal/ | - | now=0 | built=1
- Rule: every vendored vector's header, key, tag and blob SHA-256 from the Go code; the file's SHA-256 equals its
  provenance entry (D4 (a)). RED: absent. Mutant: AAD without the public section.
ROW TestV2OpenRoundTripsEachPositiveVector | SH-3 | sh | ./seal/ | - | now=0 | built=1
- Rule: as the Rust row through `Opener.Open`; `Payload.Titles` nil for V2-C; the titles record is not in
  `Payload.Secret`. RED: absent. Mutant: titles left in `Secret`.
ROW TestV2PasswordNormalisesLikePlatesKey | SH-3 | sh | ./seal/ | - | now=0 | built=1
- Rule: `Opener.Open(V2-A, "  BEEF\tbeef beef  Beef beef\nbeef ")` opens. RED: absent. Mutant: no normalisation.
ROW TestV2WrongPasswordFailsAuthentication | SH-3 | sh | ./seal/ | - | now=0 | built=1
- Rule: `beef`×5 `bacon` fails authentication; nothing in `Payload.Secret`. RED: absent. Mutant: partial plaintext.
ROW TestV2TitlesRefusals | SH-3 | sh | ./seal/ | - | now=0 | built=1
- Rule: every N vector of §14.9 plus this plan's two: the `TitlesError` check and its Rust Display text; N-share →
  `NonMs1Secret(0)` in Go; after the error every record of `Payload.Public` and `Payload.Secret` is zero.
  RED: absent. Mutant: any one of checks 1-10 dropped (one vector each).
ROW TestV1OpenTreatsTextRecordsAsRecords | SH-3 | sh | ./seal/ | - | now=0 | built=1
- Rule: N-v1-text refused as a record (`ErrRecordNotPermitted`, index 1). RED: absent. Mutant: titles on v1.
ROW TestV2RecordBudget | SH-3 | sh | ./seal/ | - | now=0 | built=1
- Rule (D4 (b)): 18 × `A16` with a titles block of 18 `title SEED A` lines (19 records; titles record 509 bytes)
  opens; 19 × `A16` with 19 such lines (titles record 535 bytes) is refused `ErrRecordTooLong`; 24 × `A16` with no
  titles record opens; 24 × `A16` plus any titles record (25 records) is refused `ErrTooManyRecords`. RED: absent
  (v2 refused at the version). Mutant: the titles record not counted toward 24.
ROW TestBoundBlobReadsV2 | SH-3 | sh | ./seal/ | - | now=0 | built=1
- Rule (D4 (c)): `boundBlob` over a 65,536-byte region holding V2-B returns 831; the largest legal v2 blob returns
  16,450; no allocation in `seal/` is sized by `RegionLen` (`testing.AllocsPerRun` bound as F-79 fix C's test).
  RED: today `boundBlob` refuses V2-B at the version. Mutant: `ParseHeader` still v1-only.

### SH-4 (device texts are S1b §14.4-§14.6, quoted in the coverage map)
ROW TestUnlockNWordNotice | SH-4 | sh | ./gui/ | - | now=0 | built=1
- Rule: V2-A shows "Enter the 6-word plates password for this payload.", a blank line, "These words are the
  payload's password. They are NOT a seed and no wallet is derived from them." once per session; V2-C shows
  today's notice. RED: absent (ParseHeader refuses v2). Mutant: "passphrase" kept.
ROW TestUnlockNWordEntry | SH-4 | sh | ./gui/ | - | now=0 | built=1
- Rule: V2-B with `zoo`×12 typed into 12 slots unlocks (no checksum gate); V2-A with 6 slots ends entry at slot 6
  (no `done`); Back after 3 words leaves the program with no KDF (deriver fake not called). RED: absent. Mutant:
  `m.Valid()` kept for `pw_form` 12.
ROW TestUnlockNWordFirstFrame | SH-4 | sh | ./gui/ | - | now=0 | built=1
- Rule: first frame "Unlocking. This takes about 80 seconds." for 600,000 and "Unlocking. This takes about 90
  seconds." for 600,001 (S = 10 × ceil(iterations / 75,000)); a v1 or `pw_form` 0 payload keeps "Unlocking. This
  takes about 30 seconds." RED: absent. Mutant: floor instead of ceil (600,001 would read 80).
ROW TestUnlockNWordWrongPasswordBody | SH-4 | sh | ./gui/ | - | now=0 | built=1
- Rule: V2-A with a wrong word: "Wrong password, or this payload has been altered." then "Check the words and try
  again." (no public section), then entry again without the notice; V2-C keeps "Wrong passphrase, …". RED: absent.
  Mutant: notice repeated.
ROW TestUnlockNWordPasswordBufferZeroed | SH-4 | sh | ./gui/ | - | now=0 | built=1
- Rule: the password bytes handed to the deriver fake have capacity 128 and are all zero after success, wrong
  password and Back. RED: absent. Mutant: password built with `strings.Join`.
ROW TestUnlockV2TitlesRefusalT1 | SH-4 | sh | ./gui/ | - | now=0 | built=1
- Rule: each T-1 vector (N-count, N-magic, N-two, N-not-last, N-title-set, N-no-final-lf, N-empty, N-share,
  N-not-hex, N-mnemonic) shows "This payload's plate titles do not match its seed records. Nothing was opened."
  under "Sealed Payload", then the menu; no plate offered; every record zero. RED: absent. Mutant: fall back to
  the id title.
ROW TestUnlockV2TitlesRefusalT2 | SH-4 | sh | ./gui/ | - | now=0 | built=1
- Rule: N-letter "This payload titles a plate SEED C, but that seed's letter is B. Nothing was opened.";
  N-hot-letter "This payload titles a plate HOT SEED F, but that seed's letter is G. Nothing was opened." RED:
  absent. Mutant: letter check only on `SEED <L>`.
ROW TestUnlockV2TitleLeadLine | SH-4 | sh | ./gui/ | - | now=0 | built=1
- Rule: V2-A plate 1: title "ms1 1/2", lead "title SEED A" then "SECRET seed material"; plate 2 lead "title KEEP;
  DO NOT LOAD"; V2-C: lead unchanged. RED: absent. Mutant: screen title replaced by the title.
ROW TestUnlockV2TitleOnSteel | SH-4 | sh | ./gui/ | - | now=0 | built=1
- Rule: default build, V2-A plate 1 is `EngraveSeedString` with Title "SEED A"; V2-C plate titled "ENTR". RED:
  absent. Mutant: `unlockCodex32Plan` keeps the id.
ROW TestRefugiumSealedMS1SeedQR | SH-4 | sh | ./gui/ | refugium | now=0 | built=1
- Rule: Refugium build, V2-A plate 1 equals `backup.EngraveSeedStringSeedQR(params, {Title "SEED A", Seed A16,
  MasterFingerprint 0}, the 12 words of A16's entropy)`; plate 2 (P) the 24-word SeedQR titled "KEEP; DO NOT LOAD";
  a v1 payload's ms1 (vector C) gets the SeedQR titled with its id. RED: `TestRefugiumUnlockCodex32PlateIsTextOnly`
  passes today (probe run 2026-11-22); also the upper-case `A16` (`MS10ENTRSQQQQQQQQQQQQQQQQQQQQQQQQQQQQCJ9SXRAQ34V7F`) gets the SeedQR (it cuts today; the id compare is case-insensitive, §5). Mutant: SeedQR of the ms1 string; an id compare of `== "entr"` (the upper-case `A16` goes text only).
ROW TestRefugiumSealedMS1TextOnlyFallback | SH-4 | sh | ./gui/ | refugium | now=0 | built=1
- Rule: a v1 payload holding the N-share string or a plain BIP-93 seed → `EngraveSeedStringTextOnly` titled with
  its id; a titled v2 payload holding A20 = `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqke34pjrgn5p6k` (`ms encode
  --hex -` of 20 zero bytes, ms 0.20.4) titled `SEED A` → text only titled "SEED A"; no plate holds a QR of an ms1
  string; also `ms12entraqqqqqqqqqqqqqqqqqqqqqqqqqqqqmzgn0vnkuflf8` (2-of-n share, prefix 0x00) titled by its id `entr` and
  `ms10testsqqqqqqqqqqqqqqqqqqqqqqqqqqqqpep4hndkvqa6p` titled `test`, both text only with no QR and never "This record does
  not fit any plate size."; also `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqppf5cx6yeww5we` (A16 with the low padding bit of its last payload character set, checksum recomputed:
  checksum-valid, `DecodeMS1` ok with 16 bytes, `ParsePrefix` unshared `entr`, but not equal to `codex32.EncodeMS1` of its
  entropy, which `EngraveSeedStringSeedQR` refuses with `errSeedQRDisagree`) titled `entr`, and its upper-case form, each text
  only with no QR and never "does not fit"; also the 24-byte `entr` string `ms10entrsqqqsyqcyq5rqwzqfpg9scrgwpugpzysnzs23v9ccruln67vrlr6dg`
  (entropy bytes 0x01..0x18, `codex32.EncodeMS1`; `DecodeMS1` prefix 0, 24 bytes; `ParsePrefix` unshared `entr`) titled `entr`, text only, no QR (a 24-byte entropy would otherwise reach an 18-word SeedQR). RED: absent. Mutant: SeedQR for 20-byte entropy; no `ParsePrefix` gate (the share
  and `test` strings reach `EngraveSeedStringSeedQR` and show "does not fit"); "ParsePrefix gate only" (no handling of
  `errSeedQRDisagree`: the padding-bit string shows "does not fit"); no length gate: 24 bytes gets an 18-word SeedQR.
ROW TestRefugiumSealedMS1WipePoints | SH-4 | sh | ./gui/ | refugium | now=0 | built=1
- Rule (§5): the entropy slice and the `bip39.Mnemonic` handed to the §7 hooks are all zero before the engrave
  screen's job starts; `rec` is zero as today. Each hook (`unlockSeedQREntropyHook`, `unlockSeedQRWordsHook`) fires
  exactly once per `unlockCodex32SeedQRPlan` call, on the success and the error path alike, with the plan's own
  slice (not a copy) while it still holds the secret; the test keeps the slice itself and a copy of its contents
  taken inside the hook, asserts the copy equals the vector and, after the call returns, that the kept slice is
  zero. V2-A: each hook
  fires twice; the entropy copies have `len` 16 (`A16`, 16 × `0x00`) and 32 (`P`, 32 × `0xff`), the mnemonic copies
  `len` 12 ("abandon" × 11, "about") and 24 ("zoo" × 23, "vote"). This row also runs the `A16` padding-bit
  string of `TestRefugiumSealedMS1TextOnlyFallback` (refused by `EngraveSeedStringSeedQR`): each hook fires once,
  `len` 16 and 12, the same contents as `A16`'s. Also `P` with its padding bit set,
  `ms10entrsqrlllllllllllllllllllllllllllllllllllllllllllllllllllak6svw5c6lqsf` (checksum recomputed; `DecodeMS1` ok,
  32 bytes of `0xff`; not equal to `codex32.EncodeMS1` of its entropy, so `EngraveSeedStringSeedQR` refuses it):
  each hook fires once, `len` 32 and 24, the contents of `P`'s. After each
  `unlockCodex32SeedQRPlan` call returns, every kept byte is 0 and every kept word is 0 (the zero `bip39.Word`, "abandon"). A hook call count of 0 fails: a nil capture is not
  "all zero", and `A16`'s entropy is all zero while live, so `P` carries the wipe proof on the success path
  and `P`'s padding-bit string on the refusal path. RED: absent. Mutants: `clear(m)` after `Engrave` (resident
  through the ~21-minute cut); hook not called; hook called with a copy; no `clear(entropy)` on the refusal path.
ROW TestDefaultBuildSealedMS1Unchanged | SH-4 | sh | ./gui/ | - | now=0 | built=1
- Rule: untagged build, vector C's ms1 plate equals `EngraveSeedString` (text + QR of the string), titled with its
  id. GUARD: green today. Mutant: SeedQR not profile-gated.
ROW TestUnlockHeaderRefusalsUnreadable | SH-4 | sh | ./gui/ | - | now=0 | built=1
- Rule: a version-3 header and N-floor-599999 each show "Payload unreadable." with no notice and no KDF (deriver
  fake not called). GUARD: both read "Payload unreadable." today (version 2 is refused); the row keeps the floor
  case unreadable once v2 opens. Mutant: the floor checked after the prompt (the notice would show).

### SH-5 (walks; `scripts/emu-kit-walk.sh <walk> <build>` prints the walk's JSON and exits 0 only on `ok:true`)
WALK kit-default | SH-5 | `scripts/emu-kit-walk.sh kit default` | boot → Load Payload → "Engrave kit now (6 cards, 6 plates)?" → ENGRAVE KIT → card 1 → six plates cut, each "CARD k/6" / "PLATE 1/1" in the plate recorder → "Cards 1 to 6 of this kit are cut." → list shows six " (cut)"
WALK kit-refugium | SH-5 | `scripts/emu-kit-walk.sh kit refugium` | the same in the Refugium build, and no Refugium warning line (no secret)
WALK v2-default | SH-5 | `scripts/emu-kit-walk.sh v2 default` | Sealed Payload → the 6-word notice → `beef`×6 → "Unlocking. This takes about 80 seconds." → "ms1 1/2" lead "title SEED A" → Cut → plate recorder title "SEED A" with a QR of the ms1 string (default build)
WALK v2-refugium | SH-5 | `scripts/emu-kit-walk.sh v2 refugium` | the same, the plate a Standard SeedQR of the 12 words, no QR of the ms1 string
ROW TestEmuKitPayloadProvenance | SH-5 | sh | ./cmd/emu/ | - | now=0 | built=1
- Rule: the embedded kit blob's SHA-256 and its `me sysw pack` provenance line; its digest equals `me sysw show`'s.
  RED: absent. Mutant: a stale blob.
ROW TestEmuSealedV2PayloadProvenance | SH-5 | sh | ./cmd/emu/ | - | now=0 | built=1
- Rule: the embedded v2 blob's SHA-256 equals V2-A's `blob_sha256` (`0e0dc8706b48d1522aaca3a90af983954e6d287284035977181029f94cbac8b2`).
  RED: absent. Mutant: a re-sealed blob with another salt.

### SH-6 (hardware; SeedHammer II, key 846aa289 build, signet test material only)
HW H-v2 | SH-6 | S1b §14.8 exactly: V2-B at `0x10E00000` and the `kit` blob at `0x10D00000` in one power cycle, PD power; unlock `zoo`×12; cut `SEED A`; idle to the 3:00 warning and the 3:30 wipe; re-unlock; Skip to plate 2 and cut it | both unlocks 80 ± 10 s wall (record the figure), wipe at 3:30 to the menu, second unlock offers "ms1 1/7" first, both plates titled with a Standard SeedQR, free heap logged before and after each unlock, no restart
HW H-size | SH-6 | the firmware size recipe (this repo's CLAUDE.md) with `-tags refugium`, at the baseline and at the SH-4 head | static `ram` rises by at most 1,024 bytes; both figures recorded (the titles buffer is ≤ 253 bytes, the password buffer 128)

## 4. Owed-to-build and FOLLOWUPS closures

- `s1br3-nothex-nonms1-vectors` (owed-to-build, owner mnemblob-v2): closed by ME-2's RED commit (the leg of
  `v2_titles_refusals`) and its GREEN; its FOLLOWUPS line (direct-pushed after S1b merges) is flipped `resolved`
  in that GREEN commit.
- mnemonic-refugium `w6-plates-device-path` and `w6-plates-title-device`: their device halves (the plates payload
  is a v2 Sealed Payload at `0x10E00000`; the title on steel) are met by SH-3/SH-4; the host halves are S3's. This
  plan proposes their closing lines in its report; it edits no FOLLOWUPS file.
- F-699 (`mr-gui-e1-mr1-class`): closed by ME-1 (S1 §14a: no injected validator).
- F-702 lane F F1: met as D5 says.

## 5. Wipe points (S1b §14.6 "Spec S2 names the functions"; secret rows kitsealx-*, s1b-*)

- `unlockEngraveCodex32(rec)` (Refugium build) routes on the SeedQR outcome, not on `codex32.ParsePrefix` alone (Brian ruling A
  2026-11-22: "DecodeMS1-ok but SeedQR-refused ms1 strings cut as text only"). A string that is not an unshared `entr` string (threshold `0`, share index `s`, id `entr`, the id compared case-insensitively:
  mixed case is already refused by `ParsePrefix`; S1b §14.6, S1b head 844572cd), a string whose `DecodeMS1` fails, whose prefix
  is not `0x00` or whose entropy is not 16 or 32 bytes (a 20-byte entropy would otherwise reach a 15-word SeedQR), goes to
  `engraveSeedStringPlate` -> `EngraveSeedStringTextOnly`, titled by `<T>` or the codex32 id, never "does not fit". Otherwise it calls a new `unlockCodex32SeedQRPlan(params, s, title)`:
  `codex32.DecodeMS1` → entropy (a view into `s.Seed()`'s fresh slice) → `bip39.New(entropy)` → `m`;
  `backup.EngraveSeedStringSeedQR(params, {Title, Seed, Font, MasterFingerprint 0}, m)`.
  Any string for which `backup.EngraveSeedStringSeedQR` returns a non-nil error, whatever the error (no sentinel is named or
  exported; this includes the re-encode check, bytes not equal to `codex32.EncodeMS1` of the entropy in either case), also goes
  to text only: on that error the plan zeroes `entropy` and `m`, then calls `engraveSeedStringPlate`; if that also fails the
  screen shows today's `This record does not fit any plate size.` (text-only layout failure). A SeedQR refusal itself is never
  reported as 'does not fit'. **ZEROED** before the
  plan returns: `entropy` (and with it the decoded seed bytes after the prefix byte) and `m`; **ZEROED** before
  `Engrave` as today: `rec`. **LIVE**, listed in the HONEST CAVEAT block: the `codex32.String` and `s.String()` /
  `SeedString.Seed` Go strings (the existing caveat), `bip39.New`'s checksum hash state, the SeedQR digit string
  and `qr.Code.Bitmap` (F-88), `plate.Spline` during the cut (F-83). No BIP-39 seed or BIP-32 key is derived
  (`MasterFingerprint` 0), which closes `s1b-r1-f5-fingerprint-inventory` for the kit path.
- `seal.ms1Letter(rec)`: tests the threshold digit before `DecodeMS1` (a share is never decoded), keeps
  `entropy[0] >> 4`, and clears `entropy` before return, before the next record is examined or any refusal is
  shown (`s1b-r2-letter-check-materialise`). LIVE: the `codex32.New(string(rec))` string (existing caveat).
- The titles block: decoded into a fixed 253-byte buffer, not secret, cleared with the records on a refusal.
- The n-word password: `passphraseBytes(m)` (fixed capacity 128) for n ≤ 12 words; cleared on every exit
  (`unlockAttemptOnce`'s defer), and `m` cleared on Back and after the attempt as today.
- Kept, not closed, by this plan: `kitsealx-q5-multi-seed-residency` (the n−1 not-yet-offered seeds stay
  decrypted in RAM through each cut, accepted EPD §10.2.2 model) and `kitsealx-q13-six-word-plates-password`
  (S1b's). Not touched: `kitarch-a-s1`, `kitarch-b-s1`, `kitarch-c-a1` (S1 warns, nothing refuses; alpha list);
  `kitarch-a-s2`, `kitarch-b-s2`, `kitarch-c-a2`, `kitsynth-s1` (sysw secrets; the kit path no longer routes
  secrets there, D2); `kitarch-c-a3` (S3's).

## 6. Kit door entry and boot routing (S1 §14e; files in SH-2)

The menu (`syswPayloadMenu`) computes the kit verdict once per opening from the loaded session; the boot route is
the existing post-load menu (G-P3.15), so a kit payload is one press (ENGRAVE KIT) from the list at boot. The
Engrave Bundle entry (`bundleFlow`) checks the session before the gather. The verdict, the card list and N, M are
pure functions of the session's records and the packer (`bundleCardPlates`), recomputed on each entry; the " (cut)"
marks live in the session (`syswSession`), cleared by LOAD AGAIN and UNLOAD; nothing is written to flash.

## 7. Test seams

- Payloads: menu, door and list rows load a session with the existing `sessionWith(records...)` helper over the
  vendored kit vectors' records; load-time rows (`TestKitRefusedAfterWarnings`, `TestRefugiumSecretWarningLine`,
  `TestDefaultBuildNoRefugiumWarning`) read `me sysw pack` blobs from `gui/testdata/chain/chain_payloads.json`,
  to which `scripts/gen-chain-fixtures.sh` adds `kit-2of3` and `kit-with-ms1` (both unsealed; the generator pins only unsealed payloads) in SH-2; the sealed cases of
  `TestRefugiumSecretWarningLine` load `syswRegionFor(t, "S-D")` (the line alone, no flag) and `syswRegionFor(t, "S-E")`
  (F2's line, then the §14f line last) from the vendored `sysw/testdata/sysw_vectors.json`;
  v2 blobs reach `unlockPayloadFlow` through the existing `payloadReaderFor` / `payloadReaderFrom` seam
  (`gui/unlock_program_test.go`), built by the `sealVector(…).blob` pattern from the vendored
  `seal_v2_vectors.json`. No test reads flash or the network.
- KDF: the existing `newDeriver` var replaced by a fake that records its password bytes and returns the vector's
  derived key at once (no 600,000-iteration run in `gui` tests); `seal/` tests run the real KDF on the vectors.
- Word entry: the existing `unlockPassphraseWordsHook` fills the n slots; a short fill plus Back exercises the
  leave path.
- Wipes: new package vars `unlockSeedQREntropyHook func([]byte)` and `unlockSeedQRWordsHook func(bip39.Mnemonic)`
  in `gui/unlock_session.go`, and `ms1LetterEntropyHook func([]byte)` in `seal/titles.go`, each nil in production;
  each is called with the function's own slice while it still holds the secret; tests count the calls, keep the
  slice itself and a copy of its contents taken inside the hook, assert the copy equals the vector and the kept slice
  zero afterwards, and a hook that never fired fails the test (the
  `master_key_residue_test.go` pattern, with its "never fired" check).
- Kit door: `kitCardPlates` (a package var defaulting to `bundleCardPlates`) and `kitPlateLayout` (defaulting to
  the TEXT ONLY layout) are the two seams `TestKitDoorRun` (one string per plate) and `TestKitDoorPlateDoesNotFit`
  (layout error) replace; `TestKitLeadPlatesEqualCuts` uses neither. Engraving is the test engraver every bundle
  test uses (`bundle_engrave_test.go`).
- Build tags: Refugium rows are in `//go:build refugium` files run by `scripts/refugium-tagged-test.sh`; their
  default-build twins are untagged.

## 8. Existing tests that change (rule: rewrite in the RED commit of the phase that changes it; delete only when a named new test replaces it)

CHANGE rejects_bad_magic_version_reserved_kdf_and_aead | ME-2 | me | me-cli | - | its `(8, 0x02, "version")` case becomes `(8, 0x03, "version")`
CHANGE assert_every_named_test_is_placed | ME-1 | me | me-cli | - | `HIGHEST` 23 → 32; entries `(24, Where::Unit("sysw_mr_string_vectors"))`, `(25, Where::Unit("sysw_mr_set_vectors"))`, `(26, Where::Unit("sysw_mr_kit_vectors"))`, `(27, Where::Cli)`, and 28-32 as `Where::DeviceUnbuilt("<S1 § and the fork phase>")` (flipped to `Where::Device` by SH-2 step 3)
CHANGE a_valid_bip93_string_is_told_it_is_bip93_and_not_a_constellation_ms1 | ME-1 | me | me-cli | - | `not an md1/mk1/ms1/mt1 string` → `not an md1/mk1/mr1/ms1/mt1 string`
CHANGE a_record_of_no_class_at_all_still_names_the_classifier | ME-1 | me | me-cli | - | the same text change
CHANGE TestRejectsBadMagicVersionReservedKDFAndAEAD | SH-3 | sh | ./seal/ | - | its `{8, 0x02, ErrUnknownVersion}` case becomes `{8, 0x03, ErrUnknownVersion}`
CHANGE TestEverySyswConsumptionSiteNamesAnAdmittedClass | SH-2 | sh | ./gui/ | - | the site table gains `bundle_flow.go` `kitCards` for `progBundle` (the kit door), and the class map gains `"ClassMr": sysw.ClassMr`
CHANGE TestRefugiumUnlockCodex32PlateIsTextOnly | SH-4 | sh | ./gui/ | refugium | deleted; replaced by `TestRefugiumSealedMS1SeedQR` and `TestRefugiumSealedMS1TextOnlyFallback` (S1b STOP-3: every ms1 gets the SeedQR)

Also re-read in the phase's RED commit, changed only if they pin a changed text (none found by grep at
`25abe8e`): `gui/sysw_payload_menu_test.go` and `gui/sysw_boot_menu_test.go` (SH-2; their payloads hold no mr1,
so they stay green), and the ten test files pinning "SECRET seed material" (`gui/unlock_*`, `run_reentry_test.go`,
`wipe_guard_test.go`, `wipe_inventory_audit_test.go`; SH-4; the lead keeps that line). No golden is re-blessed by this plan.

## 9. G-REL — the F-124 / F-123 release gate (Brian 2026-11-22, round 2)

No release other than an alpha, of a fork build carrying this plan's SH-3/SH-4 code, is tagged while this repo's
FOLLOWUPS F-124 or F-123 is OPEN. `scripts/plan-gate-fork-refugium-kit.sh release` exits 1 and prints "G-REL:
F-124 OPEN" / "G-REL: F-123 OPEN" while either is; the release ritual runs it. It fails closed: a missing FOLLOWUPS file exits 2, and only a `**Status:** CLOSED`
entry clears an id (a renamed heading prints "G-REL: <id> not found", exit 1). SH-4's GREEN commit adds to the fork's
CLAUDE.md a "Release" hazard line: before tagging any fork release other than an alpha, run `ME_WT=<mnemonic-engrave
master checkout> <that checkout>/scripts/plan-gate-fork-refugium-kit.sh release`; exit non-zero blocks the tag. F-123's fix (README security
section, EPD §2.2 item 12, sysw spec §11 say the classes differ in behaviour, not in whether funds are safe if
the machine is taken; S1b §14.11 already folds EPD §2.3) is docs. What remedies F-124 on the kit path is STOP-1.

## 10. The gate

`scripts/plan-gate-fork-refugium-kit.sh` modes: `static` (no code fence in this plan; ROW/CHANGE/WALK/HW lines
unique and in a listed PHASE; every coverage-map quote occurs in its spec at the pinned revision; every owner cell
names a row of this plan or reads "not this plan's: …"; the two owed vectors recomputed with the S1b generator
equal §3's values), `counts` (each ROW and CHANGE's `now` count at the baselines; `--built [PHASES]` checks `built`, for the comma-separated phases only when given),
`probes` (the three RED probes against a clean fork checkout), `selftest` (the gate shown RED on four broken
copies, on a missing and a renamed FOLLOWUPS entry for `release`, and on `counts --built ME-1`), `release` (§9, fail-closed). It does not cover: whether a Rule cell is the right behaviour, the rows' RED/GREEN runs
after the baseline (the phase builds run them), the emulator walks, hardware rows, and any spec sentence outside
S1 §14b-§14g and S1b §14.2-§14.8.

Run 2026-11-22 at the baselines (Claude Opus 5.5): `static` 0 failures (75 ROW lines, 149 map lines, 2 owed
vectors recomputed); `selftest` 4 of 4 broken copies RED (rc 1 each); `probes` 3 of 3 RED states seen (`seal:
unsupported format version 2`; `Classify(single-card)` is `ClassUnknown`; `TestRefugiumUnlockCodex32PlateIsTextOnly`
passes); `counts` 82 lines, 0 failures; `counts --built` 76 failures (RED until built); `release` exit 1 ("G-REL:
F-124 OPEN", "G-REL: F-123 OPEN").

## 11. STOPs

- **STOP-1. What remedies F-124 on the kit path, and who plans it?** Brian (round 2) made the fix a gate before
  any non-alpha release but did not say what it covers. (a) A separate F-124 spec and plan, scoped to the kit
  path's sites (F-110's engraver arrays, F-88/F-90/F-104 2b's unwipeable copies on `unlockEngraveCodex32` and
  the SeedQR encoder, F-109), written before the first non-alpha release; this plan carries only the gate (§9).
  (b) Fold that remedy into this plan as a phase SH-7 now, before its R0. **Recommended: (a)**: F-124's items
  reach outside the kit path (the engraver, `LastWordCandidates`) and need their own measurements; the gate keeps
  the release blocked either way.
