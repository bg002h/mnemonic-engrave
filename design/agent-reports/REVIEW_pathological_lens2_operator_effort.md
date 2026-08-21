# Operator-effort review — the pathological vault, wsh vs tr journeys

Lens 2 of the pathological-journey review. Scope: labour, time, physical
artifacts, and the mistakes a tired operator can make that the machine accepts.
Not restore-possibility (lens elsewhere), not document wording (lens elsewhere) —
though two wording defects are cited below because they miscount the labour itself.

Sources: every count below is from a file read or a command run —
`transcript_pathological.txt`, `transcript_tr_pathological.txt`, the two journey
HTMLs, `out/pathological/{backup-strings.txt,card-index.txt,manifest.json,plates/*.png}`,
`out/tr-pathological/{md1-template.txt,md1-keyed.txt}`, `README.md`,
`transcript_pathological.sh`, `capture_tr_pathological.py`, `me --help`,
`me bundle --help`, and FOLLOWUPS entries F-134/F-135. Settled facts (F-60,
F-127, F-134, chunk counts) are used, not re-derived.

---

## 1. THE EFFORT LEDGER

### 1a. wsh journey (`transcript_pathological.txt`, `journey_pathological.html`)

| item | count | ground |
| --- | --- | --- |
| Steel plates, total | **36** (3 md1 + 30 mk1 + 3 ms1) — the tool says **34** (it prints one `ms1` line for three masters); the doc prose says 28 and 25 (stale) | `me bundle` output "backup needs 34 plates (33 public + ms1 on device)"; `card-index.txt` = 30 lines; 3 masters in `inputs-pathological/seeds/`; plate-list in HTML runs 1–33 |
| md1 descriptor chunks | 3 (88+88+85 = 261 chars) | `out/pathological/md1.txt`, `awk length` |
| mk1 key-card chunks | 30 for 11 cards: 8 cards × 3 + 3 cards × 2 | `wc -l card-index.txt` = 30; per-card totals in `manifest.json` `sets[]` |
| Characters on public plates | **2,842** across 33 strings: 19 × 111, 8 × 29, 3 × 80, 2 × 88, 1 × 85 | `awk` over `backup-strings.txt` |
| Host commands (as documented) | 21 shown `$` blocks + an 11-iteration `mk encode` loop + backup-strings assembly ≈ **33 invocations** | `grep -c '^\$'` on extracted HTML; loop at `transcript_pathological.sh:130` |
| Hand-carried values per key card | 4 fields × 11 cards = 44 (xpub 111 chars, fingerprint 8 hex, origin path, policy-id-stub 8 hex); the stub is hand-derived from `md inspect` because `--from-md1` rejects chunked md1 (F-127) | transcript §5–§7; `transcript_pathological.sh:144` |
| NFC presentations (real hardware) | 33 — one per public plate; `me` converts **one string per invocation**, no batch push | checklist "push via NFC & engrave" × 33; `me --help` (`--in <FILE>`, single string) |
| Device typing | 3 seeds × 12 words (autocomplete at 3 letters, ≈ 40–50 keypresses per seed) + passphrase-skip; journey shows master A, prose extends to all three | HTML "On the machine — the seed, typed"; "Plus three seed plates" |
| Plate-handling cycles | 36 × (insert blank, hold-to-start, wait, remove, stack) | checklist; engrave overlay section |
| Machine cut time | **≈ 10–12.5 h**: 22 full-length public plates + 3 seed plates ("the long one", 190 M steps measured) at ~21 min (F-60), 8 near-empty 29-char plates cheaper to cut but identical to handle | F-60 settled; step counts in HTML overlay captions |
| Artifacts that are NOT plates | 1 memorized word (`opensessame` preimage — **0 plates carry it**, F-132); `backup-strings.txt`, `manifest.json`, `card-index.txt` on the host | HTML; README F-132 row |
| Restore labour (full) | regroup 33 unlabeled plates into 12 sets by the in-string 7-char set-id prefix (no engraved fingerprint/path/chunk label — verified on `plate-4.png`, `plate-6.png`); re-enter 2,842 chars by hand (device has no reader: "No plate read-back"); re-type 36 seed words; recall the preimage from memory | plate PNGs; tr HTML "No plate read-back"; F-132 |
| Restore labour (cheapest tier, tier 4) | 3 md1 plates + one 3-chunk mk1 card + master-C seed ≈ 6 plates ≈ 590 chars + 12 words + a 365-day wait | card-index rows for 28645006; F-133 |

Flag-dependence: this 34/36 count is the `md encode --force-chunked --path bip48`
+ `me bundle` route. F-134 (settled): the same wallet class spans **26 → 58 plates**
depending on `--md1-form` (`mnemonic bundle` defaults to the 58-plate policy
form), and nothing shows the operator the trade.

