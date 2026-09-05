# Refute pass (skeptic 1 of 2): lens `interruption` I-1

**Target:** SeedHammer fork branch `hashlock-h2`, worktree pinned at `17b3979`.
**Claim under test:** the hashlock phrase screen (`hashlockPhraseFlow`,
`gui/composer_hashlock.go`) draws no readout at all — masked or revealed —
because `kbd.MaxHeight` leaves only 11 px against a 19+ px line, so
`PassphraseKeyboard.Layout`'s clamp binary-searches every rune away; this
matters under the `interruption` lens because §4.6 preserves the typed phrase
across 4 of 5 interruptions on this route, and at every resumption the operator
can verify only how many bytes survived, never which.

**Verdict: CONFIRMED.** The mechanism reproduces exactly, byte-for-byte against
the numbers the original reviewer cited, and the interruption framing (severity
Important, not Critical) is accurate.

## Reproduction

Wrote `gui/zz_refute_readout_test.go` (not committed — scratch worktree,
removed with the worktree at the end of this task) that mirrors
`hashlockPhraseFlow`'s own layout arithmetic line-for-line
(`gui/composer_hashlock.go:163-176`) and then calls the real
`PassphraseKeyboard.Layout` with a 4-rune fragment (`"abcd"`), extracting drawn
text with `op.Drawer.ExtractText` exactly as the codebase's own tests do
(`gui/frame_hook_test.go:26` is the pattern followed).

```
go test ./gui/ -run 'TestZZReadoutBudget' -count=1 -v
```

```
=== RUN   TestZZReadoutBudgetShippedLead
    dims=(480,320)
    content after leading/bottom cut: {(0,44) (480,312)} (Dy=268)
    leadSz=(440,44)
    cntsz=(44,23)
    kbd.MaxHeight (content3.Dy()) = 201
    kbd grid+readout combined size = (340,209)
    extracted text from kbdOp: "qwertyuiopasdfghjklzxcvbnmABCspaceshow"
    SHIPPED lead: leadY=44 cntY=23 avail=11 containsFragment=false
--- PASS: TestZZReadoutBudgetShippedLead (0.00s)
=== RUN   TestZZReadoutBudgetSpecLead
    dims=(480,320)
    content after leading/bottom cut: {(0,44) (480,312)} (Dy=268)
    leadSz=(396,23)
    cntsz=(44,23)
    kbd.MaxHeight (content3.Dy()) = 222
    kbd grid+readout combined size = (340,209)
    extracted text from kbdOp: "qwertyuiopasdfghjklzxcvbnmABCspaceshow****"
    SPEC-LITERAL lead: leadY=23 cntY=23 avail=32 containsFragment=false
--- PASS: TestZZReadoutBudgetSpecLead (0.00s)
=== RUN   TestZZReadoutBudgetPassphraseEntryFlowForComparison
    passphraseEntryFlow kbd.MaxHeight=245, avail=55
    passphraseEntryFlow extracted: "qwertyuiopasdfghjklzxcvbnmABCspaceshow****" containsFragment=false
--- PASS: TestZZReadoutBudgetPassphraseEntryFlowForComparison (0.00s)
=== RUN   TestZZReadoutBudgetShippedLeadRevealed
    REVEALED, shipped lead: extracted="qwertyuiopasdfghjklzxcvbnmABCspacehide" containsFragment=false containsStars=false
--- PASS: TestZZReadoutBudgetShippedLeadRevealed (0.00s)
PASS
ok  	seedhammer.com/gui	0.006s
```

Every number the reviewer cited reproduces exactly:

- panel `(480,320)`, content `268 px` — confirmed (`content after leading/bottom
  cut ... Dy=268`).
- lead band `44 px` for the **shipped** lead — confirmed (`leadSz=(440,44)`),
  versus `23 px` for the spec-literal lead (`leadSz=(396,23)`) — the second
  measurement independently confirms the *cause*: the departure from §4.2's
  quoted copy (`design/SPEC_hashlock_H2_device.md:189-191`, the lead
  is *"Use a phrase you have never used anywhere else."*) is what doubles the
  band from one line to two. `composerCopyHashlockPhraseLead()`
  (`gui/composer_copy.go:367-370`) prepends *"This screen does that hashing for
  you."* — a deliberate, reviewed change: it is the R0 journey-I-5 fix recorded
  at `design/IMPLEMENTATION_PLAN_hashlock_H2_device.md:3178-3179` and covered by
  the citation test `gui/composer_copy_test.go:120-121`. That round evidently
  never re-measured this screen's readout budget against the new, longer lead.
