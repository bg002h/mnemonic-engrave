# PLAN — round-trip journey: the TAPROOT pathological wallet on SeedHammer II

Status: PLAN (no implementation). 2026-08-20.
Subject: `design/journeys/inputs-pathological/wallet-policy-tr.txt` — the taproot
form of the four-tier degrading vault, 11 keys, 3 masters, four timelock kinds,
one sha256 hashlock, every tap leaf an `and_v(v:…)` wrap.
Model journey: the Wallet Policy set (`transcript_walletpolicy.sh` /
`capture_walletpolicy.py` / `cmd/emu/shots_walletpolicy.js` /
`build_pdf_walletpolicy.py`). This plan reuses its mechanisms verbatim wherever
they apply and says so; only what is genuinely new here is new.

---

## 1. WHAT THE JOURNEY PROVES — the round-trip definition, defended

**Chosen definition: two equalities at two different layers, both machine-made,
both able to fail.**

- **E1 — structural (host, Rust only).** The policy text encodes to an md1
  chunk set, and `md decode` of that chunk set is **byte-identical** to the
  committed policy file, exit 0. This proves the bytes survive the card format.
  It is asserted by a `diff` inside the transcript, as a FATAL.
- **E2 — functional, cross-implementation (host Rust vs device Go).** The same
  wallet, as a **keyed** md1 card set, presented to the emulator over NFC; the
  device's consent screen must show the **wallet-policy-id** and the **first two
  receive and first two change addresses** that the host derived from the same
  cards — and the walk THROWS if the screen disagrees. This proves the cards
  *mean* the same wallet to two implementations in two languages, which is the
  only equality that matters to funds.

E1 without E2 is the trap this wallet has already sprung once: the host
round-trip has worked all along while the device could say nothing about where
the wallet pays (F-214). E2 without E1 would prove agreement about cards nobody
can regenerate. Together they are the round trip: **policy → cards → same
policy back, AND cards → device → same identity and addresses the host
computes.**

**Two card sets, one wallet — and the transcript must bind them.** The engraved
artifact stays the compact **keyless template** set (the 3-chunk
`backup-strings-tr.txt` form — that file finally gets a producer and a
consumer); the device leg rides the **keyed full-policy** set, because F-216
means a keyless template gathered with mk1 key cards still shows no addresses —
the keyed card is the only form the device can prove today, and this plan
invents no feature to change that. The bind: `md inspect` of BOTH sets must
report the **same `wallet-descriptor-template-id`** (asserted in the transcript,
FATAL on mismatch). Without that assertion the journey could engrave one wallet
and prove another.

**Deliberately NOT asserted** (expanded in §6): plate→card read-back (no reader
exists — the manifest equality in G7 is the honest proxy); the ms1 secret leg
(taproot-independent, already shown in the wsh journey, and F-130 makes
descriptor-string equality falsely red there); any spend (no signer, no chain —
and for tiers 1–2, no preimage exists in the backup to spend WITH, per F-132).

---

## 2. PRECONDITIONS — what must land first

- **P1 — F-214's fix, covering all four leaf shapes of THIS policy.** The
  device must derive addresses for: `and_v(v:after(1000000), and_v(v:sha256(…),
  multi_a(3,…)))` (height lock + hashlock, **nested** and_v),
  `and_v(v:after(1893456000), and_v(v:sha256(…), multi_a(2,…)))` (time-form
  CLTV + hashlock, nested), `and_v(v:older(65535), multi_a(2,…))` (relative
  blocks), `and_v(v:older(4255898), multi_a(1,…))` (relative 512-s units). A
  fix that lands only the single-depth `and_v(v:older…)` shape of the vendored
  `gap_tr_leaf_and_v` vector is NOT enough — the flagship tiers nest twice and
  mix CLTV forms. **Gate G0 below is the check**, and it runs before any
  emulator work is spent: a fork-side conformance vector for this exact keyed
  policy with Rust's addresses as ground truth. The existing gap vector is
  pinned to fail with "THE GAP IS CLOSED" the moment the emitter grows; the fix
  must convert/retire it, and G0 must be green.
