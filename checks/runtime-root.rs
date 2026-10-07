use std::{env, fs, os::unix::fs::symlink, process::Command};

mod support;
use support::{RuntimeCase, TempDir, successful_output, successful_stdout, write_executable};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let [_, fixture, script, darwin, out] = args.as_slice() else {
        panic!("usage: runtime-root-check <fixture> <script> <darwin> <out>")
    };
    let temp = TempDir::new();
    let installed = temp.path.canonicalize().unwrap();
    let first = installed.join("first root");
    let moved = installed.join("moved root's");
    successful_output(
        Command::new("cp").args(["-R", fixture]).arg(&first),
        "install root fixture",
    );
    successful_output(
        Command::new("chmod").args(["-R", "u+w"]).arg(&first),
        "writable disposable fixture",
    );
    let case = RuntimeCase::new(&temp.path, "user");
    let config = case.write_config("[welcome]\nenabled = false\n[appearance]\nmode = \"light\"\n");
    fs::create_dir_all(case.config_home.join("helix")).unwrap();
    let user_helix = case.config_home.join("helix/config.toml");
    fs::write(
        &user_helix,
        "# user configuration\n[editor]\nline-number = \"relative\"\n",
    )
    .unwrap();
    let entry = temp.path.join("package-manager-yzx");
    let steel = temp.path.join("steel-home");
    for root in [&first, &moved] {
        if root == &moved {
            fs::rename(&first, &moved).unwrap();
            fs::remove_file(&entry).unwrap();
        }
        symlink(root.join("bin/yzx"), &entry).unwrap();
        let command = |args: &[&str]| {
            let mut command = Command::new(&entry);
            command
                .env("YAZELIX_CONFIG_HOME", &case.config_home)
                .env("YAZELIX_STATE_DIR", &case.state_dir);
            command
                .args(args)
                .current_dir(&temp.path)
                .env("YAZELIX_RUNTIME_ROOT", "/wrong/inherited/root")
                .env("STEEL_HOME", &steel)
                .env("XDG_CONFIG_HOME", temp.path.join("config"))
                .env("XDG_DATA_HOME", temp.path.join("data"))
                .env("XDG_CACHE_HOME", temp.path.join("cache"))
                .env("NO_COLOR", "1")
                .env_remove("ZELLIJ")
                .env_remove("ZELLIJ_SESSION_NAME");
            command
        };
        let environment = successful_stdout(
            &mut command(&[
                "run",
                "/bin/sh",
                "-c",
                "printf '%s\\n' \"$YAZELIX_RUNTIME_ROOT\" \"$EDITOR\" \"$YZX_ZELLIJ\"",
            ]),
            "front-door root and child bindings",
        );
        assert_eq!(
            environment.lines().collect::<Vec<_>>(),
            [
                root.display().to_string(),
                root.join("libexec/yazelix/yzx-editor")
                    .display()
                    .to_string(),
                root.join("libexec/yazelix/yzx-zellij")
                    .display()
                    .to_string()
            ]
        );
        let config_helper = root.join("libexec/yazelix/yzx-config");
        let real_config_helper = config_helper.with_extension("real");
        fs::rename(&config_helper, &real_config_helper).unwrap();
        write_executable(
            &config_helper,
            "#!/bin/sh\nif [ \"${1:-}\" = --get ] && [ \"${2:-}\" = shell.atuin ]; then\n  printf '%s' \"$YAZELIX_RUNTIME_ROOT\" > \"$YAZELIX_STATE_DIR/atuin-helper-root\"\nfi\nexec \"$YAZELIX_RUNTIME_ROOT/libexec/yazelix/yzx-config.real\" \"$@\"\n",
        );
        let nu = successful_output(
            &mut command(&["run", "yzx-shell", "-c", "print $env.YAZELIX_RUNTIME_ROOT"]),
            "managed Nu startup",
        );
        assert_eq!(
            String::from_utf8_lossy(&nu.stdout).trim(),
            root.to_str().unwrap()
        );
        assert!(!String::from_utf8_lossy(&nu.stderr).contains("managed Atuin init failed"));
        assert_eq!(
            fs::read_to_string(case.state_dir.join("atuin-helper-root")).unwrap(),
            root.to_str().unwrap(),
            "Atuin setting lookup must use the installed config helper"
        );
        fs::rename(&real_config_helper, &config_helper).unwrap();
        let nu_config = fs::read_to_string(case.state_dir.join("nu/packaged/config.nu")).unwrap();
        assert!(
            nu_config.contains(
                &root
                    .join("share/yazelix/nu/zoxide.nu")
                    .display()
                    .to_string()
            )
        );
        let probe = command(&["run", "yazi", "--version"]);
        let mut terminal = Command::new(script);
        terminal.current_dir(&temp.path);
        for (key, value) in probe.get_envs() {
            if let Some(value) = value {
                terminal.env(key, value);
            } else {
                terminal.env_remove(key);
            }
        }
        let shell_command = format!(
            "stty rows 24 cols 80; exec {}",
            std::iter::once(probe.get_program())
                .chain(probe.get_args())
                .map(|arg| format!("'{}'", arg.to_str().unwrap().replace('\'', "'\\''")))
                .collect::<Vec<_>>()
                .join(" ")
        );
        if darwin == "true" {
            terminal.args(["-qe", "/dev/null", "/bin/sh", "-c", &shell_command]);
        } else {
            terminal.args(["-qec", &shell_command, "/dev/null"]);
        }
        let yazi = successful_stdout(&mut terminal, "managed Yazi startup inputs");
        assert!(
            yazi.contains("Yazi") && yazi.contains("Version:"),
            "Yazi version output: {yazi:?}"
        );
        assert_eq!(
            case.state_dir
                .join("yazi/yazi.toml")
                .canonicalize()
                .unwrap(),
            root.join("share/yazelix/yazi/yazi.toml")
        );
        assert!(
            fs::read_to_string(root.join("share/yazelix/yazi/yazi.toml"))
                .unwrap()
                .contains("\"$YZX_OPEN\" %s")
        );
        let zellij = fs::read_to_string(case.state_dir.join("zellij/config.kdl")).unwrap();
        let layout = fs::read_to_string(case.state_dir.join("zellij/layout.kdl")).unwrap();
        let version_command = zellij
            .lines()
            .find_map(|line| line.trim().strip_prefix("command_version_command "))
            .unwrap();
        let version_command = successful_stdout(
            Command::new(root.join("libexec/yazelix/jq")).args([
                "-nr",
                "--argjson",
                "command",
                version_command,
                "$command",
            ]),
            "decode generated bar command",
        );
        let badge = successful_stdout(
            &mut command(&["run", "/bin/sh", "-c", version_command.trim()]),
            "generated bar command execution",
        );
        assert!(badge.starts_with("NOVA "), "missing runtime badge: {badge}");
        for line in zellij
            .lines()
            .filter(|line| line.contains("command_marker "))
        {
            let marker = line.split('"').nth(1).unwrap();
            assert!(
                root.join("libexec/yazelix/yzx-agent")
                    .to_str()
                    .unwrap()
                    .contains(marker),
                "agent identity marker does not match the installed command: {marker}"
            );
        }
        let mut zellij_check = command(&["run", "yzx-zellij", "--config"]);
        zellij_check
            .arg(case.state_dir.join("zellij/config.kdl"))
            .args(["setup", "--check"]);
        successful_output(&mut zellij_check, "materialized Zellij config");
        for text in [&zellij, &layout] {
            assert!(text.contains(&root.join("libexec/yazelix").display().to_string()));
            assert!(!text.contains("__YZX_RUNTIME_ROOT__"));
            if root == &moved {
                assert!(!text.contains(first.to_str().unwrap()));
            }
        }
        let guide = successful_stdout(&mut command(&["tutor", "hx"]), "Helix tutor guide");
        let tutor_command = guide
            .lines()
            .find_map(|line| line.strip_prefix("- `")?.strip_suffix(" --tutor`"))
            .unwrap();
        successful_output(
            &mut command(&[
                "run",
                "nu",
                "--no-config-file",
                "-c",
                &format!("{tutor_command} --health"),
            ]),
            "advertised Helix command and managed runtime",
        );
        let library = format!("libnova_helix_file_watcher{}", env::consts::DLL_SUFFIX);
        assert_eq!(
            fs::read_link(steel.join("native").join(&library)).unwrap(),
            root.join("lib").join(&library)
        );
        let helix = root.join("libexec/yazelix/helix");
        let real_helix = helix.with_extension("real");
        fs::rename(&helix, &real_helix).unwrap();
        write_executable(
            &helix,
            "#!/bin/sh\nprintf '%s\\n' \"$HELIX_RUNTIME\" \"$STEEL_SEARCH_PATHS\" \"$YZX_HELIX_REGISTER\" \"$YZX_HELIX_WATCHER_START\" \"$YZX_OPEN_TERMINAL\"\n",
        );
        let inputs =
            successful_stdout(&mut command(&["run", "hx"]), "Helix wrapper asset bindings");
        for relative in [
            "share/helix/runtime",
            "share/helix/steel",
            "share/steel",
            "libexec/yazelix/yzx-helix-register",
            "share/yazelix/helix-steel/watcher-start.scm",
            "libexec/yazelix/yzx-open-terminal",
        ] {
            assert!(
                inputs.contains(&root.join(relative).display().to_string()),
                "missing {relative}: {inputs}"
            );
        }
        fs::rename(&real_helix, &helix).unwrap();
        assert_eq!(
            fs::read_to_string(&user_helix).unwrap(),
            "# user configuration\n[editor]\nline-number = \"relative\"\n"
        );
        assert_eq!(
            fs::read_to_string(&config).unwrap(),
            "[welcome]\nenabled = false\n[appearance]\nmode = \"light\"\n"
        );

        let nu = root.join("libexec/yazelix/nu");
        let saved = nu.with_extension("saved");
        fs::rename(&nu, &saved).unwrap();
        let missing = command(&["run", "yzx-shell", "-c", "print 'host fallback'"])
            .output()
            .unwrap();
        assert!(!missing.status.success());
        assert!(String::from_utf8_lossy(&missing.stderr).contains(nu.to_str().unwrap()));
        assert!(!String::from_utf8_lossy(&missing.stdout).contains("host fallback"));
        symlink(&saved, &nu).unwrap();
        successful_output(
            &mut command(&["run", "yzx-shell", "-c", "print $env.YAZELIX_RUNTIME_ROOT"]),
            "contained symlink",
        );
        fs::remove_file(&nu).unwrap();
        symlink("/bin/sh", &nu).unwrap();
        let escape = command(&["env"]).output().unwrap();
        assert!(!escape.status.success());
        assert!(String::from_utf8_lossy(&escape.stderr).contains("escapes runtime root"));
        fs::remove_file(&nu).unwrap();
        fs::rename(saved, nu).unwrap();
        // Nix delivery still requires its owned Git, even with host Git on PATH.
        let git = root.join("libexec/yazelix/git");
        if git.is_file() {
            write_executable(&temp.path.join("git"), "#!/bin/sh\nexit 0\n");
            let saved = git.with_extension("saved");
            fs::rename(&git, &saved).unwrap();
            let missing = command(&["env"]).env("PATH", &temp.path).output().unwrap();
            fs::rename(&saved, &git).unwrap();
            assert!(!missing.status.success());
            assert!(String::from_utf8_lossy(&missing.stderr).contains(git.to_str().unwrap()));
        }
        // Native archives guard private libraries even when the host has copies.
        if let Ok(inputs) = fs::read_to_string(root.join("share/yazelix/native-inputs.txt")) {
            let relative = inputs
                .lines()
                .find(|line| line.ends_with("/libssl.so.3"))
                .unwrap();
            let library = root.join(relative);
            let saved = library.with_extension("saved");
            fs::rename(&library, &saved).unwrap();
            let missing = command(&["--version"]).output().unwrap();
            fs::rename(&saved, &library).unwrap();
            assert!(!missing.status.success());
            assert!(String::from_utf8_lossy(&missing.stderr).contains(library.to_str().unwrap()));
        }
    }
    fs::write(
        out,
        "root selection, relocation, child bindings, configs, assets and guards: ok\n",
    )
    .unwrap();
}
