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

    /// Encrypt a payload for delivery to SeedHammer II flash.
    ///
    /// The passphrase is GENERATED and printed to STDERR — write it down and
    /// store it apart from the machine. There is deliberately no way to supply
    /// your own: total strength is the passphrase plus about 20 bits from the
    /// KDF, and a memorable passphrase does not survive an offline attack on a
    /// stolen machine.
    Seal {
        /// Records to ENCRYPT. Must be canonical: if they came from
        /// `mnemonic bundle`, use --group-size 0.
        payload: Vec<String>,

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

        /// PBKDF2 iterations. 230,000 = ~31 s on device, from the IN-SITU
        /// 7,463 iters/sec measured 2026-08-09 on the real machine through the
        /// unlock flow (Task 8.1a). §7.1's 9,715 it/s is a RAW PBKDF2 figure and
        /// does not hold in situ: the KDF runs 500 iterations per frame, so
        /// 300,000 is 600 frames and cost 40.2 s +/- 1 measured -- about 15.5 ms
        /// per frame of GUI overhead on top of the raw work.
        #[arg(long, default_value_t = 230_000)]
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

    if let Some(Command::Seal {
        payload,
        plaintext,
        out,
        seal_secret,
        iterations,
    }) = &cli.command
    {
        return run_seal_cli(payload, plaintext, out, *seal_secret, *iterations);
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
    } else if cli.hex {
        let s: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        println!("{s}");
    } else if cli.base64 {
        println!("{}", base64_encode(&bytes));
    } else if cli.stdout {
        if std::io::stdout().write_all(&bytes).is_err() {
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
    plaintext: &[String],
    out: &std::path::Path,
    seal_secret: bool,
    iterations: u32,
) -> i32 {
    use mnemonic_engrave::classify::{classify, Format};
    use mnemonic_engrave::seal::{self, pubhash, Payload};

    // These records are NOT zeroized, per the Global Constraint. §9 puts them on
    // argv, so /proc/$PID/cmdline already exposes them — the heap copy is not the
    // binding exposure, and claiming a wipe here would be a guarantee this code
    // does not deliver.
    // Do NOT trim here. `encode_section` scans the UNTRIMMED record for `\r`
    // (§6.4: "no CR anywhere"), and trimming first strips a leading/trailing CR
    // before that check ever sees it — the CLI would normalise where §9 says
    // refuse. `encode_section` trims internally, once, after the CR scan.
    let secret: Vec<String> = payload.to_vec();
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
    f.write_all(bytes)
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
