# Architect R0 (v3) — PLAN_key_index_legibility.md @ 5b786d4

Reviewer: independent read-only architect agent, 2026-08-19. Third round. Same
two questions — (a) can a competent implementer execute this without inventing
anything, (b) is any stated assumption factually wrong. No redesign. Prior
rounds: `plan-key-index-legibility-architect-r0.md` (v1) and
`…-architect-r0-v2.md` (v2).

**Everything below marked "measured" was run on this machine** (24 cores)
against the release binaries (`mnemonic-toolkit/target/release/mnemonic`,
`descriptor-mnemonic/…/md`, `mnemonic-key/…/mk`, `mnemonic-engrave/…/me`). The
tool shell here is fish and does **not** word-split unquoted `$VAR`; every
command was run through `bash -c`, as the journey scripts do. The fixture is not
a hand rebuild this round — **I ran `transcript_pathological.sh` and
`transcript.sh` end to end** and used the `out/pathological/` artifacts they
produced. Nothing tracked was modified (`out/` is git-ignored,
`design/journeys/.gitignore:9`).

---

## Verdict

**(a) No — but it is close, and the residue is two decisions, not twelve.**
v3 is a genuine improvement over v2 in kind, not just in degree: the impossible
`§1` emitter is deleted, the recovery target's producer is real and fast, §2 is
specified to the byte in five forms, §3 names its shuffle unit and its exit
codes, and §4 has a file, a heading, a sentence and an acceptance. Round 0 found
12 forced decisions, round 1 found 13; **this round finds 9, of which 2 are
Critical.** Both Criticals share one shape: the plan states a rule whose two
plausible readings produce **different committed bytes**, and then supplies an
acceptance criterion that **passes under both**.

- **§1 does not say whether `--mk1` is passed to `verify-bundle` — and §0's own
  command sketch says it is.** Measured, that shape returns
  `✗ mk1_template_stub_bind` / `mismatch` / **exit 4**. Without `--mk1`: `OK` /
  exit 0. §1's acceptance ("prints the id and the warning") is satisfied by
  both, so the gate cannot see the difference. An implementer copying §0 ships a
  red step into a published operator journey.
- **§2 does not say whether collision uniqueness is scanned over PLATES or over
  CARDS.** Read literally (per plate), every multi-chunk card collides with
  itself: **30 of 30** pathological plates and **2 of 2** golden mk1 plates gain
  a `set 0x…` suffix. Read per card: **0 of 30**. §2's own specified assertion
  uses `contains`, so it passes either way. This is v2's D5 scope half, which
  the §8 ledger records as resolved.

**(b) Yes — one Critical false claim, and §0's headline block does not reproduce
its own implied output.**

- **`--accept-search-time 4000` does not "remove the estimate gate".** Measured:
  `error: --accept-search-time 4000s is below the estimated exhaustive time
  2087963.184384s; restate a duration ≥ the estimate`, exit 1. `cap_decision`
  (`permutation_search.rs:388-415`) consults the flag **only** when
  `estimate > SEARCH_CEILING`, and then demands `accepted >= estimate`. The flag
  moves the ceiling 3600 s → 4000 s — **+11 %** — against a measured estimate
  spread of 187 s → 1285 s. §3's stated rationale ("so the gate is the *actual*
  work (≈16 s), not a variable estimate") and ledger row D9 ("moot —
  `--accept-search-time` removes the estimate gate") are both false.
- §0's `--mk1 ×2` block, above.

**The good news is large and I want it recorded so a fourth round does not
re-derive it.** v2's two Criticals are genuinely dead. And the dependency that
blocked §1/§2/§3 in both prior rounds — F-210, "the operator journeys cannot be
regenerated" — **is gone, measured**: both scripts now run clean and reproduce
their committed transcripts **byte-identically**.

```
$ bash transcript_pathological.sh > /tmp/tp.txt 2>&1 ; diff transcript_pathological.txt /tmp/tp.txt
(no output; script exit 0; 3 non-zero inner exits, same as committed)
$ bash transcript.sh > /tmp/t.txt 2>&1 ; diff transcript.txt /tmp/t.txt
(no output; script exit 0; 1 non-zero inner exit, same as committed)
```

So v2's H1 and MA-4 are closed **by execution**, not by assertion — the repairs
landed in `b822e4a`, `6a42d89`, `c6c6943`, `e59ce9f`, `3cba69b`, all after F-210
was filed at `ee2e6e3`. (`design/FOLLOWUPS.md:7561`'s heading still reads
"cannot be regenerated" — stale bookkeeping against the repo's own
status-in-the-heading convention, `FOLLOWUPS.md:5-20`. Not this plan's defect,
but worth one line to whoever owns F-210.)

---

## Ledger audit

Row-by-row against the body. 36 rows. **28 genuinely resolved (or explicitly and
legitimately rejected), 5 partial-claimed-as-resolved, 3 false.**

