use std::{env, fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

mod support;

use support::{
    RuntimeCase, TempDir, binary_text, embedded_store_path, excerpt, expect_contains, expect_order,
    write_executable,
};

macro_rules! expect_contains_all {
    ($haystack:expr, $context:expr; $($needle:expr),+ $(,)?) => {
        $(expect_contains($haystack, &$needle, $context);)+
    };
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let [_, yzx, out] = args.as_slice() else {
        panic!("usage: helix-contracts-check <yzx-package> <out>");
    };

    let yzx = Path::new(yzx);
    let yzx_launcher = binary_text(&yzx.join("bin/yzx"));
    let helix = embedded_store_path(&yzx_launcher, "/bin/yzx-hx");

    expect_helix_wrapper(&helix);
    expect_helix_doctor_warnings(yzx);

    fs::write(out, "ok\n").unwrap();
}

fn expect_helix_wrapper(helix: &Path) {
    let helix_script = fs::read_to_string(helix).unwrap();
    let context = format!("{} managed Helix wrapper", helix.display());
    expect_contains(&helix_script, "YAZELIX_HELIX_BRIDGE=1", &context);
    expect_contains(&helix_script, "STEEL_SEARCH_PATHS=", &context);
    expect_contains(&helix_script, "--get forest.side", &context);
    expect_contains(&helix_script, "export YAZELIX_FOREST_SIDE", &context);

    let helix_config =
        fs::read_to_string(embedded_store_path(&helix_script, "-config.toml").join("config.toml"))
            .unwrap();
    assert!(
        !helix_config.contains(":forest-open"),
        "packaged Helix config must leave the Forest binding to root config"
    );
    expect_contains(
        &helix_config,
        r#"A-r = ':sh yzx reveal "%{buffer_name}"'"#,
        "managed Helix reveal binding",
    );
    expect_contains(
        &helix_config,
        "C-r = [\n  \":config-reload\",\n  \":reload\",\n]",
        "managed Helix reload binding",
    );
    expect_order(
        &helix_config,
        &["A-ret = [", "ret = [", "C-j = ["],
        "managed Helix enter movement bindings",
    );

    let helix_steel = embedded_store_path(&helix_script, "-yzx-helix-steel-config");
    let helix_module = fs::read_to_string(helix_steel.join("helix.scm")).unwrap();
    expect_contains_all! {
        &helix_module, "packaged Helix Steel module";
        "(provide yzx-new-shell)",
        "(require (only-in \"helix/static.scm\" cx->current-file get-helix-cwd))",
        "(require (only-in \"helix/commands.scm\" run-shell-command))",
        "(define (yzx-new-shell-command target)",
        "/bin/yzx-open-terminal",
        "(define (yzx-new-shell)",
    }
    assert!(
        !helix_module.contains("recentf"),
        "packaged Helix Steel module still references recentf\n{}",
        excerpt(&helix_module)
    );
    let open_terminal = embedded_store_path(&helix_module, "/bin/yzx-open-terminal");
    let open_terminal_script = fs::read_to_string(&open_terminal).unwrap();
    expect_contains_all! {
        &open_terminal_script, "packaged Helix new-shell helper";
        "zellij action new-pane --cwd",
        "dirname -- \"$target\"",
    }

    let helix_init = fs::read_to_string(helix_steel.join("init.scm")).unwrap();
    expect_contains_all! {
        &helix_init, "packaged Helix Steel init";
        "yzx-helix-start",
        "transport-local-addr",
        "/share/yazelix-helix/steel/yazelix/bridge.scm",
        "/bin/yzx-helix-register",
        "YAZELIX_HELIX_USER_STEEL_INIT",
        "(require (only-in \"helix/misc.scm\" enqueue-thread-local-callback))",
        "forest/forest.scm",
        "forest-configure!",
        "forest-set-toggle-key!",
        "YAZELIX_FOREST_SIDE",
        "YAZELIX_FOREST_TOGGLE_KEY",
        "YAZELIX_FOREST_START_UNFOCUSED",
        "(forest-open #:focused #f)",
        "(load yzx-user-init)",
    }
    expect_order(
        &helix_init,
        &[
            "(forest-set-toggle-key! yzx-forest-toggle-key)",
            "(enqueue-thread-local-callback",
            "(forest-open #:focused #f)",
            "(load yzx-user-init)",
        ],
        "managed Helix Forest startup",
    );
    assert!(
        !helix_init
            .lines()
            .any(|line| line.trim() == "(forest-open)"),
        "managed Helix must not open Forest before its first view exists\n{}",
        excerpt(&helix_init)
    );
    let forest_cogs = embedded_store_path(&helix_script, "-yzx-forest-cogs");
    for module in [
        "forest/forest.scm",
        "forest/core.scm",
        "notify/notify.scm",
        "glyph/glyph.scm",
    ] {
        assert!(
            forest_cogs.join(module).is_file(),
            "managed Helix is missing Forest module {module}"
        );
    }
    let forest_source = fs::read_to_string(forest_cogs.join("forest/forest.scm")).unwrap();
    expect_contains_all! {
        &forest_source, "managed Forest defaults";
        "(define *forest-side* 'left)",
        "(define *forest-style* 'snacks)",
        "(provide forest-set-toggle-key!)",
        "(define (forest-open #:focused [focused? #t])",
        "(hashset \".git\" \"target\" \".direnv\" \"node_modules\" \"__pycache__\" \".hg\")",
    }
    expect_bridge_registry_publisher(&embedded_store_path(&helix_init, "/bin/yzx-helix-register"));

    expect_helix_wrapper_config_selection(&helix_script);
}

fn expect_bridge_registry_publisher(publisher: &Path) {
    let temp = TempDir::new();
    let publisher_command = |endpoint: &str, session_id: &str, instance_id: &str| {
        let mut command = Command::new(publisher);
        command
            .arg(endpoint)
            .env("YAZELIX_STATE_DIR", &temp.path)
            .env("YAZELIX_HELIX_BRIDGE_SESSION_ID", session_id)
            .env("YAZELIX_HELIX_BRIDGE_INSTANCE_ID", instance_id)
            .env("YAZELIX_HELIX_BRIDGE_AUTH_TOKEN", "secret");
        command
    };
    let output = publisher_command("127.0.0.1:4567", "test-session", "hx-test")
        .env("YAZELIX_HELIX_MANAGED_CONFIG_PATH", "/config.toml")
        .env("ZELLIJ_SESSION_NAME", "zellij-test")
        .env("ZELLIJ_TAB_POSITION", "2")
        .env("ZELLIJ_PANE_ID", "terminal:7")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "bridge registry publisher failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let bridge_dir = temp.path.join("helix_bridge/test-session");
    let token_path = bridge_dir.join("hx-test.token");
    let registry_path = bridge_dir.join("hx-test.json");
    assert_eq!(fs::read_to_string(&token_path).unwrap(), "secret");
    assert_eq!(
        fs::metadata(&bridge_dir).unwrap().permissions().mode() & 0o777,
        0o700
    );
    for path in [&token_path, &registry_path] {
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    let registry = fs::read_to_string(registry_path).unwrap();
    expect_contains_all! {
        &registry, "published Helix bridge registry";
        "\"schema_version\": 2",
        "\"session_id\": \"test-session\"",
        "\"instance_id\": \"hx-test\"",
        "\"kind\": \"tcp\"",
        "\"addr\": \"127.0.0.1:4567\"",
        token_path.display().to_string(),
        "\"zellij_session_name\": \"zellij-test\"",
        "\"zellij_tab_position\": \"2\"",
        "\"zellij_pane_id\": \"terminal:7\"",
        "\"managed_config_path\": \"/config.toml\"",
    }

    let rejected = publisher_command("0.0.0.0:4567", "test-session", "hx-rejected")
        .output()
        .unwrap();
    assert!(
        !rejected.status.success(),
        "publisher accepted a non-loopback address"
    );

    let traversal = publisher_command("127.0.0.1:4567", "..", "hx-escaped")
        .output()
        .unwrap();
    assert!(
        !traversal.status.success(),
        "publisher accepted a path-traversing session ID"
    );
}

fn expect_helix_doctor_warnings(yzx: &Path) {
    let yzx_bin = yzx.join("bin/yzx");
    let temp = TempDir::new();

    let default = RuntimeCase::new(&temp.path, "default");
    default.write_default_config("");
    let doctor = default.run_yzx(&yzx_bin, "doctor", "default Helix doctor");
    assert!(
        !doctor.contains("warn helix config:"),
        "default doctor should not warn about packaged Helix config\n{}",
        excerpt(&doctor)
    );

    let helix_override = RuntimeCase::new(&temp.path, "helix-override");
    helix_override.write_default_config("");
    let helix_override_config = helix_override.config_home.join("helix/config.toml");
    fs::create_dir_all(helix_override_config.parent().unwrap()).unwrap();
    fs::write(&helix_override_config, "theme = \"ayu_evolve\"\n").unwrap();
    let doctor = helix_override.run_yzx(&yzx_bin, "doctor", "Helix preference doctor");
    assert!(
        !doctor.contains("warn helix config:"),
        "ordinary Helix preference override should not warn\n{}",
        excerpt(&doctor)
    );

    fs::write(
        &helix_override_config,
        "[keys.normal]\nA-r = \":sh yzx reveal \\\"%{buffer_name}\\\"\"\n",
    )
    .unwrap();
    let doctor = helix_override.run_yzx(&yzx_bin, "doctor", "Helix reveal binding doctor");
    assert!(
        !doctor.contains("warn helix config:"),
        "supported Helix reveal binding should not warn\n{}",
        excerpt(&doctor)
    );

    fs::write(&helix_override_config, "[keys.normal]\nA-r = \":noop\"\n").unwrap();
    let doctor = helix_override.run_yzx(&yzx_bin, "doctor", "Helix Alt r doctor");
    expect_contains_all! {
        &doctor, "Helix Alt r doctor";
        r#"warn helix config: helix config override sets reserved Alt r; generated config keeps ':sh yzx reveal "%{buffer_name}"'"#,
        helix_override_config.display().to_string(),
    }
}

fn expect_helix_wrapper_config_selection(helix_script: &str) {
    const FAKE_HX: &str = "#!/bin/sh\n\
printf 'HELIX_STEEL_CONFIG=%s\\n' \"${HELIX_STEEL_CONFIG-}\" > \"$YZX_FAKE_HX_OUT\"\n\
printf 'STEEL_SEARCH_PATHS=%s\\n' \"${STEEL_SEARCH_PATHS-}\" >> \"$YZX_FAKE_HX_OUT\"\n\
printf 'YAZELIX_HELIX_USER_STEEL_INIT=%s\\n' \"${YAZELIX_HELIX_USER_STEEL_INIT-}\" >> \"$YZX_FAKE_HX_OUT\"\n\
printf 'YAZELIX_FOREST_TOGGLE_KEY=%s\\n' \"${YAZELIX_FOREST_TOGGLE_KEY-}\" >> \"$YZX_FAKE_HX_OUT\"\n\
printf 'YAZELIX_FOREST_START_UNFOCUSED=%s\\n' \"${YAZELIX_FOREST_START_UNFOCUSED-}\" >> \"$YZX_FAKE_HX_OUT\"\n\
printf 'YAZELIX_HELIX_MANAGED_CONFIG_PATH=%s\\n' \"$YAZELIX_HELIX_MANAGED_CONFIG_PATH\" >> \"$YZX_FAKE_HX_OUT\"\n\
for arg do printf 'arg=%s\\n' \"$arg\" >> \"$YZX_FAKE_HX_OUT\"; done\n";

    let temp = TempDir::new();
    let packaged_config = embedded_store_path(helix_script, "-config.toml").join("config.toml");
    let packaged_steel = embedded_store_path(helix_script, "-yzx-helix-steel-config");
    let fake_hx = temp.path.join("hx");
    write_executable(&fake_hx, FAKE_HX);
    let real_hx = embedded_store_path(helix_script, "/bin/hx");
    let test_wrapper = temp.path.join("yzx-hx");
    write_executable(
        &test_wrapper,
        helix_script.replace(real_hx.to_str().unwrap(), fake_hx.to_str().unwrap()),
    );

    for (name, files, uses_user_steel) in [
        ("packaged", &[] as &[(&str, &str)], false),
        (
            "languages",
            &[("languages.toml", "# managed languages\n")] as &[(&str, &str)],
            false,
        ),
        (
            "toml",
            &[(
                "config.toml",
                "[editor]\nline-number = \"relative\"\n\n[keys.normal]\nA-r = \":noop\"\nC-r = \":noop\"\n",
            )] as &[(&str, &str)],
            false,
        ),
        (
            "steel",
            &[("helix.scm", ";; module\n"), ("init.scm", ";; init\n")] as &[(&str, &str)],
            true,
        ),
    ] {
        expect_helix_wrapper_case(
            &test_wrapper,
            &temp.path,
            &packaged_config,
            &packaged_steel,
            name,
            files,
            uses_user_steel,
        );
    }

    for (name, root_config, expected_key, enabled) in [
        (
            "remapped-key",
            "[keybindings]\nsidebar_focus = \"Ctrl Shift E\"\n",
            "C-S-e",
            true,
        ),
        (
            "disabled-key",
            "[keybindings]\nsidebar_focus = false\n",
            "",
            false,
        ),
    ] {
        let home = temp.path.join(format!("{name}-config"));
        fs::create_dir_all(&home).unwrap();
        fs::write(home.join("config.toml"), root_config).unwrap();
        let state = temp.path.join(format!("{name}-state"));
        let output = run_helix_wrapper(
            &test_wrapper,
            &home,
            &state,
            &temp.path.join(format!("{name}-output")),
            &[],
        );
        assert!(
            output.contains(&format!("YAZELIX_FOREST_TOGGLE_KEY={expected_key}\n")),
            "{name} passed the wrong Forest toggle key\n{}",
            excerpt(&output)
        );
        let generated = fs::read_to_string(state.join("helix/config.toml")).unwrap();
        assert_eq!(generated.contains(":forest-open"), enabled);
    }

    let picker_dir = temp.path.join("picker-dir");
    fs::create_dir(&picker_dir).unwrap();
    let file = temp.path.join("file.txt");
    fs::write(&file, "test\n").unwrap();
    for (name, target, expected) in [
        ("directory", picker_dir.as_path(), "1"),
        ("file", file.as_path(), ""),
    ] {
        let output = run_helix_wrapper(
            &test_wrapper,
            &temp.path.join(format!("{name}-config")),
            &temp.path.join(format!("{name}-state")),
            &temp.path.join(format!("{name}-output")),
            &[target],
        );
        expect_contains(
            &output,
            &format!("YAZELIX_FOREST_START_UNFOCUSED={expected}\n"),
            &format!("{name}-start Helix wrapper"),
        );
    }
}

fn expect_helix_wrapper_case(
    wrapper: &Path,
    root: &Path,
    packaged_config: &Path,
    packaged_steel: &Path,
    name: &str,
    files: &[(&str, &str)],
    uses_user_steel: bool,
) {
    let home = root.join(format!("{name}-config"));
    let helix = home.join("helix");
    if !files.is_empty() {
        fs::create_dir_all(&helix).unwrap();
        for (file, contents) in files {
            fs::write(helix.join(file), contents).unwrap();
        }
    }
    let state = root.join(format!("{name}-state"));
    let output = run_helix_wrapper(
        wrapper,
        &home,
        &state,
        &root.join(format!("{name}-output")),
        &[],
    );
    let expected_config_dir = if files.is_empty() {
        packaged_config.parent().unwrap().to_path_buf()
    } else {
        helix.clone()
    };
    let expected_config_file = state.join("helix/config.toml");
    let expected_steel_dir = if uses_user_steel {
        state.join("helix-steel")
    } else {
        packaged_steel.to_path_buf()
    };
    expect_helix_wrapper_output(
        &output,
        &expected_config_dir,
        &expected_config_file,
        &expected_steel_dir,
        &format!("{name} Helix config selection"),
    );
    assert!(
        expected_steel_dir.is_dir(),
        "{name} Helix config should select an existing Steel directory"
    );
    let expected_user_init = if uses_user_steel {
        helix.join("init.scm").display().to_string()
    } else {
        String::new()
    };
    assert!(
        output.contains(&format!(
            "YAZELIX_HELIX_USER_STEEL_INIT={expected_user_init}\n"
        )),
        "{name} Helix config selected the wrong user Steel init\n{}",
        excerpt(&output)
    );
    assert!(
        output.contains("YAZELIX_FOREST_TOGGLE_KEY=C-y\n"),
        "{name} Helix config passed the wrong default Forest toggle key\n{}",
        excerpt(&output)
    );
    if uses_user_steel {
        assert_eq!(
            fs::canonicalize(expected_steel_dir.join("helix.scm")).unwrap(),
            fs::canonicalize(helix.join("helix.scm")).unwrap()
        );
        assert_eq!(
            fs::canonicalize(expected_steel_dir.join("init.scm")).unwrap(),
            fs::canonicalize(packaged_steel.join("init.scm")).unwrap()
        );
    }
    let generated_config = fs::read_to_string(&expected_config_file).unwrap();
    expect_contains_all! {
        &generated_config, &format!("{name} generated Helix reveal binding");
        "A-r = ",
        ":sh yzx reveal",
        "%{buffer_name}",
        "C-y = ",
        ":forest-open",
    }
    if name == "toml" {
        expect_contains_all! {
            &generated_config, "user Helix TOML merge";
            "line-number = \"relative\"",
            "C-r = \":noop\"",
        }
        assert!(
            !generated_config.contains("A-r = \":noop\""),
            "generated config kept user Alt r override\n{}",
            excerpt(&generated_config)
        );
    }
}

fn run_helix_wrapper(
    wrapper: &Path,
    config_home: &Path,
    state_dir: &Path,
    output_path: &Path,
    args: &[&Path],
) -> String {
    let output = Command::new(wrapper)
        .args(args)
        .env("YAZELIX_CONFIG_HOME", config_home)
        .env("YAZELIX_STATE_DIR", state_dir)
        .env("YZX_FAKE_HX_OUT", output_path)
        .env("YAZELIX_HELIX_USER_STEEL_INIT", "/ambient/init.scm")
        .env("YAZELIX_FOREST_TOGGLE_KEY", "A-x")
        .env_remove("HELIX_STEEL_CONFIG")
        .env_remove("STEEL_SEARCH_PATHS")
        .env_remove("YAZELIX_HELIX_MANAGED_CONFIG_PATH")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "Helix wrapper failed: stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::read_to_string(output_path).unwrap()
}

fn expect_helix_wrapper_output(
    output: &str,
    config_dir: &Path,
    config_file: &Path,
    steel_dir: &Path,
    context: &str,
) {
    let steel_line = format!("HELIX_STEEL_CONFIG={}\n", steel_dir.display());
    let managed_line = format!(
        "YAZELIX_HELIX_MANAGED_CONFIG_PATH={}",
        config_file.display()
    );
    let config_dir_arg = format!("arg={}", config_dir.display());
    let config_file_arg = format!("arg={}", config_file.display());
    expect_contains_all! {
        output, context;
        steel_line,
        "STEEL_SEARCH_PATHS=/nix/store/",
        "-yzx-forest-cogs",
        managed_line,
    }
    expect_order(
        output,
        &[
            "arg=--config-dir",
            config_dir_arg.as_str(),
            "arg=-c",
            config_file_arg.as_str(),
        ],
        context,
    );
}
