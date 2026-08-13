# Journeys review — systemwide payloads, spec ↔ plan ↔ both implementations

Reviewer: fable (dispatched 2026-08-12). Two questions: (1) does the
implementation match the spec in both directions; (2) do the spec and the plan
let a user do the thing. Question 2 got the effort, per the brief.

Artifacts reviewed at: `mnemonic-engrave` branch `sysw-container`,
`seedhammer` branch `sysw-port` (HEAD `8ea224c`), spec and plan as on disk.
Both suites, clippy, gofmt, tinygo and wasm builds, and the shared vectors were
given as machine-verified and were not re-derived. What I ran myself: the built
`me` end-to-end (pack/show/wipe, all passphrase modes), and three scratch UI
tests in the `gui` package (written, run green, deleted; both trees left clean)
that drive the load flow and consumption from the exact bytes `me sysw pack
--region` emitted.

---

## Verdict in three sentences

The spine journey the feature exists for — pack on the host, compare the
digest, consume on the device — **closes, and I watched it close**: the digest
the device drew matched the one `me` printed, and Backup Wallet handed back the
packed mnemonic. But an entire second tier of spec-promised operator surface
has **no owning plan stage** — the §7 verification menu, the §5.5 device-side
erase/reminder, §8c's confirmation screen, §8a's second keyboard, NFC as a
source for the eight programs, and most of the admission table's admitted
cells — and the repo's own coverage map (`sysw/coverage.rs`) is the receipt:
of the ten §8.3 tests it marks `Where::Device`, the behaviour behind five
(2, 3, 13, 17, 22) does not exist anywhere in the Go tree. That is F-144's
shape again, five more times, and in every case the miss is traceable to the
plan's six stages covering plumbing and entry wiring but never the
*post-consumption* and *alternate-source* halves of the spec.

---

## QUESTION 2 — the journeys, walked

For each: the function at every step, whether I executed it or read it, and
where any hole lives (CODE / PLAN / SPEC).

### J-A. Plaintext payload, host → flash → program consumes — **CLOSES (driven)**

| step | function | how verified |
| --- | --- | --- |
| pack | `me sysw pack --no-passphrase <seed> <text:…> --region --out j1.bin` — `run_sysw`→`sysw::pack`→`bound` | **executed**: exit 0, digest `b13b d6cd … 5f7a` on stderr, §13-D3 warning printed, 65,536-byte image |
| write to 0x10D00000 | *(no named command anywhere — see Minor-7)* | gap noted; fixture written via `sysw.FileReader` exactly as `syswRegionFor` does |
| boot / carousel | `uiFlow` → `syswLoadFlow` (gui.go:1736 boot, :1771 `loadPayload` carousel entry) | **executed** (carousel form) via scratch test |
| parse + hash | `sysw.ParseHeader`, `sysw.Identity`, `sysw.PublicDataHash`/`FormatHash` | **executed**: digest screen appeared, matched `me`'s stderr digest byte-for-byte |
| operator compares | `confirmReviewScreen("Payload Digest", …)` → `compared = true` | **executed** (Button3) |
| F1 flag | `syswLoadWarnings` → "A SECRET is stored unencrypted in flash." | **executed**: screen appeared |
| session fill | `syswSession.load` (classify-once via `sysw.Classify`) | **executed** |
| program consumes | `newInputFlow` → `syswOffer(ClassMnemonic)` → `take` → `bip39.ParseMnemonic` | **executed**: chose FROM PAYLOAD, got the packed 12-word seed back (`ABANDON…ABOUT` asserted) |
| engrave | `engraveObjectFlow(obj)` | read only — pre-existing path, unchanged by this feature |

Also driven: the **decline** branch. Backing out of the digest screen shows
"Loaded, but not compared — no program will use it. Load again to compare.",
`take` then refuses — but see Minor-1: the FROM PAYLOAD menu item still
appears and silently falls through to typed entry (observed: "fell-through").

### J-B. Sealed payload, generated words — **CLOSES for word passphrases (host driven, device half read)**

