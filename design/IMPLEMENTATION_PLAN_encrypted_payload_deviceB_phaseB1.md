# Encrypted Payload Delivery — Plan B Phase B1 (device UI, the unsealed path) — Implementation Plan

**Status:** DRAFT — R0 in progress. **No code before 0C/0I.**

| round | verdict | report |
| --- | --- | --- |
| 0 | 0C / 4I | `design/agent-reports/encrypted-payload-planB-phaseB1-R0-round0.md` |
| 1 | 0C / 1I — 3 of 4 FIXED, labelling PARTIAL | `design/agent-reports/encrypted-payload-planB-phaseB1-R0-round1.md` |
| 2 | 0C / 1I — ordering fix SOUND; defect in a paragraph the round-1 fold volunteered | `design/agent-reports/encrypted-payload-planB-phaseB1-R0-round2.md` |

**B1's implementable content is GREEN at 0C/0I.** Round 2 verified Task 4a's
ordering fix against the real control flow and closed both open questions —
`permitted()` admits only `ClassMDMK` publicly, and `groupCards`' key order is
deterministic (first-seen, not map iteration). Its single Important was against
a *note to B2*, not against anything an implementer of B1 does; the round-2 fold
corrects that note and files **F-77**. Per the proportional re-review rule a
prose/factual fold that changes no task, no logic and no control flow does not
re-trigger the gate, so **no round 3** — the loop is closed.

**The pattern worth carrying forward:** rounds 1 and 2 each found a defect
**authored by the previous fold**, and in both cases by content the author
*volunteered* beyond what the reviewer asked for. Continuity §4 recorded folds as
the dominant defect source; this cycle sharpens it — the dangerous part of a fold
is the part nobody requested.

**Descends from:** `SPEC_encrypted_payload_delivery.md` §10, which is GREEN and
normative. This plan implements §10.2 steps 1–4 plus the plate list and engrave.
It does **not** restate requirements — where this plan and §10 disagree, §10
wins and this plan is defective.

**Predecessor:** `IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseA.md`
(merged, `3ee6c65`). Phase A is the headless core; B1 is the first half of the
UI that drives it.

---

## Why the phase boundary is here

The B1/B2 seam is **normative, not invented**. §10.2 step 4:

> If `ct_len == 0`, stop here: no passphrase is prompted. Show the
> unauthenticated warning of §10.2.3 with the hash, require an explicit
> confirmation, then go to the plate list. **Steps 5–8 are skipped entirely.**

So the spec already describes a complete flow that terminates before the
passphrase. B1 builds exactly that flow.

| | B1 (this plan) | B2 (next plan) |
| --- | --- | --- |
| §10.1 detection + menu entry | ✅ | — |
| §10.2 steps 1–3 (`Inspect`, hash) | ✅ | — |
| §10.2 step 4 (`ct_len == 0` → §10.2.3 warning) | ✅ | — |
| plate list (paged) + engrave public records | ✅ | extended |
| §10.2 steps 5–9 (words, KDF, retry loop) | — | ✅ |
| §10.2.2 secrets-first session lifecycle | — | ✅ |
| §10.2.4 residency-keyed idle wipe | — | ✅ |
| F-73 hardware verification | ✅ | — |

**The property that makes B1 cheap to review: no secret is ever resident.** B1
never derives a key, never decrypts, and never holds a `Payload.Secret` record.
§10.2.1's allow-list — already Phase A code, already vector-tested — is what
guarantees the public section contains nothing secret. So the entire wipe
lifecycle (§10.2.2) and idle timer (§10.2.4) are **out of scope** rather than
half-built, and a reviewer's budget goes to the menu-conditionality machinery
and the plate list, which is where B1's actual risk is.

**What B1 does with a sealed payload:** `Inspect` it, show the hash and record
count with `SEALED`, and stop at a terminal screen saying the passphrase flow is
not yet available (Task 6). It does **not** silently behave as if the payload
were unsealed, and it does **not** prompt for words it cannot check.

---

## Global Constraints

Phase A's global constraints carry forward **unchanged** and are not repeated in
full. The load-bearing ones for B1:

- **All Go work runs under `nix develop --command …`.** `nix` is NOT on `PATH` —
  use `/nix/var/nix/profiles/default/bin/nix`.
- **`go.mod` says `go 1.25.10`; TinyGo is 0.41.1.** The host `go` in the dev
  shell is 1.26.3 and is **not** the firmware compiler. A screen that builds on
  the host can still fail under TinyGo.
- **`go test ./...` GREEN means exactly TWO setup failures**, `cmd/kdfbench` and
  `cmd/sealread`, both TinyGo-only (`machine` is not in host std). Any third
  failure is a regression. If B1 adds a TinyGo-only command, update this line.
