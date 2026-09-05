# H3 records — the toolkit manual's device section for the hashlock phrase

Agent report. Written as the final action of the drafting agent; the controller
has not edited it.

- **Repo:** `/scratch/code/shibboleth/mnemonic-toolkit`
- **Branch:** `h3-hashlock-device-manual` (from `master` `6cf3ecd8`)
- **Worktree:** `/scratch/code/shibboleth/tk-worktrees/h3-hashlock-device-manual`
- **Commit:** `2c5f31cddc3da1442bfa596ce4e4f76c0e473a51`
- **Pushed:** no. **Files touched:** two, both named by the item.
- **Device source of truth:** fork branch `hashlock-h2` at **`17b3979`**, read
  via `git show 17b3979:<path>` in the read-only worktree
  `/scratch/code/shibboleth/.tmp/seedhammer-hashlock-h2`.

---

## 1. Read this first — the pinned commit is two commits behind the branch tip,
and one of the two commits falsifies the F-481 paragraph the item asked for

The read-only worktree's HEAD is **`a1fd139`**, not `17b3979`:

```
$ git -C /scratch/code/shibboleth/.tmp/seedhammer-hashlock-h2 log --oneline -3
a1fd139 gui: count-free other-path line in the hashlock confirm modal; Remove path re-syncs hashByPhrase (hashlock H2 ultracode-lens fold)
26fd1dd gui+hashlock: assert the confirm modal's first8..last8 and chars tokens; draw the phrase screen's readout (F-481); fail closed on a dead Deriver; ASCII-only case fold in IsMS1Shaped; every refusal sentinel has copy (hashlock H2 post-impl fold)
17b3979 seal+gui: name the record the allow-list refused instead of "Payload unreadable." (F-474, hashlock H2)
```

The brief pins `17b3979` and says every sentence must be true there, so the
section is written against `17b3979`. Two of the five non-test changes between
`17b3979` and `a1fd139` reach user-visible behaviour:

```
$ git -C <fork worktree> diff --stat 17b3979..a1fd139
 gui/composer_copy.go          |   7 ++-
 gui/composer_copy_test.go     |   4 +-
 gui/composer_hashlock.go      |   4 +-
 gui/composer_hashlock_test.go | 106 ++++++++++++++++++++++++++++++++++++++++--
 gui/composer_shape.go         |   3 ++
 hashlock/hashlock.go          |  22 +++++++--
```

### 1a. F-481 is CLOSED at `26fd1dd`. The manual's "Known limitation" heading is true at `17b3979` and FALSE at the tip.

`26fd1dd` removes the `content, _ = content.CutBottom(8)` line at
`gui/composer_hashlock.go:166` (the line is present at `17b3979`,
replaced by a three-line comment at `a1fd139`), which is exactly the F-481
mechanism. The measurement is already persisted in this repo, not re-derived
here: `design/agent-reports/hashlock-H2-post-impl-r1-fold-verification.md:61-100`
records the mutation (restoring the cut → `the phrase screen drew 0 asterisks
for 10 typed characters`, `TestHashlockPhraseScreenDrawsTheMaskedReadout` FAILs)
and the geometry (pre-fix `avail` **11 px**, post-fix **19 px**, one masked line
**19 px**).

So the `#### Known limitation: the phrase screen shows no readout` subsection is
correct for `17b3979` **and wrong for the branch as it now stands.** It is
deliberately self-contained — one `####` heading and one paragraph, nothing else
in the section depends on it — so it is a single-hunk delete or replace.

**Ready replacement, if the branch merges at `26fd1dd` or later** (drop the
heading and paragraph and substitute this, keeping it as a `####`):

> #### The readout, and how much it shows
>
> The keyboard's readout line above the keys is masked by default — one `*` per
> character — and the `show` key toggles it to cleartext and back. Its height
> budget on this screen is one line with no margin to spare, so a long phrase is
> clamped to its **tail**: the characters you just typed stay visible and the
> head scrolls out of sight. The `n/100` counter, not the readout, is what
> reports the true length, and the confirm screen's `chars:` count is the length
> the device will actually hash.

