# H3 records draft — the spec sentences

**Item:** the two spec files whose sentences H2 made false, folded at H3 (records only).
**Branch:** `h3-composer-spec` (engrave), one commit `2627c4b`, branched from `master`.
**Worktree:** `/scratch/code/shibboleth/me-worktrees/h3-composer-spec`.
**Files edited:** `design/SPEC_wallet_policy_composer.md`, `design/SPEC_hashlock_H2_device.md`.
**Not edited:** the H2 plan, `design/FOLLOWUPS.md`, any fork file. Nothing pushed.

---

## 0. Two drifts from the brief, up front

Both were found by measurement, not assumed, and both change what a reader should
check.

1. **Engrave `master` is at `68aae89`, not `d81714e`.** The brief names `d81714e`;
   `git merge-base --is-ancestor d81714e HEAD` returns 0, so `d81714e` is an
   ancestor and `master` has moved two commits past it (`49b03eb` reports, `68aae89`
   continuity). The worktree was created from `master` as instructed, so it is
   based on `68aae89`.

2. **The fork branch `hashlock-h2` is at `a1fd139`, not `17b3979`.** `17b3979` is
   an ancestor; two commits sit on top of it:

   ```
   $ git -C /scratch/code/shibboleth/.tmp/seedhammer-hashlock-h2 log --oneline 17b3979..HEAD
   a1fd139 gui: count-free other-path line in the hashlock confirm modal; Remove path re-syncs hashByPhrase (hashlock H2 ultracode-lens fold)
   26fd1dd gui+hashlock: assert the confirm modal's first8..last8 and chars tokens; draw the phrase screen's readout (F-481); fail closed on a dead Deriver; ASCII-only case fold in IsMS1Shaped; every refusal sentinel has copy (hashlock H2 post-impl fold)
   ```

   This matters for exactly one sentence — see §3. **Every citation below was
   checked at BOTH revisions and holds identically at both**, except
   `composerCopyHashlockOtherPath`'s string and its line numbers.

   Also noted, not acted on: the brief says `design/FOLLOWUPS.md` holds F-475 and
   F-481. At `68aae89` it holds F-475 (`design/FOLLOWUPS.md:15742`); **F-481 is not
   in `FOLLOWUPS.md`** — `grep -rn 'F-481' design/` finds it only in
   `design/agent-reports/hashlock-H2-post-impl-*.md`. FOLLOWUPS is not my item, so
   I filed nothing; flagging it because whoever owns the FOLLOWUPS half of Task 6
   will need to create F-481 rather than edit it.

---

## 1. What changed in `SPEC_wallet_policy_composer.md`

The prescription is `IMPLEMENTATION_PLAN_hashlock_H2_device.md:2941` (Task 6):
*"the composer spec's §6c line 386 and §14 sentence ("never derives … a preimage
this cycle") → owning phase **H3**, with the replacement wording"*. The replacement
wording itself is stated by the H2 spec, in two places that agree:

- `SPEC_hashlock_H2_device.md:34-41` (opening): *"From H2 the composer DERIVES one,
  in RAM, for the length of one screen; it still never stores, shows or engraves
  it."*
- `SPEC_hashlock_H2_device.md:61-62` (§1 item 5), the fork record's new text:
  *"THE COMPOSER DERIVES A PREIMAGE IN RAM FOR ONE SCREEN (H2) AND NEVER STORES,
  SHOWS OR ENGRAVES IT. It puts a digest in a script."*

### 1a. §6c, line 386

```
-The composer never derives, stores or engraves a preimage this cycle (§14). When
+From H2 the composer derives a preimage in RAM for the length of one screen and
+never stores, shows or engraves it; it puts a digest in a script (§14). When
 every path of the policy carries a hash, the §8h warning fires before consent.
```

**The replacement sentence begins on line 386, the same line the superseded one
occupied** (`grep -n 'From H2 the composer derives'` → `386`). That was checked
after every edit and after the rewrap, because two other documents cite
`SPEC_wallet_policy_composer.md:386` by line number —
`SPEC_hashlock_H2_device.md:35` and its §10 citation table at
`SPEC_hashlock_H2_device.md:506`. Both remain accurate; neither was touched.

