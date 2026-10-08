{
  pkgs,
  linuxLibc,
}: let
  libc = (import linuxLibc {system = "x86_64-linux";}).glibc;
  cc = pkgs.stdenv.cc.override {
    inherit libc;
    # GCC otherwise searches the newer libc headers compiled into its binary.
    nixSupport.cc-cflags = "--sysroot=${libc}";
    bintools = pkgs.stdenv.cc.bintools.override {inherit libc;};
  };
  stdenv = pkgs.overrideCC pkgs.stdenv cc;
  replacements = builtins.mapAttrs (_: package: package.override {inherit stdenv;}) {
    inherit (pkgs) coreutils ncurses libssh librist;
  };
in {
  glibcBaseline = "2.39";
  # Select runtime providers without changing Nixpkgs' compiler/build graph.
  pkgs = pkgs // replacements;
  libraryReplacements = pkgs.writeText "nova-linux-library-providers" (pkgs.lib.concatMapStrings
    (name: "${pkgs.${name}.out}\t${replacements.${name}.out}\n")
    ["ncurses" "libssh" "librist"]);
}