- **Never a bare `go test ./... -update`.** Scope with `-run`, then `git status`.
- `gofmt` clean before every commit; `go vet ./<touched>/...` clean.
- Stage paths explicitly. Never `git add -A`.

B1-specific:

- **`gui` gains an import of `seedhammer.com/seal`.** B1 calls only `Inspect`,
  which runs no KDF and no AES. B2's `Unlock` is what pulls `crypto/aes` and
  `crypto/cipher` in (~1.6 KB marginal, measured in Phase A). Do not
  pre-emptively import them in B1 and do not treat B1's binary size as
  predictive of B2's.
- **`seal.XIPReader` exists only under the TinyGo build** (`seal/read_tinygo.go`);
  `seal.FileReader` is the host stand-in (`seal/read_host.go`). Neither may be
  named directly from `gui` — see Task 1's platform seam.

---

## A deliberate departure from a carried-forward constraint

Phase A's plan carries this forward (`IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseA.md:632`):

> **Adding a menu program touches four sites** — the `program` enum (inserted
> *before* `bip85Derive`, **not appended**) …

**B1 appends `unlockPayload` after `bip85Derive` instead, and moves the
compile-time guard onto it.** This is a departure and it is deliberate.

That advice is correct for an *unconditional* program — it is why
`engravePassphrase` and `engraveText` were inserted mid-enum, and it works
because the wrap and pager sites are all keyed to `bip85Derive` staying last, so
inserting earlier leaves them untouched.

**For a conditional program the calculus inverts.** §10.1 requires the entry to
be *invisible* when no payload is present. Inserted mid-enum, conditional
visibility means the carousel must **skip an interior index** in both wrap
directions, and `layoutMainPager` (`gui/gui.go:1938`) fills dot `int(page)` —
with an interior entry skipped, the filled dot no longer corresponds to the
displayed program. Appended, the entire conditionality collapses to a single
runtime bound: *what is the last navigable program?*

The guard at `gui/gui.go:172` is designed to be moved when the last navigable
program changes — its own comment says "Keep these in lockstep." Moving it from
`bip85Derive` to `unlockPayload` preserves its meaning exactly: it still fails
the build if a program is inserted between the last navigable one and
`qaProgram`.

**Reviewer: this is the one place B1 knowingly contradicts a prior artifact.**
If the reasoning above is wrong, Task 1 is wrong.

---

## Task 1 — detection, the platform seam, and the conditional carousel

The largest and riskiest task. Everything else in B1 is a screen.

### 1a. The platform seam

`gui` must obtain a `seal.Reader` without naming a build-tagged type. The house
pattern for an optional platform capability already exists and is exactly this
shape — `NFCReader() io.ReadCloser`, consumed at `gui/gui.go:1592` as
`if r := ctx.Platform.NFCReader(); r != nil`.

Add to the `Platform` interface (`gui/gui.go:2771` is the `NFCReader` line;
insert adjacent):

```go
// PayloadReader returns a reader for the §5 payload region, or nil if this
// platform has none. nil is not an error: the emulator and the test platform
// have no XIP flash, and §10.1 makes the feature invisible when no payload is
// readable, which is the same operator-visible outcome.
PayloadReader() seal.Reader
```

Three implementations must be updated — machine-counted, not estimated:

| file | returns |
| --- | --- |
| `cmd/controller/platform_sh2.go:564` (adjacent to `NFCReader`) | `seal.XIPReader{}` |
| `cmd/emu/platform.go:189` (adjacent to `NFCReader`) | `nil` — see below |
| `gui/gui_test.go:428` (adjacent to `NFCReader`) | a settable field, default `nil` |

**`cmd/emu` returns `nil`. (R0 round 0, finding 4.)** An earlier draft of this
plan said "`seal.FileReader{Path: …}` from a flag", which cannot work: `cmd/emu`
is `//go:build js` (`cmd/emu/platform.go:1`), built `GOOS=js GOARCH=wasm` and
run from a static page through `wasm_exec.js`. There is no `flag.Parse()` in the
package and `os.Args` carries no real argv under the browser glue, so a
`flag.String` would compile, run, and **silently never receive a value** — a
non-failing no-op, which is the worst shape a test affordance can have.

It is also not needed. Task 1d's tests drive B1 through `testPlatform`, not
through the emulator, so the automated coverage is unaffected. If a browser-side
payload is ever wanted, the mechanism is a `syscall/js` read of
`location.search` or a JS global set from the host page — **not** a flag, and
not in B1.

### 1b. Detection

§10.1: read at GUI start, compare to `MNEMBLOB`, present → entry appears.

