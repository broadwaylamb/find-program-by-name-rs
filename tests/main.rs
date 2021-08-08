use find_program_by_name::*;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

mod helpers;
use helpers::*;

/// Creates the following tree in the current directory:
///
/// ```
/// test_dir/
///   foo/
///     a.exe/
///       a.exe
///     b.exe
///   bar/
///     a.exe
///   non-executables/
///     impostor.exe
///   a.exe
/// ```
fn create_test_tree(test_dir: &str) -> DirectoryRAII {
    let test_dir = Path::new(option_env!("CARGO_TARGET_TMPDIR").unwrap_or("")).join(test_dir);
    match fs::remove_dir_all(&test_dir) {
        Ok(()) => (),
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => (),
            _ => panic!("{}", err),
        },
    }
    let test_dir_raii = DirectoryRAII::new(&test_dir);
    create_dir(test_dir.join("foo"));
    create_dir(test_dir.join("foo/a"));
    create_dir(test_dir.join("bar"));
    create_dir(test_dir.join("non-executables"));
    create_executable(test_dir.join("foo/a/a.exe"));
    create_executable(test_dir.join("foo/b.exe"));
    create_executable(test_dir.join("a.exe"));
    create_executable(test_dir.join("bar/a.exe"));
    create_empty_file(test_dir.join("non-executables/impostor.exe"));
    test_dir_raii
}

#[test]
fn test_has_slashes() {
    assert_eq!(
        find_program_by_name_at_paths("nonexistent/file.exe", &["nonexistent_dir"]),
        Some(PathBuf::from("nonexistent/file.exe"))
    )
}

#[test]
#[cfg(unix)]
fn test_has_backslashes_unix() {
    let test_dir = create_test_tree("test_has_backslashes_unix");
    create_dir(test_dir.join("\\"));
    create_executable(test_dir.join("\\/executab\\e"));
    assert_eq!(
        find_program_by_name_at_paths(
            "executab\\e",
            &[
                test_dir.to_path_buf(),
                test_dir.join("foo"),
                test_dir.join("\\")
            ]
        ),
        Some(test_dir.join("\\/executab\\e"))
    )
}

#[test]
#[cfg(windows)]
fn test_has_backslashes_windows() {
    assert_eq!(
        find_program_by_name_at_paths("nonexistent\\file.exe", &["nonexistent_dir"]),
        Some(PathBuf::from("nonexistent\\file.exe"))
    )
}

#[test]
fn test_uses_env_path() {
    let test_dir = create_test_tree("test_uses_env_path");

    let mut path_raii = TemporarilySetEnvVar::path_var(&[
        test_dir.join("bar"),
        test_dir.join("foo"),
        test_dir.to_path_buf(),
    ]);

    assert_eq!(
        find_program_by_name("a.exe"),
        Some(test_dir.join("bar/a.exe"))
    );

    assert_eq!(find_program_by_name("non_existent.exe"), None);

    drop(path_raii);

    path_raii = TemporarilySetEnvVar::path_var(&["non_existent"]);

    assert_eq!(find_program_by_name("a.exe"), None);

    drop(path_raii);
}

#[test]
fn test_not_found() {
    let test_dir = create_test_tree("test_not_found");

    assert_eq!(
        find_program_by_name_at_paths(
            "a.exe",
            &[
                test_dir.join("foo"),
                test_dir.to_path_buf(),
                test_dir.join("bar"),
            ]
        ),
        Some(PathBuf::from(test_dir.join("a.exe")))
    )
}

#[test]
#[cfg(unix)]
fn test_ignores_non_executables() {
    let test_dir = create_test_tree("test_ignores_non_executables");
    assert_eq!(
        find_program_by_name_at_paths("impostor.exe", &[test_dir.join("non-executables")]),
        None
    )
}

#[test]
#[cfg(windows)]
fn test_appends_extension() {
    let test_dir = create_test_tree("test_appends_extension");
    assert_eq!(
        find_program_by_name_at_paths(
            "a",
            &[
                test_dir.join("foo"),
                test_dir.to_path_buf(),
                test_dir.join("bar"),
            ]
        ),
        Some(test_dir.join("a.exe"))
    )
}