### 1b. The one-line provenance (wrapped to the file's ~78-column majority)

```
(H2, fork `hashlock-h2` `a1fd139`, the leg reviewed at `17b3979`:
`SPEC_hashlock_H2_device.md` §4.1 adds a `Type a hashlock phrase` row ahead of
`Type 64 hex`; the phrase route derives X on the stack and drops it when the
route returns (`gui/composer_hashlock.go:19-20`), assigning only
`hashlock.Digest(&x)` to `Paths[idx].Hash` (`gui/composer_hashlock.go:64-69`).
The fork's own record is `gui/composer_hash.go:27-28`.)
```

The `§4.1` clause is there for a reason: §6c as written names two entry routes
("Primary: pick from the payload's `hash:` records … Fallback: type 64 hex"), and
after H2 there are three. A replacement sentence saying the composer *derives* a
preimage, in a section that never mentions a phrase, would read as a non-sequitur.
The clause is a cross-reference, not a new normative rule — **§6c's own row set is
still stale and is named as residue in §5 below.**

### 1c. §14's row

```
-| on-device preimage derivation, storage or engraving | C25; §6c |
+| on-device preimage storage, display or engraving | C25; §6c. Derivation is no longer out of scope: from H2 the device derives one in RAM for the length of one screen (`SPEC_hashlock_H2_device.md` §1 item 5) |
```

Storage, display and engraving stay out of scope — true of the branch. Derivation
does not, and the cell now says which document owns it.

---

## 2. What changed in `SPEC_hashlock_H2_device.md`

The prescription is the plan's `## R0 round 0 folded here`, committed at **`f60c2df`**
(`git log -S'## R0 round 0 folded here' -- design/IMPLEMENTATION_PLAN_hashlock_H2_device.md`;
the message ends *"2 spec departures recorded as H3 items"*). Both replacement
sentences were applied to the sections the plan names.

### 2a. Departure one — §4.5's drop order, last clause (plan's wording, verbatim)

```
-  move the reconciliation line into the phrase-route §8h at Done (§4.7). The
-  backup line and the relation line are never dropped.
+  move the reconciliation line out of the confirm modal and onto its own
+  dismissible screen shown immediately after HOLD, where it is reachable for
+  every policy that has a phrase-set hash — NOT into the phrase-route §8h at
+  Done, whose `composerEveryPathHashed` guard is false for any policy with one
+  un-hashed path. The backup line and the relation line are never dropped.
```

### 2b. Departure two — §4.5's line list gains the other-path line

In the code block, after `<relation line, …>`:

```
<other-path line, only when another path of this policy already carries a different
hash: "another path has a different hash: back up every phrase">
```

And as a bullet, after the relation-line bullet — the plan's wording, verbatim:

```
- The other-path line (journey I-1): when any OTHER path of the same policy
  already carries a `Hash` that differs from this digest, the modal says so,
  because `md.ValidatePathList` has no clause about two paths' `Hash` values and
  "One phrase per policy" is advice — a second phrase is legal, and a second
  backup burden the operator must choose knowingly. Omitted when no other path
  carries a hash, or when the hashes are equal.
```

### 2c. New `## H3 fold` paragraph

Appended at the end of the file, citing plan commit `f60c2df`, naming both
departures with their code citations, recording the one substitution (§3), and
stating that the composer spec's two sentences are folded in the same commit.

### 2d. Two sentences this fold falsified in its own file

Not in the brief, and folded anyway because the commit that changes the composer
spec is the commit that makes them false — *a diff falsifies text it never touches*:

- The opening paragraph said the composer spec's two sentences *"stay as they are
  until then"*. False the moment §1 landed. Now: *"they are named here so H3 cannot
  miss them (fidelity I-5; r1 NF-C), and H3 has now folded both (`## H3 fold`)."*