Probe **once**, in `uiFlow` before the `StartScreen` loop — not per frame. The
region does not change while the GUI runs (writing it requires `picotool` and a
reboot), and probing per frame would put a 64 KB `unsafe.Slice` read in the
frame path.

```go
// Probed once: the region cannot change without a reboot, and §10.1's
// "absent → the feature is invisible" is a startup property.
var payload []byte
if r := ctx.Platform.PayloadReader(); r != nil {
        if b, err := r.Read(); err == nil {
                payload = b
        }
        // errors.Is(err, seal.ErrNoPayload) is the ORDINARY case: erased
        // flash. It is not logged as a failure. Any other error is equally
        // "no feature" for §10.1 purposes -- a region that will not read is
        // indistinguishable from one that is empty, from the menu's point of
        // view. The DIFFERENCE between absent and corrupt is made visible in
        // Task 2, AFTER the operator has chosen to open the entry.
}
```

**Note the asymmetry and preserve it.** §10.1's absent/present decision is
deliberately coarse — it only asks "are the first 8 bytes `MNEMBLOB`". A blob
that is present but violates §6.2 still shows the menu entry, and then Task 2
reports "payload unreadable". Collapsing those two would hide a tampered payload
behind an invisible menu entry, which is precisely the signal §2.2 item 4 exists
to raise.

### 1c. The conditional carousel

Add `unlockPayload` after `bip85Derive`, before `qaProgram`. Add
`lastNav program` to `StartScreen`, set once from the Task 1b probe.

The **four boundary sites** — machine-located via
`grep -n "bip85Derive" gui/gui.go`, not recalled:

| site | today | becomes |
| --- | --- | --- |
| `gui/gui.go:1690` | `m.prog = bip85Derive` | `m.prog = m.lastNav` |
| `gui/gui.go:1695` | `if m.prog > bip85Derive` | `if m.prog > m.lastNav` |
| `gui/gui.go:1904` | `const npage = int(bip85Derive) + 1` | `npage := int(m.lastNav) + 1` |
| `gui/gui.go:1938` | `const npages = int(bip85Derive) + 1` | parameter `lastNav program` |

`layoutMainPager` and `layout` are free functions / methods that do not see
`StartScreen.lastNav`; thread it as a parameter rather than reaching for a
package-level variable.

**Changing `layoutMainPager`'s arity means updating its callers, and there are
exactly TWO. (R0 round 0, finding 1.)** Machine-located with
`grep -rn "layoutMainPager"` across the whole repo, not just `gui.go`:

| call site | becomes |
| --- | --- |
| `gui/gui.go:1736` | `layoutMainPager(&ctx.B, th, m.prog, m.lastNav)` |
| `gui/text_program_test.go:79` | pass `bip85Derive` as the fourth argument |

The second is an **existing green test**, `TestStartScreenFitsAtEightPagerDots`.
Passing `bip85Derive` there is behaviourally correct: that test only exercises
the no-payload state. Missing it is a `go test ./gui/...` build failure that
would present as a third setup failure and violate this plan's own GREEN
baseline.

Two more things in that file move in the same commit:

- `gui/text_program_test.go:8-13`'s comment says `bip85Derive` "must stay the
  last navigable program" and cites `gui.go:168` for the guard. Both go stale
  when B1 ships — `unlockPayload` becomes last navigable, and the guard is at
  `gui/gui.go:172`, not `:168`. Update the comment; do not leave a test
  explaining the opposite of what the code now does.
- **Add a NINE-dot width test.** The existing test is named
  `…FitsAtEightPagerDots` and asserts `sz.X <= sh2DisplaySize.X`. The pager is
  `(sz.X+space)*npages - space` (`gui/gui.go:1938`), so a ninth dot makes it
  wider, and **nothing currently proves nine fits the panel.** This was not in
  the R0 report; it was found while folding it. If nine does not fit, that is a
  Task 1 blocker and the dot pager needs a different treatment at nine —
  discover it with a test, not on hardware.

The **three additive sites**:

| site | change |
| --- | --- |
| `gui/gui.go:1553` (dispatch switch) | `case unlockPayload: unlockPayloadFlow(ctx, th, payload)` |
| `gui/gui.go:1726` (title switch) | `case unlockPayload: titleTxt = "Sealed Payload"` |
| `gui/gui.go:1929` (`layoutMainPlates` case list) | add `unlockPayload` |

And the guard at `gui/gui.go:172`:

```go
// Compile-time guard: unlockPayload MUST remain the LAST navigable program,
// immediately before the non-navigable qaProgram. It is CONDITIONALLY shown
// (§10.1) -- StartScreen.lastNav is bip85Derive when no payload is present and
// unlockPayload when one is -- which is exactly why it must be last: the wrap
// and pager sites take lastNav as a bound, and a bound only works if the
// conditional entry is at the END of the range. An entry inserted mid-enum
// would require SKIPPING an interior index, and layoutMainPager fills dot
// int(page), which would then point at the wrong dot.
var _ [1]struct{} = [qaProgram - unlockPayload]struct{}{}
```

