use std::{ffi::OsStr, path::Path};

use zellij::Distribution;

fn main() {
    let mut args = std::env::args_os();
    if args
        .next()
        .is_some_and(|arg| Path::new(&arg).file_name() == Some(OsStr::new("zellij")))
        && args.eq(["--version"])
    {
        println!("zellij {}", Distribution::zellij().version);
        return;
    }
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
