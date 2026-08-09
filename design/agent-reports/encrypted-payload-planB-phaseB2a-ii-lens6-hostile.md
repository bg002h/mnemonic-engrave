# B2a-ii whole-diff review — LENS 6: THE HOSTILE PAYLOAD

Reviewer: independent agent (opus), 2026-08-08.
Diff: `feat/encrypted-payload-b2a-ii`, `421dca8..HEAD` (10 commits: 4 implementation,
6 review folds), worktree `/scratch/code/shibboleth/seedhammer-wt-b2aii`.
Normative: `design/SPEC_encrypted_payload_delivery.md` (wins over plan and code).

**Lens:** §2.2 item 4 concedes the 64 KB blob is attacker-**writable** — it lies
outside the signed image's `LOAD_MAP`. The device MUST NOT assume a conforming
sealer. Everything below asks what a malicious region does to *this* phase.

**Verdict: 0 Critical, 0 Important. 1 Minor, 2 Nits.**

Nothing an attacker can write into that region reaches `Platform.LockBoot`, the
plate list, the engrave path, a KDF ahead of its bound checks, a panic, an
unbounded allocation, or a seed in the clear. Every probe below was **built and
run**, not reasoned about.

Out of scope per the brief and **not re-reported**: the wipe-lens findings
(C1/I1/M1/D1/D2/pass 3), F-83, F-86, F-87, F-88, F-89, the surviving
`clear(blob)` mutant, and "there is no idle timer" (that is B2b).

---

## What was executed

All work in a private copy (`/tmp/hostile-<pid>`, deleted); the shared worktree was
never written. Commands via
`nix develop /scratch/code/shibboleth/seedhammer --command go test …`.

| Probe | Cases | Result |
| --- | --- | --- |
| §10.2.1 allow-list on the DECRYPTED section | 8 (lock-boot at index 0 / 1 / middle / last-of-7; descriptor; mainnet address; junk) | all reject the **whole** payload, `ErrRecordNotPermitted`, `p == nil` |
| §6.4 container on the DECRYPTED bytes | 13 (trailing LF, leading LF, `\n\n`, CR interior, CRLF, CR at end, empty plaintext, 25 records, 513-byte record, space-grouped, hyphenated, uppercase, NUL) | all reject the whole payload |
| §6.2 header bounds before allocation/KDF | 13 mutated headers + 7 truncation lengths | all rejected, `countingKDF.calls == 0` in every case |
| ms1 / bare mnemonic / lock-boot / address in the PUBLIC section | 7 shapes × {sealed, unsealed} | all rejected; on the sealed shape **with no KDF run** |
| §6.4 container on the PUBLIC (pre-auth) section | 9 (incl. 8191 consecutive LFs) | all rejected; the 8191-LF case reports `8192 records` from the pre-split scan |
| §6.3 smuggled entropy in the clear | the spec's own example | rejected: `d-card: md: wire version mismatch` |
| cross-section record cap | G(12 public) + F(15 secret) = 27 | `ErrTooManyRecords: 27 records (12 public + 15 encrypted)` |
| **fuzz, pre-auth public surface** (`ParseHeader`→`SplitSection`→`Classify`→`cardKey`→`decodePublicSet`) | **40,547,912 execs / 150 s**, 768 corpus entries | 0 panics, 0 hangs |
| **structured BCH-valid hostile md1** (own bit-writer + `codex32.AssembleMD1`) | 256 payload patterns × 24 maximal chunks, one card | worst cumulative alloc **84,128 B**, worst wall **601 µs**; all rejected |
| **structured BCH-valid hostile mk1** (`codex32.MKChecksumSymbols`, regular + long) | 10 configs × 24 chunks, plus declared-count sweep 1/2/24/32 and a single-type card | worst cumulative alloc **85,072 B**, worst wall **667 µs**; all rejected |
| GUI end to end through `unlockPayloadFlow` | lock-boot at 3 positions in the encrypted section; 3 hostile public records; 9 decrypted container rules; the downgrade | no record reached `unlockEngraveHook` or `unlockSecretHook`; screens are "Payload unreadable." / "…more records than the machine accepts." |
| GUI control (anti-vacuity) | conforming vector C | **does** reach `SECRET seed material / ms1` — so the negatives above are not vacuous |
| mutation: allow-list only record 0 (`if i == 0 && !permitted(...)`) | whole seal suite | **killed by 5 existing tests**, incl. `TestEncryptedDebugCommandRejectsTheBundle` (lock-boot at index 2 of 6, encrypted section) |

