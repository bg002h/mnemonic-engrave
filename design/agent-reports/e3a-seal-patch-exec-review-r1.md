# E3a seal patch: post-implementation adversarial execution review, round 1

*Reviewer: independent subagent (Opus), 2026-10-05. Scope: `git diff 0afd496 2f757ec` on branch
`claude/project-thread-eda5dt` (HEAD `2f757ec`, clean tree). Plan:
`design/IMPLEMENTATION_PLAN_e3a_picotool_seal_patch.md` (GREEN). Implementer report:
`design/agent-reports/e3a-seal-patch-impl-report.md`, which I checked instead of trusting. I modified
nothing in the repo except this file. Every key I used was a throwaway from `openssl ecparam`, and no
device was touched.*

## What I re-ran (measured, not quoted)

| run | result |
|---|---|
| `seal-clear-test.sh` on the nix patched build `ri9krl08…` | 11 passed / 0 failed, rc 0 |
| `seal-clear-test.sh --expect-fail` on nix stock `5cxc3bj…` | 7 / 0, rc 0 |
| negative controls: stock in default mode; patched with `--expect-fail` | both red, rc 1 (`exit 248` and `exit 0, want 248` respectively) |
| `seal-clear-test.sh` on 2.2.0-a4 `j7pl045…` | 11 / 0 (the Clear line format matches) |
| the hunk-B mutant (`sealimpl/mut/b-mutB`; its source diff confirmed to be hunk A only) | **red 40/40**, always on `--clear output: signature verdict 'incorrect'`; the retry never fired |
| argv probe on the patched binary | `PROBE PASS`, 58 passed / 0 failed |
| shellcheck 0.11.0 `-x -S warning` on CI's exact list, plus `sign-firmware.sh` and `seal-check.sh` | clean (only SC2016 *info* on the wiring greps, which is intended) |
| `nix eval` (path: flake ref) | `picotool` = `default` = `ri9krl08…`; `picotool-unpatched` = `5cxc3bj…`; `version` 2.3.1; `patches` = `[n4ws2izy…-picotool-2.3.1-seal-clear-fix.patch]` for the patched build and `[]` for stock; `devShells.otp` contains `ri9krl08…` and not the stock path |
| `cmp` of the store patch against the tree file | identical. The hunks are byte-identical to the R0-reviewed `design/patches/` copy at `0afd496`, and only the header changed |
| fixture | sha256 matches the README; `info -a` shows one metadata block with no signature, public key or load map |

`run-e2e-otp.sh` was not re-run: I changed nothing.

## Lens 1: the `der_to_raw` deviation (`0de77eb`)

**The bug is real.** In the source, `bintool/mbedtls_wrapper.c` at 2.3.1 (`2041936`) has these two lines:

- `:169`: `memcpy(r + (32 - b2), sig->der + 4, (32 - b2));`
- `:179`: the same for `s`, with `b3`.

The same lines are present at `2.2.0`, `2.2.0-a4` and `develop@ba3df40`. The copy length should be `b2`.

- With `b2 = 31`, the code stores `00 ‖ der[4] ‖ 00×30`.
- A 31-byte positive DER integer encodes v ∈ [2^239, 2^247). Shorter integers give `b2 ≤ 30`, which is k ≥ 2 in the test's pattern.
- So "wrong whenever r or s < 2^247" is exact. Per half that is ≈ 2^247/n ≈ 2^-9, or about 0.39% per signature.

**Measured with my own harness** (`--clear` seals, a fresh key each, run 22 in parallel; verdicts from the patched `info -a`):

| signer | seals | `incorrect` | of those, fingerprint match | `incorrect` without the fingerprint |
|---|---|---|---|---|
| nix patched 2.3.1 | 10,000 | 43 (0.43%) | 43 | **0** |
| 2.2.0-a4 | 3,000 | 8 (0.27%) | 8 | **0** |
| **native patched 2.3.1 + `der_to_raw` fixed to copy `b2`/`b3`** (my build, `rev/b-dtr`) | 10,000 | **0** | n/a | 0. Here 72 seals had a raw half beginning `00`, and all of them verified |

All 51 bad signatures are `00 XX 00…00` in exactly one half (k = 1). Every `XX` is ≤ 0x7F, which is what a DER integer's first byte must be; random bytes would satisfy that with probability 2^-51. One case, `00 00 …`, is the padded sub-case v ∈ [2^239, 2^240). The fix-build run is causal: with only `der_to_raw` changed, the failures go to zero. The observed rate (51 / 13,000 = 0.39%) matches the 2^-8 prediction.

