# Composer journey follow-ons — independent adversarial execution review

**Range reviewed:** `git diff 6728c22..1dab84a` in `/scratch/code/shibboleth/seedhammer`
(branch `composer-review-names-script`) — exactly two commits:

- `c4d8527` the Review names the script, in the picker's words (journey I-7)
- `1dab84a` the changed-id banner compares the IDS (journey I-5)

`6728c22` (the script picker's preselect, journey C-1) was out of scope and treated as given.

**Method:** a detached scratch worktree at `1dab84a`; Go 1.26.7 from
`/scratch/code/shibboleth/.toolchain/go/bin`; `TMPDIR=/scratch/code/shibboleth/.tmp`.
Every probe file was added, run, and deleted; every mutation was reverted; both
worktrees removed. `git status --porcelain` is empty in the fork and both
worktrees were verified clean before removal.

**THE ONE QUESTION — answered: YES, twice, and both are test-gate failures rather
than code failures.** Neither `composerScriptLine` nor `composerIdChanged` is
itself wrong on any input the composer can reach today (measured, below). What
is wrong is that **the two tests these commits add cannot fail on the guarantees
they are named for**, so both fixes are unpinned: a one-token edit to a literal
table puts a false script name on the Review, and a verbatim revert of the id
predicate restores F-520 — each with all 1296 gui tests green.

---

## MID-REVIEW GROUND SHIFT — read this before folding

While this review ran, the branch advanced from `1dab84a` to **`b23f6cf`**
("composer: fold the preselect review -- I-1, M-1, M-3, N-2") — the parallel
agent's fold of the out-of-scope `6728c22` review. It touches
`gui/composer_shape.go`, `gui/composer_gates_test.go`, `gui/freetext_sizeproof_test.go`.

**That fold independently found and closed C-1 and M-2 below.** Its new
`TestWrapperLabelsNameTheirOwnWrapper` (`gui/composer_gates_test.go:1535+`) both
asserts `len(composerWrapperLabels) == len(composerWrapperOrder)` and binds
`labels[i]` to `order[i]` against the encoded template's root. Verified at
`b23f6cf`:

```
$ sed -i 's|^var composerWrapperLabels = \[\]string{"Taproot (tr)", "Segwit (wsh)"|var composerWrapperLabels = []string{"Segwit (wsh)", "Taproot (tr)"|' gui/composer_shape.go
$ go test ./gui/ -run 'TestWrapperLabelsNameTheirOwnWrapper|TestConsentNamesTheScript' -count=1
--- FAIL: TestWrapperLabelsNameTheirOwnWrapper (0.00s)
    composer_gates_test.go:1633: row 0 reads "Segwit (wsh)" but its wrapper encodes tr(...), not the wsh(...) that "wsh" names.
    composer_gates_test.go:1633: row 1 reads "Taproot (tr)" but its wrapper encodes wsh(...), not the tr(...) that "tr" names.
FAIL	seedhammer.com/gui	0.011s
```

**C-2 is NOT closed at `b23f6cf`** — `composer_stub.go` and `composer_stub_test.go`
are untouched by that fold, and the revert mutation is still green there:

```
$ # composerIdChanged reverted to `return !slices.Equal(shown, current)` at b23f6cf
$ go test ./gui/ -run 'TestComposerIdChanged' -count=1
ok  	seedhammer.com/gui	0.003s
```

So: **do not fold C-1 or M-2 a second time.** They are recorded here because they
are real against the range I was given, and because the fact that two independent
reviewers reached the same defect from opposite ends of the branch is itself
evidence about the shared-table design. Everything else below is open at both revisions.

---

## Findings

### C-1 — `TestConsentNamesTheScript` cannot fail on whether the name is TRUE, only on whether a name is PRESENT; swapping two label literals puts a false script on the Review

*(Status: OPEN at `1dab84a`, my reviewed tip. **CLOSED at `b23f6cf`** by the
parallel fold — see the ground-shift section. Recorded for the record and for
`git diff <report>..<fold>` to mean something.)*

The test asserts the expected label by reading `composerWrapperLabels` — the same
table `composerScriptLine` reads:

```go
{md.ComposeWsh, composerWrapperLabels[1]},
```

`composerScriptLine` maps `ScriptWsh -> ComposeWsh`, finds `ComposeWsh` at index 1
of `composerWrapperOrder`, and returns `composerWrapperLabels[1]`. The assertion
is therefore `labels[1] == labels[1]`. The only facts it pins are that
`ComposeWsh` sits at index 1 of *order* and that the root-to-wrapper switch is
right. **It cannot pin which words name which wrapper**, which is the half of the
requirement journey C-1 was about.

**Counterexample — mutation M-B.** Swap `labels[0]` and `labels[1]`; leave
`composerWrapperOrder` untouched. Picker and Review stay perfectly consistent
with each other, and both lie.

```
$ sed: var composerWrapperLabels = []string{"Segwit (wsh)", "Taproot (tr)", "Nested (sh-wsh)", "Legacy (sh)"}
$ go test ./gui/ -run 'TestConsentNamesTheScript' -count=1 -v
=== RUN   TestConsentNamesTheScript
=== RUN   TestConsentNamesTheScript/Taproot_(tr)
=== RUN   TestConsentNamesTheScript/Segwit_(wsh)
=== RUN   TestConsentNamesTheScript/Nested_(sh-wsh)
=== RUN   TestConsentNamesTheScript/Legacy_(sh)
--- PASS: TestConsentNamesTheScript (0.00s)
    --- PASS: TestConsentNamesTheScript/Taproot_(tr) (0.00s)
    --- PASS: TestConsentNamesTheScript/Segwit_(wsh) (0.00s)
    --- PASS: TestConsentNamesTheScript/Nested_(sh-wsh) (0.00s)
    --- PASS: TestConsentNamesTheScript/Legacy_(sh) (0.00s)
```

What the operator sees under that mutation (probe through the real
`composerConsentLinesFor`):

```
PROBE7 | picker row 0 reads "Segwit (wsh)"   -> wrapper 0 | plate Root=4 | Review says "Script: Segwit (wsh)"
PROBE7 | picker row 1 reads "Taproot (tr)"   -> wrapper 1 | plate Root=3 | Review says "Script: Taproot (tr)"
PROBE7 | picker row 2 reads "Nested (sh-wsh)" -> wrapper 2 | plate Root=2 | Review says "Script: Nested (sh-wsh)"
PROBE7 | picker row 3 reads "Legacy (sh)"    -> wrapper 3 | plate Root=2 | Review says "Script: Legacy (sh)"
```

Row 0 reads "Segwit (wsh)", `Root=4` is `ScriptTr`, and the Review — the screen
copied onto steel — confirms "Script: Segwit (wsh)" over a Taproot plate. This is
journey C-1's exact outcome, reinstated, with the test written to prevent it
passing.

**Nothing in the repository catches it.** Full sharded run under M-B:

```
$ bash scripts/gui-shard-test.sh ./gui/ 24
=== wall: 26s ===
RESULT: ok -- all 1296 tests ran across 24 shards
```

**Line violated.** `gui/composer_flow_test.go:504-507`:

> The label is asserted against composerWrapperLabels rather than a literal,
> because "the words the operator chose it with" is the requirement

The requirement has two conjuncts — *the words the operator chose it with* **and**
*which script wrapper the policy uses* (`gui/composer_flow_test.go:490-491`). The
tautological assertion buys the first and forfeits the second. The settled
mutations (`the Review line removed`, `sh-wsh collapsed into sh`) both test
presence and the `InnerWsh` split; neither touches the label-to-wrapper pairing,
which is why the mutation set read as sufficient.

**Remedy (already landed at `b23f6cf`):** assert against an independent ground
truth — the encoded template's root — not against the table under test.

---

### C-2 — `TestComposerIdChangedComparesIdsNotChunks` cannot fail on the change it is named for: a verbatim revert to the pre-fix predicate leaves all 1296 tests green

*(Status: OPEN at `1dab84a` and at `b23f6cf`.)*

Restore `composerIdChanged` to exactly what `6728c22` had:

```go
func composerIdChanged(shown, current []string) bool {
	if len(shown) == 0 {
		return false
	}
	return !slices.Equal(shown, current)
}
```

Every sub-test passes:

```
$ go test ./gui/ -run 'TestComposerIdChangedComparesIdsNotChunks' -count=1 -v
--- PASS: TestComposerIdChangedComparesIdsNotChunks (0.00s)
    --- PASS: TestComposerIdChangedComparesIdsNotChunks/never_shown_one_is_not_a_change (0.00s)
    --- PASS: TestComposerIdChangedComparesIdsNotChunks/same_shape_re-derived_is_not_a_change (0.00s)
    --- PASS: TestComposerIdChangedComparesIdsNotChunks/a_genuinely_different_shape_is_a_change (0.00s)
    --- PASS: TestComposerIdChangedComparesIdsNotChunks/an_unreadable_id_is_reported_as_changed (0.00s)
```

And so does everything else:

```
$ bash scripts/gui-shard-test.sh ./gui/ 24
=== wall: 27s ===
RESULT: ok -- all 1296 tests ran across 24 shards
```

Each of the four sub-tests is satisfied by both predicates by construction: with
identical chunks `slices.Equal` is true; with different shapes the chunks differ;
with `nil` shown the length guard fires first; with an unreadable `shown` the
slices differ anyway. **The commit's entire normative change is unguarded.**

Note how the settled mutation set produced false confidence: *constant-false*,
*constant-true* and *unreadable-id-swallowed* are all **degenerate** — they break
a sub-test because they ignore the inputs. The actual pre-fix predicate is
non-degenerate and agrees with the new one on all four chosen cases. A mutation
set that only contains constants cannot distinguish two real predicates.

**Line violated.** `gui/composer_stub_test.go` — the test's own doc concedes the
gap and then attributes it to an unknown:

> WHAT THIS TEST CANNOT DO, stated plainly: it does not reproduce the leg that
> produced differing chunks for an unchanged shape. […] the leg is a state change
> that moves the chunks without moving the id, and it is still unidentified.

The leg is identified in I-1 below and is four lines of test. With it, the revert
goes red; without it, the test is documentation.

---

### I-1 — the F-520 leg IS identified: seating declares origins, and `WalletDescriptorTemplateId` is origin-invariant by construction

The brief asked me to say so if I found it. I did, and it is not exotic.

`WalletDescriptorTemplateId` hashes only `useSitePath ‖ writeNode(tree) ‖
[UseSitePathOverrides-TLV]` — `md/template_id.go:22-38`:

> Key-independent and origin-invariant (no keys/fingerprints/origin enter the preimage).

`composerTemplateChunksFor` (`gui/composer_flow.go:243-249`) composes through
`composerDeclaredOrigins(st)`, which is a **function of `st.assigned`** — i.e. of
the seating. So seating moves the chunks and cannot move the id:

```
PROBE3 | chunks equal = false | ids equal = true (b02b4403 vs b02b4403)
PROBE3 | OLD predicate (chunk compare) fires = true | NEW predicate fires = false
```

**The leg is reachable in `composerFlow` with no exotic input.** Seat at least one
slot, then take Back/decline at the keyed stub screen (`gui/composer_flow.go:129-131`)
or at the consent (`:136-138`); both `continue` to the top of the loop.
`composerShapeFlow`'s "Done" arm returns true without touching `st.assigned`
(`gui/composer_shape.go:512-522`), and `composerSizeAssignments` returns early
when the slot count is unchanged (`gui/composer_flow.go:216-225`) — seats survive.
`composerApplyShapeEdit` clears assignments **only** when the shape signature
actually changed (`gui/composer_discard.go:144-156`). So the next
`composerTemplateChunksFor` carries the declared origins, the chunks differ, the
id does not, and the old predicate fired a banner directly above a byte-identical
printed Template-ID. That is F-520.

**Action:** F-520's "leg unidentified" can be closed, and the four-line
regression test derived from it is exactly what C-2 needs.

---

### I-2 — the commit's stated justification is false by measurement: two chunk sets can carry one id and the cards do NOT seat

`gui/composer_flow.go:106-107` and the `1dab84a` commit message both assert:

> and if two chunk sets really do carry one id, the cards DO seat and there was
> nothing to warn about.

Constructed counterexample. Compose a 2-of-3 wsh, then seat slot @0 at an account
the §4f defaults would not have used (5'), leaving @1/@2 unseated. The unseated
slots' advertised origins **shift**, because the codec assigns the lowest free
account:

```
PROBE10 | BEFORE (nothing seated):
PROBE10 |   Slot @0 expects a key at m/48h/0h/0h/2h
PROBE10 |   Slot @1 expects a key at m/48h/0h/1h/2h
PROBE10 |   Slot @2 expects a key at m/48h/0h/2h/2h
PROBE10 | AFTER (slot @0 seated at account 5'):
PROBE10 |   Slot @0: deadbeef m/48h/0h/5h/2h
PROBE10 |   Slot @1 expects a key at m/48h/0h/0h/2h
PROBE10 |   Slot @2 expects a key at m/48h/0h/1h/2h
PROBE10 | Template-ID equal = true | banner OLD=true NEW=false
```

Those "expects a key at" lines are not decoration — they are the screen's own
instruction for minting the card
(`mk encode --xpub … --origin-path <path> --policy-id-stub <stub>`,
`gui/composer_stub.go:101-104`). An operator who wrote down "Slot @1 expects a key
at m/48h/0h/1h/2h", minted the cosigner card, then seated slot @0 and came back,
sees the same screen advertising a different origin with **no banner**.

The card then passes **layer 1** (stub membership — the stub is the top 4 bytes of
the unchanged Template-ID) and fails **layer 2** (`slotMatchesCard`, the
declaration match) with `errSeatNoSlot` — `gui/key_card_seating.go:64-98`. So the
cards do *not* seat, on precisely the input the justification says they do.

The banner's text ("The shape changed, so this id changed. Cards minted with the
old stub will not seat here.", `gui/composer_copy.go:306-309`) is not *falsified*
by the silence — silence asserts nothing, and the card fails on origin rather than
on stub. That is why this is Important and not Critical. But the narrowing from
"the chunks moved" to "the id moved" removed a warning that was doing real work
along with the spurious one, and it was justified by a claim that does not hold.

Either widen the predicate (id **or** the advertised per-slot origins), or keep the
id comparison and give the origin drift its own line. Do not leave the false
justification in the source comment and the commit message.

---

### I-3 — `composerScriptLine` labels a decoded `sh(wpkh)` "Legacy (sh)": the confident wrong answer its own default arm claims to prevent

`gui/composer_shape.go:145-169` switches on `tpl.Root` and splits `ScriptSh` on
`tpl.InnerWsh` alone. But `ScriptSh` is a **three**-way collapse, not two: the
decoder summarises a BIP-49 `sh(wpkh)` wire to `Root==ScriptSh` with
`InnerWpkh==true` (`md/md.go:1165-1177`, `md/md.go:1220-1226`). It therefore falls
into the `else` and is named as a bare legacy P2SH multisig — a different script,
a different address:

```
PROBE2 | wpkh (BIP-84)      | Root=0 InnerWsh=false InnerWpkh=false | composerScriptLine = "Script: 0"
PROBE2 |   consent line 1 = "Script: 0"
PROBE2 | pkh (BIP-44)       | Root=1 InnerWsh=false InnerWpkh=false | composerScriptLine = "Script: 1"
PROBE2 |   consent line 1 = "Script: 1"
PROBE2 | sh(wpkh) (BIP-49)  | Root=2 InnerWsh=false InnerWpkh=true | composerScriptLine = "Script: Legacy (sh)"
PROBE2 |   consent line 1 = "Script: Legacy (sh)"
```

(Built through the real `md.EncodeSingleSig` → `ExpandWalletPolicyChunks` path and
fed through the real `composerConsentLinesFor`, not a hand-built `md.Template`.)

**Line violated.** `gui/composer_shape.go:157-160`:

> Single-sig roots the composer cannot build. Naming the root honestly beats
> forcing it into one of the four labels, which would be a confident wrong answer
> on the screen that consents to steel.

`sh(wpkh)` *is* a single-sig root the composer cannot build, and it *is* forced
into one of the four labels. The advertised defence has a hole exactly one
`if tpl.InnerWpkh` wide. The adjacent claim at `:141-143` — "sh(wsh(...)) and bare
sh(...) … so the two are never collapsed here" — is true of those two and silently
untrue of the third.

**Reachability, measured and stated plainly:** not reachable today.
`composerConsentLinesFor`'s only production caller is
`gui/composer_selfcheck.go:225`, reached only from `composerConsentFlow`, whose
chunks always come from `composerTemplateChunksFor`/`composerArtifactsFor`. And
all ten composer-reachable shapes land in the four labelled arms (table below), so
no composed policy can reach the hole. What holds it shut is the **call site**,
not a guard — while the function's contract is written as if it were total, and it
takes arbitrary chunks. Filed Important as "an unsound assumption / a missing
case" rather than Critical, since no current path puts it on a screen.

---

### M-1 — the default arm prints a raw enum integer, so it does not "name the root" at all

`fmt.Sprintf("Script: %v", tpl.Root)` with no `String()` method on
`md.ScriptKind` (confirmed: `grep -rn "ScriptKind) String"` returns nothing)
renders `"Script: 0"` / `"Script: 1"` — see the I-3 output. Not false, but the
comment's promise is *"Naming the root honestly"*, and `0` names nothing to an
operator. Either give `ScriptKind` a `String()` or refuse the line outright the
way `composerConsentLinesFor` refuses an incomplete shape
(`gui/composer_consent.go:122-128`).

---

### M-2 — the `labels`/`order` alignment is enforced by nothing; a fifth entry in either is a runtime panic, not a compile error and not a silent mislabel

*(Status: OPEN at `1dab84a`. **CLOSED at `b23f6cf`** — the parallel fold's
`TestWrapperLabelsNameTheirOwnWrapper` asserts the lengths at
`gui/composer_gates_test.go:1577`.)*

The brief's question, answered by measurement. Both slices are indexed by the
other's length — `composerWrapperPick` returns `wrappers[sel]` where `sel` is
bounded only by `len(choices) == len(composerWrapperLabels)`
(`gui/composer_shape.go:184-204`), and `composerScriptLine` iterates
`composerWrapperOrder` while indexing `composerWrapperLabels[i]`
(`gui/composer_shape.go:164-167`). Nothing asserts they agree.

```
### BASELINE ###
PROBE6 | len(composerWrapperLabels)=4 len(composerWrapperOrder)=4
PROBE6 | composerWrapperPick's `wrappers[sel]` at sel=len(labels)-1 = ok
PROBE6 | composerScriptLine's `labels[i]` at i=len(order)-1 = ok
### CASE A: 5th LABEL only ###
PROBE6 | len(composerWrapperLabels)=5 len(composerWrapperOrder)=4
PROBE6 | composerWrapperPick's `wrappers[sel]` at sel=len(labels)-1 PANICS: runtime error: index out of range [4] with length 4
### CASE B: 5th WRAPPER only ###
PROBE6 | len(composerWrapperLabels)=4 len(composerWrapperOrder)=5
PROBE6 | composerScriptLine's `labels[i]` at i=len(order)-1 PANICS: runtime error: index out of range [4] with length 4
```

Both cases `go build ./gui/` clean (`build exit: 0`) and pass the composer tests.
So the answer is **neither** of the brief's two options: a device panic, uncaught
by compiler or suite. The comment at `gui/composer_shape.go:125-127` ("the ORDER
IS LOAD-BEARING in both directions") states the invariant correctly and nothing
enforced it.

---

### M-3 — `composerIdChanged` compares ids without comparing `WalletIdKind`

`md.FormAwareIdChunks` returns `(id, kind, err)` and `composerIdChanged` discards
`kind` twice (`gui/composer_stub.go:45,49`). A keyless template yields a
key-stable `WalletIdTemplate`; a keyed policy yields a key-dependent
`WalletIdPolicy` — two 16-byte values in **different id spaces**, which
`md/template_id.go:130-137` exists specifically to keep apart:

> This exists because the two ids are indistinguishable once rendered: both are 16
> bytes of hex, and they differ for the same wallet.

Measured for one policy:

```
PROBE4 | keyless: Template-ID 32795543e9a2 | keyed: Policy-ID fee01d58e2fa | composerIdChanged(keyless,keyed) = true
```

Safe today only because the sole call site passes keyless sets on both sides
(`shown` is only ever assigned `template`, `gui/composer_flow.go:116,119`). A
one-line `kind` check would make that structural rather than incidental.

---

### N-1 — `shown` records a template the operator was never shown when the stub screen failed to render

`composerStubFlow` returns false both for Back **and** for a
`composerStubLines` error (`gui/composer_stub.go:124-130`), and the caller assigns
`shown = template` on that branch regardless (`gui/composer_flow.go:116`). A later
banner then speaks of "Cards minted with the old stub" for a stub that was never
displayed. Pre-existing — both branches predate this diff and only the `changed :=`
line changed — and every outcome is safe-side, so: noted, not blocking.

---

## Table — every wrapper and root exercised, with the label produced

Built through `md.Compose` → `Chunks()` → `md.ExpandWalletPolicyChunks` →
`composerScriptLine`, i.e. the real decode path the Review uses. `Root` is the
`md.ScriptKind` ordinal (0=Wpkh 1=Pkh 2=Sh 3=Wsh 4=Tr).

### Composer-reachable shapes (all correct)

| shape | Root | InnerWsh | InnerWpkh | label produced | verdict |
| --- | --- | --- | --- | --- | --- |
| tr 1-of-1 | 4 | false | false | `Script: Taproot (tr)` | correct |
| tr 2-of-3 | 4 | false | false | `Script: Taproot (tr)` | correct |
| tr internal key + leaf | 4 | false | false | `Script: Taproot (tr)` | correct |
| wsh 1-of-1 | 3 | false | false | `Script: Segwit (wsh)` | correct |
| wsh 2-of-3 | 3 | false | false | `Script: Segwit (wsh)` | correct |
| wsh 2 paths | 3 | false | false | `Script: Segwit (wsh)` | correct |
| wsh 2-of-3 + hashlock path | 3 | false | false | `Script: Segwit (wsh)` | correct |
| sh-wsh 2-of-3 | 2 | **true** | false | `Script: Nested (sh-wsh)` | correct |
| sh 2-of-3 | 2 | false | false | `Script: Legacy (sh)` | correct |
| sh 1-of-2 | 2 | false | false | `Script: Legacy (sh)` | correct |

**No composer-reachable wrapper lands in the default arm** — the brief's question 1,
answered by measurement, 10 of 10. `ValidatePathList` is why: legacy wrappers are
held to a sole bare sorted multi with `N >= 2` (`md/compose.go:330-337`,
`isBareMulti` at `:171-173`), so `sh(wpkh)`/`sh(pk)` cannot be lowered, and
`keyLeaf`'s `N == 1` single-key arm only ever sits *under* a wsh/tr/sh wrapper node.

### Roots the composer cannot build, fed in via `md.EncodeSingleSig`

| shape | Root | InnerWsh | InnerWpkh | label produced | verdict |
| --- | --- | --- | --- | --- | --- |
| wpkh (BIP-84) | 0 | false | false | `Script: 0` | M-1 — uninformative, not false |
| pkh (BIP-44) | 1 | false | false | `Script: 1` | M-1 — uninformative, not false |
| **sh(wpkh) (BIP-49)** | 2 | false | **true** | `Script: Legacy (sh)` | **I-3 — confidently WRONG** |

### Is the Review rendered from the chunks that will be engraved? (brief question 2)

Verified by reading every leg of `composerFlow`:

- **Form A (concrete):** consent renders `keyed`; the plate is `keyed`. Byte-identical.
- **Form B (template + key cards):** consent renders `keyed` when seating is
  complete (`gui/composer_flow.go:132-136`) while the plate is `template`
  (`:371`). The two are the same `st.list` and the same `declared` composed twice,
  differing only by `Bind`, which touches pubkeys/fingerprints and not the tree —
  so **the script line is form-invariant**, measured:
  `PROBE4 | keyless consent[0] = "Script: Nested (sh-wsh)" | keyed consent[0] = "Script: Nested (sh-wsh)"`.
  The form choice happens *after* consent and cannot falsify the script line.
  (The *ids* do differ across the two, which is why the screen labels them by kind.)
- **No caller passes chunks from a different composition.** The only substitution
  point is `composerSelfCheckFaultHook` (`gui/composer_selfcheck.go:26,214-215`),
  set only in `composer_selfcheck_test.go` and covered by a nil-at-rest assertion
  at `composer_selfcheck_test.go:212-213`.
- Worth recording for whoever owns §8q: **`composerSelfCheck` never compares the
  root or wrapper** — it compares branch count, thresholds, locks, digests, slot
  count, origins and fingerprints (`gui/composer_selfcheck.go:63-150`). The new
  script line is therefore the *only* place a wrong wrapper would surface, which
  raises the stakes on C-1 rather than lowering them.

### Id collisions (brief question 4)

Infeasible, and measured rather than asserted. Over 208 composer-reachable shapes
(4 wrappers × n=1..5 × k=1..n × sorted × 3 path shapes):

```
PROBE5 | 208 shapes composed, 160 distinct ids, 48 benign dupes (identical chunks), 0 REAL collisions
```

The 48 duplicates are **not** collisions: they are `Sorted=true`/`Sorted=false`
pairs that lower to byte-identical chunks, because `keyLeaf` ignores `Sorted`
unless `sortedLegal` (`md/compose.go:372-384`, `wshChain` at `:421-438`). Same
plate, same id, correctly no banner. A genuine collision would need a 128-bit
truncated-SHA-256 preimage collision; the theoretical alternative — one tree's
bit-encoding being a prefix of another's, colliding after `intoBytes()` zero-pads
to a byte boundary (`md/bits.go:147-149`) — is closed by the grammar being
self-delimiting, since the decoder parses a tree from the bitstream with no length
prefix and so no valid encoding is a proper prefix of another.

### `composerIdChanged` edge matrix (brief question 4)

All safe-side; the unreadable cases fail toward the spurious warning as documented.

| `shown` | `current` | result |
| --- | --- | --- |
| nil | good | false |
| empty slice | good | false |
| good | nil | true |
| good | empty slice | true |
| good | unreadable | true |
| unreadable | good | true |
| unreadable | same unreadable | true |
| good | same good | false |

`shown` is assigned on **every** leg that reaches the stub screen
(`gui/composer_flow.go:116` on Back/error, `:119` on forward), and cannot hold a
set from a different shape: it is only ever the `template` just passed to
`composerStubFlow`. The second stub screen (`:129`) hardcodes `changed=false` and
does not update `shown`; harmless, because the post-seating template it displays
shares the pre-seating template's id (I-1) — but note the correctness of that
hardcoded `false` depends on the origin-invariance in I-1, which is nowhere stated
at that line.

---

## Mutation table

Every mutation applied to the worktree at `1dab84a`, run, then reverted.
Suite runs are `scripts/gui-shard-test.sh ./gui/ 24` (1296 tests, partition
asserted exhaustive).

| # | mutation | target guarantee | result | finding |
| --- | --- | --- | --- | --- |
| M-A | `composerIdChanged` → `return !slices.Equal(shown, current)` (the verbatim `6728c22` predicate) | "compares ids not chunks" | **SURVIVES** — 4/4 sub-tests pass; 1296/1296 green | **C-2** |
| M-B | swap `composerWrapperLabels[0]` and `[1]`; leave `composerWrapperOrder` | "the Review names the script the policy uses" | **SURVIVES** — 4/4 sub-tests pass; 1296/1296 green | **C-1** |
| M-C | swap `composerWrapperOrder[0]` and `[1]`; leave labels | same | RED — expected `Segwit (wsh)`, got `Taproot (tr)` | (gate holds) |
| M-D | append a 5th entry to `composerWrapperLabels` only | slice alignment | builds clean, suite green, **runtime panic** `index out of range [4] with length 4` in `composerWrapperPick` | **M-2** |
| M-E | append a 5th entry to `composerWrapperOrder` only | slice alignment | builds clean, suite green, **runtime panic** `index out of range [4] with length 4` in `composerScriptLine` | **M-2** |
| M-B′ | M-B re-run at branch tip `b23f6cf` | same as M-B | **RED** — `TestWrapperLabelsNameTheirOwnWrapper` fails both rows | C-1 closed there |
| M-A′ | M-A re-run at branch tip `b23f6cf` | same as M-A | **SURVIVES** — still green | C-2 open there |

Already settled per the brief and not re-derived: the Review line removed (4
sub-tests RED); sh-wsh collapsed into sh (Nested alone RED); the id predicate
constant-false, constant-true, and unreadable-id-swallowed (RED).
**Those five do not discriminate between the old and new predicates, nor between
a right and a wrong label table** — which is the mechanism behind C-1 and C-2.

---

## Counts

**Against the reviewed range `6728c22..1dab84a`:**

| severity | count | ids |
| --- | --- | --- |
| Critical | 2 | C-1, C-2 |
| Important | 3 | I-1, I-2, I-3 |
| Minor | 3 | M-1, M-2, M-3 |
| Nit | 1 | N-1 |

**Still open at the current branch tip `b23f6cf`** (C-1 and M-2 closed there by the
parallel fold): **1 Critical (C-2), 3 Important (I-1, I-2, I-3), 2 Minor (M-1, M-3),
1 Nit (N-1).**

## Verdict

**NOT GREEN.**

Neither fix is wrong on any input the composer can reach — I could not construct a
false script name or a wrong-direction banner from composed chunks alone, and the
label matrix is 10 for 10. What I could construct is the thing a gate exists to
prevent: **both commits ship an unpinned guarantee.** C-2 is the one that matters
now — `composerIdChanged` can be reverted wholesale to the predicate that produced
F-520 with every one of 1296 tests green, and the regression case that would close
it is the leg the test called unidentified and which I-1 identifies in four lines.
I-2 additionally shows the reasoning that justified the narrowing is false: an
unchanged id does **not** imply the cards seat.

---

*Reviewed read-only in detached worktrees at `1dab84a` and `b23f6cf`; no commits
made; every probe file deleted and every mutation reverted; both scratch
worktrees removed; `git status --porcelain` empty in
`/scratch/code/shibboleth/seedhammer` on exit.*

---

# Addendum — fold re-review at `c1f7e3a`

**Range:** `git diff b23f6cf..c1f7e3a`, one commit, ten files. Scope was *did the
fold close each finding, and did it introduce a new defect* — not a fresh audit.
C-1 and M-2 were verified closed at `b23f6cf` in the body above and were not
revisited. Method as before: detached worktree at `c1f7e3a`, every probe file
deleted, every mutation reverted, worktree removed, `git status --porcelain`
empty in the fork on exit. I did not re-run the suite, the vet or the tinygo
build — taken as given per the dispatch (gui 1299/1299, +896 flash).

**Verdict: GREEN.** Every Critical and Important is closed and — except where
noted — pinned by a mutation that reds. Three new Minors, none blocking.

## Verdict per finding

| finding | verdict | pinned by a mutation? |
| --- | --- | --- |
| C-1 mislabel test is tautological | CLOSED at `b23f6cf` (not revisited) | yes (verified in body) |
| C-2 id predicate revert survives | **CLOSED** | **yes** — M-A′ reds |
| I-1 F-520 leg unidentified | **CLOSED / SUPERSEDED** — named in source, encoded in the fixture | yes, via M-F |
| I-2 "one id means the cards seat" is false | **CLOSED** — direction corrected | **yes** — M-F reds |
| I-3 `sh(wpkh)` named "Legacy (sh)" | **CLOSED** | **yes** — M-H reds |
| M-1 default arm printed the enum | CLOSED in behaviour, **UNPINNED** | no — M-I survives |
| M-2 labels/order alignment | CLOSED at `b23f6cf` (not revisited) | yes (verified in body) |
| M-3 kind not compared | CLOSED in behaviour, **UNPINNED** | no — M-G survives |
| N-1 `shown` set on a render failure | NOT FOLDED — **I agree**, pre-existing and safe-side | n/a |

## Mutation table (at `c1f7e3a`)

Baseline of the named tests: `ok  seedhammer.com/gui  0.109s`.

| # | mutation | result |
| --- | --- | --- |
| M-A′ | `composerStubDelta` → the verbatim pre-fix `!slices.Equal(shown, current)` | **RED** — `--- FAIL: TestComposerStubDeltaCatchesOriginDriftUnderOneId` |
| M-F | drop the origin comparison, `return composerStubUnchanged` | **RED** — `--- FAIL: TestComposerStubDeltaCatchesOriginDriftUnderOneId` |
| M-G | drop `\|\| wasKind != nowKind` (M-3's addition) | **SURVIVES** — `ok  seedhammer.com/gui  0.118s` |
| M-H | delete the `case tpl.InnerWpkh:` arm | **RED** — `md1_expand_test.go:394: composerScriptLine = "Script: Legacy (sh)", want it to name "Nested single-sig (sh(wpkh))" (Root=2 InnerWsh=false InnerWpkh=true)` |
| M-I | UNKNOWN arm → `fmt.Sprintf("Script: %v", tpl.Root)` | **SURVIVES** — `ok  seedhammer.com/gui  0.113s` |

M-A′ is the exact mutation that survived at `1dab84a` and `b23f6cf` and produced
C-2. It now reds, because the revert reports `IdMoved` where only the origins
moved and the test asserts the screen carries the origins sentence and *not* the
id sentence. C-2 is genuinely closed, not relabelled.

M-G and M-I survive because neither guarded fact is reachable at the call site
(the kind cannot cross while both arguments are keyless templates; no composed
policy reaches the unknown-root arm). Both changes are correct and cheap; they
are defensive code without a test, which is a different thing from a false gate.
Recorded, not blocking.

## The I-2 fixture: it is mine, and it cannot decay into a tautology

`TestComposerStubDeltaCatchesOriginDriftUnderOneId` is the counterexample I
constructed, reproduced exactly: 2-of-3 `wsh`, slot @0 declared at account 5' via
`composerTestOrigin(2, 5)` = `md.DefaultOrigin(md.ComposeWsh, 5)`, slots @1 and
@2 left `nil`. Same shape, same account, same displacement.

Its three guards were each broken to confirm they fail loudly rather than
vacuously passing:

```
### seat @0 at account 0' (the default -- nothing drifts) ###
--- FAIL: TestComposerStubDeltaCatchesOriginDriftUnderOneId
    composer_stub_test.go:334: the advertised origins did not move:

### seating declares nothing (slot left nil) ###
--- FAIL: TestComposerStubDeltaCatchesOriginDriftUnderOneId
    composer_stub_test.go:316: the fixture produced identical chunks: seating declared nothing
```

The id-equality guard is the third and is the one that matters most: if
`WalletDescriptorTemplateId` ever stopped being origin-invariant, the fixture
would `t.Fatalf` ("it no longer isolates origin drift") instead of quietly
becoming a test of something else. The origins guard calls the function under
test, but degeneracy there fails the guard rather than satisfying it. **It cannot
decay silently.**

## Answers to the three questions asked

### (1) A new false positive? YES — one, Minor. See N-3.

The common case is quiet, which is the important half. Seating a slot at the
account the screen already advertised moves nothing:

```
FOLD3 | seat @0 at the account it already advertised (0') -> delta=0 | advertised=[@0 m/48h/0h/0h/2h @1 m/48h/0h/1h/2h @2 m/48h/0h/2h/2h]
FOLD3 | seat @0 at 1' (the account @1 advertised)      -> delta=2 | advertised=[@0 m/48h/0h/1h/2h @1 m/48h/0h/0h/2h @2 m/48h/0h/2h/2h]
FOLD3 | seat @0 at 5' (nobody's default)               -> delta=2 | advertised=[@0 m/48h/0h/5h/2h @1 m/48h/0h/0h/2h @2 m/48h/0h/1h/2h]
```

But the comparison includes **seated** slots, and a seated slot's origin is no
longer an instruction to mint anything. Counterexample — a sole-slot policy:

```
FOLD3 | SOLE SLOT, seated at 5': delta=2 | [@0 m/48h/0h/0h/2h] -> [@0 m/48h/0h/5h/2h]
```

The banner fires with "Cards minted for the old origins will not seat here" on a
policy that now has **zero** unseated slots — no card is needed, and none can be
minted. The sentence is conditionally true (a card minted for the old advertised
origin indeed would not seat), so this is not a false statement and not Critical.
It is noise on the screen this whole cycle has been about not crying wolf on.

Restricting the comparison to slots still advertising (`!k.FingerprintPresent`,
the "expects a key at" rows) silences it **and still catches I-2** — verified
against the fixture: @1 moves `1h -> 0h` and @2 moves `2h -> 1h`, both unseated.
Offered as a direction, not a prescription.

### (2) Does `composerAdvertisedOrigins` reproduce what the screen prints? NO. See N-2.

The screen branches on `k.FingerprintPresent` alone
(`gui/composer_stub.go`, the "Slot @%d: %x %s" / "Slot @%d expects a key at %s"
pair). `composerAdvertisedOrigins` branches on `k.XpubPresent &&
k.FingerprintPresent`. For a **keyless template — the only input the production
caller ever passes** — `XpubPresent` is false on every slot, measured:

```
FOLD1 | slot @0 XpubPresent=false FingerprintPresent=true  fp=deadbeef origin=m/48h/0h/5h/2h
FOLD1 | --- what the SCREEN prints ---
FOLD1 |   Slot @0: deadbeef m/48h/0h/5h/2h
FOLD1 | --- what composerAdvertisedOrigins COMPARES ---
FOLD1 |   @0 m/48h/0h/5h/2h
```

So the fingerprint arm is **dead code** and the fingerprint is never compared.
Consequence, measured — two templates differing only in a seated slot's declared
fingerprint:

```
FOLD2 | screen A: "Slot @0: aaaaaaaa m/48h/0h/0h/2h"
FOLD2 | screen B: "Slot @0: bbbbbbbb m/48h/0h/0h/2h"
FOLD2 | screen lines differ = true | composerStubDelta = 0   (Unchanged)
```

A printed fact changed and the comparison was silent — precisely the drift the
function's own doc says the design makes impossible ("in the form the screen
prints them, so what is COMPARED is what was SHOWN"). **Bounded:** a seated slot
needs no card, so no card-seating warning is missed and I-2's guarantee is
intact. Minor.

Note the remedy is not simply deleting `k.XpubPresent &&` — that would make a
fingerprint change report `OriginsMoved` and put "the slot origins below changed"
on screen when no origin moved, trading a silent drift for a false sentence.
Either correct the comment to say it compares *origins* (which is what the
message is about and what layer 2 uses for an unseated slot), or give a
fingerprint change its own line.

**Verified sound, same question, other paths:** the keyed arm never carries a
banner — `composerStubFlow(..., keyed, composerStubUnchanged)` is hardcoded at
`gui/composer_flow.go:135` — and that is correct rather than lucky, because
`composerArtifactsFor` returns a non-empty `keyed` only when
`composerSeatingComplete` (`gui/composer_flow.go:272-273`), so that screen is
reached only with zero unseated slots and nothing to mint. `composerStubLines`
always derives its slot lines from `templateChunks`, never from `keyedChunks`, so
the keyed screen cannot show a different composition's origins.
`composerStubDelta`'s only production caller is `gui/composer_flow.go:120` with
template chunks on both sides, and `composerAdvertisedOrigins` has no caller
outside it. `composerIdChanged` is fully gone (0 references).

### (3) Is `IdMoved` still the right safe side for unreadable input? For the ID legs yes; for the ORIGINS legs it is the wrong sentence. See N-4.

```
FOLD4 | nil shown              -> 0   (Unchanged)
FOLD4 | unreadable shown       -> 1   (IdMoved)
FOLD4 | unreadable current     -> 1   (IdMoved)
FOLD4 | identical              -> 0   (Unchanged)
```

The two **id** error legs are right: nothing is known, and `IdMoved` is the
stronger of the two warnings (a layer-1 failure implies re-minting everything),
so it dominates `OriginsMoved`. Keep them.

The two **origins** error legs are not. Both sit *below* `if wasID != nowID ||
wasKind != nowKind`, so by the time they run the function has **proven the ids
equal** — and returning `composerStubIdMoved` makes the screen print "The shape
changed, so this id changed" directly above a byte-identical Template-ID. That is
F-520's exact failure mode, in the one place the fold did not look.

**Latent, not live:** `ExpandWalletPolicy` has zero error returns (measured:
`awk '/^func ExpandWalletPolicy\(/,/^}/' md/expand.go | grep -c "return .*err"`
→ `0`; its only exit is `return out, nil`), so `composerAdvertisedOrigins` can
fail only via `Reassemble`, which `FormAwareIdChunks` has already caught two
lines above. The branch is unreachable today. `OriginsMoved` — warn about the
fact you could not check — is the honest safe side there.

## New findings

### N-2 — `composerAdvertisedOrigins` compares a different thing from what the screen prints; the fingerprint arm is dead code

Evidence and reasoning under question (2) above. Minor: no missed card-seating
warning, but the function's stated equivalence to the screen is false and one
printed fact is uncompared.

### N-3 — the origins comparison fires for seated slots, which are not minting instructions

Evidence under question (1) above. Minor: true-but-noisy, quiet in the common
case, strictly less noisy than the chunk predicate it replaced.

### N-4 — the origins error legs report `IdMoved` after proving the ids equal

Evidence under question (3) above. Minor: unreachable today because
`ExpandWalletPolicy` cannot error.

## Pushback where asked, and none where not

**M-2 / the alignment guard — I do not push back; your judgement is right.**
`b23f6cf` pins `len(composerWrapperLabels) == len(composerWrapperOrder)`
(`composer_gates_test.go:1577`) and `TestWrapperLabelsNameTheirOwnWrapper`
iterates the labels checking each row's wrapper actually encodes the script its
token names. A fifth entry in **one** slice trips the length assertion; a fifth
entry in **both** is covered by the binding test, which would check the new row
like any other. A compile-time guard would move the failure from `go test` to
`go build`, which is nicer but buys nothing a run of the suite does not already
catch. Not worth a commit.

**N-1 — agreed, correctly not folded.** It is pre-existing (both branches predate
the diff), every outcome is safe-side, and the new three-way state does not change
that: an unreadable `shown` still reports `IdMoved`, which is the stronger
warning. I would leave it.

**I-2 direction.** For the record, since you raised it: the corrected comment at
`gui/composer_flow.go` and `gui/composer_stub.go` now states plainly that the
earlier claim was measured and is false, rather than quietly deleting it. That is
the right disposition — the false claim had already reached a commit message, and
a future reader tracing this line needs to know it was tested, not merely
rewritten.

## Closing counts

Against the original report: **0 Critical, 0 Important open.** C-1, C-2, I-1, I-2,
I-3, M-1, M-2, M-3 all CLOSED (M-1 and M-3 in behaviour but unpinned); N-1
deliberately not folded and agreed.

New at `c1f7e3a`: **0 Critical, 0 Important, 3 Minor** (N-2, N-3, N-4).

**GREEN.** Nothing here blocks a gate. If any of the three new Minors is worth a
follow-up entry, N-3 is the one with an operator-visible symptom and N-2 the one
most likely to grow teeth, since it is a comparison whose advertised contract is
already wrong.

*Re-reviewed read-only in a detached worktree at `c1f7e3a`; no commits made;
every probe file deleted and every mutation reverted; worktree removed;
`git status --porcelain` empty in `/scratch/code/shibboleth/seedhammer` on exit.*

---

## Addendum 2 — intersection rule at `aa93c71`

**Range:** `git diff c1f7e3a..aa93c71`, two files, both `gui/`. Scope: the origin
comparison rule only. Everything previously CLOSED stays closed. Suite, vet and
tinygo numbers taken as given, not re-run.

**Verdict: NOT GREEN — one Important.** The intersection rule is silent on a
reachable walk that leaves the operator holding a card that will not seat. This
is I-2's defect class returning through a different gap, which is the direction
you asked me to push on.

### I-4 — a slot that leaves the advertising set carries no history, so drift across that gap is never reported

The loop iterates `wasOrigins` and requires the slot to be present in both:

```go
for idx, was := range wasOrigins {
    if now, ok := nowOrigins[idx]; ok && now != was {
```

`shown` holds only the immediately-previous reading, so a slot absent from
*either* side is dropped. Two consequences, the second unconditional: a slot that
is seated in one reading and unseated in a later one is never compared across
that gap, and **a fully-seated reading has an empty advertised set, which makes
the next comparison silent no matter what moves.**

Counterexample, measured — one 2-of-3 `wsh`, one id throughout, no shape edit:

```
ISECT | E1   id=b02b4403 advertised=map[0:m/48h/0h/0h/2h 1:m/48h/0h/1h/2h 2:m/48h/0h/2h/2h]
ISECT |        Slot @2 expects a key at m/48h/0h/2h/2h
ISECT | E2   id=b02b4403 advertised=map[]
ISECT |        Slot @0: bbbbbbbb m/48h/0h/2h/2h
ISECT |        Slot @2: aaaaaaaa m/48h/0h/5h/2h
ISECT | E3   id=b02b4403 advertised=map[2:m/48h/0h/0h/2h]
ISECT |        Slot @0: bbbbbbbb m/48h/0h/2h/2h
ISECT |        Slot @2 expects a key at m/48h/0h/0h/2h
ISECT5 | E1->E2 = 0 | E2->E3 = 0 | (net) E1->E3 = 2
ISECT5 |   card for m/48h/0h/2h/2h matches slot index -1 at E3 (-1 = no slot => errSeatNoSlot)
```

E1 instructs the operator to mint against `m/48h/0h/2h/2h`; they send it to a
cosigner. They seat every slot (E2, empty set, silent), then release @2 (E3,
silent because `wasOrigins` was empty). @2 now advertises `m/48h/0h/0h/2h`, and
`m/48h/0h/2h/2h` is held by seated @0 under fingerprint `bbbbbbbb`. Run through
the real `slotMatchesCard`, the cosigner's card matches **no slot** —
`errSeatNoSlot`. The net delta is `OriginsMoved`; every step of it was silent.

Both legs are ordinary composer traffic: the stub screen is drawn once per loop
iteration and `shown` updated there, seating accumulates between iterations, and
`composerReleaseLastSeat` is the Back leg out of the mapping review. A three-
reading variant (seat @2, then release it while seating @0 and @1) is silent the
same way; the previous equality rule fired on both steps of it —
`ISECT2 | equality rule: R1->R2 fires=true R2->R3 fires=true` — so the silence is
new with this revision, not inherited.

**Direction, not a prescription:** compare against the last origin each slot was
*seen advertising*, not against the previous reading — a per-slot map in
`composerFlow` updated at each showing, entries never removed. Verified it would
fire here: net E1→E3 is `OriginsMoved`. That keeps the departure exclusion you
wanted (a slot seated and never re-advertised is still never compared) while
restoring history across the gap.

### The departure exclusion itself is right

Seating @1 at the account it already advertised is silent
(`ISECT3 | delta = 0`) and should be: the card that stops seating is one the
operator personally made unnecessary, and the screen shows the slot filled. That
is the N-3 false positive, correctly removed. My finding is only about slots that
come *back*.

### The two secondary items

**The I-2 fixture guard did not become weaker.** It now reimplements the
intersection inline and counts `moved`, then asserts
`composerStubDelta == composerStubOriginsMoved`. The guard is independent code
from the production loop, so a mutation to the rule still reds the assertion; and
requiring an *intersecting* slot to move is what the rule needs to fire, where
list inequality could have been satisfied by a departure alone. Correctly
tightened. The I-2 case still fires: there @1 and @2 advertise in both readings
and both move (`1h -> 0h`, `2h -> 1h`).

**`OriginsMoved` on an unreadable expansion is right, not merely different.**
Those two legs run after `wasID == nowID` is proved, so `IdMoved` would print
"this id changed" above a byte-identical Template-ID — journey I-5's shape via
its own fix's error path. `OriginsMoved` warns about the fact that could not be
checked without asserting a falsehood, and it is the weaker of the two warnings,
which is correct for "unknown". Still unreachable (`ExpandWalletPolicy` has no
error returns; this diff touches only `gui/`).

### Counts

**1 Important (I-4), 0 Critical.** Nothing else in scope changed status.

*Read-only in a detached worktree at `aa93c71`; no commits; probes deleted;
worktree removed; `git status --porcelain` empty in the fork on exit.*

---

## Addendum 3 — per-slot memory at `a299218`

**Range:** `git diff aa93c71..a299218`, three files. Scope: the origin comparison
rule only. Suite/vet/tinygo taken as given. Read-only worktree, probes deleted,
worktree removed, fork clean.

**Verdict: GREEN.** I could not construct a counterexample in any of the three
directions. I-4 is closed and the fix does not trade into its opposite.

**I-4 closed, measured.** The walk that was silent at `aa93c71`:

```
MEM2 | E1 nothing seated   delta=0 advertised=map[0:…0h/2h 1:…1h/2h 2:…2h/2h]
MEM2 | E2 all seated       delta=0 advertised=map[]
MEM2 | E3 release @2       delta=2 advertised=map[2:m/48h/0h/0h/2h]
```

E2→E3 now returns `OriginsMoved` where the previous revision returned
`Unchanged`, and the empty middle reading no longer erases history.

### 1. Too loud? No — no false positive found

Every fire requires that a slot advertising **now** differs from the last origin
that slot was **seen** advertising, and recording happens only at a stub-screen
showing — so the remembered origin was always on screen and always mintable. The
rule cannot warn about an origin the operator was never shown.

The memory is not sticky: `remember` overwrites, so it holds the *last* advertised
origin, and a fire is followed by quiet rather than repeating. Depart-and-return
to the same origin stays silent, which is the case you were right to want quiet:

```
MEM4 | Q2 seat @2 at its own 2'   delta=0
MEM4 | Q3 release @2              delta=0     (@2 re-advertises m/48h/0h/2h/2h; a card for it still seats)
```

And the round trip stays correctly loud — R2 and R3 both fire because @1 and @2
really do move under the operator's feet in both directions:

```
MEM3 | R1 nothing seated   delta=0 advertised=map[0:…0h 1:…1h 2:…2h]
MEM3 | R2 seat @0 at 5'    delta=2 advertised=map[1:…0h 2:…1h]
MEM3 | R3 release @0       delta=2 advertised=map[0:…0h 1:…1h 2:…2h]
```

### 2. Shape edits do not corrupt the memory — and it is an invariant, not luck

The index-keyed memory is safe because **the only edits that preserve seats are
exactly the edits that preserve slot numbering.** An edit that renumbers changes
`composerShapeSignature`, so `composerApplyShapeEdit` discards every assignment;
the next reading then has all slots advertising, and the whole memory is
overwritten before any comparison can use a pre-edit entry. An edit that does not
renumber keeps the seats — and cannot move an origin. Measured on the one edit
class that changes the id while leaving the signature alone, a lock-value change
under `wsh`, with a slot seated across it:

```
MEM1 | signature equal = true
MEM1 | id equal        = false (30c7d8d1 vs d1f5fb83)
MEM1 | advertised identical = true
MEM1 | composerEditCanRenumber(lock) = false
```

Signature equal, id moved, advertised origins byte-identical. So a memory entry
carried across such an edit is still about the same slot and the same origin. I
could produce neither a cross-slot comparison nor a silence from an inherited
value. (Belt and braces: a reading immediately after a renumbering edit returns
`IdMoved` anyway, which is the louder verdict.)

### 3. Both recording legs are right

Recording on the Back leg as well as the forward one is correct — the operator
read the screen either way, and which button they left by does not change what
they were told.

The keyed second call recording nothing is also right, and for a reason rather
than by luck: it is reached only when seating is complete, and if the operator
loops, the next iteration re-derives the identical template and records it, so no
comparison is lost. With no next reading there is no later warning to miss.

One narrow caveat worth knowing, inherited rather than introduced: a slot seated
from an mk1 card whose `Fingerprint` field is missing or malformed gets
`fpPresent = false` (`gui/composer_sources.go:115-124`), so the advertising split
keeps it in the set and a fully-seated reading is not always empty —

```
MEM5 | every slot seated; advertised set = map[2:m/48h/0h/4h/2h] (len 1)
MEM5 |   Slot @2 expects a key at m/48h/0h/4h/2h
```

The screen tells the operator to mint for a slot that is already filled. That text
predates all four revisions of this predicate; for the **memory** it is the safe
direction, since the slot stays compared and recorded.

Also checked: `seen.remember` runs *before* `composerStubFlow` renders, so an
unshown reading could in principle be recorded. It cannot in practice —
`composerAdvertisedOrigins` and `composerStubLines` both go through `Reassemble`,
so the failure that hides the screen also returns nil origins and `remember(nil)`
is a no-op.

### Counts

**0 Critical, 0 Important, 0 Minor** in scope. Nothing else changed status.
Ship it.

*Read-only in a detached worktree at `a299218`; no commits; probes deleted;
worktree removed; `git status --porcelain` empty in the fork on exit.*
