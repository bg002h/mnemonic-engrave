# Hashlock H0 — post-implementation adversarial execution review

**Verdict: NOT GREEN. 1 Critical / 2 Important / 4 Minor / 1 Nit.**

Reviewer: independent opus execution reviewer, brief
`design/agent-briefs/hashlock-H0-post-impl-brief.md`.
Tips reviewed: mnemonic-engrave `hashlock-h0` **265dc8e** (base master `e7af98a`),
seedhammer fork `hashlock-h0` **14afdff** (base main `839fa5aa`).

Everything below was executed. My own copies (removed after the run):
- engrave: `git worktree add --detach /scratch/code/shibboleth/me-worktrees/h0-review 265dc8e`,
  `CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/h0-review-target`, `TMPDIR=/scratch/code/shibboleth/.tmp`.
- fork: `git ls-files -z | tar` export into `/scratch/code/shibboleth/.tmp/h0-review-fork`,
  Go 1.26.7 at `/scratch/code/shibboleth/.toolchain/go`.

Neither branch worktree, neither repo, and nothing under
`.tmp/seedhammer-hashlock-h0/cmd/emu` was written. Every mutation was applied to
a copy, observed, restored from a byte backup and `touch`ed; both copies verified
pristine at the end (`git status --porcelain` empty on the engrave worktree;
`diff -r` clean between the fork copy and its source).

---

## The one question

> Can you construct a string, a payload, or a sequence of screens under which the
> flashed device or `me` still treats a preimage plate as a seed — or under which
> a legitimate seed, share or plain BIP-93 secret is now refused — and does every
> test the diff adds actually fail on the defect it names?

**Yes to the first (C-1: a sequence of screens), yes to the second in a narrow
population the code says is impossible (I-1), and yes — every one of the eleven
tests the diff adds or edits fails on the defect it names (mutation table
below; no survivors).**

---

## Findings

### C-1 — `engraveCodex32`'s guard runs once, before a loop that reassigns the object it guards: Recover reaches `Confirm Codex32 Secret` and `Engrave Plate` for a preimage single

**Site:** fork `gui/codex32_polish.go:219-227` (the added guard) and `:229-247`
(the pre-existing loop).

```go
func engraveCodex32(ctx *Context, th *Colors, scan codex32.String) bool {
	if codex32.IsPreimage(scan) {
		showError(ctx, th, "Hashlock preimage", "...")
		return true
	}
	for {
		switch confirmCodex32Flow(ctx, th, scan) {
		...
		case codex32Recover:
			secret, ok := recoverCodex32Flow(ctx, th, scan)
			if !ok {
				continue
			}
			scan = secret // recovered unshared secret; loop re-confirms it
			continue
		case codex32Engrave:
			id, _, _ := scan.Split()
			s := backup.SeedString{Title: id, Seed: scan.String(), Font: constant.Font}
			backupSeedStringFlow(ctx, th, s)
```

`scan = secret` reassigns the guarded variable **inside** the loop; the
`IsPreimage` test is outside it. `Interpolate(shares, 'S')` on a K-of-N set of a
`0x03`/33-byte payload returns exactly a preimage single — measured — so the
loop then hands it to `confirmCodex32Flow` (which titles it
`Confirm Codex32 Secret`) and, on the next tap, to `backupSeedStringFlow` →
`backup.EngraveSeedString`.

The guard's own comment states the assumption that fails:

> `// Both doors that hand a codex32.String to engraveObjectFlow -- the NFC scan`
> `// and the typed M*1 STRING -- end here, and this is the call that titles the`
> `// plate and cuts it.`

True for the two doors; false for the object the Recover arm manufactures, which
never passes through `engraveObjectFlow` at all.

**Reproduction (executed, `gui` package, my fork copy):** a 2-of-n set was built
with the fork's own `codex32.NewSeed` + `Interpolate` over a 33-byte payload
beginning `0x03` under the id `hash`, and one share was handed to
`engraveObjectFlow` (i.e. the ordinary NFC / typed-`M*1 STRING` door — a share
is correctly *not* a preimage, so nothing refuses it). Verbatim output:

```
first  share = ms12hashcm7nwxhky4jx7xxu80vrhwt490vrs2xvuf4cqw89vd5ax7eundufxptf4e7vtfxlsfe (IsPreimage=false)
second share = ms12hashd3jvffkjymrtenc5rtjj4u7wvt02qlfytqc5spg73fh7yt75hfzcmj06747wdewgvjz (IsPreimage=false)
secret       = ms12hashsqvqsyqcyq5rqwzqfpg9scrgwpugpzysnzs23v9ccrydpk8qarc0jq9get7tzc6sn5y (IsPreimage=true)
screen 1: "ConfirmCodex32ShareidHASHShareCofak-of-nsetEngravethisshare,orRecoverthesecret75chars"
screen 2: "23456789WERTYUIOSDFGHJKZXCVBNM0charsShare2of2|idHASH"
REACHED THE SECRET CONFIRM SCREEN FOR A RECOVERED PREIMAGE (frame 0): "ConfirmCodex32SecretidHASHUnsharedsecret(S)75chars"
AFTER TAPPING ENGRAVE (frame 0): "Insertablankplateandclosethelock.Holdbuttontostarttheengravingprocess.Theprocessisloud,usehearingprotection.EngravePlate"
```

The screen sequence is: Confirm Codex32 **Share** → tap Recover (Button2) → type
share 2 on the keypad → tap OK (Button3) → **`Confirm Codex32 Secret`** → tap
Engrave (Button3) → **`Engrave Plate`**. The goroutine stack at that point,
captured from the same run under `-timeout 25s`:

```
seedhammer.com/gui.(*EngraveScreen).Engrave(...)  gui/gui.go:3327
seedhammer.com/gui.backupSeedStringFlow(...)      gui/gui.go:2825
seedhammer.com/gui.engraveCodex32(...)            gui/codex32_polish.go:242
seedhammer.com/gui.engraveObjectFlow(...)         gui/gui.go:2556
```

`gui/codex32_polish.go:242` is the `backupSeedStringFlow` call in the
`codex32Engrave` arm — the one call site of `backupSeedStringFlow` in the whole
tree (`grep -rn "backupSeedStringFlow(" --include=*.go .` returns exactly it and
the definition). So this is the same metal-cutting call the guard was added to
protect, reached with a preimage single, on the shipped tip.

