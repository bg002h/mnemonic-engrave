# RECON — F-210: why the pathological journey doesn't regenerate, and the minimum fix

Scope: read-only recon. Ran `design/journeys/transcript_pathological.sh` and did a
supplementary manual walk of the same tools with hand-supplied intermediates.
Did not edit any journey file.

## Verdict

The pathological transcript fails to regenerate for two independent reasons, not
one: (1) four of the six named intermediates (`md-encode-raw.txt`,
`mk-encode-raw.txt`, `ms-encode.txt`, `md1.txt`) have **never had a writer in any
committed version of the scripts**, since the very first commit (`bdf954f`,
2026-08-11) — the scripts were written to *consume* artifacts a separate,
uncommitted process produced, and `transcript_pathological.sh` even reads
`out/md1.txt` at line 18, before line 34's `md encode` (the only command that
could produce it) has run; and (2) two more (`manifest.json`, `sysw-public.bin`)
have a real writer (a `--manifest`/`--out` flag) that never fires at runtime
because the command carrying it dies first — from the cascaded-empty-argument
chain in the pathological run, and independently from a stale local
`me-preview` binary (0.5.1) sitting beside a rebuilt `me` (0.6.0) that now
enforces sidecar-version equality. **Manually supplying the missing
intermediates shows the substantive content is intact**: `mk`'s "wire-format
version mismatch: got 9, expected 4" against a real chunked md1 reproduces
byte-for-byte with today's `md`/`mk` (0.13.0/0.13.0), and the 26-plate manifest
and checklist are byte-identical to the committed transcript once `--preview` is
dropped. The minimum fix is a small script refactor (~20-30 line diff across two
files: capture each producer's stdout at the point of production, and move the
top-of-script reads to after their producers run) plus rebuilding `me-preview`;
call it a single short session, not a redesign. **F-210 is NOT actually on this
cycle's critical path**, though it is due in it: even a perfectly regenerated
pathological journey has zero emulator interaction with the md1 gather/expand/
descriptor-display/address-verify flow — its only device steps are seed typing
and plate cutting — and the wallet's own miniscript shape (`or_i`/`and_v`/
unsorted `multi`/`sha256`/timelocks) is explicitly excluded from
descriptor-build/address-verify by the shipped #10b `D2` "faithful-or-refuse"
subset (singlesig + `wsh(sortedmulti)` + `sh(wsh(sortedmulti))` only). A new
journey — or substantial new content in this one — is needed regardless of
F-210's repair, to actually exercise the flow the "arbitrary tr()/wsh()" cycle
is about.

## What F-210 already says