Host: `me sysw pack <seed>` defaults to 12 generated words — **executed**;
passphrase printed to stderr, `digest: none — this payload has no public
section` (correct per `[digest-shown]`). Device, by code reading (no
keyboard-driving harness exists, so this half is *named, not watched*):
`syswLoadFlow` → `h.Sealed()` → `emptyBIP39Mnemonic(24)` →
`inputWordsFlow(…, wordEntryOpts{checksumGate: false, terminator: true})` →
join → `sysw.Open` (KDF via `seal.DeriveKey`/`seal.Open`) → `compared = true`
(any open authenticates, §12.2 D1) → F2 warning if below `[cliff]` → consume
as J-A. Every function exists and is wired; the conformance vectors (S-C, S-D,
S-E) prove `Open` agrees with `pack` including at 2 words and at
`pub_len == 0`.

**Two holes on this path**, both at the same step:

- **§8c's confirmation screen does not exist** (Important-5). `sysw_load.go:96`
  goes straight from `inputWordsFlow`'s return to the KDF. And in
  `inputWordsFlow` (gui.go:841–848) **Back and `done` return the same thing**
  — `entered()` — so backing out with three words filled is indistinguishable
  from finishing at three: the load flow runs the KDF on a truncated
  passphrase the operator meant to abandon, and the failure reads as "wrong
  passphrase". §8c exists verbatim to prevent this ("A `N words — unlock?`
  screen makes the truncation visible before it costs anything"); test 22 is
  unimplemented. Missing from: **PLAN** (stage 5c named the button, never the
  confirmation) and therefore CODE.
- The passphrase transits as Go strings (`strings.Join`, `NormalisePassphrase`)
  — accepted residue per §6.2.2a, but test 21's no-regrow buffer has nothing
  to assert against (Minor-4).

### J-B′. Sealed with `--passphrase-ask` ASCII — **DIES on the device (known; death point confirmed)**

`me sysw pack --passphrase-ask` works at a real tty (rpassword refuses a pipe,
correctly — executed, exit 2 on no tty). On the device the ONLY entry surface
`syswLoadFlow` offers is `inputWordsFlow`'s word keyboard, which completes
exclusively to wordlist entries — a passphrase containing any non-BIP-39 token
cannot be typed. §8a's keyboard choice is unbuilt; this was declared known and
I re-report only the journey verdict: **it dies at passphrase entry**, and §12.2
D1 (any open authenticates) means the keyboard is now the *only* blocker.
Missing from: **PLAN** (no stage owns the free-text unlock keyboard; §2.2
item 8 promised "a NEW device unlock flow" and stage 5c delivered only the
word-path halves). Compounded by Important-4: `me` also fails to enforce
`[passphrase-bounds]` on this mode, so payloads can be created today that even
the future keyboard cannot open.

### J-C. Record arrives over NFC — **PARTIAL: pre-existing paths only; the eight-program promise is unwired**

What exists (read; stage 6 delivered it): `cmd/emu/nfc.go` + `nfc_js.go`
(`shNFC.present(...)`) feed `Platform.NFCReader()`; consumers are the
pre-existing `StartScreen` scan → `engraveObjectFlow` (mnemonic, codex32,
descriptor, md1/mk1 → engraving) and the bundle/md1 gatherers.

What the spec promises and nothing implements:

- §3.1 NORMATIVE: `seedEntryFlow … offers Typed / Scanned / Payload`. The code
  (`derive_xpub.go:88`, `syswSeedPicker`) offers **TYPE IT / FROM PAYLOAD** —
  no Scanned. A seed on a tag cannot feed Account Xpub, Single-Sig, Multisig
  or BIP-85.
- `scan.go` has no `text:`/`pass:` case → a tag cannot feed Engrave Text or
  BIP-39 Password (`errScanUnknownFormat`).
- §3.3.2a: "NFC records go through the SAME function." `admits()` has **zero
  production call sites**; F4 (`flagNFCNoIntegrity`) and `srcNFC` are
  constructed in exactly one place — a unit test. F4 can never fire.