**Can the retry mask a real defect?** No, and the argument is stronger than the one in the script's comment:

- A retry only re-seals. Green still requires a **later, fresh seal to verify** under `info -a`.
- picotool's verifier hashes through the *existing-load-map* branch of `get_lm_hash_data` (`bintool.cpp:897-925`, reached via `verify_block`, `:987`). That is entry-order hashing, independent of the signer's no-load-map branch that hunk B changes.
- So any deterministic wrong-digest defect, whether hunk B reverted, a mis-ordered pin word or a wrong size word, is `incorrect` on every attempt. The retry cannot turn it green: at worst it costs two extra seals and then reports red, because after attempt 3 `V` stays `incorrect`.
- The only defects a retry can absorb are ones that are intermittent, key-dependent **and** carry this exact fingerprint. That is the `der_to_raw` class and nothing else plausible.
- The discriminator is tight in any case. A random half matches the k=1..4 patterns with probability ≲ 2^-223. A hunk-A regression exits 248, and a non-zero exit is never retried (`[ "$RC" -eq 0 ] && [ -s "$out" ] || return 0`).
- The mutant result above (40/40 red, no `note` line) confirms this empirically.

**Does it affect anything we ship?** No:

- `sign-firmware.sh` seals with a throwaway key and then runs `picosign sign -clear` (`:106-110`, zeroing pubkey + signature). The final signature is `openssl pkeyutl` DER converted by `picosign`. Step 6b (`:157-170`) left-pads r/s and requires byte equality with what is embedded.
- The fork's `build-firmware` (`flake.nix:121-123` at `d156a3e`) also runs `picosign sign -clear` right after its dummy-key seal.
- R's `make_otp_json` (`pico2-bootkey-rehearsal.sh:324`) discards the sealed image (`.seal-discard.uf2`) and keeps only the OTP JSON, which is derived from the public key.
- `sh2-flash` signs through `sign-firmware.sh`.
- Nothing in the repo uses picotool's own signature as the final one.

**Recommendation: KEEP.** The optional tightenings in SPX-N2 are cosmetic. Replacing the retry with a third `der_to_raw` hunk would be a toolchain change outside the GREEN plan, and it fixes nothing we ship. It is a reasonable follow-up only if someone ever wants picotool's own signature to be final.

## Lens 2: false pass

- **`seal-clear-test.sh`:**
  - `set -uo pipefail` without `-e` is deliberate: verdicts are counted, and the exit status is `[ "$FAIL" -eq 0 ]`. Both exit paths (`exit 2`) reach CI as red, and the EXIT trap preserves the status.
  - Seal exit codes are captured correctly (`OUT="$(…)" || RC=$?`).
  - `sig_verdict` returns `none` when there are zero `signature:` lines and `mixed` on any disagreement. Only all-`verified` passes.
  - The Clear line is matched with `grep -qxF`.
  - The tamper flip is self-checked (target word, then read-back of the byte).
  - All greps on captured output use here-strings. The one pipe is SPX-N3, and it can only fail red.
- **`--expect-fail`** requires both rc 248 and the message text, so a mislabelled patched binary goes red (re-run above).
- **CI:**
  - The patched and stock builds use separate out-links, and the step checks that their store paths differ. If `overrideAttrs` produced a new derivation without applying the patch, the default-mode test still fails at 248. So the store-path check is belt-and-braces, not the sole guard.
  - `picotool-probe` has no `if:` and no docs-only skip, so it cannot be silently skipped ahead of `assemble`.
- **`seal_has_clear_entry`** is anchored on both ends and requires entry **0** with the exact RP2350 range. Its only gap is SPX-N1 (it is not scoped to a block), which matters only for an adversarially built image.
- **A path that bypasses the new check:** SPX-M3 (`sh2-flash` flashes an existing `*.signed.uf2` as-is).

## Lens 3: signing path

The `sign-firmware.sh` diff only adds two things: sourcing the lib (`:55-57`) and the check plus `die` (`:187-199`), placed after the existing verified check. Every other line is unchanged, so for any image that carries the Clear entry the behaviour is identical. Legitimate fork firmware carries it:

- The fork's `build-firmware` seals with `--clear` using 2.2.0-a4.
- 2.2.0-a4's `get_lm_hash_data` pushes the Clear entry before Load (`bintool.cpp:768-780` at `2.2.0-a4`), so it is entry 0.
- The implementer's R e2e run 2 exercised exactly this (phase 5b), with one caveat: the image was hand-reproduced and not made by `nix run .#build-firmware`.

