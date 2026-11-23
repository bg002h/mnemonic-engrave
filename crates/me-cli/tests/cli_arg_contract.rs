//! FIX_me_cli_arg_contract (bh-me-I3, bh-me-I11; ruling T32): a converter option
//! typed before a subcommand is honoured or refused, never dropped; a path
//! printed inside a command is one shell word. Each test is a row of the gate
//! script `scripts/fix-me-cli-arg-contract-gate.sh` (section A) and runs the
//! built binary with a fresh tempdir as its current directory, so the printed
//! paths are the relative names the rows pin.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output, Stdio};

const MD1: &str = "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3";
const MD1B: &str = "md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374";
const MD1C: &str = "md1fv9wjpqsp2026hh65xpvugtfhd9792zxgunymm0a82pdju6442q0jskj9gzfaqmz";
const MS1: &str = "ms10entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw";

const ME: &str = env!("CARGO_BIN_EXE_me");

fn me_in(dir: &Path, args: &[&str], stdin: &[u8]) -> Output {
    use std::io::Write;
    let mut child = Command::new(ME)
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn me");
    let mut si = child.stdin.take().unwrap();
    let _ = si.write_all(stdin);
    drop(si);
    child.wait_with_output().expect("wait me")
}

fn me(dir: &Path, args: &[&str]) -> Output {
    me_in(dir, args, b"")
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn code(o: &Output) -> i32 {
    o.status.code().expect("exit code")
}

/// A tempdir holding `given` (text:6162), `other` (text:7172), `md1.txt`, `recs.txt`
/// and the two reference containers.
fn setup() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    fs::write(p.join("given"), "text:6162\n").unwrap();
    fs::write(p.join("other"), "text:7172\n").unwrap();
    fs::write(p.join("md1.txt"), format!("{MD1}\n")).unwrap();
    fs::write(p.join("recs.txt"), format!("{MD1}\n{MD1B}\n{MD1C}\n")).unwrap();
    for r in ["6162", "6869"] {
        let o = me(
            p,
            &[
                "sysw",
                "pack",
                "--no-passphrase",
                "--out",
                &format!("ref{r}.bin"),
                &format!("text:{r}"),
            ],
        );
        assert_eq!(code(&o), 0, "reference pack {r}: {}", err(&o));
    }
    d
}

fn conflict(opt: &str, leaf: &str) -> String {
    format!(
        "me: --{opt} was given before '{leaf}' and again after it, with a different value; give it once, after '{leaf}'\n"
    )
}

fn notake(opt: &str, leaf: &str) -> String {
    format!(
        "me: --{opt} is a converter option (me with no subcommand); '{leaf}' does not take it\n"
    )
}

#[test]
fn bh_me_i3_root_in_honoured_by_sysw_pack() {
    let d = setup();
    let o = me_in(
        d.path(),
        &[
            "--in",
            "given",
            "sysw",
            "pack",
            "--no-passphrase",
            "--out",
            "a1.bin",
        ],
        b"text:6364\n",
    );
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(
        fs::read(d.path().join("a1.bin")).unwrap(),
        fs::read(d.path().join("ref6162.bin")).unwrap()
    );
}

#[test]
fn bh_me_i3_root_out_honoured_by_sysw_pack() {
    let d = setup();
    let o = me(
        d.path(),
        &[
            "--out",
            "a2.bin",
            "sysw",
            "pack",
            "--no-passphrase",
            "text:6869",
        ],
    );
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(
        fs::read(d.path().join("a2.bin")).unwrap(),
        fs::read(d.path().join("ref6869.bin")).unwrap()
    );
    assert!(o.stdout.is_empty());
}

#[test]
fn bh_me_i3_root_in_conflict_refused_by_sysw_pack() {
    let d = setup();
    let o = me(
        d.path(),
        &[
            "--in",
            "given",
            "sysw",
            "pack",
            "--in",
            "other",
            "--no-passphrase",
            "--out",
            "a3.bin",
        ],
    );
    assert_eq!(code(&o), 2);
    assert_eq!(err(&o), conflict("in", "sysw pack"));
    assert!(!d.path().join("a3.bin").exists());
}

