# DRAFT — upstream issue for `raspberrypi/picotool` (for Brian to post; not posted by Claude)

*Drafted 2026-10-05 for F-701 (plan `design/IMPLEMENTATION_PLAN_e3a_picotool_seal_patch.md` §2).
Edit freely. Everything below the line is the proposed issue body. Line numbers were checked
against tag `2.3.1` (`2041936`) and `develop` at `ba3df40` (2026-09-28). The patch at the end is
`nix/patches/picotool-2.3.1-seal-clear-fix.patch` without its header; against `develop` the same
two one-line changes apply at the `develop` lines given.*

---

**Title:** `seal --clear` (and `--pin-xip-sram`) on UF2/BIN input fails with "unknown sram end"; with that fixed, the signature is computed in the wrong order

### Summary

Since 2.3.0, `picotool seal --sign --clear in.uf2 out.uf2 key.pem` exits with status 248 and
`ERROR: unknown sram end` for any UF2 or BIN input, including images that carry a valid RP2350
IMAGE_DEF. Fixing that exposes a second bug: the hash picotool signs puts the image bytes before the
Clear entry's size word, so its own signature comes out `incorrect` under `picotool info -a` and would
be rejected by the boot ROM. Both come from #298 (commit `3c743bd`, "Add support for pinning XIP
SRAM"). ELF input is not affected. 2.2.0 worked.

### Reproduce

Any UF2 with an RP2350 IMAGE_DEF and no load map (we used a TinyGo `-target pico2` blinky; the IMAGE_DEF
is EXE | ARM | Secure | RP2350):

```
openssl ecparam -name secp256k1 -genkey -noout -out key.pem
picotool seal --sign --clear in.uf2 out.uf2 key.pem
# ERROR: unknown sram end        (exit 248)
picotool seal --sign in.uf2 out.uf2 key.pem
# works; `picotool info -a out.uf2` -> signature: verified
```

The same happens with a `.bin` input, with `--pin-xip-sram`, with `--clear --pin-xip-sram`, and with
`--hash --clear` (no `--sign`).

### Bug A — `sign_guts_bin` uses the input file's model, which is never set

`seal_command::execute` computes the model from the image (`model_t model = get_model(0);`, 2.3.1
`main.cpp:6180`) and passes it to `sign_guts_bin` (`main.cpp:6217`, `:6229`). But `sign_guts_bin`
ignores its `model` parameter and passes `in.get_model()` to `hash_andor_sign`:

| | 2.3.1 | develop@ba3df40 |
|---|---|---|
| `in.get_model(),` in `sign_guts_bin` | `main.cpp:5646` | `main.cpp:5807` |

`in` is the `file_memory_access` from `get_file_memory_access(0)`; nothing on the seal path calls
`set_model` on it, so its model is the default `models::unknown`. With `--clear`,
`get_lm_hash_data` evaluates `model->sram_end() - model->sram_start()` (2.3.1 `bintool/bintool.cpp:863`),
and `model_info::sram_end()` is `fail(ERROR_NOT_POSSIBLE, "unknown sram end")`
(`model/model.h:124`). `ERROR_NOT_POSSIBLE` is -8, hence exit 248.

Without `--clear` no model accessor is reached (`detect_generic_load_map` returns early when there is
no load map, `bintool.cpp:729`), which is why plain `seal --sign` works. A UF2/BIN that already holds
a one-entry load map also reaches the model (`detect_generic_load_map_entry` calls
`model->xip_sram_start()`, `bintool.cpp:720`). The ELF path passes the real `model`
(`sign_guts_elf`, 2.3.1 `main.cpp:5573-5575`), so ELF input works.

### Bug B — the Clear/pin size words are hashed after the image

In the no-load-map branch of the BIN `get_lm_hash_data`, the Clear size word is appended to
`to_hash` (2.3.1 `bintool.cpp:870`), then the pin size word (`:883`), and then the image is inserted
at the **front**:

| | 2.3.1 | develop@ba3df40 |
|---|---|---|
| `to_hash.insert(to_hash.begin(), bin.begin(), bin.end());` | `bintool.cpp:886` | `bintool.cpp:886` |

So the digest is `bin ‖ clear_size [‖ pin_size]`, while the load map written is
`[Clear, (Pin,) Load]` and the boot ROM hashes the load map in entry order — a storage-address-0
entry contributes its size word (`pico-bootrom-rp2350` `varm_blocks.c`, around lines 973-997) —
giving `clear_size [‖ pin_size] ‖ bin`. `picotool info -a` agrees with the boot ROM: with bug A
fixed alone, picotool's own `--clear` output reports `signature: incorrect`.

2.2.0 got this right: it inserted the image at the front first and then the size at the front
(`bintool.cpp:768`, `:780` at `2.2.0-a4`). #298 switched the size to `back_inserter` but kept the
front insert of the image.

