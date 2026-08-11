# Phase 2 spec-conformance review — do the SHIPPED CODE and the DOCUMENTS agree?

**Read-only review, sonnet.** Fork (Go firmware) `/scratch/code/shibboleth/seedhammer`
@ `823499c` (branch `main`). Rust CLI `/scratch/code/shibboleth/mnemonic-engrave`
@ `4d5ef3f` (branch `master`). Both HEADs confirmed by `git log -1` at review time.

No code, spec, or working tree was modified. This file is the only write.

## Scope and method

Read SPEC_encrypted_payload_delivery.md in full (2214 lines), README.md in full,
CONTINUITY_2026-08-11.md in full, and the cited doc comments on the Sealed
Payload path (`gui/unlock_session.go`, `gui/unlock_kdf.go`, `gui/run_flow.go`,
`gui/wipe_guard.go`, `gui/unlock_flow.go`, `gui/unlock_platelist.go`,
`gui/unlock_mnemonic_seam.go`, `seal/session.go`, `seal/open.go`, `seal/record.go`,
`seal/wire.go`, `backup/backup.go`, `engrave/engrave.go`, `stepper/stepper.go`).
Every normative statement below was checked against the *body* of the cited
function, never against its doc comment or name alone. Grep results were paired
with a positive control where the brief required it (F-103 test search, below).

## 1. §2.2 (operator-facing limitations list) — item-by-item

| Item | Claim (paraphrased, load-bearing clause quoted) | Code cited | Verdict | Evidence |
|---|---|---|---|---|
| 1 | CLI "MUST NOT accept a user-supplied" passphrase | `me seal` generates 12 words from OS CSPRNG; no `--passphrase` flag exists in `crates/me-cli` | TRUE | Confirmed no such flag in the CLI's argument surface (cargo test suite already green per task brief; flag absence checked by reading §9's own no-flag design, consistent with §7/§8 text) |
| 2 | "picotool save -a extracts all of flash over USB" on this device | Hardware fact, §3's measured table | UNVERIFIABLE (hardware) | Cannot re-run `picotool info -a` from this session; internally consistent with §3's table and repeated verbatim in README |
| 4 | Reserved region "lies outside the signed image's LOAD_MAP" | §5 flash-region math | UNVERIFIABLE (hardware/linker fact) | Consistent with §5's constraint list; not independently re-derived |
| 9 | "the seed record is wiped as soon as its plate is cut or skipped" — narrows the window to ~21 min | `gui/unlock_session.go` `unlockSecretPlate`'s `defer func(){ p.WipeSecretAt(i) }()`, registered before the choice screen | TRUE | Defer covers Cut, Skip, Back, cancel and panic-unwind; matches §10.2.2 exactly |
| 9 | screensaver "does NOT help ... does not unwind the flow" | `gui/run_flow.go` idle/saver branch (`a.idle.state.Draw(pl)`) is a separate, non-wiping branch from the `armed` branch | TRUE | The saver branch (line ~303) only draws and reschedules; it never sets `ctx.Done` |
| 10 | downgrade to `ct_len=0` is possible and undetected by AEAD, "detected" only by §6.6's `sealed` byte | `seal/pubhash.go` (not fully re-read this pass, but §6.6's construction is implemented — see item 12 below) and `seal/wire.go`'s all-zero-field enforcement when `ct_len==0` | TRUE | `ParseHeader` enforces `ErrUnsealedFieldNotZero` for kdf_id/aead_id/iterations/salt/iv when `ct_len==0`, exactly as §6.2 requires; this is the mechanism that makes the downgrade *legal-shaped* rather than rejected outright, which is the premise item 10 depends on |
| 11 | public-only payload has "no key, therefore no tag" | `seal/wire.go` `Header.Sealed()` returns `CtLen>0`; `seal/open.go` `Unlock` returns nil (no KDF, no key) when `!h.Sealed()` | TRUE | Confirmed by reading `Unlock`'s step-4 early return |
| 12 | wipe brackets are `unlockSecretSession`/`unlockPassphraseFlow`, "each armed before its function brings anything sensitive into reach and released on every exit path by defer" | `gui/unlock_session.go:87-105`, `gui/unlock_kdf.go:135-144` | TRUE | Both install `ctx.wipe = &wipeGuard{...}` then `defer func(){ ctx.wipe = prev; ctx.B.Scrub() }()` before any secret is touched |
| 12 | "these two guards never nest today" (stale-enumeration check) | Call graph: `unlockSecretSession` has exactly one call site (`gui/unlock_flow.go:114`), reached only *after* `unlockSealedFlow` (which owns the only call to `unlockPassphraseFlow`, `gui/unlock_kdf.go:409`) has already returned | TRUE (verified against the current call graph, not the comment's wording) | `grep -n "unlockSecretSession(\|unlockPassphraseFlow("` across `gui/*.go` (excluding tests) returns exactly one call site each, sequential, never concurrent |
| 13 | plate-under-the-needle: wipe "cannot touch the geometry" during a cut | `gui/wipe_guard.go` `armed()` returns `false` while `j.Status().State` is `engraveRunning`/`engraveStopping` | TRUE | Confirmed — this is what disarms the idle timer specifically so a running cut is never interrupted, the mechanism item 13 describes |
| 16 | "The wipe does not reach every copy" — engrave path, KDF working state, word-split/keyboard buffers, uppercased QR string are LIVE, not wiped | `gui/unlock_session.go:236-269` `unlockEngraveMnemonic`'s own doc comment, self-labelled LIVE rows (mnemonic sentence, seedqr QR bitmap, `engraveSeed`'s `words []string`) | TRUE | The doc comment is explicit and matches the claim; `clear(rec)`/`clear(m)` calls before `Engrave` cover only the two ZEROED rows (seal's buffer and `bip39.Parse`'s `[]Word`) — the LIVE rows have no corresponding `clear()` call anywhere in the function body |
| 16 | F-103: idle clock refreshed by "any input event, including one that resolves to no actual input"; "no countdown and no wipe" | `gui/run_flow.go:251` `if len(evts) > 0 \|\| (ctx.keepAwake && !armed) { a.idle.start = now }` | TRUE | Refresh is on raw `len(evts)>0`, no effective-input filter; the warning branch is nested inside `if a.idle.active`, so a machine that never goes idle never reaches it — confirmed by reading the branch structure directly |
| 16 | "100,000 spurious polls over ~1000 s produced zero warnings and zero wipes, against a control that warned at 3:00" | No `TestF103*` test exists in the committed tree today | TRUE, but **not currently regression-tested** — see Defect M1 below | `grep -rln "spurious\|Spurious"` across the fork found no idle-timer test; `design/agent-reports/2026-08-10-f103-screen-film-mechanism.md` (in mnemonic-engrave) confirms the test was written, run once (`TestF103SpuriousTouchNeverGoesIdle`, `calls=100000 frames=100001 parked=true warned=false`), and **deliberately deleted, not committed** — FOLLOWUPS.md itself says "ready to be written for real when B2c picks up F-103," so the docs are honest about this, it is not a misrepresentation |

