# Lens 1 — RESTORABILITY: the two pathological journeys

Reviewer scope: can a user get this wallet back, and what must they be holding.
Grounding: `transcript_pathological.txt`, `transcript_tr_pathological.txt`,
`out/pathological/journey_pathological.html`, `out/tr-pathological/tr-pathological-journey.html`
(text-extracted), `out/pathological/{backup-strings.txt,card-index.txt,manifest.json,plates/plate-4.png}`,
`inputs-pathological/{keys,seeds}`, `restore_test_pathological.py`, `README.md` —
plus read-only runs of `md 0.13.0` and `me 0.7.0` cited inline. Settled facts
(3-of-11, F-219/220/221/132/130) are taken as given, not re-measured.

One structural fact both inventories hang on, from the policies themselves: in
`wsh`, spending **any** tier requires reconstructing the entire witnessScript —
all eleven pubkeys, in slot order — not just the signing tier's keys. In `tr`,
spending any leaf requires the control block, whose merkle path is computed from
the *other* leaves' scripts, so it too needs all eleven pubkeys. "Tier 4 = any 1
of 3 keys" is a signing statement, never a holdings statement.

---

## 1. RESTORE INVENTORY

### 1a. wsh journey (as published: 36 physical plates = 3 md1 + 30 mk1 + 3 ms1 typed on device)

| need | on the plates? | where | must be kept elsewhere |
| --- | --- | --- | --- |
| Policy structure (tiers, thresholds, timelocks, hash `H`) | YES | md1 plates 1–3 (transcript §4 `md inspect`) | — |
| Per-slot derivation paths | **NO — 8/11 wrong** | md1 card declares `m/48'/0'/0'/2'` for all slots (settled; `restore_test_pathological.py`) | the true paths *are* on the mk1 plates — restorer must know mk overrides md (F-129: precedence unpinned, README.md:—"Findings these runs produced") |
| The 11 xpubs | YES | mk1 plates 4–33; each card carries xpub + true fingerprint + true path + stub `5b48af35` (transcript §7 `mk decode`) | — |
| Slot assignment (@N → which xpub; `multi()` is positional) | **NO** | not in mk1 wire (no slot field in `mk decode` output), not on md1 (all origins identical), plate order is chunk-set-id-sorted, not slot-sorted (`manifest.json`: plate 4 = @5) | slot map, coordinator export, keyed policy-id, or a known address — else brute force ≈ 11! ≈ 4.0×10⁷ assignments against the chain |
| The 3 master seeds | YES | 3 ms1 plates typed+engraved on device, titled by fingerprint (HTML: "Plus three seed plates that are not here") — but `me` checklist names **one** ms1 plate ("plate 34/34 ms1 secret"; `manifest.json` `ms1_required: true`, boolean) | — (if all three were actually engraved) |
| Hashlock preimage `X` for tiers 1–2 | **NO — 0 strings carry it** (settled F-132) | — | the word `opensessame` **and** the convention preimage = sha256(word), H = sha256(sha256(word)) (HTML "The wallet" section). Neither on any plate or checklist line |
| BIP-39 passphrases (if used) | NO | `ms encode`: "passphrase: not stored in ms1" | separate record (none used here) |

**Seeds needed to sign, per tier** (from `inputs-pathological/keys/*.xpub` tier comments):
T1 (3-of-3, @0–@2): master A only + preimage + block ≥ 1,000,000. T2 (2-of-3,
@3–@5): master B alone suffices (@4+@5), or A+B, + preimage + ≥ 2030-01-01.
T3 (2-of-2, @6–@7): master B only, + 65,535-block relative (~455 d). T4 (1-of-3,
@8–@10): any one derivation of master C, + older(4255898) relative (~365 d).
Eight minimal key-sets total (README F-131 row). Every tier additionally needs
the full 11-xpub, correctly-ordered script reconstruction above.

### 1b. tr journey (as engraved: 4 md1 template plates, nothing else)

