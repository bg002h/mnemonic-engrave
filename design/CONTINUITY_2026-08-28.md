# CONTINUITY — 2026-08-28

**Resume with:** `/resume-f412` — or just say *"F-412"*. Read this file first, then
`design/FOLLOWUPS.md`'s F-412 entry. Nothing else needs reading to start.

## Next task: F-412, then the descriptor spec gap

**F-412 is a RULING, not code.** `--path` replaces a template's declared origin
wholesale, so after F-411 the keyed note can fire about a declaration the card no
longer carries: `--path bip84` over `wpkh(@0/84'/0'/0'/0/*)` with a depth-3 key
notes an overshoot while the card is consistent.

The implementer refused to suppress it, correctly — narrowing was not authorised
either, and **tier 1 has always behaved this way**, so a tier-2-only patch would
split the tiers. **One decision covering both tiers is owed.** Route it to a
`fable` consult; it is exactly the shape the last two rulings were.

**Then** return to `SPEC_descriptor_input.md` (below), which is written and parked.

## What shipped today — the whole CLI-uniformity cycle

P0..P3 are complete, merged and **pushed**; P4 is deferred by operator ruling
(F-370, first post-release item). `mnemonic-io-lib 0.1.0` is on crates.io, tagged
`mnemonic-io-lib-v0.1.0` at `7785a69` — the exact tree `cargo publish` packaged.
All three consumers take it from the **registry**, not a git rev.

Test movement, each measured: `mt` 237→275 · `ms` 414→476 · `md` 805→863 ·
`mk` 337→370 · `mnemonic` 3960→4007.

## UNPUSHED — the first thing to clear

```
mnemonic-engrave      16 commits
descriptor-mnemonic    3 commits   (F-410, F-411, opt-level profile)
mnemonic-secret        1 commit    (+ untracked err.txt, not ours)
mnemonic-key           1 commit    (+ 1 untracked, not ours)
mnemonic-toolkit       1 commit    (+ 38 untracked cycle-prep files, not ours)
seedhammer (fork)      4 commits   BLOCKED — see below
```

**All five constellation repos have branch protection.** Use the `ci/staging`
ritual: push to `ci/staging`, watch every required context **by name**, push the
branch, delete the staging ref. Freeze the tip for the whole window — a commit
landing mid-window is accepted against the older gated ancestor and prints
*"Bypassed rule violations"*.

**The fork push is blocked by Claude Code's permission classifier**, not by
anything in the repo. `seedhammer` `main` is at `d402f18`, validated (`go build`
clean, full `gui` suite green, 12 chain tests). It needs the operator to allow
the push; there is no legitimate substitute, since `git push` *is* the natural
tool.

## Parked and ready: SPEC_descriptor_input.md

**982 lines, committed, R0 not started. No code exists and none may until it
closes 0C/0I.**

The gap: the device accepts descriptors via `nonstandard.OutputDescriptor`
(BlueWallet, plain BIP-380, `{label,descriptor}` JSON, and a promoted bare key)
and admits `ClassDescriptor` in three programs. The host packs none of them.

Operator rulings, binding: **broadly accepting on input, expressive on output**;
two output forms with `--as md1` / `--as descriptor` **chosen by the user, never a
silent fallback**; both parse host-side first.

**Three findings in the spec change its shape** — read §2.6 and §5.3 before
planning:

1. `sysw.Classify` **never returns `ClassDescriptor`** — 39 of 39 probed inputs
   returned `ClassUnknown`. The three admission cells are live code no input can
   reach, so `--as descriptor` needs a **device arm too**.
2. `descriptor_to_template` takes the md1 AST, not a BIP-380 descriptor — it is
   the *output* direction and is **not** the head start I claimed when briefing.
3. `md-cli` has no `[lib]`, so its 2619-line template parser is unreachable from
   `me`.

**Phase order is an open question the spec deliberately left to the operator:**
S3 (`--as md1`) needs no device change and could be demonstrated the day it
compiles; S2 (`--as descriptor`) needs a firmware build and flash. The spec
specifies S2 first as ruled. **I recommend swapping them.**

## Also parked

- **P3's three joins** — toolkit release → GUI pin bump + mirror → journey
  goldens (the last waits on P2, which has shipped). Unblocked, not started.
- **SPEC_sh2_sysw_consumption.md** — documents shipped behaviour. Its two open
  defects are **G-P3.10** (transaction candidates merge on the derived txid, not
  bytes, so a byte-different twin sharing a txid is **silently dropped** — the
  operator ruled *"engrave both"*, and the code does worse than the ruling
  assumes) and **G-P3.14** (the review screen shows no outputs, amounts, fee,
  locktime or network; **Rust-first**, owned by `me-cli`'s `sysw::tx`).

## Standing facts worth not re-deriving

- **Tests are fast now.** `[profile.test] opt-level = 2` is set in all five Rust
  repos: `md` 29.9s→0.80s, `mk` ~4s→0.075s. **`mnemonic-engrave` did not move**
  (33.6s→32.1s) because its time is real interactive zsh/fish on a pty with
  30s timeouts. Verified safe: `debug_assertions = true` and overflow still
  panics at `opt-level = 2` — this is *not* `--release`.
- **Go:** use `/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go`.
  `go.mod` pins 1.25.10 but the tree uses a 1.26 API (`t.ArtifactDir`), so 1.25
  **cannot build `./gui/`**.
- **`md` is aliased to `mkdir -p`** in the login shell. Always absolute paths.
- The **shared-vector gate pattern** (one file, vendored byte-identically, sha256
  pinned as a literal in both tests) is proven twice now — codex32 seam and the
  chain fixtures. Reuse it rather than inventing.

## The lesson of the day, because it cost the most

**I produced three successive wrong diagnoses of a non-defect**, each refuted by
one more measurement: md1 cannot represent a fixed use-site index (wrong — the
encodings differ); the renderer drops it (wrong — `UseSitePath` has no such
field); the parser relocates it into the origin, producing a different key
(wrong — the addresses are identical).

The error underneath all three was **asserting past a comparison I never made**:
I never derived an address through the md1 round trip before claiming they
differed. One of them came from reading only the **first line** of `md decode`,
which made me report it printed no origin when it prints one in a `note:` line.

The operator's pushback — *"it sounds like you want to NOT be permissive on VALID
input"* — was right and stopped a refusal that would have been wrong.

**Read the whole output. Complete the comparison before naming a cause.**
