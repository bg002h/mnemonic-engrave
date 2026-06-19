<!--
Persisted verbatim. opus-architect MANDATORY whole-diff execution review of the T2a ms1-decode
implementation (worktree seedhammer-wt-t2a-ms1, branch feat/ms1-decode-display, 2 commits over
68e6ead: 9022042 codex32.DecodeMS1, ac69291 gui ms1DecodeFlow+gate), BEFORE merge. Reviewer agentId
ab266cee53028cb43. Verdict: GREEN 0C/0I. Independently re-ran the full suite + re-proved the 5 parity
vectors against ms-codec source (re-running the live Rust encoder for the Japanese vector = exact
match), confirmed DecodeMS1 branches on Seed()[0] with the length guard preceding any bip39.New
panic path, the showSecret gate computed once correctly refuses the BIP-93 ms10test secret (Button2
inert, no hang → TestConfirmCodex32UnsharedNoRecover passes), ms1DecodeFlow display-only (no
engrave/NFC, scrubs, surfaces the mnem language byte), gui.go untouched, paging gap-free. The sole
go vet line (gui/op/draw_test.go ArtifactDir go1.25 tag) is pre-existing/out-of-scope. Disposition:
GREEN — merged to fork main 4d02021, pushed bg002h. The text below is the agent's report verbatim.
-->

# WHOLE-DIFF EXECUTION REVIEW — T2a ms1 decode

**Scope.** Worktree `/scratch/code/shibboleth/seedhammer-wt-t2a-ms1`, branch `feat/ms1-decode-display`, 2 commits over base `68e6ead` (`9022042` codex32.DecodeMS1, `ac69291` gui ms1DecodeFlow + gate). Diff confined to exactly 5 files (`codex32/mspayload.go`+test, `gui/ms1_decode.go`+test, `gui/codex32_polish.go`); `gui/gui.go` is **untouched** (`git diff --stat 68e6ead..HEAD -- gui/gui.go` empty). Working tree clean. Re-verified independently with Go at `/home/bcg/.local/go/bin/go` (go1.26.4) and the Rust ms-codec encoder.

## Verification Results (real output)

