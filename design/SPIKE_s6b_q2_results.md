# S6b — Q2 spike results (executable, run 2026-08-17)

**This is the gate §3 Q2 asked for and no document had ever run.** Everything
below is a measurement produced by running code, not a reading of it. The spike
itself is throwaway and is **not merged** — it lived in worktree
`wt-s6b-spike` on branch `s6b-q2-spike`, off fork `main` = `b1479a1`.

Toolchain: **go1.26.3** from the nix store. Tests scoped with `-run`; stdout and
stderr captured to separate files, **stderr empty on every run**.

> **Correction to the standing runbook.** `CONTINUITY_2026-08-17.md` and this
> repo's notes say to use `export PATH="/nix/var/nix/profiles/default/bin:$PATH"`.
> **That profile contains no `go`** — it holds only the `nix*` tools. The
> toolchain is at `/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin`,
> and CI uses `actions/setup-go@v6` with `go-version: '1.26'` — the fork's
> workflow file `.github/workflows/test.yml`, lines 10-12, **hand-verified**
> because `plan-cite-check.sh` strips the leading dot and reports the path as
> `github/workflows/test.yml … (no such file under any root)`. That is a parser
> limitation, not a bad citation; expect `dangling: 1` on this file. The old
> PATH line produces `command not found: go`, which the runbook itself warns
> "proves nothing".

---

## 1. The documented variant thresholds REPRODUCE — but their unit is unstated

`backup/backup.go:397-403` claims: *"TEXT+QR fails first (works through 268
chars, fails at 269), then QR-ONLY (641, fails at 642), and TEXT-ONLY fails LAST
(645, fails at 646)."*

Measured by growing `"md1" + strings.Repeat("q", n-3)` through `validateMdmk`
and recording each change in the offered label set:

| variant | comment | measured, TOTAL string | measured, FILLER only |
| --- | --- | --- | --- |
| TEXT + QR | 268 / fails 269 | 271 / fails 272 | **268** ✓ |
| QR ONLY | 641 / fails 642 | 644 / fails 645 | **641** ✓ |
| TEXT ONLY | 645 / fails 646 | 648 / fails 649 | **645** ✓ |

**Every figure is exactly +3 — the `"md1"` prefix.** The documented numbers are
**correct**, and count the filler rather than the whole string. The comment does
not say which, and the difference is large enough to be read as drift. **A
future edit should state the unit**; this spike nearly reported a regression
that does not exist.

---

## 2. Q2's headline: a title/footer band FITS, with large margin

Measured by reserving *N* blank text rows at the top of `EngraveText` (a proxy
for R-F's optional band: what a band costs the body is the vertical space it
occupies) and re-measuring the TEXT+QR limit.

| title rows reserved | TEXT+QR holds | slack over the longest real string (111) |
| --- | --- | --- |
| 0 (today) | 271 | +160 |
| **1 (title)** | **262** | **+151** |
| **2 (title + footer)** | **240** | **+129** |
| 3 | 230 | +119 |
| 4 | 228 | +117 |

**A one-row title costs 9 characters of body budget. Title + footer costs 31.**

Against real strings, measured in-repo:

| string | length |
| --- | --- |
| single-sig `md1` (gui tests) | 67 |
| `mk1` key card (gui tests) | 74 |
| chunked `md1` (s2 golden) | 80 |
| longest `md1`/`mk1` literal found anywhere in-repo | 111 |

**Answer to Q2: YES, comfortably.** Even title + footer leaves 240 characters
against a longest-observed 111 — better than 2× margin.

**Stated limit of this measurement:** 111 is the longest string *found in the
repository*, not a proven upper bound on `md1`/`mk1`. The bound that makes this
robust is **chunking** — the s2 golden's multisig cards are 80 characters each
because a long policy is split across cards — but the spike did **not** measure
the maximum chunk payload. **A spec relying on this should measure that bound.**

---

## 3. Q3's proposal is WRONG about the passphrase plate: both bands are FULL