| finding | claimed disposition | actual | verdict |
| --- | --- | --- | --- |
| D1 type | §2: `String`, with rationale | §2 "Decided: representation" gives `String` + why. Verified: `crates/me-cli/Cargo.toml:22-27` = md-codec, mk-codec, clap, zeroize, serde, serde_json — no `bitcoin`; `Display` needs no type name. | **RESOLVED** |
| D2 manifest fields | §2: yes — `card_fingerprint`, `card_path`; golden + SPEC change | Both named. No key collision with the existing set (`plate, of, kind, string, chunk_set_id, chunk_index, integrity, preview` — `manifest.rs:41-57`). | **RESOLVED** |
| D3 `None` fallback | §2: exact string given | `mk1 [path 48'/0'/1'/2', no fingerprint] chunk 1/3`. Case reachable — measured: `mk encode --privacy-preserving` → `origin_fingerprint: (omitted, privacy-preserving mode)`. | **RESOLVED** |
| D4 empty path | §2: exact string given | `mk1 [73c5da0a, no path] chunk 1/3`. Reachable — measured: a **depth-0** xpub with `--origin-path m` encodes (exit 0) and decodes with `origin_path:` empty (`cat -A` → `origin_path:         $`). | **RESOLVED** |
| D5 collision | §2: `set <chunk_set_id>` suffix, placement specified | Placement **is** now fixed ("after the chunk counter"). **Scope is not**, and scope is the whole question — see C2 below. Row claims more than the body delivers. | **FALSE (partial as resolved)** |
| D6 multi-wallet / multi-stub | §2: irrelevant; ignored deliberately | An explicit, recorded rejection. Legitimate. | **REJECTED (explicit)** |
| D7 restore invocation | §3 + §0 transcript give the full command | True for §3's `restore` (shuffle unit, target, exit codes, floor). **Not true for §1's `verify-bundle`** — §0's shape is the one §1 inherits and it is wrong (C1). | **FALSE for the §1 half** |
| D8 which target | §3: `--expect-wallet-id` | Measured: 8 hex → exit 4 floor mismatch; 16 hex → exit 0. | **RESOLVED** |
| D9 which number decides | §3: moot — `--accept-search-time` removes the estimate gate | **Refuted by execution.** See FI-1. | **FALSE** |
| D10 doc location | §4: named file and heading | `design/journeys/README.md` has no wallet-creation section (headings measured: lines 1, 12, 24, 41, 49, 69, 92, 107, 123, 164) — §4 creates one, named. | **RESOLVED** |
| D11 canonicalisation | §4: encoded-template index; measured identity here | Concept correct (`canonicalize_placeholder_indices` exported at `md-codec/src/lib.rs:46`) and the identity claim re-measured: the fixture policy's placeholders are literally `@0 @1 @2 @3 @4 @5 @6 @7 @8 @9 @10` in first-occurrence order. Residue: it never says which **command** shows a reader the canonical index. | **RESOLVED (Minor residue)** |
| D12 release mechanics | §2: version bump + CHANGELOG | Restated, not sized (no target version), and it silently drags in a `me-preview` rebuild — see H-A. | **PARTIAL** |
| F1 seed required | §0 | Measured: exit **2**, `restore of a keyless MULTISIG TEMPLATE md1 requires --from <seed>`. | **RESOLVED** |
| F2 `--search-address` refused | §0 | Reproduced 5× this round, exit 1. | **RESOLVED** |
| F3 wrong target id | §1 rewritten around `verify-bundle` | Measured: `wallet-policy-id: ced2270948ecb5af0779249ac7181f4a`, 6 runs at 0.004–0.006 s. | **RESOLVED** |
| F4 dependency conditional | §2: `String` makes it unconditional | Correct. | **RESOLVED** |
| F5 manifest contract | §2: golden + SPEC named | `SPEC_me_bundle_phaseA.md:61` **is** §6 and **is** the manifest schema — correct section. But `SPEC` **§7** (`:86-94`) prints the checklist example `plate 2/4  mk1 chunk 1/2` verbatim, and §2 falsifies it. Unnamed. | **PARTIAL** |
| F6 `@N` reason over-broad | §2: claim narrowed | §2 now says only "a keyless template carries no key order" and "the label states the card's **origin**, never a slot number". True for this fixture. | **RESOLVED** |
| F7 fixture correction correct | §4 limit 2 | Re-measured: `key-04` = `b8688df1` / `48'/0'/0'/2'`. | **RESOLVED** |
| F8 recoverable-shape claim correct | §0 | Measured: explicit mode exit 0. | **RESOLVED** |
| MA-1 collision untested | §2 acceptance: a test | A test is demanded, and the fixture **is** buildable — measured: two cards sharing `(73c5da0a, 48'/0'/0'/2')` with different xpubs both encode and `me bundle` accepts them (5 plates, exit 0). But the test cannot distinguish D5's two scopings, so it does not close D5. | **RESOLVED, weak** |
| MA-2 "deliberately" not a criterion | §2 acceptance: exact assertion text | An exact string replaces the intent — real progress. But the target, `manifest.rs:228`, sits in a unit test whose plates are **hand-built with no card** (`manifest.rs:192-223`, `string: Some("mk1a".into())`). The assertion cannot hold without an unstated fixture edit, and `contains` makes it pass under both D5 readings. | **PARTIAL** |
| MA-3 golden/schema undecided | §2: both change | Golden confirmed to need regeneration: `tests/vectors/bundle-md1-mk1.json` plates 2–3 are `mk1-chunk` carrying a real card (`mk decode MK1_A MK1_B` → `origin_fingerprint: aabbccdd`, `origin_path: 48'/0'/0'/2'`, `chunks: 2`), pinned by `assert_eq!(v, expected)` at `cli.rs:308` and `:745`. The regenerated **content** still depends on D5. | **PARTIAL** |
| MA-4 other journey | §2 acceptance: both transcripts | **Resolved and now satisfiable** — both regenerate byte-identically (measured, above). Counts confirmed: `grep -c 'mk1 chunk'` → `transcript.txt` **24**, `transcript_pathological.txt` **31**; checklist plate lines **24** and **30** (the 31st hit is prose at `transcript_pathological.txt:71`). | **RESOLVED** |
| MA-5 §4 no deliverable | §4: deliverable + sentence + acceptance | Correct; round 0's harshest finding is fully closed. | **RESOLVED** |
| MA-6 which exit code | §3: all four named | 0/1/2 correct (measured). "`4` NO MATCH" is **incomplete**: exit 4 is also the prefix-floor refusal, and the accept-too-low refusal (exit **1**) is not named — and it is the outcome §3's own decision creates. | **PARTIAL** |
| MA-7 prefix floor | §1 + §3: 16 hex | Measured both sides. | **RESOLVED** |
| H1 §3 prerequisite | §3: dependency on §1 | Stated in §3 and enforced by acceptance ("consumes **that printed value**, not a literal"). | **RESOLVED** |
| H2 two transcripts | §2 acceptance | See MA-4. | **RESOLVED** |
| H3 double regeneration | §6: regenerate once after §1 and §3 | Stated. | **RESOLVED** |
| H4 §2 decision not orderable | resolved — the decision is made in §2 | Correct. | **RESOLVED** |
| v2-C1 `verify-bundle` prints it | §1 rewritten | **Fixed.** The impossible-homes false choice is deleted; the producer is real, and measured at 4 ms. | **RESOLVED** |
| v2-C2 root cause disproven | §0: origin-flattening + omitted fingerprints | **Fixed**, and the fingerprint half independently re-measured this round: all 11 cards re-encoded `--privacy-preserving` → `wallet-policy-id: d09840c3aa78035368f2dfb4bc271a27` with **identical** `first recv: bc1qkuknuy6…ejms64`. | **RESOLVED** |
| v2-I2 estimate variance | §3: `--accept-search-time` | The remedy does not do what the row says. | **FALSE** |
| v2-I4 empty-path card | §2 rendering table | See D4. | **RESOLVED** |
| PDF already captions `@N` | noted: the document shows it; the checklist does not | **True, verified:** `design/journeys/build_pdf_pathological.py:301` → `f"plate {n} — @{ki} [{fp}/{path}] chunk {nth}/{tot}"`, while `transcript_pathological.txt:140-169` carries `mk1 chunk 1/3` with no origin. | **RESOLVED** |

