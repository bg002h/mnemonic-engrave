# R0 round 0 — JOURNEY lens — SPEC_hashlock_H6_preimage_plates.md

**Artifact:** `design/SPEC_hashlock_H6_preimage_plates.md` at engrave master `a0f832d0`
**Trees walked:** fork `fb0dd04` (detached worktree `/scratch/code/shibboleth/.tmp/h6-lens-journey`, removed), mnemonic-secret `504ff46`, engrave master.
**Question asked:** walk every journey the spec defines and find where the SPEC is SILENT — a step whose wrong outcome is worse than telling the operator nothing.
**Not reopened:** decisions 1–9, rulings A1–A8, Group B, journey Q1–Q14.
**Method note:** every claim below was executed or grepped against the real tree; commands and output are quoted. Secret-handling findings are classified non-gating per the 2026-08-27 ruling and are marked as such where they arise.

**Counts: 1 Critical / 7 Important / 7 Minor / 2 Nit.**

---

## Journey 1 — device-derived phrase → HOLD → Done → review → cut → a year later

| # | operator has in hand | spec says the device does | what else they might do | outcome |
| --- | --- | --- | --- | --- |
| 1.1 | a phrase in their head, at `Which hash?` | row 4 `Type a hashlock phrase` → `hashlockPhraseRoute` (`gui/composer_hashlock.go:43-86`) | pick a payload `hash:` row instead, when the payload also holds the preimage | **M-3 / I-3 area**; see M-4 below |
| 1.2 | phrase + method picked | derive (~10 s), confirm modal with `first8..last8`, reconcile screen | — | shipped, unchanged |
| 1.3 | a digest on path *n*; §2.2 now retains phrase+method+preimage keyed by digest | HOLD → `composerHoldHashlockMaterial` | Back out of engrave: §2.2 item 4 keeps it | fine, stated |
| 1.4 | at Done, the `Plates To Cut` census | §5.3: per-plate **form**, **QR**, **decline**, and a held **`show`** toggle, "on the same screen" | try to change the form of the *second* plate | **I-1** — the named screen cannot host any of it |
| 1.5 | form = the default, the **ms1 preimage string** (decision 1: "default plate string / no QR") | §6.1 `MS1 string // Engraved VERBATIM` | — | **C-1** — nothing can produce that string for a device-derived preimage |
| 1.6 | form = phrase, QR on | §8.5 warning, then §6.4's ConstantQR at 53 modules, scale 2 | — | **M-7** |
| 1.7 | plates cut: preimage first, then md1 | §5.4 order; §8.4 abort arm | run aborts *after* every preimage plate, during `bundleEngrave` | **I-7** — §8.4 does not fire and nothing is said |
| 1.8 | a year later, string-form plate only | retype 75 chars into `ms hashlock --in` | — | **M-1** — ungrouped, lowercase, no QR, against a shipped sibling that groups + upper-cases + QRs |
| 1.9 | a year later, phrase-form plate, QR scanned | §8.6's three lines | run `ms hashlock` with the method the plate names | **M-2** — the plate names the algorithm, not the `--method` selector |

---

## Journey 2 — host-derived: `ms hashlock` → `me sysw pack --pack-preimage` → tap → Hashlock plates flow

| # | operator has in hand | spec / code does | what else they might do | outcome |
| --- | --- | --- | --- | --- |
| 2.1 | `X.txt` holding the 75-char ms1 string (`--out` writes `{ms1}\n`, `cmd/hashlock.rs:299-306`) | `me sysw pack --in X.txt --pack-preimage` | forget the flag | §8.1.1 |
| 2.2 | no `hash:` record in the file (`ms hashlock` prints it to **stdout**) | §8.2.3 orphan warning fires | — | **M-6** — the spec's own §12 item 3 acceptance path is a warning path |
| 2.3 | **no** no-seal flag | seals by default; mints a 12-word passphrase | — | verified working, see below |
| 2.4 | `--no-passphrase` | plaintext; secret records move to the public section | — | verified working, see below |
| 2.5 | tap; door | §5.2's 4th route, gated on ≥1 preimage/phrase record | read the door lead | **M-5** — lead says "no keys" |
| 2.6 | Hashlock plates flow, a `phrase:` record picked | §5.2 "list … the operator picks … then it cuts"; §6.3 prints the digest **always** | — | **I-2** — the digest requires a ~10 s derive §5.2 never specifies |
| 2.7 | a wrong-id kind-`0x03` record | §8.1.2 refusal | follow its advice | **I-4** |

### 2.3 / 2.4 verified, not assumed

I tested the hypothesis that a preimage-only **plaintext** payload would have `pub_len == 0`, take neither route to `[compared]` in `gui/sysw_load.go:77-205`, and be **refused at load** — which would have made §12 item 3 impossible. **It is refuted**, and the spec is correct:

```
$ me sysw pack --in secrets_only.txt --no-passphrase --out plain_secret.bin
sealing:  NOT SEALED — you passed --no-passphrase, and this payload HOLDS
      SECRET MATERIAL (record 0 (BIP-39 mnemonic)). It will sit in flash in cleartext.
$ me sysw show plain_secret.bin
sealed:   false
pub_len:  93        <-- NOT zero
ct_len:   0
```

`crates/me-cli/src/sysw/mod.rs:385-392` moves secret records into the cleartext section when there is no passphrase (*"UNSEALED carries secret classes in the cleartext section — decision 6 permits it and F1 flags it at load"*), so `h.PubLen > 0`, the digest screen draws, and `compared` can be set. The **sealed** default reaches `[compared]` by the AEAD open (`gui/sysw_load.go:157`):

