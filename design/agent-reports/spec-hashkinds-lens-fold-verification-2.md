# Fold verification round 2 — did the fold close C-NEW-1 / I-NEW-1..4, and did it introduce anything new

**Artifact:** `design/SPEC_hashlock_kinds.md` at `47ce5013`, 644 lines. **Fold
diff:** `git diff 55861745..47ce5013` — one file, +57/-19.

**Scope:** (1) did the fold close each of C-NEW-1, I-NEW-1, I-NEW-2, I-NEW-3,
I-NEW-4 from `design/agent-reports/spec-hashkinds-lens-fold-verification.md`
(committed `55861745`); (2) did the fold introduce a new defect, including a
false citation. Not a fresh audit; the three correctness rounds, §2's operator
decisions, and prior-CLOSED findings are not re-opened.

**Trees read:** mnemonic-engrave `47ce5013` (working tree, clean), fork
`seedhammer` at `0562e811` (same SHA the prior report measured against — no
drift), mnemonic-secret working tree (clean). No experiments written; nothing
to revert.

---

# Verdict first

**NOT GREEN — 0 Critical / 1 Important / 1 Minor.**

C-NEW-1 and I-NEW-2 through I-NEW-4 are cleanly CLOSED, each verified against
the real source, not the report's claims about it. I-NEW-1 is CLOSED as to its
own literal ask (wrong identifier fixed, phase/vector homes stated), but the
fold's own remedy for C-NEW-1 broadened the same §11 row's title to "QR text
**and locator row**" without extending its citations to cover the locator
half — reproducing, one paragraph later, the exact defect class I-NEW-1 was
about. That is the one new defect below.

---

# Machine checks run

| claim | result |
| --- | --- |
| `gui/composer_copy.go:717-725` is `composerCopyPreimagePlateLead` and its final line prints a bare `method:` | **TRUE.** Function spans exactly 717-725; line 725 is `head + fmt.Sprintf("\nphrase: %d characters   method: %s", chars, method)` — no kind token. |
| `me-cli/src/main.rs:195,197` is hashlock help text reading `hash:<64 lowercase hex>` / "a sha256 hashlock digest" | **TRUE**, verbatim at those exact lines. |
| `hashlock.MethodLine` exists | **TRUE.** `hashlock/hashlock.go:176`, doc comment ties it to `ms_codec::hashlock::qr_text`, "Rust is primary". |
| `ms_codec::hashlock::qr_text` exists | **TRUE.** `ms-codec/src/hashlock.rs:208`, `pub fn qr_text(...)`. |
| `hashlock.QRText` embeds `MethodLine` | **TRUE.** `hashlock/hashlock.go:189`: `"hashlock v1\n" + MethodLine(hardened) + "\nphrase: " + phrase`. |
| H6 §6.5 pins the method line at 73 characters | **TRUE.** `SPEC_hashlock_H6_preimage_plates.md:1869`, "the hardened line is 73 characters, which is §6.5's own pin"; §6.5 itself is "Geometry — MEASURED, and the fit is tight" (`:1239`). |
| §13.1's 5-row probe table matches the prior report's probe output | **TRUE**, row for row: baseline (10 rows/1.20mm spare) = report row A; QR-gains-kind (210 bytes/53 modules/fits) = row E; method-line-88-chars (REFUSES, `427520`/`416000`) = row B's error text; body-row (REFUSES, same arithmetic) = row D; locator-row-35-chars (fits, 10 rows, 1.20mm spare) = row C. No transposition. |
| `SPEC_hashlock_H2_device.md` §4.6's phrase-dropped leg is quoted verbatim | **TRUE.** `:346`, "Back from the phrase screen → `Which hash?` (phrase dropped)" — exact string match. |
| The locator's `hash` row is built by `hashlockPlateLocator` (`gui/composer_preimage_plate.go:232-238`), called from `composerHashlockLocator` (`:256`) and `hashlockPlatesLocator` (`gui/composer_hashlock_plates.go:205-207`) | **TRUE** — none of these three identifiers or `composer_preimage_plate` appears anywhere in the spec (grep, 0 hits, both before and after this fold). |

---

# Closure, finding by finding

## C-NEW-1 — §13.1's "no layout change" — **CLOSED**

