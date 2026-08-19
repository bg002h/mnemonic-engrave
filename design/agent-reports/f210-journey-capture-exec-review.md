# F-210 journey-capture execution review — commit `b822e4a`

Independent adversarial execution review. Read-only on source; nothing edited,
nothing committed. `git status` verified clean before and after.

**Scope:** the one question — does `b822e4a` make the operator-journey
transcripts genuinely regenerate, or does it merely make them *run* while
producing subtly wrong artifacts?

**Method:** both scripts executed twice each from a fully removed `out/`
directory (`rm -rf out`), with stdout and stderr separated; every captured file
inspected byte-wise; every degenerate path (empty capture, failing producer,
missing binary, short-arg `runcap`) exercised in a standalone harness; both
regenerated transcripts diffed against the committed `.txt` artifacts.

Toolchain present at review time: `md 0.13.0`, `mk 0.13.0`, `ms 0.16.0`,
`me 0.6.0`, `me-preview 0.6.0`, GNU bash 5.3.15.

---

## Verdict

**The fix works. The six intermediates now all have producers, the producers all
precede their consumers, both scripts are byte-for-byte reproducible across
runs, and the exit-code accounting is correct.** The specific defect F-210 named
is closed, and I independently reproduced the author's evidence *and found a
second instance of it the commit message does not mention*: the committed
**operator** transcript has the same stale-file signature as the pathological
one — `mk encode` prints `mk1qpf7f8pq…` while the `mk inspect` on the next line
consumes `mk1qpmn4up…`, a third distinct string. After the fix both are
`mk1qpj6vvpq…`. Every captured file is not merely present but semantically
correct, proven by the consumers themselves succeeding (`md inspect` returns the
full 11-key policy and the template-id that justifies the hardcoded stub; `mk
inspect` returns the right xpub/fingerprint/path/stub; `me` returns `[exit 3]`,
the ms1-specific refusal, which is only reachable if the string was recognised
*as ms1*).

**But the regenerated artifact is now internally inconsistent, and the commit
leaves it that way.** The journeys still consume two files that nothing in the
repo produces — `inputs/backup-strings.txt` and
`inputs-pathological/backup-strings.txt` — and those files are stale against
`mk 0.13.0`. Step 2 of the operator journey now prints card `mk1qpj6vvpq…` for
cosigner-00; step 4 then engraves 25 plates from a file whose card for that same
key is `mk1qpmn4upq…`. Both decode to *identical* fields, so `me bundle` exits 0
and nothing complains — the transcript reads as a coherent journey while showing
the operator one string and plating another. That is exactly the F-210 defect
class (a consumed file with no producer, masked by a stale copy), one layer up
at the input tier, and it is the reason I cannot call the artifact correct even
though the diff itself is sound. It is **pre-existing, not introduced by this
diff**, but the diff is what makes it observable and the commit message treats
it only as "a real behavioural drift" without noting that the journey now
contradicts itself.

Secondarily, the `|| true` on the capture grep is a genuine latent hole, and
there is exactly one call site where it degrades silently rather than loudly
(pathological step 6, which prints nothing at `[exit 0]`).

---

## Findings

### Critical

**None.** No path was found on which the current toolchain produces a wrong
captured file, a wrong exit code, or a wrong-but-plausible transcript.

### Important

**I-1. `inputs*/backup-strings.txt` is consumed by `me bundle` with no producer
anywhere in the repo, and is now stale against `mk 0.13.0` — so the regenerated
journey shows one key card and engraves a different one.**

*Failure scenario, reproduced:* run `transcript.sh` from a clean `out/`. Step 2
(`transcript.sh:73-76`) prints and inspects card
`mk1qpj6vvpqqsqhy6nx…` / `mk1qpj6vvppg9lsn2…` for cosigner-00. Step 4
(`transcript.sh:87-88`) feeds `inputs/backup-strings.txt` to `me bundle`, which
renders 25 plates and exits 0. That file's cosigner-00 card is
`mk1qpmn4upqqsqhy6nx…` / `mk1qpmn4uppg9lsn2…`. Decoding both with `mk 0.13.0`:

