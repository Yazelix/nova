"""Preserve pinned Rust/Go notice material; redistribution review remains required."""

import argparse
import hashlib
import json
import os
import re
import tarfile
import tempfile
from pathlib import Path, PurePosixPath

from grammar_sources import bundle, reachable, realise_inputs, scan_error

STORE = Path("/nix/store")


def collect(source, directory, required):
    if source.parent != STORE or source.is_symlink() or not source.is_file():
        raise ValueError(f"missing notice archive: {source}")
    found = set()
    with source.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    with tarfile.open(source, mode="r|*") as archive:
        for member in archive:
            path = PurePosixPath(member.name)
            if member.isdir() or not (
                    path.name.lower().startswith(("license", "licence", "copying", "copyright", "notice", "patents"))
                    or any(p.lower() in ["licenses", "licences"] for p in path.parts[:-1])):
                continue
            if path.is_absolute() or ".." in path.parts or not member.isfile():
                raise ValueError(f"unsafe notice member: {member.name}")
            target = directory / source.name / path
            target.parent.mkdir(parents=True, exist_ok=True)
            with archive.extractfile(member) as content, target.open("xb") as output:
                output.write(content.read())
            target.chmod(0o644)
            found.add(path.as_posix())
    if missing := required - found:
        raise ValueError(f"missing required notices in {source.name}: {sorted(missing)}")
    return {"archive": source.name, "sha256": digest, "notices": sorted(found)}


def write(document, root, output, realise=False):
    if output.exists() or output.is_symlink():
        raise FileExistsError(output)
    selected = reachable(document, root)
    environments = {key: d["env"] | d.get("structuredAttrs", {}) for key, d in selected.items()}
    providers, archives = [], {}
    for key, env in sorted(environments.items()):
        name = env.get("name", selected[key].get("name", ""))
        if not env.get("src"):
            continue
        source = Path(env["src"])
        original = source.name.partition("-")[2]
        if name.startswith("rust-std-"):
            match = re.fullmatch(r"rust-std-(\d+\.\d+\.\d+)-(.+)\.tar\.(?:gz|xz)", original)
            if not match:
                raise ValueError(f"unsupported Rust runtime archive: {source}")
            version, target = match.groups()
            prefix = f"rust-std-{version}-{target}/"
            archives[source] = {prefix + n for n in ["COPYRIGHT", "LICENSE-MIT", "LICENSE-APACHE"]}
            candidates = sorted({Path(e["src"]) for e in environments.values() if e.get("src")
                                 and re.fullmatch(r"[a-z0-9]{32}-rustc-" + re.escape(version)
                                                  + r"-x86_64-unknown-linux-gnu\.tar\.(?:gz|xz)",
                                                  Path(e["src"]).name)})
            if not candidates:
                raise ValueError(f"missing Rust compiler notice archive for {version}")
            compiler = candidates[0]
            prefix = f"rustc-{version}-x86_64-unknown-linux-gnu/"
            archives[compiler] = {prefix + n for n in ["COPYRIGHT", "LICENSE-MIT", "LICENSE-APACHE",
                "rustc/share/doc/rust/COPYRIGHT-library.html"]}
            provider = {"version": version, "target": target, "compiler_notices": compiler.name}
        elif re.fullmatch(r"go-\d+\.\d+\.\d+", name):
            version = name.removeprefix("go-")
            if not re.fullmatch("go" + re.escape(version) + r"\.src\.tar\.(?:gz|xz)", original):
                raise ValueError(f"Go source version mismatch: {source}")
            archives[source] = {"go/LICENSE", "go/PATENTS"}
            provider = {"version": version}
        else:
            continue
        providers.append(provider | {"name": name, "derivation": key, "archive": source.name})
    if not providers:
        raise ValueError("no language-runtime providers selected")
    if realise:
        realise_inputs(archives, Path(str(output.absolute()) + ".inputs"), selected)
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary) / "language-runtime-notices"
        records = [collect(source, directory, required) for source, required in sorted(archives.items())]
        for parent, _, _ in os.walk(directory, onerror=scan_error):
            Path(parent).chmod(0o755)
        manifest = {"schema_version": 1, "archive_derivation": Path(root).name,
                    "coverage": "reachable build-graph language-runtime candidates; not a shipped-component census",
                    "redistribution_review": "required", "providers": providers, "notice_archives": records}
        bundle(manifest, {directory.name: directory}, output)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("graph", type=Path, help="nix derivation show --recursive JSON")
    parser.add_argument("root", help="exact archive derivation path")
    parser.add_argument("output", type=Path, help="new .tar.gz destination")
    parser.add_argument("--realise", action="store_true", help="realise and GC-root selected notice inputs before capture")
    args = parser.parse_args()
    write(json.loads(args.graph.read_text()), args.root, args.output, args.realise)
    print("Preserved language-runtime notices; redistribution review remains required.")
