#!/bin/sh
# NOVA-INSTALL-001 (docs/portable-runtime.md): install Nova's portable release archive without Nix.
# Only the final main "$@" call executes the body; truncation before it executes nothing.
set -eu

releases=https://github.com/Yazelix/nova/releases
target=x86_64-unknown-linux-gnu
variant=no-rio

fail() {
    stop_bar
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
Output has no color when stdout is not a terminal or NO_COLOR is set.

The installer checks every download against the release's SHA256SUMS. To check
the archive's provenance as well, download it from the release and run:
  gh attestation verify <archive> --repo Yazelix/nova
EOF
}

absolute() {
    case $1 in
        /*) printf '%s' "$1" ;;
        *) printf '%s/%s' "$PWD" "$1" ;;
    esac
}

tilde() {
    case $1 in
        "${HOME:-/nonexistent}"/*) printf '%s/%s' '~' "${1#"$HOME"/}" ;;
        *) printf '%s' "$1" ;;
    esac
}

fetch() {
    curl --proto '=https' --tlsv1.2 --fail --location --silent --show-error --retry 3 "$@"
}

# Fetch a release file and require the hash that SHA256SUMS lists for it.
verified() {
    hash=$(awk -v name="$1" '$2 == name { print $1 }' "$work/SHA256SUMS")
    [ -n "$hash" ] || fail "release $version lists no checksum for $1"
    fetch --output "$work/$1" "$url/$1" || fail "could not download $1 from release $version"
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

# Redraw the archive download as one progress line.
bar() {
    got=$(wc -c 2>/dev/null < "$work/$name") || got=0
    awk -v got="$got" -v total="$total" -v label="$label" -v dim="$dim" -v reset="$reset" 'BEGIN {
        n = int(30 * got / total)
        line = "\r" label " "
        for (i = 0; i < 30; i++) line = line (i == n ? dim : "") (i < n ? "━" : "─")
        printf "%s%s %3d%%", line, reset, 100 * got / total
    }'
}

stop_bar() {
    [ -n "${drawer:-}" ] || return 0
    kill "$drawer" 2>/dev/null || true
    wait "$drawer" 2>/dev/null || true
    drawer=''
    bar
    echo
}

# A terminal sees a progress bar sized from nova-release.json; other output gets one line.
download() {
    total=$(awk -v entry="\"name\": \"$name\"" 'index($0, entry) { found = 1 }
        found && /"size":/ {
            sub(/,$/, "", $2)
            if ($2 ~ /^[1-9][0-9]*$/) print $2
            exit
        }' "$work/nova-release.json")
    label="Downloading${total:+ $(((total + 999999) / 1000000)) MB}"
    if [ -t 1 ] && [ -n "$total" ]; then
        # The trap lets a running bar finish, so no stale line prints after stop_bar's.
        (
            trap 'exit 0' TERM
            while :; do
                bar
                sleep 0.2
            done
        ) &
        drawer=$!
    else
        echo "$label"
    fi
    verified "$name"
    stop_bar
}

note() { printf '          %s\n' "$*"; }

row() {
    if [ -n "${3:-}" ]; then
        printf '  %-8s%s%-*s%s%s%s%s\n' "$1" "$bold" "$width" "$2" "$reset" "$dim" "$3" "$reset"
    else
        printf '  %-8s%s%s%s\n' "$1" "$bold" "$2" "$reset"
    fi
}

path_help() {
    case $(basename "${SHELL:-sh}") in
        bash) note "To run plain yzx, add this line to ~/.bashrc:" && note "export PATH=\"$bin:\$PATH\"" ;;
        zsh) note "To run plain yzx, add this line to ~/.zshrc:" && note "export PATH=\"$bin:\$PATH\"" ;;
        fish) note "To run plain yzx, run this once in fish:" && note "fish_add_path '$bin'" ;;
        nu) note "To run plain yzx, add this line to your config.nu:" && note "\$env.PATH = (\$env.PATH | prepend '$bin')" ;;
        *) note "To run plain yzx, add this line to ~/.profile:" && note "export PATH=\"$bin:\$PATH\"" ;;
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
    if [ "$cmd" = yzx ]; then
        note "yzx runs this install, ahead of $(tilde "$other")"
    else
        note "plain yzx runs $(tilde "$other") instead"
    fi
}

summary() {
    cmd=yzx
    [ "$(readlink "$(command -v yzx)")" = "$selected" ] || cmd=$(tilde "$link")
    width=$((${#cmd} + 16))
    printf '\n%s✓%s Installed %sYazelix Nova %s%s\n\n' "$green" "$reset" "$bold" "$version" "$reset"
    row Start "$cmd enter" "opens Nova in this terminal"
    case ":$PATH:" in
        *":$bin:"*) ;;
        *) path_help ;;
    esac
    other_yzx
    row Learn "$cmd tutor begin" "run it inside Nova"
    row Docs https://nova.yazelix.com/docs/
    row Help "$cmd doctor"
    printf '\n  %sFiles in %s · yzx linked in %s%s\n' "$dim" "$(tilde "$root")" "$(tilde "$bin")" "$reset"
}

cleanup() {
    stop_bar
    if [ -n "${work:-}" ] && [ -d "$work" ]; then
        chmod -R u+w "$work" 2>/dev/null || true
        rm -rf "$work"
    fi
}

uninstall() {
    [ -f "$marker" ] || fail "$root was not created by this installer; nothing was removed"
    kept_link='' kept_root=''
    if [ -L "$link" ] && [ "$(readlink "$link")" = "$selected" ]; then
        rm "$link"
    elif [ -e "$link" ] || [ -L "$link" ]; then
        kept_link="Kept $(tilde "$link"): it does not point into $(tilde "$root")"
    fi
    for path in "$root"/versions "$root"/.install.*; do
        [ ! -e "$path" ] || chmod -R u+w "$path"
    done
    rm -rf "$root/versions" "$root/current" "$root"/.install.* "$marker"
    rmdir "$root" 2>/dev/null || kept_root="Kept $(tilde "$root"): it holds files this installer did not create"
    printf '%s✓%s Removed %sYazelix Nova%s\n\n' "$green" "$reset" "$bold" "$reset"
    for line in "$kept_link" "$kept_root" "Your configuration and state are untouched"; do
        [ -z "$line" ] || printf '  %s\n' "$line"
    done
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
    fetch --output "$work/SHA256SUMS" "$url/SHA256SUMS" || fail "could not download release $version"
    name=$(grep -E "^[0-9a-f]{64}  yazelix-nova-[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?-$target-$variant\.tar\.gz$" "$work/SHA256SUMS" | cut -c67-)
    [ "$(printf '%s\n' "$name" | grep -c .)" = 1 ] || fail "release $version has no single $target archive"
    found=${name#yazelix-nova-}
    found=${found%-"$target-$variant".tar.gz}
    [ "$version" = latest ] || [ "$version" = "$found" ] || fail "release $version lists the archive for $found"
    version=$found
    url=$releases/download/nova-v$version

    verified nova-release.json
    glibc_check
    printf '%sYazelix Nova %s%s\n' "$bold" "$version" "$reset"
    download
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
    summary
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
    bold='' dim='' green='' reset=''
    if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
        bold=$(printf '\033[1m') dim=$(printf '\033[2m') green=$(printf '\033[32m') reset=$(printf '\033[0m')
    fi
    "$action"
}

main "$@"
