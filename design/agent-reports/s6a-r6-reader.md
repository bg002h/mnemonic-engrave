# S6a round 6 — READER-COMPREHENSION lens

**Artifact:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
**Question:** can the actual reader (a stranger, years later, holding steel) act
correctly on what the restore document says?
**Scope:** the rendered document only, as specified in §4.2/§4.3/§4.4/§4.7d and
`buildPassphraseInventoryLines` (`gui/multisig_build_census.go`). No factual
accuracy, no verify logic, no build order, no test plan, no rewrite.

## VERDICT: RED — 0 Critical, 3 Important

---

## THE ASSEMBLED DOCUMENT, FOUR CASES

Content sourced verbatim from the plan (§4.2, §4.3, §4.4, §4.7d) and from
`buildPassphraseInventoryLines` / `singleSigRestoreLines` / `multisigRestoreLines`
as they exist today (`gui/multisig_build_census.go`, `gui/singlesig_restore.go`,
`gui/multisig_restore.go`), which the plan reuses unchanged for the "wallet
facts" block. Plate counts for single-sig are pinned by the shipped walk tests
(`"Card 1 of 3"` full, `"Card 1 of 2"` watch-only); the multisig build case's
non-seed card labels are illustrative (I did not re-derive
`multisig_build_engrave.go`'s exact card set) — the seed/passphrase/status lines
are exact.

### Case 1 — single-sig, full, no passphrase, VERIFIED

```
1.  Plates VERIFIED: each plate was read back and matched.
2.  Master fp: xxxxxxxx
3.  Descriptor:
4.  <chunked descriptor, ~20 chars/line>
5.  First receive:
6.  <address>
7.  First change:
8.  <address>
9.  This backup is 3 plates:
10. ms1 secret share: 1 plate (secret seed backup)
11. mk1 key: 1 plate (account key card)
12. md1 descriptor: 1 plate (wallet policy descriptor)
13. If any of them is missing, this backup is incomplete.
14. Seed: this set contains YOUR seed, on the plate marked 'ms1 secret share'.
    Treat that plate as the secret itself.
15. No BIP-39 passphrase was used, so no passphrase is needed to spend from
    this wallet.
16. Seed handling: this build does not time out. The seed you entered -- this
    build holds exactly one -- stays in device memory until the build ends,
    and on a full build the words are also on the plates as they are cut. Do
    not leave a mid-build machine unattended: the plates are the secret.
    Power the device off when you are done.
```

### Case 2 — single-sig, full, with passphrase, PLATES UNACCOUNTED FOR

```
1.  Some plates could not be checked against this run. Either a plate was not
    presented, or it is not one this run cut. Present every plate this run
    engraved and check again; if this repeats, re-cut the set.
2.  Master fp: xxxxxxxx
3.  Descriptor:
4.  <chunked descriptor>
5.  First receive:
6.  <address>
7.  First change:
8.  <address>
9.  This backup is 3 plates:
10. ms1 secret share: 1 plate (secret seed backup)
11. mk1 key: 1 plate (account key card)
12. md1 descriptor: 1 plate (wallet policy descriptor)
13. If any of them is missing, this backup is incomplete.
14. Seed: this set contains YOUR seed, on the plate marked 'ms1 secret share'.
    Treat that plate as the secret itself.
15. A BIP-39 passphrase WAS used. It is not on these plates and cannot be
    recovered from them: nothing this device engraves carries a passphrase.
16. Without it, these plates do not reach the money. Keep it somewhere
    separate, and make sure whoever needs this backup can also get the
    passphrase.
17. Seed handling: this build does not time out. The seed you entered -- this
    build holds exactly one -- ... the plates are the secret. Power the
    device off when you are done.
```

### Case 3 — single-sig, watch-only, NOT VERIFIED

```
1.  Plates NOT VERIFIED. Confirm they restore before relying on this backup.
2.  Master fp: xxxxxxxx
3.  Descriptor:
4.  <chunked descriptor>
5.  First receive:
6.  <address>
7.  First change:
8.  <address>
9.  This backup is 2 plates:
10. mk1 key: 1 plate (account key card)
11. md1 descriptor: 1 plate (wallet policy descriptor)
12. If any of them is missing, this backup is incomplete.
13. Seed: this set contains NO seed. It is watch-only: it records the
    wallet, but it can never spend. If funds must be recovered, the seed
    words must come from somewhere else -- no plate in this set holds them.
14. No BIP-39 passphrase was used, so no passphrase is needed to spend from
    this wallet.
15. Seed handling: this build does not time out. The seed you entered -- this
    build holds exactly one -- stays in device memory until the build ends.
    Do not leave a mid-build machine unattended: it is still holding seed
    material. Power the device off when you are done.
```

### Case 4 — multisig build, full, two seeds (one passphrased), DISAGREED

```
1.  WARNING: a read-back check DISAGREED with these plates. Do NOT rely on
    this backup: engrave a fresh set and check it before use.
2.  Type:
3.  <script/policy summary>
4.  Descriptor:
5.  <chunked descriptor>
6.  First receive:
7.  <address>
8.  First change:
9.  <address>
10. This backup is 4 plates:
11. ms1 secret share 1 of 2: 1 plate (secret seed backup)
12. ms1 secret share 2 of 2: 1 plate (secret seed backup)
13. mk1 key: 1 plate (cosigner key card)
14. md1 descriptor: 1 plate (wallet policy descriptor)
15. If any of them is missing, this backup is incomplete.
16. Seed: this set contains YOUR seeds, on the plates marked 'ms1 secret
    share'. Treat each of those plates as the secret itself.
17. A BIP-39 passphrase WAS used. It is not on these plates and cannot be
    recovered from them: nothing this device engraves carries a passphrase.
18. Without it, these plates do not reach the money. Keep it somewhere
    separate, and make sure whoever needs this backup can also get the
    passphrase.
19. Needs a passphrase: your seed for @1 (master fingerprint xxxxxxxx). If
    more than one is listed here they may be DIFFERENT passphrases; record
    each one against its fingerprint.
20. Needs NO passphrase: your seed for @0 (master fingerprint yyyyyyyy).
21. Seed handling: this build does not time out. Every seed you entered --
    this build can hold several -- stays in device memory until the build
    ends, and on a full build the words are also on the plates as they are
    cut. Do not leave a mid-build machine unattended: the plates are the
    secret. Power the device off when you are done.
```

---

### I-1 — a condemning or hedged status line is never referenced again; the rest of the page reads like a healthy backup, and one sentence flatly re-asserts reliance

**What the reader sees:** in Cases 2 and 4, line 1 is a hedge or an explicit
"do not rely" warning. Every line after the wallet-facts block (lines 9–21 in
Case 4, roughly two-thirds of the document) is written in the same confident,
unconditioned voice the VERIFIED case uses: "This backup is 4 plates ... If any
of them is missing, this backup is incomplete," "Treat each of those plates as
the secret itself," and — sharpest of all — "make sure **whoever needs this
backup** can also get the passphrase." That last clause reuses the identical
noun phrase ("this backup") that line 1 just said not to rely on, now used to
presuppose a future user who *does* rely on it.

**What they would plausibly do:** this is a paged, Button2-advanced screen; the
target reader reads it once, years apart from the operator, often to answer one
question ("what do I have, is it complete"). That question is answered on the
inventory pages, not the status page. A reader who retains the gist of page 1
but not its exact force, or who is handed a transcription that starts at "This
backup is 4 plates" (plausible — that line looks like the substantive content,
the warning looks like a caveat), will treat the set as usable: keep it, hand it
to an heir with the passphrase "so they can use this backup," or attempt a
restore from it.