#[test]
fn bh_me_i3_root_in_same_value_honoured() {
    let d = setup();
    let o = me(
        d.path(),
        &[
            "--in",
            "given",
            "sysw",
            "pack",
            "--in",
            "given",
            "--no-passphrase",
            "--out",
            "a4.bin",
        ],
    );
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(
        fs::read(d.path().join("a4.bin")).unwrap(),
        fs::read(d.path().join("ref6162.bin")).unwrap()
    );
}

#[test]
fn bh_me_i3_root_converter_option_refused_by_sysw_pack() {
    let d = setup();
    let o = me(
        d.path(),
        &[
            "--hex",
            "sysw",
            "pack",
            "--no-passphrase",
            "--out",
            "a5.bin",
            "text:6162",
        ],
    );
    assert_eq!(code(&o), 2);
    assert_eq!(err(&o), notake("hex", "sysw pack"));
    assert!(!d.path().join("a5.bin").exists());
}

#[test]
fn bh_me_i3_first_misplaced_option_is_named() {
    let d = setup();
    // A5b: the first refused option in command-line order is the one named.
    let o = me(
        d.path(),
        &["--echo", "--in", "given", "bundle", "--in", "other"],
    );
    assert_eq!(code(&o), 2);
    assert_eq!(err(&o), notake("echo", "bundle"));
    // A5c: a conflict that comes before a not-taken option is the one named.
    let o = me(
        d.path(),
        &["--in", "given", "--echo", "bundle", "--in", "other"],
    );
    assert_eq!(code(&o), 2);
    assert_eq!(err(&o), conflict("in", "bundle"));
}

fn seal_load_line(file: &str) -> String {
    format!("load:  picotool load --verify {file}   (machine in BOOTSEL)")
}

#[test]
fn bh_me_i3_root_out_honoured_by_seal() {
    let d = setup();
    let o = me(
        d.path(),
        &[
            "--out",
            "a6.uf2",
            "seal",
            "--plaintext",
            MD1,
            "--plaintext",
            MD1B,
            "--plaintext",
            MD1C,
        ],
    );
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert!(d.path().join("a6.uf2").exists());
    assert!(err(&o).lines().any(|l| l == seal_load_line("a6.uf2")));
}

