#!/bin/sh
set -eu
shell_program="$(@yzxConfig@ --get shell.program)"

if [ "$shell_program" = nu ]; then
  exec @yzxNu@ "$@"
fi

if [ -n "${YAZELIX_RUNTIME_ROOT:-}" ]; then
  PATH="$YAZELIX_RUNTIME_ROOT/libexec/yazelix${PATH:+:$PATH}"
else
  PATH="@atuinPath@${PATH:+:$PATH}"
fi
export PATH

atuin_enabled="$(@yzxConfig@ --get shell.atuin)"
if [ "$atuin_enabled" = true ]; then
  YZX_ATUIN_INIT_BASE=@atuinInit@
  export YZX_ATUIN_INIT_BASE
fi

case "$shell_program" in
  bash)
    if [ "$atuin_enabled" = true ]; then
      exec @bash@ --rcfile @bashAtuinRc@ -i "$@"
    fi
    exec @bash@ -i "$@"
    ;;
  zsh)
    if [ "$atuin_enabled" = true ]; then
      export YZX_USER_ZDOTDIR="${ZDOTDIR:-$HOME}"
      YZX_MANAGED_ZDOTDIR=@zshAtuinConfig@
      ZDOTDIR="$YZX_MANAGED_ZDOTDIR"
      export YZX_MANAGED_ZDOTDIR ZDOTDIR
    fi
    exec @zsh@ -i "$@"
    ;;
  fish)
    if [ "$atuin_enabled" = true ]; then
      YZX_FISH_ATUIN_INIT=@fishAtuinInit@
      export YZX_FISH_ATUIN_INIT
      exec @fish@ -C 'source $YZX_FISH_ATUIN_INIT' -i "$@"
    fi
    exec @fish@ -i "$@"
    ;;
esac

printf '%s\n' "Unsupported shell.program: $shell_program" >&2
exit 64