`design/FOLLOWUPS.md:7561-7621`. Found 2026-08-18 by running
`transcript.sh` (the 5-of-12 operator journey, NOT the pathological one) at the
operator's request. Measured there: 9 non-zero exits fresh vs. 1 committed;
`mk` 0.12.1→0.13.0, `ms` 0.14.1→0.16.0, `me` 0.5.1→0.6.0. **Defect 1**: six
intermediates are read across the three transcripts and none exists on disk —
named as `md-encode-raw.txt`, `mk-encode-raw.txt`, `ms-encode.txt`, `md1.txt`,
`manifest.json`, `sysw-public.bin` — with read/write counts per script
(`transcript.sh` 9/2, `transcript_pathological.sh` 5/1, `transcript_payload.sh`
1/0); `out/` is untracked so nothing carries these across sessions. **Defect
2**: the committed transcript's line 23 shows a scratchpad path from a dead
session — the artifact of record was made by a script that no longer exists in
this form. F-210 assigns the owning phase to "the arbitrary-`tr()`/`wsh()`
cycle — before it leans on the pathological journey," and offers two repair
shapes without picking one: self-contained scripts (slower, regenerates
anywhere) vs. committing the intermediates as fixtures (fast, decays again on
the next version bump) — noting the version drift argues for the first.
Everything below is additive: the pathological-specific regeneration trace,
the exact six-file read/write map, confirmation the substantive finding
(`mk`'s chunked-md1 rejection) survives today's tools untouched, and the answer
to whether the journey would even exercise the new feature once regenerated.

## The six unwritten intermediates

| file | read by | should be written by | why it isn't |
| --- | --- | --- | --- |
| `out/md1.txt` | `transcript_pathological.sh:18` (`MD1S=$(tr '\n' ' ' < "$W/out/md1.txt")`), `:41` (`FIRST=$(head -1 ...)`) | `transcript_pathological.sh:34`, the `md encode --group-size 0 --force-chunked --path bip84 "$T"` call (its 3 stdout lines starting `md1...` ARE the content) | No script line ever redirects that command's stdout to this path — `run()` only echoes to the transcript's own stdout. Worse: the read at line 18 happens **before** line 34 runs at all, so even a `tee` bolted onto line 34 wouldn't satisfy line 18's read without also moving it. |
| `out/md-encode-raw.txt` | `transcript.sh:34` (`md bytecode "$(grep '^md1' ...)"`), `:35` (`md inspect ...`), `:40` (`MD1=$(grep '^md1' ...)`) | `transcript.sh:33`, the `md encode --group-size 0 "$(cat ...)"` call | Same shape as above: `run "$MD" encode ...` at line 33 never redirects to this file; nothing in `transcript.sh` ever writes it. |
| `out/mk-encode-raw.txt` | `transcript_pathological.sh:58` (`mk decode $(sed -n '1,2p' ...)`), `transcript.sh:43` (`mk inspect $(sed -n '1,2p' ...)`) | The preceding `mk encode --xpub ... --policy-id-stub ...` call in each script (`transcript_pathological.sh:56-57`, `transcript.sh:41-42`) | Same shape: the `mk encode` stdout (the mk1 lines) is only echoed via `run()`, never captured to this path in either script. |
| `out/ms-encode.txt` | `transcript.sh:60` (`MS1=$(grep '^ms1' "$W/out/ms-encode.txt" ...)`) | `transcript.sh:47`, the `ms encode --phrase "$(cat ...)"` call | Same shape: `run "$MS" encode ...` output is echoed, not redirected to a file. |
| `out/manifest.json` | Not read anywhere in the committed scripts or `build_pdf*.py` — grepped both; no hit outside the `--manifest` write flags themselves. | `transcript.sh:54` / `transcript_pathological.sh:62`, via `me bundle --manifest "$W/out/manifest.json"` | This one **does** have a real writer — the flag itself. It's absent on disk only because the `me bundle` invocation dies before completing: in the pathological run, on the `me`/`me-preview` sidecar version mismatch (`me 0.6.0` vs. stale `me-preview 0.5.1`, exit 2); in the operator run, on the cascaded-empty `$MD1` from the unwritten `md-encode-raw.txt` above. No read-site was found for this file in current code — flagged under Open below rather than asserted. |
| `out/sysw-public.bin` | `transcript.sh:65` (`me sysw show "$W/out/sysw-public.bin"`) | `transcript.sh:64`, `me sysw pack --no-passphrase "$MD1" --out "$W/out/sysw-public.bin"` | Same shape as manifest.json: real writer via `--out`, but `$MD1` is empty (cascaded from the unwritten `md-encode-raw.txt`), so the pack call fails/produces nothing before line 65 tries to read it back. |

Four files (`md1.txt`, `md-encode-raw.txt`, `mk-encode-raw.txt`, `ms-encode.txt`)
have **no writer at all** in any committed script — confirmed by
`git log --oneline --follow -- design/journeys/transcript_pathological.sh`
(2 commits: `bdf954f` original, `2403d74` path-rename repair) and inspecting
both diffs — neither ever added a redirect/tee to these paths. Two
(`manifest.json`, `sysw-public.bin`) have a real writer whose command simply
never reaches completion at runtime, for reasons unrelated to each other
(sidecar version pin vs. cascaded empty argument).

## Regeneration attempt

```sh
cd design/journeys && mkdir -p out shots
bash transcript_pathological.sh > /tmp/patho_stdout.txt 2>/tmp/patho_stderr.txt
echo "EXIT=$?"
```
Result: `EXIT=0` (the script itself has no `set -e` and doesn't propagate a
sub-failure to its own exit code — a fresh run "succeeds" at the shell level
while its content is wrong throughout).

**stderr (3 lines, all from unquoted top-level reads that are NOT inside
`run()`, so they never reach the transcript at all — they'd be invisible if
someone piped only stdout to the PDF builder):**
```
transcript_pathological.sh: line 18: /scratch/code/shibboleth/mnemonic-engrave/design/journeys/out/md1.txt: No such file or directory
head: cannot open '/scratch/code/shibboleth/mnemonic-engrave/design/journeys/out/md1.txt' for reading: No such file or directory
sed: can't read /scratch/code/shibboleth/mnemonic-engrave/design/journeys/out/mk-encode-raw.txt: No such file or directory
```

**stdout — non-zero `[exit N]` markers** (`grep -n '^\[exit' /tmp/patho_stdout.txt`):
```
line 30: [exit 1]    step 2  "it does not fit one string"       — EXPECTED (script's own docstring: 3 designed refusals)
line 49: [exit 2]    step 4  "md inspect" with NO args           — BROKEN: cascaded from unwritten md1.txt
line 54: [exit 2]    step 5  "mk encode --from-md1 ''"           — coincidentally still exit 2, but WRONG reason (see below)
line 76: [exit 64]   step 7  "mk decode" with NO args            — BROKEN: cascaded from unwritten mk-encode-raw.txt
line 81: [exit 2]    step 8  "me bundle --preview"                — BROKEN: me/me-preview sidecar version mismatch
line 85: [exit 1]    step 9  grep on md-inspect's empty output    — BROKEN: cascaded, same root as step 4
line 102:[exit 3]    step 10 "me --in <ms1> --hex"                — EXPECTED (the 3rd designed refusal)
```
7 non-zero exits vs. the committed transcript's 3 (all designed refusals — see
below). 4 are new breakage; the pattern (empty args → decode/HRP errors) is the
same one F-210 already described for `transcript.sh`, now confirmed to hold in
`transcript_pathological.sh` as well.

