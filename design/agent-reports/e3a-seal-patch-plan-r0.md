# R0 review: IMPLEMENTATION_PLAN_e3a_picotool_seal_patch.md (draft 1)

*Independent adversarial R0 review, 2026-10-05. Repo `mnemonic-engrave`, branch
`claude/project-thread-eda5dt` at `7f314e1`. Reviewed: the plan, INV
(`design/agent-reports/e3a-seal-clear-investigation.md`), the patch
(`design/patches/picotool-2.3.1-seal-clear-fix.patch`), `flake.nix`/`flake.lock`,
`design/PICOTOOL_PIN.md`, `.github/workflows/release.yml`, `scripts/sign-firmware.sh`,
`scripts/pico2-bootkey-rehearsal.sh` (R), `design/RUNBOOK_custom_boot_key.md`, and E3a plan §6.4/§7.
External facts were checked against source and by running binaries, not taken from the plan or INV.
No file was modified except this report.*

## What was verified independently (settled facts, do not re-derive)

**Source (picotool tag `2.3.1` = `2041936`, identical to the nixpkgs `src` store path except the
`lib/CMakeLists.txt` mbedtls substitution):**

- `sign_guts_bin` passes `in.get_model()` at `main.cpp:5646`. `in` is the `file_memory_access` from
  `get_file_memory_access(0)`, whose model defaults to `models::unknown` (`main.cpp:2207`). The
  `model` parameter, from `get_model(0)` (`main.cpp:4560`), resolves through `get_access_model`
  (`main.cpp:4467`) to `model_rp2350_arm_s` for an ARM-S IMAGE_DEF. `model_info::sram_end()` is
  `fail(ERROR_NOT_POSSIBLE, "unknown sram end")` (`model/model.h:124`). Bug A is confirmed.
- BIN `get_lm_hash_data` (`bintool.cpp:852`): the no-load-map branch appends the clear size
  (`:870`), then the pin size (`:883`), then does `to_hash.insert(to_hash.begin(), bin…)` (`:886`).
  The entries it writes are `[Clear?, Pin?, Load]`. Bug B is confirmed. After the patch the digest
  is `clear_size ‖ pin_size ‖ bin`, which is exactly entry order.
- **Bootrom order** (`pico-bootrom-rp2350` `c6cdb17`, `varm_blocks.c` ~930-1012): the load map is
  walked entry by entry. A storage-0 entry hashes its size word via `s_varm_sha256_put_word_inc`
  and `continue`s. Other entries hash their storage bytes. Relative storage addresses resolve
  against `block base + load_map_word_index*4`. So the patched order matches the bootrom for
  `--clear`, `--pin-xip-sram` and both together.
- **Independent digest recomputation** (my own Python, not picotool code): I computed
  SHA-256(`0x00082000` LE ‖ flash `0x10000000..0x10005c00` ‖ first 11 block words) on patched
  `seal --hash --clear` output of INV's `blinky.uf2`. It equals the stored HASH_VALUE
  `57300d7d…d160`. The same computation on INV's A-only output (`blinky.pat.clear.uf2`) does
  not match.
- **`info -a` on a FILE is a real verifier.** `verify_block` (`bintool.cpp:980-1044`) recomputes
  the hash through the *existing-load-map* branch, which neither hunk touches. It runs
  `verify_signature_secp256k1` against the embedded public key. It uses `raw_access.get_model()`,
  which `info` sets with `set_model` (`main.cpp:4738`).
- **No upstream fix.** The `develop` head is `ba3df40` (`git ls-remote`). Raw `develop` still has
  `in.get_model(),` at `main.cpp:5807` and `to_hash.insert(to_hash.begin(), bin…)` at
  `bintool.cpp:886`.

