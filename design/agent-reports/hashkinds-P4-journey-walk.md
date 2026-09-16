# Journey walk — hashkinds phase 4 (fork branch `hashkinds-p4`)

**NOT GREEN — 0 Critical / 6 Important / 4 Minor / 2 Nit.**

Walked against `/scratch/code/shibboleth/seedhammer`, `git diff main..hashkinds-p4`
(6 commits, `285f7a9` tip). Screen text below is quoted from the production copy
bodies and the layout calls, not paraphrased; every quote is a `file:line`
citation you can `sed -n`. The device was not run — this is a read of the flow,
cross-checked against the branch's own harness tests, which walk the same
sequence (`gui/composer_hashlock_test.go:1702`, `:855`, `:488`).

This is a journey lens, not a correctness lens. Findings are **missing things at
moments**. Three of the six Importants are unimplemented normative spec sections
(§7.5, §8, §13.3) — a correctness pass may reach those by another road; they are
here because each one is a *silence at a step an operator is standing on*.

---

## Journey A — a 40-hex `ripemd160` digest on paper → a wallet committed to it

**In hand:** a scrap of paper with `98a20fc25dbcdf236fb0307e3f82cad47fca2e80`.
The operator knows it is a ripemd160 digest. They do **not** have the preimage on
them; it is elsewhere.

### A1. `Path 1` → `A hash, no keys` → the EXPERIMENTAL confirm

Title `Path 1`, lead `What can spend on this path?`, rows `Keys` /
`A hash, no keys` (`gui/composer_shape.go:336-341`). Then an unskippable
hold-to-confirm, `composerCopyKeylessPath()`. Nothing kind-related. Fine.

### A2. `Which hash?`

Title `Path 1 hash` (`gui/composer_hash.go:499`). With no payload loaded the lead is

> No hash record in the payload. Type a phrase below, or make one with ms hashlock on the host.

Rows (`gui/composer_hash.go:421-465`):

    Type a hashlock phrase
    Type a digest
    No hash lock

`Type a digest` is the row this cycle renamed from `Type 64 hex`
(`gui/composer_hash.go:246`). Correct call — the label no longer promises a
width the next screen may not ask for.

**What else might they do?** Nothing harmful here. The three rows are
unambiguous. → *not our concern.*

### A3. The §8i rule modal

Fires on every row except `No hash lock` (`gui/composer_hash.go:516-519`), body
`composerCopyHashRule()` (`gui/composer_copy.go:204`):

> The preimage must be a 32-byte value. A passphrase must be hashed to 32 bytes first, then hashed again. A hash of the passphrase itself can never be spent.

Correctly kind-silent — the kind does not exist yet. Good.

**But note what the operator is now holding in their head:** *"32 bytes."* They
are carrying a **20-byte digest**. The word "digest" was on the row they just
left and does not appear again until the consent screen. This is the seed of
**I-3** below.

### A4. The kind screen — `Hash function`

`composerHashKindPick` (`gui/composer_hash.go:122`), title `Hash function`, lead
(`gui/composer_copy.go:256`):

> All four take a 32-byte preimage. sha256 is the usual choice.

Rows, `fmt.Sprintf("%-10s %d hex", ...)` (`gui/composer_hash.go:101`):

    sha256     64 hex
    hash256    64 hex
    ripemd160  40 hex
    hash160    40 hex