**Why that is wrong:** DISAGREED means the device has already confirmed, by
reading the actual steel back, that it does not match. PLATES UNACCOUNTED FOR
means the device cannot rule out bad steel. Neither status is retracted,
softened, or even mentioned again after line 1. Nothing downstream is
conditioned on the status — the same `buildPlateInventoryLines` /
`buildPassphraseInventoryLines` text renders whether the status is VERIFIED or
DISAGREED. The design correctly puts the status line first (so it is never
absent), but "first" is not "load-bearing throughout" — placement alone does
not stop the rest of the document from vouching for itself in a scenario that
exists specifically because the device is not vouching for it.

---

### I-2 — `NOT VERIFIED` and `VERIFIED` differ by one leading word, at the single highest-stakes position in the document

**What the reader sees:** the very first line of the page is either
`"Plates VERIFIED: each plate was read back and matched."` or
`"Plates NOT VERIFIED. Confirm they restore before relying on this backup."` —
identical opening word, identical second word, the entire distinction carried
by one three-letter negation sandwiched between them.

**What they would plausibly do:** treat an unverified backup as verified (or
vice versa) on a skim, particularly under the low-attention, low-light,
small-screen conditions this document is actually read under (a stranger going
through a safe-deposit box, not proofreading).

**Why that is wrong:** this is exactly the class of substring/near-miss trap
the plan itself names and defends against **in its own test plan** (§5,
"NO SUBSTRING ASSERTIONS ON STATUS LINES ... `Contains(\"VERIFIED\")` passes on
the *not-verified* line") — the plan protects the *test suite* from this
confusion but ships the identical hazard to the *human reader*, unmitigated by
typography, a leading symbol, or a differently-worded opener.

---

### I-3 — the one PASS status that mentions `DISAGREED` shares its closing hedge with the two "unknown" statuses

**What the reader sees:** `"Plates VERIFIED on a repeat check, after an earlier
read-back DISAGREED. Confirm they restore before relying on this backup."` This
line is P1's "always prints a pass line" case — substantively a green light —
but it is the only pass line that (a) contains the word `DISAGREED`, capitalized,
and (b) ends with the identical "Confirm they restore before relying on this
backup" clause that `NOT VERIFIED` and `DID NOT COMPLETE` also use.

**What they would plausibly do:** a reader scanning for the alarming word reacts
to `DISAGREED` and treats this as a condemned backup (risk: discards or
re-engraves *good* plates unnecessarily); a reader scanning for `VERIFIED` reacts
to the leading clause and skips the trailing "confirm before relying" entirely,
collapsing it to plain `VERIFIED` and losing the one piece of history — an
earlier mismatch happened — the line exists to carry.

**Why that is wrong:** the line is trying to do two jobs (announce a pass, and
preserve a disagreement's memory) in one sentence that borrows vocabulary from
both the celebratory and the alarming ends of the six-status set, at the exact
point where the prompt's own test asks "can a lay reader tell the statuses
apart, or is one hedged enough to read as noise" — this one reads as *both*
noise and alarm simultaneously.

---

## THE SINGLE WORST MOMENT

In Case 4, line 1 reads **"Do NOT rely on this backup: engrave a fresh set and
check it before use."** Nine lines later, unconditioned and un-cross-referenced,
line 18 reads **"...make sure whoever needs this backup can also get the
passphrase."** — the identical phrase "this backup," reused to describe a future
user relying on the very artifact the document opened by condemning. No line in
between walks that back. A reader who reaches line 18 without perfect recall of
line 1's force is being told, in the document's own words, to provision a future
user of a backup the document itself says not to use.

## WHAT I FOUND CLEAR

- Status-line-first, exactly-one-line placement is the right structural call and
  is well executed — it is genuinely unmissable as the opening of the page.
- "YOUR seed" vs. "THE seed" (§4.4 point 1) is a sharp, deliberate,
  reader-tested choice: it answers "is this everything" honestly on a k-of-n
  multisig where a naive "the seed" would falsely read as "yes."
- The passphrase statement's two-sided disclosure (names who needs it *and*
  who doesn't, by label + fingerprint) prevents a reader from assuming every
  seed in a multi-master set needs the one passphrase they have.
- The watch-only absence line — "this set contains NO seed ... no plate in this
  set holds them" — is unhedged and gives exactly the right guidance with no
  ambiguity.
- The what-is → what-is-not → how-to-handle-it ordering within the inventory
  block (§4.4 placement) is a sound structure in principle; it is the
  *unconditioned reuse of that block regardless of verify status* (I-1) that
  undermines it, not the ordering itself.
- ASCII-only enforcement (§4.4, §6.2) is a direct, low-drama service to this
  reader: a string that cannot draw on the machine is worse than no string.