| need | on the plates? | where | must be kept elsewhere |
| --- | --- | --- | --- |
| Policy structure + NUMS internal key + `H` + timelocks | YES | 4-chunk keyless template set 0x3d896 (transcript §3, §7 inspect) | — |
| Per-slot derivation paths | **YES — all 11 true** (settled) | template card origins @0–@10 | — |
| Per-slot fingerprints (which master serves @N) | **NO** | keyless card origins are `m/48'/…` form — path only; `[fp/path]` per slot exists only in the *keyed* 24-chunk set the device gathers, which is **not engraved** (`out/tr-pathological/md1-keyed.txt`) | slot→master map, the keyed card set, policy-id `590f3abc…` (device displayed it; `tr.id.txt`), or an address — else brute force ≤ 3¹¹ = 177,147 (1,296 under a distinct-master-per-account guess) |
| The 11 xpubs | **NO** | no mk1 cards exist in this journey; xpubs appear nowhere in the engraved set | must be **derived from the seeds** — so any spend, even tier 4's "any 1 of 3", requires **all three master seeds** present to rebuild the tree and control block |
| The 3 master seeds | **NO** | "No ms1 secret leg… already shown in the earlier journeys" (HTML, last section) | 3 seed plates from another journey's process, or the phrases themselves |
| Hashlock preimage `X` for tiers 1–2 | NO (settled F-132; HTML states it: "for tiers 1–2 no spend is possible from this backup at all") | — | same word + convention as wsh |

The wsh mk1 cards do not transfer: their stub `5b48af35` binds to the wsh
template-id; the tr template-id is `44ad26a1…` (both transcripts' inspect
output). The two vaults share masters but not public-key plates.

---

## 2. GAPS, RANKED BY SILENCE (most silent first)

1. **The hashlock preimage exists nowhere in either backup (settled F-132), and
   no checklist line demands it.** Maximally silent: a card-only restore
   *succeeds* — structure verifies, addresses derive, a watch wallet finds the
   funds — and the failure appears only at witness assembly for tiers 1–2,
   possibly decades after engraving. Partial floor: tiers 3–4 need no preimage,
   so funds are eventually movable after the relative timelocks — the vault
   degrades to its weakest tiers rather than to zero. The journeys *say* this
   (tr HTML "What this journey does NOT show"; README F-132 row) but no tool
   output does, and the operator's working artifact is the checklist.

2. **The tr engraved set cannot spend anything without ALL three seeds, and the
   document implies the opposite.** No xpubs are engraved; rebuilding any leaf's
   control block needs all eleven pubkeys, hence seeds A **and** B **and** C.
   "Tier 4: any 1 of 3" reads as one-seed recovery; the loss of any single
   master's seed bricks *every* tier of the tr backup, where the wsh backup
   (xpubs on plates) survives it for the surviving masters' tiers. Surfaces only
   at spend time. Stated nowhere in the tr HTML; "No ms1 secret leg.
   Taproot-independent" actively suggests seed handling is unchanged.

3. **Slot assignment is held nowhere, in either journey.** No plate, wire field
   or document records which xpub occupies @N, and `multi`/`multi_a` are
   positional. Measured: `mk decode` emits no slot; the wsh md1's origins are
   uniform so they cannot discriminate; `me bundle` orders plates by
   chunk-set-id (manifest: plate 4 is @5). Surfaces at restore/watch time as
   "derived wallet, no funds", with no error pointing at ordering. Recoverable
   only by brute force against the chain: ≈ 11! candidates for wsh as published,
   ≤ 3¹¹ for tr (paths prune it). The verifying anchors that exist — the keyed
   policy-id (`590f3abc…`, shown on the tr device at consent), a first address —
   are displayed and then discarded; nothing tells the operator to keep one.

4. **The wsh descriptor card actively lies about 8 of 11 origins (settled), and
   nothing at engrave time can catch it.** Settled F-220/F-221 mechanics: the
   canonical wrapper never demands the origin, and the impossible-origin check
   passes a keyless template silently. The correction data sits on the mk1
   plates in the same vault, but which source wins is unpinned (F-129), so two
   competent restorers can diverge silently — one gets 3/11 keys, the other
   11/11. Half-silent: discoverable before spend, but only by running `md
   inspect` against `mk decode` and knowing to prefer the latter.