This spike was written to check Q2 and found a defect in the **Q3** proposal,
which claimed the wallet-policy id needed only *"one optional field and one
conditional line"*. Structurally true; **there is nowhere to put the line.**

Measured `passphraseLayoutFor` occupancy:

| case | topLines | bottomLines |
| --- | --- | --- |
| no metadata | 0 | 0 |
| both fingerprints | 2 | 1 |
| **worst case: both fps + a spaced passphrase** | **2** | **2** |

Worst-case contents:

```
top[0]     "SEED FP: 73c5 da0a"
top[1]     "EXPECTED COMB FP: fc60 c6df"
bottom[0]  "\x1f = SPACE"
bottom[1]  "FINGERPRINTS TYPED, NOT VERIFIED"
```

And the band geometry, measured (`smallEm = 19200` device units = 3 mm;
`bottomY = 480000` = 75 mm; band must stop at `plateSize - outerMargin` = 82 mm
= 524800; the metal ends at 85 mm = 544000):

| bottom lines | ends at | past the BAND | past the PLATE |
| --- | --- | --- | --- |
| 2 | 518400 | no | no |
| **3** | 537600 | **YES** | no |
| 4 | 556800 | yes | yes |

**So the code comment at `backup/passphrase.go:171-174` is correct — at most two
lines per band — and a third line does not error, does not clip: it silently
cuts into the 3 mm outer margin**, the plate-edge/screw-hole zone. `band`
(`backup/passphrase.go:228-235`) has no refusal of any kind.

### What this forces the spec to decide

There is **no free band line in the worst case**, so R6's policy id cannot
simply be added. The options, none chosen here:

1. **Render it only when a slot is free**, never displacing a safety line.
   Omission under-claims, which is the direction **R-D** tolerates — but the id
   is then absent exactly when the plate is busiest.
2. **Displace `passphraseLegend`** (`"\x1f = SPACE"`). Rejected on its face: it
   is needed precisely when the passphrase contains spaces, and it is more
   safety-critical than an identifier.
3. **Merge the two fingerprint lines** to free a top slot.
4. **Shrink `smallEm` for band lines**, buying a third line within 7 mm.
5. **Do not put the policy id on this plate at all.**

**Note the interaction with R-C:** the preloaded path already requires a
*different* footer string (`"FINGERPRINTS TYPED, NOT VERIFIED"` is false when
the device derived them). Whatever replaces it competes for the same
`bottomLines` budget measured above.

---

## 3b. The band's HORIZONTAL budget — 42 characters (measured after the fact)

§4 below originally listed the band's horizontal fit as **not measured**. Three
of the five options for the policy id turned out to depend on it, so it was
measured rather than reasoned about.

`band` centres each line at `(plateX-w)/2` with **no refusal**, so a line wider
than the plate simply runs off both edges. Measured against
`engrave.String(constant.Font, smallEm, s).Measure()`:

- plate width = **544000** device units
- the band face is **fixed-width at 12800 units per character**, confirmed by
  `W`, `X`, `0` and space all fitting **42** and failing at 43
- **so a band line holds 42 characters**

| line | chars | units | fits |
| --- | --- | --- | --- |
| `SEED FP: 73C5 DA0A` | 18 | 230400 | yes |
| `EXPECTED COMB FP: FC60 C6DF` | 27 | 345600 | yes |
| `FINGERPRINTS TYPED, NOT VERIFIED` | 32 | 409600 | yes |
| `FINGERPRINTS DERIVED, NOT TYPED` | 31 | 396800 | yes |
| `SEED 73C5 DA0A  COMB FC60 C6DF` | 30 | 384000 | yes |
| `POLICY 1A2B 3C4D` | 16 | 204800 | yes |
| **`POLICY 1A2B 3C4D  DERIVED, NOT TYPED`** | **36** | 460800 | **yes** |
| `SEED 73C5 DA0A  COMB FC60 C6DF  POLICY 1A2B 3C4D` | 48 | 614400 | **NO** |

