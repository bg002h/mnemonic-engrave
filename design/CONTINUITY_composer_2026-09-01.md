# CONTINUITY — Wallet Policy COMPOSER cycle (arbitrary tr/wsh authoring on the SH2), 2026-09-01

**Resume here.** Usage ran low with R0 round 0 in flight; the operator ruled
"finish what is in flight but don't launch anything new" (2026-09-01).

## State

| artifact | where | commit |
| --- | --- | --- |
| Brainstorm record, 29 operator rulings C1..C29, measurements §3.1-3.11 | `design/BRAINSTORM_wallet_policy_composer.md` | `9100230`, folded `b452a79` |
| SPEC, DRAFT, R0 round 0 dispatched, no PENDING sections | `design/SPEC_wallet_policy_composer.md` | `f68134b`, fold `b452a79` |
| Lowering reviews (fable ×2, operator-directed) | `agent-reports/composer-lowering-rules-bitcoin-expert-review.md`, `composer-lowering-i1-single-key-head-miniscript-review.md` | `e9f2ba0`, `e0375f1` |
| Recon (opus ×2, operator-approved fan-out) | `agent-reports/composer-recon-taproot-multisig-origin-convention.md`; `composer-recon-same-fingerprint-two-accounts-import.md` + `-core-` + `-sparrow-` | `9f55eb6`, `f7f0b27` |
| Follow-ups filed | here: F-448, F-449; descriptor-mnemonic: `md-older-zero-time-units-not-refused` (790fc224), `md-descriptor-depth0-xpub-ledger-registration` (3b0944fb); mnemonic-secret: `ms-derive-taproot-justifications-stale` (5f37b43) | committed in each repo |

Heads at spec time: fork `169073c`, engrave `b452a79`, descriptor-mnemonic `3b0944fb` (md 0.14.0), toolkit `d8f06483`, mnemonic-secret `5f37b43`.

## R0 ROUND 0 FOLDED (`bc1c07c`); ROUND 1 RUN AND FOLDED (see `git log`: reports aa022ae/12abf0f/1630465, fold commit after them; gates structure 0, glyph 87/0, cites 61/61). Lenses run so far: correctness, adversarial, journey x2, coverage, feasibility, fold-verification x1. Round-1 fold verified (44bb06b) and its gaps folded at 99463ac (gates structure 0, glyph 93/0, cites 64/64). Lenses run: correctness, adversarial (on the ORIGINAL §4-§14 only), journey x2, coverage, feasibility, fold-verification x2. Adversarial-on-folds (073bd7f) found 1C/7I, folded at the round-3 fold (see `git log`; gates structure 0, glyph 103/0, cites 68/68). NEXT: sonnet verification of the round-3 fold; if clean, closure (lenses exhausted: correctness, adversarial x2, journey x2, coverage, feasibility, fold-verification x3) and the writing-plans skill: a STAGED plan with Stage 0 (md-codec `compose` + vectors + `md compose`) in full detail.

Each agent writes its own report (the controller never transcribes):

| lens | report path |
| --- | --- |
| correctness + internal consistency: **1C/11I/10M/2N** | `design/agent-reports/composer-spec-R0-r0-correctness.md` |
| adversarial funds safety (counterexamples): **5C/5I/2M/1N** | `design/agent-reports/composer-spec-R0-r0-adversarial.md` |
| operator journey walk (J1 two-path tr, J2 RCW, J3 no-payload template): **6C/10I/8M/2N** | `design/agent-reports/composer-spec-R0-r0-journey.md` |
| coverage + traceability (rulings→spec, rules→acceptance, work items→sections): **2C/8I/11M/6N** | `design/agent-reports/composer-spec-R0-r0-coverage.md` |