- counter band `23 px` — confirmed (`cntsz=(44,23)`).
- grid `182 px` tall (`kbd.MaxHeight − avail − 8` = `201 − 11 − 8`) at `340 px`
  wide — confirmed (combined size `(340,209)` = grid `182` + gap `8` + readout
  `19`, at avail = `11`).
- readout budget **11 px** for `hashlockPhraseFlow` under the shipped lead —
  confirmed (`avail=11`) — versus **55 px** for `passphraseEntryFlow`, which
  cuts no lead band at all (`gui/passphrase_flow.go:113-135` — no
  `widget.Labelw(... lead)` call in that flow) — confirmed
  (`kbd.MaxHeight=245, avail=55`).
- **Neither the masked `****` nor the revealed cleartext is drawn** under the
  shipped lead: the extracted frame text for both `kbd.revealed == false`
  (`TestZZReadoutBudgetShippedLead`, text `"...space​show"`, no `abcd`, no
  `****`) and `kbd.revealed == true`
  (`TestZZReadoutBudgetShippedLeadRevealed`, text `"...spacehide"` — confirming
  the cap correctly flips to `hide` per §4.2's "inherited as-is" — but still no
  `abcd`, no `****`) is confirmed empty of any readout. Under the
  spec-literal lead (`avail=32`), the same 4-rune fragment renders `****`
  (`TestZZReadoutBudgetSpecLead`) — the clamp is not inherently broken, only
  starved by the shipped copy's extra line.

Control, independently re-run: `gui/passphrase_keyboard_test.go:134`
`TestPassphraseMaskReveal` constructs the same widget with `MaxHeight` left at
its zero value (unbounded — the doc comment at `gui/passphrase_keyboard.go:56`
states "0 means unbounded") and asserts `****` for the masked case and the
cleartext for the revealed case. This is the widget working correctly when not
starved — the defect is in the caller's arithmetic, not the widget, exactly as
the original report stated.

## Interruption framing checked against §4.6

`design/SPEC_hashlock_H2_device.md:289-303` (§4.6, "The Back contract") states,
verbatim, the five Back points on this route and which preserve the phrase:

- Back from the confirm modal → method pick, **phrase intact**
- Back from a declined method modal → method pick, **phrase intact**
- Back from the method pick → phrase screen, **phrase intact** (via `initial`)
- Back from the phrase screen → `Which hash?`, **phrase dropped**
- Back during the derivation → method pick, **phrase intact**

That is 4 of 5 preserving, exactly as claimed. Each of the four "phrase intact"
arms is present verbatim in `gui/composer_hashlock.go`'s
`hashlockPhraseRoute` (the `pick:` loop's three `continue`/`break pick` arms
plus `hashlockDeriveFlow`'s `!ok` arm) — read directly, not inferred from a
comment. The fifth channel the reviewer names — Run's screensaver — is a
different mechanism (an idle blank/wake, not a Back tap) but preserves state
for the same reason all four Back arms do: `phrase` is a Go-stack local held by
the still-running `hashlockPhraseRoute` closure, and neither a screensaver
blank nor a Back-preserving loop iteration unwinds that stack. At every one of
these five resumptions, the only phrase-derived signal on screen is the
`n/100` counter — never which bytes are present, confirmed above.

## Severity: Important, not Critical — confirmed correct

Checked against this task's severity rubric. This is not a digest-divergence
path (the digest is computed correctly from whatever `kbd.Fragment` actually
holds — that's a separate question from whether the operator can *see* it),
not lost operator work (the counter survives every interruption and the
confirm screen at §4.5 still shows the true `len(phrase)` via `chars: %d` —
`composerCopyHashlockConfirm`, `gui/composer_hashlock.go:64`), not a hash
assigned before HOLD, not a false-PASS test, and not a false record claim.
It is a real, reproduced UI defect with a missing case (no path on this screen
ever shows the operator what was typed, not only at resumption) — squarely
Important under this task's rubric, matching the original report's own
classification.

One scope note, not a severity correction: the defect is not *caused* by
interruption — a phrase typed continuously with no Back and no screensaver is
equally invisible, confirmed above (`TestZZReadoutBudgetShippedLead` never
drove a Back or an idle wakeup). The reviewer's own text already frames it this
way ("This is an interruption finding because..." — the phrase intact/no-readout
combination is what the interruption lens makes visible, not something the
interruption causes). Nothing here changes the finding's severity or scope; it
is stated only so a reader does not read "interruption finding" as "interruption-
caused."

## Closing counts

0 Critical, 1 Important (confirmed as reported — I-1), 0 Minor, 0 Nit from this
refute pass. No new defects found; no claim in the original report was refuted.
