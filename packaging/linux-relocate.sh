set -euo pipefail
: "${payload:?}" "${out:?}"
cp -R "$payload" "$out"
chmod -R u+w "$out"
mkdir -p "$out/lib/native"
declare -A copied=() origins=() seen=()
mapfile -d "" queue < <(find "$out" -type f \( -perm /111 -o -name "*.so*" \) -print0)
for ((i=0; i<${#queue[@]}; i++)); do
    binary=${queue[i]}
    [[ $(od -An -tx1 -N4 "$binary") == " 7f 45 4c 46" ]] || continue
    [[ ! ${seen[$binary]+set} ]] || continue
    seen[$binary]=1
    headers=$(readelf -l "$binary")
    if [[ $headers == *INTERP* ]]; then patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 "$binary"; fi
    dynamic=$(readelf -d "$binary")
    if [[ $dynamic != *"(NEEDED)"* ]]; then
        if [[ $headers == *DYNAMIC* ]]; then patchelf --remove-rpath "$binary"; fi
        continue
    fi
    IFS=: read -ra dirs <<< "$(patchelf --print-rpath "$binary")"
    mapfile -t needed < <(patchelf --print-needed "$binary")
    search=
    for name in "${needed[@]}"; do
        case $name in
            ld-linux-x86-64.so.2|libc.so.6|libm.so.6|libmvec.so.1|libdl.so.2|libpthread.so.0|librt.so.1|libresolv.so.2|libanl.so.1|libutil.so.1|libgcc_s.so.1) continue ;;
        esac
        [[ $name != */* ]] || { echo "absolute dependency: $binary -> $name" >&2; exit 1; }
        source=
        for directory in "${dirs[@]}"; do
            directory=${directory//\$ORIGIN/$(dirname "${origins[$binary]:-$binary}")}
            if [[ -e $directory/$name ]]; then source=$(realpath "$directory/$name"); break; fi
        done
        if [[ $source == "$out"/* ]]; then
            target=$source
        elif [[ $source == /nix/store/* ]]; then
            target="$out/lib/native/${source#/nix/store/}"
            if [[ ! ${copied[$source]+set} ]]; then
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
for binary in "${!seen[@]}"; do
    readelf --version-info "$binary" | sed -nE 's/.*Name: GLIBC_([0-9.]+) .*/\1/p' >> "$TMPDIR/glibc-versions"
done
minimum=$(sort -V "$TMPDIR/glibc-versions" | tail -1)
test -n "$minimum"
inputs="$TMPDIR/native-inputs.txt"
printf 'glibc %s\n' "$minimum" > "$inputs"
sort -u "$out/share/yazelix/native-inputs.txt" >> "$inputs"
find "$out/lib" "$out/share" \( -type f -o -type l \) -name '*.so*' | sed "s|^$out/||" | sort >> "$inputs"
mv "$inputs" "$out/share/yazelix/native-inputs.txt"
while IFS= read -r -d "" link; do
    [[ $(readlink "$link") != /* && $(realpath -e "$link") == "$out/"* ]] || {
        echo "invalid owned symlink: $link" >&2; exit 1
    }
done < <(find "$out" -type l -print0)
