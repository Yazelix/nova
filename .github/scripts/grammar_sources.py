"""Preserve Helix grammar source files and link records for redistribution review."""

import argparse
import gzip
import hashlib
import io
import json
import os
import posixpath
import re
import stat
import subprocess
import tarfile
import tempfile
import tomllib
from pathlib import Path

STORE = Path("/nix/store")


def scan_error(error):
    raise error


def reachable(document, root):
    graph = {Path(key).name: value for key, value in
             document.get("derivations", document).items()}
    pending, selected = [Path(root).name], {}
    while pending:
        key = pending.pop()
        if key in selected:
            continue
        if key not in graph:
            raise ValueError(f"missing derivation: {key}")
        derivation = selected[key] = graph[key]
        children = derivation.get("inputs", {}).get("drvs", derivation.get("inputDrvs", {}))
        pending.extend(Path(child).name for child in children)
    return selected


def realise_inputs(paths, root, graph):
    producers = {}
    for key, node in graph.items():
        env = node["env"] | node.get("structuredAttrs", {})
        for name, record in node.get("outputs", {}).items():
            if path := record.get("path") or env.get(name):
                producers[Path(path).name] = f"{STORE / Path(key).name}!{name}"
    subprocess.run(["nix-store", "--realise", "--add-root", str(root), "--indirect",
                    *(producers.get(path.name, str(path)) for path in sorted(paths))],
                   check=True, stdout=subprocess.DEVNULL)


def inventory(document, root, config, revision):
    if not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise ValueError("expected an immutable Nova revision")
    selected = reachable(document, root)
    pins = {g["name"]: g["source"] for g in config.get("grammar", [])}
    grammars, sources = [], {}
    for key, derivation in sorted(selected.items()):
        env = derivation["env"] | derivation.get("structuredAttrs", {})
        name = env.get("name", derivation.get("name", ""))
        if not name.startswith("helix-tree-sitter-"):
            continue
        match = re.fullmatch(r"helix-tree-sitter-(.+)-([0-9a-f]{40})", name)
        if not match:
            raise ValueError(f"unsupported grammar identity: {name}")
        name, pin = match.groups()
        origin = pins.get(name)
        if not origin or origin.get("rev") != pin:
            raise ValueError(f"grammar source pin mismatch: {name}")
        source = Path(env["src"])
        if source.parent != STORE or source.is_symlink() or not source.is_dir():
            raise ValueError(f"missing grammar source: {source}")
        sources[source.name] = source
        grammars.append({"name": name, "origin": origin, "derivation": key,
                         "source": "sources/" + source.name})
    if not grammars or len({g["name"] for g in grammars}) != len(grammars):
        raise ValueError("expected one selected source per grammar")
    return {"schema_version": 1, "revision": revision,
            "redistribution_review": "required",
            "grammars": sorted(grammars, key=lambda g: g["name"]), "sources": []}, sources


