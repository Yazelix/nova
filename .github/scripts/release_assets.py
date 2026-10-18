"""Prepare immutable-candidate assets for review; never publish or grant clearance."""

import argparse
import gzip
import hashlib
import json
import re
import shutil
import sys
import tarfile
import tempfile
from pathlib import Path, PurePosixPath

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "packaging"))
from archive_metadata import INDEX, PACKAGE, digest, encode, verify
from grammar_sources import verify_bundle

PART_BYTES = 1900 * 1024 * 1024  # GitHub requires each release asset to be smaller than 2 GiB.
INSTALLER = Path(__file__).resolve().parents[2] / "packaging/install.sh"


def checksums(directory):
    paths = sorted(p for p in directory.iterdir() if p.name != "SHA256SUMS")
    (directory / "SHA256SUMS").write_text("".join(f"{digest(p)}  {p.name}\n" for p in paths))


def verify_archive(path, installed, identity):
    expected = json.loads((installed / INDEX).read_text())["files"]
    for name in [PACKAGE, INDEX]:
        expected[name] = {"type": "file", "sha256": digest(installed / name),
                          "mode": (installed / name).stat().st_mode & 0o7777}
    seen = set()
    with gzip.open(path, "rb") as compressed, tarfile.open(fileobj=compressed, mode="r|") as archive:
        for member in archive:
            name = member.name.removeprefix("./")
            relative = PurePosixPath(name)
            if (relative.is_absolute() or ".." in relative.parts or str(relative) != name
                    or name in seen or member.mode & 0o6000):
                raise ValueError(f"unsafe or duplicate runtime member: {member.name}")
            seen.add(name)
            if member.isdir():
                if not (installed / name).is_dir() or (installed / name).is_symlink():
                    raise ValueError(f"unexpected runtime directory: {name}")
                continue
            if member.isfile():
                with archive.extractfile(member) as stream:
                    record = {"type": "file", "sha256": hashlib.file_digest(stream, "sha256").hexdigest(),
                              "mode": member.mode}
            elif member.issym():
                record = {"type": "symlink", "target": member.linkname}
            else:
                raise ValueError(f"unsafe runtime member: {name}")
            if record != expected.pop(name, None):
                raise ValueError(f"runtime archive differs from verified installation: {name}")
        while block := archive.fileobj.read(1024 * 1024):
            if any(block):
                raise ValueError("unexpected data after runtime archive")
    if expected or identity != json.loads((installed / PACKAGE).read_text()):
        raise ValueError("incomplete runtime archive or changed installation")


