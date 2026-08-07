# R0 round 1 — `SPEC_encrypted_payload_delivery.md`

Reviewer: fable, architect R0 gate. Dispatched 2026-08-07.
Verdict: **0 Critical / 4 Important / 2 Minor / 2 Nit — GATE BLOCKED.**

Persisted verbatim before folding, per project standard.

---

Analysis complete. All firmware claims cited in the spec check out against source; the canonical vector recomputes exactly. Findings follow.

---

### [IMPORTANT] §6.2 normatively rejects `payload_kind = 0x02`, contradicting §6.3, §11.4 and §12.6
**Location:** §6.2 bullet 2 (`payload_kind == 0x01`) vs §6.3 table, §11.4 (vector has kind `0x02`), §12 item 6 ("implement `0x01` and `0x02` only")
**Defect:** The normative bounds list admits only `payload_kind == 0x01`. Three other normative sections say v1 ships `0x01` and `0x02`, and the canonical cross-implementation test vector itself carries `0x02`.
**Failure scenario:** The Go port implements §6.2 as written (it is the section titled "checked BEFORE any allocation or KDF work") → every BIP-39-mnemonic blob, including the canonical vector, fails device-side with "payload unreadable". The two implementations resolve the contradiction differently: Rust seals kind `0x02` per §9, the device rejects it per §6.2. The vector test would eventually catch it, but two codebases would be written against contradictory normative text.
**Fix:** §6.2: `payload_kind ∈ {0x01, 0x02}`; `0x03` rejected until §12 item 6 is signed off (state that the admission set widens to `0x03` only on sign-off).

