use std::path::PathBuf;

#[cfg(unix)]
pub mod unix;

#[cfg(unix)]
pub use unix::*;

#[inline]
pub fn find_program_by_name(name: &str) -> Option<PathBuf> {
    find_program_by_name_at_paths(name, &([] as [&str; 0]))
}
