{
  coreutils,
  nushell,
  runtimeIdentity,
  novaBar,
  portableRuntime ? false,
}: {
  appearanceMode,
  widgetTray,
  shellLabel,
}: {
  zjstatus_plugin_url = "zellij:nova-bar";
  widget_tray = widgetTray;
  widget_frame = "none";
  widget_separator = "dot";
  editor_label = "hx";
  shell_label = shellLabel;
  terminal_label = "rio";
  custom_text = "";
  appearance_mode = appearanceMode;
  tab_label_mode = "full";
  nu_bin =
    if portableRuntime
    then "__YZX_RUNTIME_ROOT__/libexec/yazelix/nu"
    else "${nushell}/bin/nu";
  yzx_control_bin =
    if portableRuntime
    then "__YZX_RUNTIME_ROOT__/libexec/yazelix/false"
    else "${coreutils}/bin/false";
  nova_bar_widget_bin =
    if portableRuntime
    then "__YZX_RUNTIME_ROOT__/libexec/yazelix/nova-bar-widget"
    else "${novaBar}/${novaBar.widgetPath}";
  runtime_dir =
    if portableRuntime
    then "__YZX_RUNTIME_ROOT__/share/yazelix"
    else "${runtimeIdentity}";
  claude_usage_display = "both";
  claude_usage_periods = ["5h" "week"];
  codex_usage_display = "quota";
  codex_usage_periods = ["5h" "week"];
  opencode_go_usage_display = "both";
  opencode_go_usage_periods = ["5h" "week" "month"];
}