Missing from: **PLAN.** Stage 6 built the emulator *source*; no stage ever
sequenced the *consuming* half (the Scanned option, scanner cases, admission
call, F4 rendering). Same anatomy as F-144: the spec described the behaviour,
each stage did its local job, nobody owned the join. The code deviates from a
NORMATIVE spec line (§3.1), so the spec is right and the work is unfinished —
this is not a case for demotion, since §2.1 made NFC-for-everything a
deliberate, emphasised capability.

### J-D. Getting rid of a payload (§5.5) — **host half CLOSES (driven); both device affordances absent**

`me sysw wipe --out j4.bin` — **executed**: 65,536 bytes, `--fill ones` prints
the erased-state caveat, default random. A random-fill region has no magic →
`XIPReader.Probe()` false → boot is silent and the carousel entry reports "No
payload found" (probe-false path is covered by
`TestSyswLoadFlowIsSilentWithoutAPayload`). So the operator CAN be rid of a
payload — **only at a host, with picotool**.

The spec promises two on-device affordances; neither exists:

- **F1 "offers erase (§5.5)" / §5.3.2 "paired with an operator-initiated
  *erase this region*… a menu item the operator chooses."** No erase menu item
  exists anywhere in `gui/` (grep: zero hits for erase/overwrite outside
  comments). The F1 warning is dismiss-only — precisely the "warning with
  nothing to do besides be dismissed" §5.3.2 says the erase exists to avoid.
- **The post-engrave overwrite reminder** (§5.5 first line; test 13). Absent.
  Structurally *cannot* be built as the code stands: `take()` returns a bare
  string/mnemonic and no provenance travels with it, so no engrave flow can
  know its input was payload-sourced. This is the same missing plumbing that
  blocks F3 and §7.1.1 (see J-G).

Missing from: **PLAN** (no stage owns either; test 13 is marked
`Where::Device` in coverage.rs and no device stage picks it up).

### J-E. Load a payload, then want a different one — **CLOSES (driven)**

Scratch test: loaded `j1.bin`, then a second region (`j5.bin`, different
content) through `syswLoadFlow` again. Watched: second `Payload Digest` screen
appeared with the second digest (first digest asserted absent), session
identity replaced (asserted inside the flow), fresh comparison demanded —
test 10's re-read half, observed. The "second program does not re-prompt" half
holds structurally: `take()` contains no prompt. Caveat: replacing the region
requires a host + reboot, which is inherent to flash delivery, and the
carousel `loadPayload` entry makes the re-load reachable without a power
cycle.

### J-F. A payload holds a class some program must NOT receive (§3.3.2) — **CLOSES (negatively, as required), but by hard-coding**

Verified across every consumption site: each names exactly one class —
`newInputFlow`→Mnemonic, `engravePassphraseFlow`→Passphrase,
`engraveTextFlow`→FreeText, `bundleFlow`→MDMK, `syswSeedPicker`→Mnemonic — and
classification happens once at load (§3.2.1, transcribed). Engrave Text can
never receive a mnemonic; Backup Wallet can never receive a passphrase. Every
hard-coded class is a `•` in its program's row (checked cell-by-cell).

Two qualifications:

- The refusal is **silent**, not "refused with a named reason" — a payload
  holding only a `pass:` record simply produces no menu in Backup Wallet.
- It closes *by convention*: `admits()` is dead code, so a future call site
  taking a refused class compiles, runs, and fails no test. See Important-3.

### J-G. Verify the plate you just cut (§7 — implied journey the brief did not list) — **DOES NOT EXIST**

