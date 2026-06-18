<!--
Persisted verbatim. opus-architect MANDATORY whole-diff execution review of Cycle D Phase
D1 (SLIP-39 crypto port). Reviewer agentId a6a8fad838e40db94 (re-dispatch after a transient
API socket drop on the first attempt agentId ae802114665077ea5 which returned no verdict).
Diff: feat/slip39-recovery-crypto 20fa4c4..95f51d4 (7 commits). Verdict: GREEN — 0C/0I,
cleared to merge. Ran -race; proved no use-after-wipe on caller-owned Share.Value; confirmed
group-order determinism, panic-safety, port fidelity; independently round-tripped all 15
fixtures via the Rust oracle. The text below is the agent's report exactly as returned; do not edit.
-->

# EXECUTION REVIEW — Cycle D D1 (SLIP-39 crypto port) whole diff

**Reviewer:** opus architect (adversarial, read-only whole-diff execution review)
**Commit range:** `20fa4c4..95f51d4` (7 commits) — base fork `main` `20fa4c4`
**Worktree / branch:** `/scratch/code/shibboleth/seedhammer-wt-slip39-d1` @ `feat/slip39-recovery-crypto`
**Date:** 2026-06-18
**Scope of diff:** 1155 insertions, all under `slip39/` (gf256.go, lagrange.go, feistel.go, combine.go, share.go + tests + testdata). `git diff --name-only 20fa4c4..HEAD | grep -v '^slip39/'` → empty. Confirmed.

## Reproduced gates (tails)

```
go test ./slip39/ ./gui/ ./bip39/ →  ok seedhammer.com/slip39 0.195s
                                      ok seedhammer.com/gui    4.659s
                                      ok seedhammer.com/bip39  0.098s
go test -race ./slip39/         →  ok seedhammer.com/slip39 2.126s
go vet ./slip39/ ./gui/ ./bip39/ →  exit 0 (clean)
gofmt -l slip39/                →  (empty — clean)
grep -rn 'math/big' slip39/     →  only the "NO math/big" comment in gf256.go
```

Targeted `-run 'TestCombine|TestParseShare|TestGF256|TestInterpolate|TestFeistel|TestRecoverSecret|TestWipe'` — all PASS (positives recover exact hex; negatives → correct sentinels).

## Per-focus findings

