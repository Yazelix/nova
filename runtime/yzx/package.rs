// Native consumers each use a subset of this shared implementation.
#![allow(dead_code)]

use std::{
    env,
    ffi::OsString,
    io,
    os::unix::fs::PermissionsExt,
    path::{Component, Path, PathBuf},
    process::Command,
};

pub(crate) const ROOT_ENV: &str = "YAZELIX_RUNTIME_ROOT";
pub(crate) const ROOT_MARKER: &str = "__YZX_RUNTIME_ROOT__";

pub(crate) fn front_door(portable: bool) -> io::Result<Option<PathBuf>> {
    if !portable {
        return Ok(None);
    }
    executable_root(&env::current_exe()?).map(Some)
}

#[cfg(target_os = "linux")]
pub(crate) fn check_native_inputs(root: &Path) -> io::Result<()> {
    unsafe extern "C" {
        fn gnu_get_libc_version() -> *const std::ffi::c_char;
    }
    let inputs =
        std::fs::read_to_string(owned(Some(root), "", "share/yazelix/native-inputs.txt")?)?;
    let mut lines = inputs.lines();
    let minimum = lines
        .next()
        .and_then(|line| line.strip_prefix("glibc "))
        .ok_or_else(|| io::Error::other("invalid native package requirements"))?;
    let version = |text: &str| -> io::Result<Vec<u32>> {
        text.split('.')
            .map(|part| part.parse().map_err(io::Error::other))
            .collect()
    };
    // glibc returns a process-lifetime, NUL-terminated version string.
    let actual = unsafe { std::ffi::CStr::from_ptr(gnu_get_libc_version()) }
        .to_str()
        .map_err(io::Error::other)?;
    if version(actual)? < version(minimum)? {
        return Err(io::Error::other(format!(
            "this package requires glibc {minimum} or newer; host has {actual}"
        )));
    }
    for relative in lines {
        let path = owned(Some(root), "", relative)?;
        if !std::fs::File::open(&path)?.metadata()?.is_file() {
            return Err(io::Error::other(format!(
                "required native package input is not a regular file: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

fn executable_root(executable: &Path) -> io::Result<PathBuf> {
    let executable = executable.canonicalize()?;
    let bin = executable
        .parent()
        .filter(|path| path.file_name().is_some_and(|name| name == "bin"))
        .ok_or_else(|| {
            io::Error::other(format!(
                "package executable is not installed under bin: {}",
                executable.display()
            ))
        })?;
    bin.parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::other("package root is missing"))
}

pub(crate) fn child_root() -> io::Result<Option<PathBuf>> {
    env::var_os(ROOT_ENV)
        .filter(|value| !value.is_empty())
        .map(|value| {
            let path = PathBuf::from(value);
            if !path.is_absolute() {
                return Err(io::Error::other(format!(
                    "{ROOT_ENV} must be absolute: {}",
                    path.display()
                )));
            }
            path.canonicalize()
        })
        .transpose()
}

pub(crate) fn owned(root: Option<&Path>, nix: &str, relative: &str) -> io::Result<PathBuf> {
    let Some(root) = root else {
        return Ok(nix.into());
    };
    if relative.is_empty()
        || Path::new(relative)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(io::Error::other(format!(
            "invalid owned package path: {relative}"
        )));
    }
    let path = root.join(relative);
    let resolved = path.canonicalize().map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("required package path {}: {error}", path.display()),
        )
    })?;
    if !resolved.starts_with(root) {
        return Err(io::Error::other(format!(
            "owned package path escapes runtime root: {}",
            path.display()
        )));
    }
    if relative.starts_with("libexec/yazelix/") {
        let metadata = resolved.metadata()?;
        if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
            return Err(io::Error::other(format!(
                "required package command is not executable: {}",
                path.display()
            )));
        }
    }
    Ok(path)
}

pub(crate) fn path(nix: &str, relative: &str) -> io::Result<PathBuf> {
    owned(child_root()?.as_deref(), nix, relative)
}

pub(crate) fn search_path(root: Option<&Path>, nix_prefix: &str) -> io::Result<OsString> {
    let mut paths = match root {
        Some(root) => {
            for command in [
                "nu",
                "starship",
                "carapace",
                "atuin",
                "zoxide",
                "fzf",
                "jq",
                "bash",
                "sh",
                "zsh",
                "fish",
                "awk",
                "sed",
                "tput",
                "zj-radar",
                "yzx-zellij",
                "zellij",
                "tu",
            ] {
                owned(Some(root), "", &format!("libexec/yazelix/{command}"))?;
            }
            vec![root.join("libexec/yazelix"), root.join("bin")]
        }
        None => env::split_paths(nix_prefix).collect(),
    };
    if let Some(path) = env::var_os("PATH").filter(|value| !value.is_empty()) {
        paths.extend(env::split_paths(&path));
    }
    env::join_paths(paths).map_err(io::Error::other)
}

