//! `me` — convert a single md1/mk1 string to an NDEF payload (refuses ms1).

use std::io::{Read, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use mnemonic_engrave::{convert, ConvertError};
use zeroize::Zeroizing;

/// Convert a public constellation string (md1/mk1) to an NFC NDEF payload.
/// Reads the string from stdin (or --in). Refuses secret ms1.
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

    /// Build, inspect or overwrite a SYSTEMWIDE payload.
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
    /// Build a systemwide container.
    ///
    /// A record is a BIP-39 mnemonic, an md1/mk1/ms1 string, or one of the two
    /// prefixed forms — and those two carry their bodies as LOWERCASE HEX.
    ///
    /// `text:<hex of the UTF-8 bytes>` is free text.
    ///
    /// `pass:<hex of the UTF-8 bytes>` is a BIP-39 passphrase.
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
        /// --in for anything real.
        ///
        /// `text:`/`pass:` bodies are lowercase hex — see the command help.
        records: Vec<String>,
        /// Read newline-separated records from this file instead of argv.
        ///
        /// Blank lines are skipped, so a record's index is its position among
        /// the NON-blank lines, not its line number.
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
        /// Proceed even though stdout is a world-readable file (F-244).
        ///
        /// `me` refuses by default: a container is BEARER, and `>` creates a
        /// file at 0644 under the usual umask. Prefer `--out`, which `me`
        /// creates owner-only.
        #[arg(long)]
        allow_world_readable: bool,
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

const EXIT_OK: i32 = 0;
const EXIT_USAGE: i32 = 2;
const EXIT_REFUSED: i32 = 3;
const EXIT_INVALID: i32 = 4;

fn main() {
    std::process::exit(run());
}

fn run() -> i32 {
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
        if !cli.allow_world_readable && stdout_is_world_readable() {
            refuse_world_readable_stdout();
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

/// Is this process's stdout a REGULAR FILE that others can read?
///
/// F-244. `me sysw pack ... > payload.bin` hands the container to a file `me`
/// never names -- but a process can `fstat` its own stdout, so the mode is
/// visible even when the path is not.
///
/// **Only `S_ISREG` counts.** A pipe (`me ... | picotool`) has no meaningful
/// mode and a terminal persists nothing, so both must pass; firing on them would
/// break every pipeline in the constellation. That is the near-miss this check
/// is most likely to get wrong, and there are tests for both.
#[cfg(unix)]
fn stdout_is_world_readable() -> bool {
    use std::mem::ManuallyDrop;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::io::FromRawFd;
    // ManuallyDrop: fd 1 belongs to the process, and dropping the File would
    // CLOSE stdout out from under everything downstream.
    let f = unsafe { ManuallyDrop::new(std::fs::File::from_raw_fd(1)) };
    match f.metadata() {
        Ok(md) => md.file_type().is_file() && md.permissions().mode() & 0o044 != 0,
        // Unreadable stdout is not evidence of exposure; fail OPEN rather than
        // refusing a write for a reason we cannot state.
        Err(_) => false,
    }
}

#[cfg(not(unix))]
fn stdout_is_world_readable() -> bool {
    false
}

/// The refusal F-244 asks for, with the override it names.
fn refuse_world_readable_stdout() {
    eprintln!(
        "me: stdout is a world-readable file, and this payload is BEARER.\n\
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
            allow_world_readable,
            iterations,
            region,
        } => {
            if *allow_weak {
                eprintln!(
                    "me: --allow-weak is accepted and ignored; a weak passphrase now warns \
                     rather than refusing (spec §13 D3)"
                );
            }
            let recs = match read_records(records, r#in.as_ref()) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("me: {e}");
                    return 2;
                }
            };

            // Before the passphrase ceremony, not after: generating a passphrase,
            // telling the operator to write it down, and THEN refusing the
            // container teaches them that the note they just made is worthless.
            let sealing = !*no_passphrase;
            if sealing
                && !(sysw::wire::MIN_ITERATIONS..=sysw::wire::MAX_ITERATIONS).contains(iterations)
            {
                eprintln!(
                    "me: --iterations {iterations} is outside {}..={} — a container built with \
                     it is one no conforming reader will open",
                    sysw::wire::MIN_ITERATIONS,
                    sysw::wire::MAX_ITERATIONS
                );
                return 4;
            }

            // Exactly one passphrase mode. clap enforces mutual exclusion; this
            // is the "none given" case, and the DEFAULT is to generate rather
            // than to leave a payload unprotected by omission.
            let generated;
            let passphrase: Option<String> = if *no_passphrase {
                None
            } else if *passphrase_ask {
                match rpassword::prompt_password("passphrase: ") {
                    Ok(p) => Some(p),
                    Err(e) => {
                        eprintln!("me: reading the passphrase: {e}");
                        return 2;
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
                        return 2;
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

            let blob = match sysw::pack(recs, passphrase.as_deref(), *iterations) {
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
                    return 4;
                }
            };
            // The digest is computed on the CONTAINER, before any padding. It
            // must be: `identity` bounds itself by the header's declared total,
            // so a padded region yields the same number, and the operator has to
            // see the same value whichever form they wrote.
            print_digest(&blob);
            if *region {
                let n = sysw::wire::REGION_LEN;
                if blob.len() > n {
                    eprintln!(
                        "me: container is {} bytes, larger than the {n}-byte region — \
                         it cannot be written to 0x{:08X}",
                        blob.len(),
                        sysw::wire::REGION_ADDR
                    );
                    return 4;
                }
                let mut img = Zeroizing::new(vec![0xFFu8; n]);
                img[..blob.len()].copy_from_slice(&blob);
                eprintln!(
                    "me: region image — {} bytes of container, padded with 0xFF to {n}; \
                     write it at 0x{:08X}",
                    blob.len(),
                    sysw::wire::REGION_ADDR
                );
                return emit(&img, out.as_ref(), *allow_world_readable);
            }
            emit(&blob, out.as_ref(), *allow_world_readable)
        }

        SyswCmd::Wipe { out, fill } => {
            let f = match fill.as_str() {
                "random" => sysw::overwrite::Fill::Random,
                "zeros" => sysw::overwrite::Fill::Zeros,
                "ones" => sysw::overwrite::Fill::Ones,
                other => {
                    eprintln!("me: unknown --fill {other:?}; want random, zeros or ones");
                    return 2;
                }
            };
            if f == sysw::overwrite::Fill::Ones {
                eprintln!(
                    "me: note — 0xFF is the ERASED state of NOR flash, so this region will be \
                     indistinguishable from one that was never written"
                );
            }
            // NOT GATED, and deliberately. `emit` is shared, so F-244's guard
            // reached `wipe` too -- and a wipe image is 65,536 bytes of
            // random/zeros/ones with NOTHING in it. Its purpose is to DESTROY a
            // payload, so it is the opposite of bearer: refusing it buys no
            // safety and costs the operator a working command. Caught by asking
            // what else the new guard would catch; there is a test.
            const WIPE_IMAGE_CARRIES_NO_SECRET: bool = true;
            emit(
                &sysw::overwrite::region_image(f),
                out.as_ref(),
                WIPE_IMAGE_CARRIES_NO_SECRET,
            )
        }

        SyswCmd::Show { file } => {
            let blob = match std::fs::read(file) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("me: {}: {e}", file.display());
                    return 2;
                }
            };
            let h = match sysw::wire::Header::parse(&blob) {
                Ok(h) => h,
                Err(e) => {
                    // NEVER the words "payload unreadable" — spec §5.2: that
                    // phrase teaches the operator to read a wrong file as
                    // tampering.
                    eprintln!("me: not a systemwide container: {e:?}");
                    return 4;
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
                return 4;
            }
            println!(
                "identity: {}",
                hex(&sysw::identity::identity(&blob[..h.total_len()]))
            );
            print_digest(&blob);
            print_mdmk_confirmation(&blob, &h);
            0
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
            "unconfirmed — the device will treat it as a SECRET"
        } else {
            "confirmed"
        };
        println!("public record {i}: md1/mk1 — {state}");
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

fn read_records(
    argv: &[String],
    in_path: Option<&std::path::PathBuf>,
) -> Result<Vec<String>, String> {
    if let Some(p) = in_path {
        let raw = std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
        return Ok(raw
            .lines()
            .map(str::to_owned)
            .filter(|l| !l.is_empty())
            .collect());
    }
    if argv.is_empty() {
        return Err("no records: pass them on argv or with --in".into());
    }
    Ok(argv.to_vec())
}

fn emit(bytes: &[u8], out: Option<&std::path::PathBuf>, allow_world_readable: bool) -> i32 {
    use std::io::Write;
    // F-244: this used to be `std::fs::write`, which creates at 0o666 & ~umask
    // -- so an UNSEALED container holding a BIP-39 mnemonic landed at 0644 while
    // `write_private` sat in this same file, documented for exactly this threat,
    // and used only for NDEF/manifest/uf2. The container was less protected than
    // the artifact that merely depicts key material.
    if out.is_none() && !allow_world_readable && stdout_is_world_readable() {
        refuse_world_readable_stdout();
        return EXIT_USAGE;
    }
    let r = match out {
        Some(p) => write_private(p, bytes).map_err(|e| format!("{}: {e}", p.display())),
        None => std::io::stdout()
            .write_all(bytes)
            .map_err(|e| format!("stdout: {e}")),
    };
    match r {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("me: {e}");
            2
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
                U::Unrecognised => format!(
                    "record {i} (records count from 0) is not a form this container can \
                     place: not a BIP-39 mnemonic, not an md1/mk1/ms1 string, and not a \
                     `text:`/`pass:` record. Descriptors and addresses are not yet \
                     classifiable here — see sysw::classify"
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
    use super::is_plate_artifact;

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
}