---

## The six brief items, answered

**1. Is §10.2.1's allow-list applied per record on the DECRYPTED section, or only
to the first?** **Per record.** `seal/record.go:186-211` runs `Classify` +
`permitted` inside the loop over every record, and `permitted`
(`record.go:174-180`) returns false for `ClassDebugCommand` in *both* sections.
A plaintext of `command: lock-boot` at index 0, 1, mid-set and last-of-seven each
rejected the whole payload with `ErrRecordNotPermitted` and `p == nil`. The
"first record only" mutant is killed by the existing suite (measured above).
`Classify`'s branch order is byte-for-byte `gui/scan.go:56-79`'s, with the
`cmdPrefix` test **first** in both — so nothing Scan would call a `debugCommand`
can be re-classified here as something admissible.

**2. §6.4 container rules on the decrypted bytes — whole payload or dropped
record?** **Whole payload, every time.** 13 hostile containers, all rejected with
`p.Secret` never populated. Note two of the eight named shapes are enforced *by
the allow-list rather than the splitter*: a space-grouped and a hyphenated record
reject as `unknown format` (the codex32 engine's `inputChar` has no mapping for
`0x20` or `-`, so `ValidMD/ValidMK` are false) — which is what §6.4 and
`record.go:400-408` say, and it still rejects the whole payload. Uppercase is
caught earlier still, by `firstUpperASCII` before classification.

**3. §6.2 bounds BEFORE any allocation or KDF.** Confirmed by instrumentation,
not by return value: `openerWithCounter` reported `calls == 0` on all 13 mutated
headers and all 7 truncations, and the GUI test reported `installKDFCounter`
`calls == 0` and no word-entry screen when only the *public* section is hostile.
`ParseHeader` (`seal/wire.go:127-209`) allocates nothing on any failure path;
`Inspect` allocates only after it returns. The region read is bounded by
`clampRegion(RegionLen)` before any header field is trustworthy. `iterations` is
bounded to [100_000, 2_000_000] before `NewDeriver` is ever constructed, so the
no-watchdog hang argument holds. The region-fit check is genuinely unreachable
behind the 8191 caps (52+8191+8191+16 = **16450** ≤ 65536, computed) and the file
says so rather than pretending it is tested.

**4. ms1 or a bare mnemonic in the PUBLIC section.** Rejected — `ClassCodex32Secret`
and `ClassMnemonic` are admissible only when `section == SectionEncrypted`.
Verified alone, prepended and appended to a valid card set, on both the sealed
and unsealed shapes. On the sealed shape the rejection happens in `Inspect`,
before any KDF. §6.3's own smuggling example (`ValidMD == true`,
`md.Decode` fails) is caught by pass 3's card-set decode, which is the check the
spec says is load-bearing; the same record in the **encrypted** section is
admitted as `ClassMDMK` with `IsSecret == false`, which is what §6.3 prescribes.