### 1b. tr journey (`transcript_tr_pathological.txt`, `tr-pathological-journey.html`)

| item | count | ground |
| --- | --- | --- |
| Steel plates engraved | **4** (keyless template, 4 × 88 = 352 chars) ≈ **1.4 h** cut time | transcript §3; `awk length md1-template.txt`; F-60 |
| NFC chunks gathered by the device | **24** (keyed set, 2,063 chars) — transmitted, displayed, **never engraved** | transcript §6; `awk` over `md1-keyed.txt`; HTML "chunks presented 24" |
| Gather feedback | tally counts **cards, not chunks**: the counter sits at 0 through 23 presentations and jumps to 1 | README seating-walk section (measured) |
| Consent reading | 4 pages; operator compares wallet-id (32 hex) + 4 addresses (62 chars each) ≈ **280 chars of visual diffing** against host output — automated in the walk, by eye on real hardware | HTML §7–§8 |
| Host commands | 8 blocks, but the keyed encode carries **22 pasted arguments** (11 xpubs × 111 chars + 11 fingerprints) on one command line | transcript §6 |
| What the journey does NOT cut | mk1 key cards and ms1 seeds (scoped out: "already shown in the earlier journeys"); a restore-capable tr backup implies the same 30 mk1 + 3 ms1 ⇒ **≈ 37 plates**, one more than wsh | tr HTML "What this journey does NOT show" |
| What the +1 plate buys | the template itself carries all 11 true per-slot origins (`origins:` block, four distinct accounts), so descriptor + seeds restore correctly — the wsh form's card restores **3 of 11** slots (measured) | transcript §7 vs wsh `restore_test_pathological.py` output |

Bottom line: backup labour is dominated by steel, and steel is nearly identical
across contexts (36 vs 37 plates, ~10–13 h machine time). The tr context spends
one extra descriptor plate and in exchange deletes the wsh journey's worst
restore hazard. The tr journey's 24-chunk NFC gather is verification labour, not
backup labour — the plates it proves are the 4 it engraved.

---

## 2. ERROR-PRONE STEPS, RANKED BY COST OF THE MISTAKE

1. **Stopping when the checklist says stop.** `me bundle` exits 0 and prints
   "backup needs 34 plates" — one `ms1` line for **three** masters, and no
   mention of the sha256 preimage that tiers 1–2 need to spend (F-132: 0 of 33
   strings carry it; the manifest sees all three master fingerprints in its own
   `sets[]` and still prints `ms1_required: true` once). A tired operator who
   cuts plates 1–34 has a backup missing 2 of 3 seeds and the hash secret, and
   the tool has *told him it is complete*. Cost: with only master A engraved, no
   tier of the vault is spendable from the backup alone before its timelock —
   and tier 1 (the only all-A tier) still needs the un-backed-up word. The
   machine's only signal points the wrong way. This ranks first because it is
   the one mistake the tooling actively invites.

2. **Deriving the policy-id-stub by hand and picking the field the spec names.**
   F-127 forces the operator off the automatic path; `md inspect` then prints
   two identities, and SPEC_mk_v0_1.md §3.3 names the wrong one (F-128 — the
   measured stub tracks `wallet-descriptor-template-id`, not `wallet-policy-id`).
   `mk encode` accepts any 8 hex characters; `me bundle` validates the cards;
   the engraver cuts them. The mismatch surfaces only at restore, when seating
   refuses the cards **in words but after the steel exists** (the seating walk's
   refusal arm is exactly this shape). Cost: all 30 mk1 plates re-cut, ~10 h and
   the steel — or, discovered during an actual recovery, a stalled restore.
   Backup-time acceptance + restore-time refusal is the worst ordering possible.

3. **Trusting the flattened `--path` origin.** The wsh card records one origin
   for eleven keys living at four accounts; every tool downstream validates,
   derives addresses, exits 0. If the mk1 cards are lost or skipped — plausible,
   since nothing on the descriptor card says they are load-bearing — a restore
   derives the wrong key in **8 of 11 slots** (measured,
   `restore_test_pathological.py`), silently: valid-looking addresses, no funds.
   Cost: recoverable by an expert scanning nearby account indices; potentially
   unrecoverable by the tired operator it happens to. The machine does nothing
   at restore time to compare against the true wallet.

4. **Typing a wrong-but-valid seed on the device.** The device checks only the
   BIP-39 checksum; any valid phrase engraves. The catch that exists — the plate
   title is the derived master fingerprint, comparable against the fingerprints
   on the mk1 checklist — is available but never demanded. Cost: one master
   absent from the backup; discovered at restore.