```
$ me sysw pack --in secrets_only.txt --out sealed_secret.bin
sealing:  SEALED — this payload holds secret material (record 0 (BIP-39 mnemonic)) …
passphrase — write this down and store it APART from the machine:
    social pizza must hard seven other floor start minute speak pink remain
$ me sysw show sealed_secret.bin
sealed:   true
pub_len:  0
ct_len:   93
```

Both variants load. §3.4's three consequences are correct as written; **no finding here.** Recorded because a reviewer who did not run it would reasonably suspect one.

---

## Journey 3 — a payload `phrase:` record → lazy derive → masked → cut

| # | operator has in hand | spec says | what else | outcome |
| --- | --- | --- | --- | --- |
| 3.1 | a `phrase:` record they had to build by hand (§3.1 gives a Rust *function*, no CLI verb) | hex of `"<method>,<phrase>"`, cut on the **first** comma | hex `hardened, my phrase` — a space after the comma | **I-3** — legal, silently a different phrase |
| 3.2 | at `Which hash?` | row `phrase record 1 (derive to see the digest)`; derive on PICK (§5.1) | — | fine |
| 3.3 | ~10 s later, a digest | entered into `hashlockHeld`, provenance `hashlockFromPayload` | digest matches no `hash:` record / no path | **I-3** — no host check, no device copy |
| 3.4 | at Done | phrase shown as `phrase: <n> characters` + a held `show` toggle | — | **I-1** (non-gating half) |
| 3.5 | method | §5.1: "the RECORD's, never a pick", so J4-1 cannot occur | — | correct, but see **N-2** |

---

## Journey 4 — the mistakes

| mistake | spec's answer | verdict |
| --- | --- | --- |
| wrong method for a payload phrase | §5.1: not offered; the record names it | closed structurally; the *record* being wrong is **I-3** |
| a preimage matching no path | §5.3 item 4 lists it, never cuts it; §8.3's stand-alone notice | closed |
| two hashlocks in one wallet | `hashlockOtherPathLine`; §5.3 item 6 no cap; §10.1's third §8h arm | copy arm is **M-3**-unreachable on the mixed wallet |
| a declined plate | §5.3 item 5, named in the census, does not abort the run | correct; the *control* is **I-1** |
| abort between preimage and md1 plates | §5.4 order + §8.4 arm | one direction only — **I-7** |
| power loss mid-QR | §2.2 item 4: material gone on power loss | not a finding: a device-typed phrase was just typed; a payload phrase survives in flash |
| the 1-in-256 collision under id `entr` | §4.3 narrows admission; §8.1.1 then §8.1.2 | **I-4** — the copy never names the collision and its first instruction is false |
| an ms1 string into free text / passphrase | §9's `IsMS1Shaped` warning, cuts as typed | closed. Verified the payload path also passes `ftStepText` (`gui/freetext_flow.go:1462, 1556`), so a payload-delivered ms1 string warns too |
| a `phrase:` record at the Password program | §4.1: `progPassword` stays `{ClassPassphrase}` | refused structurally, **operator-invisible** — **I-6** |

---

# Findings

### C-1 — Nothing can produce the ms1 preimage string for a device-derived preimage, and the spec never notices

Decision 1 makes the **ms1 plate string the default form** ("default plate string / no QR"; "phrase form only when the phrase is known"). §6.1 declares `MS1 string // The kind-0x03 plate string, for HashlockString. Engraved VERBATIM.` — verbatim from a payload record on the host-derived path. **On the composer-native path there is no string to be verbatim from, and the fork has no encoder that can make one.**

Counterexample, run in the fork worktree at `fb0dd04`:

```
=== RUN   TestJourneyProbeNoPreimageEncoder
    EncodeMS1 -> ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc83qygjzv2p29shrqv35xcur50p7q548au6h32276
      len=75 id="entr" IsPreimage=false
    hand-built  -> ms10hashsqvqqzqsrqszsvpcgpy9qkrqdpc83qygjzv2p29shrqv35xcur50p705wv2knj6j8yz
      len=75 id="hash" IsPreimage=true
--- PASS
```

`codex32.EncodeMS1` (`codex32/msencode.go:17-31`) is the fork's **only** ms1 encoder and is `entr`-only by construction — its own header says so: *"the exact recipe `NewSeed("ms", 0, "entr", 's', [0x00‖entropy]))`"*, prefix `msPrefixEntr`, id the *"FIXED literal `entr`"*. `grep -rn "EncodeMS1\|msPrefixPreimage\|NewSeed(" --include=*.go .` over the whole fork returns five call sites, all seed-shaped, and no preimage encoder anywhere. The second line above shows the string is *buildable* from `NewSeed` — which is exactly the problem: an implementer will build it, ad hoc, at plan time.

Why this is Critical rather than a plan-time detail:

1. **It is normative wire-format work.** A kind-`0x03`/id-`hash` encoder decides what an ms1 preimage plate *is*. The project's standing Rust-primary rule says normative codec behaviour "MUST land **first** in the primary Rust repo, with test vectors, and only then be ported to Go". §3 is titled "Rust first" and covers the two **record** carriers; it says nothing about an encoder, so the spec as written routes a normative encoder into Go first.
2. **The failure is unrecoverable, not a re-cut.** §4.3 is this spec's own rule that the H6 admission path requires the id `hash`. An encoder that emits `entr` (the recipe sitting one file away, which a copy-paste reaches first) produces a plate that §4.3 refuses on the way back in and that `ms_codec::decode` rejects as `TagKindMismatch` (`crates/ms-codec/src/decode.rs:86-96`). The operator's only backup of a spend secret is then a plate no tool will read.
3. **The alternative silently deletes the default.** If the implementer instead offers only the phrase form for device-derived material, the operator who wants the compact, checksummed, BCH-correctable form of their preimage cannot have it, and the plate they get carries the *phrase* — the lower-entropy, guessable secret — in plain text instead.
4. **Nothing catches it.** §1's In-list, §11.4's plate tests, §11.6's gates and §12's acceptance never mention encoding. §12 item 1 exercises the phrase form; item 3 exercises a payload-supplied string. **A device-derived string-form plate is cut by no acceptance item.**

**SUGGESTION.** Decide it in the spec, at spec time, one of two ways.
(a) Add `EncodeMS1Preimage` as **§3's third Rust-first deliverable**: primary in `ms-codec`/`me-cli` with corpus rows pinning id `hash`, kind `0x03`, 75 characters, and `decode(encode(x)) == x`; then the Go port under a provenance pin; then §6.1 says `MS1` is produced by it on the composer-native path and taken verbatim on the payload path. Add a §11.4 mutation (emit id `entr` → the round-trip row fails) and a §12 acceptance item for a device-derived string-form plate read back through `ms hashlock --in`.
(b) Or state plainly in §6.1 and §5.3 item 3 that **the string form is offered only for payload-delivered material**, that a device-derived preimage cuts the phrase form only, and say so in §1 so the operator-facing consequence is a decision rather than an omission.
Either way, name it in §1's In/Out lists so it cannot be discovered by the implementer.

---

### I-1 — The census screen cannot host the per-plate choices §5.3 puts on it

§5.3 item 3: *"Form and QR are chosen PER PLATE, **on the same screen**"*; item 5: a plate may be **declined**; item 7: the phrase is masked *"with a `show` toggle … reveal the characters only while the toggle is held"*. The named screen is `confirmReviewScreen(ctx, th, "Plates To Cut", …)` (`gui/composer_flow.go:389-390`).

`confirmReviewScreen` is read-only and **every button is already bound** (`gui/multisig_build.go:1893-1935`):

```go
// confirmReviewScreen is a paged, read-only confirm screen: Button3 -> true
// (continue), Button1 -> false (back), Button2 pages. Mirrors bundleReviewFlow.
backBtn := &Clickable{Button: Button1}
contBtn := &Clickable{Button: Button3, AltButton: Center}
pageBtn := &Clickable{Button: Button2}
```

There is no row cursor, no selection state, and no per-row action. The spec specifies **four** interactions (pick form, toggle QR, decline, hold to reveal) and **names no control for any of them**. It is also shared: `buildReviewFlow` draws the Policy Review through it, so rewriting it into a picker changes a screen the build path depends on.

The `show` half additionally misdescribes the affordance it cites. §5.3 item 7 calls it *"the affordance the operator already met on the phrase keyboard, which `NewPassphraseKeyboard` carries (`gui/passphrase_keyboard.go:80`)"* and says it reveals *"only while the toggle is held"*. The shipped affordance is a **key on a keyboard grid** and it **latches**:

```go
{label: "show", action: ppReveal}, // gui/passphrase_keyboard.go:141
case ppReveal:
    k.revealed = !k.revealed        // gui/passphrase_keyboard.go:279-280
```

A keyboard key cannot be placed on a screen with no key grid, and "held" is not what `ppReveal` does.

**Severity split, deliberately.** The masking half is secret-handling and **does not gate** (2026-08-27 ruling) — log it. The **form / QR / decline** half is not secret-handling: it decides what goes on steel, and it is unimplementable as specified. That half gates.

**SUGGESTION.** Put the per-plate choices on their own screen and make the census report them. Concretely: a `composerPreimagePlateStep` between the mode pick and the census, one paged pick screen per held digest offering `preimage string` / `phrase + method` / `phrase + method + QR` / `do not cut` (the QR rows shown only when the phrase is held, and the QR row firing §8.5); then the census draws §8.3's rows **read-only**, exactly as `confirmReviewScreen`'s contract allows, showing the choice already made. For the mask, either drop "held" and say the census prints `phrase: <n> characters` with **no** reveal (the plate preview is the screen that shows the phrase, as item 7 itself argues), or specify a Center-press latch and say so.

---

### I-2 — The Hashlock plates flow must derive a `phrase:` record to print its own locator, and §5.2 never says it does

§6.3 makes the digest row unconditional: *"`hash  <first8>..<last8>` (always)"*. §5.2 describes the flow as *"list the payload's preimage and phrase records; the operator picks; per record it offers form and QR (§5.3 item 3); then it cuts."* For a **`phrase:` record the digest is not in the payload** — it is `sha256(PBKDF2-HMAC-SHA256(phrase, "ms-hashlock-v1", 100000, 32))`, and the spec's own measurement is *"about 10 s at the measured 9,715 iterations/s"* (§5.1; `hashlock/hashlock.go:21-27` confirms `Salt = ms-hashlock-v1`, `Iterations = 100000`, `PreimageLen = 32`).

So one of three things happens, and the spec picks none of them:

- the flow **derives eagerly to draw the list** — the exact *"three records would be a 30 s stall before a list could be drawn"* that §5.1 rejected one section earlier, now unmitigated because §5.2 has no `Deriving` countdown specified;
- the flow **derives on pick** — a ~10 s stall with no screen, which reads as a hang (this is the recorded `mt` stdin defect class: a step that is merely silent);
- the flow **does not derive**, and §6.3's "always" row is omitted — producing the worst artifact this stage can cut: a plate carrying a bearer phrase in plain text with **no locator at all** (no `path` on this route, no digest, no `mk1 stub` unless an md1 happens to be in the payload, no payload position without a digest to match).

