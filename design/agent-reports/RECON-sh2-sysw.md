# RECON — SH2 `sysw` consumption: what was measured, and what surprised me

**Date:** 2026-08-27
**Artifact produced:** `design/SPEC_sh2_sysw_consumption.md`
**Trees:** `mnemonic-engrave` `master` = `25102c5` (57 ahead of `origin/master`),
`seedhammer` `main` = `0b656d7`

---

## 1. The headline

**The capability the brief asked me to specify is already shipped and pushed.**

I was asked to spec "teach the SeedHammer II to consume `sysw` packages and
engrave from them — including transactions." Within the first two tool calls a
top-level `sysw/` Go package appeared in the fork. Within ten, the whole thing
was demonstrably built, wired, tested and merged to `main`.

This is the same hazard the brief itself warned me about — *"old docs will lie
to you"* — running in the **opposite** direction. The brief warned that recon
docs describe work that has since landed. The thing that actually bit was that
the **framing of the cycle** described work that had since landed.

---

## 2. What I measured, with the command

| fact | command | result |
| --- | --- | --- |
| the fork has a `sysw` package | `ls sysw/; wc -l sysw/*.go` | 22 files, **1,827 lines** |
| it is wired, not orphaned | `grep -rln 'seedhammer.com/sysw' --include="*.go" . \| grep -v _test.go` | **18 non-test importers** |
| the transaction program exists | `wc -l gui/transaction.go` | **1,464 lines** |
| the suite is green | `go test ./sysw/ ./mt/ ./txqr/` ; `go test -run 'Sysw\|Transaction\|Payload' ./gui/` | EXIT=0, both |
| test count | `go test -list '.*'` on both | **41 + 68 = 109** functions |
| the host producer ships | `me --version`; `me sysw --help` | **me 0.7.0**; help already names the SH2 engraving journey |
| end-to-end pack works | `mt encode --qr \| me sysw pack --no-passphrase \| me sysw show` | exit 0; `sealed: false`, `pub_len: 447`, txid `2dcf2b97…` |
| region image size | `me sysw pack --region`; `wc -c` | **65536** |
| host refusals | four invocations, stderr captured to files | **exit 4** each, strings quoted verbatim in the spec |
| container never arrives by NFC | `grep -rn 'sysw.Reader\|SyswReader' --include="*.go" .` ; `grep -rln 'sysw' --include="*.go" nfc/` | every reader is flash or an embedded blob; **no hits under `nfc/`** |
| the G-P3.10 defect is still live | `grep -n 'TxidDisplay ==' gui/transaction.go` | **line 467** (the follow-up cites 449) |
| the Rust primary lacks the fields | read `crates/me-cli/src/sysw/tx.rs:35-70` | `TxSummary` has no output values, addresses, locktime or network |
| branch state | `git rev-parse`, `git branch -a` | fork `main` = `origin/main` = `ship/tx-engraving` = `0b656d7` |

---

## 3. What surprised me

### 3.1 The device already opens sealed containers

The brief said encrypted payloads were out of scope and asked me to spec a
refusal for *"a sealed container it cannot open."* There is no such state.
`gui/sysw_load.go` prompts for a BIP-39-word passphrase, **confirms the word
count before running the KDF**, and calls `sysw.Open`. A wrong passphrase
already refuses with its own sentence.

Had I written the requested refusal without measuring, I would have specified a
screen for a state the firmware cannot reach, and implied the removal of a
shipped, tested, load-bearing path.

### 3.2 A `sysw` container never travels over NFC — the two caps are not one cap

I expected the container to be an NFC payload. It is not: it reaches the device
by `picotool` at `0x10D00000`, and **only** that way. NFC carries *bare
records*, bounded by `gui/scan.go`'s 8 KiB scan buffer at **8191** — a different
limit, on a different thing, for a different transport.

This is exactly why `MaxSectionLen` was raised from 8191 to 32,734: the flash
path had inherited a cap belonging to a transport it never uses. The two numbers
being confusable is also the entire substance of the open follow-up F-247.

The brief's requested "truncated NFC transfer" refusal therefore has no trigger
at container level. It **does** have one at record level — `scanOverflow` →
`Content too large` — which I found only because I went looking for the negative's
boundary rather than asserting it.

### 3.3 The stale document was the *continuity* doc, not a recon doc

The brief told me to distrust `cycle-prep-recon-*.md`. The document that actually
misled was `CONTINUITY_tx_engraving_2026-08-25.md`, which states **"Nothing
pushed."** Measured: the fork's `main`, `origin/main` and `ship/tx-engraving` are
all `0b656d7`, and the fork has since taken four P5 review-fold commits its phase
table does not know about.

**A continuity doc decays faster than a recon doc**, because it is written to
describe a moment rather than a finding.