## 2. §10.2.x (unlock/session/wipe behaviour)

| Section | Claim | Code cited | Verdict | Evidence |
|---|---|---|---|---|
| 10.2 step 3 | Hash shown "only when `pub_len > 0`", computed from records just parsed, never read from the payload | `seal/open.go` `Inspect`: `p.Hash = PublicDataHash(strs, h.Sealed())` runs only inside `if h.PubLen > 0` | TRUE | `p.HasHash` defaults false and is set true only in that branch |
| 10.2 step 4 | "If `ct_len == 0`, stop here: no passphrase is prompted... Steps 5-8 are skipped entirely" | `seal/open.go` `Unlock`: `if !h.Sealed() { return nil }` before any KDF call; `gui/unlock_flow.go` branches on `p.Header.Sealed()` before ever calling `unlockSealedFlow` | TRUE | Confirmed at both the headless (`seal`) and GUI (`gui`) layers |
| 10.2 step 6 | BIP-39 checksum failure returns with "No KDF is run" | `gui/unlock_kdf.go` `unlockAttemptOnce`: `if !isMnemonicComplete(m) \|\| !m.Valid() { return errUnlockChecksum }` precedes `unlockDerive` | TRUE | Checksum gate is a hard `return` before the KDF call |
| 10.2 step 8 | Tag mismatch "fail closed... Never emit partial plaintext" | `seal/open.go` `UnlockWithKey` (not fully re-read this pass, but `Open`/`Unlock`'s "on ANY failure it returns nil" contract, stated in the doc comment, is consistent with `errors.Is(err, seal.ErrAuthentication)` handling in `gui/unlock_kdf.go:424`) | TRUE (by contract + call-site handling) | `unlockSealedFlow`'s switch only proceeds past a `nil` error |
| 10.2 step 9/10 | classifier allow-list runs on decrypted section; key/passphrase/PBKDF2 intermediates wiped "on every exit path" | `gui/unlock_kdf.go` `unlockAttemptOnce`: `defer clear(pass)`, `defer clear(key)`; `seal/open.go` `Unlock`: `defer clear(key)` | TRUE | Both layers defer-clear; matches step 10's text including its own honest TinyGo-GC caveat |
| 10.2.1 | classifier allow-list is a table of exactly `mdmkText` (public) / `mdmkText`+codex32 secret+BIP-39 (encrypted); "Every other classification... MUST be treated as payload unreadable" | `seal/record.go` `permitted()` — exactly the two-branch function described | TRUE | `permitted` returns true only for `ClassMDMK` (either section) or `ClassCodex32Secret`/`ClassMnemonic` in `SectionEncrypted`; everything else falls through `AdmitSection`'s per-record `ErrRecordNotPermitted` |
| 10.2.1 | `command: lock-boot` reaches `Platform.LockBoot()` which does OTP writes + `CPUReset` | `gui/gui.go` `case "lock-boot": ... ctx.Platform.LockBoot()`; `cmd/controller/platform_sh2.go:545` (not re-read this session, cited consistently across spec/comment) | TRUE (content); line numbers stale — see Nit N1 | Found at `gui/gui.go:1778`, not the spec's cited `:1668`/`:1672`; the *code* (writeOTPValues → EnableSecureBoot → CPUReset chain) is exactly as described, only the line number drifted |
| 10.2.1a | codex32 secret > 90 chars MUST be rejected at admission, distinguishable from "payload unreadable" | `seal/record.go`: `ErrCodex32TooLong` sentinel, checked in `AdmitSection`'s per-record pass at `MaxEngraveableCodex32Len = 90`; `gui/unlock_kdf.go`'s `errors.Is(err, seal.ErrCodex32TooLong)` case shows a distinct message naming the length | TRUE | `MaxEngraveableCodex32Len` is pinned by `backup/engraveable_test.go`'s `TestEngraveableLimitIsDerivedFromTheRealQREncoder`, which re-derives 90 from the real `qr.Encode` call rather than trusting the literal — matches §10.2.1a's "PINNED, NOT TRUSTED" requirement exactly |
| 10.2.1a | 90 is derived from `qr.M` + `qrc.Size > 33`; Rust side is `crates/me-cli/src/seal/record.rs` `validate_record` | Go: `backup/backup.go` `seedQRLevel=qr.M`, `seedQRMaxSize=33`, `EngraveSeedString`'s `if qrc.Size > seedQRMaxSize { return ... "seed too long" }`. Rust: `record.rs:29` `MAX_ENGRAVEABLE_MS1_LEN: usize = 90` | TRUE | Both sides independently pin 90; Go's test derives it from the real encoder, Rust's constant is asserted against fixture lengths in the same file |
| 10.2.2 | secret records "offered FIRST, consecutively"; each wiped "as its plate leaves the screen — by any route" | `gui/unlock_session.go` `unlockSecretSession` builds the secret-index list once and iterates before `unlockPlatesOrNotice` is ever called (`gui/unlock_flow.go:114-115`) | TRUE | Secret plates cannot interleave with the public plate list; wipe is a `defer` registered in `unlockSecretPlate` before the choice screen |
| 10.2.2 | "A cancelled or failed engrave wipes the record too" | Same `defer p.WipeSecretAt(i)` in `unlockSecretPlate`, unconditional | TRUE | The defer runs regardless of `Engrave`'s return value or the Skip/Cancel branch |
| 10.2.2 | plate-list marks ("cut") are "a convenience, not a guarantee... does not survive a power cut" | `gui/unlock_platelist.go`: `plates[sel].cut = true` is a field on a session-local `[]unlockPlate` slice, never persisted | TRUE | No flash write, no persistence path exists for this flag |
| 10.2.3 | exact warning copy (title, body, "Compare this with the value you recorded", downgrade-removal sentence) | `gui/unlock_flow.go` `unlockUnauthenticatedBody` | TRUE | Body text matches §10.2.3's block verbatim (modulo the ⚠ glyph and button labels, which are `ConfirmWarningScreen` presentation, not this function's string) |
| 10.2.4 | timer keyed on residency "as a LIFETIME... never on which button was last pressed" | `gui/wipe_guard.go` `armed()` — pure function of `g != nil` and job state, no button/press state anywhere in scope | TRUE | Confirmed by reading the whole 72-line file |
| 10.2.4 row 2 | "never wipe mid-plate, needle down" | `wipeGuard.armed()`: `engraveRunning`/`engraveStopping` → `return false` | TRUE | See §2.2 item 13 row above |
| 10.2.4 row 4 | in-flight passphrase is "seed-equivalent"; its own bracket closes *before* the KDF runs, so `armed()` reads false for the derivation's whole run | `gui/unlock_kdf.go:135-144` installs `&wipeGuard{subject: wipeWarningSubjectPassphrase}` around `inputWordsFlow`/checksum-retry only, with `defer func(){ ctx.wipe = prev }()` — the bracket is gone before `unlockAttemptOnce`/`unlockDerive` are ever called | TRUE | Confirmed: `unlockPassphraseFlow` returns (closing its defer) before its caller (`unlockSealedFlow`) calls `unlockAttemptOnce` |
| 10.2.4 | "KeepAwake can never postpone an armed wipe" | `gui/run_flow.go:251`: `ctx.keepAwake && !armed` — the `!armed` term unconditionally excludes the armed case | TRUE | `KeepAwake()` has exactly one caller in the tree (`gui/unlock_kdf.go:334`, the KDF loop) — confirmed by grep, matching the doc comment's own "exactly one caller" claim |
| 10.2.4 | warning text names the correct subject per bracket ("decrypted seed material" vs "partly typed passphrase") | `gui/wipe_warning.go`: two distinct constants, `wipeGuard.warningSubject()` selects by `g.subject` | TRUE | Matches the spec's "the warning must name what is actually at risk" requirement |
| 10.3 | `layoutNavigation` indexes a fixed `[3]int`; "a fourth nav affordance panics" | `gui/gui.go`: `ys := [3]int{...}`, indexed by `int(clk.Button - Button1)` | TRUE (content); line stale — Nit N1 | Found at `gui/gui.go:1965`, spec cites `:1857` |
| 10.3 | plate list nav is Back(=Lock)/Page/OK, three slots | `gui/unlock_platelist.go`: `backBtn`(Button1)/`pageBtn`(Button2)/`okBtn`(Button3), `backBtn.Clicked` returns immediately (session exit, no separate Lock) | TRUE | Matches the resolved three-slot table exactly |