The third is the funds-relevant one and is what a literal reading of §5.2 produces, because §5.2 lists no derivation step.

**SUGGESTION.** Give §5.2 the same two sentences §5.1 has: the list draws `phrase record <i>` **without** a digest; picking one derives behind the `Deriving` countdown (H2 §4.4) and the result is cached for the rest of the flow; the locator's `hash` row is printed from that result. Add a §11.5 row: the flow's list screen draws its first frame without running the KDF (the timing assertion §11.5 already specifies for `Which hash?`), and a phrase-form plate cut from this flow carries a non-empty `hash` row.

---

### I-3 — §8.2.3's orphan check covers preimages and not `phrase:` records, which is where the mistakes are

§3.3's four warnings name their carriers precisely, and one of them is narrower than the others:

- item 1 — *"when a preimage **or `phrase:` record** is admitted"*
- item 2 — *"no preimage plate **and no `phrase:` record**"*
- item 3 — *"When an **admitted preimage's** digest matches none of the payload's `hash:` records"*
- item 4 — *"holds a preimage **or a `phrase:` record**"*

§8.2.3's body confirms the narrowing: *"record {i} … is **a preimage** whose digest {first8}..{last8} matches no `hash:` record"*.

That is backwards relative to where the errors are. A preimage record is produced by `ms hashlock --out` and is correct by construction. A **`phrase:` record has no producer at all** — §3.1 specifies a Rust *function* `phrase_record(method, phrase)`, and §3.2 adds only `--pack-preimage`; no CLI verb emits one, so the operator hand-builds `phrase:` + hex of `"<method>,<phrase>"`, following the `text:`/`pass:` precedent the `pack` help already sets (*"`text:`/`pass:` bodies are lowercase hex"*, `crates/me-cli/src/main.rs:219`). Two hand-build errors then pass every check §3.1 lists:

1. **A space after the comma.** §3.1 cuts on the **first** `,` "so the phrase may contain commas", and the remainder must be *"non-empty, printable ASCII `0x20..=0x7E`"* — a leading `0x20` is printable, so `hardened, my phrase` is admitted and derives a **different preimage** from `my phrase`. The `now:` idiom §3.1 says it follows is safe from this only because both of *its* fields are digit-constrained (`ParseNowRecord`, `sysw/composer_records.go:135-159`, via `digitsInRange`); the phrase field is free-form, so the idiom's safety does not transfer.
2. **The wrong method selector.** `sha256,<phrase>` when the host derived with `hardened` is a valid record and a valid method; §5.1 correctly removes the *pick*, but nothing checks the *record*.

Both are one PBKDF2 run away from being caught on the host — milliseconds there against 10 s on the device. Uncaught, the operator learns of it only after picking the row, waiting ~10 s, and getting a digest that matches nothing, at a screen with no copy explaining what a non-matching digest means.

**SUGGESTION.** Extend §3.3 item 3 and §8.2.3 to `phrase:` records: derive the record's preimage at pack time and warn when its digest matches no `hash:` record, wording it so the two carriers are distinguishable (*"record {i} is a hashlock phrase whose digest {first8}..{last8} matches no `hash:` record in this payload"*). Add the two §11.1 rows with their mutations (space after the comma → the digest differs and the warning fires; drop the phrase arm → the row prints nothing). Separately, consider whether §3 should give `phrase:` a producer — `ms hashlock … --emit-phrase-record`, or `me sysw record phrase --method …` — since a wire form with no writer is a wire form built by hand every time.

---

### I-4 — §8.1.1's new sentence sends the wrong-id case down a path that dead-ends, and neither refusal names the collision

§4.3 exists for one case, which `codex32/mspayload.go:78-92` states outright: *"A plain BIP-93 33-byte seed that begins 0x03 is indistinguishable from a preimage plate … Roughly 1 in 256 of 33-byte seeds."* Walk that operator through H6 as written:

1. `me sysw pack --in seeds.txt` → §8.1.1 fires, and H6's appended sentence says **"Re-run with `--pack-preimage` if that is what you intend."**
2. They re-run with `--pack-preimage`. §4.3 refuses it — this is the whole point of the id rule — with §8.1.2: **"Re-encode it with `ms hashlock` rather than editing the string."**
3. They run `ms hashlock <string>`. It refuses again, inside a different tool, with a message written for a different audience:

```rust
// crates/ms-codec/src/decode.rs:86-96 — rule 6b
x if (x == TAG_ENTR || x == TAG_HASH) && tag != payload.kind().single_tag() => {
    return Err(Error::TagKindMismatch { … })
}
// error.rs:220 renders:
"tag {:?} does not name the kind the prefix byte 0x{prefix:02x} carries;
 refusing rather than reading one kind as another"
```

Three refusals, and at no point is the operator told the one thing that is actually true and actionable: **their record may be a 33-byte seed backup, not a preimage at all.** Step 1's instruction is affirmatively false for this record — it is H6's own addition, and it is the only reason the operator reaches steps 2 and 3.

(I checked whether step 3 could be *dangerous* rather than merely useless — whether `ms hashlock` would re-tag their seed as a preimage and put it on a `NOT A SEED` plate. It cannot: `SourceKind::Ms1` goes through `ms_codec::decode`, whose rule 6b refuses `entr`-over-preimage before the `Payload::Preimage` arm (`crates/ms-cli/src/cmd/hashlock.rs:230-243`). The remedy is inert, not harmful.)