**Committed `transcript_pathological.txt` for comparison**
(`grep -n '^\[exit' design/journeys/transcript_pathological.txt`): exits are
`0,0,0,0,0,0,1,0,0,2,0,0,0,0,0,0,0,3` — exactly 3 non-zero (1, 2, 3), matching
the script's own header comment: "captured verbatim — including the three
places the toolchain refuses." So the fresh run added exactly the 4 broken
exits identified above; the 3 by-design refusals are unaffected.

**A masked finding.** Step 5 is the journey's own "OBSTACLE 1" — it's *meant*
to fail. Committed: `error: md1 input rejected: wire-format version mismatch:
got 9, expected 4` (a real chunked md1 fed to `mk`). Fresh run: `error: md1
input rejected: codex32 decode error: string does not start with HRP md1` — a
*different* failure, because `$FIRST` is an empty string (cascaded from the
unwritten `md1.txt`), not because of the wire-version gap the step exists to
demonstrate. The intermediate-file bug currently hides the actual finding.

**Manual confirmation the real obstacle still reproduces**, feeding `mk 0.13.0`
a genuine chunk `md 0.13.0` just produced:
```
$ md encode --group-size 0 --force-chunked --path bip84 "$T" | grep '^md1' | head -1
md1fqgpcpqpz3m6jzzqqvzv5e4kqq0gfqye4m2zdeeqfw4e8cwvqy08m65x4mvaevc3h25saqzxp5rc3jll8k8
$ mk encode --xpub <key-00> --origin-fingerprint 73c5da0a --origin-path "m/84'/0'/0'" \
    --from-md1 md1fqgpcpqpz3m6jzzqqvzv5e4kqq0gfqye4m2zdeeqfw4e8cwvqy08m65x4mvaevc3h25saqzxp5rc3jll8k8 --group-size 0
error: md1 input rejected: wire-format version mismatch: got 9, expected 4
[exit 2]
```
Byte-identical wording to the committed transcript. The obstacle is a real,
unfixed compatibility gap between `md`'s and `mk`'s wire versions (consistent
with README's already-filed F-127: "`mk` vendors md-codec 0.34.0 against the
primary's 0.42.0") — not an artifact of drift.

**Manual confirmation the rest of the chain reproduces once fed real
intermediates** (steps 6-7, template-id → stub → key card → decode):
```
$ md inspect <3 real chunks> | grep -E 'policy-id|template-id'
wallet-descriptor-template-id: 5b48af35d4321a3ac18b43045e2523cc
wallet-policy-id: d3dda0f3a9ef2eef1f1de404b8a352a5
wallet-policy-id-fingerprint: 0xd3dda0f3
```
Stub `5b48af35` matches the committed transcript exactly. `mk encode
--policy-id-stub 5b48af35 ...` and `mk decode` of its own output both exit 0
and decode back to `xpub`/`origin_fingerprint`/`origin_path`/`policy_id_stubs`
identical to committed — **but the literal `mk1` string bytes differ** from
the committed transcript (`mk1qp30nap...` here vs. `mk1qpdw8zp...` committed),
reproducibly across 3 repeated runs (deterministic, not a nonce). Same
semantic content, different bytes, across the `mk` 0.12.1→0.13.0 bump — logged
under Tool drift below.

**Manual confirmation `me bundle` itself is unaffected by the version
mismatch** — dropping only `--preview`:
```
$ me bundle --in inputs-pathological/backup-strings.txt --manifest /tmp/manifest.json
me: wrote manifest to /tmp/manifest.json
me: backup needs 26 plates (25 public + ms1 on device): ...
[exit 0]
```
26-plate checklist, byte-identical text to the committed transcript's step 8
output (same plate ordering, same "TYPE ON DEVICE" line for plate 26). The
`--preview` sidecar check is the *only* thing step 8 is missing.

**No dedicated generator script exists** beyond `transcript_pathological.sh`
itself; the README's "Reproducing" section (`design/journeys/README.md`) gives
the two-line manual sequence (`bash transcript_pathological.sh > out/transcript.txt
2>&1` then `python3 build_pdf_pathological.py`) as the whole procedure — there
is no separate intermediate-producing step documented or scripted anywhere.

## Tool drift

CLI **surfaces** did not break: every flag the pathological script invokes
(`md encode --group-size/--force-chunked/--path`, `md inspect`, `mk encode
--xpub/--origin-fingerprint/--origin-path/--from-md1/--policy-id-stub/
--group-size`, `mk decode`, `ms encode --phrase/--no-engraving-card`, `me
bundle --in/--preview/--png/--manifest`, `me --in/--hex`) still exists
verbatim in today's `--help` output — checked all five binaries' `--help`
against every invocation line in the script; no "unrecognized argument" or
missing-subcommand error appeared anywhere in the run. What moved is
**versions and one wire-format-adjacent behavior**:

| tool | committed | today | what actually changed |
| --- | --- | --- | --- |
| `md` | 0.13.0 | 0.13.0 | Unchanged between the committed transcript and today (F-210's table doesn't list `md` — confirmed here it was already 0.13.0 when the committed transcript was made). |
| `mk` | 0.12.1 | 0.13.0 | CLI surface identical; **output bytes changed** for identical semantic input — `mk encode` with the same xpub/origin/stub now emits `mk1qp30nap...` instead of committed's `mk1qpdw8zp...` (verified deterministic, not a nonce, by 3 repeated runs). Decoded content is unchanged (`mk decode` recovers the same xpub/origin/stub either way). |
| `ms` | 0.14.1 | 0.16.0 | No behavioral difference observed in this journey — `ms encode --phrase "abandon...about"` produces byte-identical `ms1` output both versions (public BIP-39 test vector, deterministic). |
| `me` | 0.5.1 | 0.6.0 | New (already-shipped) sidecar version gate: `me bundle --preview` now refuses (`exit 2`) unless the co-located `me-preview` binary's `--version` exactly matches `me`'s own (`crates/me-cli/src/main.rs:653`, feature landed at `crates/me-cli/Cargo.toml:3` version bump history). This is what breaks step 8 — not a `me` CLI-surface change. |
| `me-preview` | 0.5.1 | **0.5.1 (stale)** | Not rebuilt alongside `me`: `target/release/me` is dated Aug 12 (0.6.0), `target/release/me-preview` is dated Aug 11 18:49 (still 0.5.1) — a local build-sync gap, not an upstream API change. Rebuilding the sidecar (not attempted here — out of recon scope) would likely clear step 8 on its own. |

No invocation needs an updated flag; every broken exit traces to either the
missing-intermediate cascade (four files) or the `me`/`me-preview` version-pin
(one command). This is different from the wallet-policy-CLI-surface hazard
recon usually flags — here the surface held and only build hygiene + capture
plumbing drifted.

## Minimum viable fix

**(a) Fix the generator — small, single session (~20-30 line diff, 2 files).**
In `transcript_pathological.sh` and `transcript.sh`: (i) redirect/`tee` each
producing command's stdout to its named `out/*.txt` file at the point it runs
(4 sites total: the `md encode --force-chunked` call, the `md encode` call in
`transcript.sh`, the two `mk encode` calls, the `ms encode` call); (ii) move
the top-of-script reads (`transcript_pathological.sh:17-18`) to occur *after*
their producer runs, not before. Rebuild `me-preview` alongside `me` (a `go
build`, not a script change) to clear step 8. TDD is immediate and cheap: rerun
the script, the non-zero-exit count should drop from 7 to the designed 3
(pathological) / from 9 to the designed 1 (operator), and the OBSTACLE-1 error
text should read the real wire-version-mismatch message again instead of the
empty-HRP one. This is the fix F-210 itself favors ("the version drift argues
for the first" of its two offered repairs), and is the only one that stops the
decay from recurring at the next version bump.

**(b) Re-record the transcript against today's tools — trivial (minutes), but
does not fix the root cause.** Just commit the current `out/*.txt` outputs (or
capture them by hand once) as the new `transcript_pathological.txt`. This is
what F-210 calls "commit the intermediates as fixtures" — fast, but "re-creates
the same decay one version bump later," and does nothing about defect 2 (the
scratchpad-path provenance gap) or the ordering bug at line 18. Not
recommended as the sole fix given the cycle needs this journey to stay
trustworthy past one more release.

**(c) Narrow the "nothing is illustrative" claim — trivial (minutes), doesn't
unblock reproduction.** Edit `design/journeys/README.md`'s opening line and/or
the PDF's front matter to state the claim is true as of a specific regenerated
date and cite F-210 for known drift, rather than asserting it unconditionally
while the generator is broken. Worth doing immediately regardless of (a)/(b)
timing, since it's the gap between what the document *claims* and what is
currently *true*.

Recommended combination: (a) now (it's cheap and it's what unblocks reuse as
an acceptance artifact), with (c) as a same-commit companion since it costs
nothing extra once (a) is verified.

## Does it exercise the new feature?

**No — not even once regenerated.** Two independent reasons:

1. **The journey never drives the emulator through the gather/expand/display/
   verify flow at all.** `build_pdf_pathological.py`'s only `shot()` calls
   (`design/journeys/build_pdf_pathological.py:265-279`) are `a00-boot`,
   `a01-input-seed` through `a07-after-passphrase` (typing the 12-word seed),
   and `b1-screen`/`b6-screen` (cutting the seed plate). There is no shot, and
   no scripted interaction, anywhere in this journey for scanning/entering the
   md1 chunks, the gather screen, the descriptor display, or address-verify.
   `README.md` explicitly notes NFC gathering is skipped because of F-126
   (a gathering flow freezes the emulator) — the journey was built to avoid
   this exact surface, not to demonstrate it.
2. **Even if it did, the wallet's own shape is out of scope for the shipped
   feature.** The #10b "Wallet Policy" work (`design/
   IMPLEMENTATION_PLAN_seedhammer_10b_md_walletpolicy.md:16`, decision D2,
   already shipped per `design/FOLLOWUPS.md:3486`, fork `main` `bb0e506`
   2026-06-19) projects to a verifiable `*bip380.Descriptor` only for
   singlesig + `wsh(sortedmulti)` + `sh(wsh(sortedmulti))`. The pathological
   wallet's policy is `wsh(or_i(and_v(v:after(...),v:sha256(...)),multi(...)),
   or_i(...))` — unsorted `multi` nested inside `or_i`/`and_v` with timelocks
   and a hashlock. Per D2 this is explicitly refused for descriptor-build/
   address-verify and only reaches template-only display. This is exactly
   the gap the "arbitrary tr()/wsh()" cycle (F-210's owning phase) exists to
   close — meaning the pathological wallet is the right *target* wallet for a
   future acceptance journey, but not yet, and not via this transcript alone.

**Conclusion: a new journey (or a substantial new section added to this one,
with real emulator shots of the gather → expand → descriptor-display →
address-verify path) is required regardless of whether F-210 is fixed.**
Fixing F-210 restores host-CLI reproducibility and stops an unbacked "nothing
is illustrative" claim from persisting, and gives the wallet-shape data
(chunked md1, template-id, stub derivation) a trustworthy source to build a
real device-flow journey from — but it does not, by itself, produce an
acceptance artifact for the new feature. F-210 is due this cycle (per its own
owning-phase note) but is not the thing that gates the feature's acceptance
walk; a new device-flow journey is the actual gating work, and it is blocked
independently on F-126 for a genuine NFC-driven walk (a keyboard/manual-entry
gather path, if the GUI supports one, would sidestep that).

## Open / could not determine

- No read-site for `out/manifest.json` was found anywhere in the current
  `transcript*.sh` / `build_pdf*.py` sources (grepped both). F-210 lists it
  among the six "read" intermediates; I can confirm it fails to be *written*
  (via the cascade/version-mismatch mechanisms above) but could not find code
  that subsequently *reads* it. Possibly it's read by a step outside the
  committed scripts (a manual/interactive check), or F-210's methodology
  counted it more loosely (e.g. as "consumed by the reproducibility
  narrative"). Flagging rather than asserting either way.
- Did not attempt to rebuild `me-preview` (would have crossed from recon into
  fixing the environment); its being stale (0.5.1 next to `me` 0.6.0) is
  measured from file timestamps and `--version` output, but whether a rebuild
  actually clears step 8 end-to-end (including PNG rendering) is untested here.
- Did not run `transcript.sh` (the 5-of-12 operator journey) myself — F-210's
  9-vs-1 numbers for it are already given as measured and out of this task's
  scope; I only independently verified the *pathological* script's read/write
  sites and reused the same file-name evidence where the two scripts share a
  pattern (`mk-encode-raw.txt`).
- Whether the emulator's `md1Gatherer` supports a non-NFC (typed/manual chunk
  entry) path that could sidestep F-126 for a future device-flow journey was
  not checked — flagged as relevant to sizing that future journey, not to
  F-210 itself.