**5. Can a hostile payload reach the plate list, the engrave path, or LockBoot by
any route this phase adds?** **No.** The two new engrave routes —
`unlockSecretSession → unlockEngraveCodex32/unlockEngraveMnemonic` and
`unlockPlateListFlow → unlockEngraveFlow` — are both fed exclusively from
`p.Public` / `p.Secret`, and both slices exist only as `AdmitSection`'s output.
Driven end to end through `unlockPayloadFlow` with a real sealed blob and the
real passphrase, no hostile record fired `unlockEngraveHook` or
`unlockSecretHook`. Nothing in this phase constructs a `debugCommand` or calls
`Scan`. The admitted classes partition cleanly — `ClassMDMK` → plate list,
`IsSecret` → secret session — so no admitted record is silently dropped either.
The §2.2 item 10 downgrade is visible end to end: a stripped vector D shows
`70f3 e35a …` (UNSEALED) and never `a26e d22b …` (SEALED), with `kdf.calls == 0`,
and the §10.2.3 warning naming "the encrypted part has been REMOVED".

**6. Integer/slice arithmetic on attacker-controlled lengths, 32-bit.** One Minor,
below. Everything else is safe with margin: `int` arithmetic in `Inspect` operates
on lengths `ParseHeader` has already capped at 8191 two lines above;
`SplitSection`'s separator count is bounded by the 8191 section cap;
`d.Done()*100` peaks at **200,000,000** against int32's 2,147,483,647.

---

## Findings

### M1 (Minor) — `UnlockWithKey`/`Unlock` bound-check the blob but not the header, and `int(uint32)` wraps negative on the 32-bit target

**Where:** `seal/unlock_key.go:31-40`; the same shape at `seal/open.go:183-199`.

**Defect.** Both functions recompute their offsets from `p.Header`, which is a
caller-supplied exported struct with exported `PubLen`/`CtLen`:

```go
end := HeaderLen + int(h.PubLen) + int(h.CtLen) + TagLen
split := HeaderLen + int(h.PubLen)
if len(blob) < end { ... }            // guards the BLOB, not the HEADER
plaintext, err := Open(key, h.IV[:], blob[:split], blob[split:end])
```

The guard's own comment reasons carefully about the *blob* ("Nothing forces the
caller to hand back the same blob") and about panics being bricks, but the header
it derives the offsets from is never re-validated. On this target `int` is 32-bit,
so `int(uint32)` reinterprets rather than widens, and the `len(blob) < end`
comparison is then made against a *negative* `end` and passes.

**Evidence (measured, `GOARCH=386 go run`):**

```
int is 32 bits; split=-2147483596 end=-2147483480
len(blob) < end ? false
PANIC: runtime error: slice bounds out of range [:-2147483596]
```

with `pubLen = 0x80000000`, `ctLen = 100`.

**Consequence.** A panic on a watchdog-less device is a brick until the operator
re-enters BOOTSEL. **Not reachable from a hostile payload:** `Inspect` is the only
in-tree producer of a `Payload` and `ParseHeader` caps both lengths at 8191 in
`uint64` arithmetic before the struct is built — which is why this is Minor and
not Important. It is nonetheless the one place in the new code where safety rests
entirely on a check made in a different function, on an exported API that B2b will
hold across a timer, and where the local guard was written believing it covered
the hazard.

**Fix.** One clause, mirroring `ParseHeader`'s own uint64 discipline, at the top of
`UnlockWithKey` (and, if kept, `Unlock`):

```go
if h.PubLen > MaxSectionLen || h.CtLen > MaxSectionLen {
    return fmt.Errorf("%w: header declares pub_len=%d ct_len=%d", ErrTooLarge, h.PubLen, h.CtLen)
}
```

A test can drive it directly: `UnlockWithKey` with a hand-built
`Header{PubLen: 0x80000000, CtLen: 100}` must return an error, not panic.

### N1 (Nit) — the int64-overflow threshold recorded in `unlockKDFLead` is wrong by ~461×

**Where:** `gui/unlock_kdf.go:153-155`.

**Defect.** The comment states "int64 overflows only past ~10^10 ns of elapsed
time" for `int64(elapsed) * int64(total-done)`. With `total-done` bounded by
§6.2's ceiling (≤ 1,999,999), overflow needs
`elapsed > (2^63-1)/1_999_999 = 4,611,688,324,271 ns` ≈ **4611.7 s ≈ 76.9 min**,
not 10 s.