def bundle(manifest, sources, output):
    if output.exists():
        raise FileExistsError(output)
    entries, source_records, rewritten = [], [], {}
    roots = {name: root.resolve(strict=True) for name, root in sources.items()}
    for name, root in sorted(sources.items()):
        if root.is_symlink():
            raise ValueError(f"source root must not be a symlink: {root}")
        files, links = {}, {}
        paths = []
        if root.is_dir():
            for parent, directories, names in os.walk(root, onerror=scan_error):
                paths.extend(Path(parent) / name for name in directories + names)
        for path in [root, *sorted(paths)]:
            mode = path.lstat().st_mode
            relative = path.relative_to(root).as_posix()
            archive_name = "sources/" + name + ("/" + relative if relative != "." else "")
            if stat.S_ISLNK(mode):
                literal = os.readlink(path)
                archived_target = None
                try:
                    target = path.resolve(strict=True)
                    for other, directory in roots.items():
                        if target.is_relative_to(directory):
                            member = "sources/" + other + "/" + target.relative_to(directory).as_posix()
                            archived_target = posixpath.relpath(member, posixpath.dirname(archive_name))
                            break
                except (OSError, RuntimeError):
                    pass
                links[relative] = {"target": literal, "archived": archived_target is not None}
                # Generated host links remain inspectable without exposing the extraction host.
                if archived_target is None:
                    continue
                if archived_target != literal:
                    links[relative]["archive_target"] = rewritten[archive_name] = archived_target
            elif not (stat.S_ISREG(mode) or stat.S_ISDIR(mode)) or mode & 0o6000:
                raise ValueError(f"unsafe source entry: {path}")
            elif stat.S_ISREG(mode):
                with path.open("rb") as stream:
                    files[relative] = hashlib.file_digest(stream, "sha256").hexdigest()
            entries.append((path, archive_name))
        source_records.append({"path": "sources/" + name, "files": files, "symlinks": links})
    manifest = manifest | {"sources": source_records}
    metadata = (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode()
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(dir=output.parent, delete=False) as stream:
            temporary = Path(stream.name)
            with gzip.GzipFile(fileobj=stream, mode="wb", filename="", mtime=0,
                               compresslevel=6) as compressed:
                with tarfile.open(fileobj=compressed, mode="w|") as archive:
                    info = tarfile.TarInfo("SOURCE-MANIFEST.json")
                    info.size, info.mode, info.mtime = len(metadata), 0o644, 1
                    archive.addfile(info, io.BytesIO(metadata))
                    for path, name in entries:
                        info = archive.gettarinfo(str(path), arcname=name)
                        if info.islnk():
                            # Store optimisation hard-links identical files; keep full copies.
                            info.type, info.linkname = tarfile.REGTYPE, ""
                            info.size = path.stat().st_size
                        info.uid = info.gid = 0
                        info.uname = info.gname = ""
                        info.mtime = 1
                        if name in rewritten:
                            info.linkname = rewritten[name]
                        if info.isfile():
                            with path.open("rb") as source:
                                archive.addfile(info, source)
                        else:
                            archive.addfile(info)
        verify_bundle(temporary)
        os.link(temporary, output)
    finally:
        if temporary is not None:
            temporary.unlink()


def verify_bundle(path):
    with gzip.open(path, "rb") as compressed, tarfile.open(fileobj=compressed, mode="r|") as archive:
        first = archive.next()
        if first is None or first.name != "SOURCE-MANIFEST.json" or not first.isfile():
            raise ValueError("missing source manifest")
        manifest = json.load(archive.extractfile(first))
        files, links, seen = {}, {}, set()
        for source in manifest["sources"]:
            for relative, digest in source["files"].items():
                name = source["path"] + ("/" + relative if relative != "." else "")
                if name in files:
                    raise ValueError(f"duplicate source file: {name}")
                files[name] = digest
            for relative, record in source["symlinks"].items():
                if record["archived"]:
                    links[source["path"] + "/" + relative] = record.get("archive_target", record["target"])
        for member in archive:
            if Path(member.name).is_absolute() or ".." in Path(member.name).parts:
                raise ValueError(f"unsafe source member: {member.name}")
            if member.name in seen:
                raise ValueError(f"duplicate source member: {member.name}")
            seen.add(member.name)
            if member.name == first.name:
                continue
            if member.isfile():
                with archive.extractfile(member) as stream:
                    digest = hashlib.file_digest(stream, "sha256").hexdigest()
                if digest != files.pop(member.name, None):
                    raise ValueError(f"source hash changed: {member.name}")
            elif member.issym():
                target = posixpath.normpath(posixpath.join(posixpath.dirname(member.name), member.linkname))
                if posixpath.isabs(member.linkname) or not target.startswith("sources/"):
                    raise ValueError(f"unsafe source link: {member.name}")
                if member.linkname != links.pop(member.name, None):
                    raise ValueError(f"source link changed: {member.name}")
            elif not member.isdir():
                raise ValueError(f"unsafe source member: {member.name}")
        if files or links:
            raise ValueError("incomplete source archive")
        while compressed.read(1024 * 1024):
            pass
    return manifest


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("graph", type=Path, help="nix derivation show --recursive JSON")
    parser.add_argument("root", help="exact archive derivation path")
    parser.add_argument("helix_source", type=Path, help="candidate's pinned Helix source")
    parser.add_argument("revision", help="candidate's immutable Nova revision")
    parser.add_argument("output", type=Path, help="new .tar.gz destination")
    args = parser.parse_args()
    manifest, sources = inventory(json.loads(args.graph.read_text()), args.root,
                                  tomllib.loads((args.helix_source / "languages.toml").read_text()),
                                  args.revision)
    bundle(manifest, sources, args.output)
    print(f"Preserved {len(manifest['grammars'])} grammars from {len(sources)} sources; "
          "redistribution review remains required.")