**SUGGESTION.** Make §8.1.1's final sentence conditional on the id: emit *"Re-run with `--pack-preimage` if that is what you intend"* **only** when the id is `hash`; otherwise emit §8.1.2 directly, from the no-flag path too, so the operator sees one refusal instead of two. And give §8.1.2 the sentence that names the collision, e.g. appending: *"If this string is a 33-byte seed backup that happens to begin 0x03, it is not a preimage and does not belong in this payload — roughly 1 in 256 of them look like this."* Add a §11.1 row asserting a wrong-id `0x03` record produces §8.1.2 **with and without** the flag.

---

### I-5 — `PREIMAGE REQUIRED` also marks the mk1 key cards; §10.3 says otherwise and §11.5's test cannot tell

§10.3 says the marking lands on *"the composer's md1 plates"* and accounts for exactly one exclusion: *"`bundlePlateMark` … already refuses to mark a `cardMS1`, so the seed plates in the same run stay unmarked with no change."* The mechanism is broader than that:

```go
// gui/bundle_flow.go:565-578
// bundlePlateMark decides the title/footer bundleEngrave applies to ONE plate
// in its plan … every kind but cardMS1 gets the caller's marking verbatim
func bundlePlateMark(kind bundleCardKind, title, footer string) (string, string) {
	if kind == cardMS1 { return "", "" }
	return title, footer
}
```

The template form's card list is `cardMD1` **plus** the minted `cardMK1` key cards (`composerMintCards`, `gui/composer_cards.go:69-74`, `kind: cardMK1`). So passing `markTitle = "PREIMAGE REQUIRED"` stamps it on **every mk1 key card** as well — cards whose title band is empty today, and which in a multisig composition are the artifacts that leave for other cosigners.

The behaviour is arguably right (it is `singleSigPlateMark`'s precedent, `gui/singlesig.go:365-374`, where `PASSWORD REQUIRED` reaches mk1 too, and F-132's whole point is that the year-later reader must learn a preimage exists). What is wrong is that the spec **describes an effect it does not have** and then specifies a test that cannot detect the difference. §11.5's row reads: *"`PREIMAGE REQUIRED` marks the md1 plates when any path is hashed and no plate when none is, and never marks a `cardMS1`. MUTATION: mark unconditionally → the unhashed row fails."* Every assertion in that row passes whether or not the key cards are marked. This is the class the severity rule keeps blocking: a test that reports a true PASS about a claim it never checks.

**SUGGESTION.** Decide it and say it. Either (a) §10.3 states that the marking reaches **md1 and mk1** cards, gives the one-sentence reason (a cosigner holding a key card should know the wallet needs a preimage, F-132), and §11.5 asserts the mk1 cards carry it; or (b) §10.3 narrows the mechanism — pass the marking per card kind rather than per run — and §11.5 asserts `cardMK1` is unmarked. Either way the §11.5 row must name the mk1 cards, or it proves nothing about the sentence it is under.

---

### I-6 — The Password-program refusal is correct and completely invisible to the operator

§4.1 is emphatic about the rule and its reason: *"a hashlock phrase is **not** a BIP-39 passphrase and is never admitted at `progPassword`: the terminology ruling L2 exists because interchanging them opens a different wallet, and `progPassword`'s row stays `{ClassPassphrase}`."* The rule is correctly structural — `admitted[progPassword]` is `{ClassPassphrase: true}` (`gui/sysw_admit.go:35`). **The operator is told nothing.**

```go
// gui/sysw_session.go:270-278
func syswOfferAlt(ctx *Context, th *Colors, want sysw.Class, title, lead, alt string) (string, bool) {
	if ctx.sysw == nil || !ctx.sysw.has(want) {
		return "", false          // <-- no screen is drawn
	}
	…
}
```

So the journey is: the operator packs their hashlock phrase, taps, opens the Password program — the program whose name matches the thing they are holding — and gets the ordinary passphrase keyboard, with no offer, no mention that the payload holds a phrase, and no reason given. The obvious next move is the harmful one: re-pack the phrase as a `pass:` record so it "works", which is precisely the substitution §4.1 says opens a different wallet.

Note the asymmetry this leaves. §9 gives the passphrase program a warning for an operator who **types an ms1 string** at it — the rarer mistake, and one with no funds consequence because free text and passphrase both cut as typed. The operator whose payload **literally contains a hashlock phrase**, standing at the Password door, gets nothing. The spec spends a section on the first and no words on the second.

Silence is the current behaviour, so the test is whether the wrong outcome is worse than saying nothing: here saying nothing is what causes the operator to route around the guard, and the guard's own stated stake is a different wallet. It is worse.

**SUGGESTION.** Add one device modal to §8, drawn at `progPassword` when the loaded payload holds a `ClassPhrase` record and no `ClassPassphrase` record. Something in the shape of §9's body, which already measured comfortably (204 drawn / headroom 302):

> This payload holds a HASHLOCK PHRASE, not a BIP-39 passphrase. They are not
> interchangeable — using one as the other opens a different wallet. A hashlock
> phrase is used in the Wallet Policy program.

Add a §11.5 row (a payload holding only a `phrase:` record draws this at `progPassword`; a payload holding a `pass:` record does not) and a §12 acceptance item, since this is the one mistake in the brief's list whose refusal is currently unobservable from outside the code.

---