### 3.4 The right fact off the wrong module

The most dangerous near-miss in this recon. G-P3.14 says the review screen shows
no outputs or amounts, and asks whether that needs a Rust-first change.

`mt inspect` **does** print addresses and amounts — I watched it print
`bc1qc80qm4p…  0.05000000 BTC`. It would have been easy, and wrong, to conclude
"Rust already has this, so the Go port is just behind" and classify the work as a
convergence port under the Rust-primary rule's exemption (a).

That parser lives in the **`mnemonic-transaction`** repo. The Go `mt` package's
provenance pin names `me-cli/src/sysw/mt.rs and tx.rs` as primary, and
**`me-cli`'s `TxSummary` carries none of those fields either** — while its own
doc comment claims *"Everything a review screen needs."*

So it is a genuine Rust-first change, not a convergence port. The distinction
turned entirely on reading the provenance pin instead of the CLI output.

### 3.5 A follow-up already contained the ruling the code contradicts

G-P3.10 is not an open question. The operator ruled on 2026-08-25: *"we can just
engrave both."* The follow-up entry then records that **the code does something
worse than the ruling assumes** — it drops one transaction silently rather than
showing two confusing rows. Re-measured today, the drop is still live, at line
**467** rather than the cited 449.

A ruling recorded in a follow-up, against code that never changed to match, is
easy to read as closed. It is the opposite of closed.

---

## 4. Decisions I made, and what they rest on

1. **The spec documents shipped behaviour rather than proposing construction.**
   Rests on §2's measurements. Writing a build spec would have specified 3,300
   lines of existing, tested, merged code as new work.
2. **Three brief premises corrected in-document (§0.1) rather than silently.**
   Rests on the project's own standard that a wrong premise is a finding. Each
   would have produced a wrong section.
3. **G-P3.14 classified as needing a Rust-first change.** Rests on the
   provenance pin in `seedhammer/mt/mt.go` plus the field list in
   `me-cli/src/sysw/tx.rs:35`. §3.4 above.
4. **The KDF/AEAD refusal proposed as the encrypted-payload refusal.** Rests on
   `ParseHeader` returning distinct errors (`ErrVersion`, `ErrKDF`, `ErrAEAD`)
   that the GUI collapses into one sentence. It is the brief's requested refusal
   in the only form with a reachable trigger, and costs one screen — no
   passphrase surface, no new capability.
5. **Scope reported as a burndown, not a cycle.** Two defects and one ruling.

---

## 5. What I flagged as unknown rather than guessed

Recorded in the spec's §8, and repeated here because they are the honest limits:

- **No hardware.** Every device fact is source plus host-run Go tests.
- **No emulator walk.** A gate that has never executed is a hypothesis; §2's
  claims rest on unit tests, not on a walk.
- **The `mt1` text-plate path was never exercised** — only the `tx:`/QR pack path.
- **I did not read the two large sibling specs in full**
  (`SPEC_systemwide_payloads.md`, 93,890 bytes; `SPEC_engrave_transaction.md`,
  100,400 bytes) — only section lists and targeted greps. §2's statements were
  derived from **code**, so any disagreement is a real finding rather than a
  transcription slip.
- **Whether G-P3.14 should draw on `mnemonic-transaction`'s parser shape** is
  left open. I established the two parsers differ and which one the port tracks;
  I did not read that repo (the brief forbade touching it, and it is mid-push).
- **`Descriptor`/`Address` unreachability** established from `classify` on both
  sides; not exhaustively proven across every construction site.
- **The NFC negative is bounded** to the two searches named in §1.4. An ingest
  path naming neither `sysw.Reader`/`SyswReader` nor `sysw` under `nfc/` would
  not have been found.

---

## 6. Method notes worth keeping

- **`go` is not on `PATH` on this box.** The first test run returned **exit
  127**, which through a pipe would have looked like an empty result and been
  recorded as "no tests". The toolchain is at
  `/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go`.
- **The shell here rejects unmatched globs** (`(eval):1: no matches found`), so
  a `--include=*.rs` argument died before `grep` ran. Another exit code that had
  to be read rather than inferred from empty output.
- **I machine-checked the dispatched agent's four load-bearing claims** before
  folding any of them — the `Plate` struct, `toPlate`, `runEngraving`, the
  carousel entry, and the four per-flow refusal strings. All confirmed at the
  cited lines. The agent persisted its own report to
  `design/agent-reports/RECON-sh2-sysw-device-paths.md`, committed separately
  and before this one.
- **Line citations decayed within days.** G-P3.10's follow-up cites
  `gui/transaction.go:449`; it is at **467**. Re-grepping for the code rather
  than trusting the line number is what kept the finding true.
