//! `me` — convert a single md1/mk1 string to an NDEF payload (refuses ms1).

use std::io::{Read, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use mnemonic_engrave::{convert, ConvertError};
use zeroize::Zeroizing;

/// Prepare wallet backups and signed transactions for a SeedHammer II.
///
/// `me sysw pack` builds the payload the machine engraves — from `mt1`
/// strings or a `tx:` transaction record. With no subcommand, `me` is the
/// NFC converter: it turns one public constellation string (`md1`, `mk1` or
/// `mt1`) into an NDEF payload, reading it from stdin or `--in`. Refuses
/// secret `ms1`.
#[derive(Parser)]
#[command(name = "me", version, about)]
struct Cli {
    /// Read the input string from this file instead of stdin.
    #[arg(long, value_name = "FILE")]
    r#in: Option<PathBuf>,
    /// Write the NDEF bytes to this file (default: --stdout off => requires --out or an encoding flag).
    #[arg(long, value_name = "FILE")]
    out: Option<PathBuf>,
    /// Write raw NDEF bytes to stdout.
    #[arg(long, conflicts_with_all = ["hex", "base64", "out"])]
    stdout: bool,
    /// Print the NDEF bytes as hex on stdout.
    #[arg(long, conflicts_with_all = ["base64", "out"])]
    hex: bool,
    /// Print the NDEF bytes as base64 on stdout.
    #[arg(long, conflicts_with_all = ["hex", "out"])]
    base64: bool,
    /// On success, echo the validated md1/mk1 string to stderr (for pasting
    /// into a phone NFC-writer app). Off by default.
    #[arg(long)]
    echo: bool,
    /// Proceed even though stdout is a world-readable file (F-244).
    ///
    /// NDEF bytes embed md1/mk1 material, so on a multi-user host their at-rest
    /// copies must not be world- or group-readable -- the same rule `--out`
    /// already enforces by creating at 0600. Prefer `--out`.
    #[arg(long)]
    allow_world_readable: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Validate a wallet backup's public strings and emit a plate manifest + checklist.
    Bundle {
        /// Read newline-separated public strings from this file instead of stdin.
        #[arg(long, value_name = "FILE")]
        r#in: Option<PathBuf>,
        /// Write the manifest JSON to this file instead of stdout.
        #[arg(long, value_name = "FILE")]
        manifest: Option<PathBuf>,
        /// Render each public plate to an image in this directory via the
        /// `me-preview` sidecar. The sidecar is discovered only alongside the `me`
        /// executable (release archives ship them together) — `$PATH` is not
        /// searched. For a non-standard install, point at it explicitly with
        /// `ME_PREVIEW_BIN=/path/to/me-preview`. If no sidecar is found, previews
        /// are skipped (a note is printed) and the manifest is still emitted.
        #[arg(long, value_name = "DIR")]
        preview: Option<PathBuf>,
        /// With --preview, render PNG instead of SVG.
        #[arg(long, requires = "preview")]
        png: bool,
    },

    /// Build, inspect or overwrite a SYSTEMWIDE payload — the flash image a
    /// SeedHammer II engraves a transaction or a wallet backup from.
    ///
    /// A different container from `seal`, in a different flash region, read by
    /// a different set of programs — see design/SPEC_systemwide_payloads.md.
    /// The two surfaces are kept apart on purpose: no invocation should be able
    /// to produce a systemwide container while the operator believes they are
    /// producing a Sealed Payload one.
    Sysw {
        #[command(subcommand)]
        cmd: SyswCmd,
    },

    /// Encrypt a payload for delivery to SeedHammer II flash.
    ///
    /// The passphrase is GENERATED and printed to STDERR — write it down and
    /// store it apart from the machine. There is deliberately no way to supply
    /// your own: total strength is the passphrase plus about 20 bits from the
    /// KDF, and a memorable passphrase does not survive an offline attack on a
    /// stolen machine.
    Seal {
        /// Records to ENCRYPT, on argv. Kept for FIXTURES AND TESTS only.
        ///
        /// argv is a public channel: /proc/<pid>/cmdline is world-readable
        /// without hidepid, `ps` shows it, and a shell records it in history
        /// that outlives the machine. Prefer --in or stdin for anything real;
        /// a seed passed here is warned about and cannot be un-leaked (F-102).
        payload: Vec<String>,

        /// Read newline-separated records to ENCRYPT from this file, or from
        /// stdin when neither this nor argv records are given.
        ///
        /// This is the private channel the other subcommands already use, and
        /// the one to use for real seed material. Read into a Zeroizing buffer.
        #[arg(long = "in")]
        in_path: Option<PathBuf>,

        /// Records to carry in the CLEAR. Authenticated via the AAD when
        /// something is also encrypted; unauthenticated otherwise. Never an
        /// ms1 or a BIP-39 mnemonic.
        #[arg(long = "plaintext")]
        plaintext: Vec<String>,

        /// Write the UF2 here. Created 0600. REQUIRED — never stdout, because
        /// the passphrase shares that stream.
        #[arg(long, required = true)]
        out: PathBuf,

        /// Required to encrypt seed material — an ms1 record or a BIP-39
        /// mnemonic. A best-effort guard so it never happens by accident, not a
        /// security boundary (§9, §12 item 6).
        #[arg(long)]
        seal_secret: bool,

        /// Proceed even though a record on ARGV is secret or bearer material.
        ///
        /// **Declared here because the pre-parser argv guard covers `seal`
        /// too**, and the `payload` positional above is a DELIBERATELY retained
        /// channel — "Kept for FIXTURES AND TESTS only" — with a follow-up
        /// number already attached to it (F-102). Guarding the surface without
        /// offering the override would delete a documented path rather than
        /// gate it; this is the same explicit opt-in `sysw pack` uses, applied
        /// to the sibling surface with the same shape.
        ///
        /// **It is NOT `--seal-secret`, and the two do not substitute.**
        /// `--seal-secret` says *encrypting seed material is what I meant*;
        /// this one says *argv is safe where I am*. A fixture run on an
        /// air-gapped box needs both, and each still refuses on its own.
        ///
        /// Consumed by the guard, which reads raw argv before `Cli::parse()` —
        /// so clap's only job here is to ACCEPT the flag rather than reject it
        /// as unexpected and echo the record it precedes.
        #[arg(long)]
        allow_argv_secret: bool,

        /// PBKDF2 iterations. 300,000 = 30.9 s on device, from the measured
        /// 9,715 iters/sec (§7.1, measured 2026-08-07 on real RP2350).
        #[arg(long, default_value_t = 300_000)]
        iterations: u32,
    },

    /// Re-derive the §6.6 public-data hash from your own cards.
    ///
    /// No passphrase, no seal operation, no original file — so the expected
    /// value can be regenerated months later and compared against what the
    /// device displays.
    Hash {
        /// The public records, in order.
        records: Vec<String>,
        /// The payload was sealed (carries an encrypted section).
        #[arg(long, conflicts_with = "unsealed")]
        sealed: bool,
        /// The payload carries no encrypted section.
        #[arg(long)]
        unsealed: bool,
    },
}

#[derive(Subcommand)]
enum SyswCmd {
    /// Build the systemwide container: a `tx:` record (from `mt encode --qr`) becomes QR plates, `mt1` strings (from `mt encode`) become transaction TEXT plates.
    ///
    /// **That first line carries the route on purpose (F-251).** Clap renders
    /// the first LINE for `-h` and the whole comment for `--help`. The routing
    /// sentence used to sit four paragraphs down, so an operator holding a
    /// `tx:` record and typing `-h` — which is what they type — never saw the
    /// one line that told them where they were. Being first is the fix; the
    /// short/long split itself is a sound convention and was not the defect.
    ///
    /// A record is a BIP-39 mnemonic, an md1/mk1/ms1/mt1 string, or one of the
    /// prefixed forms — which carry their bodies as LOWERCASE HEX.
    ///
    /// `text:<hex of the UTF-8 bytes>` is free text.
    ///
    /// `pass:<hex of the UTF-8 bytes>` is a BIP-39 passphrase.
    ///
    /// `tx:<hex of the raw signed transaction>` feeds the device's QR
    /// engraving path — produce it with `mt encode --qr`, which
    /// checks the bytes parse AND that every input carries a signature. `me`
    /// consumes constellation strings; it manufactures none of them.
    ///
    /// mt1 strings (from `mt encode`) feed its transaction TEXT plates; pack
    /// the COMPLETE set of FULL strings — never `--elide-prefix` output, whose
    /// shortened lines are not self-verifying and are refused here — or the
    /// device will refuse to engrave it.
    ///
    /// To encode one: `printf '%s' 'correct horse battery staple' | xxd -p -c 256`
    ///
    /// The prefixes are RESERVED (spec §5.3.1): a body that is not valid
    /// lowercase hex is refused rather than quietly engraved as free text.
    /// Records are engraved verbatim and the record separator is LF, so a body
    /// carrying spaces or newlines would turn a scratch on the operator's only
    /// copy into silently-absorbed damage.
    Pack {
        /// Records, on argv. As with `seal`, argv is a PUBLIC channel — prefer
        /// --in or stdin for anything real. A `tx:` record here is REFUSED
        /// outright: a raw transaction is a BEARER instrument.
        ///
        /// `text:`/`pass:` bodies are lowercase hex — see the command help.
        records: Vec<String>,
        /// Read newline-separated records from this file instead of argv.
        ///
        /// Blank lines are skipped, so a record's index is its position among
        /// the NON-blank lines, not its line number.
        ///
        /// With neither this nor argv records, the same newline-separated form
        /// is read from STDIN — so `mt encode … | me sysw pack …` works.
        #[arg(long, value_name = "FILE")]
        r#in: Option<std::path::PathBuf>,
        /// Write the blob here instead of stdout.
        #[arg(long, value_name = "FILE")]
        out: Option<std::path::PathBuf>,
        /// Generate a passphrase of N words (2..=24).
        #[arg(long, value_name = "N", conflicts_with_all = ["passphrase_ask", "no_passphrase"])]
        passphrase_words: Option<usize>,
        /// Prompt for a passphrase on the terminal.
        ///
        /// Never argv and never an environment variable: argv is world-readable
        /// via /proc and lands in shell history that outlives the machine.
        #[arg(long, conflicts_with = "no_passphrase")]
        passphrase_ask: bool,
        /// No passphrase — the plaintext variant.
        #[arg(long)]
        no_passphrase: bool,
        /// Accepted and ignored. `me` warns rather than refusing (spec §13 D3);
        /// kept so existing invocations keep working.
        #[arg(long)]
        allow_weak: bool,
        /// Admit a `tx:` record with an input carrying NEITHER a scriptSig NOR
        /// a witness (FORWARD_PLAN §2.1).
        ///
        /// The predicate this overrides has honest false positives -- a P2A
        /// anchor-spend input carries neither and is perfectly valid -- so the
        /// escape hatch exists rather than a check nobody can get past. Every
        /// admitted record is named on stderr, with the failing input indices.
        ///
        /// It loosens NOTHING else: the body must still be lowercase hex and
        /// must still parse as one serialized transaction. It also does not
        /// reach the `mt1` chunk class, which never refuses and whose
        /// confirmation the DEVICE recomputes for itself.
        #[arg(long)]
        allow_unsigned_inputs: bool,
        /// Refuse unless the container holds every kind listed here —
        /// comma-separated: descriptor, cosigner, transaction, mnemonic,
        /// secret.
        ///
        /// **§6g.** A backup can be silently incomplete: `mk encode` refuses,
        /// the pipeline carries on, and `me sysw pack` builds a container from
        /// the `md1` records alone at exit 0. A plate is then cut from a wallet
        /// nobody can restore. `--expect descriptor,cosigner` turns that into a
        /// refusal.
        ///
        /// It checks COMPLETENESS as well as presence: a half-transmitted
        /// `md1` or `mt1` set is present and still cannot be restored from.
        ///
        /// `address` and `passphrase` are deliberately not in the vocabulary —
        /// neither can ever be satisfied here, and a kind that cannot be
        /// satisfied turns a gate into a permanent refusal.
        #[arg(long, value_name = "KINDS")]
        expect: Option<String>,
        /// Proceed even though stdout is a world-readable file (F-244).
        ///
        /// `me` refuses by default: a container is BEARER, and `>` creates a
        /// file at 0644 under the usual umask. Prefer `--out`, which `me`
        /// creates owner-only.
        #[arg(long)]
        allow_world_readable: bool,
        /// Proceed even though a record was passed on argv (ruling 2026-08-26).
        ///
        /// argv is public — `/proc`, `ps`, and a shell history file that
        /// outlives the machine — so secret and bearer records are refused
        /// there by default. **This exists because that threat model does not
        /// bite everywhere**: a single-user air-gapped box, or an amnesic Tails
        /// session, has no other observer and no persistence. The refusal is a
        /// speed bump, not a wall.
        ///
        /// Greppable in a script on purpose, so a reviewer can find it.
        #[arg(long)]
        allow_argv_secret: bool,
        /// PBKDF2 rounds.
        #[arg(long, default_value_t = 100_000)]
        iterations: u32,
        /// Pad the container out to a full `REGION_LEN` image, ready to write at
        /// `0x10D00000`.
        ///
        /// The tail is `0xFF` — the ERASED state of NOR flash — so the result is
        /// byte-for-byte what the sector looks like with only the container
        /// written. Zero-padding would be a WRITE of 65 KiB for nothing.
        #[arg(long)]
        region: bool,
    },
    /// Emit a full-region overwrite image (spec §5.5).
    Wipe {
        #[arg(long, value_name = "FILE")]
        out: Option<std::path::PathBuf>,
        /// random (default), zeros, or ones. NOTE: ones is the ERASED state of
        /// NOR flash, so that region is indistinguishable from one never written.
        #[arg(long, default_value = "random")]
        fill: String,
    },
    /// Print what a container holds, and its digest.
    Show { file: std::path::PathBuf },
}

/// THE EXIT-CODE VOCABULARY. Three failure codes, three different things, and
/// the distinction is what makes them usable from a script:
///
/// | code | meaning | the operator's next move |
/// | --- | --- | --- |
/// | 2 | **usage** — the invocation is wrong. Nothing was read and nothing about the DATA was judged | fix the command line |
/// | 3 | **policy refusal** — understood, well-formed, and refused on purpose | this tool will never do that |
/// | 4 | **invalid** — the INPUT is not what it must be | fix the input |
///
/// Every site returns one of these BY NAME. Bare integers are what let
/// `me seal --iterations 5` exit 2 while `me sysw pack --iterations 5` exited
/// 4, for the same typo on the same flag with the same range in the same
/// sentence — invisible until the two were put in one table
/// (`tests/cli.rs::the_exit_code_vocabulary_is_one_vocabulary`).
///
/// A flag value out of range is USAGE, not invalid: no input has been read at
/// the point it is caught, so there is nothing yet for "invalid" to be about.
const EXIT_OK: i32 = 0;
const EXIT_USAGE: i32 = 2;
const EXIT_REFUSED: i32 = 3;
const EXIT_INVALID: i32 = 4;

/// Every string a token could plausibly BE, normalised for classification.
///
/// **`classify` neither trims nor case-folds**, so ` TX:<hex>`, `TX:<hex>` and
/// an uppercase `MS1…` all come back `Unknown` and leak — measured on 4 and 2
/// surfaces respectively. Normalising here is what makes the pre-parser guard
/// deliberately STRONGER than the donor's shipped post-parse gate, which
/// normalises for its `tx:` PREFIX arm only (F-270, fixed in the same commit).
///
/// `=`-joined tokens are split, because `--in=<ms1>` is one argv token and the
/// secret is the right-hand half of it. Splitting on every `=` rather than the
/// first costs nothing and cannot miss a shape.
fn argv_candidates(token: &str) -> Vec<String> {
    let norm = |s: &str| s.trim().to_ascii_lowercase();
    let mut v = vec![norm(token)];
    if token.contains('=') {
        v.extend(token.split('=').map(norm));
    }
    v
}

/// Is this argv asking for `--allow-argv-secret`, **on a surface that declares
/// it**?
///
/// **The override's own parse has to run here too** (round-9 I-3): otherwise it
/// cannot be honoured without parsing the very argv the guard exists to protect.
///
/// **And it binds only where the flag is DECLARED** (round-11 M-2). `me`
/// declares `allow_argv_secret` on `sysw pack` and `seal` — so on `me`, `bundle`, `sysw show`, `sysw wipe` and the helps
/// the guard refuses even when argv carries the flag. Today those surfaces make
/// clap reject it as an unexpected argument at exit 2; the guard reaching its
/// answer first is the point of §6d's ordering.
fn argv_override_applies(argv: &[String]) -> bool {
    if !argv.iter().any(|t| t == "--allow-argv-secret") {
        return false;
    }
    // The surface is the LITERAL leading tokens at argv[1..]. It is NOT "the
    // leading run of non-flag tokens" -- that reading let a FLAG VALUE spoof the
    // surface, because filtering out `-`-prefixed tokens keeps the value that
    // follows one: `me --out seal <ms1> --allow-argv-secret` presented `seal` as
    // the surface and granted the override on bare `me`, which does not declare
    // the flag, after which clap echoed the whole secret (pre-publish review,
    // Minor 1).
    //
    // Reading argv[1..] literally is also the smaller parse, and it is the one
    // that matches how clap resolves a subcommand: the words come first.
    let words: Vec<&str> = argv.iter().skip(1).map(String::as_str).collect();
    // The two surfaces that DECLARE the flag. `sysw pack` and `seal` are the
    // only ones whose positionals are records the operator may legitimately
    // intend to be secret; everywhere else an argv-forbidden token is an
    // accident, and there is nothing to opt into.
    matches!(words.first().copied(), Some("seal"))
        || (words.len() >= 2 && words[0] == "sysw" && words[1] == "pack")
}

/// The invocation as a `sed`-safe pattern: `me`, plus the subcommand words that
/// led it.
///
/// **The purge recipe matches on the COMMAND, never on the secret** — quoting
/// the secret into a pattern is how an operator types it into history a second
/// time — so this has to name the invocation without reproducing any of it.
///
/// **The words come from an ALLOWLIST, and that is the whole safety argument.**
/// Deriving them instead — "leading tokens that classify as `Unknown`" — would
/// admit a TRUNCATED or otherwise unparseable secret into the pattern, since
/// `Unknown` is exactly what a near-miss returns. An allowlist of `me`'s own
/// eight subcommand words cannot carry material at all.
///
/// The bare `me` surface yields `"me"` alone, and that is why the pattern is
/// built rather than fixed at `"me"`: `sed '/me/d'` would delete `make`,
/// `time` and `/home/me` from the operator's history. `me` is only ever the
/// FIRST word here, so a two-word surface like `me bundle` is specific enough,
/// and the recipes quote it as written.
fn argv_surface(argv: &[String]) -> String {
    const SUBCOMMANDS: [&str; 8] = [
        "bundle", "sysw", "seal", "hash", "help", "pack", "wipe", "show",
    ];

    let mut s = String::from("me");
    for t in argv.iter().skip(1).take(2) {
        if !SUBCOMMANDS.contains(&t.as_str()) {
            break;
        }
        s.push(' ');
        s.push_str(t);
    }
    s
}

/// **THE PRE-PARSER argv GUARD (§6d, F-266).** Returns the refusal, or `None`.
///
/// **It runs before `Cli::parse()`, and that ordering is NORMATIVE.** A guard
/// downstream of the parser has already lost: `mt`'s source records the same
/// lesson from the other side — when its check lived inside the `encode`
/// subcommand, clap rejected the unexpected positional first, **and clap's
/// error echoed the entire bearer transaction to stderr.** `me` leaks exactly
/// this way today, on 15 of 24 measured surface×shape combinations, including
/// `--in <ms1>` and `--in=<ms1>` and the subcommand shapes.
///
/// **It does not invent a recogniser. It asks `me`'s own classifier** —
/// `classify(token)`, then [`Class::is_argv_forbidden`], the union of
/// `is_secret()` and `is_bearer()`, **five** classes with `pass:` among them.
/// Two earlier drafts of this work enumerated first surfaces and then shapes,
/// and both lists came up short; the classifier defines the set, so there is no
/// list to be short.
///
/// **Granularity is the classifier's, and one direction is traded away
/// knowingly.** A classifier DECODES rather than prefix-matches, so
/// `mt1-2026-08-23-transfer.txt` is a filename and is not refused, and a single
/// word is not a mnemonic — which is what keeps `bundle` and `help`, both BIP-39
/// words, from being refused as subcommands. The cost: an UNQUOTED twelve-word
/// mnemonic is twelve tokens, each `Unknown`, so the guard does not reach it.
/// Only the quoted, single-token phrase is in its reach. A secret embedded in a
/// PATH is out of reach for the same reason and is filed as **F-267**, not
/// papered over: `--in /tmp/<ms1>.txt` classifies as `Unknown` because it IS a
/// filename, and refusing it would refuse every legitimate path.
///
/// **Why it lives in `me` and not in the shared crate** (round-10 C-1): it asks
/// `me`'s own `classify()`, and `me` depends on the crate — so siting it there
/// is a reproduced `error: cyclic package dependency`, and it would break the
/// crate's rule that nothing in it ever names a `Class` variant.
fn argv_secret_guard(argv: &[String]) -> Option<String> {
    if argv_override_applies(argv) {
        return None;
    }
    for (i, token) in argv.iter().enumerate() {
        for cand in argv_candidates(token) {
            let class = mnemonic_engrave::sysw::classify(&cand);
            if !class.is_argv_forbidden() {
                continue;
            }
            // NAME THE CLASS, NEVER THE BODY. Printing it back would put the
            // material in a SECOND public place -- the defect this refusal
            // exists to name. The POSITION is named instead: it is derived
            // from the argv we were handed and tells the operator which
            // argument to stop passing.
            let what = if class.is_bearer() {
                "BEARER material -- a signed transaction, or the mt1 set carrying one. \
                 Anyone who can read it can broadcast it"
            } else {
                "SECRET key material. It can spend everything derived from it, forever"
            };
            let purge = mnemonic_engrave::io::remedy::history_purge_block(&argv_surface(argv));
            return Some(format!(
                "argument {i} on ARGV (arguments count from 0, and 0 is `me` \
                 itself) is {what}.\n      \
                 Refused BEFORE the command line was parsed; nothing was read and \
                 nothing was written.\n      \
                 argv is public: /proc, `ps` and your shell history all keep a \
                 copy, so the argument parser must not be allowed to echo it \
                 back in an error message.\n      \
                 Use a private channel instead:\n      \
                 \x20   me sysw pack --in records.txt --out p.bin\n\n      \
                 {purge}\n      \
                 If argv is safe where you are -- a single-user air-gapped box, \
                 an amnesic Tails session -- `me sysw pack --allow-argv-secret` \
                 proceeds. That flag is declared on `sysw pack` alone, so it \
                 does not buy past this refusal anywhere else."
            ));
        }
    }
    None
}

fn main() {
    std::process::exit(run());
}

fn run() -> i32 {
    // §6d, and the ORDERING IS THE FIX. This must decide before
    // `Cli::parse()`: clap names the offending VALUE in its error for every
    // shape that has no declared flag to blame, so a guard placed one line
    // lower has already lost. `me --nosuchflag <ms1>` is the test that tells
    // the two apart -- via the guard it is exit 3 with this wording, via clap
    // it is exit 2 naming the flag.
    if let Some(msg) = argv_secret_guard(&std::env::args().collect::<Vec<_>>()) {
        eprintln!("me: {msg}");
        return EXIT_REFUSED;
    }
    let cli = Cli::parse();

    if let Some(Command::Bundle {
        r#in,
        manifest,
        preview,
        png,
    }) = &cli.command
    {
        return run_bundle_cli(r#in.as_ref(), manifest.as_ref(), preview.as_ref(), *png);
    }

    if let Some(Command::Sysw { cmd }) = &cli.command {
        return run_sysw(cmd);
    }
    if let Some(Command::Seal {
        payload,
        plaintext,
        out,
        seal_secret,
        // Read by `argv_secret_guard` off raw argv before `Cli::parse()` ever
        // ran, so there is nothing left to consult here. Destructured by name
        // rather than swallowed by `..` so that a reader of this match sees
        // the flag exists and finds the comment above.
        allow_argv_secret: _,
        in_path,
        iterations,
    }) = &cli.command
    {
        return run_seal_cli(
            payload,
            in_path.as_ref(),
            plaintext,
            out,
            *seal_secret,
            *iterations,
        );
    }
    if let Some(Command::Hash {
        records,
        sealed,
        unsealed,
    }) = &cli.command
    {
        if *sealed == *unsealed {
            eprintln!("me: pass exactly one of --sealed or --unsealed");
            return EXIT_USAGE;
        }
        return run_hash_cli(records, *sealed);
    }

    // Read into a Zeroizing buffer so the input (incl. read_to_string's
    // allocation, which a secret could reach via --in) is scrubbed on drop —
    // defense-in-depth on top of the ms1 refusal.
    let mut input = Zeroizing::new(String::new());
    if let Some(path) = &cli.r#in {
        match std::fs::read_to_string(path) {
            Ok(s) => *input = s, // moves the buffer into the Zeroizing wrapper
            Err(e) => {
                eprintln!("me: cannot read {}: {e}", path.display());
                return EXIT_USAGE;
            }
        }
    } else if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("me: cannot read stdin: {e}");
        return EXIT_USAGE;
    }

    // NOTHING PIPED IN IS A USAGE ERROR, not a decode failure. Before this,
    // empty input reached `convert` and came back
    // "not a bech32 string (no '1' separator / empty HRP)" at exit 4 --
    // describing input the operator never gave as malformed, and disagreeing
    // with `me sysw pack`, which has always said "no ... given (pipe it in, or
    // --in FILE)" at exit 2 for the same situation. A new user's first action
    // is exactly this one. (The verb it used to agree with was `me tx`, which
    // has since moved to `mt encode --qr`; the agreement is with the
    // remaining verbs, and it still has to hold.)
    if input.trim().is_empty() {
        eprintln!("me: no input given (pipe an md1/mk1 string in, or --in FILE)");
        return EXIT_USAGE;
    }

    // Capture the plate-budget flag before the input is dropped.
    let too_long = mnemonic_engrave::exceeds_plate_budget(&input);

    let result = convert(&input);

    // Build the --echo line ONLY on the success path, where the input is a
    // verified PUBLIC md1/mk1 string. Building it before convert() (or on the
    // refusal path) would copy an ms1 secret into a heap String that escapes
    // the Zeroizing scrub of `input` — so the allocation must be unreachable
    // for ms1. We still wrap it in Zeroizing as belt-and-suspenders against any
    // future reordering of the refusal guard.
    let echo_line: Option<Zeroizing<String>> = if cli.echo && result.is_ok() {
        let s = input.trim();
        let label = if s.starts_with("mk1") { "mk1" } else { "md1" };
        Some(Zeroizing::new(format!("me: validated {label}: {s}")))
    } else {
        None
    };

    drop(input); // Zeroizing scrubs the input buffer here

    let bytes = match result {
        Ok(b) => b,
        Err(ConvertError::RefusedSecret) => {
            eprintln!("me: {}", ConvertError::RefusedSecret);
            return EXIT_REFUSED;
        }
        Err(e) => {
            eprintln!("me: {e}");
            return EXIT_INVALID;
        }
    };

    if too_long {
        eprintln!("me: warning: input is long; it may exceed one plate (the device will reject with ErrTooLarge if so)");
    }
    if let Some(line) = &echo_line {
        eprintln!("{}", line.as_str());
    }

    // Emit per the selected output mode. Human guidance -> stderr only.
    if let Some(path) = &cli.out {
        if let Err(e) = write_private(path, &bytes) {
            eprintln!("me: cannot write {}: {e}", path.display());
            return EXIT_USAGE;
        }
        eprintln!("me: wrote {} NDEF bytes to {}", bytes.len(), path.display());
    } else if cli.hex || cli.base64 || cli.stdout {
        // F-244: all three stdout modes carry the SAME bytes; gating raw and not
        // hex would teach the operator to reach for hex. `--out` was already
        // owner-only via write_private -- these were the paths that were not.
        if let (false, Some(mode)) = (cli.allow_world_readable, stdout_world_readable_mode()) {
            refuse_world_readable_stdout(mode);
            return EXIT_USAGE;
        }
        if cli.hex {
            let s: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
            println!("{s}");
        } else if cli.base64 {
            println!("{}", base64_encode(&bytes));
        } else if std::io::stdout().write_all(&bytes).is_err() {
            return EXIT_USAGE;
        }
    } else {
        eprintln!("me: choose an output mode: --out <file>, --stdout, --hex, or --base64");
        return EXIT_USAGE;
    }
    EXIT_OK
}

