# Journey walk — `IMPLEMENTATION_PLAN_composer_S3_fork_gui.md` (R0 round 0)

Lens: JOURNEY. Four journeys walked step by step against the plan's screens, strings
and tests (`design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md`, master, 7,638 lines),
with the extracted Go read in the read-only gate scratch
`/scratch/code/shibboleth/.plan-build-gate-go-s3/wired/gui/` to see exactly what each
screen draws. Nothing was edited anywhere; no `.jsonl` was read.

**Controller defaults assumed, as briefed:** Part A ships alone; §7f offers the
device's two real plate forms; presets absent (F-453 blocks A10), blank shape only.
Where a different answer to the author report's §5 questions would change a step, the
step says so.

**Counts: 2 Critical / 6 Important / 6 Minor / 4 Nit.**

---

## J1 — C26, no payload: door → Build → wsh 2-of-3 + a 90-day recovery path → stub → consent → template-only engrave

| # | in hand, exactly | what the device does | what ELSE they might do | class |
| --- | --- | --- | --- | --- |
| 1 | carousel, Wallet Policy | door `ChoiceScreen`: lead "No keys loaded. This builds a key-less template."; rows `Scan cards`, `Build a new policy` (`From payload` absent) — plan 1836-1895 | Back → whole program exits | DEFAULT (correct; §7a) |
| 2 | `Build a new policy` | `New policy` / "Which script?" → `Taproot (tr)`, `Segwit (wsh)`, `Nested (sh-wsh)`, `Legacy (sh)` — 2690-2699 | Back at the wrapper picker → the **whole Wallet Policy program exits**, not back to the door (1946) | **N-1** |
| 3 | `Segwit (wsh)` | `Spend paths` paged list: lead `slots: 0 / keys available: 0`; rows `Add a spend path`, `Done` — 2856-2876 | "keys available: 0" is drawn with no payload, which §7b scopes to "whenever a payload is loaded" | **N-2** |
| 4 | `Done`, immediately (a plausible first tap) | `ValidatePathList` → `ErrComposeNoPaths` → **no §8m mapping** → the raw codec string `md: compose: a wallet needs at least one spend path` (2620-2640, 2886-2889) | §4e row 1 maps this to §8m line 1; §11 requires a remedy and no encoding | **M-3** |
| 5 | `Add a spend path` | `Path 1` / "What can spend on this path?" → `Keys`, `A hash, no keys` — 2780-2794 | `A hash, no keys` → §8a confirm → hash pick list → **`No hash lock`** returns true and leaves an EMPTY path (2820-2823, 3960) | **M-4** |
| 6 | `Keys` | n picker 1..9, then k picker 1..n — 2720-2745 | picker never offers an illegal value; §8m line 5 reachable only at 32 slots | REFUSAL (correct, §4e) |
| 7 | n=3, k=2 | sole, unlocked, unhashed, n≥2 → `Key order` / "Sorted keys, or your order?" — 2746-2775 | **Back here sets `Keys = nil`** on an existing path (2754/2762) — "Back preserves everything" broken | **I-5** |
| 8 | `Sorted (usual)` | `Sorted: true` stored; back to the path list showing `Path 1: 2-of-3` | — | — |
| 9 | `Add a spend path` → `Keys` → n=1,k=1 | Path 2 added. **`composerSortedIsLegal` is now false for Path 1**, so §5 lowers it to `multi`: key order is part of the wallet. No screen says the sorted answer was reversed; the consent's UNSORTED mark is gated on `sole` (4736-4766) | the operator believes they chose sorted | **C-2** |
| 10 | select `Path 2` | `composerShapeGuard` (silent in Part A) → `Path 2` menu: `Keys`, `Time lock`, `Hash lock`, `Remove path` — 2826-2853 | no way to reorder paths, and order is normative (§5) | **N-3** |
| 11 | `Time lock` → `After a wait` → `Days` | digit pad, max 3 digits, lead "How many days?" — 3616-3628 | **empty field echoes §8u's ceiling refusal** before a digit is typed (3622) | **M-2** |
| 12 | type `90` | echo `90 days = 15188 units of 512 s (90.0 days)`; confirm icon appears — 3624-3626 | type `999` → §8u, confirm withheld. **65536 on the blocks pad → §8u, confirm withheld** (3609): J1's question answered correctly | REFUSAL (correct) |
| 13 | confirm | `composerLockAccept` → `composerReadScreen` echo → lock stored — 3675-3684 | Back at the echo discards the lock (documented, correct) | DEFAULT |
| 14 | `Done` | shape validates; §8h not triggered; **stub screen** — `Template-ID: <32 hex>`, `mk1 stub (template): <8 hex>`, the `mk encode …` command, §8d, then `Slot @i expects a key at m/48'/0'/i'/2'` per slot — 4248-4310 | **where they learn what to do with the stub:** the `mk encode` command line and §8d, both present. Answered | DEFAULT (correct, §7c) |
| 15 | Back on the stub screen | `edited = true`, return to the path list intact (4902) | press `Done` again without editing → the stub screen now asserts §8s "The shape changed, so this id changed" — **false** (4884-4917) | **M-1** |
| 16 | continue from the stub screen | consent via `composerReadScreen`: per-path lines, the id by kind, both stub labels, `Keyless template - no addresses.` — 4771-4840 | **Button3 on page 1 continues without paging** (902-940): with 8 paths the later paths and the addresses are never drawn | **I-4** |
| 17 | continue | §8l unskippable hold-to-confirm (4913-4918) | Back → `edited = true`, back to the shape | DEFAULT (correct) |
| 18 | hold | `composerEngraveTemplate`: plate census, then `bundleEngrave` — 4930-4945 | §7f's "the choice collapses to template only **and says so**" has no screen in Part A; the consent's "Keyless template - no addresses." is the only signal | DOCUMENTATION (acceptable under the Part-A-alone default) |

