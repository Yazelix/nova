{
  pkgs,
  runtime,
  version,
  revision,
  sourceRoot,
  glibcBaseline,
  libraryReplacements,
}: let
  compatibility = {
    libc = {
      family = "glibc";
      minimum = glibcBaseline;
    };
    interpreter = "/lib64/ld-linux-x86-64.so.2";
    host_commands = ["git"];
    environment_launcher = {
      path = "/usr/bin/env";
      argument_split = true;
    };
    allowed_sonames = [
      "ld-linux-x86-64.so.2"
      "libc.so.6"
      "libm.so.6"
      "libmvec.so.1"
      "libdl.so.2"
      "libpthread.so.0"
      "librt.so.1"
      "libresolv.so.2"
      "libanl.so.1"
      "libutil.so.1"
      "libgcc_s.so.1"
    ];
    symbol_versions."libgcc_s.so.1" = ["GCC_3.0" "GCC_3.3" "GCC_3.3.1" "GCC_3.4" "GCC_4.0.0" "GCC_4.2.0" "GCC_4.3.0"];
  };
  prepared = pkgs.runCommand "nova-linux-payload" {} ''
    cp -RL ${runtime} "$out"
    chmod -R u+w "$out"
    cmp "$out/libexec/yazelix/zellij" "$out/libexec/yazelix/yzx-zellij"
    cmp "$out/bin/yzx-zellij" "$out/libexec/yazelix/yzx-zellij"
    ln -sf yzx-zellij "$out/libexec/yazelix/zellij"
    ln -sf ../libexec/yazelix/yzx-zellij "$out/bin/yzx-zellij"
    wrapper="$out/libexec/yazelix/yazi"
    target=$(sed -n 's/^exec "\([^"]*\)" .*$/\1/p' "$wrapper")
    test -f "$target"
    sed -n "s/^PATH='\([^']*\)'\$PATH$/\1/p" "$wrapper" > "$TMPDIR/yazi-paths"
    test -s "$TMPDIR/yazi-paths"
    basename ${runtime} > "$out/share/yazelix/archive-origins.txt"
    mkdir -p "$out/lib/components"
    while IFS= read -r directory; do
      cp -L --update=none "$directory"/* "$out/libexec/yazelix/"
      package=$(dirname "$directory")
      basename "$package" >> "$out/share/yazelix/archive-origins.txt"
      cp -RL "$package" "$out/lib/components/$(basename "$package")"
      if test -x "$directory/magick"; then
        ln -s "$(basename "$package")" "$out/lib/components/imagemagick"
      fi
    done < "$TMPDIR/yazi-paths"
    cp --remove-destination "$target" "$wrapper"
    cp -RL ${pkgs.ncurses}/share/terminfo "$out/share/terminfo"
    cp -RL ${pkgs.file}/share/misc "$out/share/misc"
    cp -RL ${pkgs.fontconfig.out}/etc/fonts "$out/share/fontconfig"
    mkdir -p "$out/share/fonts"
    sed -n 's|.*<dir>\(/nix/store/[^<]*\)</dir>.*|\1|p' "$out/share/fontconfig/fonts.conf" | while IFS= read -r fonts; do
      origin=''${fonts#/nix/store/}
      printf '%s\n' "''${origin%%/*}" >> "$out/share/yazelix/archive-origins.txt"
      cp -RL "$fonts" "$out/share/fonts/$(basename "$fonts")"
    done
    chmod -R u+w "$out/share/fontconfig"
    sed -i -E 's|<dir>/nix/store/([^<]*)</dir>|<dir prefix="relative">../fonts/\1</dir>|g' "$out/share/fontconfig/fonts.conf"
    sed -i 's|/etc/fonts/conf.d|conf.d|' "$out/share/fontconfig/fonts.conf"
    magick_config="$out/lib/components/imagemagick/etc/ImageMagick-7"
    grep -oE '/nix/store/[^ /&";]+/bin/[a-zA-Z0-9_.+-]+' "$magick_config/delegates.xml" | sort -u | while IFS= read -r command; do
      # Auxiliary discovery must preserve the runtime's selected commands.
      origin=''${command#/nix/store/}
      printf '%s\n' "''${origin%%/*}" >> "$out/share/yazelix/archive-origins.txt"
      cp -L --update=none "$command" "$out/libexec/yazelix/"
    done
    chmod -R u+w "$magick_config"
    sed -i -E 's|/nix/store/[^ /&";]+/bin/([a-zA-Z0-9_.+-]+)|\1|g' "$magick_config/delegates.xml"
    cp -RL ${pkgs.fish}/share/fish "$out/share/fish"
    mkdir -p "$out/etc"
    cp -RL ${pkgs.fish}/etc/fish "$out/etc/fish"
    mv "$out/libexec/yazelix/fish" "$out/bin/fish"
    ln -s ../../bin/fish "$out/libexec/yazelix/fish"
    cp ${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt "$out/share/yazelix/ca-bundle.crt"
    cp -RL ${pkgs.openssl.out}/etc/ssl "$out/share/openssl"
    cp -RL ${pkgs.openssl.out}/lib/ossl-modules "$out/lib/openssl-modules"
    cp -RL ${pkgs.openssl.out}/lib/engines-3 "$out/lib/openssl-engines"
    chmod -R u+w "$out"
    : > "$out/share/yazelix/native-inputs.txt"
    find "$out/libexec/yazelix" -type f -print0 | while IFS= read -r -d "" path; do
      if [ "$(head -c2 "$path")" = '#!' ]; then
        interpreter=$(head -1 "$path" | sed -E 's|^#![[:space:]]*[^ ]*/(env )?||')
        program=''${interpreter%% *}
        test -x "$out/libexec/yazelix/$program" || {
          echo "missing script interpreter $program for $path" >&2
          exit 1
        }
        printf 'libexec/yazelix/%s\n' "$program" >> "$out/share/yazelix/native-inputs.txt"
        sed -i "1c#!/usr/bin/env -S $interpreter" "$path"
      fi
    done
  '';
  payload = pkgs.runCommand "nova-linux-runtime" {
    nativeBuildInputs = [pkgs.patchelf pkgs.binutils];
    inherit glibcBaseline libraryReplacements;
    inherit (compatibility) interpreter;
    hostLibraries = pkgs.lib.concatStringsSep " " compatibility.allowed_sonames;
    libgccSymbolVersions = pkgs.lib.concatStringsSep " " compatibility.symbol_versions."libgcc_s.so.1";
    payload = prepared;
  } "bash ${./linux-relocate.sh}";
  identity = pkgs.writeText "nova-archive-identity.json" (builtins.toJSON {
    schema_version = 1;
    inherit revision compatibility;
    target = "x86_64-unknown-linux-gnu";
    variant = "no-rio";
    entrypoints = ["bin/yzx"];
  });
in
  pkgs.runCommand "yazelix-no-rio-archive-${version}" {
    nativeBuildInputs = [pkgs.python3];
    novaSourceRevision = if revision == null then "" else revision;
    novaSourceRoot = builtins.baseNameOf (toString sourceRoot);
    passthru = {
      runtime = payload;
      inherit compatibility;
    };
  } ''
    mkdir -p "$out"
    cp -a ${payload} "$TMPDIR/indexed"
    chmod u+w "$TMPDIR/indexed/share/yazelix"
    python3 ${./archive_metadata.py} "$TMPDIR/indexed" ${identity}
    chmod --reference=${payload}/share/yazelix "$TMPDIR/indexed/share/yazelix"
    tar --sort=name --mtime=@1 --owner=0 --group=0 --numeric-owner -C "$TMPDIR/indexed" -cf - . | gzip -n > "$out/yazelix-nova-${version}-x86_64-unknown-linux-gnu-no-rio.tar.gz"
  ''
