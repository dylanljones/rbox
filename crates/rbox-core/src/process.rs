// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Snapshot queries for running Rekordbox applications and helper processes.
//!
//! [`get_rekordbox_processes`] includes process names beginning with `rekordbox`,
//! case-insensitively, such as `rekordboxAgent`. [`get_rekordbox_pid`] and
//! [`is_rekordbox_running`] select only the main name `rekordbox` (with an optional
//! `.exe` suffix). Reported names retain the operating system's spelling.
//!
//! Each call obtains a fresh system snapshot. These checks do not lock a library
//! or prevent Rekordbox from starting immediately afterwards. Results depend on
//! the process information visible to the current user; entries whose names are
//! not valid UTF-8 are omitted. Process names alone do not verify an executable's
//! identity, installation path, or active library.
//!
//! ```no_run
//! use rbox_core::process::{get_rekordbox_processes, is_rekordbox_running};
//!
//! for process in get_rekordbox_processes() {
//!     println!("{}: {}", process.pid, process.name);
//! }
//! if is_rekordbox_running() {
//!     println!("Rekordbox is running; its library may be in use.");
//! }
//! ```

/// A running Rekordbox application or helper process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Process {
    /// Operating-system process identifier at the time of the snapshot.
    pub pid: u32,
    /// Process name as reported by the operating system, including any suffix.
    pub name: String,
}

/// Return Rekordbox processes in PID order; matching is case-insensitive.
///
/// Includes helpers. An empty list means no matching process was visible in the
/// snapshot. No process is started, stopped, or signalled.
pub fn get_rekordbox_processes() -> Vec<Process> {
    let system = sysinfo::System::new_all();
    let mut processes: Vec<_> = system
        .processes()
        .iter()
        .filter_map(|(pid, process)| {
            let name = process.name().to_str()?.to_owned();
            name.to_lowercase()
                .starts_with("rekordbox")
                .then_some(Process {
                    pid: pid.as_u32(),
                    name,
                })
        })
        .collect();
    processes.sort_by_key(|p| p.pid);
    processes
}

/// Return the lowest PID of a main Rekordbox process, excluding agent helpers.
///
/// Only PIDs representable as `i32` are returned. If no main application is
/// visible, returns `None`, even when an agent or other helper is running.
pub fn get_rekordbox_pid() -> Option<i32> {
    get_rekordbox_processes()
        .into_iter()
        .filter(|p| p.name.to_lowercase().trim_end_matches(".exe") == "rekordbox")
        .find_map(|p| i32::try_from(p.pid).ok())
}

/// Whether a main Rekordbox process is running.
///
/// Equivalent to [`get_rekordbox_pid`] returning `Some`; this is a snapshot,
/// not a guarantee that the library is safe to modify afterwards.
pub fn is_rekordbox_running() -> bool {
    get_rekordbox_pid().is_some()
}
