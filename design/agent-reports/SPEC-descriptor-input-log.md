# SPEC-descriptor-input — authoring log

**Agent:** spec author, 2026-08-28. **Deliverable:**
`design/SPEC_descriptor_input.md` (982 lines, 11 sections), committed `ff9a0f2`.

This is the working record: what was measured and with which command, what had
to be decided and on what basis, what is flagged unknown, and an honest read on
scope. It is deliberately *not* a second copy of the spec.

---

## 1. How the measurements were taken

**The fork was never written to.** Every Go measurement came from a scratch
module at
`/tmp/claude-1000/.../scratchpad/probe` with

```
replace seedhammer.com => /scratch/code/shibboleth/seedhammer
```

and `go.sum` copied from the fork. Built and run with
`/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go` (`go version` →
`go1.26.3 linux/amd64`), `GOFLAGS=-mod=mod`. Two probe programs: 39 inputs
through `nonstandard.OutputDescriptor` + `sysw.Classify` + `seal.Classify`, then
18 more through `OutputDescriptor` with an `Encode()` → re-parse →
`KeyData`/`ChainCode` comparison and a `recover()` around `Encode()`.

**Exit codes were never read through a pipe.** Each run captured to a file, `rc`
written to its own file, then grepped.

**`md` was invoked by absolute path throughout** — never the bare name.

---

## 2. Two traps hit on the way, both worth carrying forward

**The installed `md` is stale and its version string does not say so.**
`/home/bcg/.cargo/bin/md` reports `md 0.13.0`; so does the repo's
`crates/md-cli/Cargo.toml`. The installed file is dated `Jul 11 23:30`, the repo
tip `fad69f1f` is `2026-08-27 19:41`, and **the installed binary has no
`descriptor` subcommand while the repo one does.** I had already run
`md --help` against the installed binary and started writing down a subcommand
list that was wrong. Every `md` measurement in the spec was re-run against
`/scratch/code/shibboleth/descriptor-mnemonic/target/release/md`. *This is
"measure by path, not by name" recurring with a new twist: the version string
matched and the binary still differed.*

**The seedhammer working tree is not on `main`.** `HEAD` was
`ship/tx-engraving` = `0b656d7`; `main` = `d402f18`, four commits ahead and
unpushed. Before trusting any working-tree read I ran
`git diff --stat 0b656d7 d402f18 -- nonstandard/ bip380/` → **empty**, so the
parser files are byte-identical across the two and the reads are valid against
both. The four commits are `gui/` chain tests plus the codex32 seam files.

---

## 3. Measured inventory — command, then result