fn run_bundle_cli(
    in_path: Option<&PathBuf>,
    manifest_path: Option<&PathBuf>,
    preview_dir: Option<&PathBuf>,
    png: bool,
) -> i32 {
    let mut input = Zeroizing::new(String::new());
    if let Some(path) = in_path {
        match std::fs::read_to_string(path) {
            Ok(s) => *input = s,
            Err(e) => {
                eprintln!("me: cannot read {}: {e}", path.display());
                return EXIT_USAGE;
            }
        }
    } else if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("me: cannot read stdin: {e}");
        return EXIT_USAGE;
    }

    let mut manifest = match mnemonic_engrave::bundle::run_bundle(&input) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("me: {e}");
            return e.exit_code();
        }
    };

    // Phase B: optional plate previews via the `me-preview` sidecar. Without
    // --preview this block is skipped entirely → byte-for-byte Phase A output.
    if let Some(dir) = preview_dir {
        if let Some(code) = wire_previews(&mut manifest, dir, png) {
            return code; // a non-zero outcome (version mismatch / render fail).
        }
    }

    let json = match serde_json::to_string_pretty(&manifest) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("me: cannot serialize manifest: {e}");
            return EXIT_USAGE;
        }
    };
    if let Some(path) = manifest_path {
        if let Err(e) = write_private(path, json.as_bytes()) {
            eprintln!("me: cannot write {}: {e}", path.display());
            return EXIT_USAGE;
        }
        eprintln!("me: wrote manifest to {}", path.display());
    } else {
        println!("{json}");
    }
    eprint!("{}", manifest.checklist());
    EXIT_OK
}