### 1d. Tests first

`gui/unlock_program_test.go`, following the existing `*_program_test.go` idiom:

1. **No payload → the entry is invisible.** `testPlatform.PayloadReader` returns
   `nil`. Assert stepping `next` from `backupWallet` wraps at `bip85Derive` and
   never reaches `unlockPayload`; assert the pager renders 8 dots.
2. **Payload present → the entry appears.** `PayloadReader` returns a
   `FileReader` over a vectors.json blob. Assert the carousel reaches
   `unlockPayload`, wraps there, and the pager renders 9 dots.
3. **Wrap in both directions**, both states. The `prev` wrap
   (`gui/gui.go:1690`) and the `next` wrap (`gui/gui.go:1695`) are separate
   sites and a fix to one does not fix the other.
4. **The other eight programs still reachable** in both states. This is the
   regression the const-to-runtime change can silently cause, and no other test
   in the repo would catch it.

> **Mutation check (required, per §11.3 and the project standard).** After these
> pass, break each of the four boundary sites in turn — e.g. revert `:1904` to
> `int(bip85Derive) + 1` — and confirm a test **fails**. A boundary site whose
> mutation nothing notices is an unpinned site. Record the four results in the
> task's commit message. Continuity §4's most transferable rule applies here
> directly: an assertion whose expected side is derived from the same constant
> the code uses pins nothing. Assert **literal 8 and 9**, not
> `int(bip85Derive)+1`.

---

## Task 2 — `Inspect` and the hash screen (§10.2 steps 1–3)

`gui/unlock_flow.go`, new file.

```go
func unlockPayloadFlow(ctx *Context, th *Colors, blob []byte) {
        var o seal.Opener
        p, err := o.Inspect(blob)
        if err != nil {
                // §6.2/§6.4: every violation is "payload unreadable" to the
                // operator, EXCEPT too-many-records, which §6.4 requires be
                // distinguishable -- the operator has been taught to read
                // "unreadable" as "someone replaced my payload".
                if errors.Is(err, seal.ErrTooManyRecords) {
                        showError(ctx, th, "Sealed Payload",
                                "This payload declares more records than the machine accepts.")
                        return
                }
                showError(ctx, th, "Sealed Payload", "Payload unreadable.")
                return
        }
        ...
}
```

Then the hash screen, per §10.2 step 3:

- Shown **only if `p.HasHash`** — which Phase A sets false exactly when
  `pub_len == 0`. Displaying a constant digest on every fully-encrypted payload
  would teach the operator it is furniture (§10.2 step 3, and `Payload.HasHash`'s
  own doc comment).
- Rendered with `seal.FormatHash(p.Hash)` — eight groups of four. Do not
  re-format it locally; the grouping is what makes it comparable against what
  the operator wrote down.
- Shows the **public** record count (`len(p.Public)`) and the sealed/unsealed
  shape from `p.Header.Sealed()`.

> **The count is `len(p.Public)`, never `len(p.Public)+len(p.Secret)`.** §6.6
> hashes the public record count; §6.4's 1..24 cap counts both sections. Vector
> D is 5 public of 6 total and the two produce **different digests**. A screen
> that displays the total next to a hash computed over the public count teaches
> the operator that mismatches are normal — which disarms the only control an
> unsealed payload has.

**Tests:** drive vectors D and E (unsealed, same five public records, different
digests) and assert the two screens render **different** hash strings. That is
the downgrade-visibility property, asserted at the UI rather than only in
Phase A. Assert vector-with-`pub_len==0` renders **no** hash region at all.

---

## Task 3 — the §10.2.3 unauthenticated warning

Shown **when and only when `ct_len == 0`**, i.e. `!p.Header.Sealed()`.

`ConfirmWarningScreen` (`gui/gui.go:240`, laid out at `gui/gui.go:336`) already
provides Title / Body / Icon, a scrollable body, a cancel and a hold-to-confirm
button, and returns `ConfirmYes` / `ConfirmNo` / `ConfirmNone`. **This is not
new machinery.** Do not build a bespoke screen.

The body is §10.2.3's copy, verbatim, with the hash interpolated. The wording is
NORMATIVE and was reviewed for what it claims — in particular it must keep
saying the hash works *only if the operator actually compares it*, and must not
imply the device verified anything.

`ConfirmNo` / `ConfirmNone` → return to the menu without reaching the plate
list. `ConfirmYes` → Task 4.