### [IMPORTANT] Unconstrained `--addr` flag can overwrite firmware or wrap past physical flash
**Location:** §9 synopsis (`[--addr ADDR]`); contradicts §5 and §10.1
**Defect:** §5 derives `0x10E00000` precisely so the blob cannot touch the signed image or wrap past `0x11000000` (the spec's own citation of datasheet §5.5.2: writes past physical flash wrap to `0x10000000` and destroy the firmware). §9 then exposes an operator-facing flag that discards that entire analysis. The device reads only `0x10E00000` (§10.1), so every other address is also useless — the flag has no legitimate value and one destructive one.
**Failure scenario:** `me seal … --addr 0x10FFF000` with any blob > 4 KiB → `picotool load` writes past `0x11000000` → wraps to `0x10000000` → signed image corrupted → machine unbootable until the operator reflashes signed firmware. Or `--addr 0x10000000` overwrites the image directly. Even a "safe" wrong address silently produces a blob the device will never see.
**Fix:** Delete `--addr` from §9. The target address is normative (`0x10E00000`), fixed by §5 and §10.1. If a test seam is ever needed, it must not be an operator-facing flag.

### [IMPORTANT] Decrypted plaintext is routed into a classifier whose acceptance surface includes an irreversible OTP-write command
**Location:** §10.2 step 6; `seedhammer/gui/scan.go:57-59` (`command: ` prefix → `debugCommand`), `gui/gui.go:1668-1681` (dispatch, incl. `lock-boot`), `cmd/controller/platform_sh2.go:545-553` (`LockBoot` → `writeOTPValues()` + `otp.EnableSecureBoot()` + `machine.CPUReset()`)
**Defect:** Step 6 hands plaintext to "the same entry point NFC scans use" with no restriction on the result type. That switch accepts more than the three §6.3 forms: a plaintext beginning `command: ` dispatches to debug commands, one of which performs irreversible OTP writes and resets the CPU; it also accepts Bitcoin addresses and output descriptors. The device-side spec relies entirely on host-side §9 validation to keep those inputs out — but the wire format is normative and public, so the device cannot assume the sealer was the conforming `me` CLI.
**Failure scenario:** A third-party or buggy sealer (the format is published; §9's validation lives only in one implementation) seals the string `command: lock-boot`; the operator types that blob's passphrase → the unlock flow executes `LockBoot()`: OTP white-label writes, `AddBootKey`, `CRIT1` secure-boot bit, then CPU reset mid-flow. On the operator's device these bits are already burned (mostly a surprise reboot); on any other fork device it permanently burns boot keys. An encrypted-delivery path must not have a reachable edge into OTP writes.
**Fix:** §10.2 step 6: the unlock flow MUST accept only classifier results corresponding to §6.3 kinds (BIP-39 mnemonic, codex32, md/mk text) and MUST treat any other classification — explicitly including `debugCommand`, addresses, and descriptors — as "payload unreadable", fail closed.

### [IMPORTANT] Two of §11.3's five mandatory mutations survive the entire specified test set
**Location:** §11.3 vs §11.1/§11.2/§11.4
**Defect:** §11.3 mandates that each listed mutation be caught, but the specified tests cannot catch two of them: (a) *salt reused across two seals* — no specified test performs two seals and compares outputs; the round-trip test and the fixed-salt vector both pass under a frozen salt. (b) *iteration count read as a constant* — the only vector uses `iterations = 100000`; a mutant hardcoding 100000 passes the positive vector (correct key, unmodified AAD) **and** passes the altered-to-50000 negative case, because the altered header changes the AAD, so the tag mismatches regardless of which iteration count the KDF used. I verified this logic against the recomputed vector: the 50000-header decryption fails on AAD alone even with the correct key.
**Failure scenario:** Exactly the project's documented false-PASS pattern: mutation testing is run, these two mutants survive, and either the gap is noticed late or the mutation step is (wrongly) recorded as green. The frozen-salt mutant is the catastrophic one — it is the precise defect §11.4's warning box exists for (the test-vector salt seam leaking into production), and nothing in the suite would see it.
**Fix:** Add to §11.1: (a) a freshness test — two `seal` invocations of the same plaintext MUST yield distinct salt, IV, mnemonic, and ciphertext; (b) a second decrypt vector (or round-trip) at a different iteration count, e.g. 100001, that MUST succeed — this fails under any hardcoded count on either side. Optionally also instrument the KDF to assert it was invoked with the header's value.

### [MINOR] `ct_len = 8192` decrypts but cannot classify: the scan buffer overflows at exactly 8 KiB
**Location:** §6.2 (`ct_len <= 8192`) vs `seedhammer/gui/scan.go:29-36` (`s.buf = make([]byte, 8*1024)`; `s.overflow = s.n == len(s.buf)`)
**Defect:** `Scan` flags overflow when the buffer is exactly full, so a maximal in-bounds payload passes every §6.2 check, burns the ~30 s KDF, authenticates, then dies in the classifier with `errScanOverflow`. Fail-closed, no security impact, but a spec-legal blob that can never engrave.
**Failure scenario:** Host seals an 8192-byte payload (spec-legal) → device decrypts successfully → classifier reports overflow → "unreadable" after a successful authentication. Confusing, not dangerous.
**Fix:** Set the §6.2 bound to `ct_len <= 8191` (or below), or specify that the unlock flow bypasses `Scan`'s buffer and invokes classification directly.

### [MINOR] `payload_kind` is advisory on-device: the `ms1` gate does not bind content
**Location:** §6.3, §10.2 step 6, §12 item 6; `seedhammer/codex32/codex32.go:98` (`New` accepts secret shares)
**Defect:** The device routes by *content* (the classifier), not by `payload_kind`, and `codex32.New` accepts `ms1` secrets. So the device-side `0x03` rejection only checks a header byte the sealer controls: `ms1` content labeled `0x01` passes bounds, decrypts, and engraves the seed — the §12.6 policy gate is enforced solely by the conforming host. No attacker leverage (AEAD prevents forgery, and the at-rest exposure is identical either way), but the spec reads as though the device enforces the gate, and it does not.
**Failure scenario:** A CLI classification bug (or third-party sealer) labels an `ms1` string `0x01` → device engraves a seed while §12.6 is unsigned. Encrypted at rest throughout, so no funds/key exposure — a policy-gate bypass only.
**Fix:** §10.2: after decrypt, the device MUST verify the classified content type is consistent with `payload_kind`, and MUST reject `ms1`-classified plaintext while §12 item 6 is unsigned, regardless of the header byte.

### [NIT] UF2 payload padding byte is pinned only by the vector's sha256
**Location:** §9.1, §11.4 "Loadable form"
**Defect:** The 207-byte blob occupies a 256-byte block payload; the padding byte is never stated. I verified the stated UF2 sha256 matches zero-padding (0xFF-padding does not match). An implementer padding with 0xFF fails the sha test with no prose explaining why.
**Failure scenario:** None beyond a confusing test failure; the device bounds all reads by `ct_len`.
**Fix:** §9.1: "payload bytes beyond the blob are `0x00`."

### [NIT] §6.2 arithmetic should be overflow-proof and the bound-violation tests should include a huge `ct_len`
**Location:** §6.2, §11.2
**Defect:** `48 + ct_len + 16` computed in 32-bit signed arithmetic (TinyGo `int` is 32-bit) wraps negative for `ct_len` near 2³², passing `<= 65536`. A conforming implementation is safe only because the separate `ct_len <= 8192` check catches it — the region-fit check alone is bypassable.
**Failure scenario:** None for a spec-conforming implementation (both checks are mandated); a port that "simplifies" to the region check alone in native int would accept a 4 GiB declared length.
**Fix:** State that the length arithmetic MUST be performed unsigned/wider-than-32-bit or checked, and add `ct_len = 0xFFFF_FFF0` to the §11.2 bound-violation cases.

---

Questions from the brief with **no finding**, and why:

1. **AEAD one-key-one-message (§7.2): the structural argument is airtight as specified.** Every encryption ever performed is one host-side `seal` invocation drawing fresh salt+IV from the OS CSPRNG; the device never encrypts (and cannot — no RNG); re-seal regenerates both; backup/restore and load-retry replay an existing ciphertext, which is not a second encryption; a `--iterations` change is a new invocation with fresh salt; same-plaintext-twice gets two salts hence two keys. Key collision requires a 128-bit salt collision (negligible), and even under a repeated key the fresh random 96-bit IV independently satisfies SP 800-38D §8.2.2 for the pair. The only way to break it is the frozen-salt implementation defect — which is why Important #4(a) (the missing freshness test) gates. GCM's lack of key-commitment is immaterial here: partitioning-oracle attacks require a low-entropy password candidate set, and the passphrase is a mandated 128-bit generated mnemonic.
2. **AAD binding is complete.** All 48 header bytes are positionally authenticated: version, algorithm IDs, kind, iterations (downgrade → tag mismatch, verified by execution), salt, IV, `ct_len` (truncation/extension → tag position moves → mismatch). Header-A + ciphertext-B splices fail. Cross-device replay is unbound but buys an attacker nothing under the ciphertext-is-published model — the passphrase is the gate.
3. **Bounds/hang:** iterations capped at 2M ≈ 133 s worst case — bounded, not a hang; allocation ≤ 8 KiB against ~452 KB free; all reads confined to the 64 KiB region well inside physical flash. Checked before KDF per §10.2 step 1.
4. **Checksum-gate-before-KDF is not an oracle.** BIP-39 checksum validity is a public predicate of the *candidate* — no secret-dependent branch. §8.1 normalisation is sufficient: both sides emit ASCII wordlist words joined by single spaces, and the device keyboard is wordlist-gated, so byte-identity is structural; the vector pins it.
5. **Test vector: verified correct by independent recomputation** (Python `hashlib.pbkdf2_hmac` — same primitive family, independent AESGCM implementation): derived key, tag, 207-byte blob hex, blob sha256, and UF2-block sha256 all match exactly; round-trip decrypts; the iterations-downgrade negative behaves as specified.

## VERDICT
Critical: 0   Important: 4   Minor: 2   Nit: 2
GATE: **BLOCKED**

**CONFIDENCE.** Verified by execution: the full §11.4 vector (key, ciphertext, tag, blob hex, both sha256s, round-trip, AAD downgrade rejection) and the survivability of the two §11.3 mutants against the specified tests. Verified by reading firmware source: the classifier's full acceptance surface including `debugCommand → LockBoot → OTP writes` (scan.go, gui.go:1668, platform_sh2.go:545, otp.go:93), `codex32.New` accepting secrets, the 8 KiB scan-buffer boundary, `rp.WATCHDOG` appearing only in the BOOTSEL-reboot scratch writes, and the cited CLI anchors (`lib.rs` ms1 refusal, `main.rs:375 write_private`). Not independently checked: the settled facts (per brief), the RP2350 flash-wrap behavior (taken from the spec's own datasheet citation and settled empirical work), and TinyGo-specific codegen questions the spec itself defers to implementation-time tests. The four Importants all have one-sentence-to-one-paragraph fixes; none undermines the core construction, which I consider cryptographically sound as specified.

---

## Controller verification note

Two findings were independently confirmed against source before folding:

- **Important #3** — `gui/scan.go` `cmdPrefix = "command: "` → `debugCommand`;
  `gui/gui.go:1668` `case "lock-boot": ctx.Platform.LockBoot()`. Confirmed.
  The classifier's full acceptance surface also includes output descriptors and
  mainnet/testnet addresses, i.e. wider than the three §6.3 kinds.
- **Minor #1** — `s.buf = make([]byte, 8*1024)` with
  `s.overflow = s.overflow || s.n == len(s.buf)`. Overflow triggers at exactly
  8192. Confirmed.
