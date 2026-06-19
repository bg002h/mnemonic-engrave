# #10b (Phase B) — Whole-Diff Adversarial Execution Review

**Reviewer:** opus architect (mandatory, non-deferrable post-implementation exec review)
**Date:** 2026-06-19
**Scope:** cumulative diff `3a55ae5..feat/10b-md-walletpolicy` (6 commits) in worktree
`/scratch/code/shibboleth/seedhammer-wt-10b`.
**Authoritative sources cross-checked:** Rust `md-codec` @ `c85cd49`
(`canonicalize.rs`, `to_miniscript.rs`, `validate.rs`); Go fork @ `3a55ae5` + the
worktree diff. Go `go1.26.4`.

---

## VERDICT: GREEN — 0 Critical / 0 Important

The feature is safe to merge. Every high-stakes claim was independently
re-derived and confirmed. Findings below are 3 Minor (doc/comment/scope) items
that do not affect correctness or fund safety and may be folded opportunistically
or logged to FOLLOWUPS.

---

## Required explicit statements

**(a) C2 nesting is correct & the two shapes derive different addresses — CONFIRMED.**
`Template.InnerWsh` is populated by `innerWshNesting(d.tree)` (md.go:1314-1323):
true iff root is `tagSh` with a single child whose tag is `tagWsh`. I verified
this is consistent with `classifyPolicy`'s sh-walk (md.go:1277-1292) for *every*
renderable sh-sortedmulti shape — the two functions can never disagree on a
renderable shape (proof by the deep-nesting probe below). `scriptForTemplate`
(md1_expand.go:69-78) maps `ScriptSh+PolicySortedMulti+InnerWsh → P2SH_P2WSH`
and `+!InnerWsh → P2SH`. This is faithful to the Rust authoritative mapping:
`sh(wsh(sortedmulti))` → `new_sh_wsh_sortedmulti` (to_miniscript.rs:220-235),
`sh(sortedmulti)` → `new_sh_sortedmulti` (to_miniscript.rs:246-254). I re-derived
both addresses through the shipped `address.Receive`: P2SH =
`39fiayD2eNRLSVCwvzuyMcxNFABRHfNSU3`, P2SH_P2WSH =
`35tek545ZwPexwHBCGtgGkgoKSZnyRw7kd` — GENUINELY DIFFERENT (different
script-hashes, not merely different enum values). `address.addressAt` confirms
the divergence: P2SH hashes the raw multisig script via `NewAddressScriptHash`,
while P2SH_P2WSH hashes `sha256(script)` to a witness-script-hash then wraps it in
`PayToAddrScript`+`NewAddressScriptHash` (address.go:122-170). Adversarial
deeper-nestings (`sh(wsh(wsh(sm)))`, `sh(wsh(sm,sm))`, `sh(sh(sm))`) all
correctly summarize to `Renderable=false` (PolicyComplex) and are refused by
`scriptForTemplate`'s `!tpl.Renderable` gate BEFORE the InnerWsh branch is read —
so a mis-walked discriminant on a non-renderable shape is unreachable. No
construction found that mis-sets the discriminant for a renderable, verifiable
shape.

**(b) The embedded goldens are authentic & the round-trip is non-circular — CONFIRMED.**
I decoded `wshSortedmultiChunks` through the SHIPPED `md.ExpandWalletPolicyChunks`:
it yields `wsh(sortedmulti(2,3))` (Root=ScriptWsh, Policy=PolicySortedMulti, K=2,
M=3, InnerWsh=false, 3 real xpubs, origin `m/48h/0h/0h/2h`, use-site `<0;1>/*`).
I then INDEPENDENTLY re-derived receive address 0 — straight from the 65-byte
xpub fields, manual `Derive(0).Derive(0)` per key, sort compressed pubkeys, build
a 2-of-3 `MultiSigScript`, P2WSH-wrap via `NewAddressWitnessScriptHash` — and got
`bc1qtahtpjkgtljxl20jgevs2tjhgzvd87jepcrsd92kcyvtzkj34mnsq0j928`, **byte-identical**
to the shipped `address.Receive(desc,0)`. The round-trip test is therefore NOT
circular: the asserted address is the genuinely correct one for those xpubs.
`tamperedCSIDChunks` trips `md.ErrChunkSetIDMismatch` specifically (NOT
`ErrChunkSetIncomplete`, not a generic error): real csid `0x2d950` on all 6
chunks vs tampered `0xce33a` on all 6 — internally consistent but wrong, so it
passes per-chunk BCH + version/csid/count consistency and fails only at the
re-derived-csid integrity gate. The golden's comment claims "real+1" but the
stamped value is not literally real+1; this is cosmetic (Minor M2) and does not
affect the test, which only requires a consistent-wrong csid.