An image that already had a load map (where picotool silently ignores `--clear`) is now refused, which is the intent. Sourcing by `$REPO_ROOT` matches how `sh2-flash` and R invoke the script.

## Lens 4: `flake.nix` / CI

- `builtins.path` gives the patch its own store path (`n4ws2izy…`), byte-identical to the tree file.
- The version assertion sits in `picotoolFor`, so `packages.picotool`, `default` and `devShells.otp` all fail evaluation on a version bump, while `picotool-unpatched` still evaluates (the implementer showed this; I confirmed the structure).
- No shell references `picotool-unpatched`.
- `flake.lock` is unchanged.
- Out-links are explicit, `readlink -f` of both paths is printed, and the job fails if they are equal.
- `picotool-probe` is in `assemble.needs`, and the job's permissions are still `contents: read`. No write permission was added.
- `nix/` is classed as code by the docs-only rule (`release.yml:123`, `grep -qvE '^(design/|[^/]+\.md$)'`).

## Lens 5: docs

- The hold text appears verbatim, on single lines, in:
  - `PICOTOOL_PIN.md:6` and `:96`;
  - `RUNBOOK_custom_boot_key.md:88`;
  - `FOLLOWUPS.md:20483`.

  I matched it with `grep -F` against the plan's exact sentence.
- The RUNBOOK's "either shell" sentence is scoped as required.
- RUNBOOK step 5 (`:425-428`) signs from the fork's `nix develop`, which uses 2.2.0-a4 and is consistent with the hold.
- `sh2-flash` also signs inside the fork's devshell.
- No document tells an operator to seal real firmware with 2.3.1. Two places still describe the fork moving to this picotool without the hold next to them (SPX-M2).

## Lens 6: upstream issue draft

Every line number below was checked against `2041936`, `ba3df40` and `2.2.0-a4`, and is correct:

- main.cpp: `5646`, `5807` (develop), `5573-5575`, `6180`, `6217`, `6229`, `6233`, `6394` (develop);
- bintool.cpp: `720`, `729`, `863`, `883`, `886` (both 2.3.1 and develop);
- `model.h:124`; `ERROR_NOT_POSSIBLE = -8` (`errors/errors.h:17`), which gives exit 248;
- 2.2.0-a4 `bintool.cpp:768` / `:780`;
- `3c743bd` is the first commit in 2.3.0 and is contained in 2.3.1.

There is one small imprecision (SPX-N4). Nothing in the draft is private: no keys, CHIPIDs, host paths, store paths or the private wallet repo's name. The only private-ish content is the "(Brian: …)" aside, which sits *inside* the body (SPX-M1).

## Findings

### SPX-M1 (Minor): an internal aside sits inside the issue body
**Evidence:** `picotool-upstream-issue-draft.md:138-140`. The draft says "Everything below the line is the proposed issue body" (`:4`), but the `der_to_raw` section opens with *"(Brian: this one may deserve its own issue … `sign-firmware.sh` takes the final signature from `picosign` …)"*. Posted verbatim, that would publish a personal note and internal tool names.
**Fix:** move the aside above the `---` line (or into an HTML comment), and make the `der_to_raw` section self-contained. Better still, split it into a second issue, as the aside itself suggests. While editing, replace "3 such signatures in 1500 seals" with a larger, theory-matching sample: 51 in 13,000 (0.39%, 2.3.1 + 2.2.0-a4) against an expected 2·2^-9.

### SPX-M2 (Minor): fork-move guidance without the hold next to it
**Evidence:**
- `PICOTOOL_PIN.md:112-120` ("How the fork and the Sitting image consume it") gives the fork the exact input snippet and says the devshell change "is its own PR in the fork thread", with no hold beside it.
- `IMPLEMENTATION_PLAN_e3a_refugium_otp.md:96-99` says "The fork thread gets the pin; its devshell change is its own PR".
- Plan §3's last bullet ("the fork thread is told") has no evidence in the implementer report.

The banner at the top of PICOTOOL_PIN does cover the first point, but a reader landing on that section (or a fork-thread agent handed the snippet) gets a do-this with no don't-yet.
**Fix:**
- Add one line under the snippet: "Held until bench R4 (see the HOLD banner and F-701); do not merge the fork's devshell change before then."
- Add the same parenthetical to the E3a plan line.
- The controller tells the fork thread about the hold, as plan §3 requires, and records that it did.