5. **`me bundle` undercounts seed plates: one "ms1 secret" line and a boolean
   `ms1_required` for a three-master wallet.** Measured on the published run
   ("plate 34/34 ms1 secret", `manifest.json`) and re-measured on a corrected
   bundle (still "plate 35/35 ms1 secret", singular). An operator following the
   tool's own checklist engraves one seed plate — for an unnamed master — and
   the shortfall surfaces at spend time. The wsh HTML prose corrects it ("Plus
   three seed plates"), the tool does not, and the checklist is the artifact an
   operator executes. Seeded at engrave time, detonates at spend time.

6. **The wsh HTML carries three mutually inconsistent plate totals.** "Each key
   splits into 2 chunks, so the eleven cards are 22 strings", "The 25 public
   plates", "the real total is 28 plates" — against its own transcript blocks:
   30 mk1 chunks, 33 public plates, true total 36 (33 + 3 seed plates). A
   restorer auditing vault completeness against the document cannot get a
   consistent answer (28 vs 34 vs 36). Surfaces at inventory/audit time; loud
   once noticed, quietly wrong until then.

7. **The tr HTML tells the restorer the origins are invisible when they are
   not.** Its closing bullet: "neither `decode` nor `inspect` will show them to
   an operator" — while its own §4 shows `md inspect` printing all eleven
   (settled F-219 scope: it is `decode` *stdout* that renders them away). This
   steers a restorer away from the one command that surfaces the field they
   need most. Cheap to fix, mildly dangerous to leave.

8. **F-130 (settled): a restore reproduces keys and addresses but not the
   descriptor string/checksum.** Bites the keyed-card→coordinator path; loud at
   import time (visible mismatch), costs confidence rather than funds. Lowest
   rank, plus the engrave-time frictions F-127/F-128, which fail with exit
   codes and are therefore the least dangerous class here.

---

## 3. WHAT WOULD MAKE RESTORE EASIER

Four changes, in order of effect. None touches a card format or wire field.

1. **Engrave the wsh descriptor with inline per-slot origins — this works
   today.** Measured this session with the shipped binaries: `md encode
   --force-chunked` (no `--path`) on the wsh template with
   `@N/48'/0'/a'/2'/<0;1>/*` keys exits 0 and yields a 4-chunk set (0x66602)
   whose `inspect` shows all eleven true origins and the **same**
   template-id `5b48af35…` — so the 30 existing mk1 plates' stubs still bind —
   and `me bundle` accepts the combined 34-string set, exit 0. The published
   journey's finding 4 claims "a divergent-origin wallet cannot state its
   origins in the descriptor card at all"; that is true of the *flags* and
   false of the *template syntax*, and the tr journey is the existence proof.
   Cost: one extra plate (4 vs 3) and a re-run of the transcript.

2. **`me bundle`: one `ms1 secret (master <fingerprint>)` checklist line per
   distinct card fingerprint, and a count instead of the boolean.** The tool
   already parses every mk1 card's fingerprint (it prints them per plate); the
   three-master wallet should demand three typed-on-device plates by name.
   Closes gap 5 at the artifact the operator actually follows.

3. **`me bundle`: when the template contains a hash-literal node, print a
   checklist line that the preimage is required to spend and is NOT in the
   backup.** `me` already validates the md1 set; the sha256/hash160 literals
   are in the template it decodes. One conditional line converts the most
   silent failure (gap 1) into an engrave-time instruction ("record the
   preimage separately, on paper, with this vault").

4. **Emit and tell the operator to keep one assignment anchor.** The keyed
   wallet-policy-id already exists (`md inspect` prints it; the device shows it
   at consent; `tr.id.txt` holds it). A checklist line — "write this id (or a
   first receive address) where the plates live" — turns gap 3's chain-scale
   brute force into an offline check over ≤ 3¹¹ candidates. Alternatively,
   for tr, engrave the keyed 24-chunk set the device already gathers instead
   of the keyless 4: deterministic restore, xpubs and fingerprints on metal,
   at the price of 20 plates.

## 4. THE ONE CHANGE

**Change 1: re-engrave the wsh descriptor card with inline per-slot origins.**
Zero tool work — verified working in `md 0.13.0` and accepted by `me 0.7.0`
this session — one extra plate, and it moves the measured card-only restore
from 3/11 correct slots to 11/11 correct paths. It also retires two other gaps
as side effects: the md-vs-mk origin contradiction (F-129's unpinned precedence
becomes moot when both sources agree) and the worst of the assignment brute
force (uniform origins give a restorer 11! candidates; true per-slot paths
prune it to ≤ 3¹¹, a ~225× reduction, before any anchor is consulted). And it
corrects a published false claim that the flattened form was forced.

**Runner-up: change 2 (per-master ms1 checklist lines).** It loses on three
counts. First, mitigation already exists in the document a careful reader has
("Plus three seed plates… engraved from words typed on the machine" is in the
wsh HTML), whereas nothing anywhere states that the wsh card could carry true
origins — the journey states the opposite, so no amount of operator care routes
around gap 4 today. Second, blast radius: the seed undercount harms the
operator who follows the checklist *and* ignores the document; the flattened
card harms every restorer of this wallet, including the careful one, because
the card itself is the misinformation. A missing line prompts a search; a
wrong recorded value terminates it. Third, cost: change 2 needs a code change
in `me` plus a release; change 1 needs a re-encode and one plate. Change 2
should still be done — it is the best *tool* change — but change 1 buys more
restore-safety for strictly less work.
