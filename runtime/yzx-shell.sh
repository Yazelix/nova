#!/bin/sh
PATH="@atuinPath@${PATH:+:$PATH}"
export PATH

shell_program="$(@yzxConfig@ --get shell.program)"

if [ "$shell_program" = nu ]; then
  exec @yzxNu@ "$@"
fi

atuin_enabled="$(@yzxConfig@ --get shell.atuin)"

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
      export YZX_MANAGED_ZDOTDIR=@zshAtuinConfig@ ZDOTDIR=@zshAtuinConfig@
    fi
    exec @zsh@ -i "$@"
    ;;
  fish)
    if [ "$atuin_enabled" = true ]; then
      exec @fish@ -C 'source "@fishAtuinInit@"' -i "$@"
    fi
    exec @fish@ -i "$@"
    ;;
esac

printf '%s\n' "Unsupported shell.program: $shell_program" >&2
exit 64
