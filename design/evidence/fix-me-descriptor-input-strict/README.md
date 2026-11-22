# Fixtures for FIX_me_descriptor_input_strict (bh-me-I2, bh-me-I15)

Signet only. The three cosigners are the BIP-39 reference vectors with entropy 00 x16, 7f x16 and 80 x16 (empty
passphrase), each derived at m/48'/1'/0'/2' with `mnemonic convert --from phrase=- --to xpub --network signet
--path "m/48'/1'/0'/2'"` (master fingerprints 73c5da0a, b8688df1, 28645006). `base.txt` is their
`wsh(sortedmulti(2,…/<0;1>/*))`; every other fixture is that text with one change, named by the file.

- `cases.txt`: one gate row per line, `fixture|--as mode (none = omitted)|exit|want key (- = none)`.
- `want/<key>.err`: the exact stderr line the row must carry. `record-refusal.err` and
  `i15-origin-2147483647.txt.md1.err` are today's texts, unchanged by the FIX; the T-DUP lines are PROPOSED.

Run: `bash scripts/fix-me-descriptor-input-strict-gate.sh --cli`.