Spec §7 is an operator ruling (R0-C3; decision 9 "Verification is never
forced") with a normative menu (§7.2 incl. the `skip` row), normative
selection rules (§7.2.1), and a normative provenance vocabulary (§7.1.1,
"Nothing may render any of these as the bare word 'verified'"). Named tests
2, 3, 17 (and 1, partially discharged by test 16's structural check).

Greps for the menu labels, provenance strings, and any depth/selection
identifier return **nothing** in the Go tree. The existing verify flows
(`singleSigVerifyFlow`, `multisigVerifyFlow`) force a full typed re-entry —
the opposite of §7.2's operator-chosen depth — and record no provenance.

Missing from: **PLAN, wholesale.** No stage mentions §7 at all. This is the
largest F-144-shaped hole: a complete spec section with zero implementation
and zero plan ownership, invisible to stage-vs-stage checking because no stage
claims it. The plan's closing note — "Spec §8.3's 23 named tests are the list;
stages map to them rather than restating them" — is the mechanism of the miss:
**the mapping was never written down in the plan**, and where it was later
written down (`sysw/coverage.rs`), the `Where::Device` rows are precisely the
unimplemented set, and nothing reconciles that file against the device tree.

### Journey summary table

| journey | verdict | hole lives in |
| --- | --- | --- |
| J-A plaintext → consume | **closes** (driven) | — (delivery step undocumented, Minor-7) |
| J-B sealed, word passphrase | closes (host driven, device read) | §8c confirmation: PLAN→CODE |
| J-B′ sealed, ASCII passphrase | **dies** at device entry (known) | §8a keyboard: PLAN; bounds: CODE |
| J-C NFC → program | partial: pre-existing paths only | PLAN (consuming half never sequenced) |
| J-D erase/overwrite | closes host-side (driven); device affordances absent | PLAN |
| J-E second payload | **closes** (driven) | — |
| J-F forbidden class | **closes** (negatively) | enforcement by convention: CODE quality |
| J-G verify the plate (§7) | **does not exist** | PLAN, wholesale |

---

## QUESTION 1 — spec ↔ implementation, both directions

### The five §12 rules

| rule | verdict |
| --- | --- |
| `[cliff]` §12.1 | **Matches, both sides.** Rust `cliff_above` (mod.rs:36) and Go `CliffAbove` (cliff.go) transcribe "≥5 tokens, all wordlist"; both pin the degenerate-abandon and correct-horse-is-2-of-4 cases in tests. CLI computes it over the normalised string (`report_strength`, main.rs:1011). Device computes it over the joined entered words (sysw_load.go:140) — word keyboard guarantees wordlist tokens, so count is the only live variable there. |
| `[compared]` §12.2 | **Matches.** Both routes decided in the one place (`syswLoadFlow`): any successful open → `compared = true` (D1, sysw_load.go:118-120); digest confirm → `compared = true` (:126-138). `take` refuses while false (sysw_session.go:72). I drove the confirm and decline branches. |
| `[identity]` §12.3 | **Matches, both sides.** `MNEMSYSW/id/v1 ‖ 0x00 ‖ region[..total_len]`, full 32 bytes (identity.rs, identity.go); the load flow bounds by `h.TotalLen()` before hashing; `me sysw show` refuses to print an identity for a truncated file (executed). Distinct identities observed in J-E. |
| `[digest-shown]` §12.4 | **Matches.** Device shows iff `PubLen > 0`, sealed or not (sysw_load.go:126). `me` prints `digest: none — this payload has no public section` at `pub_len == 0` (executed) and guards the truncated-slice case. Label is `MNEMSYSW/pub/v1` both sides; construction otherwise EPD§6.6's, `sealed` and count bound in. |
| `[passphrase-bounds]` §12.5 | **HALF-MET.** Checksum-never-required: implemented (`checksumGate: false`). Word count 2..24: enforced in `generate` (both ends tested; `--passphrase-words 25` exits 2, executed). **The 215-byte cap and the 0x20–0x7E range are enforced NOWHERE** — `PASSPHRASE_MAX`/`PassphraseMax` are declared on both sides and never referenced again (grep: zero non-definition uses). See Important-4. |

### §3.3.2 admission table

Transcribed **cell-for-cell correctly** into `admitted` (sysw_admit.go:30-39;
checked against the spec table row by row). But:

- `admits()` has **no production caller** — admission is enforced by each
  consumption site hard-coding its one class (J-F). §3.3.2a's "One function,
  every path, no exceptions" is false of the implementation.
- **Reachability of admitted cells** (the workflow direction):
  Mnemonic — all 6 admitted programs reachable. Passphrase — **BIP-39 Password
  only**; the four seam programs' optional-passphrase step is `passphraseFlow`
  (gui.go:654), which never offers the payload, so the spec's own recorded
  reason for those cells ("matching an existing parameter") is unserved.
  Codex32Secret — **reachable nowhere** (no `take(ClassCodex32Secret)` exists;
  see the spec inconsistency below). MDMK — Bundle only; the Single-Sig and
  Multisig `suppliedMd1` cells are unwired. Descriptor — unpackable (known
  Rust `classify` limitation) and unwired. FreeText — reachable. Address —
  correctly nowhere.
- **A spec-internal inconsistency the table exposes:** §3.1's NORMATIVE seam
  signature returns `bip39.Mnemonic`, which *cannot carry* the
  `ClassCodex32Secret` the same spec's §3.3.2 admits to all four seam programs
  (and to Backup Wallet, whose typed menu does accept M*1 strings —
  gui.go:2442). The spec promised cells its own seam type cannot deliver.
  SPEC defect + PLAN gap (stage 5 never enumerated class×program wiring) +
  CODE incomplete.

### §3.3.3 flags

- **F1**: implemented and driven (J-A). Fires per record class, summarised
  once at load (`syswLoadWarnings`) — the spec puts it "at load", satisfied.
  Its paired erase offer is absent (J-D).
- **F2**: implemented with an extra `sealed &&` conjunct (sysw_admit.go:85).
  Read literally, spec F2 ("passphrase is not `[cliff]`-above") would also
  fire on plaintext payloads, double-flagging beside F1. The code's reading
  matches §3.2.1's own `weak` definition ("sealed, and…"), so **the code is
  right and §3.3.3's F2 row should cite the store's `weak`**. Note the
  compensated inconsistency: `load` stores `weak = !cliffAbove` even for
  plaintext (spec §3.2.1 defines `weak` as sealed-and-below), and `syswFlags`
  re-adds the `sealed` guard. Net behaviour correct; the field's stored value
  deviates from its spec definition (Minor-5).
- **F3**: defined, **never rendered**. §3.2's "Every screen that consumes a
  record names its source — `from payload`, `from tag`, `typed`" is
  unimplemented; no provenance survives `take()`. Root cause shared with
  test 13 and §7.1.1 (J-D, J-G).
- **F4**: defined, **unreachable** — `srcNFC` never constructed in production
  (J-C).

### Things the code does that the spec never sanctioned

- `me sysw pack --region`, `--in FILE`, `--iterations N` — absent from §5.6's
  NORMATIVE surface. All three are right to exist (`--region` is the only
  delivery mechanism the feature has; `--in` is the private channel `seal`
  already established; `--iterations` mirrors `seal`). **The spec is the
  deficient artifact here**: §5.6 should gain the flags, and the spec should
  say how bytes reach `0x10D00000` at all (it never does — see Minor-7).
- `bound()` running the reader over every emitted container, and the
  `EmptyPassphrase` refusal (a `Some("")` host-side would be device-unopenable
  since Go models "none" as `""`) — both unsanctioned and both **good**;
  the second closes a real R0-C4-shaped trap the spec missed.
- `--allow-weak` accepted-and-ignored: matches §13 D3, contradicts §5.6's
  row — see Important-6; the CODE is right.

---

## Findings, ranked

Severity is applied under the operator's §13 lens (workflow blocks gate;
security-only mechanisms warn). Where a pre-§13 reading would raise a rank, I
say so.

### Critical

None open against the shipped mainline. The one candidate — Important-1 —
is Critical under the project's general severity table ("an unmet guarantee",
seeds in cleartext flash unflagged) and Important under §13's explicit ruling
for this feature; it is filed at the operator's ruling with the tension named.