| # | claim | command | result |
| --- | --- | --- | --- |
| 1 | host refuses a descriptor | `me sysw pack --no-passphrase --in desc.txt` | `rc=4`, the `Descriptors and addresses are not yet classifiable here` message |
| 2 | `me` exit constants | `grep -n 'const EXIT' crates/me-cli/src/main.rs` | `EXIT_OK=0:335`, `EXIT_USAGE=2:336`, `EXIT_REFUSED=3:337`, `EXIT_INVALID=4:338` |
| 3 | message site | `grep -rn 'Descriptors and addresses' crates/` | `src/main.rs:2425` (+ one test at `tests/sysw_cli.rs:457`) |
| 4 | `md encode` refuses a concrete descriptor | `…/target/release/md encode '<concrete desc>'` | `rc=1`, `md: template parse error: template contains no @i placeholders` |
| 5 | **`sysw.Classify` never yields `ClassDescriptor`** | Go probe, 39 inputs | **39/39 `ClassUnknown`**; 18 of those 39 are accepted by `OutputDescriptor`; `seal.Classify` returned `Descriptor` for exactly those 18 and `Address` for the one address input |
| 6 | admit-table cells | `grep -rn ClassDescriptor sysw/ gui/` | `gui/sysw_admit.go:37,39,45` — `progBundle`, `progMultisig`, `progWalletPolicy` |
| 7 | `OutputDescriptor` callers | `grep -rn 'nonstandard.OutputDescriptor' --include='*.go' .` (whole tree) | 2 non-test: `gui/scan.go:87`, `seal/record.go:206` |
| 8 | `md-codec` `me` actually links | `grep -n 'name = "md-codec"' -A 3 Cargo.lock` | `0.42.0`, `registry+crates.io` — not a path dep |
| 9 | `descriptor_to_template`'s real input | published crate `src/render.rs:19,52` | `use crate::encode::Descriptor` — the **md1 AST** |
| 10 | `md-cli` reusability | `grep -n '\[lib\]' crates/md-cli/Cargo.toml`; `ls crates/md-cli/src/lib.rs` | no `[lib]`, no `lib.rs` — **bin-only** |
| 11 | `me` has no descriptor library | whole `[dependencies]` block of `crates/me-cli/Cargo.toml` | no `miniscript`, no `bitcoin` |
| 12 | **md1 collapses a fixed use-site index** | 5 `md encode` runs differing only in the use-site path | `@0`, `@0/*`, `@0/0/*`, `@0/1/*` → **all `0x9bf18`**; `@0/<0;1>/*` → `0x16d62` |
| 13 | …and it is a different wallet | Go `address.Receive(desc,0)` vs `md address` on the encoded set | `bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a` vs `bc1qu2cc6t70nm0tw0v3tsmgur33gjnw2a32czk6xatccky9jpjxj4eqcedjh9` |
| 14 | md1 **does** preserve the multipath wallet | same two routes, `<0;1>` form | **identical** — `bc1qadgf37zk…`, Go and Rust agreeing across languages |
| 15 | no label field in md1 | published `src/tlv.rs:24`, all five fields read | `use_site_path_overrides`, `fingerprints`, `pubkeys`, `origin_path_overrides`, `unknown` — no title |
| 16 | seam-vector file shape | `python3` over the JSON + `sha256sum` | 6840 bytes, 8 rows, sha `3d53ef88…eb6a`; fork's copy at `d402f18` hashes identically |

---

## 4. The four formats — precedence, and what it actually decides

Read from `nonstandard/parse.go:36`, then probed.

**Order: 1 BlueWallet (only if `Title != ""`) → 2 `bip380.Parse` → 3
`{label, descriptor}` JSON → 4 promoted bare key.** First success returns.

**No probed input is claimed by two branches**, and it is structural, not luck —
the four grammars are disjoint on their first token. I attacked it deliberately
with `{"label":"Name: x","descriptor":"wpkh(…)"}`; branch 1 fails on it because
it splits the whole single line on `": "` and gets the key `{"label":"Name`.

**So precedence decides the DIAGNOSTIC, not admission**, and that is the finding:

- Branch 3 **returns even on failure**, so a JSON wrapper surfaces the real
  error — measured `{}` → `bip380: script: missing '('`.
- Branches 1, 2 and 4 fall through, and the terminal
  `errors.New("nonstandard: unrecognized output descriptor format")` **destroys
  every reason on the way**. A bad checksum reports the generic message, not
  `bip380: invalid checksum`. Eleven distinct causes, one message.

**Item 4 got its own scrutiny, as briefed.** The qualifying paths were read out
of `Script.DerivationPath()` (`bip380/bip380.go:122`), not assumed from
BIP-44/49/84: `P2PKH` = `m/44'/0'/0'`, `P2WPKH` = `m/84'/0'/0'`, `P2SH_P2WPKH` =
`m/49'/0'/0'`. Three hardened components, coin 0, **account 0**. The loop lists
those three only, so `86'` (taproot) and `48'/…` (multisig) are excluded even
though the same function defines them. Fifteen probed near-misses are in spec
§4.5; the three that will actually bite an operator are **account ≠ 0**,
**`86'`**, and **`[fingerprint]` with no path** (`ParseKey` needs
`originAndPath[8] == '/'`, and 8 characters is one short).

**It did not need a ruling to specify** — the accept set is fully determined by
code I could read and run. What it needed was for the paths to be read from the
function rather than from convention.

---

## 5. Defects found in the Go parser (recorded, not fixed)

Under the Rust-primary rule these are fixed later, in their own cycle; the spec
narrows the host past them instead.

1. **A reachable panic.** `Format:` absent from a BlueWallet file ⇒ `Script`
   stays `UnknownScript` ⇒ `Descriptor.Encode()` hits
   `panic("unknown script")` (`bip380.go:167`). Caught with a `recover()` in the
   probe. `Name: only\n` — one line, no keys — reaches the same panic.
