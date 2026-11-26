# Exit-code survey brief (shared by every surveyor)

Goal: we are writing ONE exit-code spec for the bg002h "constellation" of
Bitcoin backup CLIs. Your job is the evidence: what each binary you own ACTUALLY
returns today for every kind of outcome, from source and from runs. You do not
write the spec and you do not change any code.

## Hard rules
- Read-only on every repository. No edits, no commits, no pushes, no branch
  changes, no `cargo build` / `cargo test` (binaries are already built).
- Built binaries (debug, current fix-PR heads): /home/user/wt/target/debug/
  md, mk, ms, mt, me, mnemonic, mr, mr-sitting.
- Source worktrees: /home/user/wt/<repo> (descriptor-mnemonic, mnemonic-key,
  mnemonic-secret, mnemonic-transaction, mnemonic-engrave, mnemonic-toolkit,
  mnemonic-refugium, mnemonic-gui, refugium-wallet).
- Scratch files: only under /home/user/wt/ec/tmp-<yourname>/ (create it). Do
  not put large files in /tmp.
- Use only published test vectors / help-text examples / throwaway values as
  inputs. Never generate or use real secrets. No network, no bitcoind.
- Run each binary with stdin from /dev/null unless the case needs stdin, and
  record `$?` exactly. Quote the exact command you ran.

## Context (verify, do not trust)
- mnemonic-engrave/design/SPEC_constellation_cli_uniformity.md §6f ("Exit
  codes", around line 1017) has a partial table and the D26 repair rule:
  exit 5 REPAIR_APPLIED = correction verified now or verifiable by reassembly;
  exit 4 VERIFY-ME = substitution correction with no self-oracle; 2 = atomic
  fail. It notes collisions (md encode with no template exits 2 = clap's 2;
  ms repair 4 vs me sysw pack 4).
- /home/user/wt/cc/shared.md CLI-14 (around line 479): usage error is 2 in
  md, mt, me, mr, mr-sitting, mr-gui and 64 in mk, ms, mnemonic.

## Outcome classes to cover for every binary (and every verb where it differs)
1. success
2. "answer is no" (verify/compare mismatch, not spend-equal, membership no)
3. invalid input artifact (bad checksum, decode failure, wrong HRP, bad payload)
4. usage error (bad flag, missing required arg, bad value, unknown subcommand)
5. no input / empty input (empty stdin, empty file)
6. policy / safety refusal (secret on argv, unsafe file mode, refused content)
7. I/O error (input file missing/unreadable, output not writable)
8. repair applied (verified) / repair applied but unverified / uncorrectable
9. external dependency failure (bitcoin-cli, sidecar, node, subprocess)
10. partial success / warnings-only
11. internal error / panic (what code does a panic produce? does main catch it?)
12. anything else the binary does (help/--version exit codes, interrupted, etc.)

## Deliverable
Write your report to /home/user/wt/ec/<yourname>.md as your final action, with:
A. **Exit-code inventory**: a table per binary: code | meaning | verbs | where
   decided (file:line) | measured (exact command -> code) or "source only".
   Include how main() maps errors to codes (the error enum and its exit_code
   mapping, std::process::exit sites, ExitCode returns, clap's error path,
   whether `Error::exit()` or a custom handler is used).
B. **Class matrix**: for each of the 12 classes above, the code(s) returned,
   per verb where it varies.
C. **Collisions and surprises**: one code meaning two things in the same
   binary, the same class returning different codes across verbs, codes that
   differ from what help/manual/docs say (quote the doc and give the path).
D. **Documented codes**: every place the binary's --help, the repo's docs, or
   the toolkit manual (/home/user/wt/mnemonic-toolkit/docs/manual/src/40-cli-reference/)
   states an exit code, with path:line.
E. **Tests that pin exit codes**: files and approximate counts, so a renumber's
   test cost is known.
F. **Consumers**: anything that spawns this binary and reads its exit code
   (scripts, CI, GUIs, other binaries), path:line.
Keep it factual and dense; mark anything you inferred rather than measured.
Return a 5-line summary as your final message.
