# R0 v2 round 1 (opus) — `SPEC_encrypted_payload_delivery.md` @ 86c0445

Scope: the v2 changes — mixed public/encrypted payloads, the `ms1`-first session,
the fixed public hash, the two-phase timer. Structure, consistency, vector
arithmetic and test quality. A concurrent fable review covers the cryptographic
core only.

Verdict: **3 Critical / 4 Important / 5 Minor / 2 Nit — GATE BLOCKED.**

## The three Criticals, in one line each

1. **Multi-`ms1` bundles are unhandled.** §10.2.2 is written in the singular
   throughout; a 2-of-3 `wsh-sortedmulti` carries **three** `ms1` records
   (measured: 3 ms1 + 6 mk1 + 6 md1 = 15). Either the other two are never
   offered — an incomplete seed backup the operator believes complete, which
   §6.4 itself calls "the worst available outcome" — or they stay resident with
   §10.2.4's timer already disabled, which is strictly worse than the v1
   behaviour v2 replaced. §6.4's own table cites that 15-record bundle as the
   reason the cap is 24, so the spec contradicts itself.

2. **The downgrade is REAL, and §6.6's own wording disarms the control.** The
   attacker does not tamper a mixed payload — they write a fresh E-shape blob
   carrying *their* mk1/md1. It satisfies every §6.2 rule, prompts for no
   passphrase, and the operator has nothing to compare the hash against, because
   §6.6 told them that when a tag exists the hash "answers a different question
   — which wallet is this?" rather than being an integrity tripwire. So they
   never recorded it. The fingerprint and mk1↔md1 binding are self-consistent in
   the attacker's crafted set.

3. **The D==E hash assertion is satisfied by a WRONG hash.** D and E have
   byte-identical public sections, so *any* deterministic function of *any
   subset* of those bytes satisfies the equality. Demonstrated by execution:
   hashing only the first record gives `f6b9f3cbdcc85939`; hashing `pub[:-1]`
   gives `85a8687999344559`; both pass. §11.4 never requires asserting the
   literal value, and §11.2 — the device side, where the operator actually reads
   the number — has **zero** §6.6 coverage.

## Importants

4. **§6.4/§6.5's "parsed only after the tag verifies" is now false.** The public
   section is a record container parsed at §10.2 step 2, pre-authentication, and
   in the `ct_len == 0` shape never authenticated. Stale v1 text that licenses an
   implementer to write the splitter as trusted-input code. §11.2's
   allocation-count assertion covers only the plaintext path; a public section of
   8191 LF bytes needs no passphrase and no KDF to reach.
5. **§9's `me seal` unconditionally generates a passphrase, salt and IV** —
   contradicting §6.2's all-zero rule for the public-only shape, and making the
   operator store a passphrase that protects nothing. That false belief is
   exactly what Critical 2 exploits.
6. **An aborted `ms1` engrave leaves the record resident with the timer already
   disabled.** Both controls are keyed on button presses rather than residency;
   cancelling mid-plate to re-seat shifted steel — the machine's most ordinary
   failure — lands in precisely the unguarded state both controls exist to
   prevent.
7. **§11.2 never mentions vectors D or E**, and three §11.3 mutant rows name
   §11.2 tests that do not exist there. By §11.3's own rule those are unkilled
   mutants.

## Minors / Nits

Stale `payload_kind` values `0x03`/`0x04` in §9 and §12 · the `iterations →
50000` negative is rejected by §6.2's floor before any tag work, so it proves
nothing about AAD binding · the "hash read from the payload" mutant cannot be
constructed (there is no hash field) · §10.2 step 3 shows `e3b0 c442 98fc 1c14`
— `SHA-256("")` — on every fully-encrypted payload, teaching operators the number
is furniture · §11.2 cites §10.2 "step 7" for the wipe caveat, now step 10 ·
duplicated mode-0600 bullet in §9 · `--plaintext` missing from §9's synopsis.

## Verified clean

§6.2's degenerate-header surface is tight: `pub_len == 0 && ct_len == 0` rejected,
all-zero rule covers every crypto field, no over-read past `pub_len`, no hang on
the unencrypted path. **AAD splicing, truncation and boundary-shifting all fail
closed because `pub_len` and `ct_len` are inside the AAD** — flips at public
offsets 0, 1, 200 and 395 of vector D all rejected, 4/4. §6.6's definition is
unambiguous and nowhere readable from the payload; **64 bits is genuinely
adequate** for the stated grinding model.

All five vectors reproduce byte-for-byte, including all five 52-byte headers.
Four unique `(key, iv)` pairs across A–D.