That replacement is *not* verified against `26fd1dd` by me — I read only
`17b3979` per the brief. It is drafted from the clamp logic at
`gui/passphrase_keyboard.go:431-473` (unchanged between the two commits) and
must be re-checked before it ships.

### 1b. The confirm modal's "other path" line changed wording, so I described it without quoting it.

At `17b3979`, `composerCopyHashlockOtherPath` (`gui/composer_copy.go:451-453`)
returns `another path has a different hash: two phrases to back up`; at
`a1fd139` it returns `another path has a different hash: back up every phrase`.
The item did not list this line among the confirm-modal contents. Rather than
omit it (a reader would meet an undocumented line) or quote a string that has
just changed, the section says only that *"a further line follows it when
another path of the same policy already carries a different hash"* — true and
stable at both commits. **Follow-up for whoever folds this:** once the branch
settles, quote it.

### 1c. The item's "~30 s countdown" is wrong; the device says about 10 s.

The item's wording is *"the hardened PBKDF2 method with its ~30 s countdown"*.
The branch says ten, in three places, and the section follows the branch:

- `hashlock/hashlock.go:23-24` — `// Iterations is HASHLOCK_ITERATIONS; about 10 s on the SH2 (9,715 it/s measured).` / `const Iterations = 100000`
- `gui/composer_hashlock.go:189-190` — the pick row `"Hardened (about 10 s)"`
- `gui/composer_copy.go:402-404` — `"Deriving. This takes about 10 seconds."`

100,000 ÷ 9,715 = 10.3 s. Nothing on the branch mentions 30 s.

### 1d. The manual has no device-side sections to imitate — the item's premise did not hold.

The item said to read *"how the composer or payload chapters describe SeedHammer
screens."* There are none:

```
$ grep -rin "seedhammer" src/ | wc -l
0
```

Zero occurrences across all 40 source files before this change. There is no
composer chapter and no payload chapter. Placement was therefore a judgement
call — see §2.

---

## 2. What changed, and why there

Two files, both in `docs/manual/`:

| File | Change |
| --- | --- |
| `src/40-cli-reference/43-ms.md` | +157 lines: a new `### On the SeedHammer II {#hashlock-on-the-seedhammer}` subsection inside `## ms hashlock`, inserted after that section's `### Worked example` and before the `---` that opens `## ms vectors` |
| `.cspell.json` | +4 words |

**Why inside `## ms hashlock` rather than a new chapter.** The device and
`ms hashlock` compute the same digest from the same phrase with the same
parameters, and the device's own post-HOLD screen instructs the operator to
*"run ms hashlock with this phrase and method on the host and check the digest
matches."* A reader sent there by the machine looks up `ms hashlock`; putting
the device half anywhere else means the reconcile instruction lands in a chapter
that does not mention the device. It also inherits the chapter's existing worked
example — the same phrase, the same digest — so the cross-check is concrete
rather than described. The alternative considered and rejected was a new
`src/30-workflows/3B-*.md` chapter: the content is reference (screens, refusals,
a Back contract), not a procedure, and a workflows chapter would duplicate the
`ms hashlock` chapter's phrase rule to stand alone.

The section is `###` with five `####` subsections (`#### ` is used 44 times
elsewhere in `src/`, so the depth is not novel), and `--toc-depth=3` keeps the
`####` headings out of the table of contents.

---

## 3. Every device claim, with its citation at `17b3979`

All paths below are in the fork worktree at commit `17b3979`. Line numbers are
from `git show 17b3979:<path>`, not from the worktree's `a1fd139` checkout.

