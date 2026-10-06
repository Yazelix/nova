mod cli;
mod command;
mod doctor;
mod error;
mod package;
mod paths;
mod runtime;
mod status;
mod yazi;
mod zellij;

use std::process;

pub(crate) const YZX_CONFIG_UI: (&str, &str) = ("@yzxConfigUi@", "libexec/yazelix/yzx-config-ui");
pub(crate) const YZX_AGENT: (&str, &str) = ("@yzxAgent@", "libexec/yazelix/yzx-agent");
pub(crate) const YZX_MENU: (&str, &str) = ("@yzxMenu@", "libexec/yazelix/yzx-menu");
pub(crate) const YZX_TUTOR: (&str, &str) = ("@yzxTutor@", "libexec/yazelix/yzx-tutor");
pub(crate) const YZX_SCREEN: (&str, &str) = ("@yzxScreen@", "libexec/yazelix/anima");
pub(crate) const YZX_WELCOME: (&str, &str) = ("@yzxWelcome@", "libexec/yazelix/yzx-welcome");
pub(crate) const YZX_SHELL: (&str, &str) = ("@yzxShell@", "libexec/yazelix/yzx-shell");
pub(crate) const YZX_ENV_SUPERVISOR: (&str, &str) =
    ("@yzxEnvSupervisor@", "libexec/yazelix/yzx-env-supervisor");
pub(crate) const ZELLIJ: (&str, &str) = ("@zellij@", "libexec/yazelix/yzx-zellij");
pub(crate) const RIO: (&str, &str) = ("@rio@", "libexec/yazelix/rio");
pub(crate) const PACKAGE_VARIANT: &str = "@packageVariant@";
pub(crate) const MANAGED_HELIX: &str = "@managedHelix@";
pub(crate) const LAYOUT: (&str, &str) = ("@layout@", "share/yazelix/layout.kdl");
pub(crate) const LAYOUT_TEMPLATE: (&str, &str) =
    ("@layoutTemplate@", "share/yazelix/layout.template.kdl");
pub(crate) const LAYOUT_SWAP_TEMPLATE: (&str, &str) = (
    "@layoutSwapTemplate@",
    "share/yazelix/layout.swap.template.kdl",
);
pub(crate) const YZX_YAZI: (&str, &str) = ("@yzxYazi@", "libexec/yazelix/yzx-yazi");
pub(crate) const YZX_HELIX: (&str, &str) = ("@yzxHelix@", "libexec/yazelix/yzx-hx");
pub(crate) const YZX_EDITOR: (&str, &str) = ("@yzxEditor@", "libexec/yazelix/yzx-editor");
pub(crate) const YZX_CONFIG: (&str, &str) = ("@yzxConfig@", "libexec/yazelix/yzx-config");
pub(crate) const YZX_ZELLIJ_CONFIG: (&str, &str) =
    ("@yzxZellijConfig@", "libexec/yazelix/yzx-zellij-config");
pub(crate) const YZX_CONFIG_KDL: (&str, &str) = ("@yzxConfigKdl@", "share/yazelix/config.kdl");
pub(crate) const YZX_YAZI_CONFIG: (&str, &str) = ("@yzxYaziConfig@", "share/yazelix/yazi");
pub(crate) const YZX_YAZI_MATERIALIZER: (&str, &str) =
    ("@yzxYaziMaterializer@", "libexec/yazelix/yzx-yazi-config");
pub(crate) const YZX_REVEAL: (&str, &str) = ("@yzxReveal@", "libexec/yazelix/yzx-reveal");
pub(crate) const YAZI_SOURCE: &str = "@yaziSource@";
pub(crate) const YAZI_COMMAND: (&str, &str) = ("@yaziCommand@", "libexec/yazelix/yazi");
pub(crate) const YA_COMMAND: (&str, &str) = ("@yaCommand@", "libexec/yazelix/ya");
pub(crate) const YAZI_TESTED_VERSION: &str = "@yaziTestedVersion@";
pub(crate) const YZX_BAR_RENDER_REQUEST: (&str, &str) = (
    "@yzxBarRenderRequest@",
    "share/yazelix/bar-render-request.json",
);
pub(crate) const YZX_BAR_RENDER: (&str, &str) =
    ("@yzxBarRender@", "libexec/yazelix/yzx-bar-render");
pub(crate) const DEFAULT_BAR_WIDGETS_JSON: &str = r#"@defaultBarWidgetsJson@"#;
pub(crate) const DEFAULT_SHELL_PROGRAM: &str = "@defaultShellProgram@";
pub(crate) const DEFAULT_POPUP_SIDE_MARGIN: &str = "@defaultPopupSideMargin@";
pub(crate) const DEFAULT_POPUP_VERTICAL_MARGIN: &str = "@defaultPopupVerticalMargin@";
pub(crate) const AGENT_POPUP_KDL_CONFIG_PATH: &str = "agent.popup.kdl";
pub(crate) const AGENT_AUTO_COMMAND: &str = "auto";
pub(crate) const SIDEBAR_PANE_KDL_CONFIG_PATH: &str = "sidebar.pane.kdl";
pub(crate) const SIDEBAR_RADAR_COMMAND: &str = "radar";
pub(crate) const CUSTOM_POPUPS_KDL_CONFIG_PATH: &str = "popups.kdl";
pub(crate) const CUSTOM_POPUP_KEYBINDINGS_KDL_CONFIG_PATH: &str = "popups.keybindings.kdl";
pub(crate) const PACKAGE_HELPER: (&str, &str) = ("@packageHelper@", "libexec/yazelix/yzx-package");
pub(crate) const PORTABLE_RUNTIME: bool = cfg!(yzx_portable);
pub(crate) const PATH_PREFIX: &str = "@pathPrefix@";
pub(crate) const VERSION: &str = "@version@";
pub(crate) const ZELLIJ_HOME_PLACEHOLDER: &str = "\"__YZX_HOME__\"";
pub(crate) const LAYOUT_YAZI_PLACEHOLDER: &str = concat!("@", "yazi", "@");
pub(crate) const LAYOUT_BAR_PLACEHOLDER: &str = concat!("@", "bar", "@");
pub(crate) const LAYOUT_SIDEBAR_PLACEHOLDER: &str = concat!("@", "sidebar", "@");
pub(crate) const LAYOUT_BOTTOM_HINTS_PLACEHOLDER: &str = concat!("@", "bottomHintsStartTitle", "@");
pub(crate) const HELIX_REVEAL_COMMAND: &str = r#":sh yzx reveal "%{buffer_name}""#;
pub(crate) const MANAGED_KEYBINDING_SPECS: &[(&str, &str, &str)] = &[
    ("config", "keybindings.config", "@defaultConfigKeybinding@"),
    ("agent", "keybindings.agent", "@defaultAgentKeybinding@"),
    ("git", "keybindings.git", "@defaultGitKeybinding@"),
    ("menu", "keybindings.menu", "@defaultMenuKeybinding@"),
    ("screen", "keybindings.screen", "@defaultScreenKeybinding@"),
    (
        "bottom hints",
        "keybindings.bottom_hints",
        "@defaultBottomHintsKeybinding@",
    ),
    (
        "sidebar",
        "keybindings.sidebar",
        "@defaultSidebarKeybinding@",
    ),
    (
        "forest",
        "keybindings.sidebar_focus",
        "@defaultSidebarFocusKeybinding@",
    ),
];

fn main() {
    process::exit(
        cli::run()
            .map(|()| 0)
            .unwrap_or_else(error::AppError::report),
    );
}