// `out: &Path`, not `&PathBuf` — clippy::ptr_arg is warn-by-default and the
// plan's own `-D warnings` gate would reject it. Call sites deref-coerce.
fn run_seal_cli(
    payload: &[String],
    in_path: Option<&PathBuf>,
    plaintext: &[String],
    out: &std::path::Path,
    seal_secret: bool,
    iterations: u32,
) -> i32 {
    use mnemonic_engrave::classify::{classify, Format};
    use mnemonic_engrave::seal::{self, pubhash, Payload};

    // WHERE THE RECORDS COME FROM, and why the zeroizing changed with it
    // (F-102). This used to read "these records are NOT zeroized ... §9 puts
    // them on argv, so /proc/$PID/cmdline already exposes them — the heap copy
    // is not the binding exposure". That reasoning was sound and is now
    // obsolete: with --in and stdin, a record can arrive over a PRIVATE
    // channel, and then the heap copy IS the binding exposure. So the private
    // path reads into a Zeroizing buffer, exactly as `convert` and `bundle` do.
    //
    // argv survives for fixtures and tests, and warns when it carries a seed.
    let mut input = Zeroizing::new(String::new());
    let from_argv = !payload.is_empty();
    let secret: Vec<String> = if from_argv {
        if in_path.is_some() {
            eprintln!("me: pass records on argv OR via --in, not both");
            return EXIT_USAGE;
        }
        payload.to_vec()
    } else {
        if let Some(path) = in_path {
            match std::fs::read_to_string(path) {
                Ok(s) => *input = s,
                Err(e) => {
                    eprintln!("me: cannot read {}: {e}", path.display());
                    return EXIT_USAGE;
                }
            }
        } else if let Err(e) = std::io::stdin().read_to_string(&mut input) {
            eprintln!("me: cannot read stdin: {e}");
            return EXIT_USAGE;
        }
        // Split on '\n' ONLY, and do not trim. `encode_section` scans the
        // UNTRIMMED record for `\r` (§6.4: "no CR anywhere"), so a CRLF file
        // must be REFUSED rather than silently normalised — §9 says refuse.
        // Dropping empty lines is not normalisation; a lone "\r" line survives
        // and is refused, which is the point.
        input
            .split('\n')
            .filter(|l| !l.is_empty())
            .map(str::to_owned)
            .collect()
    };
    // Do NOT trim here. `encode_section` scans the UNTRIMMED record for `\r`
    // (§6.4: "no CR anywhere"), and trimming first strips a leading/trailing CR
    // before that check ever sees it — the CLI would normalise where §9 says
    // refuse. `encode_section` trims internally, once, after the CR scan.
    let public: Vec<String> = plaintext.to_vec(); // &[String] -> Vec<String>
    if secret.is_empty() && public.is_empty() {
        eprintln!("me: nothing to seal");
        return EXIT_USAGE;
    }

    // §9: seed material needs the explicit opt-in. Best-effort anti-footgun, not
    // a security boundary — it exists so nobody seals a seed by ACCIDENT, not to
    // stop anyone who means to. Covers ms1 and a bare BIP-39 mnemonic, which are
    // the same secret; `classify` wants a bech32 `1`, so it misses the mnemonic.
    let is_seed =
        |r: &String| matches!(classify(r), Ok(Format::Ms)) || seal::passphrase::is_valid(r);

    // F-102: argv is a public channel. If a seed came in that way, say so --
    // once, loudly, and without refusing, because fixtures legitimately use it
    // and a hard failure would only teach people to quote their way around it.
    // The leak has already happened by the time this prints; the warning exists
    // so the operator knows to treat that seed as compromised rather than
    // discovering it later.
    if from_argv && secret.iter().any(is_seed) {
        eprintln!(
            "me: WARNING -- seed material was passed on the COMMAND LINE.\n    \
             /proc/<pid>/cmdline is world-readable without hidepid, `ps` shows it, and your\n    \
             shell has already written it to history. Treat this seed as EXPOSED.\n    \
             For real seed material use --in <file> or stdin instead."
        );
    }
    if !seal_secret && secret.iter().any(is_seed) {
        eprintln!(
            "me: refusing to seal seed material (ms1 or a BIP-39 mnemonic) without \
             --seal-secret.\n    \
             Re-run with --seal-secret if that is what you intend."
        );
        return EXIT_REFUSED;
    }

    let sealed = match seal::seal(
        Payload {
            public: public.clone(),
            secret,
        },
        iterations,
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("me: {e}");
            return match e {
                seal::SealError::Iterations(_) => EXIT_USAGE,
                _ => EXIT_INVALID,
            };
        }
    };

    let uf2 = seal::uf2::to_uf2(&sealed.blob);
    if let Err(e) = write_private(out, &uf2) {
        eprintln!("me: cannot write {}: {e}", out.display());
        return EXIT_USAGE;
    }

    // STDERR, always (§2.3).
    eprintln!("me: wrote {} bytes to {}", uf2.len(), out.display());
    if !public.is_empty() {
        // TRIM here. `public` now holds raw argv (the CR fix removed the CLI
        // trim), but the blob's public section is what `encode_section` emits —
        // trimmed. Hashing the untrimmed form prints a value the device can
        // never display: measured, one leading space gives a BYTE-IDENTICAL
        // blob and a different hash, on the only integrity control an unsealed
        // payload has. `check_public` and `run_hash_cli` already trim.
        let refs: Vec<&str> = public.iter().map(|s| s.trim()).collect();
        let h = pubhash::public_data_hash(&refs, sealed.passphrase.is_some());
        eprintln!();
        eprintln!(
            "public data hash ({} records, {}):",
            public.len(),
            if sealed.passphrase.is_some() {
                "SEALED"
            } else {
                "UNSEALED"
            }
        );
        eprintln!("    {}", pubhash::format_hash(&h));
        eprintln!("RECORD THIS WHOLE LINE. The device shows the same value; if it");
        eprintln!("differs, the payload has been altered or its encryption removed.");
    }
    if let Some(p) = &sealed.passphrase {
        eprintln!();
        eprintln!("passphrase — write this down and store it APART from the machine:");
        eprintln!();
        eprintln!("    {}", &**p);
    }
    eprintln!();
    eprintln!(
        "load:  picotool load --verify {}   (machine in BOOTSEL)",
        out.display()
    );
    eprintln!("wipe:  picotool erase -r 0x10E00000 0x10E10000");
    EXIT_OK
}

fn run_hash_cli(records: &[String], sealed: bool) -> i32 {
    use mnemonic_engrave::seal::{pubhash, record};
    if records.is_empty() {
        eprintln!("me: no records given");
        return EXIT_USAGE;
    }
    let trimmed: Vec<String> = records.iter().map(|s| s.trim().to_string()).collect();
    for (i, r) in trimmed.iter().enumerate() {
        match record::validate_record(r) {
            Err(e) => {
                eprintln!("me: record {i}: {e}");
                return EXIT_INVALID;
            }
            // §6.3 forbids a secret in the public section, so hashing one would
            // print a confident value for a payload no device could ever hold.
            Ok(k) if k.is_secret() => {
                eprintln!(
                    "me: record {i} is secret material; the public-data hash \
                           covers public records only"
                );
                return EXIT_INVALID;
            }
            Ok(_) => {}
        }
    }
    let refs: Vec<&str> = trimmed.iter().map(|s| s.as_str()).collect();
    // Same card-set decode `me seal --plaintext` applies, so `me hash` cannot
    // bless a record list that `me seal` would refuse.
    if let Err(e) = record::decode_public_set(&refs) {
        eprintln!("me: {e}");
        return EXIT_INVALID;
    }
    println!(
        "{}",
        pubhash::format_hash(&pubhash::public_data_hash(&refs, sealed))
    );
    EXIT_OK
}

