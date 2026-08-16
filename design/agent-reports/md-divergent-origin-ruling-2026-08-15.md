# md divergent-origin ruling — 2026-08-15

**This is an agent's advisory ruling, not a human decision.** Dispatched between
S5.0 (closed green) and S5 proper to rule on whether md 0.13.0's apparent
inability to encode divergent-origin policies blocks S5. Re-scoped mid-task by
the coordinator after both of us independently measured that md's capability
exists under a different input syntax; the surviving question is whether the
DEVICE's S5 md1 actually requires divergent origins, and what form S5's oracle
derivation must therefore use.

## VERDICT

**S5 IS NOT BLOCKED BY md.** The pinned `md` 0.13.0 encodes divergent per-key
origins today, via its own inline placeholder-origin syntax
(`@i/48'/0'/x'/2'/<0;1>/*`) — measured, decoded back, and confirmed
`"tag": "Divergent"` on the wire with both origins, fingerprints and pubkeys
intact. No change to the primary, no new tag, no pin bump is needed for S5.
The consequence that remains is inside the seedhammer S5 work itself: **S5's
md1 DOES require divergent origins** (question 1 = YES), so the S5-proper
oracle work must replace `uniformOrigins`' divergence refusal — whose stated
premise ("the pinned md offers no invocation that encodes a per-slot origin")
is now falsified by measurement — with template construction in the inline
per-`@N` form.

---

## Q1 — Does S5's md1 require divergent origins? **YES.**

**RULING.** The policy md1 the device assembles at S5 for Trace B is
divergent-origin by design, not merely the cosigner cards' own mk1 origins.
The alternative reading in the dispatch brief (shared-origin policy md1 +
per-card mk1 origins) is the **S4 state**, which S5 explicitly rewires.

**WHY — design citations.**

- Plan (`design/IMPLEMENTATION_PLAN_multisig_build_repair.md`):
  - `:174` — S5 delivers "multi-slot self, divergent origins, and the engrave
    tail".
  - `:1143-1144` — test 1, `TestMultiSlotSelfAssembles`: "Trace B's shape
    assembles with `OriginDivergent` and the correct per-slot origins". The
    origin mode of the ASSEMBLED policy, asserted by name.
  - `:1145-1146` — test 2, `TestCosignerCardOriginIsHonoured` (R-3): "the
    card's declared origin reaches the descriptor, not the flow's shared
    origin". "The descriptor" — the md1, not the mk1.
  - `:1174-1178` — implementation: "`cosignerFromCard` stops discarding
    `card.Origin`; `OriginDivergent` when origins differ, `OriginShared` when
    they do not" and "Remove S2's interim foreign-origin refusal".
- Spec (`design/SPEC_multisig_build_repair.md`):
  - `:695-697` (P3) — "`OriginDivergent` is used when origins differ".
  - `:739-744` (R-3) — "for a divergent policy, the CARD's origin wins over
    the flow's shared origin".
  - `:719-723` (P5) — the hardware gate: "At least one build MUST be
    divergent-origin, multi-slot and multi-master — a shared-origin
    single-seed P5 would pass green around every §4.1a failure."
- Fixture (`seedhammer/cmd/emu/sysw_cards_payload.go:24-27,41-42`): Trace B is
  A@0 at `m/48'/0'/0'/2'` and A@1 at `m/48'/0'/1'/2'` plus B@0 and C@0 at
  `m/48'/0'/0'/2'`. A@1's origin differs from the other three, so per the
  plan's rule the assembled set is `OriginDivergent` **by construction of the
  committed payload**.
- §0.1a (plan `:104-107`) adds a second divergence source from S5: `sh(wsh)`
  slots default to script-type `1'`, template-aware — but Trace B's `wsh` case
  alone already settles the question via A@1.

