set -euo pipefail
: "${payload:?}" "${out:?}" "${glibcBaseline:?}" "${interpreter:?}" "${hostLibraries:?}" "${libgccSymbolVersions:?}"
read -r -a host_libraries <<< "$hostLibraries"
cp -R "$payload" "$out"
chmod -R u+w "$out"
mkdir -p "$out/lib/native"
declare -A copied=() origins=() seen=()
declare -A replacements=()
while IFS=$'\t' read -r original compatible; do
    replacements[$original]=$compatible
done < "${libraryReplacements:-/dev/null}"
mapfile -d "" queue < <(find "$out" -type f -print0)
for ((i=0; i<${#queue[@]}; i++)); do
    binary=${queue[i]}
    [[ $(od -An -tx1 -N4 "$binary") == " 7f 45 4c 46" ]] || continue
    [[ ! ${seen[$binary]+set} ]] || continue
    seen[$binary]=1
    headers=$(readelf -l "$binary")
    if [[ $headers == *INTERP* ]]; then patchelf --set-interpreter "$interpreter" "$binary"; fi
    dynamic=$(readelf -d "$binary")
    if [[ $dynamic != *"(NEEDED)"* ]]; then
        if [[ $headers == *DYNAMIC* ]]; then patchelf --remove-rpath "$binary"; fi
        continue
    fi
    rpath=$(patchelf --print-rpath "$binary")
    needed_output=$(patchelf --print-needed "$binary")
    IFS=: read -ra dirs <<< "$rpath"
    mapfile -t needed <<< "$needed_output"
    search=
    for name in "${needed[@]}"; do
        for host in "${host_libraries[@]}"; do
            [[ $name != "$host" ]] || continue 2
        done
        [[ $name != */* ]] || { echo "absolute dependency: $binary -> $name" >&2; exit 1; }
        source=
        for directory in "${dirs[@]}"; do
            directory=${directory//\$ORIGIN/$(dirname "${origins[$binary]:-$binary}")}
            if [[ -e $directory/$name ]]; then source=$(realpath "$directory/$name"); break; fi
        done
        if [[ $source == "$out"/* ]]; then
            target=$source
        elif [[ $source == /nix/store/* ]]; then
            for original in "${!replacements[@]}"; do
                if [[ $source == "$original/"* ]]; then
                    source="${replacements[$original]}/${source#"$original/"}"
                    [[ -f $source ]] || { echo "missing compatible library: $source" >&2; exit 1; }
                    break
                fi
            done
            target="$out/lib/native/${source#/nix/store/}"
            if [[ ! ${copied[$source]+set} ]]; then
                origin=${source#/nix/store/}
                printf '%s\n' "${origin%%/*}" >> "$out/share/yazelix/archive-origins.txt"
                copied[$source]=$target
                origins[$target]=$source
                install -D -m 755 "$source" "$target"
                queue+=("$target")
            fi
        else
            echo "unresolved library: $binary -> $name ($source)" >&2; exit 1
        fi
        if [[ $(basename "$source") != "$name" ]]; then
            ln -sfn "$(basename "$source")" "$(dirname "$target")/$name"
        fi
        relative=$(realpath --relative-to="$(dirname "$binary")" "$(dirname "$target")")
        search+="\$ORIGIN/$relative:"
    done
    patchelf --force-rpath --set-rpath "${search%:}" "$binary"
done
printf "%s\n" "processed ${#queue[@]} files; ${#copied[@]} private libraries" >&2

# Required symbol versions determine the native floor, not provenance strings.
: > "$TMPDIR/glibc-versions"
: > "$TMPDIR/gcc-versions"
for binary in "${!seen[@]}"; do
    versions=$(readelf --version-info "$binary")
    sed -nE 's/.*Name: GLIBC_([0-9.]+) .*/\1/p' <<< "$versions" >> "$TMPDIR/glibc-versions"
    sed -nE 's/.*Name: (GCC_[^ ]+) .*/\1/p' <<< "$versions" >> "$TMPDIR/gcc-versions"
done
while IFS= read -r symbol; do
    [[ " $libgccSymbolVersions " == *" $symbol "* ]] || {
        echo "undeclared host libgcc symbol: $symbol" >&2; exit 1
    }
done < <(sort -u "$TMPDIR/gcc-versions")
minimum=$(sort -V "$TMPDIR/glibc-versions" | tail -1)
test -n "$minimum"
if [[ $(printf '%s\n' "$glibcBaseline" "$minimum" | sort -V | tail -1) != "$glibcBaseline" ]]; then
    echo "archive requires glibc $minimum; baseline is $glibcBaseline" >&2
    exit 1
fi
inputs="$TMPDIR/native-inputs.txt"
printf 'glibc %s\n' "$minimum" > "$inputs"
sort -u "$out/share/yazelix/native-inputs.txt" >> "$inputs"
find "$out/lib" "$out/share" \( -type f -o -type l \) -name '*.so*' | sed "s|^$out/||" | sort >> "$inputs"
mv "$inputs" "$out/share/yazelix/native-inputs.txt"
sort -u -o "$out/share/yazelix/archive-origins.txt" "$out/share/yazelix/archive-origins.txt"
while IFS= read -r -d "" link; do
    [[ $(readlink "$link") != /* && $(realpath -e "$link") == "$out/"* ]] || {
        echo "invalid owned symlink: $link" >&2; exit 1
    }
done < <(find "$out" -type l -print0)
