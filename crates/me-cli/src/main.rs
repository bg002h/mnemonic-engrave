//! `me` — convert a single md1/mk1 string to an NDEF payload (refuses ms1).

use std::io::{Read, Write};
use std::path::PathBuf;

use clap::Parser;
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

    // Capture the plate-budget flag and (with --echo) the validated string
    // before the input is dropped.
    let too_long = mnemonic_engrave::exceeds_plate_budget(&input);
    let echo_line = if cli.echo {
        let s = input.trim();
        let label = if s.starts_with("mk1") { "mk1" } else { "md1" };
        Some(format!("me: validated {label}: {s}"))
    } else {
        None
    };

    let result = convert(&input);
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
    if let Some(line) = echo_line {
        eprintln!("{line}");
    }

    // Emit per the selected output mode. Human guidance -> stderr only.
    if let Some(path) = &cli.out {
        if let Err(e) = std::fs::write(path, &bytes) {
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