/// Wire `--preview` into the manifest, rendering each public plate via the
/// `me-preview` sidecar.
///
/// Returns:
///   - `None` to continue (the common case): either previews were rendered, or
///     the sidecar is absent and we degrade gracefully (note on stderr, exit 0).
///   - `Some(code)` to stop now: version mismatch / unreadable version / non-dir
///     target → `EXIT_USAGE` (2); a sidecar RENDER failure (bad input, e.g. a
///     string that fits no plate) → `EXIT_INVALID` (4) per spec §6; a Spawn/IO
///     failure (couldn't run the sidecar) → `EXIT_USAGE` (2).
fn wire_previews(
    manifest: &mut mnemonic_engrave::manifest::Manifest,
    dir: &std::path::Path,
    png: bool,
) -> Option<i32> {
    use mnemonic_engrave::manifest::PlateKind;
    use mnemonic_engrave::preview;

    // Explicit opt-in: `ME_PREVIEW_BIN` names a specific sidecar binary and takes
    // precedence over co-located discovery. Read it here in the wrapper (before the
    // version gate) so `locate_in` stays pure. A set-but-missing path is a FAIL-LOUD
    // usage error (EXIT_USAGE): the user vouched for a specific binary that isn't
    // there, so silently degrading — or falling back to exe-adjacent — would be
    // surprising. (An empty value is treated as unset.)
    let explicit_env = std::env::var_os("ME_PREVIEW_BIN")
        .filter(|v| !v.is_empty())
        .map(std::path::PathBuf::from);
    if let Some(p) = &explicit_env {
        if !p.is_file() {
            eprintln!(
                "me: ME_PREVIEW_BIN={} does not point to an existing file \
                 (set it to the me-preview binary, or unset it for co-located discovery)",
                p.display()
            );
            return Some(EXIT_USAGE);
        }
    }

    // Discover the sidecar. `explicit_env`, if set, is now known to exist and takes
    // precedence; otherwise co-located-only. Absent → graceful degrade (note, exit 0).
    let sidecar = match preview::locate_sidecar(explicit_env.as_deref()) {
        Some(p) => p,
        None => {
            eprintln!("me: preview skipped (install me-preview)");
            return None;
        }
    };

    // Version-gate: the sidecar must match this crate's version exactly.
    let expected = env!("CARGO_PKG_VERSION");
    match preview::sidecar_version(&sidecar) {
        Ok(found) if found == expected => {}
        Ok(found) => {
            eprintln!(
                "me: me-preview version mismatch: sidecar is {found:?}, expected {expected:?}; \
                 refusing to render (install the matching me-preview)"
            );
            return Some(EXIT_USAGE);
        }
        Err(e) => {
            eprintln!("me: cannot determine me-preview version: {e}");
            return Some(EXIT_USAGE);
        }
    }

    // The output directory must exist and be writable.
    if !dir.is_dir() {
        eprintln!(
            "me: preview directory {} is not a writable directory",
            dir.display()
        );
        return Some(EXIT_USAGE);
    }

    // Fail-closed (F8): refuse a dir that already holds foreign `plate-*` artifacts
    // (e.g. higher-index plates from a prior run) so they can't mix into this
    // manifest. We refuse rather than delete — never clobber a user file that
    // happens to match. Scanned once, here, before any render; the loop's own
    // writes below are not re-scanned.
    match std::fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if is_plate_artifact(name) {
                        eprintln!(
                            "me: preview directory {} already contains plate artifacts \
                             (e.g. {name}); use an empty/clean directory",
                            dir.display()
                        );
                        return Some(EXIT_USAGE);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("me: cannot scan preview directory {}: {e}", dir.display());
            return Some(EXIT_USAGE);
        }
    }

    // Render each PUBLIC plate; ms1 is never rendered (no secret leaves `me`).
    for plate in manifest.plates.iter_mut() {
        if plate.kind == PlateKind::Ms1 {
            continue;
        }
        let Some(string) = plate.string.as_deref() else {
            continue;
        };
        match preview::render_plate(&sidecar, string, dir, plate.plate, png) {
            Ok(path) => {
                eprintln!("me: rendered plate {} → {path}", plate.plate);
                plate.preview = Some(path);
            }
            Err(e) => {
                eprintln!("me: {e}");
                // Spec §6: a sidecar RENDER failure (e.g. a string that fits no
                // plate) is an invalid-input outcome → exit 4. A Spawn/IO failure
                // (couldn't run the sidecar) is an environment/usage error → exit 2.
                return Some(match e {
                    preview::PreviewError::Render { .. } => EXIT_INVALID,
                    // A render that produced no usable artifact (empty/garbage
                    // output) is an invalid outcome, same class as a render
                    // failure → exit 4 (the default `_` would map it to 2).
                    preview::PreviewError::EmptyOutput { .. } => EXIT_INVALID,
                    _ => EXIT_USAGE,
                });
            }
        }
    }
    None
}

/// True if `name` is a preview plate artifact this tool writes: a `plate-` prefix
/// AND a `.svg`/`.png` extension. Used by the F8 dirty-dir scan. Fail-closed: it
/// must not over-match unrelated files (`notes.txt`, `plate.txt`, `plateau.svg`).
fn is_plate_artifact(name: &str) -> bool {
    name.starts_with("plate-") && (name.ends_with(".svg") || name.ends_with(".png"))
}

// The P0 moving set now lives in the LIBRARY half (`src/io.rs`). The binary
// keeps `read_records`, `emit`, `write_private` and every `refuse_*` -- the
// acts and the announcements -- and consumes the decisions from here.
use mnemonic_engrave::io::observation::PayloadKind;
use mnemonic_engrave::io::{no_records_guard, split_record_stream, write_block, WriteBlock};

/// Write `bytes` to `path`, creating/truncating it with owner-only permissions.
///
/// F10 (D5-2): NDEF and manifest artifacts embed/depict md1/mk1 material, so on a
/// multi-user host their at-rest copies must not be world/group-readable. Under
/// Unix we create the file at mode `0o600`; on other platforms we fall back to the
/// same create+truncate semantics without a mode (mode bits differ there — the
/// threat model is POSIX). `.truncate(true)` is load-bearing: it preserves
/// `std::fs::write`'s behavior so a shrinking overwrite (a smaller manifest over a
/// larger one) can't leave trailing stale bytes → invalid JSON.
///
/// Note: `0o600` binds on CREATE. Overwriting a pre-existing world-readable file
/// keeps its old mode — accepted residual (NDEF/manifest targets are user-named;
/// preview targets are forced-fresh by the dirty-dir refusal).
fn write_private(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::fs::OpenOptions;
    let mut opts = OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts.open(path)?;
    // F-244: `0o600` binds on CREATE, so an existing world-readable target kept
    // its old mode -- measured true, and it is the case an operator re-running a
    // command actually hits. Tightening the OPEN file (rather than the path)
    // cannot be raced onto a different file between the two calls.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        f.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    f.write_all(bytes)
}

/// **`me`'s POLICY, over `io::fd`'s MECHANISM — and the mask is the whole of
/// the split.**
///
/// `io::fd::stdout_mode` hands back the raw `mode & 0o777` of a regular-file
/// stdout, or `None` for a character device or a failed `fstat`. Deciding that
/// `0o044` — read for group or other — is what disqualifies a destination is
/// **`me`'s ruling and nobody else's**: `mt` rules `0o077`, refusing a
/// group-WRITABLE destination too, because someone who can write the file can
/// alter the strings before they are cut into metal.
///
/// That disagreement is deliberate and P0 does not settle it. Keeping the mask
/// **here** is what stops it being settled by accident: a `0o044` published as
/// shared *mechanism* would let a later phase delete `mt`'s stricter refusal
/// while citing a real uniformity ruling — silent reconciliation in favour of
/// the weaker rule, on the path where the artifact is cut into metal.
fn stdout_world_readable_mode() -> Option<u32> {
    mnemonic_engrave::io::fd::stdout_mode().filter(|mode| mode & 0o044 != 0)
}

/// Report a [`WriteBlock`] and yield the exit code, or `None` to proceed.
fn refuse_write_block(b: WriteBlock, len: usize) -> Option<i32> {
    match b {
        WriteBlock::None => None,
        // F-259: the KIND is forwarded, not discarded. Matching this as
        // `Terminal(_)` and hard-coding the wording is the exact edit that
        // re-created the defect under a clean build, clean clippy and a green
        // suite -- so `tests/terminal_destination.rs` asserts the emitted
        // WORDS, which is the only thing that noticed.
        WriteBlock::Terminal(kind) => {
            refuse_terminal_destination(len, kind);
            Some(EXIT_USAGE)
        }
        WriteBlock::WorldReadable(mode) => {
            refuse_world_readable_stdout(mode);
            Some(EXIT_USAGE)
        }
    }
}

/// F-253 — refuse to paint a bearer container across the operator's terminal,
/// and say what to run instead.
///
/// **The char-device exemption in `stdout_world_readable_mode` cannot catch
/// this, and its stated reason is false.** That comment justifies exempting
/// character devices with *"a terminal and `/dev/null` persist nothing, so
/// neither can leak"*. The `/dev/null` half is right and load-bearing —
/// `/dev/null` is mode 0666, so a mode-only test would refuse
/// `me … > /dev/null`. The terminal half is not: a terminal persists in
/// scrollback, and sessions are routinely logged. This finding was itself
/// captured through `script`.
///
/// **The shape is the operator's proposal, and `me seal` set the precedent** —
/// it already prints `load:`/`wipe:` lines for its own region. `pack` printed
/// none. So this is the sibling verb's behaviour, with `sysw`'s address.
///
/// **Piping into `picotool` is deliberately NOT offered.** Settled on hardware
/// 2026-08-25: picotool sizes its input with `fstat`, a pipe reports `st_size`
/// 0, and `picotool load /dev/stdin` therefore exits **0 having written
/// nothing** — a silent no-op on a flashing operation. The file is the route.
fn refuse_terminal_destination(len: usize, kind: PayloadKind) {
    use mnemonic_engrave::sysw::wire::{REGION_ADDR, REGION_LEN};
    // 0 means "not built, and never will be" -- the early gate refuses before
    // the container exists. Naming a size there would be inventing one.
    let size = if len == 0 {
        String::new()
    } else {
        format!("{len} bytes of ")
    };
    // F-259. The refusal is the same and so is its code; only the CLAIM about
    // the operator's data is derived rather than asserted. A fill image is
    // still refused -- 64 KB of binary in a scrollback is worth refusing
    // whatever the secrecy -- but it is not bearer, and saying so taught
    // operators that the label means nothing.
    if kind == PayloadKind::CarriesNoSecret {
        refuse_terminal_fill_image(len);
        return;
    }
    eprintln!(
        "me: stdout is a TERMINAL, and this payload is BEARER.\n\
         \n\
         Writing it here would paint {size}raw binary across your \
         scrollback — and terminal sessions are often logged. Nothing was \
         written.\n\
         \n\
         Give it a file, then flash that file:\n\
         \n\
           me sysw pack --region --out payload.bin  ...\n\
           picotool load --verify payload.bin -t bin -o 0x{REGION_ADDR:08X}\n\
         \n\
         with the machine in BOOTSEL. To clear the region instead:\n\
         \n\
           picotool erase -r 0x{REGION_ADDR:08X} 0x{:08X}\n\
         \n\
         Do NOT pipe into picotool: it sizes its input with fstat, a pipe \
         reports 0 bytes, and the load exits 0 having written nothing.",
        REGION_ADDR as usize + REGION_LEN
    );
}

/// F-259's half of the terminal refusal: a fill image is refused for what it
/// WOULD DO to the scrollback, and **nothing is claimed about what it holds.**
///
/// **Wording and exit digit are an architect consult's, folded verbatim** —
/// `design/agent-reports/CONSULT-P0-row4-f259-refusal.md`. Its three
/// load-bearing facts were machine-checked before folding: `--fill` really does
/// default to `random` (`default_value = "random"`), `REGION_ADDR` is
/// `0x10D0_0000` and `REGION_LEN` is `65_536`.
///
/// **The digit stays 2.** `me`'s vocabulary discriminates by the operator's
/// next move, and 2 means *fix the command line* — which is exactly this
/// remedy, one flag away. 3 would promise *this tool will never do that*, and
/// that is false: `--out` is the sanctioned path to the same bytes. Forking the
/// digit by payload kind would also assign the LESS sensitive payload the MORE
/// severe code while the bearer arm stays at 2 for the identical condition.
///
/// **Every sentence is derived from something observed:**
/// - *"a WIPE image, not a secret"* affirms the true classification instead of
///   asserting the false one, which is the whole of F-259's ruling.
/// - The byte count and the destination are both measured.
/// - **`--fill ...` names no value**, because the fill defaults to `random` and
///   this function is downstream of the single write decision, so it does not
///   know which was asked for. Hard-coding `--fill zeros` would state a false
///   command for two of the three invocations — F-260's exact shape, one line
///   over. `wipe.bin` rather than `payload.bin` keeps a fill image from ever
///   shadowing a real payload file.
/// - **"terminal sessions are often logged" is DROPPED, not rephrased.**
///   Logging is a secrecy rationale; keeping it for a no-secret image would be
///   the same rule-name vestige F-259 exists to remove.
/// - The erase route earns its place: 0xFF is the erased state of NOR flash, so
///   `picotool erase` genuinely accomplishes the wipe with no image at all. The
///   pipe warning stays because its false-success shape is WORST on a
///   destruction op — a load that exits 0 having written nothing leaves the
///   operator believing a payload destroyed that is still in flash.
fn refuse_terminal_fill_image(len: usize) {
    use mnemonic_engrave::sysw::wire::{REGION_ADDR, REGION_LEN};
    eprintln!(
        "me: stdout is a TERMINAL, and this payload is a WIPE image, not a \
         secret.\n\
         \n\
         Writing it here would paint {len} bytes of raw binary across your \
         scrollback. Nothing was written.\n\
         \n\
         Give it a file, then flash that file:\n\
         \n\
           me sysw wipe --fill ... --out wipe.bin\n\
           picotool load --verify wipe.bin -t bin -o 0x{REGION_ADDR:08X}\n\
         \n\
         with the machine in BOOTSEL. Or wipe with no image at all:\n\
         \n\
           picotool erase -r 0x{REGION_ADDR:08X} 0x{:08X}\n\
         \n\
         Do NOT pipe into picotool: it sizes its input with fstat, a pipe \
         reports 0 bytes, and the load exits 0 having written nothing -- a \
         wipe that wiped nothing.",
        REGION_ADDR as usize + REGION_LEN
    );
}

fn refuse_world_readable_stdout(mode: u32) {
    eprintln!(
        "me: stdout is a file of mode {mode:04o} — its permissions grant read to \
         group or others — and this payload is BEARER.\n\
         \n\
         Only the file's OWN mode was checked (F-252). If a directory above it \
         denies search to others — a 0700 home directory does — nobody else can \
         open it today; the mode still becomes dangerous the moment the file is \
         moved, copied, or its parent relaxed.\n\
         \n\
         Anyone who can read that file can use what is in it. Three ways on:\n\
         \n\
           --out <FILE>              me creates it owner-only (0600)\n\
           umask 077                 then re-run, and the shell creates it 0600\n\
           --allow-world-readable    proceed anyway"
    );
}

