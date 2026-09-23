use zellij::Distribution;

fn main() {
    zellij::run(
        Distribution::new("yzx-zellij", env!("CARGO_PKG_VERSION"))
            .with_display_name("Yazelix Nova")
            .with_plugin("yzpp", include_bytes!(env!("YZX_YZPP_WASM")))
            .with_plugin(
                "yazelix_pane_orchestrator",
                include_bytes!(env!("YZX_ORCHESTRATOR_WASM")),
            )
            .with_plugin("radar", include_bytes!(env!("YZX_RADAR_WASM")))
            .with_plugin("nova-bar", include_bytes!(env!("YZX_BAR_WASM")))
            .with_plugin("nova-zjhints", include_bytes!(env!("YZX_HINTS_WASM"))),
    );
}