```
$ mk decode mk1qpmn4upqqsq…  mk1qpmn4uppg9…      # from the TRACKED input
xpub:                xpub6F794Lv12fi3eLBDjDYxbbVk5GrvFQrqb8QpYRLbr8dNtmdtogc2o5KiM3pLGd41phXxPw4ytcmwbDYmUfeRGuXE7CniQScVXrYnxz7AJE8
origin_fingerprint:  ae6647ee
origin_path:         48'/0'/0'/2'
policy_id_stubs:     726a6663
```

identical in every field to the freshly-encoded `mk1qpj6vvpq…`. Same semantics,
different string. The same holds in the pathological journey: step 7
(`transcript_pathological.sh:97-99`) produces `mk1qp30napqqsq4kj90…` for key-00,
while `inputs-pathological/backup-strings.txt:4-5` carries
`mk1qp0jgzpqqsq4kj90…` — again field-identical
(`xpub6CatWdiZiodmU… / 73c5da0a / 84'/0'/0' / 5b48af35`), textually different.

There are at least **three** generations of mk1 in play: the tracked inputs
(`qpmn4up` / `qp0jgzp`), the committed transcripts (`qpf7f8p` / `qpdw8zp` /
`qpghz4p`), and today's output (`qpj6vvp` / `qp30nap`).

*Why it is Important rather than Minor:* `design/journeys/README.md:6` states
"**Nothing in these documents is illustrative.** Every CLI block is the …" and
`transcript.sh:2-4` repeats it. `README.md:104-105` states the governing
principle in the repo's own words: "a claim that nothing here is illustrative
decays into a promise the moment the regeneration path stops being exercised."
An operator following the regenerated transcript literally would be shown a card
that is not on any plate. The failure is silent — `me bundle` validates the
stale strings and exits 0.

*Grep proving no producer exists* (repo-wide, excluding `third_party/`): the
only hits on `backup-strings` are the three **read** sites
(`transcript.sh:84`, `transcript.sh:85`, `transcript.sh:87`,
`transcript_pathological.sh:104`), plus prose in `design/DoNextList.md:289` and
two agent reports. Nothing writes them.

`design/journeys/transcript.sh:87-88`,
`design/journeys/transcript_pathological.sh:104`

---

**I-2. `grep -E "$keep" "$sout" > "$out" || true` accepts a zero-match capture
silently, and at pathological step 6 that degrades to blank output at
`[exit 0]`.**

*Mechanism:* the redirection truncates `$out` before `grep` runs, so a
no-match run leaves a 0-byte file and `|| true` discards the signal. Verified in
a standalone harness reproducing `runcap` verbatim:

```
--- B: command exits 0, NOTHING matches regex ---
$ bash -c echo ok-line; exit 0
ok-line
[exit 0]
  captured size: 0 bytes  (empty=silent)

--- D: pre-existing good capture destroyed by a non-matching rerun ---
$ bash -c echo somethingelse
somethingelse
[exit 0]
  after: [] size=0
```

*Downstream trace — most consumers fail loudly, which is the mitigating fact.*
With an empty capture:

| consumer | site | result | loud? |
| --- | --- | --- | --- |
| `md bytecode ""` / `md inspect ""` | `transcript.sh:66,67` | `md: codec error: codex32 decode error: string does not start with HRP md1`, rc 1 | yes |
| `mk encode --from-md1 ""` | `transcript.sh:74` | `error: md1 input rejected: …`, rc 2 | yes |
| `mk inspect` (no args) | `transcript.sh:76` | `error: expected at least one mk1 string …`, rc 64 | yes |
| `me --in <(blank)` | `transcript.sh:91,95` | `me: not a bech32 string …`, rc 4 (vs the expected rc 3) | yes |
| `me sysw pack --no-passphrase ""` | `transcript.sh:98` | `me: record 0 … is not a form this container can place`, rc 4, **no file written** | yes |
| `md inspect` (no args) | `transcript_pathological.sh:78` | `error: the following required arguments were not provided: <STRINGS>...`, rc 2 | yes |
| `bash -c "md inspect $MD1S 2>/dev/null \| grep -E 'policy-id\|template-id'"` | `transcript_pathological.sh:107` | blank output, rc 1 | partly |
| `bash -c "md inspect $MD1S 2>/dev/null \| sed -n 's/^wallet-descriptor-template-id: //p'"` | `transcript_pathological.sh:94` | **blank output, rc 0** | **NO** |