**Where the brief's counter-evidence actually points.** The "LOCKED shared
origin" and the foreign-origin refusal are the CURRENT code at `main`
`80d0c5d` — `gui/multisig_build.go:866-867` ("The card's Origin is IGNORED
(OriginShared mode…)"), `:973-985` (the M-E refusal loop), `:987-993`
(`OriginMode: md.OriginShared` in the encode request). All three are S2/S4
interim state that the plan's S5 bullets above name and remove. The fork's own
encoder has carried the divergent arm unused the whole time
(`md/encode_multisig.go:39-40,101-105`; spec §8: "`md.EncodeMultisig`'s
`OriginDivergent` arm exists and is unused by `gui`").

**CONSEQUENCES.** The S5 gate record for Trace B must carry a
divergent-origin md1 in its census, so the oracle derivation cannot use the
existing shared `--path` invocation for that trace — `--path` is documented as
flattening Divergent to Shared and would derive a DIFFERENT wallet's md1,
failing the byte comparison for the right reason but with no useful message.

## Q2 — md fix / re-tag / pin bump. **DROPPED by coordinator re-scope.**

There is no md feature gap. The pinned binary at
`~/.cargo/bin/md` (0.13.0; primary repo
`/scratch/code/shibboleth/descriptor-mnemonic` at `5a0a4f41` = HEAD =
`md-cli-v0.13.0`) encodes divergent origins via the inline form. No
Rust-primary escalation, no new tag, no pin bump, no schedule impact.

The residual defect — the descriptor-style bracketed form
`[fp/path]@0/<0;1>/*` failing with `internal: synthetic key [73c5da0a not
found in key map` — is real but is a wrong-syntax input that should be refused
cleanly, not a missing capability. The coordinator is fixing it separately;
per the re-scope I do not rule on it. (Mechanism, for the record:
`crates/md-cli/src/parse/template.rs:804` — `lookup_key` splits the rendered
key on `/` and the first `/` of an origin-bearing key lands inside the
bracket, so the key map is asked for `[73c5da0a`.)

## Q3 — What S5's oracle derivation must do. **The inline per-`@N` origin form, for both modes.**

**RULING.** S5-proper extends the S5.0b oracle to divergent origins by
building the template with each slot's origin inline
(`@i/48'/0'/ACCOUNT'/2'/<0;1>/*`) and dropping `--path`. One invocation shape
then covers both wire modes: md's `make_path_decl`
(`crates/md-cli/src/parse/template.rs:495-510`) emits `Divergent` when the
per-placeholder origins differ and collapses them to `Shared` when they are
equal — the exact rule the device's S5 assembly follows.

**WHY this is in-plan S5 work, not a re-opening of S5.0.** The refusal's own
comment (`seedhammer-s5/oracle/expect.go:648-654`) reserved this: "Divergent
support means finding md's own form for it and proving that form against a
real walk — which is work, not a flag, and it does not belong in a gate
written before the stage it judges." The form is now found and proven. What
must be corrected while doing it is the comment's factual premise — "there is
no invocation this gate could make" (`:648`) and the refusal text's "The
pinned md offers no invocation that encodes a per-slot origin" (`:676-680`)
are **falsified by measurement below**. This is the review-catches-reasoning /
execution-catches-facts class: the claim was measured against only the
bracketed syntax and generalized.

**Scope of the oracle change is small.** `deriveBuiltPolicy` is already
per-slot: `templateForOrigin` (`oracle/expect.go:757-790`) parses arbitrary
account indices, and the per-slot `msDerive` + origin-claim cross-check
(`:503-524`) already handles A@1's account 1. Only `uniformOrigins`'
`samePath` refusal (`:674-681`) and `mdEncode`'s single `sharedPath` /
`--path` argument (`:925-`, invocation note `:531-533`) bind the gate to
shared origins. Template/network uniformity checks (`:665-673`) stay — one
md1 is still one script type on one network.

**S5.0's committed records are not staled.** The uniform-origin inline form
is byte-identical to the `--path` form (measured below, `diff` exit 0), so
records minted via `--path` remain exactly what the inline form derives.

## Q4 — Internal error as failure mode. **Noted; being fixed separately.**

Per the re-scope, no ruling here. Two records worth keeping:

1. **Caution for whoever writes S5's derivation** (coordinator's near-miss,
   recorded at their request): a naive `lookup_key` fix that strips the
   bracket prefix makes the bracketed form ENCODE while **silently dropping
   the origins** — the lexer never captured them, so `path_decl` comes back
   empty (coordinator confirmed on the drafted fix). That turns a loud
   wrong-syntax error into a quiet wrong-policy — the failure mode with funds
   attached. Only a decode round-trip assertion catches it; "encode succeeds"
   passes. S5's oracle work should therefore round-trip its first divergent
   mint through `md decode --json` and assert `path_decl.tag == "Divergent"`
   with the exact expected paths, once, as the §4.5 walk's first divergent
   gate execution.
2. **Observed, untouched:** the primary's working tree at
   `/scratch/code/shibboleth/descriptor-mnemonic` is dirty with that in-flight
   work — `M crates/md-cli/src/parse/template.rs` (a drafted `lookup_key`
   origin-prefix strip) and an untracked
   `crates/md-cli/tests/cli_divergent_origin_encode.rs` (three tests,
   including the round-trip assertion that catches the silent-drop). The
   pinned commit `5a0a4f41` contains neither. Per my constraints I modified
   nothing and ran no builds in that tree.

Discoverability footnote: `md encode --help` shows only originless
placeholders in its TEMPLATE example and documents the inline origin form
nowhere on the encode surface — that gap is how "cannot encode divergent" got
measured as true twice. A doc line on `[TEMPLATE]` naming the form would have
prevented this entire dispatch.

---

## WHAT I VERIFIED (commands + real output)

All `md`/`ms` invocations by absolute path; true exit codes checked unpiped.

**Pin identity.** `/scratch/code/shibboleth/descriptor-mnemonic`:
`git log --oneline -1` → `5a0a4f41 release: md-codec 0.42.0 + md-cli 0.13.0 —
pathless/dead-card partial-decode`; `git tag --points-at HEAD` →
`md-cli-v0.13.0`, `md-codec-v0.42.0`. `~/.cargo/bin/md --version` →
`md 0.13.0`, exit 0. `~/.cargo/bin/ms --version` → `ms 0.16.0`, exit 0.

**Keys (BIP-39 `abandon…about` = master A, `legal winner…yellow` = B,
`letter advice…above` = C), via the pinned ms:**

```
$ ms derive --phrase "abandon … about" --template bip48-p2wsh --account 0 --network mainnet --json
{"master_fingerprint":"73c5da0a","account_path":"m/48'/0'/0'/2'","account_xpub":"xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf",…}  TRUE_EXIT=0
$ ms derive … --account 1 …
{"master_fingerprint":"73c5da0a","account_path":"m/48'/0'/1'/2'","account_xpub":"xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk",…}  TRUE_EXIT=0
$ (B@0) → fp b8688df1, xpub6FQya7…F8mX   TRUE_EXIT=0
$ (C@0) → fp 28645006, xpub6DnEBN…ekh6   TRUE_EXIT=0
```

**1. Divergent encode, the motivating 2-key shape (two accounts of one
master), inline origin form, pinned binary:**

```
$ ~/.cargo/bin/md encode "wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*))" \
    --key @0=xpub6DkFAXW… --key @1=xpub6Dzhyrn… \
    --fingerprint @0=73c5da0a --fingerprint @1=73c5da0a \
    --network mainnet --group-size 0 --force-chunked --policy-id-fingerprint
chunk-set-id: 0x7a2e1
md1f0ghpps9q2tvyyy5jmpprj5qqcy8ppgtcgu79mg9tnchdq59wpyhwsv0jskp2rsal4egz4eqdccu772e060rs
md1f0ghppsf5859p875x67p5s3wem7sgluxl3d2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfjcjukwzsuw7pydp
md1f0ghppsje20ur0anz7jwkzae8efejcxy50llpx82qfmryv7l68w6hzragnj3g5qrl85zeapccg28cpyh2qcaz
md1f0ghppse8wq0vdczfyy55tqsd5576trsa3p40nfpd7hsyjyf7vlx6hk2j6ckr4wf0m3sq5klzdk64u37vh
policy-id-fingerprint: 0x6a801edb
TRUE_EXIT=0
```

**2. Round trip — origins survive, distinct, tagged Divergent:**

```
$ ~/.cargo/bin/md decode --json <the four chunks above>   TRUE_EXIT=0
"path_decl": {"data": ["m/48'/0'/0'/2'", "m/48'/0'/1'/2'"], "tag": "Divergent"},
"fingerprints": [[0,"73c5da0a"],[1,"73c5da0a"]],
"pubkeys": [[0,"bba0c7ca…"],[1,"9960c4a3…"]]   (65-byte chaincode‖pubkey each)
```

**3. The full 4-slot Trace B shape (A@0, A@1, B@0, C@0 as 3-of-4,
fingerprints omitted — the device's Omit default):**

```
$ ~/.cargo/bin/md encode "wsh(sortedmulti(3,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/0'/2'/<0;1>/*,@3/48'/0'/0'/2'/<0;1>/*))" \
    --key @0=… @1=… @2=… @3=… --network mainnet --group-size 0 --force-chunked --policy-id-fingerprint
chunk-set-id: 0xc1678   (8 chunks)   TRUE_EXIT=0
```

**4. Uniform inline origins ≡ `--path` shared, byte for byte** (same 2 keys
A@0 + B@0, both at `m/48'/0'/0'/2'`; outputs to files, `diff`):

```
$ md encode "wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/0'/2'/<0;1>/*))" …   INLINE_EXIT=0
$ md encode "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))" --path "m/48'/0'/0'/2'" …    PATH_EXIT=0
$ diff inline-uniform.txt path-shared.txt                                             DIFF_EXIT=0
```

**5. The false-premise site**, `seedhammer-s5` worktree (HEAD `5edb162`
"S5.0b: the built-policy ExpectKind…"): `oracle/expect.go:636-687`
(`uniformOrigins` refusal + its "no invocation" comment), `:925` (`mdEncode`
signature taking one `path`), `:527-533` (the `--path` invocation the notes
record). Read, not modified.

**6. Syntax provenance in the primary** (read at the pin):
`crates/md-cli/src/parse/template.rs:55` (lexer regex, capture 2 = inline
origin), `:197-200` (`origin_path_extracted` unit test), `:495-510`
(`make_path_decl`: differ → `Divergent`, equal → `Shared`), `:675` (the
substitution regex swallows the inline origin so it never reaches miniscript);
`crates/md-cli/src/cmd/encode.rs:179-184` (comment naming the inline per-`@N`
origin form as full-decodable with no `--path`).

**Not verified / out of scope:** whether the fork's Go `EncodeMultisig`
divergent arm is byte-identical to the primary on Trace B's inputs — that is
precisely what S5's §4.5 byte-comparison gate exists to prove, and proving it
here would front-run the gate. Closed stages S0–S4 and S5.0 were not
re-audited; the descriptor-mnemonic tree was left exactly as found.