Persist commits: journey `4201b56`, coverage `85ec239`, adversarial `1154edc`,
correctness (see `git log`). Controller spot-checks held on every top finding
(consent copy for non-renderable shapes; single-stub re-mint vs seating; sh(wsh)
at script_type 1'; the 1985-11-05 midnight boundary encodes as a HEIGHT; wsh
summarised as one branch; `sysw.Classify` vs `seal.Classify`; single-leaf `{P1}`).
Recurring themes across lenses: the consent surface cannot state a composed shape;
the stub re-mint must carry BOTH stubs (C9); the origin table row for sh(wsh);
no acceptance item for refusals or lock ranges; date floor missing.

**Fold done 2026-09-01.** Controller defaults taken in the fold are listed in the
brainstorm record section 3.12 for the operator's veto; Minors/Nits from the four
reports are recorded there and NOT folded. **On resume, when launches are allowed:**
(1) re-review the fold — sonnet: did the fold fix each C/I and introduce nothing
(compare `git diff 80e6a72..bc1c07c -- design/SPEC_wallet_policy_composer.md`
against the four reports); opus: one NEW lens not yet run (e.g. implementation
feasibility of the Go builder + `md compose` against md-codec admission, or a
second journey walk on the regenerated §7); (2) fold, gate, commit; (3) then fold
the Minors/Nits; (4) the writing-plans skill. Original resume text follows.

(1) read all four persisted reports
(never the JSONL task output); dedupe findings across lenses; (2) fold all four into the spec together; run the gates and put
their output in the fold commit message:

    scripts/spec-structure-check.sh design/SPEC_wallet_policy_composer.md
    scripts/plan-glyph-check.sh     design/SPEC_wallet_policy_composer.md
    CITE_FORK_ROOT=/scratch/code/shibboleth/seedhammer scripts/plan-cite-check.sh design/SPEC_wallet_policy_composer.md

(`scripts/spec-check.py` is the systemwide-payload spec's own gate; its
remaining failures do not apply.) (3) Re-review only what the fold changed
(non-trivial folds), sonnet for fold-verification, opus for a new lens; a clean
round closes the LENS, not the spec — enumerate remaining lenses before closing.
(4) Then `superpowers:writing-plans` for the plan; implementation UC OFF, one
implementer; Rust first (md-codec `compose` + vectors) before the Go builder.

## Rulings that shape everything (verbatim in the brainstorm §2)

C1 spend-path list; C2 archetypes as presets; C3 firmware vocabulary, Rust-first;
C4 template first; C5 one slot one path, reuse the master via hardened accounts;
C6 Build inside Wallet Policy (door becomes a choice in every state); C7 Multisig
Build deprecated by COMMENT only; C8 payload-first seating (pick list, "remaining"
keys); C9 teach the mk1 stub unconditionally; C10 engraved FORM is the operator's
choice (concrete text/QR/keyed md1, or template + mk1); C11 timelock UI respects
ranges; C12 seeds (words/ms1, unsealed payload or typed) are a key source; C13
Full/Watch-only, secret as words/SeedQR/ms1; C14 no Sealed-Payload memory
treatment; C15 lowering lives in md-codec `compose` / `md compose`; C16 grammar
bounds; C17 pkh in wsh, pk in tr (F-448); C18 raw-H NUMS this cycle (F-449); C19-
C23 review findings (first-appearance numbering, or_i(pkh) head, or_d for bare
multi head, keyless wsh-only, lock-only refused, ≥1 keyed path, first-listed
single key as internal key, lock ranges sourced); C24 pack time via `now:` record
= lower bound; C25 entry UX (digit pad; kind→unit→digits→echo; hash: record);
C26 Build with no payload = keyless template; C27 recon fan-out (3 deferred);
C28 taproot seed-derived origin `m/48'/coin'/account'/3'`; C29 same seed twice in
ONE path = warning.

## Lessons this session recorded (also in memory)

- Line counts are not a fold check: five rulings silently failed to land because
  an anchor differed by one character; grep the inserted row afterwards.
- A regtest node rejects mainnet xpubs: a "refusal" can be the network, not the
  descriptor. Match the agent's environment before calling its claim wrong.
- The compiler is a validity oracle, not a byte oracle; byte identity with the
  toolkit archetypes is impossible under any uniform rule.
