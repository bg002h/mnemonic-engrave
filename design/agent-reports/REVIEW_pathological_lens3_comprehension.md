# Comprehension review — the two pathological journeys (lens 3)

Reviewer lens: does the operator, after following the journey, correctly believe
what their backup protects against, what it does not, and what they must do to
spend. Every claim below was verified by running the cited command against the
binaries the documents themselves name (`md 0.13.0`, `me 0.7.0`), on 2026-08-21,
against the artifacts as they exist today.

Files reviewed:

- wsh: `design/journeys/out/pathological/journey_pathological.html` (rebuilt
  2026-08-21 12:46, same day as this review) and its published PDF
- tr: `design/journeys/out/tr-pathological/tr-pathological-journey.html`
- `design/journeys/README.md`, `build_pdf_pathological.py`,
  `build_pdf_tr_pathological.py`, transcripts, `out/pathological/*` artifacts

Line numbers cite the HTML files; where the same sentence is hardcoded in the
build script or present in the published PDF, those locations are given too, since
the PDF is the deliverable and the build script is where a fix lands.

---

## 1. STILL FALSE OR UNSUPPORTED

### F1 — "Each key splits into 2 chunks, so the eleven cards are 22 strings."

- **Where:** `journey_pathological.html:262` (`build_pdf_pathological.py:309`;
  PDF text line 305 via `pdftotext`).
- **Truth:** 30 mk1 chunks. Measured from
  `out/pathological/card-index.txt`: 8 cards carry 3 chunks, 3 cards (`@0`,
  `@4`, `@8`) carry 2 — `8×3 + 3×2 = 30`. The document contradicts itself 17
  lines earlier: `journey_pathological.html:245` prints
  `note: 11 key cards -> 30 mk1 chunks (BIP-48 origins push most cards to 3)`.
  An operator counting plates against this sentence concludes 8 plates are
  spurious.

### F2 — "The 25 public plates" / "Three descriptor chunks, then eleven key cards at two chunks each."

- **Where:** `journey_pathological.html:395` and `:400` (section headers),
  `:396` (caption sentence); `build_pdf_pathological.py:408,413`; PDF lines
  484, 508.
- **Truth:** 33 public plates — 3 md1 + 30 mk1 — and the plate figures under
  these very headers run to plate 33, with eight cards captioned `chunk 3/3`.
  The headers and caption describe the previous generation of the document
  (before the BIP-48 re-derivation pushed most cards to 3 chunks) and were not
  updated when the artifact was.

### F3 — "so the real total is 28 plates."

- **Where:** `journey_pathological.html:404` (`build_pdf_pathological.py:417`;
  PDF line 549): "Plus three seed plates that are not here. … so the real total
  is **28 plates**."
- **Truth:** 33 public + 3 seed = **36 plates**. 28 is the stale arithmetic
  (22 + 3 + 3). The same document also reproduces `me bundle`'s
  "backup needs 34 plates (33 public + ms1 on device)". Three totals now appear
  in one document — 34, 28, and the correct 36 — and the correct one is the one
  never printed. This is a fresh instance of the F-134 class (plate count wrong)
  in the *current, regenerated* document.

### F4 — "a divergent-origin wallet cannot state its origins in the descriptor card at all — only the flattened form of finding 3 is expressible."

- **Where:** `journey_pathological.html:479-480` (finding 4;
  `build_pdf_pathological.py:501`; PDF line 610).