### SPX-M3 (Minor): `sh2-flash` flashes an existing `*.signed.uf2` without the Clear check
**Evidence:** `scripts/sh2-flash:270-293`. A `*.signed.uf2` input skips `sign-firmware.sh`, and the pre-flash assertion only greps `signature:.*verified` from `SIGINFO`. So the claim in `sign-firmware.sh:191-193`, that this is "the only place that catches it", also describes a hole. Examples that get through: a `.signed.uf2` made before this change, made elsewhere, or a renamed file. Each would be flashed with no Clear entry. The risk is low, because today's `.signed.uf2` files come from fork firmware sealed with `--clear`. But this is the operator's flash path, and `SIGINFO` is already captured.
**Fix:** source `scripts/lib/seal-check.sh` in `sh2-flash` and add `seal_has_clear_entry "$SIGINFO" || die …` next to the verified grep. This can be a separate small follow-up and does not block this PR. Record it in F-701.

### SPX-M4 (Minor): the `der_to_raw` defect is recorded only in the test and the issue draft
**Evidence:** `der_to_raw` appears in `seal-clear-test.sh`, the impl report and the issue draft. It does not appear in `FOLLOWUPS.md` (F-701 or a new entry) or in `PICOTOOL_PIN.md`. It is a defect in the pinned signing toolchain that silently produces a boot-ROM-rejected signature about 1 time in 256. Our pipeline is immune only because of a design property (picosign is the final signer). Nothing written down protects that property against a future change, for example someone adding a "sign directly with picotool" shortcut.
**Fix:**
- Add a short bullet to `PICOTOOL_PIN.md` ("What changed that matters: `seal`"): "picotool's own `seal --sign` signature is wrong ~0.4% of the time (`der_to_raw`, `mbedtls_wrapper.c:169/179`, all versions we use). Never use it as a final signature; `picosign` is the signer."
- Add a FOLLOWUPS entry or an F-701 bullet that tracks the upstream report.

### SPX-N1 (Nit): the Clear check is not bound to the signed metadata block
**Evidence:** `seal-check.sh:23-29`. The regex matches the line anywhere in `info -a`, which prints a load map per block (block 1 is unsigned and block 2 is signed). An image with the Clear entry only in an unsigned block would pass. Only a hand-built image could do this: picotool copies or derives the load map into the new block, and `sign-firmware.sh` already asserts exactly 2 blocks.
**Fix (optional):** scope the match to the block that carries `signature:` (for example, awk from the last `Metadata Block` header), or note the limitation in the lib's comment.

### SPX-N2 (Nit): the retry comment understates its own safety argument
**Evidence:** `seal-clear-test.sh:102-104` justifies the retry by the ~2^-240 match probability. The actual figure is ≲ 2^-223 across k = 1..4. The decisive property is a different one: a retry can only go green if a fresh seal verifies through the verifier's independent entry-order hash path, so a deterministic wrong digest can never be retried into green.
**Fix:** state that property in the comment. Optional extras: restrict k to 1..2 (k ≥ 3 has probability ≤ 2^-25 per signature), and print a summary count of retries.

### SPX-N3 (Nit): the one pipe into `grep -q` under `pipefail`
**Evidence:** `seal-clear-test.sh:226`, `grep -A1 … "$SF" | grep -qE '\|\|[[:space:]]*die '`. This is the F-695 shape. In theory it can only false-*fail* (a SIGPIPE on the producer), and in practice the 2-line output is written before the reader exits.
**Fix:** `w="$(grep -A1 -E … "$SF")"; grep -qE '\|\|[[:space:]]*die ' <<<"$w"`, for consistency with the repo's rule.

### SPX-N4 (Nit): line-number imprecision in the issue draft
**Evidence:** `picotool-upstream-issue-draft.md:63` says the Clear size word "is appended to `to_hash` (2.3.1 `bintool.cpp:870`)". Line 870 is `words_to_lsb_bytes`; the `back_inserter` append is `:871`. For the pin word the draft cites the append line (`:883`), so the two citations are inconsistent.
**Fix:** cite `:871`.

## Verdict on the deviation

`0de77eb` is sound, and I recommend keeping it:

- the bug is confirmed from source and causally by measurement;
- the retry cannot convert any deterministic wrong-signature defect into a pass;
- the hunk-B mutant stays red;
- nothing we ship depends on picotool's own signature.

No Critical or Important findings. The Minors are documentation and one adjacent hardening (`sh2-flash`). None of them blocks merge on its own, but SPX-M1 must be done before Brian posts the issue, and SPX-M2's fork-thread notice is a plan requirement.

VERDICT: 0C / 0I / 4M / 4N