**This is the number §2.3 lacked.** That section records only that this
mechanism has *"no 18-char cap"* — true, and not a budget. The budget is 42
horizontally and 2 lines vertically, per band.

---

## 3c. The `Text` title/footer budget — 25 characters

Measured 2026-08-17, after §3b, for the `md1`/`mk1` plates R-F marks. **This is
the provenance of the 25-character figure the spec uses**; it is recorded here
rather than in the spec because measurements belong with the spike that ran them.

- A title/footer must sit inside `[innerMargin, plateSize - innerMargin]` =
  `[64000, 480000]` = **416000 device units** — the bound
  `TestTitleCapFitsAtEveryRung` uses.
- `plateFontSizeUR`, what every `md1`/`mk1` `Text` caller constructs, is
  **3.8 mm** — *not* the free-text ladder's tight 6.0 mm rung.
- At 3.8 mm, **25 characters** fit the span.

| candidate | chars | units | fits |
| --- | --- | --- | --- |
| `PASSWORD REQUIRED` | 17 | 275621 | **yes** (66% of span) |
| `SEED FP: 73C5 DA0A` | 18 | 291834 | yes |
| `COMB FP: FC60 C6DF` | 18 | 291834 | yes |
| `SEED 73C5 DA0A  COMB FC60 C6DF` | 30 | 486390 | **NO** |
| `EXPECTED COMB FP: FC60 C6DF` | 27 | — | **NO** (> 25) |

**METHOD CAVEAT — retest properly before relying on it for a gate.** This
measured **raw string width against the inset span**.
`TestTitleCapFitsAtEveryRung` instead drives the real layout (`layAt`,
`lay.holeChars * lay.charWidth`), and **the two do not agree at the 6.0 mm
rung** — raw width admits only 16 characters where the shipped cap is 18.

The disagreement **errs in the safe direction**: raw width *under*-reports, so
25 at 3.8 mm is conservative rather than optimistic. **But the implementation's
gate must be the layout-based form, not this one.**

> **CONFIRMED BY MEASUREMENT, P2, 2026-08-17.** The layout-based budget is
> **28 characters**, not 25. This caveat predicted both the direction and the
> fact of the gap, and the gate now asserts 28
> (`backup/engravetext_test.go`).
>
> Worth recording because this caveat was **dropped in the spec's clean rewrite
> and restored a commit later** on a fidelity finding. Had it stayed dropped, P2's
> measurement would have read as a contradiction of a settled number instead of
> a confirmation of a stated limit — and the cheapest response to an apparent
> contradiction is to distrust the new measurement.

---

## 4. What the spike did NOT measure

Named so the gate's blind spot is visible, per project practice:

- ~~The maximum `md1`/`mk1` chunk payload~~ — **CLOSED. It is a hard,
  code-enforced constant**, found by R0 round 1: `ValidMD` rejects a data part
  over `mdRegularMaxLen = 93` → **md1 ≤ 96 characters**; `ValidMK` admits only
  `[14,93]` and `[96,108]` → **mk1 ≤ 111 characters**
  (`codex32/mdmk.go:49,54-57,137-143,152-160`).

  So §2's *"longest in-repo 111"* is in fact the **absolute maximum**, and the
  240-character title+footer budget carries better than 2× margin against a
  **proven** bound rather than an observed one. §2's robustness caveat is
  retired.
- ~~Whether the band's own TEXT fits its width~~ — **measured, §3b: 42
  characters.**
- ~~The title band's horizontal fit on an `md1`/`mk1` plate~~ — **measured,
  §3c: 25 characters**, with the method caveat stated there.
- **Any rendered output.** No goldens were produced or compared, so R-G's
  "unmarked path stays byte-identical" claim is **still unverified** — it
  becomes checkable only once R-F's real (non-proxy) band exists.
- **The F-208 arrow layout** — settled since, by R-I, which chose a layout
  costing no body width and thereby decoupled F-192's sweep from it.
