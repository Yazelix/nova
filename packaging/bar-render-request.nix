{
  coreutils,
  nushell,
  runtimeIdentity,
  novaBar,
}: {
  appearanceMode,
  widgetTray,
  shellLabel,
}: {
  zjstatus_plugin_url = "file:${novaBar}/${novaBar.wasmPath}";
  widget_tray = widgetTray;
  widget_frame = "none";
  widget_separator = "dot";
  editor_label = "hx";
  shell_label = shellLabel;
  terminal_label = "rio";
  custom_text = "";
  appearance_mode = appearanceMode;
  tab_label_mode = "full";
  nu_bin = "${nushell}/bin/nu";
  yzx_control_bin = "${coreutils}/bin/false";
  nova_bar_widget_bin = "${novaBar}/${novaBar.widgetPath}";
  runtime_dir = "${runtimeIdentity}";
  claude_usage_display = "both";
  claude_usage_periods = ["5h" "week"];
  codex_usage_display = "quota";
  codex_usage_periods = ["5h" "week"];
  opencode_go_usage_display = "both";
  opencode_go_usage_periods = ["5h" "week" "month"];
}
