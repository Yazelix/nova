{pkgs}: let
  # Eon's pinned CodexBar release (source 7c08c0ae783b72040b728757eb85a6cfe4fc2443).
  # Hashes match each archive's published .sha256; x86_64 Linux matches Eon's pin.
  version = "0.74.1";
  revision = "7c08c0ae783b72040b728757eb85a6cfe4fc2443";
  targets = {
    x86_64-linux = {
      asset = "linux-musl-x86_64";
      hash = "sha256-oC26jOydDYucyteY5LxuonAbfIAcud1z9vzf7b5QfcY=";
    };
    aarch64-linux = {
      asset = "linux-musl-aarch64";
      hash = "sha256-ksS7SWSvkP3cG2wlrvXkBHHu2b4nAL6KbH3HgKNqGXA=";
    };
    x86_64-darwin = {
      asset = "macos-x86_64";
      hash = "sha256-0deEEuu2Y6R6uclyq6gXev7NMg+4YXo2z8rVUmfWImM=";
    };
    aarch64-darwin = {
      asset = "macos-arm64";
      hash = "sha256-nblQWO3ciCQdIoUVowr2hoygZ7qaJkUuRkLvWeL1T1I=";
    };
  };
  target = targets.${pkgs.stdenv.hostPlatform.system};
  license = pkgs.fetchurl {
    url = "https://raw.githubusercontent.com/steipete/CodexBar/${revision}/LICENSE";
    hash = "sha256-FCk1VreZQHRRI9AWDHHSftDp/puKhICT8+149IU8qv4=";
  };
in
  pkgs.stdenvNoCC.mkDerivation {
    pname = "codexbar";
    inherit version;
    src = pkgs.fetchurl {
      url = "https://github.com/steipete/CodexBar/releases/download/v${version}/CodexBarCLI-v${version}-${target.asset}.tar.gz";
      inherit (target) hash;
    };
    sourceRoot = ".";
    dontBuild = true;
    dontStrip = true;
    # Only the executable ships: Codex and Claude reads need none of the bundled provider plugins,
    # and copying runtime commands from bin/ must not meet a resource directory.
    installPhase = ''
      runHook preInstall
      test "$(cat VERSION)" = "$version"
      install -D -m 755 CodexBarCLI "$out/bin/codexbar"
      install -D -m 444 ${license} "$out/share/licenses/codexbar/LICENSE"
      runHook postInstall
    '';
    doInstallCheck = true;
    # The archive's VERSION pins identity; without it beside the binary, --version prints only the name.
    installCheckPhase = ''
      test "$("$out/bin/codexbar" --version)" = CodexBar
    '';
    meta = {
      description = "CodexBar CLI for provider quota windows";
      homepage = "https://github.com/steipete/CodexBar";
      license = pkgs.lib.licenses.mit;
      sourceProvenance = [pkgs.lib.sourceTypes.binaryNativeCode];
      platforms = builtins.attrNames targets;
      mainProgram = "codexbar";
    };
  }
