use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::ops::Deref;
use std::path::{Path, PathBuf};

#[cfg(unix)]
fn set_executable(permissions: &mut fs::Permissions) {
    use std::os::unix::fs::PermissionsExt;
    permissions
        .set_mode(permissions.mode() | (libc::S_IXUSR | libc::S_IXGRP | libc::S_IXOTH) as u32);
}

#[cfg(not(unix))]
fn set_executable(_permissions: &mut fs::Permissions) {}

#[cfg(unix)]
const ENV_VAR_SEPARATOR: &str = ":";

#[cfg(windows)]
const ENV_VAR_SEPARATOR: &str = ";";

pub fn create_executable<P: AsRef<Path>>(path: P) {
    let file = fs::File::create(path).expect("could not create a file");
    let mut permissions = file
        .metadata()
        .expect("could not obtain file metadata")
        .permissions();
    set_executable(&mut permissions);
    file.set_permissions(permissions)
        .expect("could not set permissions");
}

pub fn create_dir<P: AsRef<Path>>(path: P) {
    match fs::create_dir(path) {
        Ok(()) => (),
        Err(err) => match err.kind() {
            io::ErrorKind::AlreadyExists => (),
            _ => panic!("{}", err),
        },
    }
}

pub fn create_empty_file<P: AsRef<Path>>(path: P) {
    fs::File::create(path).expect("could not create a file");
}

pub struct DirectoryRAII(PathBuf);

impl DirectoryRAII {
    pub fn new<P: AsRef<Path>>(path: P) -> DirectoryRAII {
        let path_buf = path.as_ref().to_path_buf();
        create_dir(&path_buf);
        DirectoryRAII(path_buf)
    }
}

impl Drop for DirectoryRAII {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl Deref for DirectoryRAII {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

pub struct TemporarilySetEnvVar {
    var: OsString,
    old_value: Option<OsString>,
}

impl TemporarilySetEnvVar {
    pub fn new(var: impl AsRef<OsStr>, new_value: impl AsRef<OsStr>) -> Self {
        let old_value = env::var_os(&var);
        env::set_var(&var, new_value);
        TemporarilySetEnvVar {
            var: OsString::from(var.as_ref()),
            old_value,
        }
    }

    pub fn path_var<P: AsRef<Path>>(new_paths: &[P]) -> Self {
        let mut new_value = OsString::new();
        if let Some((head, tail)) = new_paths.split_first() {
            new_value.push(head.as_ref());
            for path in tail {
                new_value.push(ENV_VAR_SEPARATOR);
                new_value.push(path.as_ref());
            }
        }
        Self::new("PATH", new_value)
    }
}

impl Drop for TemporarilySetEnvVar {
    fn drop(&mut self) {
        match self.old_value {
            Some(ref old_value) => env::set_var(&self.var, old_value),
            None => env::remove_var(&self.var),
        }
    }
}
