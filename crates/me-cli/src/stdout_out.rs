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
//! - a read-only stdout (S1d) is error EBADF at the first write;
//! - a CLOSED stdout (`>&-`) is the same EBADF. std's runtime opens
//!   `/dev/null` read-write on a closed fd 0-2 before `main`, so without
//!   `closed_fds` the output vanished into it at exit 0.

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

/// Exit code of a failed write (ruling T6).
pub const EXIT_WRITE: i32 = 1;

/// Set once a closed pipe was seen: later stdout writes do nothing.
static MUTED: AtomicBool = AtomicBool::new(false);

/// Set before `main` when fd 1 was closed at exec: every stdout write is
/// then EBADF.
static STDOUT_CLOSED: AtomicBool = AtomicBool::new(false);

/// A closed fd 0, 1 or 2 at exec (`me ... >&-`). std's runtime opens
/// `/dev/null` read-write on each closed standard fd before `main`, so by
/// `main` a closed stdout looks writable and its output was lost at exit 0.
/// This runs earlier, from the executable's initializer list (as ms-cli's
/// `sink.rs` and mt-cli's `out.rs` do): it records a closed fd 1 and plugs
/// each closed fd with `/dev/null` itself (the lowest free fd, so it lands on
/// that number and stays open for the whole run), so no file `me` opens later
/// can become stdout. fd 1 is plugged READ-ONLY, so an `--out /dev/stdout`
/// alias is refused with EBADF too. A target without such a list keeps std's
/// plug only.
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "dragonfly",
    target_os = "macos"
))]
mod closed_fds {
    use std::sync::atomic::Ordering;

    extern "C" fn probe() {
        for fd in 0..=2 {
            // SAFETY: fcntl(F_GETFD) and open() of a NUL-terminated literal
            // take no other pointer; a bad fd or failed open returns -1.
            unsafe {
                if libc::fcntl(fd, libc::F_GETFD) != -1
                    || std::io::Error::last_os_error().raw_os_error() != Some(libc::EBADF)
                {
                    continue;
                }
                let mode = if fd == 1 {
                    super::STDOUT_CLOSED.store(true, Ordering::Relaxed);
                    libc::O_RDONLY
                } else {
                    libc::O_RDWR
                };
                let n = libc::open(c"/dev/null".as_ptr(), mode);
                if n >= 0 && n != fd {
                    libc::close(n);
                }
            }
        }
    }

    #[used]
    #[cfg_attr(not(target_os = "macos"), link_section = ".init_array")]
    #[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
    static PROBE: extern "C" fn() = probe;
}

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
    if STDOUT_CLOSED.load(Ordering::Relaxed) || read_only_stdout() {
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

/// S1d: stdout opened with neither `O_WRONLY` nor `O_RDWR`. A closed fd 1 is
/// caught by `closed_fds` instead.
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
