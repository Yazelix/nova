"""Write and verify NPR-METADATA-001 for the Nix-owned portable payload."""

import argparse
import hashlib
import json
import os
import re
import stat
from pathlib import Path, PurePosixPath

PACKAGE = "share/yazelix/package.json"
INDEX = "share/yazelix/files.json"


def encode(value):
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def scan_error(error):
    raise error


def index(root):
    files = {}
    paths = (Path(parent) / name
             for parent, directories, names in os.walk(root, onerror=scan_error)
             for name in directories + names)
    for path in sorted(paths):
        name = path.relative_to(root).as_posix()
        mode = path.lstat().st_mode
        if stat.S_ISLNK(mode):
            target = os.readlink(path)
            try:
                contained = path.resolve(strict=True).is_relative_to(root.resolve())
            except (OSError, RuntimeError):
                contained = False
            if Path(target).is_absolute() or not contained or name in [PACKAGE, INDEX]:
                raise ValueError(f"invalid payload symlink: {name}")
            files[name] = {"type": "symlink", "target": target}
        elif not (stat.S_ISREG(mode) or stat.S_ISDIR(mode)) or mode & 0o6000:
            raise ValueError(f"unsafe payload entry: {name}")
        elif stat.S_ISREG(mode) and name not in [PACKAGE, INDEX]:
            files[name] = {"type": "file", "mode": stat.S_IMODE(mode), "sha256": digest(path)}
    return {"schema_version": 1, "files": files}


def validate(root, identity, release=False):
    if type(identity["schema_version"]) is not int or identity["schema_version"] != 1:
        raise ValueError("unsupported package schema")
    revision = identity["revision"]
    if revision is None and release:
        raise ValueError("release requires an immutable source revision")
    if identity["channel"] != "stable" and release:
        raise ValueError("release requires the stable channel")
    if revision is not None and not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise ValueError("invalid source revision")
    if identity["target"] != "x86_64-unknown-linux-gnu" or identity["variant"] != "no-rio":
        raise ValueError("unsupported target or payload variant")
    if (identity["channel"] not in ["edge", "main", "stable"]
            or not isinstance(identity["version"], str) or not identity["version"]):
        raise ValueError("invalid runtime identity")
    if not isinstance(identity["entrypoints"], list) or "bin/yzx" not in identity["entrypoints"]:
        raise ValueError("missing public entrypoint")
    for name in identity["entrypoints"]:
        path = PurePosixPath(name)
        if path.is_absolute() or ".." in path.parts or str(path) != name:
            raise ValueError(f"invalid entrypoint: {name}")
        file = root / name
        if not file.is_file() or not file.stat().st_mode & 0o111:
            raise ValueError(f"entrypoint is not executable: {name}")
    runtime = json.loads((root / "share/yazelix/runtime_identity.json").read_text())
    if any(identity[key] != runtime[key] for key in ["version", "channel"]):
        raise ValueError("package and runtime identities disagree")


def write(root, identity, release=False):
    root = root.resolve(strict=True)
    directory = root / "share/yazelix"
    if directory.resolve(strict=True) != directory:
        raise ValueError("metadata directory must be a real payload directory")
    for name in [PACKAGE, INDEX]:
        path = root / name
        if path.exists() or path.is_symlink():
            raise FileExistsError(path)
    runtime = json.loads((directory / "runtime_identity.json").read_text())
    identity = identity | {key: runtime[key] for key in ["version", "channel"]}
    validate(root, identity, release)
    contents = encode(index(root))
    identity = identity | {"file_index": {"path": INDEX, "sha256": hashlib.sha256(contents).hexdigest()}}
    with (root / INDEX).open("xb") as stream:
        stream.write(contents)
    with (root / PACKAGE).open("xb") as stream:
        stream.write(encode(identity))


def verify(root, release=False):
    root = root.resolve(strict=True)
    current = index(root)
    identity = json.loads((root / PACKAGE).read_text())
    validate(root, identity, release)
    record = identity["file_index"]
    if record["path"] != INDEX or digest(root / INDEX) != record["sha256"]:
        raise ValueError("file index digest mismatch")
    saved = json.loads((root / INDEX).read_text())
    if encode(saved) != encode(current):
        raise ValueError("payload differs from its file index")
    return identity


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("identity", type=Path, nargs="?")
    parser.add_argument("--release", action="store_true", help="require an immutable source revision and the stable channel")
    args = parser.parse_args()
    if args.identity:
        write(args.root, json.loads(args.identity.read_text()), args.release)
    metadata = verify(args.root, args.release)
    print(f"Verified {metadata['target']} {metadata['variant']} {metadata['revision'] or 'uncommitted'}")
