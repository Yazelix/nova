#[path = "yzx/package.rs"]
mod package;

use std::{env, fs, io, os::unix::fs::symlink, path::Path, process::ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("yzx-package: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<()> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [mode, link, native, store] if mode == "--watcher-link" => {
            watcher_link(Path::new(link), Path::new(native), Path::new(store))
        }
        [nix, relative] => {
            println!(
                "{}",
                package::path(
                    nix.to_str()
                        .ok_or_else(|| io::Error::other("invalid Nix binding"))?,
                    relative
                        .to_str()
                        .ok_or_else(|| io::Error::other("invalid package binding"))?
                )?
                .display()
            );
            Ok(())
        }
        _ => Err(io::Error::other("invalid private package-helper arguments")),
    }
}

fn watcher_link(link: &Path, native: &Path, store: &Path) -> io::Result<()> {
    if let Some(root) = package::child_root()? {
        let relative = native.strip_prefix(&root).map_err(io::Error::other)?;
        package::owned(
            Some(&root),
            "",
            relative
                .to_str()
                .ok_or_else(|| io::Error::other("invalid watcher path"))?,
        )?;
    }
    let owner = link.with_extension("yazelix-owner");
    if fs::symlink_metadata(&owner).is_ok_and(|metadata| !metadata.file_type().is_file()) {
        return Err(io::Error::other(format!(
            "invalid watcher ownership record: {}",
            owner.display()
        )));
    }
    match fs::symlink_metadata(link) {
        Ok(metadata) => {
            let target = fs::read_link(link).ok();
            let known = target.as_deref().is_some_and(|target| {
                fs::read(&owner).is_ok_and(|record| record == target.as_os_str().as_encoded_bytes())
                    || target.strip_prefix(store).ok().is_some_and(|path| {
                        let parts = path.iter().collect::<Vec<_>>();
                        parts.len() == 3
                            && parts[0]
                                .to_string_lossy()
                                .contains("-nova-helix-file-watcher-")
                            && parts[1] == "lib"
                            && Some(parts[2]) == native.file_name()
                    })
            });
            if !metadata.file_type().is_symlink() || !known {
                return Err(io::Error::other(format!(
                    "refusing to replace an existing Steel native library: {}",
                    link.display()
                )));
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => (),
        Err(error) => return Err(error),
    }
    let temporary_owner = owner.with_extension(format!("tmp-{}", std::process::id()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary_owner)?;
    use std::io::Write;
    let result = (|| {
        file.write_all(native.as_os_str().as_encoded_bytes())?;
        let temporary = link.with_extension(format!("yazelix-{}", std::process::id()));
        symlink(native, &temporary)?;
        let result = fs::rename(&temporary, link);
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result?;
        fs::rename(&temporary_owner, owner)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary_owner);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watcher_moves_only_its_own_links() {
        let dir = env::temp_dir().join(format!("yzx-watcher-owner-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let link = dir.join("watcher.so");
        let first = dir.join("first root/lib/watcher.so");
        let moved = dir.join("moved root/lib/watcher.so");
        watcher_link(&link, &first, Path::new("/nix/store")).unwrap();
        let owner = link.with_extension("yazelix-owner");
        for blocked in [
            owner.with_extension(format!("tmp-{}", std::process::id())),
            link.with_extension(format!("yazelix-{}", std::process::id())),
        ] {
            fs::write(&blocked, "existing temporary file").unwrap();
            assert!(watcher_link(&link, &moved, Path::new("/nix/store")).is_err());
            assert_eq!(fs::read_link(&link).unwrap(), first);
            assert_eq!(
                fs::read(&owner).unwrap(),
                first.as_os_str().as_encoded_bytes()
            );
            assert_eq!(
                fs::read_to_string(&blocked).unwrap(),
                "existing temporary file"
            );
            fs::remove_file(blocked).unwrap();
        }
        watcher_link(&link, &moved, Path::new("/nix/store")).unwrap();
        assert_eq!(fs::read_link(&link).unwrap(), moved);
        fs::remove_file(&link).unwrap();
        fs::write(&link, "user library").unwrap();
        assert!(watcher_link(&link, &first, Path::new("/nix/store")).is_err());
        assert_eq!(fs::read_to_string(&link).unwrap(), "user library");
        fs::remove_file(&link).unwrap();
        symlink("/user/library.so", &link).unwrap();
        assert!(watcher_link(&link, &first, Path::new("/nix/store")).is_err());
        assert_eq!(fs::read_link(&link).unwrap(), Path::new("/user/library.so"));
        fs::remove_file(&link).unwrap();
        let old = Path::new("/nix/store/fixture-nova-helix-file-watcher-1/lib/watcher.so");
        symlink(old, &link).unwrap();
        watcher_link(&link, &first, Path::new("/nix/store")).unwrap();
        assert_eq!(fs::read_link(&link).unwrap(), first);
        fs::remove_dir_all(dir).unwrap();
    }
}