5. **Engraving the wrong tr card set.** The session holds two valid sets for the
   same wallet (4-chunk keyless, 24-chunk keyed). Everything accepts the keyed
   one as engraving input: cost is 20 extra plates (~7 h) and eleven xpubs on
   steel. The prefixes differ visibly (`md1f8kykps` vs `md1fydqpt`), but nothing
   warns.

6. **The 24-tap gather with a frozen counter.** The tally moves per completed
   card, so it reads 0 for 23 consecutive presentations. Re-presenting is
   harmless (accepted, deduplicated) and abandonment costs only time — but this
   is where a tired operator concludes the hardware is broken. Low cost, high
   frequency.

Not an error risk, recorded for completeness: regrouping shuffled plates at
restore is pure labour — the BCH checksum and set-ids mean the machine accepts
no wrong grouping.

---

## 3. WHERE THE EFFORT IS DISPROPORTIONATE

- **Eight plates carry 29 characters each.** The BIP-48 origin pushes 8 of 11
  key cards from 2 chunks to 3, and the third chunk is a 29-char remainder —
  measured: 8 × 29 = 232 chars, **8 % of the payload on 24 % of the mk1 steel**
  (`awk` over `backup-strings.txt`; `plate-6.png` is mostly blank metal). Each
  still costs a full handle cycle (insert, push, hold, wait, remove). Nothing
  in-format can drop them today; they are the mk1 chunk cap landing badly. Worth
  a follow-up against the codec owner, not this journey.

- **The 30-plate mk1 leg exists to compensate for a 1-plate defect.** The mk1
  cards are non-optional *in the wsh form* because the descriptor card flattens
  origins (restore test: 3 of 11 without them). The tr form carries all eleven
  true origins in its template for one extra descriptor plate. For
  restore-with-seeds, ~10 h of the wsh journey's engraving is insurance against
  information the descriptor card could have carried. (The cards do also buy
  seed-free watch-only reconstruction — that is worth something, but it is worth
  less than 30 plates of it.)

- **The tr keyed gather re-proves what the seating walk already proves.** 24 NFC
  presentations exist to let the device display addresses from an ephemeral
  keyed set, while `capture_seating.py` already demonstrates the device deriving
  the same proof from the *engraved* artifacts (template + mk1 cards, 4 + n
  presentations). Once seating is the documented verification path, the keyed
  set becomes a host-side convenience, not an operator step.

- **33 `me` invocations to push 33 strings.** One string per invocation by
  design (`me --help`); the manifest already knows the full ordered list. A
  `--push`/iterate mode is pure invocation-count relief. Minor but free.

- **Effort that is NOT disproportionate:** the verification steps (`md verify`,
  the bind, the address comparison) are cheap host commands protecting a
  multi-hour irreversible engraving run; the single-char test plates (F-60,
  ~2 s) are the cheapest insurance in the whole flow. Leave them alone.

---

## 4. THE ONE CHANGE

**Sync `mk`'s vendored md-codec to the primary (0.34.0 → 0.42.0) so the
automatic key-card path works on chunked wallets — then let the existing
bundler drive it.** F-127 is a version pin, not a design gap: "version 9" is
the chunked wire form `mk`'s copy predates. Un-breaking `--from-md1` removes,
in one move:

- the hand-derivation of the stub (the `md inspect` read and the 8-hex paste),
- the F-128 trap entirely — the operator never chooses between two identity
  fields the spec and binary disagree on, so ranked error #2 (30 re-cut plates)
  ceases to exist as a category,
- the reason the 11 `mk encode` invocations must be hand-assembled with 44
  pasted fields: with `--from-md1` accepting the chunk set, the per-card inputs
  reduce to xpub + fingerprint + path, and the already-existing one-command
  bundler (`mnemonic bundle --md1-form=template`, F-134's table) becomes usable
  for exactly this wallet class instead of only for unchunked ones,
- the print-one-engrave-another divergence class (F-210/I-1) at its root: every
  stub comes from the same bytes as the card set it indexes.

It removes no verification, changes no wire byte, and cuts no plate differently
— the same 36 plates come out, produced by ~3 commands instead of ~33 with 44
hand-carried values. Safety strictly improves: the largest machine-accepted
backup-time error is deleted rather than caught.

**Runner-up: make `me bundle`'s checklist tell the whole truth** — one `ms1`
line per distinct master fingerprint (it already has all three in `sets[]`) and
a `preimage required to spend tiers 1–2 — NOT on any plate` line whenever the
template contains a hash lock. This attacks ranked error #1, which the winner
does not touch. It loses the "one change" seat on the brief's own criterion:
it removes almost no labour — it correctly *adds* two plates and one line of
reading — whereas the codec sync deletes ~30 commands, 44 transcriptions, and
an entire re-engrave-everything failure mode. It is also three lines of
checklist logic that should simply *also* be done; if the two were mutually
exclusive on safety grounds alone, the runner-up would win. On labour, it is
not close.