The lead is doing real work: it pre-empts the exact misreading ("40 hex means a
40-character secret") that the §8i modal one screen earlier could have induced.
`TestComposerHashKindScreenDrawsAllFourRows` gates that all four rows fit on
page 1. Good design, well gated.

**What else might they do?** Tap one row below the one they wanted. **The
one-row overshoot from every row lands on a same-width sibling**: sha256→hash256
(both 64 hex), ripemd160→hash160 (both 40 hex). The next screen cannot
distinguish the two. See **I-2**.

### A5. The pad

`composerHexEntry` (`gui/composer_hash.go:155`). Title `Hash lock`
(`:235`). Live readout of the fragment. Count line (`:227`):

    0 of 40 hex

**Nothing on this screen names the kind.** Not the title, not the lead (there is
none), not the counter. The two facts drawn are "Hash lock" and a width, and the
width does not determine the kind. This is the longest-dwell screen in the whole
route — 40 keystrokes, minutes of transcription — and it is the one screen in a
cycle whose own commit is titled *"the device names the kind on every screen that
shows a digest"* that shows a digest with no kind on it. **I-2.**

**What else might they do?**

- *Step back one screen to check which kind is selected.* `composerHexEntry`
  returns `nil,false` → `continue` → the kind screen reopens on the chosen kind
  (`gui/composer_hash.go:591-593`). The kind is confirmed — **and the typed hex
  is gone**, silently, with a fresh `NewKeyboard` on re-entry. The only way to
  verify the axis the pad does not show is to destroy the work. **M-3**, and the
  reopened screen draws only the chosen row and below (**M-2**).
- *Type the 32-byte preimage instead of the digest*, having been told "32 bytes"
  twice and never told on this screen that it wants a digest. 64 characters go
  in; `kbd.Fragment` is clamped to 40 every frame (`:163-165`); the readout stops
  growing at 40, the counter reads `40 of 40 hex`, and **the checkmark is lit**.
  OK commits a ripemd160 lock over 20 bytes of their preimage's hex. **I-3.**

### A6. Assignment — and no confirm

`st.list.Paths[idx].Hash = d; return true` (`gui/composer_hash.go:595-596`).
**The typed-digest arm has no confirm screen and no reconcile screen.** The
phrase route gets both; a payload preimage record gets a confirm; this gets
neither. The path-list row that follows reads (`gui/composer_state.go:413-435`):

    Path 1: hash only

No digest, no kind.

### A7. No 20-byte warning

Spec §8 — *"Fires for ripemd160/hash160 when the digest was **not**
device-derived: payload-supplied and typed digests warn"* — is **implemented
nowhere in this branch.** No `composerCopy*` body exists for it
(`gui/composer_copy.go` declares 78 bodies; `composer_copy_test.go:32` tables all
of them; none is this), and no call site fires one. A typed 40-hex digest reaches
the plates in total silence. **I-6.**

### A8. Consent — where the kind finally reappears

`composerBranchLines` (`gui/composer_consent.go:100-101`), read out of the
**decoded** shape:

      hash ripemd160 98a20fc2..7fca2e80

and the §8i consent body (`gui/composer_consent.go:224`,
`composer_copy.go:220`):

> The hash must be ripemd160 of a 32-byte value. A passphrase must be hashed to 32 bytes first, then hashed again. A hash of the passphrase itself can never be spent.

This is the whole route's only read-back of the kind, and it is a real one (it
comes back through `md.Compose` → decode, and `composerSelfCheck` compares
through `HashLock.Equal`, `gui/composer_selfcheck.go:144`). It is a good catch
point — but it is the **only** one, and it arrives after the digest has been
typed, after the stub screen and after seating.

---

## Journey B — Back at every screen

Legs read off `gui/composer_hash.go:562-597` and `gui/composer_hashlock.go:58-109`,
and confirmed against `composer_hashlock_test.go:488` / `:855`.

| Back pressed at | lands on | kept | dropped |
| --- | --- | --- | --- |
| the pad | the kind screen, **kind still selected** | the path, the kind | **the typed hex, silently** (M-3) |
| the kind screen | `Which hash?` | the path, nothing assigned | the kind (re-seeded to sha256 on re-entry, `:563`/`:584`) |
| the phrase screen | the kind screen, kind still selected | the path, the kind | **the phrase** — spec §7.1 states this; gated at `:537` |
| the method pick | the phrase screen | **the phrase** (`hashlockPhraseFlow(ctx, th, phrase)`, `:61`) | nothing |
| the confirm modal | the method pick | the phrase; nothing assigned | the derivation (re-paid, ~10 s) |
| `Which hash?` (at creation) | the path is **deleted** | — | the path |