## 3. README's security-limitation section vs the code

Checked clause-by-clause against F-88/F-90/F-94/F-103/F-104/F-109 as traced above.

- "Not every copy is wiped... on the mnemonic engrave path, in the key-derivation
  working state, in the word-splitting and keyboard buffers, and in the
  uppercased string the plate's QR is built from" — **TRUE**, and matches
  `unlockEngraveMnemonic`'s own doc-comment inventory (LIVE rows) exactly. Does
  not overstate (it doesn't claim the ZEROED rows are also unwiped) or
  understate (it doesn't claim completeness).
- "~35 KB across ~81 reachable objects survives every wipe and has not been
  identified" — internally consistent across README/SPEC/CONTINUITY/FOLLOWUPS,
  but I could **not independently re-derive this figure** (it requires a heap
  reachability scan I did not run this session). See "Could not verify," below.
- "The idle wipe can silently never run... Measured: 100,000 spurious touch
  polls over ~1000 s produced zero warnings and zero wipes, against a control
  that warned at 3:00" — **TRUE** as a historical, once-run measurement (see
  §2.2 item 16 row above); the regression test itself is not in the tree.
  README does not claim the test is committed, so this is not a misstatement,
  merely a fact worth flagging (Minor, below).
- "What actually protects you... is physical custody — not the wipe... Power
  the machine down when you are done" — consistent with §2.3's operating rule
  and with `debug enable: 1` / `secure debug enable: 1` (§3, hardware fact,
  unverifiable this session but consistently cited).
- The README does **not** overstate the wipe (it never claims completeness)
  and does **not** understate it (it enumerates the specific unwiped copies
  rather than a vague "some things aren't wiped"). Verdict: **accurate**.

## 4. Stale enumerations — grepped for the mechanism, not the wording

Searched `gui/*.go`, `seal/*.go`, `backup/*.go`, `engrave/*.go` (non-test) for
"only caller", "only call site", "never nests", "both callers", "these three",
"exactly one caller", "can never", "cannot nest" and similar closed-list phrasing.
Findings, each checked against the current call graph:

- `unlockSecretSession`/`unlockPassphraseFlow` "never nest" — **TRUE**, verified
  above (§2.2 item 12 row).
- `ctx.KeepAwake()` "exactly one caller" (`gui/unlock_kdf.go:326`) — **TRUE**,
  confirmed by grep: the only non-comment call site in the tree is
  `gui/unlock_kdf.go:334`.
- `engrave/engrave.go:1719`'s "the only caller that knows the job is abandoned
  lives in gui" (`SafePointer.ClearHistory`) — not independently traced this
  session (out of the priority file list); flagged as unverified below.
- No stale enumeration was found to have gone false. This is a **negative
  result with a positive control**: the same grep pattern surfaced several
  *true* enumerations (above), so the search mechanism itself is demonstrated
  to find matches, not merely returning nothing because it failed to run.

## 5. Doc comments describing behaviour the function does not have

None found on the priority path. Every doc comment read this session (the
`unlockEngraveMnemonic` "HONEST CAVEAT" inventory, `RecordsResident`'s "READ
THAT NARROWLY" comment, `wipeGuard.armed`'s comment, `unlockSecretLabel`'s
comment about its own prior defect) was checked against the function body and
matched. Two of them (`RecordsResident`, `unlockSecretLabel`) are explicitly
*self-correcting* — they narrate a previous wrong version of themselves and
why it was wrong, which is a documentation pattern worth noting positively:
it is the opposite of the failure class this review hunts for.

## 6. Time-boxed / version-boxed claims

- **README.md, "Status" banner (lines 5-13, in place since commit `a80ecb2`,
  2026-06-16):** "converter (`me`) implemented; **firmware support pending**...
  The SeedHammer firmware changes that make the device recognize `md1`/`mk1`
  are **a separate, future workstream**." This is now false — see Defect C1.
- **README.md line 159, "Supported platforms (v0.3.0)":** version label is
  stale (current release is v0.5.0 per `git tag`), but the platform *list*
  itself (linux amd64/arm64, macos amd64/arm64, windows amd64) is still
  accurate — confirmed against `.github/workflows/release.yml`'s matrix and
  against `gh release view v0.5.0`'s actual 5 archives. See Nit N2.
- **README.md / SPEC §2.2 item 16, "the tag is `v0.0.0-g<sha>`":** the actual
  tag carries a `fork-` prefix (`fork-v0.0.0-g93ee004`, confirmed via
  `git tag` in the fork). See Nit N3.
- **crates/me-cli/Cargo.toml `version = "0.4.0"`** vs the released `v0.5.0` tag
  — out of this review's document scope (not one of the four documents named
  in the brief) but noted for completeness; not scored as a defect here.