**1. Port fidelity (committed Go vs Rust) — crypto core.** Verified line-by-line against the Rust oracle:
- **gf256.go**: generator-3 table build `x=(x<<1)^x`, reduce `0x11b`, `exp[255]=1`; `gfMul` `s>=255 → s-=255`; `gfInv` `exp[(255-log)%255]`; `gfDiv=mul(a,inv(b))`. Identical to gf256.rs. `TestGF256MulInvDiv` (AES-inverse pair 0x53·0xCA=1, a·inv(a)=1 ∀a≠0) passes.
- **lagrange.go**: XOR Lagrange basis, per-byte interpolation, distinct-x precondition. Identical to lagrange.rs (Go omits Rust's `assert xi!=xj` but the duplicate-x is rejected upstream in combine; see focus 4).
- **feistel.go**: round order `[3,2,1,0]` (`for i:=3; i>=0; i--`), `l[j]^=f[j]` then swap `l,r=r,l`, output `R||L`; salt `"shamir"||be16(id)` / `nil` when extendable; iters `(10000<<e)/4`; password `[]byte{byte(i)}||pass`. Matches feistel.rs decrypt exactly. `itersPerRound(0)=2500`, salt assertions pass.
- **combine.go**: six cross-share sentinels in the same order as mod.rs; group-by with **sorted gids** (`sort.Ints`); per-group uniform-mt + distinct-idx + strict `len(gs)!=mt`; strict group-threshold count; `recoverSecret` `T==1` no-digest / `T>=2` digest via `subtle.ConstantTimeCompare` (Rust uses `!=` on a fixed-length slice — Go's constant-time variant is a strict improvement, semantically equivalent). Faithful.
- **share.go decodeValue**: byte-oriented MSB-first `(w>>(9-i%10))&1`, leading-pad zero check, `{20,23,27,30,33}→{16,20,24,28,32}` via `valueWords=len-7, padBits=(10·vw)%16`. Header bit-fields independently re-derived: the 40-bit `hdr` extraction (`identifier=hdr>>25`, `ext=(hdr>>24)&1`, etc.) is algebraically identical to the Rust two-word id_exp/share_params decode. Faithful.
- **No crypto transcription error found.**

**2. Go-specific hazards.** No defects.
- *Feistel append/swap aliasing:* `l`/`r` are independent `append([]byte(nil),…)` allocations; the salt is rebuilt fresh each round from `[]byte(nil)` (never aliases the reused `salt` or `r`); `R||L` built fresh from the correct post-loop vars. No aliasing.
- *Caller-owned `Share.Value` mutation / use-after-wipe (the D2 concern):* `Combine` wipes only the **freshly-allocated** intermediates (`gv` group-share buffers, `ems`, and the transient `s`/`d` inside `recoverSecret`). `recoverSecret(T==1)` returns a fresh copy; `interpolateSecretAt` reads inputs read-only. I added two throwaway adversarial tests proving (a) all `Share.Value` buffers are byte-identical after `Combine` across idx {0,3,17,35,42}, (b) `Combine` is idempotent on the same parsed shares, (c) `recoverSecret` doesn't mutate its input y-slices — **all pass.** D2's "passes parsed shares, re-combines" usage is safe.
- *Map nondeterminism:* the `gids` sort is applied; a throwaway test combining idx {17,35} in forward vs reversed input order recovers identical secrets. Pass.
- *decodeValue bit indexing:* off-by-one ruled out — identical to Rust, and the 23/27/30-word fixtures (with 2/4/6 pad bits) round-trip exactly.

**3. Scrubbing (SPEC §4.8).** `wipe` zeroes the right buffers and never the caller's `Value` (proven in focus 2). `TestWipeZeroes` + `TestRecoverSecretWipesOnDigestFail` confirm the zeroing and the digest-fail scrub path. Comment correctly states best-effort-not-guaranteed under TinyGo GC — not overclaimed.

**4. Panic-safety (SPEC §4.4).** Malformed sets error without panic: `TestCombinePanicSafety` (dup-member, bad length, empty, dup-x) passes. My throwaway test forcing a duplicate member index through `Combine` confirms it errors (`errDuplicateMemberIndex`) **before** any interpolation, so `gfInv(0)`/`gfDiv(_,0)`/dup-x are unreachable; a separate test confirms `gfInv(0)` does panic in isolation (the guard is real) but the combine path never reaches it.

**5. The two flagged deviations.** Both benign: (a) `slip39_vectors.json` keyed by upstream index as a string→4-tuple object — I diffed all 11 kept indices {0,3,17,35,42,1,4,5,9,12,13} against freshly-fetched `trezor/python-shamir-mnemonic/master/vectors.json`: **byte-faithful** (desc, mnemonics, master_hex all match; map-keying addresses the right vectors). (b) `wipe` defined once in feistel.go, referenced from combine.go — single definition, compiles clean, no duplicate.

**6. Testdata integrity.** No I1 fabrication. Official vectors byte-faithful (above). Fixtures: 15 cases, word-count distribution `{20:3, 23:3, 27:3, 30:3, 33:3}` and secret sizes `{16,20,24,28,32}` across 1-of-1 / 2-of-3 / group topologies — genuinely covers the intermediate 23/27/30-word unpack (not silently 20/33-only). `TestCombineFixtures` asserts `recovered == secret_hex` (independent declared value, not tautological). **Independently cross-checked**: a throwaway Rust integration test recovered all 15 committed fixtures via the `mnemonic-toolkit` oracle to their declared `secret_hex` ("ALL 15 COMMITTED FIXTURES RECOVER VIA RUST ORACLE") — the fixtures are genuine SLIP-39 and Go/Rust agree.

**7. Scope/guards.** `Share.Value` addition doesn't break GUI: `TestConfirmSLIP39Render` + `TestEngraveSLIP39BackoutRecognized` pass. Diff touches only `slip39/` (no GUI/codex32/bip39/backup). `errUnsupportedSize`, `wordsShort`, `wordsLong` fully removed (repo-wide grep → none); the Tier-1 256-bit rejection is gone and all five word counts are accepted.

## Findings

- **CRITICAL:** none.
- **IMPORTANT:** none.
- **MINOR:** none. (Note, not a finding: `interpolateAt` drops Rust's defensive `assert xi != xj`; this is correct by construction because `Combine` rejects duplicate member indices before interpolation, and the panic-safety test confirms the dup-x path never reaches the field math. The Go `subtle.ConstantTimeCompare` for digest verification is a security improvement over the Rust `!=`.)

## Verdict

**GREEN — 0 Critical / 0 Important. Cleared to merge.**