| Sentence in the manual | Citation |
| --- | --- |
| `Path N hash` is the screen title | `gui/composer_hash.go:203` — `title := fmt.Sprintf("Path %d hash", idx+1)` |
| rows: one per payload `hash:` record, then `Type a hashlock phrase`, `Type 64 hex`, `No hash lock` | `gui/composer_hash.go:139` (`composerHashRowPhrase`), `:160-169` |
| the lead is `Which hash?` | `gui/composer_hash.go:163` |
| …or the no-payload lead when the payload holds none | `gui/composer_hash.go:171-173` → `gui/composer_copy.go:357-360` |
| the no-payload lead text | `gui/composer_copy.go:357-360`, verbatim |
| the rule modal fires on any row that *takes* a hash (payload row, phrase row, hex row) | `gui/composer_hash.go:210-215` — `taking := sel < len(rows.digests) \|\| sel == rows.phraseRow \|\| sel == rows.hexRow` |
| the rule modal text | `gui/composer_copy.go:179-183`, verbatim |
| phrase screen title `Hashlock phrase` | `gui/composer_hashlock.go:182`; the refusal modal reuses it at `:158` |
| the phrase-screen lead | `gui/composer_copy.go:367-370`, verbatim |
| an `n/100` counter | `gui/composer_hashlock.go:171-172` — `widget.Labelf(... "%d/%d", len(kbd.Fragment), hashlock.PhraseMaxChars)`; `hashlock/hashlock.go:29-31` — `PhraseMaxChars = 100` |
| the counter is unclamped (`101/100` and beyond) | no length guard exists: `PassphraseKeyboard.commit`'s `ppRune` arm is `k.Fragment += string(key.r)` with no cap (`gui/passphrase_keyboard.go:261-264`); the only refusal is on OK |
| a four-page keyboard whose function row is page-cycle, `space`, `show`, backspace | `gui/passphrase_keyboard.go:17-26` (pages), `:133-138` (function row); the flow builds it with `NewPassphraseKeyboard(ctx)` (`gui/composer_hashlock.go:142`), i.e. `newPPKeyboard(ctx, false, false)` — **no** newline key and **no** gear key (`:76-78`) |
| the pages type **exactly** the 95 characters `0x20..=0x7E` | machine-checked, see §4 |
| nothing trims, case-folds, collapses spaces or normalises | `hashlock/hashlock.go:80-82` doc comment and body: `ValidatePhrase` reads the raw bytes; `gui/composer_hashlock.go:156` passes `[]byte(kbd.Fragment)` unmodified; `:138-140` — "NOTHING normalises the bytes" |
| the five refusals, in the host's order | `hashlock/hashlock.go:83-102` |
| the five refusal strings | `gui/composer_copy.go:372-387`, verbatim, one row per `hashlock.Err*` sentinel |
| the ms1 test runs **before** the length cap | `hashlock/hashlock.go:92-97` (ms1 at :92, cap at :95) — the host is identical: `mnemonic-secret/crates/ms-cli/src/hashlock_phrase.rs:118-142`, whose own doc comment reads *"Order matters and is the spec's: empty, printable ASCII, ms1-shape (BEFORE the cap), cap, 64-hex."* |
| the printable-ASCII refusal is unreachable from this keyboard | consequence of the 95-character union in §4 |
| a dismissed refusal returns to the screen with the phrase intact | `gui/composer_hashlock.go:157-160` — `showError(...)` then `continue`; `kbd.Fragment` is never cleared |
| `Hashlock method`, lead `Which method?`, rows `Hardened (about 10 s)` and `SHA-256` | `gui/composer_hashlock.go:188-198` |
| SHA-256 warns every time; hardened only under 20 characters | `gui/composer_hashlock.go:202-212` — `case hashlockSHA256:` unconditional; `case hashlockHardened: if len(phrase) < 20` |
| the two warning texts | `gui/composer_copy.go:389-400`, verbatim |
| `Hold button to confirm.` is appended to a confirm body | `gui/composer_copy.go:36-38` |
| Back declines a confirm | `gui/composer_shape.go:77-91` — `case ConfirmNo: return false` |
| SHA-256 is instant | `gui/composer_hashlock.go:240-242` |
| the `Deriving` screen, its percentage and its two leads | `gui/composer_hashlock.go:248` (title), `:250-254` (percent), `:228-234` (`hashlockDerivingLead`: zero state → `composerCopyHashlockDerivingLead()`, then `About %d seconds left.`); zero-state text at `gui/composer_copy.go:402-404` |
| about ten seconds — 100,000 iterations at a measured 9,715/s | `hashlock/hashlock.go:23-24`; restated at `gui/composer_hashlock.go:277` |
| the screen holds the display awake | `gui/composer_hashlock.go:281` — `ctx.KeepAwake()` inside the frame callback |
| its discard button abandons with nothing assigned | `gui/composer_hashlock.go:243`, `:258` (`assets.IconDiscard`), `:294-297`, `:301-303`; caller `continue`s at `:60-63` |
| the confirm modal is titled `Hash lock` | `gui/composer_hashlock.go:67` |
| the digest is abbreviated to first-8 `..` last-8 | `gui/composer_hashlock.go:131-134` — `s[:8] + ".." + s[len(s)-8:]` |
| the modal's line order and its two paragraphs | `gui/composer_copy.go:409-424`, verbatim, including `"hash  "` (two spaces) and `"method: %s   chars: %d"` (three spaces) |
| the relation line, when the payload holds records | `gui/composer_hashlock.go:96-108`; text at `gui/composer_copy.go:425-431` |
| a further line when another path carries a different hash | `gui/composer_hashlock.go:119-129` (predicate only — the text is deliberately not quoted, see §1b) |
| the reconcile screen is drawn for every policy that gets a phrase-set hash | `gui/composer_hashlock.go:67-83` — unconditional `showError(ctx, th, "Hash lock", composerCopyHashlockReconcile())` after the confirm returns true |
| its text | `gui/composer_copy.go:443-447`, verbatim |
| Back at the phrase screen → `Path N hash`, phrase dropped | `gui/composer_hashlock.go:46-48`, `:152-153` (`return nil, false`) |
| Back at the method pick → phrase screen, phrase kept | `gui/composer_hashlock.go:51-56` (`break pick`) + `:46` (`hashlockPhraseFlow(ctx, th, phrase)`) and `:143` (`kbd.Fragment = string(initial)`) |
| declining a warning → method pick, phrase kept | `gui/composer_hashlock.go:57-59` (`continue`) |
| Back while deriving → method pick, phrase kept | `gui/composer_hashlock.go:60-63` (`continue`) |
| Back on the confirm → method pick, nothing assigned | `gui/composer_hashlock.go:67`, `:84-86` |
| Back at `Path N hash` leaves the editor with no hash set | `gui/composer_hash.go:206-209` — *"the ONLY false this function returns (spec §4.6)"* |
| …and discards the path if it was being created | `gui/composer_shape.go:269-272` — `if !composerHashEdit(...) { st.list.Paths = st.list.Paths[:idx]; return }` |
| F-481: no readout, the counter is the only feedback | `gui/composer_hashlock.go:166` (`content, _ = content.CutBottom(8)`) driving the clamp at `gui/passphrase_keyboard.go:454-473`; measured in `design/agent-reports/hashlock-H2-post-impl-r1-fold-verification.md:61-100` — **and closed at `26fd1dd`, see §1a** |

