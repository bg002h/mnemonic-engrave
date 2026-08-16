# S6c — REQUIREMENTS CAPTURE: the passphrase gets a plate, and the plates say so

**Status:** requirements capture only. **Not a spec, not a plan, not gated.** No
code may be written against this. Its job is to make the operator's directives
outlive the conversation they were given in, and to record what has already been
measured so a later spec does not re-derive it.

**Owning phase:** S6c — its own cycle, with its own spec and R0, scheduled
**before the hardware flash**, so S6 validates the new plate layout on real steel
in the same flash cycle. Sequencing, not deferral.

**Why not folded into S6a:** S6a closes F-198's Critical with text and control
flow only. Everything here changes what the machine physically cuts, which is
exactly what a hardware gate exists to validate.

---

## 1. OPERATOR DIRECTIVES — verbatim, 2026-08-16

Recorded word for word, because a paraphrase of a decision is a decision lost.

1. > "Keep it somewhere separate" is the right approach. But we can offer to
   > engrave it and we should mark the associated keys and descriptor plates as
   > associated within a password…we need to verify the keys and descriptor do
   > include passphrase.

   — followed by the correction: **"associated WITH a password"**.

2. > Title is the perfect place for passphrase required notice

3. > SeedFP and combined FP are also perfect for title or footer

4. > Password, if user chooses to engrave it, should be on its own plate, with
   > associated key-id and wallet policy id in title or footer. QR code should
   > include only password

### What those resolve to

| # | requirement | status |
| --- | --- | --- |
| R1 | The restore document keeps saying to keep the passphrase somewhere separate | **already shipped** — `buildPassphraseInventoryLines` |
| R2 | The device **offers** to engrave the passphrase | new flow work |
| R3 | mk1 and md1 plates are marked as requiring a passphrase, in the **title** | new plate work |
| R4 | Seed FP and combined FP appear in **title or footer** | new plate work |
| R5 | An engraved passphrase is on **its own plate** | **already true** — `engravePassphraseFlow` cuts a dedicated plate |
| R6 | That plate carries the associated **key-id** and **wallet policy id** in title or footer | new plate work |
| R7 | Its **QR contains only the password** | **already true** — pin it with a test |

---

## 2. MEASURED FACTS — do not re-derive these in the spec

Every item read out of the fork at `main` = `b8a23bf`.

### 2.1 The keys and descriptor DO bind the passphrase — proven by bytes

The operator asked for this to be verified. It was, by deriving the same
mnemonic twice and comparing output, not by reading the call graph:

| artifact | bare seed | same seed + `"abandon about"` | |
| --- | --- | --- | --- |
| `ms1` | `ms10entrsqqq…cj9sxraq34v7f` | `ms10entrsqqq…cj9sxraq34v7f` | **identical** |
| master fp | `73c5da0a` | `fc60c6df` | **differs** |
| `mk1` | `mk1qph25epq…` | `mk1qpl36ypq…` | **differs** |
| `md1` | `md1fgdxlpqp…` | `md1faxr8pqp…` | **differs** |

So `ms1` encodes the **words only**; `mk1` and `md1` are passphrase-bound
through `deriveAccountXpub` (`gui/singlesig_derive.go:10`), and `ms1` is built
from mnemonic entropy alone (`:87`, `codex32.EncodeMS1(entropy)`).

**The consequence R4 exploits:** restoring the words alone yields a *different*
fingerprint from the one the key and descriptor plates encode. That mismatch is
already detectable on steel today — nothing merely says to look, or what it
means. R3+R4 turn a silent wrong-wallet restore into a self-diagnosing one.

### 2.2 No codec change is required, so the Rust-primary rule is NOT triggered

All marking is plate **layout and text**. The `mk1`/`md1`/`ms1` strings stay
byte-identical, so nothing here leads the primary Rust implementation. This must
be re-confirmed by the spec, since it is what keeps S6c a fork-native cycle.

### 2.3 There are FOUR different plate-text mechanisms, with different budgets

This is the spec's first design call, and it must precede any wording, because
the length budget differs per mechanism.

| mechanism | carries | length rule | used by |
| --- | --- | --- | --- |
| `Fitted.Title` / `.Footer` (`backup/fit.go:117-121`) | title at plate row 0, footer at the last row, at the screw-hole rows | **`MaxTitleLen = 18`, SILENT truncation** via `TitleString` (`backup/backup.go:98`) | free-text plate, preview |
| `Seed.Title` / `SeedString.Title` (`backup/backup.go:17,27`) | a title | rendered `strings.ToUpper` at `:223`, `:311` | codex32 / seed-share plates |
| the passphrase plate's own `topLines` / `bottomLines` banding | arbitrary bands | **no 18-char cap** — its footer is 32 chars | `backup/passphrase.go` |
| `Text.Paragraphs` | paragraphs only — **no title, no footer** | n/a | **`mk1` and `md1`**, via `validateMdmk` (`gui/gui.go`) |

