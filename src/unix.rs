use std::env;
use std::ffi::CString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// Find the first executable file `name` in `paths`.
///
/// This does not perform hashing as a shell would but instead stats each `PATH`
/// entry individually so should generally be avoided. Core LLVM library
/// functions and options should instead require fully specified paths.
///
/// `name` is the name of the executable to find. If it contains any system slashes, it will be
/// returned as is.
///
/// `paths` is a list of paths to search for `name`. If empty, it will use the system `PATH`
/// environment instead.
///
/// Returns the fully qualified path to the first `name` in `paths` if it exists, or `name`
/// if `name` has slashes in it. Otherwise returns `None`.
pub fn find_program_by_name_at_paths<P: AsRef<Path>>(name: &str, paths: &[P]) -> Option<PathBuf> {
    assert!(!name.is_empty(), "Must have a name!");
    // Use the given path verbatim if it contains any slashes; this matches
    // the behavior of sh(1) and friends.
    if name.find('/').is_some() {
        return Some(PathBuf::from(name));
    }

    if paths.is_empty() {
        if let Ok(path_env) = env::var("PATH") {
            return helper(name, path_env.split(':').map(AsRef::as_ref));
        }
    }

    helper(name, paths.iter().map(AsRef::as_ref))
}

fn helper<'a>(name: &str, paths: impl Iterator<Item = &'a Path>) -> Option<PathBuf> {
    let mut file_path = PathBuf::new();
    for path in paths {
        if path.as_os_str().is_empty() {
            continue;
        }

        // Check to see if this first directory contains the executable...
        file_path.push(path);
        file_path.push(name);
        if can_execute(&file_path) {
            return Some(file_path);
        }
        file_path.clear(); // clear but keep the allocation
    }

    None
}

fn can_execute<P: AsRef<Path>>(file: P) -> bool {
    let file = file.as_ref();
    let path_storage =
        CString::new(file.as_os_str().as_bytes()).expect("the path contains nul characters");

    // SAFETY: We pass a valid C string and a valid mode to the access function.
    unsafe {
        if libc::access(path_storage.as_ptr(), libc::R_OK | libc::X_OK) == -1 {
            return false;
        }
    }

    // Don't say that directories are executable.
    match fs::metadata(file) {
        Ok(metadata) => metadata.is_file(),
        Err(_) => false,
    }
}