### I-7 — The new cut order closes one abort window and opens another, and only the closed one has copy

§5.4's reasoning is explicit and correct as far as it goes: *"Ordering removes the window in which the md1 plates exist and the preimage does not; §8.4's abort arm covers the window ordering cannot (a blank runs out mid-plate)."* §8.4's arm *"fires when the run ends before **every accepted preimage plate is cut**"* and says:

> NO PREIMAGE PLATE WAS CUT. The phrase dies with this composition. Do not fund this wallet.

Now walk the other side of the new order. Every preimage plate **is** cut; the run then aborts inside `bundleEngrave` — a blank runs out, a plate is misaligned, the operator backs out of the set-level confirm. §8.4 does not fire, correctly, because a plate *was* cut. The operator is left holding:

- a preimage plate on the bench: bearer material, and §8.3's own census line tells them *"Keep each preimage plate apart from the policy plates and from the others"*;
- **no policy plates at all**, and `bundleEngrave`'s existing set-level copy (`gui/bundle_flow.go:625-631`) telling them a partial bundle cannot be used;
- material still held in `hashlockHeld` (§2.2 item 4: state survives a Back into the loop).

Nothing tells them that a bearer plate now exists outside the set. The natural response to "this bundle is unusable" is to run the composition again — and §5.3 item 6 sets **no cap**, so the second run cuts a **second** preimage plate for the same secret. The operator now has two bearer copies of one spend secret, one of which they have no record of, and the spec's own instruction is to store them apart from each other.

This is a spec silence created by *this* stage: before §5.4 the preimage plate did not exist, and the ordering decision is what puts it on the bench first.

**SUGGESTION.** Give §8.4 a second arm, fired when the run ends after **at least one** preimage plate was cut, prepended to the seed clause the same way. It has to be short — §8.4 records that the combined body already measured 411 drawn / headroom 121 in the longest variant, and that four longer drafts were rejected for costing a line each — so something at or under the arm it replaces, e.g.:

> A PREIMAGE PLATE WAS CUT and no policy plate was. Store or destroy it now; do not leave it with the blanks.

and a §11.5 row with its mutation (fire on the all-cut-and-completed run → the success row fails). §5.3 item 6's "no cap" should also say what happens on a re-run: either the census names plates already cut in this session, or the spec states plainly that it cannot know and the operator must.

---

### M-1 — The default plate is the least legible secret plate the device cuts

§6.1 engraves `MS1` **VERBATIM** and §6.4 gives the string form no QR by design (ruling A2 declines B5). The fork's shipped ms1 secret plate does neither:

```go
// backup/backup.go:158-190
func EngraveSeedString(params engrave.Params, plate SeedString) (engrave.Engraving, error) {
	seed := strings.ToUpper(plate.Seed)      // <-- upper-cased
	…
}
const groupLen = 10                          // <-- grouped in tens
	ngroups := (len(seed) + groupLen - 1) / groupLen
```

— upper-cased, grouped in tens, **and** carrying a QR. `ms hashlock`'s own engraving card prints the string grouped too (`render_grouped`, `crates/ms-cli/src/cmd/hashlock.rs:340`, default group size 5). So the artifact H6 makes the *default* is the only one of the three renderings of the same 75 characters that is a single unbroken lowercase run, and it is the one with no machine-readable copy.

