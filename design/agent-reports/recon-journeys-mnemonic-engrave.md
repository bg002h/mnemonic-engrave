# Recon: round-trip journeys that EXIST in `mnemonic-engrave` — 2026-08-19

Read-only inventory per `design/DRAFT_round_trip_journey_definition.md` §7, per the
2026-08-19 §8 rulings. Scope: this repo only. Does not enumerate what "should"
exist (deferred by ruling #3).

## What I actually ran

All read-only. `Read`/`Bash cat`/`grep`/`comm`/`wc` over: `design/DRAFT_round_trip_journey_definition.md`,
`design/journeys/README.md`, `design/journeys/{transcript.sh,transcript_pathological.sh,
transcript_payload.sh,build_pdf.py,build_pdf_pathological.py,build_pdf_payload.py,
derive-pathological-keys.sh}`, `design/journeys/{transcript.txt,transcript_pathological.txt}`
(the committed, already-present transcript captures — I did not run either
`.sh` script), `design/journeys/out/manifest.json` (parsed with `python3 -c`),
`design/FOLLOWUPS.md` (grep, plus reading the F-128/F-156/F-210 entries in full).
Verified with the actual filesystem, not descriptions: `git ls-files design/journeys/`,
`git check-ignore -v`, `git status --short`, `ls -la` on `out/`, `shots/`,
`inputs*/`; existence of the four constellation binaries and the two
`seedhammer` fork files (`cmd/emu/sysw_test_payload.{bin,go}`) via `ls -la`.
Cross-checked every `.png` name referenced by each `build_pdf*.py` against
what actually exists in `shots/` with `grep -oE` + `comm -23` (not eyeballed —
see the exact counts below). Viewed `shots/p06-seed-from-payload.png` with the
image reader and compared it by eye to `inputs-payload/records.txt`. Actually
**executed** the one command in scope that is genuinely read-only —
`bash derive-pathological-keys.sh --check` (its own `--check` mode is
documented as "verify, change nothing"; it touched no files) — result below.
Did not run any `transcript*.sh` or `build_pdf*.py` (would write `out/`/PDFs,
out of scope). One accidental stray file (`/scratch/code/shibboleth/mnemonie-engrave-placeholder`,
a typo, zero relation to this repo) was created and immediately deleted; it
touched nothing in this recon.

---

## Inventory — three journeys exist. All three are CUSTODIAL for their engraved
material; none is generative end-to-end within a single command.

### Journey 1 — "operator journey" (5-of-12 `wsh(multi(…))`)

| field | value |
| --- | --- |
| name | `SeedHammer-II-operator-journey` |
| kind | **CUSTODIAL.** Engraved wallet built from pre-supplied `inputs/keys/cosigner-*.xpub` (12 files) + `inputs/wallet-policy.txt`. The one seed used (`inputs/seeds/cosigner-00.seed`) is *not* what the xpubs were derived from within this journey — it feeds a separate `ms encode` → NFC-refusal side-check that never touches the engraved bundle. |
| tier | **Claimed T3** (README: emulator + "NFC attempted"). **Currently demonstrable: T2 only.** See finding F-J1 below — every screenshot the build script references is missing. |
| origin artifact | `design/journeys/inputs/wallet-policy.txt` (`wsh(multi(5,@0..@11/<0;1>/*))`, verified 12 key slots against 12 committed `.xpub` files) + `inputs/keys/cosigner-{00..11}.xpub` |
| invocations (repo-by-repo) | All host-CLI steps run from `design/journeys/transcript.sh` inside **this repo**, calling binaries built from `descriptor-mnemonic` (`md`), `mnemonic-key` (`mk`), `mnemonic-secret` (`ms`), and `mnemonic-engrave` (`me`, `me-preview`) — all four confirmed present on disk. Sequence: `md encode`→md1 (`runcap`) → `md bytecode`/`inspect` → `mk encode --from-md1` ×12 (one per cosigner, FATAL-guarded on xpub/origin-header parse and on chunk count ≠2) → `mk inspect` spot-check → assemble `out/backup-strings.txt` from this run's own captures → `me bundle --in … --preview … --png --manifest …` → `me --in <md1> --hex` (NDEF) → `me --in <ms1> --hex` (refused, exit 3, confirmed in `transcript.txt`) → `me sysw pack/show/wipe` (unrelated payload-subsystem demo appended at the end). **The emulator/GUI layer (seedhammer fork) is invoked by NO script in this repo** — README documents it as a manual browser walk POSTing frames to `shot_server.py`. **Cross-repo, out-of-band:** the 12 cosigner xpubs' own derivation runs in the `seedhammer` fork via `cmd/journeykeys` (named only in `build_pdf.py`'s last page, not executed by any script here). |
| structural assertion | Shell-level invariants only: 12 keys × 2 mk1 chunks expected == got; per-key `mk encode` exit checked (FATAL on nonzero, per the I-2 fold comment in the script). `build_pdf.py` cross-checks that every `mk1…` string *quoted* in the committed `transcript.txt` is a member of the live `out/backup-strings.txt` set (I-3 guard) — real, machine-checked, but a **consistency-between-two-local-artifacts** check, not a cryptographic round-trip equality. |
| functional assertion | **NONE.** Verified two ways: `grep -c` (with exit code) and a `python3` substring search both confirm the string `"address"` occurs **zero times** in `transcript.sh`, `transcript.txt`, and `build_pdf.py`. No fingerprint or wallet-id comparison exists either. This is a **missing field per §7** — the journey never computes a receive/change address, a master fingerprint, or either wallet id, so §4's mandatory functional equality is entirely absent for this journey. |
| the ONE command | **None exists.** Reproduction is `bash transcript.sh > transcript.txt` then `python3 build_pdf.py` (2 commands, no wrapper script found), and `build_pdf.py` writes `out/journey.html`, not a PDF (the README says the same). |
| stated non-coverage | No structured §6 statement (predates the definition doc by design — the definition is dated 2026-08-18/19, after these scripts). README's "Corrections" table (F-131..F-140) is the closest analogue and is not part of the journey's own output. |

**F-J1 (NEW) — the operator journey's screenshot layer is currently 100% broken, and a real, matching capture sits unused.**
Machine-counted (`grep -oE "'[a-zA-Z0-9_.-]+\.png'" build_pdf.py \| sort -u` diffed
against `shots/` with `comm -23`): **19 of 19** `.png` names `build_pdf.py`
references (`j00-boot.png`, `k01-engrave-text-enter.png`, `p0-plate.png`, …)
**do not exist anywhere in `shots/`.** The `img()`/`shot()` helper does not
raise on a missing file — it silently emits `<p class="missing">missing:
NAME</p>` and the build completes at exit 0. This is directly verified, not
inferred: the **currently-present** `out/journey.html` (built earlier this
session, predates the most recent `out/` regen) already contains 19
`class="missing"` placeholders, confirmed by both `grep -c` and `grep -o`, and
the file's tail confirms the write completed cleanly (no crash). This is a live
instance of §5's "a skipped step must fail, not pass" — applied to the whole
T3/T4-sim visual section of this document (both the emulator-flow screens and
the mid-cut plate-engraving frames `p0/p2/p5/p11-plate.png`).
**Separately:** `shots/` currently holds `r00-multisig-menu.png` through
`r13-bundle-gather.png` (14 frames, captured 2026-08-13, filenames that read
exactly as a 5-of-12-multisig-configuration walk: `r02-template-pick`,
`r04-n-picker`, `r05-k-picker`, `r09-card-chunk1`, `r12-bundle-menu`, …) —
and `build_pdf.py` references **zero** of them (grep across all three
`build_pdf*.py`: no match). So real T3 capture material for this exact journey
appears to exist locally but is wired into nothing.

**F-J2 (NEW, minor) — the operator journey's 12 key fixtures have no committed regeneration/verification script, and a comment claiming otherwise doesn't resolve.**
`derive-pathological-keys.sh`'s own header comment states: *"The OPERATOR
journey's twelve keys still have none [a producer]; they were reproducible
only by luck, and that is filed separately."* Grepped `design/FOLLOWUPS.md`
twice, by different terms (`cosigner`, `"reproducible only by luck"`,
`journeykeys`, `"twelve keys"`) — no entry matches. Also grepped the whole
`design/` tree for the exact phrase; the only hit is the comment itself. So
either the filing never happened or uses wording I could not find by any term
I tried — as it stands, `inputs/keys/cosigner-*.xpub` has no analogue to
`derive-pathological-keys.sh --check`.

---

### Journey 2 — "pathological wallet" (4-tier degrading miniscript, 11 keys, 4 timelock kinds, 1 hashlock)

| field | value |
| --- | --- |
| name | `SeedHammer-II-pathological-wallet-journey` |
| kind | **CUSTODIAL**, same shape as Journey 1: engraved bundle built from `inputs-pathological/keys/key-*.xpub` (pre-supplied) + policy text. **BUT** the generative link genuinely exists in this repo as a *separate* artifact — see the positive finding F-J4 below. |
| tier | Claimed "typed on the device, no NFC" (T3-shaped) per README. **Currently demonstrable: T2**, plus this is the only journey of the three that executes a real machine-derived functional VALUE (see below). Screenshot layer: **13 of 13 referenced `.png` names are missing from `shots/`**, and unlike Journey 1 there is no orphaned same-content capture under a different name — no file in `shots/` starts with `a` or `b` at all. Worse-off than Journey 1 on this axis. |
| origin artifact | `inputs-pathological/wallet-policy.txt` (11-key, 4-tier `wsh(or_i(…))`, needs `--force-chunked --path bip48`, 3 md1 chunks) + `inputs-pathological/keys/key-{00..10}.xpub` (BIP-48 four-level origins) + `inputs-pathological/seeds/master-{A,B,C}.seed` (BIP-39 published test vectors; master-A used only for the same end-of-run ms1-refusal side-check as Journey 1) |
| invocations | Host-only, this repo, `transcript_pathological.sh`: `md encode` (fails, no `--path`, shown as a real refusal) → `md encode --force-chunked --path bip48` (3-chunk md1, captured via `runcap`) → `md inspect` (round-trip print) → `mk encode --from-md1` (fails on a chunked md1 — **Obstacle 1**, F-127) → `md inspect \| sed` to derive the template-id, used as `--policy-id-stub` (STUB is FATAL-guarded non-empty) → `mk encode --policy-id-stub` ×11 keys (chunk count now variable: "at least one", not fixed at 2 — a fold that fixed a since-falsified assumption) → `mk decode` spot check → assemble `out/pathological/backup-strings.txt` → `me bundle --preview --png --manifest` → `md inspect` (prints both `wallet-descriptor-template-id` and `wallet-policy-id`) → **rebuilds a second, KEYED, non-engraved md1** (`md encode --force-chunked --path bip48 --key @0=… …@10=…`) purely to run `md address --chain 0 --count 3` and `md address --chain 1 --count 3` → `ms encode` (master-A seed) → `me --in <ms1> --hex` (refused, exit 3, confirmed in `transcript_pathological.txt`). No emulator invocation inside the script. |
| structural assertion | `md inspect` on the reassembled chunk set (printed, not diffed); shell FATAL guards (≥1 mk1 chunk/key, non-empty STUB, orphan-string cross-check against `card-index.txt`/`manifest.json` in `build_pdf_pathological.py`, which raises `SystemExit` on mismatch — a real, machine-checked local-consistency guard). No cryptographic diff against an independently sourced expected value. |
| functional assertion | **A real value is derived and the gate has executed** — confirmed directly in `transcript_pathological.txt`: `md address … --chain 0 --count 3` and `--chain 1 --count 3` on the keyed policy, exit 0, three real bech32 addresses each (receive + change, matching §4's requirement in *kind*). The transcript's own prose names this "the FUNCTIONAL half of a round trip." **But there is no automated equality.** The script prints the addresses and instructs the human operator to "compare these against your coordinator before engraving anything" — no independent fixture of expected addresses exists anywhere in the repo to diff against, so per §7 this is a **value without an assertion**: the gate ran, but nothing checks it. |
| the ONE command | None — same 2-command pattern as Journey 1, and the build (were it run) would currently degrade with 13/13 missing screenshots. |
| stated non-coverage | Same as Journey 1: informal (README corrections table), not a structured §6 statement. |

**F-J3 (NEW) — a hand-transcribed, non-reproducible comparison is baked into the published-document generator, 3 of 4 times as a hardcoded literal.**
`build_pdf_pathological.py:212-214` embeds this block as a **plain Python
string literal** (not pulled from the parsed transcript dict `S`/`sect()` the
rest of the file uses):
```
wallet-descriptor-template-id: 726a666305756435b7c52c5b3fc69c41
wallet-policy-id:              f05e8a1c282f7740bbfd902a759b5577
policy_id_stubs (what mk embedded):  726a6663
```
The surrounding prose says this was "measured on a single-string wallet where
`--from-md1` works" — i.e. **a different wallet than this journey's own**,
from a run with no committed reproduction path anywhere in this repo. The same
literal also appears, independently hand-typed, in `design/FOLLOWUPS.md`'s
F-128 entry (filed 2026-08-11). This is exactly §7's named example: *"a path
whose 'expected' values were transcribed by hand from a run nobody has
repeated."* If `mk`'s stub-derivation behavior or the demonstration wallet ever
changes, this block goes stale with the build still succeeding — nothing
checks it. (By contrast, the *this-wallet* stub quoted two lines later,
`5b48af35`, is also a hardcoded literal but I confirmed it currently matches
`transcript_pathological.txt`'s real, freshly-computed value — so that one
line is presently accurate despite the same sourcing gap.)

**F-J4 (POSITIVE finding, and a missed wiring opportunity) — `derive-pathological-keys.sh` is a real, currently-passing generative gate that the journey does not use.**
This script derives all 11 `key-*.xpub` files from `inputs-pathological/seeds/master-{A,B,C}.seed`
via `ms derive --template bip48-p2wsh --account N`, and its `--check` mode
verifies the committed files against a fresh derivation with no side effects.
I ran it (read-only, as its own docstring promises): **`bash
derive-pathological-keys.sh --check` → "all 11 key files match a fresh
derivation."** This is a genuine, currently-green, seed→xpub generative link —
exactly the missing half §3.1/§8-ruling-1 worries about elsewhere — but it is
a **separate script `transcript_pathological.sh` never calls**, and it is
**referenced by nothing**: grepped `README.md`, `transcript_pathological.sh`,
and both `build_pdf*.py` for the string `derive-pathological-keys` — zero
hits. So the generative capability exists and currently passes, but no single
command chains "derive the keys from seed" → "build and engrave the bundle"
into one journey; a reader of the journey's own documentation would never
learn this script exists.

---

### Journey 3 — "Load Payload" (systemwide payload container)

| field | value |
| --- | --- |
| name | `SeedHammer-II-load-payload-journey` |
| kind | **CUSTODIAL** — origin is `inputs-payload/records.txt`, a pre-written artifact (plaintext BIP-39 test-vector mnemonic `abandon…about`, a hex passphrase record, a hex free-text record), not entropy generated within the journey. |
| tier | **T3, actually demonstrated — the strongest of the three.** Machine-verified: every `.png` name `build_pdf_payload.py` references (21 names) exists in `shots/` (`comm -23` on the two sorted lists produced **empty output**). Visually confirmed one frame myself: `shots/p06-seed-from-payload.png` shows "1: ABANDON … 12: ABOUT", matching `inputs-payload/records.txt` line 1 exactly. |
| origin artifact | `design/journeys/inputs-payload/records.txt` |
| invocations | Host-only, this repo: `me sysw pack --in records-as-first-written.txt` (refused, exit 4, real defect class shown) → hex-encode via `xxd` (a manual one-liner in the transcript, not itself a constellation command) → `me sysw pack --in records.txt --out payload.bin` → `me sysw show payload.bin` (prints the digest) → **`cmp payload.bin` against `$FORK/cmd/emu/sysw_test_payload.bin`** (cross-repo, `seedhammer` fork — both files confirmed to exist on disk) + `sha256sum` of both + `grep syswTestDigest` from the fork's `.go` source → `me sysw pack --region` (65536-byte flash image) → `me sysw show` on the region (padding-invariance check) → `me sysw wipe` ×2 (`--fill ones`, random) → `me sysw show` on wiped output (confirms empty) → `nix develop -- picotool version` (tool-presence only; **the actual `picotool load` flash write is explicitly NOT run by the script** — the comment says so and states it "has been run" previously, out of band; this is an honestly disclosed, not silent, gap). Then, **outside** the script: a manual browser-driven emulator walk POSTs 21 frames to `shot_server.py` — no script in this repo drives the emulator itself. |
| structural assertion | Real and cross-repo-independent: `cmp` byte-compares the host-packed container against a file committed in the *other* repo (`seedhammer` fork's `cmd/emu/sysw_test_payload.bin`), corroborated by an independent `sha256sum` and a `grep` of the fork's own Go constant. This is the best-sourced structural check of the three journeys — genuinely two independent artifacts, not a self-produced value. Caveat: nothing in the script itself halts on a `cmp` mismatch (no `set -e`, no explicit exit-code branch); a divergence would show only as a nonzero `[exit N]` in the transcript text for a reader to notice, not as a build failure. |
| functional assertion | This journey carries a mnemonic but is not a wallet-descriptor journey, so §4's named functional-equality types (receive+change address, master fingerprint, wallet id) don't apply as written. The closest analogue — the digest match across host / fork-source / device-screenshot — is **partly hardcoded**: `build_pdf_payload.py` quotes the literal string `"55ad b800 6ec6 a066 94f3 6a0e 900a c8d5"` **four times**; only one of the four (`S.get('5. show …')`, line 184) is pulled live from the parsed transcript. The other three (lines 214, 257, 260, 263 — the "padding doesn't change it" note, the HOST/FORK/DEVICE comparison table, and the image caption) are plain Python string literals with no build-time link back to the real transcript or to the fork's `.go` source. Same class of gap as F-J3, smaller in stakes (a payload digest, not a wallet id) but structurally identical. |
| the ONE command | None — same 2-command pattern (`bash transcript_payload.sh > out/transcript_payload.txt` then `python3 build_pdf_payload.py`; the module's own docstring calls this "the two commands in the last section"). **Currently cannot even be attempted**: `out/transcript_payload.txt` does not exist anywhere in this checkout (confirmed by `find`), and unlike the other two builders, `build_pdf_payload.py:73` opens it with a bare `open(...)` — no existence check, no friendly "run transcript_payload.sh first" message (the other two builders both have that guard). Running `build_pdf_payload.py` right now would crash with a raw `FileNotFoundError` traceback. |
| stated non-coverage | Best of the three, though still informal: the document explicitly narrates F-153/F-154/F-155/F-156 as "what this run turned up," and states the `picotool load` step was not executed here. Still not a structured §6 field. |

---

## Anti-requirement checks (§5), applied where I could see them

- **Reads an intermediate nothing in the journey writes** — F-210's fix is real
  and I verified it by direct reading, not by trusting the comment: every
  `md-encode-raw.txt` / `mk-encode-raw.txt` / `ms-encode.txt` / `md1.txt` /
  `backup-strings.txt` / `card-index.txt` read in `transcript.sh` and
  `transcript_pathological.sh` is preceded, in the same script, by the write
  that produces it. `transcript_payload.sh` has no journey-internal
  intermediates of this kind (each `.bin` is written immediately before use).
  **No new instance of the F-210 class found** in the three transcripts.
  A related-but-distinct instance exists one layer up in the *document
  builders*, not the transcripts: `build_pdf_payload.py` reads
  `out/transcript_payload.txt` with no guard (see the table above) — not a
  silent pass, but a bare crash rather than the informative message its two
  siblings give.
- **Asserts against a self-produced value with no independent source** — see
  F-J3 (pathological doc) and the payload-digest note above (Journey 3). The
  `cmp`-against-the-fork check in Journey 3 is the one case that gets this
  right.
- **A skipped step passes instead of failing** — **yes, live, in two of
  three journeys**: F-J1 (Journey 1, 19/19 missing images, exit 0) and the
  parallel 13/13 count for Journey 2 (not independently re-verified against a
  built HTML since `build_pdf_pathological.py` has not produced one in this
  checkout, but the same silent-placeholder `img()` function is shared code,
  confirmed by direct reading — see the code excerpt in my working notes: it
  returns a `<p class="missing">` string rather than raising).
- **A gate that has never executed** — checked the ones that matter: the
  pathological journey's `md address --chain 0/1` gate **has** executed
  (real output in the committed `transcript_pathological.txt`, exit 0); the
  `ms1`-refusal gate in both wallet journeys **has** executed (exit 3, both
  transcripts); `derive-pathological-keys.sh --check` **has** executed (I ran
  it: "all 11 key files match a fresh derivation"). I found no journey whose
  terminal assertion is structurally unsatisfiable.
- **Empty output is not proof of absence** — every negative claim above
  (no "address" in Journey 1's files; no `derive-pathological-keys` reference;
  no `r00-multisig`/`w01-loadpayload` reference; no FOLLOWUPS.md entry
  matching F-J2's dangling comment) was checked at least two ways: a grep plus
  either a Python substring search, an explicit exit-code check, or a second
  differently-worded grep, as detailed inline above.

## The blind spot this recon cannot see (per ruling #3)

A per-journey sweep inside this one repo cannot see gaps *between* repos.
Two concrete instances surfaced anyway, stated as facts only: `cmd/journeykeys`
(Journey 1's key derivation) and `cmd/emu/sysw_test_payload.{bin,go}`
(Journey 3's structural comparison target) both live in the `seedhammer` fork
and were not audited there — only confirmed to exist on disk.

## Summary of NEW findings (not already filed)

- **F-J1** — Journey 1's screenshot layer: 19/19 referenced images missing,
  build exits 0 with placeholders; a real, matching 14-frame capture (`r00`–`r13`)
  sits in `shots/` unused by any script.
- **F-J2** — Journey 1's 12 key fixtures have no regeneration/check script; a
  comment claims this "is filed separately" but no matching FOLLOWUPS.md entry
  was found by any search term tried.
- **F-J3** — `build_pdf_pathological.py` hardcodes a 3-line id/stub comparison
  from an unreproducible foreign run, 3 of 4 times as a literal (also
  duplicated by hand in FOLLOWUPS.md's F-128).
- **F-J4 (positive)** — `derive-pathological-keys.sh --check` is a real,
  currently-green seed→xpub generative gate, wired into nothing.
- Journey 2's screenshot layer: 13/13 referenced images missing, with no
  orphaned alternative capture found under any name (worse-off than Journey 1).
- Journey 3's digest "functional" comparison is 3/4 hardcoded, not derived from
  the real transcript at build time; `build_pdf_payload.py` also lacks the
  friendly missing-input guard its two siblings have.
- None of the three journeys carries a §4 functional equality that is both
  *executed* and *automatically asserted against an independent source*:
  Journey 1 has none at all; Journey 2 executes and prints real addresses but
  asserts nothing automatically; Journey 3's closest analogue (a digest, not a
  wallet-funds value) is partly hardcoded.
- None of the three journeys is reproducible by **one** command (all need a
  transcript-capture step then a separate build step); none carries a
  structured §6 non-coverage statement (all predate the definition doc).