J1 is walkable end to end. Its defects are C-2, I-4, I-5 and four M/N.

---

## J2 — C8, payload with 4 `key:` records, a `hash:`, a `now:` and a seed

| # | in hand, exactly | what the device does | what ELSE they might do | class |
| --- | --- | --- | --- | --- |
| 1 | payload loaded at boot | F1 fires at LOAD for the seed, before any program (1105-1140) — the §9 item 4 correction is right and is gated | — | DEFAULT (correct) |
| 2 | Wallet Policy | door lead `Keys loaded: 4, plus 1 seed.`; rows `Scan cards`, `Build a new policy` (`From payload` absent — no Descriptor/MDMK record) — 1836-1895 | a malformed `hash:` would add "1 payload record was not understood." | DEFAULT (correct, §8r) |
| 3 | `Build a new policy` → wsh → shape | path list lead reads **`slots: 4 / keys available: 0`** — `st.sources` is never populated by any code path in this plan (2563-2585; `composerKeySources` is dead, see C-1) | — | **C-1** |
| 4 | Path 1 keys 2-of-3, Path 2 1 key + `hash:` record + a date lock | hash pick list shows `hash 1  abababab..abababab` after §8i (3947-3975); date pad refuses below the `now:` seconds with §8o and echoes the packed-date bound line (3660-3700) | **a date after 2038-01-19** → "that date does not exist" — false, and it is the inheritance case the preset list leads with (3649) | **I-6** |
| 5 | `Done` → stub screen → continue | **the flow goes straight to consent and the keyless engrave.** There is no seating step, no pick list, no mapping review, no §7e self-check, no §7f form choice, no census extension, no card minting — none of those functions is called from any flow (4437 promises the replacement; no task provides it) | the operator with four keys in flash reaches the census holding a KEY-LESS template | **C-1** |
| 6 | *(the rest of J2 is unreachable as the plan stands; walked against the code as written, on the assumption C-1 is fixed)* | | | |
| 6a | seating pick list, 5+ sources | `Slot @0, Path 1 key 1 of 3: choose a key` as body row 0, rows = unused sources + `Type a seed` + `Leave unseated` (6491-6560) | **the prompt is not redrawn on page 2+** (948), and paging is forward-only with wrap | **M-6** |
| 6b | page with Button2 past the end | `start` wraps to 0, `sel` is clamped only upward (992) | the cursor stays on an off-page row; the frame shows no highlight and Button3 takes it → **a key is seated that the operator never saw selected** | **I-3** |
| 6c | pick the same card twice | a used non-seed source is not offered again (6505-6520) | two distinct records holding one xpub → refused at the mapping review naming both slots (6226-6240) | REFUSAL (correct, §7d) |
| 6d | 3 keys for 4 slots | §8p: `4 slots, 3 keys available.` / `Unfilled: slot @3.` then `Back to the paths` / `Engrave a key-less template` (6568-6590) | the fallback route then meets the self-check, which fails on ≥2 unseated slots and on `len(keys) != len(st.assigned)` (6194-6220, 6891) | **I-2** |
| 6e | edit the wrapper after seating | §8j confirm at the path-list level, then the signature comparison discards (2877, 2882; B3 5932-5945) | **edit a LOCK after seating** — §7g rules it a DEFAULT with assignments kept, but the guard fires first and says "Every key you seated will be cleared"; declining leaves the lock uneditable | **I-1** |
| 6f | mapping review | slot, fingerprint, origin verbatim, "This device cannot confirm a key was derived at the origin it declares." (6314-6350) | correct, and F-217 is named | DEFAULT (correct) |
| 6g | consent | §7e's surface with both id labels and four addresses | §8d's own-wallet line is absent here; §7g answers the consent-step id comparison with that line | **M-5** |