**What is violated.** `SPEC_ms_hashlock.md` §12 item 7, verbatim: *"A `0x03`
single fed to the flashed device is INERT — `sysw.Classify` is not
`ClassCodex32Secret` and no engrave path offers it."* An engrave path offers it.
Also §9's H0 clause (*"makes a `0x03` string INERT"*) and the plan's own Goal
(*"Make a kind-`0x03` hashlock PREIMAGE plate string inert on every reader that
would today take it for a seed"*).

**Reachability, stated honestly.** This needs a K-of-N share set of a preimage.
The spec contemplates exactly that and says the codec supports it —
§1: *"The share axis is untouched. Threshold and index live in the codex32
header, orthogonal to the prefix byte; a K-of-N set of a preimage recovers to a
`0x03` payload and the codec supports it."* — and `ms split` has no ms1 source
today (F-468), so today the shares must come from third-party BIP-93 tooling or
from the fork's own `cmd/biptool`. It becomes ordinary at H2, when the device
learns the kind. The reproduction above needed no special hardware: one scan or
typed string for share 1, the on-screen keypad for share 2.

**Suggested shape of the fix is not prescribed** — but note that moving the test
to the top of the `for` body (or into the `codex32Recover` arm after
`scan = secret`) is a one-line change, and that the same defect class is *absent*
on the sealed path, where `unlockEngraveCodex32` deliberately does **not** reuse
`engraveCodex32` and re-tests after its own `codex32.New`.

**A test that would have caught it does not exist.** No test in the diff drives
the Recover arm. `TestEngraveCodex32RefusesAPreimagePlate` enters
`engraveObjectFlow` with a preimage and asserts the refusal — it can never reach
the loop.

---

### I-1 — the guard narrows a population the shipped code states it cannot touch: a plain BIP-93 33-byte master seed beginning `0x03`

**Site:** fork `codex32/mspayload.go:62-89` (the doc comment) and the plan's
Global Constraints.

The doc comment on `IsPreimage` says, verbatim:

> `// A plain BIP-93 secret whose seed begins 0x03 has a 16..32-byte payload and`
> `// is untouched.`

and the plan repeats it (*"a plain BIP-93 secret's data part is the seed
itself"*, with the enumerated collision limited to shares). **Measured false.**
The fork's own codeword bracket is 16..44 payload bytes, and the corpus's own
`bip93-plain-33-byte-payload-0x31` row is a *33-byte unshared plain BIP-93
payload* asserted `device_admits: true`. Change that row's first byte from `0x31`
to `0x03` and the device now refuses it.

**Executed** (`seal` package, my fork copy):

```
ms10seedsqvrsu9guyv4rzwplgex4gkmzd9c8wl593jfe4gdg47mtm3xt6tv7q2upgk0t0zq6j6
  seed[0]=0x03 len=33 IsPreimage=true seal.Classify=unknown format
ms10testsqvrsu9guyv4rzwplgex4gkmzd9c8wl593jfe4gdg47mtm3xt6tv7qh3pm4xrfdlvvp
  seed[0]=0x03 len=33 IsPreimage=true seal.Classify=unknown format
ms10testsxy0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5dayejmh0wrfk
  seed[0]=0x31 len=33 IsPreimage=false seal.Classify=codex32 secret
AdmitSection(entr-128 + plain 33-byte 0x03 seed) admitted=0 err=seal: record classification not permitted in this section: record 1 classifies as unknown format, which the encrypted section does not permit
```

and at the scan / sysw doors (`gui` package, same copy):

```
plain 33B 0x03 (id seed)   sysw.Classify=ClassUnknown   Scan=<nil>/scan: unknown format
33B 0x31                   sysw.Classify=ClassCodex32Secret  Scan=codex32.String/<nil>
```

So the operator-visible outcome for a real 264-bit BIP-93 master seed whose first
byte is `0x03` (≈1 in 256 of 33-byte seeds) is: the scan says *Unknown format*,
the typed door says *"This record is a hashlock preimage, not a seed"* about
their actual seed, and one such record refuses a whole encrypted section — the
exact whole-payload failure the plan's fidelity C-2 identified for shares and
then designed out only for shares.

**Why Important and not Critical.** The narrowing is a *deliberate, R0-reviewed
design decision* — the plan's Global Constraints say "The id is NOT consulted:
the kind is the prefix byte (§1)", and consulting the id would reintroduce the
tests-I-1 hazard of engraving a mistagged real preimage. The refused population
is one the Rust primary already refuses (`me` at ms-codec 0.7 refuses every
`0x03` single under every id — measured below), so this is convergence, not
divergence; and the failure mode is a refusal, never a wrong cut. What is
defective is the **claim that it cannot happen**, sitting in shipped code where a
maintainer will rely on it, plus the absence of any corpus row for the true
collision, which leaves the narrowing untested and undocumented. If the brief's
"a legitimate ... BIP-93 secret refused" clause is read literally this is the one
Critical candidate; I am filing it Important and naming the reading so the
controller can rule.

**Records that inherit the same falsehood:** the `mspayload_test.go` table
comment (*"dropping `len(d) == 33` calls the plain 16-byte BIP-93 row one"* — the
16-byte row is a control against over-width, not evidence the 33-byte case is
impossible), and `design/FOLLOWUPS.md` F-472, which lists what the device still
admits as *"plain BIP-93 secrets at 48 and 74 characters"* and never mentions the
75-character plain-BIP-93 case that H0 just removed.

---

### I-2 — `me seal` refuses a preimage plate with the raw codec error, so the named diagnosis H0 added exists on only one of the two host verbs

**Executed** (my engrave worktree, `debug/me` built at `265dc8e`):

```
$ printf '%s\n' "$PLATE" | me seal --seal-secret --out $D/a.uf2
exit=4
me: invalid record: reserved-prefix byte was 0x03, expected 0x00

$ printf '%s\n%s\n' "$ENTR32" "$PLATE" | me seal --seal-secret --out $D/b.uf2
exit=4
me: invalid record: reserved-prefix byte was 0x03, expected 0x00
```

against `me sysw pack`, which after this diff says:

```
me: record 0 (records count from 0) is a hashlock PREIMAGE plate (kind 0x03), not a seed record; this container cannot place one yet. A preimage backs a hashlock spend path, not a wallet — keep it with the policy it unlocks, and do not re-encode it as entropy.
```

`me seal` reaches `validate_record` and stops at the same
`RecordError::Invalid(e.to_string())` the sysw path used to stop at; the H0 arm
is only wired into `sysw::unknown_reason`. Two consequences:

1. **Today:** the operator who reaches for `me seal` rather than `me sysw pack`
   gets a bare codec string with no `record N` index, no kind name and none of
   the "keep it with the policy it unlocks / do not re-encode it as entropy"
   guidance that fidelity I-3 asked for. In a multi-record seal it does not even
   say which record. This is a real missing case, not wording: the plan's site
   table covers `main.rs`'s `U::Bip93OutsideTheProfile` arm and nothing else,
   so the second verb was never considered.
2. **At the 0.8 bump:** `me seal`'s refusal disappears entirely along with the
   sysw one, because both flow through `validate_record`. The tripwire covers it
   (`a_preimage_plate_is_not_a_seed_record` asserts on `validate_record`
   directly, measured red under the bump simulation below), so this half is
   *gated*; F-473 should still name `me seal` explicitly, since it currently
   names only `sysw pack`.

Neither refusal is wrong — both exit non-zero and neither echoes the record.
Filed Important for the missing case, not for the wording.

---

### M-1 — `UnknownReason::PreimagePlate`'s "75 characters is the only shape" is measured false, and a 77-character malformed `0x03` string is named a "PREIMAGE plate"

`crates/me-cli/src/sysw/mod.rs`, the new variant's doc comment:

> `/// yet (H0, §9). Carries no number: 75 characters is the only shape.`

and `crates/me-cli/src/seal/record.rs`, `preimage_plate`'s summary line:

> `/// Is `s` a hashlock PREIMAGE plate (SPEC_ms_hashlock §1: kind byte `0x03`,`
> `/// id `hash`, 75 characters) …`

The predicate checks neither the id nor the length: it asks `classify(s) ==
Format::Ms` and `ms_codec::decode(s) == Err(ReservedPrefixViolation { got: 3 })`.
Measured, on the built `me`:

| string | length | `me sysw pack` says |
| --- | --- | --- |
| `ms10hashsq…` (the plate) | 75 | hashlock PREIMAGE plate |
| `ms10entrsqv0…` (entr id) | 75 | hashlock PREIMAGE plate |
| `ms10testsqvrsu9…` (id `test`) | 75 | hashlock PREIMAGE plate |
| `ms10mnemsqvrsu9…` (id `mnem`) | 75 | hashlock PREIMAGE plate |
| `ms10seedsqvrsu9…` (id `seed`) | 75 | hashlock PREIMAGE plate |
| **`ms10hashsqvrsu9…` (34-byte payload)** | **77** | **hashlock PREIMAGE plate** |
| `ms10hashsqvrsu9…` (32-byte payload) | 74 | outside the profile |

The 77-character row is spec §1's `PreimageLengthMismatch` shape — *not* a
preimage plate — and `me` calls it one. The refusal is still correct and the
advice is harmless, so this is a records/precision defect rather than a
behavioural one; it matters because F-473 asks a future implementer to re-point
this predicate at the 0.8 arm using a doc comment that misdescribes it.

### M-2 — the sysw seam corpus carries no row for the collision I-1 names

`codex32_seam_vectors.json` gained a `0x03` control at 16 bytes (unshared),
a `0x03` control at 33 bytes (shared), and a 33-byte unshared control at `0x31`.
The one row that would pin I-1's boundary — **33 bytes, unshared, first byte
`0x03`, id not `hash`** — is absent, so no test in either repo records that the
device refuses it. Add it with `device_admits: false` (documenting the deliberate
narrowing) or `true` (if the narrowing is unintended); either way the fact stops
being invisible.

### M-3 — `unlockEngraveCodex32`'s godoc describes `recoverCodex32Flow` as NFC-driven; it is keyboard-driven

`gui/unlock_session.go:173-175`: *"whose codex32Recover branch waits on physical
NFC shares that a payload-sourced record does not have"*. `recoverCodex32Flow`
collects shares through `inputCodex32Flow` — the on-screen keypad — with no NFC
involvement (that is how C-1's reproduction supplied share 2). Pre-existing text
the diff did not introduce, reported because C-1 turns the Recover arm into
load-bearing code and the comment misdescribes what it does.

### M-4 — `me seal`'s and `me sysw pack`'s "Plain BIP-93 secrets are 48 or 74 characters" is a convention stated as a fact

Reached for the 48-character and 75-character plain-BIP-93 rows during item 3.
The corpus's own `bip93-plain-33-byte-payload-0x31` row is a 75-character plain
BIP-93 secret. Pre-existing copy, surfaced by the new rows; it is what makes
F-472's inventory (I-1) read as complete when it is not.

### N-1 — the `MsTooLong` arm in the pin test is the only Err discriminated

`tests/preimage_plate_is_not_a_seed.rs` treats every `Err(_)` other than
`MsTooLong` as a pass. That is deliberate (a tripwire for `Ok`), and the arm is
correct — but at the 0.8 bump the *distinguishing* error will be a new one, and
`Err(_) => {}` will accept a `PreimageLengthMismatch` on the wrong string as
readily as the right refusal. Cheap to tighten when F-473 lands.

---

## Door table — every door, executed

The plate throughout is
`ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c`.

| door | input | screen reached / outcome |
| --- | --- | --- |
| NFC scan, `(*scanner).Scan` | plate | `nil, scan: unknown format` → start screen "Unknown format". No codex32 object produced. |
| NFC scan | plate, **UPPERCASE** | `nil, scan: unknown format` (`codex32.New` accepts consistent case; `ParsePrefix` sets `Unshared` on `'S'` as well as `'s'`) |
| NFC scan | plate + trailing `\n`, plate + leading space | `nil, scan: unknown format` |
| typed `M*1 STRING`, `validateMStar` → `engraveObjectFlow` → `engraveCodex32` | plate | `showError("Hashlock preimage", "This record is a hashlock preimage, not a seed. It is not engraved as one.")`; no confirm screen |
| **typed / scanned SHARE → Recover → the `engraveCodex32` loop** | **2-of-n set of a `0x03`/33-byte payload** | **`Confirm Codex32 Secret`, then `Engrave Plate` — C-1** |
| sealed, `seal.AdmitSection(…, SectionEncrypted)` | plate | `ErrRecordNotPermitted: record 0 classifies as unknown format, which the encrypted section does not permit`; 0 records admitted; renders as "Payload unreadable." (F-474) |
| sealed, mixed | entr-128 + plate | 0 admitted, `record 1 … unknown format` — whole-section refusal, as designed |
| sealed, `unlockEngraveCodex32` (defence in depth) | plate | `showError(unlockTitle, "This record is a hashlock preimage, not a seed. …")`; no `EngraveSeed` screen |
| sysw container, `sysw.Classify` | plate, uppercase plate, entr-id shape | `ClassUnknown` — offered to no program (`sysw_admit.go` maps every program off `ClassCodex32Secret`) |
| sysw composer door, `composerDoorCounts` | plate | counted `inert`; not `seeds` |
| host, `me sysw pack` | plate / entr-id shape / any-id 33-byte `0x03` / uppercase plate | exit 4, "is a hashlock PREIMAGE plate (kind 0x03), not a seed record"; record never echoed |
| host, `me seal` (no `--seal-secret`) | plate | exit 3, "refusing to seal seed material … without --seal-secret" |
| host, `me seal --seal-secret` | plate, and plate + a real entr-32 | exit 4, `me: invalid record: reserved-prefix byte was 0x03, expected 0x00` — **I-2** |

**Sites the plan's table does not list, traced.**
`grep -n "codex32.New(\|ClassCodex32Secret\|EngraveSeedString(\|DecodeMS1(" --include=*.go -r .`
returns 40 non-test hits. Every one that could reach metal or a screen with a
preimage:

- `gui/gui.go:2869` `codex32.New(body)` behind `syswOffer(…, sysw.ClassCodex32Secret, …)` — gated by `sysw.Classify`, which is guarded. Safe.
- `gui/codex32_polish.go:275` `codex32.New(frag)` in `validateMStar` — feeds `engraveObjectFlow` (guarded) **and** `recoverCodex32Flow` (C-1).
- `backup/backup.go:158` `EngraveSeedString` — exactly two callers, `gui/gui.go:2825` (`backupSeedStringFlow`, one caller: `engraveCodex32:242`) and `gui/unlock_session.go:203`. Both covered; the first only partially (C-1).
- `gui/ms1_decode.go:22`, `gui/multisig_verify.go:1237`, `gui/singlesig_verify.go:185`, `gui/codex32_polish.go:106`, `bundle/verify.go:138` — all `DecodeMS1`, which is deliberately unchanged and returns `errMSBadPrefix` for `0x03` (asserted by the new codex32 test). Safe, and the "Show secret" button is consequently not offered for a preimage.
- `gui/transaction.go:280` — `txClassName`, display only.
- `cmd/biptool/main.go:314,346,402` — host dev tool, not on the device.

## Negative space — executed

`IsPreimage` and both classifiers, measured (`codex32`, `sysw`, `seal`, `gui`
packages of my fork copy):

| population | `len(Seed())` | `Seed()[0]` | `IsPreimage` | `sysw.Classify` | `seal.Classify` | `AdmitSection` |
| --- | --- | --- | --- | --- | --- | --- |
| the plate | 33 | `0x03` | **true** | Unknown | unknown format | refused |
| the plate, UPPERCASE | 33 | `0x03` | **true** | Unknown | (pass-1 lowercase rule refuses first) | refused |
| entr-id `0x03` shape (75 ch) | 33 | `0x03` | **true** | Unknown | unknown format | refused |
| plain BIP-93, 16-byte payload, `0x03` | 16 | `0x03` | false | Codex32Secret | codex32 secret | **admitted** |
| 2-of-N share, point begins `0x03` | 33 | `0x03` | false | Codex32Secret | codex32 secret | **admitted** |
| plain BIP-93, 33-byte payload, `0x31` | 33 | `0x31` | false | Codex32Secret | codex32 secret | **admitted** |
| constellation entr-128 | 17 | `0x00` | false | Codex32Secret | codex32 secret | **admitted** |
| constellation mnem-en-16 | 18 | `0x02` | false | Codex32Secret | codex32 secret | **admitted** |
| plate with a corrupted checksum | — | — | — (`New` fails: `invalid checksum`) | Unknown | unknown format | refused |
| mixed-case plate | — | — | — (`New` fails: `invalid case`) | Unknown | unknown format | refused |
| **plain BIP-93, 33-byte payload, `0x03`, id `seed`/`test`** | 33 | `0x03` | **true** | **Unknown** | **unknown format** | **refused — I-1** |
| entr / mnem payload lengths, exhaustively (16,17,20,24,28,32,33,34,44) | — | `0x03` | true only at **33** | — | — | — |

`ParsePrefix` reports `Unshared` for `'S'` as well as `'s'`
(`codex32/polish.go:134`), and `checkCase` matches the checksum engine's case
rule, so `New`-valid ⇒ `ParsePrefix`-parsable for every string tested: there is
no New-accepts / ParsePrefix-rejects gap through which a preimage could fall
open. `IsPreimage` also answers `true` under a non-`ms` HRP (`aa10hashs…`,
`xy10hashs…`), which is the conservative direction and matches `seal.Classify`'s
own HRP-blind `codex32.New`.

**Whole sealed payload with a legitimate `0x03` share:**
`AdmitSection([entr-128, 0x03 share], SectionEncrypted)` → **2 admitted, err nil**.
The plan's fidelity C-2 failure is not reintroduced for shares.

## Mutation table — every test the diff adds or edits

Applied to a copy, restored + `touch`ed after each. **No survivors.**

| # | mutation | test that died | how it died |
| --- | --- | --- | --- |
| M1 | `IsPreimage`: drop `!f.Unshared` | `codex32.TestIsPreimageReadsThePrefixByteOnly` | `IsPreimage(bip93-share-payload-0x03 …) = true, want false` |
| M2 | `IsPreimage`: `len(d) == 33` → `len(d) > 0` | same | `IsPreimage(bip93-plain-payload-0x03 (16-byte seed …)) = true, want false` |
| M3 | `d[0] == msPrefixPreimage` → `d[0] != msPrefixEntr` | same | `IsPreimage(bip93-plain-33-byte-payload-0x31 …) = true, want false` |
| M4 | key on the id `hash` instead of the prefix | same | `IsPreimage(preimage-shape-entr-id …) = false, want true` |
| M5 | `seal.Classify`: drop `!codex32.IsPreimage(c)` | `seal.TestClassifyMirrorsScanBranchOrder` + `seal.TestAdmitSectionRefusesAPreimagePlateAsUnknown` | `Classify(…) = codex32 secret, want unknown format`; `AdmitSection(…) err = <nil>, want ErrRecordNotPermitted` |
| M6 | `sysw.isStrictMs1`: drop the guard | `sysw.TestCodex32SeamDeviceAdmitsEverythingTheHostDoes` | `preimage-plate-0x03: device admits = true, want false`; `preimage-shape-entr-id: device admits = true, want false` |
| M7 | `engraveCodex32`: disable the guard | `gui.TestEngraveCodex32RefusesAPreimagePlate` | `never reached "hashlock preimage"; last frame "ConfirmCodex32SecretidHASHUnsharedsecret(S)75chars"` |
| M8 | `gui/scan.go`: drop `&& !codex32.IsPreimage(s)` | `gui.TestScanDoesNotHandAPreimagePlateToEngrave` | `Scan(preimage plate) = codex32.String, <nil>; want errScanUnknownFormat` |
| M9 | `unlockEngraveCodex32`: disable the guard | `gui.TestUnlockEngraveCodex32RefusesAPreimagePlate` | `never reached "hashlock preimage"; last frame "Insertablankplate… EngravePlate"` |
| M10 | over-wide `IsPreimage` (drop `len == 33`) vs the corpus | `sysw.TestCodex32SeamDeviceAdmitsEverythingTheHostDoes` | `bip93-plain-payload-0x03: device admits = false, want true` — the positive controls really assert |
| R1 | Rust: swap the two arms in `unknown_reason` | `sysw::tests::a_preimage_plate_is_named_not_misdiagnosed` + `preimage_plate_is_not_a_seed::sysw_pack_names_a_preimage_plate_and_never_echoes_it` | `left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))`; `stderr does not name the kind: … outside …` |
| R2 | Rust: `preimage_plate` → always `false` | the same two | as R1 (and `a_preimage_plate_is_not_a_seed_record` correctly still passes — at 0.7 the codec refuses regardless) |
| R3 | Rust: drop the `preimage-plate-0x03` row from `record_corpus_pre_s2.json` | `record_corpus::the_capture_is_the_whole_corpus`, `…the_descriptor_gate_stays_shut_on_every_corpus_record`, `…every_corpus_record_classifies_as_it_did_before_s2` | three red, exactly the plan's Step-1b claim |

**The 0.8-bump hazard, simulated** (brief item 3). `validate_record`'s `Format::Ms`
arm patched with `if s.trim().to_lowercase().starts_with("ms10hash") { return
Ok(RecordKind::Ms); }` before `ms_codec::decode`, whole crate, `--no-fail-fast`:

```
Summary [   0.547s] 616 tests run: 608 passed, 8 failed, 2 skipped
   FAIL mnemonic-engrave sysw::tests::a_preimage_plate_is_named_not_misdiagnosed
   FAIL mnemonic-engrave::codex32_seam the_host_never_admits_what_the_device_would_refuse
   FAIL mnemonic-engrave::history_purge editing_the_file_alone_is_the_trap_the_message_warns_about
   FAIL mnemonic-engrave::history_purge the_harness_records_history_at_all
   FAIL mnemonic-engrave::preimage_plate_is_not_a_seed a_preimage_plate_is_not_a_seed_record
   FAIL mnemonic-engrave::record_corpus every_corpus_record_classifies_as_it_did_before_s2
   FAIL mnemonic-engrave::history_purge the_emitted_zsh_recipe_actually_purges_the_entry
   FAIL mnemonic-engrave::preimage_plate_is_not_a_seed sysw_pack_names_a_preimage_plate_and_never_echoes_it
```

Five witnesses (the three `history_purge` are the box-local baseline failures),
in four binaries. The plan claimed three; the claim is **conservative and true**,
not overstated.

## Firmware (brief item 5)

Re-measured in my fork copy, `export PATH=/nix/var/nix/profiles/default/bin:$PATH`:

```
$ nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 \
    -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller
   code    data     bss |   flash     ram
1551304   31796   31004 | 1583100   62800
```

**1,583,100 flash / 62,800 RAM.** Byte-identical to the implementer's report.
RAM is unchanged at 62,800 as required; flash is +472 over the brief's 1,582,628
reference, inside "a few hundred bytes". **Verified.**

## Rust-primary (brief item 6)

- **No fork change decides behaviour Rust does not already have.** `me` at the
  pinned ms-codec `0.7` refuses *every* `0x03` single, measured on the built
  binary under five ids (`hash`, `entr`, `test`, `mnem`, `seed`) and in
  uppercase — all exit 4, all named. The fork's `IsPreimage` narrows a strict
  subset of what the host already refuses, so this is convergence (exemption
  (a)), not the port leading. The seam test
  `TestCodex32SeamDeviceAdmitsEverythingTheHostDoes` passes at the tip, i.e. the
  device still admits everything the host does.
- **Corpus is byte-identical and both literals match.**

```
f1f2fa6bbbf27e3697ee496636de49be2f25787deff7b3bc4a2c5e16854e391c  …/me-cli/testdata/codex32_seam_vectors.json
f1f2fa6bbbf27e3697ee496636de49be2f25787deff7b3bc4a2c5e16854e391c  …/fork/sysw/testdata/codex32_seam_vectors.json
cmp: BYTE-IDENTICAL
sysw/codex32_seam_test.go:30:const seamVectorsSHA256 = "f1f2fa6b…391c"
tests/codex32_seam.rs:26:    "f1f2fa6b…391c";
```

- Row split measured from the file: **12 rows, 2 both / 6 device-only / 4
  neither** — matching the CHANGELOG and the plan's Global Constraint exactly.
- `record_corpus_pre_s2.json`: **33 → 37** (`git show e7af98a:…| grep -c '"origin"'`
  = 33; at the tip = 37), all four new entries `"class": "Unknown"`. Matches the
  CHANGELOG.

## The implementer's report — deviations, each with a verdict

| # | claim | verdict |
| --- | --- | --- |
| 1 | Vendoring used the absolute path to the engrave worktree's edited copy; both files hash `f1f2fa6b…391c` | **TRUE.** `cmp` byte-identical, both sha literals match. |
| 2 | `gofmt` re-aligned the two preceding `ClassMDMK` rows; whitespace only | **TRUE.** The diff shows padding added to `// mk1` and `// md1` only. `gofmt -l` is clean on all eleven files the diff touches. |
| 3 | Fork commit ends `Signed-off-by` → `Co-Authored-By` → `Claude-Session`; tip is `14afdff` not the pre-amend `bc81a71` | **TRUE.** `git log -1 --format=%B 14afdff` reproduces that order verbatim. |
| 4 | The harness renders frame text space-stripped, so the quoted frame differs from the plan's spacing | **TRUE.** Every frame I captured is space-stripped (`"ConfirmCodex32SecretidHASH…"`). |
| 5 | Clippy is red on this box at `sysw/composer_records.rs:114` (`manual implementation of .is_multiple_of()`), a file this task never touched | **TRUE.** Reproduced verbatim; `composer_records.rs` is absent from both diffstats. |

Other measurable claims spot-checked and **true**: `be72e75` is "7 files changed,
186 insertions, 1 deletion"; the Step-6 RED text; the Step-7/8 mutation outcomes;
the whole-crate figure. **No false claim found in the implementation report.**

## Gates re-measured at the tips (confirming the brief's "already settled")

```
engrave  cargo nextest run --locked -p mnemonic-engrave --no-fail-fast
         Summary  616 tests run: 613 passed, 3 failed, 2 skipped
         (the 3 = history_purge, box-local, no /usr/bin/zsh)
fork     go test ./codex32/ ./sysw/ ./seal/           → ok / ok / ok
         go test ./gui/ -run 'TestEngraveCodex32|TestScan|TestUnlockEngrave|
                              TestConfirmCodex32|TestClassify|TestAdmit'  → ok
         gofmt -l  (the 11 files this diff touches)   → empty
```

Both copies restored and re-run green after every mutation.

---

## Counts

| severity | count |
| --- | --- |
| Critical | **1** (C-1) |
| Important | **2** (I-1, I-2) |
| Minor | **4** (M-1 … M-4) |
| Nit | **1** (N-1) |

**NOT GREEN.** C-1 is a live path on the tip that reaches `Engrave Plate` with a
kind-`0x03` preimage single and directly contradicts `SPEC_ms_hashlock` §12 item
7; it must close before the fork branch merges or is flashed. I-1 and I-2 are
blocking under the standing severity rule. Everything else is recorded.

What is *not* a finding, stated so the next round does not re-derive it: the
predicate's shape, both classifiers, the scan mirror, the sealed path's two
guards, the corpus pin, the record capture, the host diagnosis, the 0.8 tripwire,
the firmware budget and every one of the diff's tests are sound and executed —
the eleven mutations above have no survivors, and the negative space holds in
both directions.