### Important

1. **The §5.3.2 decode gate exists nowhere, and its named test (14) is
   falsely placed.** "A `ClassMDMK` record that does not REASSEMBLE AND DECODE
   is refused" — normative, *not* demoted in §13 — is implemented in neither
   `sysw::pack` (Rust `classify` → `validate_record`, which "Does NOT decode"
   by its own doc; `decode_public_set` is called only by `me hash`,
   main.rs:568) nor the Go load path (zero reassemble/decode references in
   `sysw/` or `gui/sysw_*`). Consequence: 32 bytes of seed entropy wrapped in
   a BCH-valid `md1` classifies non-secret — no F1, no warning — in cleartext
   flash; the exact measured bypass EPD closed with `decodePublicSet`.
   `coverage.rs` places test 14 on `Vector("S-I")`, but S-I is a *valid* md1
   that round-trips; nothing anywhere refuses a non-decodable one — a false
   coverage placement that let the "every named test is placed" gate pass.
   Also note §5.3.2's mechanism text is written against `seal`'s
   `AdmitSection`/pass-3 machinery, which the sysw container never had, and a
   verbatim complete-set decode would refuse the single-card payloads
   `bundleFlow` legitimately seeds with — so the **SPEC also owes a
   sysw-specific restatement** of the rule before code can transcribe it.
   Missing from: SPEC (mechanism mis-aimed) + PLAN (no stage) + CODE.
   Either build the sysw form of the gate or demote it explicitly as §13 D4 —
   as it stands the spec claims a protection the code lacks, the F-123 shape.

