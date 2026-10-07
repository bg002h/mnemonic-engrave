# Real picotool 2.3.1 transcripts (plan E3a §6.2 case 12)

This directory holds `otp get` transcripts captured from real silicon at the
bench rehearsal (plan §7 R1), so `scripts/test/run-e2e-otp.sh` can replay them
through the parsers in `scripts/lib/otp-read.sh` (`og_replay_dir`).

Until the bench-result PR it holds only this file, and case 12 passes vacuously.
That PR adds:

- `manifest.tsv` and the `0x0NN_{named,r,c1}.txt` files, copied from the
  `<capture>.transcripts/` directory that `refugium-otp.sh capture` writes at R1;
- `r1-capture.json`, the R1 capture itself;
- `r5-check-refusal.log` and `r7-check-refusal.log`, the `--log` transcripts of
  the two `check` refusals after `inject-copy` (R5, R7). Case 12 requires
  `FAIL BOOT_FLAGS0 copies differ` and `FAIL BOOT_FLAGS1 copies differ` in them,
  so a refusal for a parse failure (`unreadable: …`) does not count;
- `r5-capture.json` and `r7-capture.json` with their `.transcripts/`
  directories: the captures taken right after those refusals. R1's copies all
  agree, so only these hold silicon's RAW_VALUE and WARNING lines. Case 12
  replays both and requires RAW_VALUE in `0x048_named.txt` (R5) and
  `0x04b_named.txt` (R7);
- `BENCH_RUN`, an empty marker. Once it exists, an empty or partial directory
  fails case 12.

The committed set comes from the 2026-10-06 bench (Pico 2 `66D3D60FF20ABF2F`), whose R1 capture
was taken on an already-sealed board: see `design/HARDWARE_RESULT_2026-10-06_e3a.md`.
