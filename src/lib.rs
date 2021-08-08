use std::io::Result as IOResult;
use std::path::PathBuf;

#[cfg(unix)]
pub mod unix;
#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use windows::*;

#[inline]
pub fn find_program_by_name(name: &str) -> IOResult<PathBuf> {
    find_program_by_name_at_paths(name, &([] as [&str; 0]))
}