Every leg matches SPEC §7.1's normative table. The one thing the table does not
cover, and the device does not signal, is **what the material costs**: the phrase
arm's loss is documented in the spec ("must re-type the dropped phrase and pay
the KDF again"); the hex arm's identical loss is documented nowhere and warned
nowhere. On the device both look like the same button.

The reopened kind screen is drawn with `start := sel`
(`gui/composer_paged.go:313`), so re-entering on `ripemd160` (index 2) draws two
rows and on `hash160` (index 3) draws **one** — under a lead that reads *"All
four take a 32-byte preimage."* The page arrow is offered (`:392`) and wraps to
all four with the cursor intact, so it is recoverable in one press. **M-2.**

No Back leg exists from the reconcile screen — it is a `showError`, drawn after
the hash is already assigned and the material already held
(`gui/composer_hashlock.go:103`). Correct: there is nothing to undo there. But
see Journey C for what the operator is supposed to *do* at that screen.

---

## Journey C — the RECONCILE screen on a `hash256` wallet

**In hand:** a phrase they typed, a method they chose, and this screen
(`gui/composer_copy.go:743`, drawn at `gui/composer_hashlock.go:103`):

    hash  hash256 b867db87..edbc96cb
    method: sha256   chars: 24
    Before you cut plates, run ms hashlock --kind hash256 with this phrase and
    method on the host and check the digest matches. If they differ, do not fund
    this wallet: build it again.

**Does what it tells them to run reproduce what they are looking at?**

**The kind axis: YES, and this is the cycle's headline fix landing correctly.**
`ms hashlock --kind` exists, takes exactly the four tokens `sha256 | hash256 |
ripemd160 | hash160`, case-rejected
(`mnemonic-secret/crates/ms-cli/src/cmd/hashlock.rs:131-141`), and prints
`digest: <64 hex>` on the card plus `hash:hash256:<hex>` on stdout (`:451`,
`:377`). The device's `first8..last8` compares cleanly against it. §13.2 is
closed on the axis it was filed for.

**The method axis: NO.** The sentence names the flag for the kind and **not** for
the method. On the host, `--method` omitted defaults to **hardened**:

    let method = args.method.unwrap_or(Method::Hardened);
    — mnemonic-secret/crates/ms-cli/src/cmd/hashlock.rs:220

So an operator on a `method: sha256` wallet who runs the command as the screen
spells it —

    ms hashlock --kind hash256 --hashlock-phrase-stdin < phrase.txt

— gets the **hardened** digest, sees a mismatch, and is told by this very screen:
*"do not fund this wallet: build it again."* They discard a correct wallet and
re-cut five plates at ~21 minutes each.

This is §13.2's own mechanism on the axis §13.2 says caused it. The fold's own
commit message states the principle it then applied to only one of the two axes:
*"Naming it in the body alone is not enough: what the operator types is what
decides which digest comes back."* **I-1.**

Two smaller things at the same step:

- The command as printed is **incomplete but reads complete**: `ms hashlock
  --kind hash256` with no source exits 64 with `no source given; exactly one
  source: --hashlock-phrase TEXT, --hashlock-phrase-stdin, ...`
  (`hashlock.rs:166`, `:128`). That is a loud, safe failure, and the device
  cannot reasonably print the whole invocation. → *documentation only.*
- Spec §12 item 7 — *"A `hash256` wallet survives its own reconciliation screen:
  compose one, follow the screen's instruction literally, and the check passes"* —
  **has no test in this branch.** `grep -rn 'reconcile' gui/*_test.go` returns
  only body-spelling and reachability assertions. The one acceptance item that
  would have executed I-1 has never been run. **N-2.**

The payload-phrase route draws the same screen conditionally
(`gui/composer_hashlock.go:166-169`) and always at `kind = sha256`, where
`--kind sha256` is a no-op and the hardened default usually matches. I-1 bites on
the typed-phrase route with `method: sha256`.

---

## Journey D — a payload whose `hash:` record is tagged `ripemd160:`

**In hand:** an NFC tag holding `hash:ripemd160:98a20fc2...7fca2e80`, packed on
the host.