2. **§7 (plate verification) has no plan owner and no implementation** — the
   full finding is J-G above. An operator-ruled, normative section (menu,
   selection, provenance; tests 2, 3, 17; decision 9) absent end to end.
   Missing from: PLAN.

3. **The NFC consuming half was never sequenced** — J-C above. §3.1's
   normative `Scanned` option, `scan.go` cases for the two new record forms,
   the §3.3.2a admission call, and F4 are all absent; `admits()` is dead code
   and F3 is never rendered, so "provenance must never be something
   established by reading code" (§3.2) is currently established by nothing at
   all. Missing from: PLAN (spec is right).

4. **`[passphrase-bounds]` is declared and not enforced.** `me sysw pack
   --passphrase-ask` accepts a 500-byte or non-ASCII passphrase today, exits
   0, and seals with it (both constants unused; §6.2.2 says "`me` enforces the
   identical range and cap at creation"). Every such payload is permanently
   unopenable on the device — unenterable even after §8a's keyboard lands.
   This is the R0-C4 shape inside the section named after it. Missing from:
   CODE (stage 2 should have carried it; the PLAN also never mentions it).
   One-file fix in `run_sysw`'s `passphrase_ask` arm plus a `pack` check.

5. **§8c's count confirmation is missing, and Back is indistinguishable from
   `done`** — J-B above. Normative spec text; test 22 unimplemented; the
   truncation trap §8c documents is live, and back-out-at-n>0 runs the KDF
   instead of aborting. Missing from: PLAN (stage 5c under-transcribed §8c) →
   CODE.

6. **Spec defect: §5.6's `--allow-weak` row still says "Refuses with a
   non-zero exit otherwise", contradicting §13 D3 and §6.2.1's blockquote.**
   The code follows D3 (accepted-and-ignored, warn-and-proceed — verified by
   execution, exit 0). A future implementer transcribing §5.6 — the section
   the plan calls "verbatim" — reintroduces the refusal the operator demoted.
   Fix the row (and note `--passphrase-ask`'s row is now the only §5.6 flag
   text that survives D3 unchanged). CODE is right.

7. **Admitted cells unreachable** — the reachability table in Q1 above:
   Codex32Secret consumable nowhere (with the §3.1 seam-signature
   inconsistency), Passphrase unserved in the four seam programs, MDMK
   unserved in Single-Sig/Multisig. Each is a spec-recorded "•" with a
   spec-recorded reason and no path to it. Missing from: PLAN (stage 5 lacked
   a class×program checklist) → CODE; plus the one SPEC inconsistency
   (seam type vs table).

### Minor

1. **The dead offer after a declined comparison** (driven, observed
   "fell-through"): `has()` deliberately ignores `compared`, so every program
   dangles FROM PAYLOAD that silently no-ops into typed entry. The load-time
   notice mitigates; still a menu that teaches operators menus lie. Either
   gate the offer on `compared` or name the reason at selection.
2. **`engravePassphraseFlow` silently truncates** an over-long `pass:` body to
   `passphrase.MaxLen` (100) via `n = copy(secret, raw)`. The plate cap is
   real, so refusal-with-reason is the right shape, not silent truncation of
   a secret. (The fingerprint steps would catch the wrongness downstream —
   if walked.)
3. **The plan's stage-2 green line is false as written**: `me sysw pack
   --no-passphrase 'text:...' | wc -c` prints 67, not 65536 — the command
   needs `--region`, a flag §5.6 doesn't have. Executed both ways. The plan's
   own build-gate rule ("commands get executed") was not applied to it.
4. **Test 21 has nothing to assert against**: the device passphrase path is
   `bip39.Mnemonic` slots + `strings.Join`, not the §6.2.2 215-byte buffer;
   the no-regrow rule was normative and the shape ignores it. §6.2.2a's
   residue acceptance makes this tolerable; the spec/test should be
   reconciled to the implemented shape or the buffer built.
5. **`session.weak` deviates from its §3.2.1 definition** (set `!cliffAbove`
   even for plaintext), compensated by the extra `sealed &&` in `syswFlags`.
   Net behaviour correct; one of the two should move to match the other.
6. **§6.2.3 unimplemented**: `me` never prints that a user-supplied passphrase
   is lowercased/whitespace-collapsed (verified: `report_strength` prints
   strength only), and no device screen exists (moot until §8a).
7. **The delivery step has no named command anywhere** — spec, plan, and the
   device's own error string ("Write one with `me sysw pack --region`") stop
   short of the picotool invocation that actually writes `0x10D00000`. EPD
   solved this with a UF2; sysw emits a raw image and leaves the write to
   folklore. SPEC/docs gap.
8. **The emulator has no sysw source** (`cmd/emu/platform.go:237` returns
   nil), so no flash-payload screen can be qualified in the browser — §8.2's
   own rationale, applied to the region path, un-applied. Spec asked only for
   NFC; arguably a SPEC gap.
9. Cosmetics: `--passphrase-words 25` reports raw `WordCount(25)`;
   `syswOffer` says "ENTER IT" where `syswSeedPicker` says "TYPE IT".

---

## What I verified vs assumed, explicitly

**Executed**: all `me sysw` invocations quoted (pack ×5 modes, show ×2, wipe,
bounds probes); three scratch UI tests driving `syswLoadFlow` (digest match
against `me`'s stderr, F1 screen, decline branch, second-payload replacement)
and consumption through `newInputFlow`/`syswOffer`/`take` to the recovered
mnemonic. **Read but not executed**: the sealed unlock keyboard path (no
typing harness), `XIPReader` (tinygo), the NFC scan plumbing, `engraveObjectFlow`.
**Taken as given**: both suites green with `SYSW_REQUIRE_VECTORS=1`, clippy,
gofmt, tinygo + wasm builds, vector round-trip.

Both working trees were left exactly as found (scratch test deleted; fixtures
live in the session scratchpad).

## The one-paragraph answer to the question behind Question 2

The plan made implementation "transcription rather than judgement" for the
container, the CLI, the port, the plumbing, and the entry seams — and
everything it transcribed got built and works, including the piece (the load
flow) added after F-144. What the plan never contained was the feature's
*second half*: everything after `take()` (provenance, flags at use, the
overwrite reminder, plate verification) and every source other than typing
and flash (the Scanned option, the free-text keyboard). Those live only in
the spec and in `coverage.rs`'s `Where::Device` column, and no process step
reconciles either against the device tree. The cheap structural fix is to
make that reconciliation a command: a test (or plan-gate script) that walks
`COVERAGE`'s Device rows against grep-able witnesses in `gui/`, the exact
trick `assert_every_named_test_is_placed` already plays one level down — it
would have named tests 2, 3, 13, 17 and 22 as missing before this review did.