---

## 4. Machine-checked, not asserted

**(a) The keyboard types exactly printable ASCII.** The manual claims "exactly
the 95 printable-ASCII characters `0x20..=0x7E` and nothing else". Computed from
the four page constants at `gui/passphrase_keyboard.go:19-26` plus the space key
(`{r: ' ', label: "space", action: ppRune}`, `:135`):

```
$ python3 -c "<union of ppPageLower+ppPageUpper+ppPageSymbols+ppPageSymbols2+' '>"
union size 95
missing from keyboard: []
extra beyond printable ASCII: []
```

**(b) The worked digest.** The section prints `hash  3cf5d421..b70a4c12` /
`method: hardened   chars: 28` for `correct horse battery staple`. Recomputed
from the parameters, independent of both the chapter and the fork:

```
$ python3 -c "import hashlib; p=b'correct horse battery staple';
  x=hashlib.pbkdf2_hmac('sha256',p,b'ms-hashlock-v1',100000,32); ..."
len(phrase)= 28
hardened_h= 3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
abbrev= 3cf5d421..b70a4c12
sha256_h  = b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb
```

Three-way agreement: this recomputation, the chapter's existing worked example
(`43-ms.md:368`, `hash:3cf5d421…b70a4c12`), and the fork's vendored corpus
`hashlock/testdata/hashlock-v0.8.json` → `derivation[0]` (`"phrase": "correct
horse battery staple"`, `"phrase_chars": 28`, `"hardened_h":
"3cf5d421…b70a4c12"`). The `sha256_h` also matches `43-ms.md:370`.