---

## J3 — a seed only, tr with 2 paths, both slots from the seed

| # | in hand, exactly | what the device does | what ELSE they might do | class |
| --- | --- | --- | --- | --- |
| 1 | payload holds one seed | door lead `A seed is loaded. It can fill any number of slots.` — no count, which is §7a's rule and is implemented (1866-1876) | — | DEFAULT (correct) |
| 2 | Build → `Taproot (tr)` → two single-key paths | `composerSlotOrder` extracts the first-listed unlocked, unhashed one-key path as `@0` and pins it against `md.Composed.Slots()` (5368-5400, 5403-5440) | this is the right mechanism and it is checked, not assumed | — |
| 3 | seat `@0` and `@1` from the one seed | accounts by ordinal per MASTER → `m/48'/0'/0'/3'` and `m/48'/0'/1'/3'` (5713-5735); `@0`'s prompt is §8s's key-path form | **the C29 warning does NOT fire**, and that is correct: the two slots are in different paths, so it is C5's normal case → §8k's informational line plus the two-paths note (6294-6312). The brief's expectation of C29 here is what the walk corrects | DEFAULT (correct) |
| 4 | Full vs Watch-only | `composerEngraveModePick` reuses `buildFullModeLabel` (7116-7130) — **never called from any flow** | | **C-1** |
| 5 | what is cut once | §7f: "a seed that filled several slots is cut ONCE". **No code in the plan implements the once-only rule** — `composerFormsFor`/`composerSecretFormPick` return a choice and nothing consumes them, and no task walks the registry to cut each master once | | **C-1** |

---

## J4 — the confused operator

| # | what they do | what the device does | class |
| --- | --- | --- | --- |
| 1 | opens **Engrave Multisig** expecting to build a wallet | it still works, unchanged; the deprecation is a source comment only (1587-1602) | DOCUMENTATION — C7 rules it, not a finding |
| 2 | types a passphrase for the seed | `Passphrase seed 1` → `Skip` / `Add passphrase`, bound per seed (5678-5695); `buildFullModeLabel` names what Full leaves out | DEFAULT (correct) — but unreachable, **C-1** |
| 3 | writes the stub down, then changes a **lock** | id genuinely moves (locks enter it); stub re-shown with §8s | DEFAULT (correct, §7c) |
| 4 | then changes a **key count** | §8j confirm, seats discarded, stub re-shown | WARNING (correct) — but see **I-1** for the lock case |
| 5 | Backs out of the stub screen without editing, then continues | §8s asserts the id changed when it did not | **M-1** |
| 6 | compares the shown `Template-ID` with a coordinator's | the stub screen carries §8d; the consent screen does not | **M-5** |
| 7 | reads `Template-ID:` on the shipped template-engrave screen | relabelled to `mk1 stub (template):` at both `:70` and `:79` — the ambiguity that would make a match read as a mismatch is removed (4332-4345) | DEFAULT (correct, and a good catch by the plan) |

---

## Findings

### C-1 — Part B's screens are built and never joined to any flow; a payload's keys can never be seated

