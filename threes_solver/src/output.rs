//! Output module that provides "tee" functionality - writing to both stdout/stderr and a log file.
//!
//! When a log file is initialized, ALL stdout and stderr output (including from dependencies)
//! will be written to both the terminal and the log file (in append mode).
//!
//! This works by redirecting stdout/stderr file descriptors to pipes, then spawning
//! background threads that read from the pipes and write to both destinations.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use filedescriptor::{FileDescriptor, Pipe, StdioDescriptor};

/// Guard for the tee output system.
/// When dropped, flushes all pending output and waits for tee threads to complete.
pub struct TeeGuard {
    stdout_handle: Option<JoinHandle<()>>,
    stderr_handle: Option<JoinHandle<()>>,
    original_stdout: Option<FileDescriptor>,
    original_stderr: Option<FileDescriptor>,
}

impl Drop for TeeGuard {
    fn drop(&mut self) {
        // Flush stdout and stderr to push data through the pipes
        let _ = io::stdout().flush();
        let _ = io::stderr().flush();

        // Restore original stdout/stderr - this closes the pipe write ends
        // which signals EOF to the tee threads
        if let Some(fd) = self.original_stdout.take() {
            let _ = FileDescriptor::redirect_stdio(&fd, StdioDescriptor::Stdout);
        }
        if let Some(fd) = self.original_stderr.take() {
            let _ = FileDescriptor::redirect_stdio(&fd, StdioDescriptor::Stderr);
        }

        // Wait for tee threads to finish processing remaining data
        if let Some(handle) = self.stdout_handle.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.stderr_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Spawn a tee thread for one stdio stream.
fn spawn_tee_thread(
    mut pipe_read: FileDescriptor,
    mut original_fd: FileDescriptor,
    log_file: Arc<Mutex<File>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        loop {
            match pipe_read.read(&mut buffer) {
                Ok(0) => break, // EOF - pipe closed
                Ok(n) => {
                    let data = &buffer[..n];

                    // Write to original fd (terminal)
                    let _ = original_fd.write_all(data);
                    let _ = original_fd.flush();

                    // Write to log file
                    if let Ok(mut file) = log_file.lock() {
                        let _ = file.write_all(data);
                        let _ = file.flush();
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
    })
}

/// Initialize tee output to a file. All subsequent stdout and stderr output will be written
/// to both the terminal and the specified file (in append mode).
///
/// Returns a guard that must be held until the end of main() to ensure all output is captured.
/// When the guard is dropped, it will flush all pending output and wait for the tee threads.
pub fn init_log_file<P: AsRef<Path>>(path: P) -> io::Result<TeeGuard> {
    let mut log_file = OpenOptions::new().create(true).append(true).open(path)?;

    // Write a separator before each run
    log_file.write_all(b"\n---\n")?;
    log_file.flush()?;

    let log_file = Arc::new(Mutex::new(log_file));

    // Set up stdout tee
    let stdout_pipe = Pipe::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let original_stdout =
        FileDescriptor::redirect_stdio(&stdout_pipe.write, StdioDescriptor::Stdout)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let stdout_handle = spawn_tee_thread(
        stdout_pipe.read,
        original_stdout
            .try_clone()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?,
        Arc::clone(&log_file),
    );

    // Set up stderr tee
    let stderr_pipe = Pipe::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let original_stderr =
        FileDescriptor::redirect_stdio(&stderr_pipe.write, StdioDescriptor::Stderr)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let stderr_handle = spawn_tee_thread(
        stderr_pipe.read,
        original_stderr
            .try_clone()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?,
        Arc::clone(&log_file),
    );

    Ok(TeeGuard {
        stdout_handle: Some(stdout_handle),
        stderr_handle: Some(stderr_handle),
        original_stdout: Some(original_stdout),
        original_stderr: Some(original_stderr),
    })
}