**Tests:**
- `ct_len == 0` → the warning is reached; `ConfirmNo` returns to the menu and the
  plate list is **never** constructed.
- **`ct_len > 0` → the warning is NOT shown.** This is the one that matters:
  a warning shown on a sealed payload is a false claim that the payload is
  unauthenticated.
- The rendered body **contains the same string** `seal.FormatHash` produced in
  Task 2. Assert against the formatted value, not against a hard-coded digest —
  a hard-coded digest is pinned to the vector, but the *screen* is what is under
  test here.

---

## Task 4 — the plate list (paged), and Back is Lock

Per §10.3 as amended by the B1 design (see the SPEC's
"The plate list's three slots are Back / Page / OK — and Back IS Lock").

Model it on `bundleReviewFlow` (`gui/bundle_flow.go:227`), whose nav
(`gui/bundle_flow.go:275`) is already Back / Page / OK within the three-slot
budget of `ys := [3]int{…}` (`gui/gui.go:1857`).

**Differences from `bundleReviewFlow`, all deliberate:**

1. It is **selectable**, not read-only — OK engraves the highlighted entry
   (Task 5), where `bundleReviewFlow`'s OK confirms the whole set.
2. Entries are labelled per **Task 4a** below — **never** from anything the
   sealer asserted, and the contents are not rendered.
3. Back is the session exit. In B1 there is nothing to wipe, but the **shape**
   must be right here or B2 inherits a list it cannot extend without a fourth
   nav slot.

### Task 4a — surface the card grouping from `seal` (R0 round 0, finding 2)

**`AdmittedRecord.Class` cannot produce §10.2.2's labels, and an earlier draft of
this plan said it could.** `Classification` has exactly **one** value covering
both formats — `ClassMDMK` (`seal/record.go:68`) — returned identically for
`ValidMD` and `ValidMK`. There is no `ClassMD`/`ClassMK` split and
`AdmittedRecord` carries no other distinguishing field, so a plate list built
from `Class` alone cannot even print "mk1" versus "md1", let alone §10.2.2's
`mk1 1/2` / `md1 2/3`.

**Do not re-classify in `gui`.** `seal` already knows: `groupCards`
(`seal/record.go:241`) builds a `groupKey{hrp byte …}` (`seal/record.go:233`)
where `hrp` is `'d'` for md1 and `'k'` for mk1, and `cardKey`
(`seal/record.go:261`) assigns every record to its `(HRP, chunk_set_id)` card
per §6.3. A second classifier in the UI is exactly the two-code-paths divergence
`Opener.Inspect`'s doc comment exists to prevent ("Do NOT re-implement steps 1-3
in Phase B").

**Surface it instead** — but **not** where an earlier draft of this plan said.

> **Round 1 correction.** That draft said the fields are "populated by
> `AdmitSection` from the grouping it already computes." **That data flow does
> not exist.** `AdmitSection` (`seal/record.go:158`) builds `out` inside the
> per-record pass-1/pass-2 loop, where there is no grouping at all; the grouping
> is computed *afterwards*, inside `decodePublicSet` (`seal/record.go:309`),
> whose signature is `func decodePublicSet(records []string) error` — it calls
> `groupCards` and **discards `keys` and `groups`**, returning only an error.
>
> **Getting the order wrong breaks a green Phase A test.** `cardKey`'s default
> branch (`seal/record.go:286`) fails closed with `ErrUndecodableCardSet` for
> anything that is not an md1/mk1 card. `TestPublicSectionRefusesASecret`
> (`seal/record_test.go:180`) puts a BIP-39 mnemonic in the public section and
> asserts `errors.Is(err, ErrRecordNotPermitted)`. Group *before* the allow-list
> and that record reaches `cardKey` first, the sentinel changes, and the test
> fails — which then invites "fixing" the test rather than the ordering.

**The required ordering, normative for this task:**

1. Pass 1 (lowercase) and pass 2 (classify + allow-list) run **exactly as they do
   today**, unchanged, over every record. Nothing moves into or before this loop.
2. Only then compute the grouping — over records already admitted, which in
   `SectionPublic` are all `ClassMDMK` by construction, so `cardKey`'s
   fail-closed branch is genuinely unreachable and stays that way.
3. Backfill the new fields on `out` in a step **after** the loop.

Compute `groupCards` **once**. Either thread `keys`/`groups` out of
`decodePublicSet` by widening its return type, or have `AdmitSection` call
`groupCards` itself and pass the result in. Do not call it twice — two callers
is two chances to disagree about what a card is.

Add to `AdmittedRecord`:

```go
// HRP is 'd' (md1) or 'k' (mk1) for a ClassMDMK record IN THE PUBLIC SECTION,
// and 0 for every record in the encrypted section -- INCLUDING ClassMDMK ones,
// which §6.3 explicitly permits there and which vectors C and F actually
// carry. That is not a statement about what those records are; it is that
// pass 3, the only place grouping is computed, runs for SectionPublic alone.
// See F-77.
//
// It comes from the §6.3 card grouping seal already performs -- the UI must
// never re-derive it, or the plate list and the decode can disagree about what
// a record is.
HRP byte
// CardIndex/CardTotal identify which (HRP, chunk_set_id) card this record
// belongs to, 1-based, among cards of the SAME HRP. PlateIndex/PlateTotal
// identify this record within that card.
CardIndex, CardTotal   int
PlateIndex, PlateTotal int
```

**Label rule, which generalises §10.2.2's example rather than contradicting it:**

| case | label |
| --- | --- |
| one card of this HRP | `mk1 1/2` — plate index within the card |
| several cards of this HRP | `mk1 2/3 · 1/2` — card, then plate within it |

§10.2.2's `mk1 1/2` / `md1 2/3` examples are single-sig, where there is exactly
one card of each HRP, so the first row reproduces them exactly. The second row
exists because a 2-of-3 is `mk1` ×6 across **three** cards, and a flat
`mk1 1/6 … 6/6` silently conflates three distinct cosigners — the operator
cannot tell which cosigner a plate belongs to, which is §6.4's
"incomplete-backup-believed-complete" hazard wearing a label.

This mirrors the fork's own precedent: `bundlePlatePlan` (`gui/bundle_flow.go:300`)
carries exactly `cardIdx`/`cardTotal`/`plateIdx`/`plateTotal` for this purpose.

**These fields are populated for `SectionPublic` only, and the reason is NOT
that secrets aren't cards.**

> **Round 2 correction.** The round-1 fold claimed encrypted-section records "can
> be `ms1` or a bare mnemonic, neither of which is a card at all." **That is
> false.** SPEC §6.3 is explicit — "The encrypted section may carry anything —
> `ms1`, `mk1`, `md1`, a BIP-39 mnemonic". `permitted()` (`seal/record.go:147`)
> codes it: `if c == ClassMDMK { return true }`, **unconditional**, not gated on
> section. And it is not theoretical — in `seal/testdata/vectors.json`, vector
> C's secret set is `ms1`×1 / `mk1`×2 / `md1`×3, and vector F's is `ms1`×3 /
> `mk1`×6 / `md1`×6. **Twelve of vector F's fifteen secret records are cards.**

The fields are zero for **every** encrypted-section record — `ClassMDMK` ones
included — for one reason only: **pass 3 is the sole place grouping is computed,
and it runs only for `SectionPublic`** (`seal/record.go:186`).

**The trap this leaves B2, named properly:** B2 labels secret plates (§10.2.2),
its secret records routinely *are* `mk1`/`md1` cards needing exactly the
`mk1 1/2` / `mk1 2/3 · 1/2` labels Task 4a builds — and it will find no grouping
to reuse, because pass 3 never ran for them. The fix is to extend pass 3's
grouping over the encrypted section's `ClassMDMK` subset using the same
`groupCards`/`cardKey`, **not** to re-derive classification in `gui`. Filed as
**F-77**.

**This is an additive change to a merged Phase A type.** It adds no behaviour and
no new §6.3 logic — it publishes a grouping that already happens. Phase A's
vector tests must pass **unchanged**, and per round 1 that is a live constraint,
not a formality.

*Machine-checked while folding round 1, so do not re-derive it:* the suite
contains exactly **one** `AdmittedRecord` composite literal,
`seal/record_test.go:440`, and it uses **field names** (`{Record: …, Class: …}`),
not positional initialisation — so added fields do not break it. `grep -rn
"DeepEqual" seal/*_test.go` returns nothing, so no test compares
`AdmittedRecord`s structurally either. **The additive change is safe on both
counts.** If a test nonetheless needs editing during implementation, the
ordering above was violated.

**Tests:** vector F (2-of-3, three secret records) and vector G (2-of-3 mixed,
public section spanning four cards) pin both label rows. Assert the **rendered
label strings**, not the struct fields — a test that reads back `CardIndex` pins
the plumbing and not the thing §10.2.2 actually specifies.

**Tests (Task 4):**
- Vector G — a public section spanning **four cards** — renders across pages and
  every record is reachable. This is the case that a non-paged `ChoiceScreen`
  would silently truncate at ~7 (`gui/gui.go:1455`).
- Labels: vector G's `mk1` records span multiple cards, so assert the
  `mk1 2/3 · 1/2` form, and a single-sig vector for the plain `mk1 1/2` form.
- Back returns to the main menu from any page.