---

## 5. Gate output

Run exactly as `87e594e0`'s message describes (`make -C docs/manual lint` with
the CLI binaries overridden), from the branch worktree. The four binaries are
the ones already built on this box; `ms` is **0.18.0**, the version the chapter
documents:

```
MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic    (mnemonic 0.97.0)
MD_BIN=/scratch/code/shibboleth/descriptor-mnemonic/target/release/md           (md 0.14.0)
MS_BIN=/scratch/code/shibboleth/mnemonic-secret/target/debug/ms                 (ms 0.18.0)
MK_BIN=/scratch/code/shibboleth/mnemonic-key/target/release/mk                  (mk 0.13.0)
```

Final run, after the commit's content was in place:

```
  10 mermaid block(s) across 40 source file(s)

[lint] === 1/6 markdownlint ===
markdownlint-cli2 v0.13.0 (markdownlint v0.34.0)
Linting: 40 file(s)
Summary: 0 error(s)

[lint] === 2/6 cspell ===
CSpell: Files checked: 40, Issues found: 0 in 0 files.

[lint] === 3/6 lychee ===
🔍 297 Total (in 5ms) 🔗 164 Unique ✅ 274 OK 🚫 0 Errors 👻 23 Excluded

[lint] === 4/6 flag-coverage ===

[lint] === 5/6 glossary-coverage ===

[lint] === 6/6 index bidirectional ===

[lint] OK
```

The same command on the unmodified tree (before any edit) also printed
`[lint] OK` with the same counts except `297 Total / 164 Unique`, i.e. this
change adds no links and breaks none.

**`make anchor-check` (not part of `lint`; `make audit` runs it) is red, and was
already red.** Measured both ways — with the change, and with it stashed:

```
::error::anchor-check: baseline shrunk — slug '#auto-fire-behavior-all-three-subcommands-v0250' no longer dangles; ...
::error::anchor-check: baseline shrunk — slug '#auto-fire-on-decode-failure-v0221' ...
::error::anchor-check: baseline shrunk — slug '#bitcoin-core-receive-change-hard-fail' ...
::error::anchor-check: baseline shrunk — slug '#exporting-to-bitcoin-core-bip-388-sparrow-specter' ...
::error::anchor-check: baseline shrunk — slug '#manual-coverage' ...
::error::anchor-check: baseline shrunk — slug '#mnemonic-bundle' ...
::error::anchor-check: baseline shrunk — slug '#mnemonic-export-wallet' ...
::error::anchor-check: baseline shrunk — slug '#mnemonic-import-wallet' ...
::error::anchor-check: baseline shrunk — slug '#mnemonic-restore' ...
::error::anchor-check: baseline shrunk — slug '#multisig-cosigner-restore' ...
make: *** [Makefile:266: anchor-check] Error 1
```