The record parses correctly and carries its kind
(`sysw/composer_records.go:203-251`). Then:

### D1. `Which hash?`

Band 1 rows are `composerHashRow` (`gui/composer_hash.go:52`):

    hash 1  98a20fc2..7fca2e80

**No kind.** And because the row shows `first8..last8`, a 40-hex ripemd160 record
and a 64-hex sha256 record render to **exactly the same 18-character shape**. A
payload carrying one of each draws two rows that are distinguishable only by
digest characters — the kind, which is what decides whether the operator's
preimage can ever spend the path, is invisible at the moment they pick.

SPEC §7.5 provisions precisely this row. It is titled *"Row geometry"*, its
arbiter is named as **`TestWhichHashRowsDrawOnOneLine`** — which measures
`Which hash?`'s rows and nothing else — and it spends a paragraph measuring
whether a kind token fits there (`rmd160` and `sha256` at 409 px, one line). That
measurement was made for a row this branch never wrote. `composer_hash_test.go:253-259`
still measures the un-kinded forms. **I-4.**

### D2. Assignment — again with no confirm

`case sel < len(rows.digests): st.list.Paths[idx].Hash = rows.digests[sel];
return true` (`gui/composer_hash.go:521-523`). One tap, assigned, no read-back.

### D3. Where the kind IS visible, and where it is not

| screen | shows a digest? | names the kind? |
| --- | --- | --- |
| `Which hash?` band 1 (`hash 1 ..`) | yes | **no** (I-4) |
| `Which hash?` band 2 (`preimage 1 ..`) | yes | **no** (sha256 by construction) |
| `Which hash?` band 3 (`phrase 1 ..`) | yes | **no** (sha256 by construction) |
| the hex pad | yes (live) | **no** (I-2) |
| the path-list row | no | n/a |
| phrase-route confirm | yes | **yes** — `hash sha256 b867db87..edbc96cb` |
| preimage-record confirm | yes | **yes** |
| reconcile | yes | **yes** |
| masked plate-pick lead | yes | **yes** |
| plate locator row | yes | **yes** |
| **§8.3 plate census rows** | yes | **no** (I-5) |
| consent branch line | yes | **yes** |

Five of thirteen digest-bearing surfaces are silent about the kind. The four the
cycle set out to fix are all fixed.

### D4. No 20-byte warning here either

§8 names payload-supplied digests explicitly. Silent. **I-6.**

---

## Journey E — types a phrase, picks `hash160`

### E1. `Which hash?` → §8i modal → kind screen → `hash160  40 hex` → phrase screen

Phrase screen title `Hashlock phrase`, lead (`gui/composer_copy.go:564`):

> This screen does that hashing for you. Use a phrase you have never used anywhere else.

Counter `n/100`. No kind on this screen either — acceptable: no digest is shown,
so §6's both-or-neither rule is not engaged.

### E2. The method pick

`hashlockMethodPick` (`gui/composer_hashlock.go:369-379`), title
`Hashlock method`, lead `Which method?`, rows:

    Hardened (about 10 s)
    SHA-256

**Two screens after a screen that offered `sha256` as a hash *kind*, a screen
offers `SHA-256` as a choice, and neither screen has the other's axis in view.**
The title and lead do say "method", and the spelling differs (`SHA-256` vs
`sha256`), so this is not a §6 violation on the letter. It is the two-axis
collision at its closest approach, and this cycle's own continuity file names
that collision as *"the axis that caused most of this cycle's defects."* The
mis-pick is recoverable — the SHA-256 arm fires a loud brainwallet confirm
(`composer_copy.go:593`) and Back from the confirm returns to the method pick
with the phrase intact — so the guard holds. **M-1.**

### E3. Confirm

    hash  hash160 3c4f19ab..91e0d7c2
    method: sha256   chars: 24
    no hash: record in the payload has this digest
    Write down the phrase, method, hash kind and digest now. This composition
    holds them until it ends. Without them, this path can never be spent.
    One phrase per policy. Never use it as a passphrase or password anywhere else.

    Hold button to confirm.