> **Mutation check.** Replace the paged list with a `ChoiceScreen` and confirm
> the vector-G test fails. If it passes, the test is asserting on the model
> rather than on what was drawn, and §10.3's whole constraint is unpinned.

---

## Task 5 — engrave a public record

The public section is `mk1`/`md1` text by §10.2.1's allow-list, which is exactly
what `validateMdmk` (`gui/gui.go:1982`) already lays out and what
`NewEngraveScreen` (`gui/gui.go:2559`) already cuts.

**Do NOT reuse `mdmkFlow` (`gui/gui.go:2024`). (R0 round 0, finding 3.)** An
earlier draft of this plan called for it, citing "the `md1`/`mk1` inspect paths"
as a benefit. They are not a benefit here — they are a dead end.

`mdmkFlow` prepends an "Inspect key" / "Inspect descriptor" choice that calls
`mk1GatherFlow` (`gui/mk1_inspect.go:156`) or `md1GatherFlow`
(`gui/md1_gather.go:79`). Both prime a **fresh gatherer** with only the single
string handed to them, and when that alone is not a complete card — true for
**every chunked record**, which is the ordinary case — they open
`ctx.Platform.NFCReader()` (`gui/mk1_inspect.go:163`, `gui/md1_gather.go:87`)
and wait for the operator to tap the remaining **physical NFC tags**.

A payload-derived record has no tags to tap. The payload already holds every
chunk in `p.Public`, and the gatherer has no way to reach them. Single-sig's
`md1` alone is 3 records; vector G's `md1` is one 6-chunk card. So Inspect
strands the operator on a scan-waiting screen for the common case.

**Compose the three pieces directly instead:** `validateMdmk` → `ChoiceScreen`
over the returned variant labels → `NewEngraveScreen`. That is `mdmkFlow` minus
the Inspect branch, and it is what B1 actually needs.

*(Inspecting a payload record is a legitimate thing to want — the data is all
present, it just needs a gatherer primed from `p.Public` rather than from NFC.
It is not in B1. Filed as F-76.)*

**The one thing B1 must not inherit from `mdmkFlow`:** it takes `mdmkText`
(a `string`). B1 holds `AdmittedRecord.Record` as `[]byte` — deliberately, so
B2 can zero it. Converting to a string here is harmless in B1 (public data) but
establishes a pattern that is **actively wrong** in B2, where the same call
shape on a secret record produces an unwipeable copy.

> **Write the conversion at the call site with a comment naming this**, so B2's
> author sees it. Do not add a `String()` helper on `AdmittedRecord` — a helper
> is an invitation to call it on a secret.

**Tests:**
- Selecting an entry reaches the engrave screen with the record's bytes;
  returning from engrave lands back on the plate list, **on the same page**.
- **No path from the plate list reaches `mk1GatherFlow` or `md1GatherFlow`.**
  Assert with `testPlatform.NFCReader` returning a reader that fails the test if
  opened. Without this, the Inspect branch can be reintroduced by a later edit
  and nothing notices — the failure only shows up on a device with an NFC reader
  attached, which no test has.

---

## Task 6 — sealed payloads terminate honestly

`p.Header.Sealed()` → after Task 2's hash screen, a terminal screen stating the
payload is sealed and that unlocking is not available in this build.

It must **not** prompt for words, and must **not** fall through to the plate
list — `p.Public` on a sealed payload is a legitimate record set, and engraving
it while silently dropping the encrypted half is §6.4's "incomplete backup
believed complete", the worst available outcome.

**Test:** a sealed vector reaches the terminal screen; the plate list is never
constructed; no word-entry screen is reached.

---

## Task 7 — hardware pass on the SH2 (closes F-73)

F-73's owning phase is "Phase B's hardware pass, or the first SH2 session —
whichever comes first" (`design/FOLLOWUPS.md:823`). The SH2 is available, so
**F-73 closes in B1** and is not deferrable past it.

What B1 proves that Phase A could not:

- The XIP read at the **normative** `seal.PayloadAddr` = `0x10E00000`
  (`seal/read_tinygo.go:28`). Phase A's on-silicon result was at `0x10300000` on
  a 4 MB Pico 2, where `0x10E00000` **does not exist** and an XIP read aliases
  to `0x10200000`. Continuity §3 records this trap; do not re-derive it.
- Flash with `~/bin/sh/sh2-flash`, never `picotool` by hand — the build output
  is unsigned and a hand-flashed image will not boot the machine.

**Procedure:**
1. `me seal` a known payload (vector D or G) to a data-family UF2 for
   `0x10E00000`.
2. Load it, reboot, confirm the menu entry **appears** (§10.1 positive path).
3. Confirm the hash on screen **byte-matches** what `me hash` reports on the
   host for the same cards. This is the first end-to-end check of §6.6 across
   host and device on real silicon.