- **P2 — rewrite `wallet-policy-tr.txt` in the F-217-compliant form and
  regenerate `backup-strings-tr.txt` from the transcript.** The committed
  template has bare `@N/<0;1>/*` slots, and the 11 keys live at **divergent**
  accounts per master (A,B at `48'/0'/{0..3}'/2'`, C at `48'/0'/{0..2}'/2'`).
  So encoding it today has only bad options: no `--path` → cards with
  unspecified origin (decode exit 4, partial), or `--path` → one flattened
  origin over several different keys of one master, which `md encode` now
  correctly REFUSES. The fix is the one F-217's close established: per-key
  origins **in the template** — `@0/48'/0'/0'/2'/<0;1>/*`, …,
  `@10/48'/0'/2'/2'/<0;1>/*` — which simultaneously kills the exit-4 partial
  decode (the origin travels on the card) and the flatten refusal. The origin
  for each slot is read from the matching `keys/key-NN.xpub` header, and the
  transcript asserts template-origin == key-header-origin per slot (G1), so the
  two committed artifacts cannot drift apart. The regenerated
  `backup-strings-tr.txt` supersedes the committed 3 chunks (which encode the
  origin-less form); both files are recommitted together.
- **P3 — preflight: `me bundle --preview` accepts the tr template set.** The
  wsh pathological journey proves the bundle path for wsh; nothing yet has run
  it on a tr template whose leaves are all `and_v`. F-215's shape guard refuses
  `tr(sortedmulti_a)` and nested `sortedmulti` — neither occurs here — but that
  is an inference, and departures get RUN, not cited. One command settles it
  (G7's first half). If it refuses, the plates section is cut from the journey
  and the refusal is filed as its own follow-up; the consent proof (E2) does
  not depend on it.
- **P4 — current binaries.** `md` at/after the F-217 close (`fe4b1ec9`) and the
  fork at/after the F-214 fix; the capture rebuilds `emu.wasm` first, as the
  pattern already forces. `derive-pathological-keys.sh --check` must pass, so
  the 11 xpubs are known to regenerate from the committed seeds.

---

## 3. THE ARTIFACTS — the four files, mirroring the Wallet Policy set

All four mirror their `*_walletpolicy` counterpart; deltas are listed, the rest
is the same mechanism.

1. **`design/journeys/transcript_tr_pathological.sh`** — the host half.
   Uses `run`/`runcap` (with the zero-match-capture FATAL) and its own output
   subtree `out/tr-pathological/` (the two-journeys-one-out clobber lesson).
   Sections:
   - versions (md, me, me-preview);
   - the policy: `cat` the rewritten `wallet-policy-tr.txt`, with the per-slot
     origin↔key-header consistency check (G1, FATAL);
   - the **keyless template set**: `md encode --group-size 0 --force-chunked`
     (no `--path` — the origins ride in the template, and the transcript prose
     says why, citing F-217) → runcap `out/tr-pathological/md1-template.txt`;
     then `diff` against `inputs-pathological/backup-strings-tr.txt` (G3) and
     `md decode` → `diff` against the policy file (E1/G4, FATAL);
   - the **keyed set**: same encode with 11 `--key "@N=xpub…"` and 11
     `--fingerprint "@N=fp"` args built from the key files → runcap
     `out/tr-pathological/md1-keyed.txt`;
   - the **bind**: `md inspect` on each set; assert equal
     `wallet-descriptor-template-id` (FATAL); runcap the keyed set's
     `wallet-policy-id` → `out/tr-pathological/tr.id.txt`;
   - **what the device must prove to**: `md address` on the keyed CARDS (not on
     the argument list — proving what the cards carry, per the walletpolicy
     rationale), `--chain 0 --count 2` and `--chain 1 --count 2` → runcap
     `tr.receive.txt` / `tr.change.txt`;
   - **plates** (conditional on P3): `me bundle --in md1-template.txt --preview
     out/tr-pathological/plates --png --manifest manifest.json`.
2. **`design/journeys/capture_tr_pathological.py`** — the device half.
   Copy of `capture_walletpolicy.py`'s skeleton: fatal-on-missing artifact
   reads (F-210), emu.wasm rebuild, shot_server on its own port, the
   size-not-existence shot check (≥512 bytes), result JSON. Deltas:
   - reads `md1-keyed.txt`, `tr.id.txt`, `tr.receive.txt`, `tr.change.txt`;
   - shot prefix `t00-…`;
   - **`--prove-it-can-fail` mode**: corrupts one character of the first
     expected address, runs the walk, and exits 0 **iff** the walk failed with
     the mismatch message. The negative control becomes a command (G6) instead
     of a one-time memory.
3. **`cmd/emu/shots_tr_pathological.js`** (in the seedhammer fork, beside the
   other walk drivers) — the walk. Copy of `shots_walletpolicy.js` with the
   deltas in §4: higher consent `maxPages` (the id plus four addresses for an
   11-key wallet pages further than 8), a longer gather timeout scaled to the
   measured keyed chunk count, and two **absence** assertions on the consent
   text.
4. **`design/journeys/build_pdf_tr_pathological.py`** — the document.
   Copy of `build_pdf_walletpolicy.py`: transcript-section extraction, embedded
   shots, the matched-result JSON rendered as the proof table, and the
   missing-asset gate **default-on, exit 1** (`--allow-missing` opt-out only).
   New content sections: the four-tier table with the **F-133 inversion stated
   in the caption** (tier 4, the weakest 1-of-3, matures ~90 days FIRST), the
   **F-132 preimage warning** on its own panel (tiers 1–2 cannot be spent from
   this backup alone; 0 of the strings carry X), and the plates + checklist
   page (if P3 held). Output: `SeedHammer-II-tr-pathological-journey.pdf`.

Supporting edits (not new files): the P2 rewrite of
`inputs-pathological/wallet-policy-tr.txt` + regenerated
`backup-strings-tr.txt`; a README row in the journeys table plus the
three-command reproduce block; the G0 conformance vector in the fork's
`md/testdata/vectors/` (named `keyed_tr_pathological_vault.*`, Rust addresses
as ground truth — it belongs to the F-214 fix, and this journey refuses to
start until it exists and passes).

**Scope decision, and why it is defensible.** The journey drives the **whole
11-key policy** — the card set is indivisible, and the device derives addresses
for the whole tree or not at all, so "a subset of keys" is not even coherent
here. What it does NOT redo is the per-key ceremony: the 11 mk1 key-card
engravings (~30 plates) and the 3 ms1 seed legs are byte-for-byte the mechanism
the wsh pathological journey already documents (mk1 cards bind to a wallet only
through the 8-hex policy-id stub; everything else about them is
taproot-independent), and re-capturing them would add operator effort and
document pages while asserting nothing new. One gather, one consent, one plate
preview: the whole wallet, a bounded walk.

---

## 4. THE WALK — step by step, with what each step ASSERTS

Inherited coordinates and helpers (`waitFor`/`raceFor` on squashed screen text,
`tap`, `screenShot` with the data-URL emptiness refusal, `readAllPages` with
stop-on-wrap) are the walletpolicy walk's, unchanged.

1. **Boot.** Race `["SeedHammer", "systemwide payload is present"]`; on the
   payload prompt, Back = skip (this operator's wallet arrives on cards). Shot
   `t00-boot.png`. *Asserts:* the emulator reached a known first screen.
2. **Reach Wallet Policy by name.** Seven carousel right-taps, then
   `waitFor("WalletPolicy")` — the count is a hint, the title is the assertion
   (a miscount must fail, not document the neighbouring program). Shot
   `t01-carousel.png`.
3. **Enter; empty gather.** `waitFor("md1descriptors:0")`. Shot
   `t02-gather-empty.png`. *Asserts:* the program starts with nothing gathered
   — the tally that later reads 1 started at 0 in this same run.
4. **Present the keyed chunk set.** Every chunk space-stripped (NFC records
   carry no spaces), queued via `shNFC.present` without per-chunk waits (the
   tally counts CARDS, not chunks — waiting per chunk hangs on chunk 1).
   `waitFor("Cardadded", timeout scaled to the measured chunk count)`, then
   assert the tally reads **exactly 1** — N chunks assembled into ONE card, no
   more, no fewer. A dropped chunk fails loudly here: the set never completes
   and the wait times out. Shot `t03-gather-full.png`.
5. **Done → consent.** `waitFor("Policy-ID")`; `readAllPages` with `maxPages`
   raised (12), stop-on-wrap retained. Shots `t04-consent-p0.png` ….
   *Asserts, on the joined pages — this is the journey:*
   - contains the host's `wallet-policy-id` (squashed);
   - contains **all four** host addresses (2 receive + 2 change, squashed);
   - does **NOT** contain `"Complexpolicy"` / `"displayonly"` — the F-214
     regression tripwire, so a regressed device fails with a message naming
     F-214 rather than a bare "address missing";
   - does **NOT** contain `"Keylesstemplate"` — the F-216 tripwire, so
     accidentally presenting the template set cannot pass as the keyed proof.
   Any miss throws `the device's proof does not match the host's`, listing
   every missing item and the full consent text.
6. **Return** `{shots, chunksPresented, cardsGathered, consentPages, matched}`
   for the capture to persist as `tr-result.json` — the builder renders the
   proof table from this, so the PDF's claim of agreement is the walk's own
   return value, not prose.

The walk stops at consent. It does not confirm into an engrave: the emulator
engrave overlay for this card set adds capture time and asserts nothing the
plates section (host-side, `me bundle --preview`) does not already assert
better via the manifest gate.

---

## 5. GATES — each machine-checkable, each one command

Publishable = G0–G5, G7, G8 green, and G6 run at least once on the final
artifact set.

| # | gate | command | green means |
| --- | --- | --- | --- |
| G0 | device can derive THIS wallet (pre-emulator, catches a too-narrow F-214 fix cheaply) | `cd /scratch/code/shibboleth/seedhammer && go test ./md/ -run KeyedTrPathologicalVault` | fork derives the vector's addresses == Rust ground truth |
| G1 | template origins == key-file origins, per slot | inside `transcript_tr_pathological.sh` (FATAL); surfaced by G2 | the committed template and the committed keys describe the same wallet |
| G2 | host half runs green, end to end | `bash transcript_tr_pathological.sh > transcript_tr_pathological.txt 2>&1 && ! grep -qE '^\[exit [1-9]' transcript_tr_pathological.txt` | every host command exit 0 (this journey plans no deliberate-refusal steps) |
| G3 | regenerated cards == committed cards | `diff out/tr-pathological/md1-template.txt inputs-pathological/backup-strings-tr.txt` | `backup-strings-tr.txt` has a producer and cannot silently drift (closes its zero-consumers state) |
| G4 | E1: decode == policy file, byte-identical | inside the transcript: `diff <(md decode <chunks>) inputs-pathological/wallet-policy-tr.txt` (FATAL) | the structural round trip |
| G5 | E2: capture complete AND device == host | `python3 capture_tr_pathological.py` (exit 0 only if all shots ≥ 512 B and every expected string matched) | the functional round trip |
| G6 | the comparison can fail | `python3 capture_tr_pathological.py --prove-it-can-fail` (exit 0 iff the corrupted run FAILED with the mismatch message) | the negative control, as a command instead of a memory |
| G7 | plates carry exactly the card strings | `me bundle --in … --preview … --manifest manifest.json` exit 0, then a `diff` of the manifest's engraved strings (space-stripped, sorted) against `md1-template.txt` (sorted) | plate content == card content; the honest proxy for read-back |
| G8 | the document is whole | `python3 build_pdf_tr_pathological.py` | exit 1 on ANY missing shot/plate (default enforcement); exit 0 → every image in the PDF is real |

Also binding, from the transcript's internal FATALs (surfaced through G2): the
keyed and keyless sets report the same `wallet-descriptor-template-id`.

---

## 6. WHAT IS NOT SHOWN — and why, one line each

- **A spend, from any tier** — no signer, no chain, and no spend claim is made
  anywhere in the document.
- **The sha256 preimage X** — F-132: it is required to spend tiers 1–2 and is
  in **zero** backup strings; the document restates this as a warning panel
  rather than implying completeness.
- **Tier order as a device fact** — F-133's inversion (weakest key-set matures
  ~90 days first) is prose + table from the recorded measurement; no screen
  shows it and the journey does not pretend one does.
- **Key-path spending** — the internal key is the BIP-341 NUMS point,
  unspendable by construction; neither host nor device asserts anything about
  it beyond carrying it faithfully (E1 covers the bytes).
- **Plate → card read-back** — no reader exists; G7's manifest equality is the
  proxy and is labelled as such.
- **Physical engraving** — plates are `me-preview` renders; the emulator
  engrave overlay is skipped (asserts nothing G7 does not).
- **The mk1 key cards and ms1 seed legs** — taproot-independent, documented in
  the wsh pathological journey; F-130's descriptor-string caveat lives there.
- **Template + mk1 join on device** — F-216: deliberately unimplemented (a
  wrong slot mapping would present a wrong address as proof); the journey uses
  the keyed card and says why.
- **Anything F-214's fix still refuses** — if G0 is green, nothing in THIS
  wallet is refused; shapes outside it (other leaf kinds, wsh gaps) remain
  refused and remain out of scope here.

---

## 7. RISKS — how this journey could report success while proving nothing

- **Shared wrongness across both implementations.** F-217's lesson verbatim:
  the device-vs-host comparison passed happily against a wallet that could not
  exist, because addresses derive from the xpubs regardless of the declared
  origin. Two mitigations in-plan: origins are now per-key and
  encode-validated (P2), and G0's ground truth comes from rust-miniscript's
  independently reviewed serialization rather than from the fork. Residual and
  named: if Rust and Go serialize the same *wrong* leaf script, every gate
  stays green. The one escape is a third implementation — an optional one-time
  `bitcoin-cli deriveaddresses` cross-check of the first receive address is
  worth the ten minutes and is recorded in the document if run; it is not a
  gate.
- **The comparison silently degrades into a photo shoot.** If the consent
  screen renders the id but the assertions are edited loose (substring too
  short, addresses list empty), the walk passes on anything. G6 exists for
  exactly this: the corrupted run must FAIL, every time the artifacts change.
- **Display-only consent masquerading as proof.** A partially-regressed device
  could show the id (computed from the card) with no addresses; the id match
  alone would look like agreement. Mitigated twice: all four addresses are
  asserted, and the walk asserts the *absence* of the "Complex policy — display
  only" text so the failure names F-214.
- **Stale host artifacts under the capture.** The capture reads `out/` files
  the transcript wrote — in an earlier session, possibly against an older
  template. F-210's mitigation (fatal-on-missing) does not catch *stale*. G3
  narrows the window (committed cards must equal regenerated cards), but the
  keyed set has no committed twin; the residual risk is accepted as the
  walletpolicy pattern does, and the reproduce block always runs transcript →
  capture → build in one sequence.
- **The gather "succeeds" on the wrong card.** Presenting the keyless set by
  mistake gathers 1 card and reaches consent; the F-216 absence assertion and
  the four address assertions both fail it — but only because they exist.
  Conversely a truncated keyed set can never pass: the card count never reaches
  1 and the wait times out loudly.
- **Consent paging truncation.** An 11-key consent may page past the driver's
  cap; stop-on-wrap plus the raised cap handle growth, and a cap hit still
  fails (the missing address throws) rather than passing thin — but the error
  would say "address missing" when the truth is "page cap"; the driver should
  report pages read in the throw, as the walletpolicy driver reports screen
  text.
- **Device-side size ceilings.** Nothing has ever pushed an 11-xpub keyed
  policy through the NFC gather and the consent renderer; an assembly or
  rendering limit would surface here first. That failure is a *result* (a
  follow-up in the F-214 family), not a journey defect — the plan's order
  (G0 before any capture) keeps the discovery cheap.
- **`me bundle` refusal on tr templates (P3).** If the preflight refuses, and
  the plates section were kept anyway with `--allow-missing`, G8's opt-out
  would ship a document quietly asserting less than it claims — so the rule is:
  P3 refusal cuts the section *in the builder*, never via `--allow-missing`.