Both axes named, the write-down list gained the kind, the "Without **both**" slip
was caught. This is the best screen in the diff.

### E4. Reconcile

    hash  hash160 3c4f19ab..91e0d7c2
    method: sha256   chars: 24
    Before you cut plates, run ms hashlock --kind hash160 with this phrase and
    method on the host and check the digest matches. If they differ, do not fund
    this wallet: build it again.

**This is I-1's live instance.** Kind flag named, method flag not, host default
hardened, screen's prescribed remedy is destructive.

### E5. Done → the preimage-plate census

The one inventory of what must be stored apart
(`gui/composer_preimage_plate.go:412-431`), rows from
`composerCopyPreimagePlateRow` (`gui/composer_copy.go:881`) and
`hashlockPlateFormWords` (`gui/composer_preimage_plate.go:217`):

    Plus 1 preimage plate(s), cut first and NOT part of this backup:
    path 1  3c4f19ab..91e0d7c2  phrase, sha256, QR
    This is what this composition will cut. Plates cut in earlier runs are not
    known to this device and are not listed.
    Keep each preimage plate apart from the policy plates and from the others.

On a `hash160` wallet the census prints a bare **`sha256`** — the *method* token —
with no kind anywhere on the row. That is §6's forbidden shape verbatim (*"No
surface prints one bare token that an operator could take for the other"*), on
the one surface §5's own table lists as a `method` site, and SPEC §13.3 requires
it fixed by name: *"The census row distinguishes the kinds. It is the operator's
only inventory of what they must store apart, and today it describes all four
constructions identically."* Unimplemented. **I-5.**

The **plate itself** names the kind (`hash  hash160 3c4f19ab..91e0d7c2`,
`gui/composer_preimage_plate.go:252`), so the census and the steel disagree about
how much they say — the census being the thinner of the two.

---

## Findings

### Critical

None.

### Important

