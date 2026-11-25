//! FIX_me_output_integrity S1: the one writer for every byte `me` puts on stdout.
//!
//! It writes, then flushes, and checks both results (std's stdout is line
//! buffered, drops the flush error at exit, and reports a read-only descriptor
//! as a successful write). The first failed write ends the verb there (S1a):
//!
//! - a closed pipe (`BrokenPipe`, S1b) prints nothing more: stdout is muted and
//!   stderr is pointed at `/dev/null`, and the verb runs on to its own exit
//!   code (0 for a verb that succeeded);
//! - any other error (S1c) prints `me: cannot write the output: <error>` as the
//!   last stderr line and exits 1;
//! - a read-only stdout (S1d) is error EBADF at the first write.

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

/// Exit code of a failed write (ruling T6).
pub const EXIT_WRITE: i32 = 1;

/// Set once a closed pipe was seen: later stdout writes do nothing.
static MUTED: AtomicBool = AtomicBool::new(false);

/// Write `bytes` to stdout, flush, and handle a failure as S1a-S1d say.
pub fn write(bytes: &[u8]) {
    if MUTED.load(Ordering::Relaxed) {
        return;
    }
    if let Err(e) = try_write(bytes) {
        fail(&e);
    }
}

fn try_write(bytes: &[u8]) -> std::io::Result<()> {
    if read_only_stdout() {
        return Err(std::io::Error::from_raw_os_error(9));
    }
    let mut out = std::io::stdout().lock();
    out.write_all(bytes)?;
    out.flush()
}

fn fail(e: &std::io::Error) {
    if e.kind() == std::io::ErrorKind::BrokenPipe {
        MUTED.store(true, Ordering::Relaxed);
        silence_stderr();
        return;
    }
    eprintln!("me: cannot write the output: {e}");
    std::process::exit(EXIT_WRITE);
}

/// S1d: stdout opened with neither `O_WRONLY` nor `O_RDWR`. A closed fd 1
/// (`fcntl` fails) is not detected, as in md's `out.rs`.
#[cfg(unix)]
fn read_only_stdout() -> bool {
    // SAFETY: fcntl(F_GETFL) on a descriptor number reads a flag word only.
    let fl = unsafe { libc::fcntl(1, libc::F_GETFL) };
    fl >= 0 && (fl & libc::O_ACCMODE) != libc::O_WRONLY && (fl & libc::O_ACCMODE) != libc::O_RDWR
}

#[cfg(not(unix))]
fn read_only_stdout() -> bool {
    false
}

/// S1b: after a closed pipe the verb prints nothing more.
#[cfg(unix)]
fn silence_stderr() {
    use std::os::fd::AsRawFd;
    if let Ok(null) = std::fs::OpenOptions::new().write(true).open("/dev/null") {
        // SAFETY: both descriptors are open; dup2 replaces fd 2 only.
        unsafe {
            libc::dup2(null.as_raw_fd(), 2);
        }
    }
}

#[cfg(not(unix))]
fn silence_stderr() {}