Identical ten-line set in both runs — the ratchet is behind on `master`, and
nothing here touches it. `make html` succeeds and both new anchors render
(`id="hashlock-on-the-seedhammer"`, and the auto-slug for the limitation
heading).

---

## 6. cspell has a blind spot, and it is why the four new words were found at all

`.cspell.json`'s `ignoreRegExpList` contains `` `[^`]+` ``. Applied to the whole
file, a fenced code block's opening ``` leaves a third backtick that starts a
match running to the **closing** fence's first backtick, and the closing fence
then leaves a backtick that starts a match running to the next backtick anywhere
in the file — swallowing the prose in between. Demonstrated:

```
$ cspell --no-progress "src/40-cli-reference/43-ms.md"
CSpell: Files checked: 1, Issues found: 0 in 0 files.

$ # the same sentence, verbatim from 43-ms.md:352, in a fresh file:
$ cspell --no-progress "src/40-cli-reference/zz-probe.md"
src/40-cli-reference/zz-probe.md:3:10 - Unknown word (brainwallet)
```

`brainwallet` has been in this chapter three times since `87e594e0`
(`43-ms.md:335,352,377`) and has never been spell-checked. So the whole-file run
cannot be trusted to gate new prose. I checked the new section by copying it
into a standalone file under `src/40-cli-reference/` and running cspell on that
alone (probe deleted afterwards; `git status` clean). It found four:

```
zz-probe.md:2:44  - Unknown word (seedhammer)
zz-probe.md:45:41 - Unknown word (normalises)
zz-probe.md:75:15 - Unknown word (brainwallet)
zz-probe.md:76:73 - Unknown word (diceware)
```

All four are real terms, so all four were added to `.cspell.json` (the item's
condition): `seedhammer` (the product name, lowercase so it covers the anchor
slug too), `brainwallet` (already in use in this chapter, and in the device's
own SHA-256 warning), `diceware` (quoted verbatim from that warning) and
`normalises` (the house spelling — the list already carries `minimises`,
`standardises`, `serialisation`). Re-probe after the addition:
`Issues found: 0 in 0 files.`

**Follow-up worth filing (owning phase: H3 or later, toolkit repo):** narrow the
inline-code ignore regex so it cannot span a fence — e.g. anchor it to a single
line — and re-run cspell over `src/` to see what else has been unchecked.

---

## 7. What I did not do

- **Did not push**, did not touch `master`, did not commit in the engrave repo.
- **Did not add an `\index{}` marker.** The index gate is bidirectional and
  `69-index-table.md` is not a file this item names, so a marker here would have
  required editing it too.
- **Did not run `make verify-examples`.** It replays transcripts against the
  CLIs; the new section adds no transcript and no `include=` fence, and the one
  `ms hashlock` command it names is already covered by the chapter's existing
  worked example.
- **Did not re-measure the F-481 pixel budget myself.** The numbers in §1a are
  quoted from the persisted verification report in this repo, attributed there.
- **Did not read any `.jsonl`.**

---

## 8. Follow-ups this drafting surfaced

| # | Item | Owner |
| --- | --- | --- |
| 1 | The `#### Known limitation` subsection is false at `26fd1dd`+. Replace it (draft wording in §1a) or delete it before this branch merges. | **blocking on the H3 fold** |
| 2 | Quote the confirm modal's other-path line once the branch settles (§1b). | H3 |
| 3 | The item's "~30 s countdown" figure is wrong; the device says ~10 s. Correct it wherever else it was written down. | H3 |
| 4 | cspell's inline-code ignore regex spans code fences; `brainwallet` has never been checked in `43-ms.md` (§6). | toolkit |
| 5 | `make anchor-check` is red on `master` with 10 stale baseline lines (§5) — unrelated to this work, but `make audit` cannot pass until it is ratcheted. | toolkit |