**(c) No `expandOK` is reachable for a non-bip380 shape — CONFIRMED.**
Exhaustive enumeration over all `ScriptKind(5) × PolicyKind(6) × Renderable(2) ×
InnerWsh(2) × UseSite(5 variants incl. hardened/exotic) × XpubPresent(2)`
combinations: `expandOK` is returned for EXACTLY the 20 bip380-expressible
combinations, every one with a non-nil descriptor, a non-`UnknownScript` script,
and a successful `address.Receive`. Every non-OK status returns a nil descriptor.
`classifyPolicy` structurally can never emit `PolicyMultiA`/`PolicySortedMultiA`
(those are tapscript-only and live inside a refused taptree → PolicyComplex), and
unsorted `PolicyMulti` is absent from `scriptForTemplate`'s SortedMulti arm so it
falls through to refuse. `FuzzExpandedToDescriptor` ran ~46M execs, 0 crashes,
and its in-loop assertion (`expandOK ⇒ bip380-expressible ∧ non-nil ∧
known-script`) held throughout. D2 faithful-or-refuse is real.

**(d) The canonicalOrigin divergence is display-only / sound — CONFIRMED.**
Rust `expand_per_at_n` (canonicalize.rs:420-474) consults `canonical_origin` ONLY
to decide whether to RAISE `MissingExplicitOrigin` (line 452), never to substitute
the per-@N path (the INVARIANT at lines 404-419 is explicit). The Go
`resolveOriginPath` step-3 fallback (expand.go) DOES substitute
`canonicalOrigin(d.tree)` — the deliberate R0-I1 divergence, needed because the Go
decoder leaves an elided shared path empty. I confirmed this is display-only:
`bip380.Key.ExtendedKey()` (bip380.go:96-107) uses `DerivationPath` ONLY to set the
xpub-serialization `depth` (= len) and `childNum` (= last element);
`address.derivePubKey` (address.go:173-214) starts derivation from
`k.ExtendedKey()` and walks `Children` (the use-site) — the depth/childNum/
parent-fingerprint metadata never enter the point-derivation math. A wrong
`OriginPath` would only mis-serialize the *displayed* xpub string, never change a
derived address. The same applies to a zero `MasterFingerprint` when the
fingerprint TLV is absent. Address verification is unaffected either way.

---

## Re-run evidence (observed)

```
$ go test -count=1 ./...                  → all packages ok (gui 7.3s, md ok)
$ go test -count=1 -run TestAllocs ./gui/ → PASS (TestAllocs 1.15s)  [D6 alloc gate intact]
$ go vet ./md/... ./gui/...               → only: gui/op/draw_test.go:176 testing.ArtifactDir
                                              requires go1.26 (file is go1.25)
$ go vet ./gui/op/... @ 3a55ae5 (baseline)→ SAME warning, identical → NOT a regression
$ gofmt -l md/ gui/                        → (empty), exit 0
$ go test -fuzz FuzzExpandedToDescriptor -fuzztime=120s ./gui/ → 45,991,847 execs, PASS, 0 crashes
$ go test -fuzz FuzzExpandWalletPolicy   -fuzztime=90s  ./md/  → 40,434,986 execs, PASS, 0 panics
```
Both fuzzers exceed the ≥1M-exec bar by ~40–46×. The `ArtifactDir` warning is
present byte-identical at baseline `3a55ae5`, confirmed not introduced by this diff.

---

## Adversarial-focus closure

1. **C2 wrong-address safety** — closed (statement (a)). Independent
   re-derivation; deeper-nesting mis-walk attempts all fail safe (non-renderable).
2. **Golden authenticity / non-circular gate** — closed (statement (b)).
3. **D2 faithful-or-refuse (I-6)** — closed (statement (c)); exhaustive +
   46M-exec fuzz.
4. **xpub-expansion fidelity** — `ChainCode=xpub[0:32]`/`KeyData=xpub[32:65]`
   verified byte-exact (xpub[0]=chaincode, xpub[31]=last chaincode, xpub[32]=first
   pubkey, xpub[64]=last pubkey — no off-by-one). Per-@N precedence
   (override > path_decl > canonicalOrigin) matches canonicalize.rs:437-460 modulo
   the documented R0-I1 fallback. `useSiteToChildren` maps `<a;b>/*` →
   `[Range{a,b}, Wildcard]` with the `End==Index+1` guard (md1_expand.go:124-150)
   mirroring address.go:196-198; bare `*` → `[Wildcard]` (matches Rust empty-path +
   wildcard, to_miniscript.rs:116-131). D5 hardened wildcard/alt rejected. I2
   in-band hardening (`bip32.Path`=[]uint32, `value+HardenedKeyStart`); no parallel
   bool — confirmed `m/48h/0h/0h/2h` renders correctly.