### Affected

- UF2 and BIN input to `seal` with `--clear`, `--pin-xip-sram` or both, with `--sign` and with
  `--hash` alone (bug A; bug B once A is fixed).
- UF2/BIN input that already carries a one-entry generic load map (bug A).
- Not affected: ELF input; UF2/BIN `seal` without `--clear`/`--pin-xip-sram` and with no prior
  load map; `encrypt`.

### Possible fix

Two one-line changes (shown against 2.3.1):

```diff
diff --git a/bintool/bintool.cpp b/bintool/bintool.cpp
--- a/bintool/bintool.cpp
+++ b/bintool/bintool.cpp
@@ -883,7 +883,7 @@ std::vector<uint8_t> get_lm_hash_data(std::vector<uint8_t> bin, uint32_t storage
             std::copy(xip_pin_size_data.begin(), xip_pin_size_data.end(), std::back_inserter(to_hash));
             DEBUG_LOG("PIN XIP SRAM %08x + %08x\n", (int)model->xip_sram_start(), (int)xip_pin_size_vec[0]);
         }
-        to_hash.insert(to_hash.begin(), bin.begin(), bin.end());
+        to_hash.insert(to_hash.end(), bin.begin(), bin.end());
         DEBUG_LOG("HASH %08x + %08x\n", (int)storage_addr, (int)bin.size());
         entries.push_back(
             {
diff --git a/main.cpp b/main.cpp
--- a/main.cpp
+++ b/main.cpp
@@ -5643,7 +5643,7 @@ vector<uint8_t> sign_guts_bin(iostream_memory_access in, private_t private_key,
     auto sig_data = hash_andor_sign(
         bin, bin_start, bin_start,
         &new_block, public_key, private_key,
-        in.get_model(),
+        model,
         settings.seal.hash, settings.seal.sign,
         settings.seal.clear_sram, settings.seal.pin_xip_sram
     );
```

The second change is a no-op when neither `--clear` nor `--pin-xip-sram` is given (`to_hash` is
empty before the insert).

### What we verified (2.3.1 with the patch, native and nixpkgs builds)

- `seal --sign --clear` on UF2 and on BIN: exit 0, load map `Clear 0x20000000->0x20082000; Load …`,
  `signature: verified` under `info -a` of both 2.3.1 and 2.2.0.
- `--pin-xip-sram` and `--clear --pin-xip-sram`: exit 0, verified.
- `--hash --clear`: `hash: verified`; the stored hash equals an independent SHA-256 of
  `clear_size ‖ image ‖ block words` computed outside picotool.
- Flipping one byte of the sealed image, or changing the Clear size word, gives
  `signature: incorrect`.
- `seal --hash` without `--clear`: byte-identical to unpatched; `seal --sign` without `--clear`:
  differs only in the (randomized) signature bytes.
- Reverting either change alone: without the `main.cpp` change, exit 248; without the
  `bintool.cpp` change, `signature: incorrect`.

### A separate bug found while testing: `der_to_raw` truncates short integers

*(Brian: this one may deserve its own issue. It is not in the patch above, and our toolchain does
not depend on it — `sign-firmware.sh` takes the final signature from `picosign`, which pads r and s —
but anyone using `picotool seal --sign` as the final signer is affected.)*

`bintool/mbedtls_wrapper.c` `der_to_raw` (2.3.1 lines 162-180; also in 2.2.0) handles a DER integer
shorter than 32 bytes with

```c
memset(r, 0, sizeof(r));
memcpy(r + (32 - b2), sig->der + 4, (32 - b2));   // length should be b2
```

(and the same for `s` with `b3`). With `b2 = 31` this stores `00 XX 00 … 00` — the integer's first
byte and 30 zero bytes — instead of `00` followed by the 31 bytes. Whenever r or s is below 2^247
(roughly 1 signature in 256), `seal --sign` writes a signature that `picotool info -a` reports as
`incorrect` and the boot ROM would reject. Measured on 2.3.1 and 2.2.0-a4: 3 such signatures in
1500 seals, each of the form `00XX` + 60 zeros in one half, e.g.
`0024000000000000000000000000000000000000000000000000000000000000A3878DCF…`. The fix is to copy
`b2` (resp. `b3`) bytes.

### Side notes (not part of the fix)

- `bin2uf2` on the seal path is also passed `access.get_model()` (2.3.1 `main.cpp:6233`,
  develop `:6394`); it decides the absolute-block handling for `--abs-block`. We did not see a
  wrong output from it, but it is the same pattern.
- `--clear` is silently ignored when the input already has a load map (for example, resealing an
  already sealed UF2): exit 0, no Clear entry, signature verified. A warning would help.