§13.1 now carries the full 5-row placement table (verified above), explicitly
forbids the method line (citing H6 §6.5's 73-char pin, itself verified real)
and a new body row (both REFUSE at every rung, arithmetic verified), and
permits exactly two placements: QR text and the locator's `hash` row. §11
gains a dedicated "fit gate and its goldens" row; §10's vector list gains
"its locator row, and a layout fit assertion at the worst case ... Goldens for
the plate bytes." ACCEPTANCE item 8's wording in §13.1 now reads "~43 min",
matching §12 item 8's own figure — incidentally resolving the prior round's
"two costs for one gate" Minor as a side effect, not a regression.

An implementer following §13.1 no longer reds the fit test, has a named free
placement in addition to the QR, and §11/§10 both carry the obligation to
verify it stays fitting. Genuinely closed.

## I-NEW-1 — wrong identifier / no home — **CLOSED as asked; see new defect below**

- Wrong-identifier limb: closed. `hashlockPlateFormWords` is gone from the
  row; `MethodLine` and `ms_codec::hashlock::qr_text` are both real and both
  are the correct functions for the QR-text/method-line content (verified
  above).
- No-Rust-primary limb: closed. §9 phase 2 now states "The plate's QR text
  gains `hash: <kind>` (§13.1) — `qr_text` lives here", putting the Rust-side
  change in the correct phase.
- No-vector limb: closed. §10 gained the locator-row + layout-fit + goldens
  bullet quoted above.

**But** the fold, in closing C-NEW-1, retitled the same §11 row to "the
preimage plate's **QR text and locator row**" — and left its citation list
(`MethodLine`, `ms_codec::hashlock::qr_text`, `backup/hashlock`) unchanged.
None of those three identifiers is where the locator row is built. This is
the same defect class I-NEW-1 itself was about, recurring one clause later;
detailed as a new defect below rather than re-opening I-NEW-1, since I-NEW-1's
own literal text (about the method-line/QR-text citation) is satisfied.

## I-NEW-2 — `me-cli` help text dropped in the fold — **CLOSED**

§11 gained the row `` `me-cli`'s hashlock help text | `me-cli/src/main.rs:195,197` ``.
Verified verbatim present at exactly those lines (table above).

## I-NEW-3 — phrase arm's Back leg unstated — **CLOSED**

§7.1 now states, for both arms, "The kind screen precedes the material entry
on both arms," gives a 3-row Back-leg table (kind screen, hex pad, phrase
screen), and for the phrase screen explicitly quotes H2 §4.6's existing leg
verbatim and repositions it one hop earlier: "with a screen inserted in
front, it stops one earlier, and the phrase is still dropped there." Both of
I-NEW-3's specific asks — where the kind screen sits in the phrase loop, and
what happens to the H2 §4.6 leg — are answered. Traced the forward/back
topology (`Which hash?` → kind screen → phrase screen → method pick →
derivation → confirm) and it is internally consistent: nothing contradicts
H2 §4.6, which is untouched (this fold changed only `SPEC_hashlock_kinds.md`).

*Minor residue, not blocking:* the row's added parenthetical — "the operator
does not pay a second KDF to reach **the same point**" — is true only for the
Back-hop itself (Back never triggers derivation, before or after this fold);
it does not establish that *completing* a kind-only correction (re-typing the
dropped phrase, re-deriving) avoids a second KDF, which is the cost the
original finding's F1 note was actually about. A reader could over-read this
as resolving that cost when it doesn't. The required leg itself is fully and
correctly stated regardless.

## I-NEW-4 — masked plate-pick lead unnamed — **CLOSED**

`gui/composer_copy.go:717-725` verified to be `composerCopyPreimagePlateLead`,
whose final line prints a bare `method:` token (table above, matches the
report's quoted three-line body exactly). §13.2 now opens "**Three screens,
not two**," names this function and line range, and states it is bound by
§5's rule. §11 gained its own row for it.

---

# New defect

## N2-1 (Important) — §11's "QR text and locator row" gate row does not cite the locator row

§11's row (post-fold): `` the preimage plate's QR text and locator row | `MethodLine`, `ms_codec::hashlock::qr_text`, `backup/hashlock` ``.

The row's own title claims two surfaces. `MethodLine` and `ms_codec::hashlock::qr_text`
correctly cover the QR-text half (verified: `MethodLine` is embedded inside
`QRText`, `hashlock/hashlock.go:189`). `backup/hashlock` is the layout/render
package (`backup/hashlock.go`, `EngraveHashlock`, `FontSizes`) that takes
`Locator []string` and `QRText string` as **caller-supplied** content — it
does not build either.

The function that actually builds the locator's `hash` row §13.1 now
mandates carry the kind is `hashlockPlateLocator`
(`gui/composer_preimage_plate.go:232-238`: `out = append(out, "hash  "+hashlockFirst8Last8(digest))`),
called from `composerHashlockLocator` (`:256`) and
`hashlockPlatesLocator` (`gui/composer_hashlock_plates.go:205-207`) — both of
which would need a kind parameter threaded through to reach it. None of these
three identifiers, nor the file `gui/composer_preimage_plate.go`, appears
anywhere in the spec (grep, 0 hits) — confirmed unchanged by this fold (the
pre-fold row at `55861745:488` also lacked it, but at that point the row's
title didn't yet claim the locator row, so there was no gap to have).

**This is the same defect class as I-NEW-1** — a §11 gate row whose citations
don't match the surface it claims to gate — reappearing in the very row the
fold edited to close I-NEW-1, for the new half of scope C-NEW-1 just added to
it. It is not wholly invisible to an implementer: §9 phase 4 says "Also §13's
device surface: the locator `hash` row..." and §13.1's body text names the
placement explicitly. But §11 is this spec's specific machine-checkable
per-file gate list — the thing a build gate or reviewer greps — and for
exactly the clause the fold just added, it is not one.

**Cheapest fix:** add `hashlockPlateLocator` (`gui/composer_preimage_plate.go:232-238`),
`composerHashlockLocator` (`:256`) and `hashlockPlatesLocator`
(`gui/composer_hashlock_plates.go:205-207`) to the row's citation list, or
split the row into "QR text" and "locator row" rows matching §13.1's own
two-placement structure.

No other new defects found. No other new or changed citation in this diff
was false; the probe table is transcribed without transposition; the H2 §4.6
quotation is verbatim; nothing in the touched sections contradicts anything
elsewhere in the document (checked §7.1's surrounding "kind before the pad"
text, §9's phase table, §12's acceptance items 7/8, and §13.2/§13.3 for
consistency with the new material — none flagged).

---

# Counts

**0 Critical / 1 Important / 1 Minor / 0 Nit.**

Closure: C-NEW-1 CLOSED, I-NEW-1 CLOSED (as literally asked; a fresh instance
of its defect class recorded as N2-1 above), I-NEW-2 CLOSED, I-NEW-3 CLOSED
(1 Minor residue on a parenthetical's precision), I-NEW-4 CLOSED.

# Verdict

**NOT GREEN (0C/1I).** One more narrow fold — extend one §11 citation list (or
split one row) — should close this round.