**mk1 and md1 use the only mechanism with no title or footer at all.** R3 and R4
therefore require either giving `Text` a title/footer band or routing those
plates through a mechanism that has one. That choice is S6c's first gate.

### 2.4 The 18-character cap is a real trap, and truncation is silent

`TitleString` (`backup/backup.go:98-110`) stops at exactly `MaxTitleLen = 18`
and **also silently drops any rune the face cannot decode**.

    19  PASSPHRASE REQUIRED   -> TRUNCATES to 'PASSPHRASE REQUIRE'
    17  PASSPHRASE NEEDED        fits
    16  NEEDS PASSPHRASE         fits
    18  SEED FP: 73C5 DA0A       fits EXACTLY AT THE CAP
    18  COMB FP: FC60 C6DF       fits EXACTLY AT THE CAP
    27  EXPECTED COMB FP: FC60 C6DF   band only

**The operator's own phrasing, "PASSPHRASE REQUIRED", is 19 characters** and
would engrave as `PASSPHRASE REQUIRE`, permanently, with no error. Two of the
fingerprint forms sit *exactly* on the cap, which is a fragile place to live: one
character added by a later edit truncates onto steel.

**So the length budget is a first-class gate in S6c, not a wording detail** — a
test must assert the budget, not merely the current string.

Fingerprints are grouped by the house helper: `passphrase.GroupFingerprint`
splits at 4 (`73C5DA0A` → `73C5 DA0A`), so every count above is of the grouped
form.

### 2.5 The passphrase plate already does most of R5, R6, R7

`backup/passphrase.go`:

- **R5 — its own plate:** yes. `engravePassphraseFlow` (`gui/passphrase_flow.go:605`)
  is a separate top-level program cutting a dedicated plate.
- **R7 — QR is password-only:** **yes, already.** `:86` is
  `qr.Encode(plate.Passphrase, qr.L)` — the passphrase and nothing else. The
  work is to **pin this with a test**, so no future edit folds metadata in.
- **R6 — identifiers in title/footer:** partially. It carries `SeedFP` and
  `CombinedFP` as `topLines` (`:176-180`) plus a footer (`:156`). It does **not**
  carry a key-id or a wallet-policy id.

**One sentence on that plate cannot be reused, and the asymmetry is the point.**
Its footer reads `FINGERPRINTS TYPED, NOT VERIFIED` — true there, because the
operator typed them. On `mk1`/`md1` the device **derived** those fingerprints, so
those plates may legitimately vouch for their own. Copying the string across
would understate what the device knows.

### 2.6 Which identifier fits, for R6

- `md.WalletPolicyId` (`md/walletpolicyid.go:30`) is **16 bytes → 32 hex** —
  far past any title budget.
- `md.WalletPolicyIDStub` (`:106`) is **4 bytes → 8 hex**, groups as
  `XXXX XXXX` exactly like a fingerprint, and is **already the binding `mk1`
  carries** (T6a spec: the bundle's mk1 stub = `WalletPolicyIDStub(md1)`,
  non-zero and policy-bound).

The stub is the candidate that fits. `mk.Header.ChunkSetID` (`mk/mk.go:50`, a
20-bit chunk-set id) is the other identifier in play and the spec must decide
which of the two is the "key-id" the operator means.

---

## 3. OPEN QUESTIONS FOR THE SPEC — not answered here

1. **Which plate-text mechanism** do `mk1`/`md1` move to (§2.3)? Everything else
   depends on it, including the length budget.
2. **Does a title/footer actually FIT** alongside the existing text+QR on an
   `mk1`/`md1` plate at current sizes? Unverified, and measurable. Engraving
   feature-size and bounding-box limits apply.
3. **Which identifier is the "key-id"** — the wallet-policy stub, or the mk1
   chunk-set id (§2.6)?
4. **Does marking apply to watch-only sets too?** They carry `mk1`+`md1` and no
   seed, and a watch-only build can still be passphrase-derived.
5. **Do the multisig paths get the same marking?** They have the same defect and
   more plates.
6. **What happens to the goldens?** Plate rendering has golden tests; any layout
   change churns them, and a churned golden is only as good as its review.

---

## 4. WHAT THIS IS NOT

- Not gated, not reviewed, and **not implementable**. It is input to a spec.
- It does **not** change S6a, which closes F-198's Critical independently.
- It does not decide sequencing against S6b (F-199 + F-204).
