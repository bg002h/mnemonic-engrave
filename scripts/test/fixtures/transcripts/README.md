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
  the two `check` refusals after `inject-copy` (R5, R7);
- `BENCH_RUN`, an empty marker. Once it exists, an empty or partial directory
  fails case 12.