- The same paragraph said all three records *"say"* (present tense) the superseded
  sentence. All three are now rewritten — the fork comment at H2, both spec
  sentences here. Now: *"SAID … ; all three are now rewritten."* The quotation
  itself is kept: it is the record of what the stage found.

`SPEC_hashlock_H2_device.md:62` (*"The composer spec's §6c/§14 sentences are folded
by H3, not here"*) was **left alone — it is still true**: it scopes the H2
*implementation*, and "here" is H2, not this commit.
`SPEC_hashlock_H2_device.md:530-531` was left alone: it is inside
`## R0 round 1 folded here`, a historical record of what round 1 did.

---

## 3. The one deliberate deviation from the plan's exact wording

**The plan prescribed a device string that the branch tip has superseded.** Plan
text (`## R0 round 0 folded here`, "H3 record item, second"):

> `<other-path line, only when another path of this policy already carries a different
> hash: "another path has a different hash: two phrases to back up">`

Measured at both revisions:

```
$ git show 17b3979:gui/composer_copy.go | grep -n -A2 'func composerCopyHashlockOtherPath'
451:func composerCopyHashlockOtherPath() string {
452-	return "another path has a different hash: two phrases to back up"
453-}

$ grep -n -A2 'func composerCopyHashlockOtherPath' gui/composer_copy.go   # at a1fd139
454:func composerCopyHashlockOtherPath() string {
455-	return "another path has a different hash: back up every phrase"
456-}
```

`a1fd139` ("count-free other-path line in the hashlock confirm modal") replaced it.
Writing the plan's quote into §4.5 would put a string in the spec that exists
nowhere in the tree the branch will merge — a records defect created by the very
commit meant to close one. **So §4.5 quotes the live string**, and the `## H3 fold`
paragraph records the plan's original verbatim, the reason, and
`gui/composer_copy.go:454-456`, so `git diff <plan>..<spec>` still resolves.

Everything else in both departures is the plan's wording, byte for byte. This is
the only substitution.

---

## 4. Every device claim, grepped

Run against `/scratch/code/shibboleth/.tmp/seedhammer-hashlock-h2` (read-only,
HEAD `a1fd139`), each also via `git show 17b3979:<file>`. **All identical at both
revisions** unless noted.

| claim written into a spec | citation | verified |
| --- | --- | --- |
| the fork record was rewritten | `gui/composer_hash.go:27-28` — `THE COMPOSER DERIVES A PREIMAGE IN RAM FOR ONE SCREEN (H2) AND NEVER STORES, / SHOWS OR ENGRAVES IT. It puts a digest in a script.` | same at `17b3979` and `a1fd139`; at baseline `c4a64fc` the same two lines read `THE COMPOSER NEVER DERIVES, STORES OR ENGRAVES A PREIMAGE this cycle / (§14).` |
| the preimage is not retained | `gui/composer_hashlock.go:19-20` — `// The preimage lives on the stack / // here and is dropped when this function returns (L7, L15).` | both |
| only the digest is assigned | `gui/composer_hashlock.go:64-69` — `h := hashlock.Digest(&x)` … `st.list.Paths[idx].Hash = &d` | both |
| the preimage is never shown | the confirm body takes `hashlockFirst8Last8(h)` and `len(phrase)` — a digest and a count, never X (`gui/composer_hashlock.go:65-66`) | both |
| §4.1's row order | `gui/composer_hash.go:139` (`composerHashRowPhrase = "Type a hashlock phrase"`), appended at `:165`, then `"Type 64 hex"` at `:167`, `"No hash lock"` at `:169` | both |
| relation then other-path, in that order | `gui/composer_copy.go:409-417` — `if relation != ""` then `if otherPath != ""` | both |
| §8h's guard | `composerEveryPathHashed` at `gui/composer_state.go:244`; `:239` at `c4a64fc` | both |
| `ValidatePathList` has no two-path `Hash` clause | `md/compose.go:299` (func), and `awk '/^func ValidatePathList/,/^}/' md/compose.go \| grep -n 'Hash'` returns exactly one line: `p.Hash == nil` at `md/compose.go:315` | both |

The last one is the claim I was least willing to take from the plan, since the
bullet asserts it as the *reason* the line exists. One `Hash` reference in the
whole function, and it is a nil check on the current path — the plan's claim is
true.

---

## 5. Gate — superseded phrasing

Run in the worktree after the last edit, before the commit:

```
$ for pat in 'never derives, stores or engraves' 'never derives' \
             'on-device preimage derivation' 'reconciliation line into the phrase-route' \
             'stay as they are until then' 'two phrases to back up'; do
    grep -n "$pat" design/SPEC_wallet_policy_composer.md design/SPEC_hashlock_H2_device.md
  done

--- never derives, stores or engraves ---
0 hits
--- never derives ---
design/SPEC_hashlock_H2_device.md:36:`gui/composer_hash.go:27-28` SAID the composer "never derives, stores or
--- on-device preimage derivation ---
0 hits
--- reconciliation line into the phrase-route ---
0 hits
--- stay as they are until then ---
0 hits
--- two phrases to back up ---
0 hits
```

**Read the first two rows together.** `'never derives, stores or engraves'` returns
0 hits only because the rewrap in §2d split the quotation across lines 36-37
(`never derives, stores or` / `engraves a preimage this cycle`). The phrase has NOT
left the file; the broader `'never derives'` pattern is the honest one, and its
single hit is `SPEC_hashlock_H2_device.md:36` — the deliberate past-tense quotation
inside `**One sentence this stage makes false, in two places:**`, which the same
commit changed to read `SAID … ; all three are now rewritten`. That is the
historical carve-out the gate allows. Zero hits anywhere else, in either file.

Commit verification:

```
$ git log --format='%B' -1 | tail -3
Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA

$ git status --porcelain     # empty
$ git rev-parse HEAD
2627c4b465b78c663acb993611d2c6c08ae16a9e
$ git diff --stat HEAD~1
 design/SPEC_hashlock_H2_device.md     | 64 ++++++++++++++++++++++++++++++-----
 design/SPEC_wallet_policy_composer.md |  7 ++--
 2 files changed, 66 insertions(+), 10 deletions(-)
```

No Rust or Go gate was run: this commit contains no executable content — two
markdown files, zero code blocks added, zero cited API signatures beyond the
`file:line` table in §4, all of which were resolved by `grep`/`sed` against the
fork rather than compiled.

---

## 6. Residue for whoever owns the rest of H3

Named, not fixed — each is outside this item.

1. **§6c's row set is still stale.** It reads "Primary: pick from the payload's
   `hash:` records … Fallback: type 64 hex", and the device now offers three rows
   (`gui/composer_hash.go:165,167,169`). My provenance line cross-references
   `SPEC_hashlock_H2_device.md` §4.1 rather than rewriting §6c's structure, because
   the item scoped me to the sentences. A future H3 pass should fold the row set
   itself.
2. **`SPEC_hashlock_H2_device.md:506`** (§10 citation table) still reads *"the
   composer spec sentence H3 folds: `SPEC_wallet_policy_composer.md:386`"* —
   future tense, now done. The line number is still correct, so it misleads nobody;
   left alone to keep the diff to the sentences.
3. **F-481 does not exist in `design/FOLLOWUPS.md`** at `68aae89` (see §0). The
   FOLLOWUPS half of Task 6 will be creating it, not editing it.
4. **§4.5's code block still shows the unshortened reuse block and the
   reconciliation line inside the modal**, with the drop order beneath it. The
   shipped body applies both drop-order steps
   (`gui/composer_copy.go:418-422` — shortened reuse block, no reconciliation
   line). This is internally consistent as written (a pre-drop body plus a drop
   order), and the plan prescribed a change to the drop order's *destination*
   only, so I changed only that. Whether §4.5 should instead show the *shipped*
   body is an editorial call for the spec's owner.