**I-1 — the reconcile screen names `--kind` and not `--method`; the host defaults
to hardened, so a `method: sha256` wallet fails its own check and the screen
tells the operator to discard it.**
*Found at:* Journey C / E4, the reconcile screen, drawn after every phrase-route
HOLD.
*Where:* `gui/composer_copy.go:743-766`; host default at
`mnemonic-secret/crates/ms-cli/src/cmd/hashlock.rs:220`.
*Classification:* **copy change** (name the flag, as the kind's flag is named).
*Why it earns a change:* the wrong outcome is the operator destroying a correct
wallet and re-cutting five plates at ~21 min each, acting on an instruction this
screen gave them. This is §13.2's own mechanism transplanted one axis over, and
the fold's own commit message states the rule it then applied to only one of the
two: *"Naming it in the body alone is not enough: what the operator types is what
decides which digest comes back."* A reviewer could justifiably call this
Critical; I hold it at Important only because the screen does print
`method: sha256` and the sentence does say "and method", so the information is
present where §13.2's was absent. It blocks either way.

**I-2 — the hex pad names no kind, and the two 32-byte kinds and the two 20-byte
kinds are indistinguishable by the width it does print.**
*Found at:* Journey A5, the pad, for the whole 40-to-64-keystroke entry.
*Where:* `gui/composer_hash.go:155-237`; title `"Hash lock"` at `:235`, counter
`"%d of %d hex"` at `:227`.
*Classification:* **default** (draw the kind token; the screen already has a
counter band and a title).
*Why it earns a change:* an off-by-one tap on the kind screen lands on a
same-width sibling (`sha256`→`hash256`, `ripemd160`→`hash160`), the pad cannot
show the difference, and the only way to verify costs the entire entry (M-3). The
cycle's own commit title is *"the device names the kind on every screen that shows
a digest"*; this screen shows a digest being built, for longer than any other, and
does not. §6's both-or-neither rule engages here — the width is a token an
operator will read as the kind, and it is not one.

**I-3 — the pad silently truncates past the kind's width and reports
`N of N hex` with the checkmark lit.**
*Found at:* Journey A5.
*Where:* `gui/composer_hash.go:163-165` (`kbd.Fragment = kbd.Fragment[:want]`),
readout at `:209-213`, counter at `:227`, OK gated on `len(frag) == want` at
`:167`.
*Classification:* **refusal or warning** (stop accepting keystrokes past the
bound, or say the entry is over-length).
*Why it earns a change:* the route in is plausible, not contrived. The operator
has been told *"The preimage must be a 32-byte value"* (§8i modal) and *"All four
take a 32-byte preimage"* (kind lead) and the word **digest** has not appeared
since the row they tapped two screens ago; the pad's title is `Hash lock`. An
operator who types their 64-hex **preimage** into a `ripemd160` pad gets the
first 40 characters accepted, the checkmark lit, and a lock nobody can spend.
The clamp is pre-existing, but its *danger* is new: before this cycle it could
only drop characters past a complete sha256 digest, and it can now reshape one
valid-width entry into another. Catch points exist — the readout stops growing,
and the consent screen's last-8 will not match their paper — and both depend on
attention mid-transcription.

**I-4 — SPEC §7.5's kind-bearing `Which hash?` row is unimplemented; a payload
`ripemd160:` record is drawn identically to a sha256 one and assigned with no
confirm.**
*Found at:* Journey D1-D2, the `Which hash?` picker.
*Where:* `gui/composer_hash.go:52-54` (`composerHashRow`), `:268`
(`composerHashPreimageRow`), `:279` (`composerHashPhraseRow`); assignment with no
confirm at `:521-523`. The gate §7.5 names as its arbiter,
`TestWhichHashRowsDrawOnOneLine` (`gui/composer_hash_test.go:240-268`), still
measures the un-kinded forms.
*Classification:* **default** (the row form gains the kind, re-measured as a whole
per §7.5's stated constraint).
*Why it earns a change:* §7.5 exists solely to budget this row's width for a kind
token, and spends a measurement paragraph on it. The elision makes every kind
render to the same 18-character shape, so the axis that decides spendability is
invisible at the one moment the operator chooses among payload hashes — and band
1 is the only entry route with no confirm screen behind it.

**I-5 — SPEC §13.3's census row is unimplemented; the plate inventory prints a
bare `sha256` method token and no kind.**
*Found at:* Journey E5, the Done-time preimage-plate census.
*Where:* `gui/composer_copy.go:881-883` (`composerCopyPreimagePlateRow`, three
args, unchanged by this diff), `gui/composer_preimage_plate.go:217-227`
(`hashlockPlateFormWords`), rows built at `:413-415`; the not-cut forms at
`composer_copy.go:888` and `:893` carry no kind either.
*Classification:* **default** (the row names the kind, as the plate's own locator
row already does).
*Why it earns a change:* §13 says *"nothing here is optional"* and §13.3 names
this row specifically. §5's table lists "the §8.3 census row" as a **method**
site, and §6 forbids a surface printing one bare token of a colliding pair — this
row prints `sha256` meaning *method* on a wallet whose *kind* is `hash160`. The
plate the row describes names the kind; the inventory that tells the operator
what to store apart does not, so the two surfaces for the same plate disagree
about how much they say.

**I-6 — SPEC §8's 20-byte warning is implemented nowhere.**
*Found at:* Journey A7 (typed digest) and D4 (payload-supplied digest) — the
moments §8 names.
*Where:* absent. No `composerCopy*` body exists for it
(`gui/composer_copy.go`; the table at `gui/composer_copy_test.go:32-241` enumerates
every declared body and the AST gate at `:297` proves the enumeration complete),
and no call site fires one. Confirmed by `git grep -i '20.byte' hashkinds-p4 --
'*.go'`: every hit is an alloc-gate padding comment.
*Classification:* **warning** (§8's own classification).
*Why it earns a change:* §8 is a normative section of a GREEN spec, its
provenance source is named (`composerState.hashlockHeld`, "no new plumbing"), and
phase 4 is the only phase that owns device state. A ripemd160/hash160 digest the
operator did not derive on this device reaches the plates with nothing said. If
the operator has since ruled it out of scope, that ruling is not recorded in the
branch, the continuity file, or FOLLOWUPS.

### Minor

**M-1 — the method pick shows a bare `SHA-256` two screens after the kind screen
offered `sha256`, with no kind in view.**
*Found at:* Journey E2. *Where:* `gui/composer_hashlock.go:369-379`.
*Classification:* **documentation only** — the title and lead say "method", the
spelling differs, and the brainwallet confirm behind the row is loud enough to
catch a mis-pick with the phrase intact. Recorded because this is the two-axis
collision at its closest approach and it is *new*: before this cycle there was no
kind screen upstream of it, so `SHA-256` here could only mean one thing. If ever
cheap, the lead naming the chosen kind would close it.

**M-2 — the kind screen reopened from a Back at the pad draws only the chosen row
and below, under a lead reading "All four take a 32-byte preimage."**
*Found at:* Journey B. *Where:* `gui/composer_paged.go:313` (`start := sel`).
Re-entering on `ripemd160` draws 2 rows; on `hash160`, 1.
*Classification:* **default** (seed `sel` without moving `start`, or clamp `start`
so the page still fills). Recoverable in one press of the page arrow, which is
drawn (`:392`) and wraps with the cursor intact — hence Minor. Note that
`TestComposerHashKindScreenDrawsAllFourRows` only ever opens at index 0, so the
gate the commit message leans on cannot see this case.

**M-3 — Back from the pad silently discards the typed digest.**
*Found at:* Journey A5 / B. *Where:* `gui/composer_hash.go:591-593`; a fresh
`NewKeyboard` at `:157` on re-entry.
*Classification:* **documentation only** on its own; it compounds I-2, because
stepping back is the only way to verify the kind the pad does not show. SPEC
§7.1's "What that does not buy" paragraph spells out the equivalent cost on the
phrase arm and says nothing about this one.

**M-4 — the kind rows align with `%-10s` space padding in a proportional face.**
*Found at:* Journey A4. *Where:* `gui/composer_hash.go:101-103`. The columns will
be visibly ragged; §7.5's own lesson is that character counts are not the
constraint at this face. *Classification:* **not our concern** unless the ragged
column makes the width column hard to scan on the panel — worth one look at a
frame capture, not a change on paper.

### Nit

**N-1 — three doc comments contradict the code this branch just wrote.**
- `gui/composer_hash.go:151-154`: *"IT RETURNS A sha256 LOCK, and the pad's fixed
  64-character bound is what says so: `Which hash?` offers no kind"* — sits
  directly above the kind-parameterised `composerHexEntry`.
- `gui/composer_hashlock.go:256-260`: *"EVERY CALL SITE PASSES md.KindSha256
  TODAY"* — four call sites now pass a chosen kind.
- `cmd/emu/composer_js.go:44-45`: *"which is 64 hex for every kind the composer
  can produce today"* — the composer produces 40-hex kinds as of commit `285f7a9`.

Each is the "comments outlive their conditions" shape; none changes behaviour.

**N-2 — SPEC §12 acceptance item 7 has never been executed.** *"A `hash256`
wallet survives its own reconciliation screen: compose one, follow the screen's
instruction literally, and the check passes."* No test in the branch composes a
hash256 wallet and runs the printed command. It is the one gate that would have
caught I-1, and this repo's own rule is that a plan may not close while one of
its own gates has never been run.

---

## What this walk found that a correctness pass would not

The four screens §13 named are all correctly fixed, and the Back table is walked
end to end by tests and by the emulator. What is missing is not wrong text in a
section — it is **the kind going quiet at the three moments the operator is
actually deciding**: choosing among payload hashes (I-4), typing the digest
(I-2/I-3), and reading the inventory of what to store apart (I-5). All three
surfaces are internally consistent, all three pass their gates, and none of them
says the one word that decides whether the wallet can ever be spent.

And I-1 is the cycle's own Critical, surviving in the axis the cycle was about:
the reconcile screen now names the flag for `kind` and leaves `method` to be
inferred, against a host that silently defaults it.
