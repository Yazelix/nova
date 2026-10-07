#!/usr/bin/env bash
# Run in a clean Linux image with tmux, coreutils, Git and no Nix store mounted.
set -euo pipefail
root=$(realpath "${1:?usage: linux-archive.sh <extracted-root>}")
test ! -e /nix/store
command -v nix && exit 1
export NOVA_HOST_GIT
NOVA_HOST_GIT=$(command -v git)
export NOVA_HOST_GIT_EXEC_PATH
NOVA_HOST_GIT_EXEC_PATH=$(git --exec-path)
for path in bin/git libexec/yazelix/git libexec/git-core lib/perl5 bin/python3 libexec/native/perl; do
    test ! -e "$root/$path"
done
work=$(mktemp -d /tmp/na.XXXXXX)
user_environment() {
    local prefix="$work/$1"
    export HOME="$prefix/home" XDG_CONFIG_HOME="$prefix/config" XDG_CACHE_HOME="$prefix/cache"
    export XDG_DATA_HOME="$prefix/data" YAZELIX_CONFIG_HOME="$prefix/config/yazelix"
    export YAZELIX_STATE_DIR="$prefix/state" ZELLIJ_SOCKET_DIR="$prefix/s" STEEL_HOME="$prefix/steel"
    mkdir -p "$HOME" "$YAZELIX_CONFIG_HOME" "$ZELLIJ_SOCKET_DIR"
    printf '[welcome]\nstyle = "matrix"\nduration_seconds = 1\n' > "$YAZELIX_CONFIG_HOME/config.toml"
}
export TERM=xterm-256color
unset ZELLIJ ZELLIJ_SESSION_NAME YAZELIX_RUNTIME_ROOT
user_environment tools
export GIT_CONFIG_GLOBAL="$HOME/gitconfig"
git config --file "$GIT_CONFIG_GLOBAL" alias.native-proof '!printf native-git'
export GIT_TEMPLATE_DIR="$HOME/git-templates"
mkdir -p "$GIT_TEMPLATE_DIR"
printf 'host template\n' > "$GIT_TEMPLATE_DIR/native-template"
mkdir -p "$work/project"
printf 'native editor document\n' > "$work/project/archive.txt"
cd "$work/project"
yzx="$root/bin/yzx"
"$yzx" --version
"$yzx" help
"$yzx" status --json
"$yzx" doctor
PATH=/nonexistent "$yzx" status --json > "$work/owned-path.json"
if PATH=/nonexistent "$yzx" doctor > "$work/missing-git.log" 2>&1; then exit 1; fi
grep -q 'host Git.*command not found: git' "$work/missing-git.log" || { cat "$work/missing-git.log"; exit 1; }
if "$yzx" launch > "$work/launch.log" 2>&1; then exit 1; fi
grep -q 'omits Rio' "$work/launch.log"
# The managed child expands its own environment.
# shellcheck disable=SC2016
"$yzx" run sh -c '
    set -eu
    nu --no-config-file -c "print native-nu"
    bash -c "printf native-bash"
    zsh -c "printf native-zsh"
    fish -c "test \"\$__fish_data_dir\" = \"\$YAZELIX_RUNTIME_ROOT/share/fish\""
    test "$(command -v git)" = "$NOVA_HOST_GIT"
    test "$(git --exec-path)" = "$NOVA_HOST_GIT_EXEC_PATH"
    test "$(git native-proof)" = native-git
    git init -q "$HOME/git"
    cd "$HOME/git"
    test "$(cat .git/native-template)" = "host template"
    mkdir -p .git/hooks
    git config user.name Archive
    git config user.email archive@example.invalid
    printf "printf native-hook > hook-result\\n" > .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    git commit -qm proof --allow-empty
    test "$(cat hook-result)" = native-hook
    magick -font DejaVu-Sans label:Archive "$HOME/font.png"
    file "$HOME/font.png" | grep PNG
    printf "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"80\" height=\"40\"><text x=\"2\" y=\"20\" font-family=\"DejaVu Sans\">Nova</text></svg>" > "$HOME/text.svg"
    magick "$HOME/text.svg" "$HOME/svg.png"
    file "$HOME/svg.png" | grep PNG
'
if [ "${NOVA_ARCHIVE_OFFLINE:-0}" != 1 ]; then
    "$yzx" run curl -fsSL https://example.com > "$work/https.html"
    grep -q 'Example Domain' "$work/https.html"
    "$yzx" run git ls-remote https://github.com/Yazelix/nova.git HEAD > "$work/git-https.log"
    grep -Eq '^[0-9a-f]{40}[[:space:]]+HEAD$' "$work/git-https.log"
    echo 'native HTTPS passed'
fi
user_environment workspace
git init -q
git add archive.txt
git -c user.name=Archive -c user.email=archive@example.invalid commit -qm proof
printf 'host Git integration\n' >> archive.txt
tmux_socket="na-$$"
session=archive-proof
printf 'set -g remain-on-exit on\n' > "$work/tmux.conf"
tmux() { command tmux -f "$work/tmux.conf" -L "$tmux_socket" "$@"; }
cleanup() {
    "$yzx" run zellij kill-session "$session" >/dev/null 2>&1 || true
    tmux kill-server >/dev/null 2>&1 || true
}
trap cleanup EXIT
tmux new-session -d -s proof -x 160 -y 44 "$yzx" enter --session "$session"
wait_for() {
    for ((attempt=0; attempt<120; attempt++)); do
        tmux capture-pane -p -t proof > "$work/screen"
        if grep -q "$1" "$work/screen"; then return; fi
        sleep 0.25
    done
    cat "$work/screen"
    echo "missing terminal output: $1" >&2
    exit 1
}
wait_for RADAR
wait_for 'Tab/Z Search'
tmux send-keys -t proof Enter
wait_for 'NOR.*archive.txt'
wait_for 'native editor document'
test "$(readlink "$STEEL_HOME/native/libnova_helix_file_watcher.so")" = "$root/lib/libnova_helix_file_watcher.so"
# The selected file opens in managed Helix. Start a managed shell beside it.
"$yzx" run zellij -s "$session" action new-pane -- "$root/libexec/yazelix/yzx-shell"
wait_for '::'
tmux send-keys -t proof -l 'print ("native-" + "workspace")'
tmux send-keys -t proof Enter
wait_for native-workspace
tmux send-keys -t proof -l 'yzx-yazi --yzx-workspace-popup'
tmux send-keys -t proof Enter
wait_for 'Tab/Z Search'
tmux send-keys -t proof q
wait_for native-workspace
tmux send-keys -t proof -l 'print ("native-" + "yazi-exit")'
tmux send-keys -t proof Enter
wait_for native-yazi-exit
tmux send-keys -t proof -l 'yzx-git'
tmux send-keys -t proof Enter
wait_for 'Unstaged changes'
tmux send-keys -t proof q
tmux send-keys -t proof -l 'print ("native-" + "git-exit")'
tmux send-keys -t proof Enter
wait_for native-git-exit
echo 'native archive: tools, previews, host Git, Lazygit and fresh workspace passed'
