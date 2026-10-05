# E3a: `picotool seal --sign --clear` exits 248 "unknown sram end" on 2.3.1

*Investigation agent report, 2026-10-05. External-tool facts were checked against picotool source
(tag `2.3.1` at `2041936`, tag `2.2.0-a4` at `25aa087`, `develop` at `ba3df40`), against the RP2350
bootrom source (`pico-bootrom-rp2350` at `c6cdb17`, "A4 bootrom"), and by running binaries. No file
in mnemonic-engrave was edited except this report.*

## Verdict

- **Root cause: two picotool bugs, both introduced by upstream commit `3c743bd` ("Add support for
  pinning XIP SRAM (#298)", 2026-05-06, first released in 2.3.0).** The second bug was hidden behind
  the first.
- **It is not about the image.** The TinyGo blinky and the real fork firmware both carry a valid
  IMAGE_DEF. The brief's first reading (`get_access_model` returning `model_rp_generic`) is wrong
  on two counts:
  - `get_access_model` returns `model_rp2350_arm_s` for these images.
  - `model_rp_generic` *does* implement `sram_end()` (`model/model.h:192`).
- **The fork's real firmware build fails the same way on 2.3.1.** I built it from `d156a3e` with the
  fork's exact TinyGo flags and reproduced the failure.
- **No upstream fix exists.** Both bugs are still present on `develop` and on every open or closed
  PR head numbered 330 or higher.
- **`design/PICOTOOL_PIN.md` is wrong on one point.** It says "the fix belongs to `sign-firmware.sh`
  or the blinky's link layout". It does not: this is a picotool bug and the layout is irrelevant.
- **Recommendation: option (b).** Add a two-line patch to picotool 2.3.1 in mnemonic-engrave's flake
  (an `overrideAttrs` adding `patches`) and report it upstream. Details are in §4 and §6.

## 1. Root cause

### Bug A: the seal path for UF2/BIN passes the wrong model, which triggers the 248

1. `seal_command::execute` computes the correct model: `model_t model = get_model(0);`
   (`main.cpp:6180`). That resolves through `get_access_model` (`main.cpp:4467`) to the
   IMAGE_DEF's chip, cpu and security.
2. For UF2 and BIN input it then calls `sign_guts_bin(access, …, model)` (`main.cpp:6217`, `:6229`).
3. Inside `sign_guts_bin`, the call to `hash_andor_sign` passes **`in.get_model()`**
   (`main.cpp:5646`), not the `model` parameter.
4. `in` is a fresh `file_memory_access` from `get_file_memory_access(0)` (`main.cpp:6210`/`6222`).
   Nothing in the seal path calls `set_model` on it. The only callers are `main.cpp:4739` (info),
   `:5871` (encrypt) and `:9755`.
5. So `in`'s model is the class default `models::unknown` (`main.cpp:2207`;
   `model/model.cpp:3` makes it a `model_unknown`). `model_unknown` (`model/model.h:139`) overrides
   none of the address accessors.
6. With `--clear`, `get_lm_hash_data` (`bintool/bintool.cpp:852`) evaluates
   `model->sram_end() - model->sram_start()` (`bintool.cpp:863`).
7. That reaches the base `model_info::sram_end()`, which is
   `fail(ERROR_NOT_POSSIBLE, "unknown sram end")` (`model/model.h:124`).
8. `ERROR_NOT_POSSIBLE` is -8, so the exit status is 248.
9. Without `--clear` no model accessor is reached: `detect_generic_load_map` returns false at once
   when there is no load map (`bintool.cpp:729`). That is why it works.

The same bug also breaks UF2/BIN `seal --pin-xip-sram`, and any UF2/BIN whose block already holds a
one-entry load map (`detect_generic_load_map_entry` calls `model->xip_sram_start()`,
`bintool.cpp:720`). The ELF path is unaffected: `sign_guts_elf` passes the real `model`
(`main.cpp:5574`).

### Bug B: with `--clear` the hash is computed in the wrong order, so the signature is invalid

This bug is latent behind A. In the "no load map" branch of the BIN `get_lm_hash_data`:

1. The clear size is appended with `std::copy(..., back_inserter(to_hash))` (`bintool.cpp:870`).
2. The pin size is appended the same way (`:883`).
3. Then the image is inserted **at the front**: `to_hash.insert(to_hash.begin(), bin…)`
   (`bintool.cpp:886`).

The digest is therefore `bin ‖ clear_size`. The load map it writes is `[Clear, Load]`, and the
bootrom hashes entries in load-map order. A Clear entry hashes its size word
(`varm_blocks.c:973-997`, `s_varm_sha256_put_word_inc(out_size…)`). So the bootrom, picotool's
`verify_block` and the fork's `picobin.HashData` all compute `clear_size ‖ bin`.

When I fixed bug A alone, picotool's own `--clear` output came back as `signature: incorrect` under
`info -a` (measured; see §5).

### Why 2.2.0-a4 works

`git show 3c743bd -- bintool/bintool.cpp`, and the 2.2.0-a4 source:

- **No model was involved.** 2.2.0-a4's BIN path hardcoded the clear range: `{0x00082000}` at
  runtime `0x20000000` (`2.2.0-a4:bintool/bintool.cpp:772-777`). Its `sign_guts_bin`
  (`2.2.0-a4:main.cpp:5126`) took no model at all.
- **The hash order was right.** 2.2.0-a4 first did `to_hash.insert(begin, bin)` (`:768`) and *then*
  `to_hash.insert(begin, sram_size)` (`:780`), giving `clear_size ‖ bin`.
- **Commit `3c743bd` changed both.** It threaded `in.get_model()` into the call, which is bug A. It
  switched the size to `back_inserter` but moved the `insert(begin, bin)` below it, which is bug B.

The RP2350 values are unchanged. `SRAM_START 0x20000000` and `SRAM_END_RP2350 0x20082000`
(`model/addresses.h:17,21`) give the same 0x82000 that a4 hardcoded, so with a correct model the
Clear entry is identical.

### What the images carry

`picotool info -a` (2.3.1) on both the blinky (`-target pico2`, 47,104-byte UF2, TinyGo 0.42.0) and
the real firmware (`-target pico-plus2 …`, 3,334,656-byte UF2) shows:

- family `rp2350-arm-s`;
- Metadata Block 1 at `0x100000f8`, `block type: image def`, RP2350, ARM Secure, a self-loop;
- no load map.

The raw block words are `ffffded3 10210142 000001ff <link> ab123579`, which decode as
IMAGE_TYPE = EXE | ARM | Secure | RP2350. So TinyGo's output does carry an IMAGE_DEF.

## 2. The fork's real build-firmware seal fails on 2.3.1

- **Today it passes.** The fork's `flake.lock` pins nixpkgs `ec942ba`, whose `picotool.version` is
  **2.2.0-a4** (`nix eval`). Built from that rev: `/nix/store/j7pl045ik6yb73zvq3n9a52j85d2qnig-picotool-2.2.0-a4`.
- **It fails once the fork adopts the pin.** `PICOTOOL_PIN.md` plans for the fork to take
  `mnemonic-engrave.packages.picotool`, which is 2.3.1, and then `flake.nix:121` fails.
- **How I built the firmware.** From fork `d156a3e` (clean tree), with TinyGo 0.41.1 and Go 1.26.3
  taken from nixpkgs `ec942ba`. The flake's overlay also pins TinyGo 0.41.1. I ran the flake's exact
  flags:
  `tinygo build -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`.
  `nix run .#build-firmware` itself was not run, because nix sandboxed builds fail on this box (see
  §6).
- **The key** is the fork's exact `dummy.pem`, extracted from `flake.nix`.

Results:

| image | picotool | `--clear` | no `--clear` |
|---|---|---|---|
| blinky | 2.2.0-a4 | rc 0 | rc 0 |
| blinky | 2.3.1 (nix) | **rc 248 "unknown sram end"** | rc 0 |
| firmware (d156a3e) | 2.2.0-a4 | rc 0 | rc 0 |
| firmware (d156a3e) | 2.3.1 (nix) | **rc 248 "unknown sram end"** | rc 0 |

## 3. What `--clear` does, and why it matters for security

### What it changes in the output

It adds load-map entry 0, `Clear 0x20000000->0x20082000`. The raw entry is
`storage 0x00000000, runtime 0x20000000, size 0x00082000`. The size word is part of the signed hash.

From 2.2.0-a4 on the real firmware:

- with `--clear`: `load map entry 0: Clear 0x20000000->0x20082000` and
  `load map entry 1: Load 0x10000000->0x10197100`;
- without it: only `Load 0x10000000->0x10197100`.

### What the bootrom does with it

`varm_blocks.c:973-983`: a storage address of 0 means "zero it". While booting, the bootrom calls
`call_s_varm_step_safe_crit_mem_erase_by_words` over the whole 520 KiB of main SRAM (SRAM0-9,
including the SRAM8/9 scratch banks). It does this during image verification, before launch, and
hashes the size word into the signature digest (`:997`). So the clear cannot be stripped without
invalidating the signature.

### The bootrom does NOT otherwise zero main SRAM on a normal boot

The brief's premise needs correcting.

- `riscv_bootrom_rt0.S:334` says, verbatim, "don't clear all of RAM to save time".
- The ARM path clears only USB RAM (`arm8_bootrom_rt0.S:710`), bootram's zero-init area
  (`varm_boot_path.c:475`) and its own scan contexts.
- The exception is entering BOOTSEL (nsboot). `arm8_nsboot_vm.S:53-70` erases all of main SRAM
  (`SRAM_BASE..SRAM_END`) before USB boot. So the BOOTSEL/PICOBOOT path is already covered without
  `--clear`.

### Security relevance for SeedHammer (handles seeds)

- **What `--clear` protects.** It wipes a previous session's RAM before *any* code in the new image
  runs. This applies on every boot path that does not pass through nsboot: watchdog or
  `SYSRESETREQ` resets, `rom_reboot` into the image, and crash recovery.
- **What residue there could be.** Seed words, xprvs, passphrase buffers, GC heap leftovers and
  stack.
- **Why TinyGo does not cover it.** TinyGo zeroes only `.bss` and initializes `.data`. The stack and
  never-reallocated heap are not scrubbed at start.
- **What happens without it.** That residue survives into the next boot, and only an
  uninitialized-read bug (or a different validly signed image) separates it from disclosure.
- **What it does not cover.** XIP SRAM/cache. Power-cycle decay is outside its scope.

This is defence in depth, not a primary control, but it is cheap and it is what the fork ships
today.

## 4. Options, tested

### (a) Command-line route on stock 2.3.1

- **UF2 or BIN input: impossible.** Both go through `sign_guts_bin` → `in.get_model()`. No option or
  environment variable sets that access object's model:
  - `seal` has no `--family`/`--platform`;
  - `settings.model` is only read by `get_model()`, whose result `sign_guts_bin` ignores;
  - `uf2 convert --platform` does not help, because the seal input would still be UF2 or BIN.
- **ELF input works (tested).** I ran `tinygo build -o x.elf` (same flags), then
  `picotool seal --sign --clear x.elf x.sealed.elf`: rc 0. Then `picotool uf2 convert x.sealed.elf
  x.uf2`: rc 0.
  - The real firmware's ELF→UF2 without sealing is byte-identical to TinyGo's own UF2.
  - The output verifies, both from picotool directly and after picosign re-signing (§5).
- **But the ELF result is a different image layout.**
  - Its load map is
    `Clear 0x20000000->0x20082000; Load 0x10000000->0x1018f2e0; Load 0x1018f2e0->0x1018f2f4; Copy 0x1018f2f8->0x101970c8 to 0x20001000->0x20008dd0`.
  - The bootrom now copies `.data` into SRAM itself.
  - The ELF is squashed (a 4-byte hole is skipped).
  - The block moves from `0x10197100` to `0x101970c8`.
- **And it changes the pipeline.** The fork's `build-firmware` must emit ELF, then convert.
- **Verdict.** Viable but not equivalent. It is a larger change to normative signing behavior (risk
  set) than (b).

### (b) A two-line patch to picotool 2.3.1 (recommended)

```diff
--- a/main.cpp
+++ b/main.cpp
@@ -5643,7 +5643,7 @@ vector<uint8_t> sign_guts_bin(iostream_memory_access in, ...
     auto sig_data = hash_andor_sign(
         bin, bin_start, bin_start,
         &new_block, public_key, private_key,
-        in.get_model(),
+        model,
         settings.seal.hash, settings.seal.sign,
         settings.seal.clear_sram, settings.seal.pin_xip_sram
     );
--- a/bintool/bintool.cpp
+++ b/bintool/bintool.cpp
@@ -883,7 +883,7 @@ std::vector<uint8_t> get_lm_hash_data(std::vector<uint8_t> bin, ...
             DEBUG_LOG("PIN XIP SRAM %08x + %08x\n", (int)model->xip_sram_start(), (int)xip_pin_size_vec[0]);
         }
-        to_hash.insert(to_hash.begin(), bin.begin(), bin.end());
+        to_hash.insert(to_hash.end(), bin.begin(), bin.end());
         DEBUG_LOG("HASH %08x + %08x\n", (int)storage_addr, (int)bin.size());
```

**Size and risk: two tokens in two files, low risk.**

- The change touches only `seal`'s UF2/BIN path. `encrypt` and the ELF path are untouched.
- **Fix A** substitutes the model the command already computed from the image's own IMAGE_DEF. It is
  the same object the ELF path uses.
- **Fix B** is a no-op whenever neither `--clear` nor `--pin-xip-sram` is given, because `to_hash` is
  empty before the insert.
- **Measured: no-`--clear` output is unchanged.** I compared patched and stock 2.3.1
  `seal --sign` (no `--clear`) on both images. They differ in exactly 64 bytes, the ECDSA signature
  value. picotool's ECDSA is randomized: two runs of the same binary also differ there.

**Tested with a native cmake build of tag 2.3.1, unpatched and patched.** The build used the same
pico-sdk 2.3.1 store path and the same mbedtls substitution as nixpkgs, with `HAS_LIBUSB=0`. `seal`
does not need USB.

- **Unpatched native build:** reproduces rc 248 on both images, so the build is faithful.
- **Patched:**
  - blinky and firmware: `--clear` rc 0, giving
    `Clear 0x20000000->0x20082000; Load 0x10000000->…`;
  - `signature: verified` under both 2.3.1 and 2.2.0-a4 `info -a`;
  - `--hash --clear` gives `hash: verified` as well.

Nix overlay form, for `mnemonic-engrave/flake.nix`:

```nix
picotool = pkgs.picotool.overrideAttrs (o: {
  patches = (o.patches or [ ]) ++ [ ./nix/picotool-2.3.1-seal-clear.patch ];
});
```

**Caveats:**

- `version -s` stays `2.3.1`. The `otp list` fingerprint that `refugium-otp.sh` checks is unchanged
  (the patch does not touch OTP code).
- The store path changes and becomes a local build, not a binary-cache hit. `PICOTOOL_PIN.md`'s
  "no overlay" and store-path lines must be updated.
- I could not run the nix build of the overlay on this box. Every nix sandboxed build here fails with
  `setup: /dev/null: Permission denied`, an environment fault, not the patch. A draft expression is
  at `<scratchpad>/e3a/patch/default.nix`; point it at the two-hunk patch. Run that build on the
  operator box before relying on it.

### (c) Upstream fix after 2.3.1

**None.** I fetched `develop` at `ba3df40` (2026-09-28, "start 2.3.2 development" plus six commits)
and all PR heads numbered 330 or higher, 22 refs (#332–#362):

- every one still has `in.get_model(),` in `sign_guts_bin`;
- `develop`'s `bintool.cpp:849` still has `to_hash.insert(to_hash.begin(), bin…)` after the clear
  append.

Issue search was not possible: GitHub search and GraphQL are blocked for this session. Both
patches are suitable for an upstream PR to `raspberrypi/picotool` against `develop`.

### (d) Drop `--clear`

- **What works.** On 2.3.1, `seal --sign` without `--clear` succeeds and verifies.
- **What is lost.**
  - The signed load-map instruction that makes the bootrom zero all 520 KiB of main SRAM
    (`0x20000000–0x20082000`) before the image runs, on every boot that does not come through
    BOOTSEL.
  - The image hash changes, because the Clear size word leaves the digest. Signed fork firmware
    would then differ in boot semantics from what the fork ships today.
- **Verdict.** Not acceptable for seed-handling firmware without an explicit decision. It is fine
  only for the throwaway rehearsal blinky (R's `make_otp_json` already omits it).

### (e) Keep 2.2.0-a4 for sealing only

- **What it gives.** It works. It is byte-for-byte the structure the fork ships today: no
  EXTRA_SECURITY, no VECTOR_TABLE item.
- **What it costs.**
  - It contradicts `PICOTOOL_PIN.md`'s "one picotool" design.
  - It needs a second nixpkgs input, or a `src` override to tag 2.2.0-a4.
  - It leaves the sealing step on a picotool that the pin document already rejects for other
    reasons.
- **Verdict.** A reasonable stop-gap. It is what the fork does today by accident of its own
  `flake.lock`.

## 5. Does each route verify, and is its load map equivalent to 2.2.0-a4 `--clear`?

**The verification check.** I built the fork's `cmd/picosign` at `d156a3e` (Go 1.26.3). For each
output I ran the full external-signing flow with a fresh secp256k1 key:

1. `picosign sign -clear`;
2. `picosign hash`;
3. `openssl pkeyutl -sign` (DER);
4. `picosign sign -pubkey <compressed> -sig <der> -sigfmt der`;
5. `picotool info -a`.

| output | after picosign, 2.3.1 `info -a` | after picosign, 2.2.0-a4 `info -a` |
|---|---|---|
| firmware, 2.2.0-a4 `--clear` (baseline) | verified | verified |
| firmware, patched 2.3.1 `--clear` (A+B) | verified | verified |
| firmware, patched A only `--clear` | verified | verified |
| firmware, stock 2.3.1 ELF route | verified | (not run) |
| firmware, stock 2.3.1 no `--clear` | verified | verified |
| blinky: same five rows | all verified | all verified |

(A-only verifies after picosign because picosign recomputes the hash from the load map. Bug B only
corrupts picotool's *own* signature.)

**Patched-UF2 metadata block 2 against 2.2.0-a4 `--clear`.** I decoded the raw words. Everything
before the block is byte-identical. The block is at the same address (firmware `0x10197100`,
blinky `0x10005c00`).

| item | 2.2.0-a4 | patched 2.3.1 |
|---|---|---|
| IMAGE_TYPE | `10210142` | `18210142` (+EXTRA_SECURITY 0x0800) |
| VECTOR_TABLE | absent | `00000203 10000000` (new) |
| ENTRY_POINT | `00000344 100125d5 20000800` | same |
| LOAD_MAP | `02000706 00000000 20000000 00082000 ffe68eec 10000000 00197100` | `02000706 00000000 20000000 00082000 ffe68ee4 10000000 00197100` |
| HASH_DEF | `01000247 0000000e` | `01000247 00000010` |
| SIGNATURE | (random) | (random) |
| LAST | `00002eff` | `000030ff` |

- **Load map: semantically identical, not byte-identical.** Both have the same two entries, the same
  Clear range and size, and the same Load storage, runtime and size. The one differing word is the
  Load entry's *relative* storage offset (`ffe68eec` → `ffe68ee4`). The load-map item sits 8 bytes
  later because the VECTOR_TABLE item precedes it; both resolve to storage `0x10000000`.
- **The other differences are 2.3.1's own seal behavior and are independent of `--clear`.** These are
  the EXTRA_SECURITY flag, the VECTOR_TABLE item and the hashed-word count. They come from commit
  `699049f`, "Add support for extra security bit (#273)", which first shipped in 2.3.0.
  `PICOTOOL_PIN.md` already records them. The bootrom then refuses to default the entry point or
  vector table (`varm_launch_image.c:292-301`), and both are present.
- **The ELF route is not equivalent.** It has four entries including a Copy, and a different block
  address (§4a).
- **Only (e) is byte-equivalent to today's fork output.**

## 6. Recommendation

**(b): patch picotool 2.3.1 in mnemonic-engrave's flake with the two-hunk patch above, and send the
same patch upstream.**

- It keeps the single pinned picotool (the PICOTOOL_PIN design).
- It keeps `--clear`, so the bootrom SRAM wipe stays in the signed load map.
- It changes nothing for any invocation without `--clear`/`--pin-xip-sram` (measured).
- It yields a load map semantically identical to what the fork ships today with 2.2.0-a4.
- Its output verifies under both picotool versions, before and after the picosign flow.
- Fix B also makes picotool's *own* `--clear` signatures correct. Without it, anyone using
  `picotool seal --sign --clear` as the final signer on 2.3.x gets an image the bootrom would reject.

This is risk-set work (it changes the signing toolchain), so the overlay needs the usual R0 gate and
a real nix build plus the R0 bench check (`sign-firmware.sh` → `signature: verified`). It also needs
a hardware boot of a patched-and-signed image before the fork moves to the pin. EXTRA_SECURITY
images have not been booted on hardware in this investigation.

**If a patch is unwelcome:** use (e), 2.2.0-a4 only for the seal step, as a time-boxed stop-gap.
**Do not use** (d) for the fork firmware. **Do not use** (a) unless the ELF pipeline is wanted for
other reasons.

### Corrections to existing docs

- `design/PICOTOOL_PIN.md` §"What changed that matters: `seal`" says "The fix belongs to
  `sign-firmware.sh` or the blinky's link layout". It is a picotool bug (`main.cpp:5646`,
  `bintool.cpp:886`), and the real fork firmware fails identically.

### Side finding, out of scope, not triggered today

`seedhammer/picobin/picobin.go:326-330` `HashData` hashes `eidx+8` (always entry 0's size word) for
*every* storage-0 entry, instead of `eidx + i*12 + 8`.

- **When it is correct:** whenever the Clear entry is entry 0 and is the only zero-storage entry,
  which is the case today.
- **When it is wrong:** a `--clear --pin-xip-sram` image would get the wrong hash.

`picobin` is fork-native (no Rust counterpart), so a Go-side fix would be exempt from the Rust-first
rule.

## Evidence locations (scratchpad, not committed)

`/tmp/claude-0/-home-claude/19e7be54-3c48-575e-b0f5-ca3e6d615032/scratchpad/e3a/`:

- **Images:**
  - `blinky.uf2`, `firmware.uf2` and `firmware.elf` (the inputs; firmware built from fork `d156a3e`);
  - every sealed variant: `*.a4.clear`, `*.pat2.clear`, `*.pat.clear`, `*.elfroute`,
    `*.stock.noclear`;
  - `t-*.uf2`, the picosign-resigned copies.
- **Tools:**
  - `uf2dump.py`, the raw block-word dumper;
  - `build/b-orig` and `build/b-patched`, the native picotool 2.3.1 builds;
  - `patch/seal-clear-fix.patch`.
- **Nix paths:** the 2.2.0-a4 binary is `/nix/store/j7pl045ik6yb73zvq3n9a52j85d2qnig-picotool-2.2.0-a4`
  (from nixpkgs `ec942ba`).