Measured on the tree (`CharsPerLine` at `sh2.Params()`, `constant.Font`, confirming §6.5's own table 19/23/26/31/34/39):

```
rung 6.0mm: chars/line=19 lines/plate=13
raw len=75                                    -> 4 lines
grouped(10) len=82  "ms10hashsq vqqzqsrqsz svpcgpy9qk …"
```

At 19 characters per line a 10-character group plus a space plus the next group is 21 — over the line — so grouping at 6.0 mm costs **8 lines instead of 4**, +16 mm against §6.5's stated 60.0 mm of a 65 mm budget. It does not fit. It fits at 5.0 mm (23 chars/line holds two groups), which the spec's table shows is otherwise unused by this form.

This is not data loss — bech32 carries a checksum and the fork ships BCH correction (`codex32.Correct`), so a mistranscription is refused rather than silently wrong. It is a re-read cost, repeatedly, a year later, from steel.

**SUGGESTION.** State the rendering in §6.1 rather than leaving "VERBATIM" to decide it: either upper-case and group in tens at 5.0 mm, matching `EngraveSeedString` and giving the same reason, or state explicitly that the string form is cut lowercase and ungrouped because it is transcribed once into a tool that checksums it. Add the chosen form to §11.4's goldens. If grouping is taken, §6.5's string row moves from 6.0 mm to 5.0 mm and the table should say so.

---

### M-2 — The plate names the algorithm, not the flag, and §6.5 forecloses adding it

§8.6's method line is the definition, and §3.1 says the split from the wire's selector is deliberate. Measured:

```
method line chars: 73
worst-case QR text bytes: 194
"method: pbkdf2-hmac-sha256 iterations=100000 salt=ms-hashlock-v1 dklen=32"
```

The year-later operator scans the QR, gets that line, and must map it to `ms hashlock --method hardened` — a mapping that appears nowhere on the plate. They are rescued only by `args.method.unwrap_or(Method::Hardened)` (`crates/ms-cli/src/cmd/hashlock.rs:186`), i.e. the default happens to be the one the plate does not name. The `sha256` plate has no such gap: its line is literally `method: sha256`, which is the selector.

What makes this worth raising **now** rather than at plan time is that §6.5 pins it shut: *"The method line is the largest single term and must not grow. At 73 characters it is 2 lines at 3.0 mm; a 79th character would make it 3 and put the worst case 3.0 mm over budget."* Appending the selector costs 32 characters (105 total, measured), so the fix is unavailable after this spec freezes the geometry. The QR itself has room — 194 + 32 = 226 bytes, still under §7.1's ≥231 threshold for v10 — so only the plate text budget is the constraint.

**SUGGESTION.** Put the selector on its own short body row rather than inside the method line — `ms hashlock --method hardened` is 29 characters, one line at 3.0 mm — and either add it to §8.6's text as a fourth labelled line before `phrase:` (the phrase must stay last) or keep it plate-only and out of the QR. If it is declined, say so in §8.6 with the reason, so the next reader does not re-derive that it was foreclosed by §6.5 rather than decided.

---

### M-3 — §10.1's third §8h arm never draws on the ordinary mixed wallet

§10.1 adds a third arm to `composerCopyHashEveryPathFor` (`gui/composer_copy.go:527-532`). Its one call site is guarded:

```go
// gui/composer_shape.go:442-443
if composerEveryPathHashed(st.list) {
    showError(ctx, th, "Spend paths", composerCopyHashEveryPathFor(st))
}
```

`composerEveryPathHashed` is false the moment one path is keyed — which is the ordinary hashlock wallet. This is already on the record in the fork: `gui/composer_hashlock.go:70-79` says §8h *"is guarded by composerEveryPathHashed … which is false the moment ONE path is keyed -- so on the ordinary mixed wallet the line was drawn nowhere at all"*, which is why H5 moved the reconcile line into the phrase route.

So §10.1's new copy — *"This run cuts a preimage plate for each one"* — and its closing claim that *"When some hashed path has no plate in this run, the SHIPPED forms stand unchanged"* both describe a screen the mixed-wallet operator does not see. H6 does not make this worse (the guard is pre-existing), and the census block §8.3 adds does carry the information at Done for every composition. But §10.1 reads as though it fires, and a plan written from it will add an arm nobody reaches.

**SUGGESTION.** One sentence in §10.1 recording the guard and where the load actually falls: the third arm is drawn only on an all-hashed policy; on a mixed policy the §8.3 census block is what tells the operator a preimage plate is being cut. If the arm is meant to reach the mixed wallet, that is a change to the call site's guard and belongs in §10.1 explicitly.

---

### M-4 — Two adjacent rows carrying the same 16 hex characters, with different consequences

§5.1's row order puts the payload's `hash:` digests in band 1 and the payload's preimage records in band 2, both rendered `<first8>..<last8>`. A payload packed at full fidelity — `hash:` from `ms hashlock`'s stdout, the preimage from its `--out` — produces:

```
hash 1      b867db87..edbc96cb
preimage 1  b867db87..edbc96cb
```

Identical digest text, adjacent, differing by one word. Picking band 1 assigns the digest and holds **no** material (§2.3 enters material only when the composer takes a payload *preimage* onto a path), so no preimage plate is offered at Done and §5.3 item 4 will not list it either — the record is in the payload, not in `hashlockHeld`. Picking band 2 cuts a plate. The composer never says which happened.

The safety net is weaker than it looks: the shipped §8h form that would say *"the preimage is not on these plates"* is behind M-3's guard, so on a mixed wallet nothing is drawn.

**SUGGESTION.** When a payload `hash:` digest is matched by a preimage or `phrase:` record in the same payload, either suppress the bare `hash:` row (the preimage row is strictly more capable) or annotate it — `hash 1  b867db87..edbc96cb  (preimage in payload)` — measured against `composerPageLines`' 411 px band as §5.1 already does for its two new rows. §11.5's `Which hash?` by-label test should gain the both-present case.

---

### M-5 — A preimage-only payload reads "no keys" at the door it is offered from

`composerDoorCounts` (`gui/composer_door.go:37-53`) counts `ClassKey`, `ClassMnemonic`/`ClassCodex32Secret` and `ClassUnknown`, and nothing else; `composerDoorLines`' `default` arm is `composerCopyNoKeys()`. A payload holding only a preimage record — §12 acceptance item 3's payload — therefore draws a lead saying there are no keys, above a door that is offering the **Hashlock plates** route §5.2 adds for exactly that payload.

There is precedent for not counting every class (`hash:` and `now:` records do not count today), so this is a copy inconsistency rather than a defect. But §5.2 adds a route whose whole precondition is a class the lead cannot see.

**SUGGESTION.** Add one clause to §5.2: when the payload holds preimage or `phrase:` records, `composerDoorLines` names them (`preimages: N`, or a combined line), so the lead and the routes agree. Cheap, and it is the screen that tells the operator the tap worked.

---

### M-6 — The spec's own simplest host journey is a warning path

§12 acceptance item 3 is *"`ms hashlock --out X.txt`, `me sysw pack --pack-preimage --no-passphrase`, a tap, the Hashlock plates flow, and a cut plate"*. `ms hashlock` writes only the ms1 string to `--out` and prints `hash:<hex>` to **stdout** (`crates/ms-cli/src/cmd/hashlock.rs:295-334`), so that payload holds a preimage and no `hash:` record — which is precisely §8.2.3's orphan condition. The acceptance walk fires a `WARNING —` on the correct path, every time.

The warning's content is accurate and useful (*"Nothing here tells the device which policy it unlocks"*). The problem is that it is written as an exception and is in fact the default for the minimal journey, which is how a warning stops being read.

**SUGGESTION.** Either soften §8.2.3 to a note when the payload holds **no** `hash:` record at all (nothing is inconsistent — the operator simply did not pack one) and keep the `WARNING —` for the genuinely inconsistent case (a payload that holds `hash:` records, none of which match), or add the `hash:` record to §12 item 3's invocation so the acceptance path is the clean one. The first is better: it distinguishes "incomplete" from "contradictory", which is the distinction the operator needs.

---

### M-7 — ConstantQR at 53 modules and scale 2 is a first, and only a phone can check it

§6.4 and §6.5 put the phrase form's QR at **53 modules, scale 2** — 0.6 mm modules, 31.80 mm square. Neither half has precedent:

- `ConstantQR` has only ever been used at scale **3** with a 37-module envelope (`passphraseQRScale = 3`, `passphraseQREnvelope = 37`, `backup/passphrase.go:66-72`);
- 0.6 mm modules exist only on the free-text plate, which uses the **non-constant-time** `engrave.QR` (`backup/freetext.go:168`, `freeTextQRScale = 2`, `backup/fit.go:16-19`: *"0.6mm modules against the 0.9mm every other plate uses"*) and whose QR is not a secret.

§11.3 proves the right things — the alignment table derived from the encoder, constant-time move counts per version, goldens — but all of them are about **bytes and toolpath**, not about whether a phone reads 53 modules at 0.6 mm off engraved steel. §12 item 1 makes the phone scan the acceptance, and §12 itself records that **the SH2 has no camera**, so the device can never check this for itself.

Mitigated, and this is why it is Minor rather than Important: §6.4 stacks the QR **below** the text, and §6.1 engraves the phrase in plain text on the same plate. An unreadable QR costs a retype, not the secret.

**SUGGESTION.** Say the mitigation out loud in §6.4 — the text is authoritative, the QR is a convenience — so a future reader does not treat the QR as the backup. And make §12 item 1's scan a **gate on offering the QR at all**: cut one test plate at the worst case (100-character hardened phrase, v9, scale 2) and scan it before the QR toggle ships. This is the single-character-test-plate pattern at ~2 s a try rather than ~21 min a plate.

---

### N-1 — §8.4's copy must not be reused in the Hashlock plates flow

§8.4's arm says *"The phrase dies with this composition."* §5.2's flow *"builds no composition and reads nothing from `composerState`"*, and the material it cuts lives in the payload, in flash. Both clauses are false there, and the second is false in the dangerous direction — it would tell the operator a secret is gone when it is still in flash. §5.2 specifies no abort behaviour at all.

**SUGGESTION.** One sentence in §5.2: an abort in this flow needs no warning, because the material remains in the payload; §8.4's arm belongs to `composerEngraveStep` alone. Worth writing down precisely because reusing the arm is the obvious implementation.

---

### N-2 — The payload-phrase derive path cannot be `hashlockPhraseRoute`, and the spec never says so

§5.1 requires that a `phrase:` record's method is *"the RECORD's, never a pick"*, and §10.2 requires that the reconcile screen is **not** drawn for payload provenance. `hashlockPhraseRoute` (`gui/composer_hashlock.go:43-86`) does both of the forbidden things: it calls `hashlockMethodPick` and it ends with `showError(… composerCopyHashlockReconcile(…))`. So §5.1 and §10.2 together imply a second, distinct derive path, and no section says it exists or what it contains (phrase screen: no; method pick: no; `Deriving` countdown: yes; confirm modal with `hashlockRelationLine`: unstated).

**SUGGESTION.** Name it in §5.1 — a derive-only path (countdown → digest → confirm), sharing `hashlockDeriveFlow` and `composerCopyHashlockConfirm` with the typed route and skipping the phrase screen, the method pick and the reconcile screen — and let §10.2's provenance rule fall out of *which path ran* rather than out of a runtime check. §11.5's mutation for §10.2 (*"key §10.2 on `phraseDigests` instead of provenance → the payload row draws it"*) then has a mechanism to attach to.

---

## Closing counts

| severity | n | ids |
| --- | --- | --- |
| **Critical** | **1** | C-1 |
| **Important** | **7** | I-1, I-2, I-3, I-4, I-5, I-6, I-7 |
| Minor | 7 | M-1, M-2, M-3, M-4, M-5, M-6, M-7 |
| Nit | 2 | N-1, N-2 |

**Not a finding, verified rather than assumed:** a preimage-only payload loads in **both** container variants (§3.4 is correct — the plaintext path moves secret records into the public section, `crates/me-cli/src/sysw/mod.rs:385-392`, so `pub_len > 0` and the digest route to `[compared]` is open; the sealed path is compared by the AEAD open, `gui/sysw_load.go:157`). §9's warning also covers a **payload-delivered** ms1 string, because the payload path still walks `ftStepText` (`gui/freetext_flow.go:1462`, `:1556`). §5.1's removal of the method pick does close journey mistake J4-1. §6.5's chars-per-line table (19/23/26/31/34/39) reproduces exactly against `backup.CharsPerLine` at `sh2.Params()`, and §8.6's method line is 73 characters with a 194-byte worst-case text, as stated.

**One record H6 falsifies that §0 does not list** (folded into no finding above; noted for the fold): `ms hashlock`'s engraving card tells the operator *"write the method line next to your phrase; **it is on no plate**"* (`crates/ms-cli/src/cmd/hashlock.rs:353`). §6.2 and §8.6 put the method line on the phrase-form plate, so that sentence becomes false. It errs safe — it asks for a copy the operator no longer strictly needs — but §0 lists three shipped records this stage rewrites and this is a fourth, in a repo the spec otherwise treats as primary.