### The ledger's own completeness claim is slightly false

The header says *"every prior finding, resolved or explicitly rejected"* and
§0's preamble says *"§8 is a ledger of every prior finding, so none can be
silently dropped again."* 36 rows is the count of the **labels it chose to
carry**, not the count of findings. Silently absent:

- **architect-v2 D8** — where the `--privacy-preserving` test fixture comes
  from. No row; unaddressed (see decision 8).
- **architect-v2 D10** — `--from phrase=<seed>` on argv. No row; **addressed in
  substance** (§0 now shows the piped `--from phrase=-` form).
- **architect-v2 FI-4 / critique-v2 N1** — single-run calibration constants
  quoted as measurements. No row, **and the defect recurs verbatim in §0** (see
  FI-3).
- **critique-v2 N2** — the `KeyCard` citation points at the checkout while the
  build links registry `mk-codec 0.4.1` (`Cargo.lock`). No row; unchanged.
- **critique-v2 M1** — §5's rationale. No row, but §5 now carries an
  "Acknowledged tension" paragraph that engages it. Substantively addressed.
- **critique-v2 "what v2 still misses" #5** — the `tr()` fixture variant
  (`inputs-pathological/wallet-policy-tr.txt`), unexercised in all three rounds
  including this one. No row.

A ledger is the right instrument and I would keep it. But a row saying
"resolved" where the body resolves half (D5, D7, D12, F5, MA-2, MA-3, MA-6) is
the failure mode the brief names: it *looks* like completeness. Three rows are
outright wrong (D5, D7's §1 half, D9/v2-I2 — the last two being the same claim).

---

## Prior Criticals — fixed or not

### v2-C1, the impossible-homes false choice — **FIXED**

v2 escalated *which home* should emit the recovery target and offered two
candidates, neither of which could compute the value. v3 deletes the escalation
and the feature. Measured, the capability exists and is fast:

```
$ printf %s "$SEED" | mnemonic verify-bundle --network mainnet --from phrase=- \
    --account 0 <3 md1 chunks positional> <28 --cosigner @1..@10= chunks>
warning: explicit --cosigner @N= mode builds the wallet from the ASSERTED key→slot
  assignment WITHOUT verifying it against a recorded id/address. …
✓ md1_template_match: supplied md1 matches the recomposed wallet's keyless template
✓ wallet_completed: completed WalletPolicyId ced2270948ecb5af0779249ac7181f4a;
  first receive bc1qkuknuy6dsm0fq44cyyhzqy9wl3ex2n6ed39zxhx867l9wlh4yhlsejms64
OK (multisig template recomposed)
wallet-policy-id: ced2270948ecb5af0779249ac7181f4a
your seed completes cosigner slot @0
exit=0
```

Six timed runs: `0.004 0.006 0.004 0.004 0.004 0.004` s. **§0's "4 ms" is
accurate** — the one precisely-quoted constant in the document that reproduces.

### v2-FI-1, "existing manifest consumers are unaffected" — **FIXED**

v3 replaces it with the opposite and correct statement (§0: *"the manifest
golden is byte-pinned by two full-JSON-equality assertions … `skip_serializing_if`
does not help, because the golden's mk1 plates **have** cards"*). Verified in
all four parts the brief asks about:

- **Does the golden really need regenerating?** Yes. `cli.rs:308`
  (`assert_eq!(v, expected);`) and `cli.rs:745`
  (`assert_eq!(v, expected, "no --preview must be byte-for-byte Phase A");`),
  both against `include_str!("vectors/bundle-md1-mk1.json")`. That file's plates
  2 and 3 are `"kind": "mk1-chunk"` carrying `MK1_A`/`MK1_B`, which decode to a
  real card (`aabbccdd` / `48'/0'/0'/2'` / 2 chunks). Both gain keys; both
  assertions fail.
- **Are `card_fingerprint` / `card_path` sufficient names?** Yes —
  unambiguous, no collision with the eight existing `PlateEntry` keys
  (`manifest.rs:41-57`). Nit only: they diverge from the codec's own
  `origin_fingerprint` / `origin_path` (`mk-codec-0.4.1/src/key_card.rs:40,46`,
  identical in the checkout), which is what `mk decode` prints — so a consumer
  correlating the two sees two names for one field, and `card_path` is really an
  *origin* path. Not a decision; the plan names them.
- **Does `SPEC_me_bundle_phaseA.md` §6 exist, and is it the right section?**
  Yes. `design/SPEC_me_bundle_phaseA.md:61` = `## 6. Manifest schema (JSON,
  stdout/--manifest)`, and lines 62-84 write the schema out field-by-field. It
  is exactly the right target.
- **What §2 misses:** SPEC **§7** (`:86-94`) is *"Guided workflow checklist
  (stderr)"* and contains the literal example line `plate 2/4  mk1 chunk 1/2
  → push via NFC & engrave`. §2 changes precisely that string and names only §6.

### §2 — is it now implementable without invention?

Mostly yes; the rendering half is genuinely done. Checked as the brief asks:

- **The specified assertion string against what the golden's card decodes to.**
  ```
  $ mk decode "$MK1_A" "$MK1_B"
  origin_fingerprint:  aabbccdd
  origin_path:         48'/0'/0'/2'
  chunks:              2 (long)
  ```
  So `mk1 [aabbccdd/48'/0'/0'/2'] chunk 1/2` is the **right string for that
  card**. But the assertion §2 names lives at `manifest.rs:228`, inside
  `checklist_lists_public_plates_and_ms1_reminder` (`manifest.rs:180-232`),
  whose plates are **constructed by hand** at `manifest.rs:192-223` with
  `string: Some("mk1a".into())` / `Some("mk1b".into())` — there is no mk1 string
  to decode and no card. The `aabbccdd` value was taken from a *different* test
  in a *different* file (`tests/cli.rs:5-7`). The assertion is achievable, but
  only by also inventing `card_fingerprint: Some("aabbccdd".into())` /
  `card_path: Some("48'/0'/0'/2'".into())` into that fixture — a step §2 never
  mentions (decision 6).
- **`'` vs `h` for `DerivationPath`'s Display.** `'` is right.
  `bitcoin-0.32/src/bip32.rs:210-221` — `ChildNumber`'s `Display` writes
  `f.write_str(if alt { "h" } else { "'" })`, `alt = f.alternate()`; and
  `:459-471` — `DerivationPath`'s `Display` joins with `/` and emits **no
  leading `m/`**. So `format!("{}", card.origin_path)` yields `48'/0'/0'/2'`,
  matching `mk decode`. `{:#}` would yield `h`. §2's "Hardened markers `'`,
  matching `mk decode`" is correct.
- **The two other rendering forms are reachable**, so none of the five is dead
  spec (measured, D3/D4 rows above), and the collision fixture is buildable
  (MA-1 row).

### §3 — the `--accept-search-time 4000` question, four parts

- **Does the flag exist in that form?** Yes. `restore.rs:196-197`
  (`#[arg(long = "accept-search-time")]`), mirrored on `verify_bundle.rs:160-161`.
- **Does it take seconds?** Yes, with no suffix. `parse_search_duration`
  (`restore.rs:2185-2203`) strips `min`/`h`/`m`/`s` and falls through to
  `(s, 1)` — *"No suffix → seconds"* (`:2183`). `--accept-search-time 4000` =
  4000 s. Note the clap help (`restore --help`, line 75) advertises only
  *"a humantime duration (e.g. `2h`, `90min`)"*, so the bare-integer form is
  undocumented-but-real.
- **Does passing it bypass the estimate gate?** **No.** See FI-1.
- **Is 4000 the right number?** No. Nine estimate samples this round for the
  address search (449,928 / 595,596 / 1,251,077 / 1,619,640 / 1,674,981 /
  1,792,501 / 2,087,963 / 2,303,267 / 2,985,198 s) and, for the id search that
  §3 actually runs, 783.5 s this round on top of the 187.4 / 189.6 / 196.7 /
  413.1 / 913.7 / 927.7 / 1284.9 s recorded in prior rounds. Against the
  observed max of 1284.9 s, the 3600 s ceiling already gave 2.80× headroom;
  4000 s gives 3.11×. The plan spends a decision to buy **11 %**.

### §1 — sufficiency and safety

**§1 does teach the habit the tool warns against, and its acceptance criterion
cannot detect an unsafe outcome.** The warning is emitted verbatim (measured,
above): *"builds the wallet from the ASSERTED key→slot assignment WITHOUT
verifying it … A wrong assignment produces a wrong wallet silently. Record +
check `--expect-wallet-id` **or a receive address**."*

The tool names the remedy — cross-check a receive address — and **the journey
already performs it**: `transcript_pathological.txt:180-196` is *"9b. THE
ADDRESSES — the check this wallet could never do before"*, printing
`bc1qkuknuy6dsm0fq44cyyhzqy9wl3ex2n6ed39zxhx867l9wlh4yhlsejms64` under the line
*"Compare these against your coordinator before engraving anything."* That is
byte-identical to the `first receive` `verify-bundle` prints.

§1 requires printing the warning and does **not** require acting on it. The
consequence is not academic: an operator who mis-asserts the assignment gets a
*different* id, records it, and at recovery reproduces a wallet that holds no
funds. §0 itself measures how sensitive the target is — same keys, same
addresses, different card encoding → `ced22709…` vs `d09840c3…`. §1's README
criterion names the two *other* ids that are wrong, but never states the
operator-facing constraint that **the target is only valid for the exact card
encoding used at engrave time**.

The acceptance is also unsafe in a second, sharper way — see C1: it passes for a
step that ends in `mismatch`.

---

## Decisions the implementer would still be forced to make

Nine. Two Critical, three Important, four Minor. (Round 0: 12. Round 1: 13.)

### C1 (Critical) — §1: is `--mk1` passed to `verify-bundle`?

§0's command block (`PLAN…:30-31`) is:

```
$ mnemonic verify-bundle --network mainnet --from phrase=- \
    --md1 ×3 --cosigner @N=… ×28 --mk1 ×2
```

`--mk1 ×2` is not decoration — the fixture's own key-00 card is exactly 2
chunks, so the sketch reads as "supply the own card as the engraved stub card",
which is what `--mk1` is documented for (*"the engraved template STUB cards the
binding check validates"*, `verify-bundle --help`). Measured, all three shapes,
same fixture, same seed, same 28 `--cosigner @N=` args:

```
--- no --mk1        exit=0
warning: explicit --cosigner @N= mode …
✓ md1_template_match: …
✓ wallet_completed: completed WalletPolicyId ced2270948ecb5af0779249ac7181f4a; …
OK (multisig template recomposed)

--- --mk1 own card (2 chunks)   exit=4
✓ md1_template_match: …
✗ mk1_template_stub_bind: supplied mk1 stub(s) do NOT bind via the template-id
  stub (card mismatch or policy/template cross-mix)
✓ wallet_completed: … ced2270948ecb5af0779249ac7181f4a; …
mismatch

--- --mk1 all 11 cards (30 chunks)   exit=4
✗ mk1_template_stub_bind: … 
mismatch
```

So the exact shape §0 pastes yields a **failed check and exit 4**, and §0's
excerpt shows neither the `✗` line nor the verdict line. §1's acceptance —
*"The pathological journey prints `ced2270948ecb5af0779249ac7181f4a` and the
warning, from a step in the committed script"* — is satisfied by **all three**,
because the `✓ wallet_completed` line and the warning appear in every one. The
implementer must decide, the plan points them at the wrong answer, and the gate
is blind to it.

This is a funds-facing document. Publishing a journey step whose verdict is
`mismatch`, immediately under text telling the operator this is the value to
record for recovery, is worse than not adding the step.

### C2 (Critical) — §2: is collision uniqueness scanned over PLATES or over CARDS?

§2 (`PLAN…:145-148`): *"The `set <chunk_set_id>` suffix is appended **only** to
plates whose `(fingerprint, path)` pair is not unique within the bundle, placed
**after** the chunk counter."*

`PlateEntry` is **per chunk** (`manifest.rs:41-57`; `bundle.rs:288-299` pushes
one per chunk). All chunks of one card share one `(fingerprint, path)`. So the
literal per-plate reading marks every multi-chunk card as colliding with itself.
Measured on the two artifacts §2's acceptance names:

| bundle | cards | plates | distinct `(fp,path)` **per card** | suffixed, per-plate reading | suffixed, per-card reading |
| --- | --- | --- | --- | --- | --- |
| pathological | 11 | 30 | 11 (all distinct — `out/pathological/card-index.txt`) | **30 / 30** | **0 / 30** |
| golden `bundle-md1-mk1.json` | 1 | 2 | 1 | **2 / 2** | **0 / 2** |

Those are different committed bytes in `transcript_pathological.txt` (30 lines)
and in `tests/vectors/bundle-md1-mk1.json`. §2's own acceptance cannot separate
them: `assert!(c.contains("mk1 [aabbccdd/48'/0'/0'/2'] chunk 1/2"))` is a
substring test, and `mk1 [aabbccdd/48'/0'/0'/2'] chunk 1/2 set 0x12345`
**contains** it. So the specified assertion passes under both readings — a
false-PASS on the one thing it was added to pin.

v2 raised exactly this (D5, *"Scope unspecified … Detection must group by
`chunk_set_id` first"*). The ledger records D5 as resolved on the strength of
the placement half.

### I1 (Important) — §3: what value does `--accept-search-time` actually get?

Because §3's stated rationale is false (FI-1), its number cannot be inherited.
The implementer must re-decide, and the plan gives no rule. The choice is real:
`4000` leaves the journey refusing on any machine whose calibration lands above
4000 s; a value chosen to actually remove the gate must exceed the worst
plausible estimate (the critique suggested `2h` = 7200 s; the observed id-search
max on *this* box is 1285 s, and the estimate is a single-thread projection that
scales with per-core speed). The plan must also say what the journey does when
the refusal fires anyway — §3 currently states the step "must complete every
run".

### I2 (Important) — §2: how big is the version bump, and who rebuilds `me-preview`?

§2's "Decided: release mechanics" says *"`me-cli` version bump + CHANGELOG
entry"* and names no version. `Cargo.toml:3` is `0.6.0`; the change is additive
to a public schema, so minor (0.7.0) is conventional — but that is the
implementer's call, not the plan's.

The unstated half is worse. See **H-A**: `me` version-gates its Go sidecar to an
**exact** match, and the pathological journey runs `me bundle --preview`. Bump
`me` without rebuilding `me-preview` and the journey's step 8 exits 2 —
i.e. §2's version bump breaks §2's own acceptance.

### I3 (Important) — §1: does the journey/README have to close the warning?

The tool says *"Record + check `--expect-wallet-id` or a receive address."* §1
requires printing that sentence and requires nothing else. The implementer must
decide:

- whether the journey step cross-checks `first receive` against step 9b's
  `bc1qkuknuy6…` (which the journey already prints, 40 lines later, under
  *"Compare these against your coordinator"*);
- whether §1's step goes before or after 9b, since §1 says only *"After `me
  bundle`"* and the cross-check only reads as a check if the two are adjacent;
- whether the README states that the recorded target is valid **only for the
  card encoding used at engrave time** — §0 measures the failure
  (`ced22709…` vs `d09840c3…`, identical addresses) but §1's acceptance does not
  require the README to carry it.

For a funds-sensitive instruction, "print the warning" is not the same
deliverable as "make the warning unnecessary", and the plan does not choose.

### D6 (Minor) — §2: what goes into the `manifest.rs` unit-test fixture?

The specified assertion cannot hold against `manifest.rs:192-223` as written
(hand-built plates, `string: Some("mk1a".into())`, no card). The implementer
must add `card_fingerprint` / `card_path` values to that fixture. The assertion
text pins what they must be, so this is an **unstated step** rather than an open
choice — but it is the only place the acceptance is not self-executing.

### D7 (Minor) — §2: does `SPEC_me_bundle_phaseA.md` §7 get updated?

`SPEC…:86-94` prints the checklist example the change falsifies. §2 names §6
only. Update it, or record that it stays stale.

### D8 (Minor) — §2: where do the two new test fixtures come from?

Acceptance demands a `--privacy-preserving` card test and a collision-pair test.
`crates/me-cli/tests/cli.rs:5-7` holds only `MK1_A`/`MK1_B`, both fingerprinted;
`tests/vectors/` holds four `.ndef` blobs, none of them either case. Both are
producible in one `mk encode` each (measured), but the step has no owner in the
plan. v2's D8; no ledger row.

### D9 (Minor) — two counting/naming residues

- **§3: "seven facts".** Step 1 lists **eight** — the model, `Unique`
  semantics, target-required, `--search-address` refused, `n ≤ 34`, the 3600 s
  cap, `--accept-search-time`, the 16-hex floor. The acceptance says *"The
  README states all seven facts listed in step 1."* Which seven? A hand-count
  where a tool could count.
- **§4: which command shows the canonical `@N`.** §4 answers *which* index
  binds (post-canonicalisation) but not how a reader of somebody else's wallet
  observes it. One sentence (`md decode` / `md inspect` on the engraved
  template) closes it.

---

## Factually incorrect assumptions

### FI-1 (Critical) — `--accept-search-time` does **not** remove the estimate gate

§3 (`PLAN…:200-202`): *"**Decision: pass `--accept-search-time 4000` in the
journey step**, so the gate is the *actual* work (≈16 s), not a variable
estimate."* Ledger D9: *"moot — `--accept-search-time` removes the estimate
gate."* Both false.

Mechanism — `cap_decision`, `permutation_search.rs:388-415`:

```rust
if estimate < SILENT_THRESHOLD { return Ok(RunSilent { estimate }); }
if estimate <= SEARCH_CEILING { return Ok(RunWithProgress { estimate }); }
match accept_search_time {
    Some(accepted) if accepted >= estimate => Ok(RunWithProgress { estimate }),
    Some(accepted) => Err(SearchError::AcceptSearchTimeTooLow { estimate, supplied: accepted }),
    None            => Err(SearchError::SearchTimeExceedsCeiling { estimate, ceiling: SEARCH_CEILING }),
}
```

`SEARCH_CEILING = 3600s` (`:59`). The flag is consulted **only** above the
ceiling and then demands `accepted >= estimate`. Executed:

```
accept-too-low    exit=1  error: --accept-search-time 4000s is below the estimated
                          exhaustive time 2087963.184384s; restate a duration ≥ the estimate
ceiling-refusal   exit=1  error: estimated exhaustive search time 2985198.644736s
                          exceeds the 3600s ceiling; re-run with --accept-search-time ≥…
```

So the effect of `--accept-search-time 4000` is to move the refusal threshold
from 3600 s to 4000 s and change the error text. The realized ≈16 s never enters
the decision. The failure mode §3 says it eliminated is intact, reduced by 11 %.

Two consequences the plan is built on and that do not follow: §3's "this gates
the step" analysis is answered by a remedy that does not answer it, and ledger
rows D9 and v2-I2 both claim closure that was never achieved.

### FI-2 (Critical) — §0's headline command block does not reproduce its own output

The plan's preamble (`PLAN…:13`) states: *"**Every claim below is pasted command
output.**"* §0's `verify-bundle` block is not. The shape it prints (`--mk1 ×2`)
returns `✗ mk1_template_stub_bind` and `mismatch`, exit 4 (full output under C1).
The block shows the `✓ wallet_completed` line and the warning and omits the ✗ and
the verdict. Either the `--mk1 ×2` in the command line is wrong, or the output is
elided in the one place where the elision hides a failure.

This is load-bearing because §1's whole deliverable is "run this command in the
journey", and the journey publishes the full output verbatim.

### FI-3 (Minor) — §0's `890788.897152s` and `84 ms` are single-run artifacts

§0 (`PLAN…:52-54`): *"**`--search-address` is REFUSED at n=11** in 84 ms —
*'estimated exhaustive search time 890788.897152s exceeds the 3600s ceiling'*."*

Nine samples of the identical command on this machine this round:

```
wall 0.191s  est 2303267.21856s
wall 0.142s  est 1619640.12672s
wall 0.052s  est  595596.976128s
wall 0.039s  est  449928.597888s
wall 0.106s  est 1251077.933952s
             est 1674981.576576s / 1792501.425792s / 2087963.184384s / 2985198.644736s
```

Refusal wall 39–191 ms; estimates spread 6.6×; **none equals the quoted value**,
and prior rounds recorded nine more (829,804 … 2,876,401 s) that also do not. The
conclusion is robust — every observation is 125×–830× the ceiling — and nothing
is built on the number, so this is a precision defect, not a decision defect. It
is the third round it has been raised (round 0 implicitly, v2 FI-4, critique-v2
N1) and the ledger has no row for it. In a document whose first promise is
"every claim is pasted command output", a constant pasted from one run and
presented as the fact is the exact thing that promise was supposed to prevent.

### FI-4 (Minor) — §3's exit-code table is incomplete, and mislabels 4

§3: *"`0` working shape · `1` ceiling refusal · `2` missing seed / pool size ·
`4` NO MATCH."* Measured: exit **4** is also the `--expect-wallet-id`
prefix-floor refusal (`error: … multisig-template-floor mismatch — derived
--expect-wallet-id prefix too weak for this search: need ≥8 bytes …, got 4`),
which is not a NO MATCH; and the accept-too-low refusal (**exit 1**) is unnamed,
though §3's own decision is what creates it.

### FI-5 (Nit) — the `KeyCard` citation is of the checkout, not the shipping artifact

§0 cites `mnemonic-key/crates/mk-codec/src/key_card.rs:34-57`. `Cargo.lock` pins
`mk-codec 0.4.1` from the registry. **Both files are identical over 34-57**
(verified line by line: `policy_id_stubs` :34, `origin_fingerprint:
Option<Fingerprint>` :40, `origin_path: DerivationPath` :46, `xpub: Xpub` :57),
so the claim is true — it just is not a citation of what ships. Flagged in round
0 and critique-v2 N2; no ledger row; unchanged.

### Checked and **correct** — recorded so a fourth round does not re-derive them

- `verify-bundle` prints `ced2270948ecb5af0779249ac7181f4a` in **4 ms**, exit 0,
  `OK` — six runs, 0.004–0.006 s. ✓
- The `--expect-wallet-id` floor: `ced22709` (8 hex) → exit 4 floor mismatch;
  `ced2270948ecb5af` (16 hex) → exit 0. ✓
- Shuffled-card recovery (order 07 03 10 01 09 05 02 08 04 06), exit **0**,
  **15.3 s**, `✓ wallet-id (completed): ced2270948ecb5af0779249ac7181f4a`,
  `your seed completes cosigner slot @0`, address matching the journey's. ✓
- Missing seed → exit **2**; ceiling refusal → exit **1**. ✓
- `--accept-search-time` exists (`restore.rs:196`), and **`4000` parses as 4000
  seconds** (`parse_search_duration`, `restore.rs:2185-2203`, no-suffix branch
  `(s, 1)`). ✓ The flag form in §3 is real; only its stated effect is not.
- Golden card = `aabbccdd` / `48'/0'/0'/2'` / 2 chunks, so
  `mk1 [aabbccdd/48'/0'/0'/2'] chunk 1/2` is the correct string. ✓
- `'` not `h` (`bitcoin-0.32/src/bip32.rs:210-221`), no leading `m/`
  (`:459-471`). ✓
- `SPEC_me_bundle_phaseA.md:61` is §6 and is the manifest schema. ✓
- `cli.rs:308` / `:745` are the two `assert_eq!(v, expected)` sites; the golden's
  plates 2–3 are card-bearing. ✓
- **Both transcripts regenerate byte-identically** (`diff` empty, both). ✓
- `grep -c 'mk1 chunk'` → 24 / 31; checklist plate lines → 24 / 30. ✓
- `bundle.rs:279` decodes and discards the `KeyCard`; `manifest.rs:82-108` is the
  checklist loop; `manifest.rs:228` is the assertion; plates emitted in
  `chunk_set_id` order (`BTreeMap`, `bundle.rs:207-208`). ✓
- `crates/me-cli/Cargo.toml:22-27` = md-codec, mk-codec, clap, zeroize, serde,
  serde_json — no `bitcoin`. ✓
- Privacy-preserving card, empty-path (depth-0) card and a same-`(fp,path)`
  collision pair are **all reachable**, and `me bundle` accepts the collision
  pair (5 plates, exit 0). ✓
- `d09840c3aa78035368f2dfb4bc271a27` with identical `first recv` — the
  omitted-fingerprint half of §0's two causes, re-measured independently. ✓
- Fixture policy placeholders appear as `@0 @1 … @10` in first-occurrence order
  → §4's "source and canonical orders coincide" is right. ✓
- `n ≤ 34` (`permutation_search.rs:91`, `:479`); `SEARCH_CEILING = 3600s`
  (`:59`); `canonicalize_placeholder_indices` exported (`md-codec/src/lib.rs:46`);
  65-byte xpub preimage (`md-codec/src/canonicalize.rs:361-363`). ✓
- `build_pdf_pathological.py:301` captions `@{ki} [{fp}/{path}]`. ✓
- `key-04` = `b8688df1` / `48'/0'/0'/2'` → §4 limit 2 holds. ✓

---

## Missing acceptance criteria

1. **§1 has no criterion for the verdict.** "Prints the id and the warning"
   passes for the `OK`/exit-0 shape *and* for the `mismatch`/exit-4 shape (C1).
   It must require `exit 0` and the literal `OK (multisig template recomposed)`.
2. **§1 has no criterion for closing the warning** — no `first receive` ↔ step
   9b cross-check, though the tool names it and the journey already prints both
   values (I3).
3. **§1's README criterion omits the constraint that decides whether recovery
   works**: the recorded target binds to the exact card encoding used at engrave
   time. §0 measures it (`ced22709…` vs `d09840c3…`); §1 does not require it to
   be written down.
4. **§2 has no criterion that pins the collision scope.** The specified
   `contains` assertion passes under both readings (C2). A criterion that
   discriminates would assert the *absence* of `set ` on the single-card golden,
   or assert the exact full line.
5. **§2 has no criterion for `SPEC_me_bundle_phaseA.md` §7** (`:86-94`), whose
   checklist example the change falsifies (D7).
6. **§2 has no criterion for the `me-preview` rebuild**, without which the
   journey's `me bundle --preview` step exits 2 after the version bump (H-A).
7. **§2 names no target version** for the bump.
8. **§3 has no criterion for the refusal it still permits** — what the journey
   does when the estimate exceeds the accepted time (FI-1). §3 says the step
   "must complete every run"; nothing enforces or excuses that.
9. **§3's "all seven facts" is ambiguous against eight listed items** (D9).

**Criteria I would accept as-is, for the record:** §2's "All 30 card plates in
the pathological checklist name an origin" (30 is measured-correct); §2's "Both
committed transcripts regenerate" (now genuinely satisfiable, measured); §4's
whole acceptance block (file, heading, sentence, canonicalisation answer, three
limits — all checkable by reading); §3's "the journey performs a shuffled-card
recovery, exit 0, using the id an earlier step printed" (the anti-literal clause
is the plan's best single line and it survives from v2).

---

## Hidden dependencies / ordering problems

**H-A (Important, new in v3) — §2's version bump requires a lockstep
`me-preview` rebuild, or the journey cannot regenerate.**
`crates/me-cli/src/main.rs:647-657`:

```rust
// Version-gate: the sidecar must match this crate's version exactly.
let expected = env!("CARGO_PKG_VERSION");
match preview::sidecar_version(&sidecar) {
    Ok(found) if found == expected => {}
    Ok(found) => { eprintln!("me: me-preview version mismatch: sidecar is {found:?}, \
                              expected {expected:?}; refusing to render …");
                   return Some(EXIT_USAGE); }
```

`transcript_pathological.sh:216` runs `me bundle … --preview …`, and the
committed transcript pins both versions (`transcript_pathological.txt:16-20`:
`me 0.6.0`, `me-preview 0.6.0`). The sidecar's version comes only from
`-ldflags "-X main.version=${VERSION}"` (`.github/workflows/release.yml:142`).
So: bump `me` → rebuild `me-preview` with matching ldflags → *then* regenerate.
Miss the middle step and §2's own acceptance ("both committed transcripts
regenerate") fails with exit 2. The plan mentions none of it.

**H-B (Important) — §2 cannot start before C2 is answered.** The golden's
regenerated content and 30 lines of the pathological transcript both depend on
the plate-vs-card scoping. §2 presents the collision rule as decided, so an
implementer will not know they are blocked.

**H-C (Minor) — §1's step placement is unspecified relative to step 9b.** §1
says "After `me bundle`". `me bundle` is step 8; the addresses are step 9b
(`transcript_pathological.txt:180`). If the cross-check in I3 is adopted, the
two must be adjacent or the check is not legible as one.

**H-D (positive, and it changes the plan's whole risk picture) — the F-210
dependency is dead.** In rounds 0 and 1 this was the standing objection to §1,
§2 and §3 alike: three acceptance criteria rested on regenerating transcripts
that a filed, open follow-up said could not be regenerated. Measured this round,
both scripts run to completion and reproduce their committed output byte for
byte. §6's ordering ("regenerate once after §1 and §3") is now a real plan
against a real mechanism.

**H-E (Nit) — `design/FOLLOWUPS.md:7561`'s heading is stale.** F-210 still reads
"the operator journeys cannot be regenerated" although `b822e4a`/`6a42d89`/
`c6c6943`/`e59ce9f`/`3cba69b` fixed it for the two journeys this plan touches
(`transcript_payload.sh` has no `runcap` and was not tested). Against
`FOLLOWUPS.md:5-20`'s own "status lives in the heading" rule, a grep for open
items over-counts. Outside this plan's scope; one line to whoever owns F-210.

---

## Open / could not determine

- **Why `mk1_template_stub_bind` fails for this fixture.** The journey derives
  the stub `5b48af35` from the template id by hand and documents the reasoning
  (`transcript_pathological.txt:58-68`: *"mk follows the template-id"*), yet
  `verify-bundle`'s binding check rejects it for the own card and for all 11.
  Whether that is a toolkit defect, an `mk`↔toolkit disagreement, or a fixture
  artefact is undetermined — and it decides whether §1's step could ever legally
  pass `--mk1`. I could not isolate the check (`--mk1` alone refuses first with
  exit 2, "requires the operator's own seed").
- **Whether the estimate crosses 4000 s on slower hardware.** Extrapolated from
  24-core samples only; the estimate is a single-thread projection from a
  64-sample calibration and varied 6.6× (address search) and ~6.9× (id search)
  on one machine. Not reproduced on a slow runner in any round.
- **The `tr()` pathological variant.**
  `design/journeys/inputs-pathological/wallet-policy-tr.txt` and
  `backup-strings-tr.txt` are tracked and remain unexercised in all three
  rounds. Project memory warns explicitly that measuring one descriptor path
  gives a wrong answer about the other, and §2's label change touches every
  card-bearing plate regardless of descriptor shape.
- **Whether the collision marker should also appear in `manifest.json`.** §2
  adds two JSON fields and a *checklist* suffix; `chunk_set_id` is already a JSON
  key, so probably nothing new is needed — but the acceptance does not say which
  surface the collision test asserts against.
- **`transcript_payload.sh`.** Not run; it has no `runcap` (grep: 0 hits) and no
  committed `.txt`, so F-210's third journey is untested here. Nothing in this
  plan touches it.

---

## One line on what would make v3 executable

Answer two questions and correct one claim: say whether §1's `verify-bundle`
step passes `--mk1` (measured: it must not) and require `exit 0` + `OK` as the
criterion; say whether §2's uniqueness scan groups by `chunk_set_id` (measured:
per-plate suffixes 30 of 30 pathological plates and both golden plates); and
replace §3's "`--accept-search-time` removes the estimate gate" with what
`cap_decision` actually does, then pick a number on that basis. Everything else
on this list is a sentence, a filename, or a count.