2. **No threshold check at all.** `sortedmulti(0, …)` and `sortedmulti(5, …)`
   with two keys both parse. The second is unsatisfiable: funds locked forever.
3. **Structurally impossible descriptors accepted:** `tr(sortedmulti(…))`
   (taproot multisig is `multi_a`/`sortedmulti_a`), `wpkh(sortedmulti(…))`,
   `pkh(sortedmulti(…))`.
4. **`Derivation:` after the key lines silently empties every key's origin.**
   Re-encodes as `[dc567276]xpub66C1RXMi…` — a different string that **does not
   re-parse**. *I checked before writing it up whether this was a wrong-wallet
   bug: `KeyData` and `ChainCode` are unchanged, so it is a round-trip and
   display break, not a funds break.* Stated that way in the spec; the
   over-claim was tempting and would have been wrong.
5. **CRLF is refused by branches 1, 2 and 4**, and a trailing `\n` is refused by
   2 and 4. A wallet export saved from an editor, or anything through a Windows
   tool, fails today. Only the JSON branch tolerates whitespace.

---

## 6. Every decision I had to make, and what it rests on

| decision | ruling | basis |
| --- | --- | --- |
| Which descriptor parser does `me` get? | **A small Rust parser for exactly seven script shapes**, not a `bip380` port and not `rust-miniscript`. | A line-for-line port would make the *new Rust primary* bug-compatible with §5's five defects by construction — and §3 of the spec makes Rust the primary from the moment it lands. `rust-miniscript` means adding `miniscript` + `bitcoin` to a crate that has neither (measured), in order to reject most of what it can parse. |
| What grammar is admitted? | `pkh` / `wpkh` / `sh(wpkh)` / `tr(KEY)` / `wsh(sortedmulti)` / `sh(wsh(sortedmulti))` / `sh(sortedmulti)`, with `1 ≤ k ≤ n`. | It is the intersection of what `bip380.Parse` accepts and what is a real descriptor. Narrower than the device is **free** under `host ⇒ device`; wider is the only unsafe direction. |
| Is the closed set stated as a set or as an exclusion list? | **As a set.** | First draft said "everything outside is refused" and then listed five exclusions — which reads as the closed set. Rewritten so the rule is over the accept set, with the measured exclusions labelled measured and the two inspection-only ones (`wsh(KEY)`, `sh(KEY)`) labelled inspection. *"Negatives inherit the search scope."* |
| Does `me` trim whitespace when the device does not? | **Yes.** | It cannot break the seam: the record packed is the canonical re-encoding, and `sysw` records are LF-separated so a record cannot contain a newline anyway. The device never sees what the host absorbed. Mechanical, not a judgement. |
| `tpub` promotion? | **Refused**, though the device accepts it. | A bare `tpub` promotes via a version byte that maps to the **mainnet** path `m/44'/0'/0'`. Inferring a whole wallet from that is an assumption the host declines. Safe direction. |
| Silent promotion? | **No** — `me` prints the inferred descriptor before packing. | The operator supplied one line and is getting a wallet. Follows the host-side-first ruling. |
| `--as md1` on `/0/*`? | **Refuse, naming `--as descriptor`.** | Measurement 12+13: it silently changes the wallet, and `/0/*` is the shape of the fork's own shipped fixture. This is the single strongest normative claim in the spec. |
| The dropped label? | **Warn, do not refuse.** | `gui/gui.go:3161` uses `Title` for body text only. Cosmetic — a refusal would be worse than telling the operator nothing, which is the bar the journey rule sets. |
| Shell out to `md encode` for `--as md1`? | **No** — build the AST in-process. | `md-cli` is bin-only (measurement 10), so there is no library option; and a CLI's stdout as an internal channel is the exact composition hazard `SPEC_constellation_cli_uniformity` §3 documents. The md1 AST is fully public in the crate `me` already links. |
| Reorder S2/S3, given S2 needs a device change and S3 does not? | **No. Specified as ruled, and the asymmetry flagged as an open operator question.** | It is a scheduling preference that belongs to the operator. I did not spend a `fable` dispatch on it — the brief reserves that for a tie-breaker that cannot be settled by running something, and everything factual here *was* settled by running something. |