/// Minimal standard base64 (no padding-free shortcuts); avoids a dep for one use.
fn base64_encode(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(T[(n >> 18 & 63) as usize] as char);
        out.push(T[(n >> 12 & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            T[(n >> 6 & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            T[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// `me sysw` — see design/SPEC_systemwide_payloads.md §5.6.
fn run_sysw(cmd: &SyswCmd) -> i32 {
    use mnemonic_engrave::sysw;
    match cmd {
        SyswCmd::Pack {
            records,
            r#in,
            out,
            passphrase_words,
            passphrase_ask,
            no_passphrase,
            allow_weak,
            allow_unsigned_inputs,
            expect,
            allow_world_readable,
            allow_argv_secret,
            iterations,
            region,
        } => {
            if *allow_weak {
                eprintln!(
                    "me: --allow-weak is accepted and ignored; a weak passphrase now warns \
                     rather than refusing (spec §13 D3)"
                );
            }

            let recs = match read_records(records, r#in.as_ref(), *allow_argv_secret) {
                Ok(r) => r,
                Err((msg, code)) => {
                    eprintln!("me: {msg}");
                    return code;
                }
            };

            // G-P3.3. Stated BEFORE the passphrase ceremony for the same
            // reason the ceremony itself is ordered that way: a warning the
            // operator reads after writing a passphrase down is a warning
            // about work already done.
            let admission = mnemonic_engrave::sysw::Admission {
                allow_unsigned_inputs: *allow_unsigned_inputs,
            };
            if *allow_unsigned_inputs {
                report_unsigned_overrides(&recs);
            }

            // §6g — `--expect`, and it runs HERE for F-246's reason: before the
            // passphrase ceremony, so an operator is never told to write down
            // twelve words that protect an artifact this refusal then declines
            // to build.
            //
            // It takes `admission`, and that parameter is the whole of probe
            // C-2: built without it, `--allow-unsigned-inputs --expect
            // transaction` refused at exit 4 saying NO record of that kind is in
            // the stream -- for a record the SAME invocation packs at exit 0
            // without `--expect`. A false refusal carrying a false message, on
            // the funds path, inside the feature added to prevent exactly that.
            if let Some(spec) = expect {
                let kinds = match sysw::expect::parse_kinds(spec) {
                    Ok(k) => k,
                    Err(e) => {
                        // A flag VALUE out of range is USAGE, not invalid: no
                        // input has been read at the point it is caught, so
                        // there is nothing yet for "invalid" to be about.
                        eprintln!("me: {e}");
                        return EXIT_USAGE;
                    }
                };
                let unmet = sysw::expect::check(&recs, &kinds, admission);
                if !unmet.is_empty() {
                    for u in &unmet {
                        eprintln!("me: {}", sysw::expect::describe(u));
                    }
                    return EXIT_INVALID;
                }
            }

            // F-246 — ADMISSION BEFORE THE CEREMONY.
            //
            // `pack_with` rejects an unplaceable record, but it runs AFTER the
            // passphrase has been generated, printed, and captioned "write this
            // down and store it APART from the machine". The operator who obeys
            // that instruction is left holding twelve words that protect no
            // artifact, immediately above an error saying the run failed.
            //
            // `admit_check` is the same rule `split` applies -- it was lifted
            // out of it, and `split` now calls it first -- so this is an
            // ordering change, not a second implementation of admission.
            if let Err(e) = sysw::admit_check(&recs, admission) {
                eprintln!("me: {}", sysw_error(&e));
                return EXIT_INVALID;
            }

            // F-246 — THE WRITE GATE RUNS BEFORE ANYTHING DESCRIBES A CONTAINER,
            // AND AFTER EVERY REFUSAL ABOUT THE INPUT ITSELF.
            //
            // `emit` checks this too, and that is where it used to be checked
            // ONLY. By then `sealing:`, `strength:`, `digest:` and "re-print it
            // with: me sysw show <the file you just wrote>" had all been
            // printed -- for a run that then exited 2 leaving a 0-byte file.
            // The digest is the value the operator verifies the PLATE against
            // on the device, so recording it means carrying a checksum for a
            // payload that does not exist, beneath a line that is false as it
            // prints.
            //
            // This is the rule the passphrase ceremony already follows a few
            // lines below -- "generating a passphrase, telling the operator to
            // write it down, and THEN refusing the container teaches them that
            // the note they just made is worthless". It simply had not been
            // applied to the gate that aborts the WRITE.
            //
            // The decision is `write_block`, shared with `emit`, so the two
            // cannot disagree about when a write is refused. `emit` keeps its
            // call: it is reached by `wipe` and by the region path too, and a
            // guard that exists only at one call site is one refactor from
            // being bypassed.
            //
            // POSITION IS LOAD-BEARING, and the first attempt got it wrong.
            // Placed above `read_records`, it PRE-EMPTED R2 -- the refusal for
            // a `tx:` record passed on ARGV, which is bearer material already
            // in the shell's history and in `ps`. That refusal is both more
            // urgent and more specific, and it exits 3 rather than 2. The
            // regenerated journey caught the swap; no test did.
            {
                use std::io::IsTerminal;
                // The length is not known yet -- the container has not been
                // built -- and it does not need to be: this refusal is about
                // WHERE, and the byte count only sharpens a message the
                // operator sees when they retry. `emit` reports the real
                // length; here 0 stands for "not built, and never will be".
                if let Some(code) = refuse_write_block(
                    write_block(
                        out.is_some(),
                        PayloadKind::Bearer,
                        *allow_world_readable,
                        std::io::stdout().is_terminal(),
                        stdout_world_readable_mode(),
                    ),
                    0,
                ) {
                    return code;
                }
            }
            // G-P3.6 / SPEC §2.4 — SEALING IS DECIDED BY CONTENT.
            //
            // This used to be `let sealing = !*no_passphrase;`: seal unless
            // told otherwise. Right for a mnemonic and wrong for a
            // transaction, whose whole purpose is to become a steel plate
            // anyone can read -- and sealing one costs the operator a 12-word
            // passphrase to store, those 12 words typed on the device's
            // on-screen keyboard, ~31 s of on-device KDF, and a new way to
            // lose the backup, to protect nothing.
            //
            // The flags still WIN when given: they are the operator saying
            // what they want, and the content default only decides when
            // nobody has. What §2.4 forbids is deciding SILENTLY.
            // P5 N-1 — BEFORE `decide_sealing`, which PRINTS. F-246's rule is
            // that no line describing a container may print until every gate
            // that can abort the write has run, and this was the one gate still
            // sitting behind it: `--iterations 5` printed `sealing: SEALED ...`
            // and then exited 2.
            //
            // The `sealing &&` guard came off with the move, and it costs
            // nothing: MIN_ITERATIONS is the clap default (100_000), so a value
            // outside the range is ALWAYS one the operator typed. Refusing a
            // mistyped flag on the unsealed path too is the honest reading --
            // silently ignoring it was never the intent.
            if !(sysw::wire::MIN_ITERATIONS..=sysw::wire::MAX_ITERATIONS).contains(iterations) {
                eprintln!(
                    "me: --iterations {iterations} is outside {}..={} — a container built with \
                     it is one no conforming reader will open",
                    sysw::wire::MIN_ITERATIONS,
                    sysw::wire::MAX_ITERATIONS
                );
                // USAGE, not invalid, and `me seal` has always said so
                // (`SealError::Iterations(_) => EXIT_USAGE`). Nothing has been
                // read yet; the operator mistyped a flag.
                return EXIT_USAGE;
            }

            let sealing = decide_sealing(&recs, *no_passphrase, *passphrase_ask, *passphrase_words);

            // Exactly one passphrase mode. clap enforces mutual exclusion; this
            // is the "none given" case, and the DEFAULT is to generate rather
            // than to leave a payload unprotected by omission.
            let generated;
            let passphrase: Option<String> = if !sealing {
                None
            } else if *passphrase_ask {
                match rpassword::prompt_password("passphrase: ") {
                    Ok(p) => Some(p),
                    Err(e) => {
                        eprintln!("me: reading the passphrase: {e}");
                        return EXIT_USAGE;
                    }
                }
            } else {
                let n = passphrase_words.unwrap_or(sysw::wire::WORDS_DEFAULT);
                match sysw::passphrase::generate(n) {
                    Ok(p) => {
                        generated = p;
                        eprintln!(
                            "passphrase — write this down and store it APART from the machine:"
                        );
                        eprintln!();
                        eprintln!("    {}", &*generated);
                        eprintln!();
                        Some((*generated).clone())
                    }
                    Err(e) => {
                        eprintln!("me: {e:?}");
                        return EXIT_USAGE;
                    }
                }
            };

            // The strength line is printed whatever the choice: the operator is
            // told, never blocked (spec decision 8).
            report_strength(passphrase.as_deref(), &recs);
            // `[mdmk-decode]` (§12.6). Indices are the OPERATOR'S — argv order,
            // or the order of `--in`'s lines — because that is the list they can
            // act on. Nothing here refuses: §13 D6 demoted the refusal, and a
            // single card of a chunked set is exactly what `me bundle`
            // legitimately produces.
            report_unconfirmed(&recs);

            let blob = match sysw::pack_with(recs, passphrase.as_deref(), *iterations, admission) {
                Ok(b) => b,
                Err(e) => {
                    // On the WRITE path "malformed container" is the wrong
                    // sentence — nothing is malformed, the operator simply gave
                    // more than a section can hold. Saying it the reader's way
                    // sends them looking for a corrupt file they do not have.
                    if e == sysw::SyswError::Wire(sysw::wire::WireError::SectionTooLong) {
                        eprintln!(
                            "me: these records are too long for one payload: a section caps at \
                             {} bytes. Split them across two payloads.",
                            sysw::wire::MAX_SECTION_LEN
                        );
                    } else {
                        eprintln!("me: {}", sysw_error(&e));
                    }
                    // The library's own out-of-range iteration check reports
                    // the same operator mistake as the flag guard above, so it
                    // gets the same code rather than the generic one.
                    if matches!(
                        e,
                        sysw::SyswError::Wire(sysw::wire::WireError::Iterations(_))
                    ) {
                        return EXIT_USAGE;
                    }
                    return EXIT_INVALID;
                }
            };
            // The digest is computed on the CONTAINER, before any padding. It
            // must be: `identity` bounds itself by the header's declared total,
            // so a padded region yields the same number, and the operator has to
            // see the same value whichever form they wrote.
            print_digest(&blob);
            // G-P3.16 / SPEC §3.2. The DEVICE tells the operator to compare
            // this number against `me sysw show <file>`, so `pack` names the
            // same command rather than leaving them to find it. Pointing back
            // at `pack` would be pointing at the WRITE path: re-running it
            // needs every record again and, on the sealed path, mints a fresh
            // passphrase. The operator standing at the machine has the file.
            match out.as_ref() {
                Some(p) => eprintln!("          re-print it with: me sysw show {}", p.display()),
                None => eprintln!(
                    "          re-print it with: me sysw show <the file you just wrote>"
                ),
            }
            if *region {
                let n = sysw::wire::REGION_LEN;
                if blob.len() > n {
                    eprintln!(
                        "me: container is {} bytes, larger than the {n}-byte region — \
                         it cannot be written to 0x{:08X}",
                        blob.len(),
                        sysw::wire::REGION_ADDR
                    );
                    return EXIT_INVALID;
                }
                let mut img = Zeroizing::new(vec![0xFFu8; n]);
                img[..blob.len()].copy_from_slice(&blob);
                eprintln!(
                    "me: region image — {} bytes of container, padded with 0xFF to {n}; \
                     write it at 0x{:08X}",
                    blob.len(),
                    sysw::wire::REGION_ADDR
                );
                return emit(&img, out.as_ref(), PayloadKind::Bearer, *allow_world_readable);
            }
            emit(&blob, out.as_ref(), PayloadKind::Bearer, *allow_world_readable)
        }

        SyswCmd::Wipe { out, fill } => {
            let f = match fill.as_str() {
                "random" => sysw::overwrite::Fill::Random,
                "zeros" => sysw::overwrite::Fill::Zeros,
                "ones" => sysw::overwrite::Fill::Ones,
                other => {
                    eprintln!("me: unknown --fill {other:?}; want random, zeros or ones");
                    return EXIT_USAGE;
                }
            };
            if f == sysw::overwrite::Fill::Ones {
                eprintln!(
                    "me: note — 0xFF is the ERASED state of NOR flash, so this region will be \
                     indistinguishable from one that was never written"
                );
            }
            // A wipe image is 65,536 bytes of random/zeros/ones with NOTHING
            // in it. Its purpose is to DESTROY a payload, so it is the opposite
            // of bearer, and the world-readable gate must not fire on it:
            // refusing it buys no safety and costs the operator a working
            // command. (Caught by asking what else F-244's new guard would
            // catch; there is a test.)
            //
            // **F-259: that fact now travels as a KIND, not as `true` in the
            // `allow_world_readable` seat.** Passing it there bought the right
            // answer from the one gate that reads that parameter and a FALSE
            // MESSAGE from the one that does not -- the terminal arm, which
            // called a zeros image BEARER. The flag's own value is `false`
            // here because `sysw wipe` declares no such flag; nothing about
            // this payload is the operator's permission problem.
            emit(
                &sysw::overwrite::region_image(f),
                out.as_ref(),
                PayloadKind::CarriesNoSecret,
                false,
            )
        }

        SyswCmd::Show { file } => {
            let blob = match std::fs::read(file) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("me: {}: {e}", file.display());
                    return EXIT_USAGE;
                }
            };
            let h = match sysw::wire::Header::parse(&blob) {
                Ok(h) => h,
                Err(e) => {
                    // NEVER the words "payload unreadable" — spec §5.2: that
                    // phrase teaches the operator to read a wrong file as
                    // tampering.
                    eprintln!("me: not a systemwide container: {e:?}");
                    return EXIT_INVALID;
                }
            };
            println!("sealed:   {}", h.sealed());
            println!("pub_len:  {}", h.pub_len);
            println!("ct_len:   {}", h.ct_len);

            // Truncation is decided BEFORE anything derived from the header is
            // printed. `identity` clamps to what is present, so on a short file
            // it yields a real-looking 32 bytes for a payload that does not
            // exist — and the operator would compare it against the machine.
            if blob.len() < h.total_len() {
                eprintln!(
                    "me: this file is {} bytes but its header declares {}; it is truncated. \
                     No identity or digest is shown, because either would be a number for a \
                     payload that is not all here.",
                    blob.len(),
                    h.total_len()
                );
                return EXIT_INVALID;
            }
            println!(
                "identity: {}",
                hex(&sysw::identity::identity(&blob[..h.total_len()]))
            );
            print_digest(&blob);
            print_mdmk_confirmation(&blob, &h);
            EXIT_OK
        }
    }
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// The digest goes to STDERR, so `me sysw pack > f.bin` still shows the operator
/// the number they must compare on the machine.
fn print_digest(blob: &[u8]) {
    use mnemonic_engrave::sysw;
    let Ok(h) = sysw::wire::Header::parse(blob) else {
        return;
    };
    if h.pub_len == 0 {
        // EPD §6.6: with no public section the digest is a constant every such
        // payload shares, so there is nothing to compare and none is shown.
        eprintln!("digest:   none — this payload has no public section");
        return;
    }
    // `pub_len` is the FILE's claim about itself, so a truncated container
    // declares more than it holds and this slice panicked (pre-flash review I2,
    // exit 101) — after `show` had already printed a plausible-looking identity.
    // A number the operator has read and cannot use is worse than no number.
    let end = sysw::wire::HEADER_LEN + h.pub_len as usize;
    let Some(section) = blob.get(sysw::wire::HEADER_LEN..end) else {
        eprintln!(
            "digest:   none — this file is {} bytes but its header declares {end}; it is \
             truncated, not tampered with",
            blob.len()
        );
        return;
    };
    let Ok(s) = std::str::from_utf8(section) else {
        return;
    };
    let refs: Vec<&str> = s.split('\n').collect();
    let d = sysw::pubhash::public_data_hash(&refs, h.sealed());
    eprintln!("digest:   {}", sysw::pubhash::format_hash(&d));
}

/// `[mdmk-decode]` (§12.6) at pack time: one line per unconfirmed record, then
/// the container is built anyway.
///
/// STDERR, like every other advisory line here, so `me sysw pack > f.bin` still
/// shows the operator what they are about to flash.
///
/// The index is into the records AS GIVEN, and it says so — because `me sysw
/// show` numbers the PUBLIC SECTION instead, and on a sealed payload the two
/// diverge: secrets move out of the public section, so argv record 3 can be
/// public record 1. Renumbering either would be worse than naming both.
fn report_unconfirmed(records: &[String]) {
    for i in mnemonic_engrave::sysw::record::mdmk_unconfirmed(records) {
        eprintln!(
            "me: record {i}, as given (records count from 0): an md1/mk1 this tool \
             could not decode; the device will treat it as a SECRET"
        );
    }
    // `[mt-decode]` — the same rule for mt1 chunk sets, but PER SET rather
    // than per record. G-P3.7: ruling 2026-08-25 makes "loudly" normative and
    // more than the md/mk sibling does, because the five ways an mt1 set fails
    // have five different remedies.
    for (csid, idxs, problem) in mnemonic_engrave::sysw::mt::set_problems(records) {
        let Some(p) = problem else { continue };
        eprintln!(
            "me: mt1 set {csid:05x} (records {}, as given; records count from 0) did NOT \n      \
             confirm as one signed transaction. {}",
            idxs.iter().map(usize::to_string).collect::<Vec<_>>().join(", "),
            describe_set_problem(&p)
        );
        eprintln!(
            "      It is PACKED and ENGRAVEABLE anyway (ruling 2026-08-25) -- every mt1 \n      \
             string is independently valid, so the strings you have are worth cutting \n      \
             and a missing one can be added later.\n      \
             The device will REPLACE your plate legend with a re-encode warning, and \n      \
             QR plates will be unavailable: a set that does not reassemble has no \n      \
             transaction bytes to encode."
        );
    }
}

/// One sentence per [`SetProblem`], naming the remedy — which is the whole
/// point of distinguishing them.
///
/// **Chunk numbers are 1-BASED here and everywhere an operator reads them**,
/// which is `mt`'s own convention (SPEC_mt §1.1: the wire index is 0-based and
/// appears in no message). Printing the wire index would send someone counting
/// the strings on their desk and finding the wrong one.
fn describe_set_problem(p: &mnemonic_engrave::sysw::mt::SetProblem) -> String {
    use mnemonic_engrave::sysw::mt::SetProblem as P;
    match p {
        P::Missing { count, missing } => {
            let names: Vec<String> = missing.iter().map(|i| (i + 1).to_string()).collect();
            format!(
                "MISSING string{} {} of {count}. Pack every string of the set -- \n      \
                 `mt encode` emits them all, and `--elide-prefix` output is refused here.",
                if names.len() == 1 { "" } else { "s" },
                match names.len() {
                    1 => names[0].clone(),
                    _ => format!(
                        "{} and {}",
                        names[..names.len() - 1].join(", "),
                        names[names.len() - 1]
                    ),
                }
            )
        }
        P::DoesNotReassemble => "Every string of the set is here and they still do not \n      \
             reassemble -- duplicates that disagree, or a chunk the codec reads as \n      \
             ambiguous. Re-encode the transaction with `mt encode`."
            .into(),
        P::NotATransaction => "The set is COMPLETE and reassembles, and the bytes are NOT \n      \
             one serialized Bitcoin transaction. Whatever produced these strings did not \n      \
             encode a transaction; re-encode from the signed transaction itself."
            .into(),
        P::TxidDoesNotBind { txid, csid } => format!(
            "The set is complete and parses, and the transaction's txid is {txid}, whose \n      \
             top 20 bits are {:05x} -- not the {csid:05x} every string declares. These \n      \
             strings were not made from this transaction.",
            u32::from_str_radix(&txid[..5], 16).unwrap_or(0)
        ),
        P::UnsignedInputs { txid, inputs } => format!(
            "The set is complete, reassembles, parses and binds to its set id -- and {} \n      \
             carr{} NEITHER a scriptSig NOR a witness. Nothing else here can see that: \n      \
             stripping the signatures leaves the txid ({txid}) unchanged, which is \n      \
             precisely what the txid is defined to ignore. Re-export the FINALIZED \n      \
             transaction from your signer.",
            name_inputs(inputs),
            if inputs.len() == 1 { "ies" } else { "y" }
        ),
    }
}

/// "input 1" / "inputs 1 and 3" / "inputs 0, 2 and 5" — the operator reads
/// this, so it is prose rather than a Debug-formatted Vec.
fn name_inputs(idx: &[usize]) -> String {
    let n: Vec<String> = idx.iter().map(usize::to_string).collect();
    match n.len() {
        0 => "no input".into(),
        1 => format!("input {}", n[0]),
        _ => format!("inputs {} and {}", n[..n.len() - 1].join(", "), n[n.len() - 1]),
    }
}

/// `--allow-unsigned-inputs` (G-P3.3): one loud line per record the override
/// actually admitted, naming the failing inputs.
///
/// SILENT when nothing needed it. A flag that shouts on every payload trains
/// the operator to ignore the one payload where it matters — and this one
/// matters: the artifact it lets through has the txid of a transaction that
/// can never be broadcast.
fn report_unsigned_overrides(records: &[String]) {
    use mnemonic_engrave::sysw;
    for (i, r) in records.iter().enumerate() {
        if !r.starts_with(sysw::record::TX_PREFIX) {
            continue;
        }
        let Ok(body) = sysw::record::decode_body(r) else {
            continue;
        };
        let Ok(t) = sysw::tx::parse(&body) else {
            continue;
        };
        if t.every_input_signed {
            continue;
        }
        eprintln!(
            "me: WARNING — record {i}, as given (records count from 0): {} of this \n      \
             transaction carr{} neither a scriptSig nor a witness. ADMITTED because you \n      \
             passed --allow-unsigned-inputs.\n      \
             txid {} — the SAME txid a fully signed version would have, because \n      \
             stripping signatures is exactly what the txid ignores. If those inputs are \n      \
             not honestly empty, the plate you are about to cut can never be broadcast.",
            name_inputs(&t.unsigned_inputs),
            if t.unsigned_inputs.len() == 1 { "ies" } else { "y" },
            t.txid_display,
        );
    }
}

/// `me sysw show`: the same rule, stated per record, so the operator can see
/// which cards the machine will treat as secrets before anything is flashed.
///
/// Indices are into the PUBLIC SECTION, which is the list `show` can see — and
/// the only one that matters, since `Class::MdMk` is not secret and so never
/// reaches the ciphertext. A sealed payload's secret records stay unread here;
/// `show` has no passphrase.
fn print_mdmk_confirmation(blob: &[u8], h: &mnemonic_engrave::sysw::wire::Header) {
    use mnemonic_engrave::sysw;
    if h.pub_len == 0 {
        return;
    }
    let end = sysw::wire::HEADER_LEN + h.pub_len as usize;
    let Some(section) = blob.get(sysw::wire::HEADER_LEN..end) else {
        return;
    };
    let Ok(s) = std::str::from_utf8(section) else {
        return;
    };
    let records: Vec<String> = s.split('\n').map(str::to_owned).collect();
    let unconfirmed = sysw::record::mdmk_unconfirmed(&records);
    for (i, r) in records.iter().enumerate() {
        if sysw::classify(r) != sysw::record::Class::MdMk {
            continue;
        }
        let state = if unconfirmed.contains(&i) {
            "unconfirmed — engraveable, but the device REPLACES the legend"
        } else {
            "confirmed"
        };
        println!("public record {i}: md1/mk1 — {state}");
    }
    print_mt_confirmation(&records);
}

/// `[mt-decode]` in `show`: per mt1 record, whether its chunk set confirmed —
/// and per confirmed SET, the transaction it carries, because the txid is what
/// the operator can check against `mt encode`'s own report.
fn print_mt_confirmation(records: &[String]) {
    use mnemonic_engrave::sysw;
    let unconfirmed = sysw::mt::mt_unconfirmed(records);
    for (i, r) in records.iter().enumerate() {
        if sysw::classify(r) != sysw::record::Class::Mt {
            continue;
        }
        let state = if unconfirmed.contains(&i) {
            "unconfirmed — engraveable, but the device REPLACES the legend"
        } else {
            "confirmed"
        };
        println!("public record {i}: mt1 chunk — {state}");
    }
    for (i, r) in records.iter().enumerate() {
        // Keyed on the PREFIX, not on `classify`. `classify` is strict, so a
        // record admitted by `--allow-unsigned-inputs` reads back as
        // `Class::Unknown` -- and `show` listing nothing at all for a record
        // the container demonstrably holds is worse than either verdict.
        // A reader may disagree with the writer; it may not go quiet.
        if !r.starts_with(sysw::record::TX_PREFIX) {
            continue;
        }
        let Ok(b) = sysw::record::decode_body(r) else {
            println!("public record {i}: `tx:` record whose body is not lowercase hex");
            continue;
        };
        let Ok(t) = sysw::tx::parse(&b) else {
            println!("public record {i}: `tx:` record whose body is not a transaction");
            continue;
        };
        if t.every_input_signed {
            println!(
                "public record {i}: raw signed transaction — txid {}, {} bytes",
                t.txid_display, t.size
            );
        } else {
            println!(
                "public record {i}: raw transaction with UNSIGNED input(s) — txid {}, {} \
                 bytes; {} carr{} neither a scriptSig nor a witness, so a plate cut from \
                 this can never be broadcast. It was packed with --allow-unsigned-inputs.",
                t.txid_display,
                t.size,
                name_inputs(&t.unsigned_inputs),
                if t.unsigned_inputs.len() == 1 { "ies" } else { "y" },
            );
        }
    }
    // PER SET, and the unconfirmed ones say WHY -- ruling 2026-08-25 requires
    // `show` to carry the same diagnosis `pack` printed, because a stderr line
    // is gone in a week and this is the one an operator can re-run.
    for (csid, idxs, problem) in sysw::mt::set_problems(records) {
        let set: Vec<String> = idxs.iter().map(|&i| records[i].clone()).collect();
        match problem {
            None => {
                let Some((_, t)) = sysw::mt::decode_confirmed(&set) else {
                    continue;
                };
                println!(
                    "  mt set {csid:05x}: txid {} — {} bytes, {} input(s), {} output(s), \
                     {} string(s)",
                    t.txid_display,
                    t.size,
                    t.inputs,
                    t.outputs,
                    set.len()
                );
            }
            Some(p) => println!(
                "  mt set {csid:05x}: INCOMPLETE — {} string(s) present. {}",
                set.len(),
                describe_set_problem(&p)
            ),
        }
    }
}

/// SPEC §2.4: **seal iff the payload holds a `Class::is_secret()` record.**
///
/// Content decides, and the flags choose only HOW — `--passphrase-ask` to
/// supply one, `--passphrase-words N` for the generated length,
/// `--no-passphrase` to keep secret material in the clear (§13 D6 permits it;
/// F1 flags it at load).
///
/// **`--passphrase-ask`/`--passphrase-words` CANNOT seal a payload with
/// nothing secret in it, and this function refuses to pretend otherwise.**
/// Measured before this gate, and the defect predates it: `me sysw pack
/// --passphrase-words 4 <md1>` generated a passphrase, told the operator to
/// write it down and store it apart from the machine — and emitted
/// `sealed: false, ct_len: 0`. `pack` moves only SECRET records into the
/// ciphertext, so with none there the plaintext is empty, `sealed()` is
/// `ct_len > 0`, and the container is cleartext with a 16-byte AEAD tag
/// stranded past `total_len()`. The passphrase protected nothing and opened
/// nothing. So the flag is reported IGNORED, loudly, and no passphrase is
/// minted for an operator to keep forever.
///
/// **It says which way it went and why, on stderr, every time.** A
/// content-dependent default that is silent is worse than the default it
/// replaces: the operator cannot tell a deliberately-cleartext container from
/// a flag they forgot to pass.
///
/// The class names are the CLASSES, never the records: naming a `pass:`
/// record's body here would put a passphrase on stderr.
fn decide_sealing(
    records: &[String],
    no_passphrase: bool,
    passphrase_ask: bool,
    passphrase_words: Option<usize>,
) -> bool {
    use mnemonic_engrave::sysw;
    let secret: Vec<String> = records
        .iter()
        .enumerate()
        .filter(|(_, r)| sysw::classify(r).is_secret())
        .map(|(i, r)| format!("record {i} ({})", class_name(sysw::classify(r))))
        .collect();
    let asked = if passphrase_ask {
        Some("--passphrase-ask")
    } else if passphrase_words.is_some() {
        Some("--passphrase-words")
    } else {
        None
    };
    if secret.is_empty() {
        // THE CASE §2.4 EXISTS FOR, and the only honest verdict: there is
        // nothing this container would encrypt.
        eprint!(
            "sealing:  NOT SEALED — no record in this payload is secret material, so there \n      \
             is nothing to encrypt. The container is cleartext: anyone holding the file \n      \
             can read it."
        );
        match asked {
            Some(flag) => eprintln!(
                "\n      \
                 {flag} is IGNORED here, deliberately. `pack` encrypts only secret-class \n      \
                 records, so a passphrase would have opened nothing and protected nothing \n      \
                 — and you would have had to keep it forever."
            ),
            None => eprintln!(),
        }
        return false;
    }
    if no_passphrase {
        eprintln!(
            "sealing:  NOT SEALED — you passed --no-passphrase, and this payload HOLDS \n      \
             SECRET MATERIAL ({}). It will sit in flash in cleartext.",
            secret.join(", ")
        );
        return false;
    }
    eprintln!(
        "sealing:  SEALED — this payload holds secret material ({}), so it is encrypted \n      \
         and opens only with the passphrase{}. Pass --no-passphrase to write it in \n      \
         cleartext instead.",
        secret.join(", "),
        match asked {
            Some(flag) => format!(" you chose with {flag}"),
            None => " below".into(),
        }
    );
    true
}

/// A record CLASS, for an operator. Names the class, never the record.
fn class_name(c: mnemonic_engrave::sysw::record::Class) -> &'static str {
    use mnemonic_engrave::sysw::record::Class as C;
    match c {
        C::Mnemonic => "BIP-39 mnemonic",
        C::Codex32Secret => "codex32 secret",
        C::Passphrase => "passphrase",
        C::FreeText => "free text",
        C::Descriptor => "descriptor",
        C::MdMk => "md1/mk1 card",
        C::Mt => "mt1 chunk",
        C::Tx => "raw transaction",
        C::Address => "address",
        C::Unknown => "unrecognised record",
    }
}

fn report_strength(passphrase: Option<&str>, records: &[String]) {
    use mnemonic_engrave::sysw;
    let secret = records.iter().any(|r| sysw::classify(r).is_secret());
    let (desc, above) = match passphrase {
        None => ("no passphrase".to_string(), false),
        Some(p) => {
            let n = mnemonic_engrave::seal::passphrase::normalise(p);
            let above = sysw::cliff_above(&n);
            let words = n.split_whitespace().count();
            (format!("{words} words"), above)
        }
    };
    eprintln!(
        "strength: {desc} — {}",
        if above {
            "at or above the threshold"
        } else {
            "BELOW the threshold"
        }
    );
    if secret && !above {
        eprintln!(
            "me: WARNING — this payload carries secret material with weak or no passphrase \
             protection. Proceeding (spec §13 D3)."
        );
    }
}

/// `mt encode … | me sysw pack …` — a join an early draft claimed already
/// worked and which measurably did not.
fn read_records(
    argv: &[String],
    in_path: Option<&std::path::PathBuf>,
    allow_argv_secret: bool,
) -> Result<Vec<String>, (String, i32)> {
    if let Some(p) = in_path {
        let raw = std::fs::read_to_string(p)
            .map_err(|e| (format!("{}: {e}", p.display()), EXIT_USAGE))?;
        // R7, and it applies to THIS channel too -- see `no_records`.
        return no_records_guard(split_record_stream(&raw), Some(p)).map_err(|e| (e, EXIT_USAGE));
    }
    if !argv.is_empty() {
        // R2 / G-P3.5. THIS RUNS BEFORE ANYTHING ELSE `pack` DOES, and that is
        // the whole gate: a guard placed downstream of the work it exists to
        // prevent has already lost. Before it, `me sysw pack tx:<hex>` wrote
        // the container to stdout at exit 0 -- and on the DEFAULT path it also
        // generated a passphrase and told the operator to write it down first.
        //
        // argv is a PUBLIC channel: /proc/<pid>/cmdline is world-readable
        // without hidepid, `ps` shows it to every user on the box, and the
        // shell records it in a history file that outlives the machine. A raw
        // signed transaction is a BEARER instrument -- whoever reads it can
        // broadcast it -- so "prefer --in" is not enough. `mt` already refuses
        // a transaction as an argument for exactly this reason.
        //
        // Matched on the TRIMMED, case-folded prefix rather than through
        // `classify`, deliberately: a near-miss like ` TX:<hex>` is then
        // refused here for the BEARER reason rather than three screens later
        // for a formatting one. Neither message may name the body.
        for (i, r) in argv.iter().enumerate() {
            use mnemonic_engrave::sysw::record::Class;
            // F-270: TRIM AND CASE-FOLD, and feed the result to BOTH arms.
            // This used to be `trim_start` only, and only the `tx:` prefix arm
            // consumed it -- `classify` received the RAW token -- so the
            // near-miss protection this gate's own comment describes existed
            // for one class of five. Measured: an uppercase `MS1…` was refused
            // as "not a form this container can place" at rc 4 rather than as
            // SECRET key material at rc 3, pointing the operator at
            // `sysw::classify` instead of at purging their history.
            let trimmed = r.trim().to_ascii_lowercase();

            // The `tx:` PREFIX check stays, and stays FIRST, for the reason
            // above: a near-miss like ` TX:<hex>` is refused here for the
            // BEARER reason rather than three screens later for a formatting
            // one. `classify` would call that shape `Unknown`.
            let by_prefix = trimmed.starts_with(mnemonic_engrave::sysw::record::TX_PREFIX);
            if allow_argv_secret {
                // The operator has said where they are. A single-user
                // air-gapped box or an amnesic Tails session has no other
                // observer and no persistence, and refusing there is friction
                // that teaches people to alias the flag on permanently -- the
                // failure mode that retired R3.
                continue;
            }

            // P5 I-1 — AND THE CLASS CHECK, because the prefix covered ONE of
            // five. Measured 2026-08-26: this gate refused a `tx:` record and
            // accepted, at exit 0 in silence, an `ms1` string (seed entropy), a
            // raw BIP-39 mnemonic, a `pass:` record, and the same transaction
            // carried as `mt1` strings. It refused a TRANSACTION while accepting
            // a SEED PHRASE.
            //
            // Keyed on the CLASS, not on a list of prefixes, so a class added
            // later is covered by `is_argv_forbidden` rather than by whoever
            // remembers to extend a match arm here.
            let class = mnemonic_engrave::sysw::classify(&trimmed);
            if by_prefix || class.is_argv_forbidden() {
                // NAME THE CLASS, NEVER THE BODY. Printing it back would put the
                // material in a SECOND public place -- the defect this refusal
                // exists to name.
                // The private-channel EXAMPLE must match the class. Showing
                // `mt encode --qr | me sysw pack` to someone who pasted a seed
                // phrase is advice for a different artifact entirely.
                let example = if by_prefix || class.is_bearer() {
                    "    mt encode --qr --in tx.hex | me sysw pack --out p.bin"
                } else {
                    // `ms encode --in` DOES NOT EXIST (exit 64) -- caught by the
                    // R0 fold. `--phrase -` is ms's shipped stdin idiom and is
                    // verified to pipe into pack. Advice for a flag that is not
                    // there is worse than no advice.
                    "    ms encode --phrase - < seed.txt | me sysw pack --out p.bin"
                };
                // F-264: the purge recipes are BUILT, not spelled inline, so a
                // test can run the emitted one rather than a copy of it -- and
                // so `history -d` can be NAMED in the warning while appearing
                // in no recipe. See `io::remedy`.
                let purge = mnemonic_engrave::io::remedy::history_purge_block("me sysw pack");
                let (what, why) = if by_prefix || class == Class::Tx {
                    ("a `tx:` record", "A raw signed transaction is a BEARER instrument -- anyone who can read it can broadcast it")
                } else if class == Class::Mt {
                    ("an `mt1` string", "An mt1 set carries a signed transaction -- anyone who can read the set can broadcast it")
                } else {
                    ("SECRET key material", "It can spend everything derived from it, forever")
                };
                return Err((
                    format!(
                        "record {i}, as given (records count from 0), is {what} on \
                         ARGV. Refused; nothing was read and nothing was written.\n      \
                         {why} -- and argv is public: /proc, `ps` and \
                         your shell history all keep a copy.\n      \
                         Use a private channel instead:\n      \
                         {example}\n      \
                         \x20   me sysw pack --in records.txt --out p.bin\n\n      \
                         {purge}\n      \
                         If argv is safe where you are -- a single-user \
                         air-gapped box, an amnesic Tails session -- \
                         --allow-argv-secret proceeds."
                    ),
                    // POLICY REFUSAL, not usage: the record is understood and
                    // well-formed, and this tool will never accept it here.
                    EXIT_REFUSED,
                ));
            }
        }
        // §6b — `-` MEANS STDIN, AND IT IS IMPLEMENTED RATHER THAN MERELY
        // ACCEPTED. §6b's wording is permissive enough that an implementation
        // could take the flag and do nothing with it, and the compliant
        // implementation is then SILENTLY LOSSY: `… | me sysw pack --out b.bin
        // - text:6869` packed ONE record instead of two, at exit 0, on the
        // artifact that gets cut into metal. (Measured before this: exit 4,
        // nothing written -- `-` was read as a record and refused as
        // unclassifiable.)
        //
        // `sysw pack` is the ONLY surface that gains anything, because it is
        // the only one with a positional RECORD list. `me` and `bundle`
        // already default to stdin with no positional at all, `sysw show`'s
        // positional is a container FILE path, and `sysw wipe` has no
        // positional -- so `-` stays a clap or ENOENT error on all four, and
        // that is asserted rather than assumed.
        let dashes = argv.iter().filter(|r| r.as_str() == "-").count();
        if dashes > 1 {
            return Err((
                format!(
                    "`-` appears {dashes} times, and stdin can only be read once.\n      \
                     Pass it once, or put the records in a file and use --in."
                ),
                EXIT_USAGE,
            ));
        }
        if dashes == 1 {
            let from_stdin = read_stdin_records()?;
            // R7 again, and for the same reason: the operator ASKED for stdin,
            // so nothing arriving there is the failed-upstream signal, not an
            // instruction to pack the rest. Splicing zero records at exit 0
            // would build a container missing exactly what the pipeline was
            // supposed to supply.
            if from_stdin.is_empty() {
                return Err((
                    no_records_guard(from_stdin, None).unwrap_err(),
                    EXIT_USAGE,
                ));
            }
            // Spliced IN PLACE. A record's position is the operator's, and
            // appending stdin at the end would silently reorder the container
            // relative to the command they typed.
            let mut out = Vec::with_capacity(argv.len() + from_stdin.len() - 1);
            for r in argv {
                if r == "-" {
                    out.extend(from_stdin.iter().cloned());
                } else {
                    out.push(r.clone());
                }
            }
            return Ok(out);
        }
        return Ok(argv.to_vec());
    }
    // R7 for the stdin channel, through the same guard `--in` uses.
    no_records_guard(read_stdin_records()?, None).map_err(|e| (e, EXIT_USAGE))
}

/// Read the record stream from stdin, unguarded.
///
/// Extracted so the `-` splice and the no-argv default read stdin THE SAME WAY
/// — same tty note, same error, same splitting. Two readers of one channel is
/// how `--in` and stdin came to disagree about an empty input in the first
/// place.
fn read_stdin_records() -> Result<Vec<String>, (String, i32)> {
    // A TTY here is the "looks like a hang" case: an operator who typed
    // `me sysw pack` and pressed Enter with nothing piped in otherwise sees a
    // blank line forever. Say what is being waited for, on stderr, before
    // blocking. (`mt` was measured doing exactly this and reading as a hang.)
    {
        use std::io::IsTerminal;
        if std::io::stdin().is_terminal() {
            eprintln!(
                "me: reading records from stdin, one per line — end with Ctrl-D. \
                 (Or pass them with --in FILE, or on argv.)"
            );
        }
    }
    let mut raw = String::new();
    std::io::stdin()
        .read_to_string(&mut raw)
        .map_err(|e| (format!("stdin: {e}"), EXIT_USAGE))?;
    Ok(split_record_stream(&raw))
}

/// **`kind` is its own parameter, and F-259 is why.** It used to travel in the
/// `allow_world_readable` seat: `wipe` passed `WIPE_IMAGE_CARRIES_NO_SECRET:
/// bool = true` there, meaning "this holds nothing", while the flag means "this
/// file's permissions are my problem". The mode gate consulted it and behaved;
/// the terminal gate never looks at that parameter, so it refused a zeros image
/// as BEARER. **One `bool` that two callers read differently.**
fn emit(
    bytes: &[u8],
    out: Option<&std::path::PathBuf>,
    kind: PayloadKind,
    allow_world_readable: bool,
) -> i32 {
    use std::io::{IsTerminal, Write};
    // F-244: this used to be `std::fs::write`, which creates at 0o666 & ~umask
    // -- so an UNSEALED container holding a BIP-39 mnemonic landed at 0644 while
    // `write_private` sat in this same file, documented for exactly this threat,
    // and used only for NDEF/manifest/uf2. The container was less protected than
    // the artifact that merely depicts key material.
    // F-253 — a TERMINAL is not a destination for a bearer container. This runs
    // BEFORE the mode check because the mode check cannot reach it: a TTY is a
    // character device, and character devices are exempt there (that exemption
    // is load-bearing for `/dev/null`, which is mode 0666).
    //
    // `--allow-world-readable` does NOT override this. It is the operator saying
    // "this file's permissions are my problem"; it is not a request to paint the
    // container across their scrollback, and there is a file-shaped route two
    // lines away in the message.
    if let Some(code) = refuse_write_block(
        write_block(
            out.is_some(),
            kind,
            allow_world_readable,
            std::io::stdout().is_terminal(),
            stdout_world_readable_mode(),
        ),
        bytes.len(),
    ) {
        return code;
    }
    let r = match out {
        Some(p) => write_private(p, bytes).map_err(|e| format!("{}: {e}", p.display())),
        None => std::io::stdout()
            .write_all(bytes)
            .map_err(|e| format!("stdout: {e}")),
    };
    match r {
        Ok(()) => EXIT_OK,
        Err(e) => {
            // A write that failed is the environment, not the artifact: a
            // read-only directory, a full disk, a closed pipe. USAGE.
            eprintln!("me: {e}");
            EXIT_USAGE
        }
    }
}

fn sysw_error(e: &mnemonic_engrave::sysw::SyswError) -> String {
    use mnemonic_engrave::sysw::SyswError as E;
    match e {
        // Two situations, two remedies, and they are not close: one is "hex
        // your body", the other is "this tool cannot place that at all". The
        // first version of this said only the second, so the operator whose
        // `pass:` body was plain text was told about descriptors and addresses.
        // Neither branch prints the record — a `pass:` body is a passphrase.
        E::Unclassifiable(i, why) => {
            use mnemonic_engrave::sysw::UnknownReason as U;
            match why {
                U::NonHexBody(prefix) => format!(
                    "record {i} (records count from 0) begins `{prefix}`, but its body is \
                     not lowercase hex. That prefix is RESERVED, so a body it cannot \
                     decode is refused rather than quietly engraved as free text \
                     (§5.3.1). Encode the body first:\n      \
                     printf '%s' 'your text here' | xxd -p -c 256"
                ),
                U::NotATransaction(e) => format!(
                    "record {i} (records count from 0) begins `tx:` and its body is hex, \
                     but the bytes are not one serialized Bitcoin transaction ({e}). The \
                     prefix is RESERVED for a raw signed transaction — produce the record \
                     with `mt encode --qr` rather than by hand"
                ),
                U::UnsignedInputs(idx) => format!(
                    "record {i} (records count from 0) is a `tx:` record whose transaction \
                     parses but whose {} carries NEITHER a scriptSig NOR a witness — it is \
                     unsigned, or its signatures were stripped in transit.\n      \
                     This is refused because the txid does NOT change when signatures are \
                     removed: the record would show the txid you expect, and the plate cut \
                     from it could never be broadcast.\n      \
                     Re-export the FINALIZED transaction from your signer.\n      \
                     If those inputs are honestly empty (a P2A anchor spend and similar \
                     exotica), pass --allow-unsigned-inputs.",
                    name_inputs(idx)
                ),
                U::Unrecognised => format!(
                    "record {i} (records count from 0) is not a form this container can \
                     place: not a BIP-39 mnemonic, not an md1/mk1/ms1/mt1 string, and not \
                     a `text:`/`pass:`/`tx:` record. Descriptors and addresses are not \
                     yet classifiable here — see sysw::classify"
                ),
            }
        }
        E::TooLarge(n) => format!("{n} bytes exceeds the flash region"),
        E::PassphraseMismatch => "a sealed payload needs a passphrase".into(),
        E::NotEnterableOnDevice(w) => format!(
            "the passphrase contains {w:?}, which is not a BIP-39 word. The device \
             offers only a word keyboard, so a payload sealed with this could never \
             be opened on it. Use BIP-39 words (2 or more), or --no-passphrase."
        ),
        E::PassphraseTooLong(n) => format!(
            "the normalised passphrase is {n} bytes; the limit is {} (§12.5)",
            mnemonic_engrave::sysw::wire::PASSPHRASE_MAX
        ),
        E::EmptyPassphrase => "an empty passphrase would seal a payload the DEVICE can never \
                               open: it reads an empty passphrase as none supplied, and there \
                               is no keystroke for it. Use --no-passphrase for a plaintext \
                               payload, or supply a real one."
            .into(),
        E::Crypto => "the passphrase did not open this payload".into(),
        E::NotUtf8 => "the records are not valid UTF-8".into(),
        E::Wire(w) => format!("malformed container: {w:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{is_plate_artifact, write_block, PayloadKind, WriteBlock};
    // `destination` and `Destination` are the library half's now, and only
    // this test module consults them directly.
    use mnemonic_engrave::io::{destination, Destination};

    // A1/F8: is_plate_artifact must match the tool's own plate names and the
    // `plate-.svg` edge (a plate-artifact form), but never over-match near-misses.
    #[test]
    fn is_plate_artifact_classifies_near_miss_set() {
        // Matches (a plate artifact this tool writes, or the accepted edge form).
        assert!(is_plate_artifact("plate-2.svg"));
        assert!(is_plate_artifact("plate-1.png"));
        assert!(is_plate_artifact("plate-.svg")); // edge: accept — it IS the form.
                                                  // Near-misses that must NOT match.
        assert!(!is_plate_artifact("notes.txt")); // no prefix, no ext.
        assert!(!is_plate_artifact("plate.txt")); // no `plate-`, wrong ext.
        assert!(!is_plate_artifact("plateau.svg")); // no `-` after `plate`.
    }

    // ── F-253: a bearer container must not be dumped at a terminal ───────────

    #[test]
    fn a_terminal_is_never_a_destination_for_the_container() {
        // --out wins over everything: `me` creates the file 0600 itself.
        assert_eq!(destination(true, true), Destination::File);
        assert_eq!(destination(true, false), Destination::File);
        // No --out, and stdout is a TTY: REFUSE. The bytes would land in
        // scrollback and in any logged session -- this finding was captured
        // through `script`, which is exactly such a session.
        assert_eq!(destination(false, true), Destination::Terminal);
        // No --out, stdout redirected or piped: write. This is the ruled
        // pipeline (`mt encode --qr | me sysw pack`) and must not regress.
        assert_eq!(destination(false, false), Destination::Stream);
    }

    /// **F-270 — the post-parse argv gate normalises for EVERY class, not just
    /// for its `tx:` prefix arm.**
    ///
    /// It built a normalised copy of each record and fed it ONLY to the prefix
    /// check; `classify` received the RAW token, and `classify` itself neither
    /// trims nor case-folds. So the near-miss protection the gate's own comment
    /// describes — refuse *"for the BEARER reason rather than three screens
    /// later for a formatting one"* — existed for one class of five. Measured
    /// before the fix: an uppercase `MS1…` was refused as *not a form this
    /// container can place* at **rc 4**, pointing the operator at
    /// `sysw::classify` rather than at purging their history.
    ///
    /// **Asserted HERE, as a unit test, and that is deliberate.** P0's
    /// pre-parser guard now refuses these shapes before `Cli::parse()`, so no
    /// end-to-end invocation can reach this arm with a near-miss any more — an
    /// integration test would be measuring the guard and calling it this. The
    /// gate is defence in depth; it has to be tested where it is. Verified by
    /// mutation: putting the RAW token back into `classify` turns this RED.
    #[test]
    fn the_post_parse_argv_gate_normalises_for_every_class() {
        const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

        for (what, rec) in [
            ("canonical", MS1.to_string()),
            ("UPPERCASE", MS1.to_uppercase()),
            ("leading space", format!("  {MS1}")),
            ("trailing space", format!("{MS1}  ")),
        ] {
            let (msg, code) = super::read_records(std::slice::from_ref(&rec), None, false)
                .expect_err(&format!("{what} must be refused"));
            assert_eq!(
                code,
                super::EXIT_REFUSED,
                "{what}: a POLICY refusal (3), not an invalid-input one (4) --                  the record is understood and this tool will never take it here"
            );
            assert!(
                msg.contains("SECRET key material"),
                "{what}: refused for the right reason, not a formatting one: {msg}"
            );
            assert!(
                !msg.contains(&rec.trim()[..24]),
                "{what}: the body must never be echoed back: {msg}"
            );
        }

        // The control: a record that is genuinely nothing is NOT refused here.
        // Without it, `Err` for every input would satisfy the loop above.
        assert!(
            super::read_records(&["text:6869".to_string()], None, false).is_ok(),
            "a legitimate text: record must pass this gate"
        );
    }

    /// The COMBINED decision, exhaustively — both gates live here so the early
    /// F-246 check and `emit`'s cannot drift apart, and drift is exactly what
    /// this table pins.
    #[test]
    fn write_block_decides_both_gates_once() {
        use PayloadKind::{Bearer, CarriesNoSecret};
        use WriteBlock as W;

        // --out: me creates the file 0600, so NEITHER gate applies -- not even
        // with a world-readable stdout behind it, which is the whole point of
        // recommending --out as the remedy.
        assert_eq!(write_block(true, Bearer, false, true, Some(0o644)), W::None);
        assert_eq!(
            write_block(true, Bearer, false, false, Some(0o644)),
            W::None
        );

        // A terminal is refused, and --allow-world-readable does NOT buy past
        // it: that flag is about a FILE's permissions.
        assert_eq!(
            write_block(false, Bearer, false, true, None),
            W::Terminal(Bearer)
        );
        assert_eq!(
            write_block(false, Bearer, true, true, None),
            W::Terminal(Bearer)
        );

        // ── F-259 — THE ROW THIS TEST USED TO BE MISSING ────────────────────
        //
        // The old version of this test asserted only the two rows above and
        // argued, correctly, that the FLAG must not buy past a terminal. It
        // never contemplated the second fact riding that same bool, so it read
        // as deliberate while locking the defect in: `wipe` passed
        // `WIPE_IMAGE_CARRIES_NO_SECRET: bool = true` in the flag's seat, this
        // arm discarded it, and a 65,536-byte zeros image was refused as
        // BEARER.
        //
        // A fill image is STILL refused at a terminal -- the refusal is about
        // the scrollback, not about exposure -- but the decision now carries
        // WHICH payload it refused, so the message can be derived from it.
        assert_eq!(
            write_block(false, CarriesNoSecret, false, true, None),
            W::Terminal(CarriesNoSecret),
            "the refusal stays; what changes is that the kind survives it"
        );

        // A stream with a group/other-readable mode is refused, and the mode is
        // carried so the message can quote what it measured.
        assert_eq!(
            write_block(false, Bearer, false, false, Some(0o644)),
            W::WorldReadable(0o644)
        );
        // ...unless the operator overrode it. Here the flag DOES apply.
        assert_eq!(
            write_block(false, Bearer, true, false, Some(0o644)),
            W::None
        );
        // ...or unless there is nothing to expose. This is the outcome `wipe`
        // used to buy by passing `true` in the flag's seat -- same answer, now
        // reached through the fact that actually justifies it, and reached
        // WITHOUT claiming the operator waived anything.
        assert_eq!(
            write_block(false, CarriesNoSecret, false, false, Some(0o644)),
            W::None
        );

        // The ruled pipeline: a pipe, no mode concern. Must never be blocked.
        assert_eq!(write_block(false, Bearer, false, false, None), W::None);
    }
}