*The one silent site, verified directly:*

```
=== patho L94 with EMPTY MD1S ===
stdout=[]
exit=0   <-- run() would print [exit 0]

=== for contrast, the REAL L94 output ===
5b48af35d4321a3ac18b43045e2523cc
exit=0
```

Step 6's entire purpose is to display the template-id that justifies the
hand-derived stub `5b48af35` hardcoded at `transcript_pathological.sh:99`. Under
an empty capture the transcript would print the narrative echo lines at
`transcript_pathological.sh:87-92` asserting "this wallet's is
5b48af35d4321a3a…, giving stub 5b48af35", followed by a blank command output and
`[exit 0]` — a transcript that still makes the claim while silently showing no
evidence for it, and with plates already engraved from the hardcoded stub. This
is the "runs while producing a subtly wrong artifact" class the review asks
about.

*Trigger conditions:* the producer succeeds but its stdout stops matching
`^md1` — e.g. a future `md` prefixes or indents output, adds grouping by
default, or moves the strings to stderr. Not reachable today; the capture is
correct on `md 0.13.0`.

`design/journeys/transcript.sh:47`, `design/journeys/transcript_pathological.sh:44`
(the `|| true`); silent site at `design/journeys/transcript_pathological.sh:94`

### Minor

**M-1. The pathological journey hardcodes `--policy-id-stub 5b48af35` with no
cross-check against the template-id it prints two steps earlier.** Currently
consistent — step 6 prints `5b48af35d4321a3ac18b43045e2523cc` and step 7 uses
`5b48af35` — but the two are joined only by a comment. If `md`'s template-id
algorithm ever moves, step 6 shows the new id, step 7 keeps engraving the old
stub, and both exit 0. `design/journeys/transcript_pathological.sh:99` vs `:94`.

**M-2. `sed -n '1,2p'` hardcodes the chunk count of the mk capture.** Both
capture files hold exactly 2 lines today, so `1,2p` is equivalent to `cat`. If a
future `mk` emits 3+ chunks, the consumer silently receives a truncated chunk
set. `design/journeys/transcript.sh:76`,
`design/journeys/transcript_pathological.sh:100`.

**M-3. `runcap` leaks its two temp files if the script is killed.** No `trap`;
`rm -f` at `transcript.sh:48` / `transcript_pathological.sh:45` is only reached
on the normal path. Verified — after `kill -9` mid-command both
`/tmp/tmp.zYujzzjNhx` and `/tmp/tmp.HTGctVqPXz` survived (`LEAKED`). They land in
`/tmp` (TMPDIR unset), **not** in `out/`, so they cannot contaminate the journey
artifacts. Cosmetic.

**M-4. `runcap` called with exactly 2 arguments is a silent no-op.** `shift 2`
leaves `"$@"` empty, `set -u` does not fire on an empty `$@` in bash 4.4+, so the
function prints `$ `, runs nothing, writes an empty capture and reports
`[exit 0]`. Verified. No call site does this; recorded because it is the failure
mode a future edit would hit.
`design/journeys/transcript.sh:41-45`.

### Nit

**N-1. `transcript_pathological.sh` is mode `100644`** while `transcript.sh` and
`transcript_payload.sh` are `100755`, so `./transcript_pathological.sh` fails and
the journey only regenerates via `bash ./transcript_pathological.sh`. Pre-existing,
not touched by this diff.

**N-2. The two copies of the `runcap` docblock diverge.** The pathological copy
carries the stdout/stderr-ordering note (`transcript_pathological.sh:34-36`); the
operator copy (`transcript.sh:24-39`) omits it, though the same caveat applies to
its three call sites.

**N-3. Commit-message arithmetic.** The message says "manifest.json (11 KB, 25
plates)"; the manifest's own field is `"wallet_plates": 26` (25 rendered PNGs +
the ms1 plate implied by `"ms1_required": true`). Harmless, but it is a
hand-stated number.

---

## Captured-file correctness