- **Truth:** False, measured against the same `md 0.13.0` the document ships.
  Encoding the wsh policy with origins embedded in the key placeholders —
  `multi(3,@0/48'/0'/0'/2'/<0;1>/*,…)`, exactly the syntax the tr sibling uses —
  exits 0, produces a 4-chunk set, and `md inspect` shows all four distinct
  account origins. Critically, the origin-carrying card has the **same**
  `wallet-descriptor-template-id` (`5b48af35d4321a3ac18b43045e2523cc`) as the
  engraved flattened card, so every existing mk1 card's `policy_id_stubs:
  5b48af35` still matches; and `me bundle` accepts the origin-carrying set with
  the journey's own 30 mk1 chunks (exit 0, 35 wallet plates). The finding's
  premises about `--key`/`--fingerprint` flags are true; the conclusion is not.
  The whole 8-of-11-wrong-restore story the document tells is an avoidable
  property of the card it chose, one extra chunk away from not existing — and
  `design/FOLLOWUPS.md` (F-219 entry, line ~8485) already lists "the CLI cannot
  express per-key origins" as one of three disproved absence claims, yet the
  document rebuilt today still asserts it.

### F5 — tr: "--path would flatten eleven distinct origins onto one, which over eleven different keys is a card md encode now refuses outright."

- **Where:** `tr-pathological-journey.html:100-103`
  (`build_pdf_tr_pathological.py:155`; PDF line 38).
- **Truth:** The refusal exists only for the **keyed** set. Measured: the keyed
  encode with `--path bip48` exits 1 with "@0 and @1 declare the same key origin
  ([73c5da0a/48'/0'/0'/2']) but different xpubs; one origin identifies exactly
  one key, so this card describes a wallet that cannot exist." But the
  **keyless template** — the engraved artifact this sentence captions — flattens
  **silently**: `md encode --path bip48 <tr-policy> --force-chunked` exits 0,
  collapses all eleven origins to `m/48'/0'/0'/2'`, and the flattened card
  carries the **same** template-id (`44ad26a19b53048b6ff8957359a30c31`) as the
  true engraved card, so the document's own §4 bind check would pass against it.
  The operator is told a guard protects the card that is in fact unguarded.

### F6 — tr: "The origins are not visible on read-back (F-219) — the card carries them, and neither `decode` nor `inspect` will show them to an operator."

- **Where:** `tr-pathological-journey.html:239`
  (`build_pdf_tr_pathological.py:259`; PDF line 193).
- **Truth:** Both now show them, and the document's **own §4 output** already
  contradicts this sentence: the `md inspect` block on the same page prints an
  `origins:` list for both card sets (`@0: [73c5da0a/48'/0'/0'/2']` …).
  Measured: `md inspect` prints per-slot origins in text (F-219 item 1, DONE
  2026-08-21, `descriptor-mnemonic` 5ca5ceec), and `md decode` of the template
  set prints `note: key origins carried by this card (not shown in the
  template):` followed by all eleven, on stderr (F-219 item 2, PARTLY DONE,
  4fe5c2db). The residual truth — decode's *stdout template text* is lossy and
  not a re-encode fixpoint — is stated correctly earlier in §3; this closing
  bullet overstates it into something the page above it disproves.

### F7 — tr: the document's own title names the wrong journey.

- **Where:** `tr-pathological-journey.html:1`:
  `<title>SeedHammer II — the Wallet Policy journey</title>`
  (hardcoded at `build_pdf_tr_pathological.py:266`).
- **Truth:** This is the taproot pathological journey; "Wallet Policy journey"
  is the sibling document. Minor, but it is the first string a browser tab, a
  bookmark, or a printed header shows.

### F8 — tr: "Two `sha256` hashlocks."

- **Where:** `tr-pathological-journey.html:60`
  (`build_pdf_tr_pathological.py:122`).
- **Truth:** Two hashlock *sites*, one hash, **one preimage** — both tiers
  commit to the same `a84dce40…` digest, as the wsh document says plainly
  ("shared by tiers 1 and 2") and as the tr document's own F-132 bullet implies
  ("the 32-byte preimage", singular). As written it invites the belief that two
  secrets exist and must be kept.

### F9 — wsh: "Reusing a *hash* across tiers is fine — it is not a key."

- **Where:** `journey_pathological.html:73-75`.
- **Unsupported.** Two ways: (a) the first on-chain spend through either
  hash-gated tier publishes the preimage forever, stripping the hash condition
  from **both** tiers for every remaining UTXO — reuse means one spend degrades
  two tiers, which is a property worth a clause, not "fine"; (b) more
  fundamentally, the hash gates only tiers 1–2 while tiers 3–4 need no word at
  all (see silence S1), so the hashlock contributes nothing to the vault's
  overall theft resistance. "It is not a key" is literally true and materially
  misleading about what the word is: a spending credential whose secrecy is
  one-shot.

Verified true along the way (so reviewer budget was not spent re-deriving them):
`H = sha256(sha256("opensessame")) = a84dce40…` matches the descriptor;
`older(4255898)` = 61594 × 512 s = 365.00 d; `older(65535)` ≈ 455.1 d;
"bip48 and m/48'/0'/0'/2' produce a byte-identical chunk set" — TRUE, measured;
the restore-test set `@1,@2,@3,@5,@6,@7,@9,@10` matches the transcript; the
tier table's key-set arithmetic (F-131's eight minimal sets) is correct.

---

## 2. DANGEROUS SILENCES, RANKED

### S1 — Every tier of this vault is satisfiable by a SINGLE master. Neither document says so.

From the slot→master mapping both documents print (@0–@3 → `73c5da0a` A,
@4–@7 → `b8688df1` B, @8–@10 → `28645006` C):

- Tier 1: `multi(3,@0,@1,@2)` — all three keys are master **A**. A alone + word,
  after height 1,000,000.
- Tier 2: `multi(2,@3(A),@4(B),@5(B))` — master **B alone** satisfies it
  (@4+@5) + word, after 2030-01-01.
- Tier 3: `multi(2,@6(B),@7(B))` — master **B alone**, after ≈455 d. No word.
- Tier 4: `multi(1,@8,@9,@10)` — all master **C**; **any one key of C alone**,
  after ≈365 d. No word.

So the operator's real position: the vault **survives the loss of any two seed
plates** (good, unstated) and **falls to the theft of any one** (catastrophic,
unstated) — a thief of master C waits ~365 days per UTXO and takes everything,
no secret word required; a thief of B waits ~455 days. The documents present
"11 keys across three masters, four tiers" — which reads as multisig-grade theft
resistance — when in master terms this wallet is 1-of-3 with delays. F-131's
correction enumerates the eight key-sets but stops one abstraction level short
of the fact an operator acts on. Cost of the wrong belief: seed plates stored
with multisig-grade casualness (one per co-signer, one in a drawer), and the
entire vault is one plate-theft from gone.

### S2 — The wsh document — the primary journey for this vault, rebuilt today — contains no trace of F-132 or F-133.

`grep -c preimage journey_pathological.html` → 0. `grep -c weakest` → 0. The tr
sibling carries both corrections in "What this journey does NOT show"
(`tr-pathological-journey.html:226-232`); the README table carries them; the
wsh document — the one an operator follows to actually engrave this vault, 36
plates — never tells them that tiers 1–2 cannot be spent from anything engraved,
or that tier 4 unlocks ~90 days before tier 3. The README's stated reason for
not folding corrections in ("kept as published — a journey is a record of a
run", `README.md:136-138`) no longer covers this document: it was regenerated
2026-08-21 with new content (addresses, the restore test). The settled findings
are settled in three places the operator may never read and absent from the one
place they will. **This is F-132/F-133 incompletely fixed**, located precisely:
`build_pdf_pathological.py` has no section corresponding to
`build_pdf_tr_pathological.py`'s "What this journey does NOT show".

### S3 — The tr journey's engraved backup contains no keys and no fingerprints, and no restore is demonstrated or discussed.

The tr journey engraves only the 4-chunk **keyless** template
(`tr-pathological-journey.html:99-103`); the 24-chunk keyed set exists only as
NFC ephemera during the walk, and there are no mk1 key cards in this journey at
all. The keyless template's origins carry **no master fingerprints** (its
`origins:` block reads `@0: m/48'/0'/0'/2'`, bare paths), so the engraved
plates plus the three seed phrases do not say **which master fills which slot**
— each slot has three candidate keys and nothing on any plate disambiguates.
The "What this journey does NOT show" list (`:226-…`) names four honest gaps
but not this one, and its wording — "for tiers 1–2 no spend is possible from
this backup" — quietly implies tiers 3–4 spends *are* possible from "this
backup", which the reader has just been shown is four keyless plates. An
operator who believes those 4 plates + seeds are a backup holds a puzzle with
3^11 assignments and no documented procedure. (The wsh journey, by contrast,
engraves the fingerprints on every mk1 card.)

### S4 — `me`'s checklist accounts for ONE seed plate on a three-master wallet, and the document's correction of it carries the wrong total.

`me bundle` prints "backup needs 34 plates (33 public + ms1 on device)" and
lists exactly one `plate 34/34 ms1 secret` line (and the manifest from the
alternative set likewise emitted `"ms1_required": true`, singular). This wallet
needs **three** ms1 entries, one per master. The wsh document does correct this
in prose ("Plus three seed plates", `journey_pathological.html:404`) — but
inside the sentence whose total is wrong (F3), so the tool says 34, the prose
says 28, and neither is right. An operator who trusts the tool's own checklist
engraves one seed plate; if it is master A's, everything except tier 1 is dead,
and tier 1 also needs the preimage of S2. Which master the single ms1 line
refers to is likewise unstated.

### S5 — Relative timelocks run per-UTXO from confirmation; nothing says what the ~365 d / ~455 d clocks are relative TO.

`older()` matures per coin, from each UTXO's confirmation — not from wallet
creation, not from a death, not from "now". Neither document says this. Both
consequences are operator-relevant: an heir told "the 1-of-3 opens after a
year" measures from the wrong event, and — combined with S1 — the only defense
against a stolen master C is to **move the coins before its clock runs out**,
which resets tier 3/4 timers and is exactly the action the documents never
mention. The absolute locks are also left unanchored: `after(1000000)` is
glossed only as "absolute height" (the table at `journey_pathological.html:56-66`
dates the tier-2 lock, 2030-01-01, but not tier 1's, ≈mid-2027).

### S6 — The two journeys back up two DIFFERENT wallets, and neither says so.

Same policy shape, different script contexts, different keys-to-tiers spend
paths, different addresses, different wallet ids (`f89e23f1…` wsh vs
`590f3abc…` tr). The README calls them "the same four-tier degrading vault" in
two script contexts; nothing anywhere warns that funds sit at one wallet's
addresses and the *other* journey's plates recover none of it. An operator
holding both documents (they are designed to be read as a pair — README:41-45)
could reasonably believe the smaller tr plate-set supersedes the wsh one.

---

## 3. WHERE THE TWO DOCUMENTS DISAGREE

1. **Whether the descriptor card can carry per-key origins.** wsh finding 4:
   "cannot state its origins in the descriptor card at all"
   (`journey_pathological.html:479-480`). The tr document does exactly that,
   for the same eleven origins, in its engraved template (`:100-103`). Measured:
   the wsh form works too (F4). The wsh doc is wrong; the disagreement would
   tell a careful reader so, but nothing in either document points at the other.

2. **What protects against origin flattening.** The wsh journey *teaches*
   `--path bip48` as the required fix for this vault; the tr journey says
   `md encode` "now refuses outright" the flatten. Measured truth (F5): the
   keyed encode refuses, the keyless one flattens silently at exit 0 — so an
   operator applying the wsh document's habit to the tr wallet's template
   produces a same-template-id, wrong-origins card that both documents' checks
   would admit.

3. **How many secrets the hashlock adds.** tr: "Two sha256 hashlocks" (`:60`);
   wsh: one `H`, "shared by tiers 1 and 2" (`:73-75`). One preimage exists. The
   wsh doc is right; the tr doc's opening inflates it.

4. **Whether the operator is told the backup's fatal gaps.** tr restates F-132
   and F-133 in-document (`:226-232`); wsh — same vault, rebuilt the same day —
   contains neither (S2).

5. **What "the backup" physically is.** wsh: 36 plates including xpub cards
   with fingerprints and three seed plates. tr: 4 keyless plates. Each document
   describes its own artifact as *the* engraved backup of the pathological
   vault, and neither situates the other (S3, S6).

---

## 4. THE ONE CHANGE

**Add to the wsh journey (in `build_pdf_pathological.py`, mirroring
`build_pdf_tr_pathological.py`'s "What this journey does NOT show") a closing
section "What this backup will not do", saying:**

> To spend from tiers 1–2 you need the 32-byte preimage
> (`sha256(secret word)`); **no plate carries it** — record it separately or
> those tiers are decoration. The tiers do not degrade in the order they read:
> tier 4 (1-of-3) opens at ~365 days, tier 3 at ~455 — and each `older()` clock
> runs per-coin from confirmation, not from today. And in master terms this
> vault is 1-of-3 with delays: every tier is satisfiable by a single master
> (A: tier 1 + word; B: tiers 2–3; C: tier 4, no word), so it survives losing
> any two seed plates and falls to the theft of any one.

Three sentences; two are already written and verified in the sibling document
and the README, the third is derivable from the slot table both documents print
(S1) and was re-verified here. It converts the operator's most expensive wrong
beliefs — "my engraved set spends every tier", "the strong tiers open first",
"eleven keys means multisig-grade theft resistance" — at the exact place the
operator of *this* vault actually reads, in the document that was regenerated
today and therefore can no longer claim record-of-a-run immunity.

**Runner-up:** fix F4/F5 — tell the wsh operator the lossless origin-carrying
descriptor card exists (one extra chunk, same template-id, same mk1 stubs,
accepted by `me bundle`; all measured), and tell the tr operator the keyless
flatten is unguarded. It loses because its wrong beliefs cost **robustness and
labour, not funds**: the true origins already survive on the wsh journey's mk1
plates, so a mis-believing operator holds a harder restore, not a dead one —
while every belief the winning change corrects is in the
irreversible-loss class (unrecorded preimage, mistimed inheritance, a
single-plate theft surface). It is also the less stable text to write today:
the origin-expressibility surface is mid-flight in the tr/wsh cycle (F-219
Stage 6 owns it), whereas the winning section states facts about the wallet
itself, which no codec change will move.