**Consequence.** None to behaviour — the error is in the safe direction and the
expression cannot overflow in practice. It is a records defect: a future reader
sizing a change against "10 s" would conclude the expression is nearly at its
limit when it has three orders of magnitude of headroom, and this project's own
standing rule is that numbers in comments are measured, not estimated.

**Fix.** Replace with the computed figure, e.g. "overflow needs elapsed > 4.6×10¹²
ns (≈77 min) at §6.2's 2,000,000-iteration ceiling."

### N2 (Nit) — `d.Done()*100/d.Total()` is safe only because `MaxIterations` is 2,000,000, and nothing says so

**Where:** `gui/unlock_kdf.go:185`.

**Defect.** The percentage is computed in `int`, which is 32 bits on target.
`Done()*100` peaks at **200,000,000** against int32's **2,147,483,647** — safe,
with 10.7× headroom. But the headroom is a silent consequence of
`seal.MaxIterations`; raising that constant past **21,474,837** (computed) makes
the multiplication wrap and the screen render a negative percentage during the
one operation §10.2 step 7 exists to keep legible.

**Consequence.** None today. It is an undocumented coupling between a
`seal` constant and a `gui` expression, in the one place a hostile header's
`iterations` field is arithmetically consumed.

**Fix.** Either a one-line comment naming the dependency, or
`int(int64(d.Done()) * 100 / int64(d.Total()))`.

---

## Checked and found sound (no finding)

- **The allow-list is an allow-list**, not a deny-list, in both sections, and the
  irreversible branch (`cmdPrefix`) is evaluated first in both `Classify` and
  `Scan`. Verified by execution at four record positions and by mutation.
- **Rejection is whole-payload everywhere.** `AdmitSection` returns `nil` records
  on any failure and wipes the partial result; `Open`/`Inspect`/`Unlock` return
  `nil`. No probe produced a partially-accepted payload.
- **No KDF before the bounds**, asserted by instrument (`countingKDF`,
  `installKDFCounter`) at both the `seal` and `gui` layers, never by return value.
- **The pre-authentication decode surface is robust.** 40.5 M fuzz executions plus
  structured BCH-valid md1 and mk1 chunk sets (which random fuzzing cannot reach)
  produced no panic, no hang, and worst-case cumulative allocation of ~85 KB
  against ~452 KB free heap — bounded because `ValidMD`/`ValidMK` cap a record's
  data part at 93/108 symbols, so §6.4's 512-byte record cap is never the binding
  constraint and the concatenated chunk payload cannot exceed ~1.1 KB.
- **`pub_len`/`ct_len` are authoritative**; bytes past `52+pub_len+ct_len+16` are
  never read, and the AAD is exactly `blob[:52+pub_len]` taken from the blob's own
  bytes. Trailing UF2 padding and undefined sector bytes are inert.
- **The §6.6 hash and the plate list read the same bytes.** `p.Hash` is computed
  from the same `recs` that `AdmitSection` copies into `p.Public`, so there is no
  shape where the displayed digest and the engraved record can disagree.
- **The downgrade detector works end to end**, including the `sealed` byte, the
  public-record count (5, not 6), and the §10.2.3 wording.
- **The unsealed shape never prompts for a passphrase**, and a cancelled or failed
  unlock never falls through to the plate list.
- **`describeRecordCount`/`recordCountError`** distinguish "too many records" from
  "payload unreadable" at the UI, and no error string surfaced to the operator
  carries record content.

## Note for the plan owner (not a finding)

§6.3's DECODE requirement raises the bar on smuggling secret bytes through the
public section from "arbitrary bytes" to "must decode as a real descriptor / key
card with a matching chunk-set id" — it does not reduce the bandwidth to zero,
since a descriptor's own key material is attacker-chosen. The spec already says
so explicitly ("Be precise about what this does and does not prevent"), the code
implements the spec faithfully, and an *attacker* has no motive to smuggle their
own seed. Recorded only so a future reader does not mistake the decode for a
proof of non-secrecy.