**No `fable` agent was dispatched.** Nothing remained that a command could not
settle.

---

## 7. Flagged unknown rather than guessed

All of these are in spec §9, which exists so the document does not claim the
residue by omitting it.

1. **Nothing ran on hardware.** No payload written, no screen, no plate.
2. **The three admit-table cells have never executed** — by construction, since
   no input can reach them. Not "lightly tested". *Closure-is-lens-closure,
   second clause: a gate that has never run is a hypothesis.*
3. **The md1 address equality is ONE data point** — `wsh(sortedmulti(2, …
   /<0;1>/*))`, mainnet, receive index 0. Not single-sig, not `sh(wsh(…))`, not
   `tr`, not change, not index > 0, not testnet. The spec makes `address_0` a
   per-row vector requirement to close it.
4. **Published-vs-tree `md-codec`.** I verified the published 0.42.0 exports the
   same names with the same signatures. I did **not** verify the tarball is
   byte-identical to the tree.
5. **TinyGo has not been asked** whether a new `sysw.Classify` arm compiles for
   the RP2350 target. The fork has been caught by that gate before.
6. **Negative-claim scope, stated in the spec.** "No Rust counterpart to
   `nonstandard`" covers `mnemonic-engrave/crates` and
   `descriptor-mnemonic/crates` plus `me-cli`'s complete dependency block. It
   does **not** cover `mnemonic-toolkit`, `mnemonic-secret`, `mk-codec` or
   `mnemonic-transaction`, which were not searched.
7. **§6's refusal texts have not been walked with the operator.** A refusal
   table is exactly the shape the live journey walk finds things in, and it
   should be walked before the plan closes.

---

## 8. Build gate — run before the commit, not left to a reviewer

- **22 `file:line` citations resolved against source.** **Eight were wrong** and
  were corrected: `parse.go:77`→`:80`, `parse_test.go:20`→`:22`,
  `render.rs:20`→`:19`, `use_site_path.rs:48`→`:49`, plus the probe counts.
- **Probe counts recomputed from the captured output, not transcribed.** The
  draft said "41 inputs, 19 accepted"; `grep -c` over the saved probe output
  says **39 and 18**. Both fixed. *This is exactly the transcription error the
  never-hand-count rule exists for, and it survived my own first pass.*
- **Every `§` reference resolved.** A script parsed the headings and checked all
  40 references; two pointing at `SPEC_constellation_cli_uniformity` §2 and §6h
  were only qualified by a lead-in sentence, and `§2` **also exists in this
  document** — the false-clean cross-reference the reference spec warns about.
  Both were made explicit inline.
- Not covered: the spec carries no compilable Rust, so
  `scripts/plan-build-gate.sh` has nothing to extract. The refusal *texts* are
  prose and unverifiable until code exists.

---

## 9. Honest read on scope

**Not "two cycles". Roughly the size it was framed as, with the halves swapped.**

- **S1 (cascade + vectors) is real and shared**, and is the only part both
  output forms need. The parser is ~7 script shapes plus one key-expression
  grammar. Small.
- **S3 (`--as md1`) is smaller than expected and needs no device change.** The
  md1 AST is fully public in the crate `me` already links, and `ClassMDMK`
  already classifies and is already admitted by the same three programs. It
  could be demonstrated the day it compiles.
- **S2 (`--as descriptor`) is larger than framed**, because `sysw.Classify`
  needs a descriptor arm in both languages. Under the Rust-primary rule that is
  exactly this cycle's shape, so nothing is bent — but it means **the first
  shipping phase is the one that cannot be shown to the operator without a
  firmware build and a flash.**

**The one thing I would put in front of the operator before a plan is written:**
S2-before-S3 was ruled when both looked like host work. It no longer is. S3
reaches a real device with no reflash; S2 does not. That is a scheduling call,
and it is left open in spec §8 rather than decided here.

**One out-of-scope observation, filed nowhere yet:** `md encode --help`'s
headline example is `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))`, and `multi` is
precisely the form the device's descriptor parser refuses. The two tools'
example descriptors disagree about the only multi form the device takes. Worth a
follow-up against `md`; not this spec's to change.