4. Erase the region, reboot, confirm the entry is **invisible** (§10.1 negative
   path, this time at the real address).
5. Record the results verbatim in `design/` and close F-73.

> **Watch what you paste** (continuity §4). Two commit messages last cycle
> claimed results that were never checked. Record what the screen actually
> showed, `&&`-chain the commands, and read the output *at* it.

---

## Gate coverage — state this in the R0 brief

- **`scripts/plan-cite-gate.sh` applies and MUST be run.** Every `file:line` and
  `pkg.Symbol` in this plan resolves against the real source, and it prints the
  line. Run it before dispatch and after every fold.
- **`scripts/plan-build-gate.sh` does NOT apply.** It extracts ```rust blocks
  into a scratch crate; this plan carries **Go**. Nothing in this plan is
  compiled by any gate.
- **Therefore the Go fragments in Tasks 1–6 are UNCHECKED** and are a reviewer's
  execution pass. This is a real blind spot, stated rather than hidden. Given
  continuity §4 — three of four folds last cycle introduced a defect, several of
  them compile errors — this gap is the most likely source of a wasted round.
  Filed as a follow-up below.

**Machine-verified before this plan reached a reviewer** (do not re-derive):
- `ys := [3]int{` is at `gui/gui.go:1857`, indexed `int(clk.Button - Button1)`.
- `bundleReviewFlow` is at `gui/bundle_flow.go:227`; its nav call at `:275` is
  Back / Page / OK. *(The `:224` cited by the SPEC and the Phase A plan was
  stale — it lands on a comment line. Corrected in the SPEC by this cycle.)*
- `ConfirmWarningScreen` exists at `gui/gui.go:240`, laid out at `:336`,
  returning `ConfirmYes`/`ConfirmNo`/`ConfirmNone`.
- The `Platform` interface has exactly **three** implementations:
  `cmd/controller/platform_sh2.go:564`, `cmd/emu/platform.go:189`,
  `gui/gui_test.go:428`.
- `AppendEvents` (`cmd/controller/platform_sh2.go:368`) appends an event only on
  touch or stdin — timer expiry and wakeups return the slice unchanged. So
  `a.idle.start` is a true last-physical-input timestamp. *(Relevant to B2, not
  B1; recorded here so B2 does not re-derive it.)*

---

## What B1 does NOT cover

- **§10.2 steps 5–9** — word entry, checksum validation on the §8.1-normalised
  form, the ~31 s KDF and its progress indicator, AES-GCM open, and the retry
  loop that keeps the hash on screen. B2.
- **§10.2.2 session lifecycle** — secrets offered first, each wiped as its plate
  leaves by any route including a cancelled engrave. B2, and it is the hardest
  thing in the feature.
- **§10.2.4 residency-keyed idle wipe.** B2. Note the timer it needs does **not**
  exist in a flow-visible form today: `idleTimeout` (`gui/gui.go:2801`) drives
  the screensaver from `Run`'s frame loop and is invisible to flows. B2's chosen
  approach is a last-input timestamp on `Context`, with the engrave screen
  pausing the timer by simply not consulting it.
- **Wiping of any kind.** B1 holds only public records.

---

## Follow-ups filed by this plan

- **F-74 — no build gate covers Go plans (owning phase: before B2's plan
  review).** `plan-build-gate.sh` is Rust-only, so every Go fragment in a Plan B
  document reaches reviewers uncompiled. Continuity §4 measured folds as the
  dominant defect source and compile errors as a recurring class among them. A
  Go equivalent — extract fragments into a scratch package alongside the real
  `gui`, `go build` it — would close the same loop `plan-build-gate.sh` closed
  for Rust. Not built in B1: it is tooling, and bundling it into a feature commit
  is the third-commit case the standard workflow separates out.
- **F-76 — inspecting a payload-sourced card (owning phase: B2 or later; NOT
  B1).** `mk1GatherFlow`/`md1GatherFlow` prime a gatherer from one string and
  then wait on NFC for the rest of the chunk set, so they cannot inspect a
  payload record whose remaining chunks are already in `p.Public`. The data is
  all present; the gatherer just needs priming from the payload instead of from
  a tag. Out of B1's scope — B1 engraves, it does not inspect. Found by R0
  round 0, finding 3.
- **F-75 — stale `gui/bundle_flow.go:224` citations outside the SPEC (owning
  phase: ownerless residue).** Corrected in `SPEC_encrypted_payload_delivery.md`
  by this cycle. Two stale copies remain, in
  `IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseA.md:638` and
  `CONTINUITY_2026-08-07b.md:148`. Both are shipped records; per F-72's
  precedent they are annotated, not rewritten.
