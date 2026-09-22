use zellij::Distribution;

fn main() {
    zellij::run(
        Distribution::new("yzx-zellij-prototype", env!("CARGO_PKG_VERSION"))
            .with_display_name("Yazelix Zellij Prototype")
            .with_plugin("yzpp", include_bytes!(env!("YZX_PROTO_YZPP_WASM")))
            .with_plugin(
                "yazelix_pane_orchestrator",
                include_bytes!(env!("YZX_PROTO_ORCHESTRATOR_WASM")),
            )
            .with_plugin("radar", include_bytes!(env!("YZX_PROTO_RADAR_WASM")))
            .with_plugin("nova-bar", include_bytes!(env!("YZX_PROTO_BAR_WASM"))),
    );
}