def prepare(installed, archive, sources, notices, tag, output):
    if output.exists() or output.is_symlink():
        raise FileExistsError(output)
    identity = verify(installed, True)
    version = identity["version"]
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?", version) or tag != "nova-v" + version:
        raise ValueError("tag must match the packaged Nova version")
    base = f"yazelix-nova-{version}-{identity['target']}-{identity['variant']}"
    url = f"https://github.com/Yazelix/nova/releases/download/{tag}/"
    with tempfile.TemporaryDirectory(dir=output.parent) as temporary:
        directory = Path(temporary)
        runtime_name, notice_name = base + ".tar.gz", base + "-language-runtime-notices.tar.gz"
        for source, name in [(archive, runtime_name), (sources, "component-sources.tar.gz"), (notices, notice_name),
                             (INSTALLER, "install.sh")]:
            shutil.copyfile(source, directory / name)
        verify_archive(directory / runtime_name, installed, identity)
        source_manifest = verify_bundle(directory / "component-sources.tar.gz")
        notice_manifest = verify_bundle(directory / notice_name)
        if (any(type(m["schema_version"]) is not int or m["schema_version"] != 1
                for m in [source_manifest, notice_manifest]) or source_manifest["package"] != identity
                or not source_manifest["components"] or not source_manifest["build_graph"]
                or notice_manifest["archive_derivation"] != source_manifest["archive_derivation"]
                or not notice_manifest["providers"]
                or any(p["derivation"] not in source_manifest["build_graph"] for p in notice_manifest["providers"])
                or any(m["redistribution_review"] != "required" for m in [source_manifest, notice_manifest])):
            raise ValueError("source/notice packet does not describe this candidate or review state")

        def asset(name):
            path = directory / name
            size = path.stat().st_size
            if size >= 2 * 1024 ** 3:
                raise ValueError(f"release asset exceeds GitHub's limit: {name}")
            return {"name": name, "size": size, "sha256": digest(path), "url": url + name}

        parts, whole = [], hashlib.sha256()
        with (directory / "component-sources.tar.gz").open("rb") as stream:
            while stream.peek(1):
                name = base + f"-component-sources.tar.gz.part-{len(parts):02d}"
                remaining = PART_BYTES
                with (directory / name).open("xb") as part:
                    while remaining and (block := stream.read(min(remaining, 1024 * 1024))):
                        whole.update(block)
                        part.write(block)
                        remaining -= len(block)
                parts.append(asset(name))
        source_record = {"name": base + "-component-sources.tar.gz",
                         "size": (directory / "component-sources.tar.gz").stat().st_size,
                         "sha256": whole.hexdigest(), "parts": parts}
        (directory / "component-sources.tar.gz").unlink()
        manifest = {"schema_version": 1, "version": version, "revision": identity["revision"],
                    "tag": tag, "redistribution_review": "required",
                    "targets": {identity["target"]: {"archive": asset(runtime_name),
                        **{k: identity[k] for k in ["variant", "compatibility", "entrypoints", "file_index"]}}},
                    "sources": source_record, "language_runtime_notices": asset(notice_name),
                    "installer": asset("install.sh")}
        (directory / "nova-release.json").write_bytes(encode(manifest))
        (directory / "SOURCE-DELIVERY.md").write_text(
            f"# Nova {version} source and notice review assets\n\n"
            f"Revision: `{identity['revision']}`. Tag: `{tag}`. Publication and redistribution review remain pending.\n\n"
            "The runtime archive is unchanged. The associated component source kit preserves full selected "
            "source/vendor trees, original notices, patches and Nix/Nova recipes. Its SOURCE-MANIFEST.json "
            "maps component derivations to those materials and hashes their files. Original notices inside "
            "source archives remain inside those archives; the language-runtime packet supplements them. "
            "License declarations retain their original scopes; this kit grants no replacement license.\n\n"
            "Verify SHA256SUMS before extraction. Concatenate the following parts in this order; verify "
            "the reconstructed archive's size and SHA-256 against nova-release.json:\n\n"
            + "\n".join(f"- `{p['name']}`" for p in parts) + "\n\n"
            "## Build and replacement instructions\n\n"
            "Build the recorded revision on x86_64 Linux with Nix (required for building, not running):\n\n"
            f"```nu\nnix build 'git+https://github.com/Yazelix/nova?rev={identity['revision']}#yazelix-no-rio-archive' --out-link rebuilt-archive\n```\n\n"
            "The pinned flake.lock selects build inputs; Nix may fetch those inputs and toolchains. "
            "The extracted SOURCE-MANIFEST.json records the Nova recipe root in "
            "build_graph[archive_derivation].env.novaSourceRoot; that root is under sources/. "
            "Component materials, original patches and complete vendor trees are mapped by "
            "components[].materials. The recorded graph retains build flags and recipes. "
            "To modify the build, edit a private copy of the selected source and its Nix recipe.\n\n"
            "For a dynamic-library replacement test, use a private runtime copy. Match the library's "
            "lib/native provider with share/yazelix/archive-origins.txt and the source manifest, "
            "build an ABI-compatible replacement and replace the library at the same relative path, "
            "retaining its SONAME and loader links. Exercise it through that copy's bin/yzx. "
            "Static or embedded changes require rebuilding their consuming program. These steps "
            "do not establish that every applicable replacement/relink obligation is satisfied.\n\n"
            "Corresponding-source/build instructions, prominent native-library notices and applicable "
            "replacement/relink obligations require review before publication. Local checksums are not "
            "GitHub OIDC provenance or an immutable-release guarantee.\n")
        checksums(directory)
        output.mkdir()
        for path in directory.iterdir():
            (output / path.name).hardlink_to(path)
    return manifest


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["installed", "archive", "sources", "notices"]:
        parser.add_argument(name, type=Path)
    parser.add_argument("tag")
    parser.add_argument("output", type=Path, help="new directory; existing destinations are refused")
    args = parser.parse_args()
    manifest = prepare(args.installed, args.archive, args.sources, args.notices, args.tag, args.output)
    print(f"Prepared {manifest['revision']}; redistribution review, hosted provenance and publication remain pending.")