5. **D4 secp256k1** — `validateXpubBytes` (md.go:1073-1083) parses `xpub[32:65]`;
   `xpub` is a fixed `[65]byte` so the slice can never panic. Faithful to
   validate.rs:216-226 (chain-code prefix intentionally unchecked). I confirmed it
   runs on BOTH paths: single `Decode → decodePayloadValidated` (md.go:1225) AND
   chunked `Reassemble → decodePayloadValidated` (chunk.go:279); an off-curve
   pubkey in a chunked set is rejected with `errInvalidXpubBytes` via both
   `DecodeChunks` and `ExpandWalletPolicyChunks`.
6. **D6 alloc gate + no-regression** — `TestAllocs` green; `*bip380.Descriptor`
   built once in `gatheredDescriptorFlow` before `descriptorFlow(ctx,th,desc)`
   (md1_gather.go), never per-frame. Single-md1 (`err==nil → md1DisplayFlow`),
   mk1, ms1 paths byte-unchanged; only the `ErrChunkedUnsupported` arm of
   `mdmkFlow` changed (gui.go:1975-1979, +3 comment lines, 1 body swap). No new
   imports in gui.go, no new top-level `program` type.
7. **Gather clone correctness** — `md1Gatherer.offer` is a faithful clone of
   `mk1Gatherer.offer` plus the deliberate `!h.Chunked → gatherIgnored` prime guard
   (a single md1 cannot prime a set) and `!h.Chunked → gatherForeign` once primed.
   `ParseChunkHeader`/`readChunkHeader` enforces `index < count` (chunk.go:63), so
   an out-of-range index can't inflate the set; and `Reassemble` re-validates
   gaps/count/csid regardless. csid-mismatch dispatched via
   `errors.Is(err, md.ErrChunkSetIDMismatch)` to a distinct message
   (md1_gather.go).
8. **Security/faithfulness** — public md1/descriptor only; no secret material
   handled. No spend-path mis-render found.

---

## Findings (all Minor — non-blocking)

**M1 (Minor, doc) — stale invariant comment on `canonicalOrigin`.**
`md/md.go:1093-1094` states `canonicalOrigin` is "USED ONLY by
validateExplicitOriginRequired — never to substitute a renderable key's decoded
OriginPath." This is now contradicted by `resolveOriginPath` step 3 (expand.go),
which intentionally substitutes it (R0-I1). Behavior is correct and display-only;
only the comment is misleading.
*Fix:* update the comment to note the deliberate R0-I1 display-only reuse in
`ExpandWalletPolicy`, e.g. "USED BY validateExplicitOriginRequired (error gate)
AND, per R0-I1, by ExpandWalletPolicy as a display-only origin fallback (does not
affect derivation)."

**M2 (Minor, test-comment) — `tamperedCSIDChunks` "real+1" comment inaccurate.**
`gui/md1_gather_test.go` comments describe the tampered set as csid "real+1". The
embedded set's csid (`0xce33a`) is not literally `realCsid(0x2d950)+1`. The set is
still a valid consistent-but-wrong csid (the test only needs that), so the test is
sound; the comment overstates precision.
*Fix:* reword to "a consistent-but-WRONG chunk-set-id" (drop "real+1"), or
regenerate the golden with the exact real+1 value.

**M3 (Minor, scope) — `sh(wpkh)` (P2SH-P2WPKH) silently display-only, not in the
expressible set.** The plan's D2 (line 16) listed `sh-wpkh` among the expressible
subset, and Rust supports it (`new_sh_wpkh`, to_miniscript.rs:240-243). The Go
`classifyPolicy` never renders `sh(wpkh)` (it → PolicyComplex → display-only), and
`scriptForTemplate` deliberately omits the arm (the documented R0-Minor). This is
SAFE — `sh(wpkh)` is never address-verified, only template-displayed — but it is a
feature-scope reduction vs the plan/Rust, not a verified behavior.
*Fix:* none required for safety; log to FOLLOWUPS if P2SH-P2WPKH singlesig verify
is later wanted (would need a `classifyPolicy` sh-wpkh case + the `scriptForTemplate`
arm + a golden).

---

## Conclusion

Re-ran the full suite, `TestAllocs`, `vet`, `gofmt`, and both fuzzers (40M / 46M
execs) — all green, 0 panics, no regression vs baseline. Independently
re-derived the high-stakes address paths (C2 P2SH vs P2SH-P2WSH; the
wsh(sortedmulti) golden round-trip) and confirmed they are correct and
non-circular. The faithful-or-refuse boundary (D2) is exhaustively + fuzz-proven
to never emit `expandOK` for a non-bip380 shape. The canonicalOrigin divergence
is sound (display-only). The 3 Minor findings are documentation/scope only.

**VERDICT: GREEN (0 Critical / 0 Important).** Cleared to merge no-ff (signed +
DCO) per the project gate. Reviewer made no changes to tracked source (throwaway
verification tests were created and removed; working tree confirmed clean).
