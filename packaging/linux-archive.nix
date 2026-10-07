{
  pkgs,
  runtime,
  version,
}: let
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
    mkdir -p "$out/lib/components"
    while IFS= read -r directory; do
      cp -Lf --remove-destination "$directory"/* "$out/libexec/yazelix/"
      package=$(dirname "$directory")
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
      cp -RL "$fonts" "$out/share/fonts/$(basename "$fonts")"
    done
    chmod -R u+w "$out/share/fontconfig"
    sed -i -E 's|<dir>/nix/store/([^<]*)</dir>|<dir prefix="relative">../fonts/\1</dir>|g' "$out/share/fontconfig/fonts.conf"
    sed -i 's|/etc/fonts/conf.d|conf.d|' "$out/share/fontconfig/fonts.conf"
    magick_config="$out/lib/components/imagemagick/etc/ImageMagick-7"
    grep -oE '/nix/store/[^ /&";]+/bin/[a-zA-Z0-9_.+-]+' "$magick_config/delegates.xml" | sort -u | while IFS= read -r command; do
      cp -Lf --remove-destination "$command" "$out/libexec/yazelix/"
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
    payload = prepared;
  } "bash ${./linux-relocate.sh}";
in
  pkgs.runCommand "yazelix-no-rio-archive-${version}" {
    passthru.runtime = payload;
  } ''
    mkdir -p "$out"
    tar --sort=name --mtime=@1 --owner=0 --group=0 --numeric-owner -C ${payload} -cf - . | gzip -n > "$out/yazelix-nova-${version}-x86_64-linux.tar.gz"
  ''
