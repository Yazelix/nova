use std::{env, fs, path::Path, process::Command};

#[allow(dead_code)]
mod support;

use support::{
    RuntimeCase, TempDir, binary_text, embedded_store_path, excerpt, expect_contains,
    successful_output, successful_stdout, write_executable,
};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let [_, yzx, closure, expected_variant] = args.as_slice() else {
        panic!("usage: no-helix-contracts-check <yzx-package> <closure-paths> <expected-variant>");
    };

    let yzx = Path::new(yzx);
    let yzx_bin = yzx.join("bin/yzx");
    let closure = fs::read_to_string(closure).unwrap();
    for forbidden in [
        "-yazelix-helix-",
        "-helix-runtime",
        "-consolidated-helix-grammars",
        "-yzx-helix-steel-config",
        "-yzx-forest-cogs",
        "-helix-tree-sitter-",
        "-steel-core-",
    ] {
        assert!(
            !closure.contains(forbidden),
            "no-Helix closure contains {forbidden:?}\n{}",
            excerpt(&closure)
        );
    }

    let launcher = binary_text(&yzx_bin);
    let config_ui =
        fs::read_to_string(embedded_store_path(&launcher, "/bin/yzx-config-ui")).unwrap();
    expect_contains(
        &config_ui,
        "export YZX_HELIX_INCLUDED=0",
        "no-Helix config UI",
    );
    let helix = embedded_store_path(&launcher, "/bin/yzx-hx");
    let editor = embedded_store_path(&launcher, "/bin/yzx-editor");
    assert!(
        helix.to_string_lossy().contains("-yzx-hx-unavailable/"),
        "no-Helix package selected the wrong managed editor: {}",
        helix.display()
    );

    let temp = TempDir::new();
    let external = temp.path.join("external-editor");
    let observed = temp.path.join("observed");
    write_executable(
        &external,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$YZX_EXTERNAL_EDITOR_OUT\"\n",
    );
    let host_path = env::join_paths(
        std::iter::once(temp.path.clone())
            .chain(env::split_paths(&env::var_os("PATH").unwrap_or_default())),
    )
    .unwrap();
    let external_case = RuntimeCase::new(&temp.path, "external");
    external_case.write_config("[editor]\ncommand = \"external-editor\"\n");
    let target = temp.path.join("file");
    let mut disabled_command = external_case.yzx_command(&yzx_bin, "run");
    let disabled = disabled_command.arg("yzx-hx").output().unwrap();
    assert_eq!(disabled.status.code(), Some(69));
    expect_contains(
        &String::from_utf8_lossy(&disabled.stderr),
        "managed Helix is unavailable in this Yazelix package",
        "disabled yzx-hx diagnostic",
    );
    let mut status_command = external_case.yzx_command(&yzx_bin, "status");
    let status = successful_stdout(status_command.arg("--json"), "no-Helix status");
    expect_contains(
        &status,
        &format!(r#""package":"{expected_variant}""#),
        "no-Helix package identity",
    );
    let status_text = external_case.run_yzx(&yzx_bin, "status", "no-Helix text status");
    assert!(
        !status_text.contains("forest keybinding:"),
        "no-Helix status advertised an unavailable Forest binding\n{}",
        excerpt(&status_text)
    );
    let mut external_doctor = external_case.yzx_command(&yzx_bin, "doctor");
    let external_doctor_stdout = successful_stdout(
        external_doctor.env("PATH", &host_path),
        "external-editor doctor",
    );
    expect_contains(
        &external_doctor_stdout,
        "shell nu · editor external-editor · agent auto",
        "no-Helix external-editor doctor",
    );
    assert!(
        !external_doctor_stdout.contains("warn  Editor"),
        "no-Helix doctor warned about the configured external editor\n{}",
        excerpt(&external_doctor_stdout)
    );
    assert!(
        !external_doctor_stdout.contains("keybindings.sidebar_focus:"),
        "no-Helix doctor advertised an unavailable Forest binding\n{}",
        excerpt(&external_doctor_stdout)
    );

    let unavailable_case = RuntimeCase::new(&temp.path, "unavailable");
    let doctor_helix_config = unavailable_case.config_home.join("helix/config.toml");
    fs::create_dir_all(doctor_helix_config.parent().unwrap()).unwrap();
    fs::write(&doctor_helix_config, "[keys.normal]\nA-r = \":noop\"\n").unwrap();
    let doctor_stdout = unavailable_case.run_yzx(&yzx_bin, "doctor", "no-Helix doctor");
    expect_contains(
        &doctor_stdout,
        &format!(
            "warn  Editor           yzx-hx is unavailable in package {expected_variant}; set editor.command to an installed editor"
        ),
        "no-Helix unavailable-editor doctor",
    );
    expect_contains(
        &doctor_stdout,
        "ok    Commands         shell nu · editor yzx-hx · agent auto",
        "no-Helix unavailable-editor doctor",
    );
    assert!(
        !doctor_stdout.contains("warn  Helix config"),
        "no-Helix doctor inspected an unused Helix sidecar\n{}",
        excerpt(&doctor_stdout)
    );
    let helix_tutor = successful_stdout(
        Command::new(&yzx_bin).args(["tutor", "hx"]),
        "no-Helix tutor",
    );
    expect_contains(
        &helix_tutor,
        "If your selected package omits managed Helix",
        "no-Helix tutor",
    );
    for args in [["tutor", "list"], ["tutor", "files"]] {
        let output = successful_stdout(Command::new(&yzx_bin).args(args), "no-Helix tutor");
        for unavailable in ["Forest", "Ctrl y"] {
            assert!(
                !output.contains(unavailable),
                "no-Helix tutor advertised {unavailable}\n{}",
                excerpt(&output)
            );
        }
    }

    successful_output(
        Command::new(editor)
            .arg(&target)
            .env("YAZELIX_CONFIG_HOME", external_case.config_home)
            .env("PATH", host_path)
            .env("YZX_EXTERNAL_EDITOR_OUT", &observed)
            .env_remove("YAZELIX_EDITOR")
            .env_remove("ZELLIJ"),
        "external editor",
    );
    assert_eq!(
        fs::read_to_string(observed).unwrap(),
        format!("{}\n", target.display())
    );
}
