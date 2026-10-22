#!/bin/sh
# NOVA-INSTALL-001 (docs/portable-runtime.md): install Nova's portable release archive without Nix.
# Everything runs from the `main "$@"` call on the last line, so a truncated download executes nothing.
set -eu

releases=https://github.com/Yazelix/nova/releases
target=x86_64-unknown-linux-gnu
variant=no-rio

say() { printf 'nova-install: %s\n' "$*"; }
fail() {
    printf 'nova-install: error: %s\n' "$*" >&2
    exit 1
}

usage() {
    cat <<'EOF'
Install Yazelix Nova from its GitHub release, without Nix.

Usage: install.sh [--version X.Y.Z] [--install-root DIR] [--bin-dir DIR]
       install.sh --uninstall [--install-root DIR] [--bin-dir DIR]

  --version X.Y.Z     install this exact release instead of the latest one
  --install-root DIR  installer-owned versions (default: ${XDG_DATA_HOME:-~/.local/share}/yazelix-nova)
  --bin-dir DIR       directory for the yzx link (default: ~/.local/bin)
  --uninstall         remove installer-owned versions and the yzx link; keep configuration and state

Re-running the installer upgrades; installing an earlier --version rolls back.
EOF
}

absolute() {
    case $1 in
        /*) printf '%s' "$1" ;;
        *) printf '%s/%s' "$PWD" "$1" ;;
    esac
}

fetch() {
    curl --proto '=https' --tlsv1.2 --fail --location --show-error --retry 3 "$@"
}

# Fetch a release file and require the hash that SHA256SUMS lists for it.
verified() {
    hash=$(awk -v name="$1" '$2 == name { print $1 }' "$work/SHA256SUMS")
    [ -n "$hash" ] || fail "release $version lists no checksum for $1"
    fetch "$2" --output "$work/$1" "$url/$1" || fail "could not download $1 from release $version"
    actual=$(sha256sum < "$work/$1")
    [ "${actual%% *}" = "$hash" ] || fail "$1 does not match its published SHA-256; nothing was installed"
}

glibc_check() {
    minimum=$(sed -n 's/^ *"minimum": "\([0-9][0-9]*\.[0-9][0-9]*\)",*$/\1/p' "$work/nova-release.json")
    [ -n "$minimum" ] || fail "release $version has no glibc requirement this installer understands"
    host=$(getconf GNU_LIBC_VERSION 2>/dev/null) || fail "Nova's Linux archive needs glibc $minimum or newer; this system reports no glibc"
    host=${host#glibc }
    need_major=${minimum%%.*} need_minor=${minimum#*.}
    major=${host%%.*} minor=${host#*.}
    minor=${minor%%.*}
    if [ "$major" -lt "$need_major" ] || { [ "$major" -eq "$need_major" ] && [ "$minor" -lt "$need_minor" ]; }; then
        fail "Nova $version needs glibc $minimum or newer; this system has glibc $host"
    fi
}

path_help() {
    say "$bin is not on PATH. Run Nova now as: $link"
    case $(basename "${SHELL:-sh}") in
        bash) say "To add it for bash, append to ~/.bashrc:  export PATH=\"$bin:\$PATH\"" ;;
        zsh) say "To add it for zsh, append to ~/.zshrc:  export PATH=\"$bin:\$PATH\"" ;;
        fish) say "To add it for fish, run once:  fish_add_path '$bin'" ;;
        nu) say "To add it for Nushell, append to your config.nu:  \$env.PATH = (\$env.PATH | prepend '$bin')" ;;
        *) say "To add it, append to ~/.profile:  export PATH=\"$bin:\$PATH\"" ;;
    esac
}

# Shells run the first yzx on PATH; name any other install so the choice is visible.
other_yzx() {
    other=$(
        IFS=:
        set -f
        for dir in $PATH; do
            [ ! -x "$dir/yzx" ] || [ "$(readlink "$dir/yzx")" = "$selected" ] || { printf '%s' "$dir/yzx" && break; }
        done
    )
    [ -n "$other" ] || return 0
    if [ "$(readlink "$(command -v yzx)")" = "$selected" ]; then
        say "yzx now runs this install, ahead of $other on PATH"
    else
        say "yzx still runs $other, which PATH lists first; run $link or remove the other install"
    fi
}

cleanup() {
    if [ -n "${work:-}" ] && [ -d "$work" ]; then
        chmod -R u+w "$work" 2>/dev/null || true
        rm -rf "$work"
    fi
}

uninstall() {
    [ -f "$marker" ] || fail "$root was not created by this installer; nothing was removed"
    if [ -L "$link" ] && [ "$(readlink "$link")" = "$selected" ]; then
        rm "$link"
        say "removed $link"
    elif [ -e "$link" ] || [ -L "$link" ]; then
        say "kept $link: it does not point into $root"
    fi
    for path in "$root"/versions "$root"/.install.*; do
        [ ! -e "$path" ] || chmod -R u+w "$path"
    done
    rm -rf "$root/versions" "$root/current" "$root"/.install.* "$marker"
    rmdir "$root" 2>/dev/null || say "kept $root: it holds files this installer did not create"
    say "removed Nova; your configuration and state are untouched"
}

install() {
    case "$(uname -s) $(uname -m)" in
        "Linux x86_64") ;;
        Darwin*) fail "the Nix-free archive does not support macOS yet; install Nova with Nix: https://nova.yazelix.com/docs/" ;;
        *) fail "Nova publishes no archive for $(uname -s) $(uname -m); the Nix-free target is x86_64 Linux" ;;
    esac
    for tool in curl tar sha256sum mktemp; do
        command -v "$tool" >/dev/null 2>&1 || fail "the installer needs $tool"
    done
    if [ -e "$link" ] || [ -L "$link" ]; then
        points=$(readlink "$link") || points=''
        [ "$points" = "$selected" ] ||
            fail "$link exists and is not this installer's link${points:+ (it points to $points)}; move it or pass --bin-dir"
    fi
    if [ ! -f "$marker" ] && [ -d "$root" ] && [ -n "$(ls -A "$root")" ]; then
        fail "$root is not empty and was not created by this installer; pass another --install-root"
    fi
    mkdir -p "$root/versions"
    printf 'Yazelix Nova installer root\n' > "$marker"
    work=$(mktemp -d "$root/.install.XXXXXX")
    trap cleanup EXIT
    trap 'exit 1' INT TERM

    if [ -n "$version" ]; then
        url=$releases/download/nova-v$version
    else
        url=$releases/latest/download
        version=latest
    fi
    fetch --silent --output "$work/SHA256SUMS" "$url/SHA256SUMS" || fail "could not download release $version"
    name=$(grep -E "^[0-9a-f]{64}  yazelix-nova-[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?-$target-$variant\.tar\.gz$" "$work/SHA256SUMS" | cut -c67-)
    [ "$(printf '%s\n' "$name" | grep -c .)" = 1 ] || fail "release $version has no single $target archive"
    found=${name#yazelix-nova-}
    found=${found%-"$target-$variant".tar.gz}
    [ "$version" = latest ] || [ "$version" = "$found" ] || fail "release $version lists the archive for $found"
    version=$found
    url=$releases/download/nova-v$version

    verified nova-release.json --silent
    glibc_check
    say "downloading Yazelix Nova $version"
    verified "$name" --progress-bar
    mkdir "$work/payload"
    tar -xzf "$work/$name" -C "$work/payload" || fail "could not extract $name; nothing was installed"
    payload=$work/payload
    grep -Fqx -e "  \"version\": \"$version\"" -e "  \"version\": \"$version\"," "$payload/share/yazelix/package.json" 2>/dev/null ||
        fail "$name does not contain Nova $version; nothing was installed"
    if ! "$payload/bin/yzx" --version < /dev/null > "$work/smoke.log" 2>&1; then
        cat "$work/smoke.log" >&2
        fail "Nova $version did not start on this system; nothing was installed"
    fi

    destination=$root/versions/$version
    chmod u+w "$payload"
    if [ -e "$destination" ]; then
        chmod u+w "$destination"
        mv "$destination" "$work/previous"
    fi
    mv "$payload" "$destination"
    chmod u-w "$destination"
    ln -s "versions/$version" "$work/current"
    mv -T "$work/current" "$root/current"
    mkdir -p "$bin"
    [ -L "$link" ] || ln -s "$selected" "$link"

    say "installed Yazelix Nova $version in $destination"
    case ":$PATH:" in
        *":$bin:"*) ;;
        *) path_help ;;
    esac
    other_yzx
    say "Check the host with 'yzx doctor', then open a workspace in this terminal with 'yzx enter'."
    say "Release provenance: download $url/$name and run 'gh attestation verify $name --repo Yazelix/nova'."
}

main() {
    version='' root='' bin='' action=install
    while [ $# -gt 0 ]; do
        case $1 in
            --version | --install-root | --bin-dir)
                [ $# -ge 2 ] || fail "$1 needs a value"
                case $1 in
                    --version) version=$2 ;;
                    --install-root) root=$2 ;;
                    --bin-dir) bin=$2 ;;
                esac
                shift 2
                ;;
            --uninstall) action=uninstall && shift ;;
            -h | --help) usage && exit 0 ;;
            *) fail "unknown option $1 (see --help)" ;;
        esac
    done
    if [ -n "$version" ] && ! printf '%s\n' "$version" | grep -Eqx '[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?'; then
        fail "--version expects a Nova version such as 1.4.0"
    fi
    if [ -z "${HOME:-}" ] && { [ -z "$root" ] || [ -z "$bin" ]; }; then
        fail "HOME is not set; pass --install-root and --bin-dir"
    fi
    root=$(absolute "${root:-${XDG_DATA_HOME:-$HOME/.local/share}/yazelix-nova}")
    bin=$(absolute "${bin:-$HOME/.local/bin}")
    marker=$root/.yazelix-nova-installer
    link=$bin/yzx
    selected=$root/current/bin/yzx
    "$action"
}

main "$@"