Plan line **4437**: *"Part B replaces `gui/composer_flow.go` wholesale (the gate's ``Replace `gui/composer_flow.go` `` anchor) to insert seating between the stub screen and consent."* **No task in the plan provides that replacement.** `grep -n 'Replace \`' ` over the plan returns only line 54 (the gate's own description) and 4437 itself; `composerFlow` is defined once, at 4869, in Part A, and its only call site is `walletPolicyFlow` (1946).

Machine-checked in the wired gate scratch (production files only, declarations counted):

```
DEAD-IN-PROD: composerApplyShapeEdit, composerCardSources, composerCensusLines,
composerCensusRefusal, composerConsentFlow, composerEngraveModePick,
composerFormPick, composerKeySources, composerMappingReview, composerMintCards,
composerSeatFlow, composerSeatingComplete, composerSecretFormPick, composerShortfall
```

(14 functions with exactly one production reference — their own declaration.
`composerSeedSource`, `composerSeedDerive`, `composerMintCard` and
`composerDiscardAssignments` are transitively dead behind them.)

**What the operator gets.** J2 step 5: four `key:` records in flash, a 2-of-3 plus a
recovery path composed, and the device goes stub screen → consent → §8l → census →
cut, engraving a **key-less template**. The path list even reads `keys available: 0`
throughout, because nothing populates `st.sources` (2563-2585 counts a slice only
`composerSeatFlow` ever appends to, at 6552).

**What else it takes down.** §7e's self-check and its §8q refusal never execute in
production — the gate that exists "so a builder defect … cannot reach steel as a
reviewed wallet" is a hypothesis, exactly the class CLAUDE.md's lens-closure rule
forbids closing a plan on. §7f's form choice, Full/Watch-only, the secret plate form,
card minting and the census's card-chunk count are all unreachable, so §12 items 6 and
9 have nothing to run against and §12 item 2's journey cannot be walked.

The plan's own self-review (7609+) maps "slot-directed seating from the payload with
the paged pick list" to *"the paged-primitive task and the sources task"* — neither of
which calls anything. The gate could not catch this: every `TestComposer*` calls the
Part B functions directly.

**Fix shape:** one task, after B7/B8/B9, replacing `gui/composer_flow.go` with the
version that loads sources, runs `composerSeatFlow`, `composerShortfall`,
`composerMappingReview`, `composerConsentFlow`, `composerFormPick`,
`composerMintCards` and `composerCensusLines` in §7's order — and a walk test that
drives `walletPolicyFlow` from a keyed payload to the engrave screen, since only a
flow-level test can fail on this.

### C-2 — "Sorted (usual)" is silently reversed the moment a second path is added

`composerKeysEdit` (2746-2775) offers `Key order` → `Sorted (usual)` / `Keep my order`
whenever `composerSortedIsLegal` holds, which for the first path of a fresh list it
does. §5's key-set row then lowers a **sole** unlocked, unhashed n≥2 path to
`sortedmulti` and **any other multi-key path to `multi`** — so adding a second path
(J1 step 9, the plan's own worked shape) turns the answered-sorted path into an
order-dependent `multi`.

Nothing tells the operator. `composerBranchLines` gates the `UNSORTED (EXPERIMENTAL)`
mark on `sole` (4736-4766) — correctly, per §5a, since the operator declined nothing —
so the consent surface says only `Path 1: 2-of-3`. §8b's body exists because this
property matters: *"Key order is part of this wallet. Anyone restoring it must keep the
same order."* The operator lands in that state holding the opposite reassurance from a
screen this plan adds.

Mitigating: the engraved md1 records the order, so a restore from the plates is
unaffected; the exposure is a hand re-entry into a coordinator, and the false belief
itself. It is still a decision the device asked about and then reversed in silence,
which is the Critical bar.

**Fix shape, inside §7b's screens:** ask the key-order question at the transition out
of the shape (where `sole` is final) rather than during the first path's edit; or keep
it where it is and re-state the outcome on the path-list row and the consent line
(`2-of-3, key order matters`). Do **not** fire §8b — §5a is right about that.

### I-1 — §8j's confirm blocks the lock and hash edits §7g rules a DEFAULT

`composerShapeFlow` calls `composerShapeGuard` before entering the path editor (2877)
and before `composerAddPath` (2882). The path editor's four arms are `Keys`,
`Time lock`, `Hash lock`, `Remove path` (2830-2853), and two of them renumber nothing.
§7d: *"A lock or hash edit moves no slot, keeps assignments"*; §7g classifies it
`DEFAULT: assignments kept`.

So after seating, an operator who wants to change a lock is told **"EDITING THE SHAPE
CLEARS THE KEYS … Every key you seated will be cleared. Continue?"**, which is false for
the edit they intend. If they believe it and decline, `composerShapeGuard` returns
false → `continue` → the path editor is never reached, and **the lock cannot be edited
at all** without accepting a warning that misdescribes what will happen. The plan
argues the position at 2670-2680, but its own rule two lines earlier — *"A warning that
fires when nothing is at stake is one the operator learns to tap through"* — cuts
against it.

**Fix shape:** move the guard inside `composerPathEdit`, onto the `Keys` and
`Remove path` arms only; `composerApplyShapeEdit`'s signature comparison already makes
the discard itself exact.

### I-2 — the §4f invariant check treats unseated slots as colliding, refusing both legal key-less forms

`composerInvariantViolation` (6194-6220) iterates **every** entry of `st.assigned`,
including unseated ones (`src < 0`), whose `origin` is nil. `composerOriginKey(nil)`
is `""`, so two or more unseated slots group together with no fingerprints and the
function returns true. `composerMappingReview` then refuses with §8v — *"Two keys
declare the same origin and not both carry a fingerprint"* — about keys that are not
there, and `composerSelfCheck` (6921) returns an error, which
`composerConsentFlow` renders as §8q.

Compounding it, `composerSelfCheck` at **6891** fails whenever
`len(keys) != len(st.assigned)`, and `st.assigned` is sized only inside
`composerSeatFlow` (6496-6502) — so a composition that never entered seating (C26's
key-less template, §12 item 3) fails the check on slot count alone.

Both of §7f's legal key-less artifacts — the C26 template and the §8p partially-seated
fallback the shortfall screen explicitly offers (6577-6590) — therefore hit a refusal
whose copy tells the operator to "start again". Latent only because C-1 leaves the
path unreachable; it becomes live the moment C-1 is fixed.

**Fix shape:** skip `src < 0` entries in the invariant scan (the unseated slots' §4f
lowest-free-account origins are the codec's to assign and are already asserted
distinct by the stub-screen test at 4085-4100), and size `st.assigned` at flow entry
rather than at seating entry.

### I-3 — the paged pick list can take a row the operator cannot see

`composerPickScreen` (943-1010). On Button2 the page advances or wraps to `start = 0`,
after which the only cursor clamp is upward: `if sel < start { sel = start }` (992).
There is no `sel >= start + shown` clamp, so after a wrap the cursor stays on a row
belonging to a later page. That frame draws **no highlight at all**, and Button3
returns `sel - rowBase` — the invisible row.

On the seating list this seats a key the operator never saw selected (caught later at
the mapping review only if they read it); on the hashlock list it selects a different
digest. Up/Down handle the same situation correctly (`start = sel`, 995-999), so this
is the one path that does not.

**Fix shape:** after the page advance, clamp `sel` into `[start, start+shown)` the way
Up/Down clamp `start` into the cursor.

### I-4 — the consent screen can be confirmed before its proof is drawn

`composerReadScreen` (902-940) — and the shipped `confirmReviewScreen` §7e names
(`gui/multisig_build.go:1897-1905`) — accept Button3 on the first frame, whatever page
is showing. §7e's surface carries per-path lines, the key-path line, both ids and
**receive and change addresses 0..1**, and the plan's own measurement task (7445-7530)
expects eight paths plus four addresses to need several pages.

So the operator can consent to a wallet whose addresses — the only thing that proves
which wallet it is — were never rendered. §8l follows and is unskippable, but it
carries no policy content.

Inherited behaviour, but `composerReadScreen` is new code in this plan and the
composer's consent is the surface §7e created *because* the shipped ones did not say
enough. Multisig Build's own review has the same shape; scoping a fix to the
composer's consent is legitimate.

**Fix shape:** withhold the checkmark on `composerReadScreen`/`composerConsentFlow`
until the last page has been laid out once (`start + shown >= len(lines)` has held), or
state explicitly in the plan that partial-page consent is accepted and why.

### I-5 — Back at the `Key order` screen destroys the path's key set

`composerKeysEdit` writes the new `KeySet` at 2751 and then, if `Choose` returns
`!ok`, sets `st.list.Paths[idx].Keys = nil` (2754) — and does the same when the §8b
confirm is declined (2762). Reached from `composerAddPath` that is right (the path is
truncated), but `composerPathEdit` ignores the return value (2841), so **editing an
existing sole path and pressing Back at the last screen leaves the path with no keys**.

The path-list row then reads `Path 1: hash only` — because `composerPathLine`'s default
body is `"hash only"` whenever `Keys == nil`, regardless of `Hash` (2511) — and `Done`
refuses with §8m line 2, *"A path with only a time lock means anyone can spend after
it"*, naming a lock that was never set. The operator has no way to see what is wrong
with Path 1.

Violates the plan's own Global Constraint (**Back preserves everything**, 2026-08-19
directive) and §7b's *"Back preserves everything (going back should lose nothing)"*.

**Fix shape:** snapshot `Paths[idx].Keys` on entry and restore it on any decline; and
give `composerPathLine` a distinct body for `Keys == nil && Hash == nil`.

### I-6 — a date past 2038-01-19 is refused with "that date does not exist"

`composerLockEdit`'s date validator (3640-3652): outside the band and `y >= 2009` →
`"that date does not exist"`. `composerDateToUnix` (3486-3505) caps at
`composerDateCeilingUnix = 2147472000` (2038-01-19), which §4c's time row makes
correct — but 2045-06-01 exists, and the message says it does not.

The archetype §4d lists first is **simple-timelocked-inheritance**; a 20-year absolute
date is the ordinary use. The operator retypes, gets the same sentence, and never
learns there is a ceiling or that a block height reaches further. §8t covers the floor
and §8 has no body for the ceiling — a gap in the copy the plan inherits rather than
creates, but the plan is where a sentence gets invented.

**Fix shape:** a ceiling line naming the limit and the alternative, e.g. *"This build
writes dates up to 2038-01-19. For later, use a block height."* — and file the §8
addition so the copy stays enumerable under Task A1's AST scan.

### M-1 — the changed-id line fires after a Back with no edit

`composerFlow` (4884, 4902, 4911, 4917) sets `edited = true` on **any** Back out of the
stub screen, the consent or §8l, and never resets it. Re-reaching the stub screen
without touching the shape therefore asserts §8s: *"The shape changed, so this id
changed. Cards minted with the old stub will not seat here."* It is the safe direction
— but it is a false statement on the screen whose job is to be copied onto steel, and it
trains the operator to discount the line that will one day be true. Comparing the new
chunk set against the previous one is exact and free.

### M-2 — an empty digit field echoes the §8u ceiling refusal

The blocks and days validators (3605-3628) return `composerCopyRelativeCeiling()` for
every unparseable fragment, including `""`. Before typing a digit the operator reads
*"Relative locks reach at most 455 days in blocks or 388 days in time. Use an absolute
date."* — a refusal for a limit they have not approached. The date and height pads get
this right ("eight digits, YYYYMMDD", "1 to 499999999"); the two relative pads should
say what to type before they say what is too much.

### M-3 — `ErrComposeNoPaths` has no §8m mapping, so an empty list shows a codec string

`composerRefusalBody` (2620-2640) maps five sentinels; `md.ErrComposeNoPaths` is in the
task's Consumes list (1999) but not in the map, so `composerShowRefusal` falls through
to `err.Error()` — measured in the S2 tree as `md: compose: a wallet needs at least one
spend path` (`md/compose.go:91`). Tapping `Done` on an empty list is a plausible first
action; §4e's first row maps that case to §8m line 1 (*"Every wallet needs at least one
path with a key."*), and §11 requires every §4e refusal to name what to do instead and
print no encoding — `md: compose:` is an internal prefix on an operator screen.

### M-4 — `A hash, no keys` → `No hash lock` leaves an empty path labelled "hash only"

`composerAddPath` (2795-2823) truncates the path only when `composerHashEdit` returns
false; the pick list's last row `No hash lock` (3960) clears the digest and returns
**true**. The operator who chose the key-less route, held through §8a's EXPERIMENTAL
confirm, and then picked "No hash lock" keeps a path with neither keys nor hash. The
row and the editor lead both read `Path N: hash only` (2511) and `Done` refuses with the
lock-only body. Same display defect as I-5, reached a different way; treating a path
that ends with neither element as a cancel would close both.

### M-5 — §8d's own-wallet line is not on the consent surface

`composerConsentLines` (4771-4840) prints the id, both stub labels and the addresses,
but not §8d. §7g's divergence table answers **`consent | compares the shown id with a
coordinator's`** with *"DOCUMENTATION: §8d line; a composed wallet is its own wallet"* —
i.e. at the consent step. The plan prints §8d only on the stub screen (4295),
several screens and possibly several edits earlier. One line, already written and
already gated.

### M-6 — the seating prompt disappears from page 2 of the pick list

`composerPickScreen` puts `lead` in `lines[0]` and a spacer in `lines[1]` (948), so the
§8s prompt (*"Slot @2, Path 1 key 2 of 3: choose a key"*) is a body row and is absent
from every page but the first. The title is the constant `"Seat keys"`. With a payload
holding more keys than a frame — the case the primitive exists for (§9 item 7) — an
operator paging for the right key loses which slot they are filling, and paging is
forward-only with wrap, so recovering it costs a full cycle. Drawing the lead as a
per-page header rather than as row 0 is the same measurement loop.

### N-1 — Back at the wrapper picker exits the whole program

`composerWrapperPick` returning `!ok` makes `composerFlow` return (4872-4876), and
`walletPolicyFlow` returns immediately after calling it (1946), so the first Back
inside Build drops the operator to the carousel rather than to the door one level up.

### N-2 — the slots/keys line is drawn with no payload

`composerSlotsKeysLine` is the path-list lead unconditionally (2870); §7b scopes it to
*"whenever a payload is loaded"*. With no payload it reads `keys available: 0` on a
build the door has already described as key-less.

### N-3 — no way to reorder paths, and order is normative

§5 makes listed order decide the `or_i`/`or_d` nesting, leaf depth on the taproot spine
and which path becomes the internal key. `composerPathEdit` offers `Remove path` but no
move (2830-2853), so fixing an order mistake means removing and re-adding — which under
§8j discards every seat.

### N-4 — the date pad's echo appends the raw operand

`composerCopyLockEchoDate(y, m, d) + " (" + u + ")"` (3652) draws
`2027-03-01 00:00 UTC (1803859200)`. §6b's premise is that *"the operator never types a
raw operand"*; the confirm screen after it shows §8c's clean form, so the number adds
nothing the operator can check.

---

## What the walk found sound, so a later round does not re-derive it

- The door (§7a/§8r): all six key-state lines, counted through `has()` rather than
  `take()`, `From payload` correctly conditional, and the five broken shipped walks
  found by a **sharded** run and updated rather than skipped (4950-4990). Good work.
- §4c enforcement on the device's own side, including the `older(0x400000)` zero-units
  case md still accepts, with every boundary value in and out (3260-3320).
- Impossible dates caught by a `time.Date` round trip rather than a month-length table
  (3570-3580).
- `composerSlotOrder` checked against `md.Composed.Slots()` instead of assumed
  (5403-5440) — the one place a silent mis-seat could have hidden.
- The `Template-ID:` relabelling on both shipped occurrences (4332-4345): two values
  under one label was a real way to read a match as a mismatch.
- C29 vs C5: the warning fires inside one path and the informational line across paths
  (6254-6292), which is the correct reading of §7d and §8g.
- §8p guesses no cause, and its test forbids the words that would (6412-6432).

## Where a different answer to the author report's §5 questions changes a step

- **If Part A does NOT ship alone**, J1 step 18 stops being a documentation call: the
  unified flow must decide whether the key-less template goes through
  `composerConsentFlow`, which is where **I-2** turns from latent into blocking.
- **If the §7f secret-form split is ruled IN (F-455)**, J3 step 4 gains a third row and
  a new plate layout; nothing else in the walk moves.
- **If presets land (F-453)**, J1 step 3 gains a `preset or blank` screen between the
  wrapper and the path list (§7b), and **C-2** gets worse: a preset populates a
  multi-path list directly, so a `Key order` question asked afterwards on a
  non-sole path would be doubly moot.

---

**Closing counts: 2 Critical (C-1, C-2) / 6 Important (I-1 … I-6) / 6 Minor (M-1 …
M-6) / 4 Nit (N-1 … N-4).** C-1 alone blocks the plan: §12 item 2's journey and §7e's
self-check are gates that cannot run against what this plan builds.
