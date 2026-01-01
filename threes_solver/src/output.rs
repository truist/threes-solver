//! Output module that provides "tee" functionality - writing to both stdout and a log file.
//!
//! When a log file is initialized, ALL stdout output (including from dependencies)
//! will be written to both the terminal and the log file (in append mode).
//!
//! This works by redirecting the stdout file descriptor to a pipe, then spawning
//! a background thread that reads from the pipe and writes to both destinations.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::io::{FromRawFd, RawFd};
use std::path::Path;
use std::thread;

pub fn init_log_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let mut log_file = OpenOptions::new().create(true).append(true).open(path)?;

    // Write a separator before each run
    log_file.write_all(b"\n\n---\n")?;
    log_file.flush()?;

    // Save the original stdout file descriptor
    let original_stdout_fd = unsafe { libc::dup(libc::STDOUT_FILENO) };
    if original_stdout_fd < 0 {
        return Err(io::Error::last_os_error());
    }

    // Create a pipe
    let mut pipe_fds: [RawFd; 2] = [0; 2];
    if unsafe { libc::pipe(pipe_fds.as_mut_ptr()) } != 0 {
        unsafe { libc::close(original_stdout_fd) };
        return Err(io::Error::last_os_error());
    }
    let (pipe_read_fd, pipe_write_fd) = (pipe_fds[0], pipe_fds[1]);

    // Redirect stdout to the write end of the pipe
    if unsafe { libc::dup2(pipe_write_fd, libc::STDOUT_FILENO) } < 0 {
        unsafe {
            libc::close(original_stdout_fd);
            libc::close(pipe_read_fd);
            libc::close(pipe_write_fd);
        }
        return Err(io::Error::last_os_error());
    }

    // Close the write end in the main thread (it's now duplicated to stdout)
    unsafe { libc::close(pipe_write_fd) };

    // Spawn a thread to read from the pipe and write to both destinations
    thread::spawn(move || {
        let mut pipe_read = unsafe { File::from_raw_fd(pipe_read_fd) };
        let mut original_stdout = unsafe { File::from_raw_fd(original_stdout_fd) };
        let mut log_file = log_file;

        let mut buffer = [0u8; 4096];
        loop {
            match pipe_read.read(&mut buffer) {
                Ok(0) => break, // EOF - pipe closed
                Ok(n) => {
                    let data = &buffer[..n];

                    // Write to original stdout (terminal)
                    let _ = original_stdout.write_all(data);
                    let _ = original_stdout.flush();

                    // Write to log file
                    let _ = log_file.write_all(data);
                    let _ = log_file.flush();
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
    });

    Ok(())
}