All six verified by content **and** by the consumer succeeding, from a clean
`out/`.

| file | expected content | actual content (measured) | consumer | OK? |
| --- | --- | --- | --- | --- |
| `out/md-encode-raw.txt` | the md1 line only, no `chunk-set-id:` | 1 line, 37 B, `md1ytpqqxpp3zcpydzk0zdt492xzr7r9qxfc`. Producer stdout is 1 line (no chunk-set-id in the non-chunked case); the `note:` goes to stderr and is excluded by construction | `md bytecode`/`md inspect` (`:66,:67`) → rc 0; `MD1` (`:72`) → `mk encode` rc 0, `me --in` rc 0, `me sysw pack` rc 0 | **YES** |
| `out/mk-encode-raw.txt` (operator) | exactly the 2 mk1 chunk lines for `sed -n '1,2p'` | 2 lines, 193 B. Producer stdout is exactly 2 lines; `note: stdout is watch-only …` is on stderr | `mk inspect` (`:76`) → rc 0, returns xpub6F794…, ae6647ee, 48'/0'/0'/2', stub 726a6663 | **YES** |
| `out/ms-encode.txt` | one ms1 line surviving `grep '^ms1' \| tr -d ' '` | 1 line, 60 B, `ms10e ntrsq zu3dg … j06n7` (space-grouped). The 4 stderr lines incl. the private-key warning are excluded | `me --in <(MS1)` (`:95`) → **rc 3**, the ms1-specific NFC refusal — reachable only if the string parsed *as ms1*, which is the strongest available proof of correctness | **YES** |
| `out/md1.txt` (pathological) | the 3 md1 chunk lines, **without** the `chunk-set-id: 0x02038` line that shares stdout | 3 lines, 261 B, no `chunk-set-id`. Producer stdout is 4 lines with `chunk-set-id: 0x02038` first — **the regex is genuinely load-bearing here, and only here** | `md inspect $MD1S` (`:78`) → rc 0, `n: 11`, full wsh() template, `wallet-descriptor-template-id: 5b48af35d4321a3ac18b43045e2523cc`; `head -1` (`:82`) → OBSTACLE 1 reproduces at rc 2 | **YES** |
| `out/manifest.json` | written by `me bundle --manifest` | 11086 B (operator) / 11390 B (pathological), valid JSON, `"version": "0.6.0"`, 25 rendered plates | none in-script | **YES** |
| `out/sysw-public.bin` | written by `me sysw pack --out` | 88 B | `me sysw show` (`:99`) → rc 0 | **YES** |

Also confirmed: `out/mk-encode-raw.txt` (pathological) — 2 lines, 193 B,
consumed by `mk decode` (`:100`) at rc 0 with matching fields.

---

## Read-after-write audit

Every `$W/out/…` reference in both scripts, enumerated mechanically. `mkdir -p
"$W/out"` at `transcript.sh:53` and `transcript_pathological.sh:50` precedes all
of them.

### `design/journeys/transcript.sh`

| read | line | producer | line | order |
| --- | --- | --- | --- | --- |
| `grep '^md1' out/md-encode-raw.txt` | 66 | `runcap out/md-encode-raw.txt` | 64 | **OK** |
| `grep '^md1' out/md-encode-raw.txt` | 67 | same | 64 | **OK** |
| `MD1=$(grep '^md1' out/md-encode-raw.txt)` | 72 | same | 64 | **OK** |
| `sed -n '1,2p' out/mk-encode-raw.txt` | 76 | `runcap out/mk-encode-raw.txt` | 73 | **OK** |
| `--preview out/plates` | 88 | `rm -rf … && mkdir -p out/plates` | 86 | **OK** |
| `MS1=$(grep '^ms1' out/ms-encode.txt)` | 94 | `runcap out/ms-encode.txt` | 80 | **OK** |
| `me sysw pack … --out out/sysw-public.bin` | 98 | (this line is the writer) | 98 | **OK** |
| `me sysw show out/sysw-public.bin` | 99 | line 98 | 98 | **OK** |

Derived-variable order also checked: `MD1` assigned at 72 (after its producer at
64) and used at 74, 91, 98 — all after. **8 of 8 correct.**

### `design/journeys/transcript_pathological.sh`

