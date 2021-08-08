use std::array;
use std::env;
use std::ffi::{OsStr, OsString};
use std::io;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::ptr;
use winapi::{
    ctypes::wchar_t,
    shared::minwindef::{DWORD, MAX_PATH},
    um::processenv::SearchPathW,
};

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
pub fn find_program_by_name_at_paths<P: AsRef<Path>>(
    name: &str,
    paths: &[P],
) -> io::Result<PathBuf> {
    assert!(!name.is_empty(), "Must have a name!");
    let path_separators: &[_] = &['/', '\\'];
    if name.find(path_separators).is_some() {
        return Ok(PathBuf::from(name));
    }

    let mut path_storage: Vec<wchar_t> = Vec::new();

    if let Some((head, tail)) = paths.split_first() {
        path_storage.reserve(paths.len() * MAX_PATH);
        path_storage.extend(head.as_ref().as_os_str().encode_wide());
        for path in tail {
            path_storage.extend(OsStr::new(";").encode_wide());
            path_storage.extend(path.as_ref().as_os_str().encode_wide());
        }
        path_storage.push(0);
    }

    let default_path_extensions = ["", ".exe"];
    if let Ok(pathext_env) = env::var("PATHEXT") {
        return perform_actual_search(
            name,
            path_storage,
            array::IntoIter::new(default_path_extensions).chain(pathext_env.split(';')),
        );
    }

    perform_actual_search(
        name,
        path_storage,
        array::IntoIter::new(default_path_extensions),
    )
}

fn perform_actual_search<'a>(
    name: &str,
    path: Vec<wchar_t>,
    path_extensions: impl Iterator<Item = &'a str>,
) -> io::Result<PathBuf> {
    let mut u16_result: Vec<wchar_t> = Vec::new();
    let mut len = MAX_PATH as DWORD;
    for extension in path_extensions {
        loop {
            u16_result.resize(len as usize, 0);

            // Lets attach the extension manually. That is needed for files
            // with a point in name like aaa.bbb. SearchPathW will not add extension
            // from its argument to such files because it thinks they already had one.
            let u16_name_ext = {
                let mut name_ext: Vec<wchar_t> = OsStr::new(name).encode_wide().collect();
                name_ext.extend(OsStr::new(extension).encode_wide());
                name_ext.push(0);
                name_ext
            };
            len = unsafe {
                SearchPathW(
                    if path.is_empty() {
                        ptr::null()
                    } else {
                        path.as_ptr()
                    },
                    u16_name_ext.as_ptr(),
                    ptr::null(),
                    u16_result.len() as DWORD,
                    u16_result.as_mut_ptr(),
                    ptr::null_mut(),
                )
            };
            if len as usize <= u16_result.capacity() {
                break;
            }
        }
        if len != 0 {
            break; // Found it.
        }
    }

    if len == 0 {
        Err(io::Error::last_os_error())
    } else {
        u16_result.resize(len as usize, 0);
        Ok(PathBuf::from(OsString::from_wide(&u16_result)))
    }
}