**Suite (uncached `-count=1`):**
- `go test -timeout 90s ./codex32/ ./gui/ ./bip39/` → `ok codex32 0.001s`, `ok gui 7.728s`, `ok bip39 0.019s`.
- `go test ./...` → all packages `ok` (codex32, gui, engrave, slip39, nfc/*, picobin, stepper, …); no failures.
- `go vet ./codex32/` → rc=0. `go vet ./codex32/... ./gui/...` reports one line: `gui/op/draw_test.go:176:24: testing.ArtifactDir requires go1.26 or later (file is go1.25)` — **pre-existing** (present verbatim in `git show 68e6ead:gui/op/draw_test.go`), a go-build version-tag mismatch in a test file this diff does not touch. Not introduced here; does not block build or tests.
- `gofmt -l codex32/ gui/` → empty; `gofmt -l` on the 5 changed files → empty (all formatted).
- `go build ./...` → rc=0.
- `go test -run TestAllocs ./gui/` → `--- PASS: TestAllocs (1.83s)`.

**Named tests present + PASSING (verbose, not skipped):**
- `TestDecodeMS1Parity` with all 5 subcases: `entr16-zero`, `entr20-nonzero`, `entr32-zero`, `mnem-english16`, `mnem-japanese16` — all PASS.
- `TestDecodeMS1Refusal` — PASS.
- `TestMS1DecodeFlowEnglishWords`, `TestMS1DecodeFlowNonEnglish`, `TestMS1DecodeFlowPaging24Words`, `TestConfirmShowSecretGate` — all PASS.
- `TestConfirmCodex32UnsharedNoRecover` (the R0-caught hang) — **PASS, no hang** (returns `codex32Engrave`).

**Parity vectors authoritative (load-bearing) — re-proved against source, not the draft:**
- Prefix bytes: `RESERVED_PREFIX=0x00`, `MNEM_PREFIX=0x02` (`ms-codec/src/consts.rs:17,39`) = `msPrefixEntr`/`msPrefixMnem` (`codex32/mspayload.go:9-10`). Lengths `[16,20,24,28,32]` (`consts.rs:29`) = the guard (`mspayload.go:54-55`). Language table order matches (`consts.rs:47-58`; Go display-cased `MSLanguageNames`, index 1 = Japanese).
- 3 entr vectors (`mspayload_test.go:25-27`) match `tests/vectors/v0.1.json` byte-for-byte (incl. nonzero `0123…4567`/`…cjx3kkj`, which catches bit-ordering).
- mnem-English (`:28`) = `tests/mnem.rs:155` golden byte-for-byte (`…4cdrq2y4h82yz`, entropy `0c1e24…`, lang 0).
- mnem-Japanese (`:29`): **I ran the Rust encoder myself** (`encode(Tag::ENTR, Payload::Mnem{language:1, entropy:0c1e24…})`) → emitted `ms10entrsqgqsc83yukgh23xkvmp59xf2eldpkpefrcjje3drdq`, byte-for-byte equal to the embedded vector. Rust-encoder-sourced, not fork-round-tripped.

**Independent Go probe (committed `DecodeMS1`):** japanese → `prefix=0x2 lang=1 langname="Japanese" entropy=0c1e24… (16B)`; entr20 → `prefix=0x0 lang=0 (20B)`; entr32 → `prefix=0x0 lang=0 (32B)`. Non-English path surfaces "Japanese"+hex (never words); entr paths render words. 24-word zero-entropy mnemonic's last word is "art"→"ART", reachable only via paging.

## Findings

**Decoder correctness (`codex32/mspayload.go`).** Branches on `data[0]=Seed()[0]` (`:39`), never the id/Tag. `len<2`→`errMSBadPrefix` (`:36`); mnem `len<3`→length error (`:43`); language `>9`→`errMSBadLanguage` (`:47`); the `{16,20,24,28,32}` guard (`:54`) runs on **all** paths before return. No panic path: it precedes any `bip39.New`, which panics on `len<16||>32||%4!=0` (`bip39/bip39.go:229-234`) — fully covered by the guarded set. Refusals return errors, not panics.

**The gate / no-regression (`gui/codex32_polish.go`).** `showSecret = f.Unshared && msErr==nil` is computed **once before the loop** (`:103-104`). Button2 is always drained (`:112`, queue-head idiom); acted on only as `showSecret`→`ms1DecodeFlow` (`:114-116`) or `!f.Unshared`→`codex32Recover` (`:117-118`). I proved the BIP-93 secret `ms10tests…` has `Seed()[0]=0x31` → `DecodeMS1` returns `errMSBadPrefix` → `showSecret=false` → Button2 inert → no decode screen, no hang (this *is* the regression guard). Shares still `return codex32Recover`. Back/engrave/`codex32ConfirmAction`/`engraveCodex32`/`recoverCodex32Flow` are structurally unchanged.

**Safety §2.** `ms1DecodeFlow` (`gui/ms1_decode.go`) contains **no** engrave/NFC/NDEF/Split/backup call — only `ctx.Frame` (render) and two `return`s (error showError `:27`, Back `:59`). Loop is `for !ctx.Done`. `wipeBytes(entropy)` deferred (`:29`) scrubs. The mnem language byte is surfaced: non-English shows "Language: <name>" + entropy hex + "words not shown" (`:37-44`), **never** English words; English shows the numbered words (`:32-36`). I could not construct an engrave/NFC/mutation from the decode screen, nor a hang: the first line at `start` is emitted unconditionally so `shown≥1` whenever `start<len(lines)`, and `start+shown<len` else-wraps to 0 — `start` stays `<len(lines)`, no stuck page. md1/mk1 (`mdmkText`) and the engrave path are untouched.

No CRITICAL, IMPORTANT, or MINOR findings. The sole vet line is pre-existing and out of scope.

**GREEN — 0 Critical / 0 Important.**