| read | line | producer | line | order |
| --- | --- | --- | --- | --- |
| `MD1S=$(tr '\n' ' ' < out/md1.txt)` | 75 | `runcap out/md1.txt` | 71 | **OK** (was line 18, before its producer — this is the diff's ordering fix) |
| `FIRST=$(head -1 out/md1.txt)` | 82 | same | 71 | **OK** |
| `--preview out/plates` | 104 | `rm -rf … && mkdir -p out/plates` | 103 | **OK** |
| `sed -n '1,2p' out/mk-encode-raw.txt` | 100 | `runcap out/mk-encode-raw.txt` | 97 | **OK** |

`MD1S` (75) is used at 78, 94, 107 — all after. **4 of 4 correct.**

**No read precedes its producer in either script.** The author's ordering claim
is independently confirmed.

---

## Determinism

Each script run twice from a fully removed `out/` (`rm -rf out`), stdout and
stderr captured separately.

**`transcript.sh`:**
```
=== DIFF run1 vs run2 (transcript text) ===
IDENTICAL
=== DIFF run1 vs run2 (script stderr) ===
IDENTICAL
=== DIFF intermediates run1 vs run2 ===
md-encode-raw.txt        SAME
mk-encode-raw.txt        SAME
ms-encode.txt            SAME
manifest.json            SAME
sysw-public.bin          SAME
=== plates dir ===
PLATES IDENTICAL
```

**`transcript_pathological.sh`:**
```
=== DIFF pa run1 vs run2 ===
IDENTICAL
STDERR IDENTICAL
=== intermediates ===
md1.txt                  SAME
mk-encode-raw.txt        SAME
manifest.json            SAME
PLATES IDENTICAL
```

No timestamps, temp paths, UUIDs or ordering artefacts reach the transcript:
`mktemp` paths are never printed (`runcap` echoes `$*`, not the temp names), and
`manifest.json` contains no time/date/uuid/random field
(`grep -oiE '"[a-z_]*(time|date|stamp|uuid|rand)[a-z_]*"'` → no hits).
Both scripts' own stderr is empty. **Fully deterministic.**

### Exit-code accounting — measured against the committed artifacts

Author's claim reproduced exactly, at the same transcript line numbers:

| script | non-zero exits, fresh run | committed | lines |
| --- | --- | --- | --- |
| `transcript_pathological.sh` | 3 | 3 | 30 `[exit 1]`, 56 `[exit 2]`, 165 `[exit 3]` — identical in both |
| `transcript.sh` | 1 | 1 | 166 `[exit 3]` — identical in both |

### Regenerated vs committed transcripts

`transcript_pathological.txt`: 72 changed lines, all accounted for — 4 tool
version strings, absolute paths (the committed run was made from
`…/22fd28a4-…/scratchpad/journey2/`), and the mk1 strings. No unexplained
residue. Step 6 and step 9 outputs are byte-identical, including the
template-id.

`transcript.txt`: 85 changed lines. After removing paths and versions the
residue is the mk1 strings **plus one new line** — `public record 0: md1/mk1 —
confirmed` at regenerated line 180, inside `me sysw show`. That is a `me`
0.5.1→0.6.0 output addition, not a script defect.

---

## Angles examined and cleared

1. **Captured files correct, not merely present** — cleared; see the table. All
   six verified by content and by consumer success. The `chunk-set-id:`
   exclusion the commit message flags is real and only affects
   `out/md1.txt`; the regex is correctly required there.
2. **`|| true` swallowing grep's status** — **NOT cleared**; filed as I-2. Real
   latent hole; one genuinely silent degradation site at
   `transcript_pathological.sh:94`. Seven of eight consumers fail loudly.
3. **Ordering** — cleared. 12 of 12 `out/` reads follow their producer; verified
   mechanically, not by reading.
4. **stdout/stderr interleaving** — cleared. All 5 `runcap` call sites checked by
   running each command twice, once with `2>&1 | cat` and once with streams
   split. In every case stderr is a trailing note (`note: stdout is a keyless
   descriptor template`, `note: stdout is watch-only …`, and `ms encode`'s four
   trailing lines), so `runcap`'s stdout-then-stderr output is byte-identical to
   `run()`'s interleaved form. Confirmed end-to-end: the only non-path,
   non-version, non-mk1 diff against either committed transcript is the one
   `me` 0.6.0 line. (Rust's `Stdout` is line-buffered even when redirected, so
   this is ordering, not luck — but it is contingent on the tools never emitting
   stderr *before* stdout; a comment, not a guarantee.)
5. **`set -u` interaction** — cleared for real call sites. Fewer than 2 args is
   fatal and loud (`$2: unbound variable`, script exits 1) — verified. Exactly 2
   args is a silent no-op (M-4), unreachable from current call sites. `runcap`
   introduces no new unset-variable path; `local out="$1" keep="$2"` is a
   declaration-with-assignment that does not mask `$?`, and `rc=$?` is captured
   on the line immediately after the command.
6. **`mktemp` hygiene** — cleared for `out/` contamination (temps go to `/tmp`,
   TMPDIR unset). Leak-on-kill filed as M-3.
7. **Exit-code accounting** — cleared, and this was the most important thing to
   get right. `"$@" >"$sout" 2>"$serr"; rc=$?` captures the **command's** status
   before `cat`/`grep`/`rm` run. Verified with a command that exits 7 while
   `grep` succeeds → `[exit 7]`; and with a missing binary → `[exit 127]`, with
   the shell's `No such file or directory` message correctly reaching the
   transcript via `$serr` (identical routing to `run()`'s `2>&1`).
8. **Reproducibility across two runs** — cleared; see Determinism. Byte-identical
   transcripts, intermediates, manifests and plate PNGs.

Additionally cleared (not asked, but load-bearing for the verdict):

- **The core F-210 evidence reproduces and is fixed.** Pathological step 7 now
  shows `mk encode` printing `mk1qp30napq…` and `mk decode` consuming the same
  `mk1qp30napq…`; the committed artifact showed `mk1qpdw8zpq…` printed and
  `mk1qpghz4pq…` consumed.
- **A second, unreported instance in the operator journey.** Committed
  `transcript.txt` shows `mk encode` printing `mk1qpf7f8pq…` and `mk inspect`
  consuming `mk1qpmn4up…`. Now both are `mk1qpj6vvpq…`.
- **All six F-210 intermediates now have producers:** `md1.txt`
  (patho:71), `md-encode-raw.txt` (op:64), `mk-encode-raw.txt` (op:73,
  patho:97), `ms-encode.txt` (op:80), `manifest.json` (op:88, patho:104),
  `sysw-public.bin` (op:98).
- **`mkdir -p "$W/out"` is sufficient** — both scripts run correctly with `out/`
  entirely absent.
- **The rebuilt `me-preview` is the right one:** `me-preview 0.6.0` against
  `me 0.6.0`; `me bundle --preview` renders 25 plates and exits 0 in both
  journeys.

---

## Open / could not determine

- **Which artifact is canonical.** The commit deliberately does not re-record
  `transcript.txt` / `transcript_pathological.txt`, calling that "a separate
  decision". I did not attempt to resolve it. Note that as long as it is
  unresolved, the committed transcripts and PDFs continue to display the
  stale-file evidence (`mk encode` printing one string, `mk inspect`/`mk decode`
  consuming another) as though it were correct output — the exact artefact
  F-210 was filed about.
- **Whether I-1 should be fixed by regenerating `backup-strings.txt` or by
  pinning the toolchain.** Regenerating it moves the wallet ids and every plate,
  and `design/DoNextList.md:289` indicates that consequence is already known and
  being weighed elsewhere. I have not evaluated either option; per the brief I do
  not prescribe a remedy.
- **Why the mk1 encoding changed between 0.12.1 and 0.13.0** for
  field-identical input. I confirmed the semantics are preserved (both strings
  decode to the same xpub/fingerprint/path/stub) but did not investigate the
  encoding change itself — out of scope, and it belongs to `mnemonic-key`.
- **Whether stderr can ever precede stdout for these tools.** Empirically it
  never does at any of the 5 call sites today. I did not audit the tools'
  sources to establish that as an invariant, so the `runcap` docblock's claim is
  verified-by-measurement, not proven.