// Templates contain quoted root-relative paths, never user configuration.
pub(crate) fn render(root: Option<&Path>, text: &str) -> io::Result<String> {
    let Some(root) = root else {
        return Ok(text.into());
    };
    for suffix in text.split(ROOT_MARKER).skip(1) {
        let relative = suffix
            .strip_prefix('/')
            .ok_or_else(|| io::Error::other("invalid package root marker"))?;
        let end = relative
            .find(|ch: char| !ch.is_ascii_alphanumeric() && !"/-_.".contains(ch))
            .unwrap_or(relative.len());
        owned(Some(root), "", &relative[..end])?;
    }
    let quoted = json_string(
        root.to_str()
            .ok_or_else(|| io::Error::other("package root is not UTF-8"))?,
    );
    Ok(text.replace(ROOT_MARKER, &quoted[1..quoted.len() - 1]))
}

pub(crate) fn json_string(value: &str) -> String {
    let mut escaped = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{0008}' => escaped.push_str("\\b"),
            '\u{000C}' => escaped.push_str("\\f"),
            character if character <= '\u{001F}' => {
                escaped.push_str(&format!("\\u{:04x}", character as u32))
            }
            character => escaped.push(character),
        }
    }
    escaped.push('"');
    escaped
}

pub(crate) fn apply(root: Option<&Path>, command: &mut Command) {
    if let Some(root) = root {
        command.env(ROOT_ENV, root);
    } else {
        command.env_remove(ROOT_ENV);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::symlink};

    #[cfg(target_os = "linux")]
    #[test]
    fn native_inputs_require_the_declared_libc_and_owned_libraries() {
        let root = env::temp_dir().join(format!("yzx-native-{}", std::process::id()));
        fs::create_dir_all(root.join("share/yazelix")).unwrap();
        fs::create_dir_all(root.join("lib")).unwrap();
        let inputs = root.join("share/yazelix/native-inputs.txt");
        let library = root.join("lib/private.so");
        fs::write(&inputs, "glibc 2.0\nlib/private.so\n").unwrap();
        assert!(
            check_native_inputs(&root)
                .unwrap_err()
                .to_string()
                .contains(library.to_str().unwrap())
        );
        fs::write(&library, "packaged library").unwrap();
        check_native_inputs(&root).unwrap();
        fs::write(&inputs, "glibc 999.0\nlib/private.so\n").unwrap();
        assert!(
            check_native_inputs(&root)
                .unwrap_err()
                .to_string()
                .contains("glibc 999.0")
        );
        fs::write(&inputs, "glibc 2.0\n../host/private.so\n").unwrap();
        assert!(check_native_inputs(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn moved_roots_follow_the_executable_and_reject_missing_or_escaping_inputs() {
        let base = env::temp_dir().join(format!("yzx-package-{}", std::process::id()));
        fs::create_dir_all(&base).unwrap();
        let alias = base.join("temporary alias");
        symlink(&base, &alias).unwrap();
        for name in ["first", "moved root"] {
            let root = alias.join(name);
            fs::create_dir_all(root.join("bin")).unwrap();
            fs::create_dir_all(root.join("share/yazelix")).unwrap();
            fs::write(root.join("bin/yzx"), "native fixture").unwrap();
            fs::write(root.join("share/yazelix/config.toml"), "fixture").unwrap();
            let entry = base.join(format!("{name}-entry"));
            symlink(root.join("bin/yzx"), &entry).unwrap();
            let selected = executable_root(&entry).unwrap();
            assert_eq!(selected, root.canonicalize().unwrap());
            let relative = "share/yazelix/config.toml";
            assert_eq!(
                fs::read_to_string(owned(Some(&selected), "/old/store/config", relative).unwrap())
                    .unwrap(),
                "fixture"
            );
            fs::remove_file(root.join(relative)).unwrap();
            let missing = owned(Some(&selected), "/old/store/config", relative).unwrap_err();
            assert!(
                missing
                    .to_string()
                    .contains(&selected.join(relative).display().to_string())
            );
            symlink(base.join("outside"), root.join(relative)).unwrap();
            fs::write(base.join("outside"), "host substitute").unwrap();
            assert!(
                owned(Some(&selected), "/old/store/config", relative)
                    .unwrap_err()
                    .to_string()
                    .contains("escapes")
            );
            assert!(owned(Some(&selected), "/old/store/config", "../outside").is_err());
            assert!(owned(Some(&selected), "/old/store/config", "/outside").is_err());
        }
        assert_eq!(
            owned(None, "/pinned/nix/path", "unused").unwrap(),
            PathBuf::from("/pinned/nix/path")
        );
        fs::remove_dir_all(base).unwrap();
    }
}