#[test]
fn bh_me_i3_root_in_honoured_by_seal() {
    let d = setup();
    let o = me(d.path(), &["--in", "recs.txt", "seal", "--out", "a6b.uf2"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert!(d.path().join("a6b.uf2").exists());
    assert!(err(&o).lines().any(|l| l == seal_load_line("a6b.uf2")));
}

#[test]
fn bh_me_i3_root_out_honoured_by_sysw_wipe() {
    let d = setup();
    let r = me(
        d.path(),
        &["sysw", "wipe", "--fill", "zeros", "--out", "a6c.ref"],
    );
    assert_eq!(code(&r), 0, "{}", err(&r));
    let o = me(
        d.path(),
        &["--out", "a6c.bin", "sysw", "wipe", "--fill", "zeros"],
    );
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(
        fs::read(d.path().join("a6c.bin")).unwrap(),
        fs::read(d.path().join("a6c.ref")).unwrap()
    );
    assert!(o.stdout.is_empty());
}

#[test]
fn bh_me_i3_root_in_conflict_refused_by_bundle() {
    let d = setup();
    let o = me(d.path(), &["--in", "given", "bundle", "--in", "other"]);
    assert_eq!(code(&o), 2);
    assert_eq!(err(&o), conflict("in", "bundle"));
}

#[test]
fn bh_me_i3_root_allow_world_readable_honoured_by_sysw_pack() {
    let d = setup();
    let f = fs::File::create(d.path().join("a8.bin")).unwrap();
    fs::set_permissions(d.path().join("a8.bin"), fs::Permissions::from_mode(0o644)).unwrap();
    let o = Command::new(ME)
        .args([
            "--allow-world-readable",
            "sysw",
            "pack",
            "--no-passphrase",
            "text:6162",
        ])
        .current_dir(d.path())
        .stdin(Stdio::null())
        .stdout(Stdio::from(f))
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(
        fs::read(d.path().join("a8.bin")).unwrap(),
        fs::read(d.path().join("ref6162.bin")).unwrap()
    );
}

#[test]
fn bh_me_i3_controls_guard_seal_text_converter_unchanged() {
    let d = setup();
    // A9: the argv guard still runs first with a converter option before `seal`.
    let o = me(
        d.path(),
        &["--out", "a9.uf2", "seal", "--allow-argv-secret", MS1],
    );
    assert_eq!(code(&o), 3);
    assert!(err(&o)
        .lines()
        .next()
        .unwrap()
        .starts_with("me: argument 5 on ARGV"));
    assert!(!d.path().join("a9.uf2").exists());
    // A10: `--out` in neither place keeps clap's text byte for byte.
    let o = me(d.path(), &["seal", "--plaintext", MD1]);
    assert_eq!(code(&o), 2);
    assert_eq!(
        err(&o),
        "error: the following required arguments were not provided:\n  --out <OUT>\n\nUsage: me seal --out <OUT> --plaintext <PLAINTEXT> [PAYLOAD]...\n\nFor more information, try '--help'.\n"
    );
    // A11: the converter with no subcommand is unchanged.
    let o = me(d.path(), &["--in", "md1.txt", "--hex"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    let hex: String = String::from_utf8_lossy(&o.stdout)
        .chars()
        .filter(|c| *c != '\n')
        .collect();
    assert_eq!(hex.len(), 150);
}

#[test]
fn bh_me_i11_printed_show_command_runs_as_printed() {
    let d = setup();
    let bin_dir = Path::new(ME).parent().unwrap().to_owned();
    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    for (name, want) in [
        ("my copy.bin", "me sysw show 'my copy.bin'"),
        ("it's.bin", "me sysw show 'it'\\''s.bin'"),
        ("-dash.bin", "me sysw show ./-dash.bin"),
    ] {
        let o = me(
            d.path(),
            &[
                "sysw",
                "pack",
                "--no-passphrase",
                &format!("--out={name}"),
                "text:6162",
            ],
        );
        assert_eq!(code(&o), 0, "{name}: {}", err(&o));
        let e = err(&o);
        let got = e
            .lines()
            .find_map(|l| l.strip_prefix("          re-print it with: "))
            .unwrap_or_else(|| panic!("{name}: no re-print line in {e}"));
        assert_eq!(got, want, "{name}");
        let s = Command::new("sh")
            .args(["-c", got])
            .env("PATH", &path)
            .current_dir(d.path())
            .output()
            .unwrap();
        assert_eq!(s.status.code(), Some(0), "{name}: {}", err(&s));
        let digest = |t: &str| {
            t.lines()
                .find(|l| l.starts_with("digest:"))
                .map(str::to_owned)
        };
        let shown = String::from_utf8_lossy(&s.stdout).into_owned() + &err(&s);
        assert!(digest(&e).is_some(), "{name}");
        assert_eq!(digest(&shown), digest(&e), "{name}");
    }
}

#[test]
fn bh_me_i11_printed_show_plain_path_unchanged() {
    let d = setup();
    let o = me(
        d.path(),
        &[
            "sysw",
            "pack",
            "--no-passphrase",
            "--out",
            "plain.bin",
            "text:6162",
        ],
    );
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert!(err(&o)
        .lines()
        .any(|l| l == "          re-print it with: me sysw show plain.bin"));
}

#[test]
fn bh_me_i11_printed_picotool_path_is_one_shell_word() {
    let d = setup();
    let o = me(
        d.path(),
        &[
            "seal",
            "--plaintext",
            MD1,
            "--plaintext",
            MD1B,
            "--plaintext",
            MD1C,
            "--out=my copy.uf2",
        ],
    );
    assert_eq!(code(&o), 0, "{}", err(&o));
    let e = err(&o);
    let line = e
        .lines()
        .find(|l| l.starts_with("load:"))
        .expect("load line");
    assert_eq!(line, seal_load_line("'my copy.uf2'"));
    let word = line
        .strip_prefix("load:  picotool load --verify ")
        .and_then(|s| s.strip_suffix("   (machine in BOOTSEL)"))
        .unwrap();
    let w = Command::new("sh")
        .args(["-c", &format!("set -- {word}; echo $#")])
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&w.stdout).trim(), "1");
}