**Binaries run** (native stock and patched 2.3.1, nix 2.2.0-a4; throwaway secp256k1 key;
INV's `blinky.uf2`, sha256 `e48659d7…54a9`):

| case | stock 2.3.1 | patched 2.3.1 |
|---|---|---|
| UF2 `seal --sign --clear` | 248 "unknown sram end" | rc 0; Clear+Load; verified (2.3.1 and 2.2.0-a4 `info -a`) |
| BIN `seal --sign --clear` | 248 | rc 0; verified (both) |
| UF2 `--pin-xip-sram` | 248 | rc 0; `Clear 0x13ffc000->0x13ffc000`, Load; verified |
| UF2 `--clear --pin-xip-sram` | 248 | rc 0; 3 entries; verified (both) |
| UF2 `--hash --clear` (no sign) | 248 | rc 0; `hash: verified` |
| UF2 `--hash` / `--sign`, no flags | rc 0 | rc 0; byte-identical for `--hash`; for `--sign`, 64 differing bytes (the randomized signature) |
| reseal an already-sealed UF2 with `--clear` | rc 0, **no Clear entry**, verified, 3 blocks | same |
| sealed output, one payload byte flipped | — | `signature: incorrect` (2.3.1 and 2.2.0-a4) |
| sealed output, Clear size word `0x82000`→`0x81000` | — | `signature: incorrect` (both) |
| A-only output (INV's `blinky.pat.clear.uf2`) | — | `signature: incorrect` |

**Nix (nixpkgs `a7868a72`, `pkgs/by-name/pi/picotool/package.nix`):**

- `patches` evaluates to `[]`.
- `postPatch` only `substituteInPlace lib/CMakeLists.txt` (the mbedtls path). It does not overlap
  the two hunks.
- There is no `prePatch` and no `patchFlags`.
- `src` uses `tag = finalAttrs.version`.

`overrideAttrs (o: { patches = (o.patches or []) ++ [ ./… ]; })`, evaluated:

- `version` is still `2.3.1`, the name is `picotool-2.3.1`, and the drvPath differs from stock;
- `patch -p1 -F0 --dry-run` of the patch against the nixpkgs `src` store path applies cleanly
  (no fuzz);
- `versionCheckHook` (`version`) is unaffected;
- a `forAll (pkgs: assert pkgs.picotool.version == "2.3.1"; { … })` assertion works as described.

**Answers to the brief's questions, in short:**

1. **The patch is correct and complete for UF2/BIN `--clear`.** It also fixes `--pin-xip-sram` and
   UF2/BIN inputs with an existing one-entry generic load map. Output without these flags is
   unchanged (measured).
2. **The CI test can tell patched from unpatched (248), and `info -a` truly verifies.** The
   negative control only discriminates hunk A, though (SP-M3).
3. **The nix mechanics are sound.**
4. **No mechanism lets the fork or a real SeedHammer use the patched picotool, but no written
   gate stops it either** (SP-I2).
5. **Missing gates:** SP-I1, SP-M1, SP-M5.

## Findings

### SP-I1 (Important): the seal test runs in a CI job that gates nothing

**Evidence:**

- Plan §2/§3 put `seal-clear-test.sh` in the `picotool argv probe` job and §0.3 says "CI proves (1)
  and (2) on every change".
- That job is not a required check. FOLLOWUPS F-701 still lists "Open for E3a: adding the probe as a
  required check", and the branch-protection rule requires only `test (rust + go)`
  (release.yml header).
- `assemble` has `needs: [test, go-build, rust-build]` (`release.yml:442`), so a red probe also does
  not block a tag release.
- A red seal test on the signing toolchain therefore blocks neither a merge, nor `push-via-staging`,
  nor a release. The plan's only automated gate on the patch is advisory.

**Fix:**

1. Make "the `picotool argv probe` job is a required check" a precondition of merging this PR, in
   §3 and §4 (Brian's action through the Merging PRs thread, as E3a §6.4 already requests). Add
   `picotool-probe` to `assemble.needs`.
2. If that cannot happen first, say explicitly in §4 that merge waits on a green probe run, checked
   by hand, and recorded in the PR.

### SP-I2 (Important): "RUNBOOK prerequisites unchanged" contradicts the RUNBOOK, and the fork/SeedHammer hold-until-R4 is not written into any document

**Evidence:**

- `RUNBOOK_custom_boot_key.md:85-88` says: "Known issue: on 2.3.1, `sign-firmware.sh`'s throwaway
  seal … fails … Sign from the fork's shell until that is fixed (PICOTOOL_PIN.md, F-701)."
- `PICOTOOL_PIN.md` says "Until it is resolved, R0 cannot pass, and the fork should not move to
  this pin for signing."
- The plan rewrites PICOTOOL_PIN as "blocker resolved in the toolchain" and leaves the RUNBOOK
  "unchanged". That leaves a stale known-issue note. If an implementer updates it to "fixed", it
  removes the only written barrier between an operator following RUNBOOK step 5 from `.#otp` and
  a real SeedHammer image sealed by patched 2.3.1 (EXTRA_SECURITY + VECTOR_TABLE + Clear).
- No such image has booted on hardware. §3's last bullet ("the fork does not move … the fork thread
  is told") is a message, not a recorded gate.
- Nothing mechanical distinguishes the builds either. Patched and stock both print `version -s` =
  `2.3.1`, so `refugium-otp.sh`'s pin cannot tell them apart. (That is harmless for OTP, but it means
  no tool can enforce this gate.)

**Fix:** specify the replacement text in the plan.

- **RUNBOOK:85-88:** "Fixed in this repo's toolchain (patched 2.3.1); **not yet proven on hardware**.
  Until bench R4 (E3a §7) boots a 2.3.1-sealed image on a Pico 2, sign SeedHammer firmware from the
  fork's shell (2.2.0-a4)."
- **PICOTOOL_PIN.md:** the same hold, worded for the fork: it does not take
  `packages.picotool` for signing until R4 passes, with F-701 as the record.
- **F-701:** record R4 as the gate that lifts both.
- **The fork-thread message:** cite these lines.

### SP-M1 (Minor): nothing outside the CI fixture asserts the Clear entry is in the image that is signed and flashed

**Evidence:**

- The patch exists to keep the signed SRAM wipe. Yet the R0 bench gate (E3a §7, restated in §3)
  and `sign-firmware.sh` step 7 (`:173-183`) check only `signature: verified` and the block count.
- **Measured: picotool (stock and patched alike) silently ignores `--clear` when the input already
  has a load map.** Resealing a sealed UF2 gives rc 0, no Clear, and `verified`.
- `sign-firmware.sh` skips sealing entirely when a SIGNATURE section exists (`:99-100`). So an input
  pre-sealed without `--clear` would produce a verified, wipe-less image, silently.
- None of this is introduced by the patch, and the TinyGo blinky and fork firmware have no load map
  (INV), so the path is not exercised today.

**Fix:**

- In §3's R0 bullet, also require `load map entry 0: *Clear 0x20000000->0x20082000` on the
  `.signed.uf2` that R4 flashes.
- File a follow-up (or, if Brian agrees it is in scope, a one-line step-7 assertion in
  `sign-firmware.sh`, using a here-string per F-695) so the flashed artifact carries the property
  this PR exists for.

### SP-M2 (Minor): building `.#picotool-unpatched` with the default out-link would silently retarget the argv probe

**Evidence:**

- The probe job runs `bash scripts/test/picotool-argv-probe.sh "$PWD/result/bin/picotool"`
  (`release.yml:286`). A second `nix build .#picotool-unpatched` without `-o` overwrites `result`.
- The argv probe would then pass on the stock binary, because the OTP and argv paths are identical
  and the version is identical. A mis-ordered positive seal run would fail, but a mis-ordered
  *negative* run could point at the wrong binary.

**Fix:**

- The plan should specify `nix build .#picotool -o result-patched` and
  `nix build .#picotool-unpatched -o result-unpatched`.
- Point every step at an explicit link.
- Make the script assert that the two `readlink -f` store paths differ.

### SP-M3 (Minor): the CI negative control discriminates hunk A only, and the positive check is the tool verifying itself

**Evidence:**

- Unpatched stops at 248 before any hash is computed, so CI never exercises the hunk-B
  discrimination. Hunk B is checked only once, by the implementer's mutation run.
- `info -a` is a genuine ECDSA verify over an independent code path (see the facts above), so this
  is not a false-PASS today. But a future picotool where `info` printed `verified` for a block it did
  not verify would pass silently.

**Fix:** add two cheap controls to `seal-clear-test.sh`, both measured to work here.

- **Tamper control:** flip one payload byte, and separately change the Clear size word
  `0x00082000`→`0x00081000`, in `out.uf2`. Require `signature:           incorrect` for each.
- **Optional independent digest:** run `seal --hash --clear`, recompute SHA-256(size ‖ load bytes ‖
  `block_words_to_hash` block words) in a ~40-line python3 (present in `.#otp` and on the runner),
  and require it to equal HASH_VALUE.

The digest check pins bootrom order permanently, independently of picotool, and would catch a
hunk-B regression in CI.

### SP-M4 (Minor): upstream-issue line numbers and scope

**Evidence:**

- INV §4(c) says `develop`'s `bintool.cpp:849` still has the bad insert. Measured on `develop`
  `ba3df40`, it is at `bintool.cpp:886`, and the model bug is at `main.cpp:5807` (not 5646).
- The plan's issue text says "both bugs with line numbers", which risks citing tag-2.3.1 lines
  against `develop`.

**Fix:**

- The draft cites `develop@ba3df40` lines (`main.cpp:5807`, `bintool.cpp:886`) alongside 2.3.1's.
- It lists the other affected paths: `--pin-xip-sram`, `--hash --clear` without `--sign`, and
  UF2/BIN with a one-entry generic load map.
- It mentions the third `access.get_model()` use at `main.cpp:6233` (`bin2uf2`; it drops the E10
  absolute block for sealed UF2s when `--abs-block` is used; harmless here, where neither output
  has one, 93 blocks each).
- Optionally, it notes that `--clear` is silently ignored for inputs that already carry a load map.

### SP-M5 (Minor): "`nix build .#picotool` on Brian's box prints `2.3.1`" does not prove the patched build

**Evidence:** the stock and patched builds print the same version. The store path is the only
distinguishing mark, and the plan does not record it.

**Fix:**

- §4: record the patched x86_64-linux store path in PICOTOOL_PIN.md (replacing the
  `5cxc3bj…` binary-cache line).
- Require Brian's `nix build .#picotool` to produce that same path. It is input-addressed, so it is
  equal to CI's.
- Run `scripts/test/seal-clear-test.sh result/bin/picotool` on Brian's box at R0.

### SP-M6 (Minor): dangling references after deleting `design/patches/`

**Evidence:** `git grep design/patches` hits `design/FOLLOWUPS.md` (F-701 text) and
`design/PICOTOOL_PIN.md`, as well as the plan itself. INV also cites the path.

**Fix:**

- Update FOLLOWUPS and PICOTOOL_PIN to `nix/patches/…`.
- Leave INV verbatim (it is a persisted report), and note the move in F-701.

### SP-N1 (Nit): the grep for the `verified` line is not anchored

`signature:           verified` appears twice in `info -a` output: under Program Information and
under Metadata Block 2. Anchor the check to the Metadata Block 2 section, or require two
occurrences plus exactly two `Metadata Block` headers.

### SP-N2 (Nit): be precise about what R4 proves

- R4 boots a **picosign-signed** image (`sign_image` → `sign-firmware.sh`; R:1100). It proves
  hunk A plus picosign/bootrom agreement on a Clear+EXTRA_SECURITY image.
- It does not prove picotool's *own* `--clear` signature (hunk B). That is acceptable, because the
  fork's `build-firmware` also clears picotool's signature (`picosign sign -clear`, fork
  `flake.nix:121-123`).
- A blink proves acceptance, not that the wipe ran. The wipe rests on the bootrom source and on the
  Clear entry being semantically identical to 2.2.0-a4's.

Say this in §3 so nobody later cites R4 as hardware proof of hunk B or of the wipe.

### SP-N3 (Nit): fixture provenance

INV records `-target pico2`, TinyGo 0.42.0 (which matches `.#otp`'s tinygo at `a7868a72`) and
47,104 bytes, but not whether R's `-opt 2` was used. If the implementer cannot rebuild it
byte-identically, the README should say so and rely on the sha256 (`e48659d7…54a9`).

### SP-N4 (Nit): shellcheck coverage

Add `scripts/test/seal-clear-test.sh` to the `shellcheck -x -S warning` list in `test (rust + go)`
(`release.yml:232-234`).

VERDICT: 0C / 2I / 6M / 4N