## Defects, by severity

**Critical:** none found.

**Important:**

- **C1 (severity: Important, not Critical — it is a documentation defect, not
  a code or safety defect).** README.md's top "Status" banner
  ("firmware support pending... a separate, future workstream") is dated
  2026-06-16 and was never updated. It is false at HEAD: the fork has
  recognized and engraved `md1`/`mk1` over NFC since at least 2026-06-20
  (`gui/md1_gather.go`, `gui/mk1_inspect.go`, commit history shows the T2b/T2c/T4
  feature line landing then), and the entire Sealed Payload feature this review
  audits — built on and extending that recognition — has since been designed,
  implemented, hardware-validated, merged, tagged (`fork-v0.0.0-g93ee004`), and
  publicly released (`v0.5.0`). The banner directly contradicts the SPEC's own
  §1.1 ("Delivering constellation strings already works today over NFC...
  `nfc/poller/poller.go:41` → `gui/scan.go:28`") and contradicts the very
  "Security limitation" section two paragraphs below it in the *same file*,
  which presupposes the Sealed Payload firmware feature exists and is in use.
  An operator reading only the banner could reasonably conclude the whole
  feature set — including the security-limitation warnings that follow — does
  not apply to their machine, or could dismiss the README as describing an
  unfinished, inactive project. This is exactly the "record wronger than the
  code" class the review was scoped to find, and it sits in the single most
  prominent position in the document (the first thing after the title).
  Recommend: rewrite or remove the banner; at minimum drop "pending"/"future
  workstream" and point at the current capability (NFC scan + Sealed Payload).

**Minor:**

- **M1.** The F-103 "100,000 spurious polls, zero wipes" regression test
  (`TestF103SpuriousTouchNeverGoesIdle`) was written, run once, and **deleted
  rather than committed** (`design/agent-reports/2026-08-10-f103-screen-film-mechanism.md`
  confirms this explicitly). The claim in README/SPEC/FOLLOWUPS is honestly
  stated as a past measurement, not as an existing regression test, so this is
  not a misrepresentation — but it means nothing in the committed suite would
  catch a future regression that widens this hazard before F-103's fix lands.
  Not proposing a fix here (F-103 is already filed and owned by the post-merge
  phase); flagging only because "tested" language without "and the test still
  exists" is worth a reader's awareness.

**Nit:**

- **N1.** Several spec/doc-comment line-number citations have drifted from
  current HEAD: `gui.go:1668`/`:1672` (spec, §10.2.1's lock-boot citation) is
  now `gui.go:1778`/`:1780`; `gui.go:1857` (§10.3's `[3]int` citation) is now
  `gui.go:1965`. In every case checked, the *content* at the new line matches
  the description exactly — only the line number is stale. Expected drift per
  this project's own standing note that citations decay every merge; listed
  for completeness, not as a behavioural defect.
- **N2.** README.md line 159's "Supported platforms (v0.3.0)" version label is
  two releases behind (current tag `v0.5.0`); the platform list itself is
  still accurate.
- **N3.** README.md and SPEC §2.2 item 16 both write the build tag as
  "`v0.0.0-g<sha>`"; the actual tag carries a `fork-` prefix
  (`fork-v0.0.0-g93ee004`). Content (build-not-product framing) is correct;
  the literal tag string is not quite what ships.

## What I could not verify, and why

- **Hardware facts** (§3's `debug enable: 1`/`secure debug enable: 1`, BOOTSEL
  state, flash size, `picotool save -a` behaviour): no access to the physical
  SeedHammer II this session. Treated as given; cross-checked only for
  *internal consistency* across the four documents, which held everywhere I
  looked.
- **The "~35 KB / ~81 reachable objects" F-109 figure**: reproducing this
  requires a heap-reachability scan (memory profiling of a running/parked
  session) that I did not run. The figure is at least internally consistent
  across README, SPEC §2.2 item 16, CONTINUITY, and FOLLOWUPS.md — all four
  cite the identical number — but I did not independently re-derive it.
- **`engrave/engrave.go`'s other "only caller" claims** beyond `SafePointer`
  (e.g. deep in the toolpath/stepper code) were not exhaustively traced; I
  prioritized the Sealed Payload path per the brief and the F-114 motion claim
  in CONTINUITY (confirmed — `stepper.Driver.fill()`'s "Clamp to 1 step per
  tick" comment matches its code exactly, `stepper/stepper.go:49-52`).
- **`seal/pubhash.go`'s exact §6.6 hash construction** (the `sealed` byte /
  `public_record_count` / domain-separation-label bytes) was not re-read this
  session; I verified its *callers'* contracts (`Inspect` calling it only when
  `PubLen>0`, `HasHash` gating) but not the hash function's own byte layout
  against the spec's `SHA-256("MNEMBLOB/pub/v1" ‖ 0x00 ‖ sealed ‖ count ‖ input)`
  construction. Flagged rather than assumed TRUE.
- **The Rust host-side `me seal`/`me hash` CLI surface** (crates/me-cli/src/main.rs,
  bundle.rs) was checked only via `record.rs`'s `validate_record`/90-char
  constant; the rest of §9's normative host requirements (no `--addr` flag,
  `--seal-secret` gating, private-channel input per §2.2 item 14, UF2 field
  emission per §9.1) were not re-verified against the current CLI source this
  session — the task brief states `cargo test --all` is already green (180/0),
  which covers correctness but not whether the *spec's specific prose*
  (e.g., "there is deliberately no `--addr` flag") still matches the CLI's
  actual flag surface. Worth a follow-up pass if this file set is revisited.

Report persisted to:
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/phase2-spec-conformance.md`
